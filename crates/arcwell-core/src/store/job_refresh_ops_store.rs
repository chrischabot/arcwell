use super::*;

impl Store {
    pub fn import_job_batch(&self, input: JobImportBatchInput) -> Result<JobImportBatchReport> {
        let total_items = job_import_batch_item_count(&input);
        if total_items == 0 {
            bail!("job import batch is empty");
        }
        if total_items > JOB_MAX_IMPORT_ITEMS {
            bail!("job import batch has too many items");
        }

        let mut report = JobImportBatchReport {
            imported_at: now(),
            proof_level: "local_proof_reviewed_packet".to_string(),
            profile_ids: Vec::new(),
            evidence_card_ids: Vec::new(),
            evidence_claim_ids: Vec::new(),
            privacy_rule_ids: Vec::new(),
            source_ids: Vec::new(),
            source_health_ids: Vec::new(),
            role_ids: Vec::new(),
            role_source_link_ids: Vec::new(),
            fit_score_ids: Vec::new(),
            skeptic_finding_ids: Vec::new(),
            packet_ids: Vec::new(),
            company_ids: Vec::new(),
            contact_ids: Vec::new(),
            intro_path_ids: Vec::new(),
            search_run_ids: Vec::new(),
            role_status_event_ids: Vec::new(),
            application_ids: Vec::new(),
            warnings: vec![
                "Reviewed packet import records supplied facts only; it does not prove live source discovery, source freshness, or scheduled refresh.".to_string(),
            ],
        };

        if let Some(profile) = input.profile {
            report
                .profile_ids
                .push(self.record_job_candidate_profile(profile)?.id);
        }
        for rule in input.privacy_rules {
            report
                .privacy_rule_ids
                .push(self.record_job_privacy_rule(rule)?.id);
        }
        for evidence in input.evidence_cards {
            let card = self.record_job_evidence_card(evidence)?;
            if card.visibility == "needs_review" {
                report.warnings.push(format!(
                    "Evidence card `{}` remains needs_review and should not be treated as application-approved.",
                    card.title
                ));
            }
            report.evidence_card_ids.push(card.id);
        }
        for claim in input.evidence_claims {
            report
                .evidence_claim_ids
                .push(self.record_job_evidence_claim(claim)?.id);
        }
        for source in input.sources {
            report.source_ids.push(self.record_job_source(source)?.id);
        }
        for health in input.source_health {
            let health = self.record_job_source_health(health)?;
            if health.status != "healthy" {
                report.warnings.push(format!(
                    "Source health `{}` recorded as {}; downstream reports must surface it.",
                    health.source_id, health.status
                ));
            }
            report.source_health_ids.push(health.id);
        }
        for company in input.companies {
            report
                .company_ids
                .push(self.record_job_company_card(company)?.id);
        }
        for contact in input.contacts {
            report
                .contact_ids
                .push(self.record_job_contact(contact)?.id);
        }
        for role in input.roles {
            let role = self.record_job_role_card(role)?;
            if role.source_confidence != "canonical_confirmed" {
                report.warnings.push(format!(
                    "Role `{}` at `{}` is {}; it cannot become apply-now Tier 1 without stronger source confidence.",
                    role.role_title, role.company, role.source_confidence
                ));
            }
            report.role_ids.push(role.id);
        }
        for link in input.role_source_links {
            report
                .role_source_link_ids
                .push(self.record_job_role_source_link(link)?.id);
        }
        for score in input.fit_scores {
            report
                .fit_score_ids
                .push(self.record_job_fit_score(score)?.id);
        }
        for finding in input.skeptic_findings {
            report
                .skeptic_finding_ids
                .push(self.record_job_skeptic_finding(finding)?.id);
        }
        for packet in input.packets {
            report
                .packet_ids
                .push(self.create_job_application_packet(packet)?.id);
        }
        for intro in input.intro_paths {
            report
                .intro_path_ids
                .push(self.record_job_intro_path(intro)?.id);
        }
        for run in input.search_runs {
            report
                .search_run_ids
                .push(self.record_job_search_run(run)?.id);
        }
        for event in input.role_status_events {
            report
                .role_status_event_ids
                .push(self.record_job_role_status_event(event)?.id);
        }
        for application in input.applications {
            report
                .application_ids
                .push(self.record_job_application(application)?.id);
        }

        Ok(report)
    }

    pub fn run_job_manual_refresh(
        &self,
        input: JobManualRefreshInput,
    ) -> Result<JobManualRefreshReport> {
        let input = normalize_job_manual_refresh_input(input)?;
        self.require_job_profile(&input.profile_id)?;

        let mut observed_role_ids = input.observed_role_ids.clone();
        let stale_role_ids = input.stale_role_ids.clone();
        let closed_role_ids = input.closed_role_ids.clone();
        for role_id in stale_role_ids.iter().chain(closed_role_ids.iter()) {
            observed_role_ids.retain(|existing| existing != role_id);
        }

        let mut planned_events: Vec<(String, String, Option<String>, Option<String>, String)> =
            Vec::new();
        let mut new_role_count = 0usize;
        let mut unchanged_role_count = 0usize;
        let mut promoted_role_count = 0usize;
        let mut demoted_role_count = 0usize;
        let mut stale_role_count = 0usize;
        let mut closed_role_count = 0usize;

        for role_id in &observed_role_ids {
            let role = self
                .read_job_role_card(role_id)?
                .with_context(|| format!("job role card not found: {role_id}"))?;
            let previous = self.latest_job_role_status_event(role_id)?;
            let previous_tier = previous
                .as_ref()
                .and_then(|event| event.current_tier.clone());
            let current_tier = self
                .latest_job_fit_score(role_id, &input.profile_id)?
                .map(|score| job_effective_score_for_role(&role, score).tier);
            let status = match previous_tier.as_deref().zip(current_tier.as_deref()) {
                None if previous.is_none() => {
                    new_role_count += 1;
                    "new"
                }
                Some((previous_tier, current_tier))
                    if job_tier_sort_rank(current_tier) < job_tier_sort_rank(previous_tier) =>
                {
                    promoted_role_count += 1;
                    "promoted"
                }
                Some((previous_tier, current_tier))
                    if job_tier_sort_rank(current_tier) > job_tier_sort_rank(previous_tier) =>
                {
                    demoted_role_count += 1;
                    "demoted"
                }
                _ => {
                    unchanged_role_count += 1;
                    "unchanged"
                }
            };
            planned_events.push((
                role_id.clone(),
                status.to_string(),
                previous_tier,
                current_tier,
                "Manual refresh observed this role in the current source set.".to_string(),
            ));
        }

        for role_id in &stale_role_ids {
            self.update_job_role_current_status(role_id, "stale")?;
            let previous_tier = self
                .latest_job_role_status_event(role_id)?
                .and_then(|event| event.current_tier);
            planned_events.push((
                role_id.clone(),
                "stale".to_string(),
                previous_tier,
                Some("blocked".to_string()),
                "Manual refresh marked this role stale; it cannot remain apply-now.".to_string(),
            ));
            stale_role_count += 1;
        }

        for role_id in &closed_role_ids {
            self.update_job_role_current_status(role_id, "closed")?;
            let previous_tier = self
                .latest_job_role_status_event(role_id)?
                .and_then(|event| event.current_tier);
            planned_events.push((
                role_id.clone(),
                "closed".to_string(),
                previous_tier,
                Some("blocked".to_string()),
                "Manual refresh marked this role closed; it cannot remain apply-now.".to_string(),
            ));
            closed_role_count += 1;
        }

        let source_health = self.read_job_source_health_ids(&input.source_health_ids)?;
        let source_count = source_health
            .iter()
            .map(|health| health.source_id.clone())
            .collect::<BTreeSet<_>>()
            .len();
        let error_count = source_health
            .iter()
            .filter(|health| health.status != "healthy")
            .count();
        let stale_total = stale_role_count + closed_role_count;
        let run = self.record_job_search_run(JobSearchRunInput {
            profile_id: input.profile_id.clone(),
            scope: input.scope.clone(),
            proof_level: input.proof_level.clone(),
            source_count,
            role_count: planned_events.len(),
            new_role_count,
            stale_role_count: stale_total,
            error_count,
            report_artifact_id: input.report_artifact_id.clone(),
            completed_at: Some(now()),
        })?;

        let mut events = Vec::new();
        for (role_id, status, previous_tier, current_tier, note) in planned_events {
            events.push(self.record_job_role_status_event(JobRoleStatusEventInput {
                role_id,
                run_id: Some(run.id.clone()),
                status,
                previous_tier,
                current_tier,
                note: Some(note),
            })?);
        }

        let mut warnings = vec![
            "Manual refresh reconciliation uses caller-supplied observed/stale/closed role ids; it is not live source discovery.".to_string(),
        ];
        if error_count > 0 {
            warnings.push(
                "One or more source-health rows were not healthy and must remain visible in reports."
                    .to_string(),
            );
        }

        Ok(JobManualRefreshReport {
            run,
            events,
            source_health,
            new_role_count,
            unchanged_role_count,
            promoted_role_count,
            demoted_role_count,
            stale_role_count,
            closed_role_count,
            error_count,
            warnings,
        })
    }

    pub fn audit_job_refresh_history(
        &self,
        profile_id: &str,
        scope: &str,
        minimum_elapsed_hours: Option<i64>,
    ) -> Result<JobRefreshAudit> {
        validate_id(profile_id)?;
        self.require_job_profile(profile_id)?;
        let scope = sanitize_required_job_text(scope, "refresh scope", JOB_MAX_TEXT)?;
        let minimum_elapsed_hours = minimum_elapsed_hours.unwrap_or(24).max(0);
        let runs = self.list_job_search_runs_for_scope(profile_id, &scope)?;

        let mut missing_requirements = Vec::new();
        let mut warnings = vec![
            "Refresh audit reads durable job_search_runs and job_role_status_events only; it does not fetch sources.".to_string(),
            "Operational promotion still requires a real controlled-home proof run, not only this local audit shape.".to_string(),
        ];
        if minimum_elapsed_hours < 24 {
            warnings.push(
                "minimum_elapsed_hours is below the operational one-day gate; this can prove audit logic only."
                    .to_string(),
            );
        }

        if runs.len() < 2 {
            missing_requirements.push(
                "At least two completed job search runs for the same profile and scope are required."
                    .to_string(),
            );
        }

        let mut run_evidence = Vec::new();
        let mut transition_counts: BTreeMap<String, usize> = BTreeMap::new();
        let mut total_source_count = 0usize;
        let mut total_role_count = 0usize;
        let mut total_error_count = 0usize;

        for run in &runs {
            total_source_count += run.source_count;
            total_role_count += run.role_count;
            total_error_count += run.error_count;
            let events = self.list_job_role_status_events_for_run(&run.id)?;
            let mut event_counts = BTreeMap::new();
            for event in events {
                *event_counts.entry(event.status.clone()).or_insert(0) += 1;
                *transition_counts.entry(event.status).or_insert(0) += 1;
            }
            if event_counts.is_empty() {
                missing_requirements.push(format!(
                    "Search run {} has no linked role-status events.",
                    run.id
                ));
            }
            if run.source_count == 0 {
                missing_requirements.push(format!(
                    "Search run {} has no recorded source-health/source evidence.",
                    run.id
                ));
            }
            run_evidence.push(JobRefreshAuditRunEvidence {
                run_id: run.id.clone(),
                started_at: run.started_at.clone(),
                completed_at: run.completed_at.clone(),
                proof_level: run.proof_level.clone(),
                source_count: run.source_count,
                role_count: run.role_count,
                new_role_count: run.new_role_count,
                stale_role_count: run.stale_role_count,
                error_count: run.error_count,
                event_counts,
            });
        }

        let first_run = runs.first();
        let latest_run = runs.last();
        let elapsed_hours = match (first_run, latest_run) {
            (Some(first), Some(latest)) => {
                let first_started = DateTime::parse_from_rfc3339(&first.started_at)
                    .with_context(|| format!("parsing job search run started_at {}", first.id))?
                    .with_timezone(&Utc);
                let latest_started = DateTime::parse_from_rfc3339(&latest.started_at)
                    .with_context(|| format!("parsing job search run started_at {}", latest.id))?
                    .with_timezone(&Utc);
                let elapsed = latest_started
                    .signed_duration_since(first_started)
                    .num_seconds();
                Some(elapsed.max(0) as f64 / 3600.0)
            }
            _ => None,
        };

        if elapsed_hours
            .map(|elapsed| elapsed < minimum_elapsed_hours as f64)
            .unwrap_or(true)
        {
            missing_requirements.push(format!(
                "First and latest completed runs are not at least {minimum_elapsed_hours} hours apart by started_at."
            ));
        }
        if total_source_count == 0 {
            missing_requirements.push(
                "Refresh audit has no source-count evidence across completed runs.".to_string(),
            );
        }
        if total_role_count == 0 {
            missing_requirements.push(
                "Refresh audit has no role-count evidence across completed runs.".to_string(),
            );
        }
        for status in ["new", "unchanged", "stale", "closed"] {
            if transition_counts.get(status).copied().unwrap_or(0) == 0 {
                missing_requirements.push(format!(
                    "Refresh audit lacks a `{status}` role-status transition."
                ));
            }
        }

        let decision = if missing_requirements.is_empty() {
            "pass"
        } else {
            "block"
        }
        .to_string();

        Ok(JobRefreshAudit {
            profile_id: profile_id.to_string(),
            scope,
            generated_at: now(),
            decision,
            proof_level: "local_proof_refresh_audit_gate".to_string(),
            minimum_elapsed_hours,
            elapsed_hours,
            completed_run_count: runs.len(),
            first_run_id: first_run.map(|run| run.id.clone()),
            latest_run_id: latest_run.map(|run| run.id.clone()),
            first_started_at: first_run.map(|run| run.started_at.clone()),
            latest_started_at: latest_run.map(|run| run.started_at.clone()),
            total_source_count,
            total_role_count,
            total_error_count,
            transition_counts,
            run_evidence,
            missing_requirements,
            warnings,
        })
    }

    pub fn audit_job_operational_readiness(
        &self,
        profile_id: &str,
        scope: &str,
        minimum_elapsed_hours: Option<i64>,
    ) -> Result<JobOperationalAudit> {
        validate_id(profile_id)?;
        self.require_job_profile(profile_id)?;
        let scope = sanitize_required_job_text(scope, "refresh scope", JOB_MAX_TEXT)?;
        let minimum_elapsed_hours = minimum_elapsed_hours.unwrap_or(24).max(24);
        let ops_summary = self.job_ops_summary()?;
        let refresh_audit =
            self.audit_job_refresh_history(profile_id, &scope, Some(minimum_elapsed_hours))?;
        let outreach_readiness = self.compile_job_outreach_readiness_report(profile_id, 100)?;
        let source_family_counts = self.count_job_group("job_sources", "source_family")?;
        let evidence_visibility_counts =
            self.count_job_evidence_visibility_for_profile(profile_id)?;
        let evidence_card_count = evidence_visibility_counts.values().sum::<usize>();
        let packet_status_counts =
            self.count_job_application_packet_status_for_profile(profile_id)?;
        let intro_status_counts = self.count_job_intro_path_status_for_profile(profile_id)?;
        let role_status_counts = self.count_job_role_status_for_profile(profile_id)?;
        let role_count = role_status_counts.values().sum::<usize>();
        let score_tier_counts = self.count_job_fit_score_tiers_for_profile(profile_id)?;
        let privacy_decision_counts =
            self.count_job_privacy_decisions_for_profile_scope(profile_id, &scope)?;
        let application_status_counts =
            self.count_job_application_status_for_profile(profile_id)?;
        let follow_up_count = self.count_job_follow_ups_for_profile(profile_id)?;
        let weekly_report_count = self.count_job_weekly_reports_for_scope(profile_id, &scope)?;
        let weekly_delivery_status_counts =
            self.count_job_weekly_report_delivery_status_for_scope(profile_id, &scope)?;
        let weekly_report_delivery_attempt_count =
            self.count_job_weekly_report_delivery_attempts_for_scope(profile_id, &scope)?;
        let job_radar_jobs = self
            .list_wiki_jobs()?
            .into_iter()
            .filter(|job| {
                job.kind == "job_radar_refresh"
                    && job.input_json.get("profile_id").and_then(Value::as_str) == Some(profile_id)
                    && job.input_json.get("scope").and_then(Value::as_str) == Some(scope.as_str())
            })
            .collect::<Vec<_>>();
        let mut job_radar_job_status_counts: BTreeMap<String, usize> = BTreeMap::new();
        let mut job_radar_completed_count = 0usize;
        let mut job_radar_dead_lettered_count = 0usize;
        let mut job_radar_attempt_count = 0i64;
        for job in &job_radar_jobs {
            *job_radar_job_status_counts
                .entry(job.status.clone())
                .or_insert(0) += 1;
            if job.status == "completed" {
                job_radar_completed_count += 1;
            }
            if job.status == "dead_lettered" {
                job_radar_dead_lettered_count += 1;
            }
            job_radar_attempt_count += job.attempts;
        }

        let mut gates = Vec::new();
        push_job_operational_gate(
            &mut gates,
            "evidence_ledger",
            json!({
                "evidence_card_count": evidence_card_count,
                "visibility_counts": evidence_visibility_counts,
            }),
            gate_missing(
                evidence_card_count > 0,
                "No candidate evidence cards are recorded for the operational audit.",
            ),
        );
        push_job_operational_gate(
            &mut gates,
            "source_map",
            json!({
                "source_count": ops_summary.source_count,
                "source_family_counts": source_family_counts,
            }),
            gate_missing(
                ops_summary.source_count > 0,
                "No job sources are configured or imported.",
            ),
        );
        push_job_operational_gate(
            &mut gates,
            "role_scoring",
            json!({
                "role_count": role_count,
                "role_status_counts": role_status_counts,
                "score_tier_counts": score_tier_counts,
            }),
            gate_missing(
                role_count > 0
                    && ["tier_1", "tier_2"]
                        .iter()
                        .any(|tier| score_tier_counts.get(*tier).copied().unwrap_or(0) > 0),
                "No live role set with Tier 1 or Tier 2 scoring evidence is present.",
            ),
        );
        let mut privacy_missing = Vec::new();
        if privacy_decision_counts.get("block").copied().unwrap_or(0) > 0 {
            privacy_missing.push(
                "At least one job privacy check is blocked; public material needs review."
                    .to_string(),
            );
        }
        push_job_operational_gate(
            &mut gates,
            "privacy",
            json!({
                "privacy_decision_counts": privacy_decision_counts,
            }),
            privacy_missing,
        );
        push_job_operational_gate(
            &mut gates,
            "application_packets",
            json!({
                "packet_status_counts": packet_status_counts,
            }),
            gate_missing(
                packet_status_counts.get("approved").copied().unwrap_or(0) > 0,
                "No approved application packet is recorded.",
            ),
        );
        push_job_operational_gate(
            &mut gates,
            "outreach_readiness",
            json!({
                "ready_count": outreach_readiness.ready_count,
                "blocked_count": outreach_readiness.blocked_count,
                "intro_status_counts": intro_status_counts,
            }),
            gate_missing(
                outreach_readiness.ready_count > 0,
                "No scored role has an approved packet, fresh privacy pass, and warm or user-confirmed outreach route.",
            ),
        );
        push_job_operational_gate(
            &mut gates,
            "application_tracking",
            json!({
                "application_status_counts": application_status_counts,
                "follow_up_count": follow_up_count,
            }),
            gate_missing(
                !application_status_counts.is_empty(),
                "No application status or outcome rows are recorded.",
            ),
        );
        push_job_operational_gate(
            &mut gates,
            "weekly_refresh",
            json!({
                "refresh_decision": refresh_audit.decision,
                "minimum_elapsed_hours": refresh_audit.minimum_elapsed_hours,
                "elapsed_hours": refresh_audit.elapsed_hours,
                "completed_run_count": refresh_audit.completed_run_count,
                "transition_counts": refresh_audit.transition_counts,
            }),
            refresh_audit.missing_requirements.clone(),
        );
        push_job_operational_gate(
            &mut gates,
            "weekly_report",
            json!({
                "weekly_report_count": weekly_report_count,
            }),
            gate_missing(
                weekly_report_count > 0,
                "No weekly report exists for this profile and scope.",
            ),
        );
        push_job_operational_gate(
            &mut gates,
            "delivery_preparation",
            json!({
                "weekly_delivery_status_counts": weekly_delivery_status_counts,
            }),
            gate_missing(
                ["prepared", "sent"]
                    .iter()
                    .map(|status| {
                        weekly_delivery_status_counts
                            .get(*status)
                            .copied()
                            .unwrap_or(0)
                    })
                    .sum::<usize>()
                    > 0,
                "No prepared or sent weekly-report delivery row exists for this profile and scope.",
            ),
        );
        push_job_operational_gate(
            &mut gates,
            "provider_delivery",
            json!({
                "weekly_report_delivery_attempt_count": weekly_report_delivery_attempt_count,
            }),
            gate_missing(
                weekly_report_delivery_attempt_count > 0,
                "No successful provider delivery attempt is linked to a weekly job report for this profile and scope.",
            ),
        );
        push_job_operational_gate(
            &mut gates,
            "scheduled_radar",
            json!({
                "job_radar_job_status_counts": job_radar_job_status_counts,
                "completed_count": job_radar_completed_count,
                "dead_lettered_count": job_radar_dead_lettered_count,
                "attempt_count": job_radar_attempt_count,
            }),
            gate_missing(
                job_radar_completed_count >= 2 && job_radar_dead_lettered_count == 0,
                "Operational radar requires repeated completed job_radar_refresh jobs with no unresolved dead letters.",
            ),
        );

        let operational_blockers = gates
            .iter()
            .flat_map(|gate| {
                gate.missing_requirements
                    .iter()
                    .map(|missing| format!("{}: {missing}", gate.name))
            })
            .collect::<Vec<_>>();
        let decision = if operational_blockers.is_empty() {
            "pass"
        } else {
            "block"
        }
        .to_string();

        Ok(JobOperationalAudit {
            profile_id: profile_id.to_string(),
            scope,
            generated_at: now(),
            decision,
            proof_level: "local_operational_audit".to_string(),
            ops_summary,
            refresh_audit,
            outreach_readiness,
            source_family_counts,
            evidence_visibility_counts,
            packet_status_counts,
            intro_status_counts,
            weekly_report_count,
            weekly_delivery_status_counts,
            weekly_report_delivery_attempt_count,
            job_radar_job_status_counts,
            job_radar_completed_count,
            job_radar_dead_lettered_count,
            job_radar_attempt_count,
            gates,
            operational_blockers,
            warnings: vec![
                "This audit reads existing durable state only; it does not fetch sources, send messages, submit applications, or advance cursors.".to_string(),
                "A pass would mean the local operational proof packet shape is satisfied for this home, not exhaustive market coverage.".to_string(),
            ],
            non_claims: vec![
                "This audit is not a live source refresh.".to_string(),
                "This audit is not wall-clock recurrence proof by itself.".to_string(),
                "This audit is not provider delivery proof unless successful delivery-attempt rows are linked to weekly job reports.".to_string(),
                "This audit is not evidence of real user-network warm-intro agreement.".to_string(),
                "This audit is not application submission proof.".to_string(),
            ],
        })
    }

    pub fn run_job_source_refresh(
        &self,
        input: JobSourceRefreshInput,
    ) -> Result<JobSourceRefreshReport> {
        let input = normalize_job_source_refresh_input(input)?;
        let source = self
            .read_job_source(&input.source_id)?
            .with_context(|| format!("job source not found: {}", input.source_id))?;
        let mut warnings = Vec::new();
        let (body, fetched_url, proof_level) = if let Some(body) = input.body {
            warnings.push(
                "Job source refresh used caller-supplied page text/html; it is not a live fetch."
                    .to_string(),
            );
            (
                body,
                input.fetched_url.unwrap_or_else(|| source.url.clone()),
                "manual_snapshot",
            )
        } else if input.fetch_live {
            let fetch_result = (|| -> Result<String> {
                let url = validate_fetch_url(&source.url)?;
                self.guard_provider_network_policy(
                    "arcwell-job-hunting",
                    "web",
                    "job_source_refresh",
                    url.as_str(),
                    estimated_network_fetch_cost(1),
                    json!({ "source_id": source.id, "source_family": source.source_family }),
                )?;
                fetch_text(url.as_str(), None)
            })();
            match fetch_result {
                Ok(body) => (body, source.url.clone(), "live_fetch"),
                Err(error) => {
                    let health = self.record_job_source_health(JobSourceHealthInput {
                        source_id: source.id.clone(),
                        status: "failed".to_string(),
                        http_status: None,
                        error_code: Some(job_source_refresh_error_code(&error.to_string())),
                        fetched_count: 0,
                        accepted_count: 0,
                        rejected_count: 0,
                        note: Some(error.to_string()),
                    })?;
                    return Ok(JobSourceRefreshReport {
                        source,
                        source_health: health,
                        roles: Vec::new(),
                        companies: Vec::new(),
                        role_source_links: Vec::new(),
                        stale_role_events: Vec::new(),
                        fetched_count: 0,
                        accepted_count: 0,
                        rejected_count: 0,
                        warnings: vec![
                            "Live job source refresh failed before any role/company writes."
                                .to_string(),
                        ],
                    });
                }
            }
        } else {
            bail!("job source refresh requires body or fetch_live=true");
        };

        let parsed = parse_job_source_refresh_body(&source, &body, &fetched_url, proof_level)?;
        warnings.extend(parsed.warnings.clone());

        let mut companies = Vec::new();
        for company in parsed.companies {
            companies.push(self.record_job_company_card(company)?);
        }

        let mut roles = Vec::new();
        let mut role_source_links = Vec::new();
        let mut accepted_role_keys = BTreeSet::new();
        for role_input in parsed.roles {
            let source_url = role_input.source_url.clone();
            let role = self.record_job_role_card(role_input)?;
            accepted_role_keys.insert(job_role_refresh_key(
                &role.company,
                &role.role_title,
                &source_url,
            ));
            role_source_links.push(self.record_job_role_source_link(JobRoleSourceLinkInput {
                role_id: role.id.clone(),
                source_id: Some(source.id.clone()),
                source_url,
                confidence: role.source_confidence.clone(),
                evidence_excerpt: Some("Observed during job source refresh.".to_string()),
            })?);
            roles.push(role);
        }

        let existing_roles = self.list_job_roles_for_source(&source.id)?;
        for existing in &existing_roles {
            let key = job_role_refresh_key(
                &existing.company,
                &existing.role_title,
                &existing.source_url,
            );
            if accepted_role_keys.contains(&key)
                || !job_source_refresh_directly_confirms_existing_role(
                    &source,
                    &fetched_url,
                    &parsed.readable_text,
                    existing,
                    parsed.direct_role_title.as_deref(),
                )
            {
                continue;
            }
            accepted_role_keys.insert(key);
            role_source_links.push(self.record_job_role_source_link(JobRoleSourceLinkInput {
                role_id: existing.id.clone(),
                source_id: Some(source.id.clone()),
                source_url: existing.source_url.clone(),
                confidence: existing.source_confidence.clone(),
                evidence_excerpt: Some(
                    "Direct linked role source remained reachable during refresh.".to_string(),
                ),
            })?);
            roles.push(existing.clone());
        }

        let mut stale_role_events = Vec::new();
        for existing in existing_roles {
            let key = job_role_refresh_key(
                &existing.company,
                &existing.role_title,
                &existing.source_url,
            );
            if accepted_role_keys.contains(&key) || existing.current_status != "live" {
                continue;
            }
            self.update_job_role_current_status(&existing.id, "stale")?;
            stale_role_events.push(
                self.record_job_role_status_event(JobRoleStatusEventInput {
                    role_id: existing.id,
                    run_id: None,
                    status: "stale".to_string(),
                    previous_tier: None,
                    current_tier: Some("blocked".to_string()),
                    note: Some(
                        "Job source refresh did not observe this previously linked live role."
                            .to_string(),
                    ),
                })?,
            );
        }

        let accepted_count = roles.len() + companies.len();
        let status = job_source_refresh_health_status(
            accepted_count,
            parsed.rejected_count,
            parsed.no_openings_signal,
            stale_role_events.len(),
        );
        let health = self.record_job_source_health(JobSourceHealthInput {
            source_id: source.id.clone(),
            status,
            http_status: None,
            error_code: None,
            fetched_count: parsed.fetched_count,
            accepted_count,
            rejected_count: parsed.rejected_count,
            note: Some(job_source_refresh_health_note(
                proof_level,
                roles.len(),
                companies.len(),
                stale_role_events.len(),
                parsed.no_openings_signal,
            )),
        })?;

        Ok(JobSourceRefreshReport {
            source,
            source_health: health,
            roles,
            companies,
            role_source_links,
            stale_role_events,
            fetched_count: parsed.fetched_count,
            accepted_count,
            rejected_count: parsed.rejected_count,
            warnings,
        })
    }

    pub(crate) fn require_job_profile(&self, profile_id: &str) -> Result<JobCandidateProfile> {
        self.read_job_candidate_profile(profile_id)?
            .with_context(|| format!("job candidate profile not found: {profile_id}"))
    }

    pub(crate) fn validate_job_source_ids(&self, source_ids: Vec<String>) -> Result<Vec<String>> {
        let source_ids = normalize_job_id_list(source_ids, "job source id")?;
        if source_ids.is_empty() {
            bail!("job radar refresh requires at least one source id");
        }
        if source_ids.len() > 50 {
            bail!("job radar refresh has too many source ids");
        }
        for source_id in &source_ids {
            self.read_job_source(source_id)?
                .with_context(|| format!("job source not found: {source_id}"))?;
        }
        Ok(source_ids)
    }

    pub(crate) fn validate_job_evidence_card_ids(
        &self,
        evidence_card_ids: &[String],
        profile_id: Option<&str>,
        allow_private_blocked: bool,
    ) -> Result<Vec<JobEvidenceCard>> {
        if evidence_card_ids.len() > 100 {
            bail!("too many job evidence cards");
        }
        let mut seen = BTreeSet::new();
        let mut cards = Vec::new();
        for evidence_card_id in evidence_card_ids {
            validate_id(evidence_card_id)?;
            if !seen.insert(evidence_card_id.clone()) {
                continue;
            }
            let card = self
                .read_job_evidence_card(evidence_card_id)?
                .with_context(|| format!("job evidence card not found: {evidence_card_id}"))?;
            if let Some(profile_id) = profile_id {
                if card.profile_id != profile_id {
                    bail!("job evidence card belongs to a different profile");
                }
            }
            if !allow_private_blocked && card.visibility == "private_blocked" {
                bail!("private-blocked job evidence cannot be used in role material");
            }
            cards.push(card);
        }
        Ok(cards)
    }

    pub(crate) fn list_job_evidence_claims_for_profile(
        &self,
        profile_id: &str,
    ) -> Result<Vec<JobEvidenceClaim>> {
        validate_id(profile_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT c.id, c.evidence_card_id, c.claim, c.claim_kind, c.proof_level, c.can_use_in_resume, c.can_use_in_outreach, c.can_use_in_interview, c.created_at
            FROM job_evidence_claims c
            JOIN job_evidence_cards e ON e.id = c.evidence_card_id
            WHERE e.profile_id = ?1
            ORDER BY c.created_at ASC
            "#,
        )?;
        rows(stmt.query_map(params![profile_id], job_evidence_claim_from_row)?)
    }

    pub(crate) fn evaluate_job_privacy_text(
        &self,
        text: &str,
        extra_blocked_terms: &[String],
    ) -> Result<Vec<JobPrivacyFinding>> {
        let mut findings = Vec::new();
        let haystack = text.to_ascii_lowercase();
        for rule in self.list_job_privacy_rules()? {
            if haystack.contains(&rule.pattern.to_ascii_lowercase()) {
                findings.push(JobPrivacyFinding {
                    rule_id: Some(rule.id),
                    pattern: rule.pattern,
                    rule_type: rule.rule_type,
                    severity: rule.severity,
                    replacement_guidance: rule.replacement_guidance,
                });
            }
        }
        for term in extra_blocked_terms {
            let term = term.trim();
            if term.is_empty() {
                continue;
            }
            if haystack.contains(&term.to_ascii_lowercase()) {
                findings.push(JobPrivacyFinding {
                    rule_id: None,
                    pattern: term.to_string(),
                    rule_type: "blocked_term".to_string(),
                    severity: "block".to_string(),
                    replacement_guidance: Some(
                        "Remove this private term before using the material.".to_string(),
                    ),
                });
            }
        }
        Ok(findings)
    }

    pub(crate) fn record_job_privacy_check_result(
        &self,
        artifact_type: &str,
        artifact_id: Option<&str>,
        decision: &str,
        findings: Vec<JobPrivacyFinding>,
        checked_text: &str,
    ) -> Result<JobPrivacyCheck> {
        let id = job_privacy_check_id();
        let checked_at = now();
        self.conn.execute(
            r#"
            INSERT INTO job_privacy_checks
              (id, artifact_type, artifact_id, checked_at, decision, findings_json, checked_text_hash)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                id,
                artifact_type,
                artifact_id,
                checked_at,
                decision,
                serde_json::to_string(&findings)?,
                sha256(checked_text.as_bytes()),
            ],
        )?;
        self.read_job_privacy_check(&id)?
            .with_context(|| format!("job privacy check not found: {id}"))
    }

    pub(crate) fn read_job_privacy_check(&self, id: &str) -> Result<Option<JobPrivacyCheck>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, artifact_type, artifact_id, checked_at, decision, findings_json, checked_text_hash
                FROM job_privacy_checks
                WHERE id = ?1
                "#,
                params![id],
                job_privacy_check_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub(crate) fn latest_job_fit_score(
        &self,
        role_id: &str,
        profile_id: &str,
    ) -> Result<Option<JobFitScore>> {
        validate_id(role_id)?;
        validate_id(profile_id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, role_id, profile_id, scored_at, scorer, role_fit, domain_fit, evidence_fit, geo_work_fit, stage_fit, practical_odds, interest_energy, weighted_score, tier, blockers_json, evidence_card_ids_json, explanation
                FROM job_fit_scores
                WHERE role_id = ?1 AND profile_id = ?2
                ORDER BY scored_at DESC
                LIMIT 1
                "#,
                params![role_id, profile_id],
                job_fit_score_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub(crate) fn list_job_applications(&self) -> Result<Vec<JobApplication>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, role_id, packet_id, status, applied_at, follow_up_at, outcome_note, created_at, updated_at
            FROM job_applications
            ORDER BY updated_at DESC
            "#,
        )?;
        rows(stmt.query_map([], job_application_from_row)?)
    }

    pub(crate) fn list_job_source_health_recent(
        &self,
        limit: usize,
    ) -> Result<Vec<JobSourceHealth>> {
        let limit = limit.clamp(1, 500) as i64;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, source_id, checked_at, status, http_status, error_code, fetched_count, accepted_count, rejected_count, note
            FROM job_source_health
            ORDER BY checked_at DESC
            LIMIT ?1
            "#,
        )?;
        rows(stmt.query_map(params![limit], job_source_health_from_row)?)
    }

    pub(crate) fn read_job_source_health_ids(
        &self,
        ids: &[String],
    ) -> Result<Vec<JobSourceHealth>> {
        let mut seen = BTreeSet::new();
        let mut out = Vec::new();
        for id in ids {
            validate_id(id)?;
            if !seen.insert(id.clone()) {
                continue;
            }
            out.push(
                self.read_job_source_health(id)?
                    .with_context(|| format!("job source health not found: {id}"))?,
            );
        }
        Ok(out)
    }

    pub fn job_ops_summary(&self) -> Result<JobOpsSummary> {
        Ok(JobOpsSummary {
            profile_count: self.count_job_rows("job_candidate_profiles")?,
            evidence_card_count: self.count_job_rows("job_evidence_cards")?,
            source_count: self.count_job_rows("job_sources")?,
            role_count: self.count_job_rows("job_role_cards")?,
            role_status_counts: self.count_job_group("job_role_cards", "current_status")?,
            score_tier_counts: self.count_job_group("job_fit_scores", "tier")?,
            source_health_counts: self.count_job_group("job_source_health", "status")?,
            privacy_decision_counts: self.count_job_group("job_privacy_checks", "decision")?,
            application_status_counts: self.count_job_group("job_applications", "status")?,
            follow_up_count: self.count_job_follow_ups()?,
            stale_or_closed_roles: self.list_job_stale_or_closed_roles(25)?,
            source_health_failures: self.list_job_source_health_failures(25)?,
        })
    }

    pub(crate) fn count_job_rows(&self, table: &str) -> Result<usize> {
        let sql = match table {
            "job_candidate_profiles" => "SELECT COUNT(*) FROM job_candidate_profiles",
            "job_evidence_cards" => "SELECT COUNT(*) FROM job_evidence_cards",
            "job_sources" => "SELECT COUNT(*) FROM job_sources",
            "job_role_cards" => "SELECT COUNT(*) FROM job_role_cards",
            "job_search_runs" => "SELECT COUNT(*) FROM job_search_runs",
            "job_application_packets" => "SELECT COUNT(*) FROM job_application_packets",
            "job_contacts" => "SELECT COUNT(*) FROM job_contacts",
            "job_intro_paths" => "SELECT COUNT(*) FROM job_intro_paths",
            "job_weekly_reports" => "SELECT COUNT(*) FROM job_weekly_reports",
            "job_weekly_report_deliveries" => "SELECT COUNT(*) FROM job_weekly_report_deliveries",
            other => bail!("unsupported job ops count table: {other}"),
        };
        let count: i64 = self.conn.query_row(sql, [], |row| row.get(0))?;
        Ok(count as usize)
    }

    pub(crate) fn count_job_group(
        &self,
        table: &str,
        column: &str,
    ) -> Result<BTreeMap<String, usize>> {
        let sql = match (table, column) {
            ("job_role_cards", "current_status") => {
                "SELECT current_status, COUNT(*) FROM job_role_cards GROUP BY current_status"
            }
            ("job_sources", "source_family") => {
                "SELECT source_family, COUNT(*) FROM job_sources GROUP BY source_family"
            }
            ("job_evidence_cards", "visibility") => {
                "SELECT visibility, COUNT(*) FROM job_evidence_cards GROUP BY visibility"
            }
            ("job_fit_scores", "tier") => "SELECT tier, COUNT(*) FROM job_fit_scores GROUP BY tier",
            ("job_application_packets", "status") => {
                "SELECT status, COUNT(*) FROM job_application_packets GROUP BY status"
            }
            ("job_intro_paths", "status") => {
                "SELECT status, COUNT(*) FROM job_intro_paths GROUP BY status"
            }
            ("job_source_health", "status") => {
                "SELECT status, COUNT(*) FROM job_source_health GROUP BY status"
            }
            ("job_privacy_checks", "decision") => {
                "SELECT decision, COUNT(*) FROM job_privacy_checks GROUP BY decision"
            }
            ("job_applications", "status") => {
                "SELECT status, COUNT(*) FROM job_applications GROUP BY status"
            }
            other => bail!("unsupported job ops group: {other:?}"),
        };
        let mut stmt = self.conn.prepare(sql)?;
        let pairs = rows(stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
        })?)?;
        Ok(pairs.into_iter().collect())
    }

    pub(crate) fn count_job_weekly_reports_for_scope(
        &self,
        profile_id: &str,
        scope: &str,
    ) -> Result<usize> {
        validate_id(profile_id)?;
        let count: i64 = self.conn.query_row(
            r#"
            SELECT COUNT(*)
            FROM job_weekly_reports
            WHERE profile_id = ?1 AND scope = ?2
            "#,
            params![profile_id, scope],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    pub(crate) fn count_job_weekly_report_delivery_status_for_scope(
        &self,
        profile_id: &str,
        scope: &str,
    ) -> Result<BTreeMap<String, usize>> {
        validate_id(profile_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT d.status, COUNT(*)
            FROM job_weekly_report_deliveries d
            JOIN job_weekly_reports r ON r.id = d.report_id
            WHERE r.profile_id = ?1 AND r.scope = ?2
            GROUP BY d.status
            "#,
        )?;
        let pairs = rows(stmt.query_map(params![profile_id, scope], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
        })?)?;
        Ok(pairs.into_iter().collect())
    }

    pub(crate) fn count_job_weekly_report_delivery_attempts_for_scope(
        &self,
        profile_id: &str,
        scope: &str,
    ) -> Result<usize> {
        validate_id(profile_id)?;
        let count: i64 = self.conn.query_row(
            r#"
            SELECT COUNT(*)
            FROM job_weekly_report_deliveries d
            JOIN job_weekly_reports r ON r.id = d.report_id
            JOIN channel_delivery_attempts a ON a.message_id = d.channel_message_id
            WHERE r.profile_id = ?1 AND r.scope = ?2
              AND a.ok = 1
            "#,
            params![profile_id, scope],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    pub(crate) fn count_job_evidence_visibility_for_profile(
        &self,
        profile_id: &str,
    ) -> Result<BTreeMap<String, usize>> {
        validate_id(profile_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT visibility, COUNT(*)
            FROM job_evidence_cards
            WHERE profile_id = ?1
            GROUP BY visibility
            "#,
        )?;
        let pairs = rows(stmt.query_map(params![profile_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
        })?)?;
        Ok(pairs.into_iter().collect())
    }

    pub(crate) fn count_job_role_status_for_profile(
        &self,
        profile_id: &str,
    ) -> Result<BTreeMap<String, usize>> {
        validate_id(profile_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT r.current_status, COUNT(DISTINCT r.id)
            FROM job_role_cards r
            JOIN job_fit_scores s ON s.role_id = r.id
            WHERE s.profile_id = ?1
            GROUP BY r.current_status
            "#,
        )?;
        let pairs = rows(stmt.query_map(params![profile_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
        })?)?;
        Ok(pairs.into_iter().collect())
    }

    pub(crate) fn count_job_fit_score_tiers_for_profile(
        &self,
        profile_id: &str,
    ) -> Result<BTreeMap<String, usize>> {
        validate_id(profile_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT tier, COUNT(*)
            FROM job_fit_scores
            WHERE profile_id = ?1
            GROUP BY tier
            "#,
        )?;
        let pairs = rows(stmt.query_map(params![profile_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
        })?)?;
        Ok(pairs.into_iter().collect())
    }

    pub(crate) fn count_job_application_packet_status_for_profile(
        &self,
        profile_id: &str,
    ) -> Result<BTreeMap<String, usize>> {
        validate_id(profile_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT status, COUNT(*)
            FROM job_application_packets
            WHERE profile_id = ?1
            GROUP BY status
            "#,
        )?;
        let pairs = rows(stmt.query_map(params![profile_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
        })?)?;
        Ok(pairs.into_iter().collect())
    }

    pub(crate) fn count_job_intro_path_status_for_profile(
        &self,
        profile_id: &str,
    ) -> Result<BTreeMap<String, usize>> {
        validate_id(profile_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT ip.status, COUNT(DISTINCT ip.id)
            FROM job_intro_paths ip
            WHERE ip.role_id IN (
              SELECT role_id FROM job_fit_scores WHERE profile_id = ?1
            )
            GROUP BY ip.status
            "#,
        )?;
        let pairs = rows(stmt.query_map(params![profile_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
        })?)?;
        Ok(pairs.into_iter().collect())
    }

    pub(crate) fn count_job_privacy_decisions_for_profile_scope(
        &self,
        profile_id: &str,
        scope: &str,
    ) -> Result<BTreeMap<String, usize>> {
        validate_id(profile_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT decision, COUNT(*)
            FROM job_privacy_checks
            WHERE artifact_id IN (
              SELECT id FROM job_application_packets WHERE profile_id = ?1
              UNION
              SELECT id FROM job_weekly_reports WHERE profile_id = ?1 AND scope = ?2
            )
            GROUP BY decision
            "#,
        )?;
        let pairs = rows(stmt.query_map(params![profile_id, scope], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
        })?)?;
        Ok(pairs.into_iter().collect())
    }

    pub(crate) fn count_job_application_status_for_profile(
        &self,
        profile_id: &str,
    ) -> Result<BTreeMap<String, usize>> {
        validate_id(profile_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT status, COUNT(*)
            FROM job_applications
            WHERE role_id IN (
              SELECT role_id FROM job_fit_scores WHERE profile_id = ?1
              UNION
              SELECT role_id FROM job_application_packets WHERE profile_id = ?1
            )
              OR packet_id IN (
                SELECT id FROM job_application_packets WHERE profile_id = ?1
              )
            GROUP BY status
            "#,
        )?;
        let pairs = rows(stmt.query_map(params![profile_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
        })?)?;
        Ok(pairs.into_iter().collect())
    }

    pub(crate) fn count_job_follow_ups_for_profile(&self, profile_id: &str) -> Result<usize> {
        validate_id(profile_id)?;
        let count: i64 = self.conn.query_row(
            r#"
            SELECT COUNT(*)
            FROM job_applications
            WHERE (
              role_id IN (
                SELECT role_id FROM job_fit_scores WHERE profile_id = ?1
                UNION
                SELECT role_id FROM job_application_packets WHERE profile_id = ?1
              )
              OR packet_id IN (
                SELECT id FROM job_application_packets WHERE profile_id = ?1
              )
            )
              AND follow_up_at IS NOT NULL
              AND status NOT IN ('rejected', 'offer', 'withdrawn')
            "#,
            params![profile_id],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    pub(crate) fn count_job_follow_ups(&self) -> Result<usize> {
        let count: i64 = self.conn.query_row(
            r#"
            SELECT COUNT(*)
            FROM job_applications
            WHERE follow_up_at IS NOT NULL
              AND status NOT IN ('rejected', 'offer', 'withdrawn')
            "#,
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    pub(crate) fn list_job_stale_or_closed_roles(&self, limit: usize) -> Result<Vec<JobRoleCard>> {
        let limit = limit.clamp(1, 100) as i64;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, company, role_title, canonical_url, source_family, source_url, source_confidence, date_accessed, posting_freshness, location, work_mode, company_stage_or_size, role_seniority, core_requirements_json, implied_business_problem, why_they_might_need_user, evidence_card_ids_json, gaps_or_blockers_json, cluster, current_status, metadata_json, created_at, updated_at
            FROM job_role_cards
            WHERE current_status != 'live'
            ORDER BY updated_at DESC
            LIMIT ?1
            "#,
        )?;
        rows(stmt.query_map(params![limit], job_role_card_from_row)?)
    }

    pub(crate) fn list_job_source_health_failures(
        &self,
        limit: usize,
    ) -> Result<Vec<JobSourceHealth>> {
        let limit = limit.clamp(1, 100) as i64;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, source_id, checked_at, status, http_status, error_code, fetched_count, accepted_count, rejected_count, note
            FROM job_source_health
            WHERE status != 'healthy'
            ORDER BY checked_at DESC
            LIMIT ?1
            "#,
        )?;
        rows(stmt.query_map(params![limit], job_source_health_from_row)?)
    }

    pub(crate) fn latest_job_role_status_event(
        &self,
        role_id: &str,
    ) -> Result<Option<JobRoleStatusEvent>> {
        validate_id(role_id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, role_id, run_id, status, previous_tier, current_tier, note, created_at
                FROM job_role_status_events
                WHERE role_id = ?1
                ORDER BY created_at DESC
                LIMIT 1
                "#,
                params![role_id],
                job_role_status_event_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub(crate) fn update_job_role_current_status(
        &self,
        role_id: &str,
        status: &str,
    ) -> Result<JobRoleCard> {
        validate_id(role_id)?;
        let status = normalize_job_role_status(status)?;
        let changed = self.conn.execute(
            r#"
            UPDATE job_role_cards
            SET current_status = ?2, updated_at = ?3
            WHERE id = ?1
            "#,
            params![role_id, status, now()],
        )?;
        if changed == 0 {
            bail!("job role card not found: {role_id}");
        }
        self.read_job_role_card(role_id)?
            .with_context(|| format!("job role card not found: {role_id}"))
    }
}

fn gate_missing(condition: bool, missing: &str) -> Vec<String> {
    if condition {
        Vec::new()
    } else {
        vec![missing.to_string()]
    }
}

fn push_job_operational_gate(
    gates: &mut Vec<JobOperationalAuditGate>,
    name: &str,
    evidence: Value,
    missing_requirements: Vec<String>,
) {
    gates.push(JobOperationalAuditGate {
        name: name.to_string(),
        decision: if missing_requirements.is_empty() {
            "pass"
        } else {
            "block"
        }
        .to_string(),
        evidence,
        missing_requirements,
    });
}
