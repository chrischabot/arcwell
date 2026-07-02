use super::*;

impl Store {
    pub(crate) fn execute_radar_run(&self, input: &Value) -> Result<Value> {
        let profile = input
            .get("profile")
            .and_then(Value::as_str)
            .context("radar_run missing profile")?;
        let window_hours = match input.get("window_hours") {
            Some(Value::Null) | None => None,
            Some(value) => Some(
                value
                    .as_i64()
                    .context("radar_run window_hours must be an integer")?,
            ),
        };
        let fetch_live = input
            .get("fetch_live")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let report = self.run_radar_profile_with_options(profile, window_hours, fetch_live)?;
        Ok(json!({
            "run_id": report.run.id,
            "profile_id": report.profile.id,
            "profile_name": report.profile.name,
            "status": report.run.status,
            "stage": report.run.stage,
            "items_inserted": report.items_inserted,
            "scores_inserted": report.scores_inserted,
            "selected_items": report.selected_items,
            "adapter_job_count": report.adapter_jobs.len(),
            "adapter_jobs": report.adapter_jobs.iter().map(|job| json!({
                "id": job.id,
                "kind": job.kind,
                "status": job.status,
                "error": job.error,
                "result": job.result_json
            })).collect::<Vec<_>>(),
            "adapter_run_count": report.adapter_runs.len(),
            "adapter_runs": report.adapter_runs,
            "unsupported_selectors": report.unsupported_selectors,
            "warnings": report.warnings,
            "fetch_live": fetch_live
        }))
    }

    pub(crate) fn execute_radar_scheduled_delivery(&self, input: &Value) -> Result<Value> {
        let tick_id = input
            .get("tick_id")
            .and_then(Value::as_str)
            .context("radar_scheduled_delivery missing tick_id")?;
        let tick = self
            .get_radar_schedule_tick(tick_id)?
            .with_context(|| format!("radar schedule tick not found: {tick_id}"))?;
        let profile = self
            .read_radar_profile(&tick.profile_id)?
            .with_context(|| format!("radar schedule profile not found: {}", tick.profile_id))?;
        let policy = scheduled_radar_delivery_policy(&profile)?
            .context("radar profile is not configured for scheduled delivery")?;
        if let Some(deferred_until) = radar_quiet_hours_deferred_until(&policy, Utc::now())? {
            let note = format!("quiet hours active until {}", deferred_until.to_rfc3339());
            let updated = self.update_radar_schedule_tick(
                &tick.id,
                "deferred",
                tick.run_id.as_deref(),
                tick.summary_id.as_deref(),
                tick.delivery_id.as_deref(),
                Some(&note),
            )?;
            return Ok(json!({
                "tick": updated,
                "status": "deferred",
                "deferred_until": deferred_until.to_rfc3339(),
                "reason": "quiet_hours",
                "proof_level": "Local Proof: quiet-hours policy deferred scheduled radar delivery before provider send"
            }));
        }
        self.update_radar_schedule_tick(
            &tick.id,
            "running",
            tick.run_id.as_deref(),
            tick.summary_id.as_deref(),
            tick.delivery_id.as_deref(),
            None,
        )?;
        let run = if let Some(run_id) = tick.run_id.as_deref() {
            self.read_radar_run(run_id)?
                .with_context(|| format!("radar schedule run not found: {run_id}"))?
        } else {
            let report = self.run_radar_profile_with_options(
                &profile.id,
                Some(profile.window_hours),
                policy.fetch_live,
            )?;
            self.update_radar_schedule_tick(
                &tick.id,
                "running",
                Some(&report.run.id),
                None,
                None,
                None,
            )?;
            report.run
        };
        if run.status != "scored" {
            let error = format!("scheduled radar run did not score cleanly: {}", run.status);
            let updated = self.update_radar_schedule_tick(
                &tick.id,
                "blocked",
                Some(&run.id),
                tick.summary_id.as_deref(),
                tick.delivery_id.as_deref(),
                Some(&error),
            )?;
            return Ok(json!({ "tick": updated, "status": "blocked", "error": error }));
        }
        let audit = self.audit_radar_run(&run.id)?;
        if !audit.ok {
            let error = format!(
                "scheduled radar delivery requires audit_ok run; finding_count={}",
                audit.findings.len()
            );
            let updated = self.update_radar_schedule_tick(
                &tick.id,
                "blocked",
                Some(&run.id),
                tick.summary_id.as_deref(),
                tick.delivery_id.as_deref(),
                Some(&error),
            )?;
            return Ok(
                json!({ "tick": updated, "status": "blocked", "audit": audit, "error": error }),
            );
        }
        let summary = if let Some(existing) =
            self.read_radar_summary(&run.id, &policy.language, &policy.format)?
        {
            existing
        } else {
            self.summarize_radar_run(&run.id, &policy.language, &policy.format)?
        };
        self.update_radar_schedule_tick(
            &tick.id,
            "running",
            Some(&run.id),
            Some(&summary.id),
            tick.delivery_id.as_deref(),
            None,
        )?;
        let credentials = (|| -> Result<_> {
            match policy.channel.as_str() {
                "telegram" => Ok((
                    Some(self.configured_telegram_bot_token()?.context(
                        "TELEGRAM_BOT_TOKEN is required for scheduled radar Telegram delivery",
                    )?),
                    None,
                    None,
                    None,
                    self.configured_telegram_api_base()?,
                    "Local Proof: scheduled Telegram radar delivery through resident worker",
                )),
                "email" => Ok((
                    None,
                    Some(self.configured_cloudflare_account_id()?.context(
                        "CLOUDFLARE_ACCOUNT_ID is required for scheduled radar email delivery",
                    )?),
                    Some(self.configured_cloudflare_email_api_token()?.context(
                        "CLOUDFLARE_EMAIL_API_TOKEN or CLOUDFLARE_API_TOKEN is required for scheduled radar email delivery",
                    )?),
                    Some(self.configured_agent_email_from()?.context(
                        "ARCWELL_AGENT_EMAIL_FROM or ARCWELL_AGENT_EMAIL is required for scheduled radar email delivery",
                    )?),
                    self.configured_cloudflare_email_api_base()?,
                    "Local Proof: scheduled email radar delivery through resident worker",
                )),
                other => bail!("unsupported scheduled radar delivery channel: {other}"),
            }
        })();
        let (
            telegram_bot_token,
            email_account_id,
            email_api_token,
            email_from,
            api_base,
            proof_level,
        ) = match credentials {
            Ok(credentials) => credentials,
            Err(error) => {
                let error = error.to_string();
                let updated = self.update_radar_schedule_tick(
                    &tick.id,
                    "blocked",
                    Some(&run.id),
                    Some(&summary.id),
                    tick.delivery_id.as_deref(),
                    Some(&error),
                )?;
                return Ok(json!({
                    "tick": updated,
                    "run_id": run.id,
                    "summary_id": summary.id,
                    "status": "blocked",
                    "error": sanitize_radar_delivery_error(&error)?,
                    "proof_level": "Local Proof boundary: scheduled radar delivery is blocked before provider send when channel config is missing"
                }));
            }
        };
        let delivery = self.deliver_radar_summary(RadarDeliveryInput {
            run_id: run.id.clone(),
            language: policy.language.clone(),
            format: policy.format.clone(),
            channel: policy.channel.clone(),
            recipient_ref: policy.recipient_ref.clone(),
            idempotency_key: Some(format!("radar-schedule-{}", tick.tick_key)),
            telegram_bot_token,
            email_account_id,
            email_api_token,
            email_from,
            api_base,
        })?;
        let tick_status = match delivery.delivery.status.as_str() {
            "sent" => "sent",
            "blocked" => "blocked",
            "failed" => "failed",
            other => other,
        };
        let updated = self.update_radar_schedule_tick(
            &tick.id,
            tick_status,
            Some(&run.id),
            Some(&summary.id),
            Some(&delivery.delivery.id),
            delivery.delivery.error.as_deref(),
        )?;
        Ok(json!({
            "tick": updated,
            "run_id": run.id,
            "summary_id": summary.id,
            "delivery": delivery,
            "status": tick_status,
            "proof_level": proof_level
        }))
    }

    pub(crate) fn execute_digest_scheduled_alert(&self, input: &Value) -> Result<Value> {
        let tick_id = input
            .get("tick_id")
            .and_then(Value::as_str)
            .context("digest_scheduled_alert missing tick_id")?;
        let tick = self
            .get_digest_alert_tick(tick_id)?
            .with_context(|| format!("digest alert tick not found: {tick_id}"))?;
        let schedule = self
            .get_digest_alert_schedule(&tick.schedule_id)?
            .with_context(|| format!("digest alert schedule not found: {}", tick.schedule_id))?;
        let policy = digest_alert_schedule_policy(&schedule)?;
        if let Some(deferred_until) = digest_alert_quiet_hours_deferred_until(&policy, Utc::now())?
        {
            let note = format!("quiet hours active until {}", deferred_until.to_rfc3339());
            let updated = self.update_digest_alert_tick(
                &tick.id,
                "deferred",
                &tick.candidate_ids,
                &tick.delivery_ids,
                Some(&note),
            )?;
            return Ok(json!({
                "tick": updated,
                "schedule": schedule,
                "status": "deferred",
                "deferred_until": deferred_until.to_rfc3339(),
                "reason": "quiet_hours",
                "proof_level": "Production-shape proof: quiet-hours policy deferred scheduled digest alert before provider send"
            }));
        }
        let candidates = if digest_alert_schedule_is_credential_reminder(&schedule) {
            match self.materialize_credential_reminder_digest_candidate(&schedule, &tick, &policy) {
                Ok(Some(candidate)) => vec![candidate],
                Ok(None) => Vec::new(),
                Err(error) => {
                    let error = sanitize_radar_delivery_error(&error.to_string())?;
                    let updated = self.update_digest_alert_tick(
                        &tick.id,
                        "blocked",
                        &tick.candidate_ids,
                        &tick.delivery_ids,
                        Some(&error),
                    )?;
                    return Ok(json!({
                        "tick": updated,
                        "schedule": schedule,
                        "status": "blocked",
                        "selected_candidates": tick.candidate_ids,
                        "deliveries": tick.delivery_ids,
                        "error": error,
                        "proof_level": "Production-shape boundary: scheduled credential reminder is blocked before provider send when source/write/review policy is missing"
                    }));
                }
            }
        } else {
            self.select_digest_alert_candidates(&policy)?
        };
        let candidate_ids = candidates
            .iter()
            .map(|candidate| candidate.id.clone())
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            let updated = self.update_digest_alert_tick(&tick.id, "empty", &[], &[], None)?;
            return Ok(json!({
                "tick": updated,
                "schedule": schedule,
                "status": "empty",
                "selected_candidates": [],
                "deliveries": [],
                "proof_level": "Production-shape proof: scheduled digest alert ran threshold selection and found no approved unsent candidates"
            }));
        }
        let credentials = (|| -> Result<_> {
            match policy.channel.as_str() {
                "telegram" => Ok((
                    Some(self.configured_telegram_bot_token()?.context(
                        "TELEGRAM_BOT_TOKEN is required for scheduled digest Telegram alerts",
                    )?),
                    None,
                    None,
                    None,
                    self.configured_telegram_api_base()?,
                    "Production-shape proof: scheduled Telegram digest alert through resident worker",
                )),
                "email" => Ok((
                    None,
                    Some(self.configured_cloudflare_account_id()?.context(
                        "CLOUDFLARE_ACCOUNT_ID is required for scheduled digest email alerts",
                    )?),
                    Some(self.configured_cloudflare_email_api_token()?.context(
                        "CLOUDFLARE_EMAIL_API_TOKEN or CLOUDFLARE_API_TOKEN is required for scheduled digest email alerts",
                    )?),
                    Some(self.configured_agent_email_from()?.context(
                        "ARCWELL_AGENT_EMAIL_FROM or ARCWELL_AGENT_EMAIL is required for scheduled digest email alerts",
                    )?),
                    self.configured_cloudflare_email_api_base()?,
                    "Production-shape proof: scheduled email digest alert through resident worker",
                )),
                other => bail!("unsupported scheduled digest alert channel: {other}"),
            }
        })();
        let (
            telegram_bot_token,
            email_account_id,
            email_api_token,
            email_from,
            api_base,
            proof_level,
        ) = match credentials {
            Ok(credentials) => credentials,
            Err(error) => {
                let error = sanitize_radar_delivery_error(&error.to_string())?;
                let updated = self.update_digest_alert_tick(
                    &tick.id,
                    "blocked",
                    &candidate_ids,
                    &[],
                    Some(&error),
                )?;
                return Ok(json!({
                    "tick": updated,
                    "schedule": schedule,
                    "status": "blocked",
                    "selected_candidates": candidate_ids,
                    "deliveries": [],
                    "error": error,
                    "proof_level": "Production-shape boundary: scheduled digest alert is blocked before provider send when channel config is missing"
                }));
            }
        };
        let mut deliveries = Vec::new();
        let mut delivery_ids = Vec::new();
        let mut errors = Vec::new();
        for candidate in candidates {
            let idempotency_key = format!("digest-alert-{}-{}", tick.tick_key, candidate.id);
            let result = match policy.channel.as_str() {
                "telegram" => self
                    .send_digest_candidate_telegram(
                        &candidate.id,
                        telegram_bot_token.as_deref().unwrap_or_default(),
                        digest_alert_telegram_chat_id(&policy.recipient_ref)?,
                        Some(&idempotency_key),
                        api_base.as_deref(),
                    )
                    .map(|report| json!(report.digest_delivery)),
                "email" => self
                    .send_digest_candidate_email(
                        &candidate.id,
                        email_account_id.as_deref().unwrap_or_default(),
                        email_api_token.as_deref().unwrap_or_default(),
                        email_from.as_deref().unwrap_or_default(),
                        digest_alert_email_recipient(&policy.recipient_ref)?,
                        Some(&idempotency_key),
                        api_base.as_deref(),
                    )
                    .map(|report| json!(report.digest_delivery)),
                other => bail!("unsupported scheduled digest alert channel: {other}"),
            };
            match result {
                Ok(delivery) => {
                    if let Some(id) = delivery.get("id").and_then(Value::as_str) {
                        delivery_ids.push(id.to_string());
                    }
                    deliveries.push(delivery);
                }
                Err(error) => {
                    let error = sanitize_radar_delivery_error(&error.to_string())?;
                    if let Some(delivery) =
                        self.find_digest_alert_delivery(&candidate.id, &policy, &idempotency_key)?
                    {
                        delivery_ids.push(delivery.id.clone());
                        deliveries.push(json!(delivery));
                    }
                    errors.push(format!("{}: {error}", candidate.id));
                }
            }
        }
        let sent = deliveries
            .iter()
            .filter(|delivery| delivery.get("status").and_then(Value::as_str) == Some("sent"))
            .count();
        let status = if errors.is_empty() && sent == delivery_ids.len() {
            "sent"
        } else if sent > 0 {
            "partial"
        } else {
            "failed"
        };
        let error_text = if errors.is_empty() {
            None
        } else {
            Some(errors.join("; "))
        };
        let updated = self.update_digest_alert_tick(
            &tick.id,
            status,
            &candidate_ids,
            &delivery_ids,
            error_text.as_deref(),
        )?;
        Ok(json!({
            "tick": updated,
            "schedule": schedule,
            "status": status,
            "selected_candidates": candidate_ids,
            "deliveries": deliveries,
            "errors": errors,
            "proof_level": proof_level
        }))
    }

    pub(crate) fn execute_knowledge_daily_briefing(&self, input: &Value) -> Result<Value> {
        let tick_id = input
            .get("tick_id")
            .and_then(Value::as_str)
            .context("knowledge_daily_briefing missing tick_id")?;
        let tick = self
            .get_issue_schedule_tick(tick_id)?
            .with_context(|| format!("issue schedule tick not found: {tick_id}"))?;
        let schedule = self
            .get_issue_schedule(&tick.schedule_id)?
            .with_context(|| format!("issue schedule not found: {}", tick.schedule_id))?;
        let issue_label = issue_schedule_reader_label(&schedule);
        if schedule.kind != "knowledge_daily_briefing" {
            let error = format!(
                "knowledge issue job cannot execute issue kind {}",
                schedule.kind
            );
            let updated =
                self.update_issue_schedule_tick(&tick.id, "blocked", None, None, Some(&error))?;
            return Ok(
                json!({ "tick": updated, "schedule": schedule, "status": "blocked", "error": error }),
            );
        }
        self.update_issue_schedule_tick(&tick.id, "running", None, None, None)?;
        let max_reports = schedule
            .metadata
            .get("max_reports")
            .and_then(Value::as_u64)
            .unwrap_or(12) as usize;
        let report_scan_limit = schedule
            .metadata
            .get("report_scan_limit")
            .or_else(|| schedule.metadata.get("max_report_scan"))
            .and_then(Value::as_u64)
            .map(|value| value as usize)
            .unwrap_or_else(|| max_reports.saturating_mul(12).max(120));
        let report_scan_limit = report_scan_limit.clamp(max_reports.clamp(1, 500), 500);
        let max_source_cards = schedule
            .metadata
            .get("max_source_cards")
            .and_then(Value::as_u64)
            .unwrap_or(80) as usize;
        let source_card_scan_limit = schedule
            .metadata
            .get("source_card_scan_limit")
            .or_else(|| schedule.metadata.get("max_source_card_scan"))
            .and_then(Value::as_u64)
            .map(|value| value as usize)
            .unwrap_or_else(|| max_source_cards.max(report_scan_limit.saturating_mul(6)));
        let source_card_scan_limit =
            source_card_scan_limit.clamp(max_source_cards.clamp(1, 2_000), 2_000);
        let window_hours = schedule
            .metadata
            .get("window_hours")
            .and_then(Value::as_i64)
            .unwrap_or(24)
            .clamp(1, 24 * 14);
        let window_end = Utc::now();
        let window_start = window_end - ChronoDuration::hours(window_hours);
        let reports = self.list_knowledge_reports_updated_between(
            &window_start.to_rfc3339(),
            &window_end.to_rfc3339(),
            report_scan_limit,
        )?;
        if reports.is_empty() {
            let updated = self.update_issue_schedule_tick(&tick.id, "empty", None, None, None)?;
            return Ok(json!({
                "tick": updated,
                "schedule": schedule,
                "status": "empty",
                "window_start": window_start.to_rfc3339(),
                "window_end": window_end.to_rfc3339(),
                "proof_level": format!("Operational boundary: scheduled {issue_label} ran, but no source-backed knowledge reports were updated in the configured window")
            }));
        }
        let mut source_card_ids = BTreeSet::new();
        for report in &reports {
            for source_card_id in &report.source_card_ids {
                source_card_ids.insert(source_card_id.clone());
            }
        }
        let source_card_ids = source_card_ids
            .into_iter()
            .take(source_card_scan_limit)
            .collect::<Vec<_>>();
        if source_card_ids.is_empty() {
            let error = "knowledge daily briefing requires source-card-backed reports";
            let updated =
                self.update_issue_schedule_tick(&tick.id, "blocked", None, None, Some(error))?;
            return Ok(
                json!({ "tick": updated, "schedule": schedule, "status": "blocked", "error": error }),
            );
        }
        let source_cards = self.read_source_cards_by_ids(&source_card_ids)?;
        if source_cards.iter().all(is_generated_source_card) {
            let error = "knowledge daily briefing refuses generated-only evidence";
            let updated =
                self.update_issue_schedule_tick(&tick.id, "blocked", None, None, Some(error))?;
            return Ok(
                json!({ "tick": updated, "schedule": schedule, "status": "blocked", "error": error }),
            );
        }
        let related_wiki_pages = self.daily_briefing_related_wiki_pages(&reports, &source_cards)?;
        let body = render_knowledge_daily_briefing(
            &schedule,
            &tick,
            &reports,
            &source_cards,
            &window_start.to_rfc3339(),
            &window_end.to_rfc3339(),
            &related_wiki_pages,
        );
        let forbidden_reader_terms = daily_briefing_forbidden_reader_terms(&body);
        if !forbidden_reader_terms.is_empty() {
            let error = format!(
                "knowledge daily briefing renderer produced internal pipeline language: {}",
                forbidden_reader_terms.join(", ")
            );
            let updated =
                self.update_issue_schedule_tick(&tick.id, "blocked", None, None, Some(&error))?;
            return Ok(json!({
                "tick": updated,
                "schedule": schedule,
                "status": "blocked",
                "error": error,
                "forbidden_reader_terms": forbidden_reader_terms,
                "forbidden_reader_excerpt": excerpt(&body, 2_000),
                "window_start": window_start.to_rfc3339(),
                "window_end": window_end.to_rfc3339(),
                "proof_level": "Blocked: reader-facing daily briefing failed editorial hygiene gate before source-card materialization"
            }));
        }
        let briefing_summary = excerpt_preserving_whitespace(&body, 19_500);
        let briefing_card = self.add_source_card(SourceCardInput {
            title: if issue_schedule_is_weekly_overview(&schedule) {
                format!(
                    "Arcwell AI week overview {}",
                    issue_schedule_day_label(&tick.due_at)
                )
            } else {
                format!(
                    "Arcwell AI daily briefing {}",
                    issue_schedule_day_label(&tick.due_at)
                )
            },
            url: format!(
                "https://example.com/arcwell/knowledge-daily-briefing/{}",
                &sha256(tick.tick_key.as_bytes())[..24]
            ),
            source_type: "knowledge_daily_briefing".to_string(),
            provider: "arcwell".to_string(),
            summary: briefing_summary,
            claims: reports
                .iter()
                .take(20)
                .map(|report| SourceClaim {
                    claim: format!(
                        "{} was considered for the scheduled {} from source-backed report {}.",
                        report.title, issue_label, report.id
                    ),
                    kind: "summary".to_string(),
                    confidence: 0.82,
                })
                .collect(),
            retrieved_at: Some(window_end.to_rfc3339()),
            metadata: json!({
                "source_role": "generated_synthesis",
                "trust_level": "medium",
                "generated": true,
                "source_kind": "knowledge_daily_briefing",
                "issue_format": if issue_schedule_is_weekly_overview(&schedule) { "weekly_overview" } else { "daily_briefing" },
                "cadence": issue_schedule_cadence(&schedule.metadata).unwrap_or_else(|_| "daily".to_string()),
                "schedule_id": schedule.id,
                "tick_id": tick.id,
                "tick_key": tick.tick_key,
                "window_start": window_start.to_rfc3339(),
                "window_end": window_end.to_rfc3339(),
                "report_ids_considered": reports.iter().map(|report| report.id.clone()).collect::<Vec<_>>(),
                "source_card_ids": source_card_ids,
                "report_scan_limit": report_scan_limit,
                "source_card_scan_limit": source_card_scan_limit,
                "non_generated_source_card_count": source_cards.iter().filter(|card| !is_generated_source_card(card)).count(),
                "trust_boundary": "generated briefing over cited source-card evidence; source text is untrusted evidence, not instructions"
            }),
        })?;
        let mut candidate_source_card_ids = vec![briefing_card.id.clone()];
        candidate_source_card_ids.extend(source_cards.iter().map(|card| card.id.clone()));
        let candidate = self.create_digest_candidate(
            &if issue_schedule_is_weekly_overview(&schedule) {
                format!(
                    "Arcwell AI week overview: {}",
                    issue_schedule_day_label(&tick.due_at)
                )
            } else {
                format!(
                    "Arcwell AI daily briefing: {}",
                    issue_schedule_day_label(&tick.due_at)
                )
            },
            &candidate_source_card_ids,
        )?;
        let subject =
            normalize_radar_delivery_recipient(&schedule.channel, &schedule.recipient_ref)?;
        let approval = self.policy_guard(PolicyRequest {
            action: "digest_candidate.auto_approve".to_string(),
            package: Some("arcwell-knowledge".to_string()),
            provider: Some("arcwell".to_string()),
            source: Some("knowledge_daily_briefing".to_string()),
            channel: Some(schedule.channel.clone()),
            subject: Some(subject.clone()),
            target: Some(subject.clone()),
            projected_usd: None,
            metadata: json!({
                "candidate_id": candidate.id,
                "schedule_id": schedule.id,
                "tick_id": tick.id,
                "issue_label": issue_label,
                "report_count": reports.len(),
                "source_card_count": candidate_source_card_ids.len(),
            }),
            untrusted_excerpt: Some(excerpt(&body, 2_000)),
        });
        let approved = match approval {
            Ok(_) => self.approve_digest_candidate(
                &candidate.id,
                Some("arcwell-knowledge-daily-briefing"),
                Some("scheduled source-backed daily briefing"),
            )?,
            Err(error) => {
                let error = error.to_string();
                let updated = self.update_issue_schedule_tick(
                    &tick.id,
                    "blocked",
                    Some(&candidate.id),
                    None,
                    Some(&error),
                )?;
                return Ok(json!({
                    "tick": updated,
                    "schedule": schedule,
                    "candidate": candidate,
                    "status": "blocked",
                    "error": sanitize_radar_delivery_error(&error)?,
                    "proof_level": format!("Operational boundary: {issue_label} candidate was generated but auto-approval policy blocked delivery")
                }));
            }
        };
        let idempotency_key = Some(format!("issue-schedule-{}", tick.tick_key));
        let delivery_result = match schedule.channel.as_str() {
            "email" => {
                let account_id = self.configured_cloudflare_account_id()?.context(format!(
                    "CLOUDFLARE_ACCOUNT_ID is required for scheduled knowledge {issue_label} email"
                ))?;
                let api_token = self.configured_cloudflare_email_api_token()?.context(
                    format!("CLOUDFLARE_EMAIL_API_TOKEN or CLOUDFLARE_API_TOKEN is required for scheduled knowledge {issue_label} email"),
                )?;
                let from = self.configured_agent_email_from()?.context(
                    format!("ARCWELL_AGENT_EMAIL_FROM or ARCWELL_AGENT_EMAIL is required for scheduled knowledge {issue_label} email"),
                )?;
                let to = digest_alert_email_recipient(&schedule.recipient_ref)?.to_string();
                self.send_digest_candidate_email(
                    &approved.id,
                    &account_id,
                    &api_token,
                    &from,
                    &to,
                    idempotency_key.as_deref(),
                    self.configured_cloudflare_email_api_base()?.as_deref(),
                )
                .map(|report| {
                    let delivery_id = report.digest_delivery.id.clone();
                    let status = report.digest_delivery.status.clone();
                    (delivery_id, status, json!(report))
                })
            }
            "telegram" => {
                let bot_token = self.configured_telegram_bot_token()?.context(format!(
                    "TELEGRAM_BOT_TOKEN is required for scheduled knowledge {issue_label} Telegram"
                ))?;
                let chat_id = digest_alert_telegram_chat_id(&schedule.recipient_ref)?.to_string();
                self.send_digest_candidate_telegram(
                    &approved.id,
                    &bot_token,
                    &chat_id,
                    idempotency_key.as_deref(),
                    self.configured_telegram_api_base()?.as_deref(),
                )
                .map(|report| {
                    let delivery_id = report.digest_delivery.id.clone();
                    let status = report.digest_delivery.status.clone();
                    (delivery_id, status, json!(report))
                })
            }
            other => bail!("unsupported scheduled knowledge issue channel: {other}"),
        };
        match delivery_result {
            Ok((delivery_id, status, delivery)) => {
                let tick_status = match status.as_str() {
                    "sent" => "sent",
                    "blocked" => "blocked",
                    "failed" => "failed",
                    _ => "partial",
                };
                let updated = self.update_issue_schedule_tick(
                    &tick.id,
                    tick_status,
                    Some(&approved.id),
                    Some(&delivery_id),
                    None,
                )?;
                Ok(json!({
                    "tick": updated,
                    "schedule": schedule,
                    "candidate": approved,
                    "daily_briefing_source_card_id": briefing_card.id,
                    "reports": reports,
                    "delivery": delivery,
                    "status": tick_status,
                    "proof_level": format!("Operational proof: native issue schedule created a source-backed {issue_label} candidate and attempted authorized delivery")
                }))
            }
            Err(error) => {
                let error = sanitize_radar_delivery_error(&error.to_string())?;
                let updated = self.update_issue_schedule_tick(
                    &tick.id,
                    "blocked",
                    Some(&approved.id),
                    None,
                    Some(&error),
                )?;
                Ok(json!({
                    "tick": updated,
                    "schedule": schedule,
                    "candidate": approved,
                    "daily_briefing_source_card_id": briefing_card.id,
                    "reports": reports,
                    "status": "blocked",
                    "error": error,
                    "proof_level": format!("Operational boundary: native {issue_label} generated and approved, but delivery was blocked before/at provider send")
                }))
            }
        }
    }

    pub(crate) fn materialize_credential_reminder_digest_candidate(
        &self,
        schedule: &DigestAlertSchedule,
        tick: &DigestAlertTick,
        policy: &DigestAlertSchedulePolicy,
    ) -> Result<Option<DigestCandidate>> {
        let warnings = self.credential_reminder_secret_warnings()?;
        if warnings.is_empty() {
            return Ok(None);
        }
        let warning_count = warnings
            .iter()
            .map(|(_, warnings)| warnings.len().max(1))
            .sum::<usize>();
        let names = warnings
            .iter()
            .map(|(health, _)| health.name.clone())
            .collect::<Vec<_>>();
        let summary = format!(
            "Arcwell detected {warning_count} credential health warning(s) across {} secret(s): {}. Scheduled provider ingestion, refresh, or delivery may fail until these are corrected.",
            names.len(),
            names.join(", ")
        );
        let mut claims = Vec::new();
        for (health, health_warnings) in &warnings {
            for warning in health_warnings.iter().take(4) {
                claims.push(SourceClaim {
                    claim: format!(
                        "{} is {} for scope {}{}: {}",
                        health.name,
                        health.status,
                        health.scope,
                        health
                            .provider
                            .as_ref()
                            .map(|provider| format!(" via provider {provider}"))
                            .unwrap_or_default(),
                        redact_secret_like_text(warning)
                    ),
                    kind: "warning".to_string(),
                    confidence: 1.0,
                });
                if claims.len() >= 40 {
                    break;
                }
            }
            if claims.len() >= 40 {
                break;
            }
        }
        if claims.is_empty() {
            claims.push(SourceClaim {
                claim: summary.clone(),
                kind: "warning".to_string(),
                confidence: 1.0,
            });
        }
        let card = self.add_source_card(SourceCardInput {
            title: format!("Arcwell credential health snapshot {}", tick.tick_key),
            url: format!(
                "https://example.com/arcwell/credential-reminders/{}",
                tick.tick_key
            ),
            source_type: "credential_health".to_string(),
            provider: "arcwell".to_string(),
            summary: summary.clone(),
            claims,
            retrieved_at: Some(now()),
            metadata: json!({
                "source_kind": "credential_reminder",
                "source_detail": "secret_health",
                "schedule_id": schedule.id,
                "tick_id": tick.id,
                "tick_key": tick.tick_key,
                "warning_count": warning_count,
                "secret_names": names,
                "trust_boundary": "local_secret_health_metadata_only",
                "raw_secret_values_included": false,
                "generated_by": "arcwell-secret-health"
            }),
        })?;
        let candidate = self.create_digest_candidate(
            "Arcwell credential health reminder",
            std::slice::from_ref(&card.id),
        )?;
        let subject = digest_alert_delivery_subject(policy)?;
        self.policy_guard(PolicyRequest {
            action: "credential_reminder.auto_approve".to_string(),
            package: Some("arcwell-ops".to_string()),
            provider: Some("arcwell".to_string()),
            source: Some("secret_health".to_string()),
            channel: Some(policy.channel.clone()),
            subject: Some(subject.clone()),
            target: Some(subject),
            projected_usd: None,
            metadata: json!({
                "candidate_id": candidate.id.clone(),
                "source_card_id": card.id.clone(),
                "schedule_id": schedule.id,
                "tick_id": tick.id,
                "warning_count": warning_count,
            }),
            untrusted_excerpt: Some(summary),
        })?;
        let candidate = self.approve_digest_candidate(
            &candidate.id,
            Some("arcwell-credential-reminder"),
            Some("scheduled credential health reminder"),
        )?;
        Ok(Some(candidate))
    }

    pub(crate) fn credential_reminder_secret_warnings(
        &self,
    ) -> Result<Vec<(SecretHealth, Vec<String>)>> {
        let mut warnings = Vec::new();
        for health in self.secret_health()? {
            let mut health_warnings = health
                .warnings
                .iter()
                .map(|warning| redact_secret_like_text(warning))
                .filter(|warning| !warning.trim().is_empty())
                .collect::<Vec<_>>();
            if health_warnings.is_empty()
                && matches!(
                    health.status.as_str(),
                    "missing" | "expired" | "expiring_soon" | "unresolved"
                )
            {
                health_warnings.push(format!(
                    "secret {} status is {} for scope {}",
                    health.name, health.status, health.scope
                ));
            }
            if !health_warnings.is_empty() {
                warnings.push((health, health_warnings));
            }
        }
        warnings.sort_by(|(left, _), (right, _)| {
            left.name
                .cmp(&right.name)
                .then_with(|| left.scope.cmp(&right.scope))
        });
        Ok(warnings)
    }

    pub(crate) fn select_digest_alert_candidates(
        &self,
        policy: &DigestAlertSchedulePolicy,
    ) -> Result<Vec<DigestCandidate>> {
        let subject = digest_alert_delivery_subject(policy)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, topic, score, reason, status, source_card_ids_json,
                   review_status, reviewed_at, reviewed_by, review_note,
                   created_at, updated_at
            FROM digest_candidates candidate
            WHERE candidate.status = 'approved'
              AND candidate.review_status = 'approved'
              AND candidate.score >= ?1
              AND NOT EXISTS (
                SELECT 1
                FROM digest_deliveries delivery
                WHERE delivery.candidate_id = candidate.id
                  AND delivery.channel = ?2
                  AND delivery.subject = ?3
                  AND delivery.target = ?3
                  AND delivery.status IN ('pending', 'sent')
              )
            ORDER BY candidate.score DESC, candidate.updated_at DESC
            LIMIT ?4
            "#,
        )?;
        rows(stmt.query_map(
            params![
                policy.min_score,
                policy.channel,
                subject,
                policy.max_candidates
            ],
            digest_candidate_from_row,
        )?)
    }

    pub(crate) fn find_digest_alert_delivery(
        &self,
        candidate_id: &str,
        policy: &DigestAlertSchedulePolicy,
        explicit_idempotency_key: &str,
    ) -> Result<Option<DigestDelivery>> {
        let subject = digest_alert_delivery_subject(policy)?;
        let idempotency_key = digest_delivery_idempotency_key(
            candidate_id,
            &policy.channel,
            &subject,
            &subject,
            Some(explicit_idempotency_key),
        )?;
        self.find_digest_delivery(
            candidate_id,
            &policy.channel,
            &subject,
            &subject,
            &idempotency_key,
        )
    }
}
