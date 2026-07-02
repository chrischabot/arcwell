use super::*;

impl Store {
    pub fn web_search(&self, query: &str, config: WebSearchConfig) -> Result<WebSearchResponse> {
        self.web_search_with_cost_decision(query, config)
            .map(|(response, _decision)| response)
    }

    pub(crate) fn web_search_with_cost_decision(
        &self,
        query: &str,
        config: WebSearchConfig,
    ) -> Result<(WebSearchResponse, Option<CostDecision>)> {
        validate_query(query)?;
        let provider = config.provider.trim().to_ascii_lowercase();
        let max_results = config.max_results.clamp(1, 20);
        let timeout = Duration::from_secs(config.timeout_seconds.clamp(1, 30));
        let mut cost_decision = None;
        if !matches!(provider.as_str(), "host" | "host-native" | "native") {
            self.policy_guard(PolicyRequest {
                action: "provider.network".to_string(),
                package: Some("arcwell-deep-research".to_string()),
                provider: Some(provider.clone()),
                source: Some("web_search".to_string()),
                channel: None,
                subject: None,
                target: config.endpoint.clone(),
                projected_usd: Some(estimated_web_search_cost(max_results)),
                metadata: json!({ "query": query, "max_results": max_results }),
                untrusted_excerpt: None,
            })?;
            cost_decision = Some(self.require_cost_budget(
                "arcwell-deep-research",
                "web_search",
                &provider,
                "web_search",
                Some("web_search"),
                estimated_web_search_cost(max_results),
                "web search",
            )?);
        }
        let response = match provider.as_str() {
            "brave" => brave_search(query, &config, max_results, timeout),
            "openai" => openai_web_search(query, &config, max_results, timeout),
            "perplexity" => perplexity_search(query, &config, max_results, timeout),
            "host" | "host-native" | "native" => bail!(
                "host-native search must be run by the calling agent; choose brave, openai, or perplexity for daemon-side search"
            ),
            other => bail!("unsupported web search provider: {other}"),
        }?;
        Ok((response, cost_decision))
    }

    pub fn web_search_to_wiki(
        &self,
        query: &str,
        config: WebSearchConfig,
    ) -> Result<(WebSearchResponse, String)> {
        let response = self.web_search(query, config)?;
        let markdown = render_search_source_card(&response);
        let page_id = self.add_wiki_page(
            &format!("Source Card: {}", response.query),
            &markdown,
            &format!("web-search:{}:{}", response.provider, response.query),
        )?;
        Ok((response, page_id))
    }

    pub fn list_research_runs(&self) -> Result<Vec<ResearchRun>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, query, status, result_page_id, created_at, updated_at
            FROM research_runs
            ORDER BY updated_at DESC
            "#,
        )?;
        rows(stmt.query_map([], research_run_from_row)?)
    }

    pub(crate) fn insert_wiki_job(&self, kind: &str, input_json: Value) -> Result<WikiJob> {
        validate_job_kind(kind)?;
        self.insert_wiki_job_with_status(kind, "running", input_json)
    }

    pub(crate) fn mark_expired_edge_events(&self, timestamp: &str) -> Result<()> {
        self.conn.execute(
            r#"
            UPDATE edge_events
            SET status = 'expired',
                leased_until = NULL,
                next_run_at = NULL,
                error = 'event expired before local drain',
                updated_at = ?1
            WHERE status IN ('pending', 'failed', 'leased')
              AND expires_at <= ?1
            "#,
            params![timestamp],
        )?;
        Ok(())
    }

    pub(crate) fn insert_wiki_job_with_status(
        &self,
        kind: &str,
        status: &str,
        input_json: Value,
    ) -> Result<WikiJob> {
        validate_key(kind)?;
        let id = Uuid::new_v4().to_string();
        let now = now();
        self.conn.execute(
            r#"
            INSERT INTO wiki_jobs (id, kind, status, input_json, result_json, error, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, NULL, NULL, ?5, ?5)
            "#,
            params![id, kind, status, serde_json::to_string(&input_json)?, now],
        )?;
        self.get_wiki_job(&id)?
            .with_context(|| format!("inserted wiki job not found: {id}"))
    }

    pub(crate) fn claim_next_pending_job(&self) -> Result<Option<WikiJob>> {
        let job: Option<WikiJob> = self
            .conn
            .query_row(
                r#"
                SELECT id, kind, status, input_json, result_json, error,
                       attempts, max_attempts, leased_until, worker_id, next_run_at, dead_lettered_at,
                       created_at, updated_at
                FROM wiki_jobs
                WHERE (
                    status = 'pending'
                    OR (status = 'deferred' AND next_run_at IS NOT NULL AND next_run_at <= ?1)
                    OR (status = 'failed' AND (next_run_at IS NULL OR next_run_at <= ?1))
                    OR (status = 'running' AND leased_until IS NOT NULL AND leased_until <= ?1)
                )
                AND attempts < max_attempts
                ORDER BY
                    CASE kind
                        WHEN 'knowledge_daily_briefing' THEN 0
                        WHEN 'digest_scheduled_alert' THEN 1
                        WHEN 'radar_scheduled_delivery' THEN 1
                        WHEN 'email_delivery_verification_request' THEN 2
                        WHEN 'email_delivery_mailbox_repair' THEN 2
                        ELSE 10
                    END ASC,
                    created_at ASC
                LIMIT 1
                "#,
                params![now()],
                wiki_job_from_row,
            )
            .optional()?;
        let Some(job) = job else {
            return Ok(None);
        };
        let claimed = self.conn.execute(
            r#"
            UPDATE wiki_jobs
            SET status = 'running',
                attempts = attempts + 1,
                leased_until = ?2,
                worker_id = ?3,
                next_run_at = NULL,
                updated_at = ?4
            WHERE id = ?1
              AND (
                status = 'pending'
                OR (status = 'deferred' AND next_run_at IS NOT NULL AND next_run_at <= ?4)
                OR (status = 'failed' AND (next_run_at IS NULL OR next_run_at <= ?4))
                OR (status = 'running' AND leased_until IS NOT NULL AND leased_until <= ?4)
              )
              AND attempts < max_attempts
            "#,
            params![job.id, now_plus_seconds(300), default_worker_id(), now()],
        )?;
        if claimed == 0 {
            return Ok(None);
        }
        self.get_wiki_job(&job.id)
    }

    pub(crate) fn execute_wiki_job(&self, job: WikiJob) -> Result<WikiJob> {
        let result = self
            .guard_wiki_job_provider_policy(&job)
            .and_then(|_| self.guard_wiki_job_cost(&job))
            .and_then(|_| match job.kind.as_str() {
                "ingest_file" => self.execute_ingest_file(&job.input_json),
                "ingest_url" => self.execute_ingest_url(&job.input_json),
                "ingest_rendered_page" => self.execute_ingest_rendered_page(&job.input_json),
                "compile" => self.execute_compile(&job.input_json),
                "expand_page" => self.execute_expand_page(&job.input_json),
                "rss_fetch" => self.execute_rss_fetch(&job.input_json),
                "github_repo" => self.execute_github_repo(&job.input_json),
                "github_owner" => self.execute_github_owner(&job.input_json),
                "arxiv_search" => self.execute_arxiv_search(&job.input_json),
                "hackernews_fetch" => self.execute_hackernews_fetch(&job.input_json),
                "reddit_fetch" => self.execute_reddit_fetch(&job.input_json),
                "x_recent_search" => self.execute_x_recent_search(&job.input_json, Some(&job.id)),
                "x_import_bookmarks" => self.execute_x_import_bookmarks(&job.input_json),
                "x_profile_enrichment" => self.execute_x_profile_enrichment(&job.input_json),
                "x_monitor_watch_source" => self.execute_x_monitor_watch_source(&job.input_json),
                "radar_run" => self.execute_radar_run(&job.input_json),
                "radar_scheduled_delivery" => {
                    self.execute_radar_scheduled_delivery(&job.input_json)
                }
                "digest_scheduled_alert" => self.execute_digest_scheduled_alert(&job.input_json),
                "knowledge_daily_briefing" => {
                    self.execute_knowledge_daily_briefing(&job.input_json)
                }
                "email_delivery_verification_request" => {
                    self.execute_email_delivery_verification_request(&job.input_json)
                }
                "email_delivery_mailbox_repair" => {
                    self.execute_email_delivery_mailbox_repair(&job.input_json)
                }
                "knowledge_cluster_editorial_decide" => {
                    self.execute_knowledge_cluster_editorial_decide(&job.input_json)
                }
                "knowledge_cluster_expand" => {
                    self.execute_knowledge_cluster_expand(&job.input_json)
                }
                "knowledge_cluster_model_write" => {
                    self.execute_knowledge_cluster_model_write(&job.input_json)
                }
                "knowledge_entity_resolution_model" => {
                    self.execute_knowledge_entity_resolution_model(&job.input_json)
                }
                "knowledge_cluster_backlog" => {
                    self.execute_knowledge_cluster_backlog(&job.input_json)
                }
                "knowledge_cluster_model_propose" => {
                    self.execute_knowledge_cluster_model_propose(&job.input_json)
                }
                "knowledge_cluster_investigate" => {
                    self.execute_knowledge_cluster_investigate(&job.input_json)
                }
                "knowledge_cluster_investigation_execute" => {
                    self.execute_knowledge_cluster_investigation_execute(&job.input_json)
                }
                "research_convergence_run" => {
                    self.execute_research_convergence_run(&job.input_json)
                }
                "job_radar_refresh" => self.execute_job_radar_refresh(&job.input_json),
                other => bail!("unsupported wiki job kind: {other}"),
            });
        match result {
            Ok(result) => {
                if let Some(deferred_until) = deferred_job_until(&result)? {
                    self.defer_wiki_job(&job.id, result, &deferred_until)
                } else {
                    self.complete_wiki_job(&job.id, result)
                }
            }
            Err(error) => {
                let error = format!("{error:#}");
                if job.kind == "knowledge_cluster_model_propose"
                    && crate::knowledge::knowledge_cluster_model_proposal_error_is_non_retryable(
                        &error,
                    )
                {
                    return self.complete_wiki_job(
                        &job.id,
                        json!({
                            "status": "rejected_non_retryable",
                            "reason": excerpt(&error, 1000),
                            "boundary": "Knowledge cluster model output failed deterministic safety validation and was not retried."
                        }),
                    );
                }
                if job.kind == "knowledge_cluster_model_propose"
                    && knowledge_cluster_model_proposal_error_is_deferable_provider_failure(&error)
                {
                    let (source_key, deferred_until) = self
                        .record_knowledge_cluster_model_proposal_job_failure_health(
                            &job.input_json,
                            &error,
                        )?;
                    return self.defer_wiki_job(
                        &job.id,
                        json!({
                            "status": "deferred",
                            "deferred_until": deferred_until,
                            "source_health_key": source_key,
                            "reason": excerpt(&error, 1000),
                            "boundary": "Scheduled model clustering provider/request failure was recorded in source health and deferred instead of dead-lettering the job."
                        }),
                        &deferred_until,
                    );
                }
                if job.kind == "x_import_bookmarks" {
                    let classification = classify_provider_failure(&error);
                    if matches!(classification.status, "auth_failed" | "rate_limited") {
                        let deferred_until = now_plus_seconds(classification.backoff_seconds);
                        return self.defer_wiki_job(
                            &job.id,
                            json!({
                                "status": "deferred",
                                "deferred_until": deferred_until,
                                "source_health_key": "x:bookmarks",
                                "provider_health_status": classification.status,
                                "reason": excerpt(&error, 1000),
                                "boundary": "X bookmark import provider auth/rate failure was recorded in source health and deferred instead of burning worker retry attempts."
                            }),
                            &deferred_until,
                        );
                    }
                }
                let failed = self.fail_wiki_job(&job.id, &error)?;
                if job.kind == "knowledge_entity_resolution_model" {
                    let _ = self.record_knowledge_entity_resolution_job_failure_health(
                        &job.input_json,
                        &error,
                    );
                }
                if job.kind == "job_radar_refresh" {
                    let _ =
                        self.record_job_radar_refresh_job_failure_health(&job.input_json, &error);
                }
                if job.kind == "ingest_url" {
                    let _ = self.record_ingest_url_job_failure_health(&job.input_json, &error);
                }
                Ok(failed)
            }
        }
    }

    pub(crate) fn record_knowledge_entity_resolution_job_failure_health(
        &self,
        input: &Value,
        error: &str,
    ) -> Result<()> {
        let source_key = input
            .get("lineage")
            .and_then(|lineage| lineage.get("watch_source_key"))
            .and_then(Value::as_str)
            .unwrap_or("knowledge:entity-resolution:entities");
        let locator = input
            .get("lineage")
            .and_then(|lineage| lineage.get("locator"))
            .and_then(Value::as_str)
            .unwrap_or("entities");
        self.record_source_failure(
            source_key,
            "knowledge_entity_resolution",
            "knowledge_entity_resolution",
            locator,
            error,
        )
    }

    pub(crate) fn record_job_radar_refresh_job_failure_health(
        &self,
        input: &Value,
        error: &str,
    ) -> Result<()> {
        let profile_id = input
            .get("profile_id")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let source_key = input
            .get("lineage")
            .and_then(|lineage| lineage.get("watch_source_key"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("job:radar:{profile_id}"));
        let source_kind = input
            .get("lineage")
            .and_then(|lineage| lineage.get("source_kind"))
            .and_then(Value::as_str)
            .unwrap_or("job_radar");
        let locator = input
            .get("lineage")
            .and_then(|lineage| lineage.get("locator"))
            .and_then(Value::as_str)
            .unwrap_or(profile_id);
        self.record_source_failure(&source_key, "arcwell", source_kind, locator, error)
    }

    pub(crate) fn record_knowledge_cluster_model_proposal_job_failure_health(
        &self,
        input: &Value,
        error: &str,
    ) -> Result<(String, String)> {
        let query = input
            .get("query")
            .and_then(Value::as_str)
            .unwrap_or("source-cards");
        let source_key = input
            .get("lineage")
            .and_then(|lineage| lineage.get("watch_source_key"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("knowledge:model-clusters:{query}"));
        let source_kind = input
            .get("lineage")
            .and_then(|lineage| lineage.get("source_kind"))
            .and_then(Value::as_str)
            .unwrap_or("knowledge_model_clusters");
        let locator = input
            .get("lineage")
            .and_then(|lineage| lineage.get("locator"))
            .and_then(Value::as_str)
            .unwrap_or(query);
        self.record_source_failure(&source_key, "arcwell", source_kind, locator, error)?;
        let deferred_until = self
            .get_source_health(&source_key)?
            .and_then(|health| health.next_run_at)
            .unwrap_or_else(|| {
                now_plus_seconds(self.watch_source_next_run_seconds(source_kind, locator, 60 * 60))
            });
        Ok((source_key, deferred_until))
    }

    pub(crate) fn record_ingest_url_job_failure_health(
        &self,
        input: &Value,
        error: &str,
    ) -> Result<()> {
        let Some(url) = input.get("url").and_then(Value::as_str) else {
            return Ok(());
        };
        self.record_blog_watch_source_failure_for_url_ingest(url, error)
    }

    pub(crate) fn execute_job_radar_refresh(&self, input: &Value) -> Result<Value> {
        let profile_id = input
            .get("profile_id")
            .and_then(Value::as_str)
            .context("job_radar_refresh missing profile_id")?;
        let profile = self.require_job_profile(profile_id)?;
        let scope = input
            .get("scope")
            .and_then(Value::as_str)
            .unwrap_or("scheduled job radar refresh");
        let scope = sanitize_required_job_text(scope, "job radar scope", JOB_MAX_TEXT)?;
        let source_ids = job_source_ids_from_value(input.get("source_ids"))?;
        let fetch_live = input
            .get("fetch_live")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let source_snapshots = input.get("source_snapshots").unwrap_or(&Value::Null);
        let proof_level =
            job_radar_refresh_derived_proof_level(fetch_live, &source_ids, source_snapshots)?;

        let mut observed_role_ids = BTreeSet::new();
        let mut stale_role_ids = BTreeSet::new();
        let mut source_health_ids = Vec::new();
        let mut refreshes = Vec::new();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        for source_id in &source_ids {
            let source = self
                .read_job_source(source_id)?
                .with_context(|| format!("job source not found: {source_id}"))?;
            let snapshot = job_source_snapshot_for(source_snapshots, source_id)?;
            let refresh = match snapshot {
                Some(snapshot) => self.run_job_source_refresh(JobSourceRefreshInput {
                    source_id: source_id.clone(),
                    body: Some(snapshot.body),
                    fetched_url: snapshot.fetched_url,
                    fetch_live: false,
                })?,
                None if fetch_live => self.run_job_source_refresh(JobSourceRefreshInput {
                    source_id: source_id.clone(),
                    body: None,
                    fetched_url: None,
                    fetch_live: true,
                })?,
                None => {
                    let health = self.record_job_source_health(JobSourceHealthInput {
                        source_id: source_id.clone(),
                        status: "failed".to_string(),
                        http_status: None,
                        error_code: Some("missing_snapshot".to_string()),
                        fetched_count: 0,
                        accepted_count: 0,
                        rejected_count: 0,
                        note: Some(
                            "Scheduled job radar refresh had no replay snapshot and fetch_live=false."
                                .to_string(),
                        ),
                    })?;
                    source_health_ids.push(health.id.clone());
                    let message = format!(
                        "{}: missing replay snapshot and fetch_live=false",
                        source.name
                    );
                    errors.push(message.clone());
                    refreshes.push(json!({
                        "source_id": source_id,
                        "source_name": source.name,
                        "status": "failed",
                        "error_code": "missing_snapshot",
                        "role_count": 0,
                        "company_count": 0,
                        "stale_role_count": 0,
                    }));
                    continue;
                }
            };

            for role in &refresh.roles {
                observed_role_ids.insert(role.id.clone());
            }
            for event in &refresh.stale_role_events {
                stale_role_ids.insert(event.role_id.clone());
            }
            source_health_ids.push(refresh.source_health.id.clone());
            if job_source_health_status_counts_as_error(&refresh.source_health.status) {
                errors.push(format!(
                    "{}: source health {}",
                    refresh.source.name, refresh.source_health.status
                ));
            } else if refresh.source_health.status == "partial" {
                warnings.push(format!(
                    "{}: source health partial; accepted roles were kept and rejected source noise remains visible",
                    refresh.source.name
                ));
            }
            warnings.extend(refresh.warnings.clone());
            refreshes.push(json!({
                "source_id": refresh.source.id,
                "source_name": refresh.source.name,
                "status": refresh.source_health.status,
                "source_health_id": refresh.source_health.id,
                "role_count": refresh.roles.len(),
                "company_count": refresh.companies.len(),
                "stale_role_count": refresh.stale_role_events.len(),
                "fetched_count": refresh.fetched_count,
                "accepted_count": refresh.accepted_count,
                "rejected_count": refresh.rejected_count,
            }));
        }

        for stale_role_id in &stale_role_ids {
            observed_role_ids.remove(stale_role_id);
        }

        let refresh_report = self.run_job_manual_refresh(JobManualRefreshInput {
            profile_id: profile.id.clone(),
            scope: scope.clone(),
            observed_role_ids: observed_role_ids.into_iter().collect(),
            stale_role_ids: stale_role_ids.into_iter().collect(),
            closed_role_ids: Vec::new(),
            source_health_ids: source_health_ids.clone(),
            proof_level: proof_level.clone(),
            report_artifact_id: None,
        })?;
        let weekly_report = self.compile_job_weekly_report(&profile.id, &scope)?;
        let delivery_report = self.run_job_radar_weekly_report_delivery(
            weekly_report.id.as_str(),
            input.get("delivery"),
        )?;

        if let Some(lineage) = input.get("lineage")
            && let Some(source_key) = lineage.get("watch_source_key").and_then(Value::as_str)
        {
            let source_kind = lineage
                .get("source_kind")
                .and_then(Value::as_str)
                .unwrap_or("job_radar");
            let locator = lineage
                .get("locator")
                .and_then(Value::as_str)
                .unwrap_or(profile.id.as_str());
            if refresh_report.error_count == 0 {
                let next_run_at = lineage
                    .get("cadence")
                    .and_then(Value::as_str)
                    .and_then(watch_source_cadence_seconds)
                    .map(now_plus_seconds);
                self.record_source_success(SourceHealthUpdate {
                    key: source_key,
                    provider: "arcwell",
                    source_kind,
                    locator,
                    last_item_id: Some(&refresh_report.run.id),
                    last_item_date: refresh_report.run.completed_at.as_deref(),
                    cursor_key: None,
                    cursor_value: None,
                    next_run_at: next_run_at.as_deref(),
                })?;
            } else {
                self.record_source_failure(
                    source_key,
                    "arcwell",
                    source_kind,
                    locator,
                    &format!(
                        "job radar refresh completed with {} unhealthy source(s)",
                        refresh_report.error_count
                    ),
                )?;
            }
        }

        warnings.extend(refresh_report.warnings.clone());
        warnings.sort();
        warnings.dedup();
        errors.sort();
        errors.dedup();

        Ok(json!({
            "action": "job_radar_refresh",
            "profile_id": profile.id,
            "scope": scope,
            "proof_level": proof_level,
            "source_count": source_ids.len(),
            "fetch_live": fetch_live,
            "refreshes": refreshes,
            "search_run_id": refresh_report.run.id,
            "weekly_report_id": weekly_report.id,
            "observed_role_count": refresh_report.new_role_count
                + refresh_report.unchanged_role_count
                + refresh_report.promoted_role_count
                + refresh_report.demoted_role_count,
            "stale_role_count": refresh_report.stale_role_count,
            "closed_role_count": refresh_report.closed_role_count,
            "source_health_ids": source_health_ids,
            "error_count": refresh_report.error_count,
            "errors": errors,
            "warnings": warnings,
            "delivery": delivery_report,
        }))
    }

    pub(crate) fn run_job_radar_weekly_report_delivery(
        &self,
        weekly_report_id: &str,
        delivery: Option<&Value>,
    ) -> Result<Option<Value>> {
        let Some(delivery) = delivery else {
            return Ok(None);
        };
        if delivery.is_null() {
            return Ok(None);
        }
        let Some(object) = delivery.as_object() else {
            bail!("job radar delivery config must be an object");
        };
        let channel = object
            .get("channel")
            .and_then(Value::as_str)
            .unwrap_or("email")
            .to_string();
        let subject = object
            .get("subject")
            .and_then(Value::as_str)
            .context("job radar delivery missing subject")?
            .to_string();
        let target = object
            .get("target")
            .and_then(Value::as_str)
            .context("job radar delivery missing target")?
            .to_string();
        let idempotency_key = object
            .get("idempotency_key")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);

        let prepared = self.prepare_job_weekly_report_delivery(JobWeeklyReportDeliveryInput {
            report_id: weekly_report_id.to_string(),
            channel,
            subject,
            target,
            idempotency_key,
        })?;
        if prepared.delivery.status != "prepared" {
            return Ok(Some(json!({
                "status": prepared.delivery.status,
                "delivery_id": prepared.delivery.id,
                "report_id": weekly_report_id,
                "channel_message_id": prepared.delivery.channel_message_id,
                "error": prepared.delivery.error,
                "sent": false,
            })));
        }

        let sent = self.send_job_weekly_report_delivery(JobWeeklyReportDeliverySendInput {
            delivery_id: prepared.delivery.id.clone(),
            telegram_bot_token: None,
            email_account_id: None,
            email_api_token: None,
            email_from: None,
            api_base: None,
        })?;
        Ok(Some(json!({
            "status": sent.delivery.status,
            "delivery_id": sent.delivery.id,
            "report_id": weekly_report_id,
            "channel_message_id": sent.delivery.channel_message_id,
            "channel_delivery_attempt_id": sent.channel_delivery_attempt.as_ref().map(|attempt| attempt.id.clone()),
            "sent": sent.channel_delivery_attempt.as_ref().map(|attempt| attempt.ok).unwrap_or(false),
            "proof_level": sent.proof_level,
            "non_claims": sent.non_claims,
            "error": sent.delivery.error,
        })))
    }

    pub(crate) fn guard_wiki_job_provider_policy(&self, job: &WikiJob) -> Result<()> {
        let (package, provider, target, projected_usd) =
            wiki_job_policy_context(&job.kind, &job.input_json);
        let Some(provider) = provider else {
            return Ok(());
        };
        self.policy_guard(PolicyRequest {
            action: "provider.network".to_string(),
            package: Some(package.to_string()),
            provider: Some(provider.to_string()),
            source: Some(provider_network_source_for_job(&job.kind).to_string()),
            channel: None,
            subject: None,
            target,
            projected_usd,
            metadata: json!({ "job_id": job.id, "kind": job.kind }),
            untrusted_excerpt: None,
        })?;
        Ok(())
    }

    pub(crate) fn guard_wiki_job_cost(&self, job: &WikiJob) -> Result<()> {
        let Some((provider, model, source, projected)) = scheduled_job_cost_projection(job)? else {
            return Ok(());
        };
        self.require_cost_budget(
            "arcwell-llm-wiki",
            &job.id,
            provider,
            model,
            Some(source),
            projected,
            &format!("scheduled {} job", job.kind),
        )?;
        Ok(())
    }

    pub(crate) fn execute_ingest_file(&self, input: &Value) -> Result<Value> {
        let path = input
            .get("path")
            .and_then(Value::as_str)
            .context("ingest_file missing path")?;
        let page_id = self.ingest_wiki_file(Path::new(path))?;
        Ok(json!({ "page_id": page_id }))
    }

    pub(crate) fn execute_ingest_url(&self, input: &Value) -> Result<Value> {
        let url = input
            .get("url")
            .and_then(Value::as_str)
            .context("ingest_url missing url")?;
        let url = validate_fetch_url(url)?;
        let research_context = self.research_url_ingest_context(input)?;
        self.guard_provider_network_policy(
            "arcwell-llm-wiki",
            "web",
            "url_ingest",
            url.as_str(),
            estimated_network_fetch_cost(1),
            json!({ "entrypoint": "execute_ingest_url" }),
        )?;
        let doc = fetch_url_ingest_document(url)?;
        let markdown = render_url_ingest_page(&doc);
        let page_id = self.add_wiki_page(&doc.title, &markdown, &doc.canonical_url)?;
        let source_health_key = self.record_blog_watch_source_success_for_url_ingest_urls(
            &[
                doc.requested_url.as_str(),
                doc.final_url.as_str(),
                doc.canonical_url.as_str(),
            ],
            &page_id,
        )?;
        let research_promotion =
            self.promote_research_url_ingest_document(research_context.as_ref(), &doc, &page_id)?;
        Ok(json!({
            "page_id": page_id,
            "bytes": doc.byte_len,
            "canonical_url": doc.canonical_url,
            "final_url": doc.final_url,
            "content_type": doc.content_type,
            "source_health_key": source_health_key,
            "research_promotion": research_promotion
        }))
    }

    pub(crate) fn execute_x_profile_enrichment(&self, input: &Value) -> Result<Value> {
        let handles = input
            .get("handles")
            .and_then(Value::as_array)
            .context("x_profile_enrichment missing handles")?
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(ToOwned::to_owned)
                    .context("x_profile_enrichment handles must be strings")
            })
            .collect::<Result<Vec<_>>>()?;
        if handles.is_empty() {
            bail!("x_profile_enrichment handles must not be empty");
        }
        let limit = input.get("limit").and_then(Value::as_u64).unwrap_or(100) as usize;
        let run_id = input.get("run_id").and_then(Value::as_str);
        let report = self.x_enrich_watch_profiles(run_id, &handles, limit)?;
        Ok(serde_json::to_value(report)?)
    }

    pub(crate) fn promote_research_url_ingest_document(
        &self,
        context: Option<&ResearchUrlIngestContext>,
        doc: &UrlIngestDocument,
        url_ingest_wiki_page_id: &str,
    ) -> Result<Option<Value>> {
        let Some(context) = context else {
            return Ok(None);
        };
        let title = context
            .host_search_result
            .as_ref()
            .map(|result| result.title.as_str())
            .filter(|title| !title.trim().is_empty())
            .unwrap_or(&doc.title);
        let mut summary = excerpt(&doc.readable_text, 8_000);
        if summary.trim().is_empty() {
            summary = "URL ingestion produced no readable summary.".to_string();
        }
        let claim_text = research_url_ingest_claim_text(&doc.readable_text);
        let source_claims = claim_text
            .as_ref()
            .map(|text| SourceClaim {
                claim: text.clone(),
                kind: "fact".to_string(),
                confidence: 0.64,
            })
            .into_iter()
            .collect::<Vec<_>>();
        let host_search_id = context.host_search_id.clone().or_else(|| {
            context
                .host_search_result
                .as_ref()
                .map(|result| result.host_search_id.clone())
        });
        let host_search_result_id = context
            .host_search_result
            .as_ref()
            .map(|result| result.id.clone());
        let card = self.add_source_card(SourceCardInput {
            title: excerpt(title, 200),
            url: doc.canonical_url.clone(),
            source_type: context.source_type.clone(),
            provider: "research-url-ingest".to_string(),
            summary,
            claims: source_claims,
            retrieved_at: doc.captured_at.clone(),
            metadata: json!({
                "source_role": "secondary",
                "trust_level": "medium",
                "provenance_strength": "direct",
                "source_family": context.source_family.clone(),
                "url_ingest_wiki_page_id": url_ingest_wiki_page_id,
                "requested_url": doc.requested_url,
                "final_url": doc.final_url,
                "content_type": doc.content_type,
                "extraction_method": doc.extraction_method,
                "robots_meta": doc.robots_meta,
                "robots_noindex": doc.robots_noindex,
                "robots_nofollow": doc.robots_nofollow,
                "crawl_rate_policy": doc.crawl_rate_policy,
                "host_search_id": host_search_id,
                "host_search_result_id": host_search_result_id,
                "source": "research_url_ingest"
            }),
        })?;
        let notes =
            format!("Promoted from research-scoped URL ingest page {url_ingest_wiki_page_id}.");
        let linked = if let Some(result) = &context.host_search_result
            && let Some(source_id) = &result.research_source_id
            && let Some(source) = self.read_research_source(source_id)?
        {
            let mut metadata = source.metadata.as_object().cloned().unwrap_or_default();
            metadata.insert("source_card_id".to_string(), json!(card.id));
            metadata.insert(
                "url_ingest_wiki_page_id".to_string(),
                json!(url_ingest_wiki_page_id),
            );
            metadata.insert(
                "host_search_result_id".to_string(),
                json!(result.id.clone()),
            );
            let source = self.upsert_research_source(ResearchSourceInput {
                url: Some(card.url.clone()),
                local_ref: Some(format!("source-card:{}", card.id)),
                title: card.title.clone(),
                source_family: context.source_family.clone(),
                source_type: card.source_type.clone(),
                provider: card.provider.clone(),
                author: source.author,
                published_at: source.published_at,
                language: source.language,
                priority: source.priority,
                reason: notes.clone(),
                canonical_key: Some(source.canonical_key),
                fetch_status: "carded".to_string(),
                read_depth: "full-text".to_string(),
                metadata: Value::Object(metadata),
            })?;
            self.link_research_source_to_run(
                &context.run_id,
                &source.id,
                Some(&card.id),
                "read",
                "full-text",
                Some(&notes),
            )?
        } else {
            self.link_source_card_to_research_run(
                &context.run_id,
                &card.id,
                &context.source_family,
                "full-text",
                "read",
                Some(&notes),
            )?
        };
        let mut claims_ingested = 0usize;
        if let Some(claim_text) = research_url_ingest_claim_text(&doc.readable_text) {
            let output = json!({
                "claims": [{
                    "text": claim_text,
                    "kind": "fact",
                    "confidence": 0.64,
                    "caveats": [
                        "Extracted deterministically from bounded URL-ingest readable text; verify quoted, numeric, or currentness-sensitive claims against the original source before publication."
                    ],
                    "quote": excerpt(&doc.readable_text, 500),
                    "source_anchor": format!("url-ingest:{url_ingest_wiki_page_id}")
                }]
            });
            claims_ingested = self
                .ingest_research_claims_from_model_output(
                    &context.run_id,
                    &card.id,
                    "research-url-ingest",
                    "deterministic-readable-text",
                    &output.to_string(),
                )?
                .len();
        }
        if let Some(result) = &context.host_search_result {
            self.conn.execute(
                r#"
                UPDATE research_host_search_results
                SET source_card_id = ?1, research_source_id = COALESCE(research_source_id, ?2)
                WHERE id = ?3
                "#,
                params![card.id, linked.source.id, result.id],
            )?;
        }
        Ok(Some(json!({
            "run_id": context.run_id,
            "source_card_id": card.id,
            "research_source_id": linked.source.id,
            "research_run_source_link_id": linked.link.id,
            "host_search_id": host_search_id,
            "host_search_result_id": host_search_result_id,
            "claims_ingested": claims_ingested,
            "read_depth": "full-text"
        })))
    }

    pub(crate) fn execute_ingest_rendered_page(&self, input: &Value) -> Result<Value> {
        let input: RenderedPageSnapshotInput = serde_json::from_value(input.clone())
            .context("invalid rendered page snapshot input")?;
        let doc = rendered_page_snapshot_document(&input)?;
        let markdown = render_url_ingest_page(&doc);
        let page_id = self.add_wiki_page(&doc.title, &markdown, &doc.canonical_url)?;
        Ok(json!({
            "page_id": page_id,
            "bytes": doc.byte_len,
            "canonical_url": doc.canonical_url,
            "final_url": doc.final_url,
            "content_type": doc.content_type,
            "extraction_method": doc.extraction_method,
            "capture_method": "host_browser_rendered_snapshot"
        }))
    }

    pub(crate) fn execute_compile(&self, input: &Value) -> Result<Value> {
        let query = input
            .get("query")
            .and_then(Value::as_str)
            .context("compile missing query")?;
        let brief = self.create_research_brief_from_wiki(query, true)?;
        Ok(json!({
            "run_id": brief.run.id,
            "page_id": brief.result_page_id,
            "source_count": brief.source_count
        }))
    }

    pub(crate) fn execute_expand_page(&self, input: &Value) -> Result<Value> {
        let topic = input
            .get("topic")
            .and_then(Value::as_str)
            .context("expand_page missing topic")?;
        validate_query(topic)?;
        let sources: Vec<SourceCard> = self
            .search_source_cards(topic)?
            .into_iter()
            .filter(source_card_is_primary_evidence)
            .collect();
        let pages = self.search_wiki_pages_for_research(topic)?;
        let markdown = render_expanded_wiki_page(topic, &sources, &pages)?;
        let page_id =
            self.add_wiki_page(&format!("Expanded: {topic}"), &markdown, "wiki-expand")?;
        Ok(json!({
            "page_id": page_id,
            "source_cards": sources.len(),
            "wiki_pages": pages.len()
        }))
    }
}

fn knowledge_cluster_model_proposal_error_is_deferable_provider_failure(error: &str) -> bool {
    let lower = error.to_ascii_lowercase();
    if lower.contains("policy denied")
        || lower.contains("policy deferred")
        || lower.contains("budget blocked")
        || lower.contains("openai_api_key is required")
        || crate::knowledge::knowledge_cluster_model_proposal_error_is_non_retryable(error)
    {
        return false;
    }
    lower.contains("knowledge cluster proposal request failed")
        || lower.contains("knowledge cluster proposal returned an error status")
        || lower.contains("timed out")
        || lower.contains("timeout")
        || lower.contains("429")
        || lower.contains("rate limit")
        || lower.contains("temporarily unavailable")
        || lower.contains("connection")
        || lower.contains("dns")
}
