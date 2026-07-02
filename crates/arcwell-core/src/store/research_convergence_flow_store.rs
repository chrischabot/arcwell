use super::*;

impl Store {
    pub fn run_research_convergence_provider_search(
        &self,
        input: ResearchConvergenceProviderSearchInput,
    ) -> Result<ResearchConvergenceProviderSearchResult> {
        let input = normalize_research_convergence_provider_search_input(input)?;
        self.require_research_run(&input.run_id)?;
        let provider = input.provider.clone();
        let max_results = input.max_results.unwrap_or(5).clamp(1, 20);
        let max_tasks = input.max_tasks.unwrap_or(8).clamp(1, 50);
        let max_provider_calls = input.max_provider_calls.unwrap_or(max_tasks).clamp(1, 50);
        let enqueue_selected_url_ingest = input.enqueue_selected_url_ingest.unwrap_or(false);
        let mut remaining_ingest_jobs = input.max_ingest_jobs.unwrap_or(0).min(100);
        let call_limit = max_tasks.min(max_provider_calls);
        let pending_tasks = self
            .list_research_convergence_host_search_tasks(&input.run_id)?
            .into_iter()
            .filter(|task| task.status == "pending")
            .take(call_limit)
            .collect::<Vec<_>>();
        let projected_cost_usd =
            estimated_web_search_cost(max_results) * pending_tasks.len() as f64;
        if let Some(cap) = input.cost_cap_usd
            && projected_cost_usd > cap
        {
            bail!(
                "convergence provider search projected cost ${projected_cost_usd:.4} exceeds cap ${cap:.4}"
            );
        }
        let mut attempted = Vec::new();
        let mut ingest_jobs = Vec::new();
        let mut stopped_reason = None;
        for task in pending_tasks {
            let config = WebSearchConfig {
                provider: provider.clone(),
                max_results,
                endpoint: input.endpoint.clone(),
                api_key: input.api_key.clone(),
                model: input.model.clone(),
                timeout_seconds: input.timeout_seconds.unwrap_or(15),
            };
            let search = self.web_search_with_cost_decision(&task.query, config);
            match search {
                Ok((response, cost_decision)) => {
                    let result_inputs = response
                        .results
                        .iter()
                        .filter_map(|result| {
                            web_search_result_to_host_search_input(
                                result,
                                research_task_primary_source_family(&task).as_deref(),
                                &response.warnings,
                            )
                        })
                        .collect::<Vec<_>>();
                    if result_inputs.is_empty() {
                        let message = "provider returned no safe public URLs for convergence task";
                        self.record_convergence_provider_search_blocked(
                            &task,
                            &provider,
                            cost_decision
                                .as_ref()
                                .and_then(|decision| decision.decision_id.clone()),
                            message,
                        )?;
                        attempted.push(ResearchConvergenceProviderSearchAttempt {
                            task,
                            status: "blocked".to_string(),
                            host_search_id: None,
                            cost_decision_id: cost_decision
                                .and_then(|decision| decision.decision_id),
                            result_count: response.results.len(),
                            selected_result_count: 0,
                            ingest_job_ids: Vec::new(),
                            error_message_redacted: Some(message.to_string()),
                        });
                        stopped_reason = Some("provider_returned_no_safe_urls".to_string());
                        break;
                    }
                    let cost_decision_id = cost_decision
                        .as_ref()
                        .and_then(|decision| decision.decision_id.clone());
                    let record = self.record_research_host_search(ResearchHostSearchInput {
                        run_id: input.run_id.clone(),
                        role_run_id: None,
                        host: "arcwell-provider".to_string(),
                        tool_surface: format!("research_web_search:{provider}"),
                        query: task.query.clone(),
                        query_intent: Some(format!(
                            "Provider fallback for convergence challenge {}",
                            task.challenge_id
                        )),
                        requested_recency: None,
                        requested_domains: Vec::new(),
                        cost_decision_id: cost_decision_id.clone(),
                        results: result_inputs,
                    })?;
                    let mut ingest_job_ids = Vec::new();
                    if enqueue_selected_url_ingest && remaining_ingest_jobs > 0 {
                        for result in record
                            .results
                            .iter()
                            .filter(|result| result.selected_for_ingest)
                        {
                            if remaining_ingest_jobs == 0 {
                                break;
                            }
                            validate_public_http_url(&result.url)?;
                            let job = self.enqueue_wiki_job(
                                "ingest_url",
                                json!({
                                    "url": result.url,
                                    "research_run_id": input.run_id,
                                    "host_search_id": record.search.id,
                                    "host_search_result_id": result.id,
                                    "source": "research_convergence_provider_search"
                                }),
                            )?;
                            remaining_ingest_jobs -= 1;
                            ingest_job_ids.push(job.id.clone());
                            ingest_jobs.push(job);
                        }
                    }
                    let selected_result_count = record
                        .results
                        .iter()
                        .filter(|result| result.selected_for_ingest)
                        .count();
                    attempted.push(ResearchConvergenceProviderSearchAttempt {
                        task,
                        status: "recorded".to_string(),
                        host_search_id: Some(record.search.id),
                        cost_decision_id,
                        result_count: record.results.len(),
                        selected_result_count,
                        ingest_job_ids,
                        error_message_redacted: None,
                    });
                }
                Err(error) => {
                    let message = redact_secret_like_text(&error.to_string());
                    self.record_convergence_provider_search_blocked(
                        &task, &provider, None, &message,
                    )?;
                    attempted.push(ResearchConvergenceProviderSearchAttempt {
                        task,
                        status: "blocked".to_string(),
                        host_search_id: None,
                        cost_decision_id: None,
                        result_count: 0,
                        selected_result_count: 0,
                        ingest_job_ids: Vec::new(),
                        error_message_redacted: Some(message.clone()),
                    });
                    stopped_reason = Some("provider_search_failed".to_string());
                    break;
                }
            }
        }
        let remaining_tasks = self
            .list_research_convergence_host_search_tasks(&input.run_id)?
            .into_iter()
            .filter(|task| task.status == "pending")
            .collect::<Vec<_>>();
        let provider_call_count = attempted.len();
        Ok(ResearchConvergenceProviderSearchResult {
            run_id: input.run_id,
            provider,
            attempted,
            remaining_tasks,
            provider_call_count,
            ingest_jobs,
            projected_cost_usd,
            stopped_reason,
        })
    }

    pub(crate) fn record_convergence_provider_search_blocked(
        &self,
        task: &ResearchConvergenceHostSearchTask,
        provider: &str,
        cost_decision_id: Option<String>,
        error_message: &str,
    ) -> Result<ResearchArtifact> {
        self.record_research_artifact(ResearchArtifactInput {
            run_id: task.run_id.clone(),
            role_run_id: None,
            artifact_type: "convergence_provider_search_blocked".to_string(),
            title: format!("Blocked convergence provider search: {}", task.query),
            body: format!(
                "Provider `{provider}` could not complete convergence host-search task `{}`.\n\n{}",
                task.id,
                redact_secret_like_text(error_message)
            ),
            metadata: json!({
                "artifact_role": "convergence_provider_search_blocked",
                "task_id": task.id,
                "challenge_id": task.challenge_id,
                "statement_id": task.statement_id,
                "provider": provider,
                "query": task.query,
                "cost_decision_id": cost_decision_id,
            }),
        })
    }

    pub fn list_research_disproofs(&self, run_id: &str) -> Result<Vec<ResearchDisproof>> {
        self.require_research_run(run_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, run_id, iteration_id, challenge_id, statement_id, verdict, strength,
                   evidence_json, reasoning_summary, confidence_delta, requires_revision,
                   created_by_role, created_at
            FROM research_disproofs
            WHERE run_id = ?1
            ORDER BY created_at ASC
            "#,
        )?;
        rows(stmt.query_map(params![run_id], research_disproof_from_row)?)
    }

    pub fn list_research_revisions(&self, run_id: &str) -> Result<Vec<ResearchRevision>> {
        self.require_research_run(run_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, run_id, iteration_id, from_statement_id, to_statement_id, revision_type,
                   rationale, trigger_disproof_ids_json, evidence_delta_json, created_at
            FROM research_revisions
            WHERE run_id = ?1
            ORDER BY created_at ASC
            "#,
        )?;
        rows(stmt.query_map(params![run_id], research_revision_from_row)?)
    }

    pub fn list_research_fact_checks(&self, run_id: &str) -> Result<Vec<ResearchFactCheck>> {
        self.require_research_run(run_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, run_id, iteration_id, statement_id, label, impact, evidence_json, notes, created_at
            FROM research_fact_checks
            WHERE run_id = ?1
            ORDER BY created_at ASC
            "#,
        )?;
        rows(stmt.query_map(params![run_id], research_fact_check_from_row)?)
    }

    pub fn run_research_active_fact_check(
        &self,
        input: ResearchActiveFactCheckInput,
    ) -> Result<ResearchActiveFactCheckResult> {
        validate_id(&input.run_id)?;
        self.require_research_run(&input.run_id)?;
        let latest_iteration = self
            .research_convergence_status(&input.run_id)?
            .latest_iteration
            .context("active fact-check requires at least one convergence iteration")?;
        let artifact = match input.artifact_id.as_deref() {
            Some(id) => {
                validate_id(id)?;
                self.read_research_artifact(id)?
                    .with_context(|| format!("research artifact not found: {id}"))?
            }
            None => self
                .list_research_artifacts(&input.run_id)?
                .into_iter()
                .rev()
                .find(|artifact| {
                    matches!(
                        artifact.artifact_type.as_str(),
                        "convergence_report"
                            | "generated_synthesis"
                            | "citation_verified_draft"
                            | "deep_research_report"
                    )
                })
                .context("active fact-check requires a report or generated synthesis artifact")?,
        };
        if artifact.run_id != input.run_id {
            bail!("active fact-check artifact belongs to a different research run");
        }
        let max_sentences = input.max_sentences.unwrap_or(40).clamp(1, 200);
        let create_challenges = input.create_challenges.unwrap_or(true);
        let existing_statements = self.list_research_statements(&input.run_id)?;
        let claim_records = self.list_research_claims(&input.run_id)?;
        let run_sources = self.list_research_run_sources(&input.run_id)?;
        let sentences = active_fact_check_sentences(&artifact.body, max_sentences);
        let mut checks = Vec::new();
        let mut challenges = Vec::new();
        let mut matched_existing_statements = 0usize;
        let mut created_statement_count = 0usize;
        let mut support_metadata = json!({
            "claim_ids": [],
            "source_card_ids": [],
            "matched_claim_ids": [],
            "acceptable_source_card_ids": [],
            "missing_claim_ids": [],
            "unacceptable_source_card_ids": [],
            "has_acceptable_evidence": false,
        });
        for sentence in sentences {
            let matched = existing_statements
                .iter()
                .filter(|statement| statement.created_by_role != "active_fact_checker")
                .find(|statement| active_fact_sentence_matches_statement(&sentence, statement));
            let prompt_injection_instruction =
                active_fact_sentence_is_prompt_injection_instruction(&sentence);
            let not_checkable =
                prompt_injection_instruction || active_fact_sentence_is_not_checkable(&sentence);
            let (statement, label, notes) = if let Some(statement) = matched {
                matched_existing_statements += 1;
                let support =
                    research_statement_fact_support(statement, &claim_records, &run_sources);
                support_metadata = support.metadata_json();
                let (label, notes) = if statement.status == "refuted" {
                    (
                        "wrong",
                        format!(
                            "Report sentence matched refuted convergence statement `{}`.",
                            statement.id
                        ),
                    )
                } else if matches!(statement.status.as_str(), "weakened" | "unresolved") {
                    (
                        "unknown",
                        format!(
                            "Report sentence matched weakened or unresolved convergence statement `{}`.",
                            statement.id
                        ),
                    )
                } else if support.claim_ids.is_empty() {
                    (
                        "unknown",
                        format!(
                            "Report sentence matched statement `{}`, but that statement has no linked extracted claim evidence.",
                            statement.id
                        ),
                    )
                } else if !support.missing_claim_ids.is_empty() {
                    (
                        "unknown",
                        format!(
                            "Report sentence matched statement `{}`, but some linked claim ids are not present in this research run.",
                            statement.id
                        ),
                    )
                } else if support.acceptable_source_card_ids.is_empty() {
                    (
                        "unknown",
                        format!(
                            "Report sentence matched statement `{}`, but its evidence is not backed by acceptable run-linked source cards.",
                            statement.id
                        ),
                    )
                } else {
                    (
                        "right",
                        format!(
                            "Report sentence matched existing convergence statement `{}` with acceptable run-linked source evidence.",
                            statement.id
                        ),
                    )
                };
                (statement.clone(), label, notes)
            } else {
                created_statement_count += 1;
                support_metadata = json!({
                    "claim_ids": [],
                    "source_card_ids": [],
                    "matched_claim_ids": [],
                    "acceptable_source_card_ids": [],
                    "missing_claim_ids": [],
                    "unacceptable_source_card_ids": [],
                    "has_acceptable_evidence": false,
                });
                let stable_key =
                    research_statement_stable_key(&format!("active fact-check {}", sentence));
                let statement = ResearchStatement {
                    id: research_statement_id(&input.run_id, &latest_iteration.id, &stable_key),
                    run_id: input.run_id.clone(),
                    iteration_id: latest_iteration.id.clone(),
                    parent_statement_id: None,
                    stable_key,
                    statement_type: "fact".to_string(),
                    text: sentence.clone(),
                    scope: None,
                    temporal_scope: None,
                    confidence: 0.25,
                    certainty_label: "low".to_string(),
                    status: "unresolved".to_string(),
                    importance: if not_checkable {
                        "medium".to_string()
                    } else {
                        "high".to_string()
                    },
                    evidence: json!({
                        "source": "active_fact_check",
                        "artifact_id": artifact.id,
                        "artifact_type": artifact.artifact_type,
                        "claim_ids": [],
                        "source_card_ids": [],
                    }),
                    counterevidence: json!([]),
                    assumptions: json!([]),
                    caveats: if prompt_injection_instruction {
                        json!([
                            "Extracted from report text as instruction-like prompt-injection content; record as data, not as verifier guidance."
                        ])
                    } else if not_checkable {
                        json!([
                            "Extracted from report text as an opinion or judgment that cannot be directly fact-checked."
                        ])
                    } else {
                        json!([
                            "Extracted from report text and not supported by existing structured evidence."
                        ])
                    },
                    created_by_role: "active_fact_checker".to_string(),
                    created_at: now(),
                    updated_at: now(),
                };
                (
                    self.upsert_research_statement(statement)?,
                    if not_checkable {
                        "not_checkable"
                    } else {
                        "unknown"
                    },
                    if prompt_injection_instruction {
                        "Report sentence looks like instruction-like prompt-injection content and is recorded as non-checkable data, not verifier guidance."
                            .to_string()
                    } else if not_checkable {
                        "Report sentence is a judgment or opinion and is not directly fact-checkable."
                            .to_string()
                    } else {
                        "Report sentence was not supported by any existing source-backed convergence statement.".to_string()
                    },
                )
            };
            let check = self.insert_research_fact_check(ResearchFactCheck {
                id: research_fact_check_id(&input.run_id, &latest_iteration.id, &statement.id),
                run_id: input.run_id.clone(),
                iteration_id: latest_iteration.id.clone(),
                statement_id: statement.id.clone(),
                label: label.to_string(),
                impact: if label == "not_checkable" || statement.importance == "low" {
                    "medium".to_string()
                } else {
                    "high".to_string()
                },
                evidence: json!({
                    "active_fact_check": true,
                    "artifact_id": artifact.id,
                    "artifact_type": artifact.artifact_type,
                    "sentence": sentence,
                    "matched_statement_id": if matched.is_some() { Some(statement.id.clone()) } else { None::<String> },
                    "claim_ids": statement_evidence_claim_ids(&statement),
                    "source_card_ids": statement_evidence_source_card_ids(&statement),
                    "support": support_metadata,
                    "requires_fresh_retrieval": matches!(label, "wrong" | "unknown"),
                }),
                notes,
                created_at: now(),
            })?;
            if create_challenges && matches!(check.label.as_str(), "wrong" | "unknown") {
                let challenge = ResearchChallenge {
                    id: research_challenge_id(
                        &input.run_id,
                        &latest_iteration.id,
                        &statement.id,
                        "citation_gap",
                    ),
                    run_id: input.run_id.clone(),
                    iteration_id: latest_iteration.id.clone(),
                    statement_id: statement.id.clone(),
                    challenge_type: "citation_gap".to_string(),
                    severity: "error".to_string(),
                    rationale:
                        "Active fact-check found a report sentence without source-backed support."
                            .to_string(),
                    would_change_answer_if_true: true,
                    search_plan: json!({
                        "queries": [statement.text.clone()],
                        "requires_host_search_proof": true,
                        "status": "active_fact_check_needs_fresh_retrieval",
                        "source_artifact_id": artifact.id
                    }),
                    required_source_families: json!([
                        "primary",
                        "official",
                        "paper",
                        "benchmark",
                        "technical"
                    ]),
                    status: "open".to_string(),
                    created_by_role: "active_fact_checker".to_string(),
                    created_at: now(),
                    updated_at: now(),
                };
                challenges.push(self.upsert_research_challenge(challenge)?);
            }
            checks.push(check);
        }
        Ok(ResearchActiveFactCheckResult {
            run_id: input.run_id,
            artifact_id: artifact.id,
            checked_sentences: checks.len(),
            matched_existing_statements,
            created_statement_count,
            created_challenge_count: challenges.len(),
            checks,
            challenges,
        })
    }

    pub fn run_research_convergence_close_loop(
        &self,
        input: ResearchConvergenceCloseLoopInput,
    ) -> Result<ResearchConvergenceCloseLoopResult> {
        validate_id(&input.run_id)?;
        self.require_research_run(&input.run_id)?;
        if input.no_write.unwrap_or(false) {
            bail!(
                "research_convergence_close_loop writes fact-check, challenge, iteration, and report artifacts; no_write=true is not supported"
            );
        }
        if let Some(max_sentences) = input.max_sentences
            && (max_sentences == 0 || max_sentences > 200)
        {
            bail!("max_sentences must be between 1 and 200");
        }

        let convergence_input = research_close_loop_convergence_input(&input);
        let convergence_config = normalize_research_convergence_config(&convergence_input)?;
        let mut initial_status = self.research_convergence_status(&input.run_id)?;
        if initial_status.latest_iteration.is_none()
            || (!initial_status.settled
                && initial_status
                    .stop_reason
                    .as_deref()
                    .is_none_or(|reason| reason == "continue"))
        {
            let step = self.run_research_convergence_to_stop(convergence_input.clone())?;
            initial_status = step.status;
        }

        let checked_artifact_id = match input.artifact_id.clone() {
            Some(artifact_id) => {
                validate_id(&artifact_id)?;
                artifact_id
            }
            None if input.compile_report_before_check.unwrap_or(true) => self
                .compile_research_convergence_report(&input.run_id)?
                .artifact
                .id,
            None => self
                .list_research_artifacts(&input.run_id)?
                .into_iter()
                .rev()
                .find(|artifact| {
                    matches!(
                        artifact.artifact_type.as_str(),
                        "convergence_report"
                            | "generated_synthesis"
                            | "citation_verified_draft"
                            | "deep_research_report"
                    )
                })
                .context("close-loop active fact-check requires an artifact or compile_report_before_check=true")?
                .id,
        };

        let active_fact_check =
            self.run_research_active_fact_check(ResearchActiveFactCheckInput {
                run_id: input.run_id.clone(),
                artifact_id: Some(checked_artifact_id.clone()),
                max_sentences: input.max_sentences,
                create_challenges: input.create_challenges,
            })?;
        let after_active_fact_check_status = self.research_convergence_status(&input.run_id)?;

        let provider = input
            .provider
            .as_deref()
            .map(str::trim)
            .filter(|provider| !provider.is_empty())
            .map(ToOwned::to_owned);
        let provider_search = if let Some(provider) = provider {
            if after_active_fact_check_status
                .host_search_tasks
                .iter()
                .any(|task| task.status == "pending")
            {
                Some(self.run_research_convergence_provider_search(
                    ResearchConvergenceProviderSearchInput {
                        run_id: input.run_id.clone(),
                        provider,
                        max_tasks: input.provider_max_tasks,
                        max_results: input.provider_max_results,
                        max_provider_calls: input.provider_max_provider_calls,
                        enqueue_selected_url_ingest: input.enqueue_selected_url_ingest,
                        max_ingest_jobs: input.max_ingest_jobs,
                        cost_cap_usd: input.provider_cost_cap_usd,
                        endpoint: input.provider_endpoint.clone(),
                        api_key: input.provider_api_key.clone(),
                        model: input.provider_model.clone(),
                        timeout_seconds: input.provider_timeout_seconds,
                    },
                )?)
            } else {
                None
            }
        } else {
            None
        };
        let after_provider_search_status = self.research_convergence_status(&input.run_id)?;

        let convergence_rerun = if input.rerun_after_check.unwrap_or(true)
            && !after_provider_search_status.settled
            && after_provider_search_status
                .stop_reason
                .as_deref()
                .is_none_or(|reason| reason == "continue")
        {
            Some(self.run_research_convergence_to_stop(convergence_input)?)
        } else {
            None
        };
        let final_status = convergence_rerun
            .as_ref()
            .map(|step| step.status.clone())
            .unwrap_or_else(|| {
                self.research_convergence_status(&input.run_id)
                    .expect("validated research convergence status should remain readable")
            });
        let final_report = if input.compile_final_report.unwrap_or(true) {
            Some(self.compile_research_convergence_report(&input.run_id)?)
        } else {
            None
        };
        let editorial_from_rerun = convergence_rerun
            .as_ref()
            .and_then(|step| step.editorial.clone());
        let editorial = if editorial_from_rerun.is_some() {
            editorial_from_rerun
        } else if convergence_config.editorial_provider.is_some()
            && !self.convergence_accepted_editorial_judgment_recorded(&input.run_id)?
        {
            Some(self.run_research_convergence_editorial_loop(&input.run_id, &convergence_config)?)
        } else {
            None
        };
        let final_report = editorial
            .as_ref()
            .map(|editorial| editorial.report.clone())
            .or(final_report);
        let remaining_host_search_tasks = self
            .list_research_convergence_host_search_tasks(&input.run_id)?
            .into_iter()
            .filter(|task| task.status == "pending")
            .collect::<Vec<_>>();
        let blockers = research_close_loop_blockers(
            &final_status,
            provider_search.as_ref(),
            &remaining_host_search_tasks,
            final_report.as_ref(),
        );
        let closure_status = research_close_loop_status(
            &final_status,
            provider_search.as_ref(),
            &remaining_host_search_tasks,
        );

        Ok(ResearchConvergenceCloseLoopResult {
            run_id: input.run_id,
            initial_status,
            checked_artifact_id,
            active_fact_check,
            after_active_fact_check_status,
            provider_search,
            after_provider_search_status,
            convergence_rerun,
            final_status,
            final_report,
            editorial,
            remaining_host_search_tasks,
            closure_status,
            blockers,
        })
    }

    pub fn list_research_convergence_snapshots(
        &self,
        run_id: &str,
    ) -> Result<Vec<ResearchConvergenceSnapshot>> {
        self.require_research_run(run_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, run_id, iteration_id, source_count_total, source_count_new,
                   primary_source_count_new, claim_count_total, statement_count_current,
                   statement_count_changed, critical_open_challenges, high_open_challenges,
                   strong_refutations, unknown_high_impact_claims, mean_confidence_delta,
                   max_confidence_delta, source_novelty_score, claim_novelty_score,
                   position_edit_distance, citation_support_score, active_fact_check_score,
                   evaluator_score, cost_usd_estimated, elapsed_seconds, stop_rule_json,
                   settled, created_at
            FROM research_convergence_snapshots
            WHERE run_id = ?1
            ORDER BY created_at ASC
            "#,
        )?;
        rows(stmt.query_map(params![run_id], research_convergence_snapshot_from_row)?)
    }

    pub fn list_research_report_judgments(
        &self,
        run_id: &str,
    ) -> Result<Vec<ResearchReportJudgment>> {
        self.require_research_run(run_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, run_id, report_id, judgment_version, overall_decision, scores_json,
                   blocking_findings_json, non_blocking_findings_json, evidence_checked_json,
                   remaining_risks_json, commands_or_artifacts_reviewed_json, created_at
            FROM research_report_judgments
            WHERE run_id = ?1
            ORDER BY created_at ASC
            "#,
        )?;
        rows(stmt.query_map(params![run_id], research_report_judgment_from_row)?)
    }

    pub(crate) fn latest_research_iteration(
        &self,
        run_id: &str,
    ) -> Result<Option<ResearchIteration>> {
        validate_id(run_id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, run_id, iteration_index, parent_iteration_id, status, objective,
                       position_artifact_id, statement_set_artifact_id, challenge_pack_artifact_id,
                       disproof_pack_artifact_id, revision_artifact_id, convergence_snapshot_id,
                       cost_decision_id, started_at, completed_at, stop_reason, error_message_redacted
                FROM research_iterations
                WHERE run_id = ?1
                ORDER BY iteration_index DESC
                LIMIT 1
                "#,
                params![run_id],
                research_iteration_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub(crate) fn latest_research_convergence_snapshot(
        &self,
        run_id: &str,
    ) -> Result<Option<ResearchConvergenceSnapshot>> {
        validate_id(run_id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, run_id, iteration_id, source_count_total, source_count_new,
                       primary_source_count_new, claim_count_total, statement_count_current,
                       statement_count_changed, critical_open_challenges, high_open_challenges,
                       strong_refutations, unknown_high_impact_claims, mean_confidence_delta,
                       max_confidence_delta, source_novelty_score, claim_novelty_score,
                       position_edit_distance, citation_support_score, active_fact_check_score,
                       evaluator_score, cost_usd_estimated, elapsed_seconds, stop_rule_json,
                       settled, created_at
                FROM research_convergence_snapshots
                WHERE run_id = ?1
                ORDER BY created_at DESC
                LIMIT 1
                "#,
                params![run_id],
                research_convergence_snapshot_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub(crate) fn list_research_statements_for_iteration(
        &self,
        iteration_id: &str,
    ) -> Result<Vec<ResearchStatement>> {
        validate_id(iteration_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, run_id, iteration_id, parent_statement_id, stable_key, statement_type, text,
                   scope, temporal_scope, confidence, certainty_label, status, importance,
                   evidence_json, counterevidence_json, assumptions_json, caveats_json,
                   created_by_role, created_at, updated_at
            FROM research_statements
            WHERE iteration_id = ?1
            ORDER BY created_at ASC
            "#,
        )?;
        rows(stmt.query_map(params![iteration_id], research_statement_from_row)?)
    }

    pub(crate) fn insert_research_iteration(
        &self,
        run_id: &str,
        iteration_index: usize,
        parent_iteration_id: Option<&str>,
        status: &str,
        objective: &str,
        started_at: &str,
    ) -> Result<ResearchIteration> {
        self.require_research_run(run_id)?;
        if let Some(parent_id) = parent_iteration_id {
            let parent = self
                .read_research_iteration(parent_id)?
                .with_context(|| format!("parent research iteration not found: {parent_id}"))?;
            if parent.run_id != run_id {
                bail!("parent research iteration belongs to a different run");
            }
        }
        validate_research_iteration_status(status)?;
        validate_notes(objective)?;
        let id = research_iteration_id(run_id, iteration_index);
        self.conn.execute(
            r#"
            INSERT INTO research_iterations
              (id, run_id, iteration_index, parent_iteration_id, status, objective,
               started_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(run_id, iteration_index) DO UPDATE SET
              status = excluded.status,
              objective = excluded.objective,
              started_at = excluded.started_at,
              error_message_redacted = NULL
            "#,
            params![
                id,
                run_id,
                iteration_index as i64,
                parent_iteration_id,
                status,
                objective,
                started_at
            ],
        )?;
        self.read_research_iteration(&id)?
            .with_context(|| format!("inserted research iteration not found: {id}"))
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn finish_research_iteration(
        &self,
        iteration_id: &str,
        position_artifact_id: &str,
        statement_set_artifact_id: &str,
        challenge_pack_artifact_id: &str,
        disproof_pack_artifact_id: &str,
        revision_artifact_id: &str,
        snapshot_id: &str,
        status: &str,
        stop_reason: Option<&str>,
    ) -> Result<ResearchIteration> {
        validate_research_iteration_status(status)?;
        let iteration = self
            .read_research_iteration(iteration_id)?
            .with_context(|| format!("research iteration not found: {iteration_id}"))?;
        for artifact_id in [
            position_artifact_id,
            statement_set_artifact_id,
            challenge_pack_artifact_id,
            disproof_pack_artifact_id,
            revision_artifact_id,
        ] {
            let artifact = self
                .read_research_artifact(artifact_id)?
                .with_context(|| format!("research artifact not found: {artifact_id}"))?;
            if artifact.run_id != iteration.run_id {
                bail!("research iteration artifact belongs to a different run");
            }
        }
        self.conn.execute(
            r#"
            UPDATE research_iterations
            SET status = ?2,
                position_artifact_id = ?3,
                statement_set_artifact_id = ?4,
                challenge_pack_artifact_id = ?5,
                disproof_pack_artifact_id = ?6,
                revision_artifact_id = ?7,
                convergence_snapshot_id = ?8,
                completed_at = ?9,
                stop_reason = ?10
            WHERE id = ?1
            "#,
            params![
                iteration_id,
                status,
                position_artifact_id,
                statement_set_artifact_id,
                challenge_pack_artifact_id,
                disproof_pack_artifact_id,
                revision_artifact_id,
                snapshot_id,
                now(),
                stop_reason
            ],
        )?;
        self.read_research_iteration(iteration_id)?
            .with_context(|| format!("finished research iteration not found: {iteration_id}"))
    }

    pub(crate) fn compile_research_statements_for_iteration(
        &self,
        run_id: &str,
        iteration_id: &str,
        previous_iteration_id: Option<&str>,
    ) -> Result<Vec<ResearchStatement>> {
        self.require_research_run(run_id)?;
        let previous_by_key: BTreeMap<String, ResearchStatement> = match previous_iteration_id {
            Some(id) => self
                .list_research_statements_for_iteration(id)?
                .into_iter()
                .map(|statement| (statement.stable_key.clone(), statement))
                .collect(),
            None => BTreeMap::new(),
        };
        let claims = self.list_research_claims(run_id)?;
        let narrative_claims = narrative_research_claims(&claims);
        let mut statements = Vec::new();
        for record in narrative_claims {
            let stable_key = research_statement_stable_key(&record.claim.text);
            let parent_statement_id = previous_by_key
                .get(&stable_key)
                .map(|statement| statement.id.clone());
            let statement = ResearchStatement {
                id: research_statement_id(run_id, iteration_id, &stable_key),
                run_id: run_id.to_string(),
                iteration_id: iteration_id.to_string(),
                parent_statement_id,
                stable_key,
                statement_type: research_statement_type_from_claim(&record.claim.kind),
                text: record.claim.text.clone(),
                scope: record.claim.subject.clone(),
                temporal_scope: record.claim.temporal_scope.clone(),
                confidence: record.claim.confidence.clamp(0.0, 1.0),
                certainty_label: research_certainty_label(record.claim.confidence),
                status: "survived".to_string(),
                importance: research_statement_importance(&record.claim),
                evidence: json!({
                    "claim_ids": [record.claim.id.clone()],
                    "source_card_ids": record.sources.iter().map(|source| source.source_card_id.clone()).collect::<Vec<_>>(),
                    "document_anchor_ids": record.document_anchors.iter().map(|anchor| anchor.id.clone()).collect::<Vec<_>>(),
                }),
                counterevidence: json!([]),
                assumptions: json!([]),
                caveats: json!(record.claim.caveats),
                created_by_role: "statement_compiler".to_string(),
                created_at: now(),
                updated_at: now(),
            };
            statements.push(self.upsert_research_statement(statement)?);
        }
        Ok(statements)
    }

    pub(crate) fn upsert_research_statement(
        &self,
        mut statement: ResearchStatement,
    ) -> Result<ResearchStatement> {
        normalize_research_statement(&mut statement)?;
        self.conn.execute(
            r#"
            INSERT INTO research_statements
              (id, run_id, iteration_id, parent_statement_id, stable_key, statement_type, text,
               scope, temporal_scope, confidence, certainty_label, status, importance,
               evidence_json, counterevidence_json, assumptions_json, caveats_json,
               created_by_role, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                    ?14, ?15, ?16, ?17, ?18, ?19, ?20)
            ON CONFLICT(run_id, iteration_id, stable_key) DO UPDATE SET
              parent_statement_id = excluded.parent_statement_id,
              statement_type = excluded.statement_type,
              text = excluded.text,
              scope = excluded.scope,
              temporal_scope = excluded.temporal_scope,
              confidence = excluded.confidence,
              certainty_label = excluded.certainty_label,
              status = excluded.status,
              importance = excluded.importance,
              evidence_json = excluded.evidence_json,
              counterevidence_json = excluded.counterevidence_json,
              assumptions_json = excluded.assumptions_json,
              caveats_json = excluded.caveats_json,
              updated_at = excluded.updated_at
            "#,
            params![
                statement.id,
                statement.run_id,
                statement.iteration_id,
                statement.parent_statement_id,
                statement.stable_key,
                statement.statement_type,
                statement.text,
                statement.scope,
                statement.temporal_scope,
                statement.confidence,
                statement.certainty_label,
                statement.status,
                statement.importance,
                canonical_json(&statement.evidence)?,
                canonical_json(&statement.counterevidence)?,
                canonical_json(&statement.assumptions)?,
                canonical_json(&statement.caveats)?,
                statement.created_by_role,
                statement.created_at,
                statement.updated_at,
            ],
        )?;
        self.read_research_statement(&statement.id)?
            .with_context(|| format!("inserted research statement not found: {}", statement.id))
    }

    pub(crate) fn read_research_statement(&self, id: &str) -> Result<Option<ResearchStatement>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, run_id, iteration_id, parent_statement_id, stable_key, statement_type, text,
                       scope, temporal_scope, confidence, certainty_label, status, importance,
                       evidence_json, counterevidence_json, assumptions_json, caveats_json,
                       created_by_role, created_at, updated_at
                FROM research_statements
                WHERE id = ?1
                "#,
                params![id],
                research_statement_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub(crate) fn generate_research_challenges_for_iteration(
        &self,
        run_id: &str,
        iteration_id: &str,
        statements: &[ResearchStatement],
    ) -> Result<Vec<ResearchChallenge>> {
        let sources = self.list_research_run_sources(run_id)?;
        let has_primary = sources
            .iter()
            .filter_map(|record| record.source_card.as_ref())
            .any(|card| infer_source_role_from_card(card) == "primary");
        let contradictions = self
            .run_research_skeptic_pass(run_id)?
            .contradictions
            .into_iter()
            .collect::<Vec<_>>();
        let mut challenges = Vec::new();
        for statement in statements {
            let mut types = vec!["alternative_hypothesis"];
            if !has_primary && matches!(statement.importance.as_str(), "critical" | "high") {
                types.push("missing_primary_source");
            }
            if statement_evidence_claim_ids(statement).is_empty() {
                types.push("citation_gap");
            }
            if statement_has_stale_source(statement, &sources) {
                types.push("stale_evidence");
            }
            if statement_has_contradiction(statement, &contradictions) {
                types.push("contradiction");
            }
            for challenge_type in types {
                let severity = research_challenge_severity(statement, challenge_type);
                let challenge = ResearchChallenge {
                    id: research_challenge_id(run_id, iteration_id, &statement.id, challenge_type),
                    run_id: run_id.to_string(),
                    iteration_id: iteration_id.to_string(),
                    statement_id: statement.id.clone(),
                    challenge_type: challenge_type.to_string(),
                    severity,
                    rationale: research_challenge_rationale(statement, challenge_type),
                    would_change_answer_if_true: true,
                    search_plan: json!({
                        "queries": research_challenge_queries(statement, challenge_type),
                        "requires_host_search_proof": true,
                        "status": "not_searched_by_deterministic_step"
                    }),
                    required_source_families: json!(research_challenge_source_families(
                        challenge_type
                    )),
                    status: "open".to_string(),
                    created_by_role: "red_teamer".to_string(),
                    created_at: now(),
                    updated_at: now(),
                };
                challenges.push(self.upsert_research_challenge(challenge)?);
            }
        }
        Ok(challenges)
    }

    pub(crate) fn apply_research_host_search_proofs_to_challenges(
        &self,
        run_id: &str,
        challenges: Vec<ResearchChallenge>,
    ) -> Result<Vec<ResearchChallenge>> {
        let host_searches = self.list_research_host_searches(run_id)?;
        let mut resolved = Vec::with_capacity(challenges.len());
        for mut challenge in challenges {
            if let Some(proof) = host_search_proof_for_challenge(&challenge, &host_searches) {
                challenge.status = "answered".to_string();
                challenge.search_plan = merge_challenge_host_search_proof(
                    &challenge.search_plan,
                    &proof,
                    "host_search_recorded",
                );
                challenge.updated_at = now();
                resolved.push(self.upsert_research_challenge(challenge)?);
            } else {
                resolved.push(challenge);
            }
        }
        Ok(resolved)
    }

    pub(crate) fn refresh_research_challenges_from_host_search_proofs(
        &self,
        run_id: &str,
    ) -> Result<()> {
        let challenges = self.list_research_challenges(run_id)?;
        self.apply_research_host_search_proofs_to_challenges(run_id, challenges)?;
        Ok(())
    }

    pub(crate) fn upsert_research_challenge(
        &self,
        mut challenge: ResearchChallenge,
    ) -> Result<ResearchChallenge> {
        normalize_research_challenge(&mut challenge)?;
        self.conn.execute(
            r#"
            INSERT INTO research_challenges
              (id, run_id, iteration_id, statement_id, challenge_type, severity, rationale,
               would_change_answer_if_true, search_plan_json, required_source_families_json,
               status, created_by_role, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            ON CONFLICT(run_id, iteration_id, statement_id, challenge_type) DO UPDATE SET
              severity = excluded.severity,
              rationale = excluded.rationale,
              would_change_answer_if_true = excluded.would_change_answer_if_true,
              search_plan_json = excluded.search_plan_json,
              required_source_families_json = excluded.required_source_families_json,
              status = excluded.status,
              updated_at = excluded.updated_at
            "#,
            params![
                challenge.id,
                challenge.run_id,
                challenge.iteration_id,
                challenge.statement_id,
                challenge.challenge_type,
                challenge.severity,
                challenge.rationale,
                if challenge.would_change_answer_if_true {
                    1
                } else {
                    0
                },
                canonical_json(&challenge.search_plan)?,
                canonical_json(&challenge.required_source_families)?,
                challenge.status,
                challenge.created_by_role,
                challenge.created_at,
                challenge.updated_at,
            ],
        )?;
        self.read_research_challenge(&challenge.id)?
            .with_context(|| format!("inserted research challenge not found: {}", challenge.id))
    }

    pub(crate) fn read_research_challenge(&self, id: &str) -> Result<Option<ResearchChallenge>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, run_id, iteration_id, statement_id, challenge_type, severity, rationale,
                       would_change_answer_if_true, search_plan_json, required_source_families_json,
                       status, created_by_role, created_at, updated_at
                FROM research_challenges
                WHERE id = ?1
                "#,
                params![id],
                research_challenge_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub(crate) fn generate_research_disproofs_for_iteration(
        &self,
        run_id: &str,
        iteration_id: &str,
        challenges: &[ResearchChallenge],
    ) -> Result<Vec<ResearchDisproof>> {
        let mut disproofs = Vec::new();
        for challenge in challenges {
            let host_search_proof = challenge
                .search_plan
                .get("host_search_proof")
                .cloned()
                .unwrap_or(Value::Null);
            let (verdict, strength, delta, requires_revision, notes) = if host_search_proof
                .is_null()
            {
                deterministic_disproof_verdict(challenge)
            } else {
                (
                        "supports".to_string(),
                        "moderate".to_string(),
                        0.0,
                        false,
                        "Recorded host-native search proof answered this challenge; selected results are linked as research sources for inspection."
                            .to_string(),
                    )
            };
            let disproof = ResearchDisproof {
                id: research_disproof_id(run_id, iteration_id, &challenge.id),
                run_id: run_id.to_string(),
                iteration_id: iteration_id.to_string(),
                challenge_id: challenge.id.clone(),
                statement_id: challenge.statement_id.clone(),
                verdict,
                strength,
                evidence: json!({
                    "challenge_id": challenge.id,
                    "search_plan": challenge.search_plan,
                    "host_search_proof": host_search_proof,
                    "deterministic_step": true
                }),
                reasoning_summary: notes,
                confidence_delta: delta,
                requires_revision,
                created_by_role: "verifier".to_string(),
                created_at: now(),
            };
            disproofs.push(self.insert_research_disproof(disproof)?);
        }
        Ok(disproofs)
    }

    pub(crate) fn insert_research_disproof(
        &self,
        mut disproof: ResearchDisproof,
    ) -> Result<ResearchDisproof> {
        normalize_research_disproof(&mut disproof)?;
        self.conn.execute(
            r#"
            INSERT INTO research_disproofs
              (id, run_id, iteration_id, challenge_id, statement_id, verdict, strength,
               evidence_json, reasoning_summary, confidence_delta, requires_revision,
               created_by_role, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(run_id, iteration_id, challenge_id) DO UPDATE SET
              verdict = excluded.verdict,
              strength = excluded.strength,
              evidence_json = excluded.evidence_json,
              reasoning_summary = excluded.reasoning_summary,
              confidence_delta = excluded.confidence_delta,
              requires_revision = excluded.requires_revision
            "#,
            params![
                disproof.id,
                disproof.run_id,
                disproof.iteration_id,
                disproof.challenge_id,
                disproof.statement_id,
                disproof.verdict,
                disproof.strength,
                canonical_json(&disproof.evidence)?,
                disproof.reasoning_summary,
                disproof.confidence_delta,
                if disproof.requires_revision { 1 } else { 0 },
                disproof.created_by_role,
                disproof.created_at,
            ],
        )?;
        self.read_research_disproof(&disproof.id)?
            .with_context(|| format!("inserted research disproof not found: {}", disproof.id))
    }

    pub(crate) fn read_research_disproof(&self, id: &str) -> Result<Option<ResearchDisproof>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, run_id, iteration_id, challenge_id, statement_id, verdict, strength,
                       evidence_json, reasoning_summary, confidence_delta, requires_revision,
                       created_by_role, created_at
                FROM research_disproofs
                WHERE id = ?1
                "#,
                params![id],
                research_disproof_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub(crate) fn apply_research_revisions_for_iteration(
        &self,
        run_id: &str,
        iteration_id: &str,
        disproofs: &[ResearchDisproof],
        statements: &mut [ResearchStatement],
    ) -> Result<Vec<ResearchRevision>> {
        let mut revisions = Vec::new();
        for disproof in disproofs
            .iter()
            .filter(|disproof| disproof.requires_revision)
        {
            let Some(statement) = statements
                .iter_mut()
                .find(|statement| statement.id == disproof.statement_id)
            else {
                continue;
            };
            let revision_type = match disproof.verdict.as_str() {
                "refutes" => {
                    statement.status = "refuted".to_string();
                    statement.confidence =
                        (statement.confidence + disproof.confidence_delta).clamp(0.0, 1.0);
                    statement.certainty_label = research_certainty_label(statement.confidence);
                    "dropped"
                }
                "weakens" => {
                    statement.status = "weakened".to_string();
                    statement.confidence =
                        (statement.confidence + disproof.confidence_delta).clamp(0.0, 1.0);
                    statement.certainty_label = research_certainty_label(statement.confidence);
                    "confidence_downgraded"
                }
                "unknown" => {
                    statement.status = "unresolved".to_string();
                    "caveated"
                }
                _ => "caveated",
            };
            let mut caveats = statement.caveats.as_array().cloned().unwrap_or_default();
            caveats.push(json!(format!(
                "Convergence {} challenge: {}",
                disproof.verdict, disproof.reasoning_summary
            )));
            statement.caveats = Value::Array(caveats);
            statement.updated_at = now();
            let updated = self.upsert_research_statement(statement.clone())?;
            *statement = updated;
            let revision = ResearchRevision {
                id: research_revision_id(
                    run_id,
                    iteration_id,
                    &disproof.statement_id,
                    revision_type,
                ),
                run_id: run_id.to_string(),
                iteration_id: iteration_id.to_string(),
                from_statement_id: disproof.statement_id.clone(),
                to_statement_id: None,
                revision_type: revision_type.to_string(),
                rationale: disproof.reasoning_summary.clone(),
                trigger_disproof_ids: json!([disproof.id.clone()]),
                evidence_delta: disproof.evidence.clone(),
                created_at: now(),
            };
            revisions.push(self.insert_research_revision(revision)?);
        }
        Ok(revisions)
    }

    pub(crate) fn insert_research_revision(
        &self,
        mut revision: ResearchRevision,
    ) -> Result<ResearchRevision> {
        normalize_research_revision(&mut revision)?;
        self.conn.execute(
            r#"
            INSERT INTO research_revisions
              (id, run_id, iteration_id, from_statement_id, to_statement_id, revision_type,
               rationale, trigger_disproof_ids_json, evidence_delta_json, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(run_id, iteration_id, from_statement_id, revision_type) DO UPDATE SET
              to_statement_id = excluded.to_statement_id,
              rationale = excluded.rationale,
              trigger_disproof_ids_json = excluded.trigger_disproof_ids_json,
              evidence_delta_json = excluded.evidence_delta_json
            "#,
            params![
                revision.id,
                revision.run_id,
                revision.iteration_id,
                revision.from_statement_id,
                revision.to_statement_id,
                revision.revision_type,
                revision.rationale,
                canonical_json(&revision.trigger_disproof_ids)?,
                canonical_json(&revision.evidence_delta)?,
                revision.created_at,
            ],
        )?;
        self.read_research_revision(&revision.id)?
            .with_context(|| format!("inserted research revision not found: {}", revision.id))
    }

    pub(crate) fn read_research_revision(&self, id: &str) -> Result<Option<ResearchRevision>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, run_id, iteration_id, from_statement_id, to_statement_id, revision_type,
                       rationale, trigger_disproof_ids_json, evidence_delta_json, created_at
                FROM research_revisions
                WHERE id = ?1
                "#,
                params![id],
                research_revision_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub(crate) fn run_research_fact_checks_for_iteration(
        &self,
        run_id: &str,
        iteration_id: &str,
        statements: &[ResearchStatement],
    ) -> Result<Vec<ResearchFactCheck>> {
        let mut checks = Vec::new();
        for statement in statements {
            let claim_ids = statement_evidence_claim_ids(statement);
            let (label, notes) = if statement.status == "refuted" {
                (
                    "wrong",
                    "Statement was refuted by a convergence disproof and cannot appear as a final conclusion.",
                )
            } else if matches!(statement.status.as_str(), "weakened" | "unresolved") {
                (
                    "unknown",
                    "Statement is weakened or unresolved and must remain caveated.",
                )
            } else if claim_ids.is_empty() {
                (
                    "unknown",
                    "Statement has no linked extracted claim evidence.",
                )
            } else {
                (
                    "right",
                    "Statement has linked extracted claim evidence and no deterministic refutation.",
                )
            };
            let check = ResearchFactCheck {
                id: research_fact_check_id(run_id, iteration_id, &statement.id),
                run_id: run_id.to_string(),
                iteration_id: iteration_id.to_string(),
                statement_id: statement.id.clone(),
                label: label.to_string(),
                impact: if matches!(statement.importance.as_str(), "critical" | "high") {
                    "high".to_string()
                } else {
                    "medium".to_string()
                },
                evidence: json!({
                    "statement_status": statement.status,
                    "claim_ids": claim_ids,
                }),
                notes: notes.to_string(),
                created_at: now(),
            };
            checks.push(self.insert_research_fact_check(check)?);
        }
        Ok(checks)
    }

    pub(crate) fn insert_research_fact_check(
        &self,
        mut check: ResearchFactCheck,
    ) -> Result<ResearchFactCheck> {
        normalize_research_fact_check(&mut check)?;
        self.conn.execute(
            r#"
            INSERT INTO research_fact_checks
              (id, run_id, iteration_id, statement_id, label, impact, evidence_json, notes, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(run_id, iteration_id, statement_id) DO UPDATE SET
              label = excluded.label,
              impact = excluded.impact,
              evidence_json = excluded.evidence_json,
              notes = excluded.notes
            "#,
            params![
                check.id,
                check.run_id,
                check.iteration_id,
                check.statement_id,
                check.label,
                check.impact,
                canonical_json(&check.evidence)?,
                check.notes,
                check.created_at,
            ],
        )?;
        self.read_research_fact_check(&check.id)?
            .with_context(|| format!("inserted research fact check not found: {}", check.id))
    }

    pub(crate) fn read_research_fact_check(&self, id: &str) -> Result<Option<ResearchFactCheck>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, run_id, iteration_id, statement_id, label, impact, evidence_json, notes, created_at
                FROM research_fact_checks
                WHERE id = ?1
                "#,
                params![id],
                research_fact_check_from_row,
            )
            .optional()
            .map_err(Into::into)
    }
}
