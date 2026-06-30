use super::*;

pub(crate) fn call_mcp_tool(paths: &AppPaths, name: &str, arguments: Value) -> Result<Value> {
    let store = Store::open(paths.clone())?;
    match name {
        "research_capabilities" => Ok(research_capabilities(paths)),
        "commerce_research_capabilities" => Ok(commerce_capabilities(paths)),
        "provider_credential_probe" => Ok(json!(
            store.provider_credential_probe(&provider_list_from_mcp(&arguments))?
        )),
        "commerce_run_config_set" => Ok(json!(
            store.record_commerce_run_config(commerce_run_config_input_from_mcp(&arguments)?)?
        )),
        "commerce_run_config" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.read_commerce_run_config(&run_id)?))
        }
        "commerce_candidate_add" => Ok(json!(
            store.record_commerce_candidate(commerce_candidate_input_from_mcp(&arguments)?)?
        )),
        "commerce_candidates" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.list_commerce_candidates(&run_id)?))
        }
        "commerce_availability_proof_add" => Ok(json!(store.record_commerce_availability_proof(
            commerce_availability_proof_input_from_mcp(&arguments)?
        )?)),
        "commerce_availability_proofs" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.list_commerce_availability_proofs(&run_id)?))
        }
        "commerce_rendered_page_check" => Ok(json!(store.record_commerce_rendered_page_check(
            commerce_rendered_page_check_input_from_mcp(&arguments)?
        )?)),
        "commerce_context_fact_add" => Ok(json!(
            store
                .record_commerce_context_fact(commerce_context_fact_input_from_mcp(&arguments)?)?
        )),
        "commerce_context_facts" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.list_commerce_context_facts(&run_id)?))
        }
        "commerce_context_packet_compile" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.compile_commerce_context_packet(&run_id)?))
        }
        "commerce_verification_attempt_add" => {
            Ok(json!(store.record_commerce_verification_attempt(
                commerce_verification_attempt_input_from_mcp(&arguments)?
            )?))
        }
        "commerce_verification_attempts" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.list_commerce_verification_attempts(&run_id)?))
        }
        "commerce_report_judgment_add" => Ok(json!(store.record_commerce_report_judgment(
            commerce_report_judgment_input_from_mcp(&arguments)?
        )?)),
        "commerce_report_judgments" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.list_commerce_report_judgments(&run_id)?))
        }
        "commerce_report_compile" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.compile_commerce_report(&run_id)?))
        }
        "job_profile_add" => Ok(json!(
            store.record_job_candidate_profile(JobCandidateProfileInput {
                label: required_string(&arguments, "label")?,
                current_resume_source: arguments
                    .get("current_resume_source")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                linkedin_source: arguments
                    .get("linkedin_source")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                github_profile: arguments
                    .get("github_profile")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                blog_url: arguments
                    .get("blog_url")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                metadata: json_argument(&arguments, "metadata", "metadata_json", json!({}))?,
            })?
        )),
        "job_profiles" => Ok(json!(store.list_job_candidate_profiles()?)),
        "job_import_batch" => Ok(json!(
            store.import_job_batch(job_import_batch_from_mcp(&arguments)?)?
        )),
        "job_evidence_add" => Ok(json!(
            store.record_job_evidence_card(JobEvidenceCardInput {
                profile_id: required_string(&arguments, "profile_id")?,
                title: required_string(&arguments, "title")?,
                evidence_type: required_string(&arguments, "evidence_type")?,
                visibility: required_string(&arguments, "visibility")?,
                summary: required_string(&arguments, "summary")?,
                proof_url: arguments
                    .get("proof_url")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                local_path: arguments
                    .get("local_path")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                source_date: arguments
                    .get("source_date")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                confidence: required_string(&arguments, "confidence")?,
                tags: string_array_argument(&arguments, "tags")?,
                safe_application_text: required_string(&arguments, "safe_application_text")?,
                unsafe_terms: string_array_argument(&arguments, "unsafe_terms")?,
                metadata: json_argument(&arguments, "metadata", "metadata_json", json!({}))?,
            })?
        )),
        "job_evidence_list" => {
            let profile_id = required_string(&arguments, "profile_id")?;
            Ok(json!(store.list_job_evidence_cards(&profile_id)?))
        }
        "job_evidence_review_report" => {
            let profile_id = required_string(&arguments, "profile_id")?;
            Ok(json!(
                store.compile_job_evidence_review_report(&profile_id)?
            ))
        }
        "job_privacy_check" => {
            let artifact_type = required_string(&arguments, "artifact_type")?;
            let artifact_id = arguments.get("artifact_id").and_then(Value::as_str);
            let text = required_string(&arguments, "text")?;
            let blocked_terms = string_array_argument(&arguments, "blocked_terms")?;
            Ok(json!(store.check_job_privacy_text(
                &artifact_type,
                artifact_id,
                &text,
                &blocked_terms,
            )?))
        }
        "job_role_add" => Ok(json!(
            store.record_job_role_card(JobRoleCardInput {
                company: required_string(&arguments, "company")?,
                role_title: required_string(&arguments, "role_title")?,
                canonical_url: arguments
                    .get("canonical_url")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                source_family: required_string(&arguments, "source_family")?,
                source_url: required_string(&arguments, "source_url")?,
                source_confidence: required_string(&arguments, "source_confidence")?,
                date_accessed: arguments
                    .get("date_accessed")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                posting_freshness: required_string(&arguments, "posting_freshness")?,
                location: arguments
                    .get("location")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                work_mode: arguments
                    .get("work_mode")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                company_stage_or_size: arguments
                    .get("company_stage_or_size")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                role_seniority: arguments
                    .get("role_seniority")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                core_requirements: string_array_argument(&arguments, "core_requirements")?,
                implied_business_problem: arguments
                    .get("implied_business_problem")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                why_they_might_need_user: arguments
                    .get("why_they_might_need_user")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                evidence_card_ids: string_array_argument(&arguments, "evidence_card_ids")?,
                gaps_or_blockers: string_array_argument(&arguments, "gaps_or_blockers")?,
                cluster: arguments
                    .get("cluster")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                current_status: required_string(&arguments, "current_status")?,
                metadata: json_argument(&arguments, "metadata", "metadata_json", json!({}))?,
            })?
        )),
        "job_roles" => Ok(json!(store.list_job_role_cards()?)),
        "job_score_add" => Ok(json!(store.record_job_fit_score(JobFitScoreInput {
            role_id: required_string(&arguments, "role_id")?,
            profile_id: required_string(&arguments, "profile_id")?,
            scorer: optional_string(&arguments, "scorer", "human"),
            role_fit: required_f64_arg(&arguments, "role_fit")?,
            domain_fit: required_f64_arg(&arguments, "domain_fit")?,
            evidence_fit: required_f64_arg(&arguments, "evidence_fit")?,
            geo_work_fit: required_f64_arg(&arguments, "geo_work_fit")?,
            stage_fit: required_f64_arg(&arguments, "stage_fit")?,
            practical_odds: required_f64_arg(&arguments, "practical_odds")?,
            interest_energy: required_f64_arg(&arguments, "interest_energy")?,
            blockers: string_array_argument(&arguments, "blockers")?,
            evidence_card_ids: string_array_argument(&arguments, "evidence_card_ids")?,
            explanation: required_string(&arguments, "explanation")?,
        })?)),
        "job_shortlist" => {
            let profile_id = required_string(&arguments, "profile_id")?;
            Ok(json!(store.compile_job_shortlist(&profile_id)?))
        }
        "job_outreach_readiness" => {
            let profile_id = required_string(&arguments, "profile_id")?;
            let limit = arguments.get("limit").and_then(Value::as_u64).unwrap_or(20) as usize;
            Ok(json!(store.compile_job_outreach_readiness_report(
                &profile_id,
                limit
            )?))
        }
        "job_company_targets" => {
            let profile_id = required_string(&arguments, "profile_id")?;
            let market = arguments.get("market").and_then(Value::as_str);
            let limit = optional_usize(&arguments, "limit", 20);
            Ok(json!(store.compile_job_company_target_report(
                &profile_id,
                market,
                limit
            )?))
        }
        "job_packet_create" => Ok(json!(
            store.create_job_application_packet(JobApplicationPacketInput {
                role_id: required_string(&arguments, "role_id")?,
                profile_id: required_string(&arguments, "profile_id")?,
                evidence_card_ids: string_array_argument(&arguments, "evidence_card_ids")?,
                resume_emphasis: required_string(&arguments, "resume_emphasis")?,
                tailored_bullets: string_array_argument(&arguments, "tailored_bullets")?,
                outreach_note: required_string(&arguments, "outreach_note")?,
                proof_links: json_argument(
                    &arguments,
                    "proof_links",
                    "proof_links_json",
                    json!({})
                )?,
                likely_objections: string_array_argument(&arguments, "likely_objections")?,
                interview_stories: string_array_argument(&arguments, "interview_stories")?,
                questions_to_ask: string_array_argument(&arguments, "questions_to_ask")?,
                reviewer_note: arguments
                    .get("reviewer_note")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
            })?
        )),
        "job_packet_approve" => Ok(json!(store.update_job_application_packet_status(
            JobApplicationPacketStatusInput {
                packet_id: required_string(&arguments, "packet_id")?,
                status: "approved".to_string(),
                reviewer_note: Some(required_string(&arguments, "reviewer_note")?),
            }
        )?)),
        "job_packet_export" => {
            let packet_id = required_string(&arguments, "packet_id")?;
            let out_dir = required_string(&arguments, "out_dir")?;
            Ok(json!(store.export_job_application_packet(
                &packet_id,
                &PathBuf::from(out_dir)
            )?))
        }
        "job_packet_export_set" => {
            let profile_id = required_string(&arguments, "profile_id")?;
            let packet_ids = string_array_argument(&arguments, "packet_ids")?;
            let out_dir = required_string(&arguments, "out_dir")?;
            Ok(json!(store.export_job_application_packet_set(
                &profile_id,
                packet_ids,
                &PathBuf::from(out_dir),
            )?))
        }
        "job_application_record" => Ok(json!(
            store.record_job_application(JobApplicationInput {
                role_id: required_string(&arguments, "role_id")?,
                packet_id: arguments
                    .get("packet_id")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                status: required_string(&arguments, "status")?,
                applied_at: arguments
                    .get("applied_at")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                follow_up_at: arguments
                    .get("follow_up_at")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                outcome_note: arguments
                    .get("outcome_note")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
            })?
        )),
        "job_source_refresh" => Ok(json!(
            store.run_job_source_refresh(JobSourceRefreshInput {
                source_id: required_string(&arguments, "source_id")?,
                body: arguments
                    .get("body")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                fetched_url: arguments
                    .get("fetched_url")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                fetch_live: optional_bool(&arguments, "fetch_live", false),
            })?
        )),
        "job_radar_schedule" => {
            let profile_id = required_string(&arguments, "profile_id")?;
            let scope = required_string(&arguments, "scope")?;
            let source_ids = string_array_argument(&arguments, "source_ids")?;
            let fetch_live = optional_bool(&arguments, "fetch_live", false);
            let source_snapshots = json_argument(
                &arguments,
                "source_snapshots",
                "source_snapshots_json",
                json!({}),
            )?;
            let cadence = optional_string(&arguments, "cadence", "warm");
            let status = optional_string(&arguments, "status", "active");
            Ok(json!(store.schedule_job_radar_refresh(
                &profile_id,
                &scope,
                source_ids,
                fetch_live,
                source_snapshots,
                &cadence,
                &status,
            )?))
        }
        "job_radar_enqueue" => {
            let profile_id = required_string(&arguments, "profile_id")?;
            let scope = required_string(&arguments, "scope")?;
            let source_ids = string_array_argument(&arguments, "source_ids")?;
            let fetch_live = optional_bool(&arguments, "fetch_live", false);
            let source_snapshots = json_argument(
                &arguments,
                "source_snapshots",
                "source_snapshots_json",
                json!({}),
            )?;
            Ok(json!(store.enqueue_job_radar_refresh_job(
                &profile_id,
                &scope,
                source_ids,
                fetch_live,
                source_snapshots,
            )?))
        }
        "job_refresh_manual" => Ok(json!(
            store.run_job_manual_refresh(JobManualRefreshInput {
                profile_id: required_string(&arguments, "profile_id")?,
                scope: required_string(&arguments, "scope")?,
                observed_role_ids: string_array_argument(&arguments, "observed_role_ids")?,
                stale_role_ids: string_array_argument(&arguments, "stale_role_ids")?,
                closed_role_ids: string_array_argument(&arguments, "closed_role_ids")?,
                source_health_ids: string_array_argument(&arguments, "source_health_ids")?,
                proof_level: optional_string(&arguments, "proof_level", "local_proof"),
                report_artifact_id: arguments
                    .get("report_artifact_id")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
            })?
        )),
        "job_refresh_audit" => {
            let profile_id = required_string(&arguments, "profile_id")?;
            let scope = required_string(&arguments, "scope")?;
            let minimum_elapsed_hours = arguments.get("min_elapsed_hours").and_then(Value::as_i64);
            Ok(json!(store.audit_job_refresh_history(
                &profile_id,
                &scope,
                minimum_elapsed_hours,
            )?))
        }
        "job_operational_audit" => {
            let profile_id = required_string(&arguments, "profile_id")?;
            let scope = required_string(&arguments, "scope")?;
            let minimum_elapsed_hours = arguments.get("min_elapsed_hours").and_then(Value::as_i64);
            Ok(json!(store.audit_job_operational_readiness(
                &profile_id,
                &scope,
                minimum_elapsed_hours,
            )?))
        }
        "job_weekly_report" => {
            let profile_id = required_string(&arguments, "profile_id")?;
            let scope = required_string(&arguments, "scope")?;
            Ok(json!(store.compile_job_weekly_report(&profile_id, &scope)?))
        }
        "job_weekly_report_delivery_prepare" => Ok(json!(
            store.prepare_job_weekly_report_delivery(JobWeeklyReportDeliveryInput {
                report_id: required_string(&arguments, "report_id")?,
                channel: required_string(&arguments, "channel")?,
                subject: required_string(&arguments, "subject")?,
                target: required_string(&arguments, "target")?,
                idempotency_key: arguments
                    .get("idempotency_key")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
            })?
        )),
        "job_weekly_report_delivery_send" => Ok(json!(
            store.send_job_weekly_report_delivery(JobWeeklyReportDeliverySendInput {
                delivery_id: required_string(&arguments, "delivery_id")?,
                telegram_bot_token: arguments
                    .get("telegram_bot_token")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                email_account_id: arguments
                    .get("email_account_id")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                email_api_token: arguments
                    .get("email_api_token")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                email_from: arguments
                    .get("email_from")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                api_base: arguments
                    .get("api_base")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
            })?
        )),
        "job_weekly_report_deliveries" => {
            let report_id = arguments.get("report_id").and_then(Value::as_str);
            Ok(json!(store.list_job_weekly_report_deliveries(report_id)?))
        }
        "arcwell_health" => Ok(json!(store.health()?)),
        "profile_list" => Ok(json!(store.list_profile()?)),
        "profile_search" => {
            let query = required_string(&arguments, "query")?;
            Ok(json!(store.search_profile(&query)?))
        }
        "profile_set" => {
            let key = required_string(&arguments, "key")?;
            let value = required_string(&arguments, "value")?;
            let sensitivity = optional_string(&arguments, "sensitivity", "normal");
            let source = optional_string(&arguments, "source", "mcp");
            store.set_profile(&key, &value, &sensitivity, &source)?;
            Ok(json!({ "ok": true, "key": key }))
        }
        "memory_search" => {
            let query = required_string(&arguments, "query")?;
            Ok(json!(store.search_memories(&query)?))
        }
        "memory_add" => {
            let text = required_string(&arguments, "text")?;
            let kind = optional_string(&arguments, "kind", "fact");
            let sensitivity = optional_string(&arguments, "sensitivity", "normal");
            let source = optional_string(&arguments, "source", "mcp");
            let id = store.add_memory(&text, &kind, &sensitivity, &source, 0.8)?;
            Ok(json!({ "ok": true, "id": id }))
        }
        "mem0_add" => {
            let text = required_string(&arguments, "text")?;
            let user_id = arguments.get("user_id").and_then(Value::as_str);
            let source = optional_string(&arguments, "source", "mcp");
            let sensitivity = optional_string(&arguments, "sensitivity", "normal");
            let infer = optional_bool(&arguments, "infer", false);
            Ok(json!(store.mem0_add_memory(
                &text,
                user_id,
                &source,
                &sensitivity,
                infer
            )?))
        }
        "mem0_search" => {
            let query = required_string(&arguments, "query")?;
            let user_id = arguments.get("user_id").and_then(Value::as_str);
            let limit = arguments.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize;
            Ok(json!(store.mem0_search_memories(&query, user_id, limit)?))
        }
        "mem0_update" => {
            let id = required_string(&arguments, "id")?;
            let text = required_string(&arguments, "text")?;
            let user_id = arguments.get("user_id").and_then(Value::as_str);
            Ok(json!(store.mem0_update_memory(&id, &text, user_id)?))
        }
        "mem0_delete" => {
            let id = required_string(&arguments, "id")?;
            let user_id = arguments.get("user_id").and_then(Value::as_str);
            Ok(json!(store.mem0_delete_memory(&id, user_id)?))
        }
        "mem0_history" => {
            let id = required_string(&arguments, "id")?;
            Ok(json!(store.mem0_history(&id)?))
        }
        "mem0_forget_user" => {
            let user_id = arguments.get("user_id").and_then(Value::as_str);
            Ok(json!(store.mem0_forget_user(user_id)?))
        }
        "memory_recall_context" => {
            let query = required_string(&arguments, "query")?;
            let user_id = arguments.get("user_id").and_then(Value::as_str);
            let limit = arguments.get("limit").and_then(Value::as_u64).unwrap_or(8) as usize;
            Ok(json!(store.memory_recall_context(&query, user_id, limit)?))
        }
        "memory_capture" => {
            let text = required_string(&arguments, "text")?;
            let source_ref = optional_string(&arguments, "source_ref", "mcp");
            let user_id = arguments.get("user_id").and_then(Value::as_str);
            let auto_apply = optional_bool(&arguments, "auto_apply", false);
            let infer = optional_bool(&arguments, "infer", false);
            Ok(json!(store.capture_memory_from_text(
                &text,
                &source_ref,
                user_id,
                auto_apply,
                infer
            )?))
        }
        "memory_lifecycle_events" => {
            let limit = arguments.get("limit").and_then(Value::as_u64).unwrap_or(50) as u32;
            Ok(json!(store.list_memory_lifecycle_events(limit)?))
        }
        "memory_extract_candidates" => {
            let text = required_string(&arguments, "text")?;
            let source_ref = optional_string(&arguments, "source_ref", "mcp");
            Ok(json!(
                store.extract_memory_candidates_from_text(&text, &source_ref)?
            ))
        }
        "memory_dream_reconcile" => Ok(json!(store.dream_reconcile_memories()?)),
        "candidate_list" => {
            let status = optional_string(&arguments, "status", "pending");
            Ok(json!(store.list_candidates(&status)?))
        }
        "candidate_apply" => {
            let id = required_string(&arguments, "id")?;
            Ok(json!(store.apply_candidate(&id)?))
        }
        "backup_create" => {
            let path = store.create_backup()?;
            Ok(json!({ "ok": true, "path": path }))
        }
        "backup_verify" => Ok(json!(store.verify_latest_backup()?)),
        "worker_run_once" => {
            let max_jobs = arguments
                .get("max_jobs")
                .and_then(Value::as_u64)
                .unwrap_or(10) as usize;
            Ok(json!(store.run_worker_once(max_jobs)?))
        }
        "edge_event_enqueue" => {
            let source = required_string(&arguments, "source")?;
            let idempotency_key = required_string(&arguments, "idempotency_key")?;
            let payload = arguments
                .get("payload")
                .cloned()
                .unwrap_or_else(|| json!({}));
            let max_age_seconds = arguments
                .get("max_age_seconds")
                .and_then(Value::as_i64)
                .unwrap_or(3600);
            Ok(json!(store.enqueue_edge_event(
                &source,
                &idempotency_key,
                payload,
                max_age_seconds
            )?))
        }
        "edge_event_lease" => Ok(json!(store.lease_edge_event()?)),
        "edge_event_ack" => {
            let id = required_string(&arguments, "id")?;
            Ok(json!(store.ack_edge_event(&id)?))
        }
        "edge_event_nack" => {
            let id = required_string(&arguments, "id")?;
            let error = required_string(&arguments, "error")?;
            Ok(json!(store.nack_edge_event(&id, &error)?))
        }
        "edge_event_dead_letter" => {
            let id = required_string(&arguments, "id")?;
            let error = required_string(&arguments, "error")?;
            Ok(json!(store.dead_letter_edge_event(&id, &error)?))
        }
        "edge_event_list" => Ok(json!(store.list_edge_events()?)),
        "cost_summary" => {
            let (estimated_usd, actual_usd, entries) = store.cost_summary()?;
            let recent_decisions = store.list_cost_decisions(25)?;
            Ok(json!({
                "estimated_usd": estimated_usd,
                "actual_usd": actual_usd,
                "entries": entries,
                "recent_decisions": recent_decisions
            }))
        }
        "cost_policy_set" => {
            let scope = required_string(&arguments, "scope")?;
            let key = required_string(&arguments, "key")?;
            let limit_usd = arguments.get("limit_usd").and_then(Value::as_f64);
            let kill_switch = optional_bool(&arguments, "kill_switch", false);
            let override_until = arguments.get("override_until").and_then(Value::as_str);
            Ok(json!(store.set_cost_policy(
                &scope,
                &key,
                limit_usd,
                kill_switch,
                override_until
            )?))
        }
        "cost_policy_list" => Ok(json!(store.list_cost_policies()?)),
        "cost_check" => {
            let package = required_string(&arguments, "package")?;
            let provider = required_string(&arguments, "provider")?;
            let source = arguments.get("source").and_then(Value::as_str);
            let projected_usd = arguments
                .get("projected_usd")
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            Ok(json!(store.cost_decision(
                &package,
                &provider,
                source,
                projected_usd
            )?))
        }
        "policy_check" => Ok(json!(
            store.policy_check(policy_request_from_mcp_args(&arguments,)?)?
        )),
        "policy_explain" => Ok(json!(
            store.policy_explain(policy_request_from_mcp_args(&arguments,)?)?
        )),
        "policy_decision_list" => {
            let limit = optional_usize(&arguments, "limit", 50);
            Ok(json!(store.list_policy_decisions(limit)?))
        }
        "policy_rule_list" => Ok(json!(store.list_policy_rules()?)),
        "policy_override_allow" => {
            let reason = required_string(&arguments, "reason")?;
            let expires_at = required_string(&arguments, "expires_at")?;
            Ok(json!(store.create_policy_allow_override(
                policy_request_from_mcp_args(&arguments)?,
                &reason,
                &expires_at,
            )?))
        }
        "policy_approval_list" => {
            let status = arguments.get("status").and_then(Value::as_str);
            Ok(json!(store.list_policy_approvals(status)?))
        }
        "policy_approval_approve" => {
            let id = required_string(&arguments, "id")?;
            let reason = arguments.get("reason").and_then(Value::as_str);
            Ok(json!(store.approve_policy_approval(&id, reason)?))
        }
        "policy_approval_reject" => {
            let id = required_string(&arguments, "id")?;
            let reason = arguments.get("reason").and_then(Value::as_str);
            Ok(json!(store.reject_policy_approval(&id, reason)?))
        }
        "research_plan" => {
            let query = required_string(&arguments, "query")?;
            let max_sources = arguments
                .get("max_sources")
                .and_then(Value::as_u64)
                .unwrap_or(5) as usize;
            Ok(json!(store.create_research_plan(&query, max_sources)?))
        }
        "research_run" => {
            let query = required_string(&arguments, "query")?;
            Ok(json!(store.create_deep_research_run(&query)?))
        }
        "research_status" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.research_run_status(&run_id)?))
        }
        "research_read" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.read_research_run(&run_id)?))
        }
        "research_audit_run" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.audit_research_run(&run_id)?))
        }
        "research_stop" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.stop_research_run(&run_id)?))
        }
        "research_sources" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.list_research_run_sources(&run_id)?))
        }
        "research_source_add" => {
            let run_id = required_string(&arguments, "run_id")?;
            let title = required_string(&arguments, "title")?;
            let source_family = optional_string(&arguments, "source_family", "uncategorized");
            let source_type = optional_string(&arguments, "source_type", "web");
            let provider = optional_string(&arguments, "provider", "mcp");
            let fetch_status = optional_string(&arguments, "fetch_status", "candidate");
            let read_depth = optional_string(&arguments, "read_depth", "snippet-only");
            let triage_status = optional_string(&arguments, "triage_status", "candidate");
            let reason = arguments
                .get("reason")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| format!("Candidate source for {title}"));
            let priority = arguments
                .get("priority")
                .and_then(Value::as_i64)
                .unwrap_or(50);
            let source = store.upsert_research_source(ResearchSourceInput {
                url: arguments
                    .get("url")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                local_ref: arguments
                    .get("local_ref")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                title,
                source_family,
                source_type,
                provider,
                author: arguments
                    .get("author")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                published_at: arguments
                    .get("published_at")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                language: arguments
                    .get("language")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                priority,
                reason,
                canonical_key: arguments
                    .get("canonical_key")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                fetch_status,
                read_depth: read_depth.clone(),
                metadata: arguments.get("metadata").cloned().unwrap_or(Value::Null),
            })?;
            Ok(json!(store.link_research_source_to_run(
                &run_id,
                &source.id,
                None,
                &triage_status,
                &read_depth,
                arguments.get("notes").and_then(Value::as_str),
            )?))
        }
        "research_source_card_link" => {
            let run_id = required_string(&arguments, "run_id")?;
            let source_card_id = required_string(&arguments, "source_card_id")?;
            let source_family = optional_string(&arguments, "source_family", "uncategorized");
            let read_depth = optional_string(&arguments, "read_depth", "full-text");
            let triage_status = optional_string(&arguments, "triage_status", "must-read-primary");
            Ok(json!(store.link_source_card_to_research_run(
                &run_id,
                &source_card_id,
                &source_family,
                &read_depth,
                &triage_status,
                arguments.get("notes").and_then(Value::as_str),
            )?))
        }
        "research_extraction_prompt" => {
            let run_id = required_string(&arguments, "run_id")?;
            let source_card_id = required_string(&arguments, "source_card_id")?;
            Ok(json!(store.build_research_extraction_prompt(
                &run_id,
                &source_card_id
            )?))
        }
        "research_claims_ingest" => {
            let run_id = required_string(&arguments, "run_id")?;
            let source_card_id = required_string(&arguments, "source_card_id")?;
            let provider = optional_string(&arguments, "provider", "mcp");
            let model = optional_string(&arguments, "model", "manual");
            let output_json = required_string(&arguments, "output_json")?;
            Ok(json!(store.ingest_research_claims_from_model_output(
                &run_id,
                &source_card_id,
                &provider,
                &model,
                &output_json,
            )?))
        }
        "research_claims" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.list_research_claims(&run_id)?))
        }
        "research_clusters" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.build_research_clusters(&run_id)?))
        }
        "research_skeptic_pass" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.run_research_skeptic_pass(&run_id)?))
        }
        "research_report_compile" => {
            let run_id = required_string(&arguments, "run_id")?;
            let saturation_reason = required_string(&arguments, "saturation_reason")?;
            let no_write = arguments
                .get("no_write")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            Ok(json!(store.compile_research_report(
                &run_id,
                &saturation_reason,
                !no_write,
            )?))
        }
        "research_convergence_start" => {
            Ok(json!(store.start_research_convergence(
                research_convergence_start_input_from_mcp(&arguments)?
            )?))
        }
        "research_convergence_step" => Ok(json!(store.run_research_convergence_step(
            research_convergence_step_input_from_mcp(&arguments)?
        )?)),
        "research_convergence_run" => Ok(json!(store.run_research_convergence_to_stop(
            research_convergence_step_input_from_mcp(&arguments)?
        )?)),
        "research_convergence_enqueue" => Ok(json!(store.enqueue_research_convergence_job(
            research_convergence_step_input_from_mcp(&arguments)?
        )?)),
        "research_convergence_status" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.research_convergence_status(&run_id)?))
        }
        "research_iterations" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.list_research_iterations(&run_id)?))
        }
        "research_iteration_read" => {
            let id = required_string(&arguments, "id")?;
            Ok(json!(store.read_research_iteration(&id)?))
        }
        "research_statements" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.list_research_statements(&run_id)?))
        }
        "research_challenges" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.list_research_challenges(&run_id)?))
        }
        "research_convergence_host_search_tasks" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(
                store.list_research_convergence_host_search_tasks(&run_id)?
            ))
        }
        "research_convergence_provider_search" => {
            Ok(json!(store.run_research_convergence_provider_search(
                research_convergence_provider_search_input_from_mcp(&arguments)?
            )?))
        }
        "research_disproofs" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.list_research_disproofs(&run_id)?))
        }
        "research_revisions" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.list_research_revisions(&run_id)?))
        }
        "research_fact_checks" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.list_research_fact_checks(&run_id)?))
        }
        "research_active_fact_check" => Ok(json!(store.run_research_active_fact_check(
            research_active_fact_check_input_from_mcp(&arguments)?
        )?)),
        "research_convergence_close_loop" => Ok(json!(store.run_research_convergence_close_loop(
            research_convergence_close_loop_input_from_mcp(&arguments)?
        )?)),
        "research_convergence_snapshots" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.list_research_convergence_snapshots(&run_id)?))
        }
        "research_convergence_report_compile" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.compile_research_convergence_report(&run_id)?))
        }
        "research_report_judgments" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.list_research_report_judgments(&run_id)?))
        }
        "research_web_search" => {
            let query = required_string(&arguments, "query")?;
            let provider = optional_string(&arguments, "provider", "host");
            let max_results = arguments
                .get("max_results")
                .and_then(Value::as_u64)
                .unwrap_or(5) as usize;
            let timeout_seconds = arguments
                .get("timeout_seconds")
                .and_then(Value::as_u64)
                .unwrap_or(15);
            let config = WebSearchConfig {
                provider,
                max_results,
                endpoint: arguments
                    .get("endpoint")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                api_key: arguments
                    .get("api_key")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                model: arguments
                    .get("model")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                timeout_seconds,
            };
            let write_wiki = arguments
                .get("write_wiki")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            if write_wiki {
                let (response, page_id) = store.web_search_to_wiki(&query, config)?;
                Ok(json!({ "response": response, "page_id": page_id }))
            } else {
                Ok(json!(store.web_search(&query, config)?))
            }
        }
        "research_workflow_create" => {
            let query = required_string(&arguments, "query")?;
            Ok(json!(store.create_research_workflow(&query)?))
        }
        "research_tasks" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.list_research_tasks(&run_id)?))
        }
        "research_role_start" => {
            let run_id = required_string(&arguments, "run_id")?;
            let role = required_string(&arguments, "role")?;
            let host = optional_string(&arguments, "host", "codex");
            let execution_mode = optional_string(&arguments, "execution_mode", "host_sequential");
            let prompt_version = optional_string(&arguments, "prompt_version", "v1");
            Ok(json!(
                store.start_research_role_run(ResearchRoleRunStart {
                    run_id,
                    role,
                    host,
                    host_thread_id: arguments
                        .get("host_thread_id")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    host_subagent_id: arguments
                        .get("host_subagent_id")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    tool_surface: arguments
                        .get("tool_surface")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    prompt_version,
                    prompt_hash: arguments
                        .get("prompt_hash")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    execution_mode,
                    input_artifact_ids: string_array_argument(&arguments, "input_artifact_ids")?,
                })?
            ))
        }
        "research_role_finish" => {
            let role_run_id = required_string(&arguments, "role_run_id")?;
            let status = required_string(&arguments, "status")?;
            Ok(json!(store.finish_research_role_run(
                &role_run_id,
                &status,
                arguments.get("output_artifact_id").and_then(Value::as_str),
                arguments.get("error_kind").and_then(Value::as_str),
                arguments.get("error_message").and_then(Value::as_str),
            )?))
        }
        "research_role_runs" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.list_research_role_runs(&run_id)?))
        }
        "research_artifact_add" => {
            let run_id = required_string(&arguments, "run_id")?;
            let artifact_type = required_string(&arguments, "artifact_type")?;
            let title = required_string(&arguments, "title")?;
            let body = required_string(&arguments, "body")?;
            let metadata = match arguments.get("metadata_json").and_then(Value::as_str) {
                Some(raw) => serde_json::from_str(raw).context("parsing metadata_json")?,
                None => arguments
                    .get("metadata")
                    .cloned()
                    .unwrap_or_else(|| json!({})),
            };
            Ok(json!(
                store.record_research_artifact(ResearchArtifactInput {
                    run_id,
                    role_run_id: arguments
                        .get("role_run_id")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    artifact_type,
                    title,
                    body,
                    metadata,
                })?
            ))
        }
        "research_artifacts" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.list_research_artifacts(&run_id)?))
        }
        "research_artifact_read" => {
            let id = required_string(&arguments, "id")?;
            Ok(json!(store.read_research_artifact(&id)?))
        }
        "research_host_search_record" => {
            let run_id = required_string(&arguments, "run_id")?;
            let query = required_string(&arguments, "query")?;
            let host = optional_string(&arguments, "host", "codex");
            let tool_surface = optional_string(&arguments, "tool_surface", "host-native");
            let results = arguments
                .get("results")
                .and_then(Value::as_array)
                .cloned()
                .context("missing array argument: results")?
                .into_iter()
                .map(serde_json::from_value)
                .collect::<std::result::Result<Vec<ResearchHostSearchResultInput>, _>>()
                .context("parsing host search results")?;
            Ok(json!(
                store.record_research_host_search(ResearchHostSearchInput {
                    run_id,
                    role_run_id: arguments
                        .get("role_run_id")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    host,
                    tool_surface,
                    query,
                    query_intent: arguments
                        .get("query_intent")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    requested_recency: arguments.get("requested_recency").and_then(Value::as_i64),
                    requested_domains: string_array_argument(&arguments, "requested_domains")?,
                    cost_decision_id: arguments
                        .get("cost_decision_id")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    results,
                })?
            ))
        }
        "research_host_searches" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.list_research_host_searches(&run_id)?))
        }
        "research_host_search_read" => {
            let id = required_string(&arguments, "id")?;
            Ok(json!(store.read_research_host_search(&id)?))
        }
        "research_document_extract" => {
            let run_id = required_string(&arguments, "run_id")?;
            let path = required_string(&arguments, "path")?;
            Ok(json!(
                store.extract_research_document_file(ResearchDocumentInput {
                    run_id,
                    research_source_id: arguments
                        .get("research_source_id")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    source_card_id: arguments
                        .get("source_card_id")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    path: PathBuf::from(path),
                    media_type: arguments
                        .get("media_type")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                })?
            ))
        }
        "research_documents" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.list_research_documents(&run_id)?))
        }
        "research_document_read" => {
            let id = required_string(&arguments, "id")?;
            Ok(json!(store.read_research_document(&id)?))
        }
        "research_evidence_pack" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.build_research_evidence_pack(&run_id)?))
        }
        "research_editorial_invoke" => {
            let run_id = required_string(&arguments, "run_id")?;
            let stage = required_string(&arguments, "stage")?;
            Ok(json!(
                store.invoke_research_editorial(ResearchEditorialInvokeInput {
                    run_id,
                    stage,
                    model_provider: optional_string(&arguments, "model_provider", "openai"),
                    model_name: arguments
                        .get("model_name")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    prompt_version: optional_string(&arguments, "prompt_version", "v1"),
                    input_artifact_id: arguments
                        .get("input_artifact_id")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    endpoint: arguments
                        .get("endpoint")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    api_key: arguments
                        .get("api_key")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    timeout_seconds: arguments.get("timeout_seconds").and_then(Value::as_u64),
                },)?
            ))
        }
        "research_editorial_record" => {
            let run_id = required_string(&arguments, "run_id")?;
            let stage = required_string(&arguments, "stage")?;
            let model_name = required_string(&arguments, "model_name")?;
            Ok(json!(
                store.record_research_editorial_run(ResearchEditorialRunInput {
                    run_id,
                    stage,
                    model_provider: optional_string(&arguments, "model_provider", "openai"),
                    model_name,
                    prompt_version: optional_string(&arguments, "prompt_version", "v1"),
                    input_artifact_id: arguments
                        .get("input_artifact_id")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    output_artifact_id: arguments
                        .get("output_artifact_id")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    cost_decision_id: arguments
                        .get("cost_decision_id")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    status: optional_string(&arguments, "status", "completed"),
                    score: arguments.get("score").cloned().unwrap_or_else(|| json!({})),
                    error_message: arguments
                        .get("error_message")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                })?
            ))
        }
        "research_editorial_runs" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.list_research_editorial_runs(&run_id)?))
        }
        "research_editorial_read" => {
            let id = required_string(&arguments, "id")?;
            Ok(json!(store.get_research_editorial_run(&id)?))
        }
        "research_task_complete" => {
            let task_id = required_string(&arguments, "task_id")?;
            let notes = required_string(&arguments, "notes")?;
            Ok(json!(store.complete_research_task(&task_id, &notes)?))
        }
        "research_brief_from_wiki" => {
            let query = required_string(&arguments, "query")?;
            let no_write = arguments
                .get("no_write")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            Ok(json!(
                store.create_research_brief_from_wiki(&query, !no_write)?
            ))
        }
        "research_audit" => {
            let query = required_string(&arguments, "query")?;
            Ok(json!(store.audit_research_output(&query)?))
        }
        "research_runs" => Ok(json!(store.list_research_runs()?)),
        "project_create" => {
            let name = required_string(&arguments, "name")?;
            let summary = required_string(&arguments, "summary")?;
            let aliases = string_array_argument(&arguments, "aliases")?;
            Ok(json!(store.create_project(&name, &summary, &aliases)?))
        }
        "project_list" => Ok(json!(store.list_projects()?)),
        "project_resolve" => {
            let query = required_string(&arguments, "query")?;
            let context_project_id = arguments.get("context_project_id").and_then(Value::as_str);
            Ok(json!(store.resolve_project(&query, context_project_id)?))
        }
        "project_status_record" => {
            let project_id = required_string(&arguments, "project_id")?;
            let status = required_string(&arguments, "status")?;
            let summary = required_string(&arguments, "summary")?;
            let source = optional_string(&arguments, "source", "mcp");
            let thread_ref = arguments.get("thread_ref").and_then(Value::as_str);
            let confidence = arguments
                .get("confidence")
                .and_then(Value::as_f64)
                .unwrap_or(0.5);
            Ok(json!(store.record_project_status(
                &project_id,
                &status,
                &summary,
                &source,
                thread_ref,
                confidence
            )?))
        }
        "project_status_sync_record" => {
            let project_id = required_string(&arguments, "project_id")?;
            let status = required_string(&arguments, "status")?;
            let summary = required_string(&arguments, "summary")?;
            let host = required_string(&arguments, "host")?;
            let thread_id = required_string(&arguments, "thread_id")?;
            let confidence = arguments
                .get("confidence")
                .and_then(Value::as_f64)
                .unwrap_or(0.8);
            let stale_after_seconds = arguments.get("stale_after_seconds").and_then(Value::as_i64);
            Ok(json!(store.record_verified_project_status_sync(
                &project_id,
                &status,
                &summary,
                &host,
                &thread_id,
                confidence,
                stale_after_seconds
            )?))
        }
        "project_status_get" => {
            let project_id = required_string(&arguments, "project_id")?;
            let channel = arguments.get("channel").and_then(Value::as_str);
            let subject = arguments.get("subject").and_then(Value::as_str);
            Ok(json!(store.project_status_report_for_channel(
                &project_id,
                channel,
                subject
            )?))
        }
        "controller_route_text" => {
            let channel = optional_string(&arguments, "channel", "telegram");
            let account_id = arguments.get("account_id").and_then(Value::as_str);
            let conversation_id = required_string(&arguments, "conversation_id")?;
            let sender = required_string(&arguments, "sender")?;
            let text = required_string(&arguments, "text")?;
            Ok(json!(store.controller_route_text(
                &channel,
                account_id,
                &conversation_id,
                &sender,
                &text
            )?))
        }
        "controller_thread_upsert" => {
            let host = required_string(&arguments, "host")?;
            let host_thread_id = required_string(&arguments, "host_thread_id")?;
            let status = optional_string(&arguments, "status", "active");
            Ok(json!(
                store.upsert_controller_thread(
                    &host,
                    &host_thread_id,
                    arguments.get("project_id").and_then(Value::as_str),
                    arguments.get("title").and_then(Value::as_str),
                    arguments.get("cwd").and_then(Value::as_str),
                    arguments.get("branch").and_then(Value::as_str),
                    arguments.get("worktree").and_then(Value::as_str),
                    &status,
                    optional_bool(&arguments, "active", true),
                    optional_bool(&arguments, "archived", false),
                    arguments.get("current_goal").and_then(Value::as_str),
                    arguments.get("latest_summary").and_then(Value::as_str),
                    arguments
                        .get("latest_summary_source")
                        .and_then(Value::as_str),
                    arguments.get("last_activity_at").and_then(Value::as_str),
                )?
            ))
        }
        "controller_thread_list" => Ok(json!(store.list_controller_threads(
            arguments.get("project_id").and_then(Value::as_str),
            arguments.get("status").and_then(Value::as_str),
            optional_usize(&arguments, "limit", 25),
        )?)),
        "controller_thread_get" => {
            let id = required_string(&arguments, "id")?;
            Ok(json!(store.get_controller_thread(&id)?.with_context(
                || format!("controller thread not found: {id}")
            )?))
        }
        "controller_run_create" => {
            let host = optional_string(&arguments, "host", "codex");
            let kind = optional_string(&arguments, "kind", "work");
            let status = optional_string(&arguments, "status", "running");
            let requested_action = required_string(&arguments, "requested_action")?;
            Ok(json!(
                store.create_controller_run(
                    arguments.get("thread_id").and_then(Value::as_str),
                    arguments.get("project_id").and_then(Value::as_str),
                    arguments
                        .get("origin_channel_message_id")
                        .and_then(Value::as_str),
                    &host,
                    arguments.get("host_run_id").and_then(Value::as_str),
                    &kind,
                    &status,
                    &requested_action,
                )?
            ))
        }
        "controller_run_list" => Ok(json!(store.list_controller_runs(
            arguments.get("project_id").and_then(Value::as_str),
            arguments.get("status").and_then(Value::as_str),
            optional_usize(&arguments, "limit", 25),
        )?)),
        "controller_run_get" => {
            let id = required_string(&arguments, "id")?;
            Ok(json!(store.get_controller_run(&id)?.with_context(
                || format!("controller run not found: {id}")
            )?))
        }
        "controller_run_update" => {
            let run_id = required_string(&arguments, "run_id")?;
            let status = required_string(&arguments, "status")?;
            Ok(json!(store.update_controller_run_status(
                &run_id,
                &status,
                arguments.get("host_run_id").and_then(Value::as_str),
            )?))
        }
        "controller_stop" => {
            let run_id = required_string(&arguments, "run_id")?;
            let reason = required_string(&arguments, "reason")?;
            Ok(json!(store.request_controller_stop(&run_id, &reason)?))
        }
        "controller_event_record" => {
            let event_type = required_string(&arguments, "event_type")?;
            let summary = required_string(&arguments, "summary")?;
            let source = optional_string(&arguments, "source", "mcp");
            let data = arguments.get("data").cloned().unwrap_or_else(|| json!({}));
            Ok(json!(store.record_controller_event(
                arguments.get("run_id").and_then(Value::as_str),
                arguments.get("thread_id").and_then(Value::as_str),
                arguments.get("project_id").and_then(Value::as_str),
                &event_type,
                &summary,
                data,
                &source,
            )?))
        }
        "controller_event_list" => Ok(json!(store.list_controller_events(
            arguments.get("run_id").and_then(Value::as_str),
            arguments.get("project_id").and_then(Value::as_str),
            optional_usize(&arguments, "limit", 25),
        )?)),
        "controller_pending_list" => Ok(json!(store.list_controller_pending_actions(
            arguments.get("status").and_then(Value::as_str),
            optional_usize(&arguments, "limit", 25),
        )?)),
        "controller_pending_resolve" => {
            let id = required_string(&arguments, "id")?;
            let status = required_string(&arguments, "status")?;
            Ok(json!(store.resolve_controller_pending_action(
                &id,
                &status,
                arguments.get("thread_id").and_then(Value::as_str),
                arguments.get("run_id").and_then(Value::as_str),
            )?))
        }
        "work_run_start" => {
            let goal = required_string(&arguments, "goal")?;
            let project_id = arguments.get("project_id").and_then(Value::as_str);
            let host_id = arguments.get("host_id").and_then(Value::as_str);
            let thread_id = arguments.get("thread_id").and_then(Value::as_str);
            let agent_surface = optional_string(&arguments, "agent_surface", "mcp");
            Ok(json!(store.start_work_run(
                &goal,
                project_id,
                host_id,
                thread_id,
                &agent_surface
            )?))
        }
        "work_event_record" => {
            let run_id = required_string(&arguments, "run_id")?;
            let event_type = required_string(&arguments, "event_type")?;
            let summary = required_string(&arguments, "summary")?;
            let data = arguments.get("data").cloned().unwrap_or_else(|| json!({}));
            Ok(json!(store.record_work_event(
                &run_id,
                &event_type,
                &summary,
                data
            )?))
        }
        "work_artifact_add" => {
            let run_id = required_string(&arguments, "run_id")?;
            let artifact_type = required_string(&arguments, "artifact_type")?;
            let locator = required_string(&arguments, "locator")?;
            let role = optional_string(&arguments, "role", "evidence");
            let metadata = arguments
                .get("metadata")
                .cloned()
                .unwrap_or_else(|| json!({}));
            Ok(json!(store.add_work_artifact(
                &run_id,
                &artifact_type,
                &locator,
                &role,
                metadata
            )?))
        }
        "work_link_add" => {
            let run_id = required_string(&arguments, "run_id")?;
            let target_type = required_string(&arguments, "target_type")?;
            let target_id = required_string(&arguments, "target_id")?;
            let role = optional_string(&arguments, "role", "evidence");
            let generated_summary = optional_bool(&arguments, "generated_summary", false);
            Ok(json!(store.add_work_link(
                &run_id,
                &target_type,
                &target_id,
                &role,
                generated_summary
            )?))
        }
        "work_run_finish" => {
            let run_id = required_string(&arguments, "run_id")?;
            let status = required_string(&arguments, "status")?;
            let outcome = required_string(&arguments, "outcome")?;
            let validation_summary = arguments.get("validation_summary").and_then(Value::as_str);
            let follow_ups = string_array_argument(&arguments, "follow_ups")?;
            let reusable_lessons = string_array_argument(&arguments, "reusable_lessons")?;
            Ok(json!(store.finish_work_run(
                &run_id,
                &status,
                &outcome,
                validation_summary,
                &follow_ups,
                &reusable_lessons
            )?))
        }
        "work_run_search" => {
            let query = arguments.get("query").and_then(Value::as_str);
            let project_id = arguments.get("project_id").and_then(Value::as_str);
            let status = arguments.get("status").and_then(Value::as_str);
            let limit = arguments.get("limit").and_then(Value::as_u64).unwrap_or(25) as usize;
            Ok(json!(
                store.search_work_runs(query, project_id, status, limit)?
            ))
        }
        "work_run_read" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.read_work_run(&run_id)?))
        }
        "work_run_stale" => {
            let max_age_days = arguments
                .get("max_age_days")
                .and_then(Value::as_i64)
                .unwrap_or(7);
            let limit = arguments.get("limit").and_then(Value::as_u64).unwrap_or(25) as usize;
            Ok(json!(store.list_stale_work_runs(max_age_days, limit)?))
        }
        "work_follow_up_list" => {
            let limit = arguments.get("limit").and_then(Value::as_u64).unwrap_or(25) as usize;
            Ok(json!(store.list_work_follow_ups(limit)?))
        }
        "work_consolidation_candidates" => {
            let limit = arguments.get("limit").and_then(Value::as_u64).unwrap_or(25) as usize;
            Ok(json!(store.list_work_consolidation_candidates(limit)?))
        }
        "work_retrieval_context" => {
            let query = required_string(&arguments, "query")?;
            let stale_after_days = arguments
                .get("stale_after_days")
                .and_then(Value::as_i64)
                .unwrap_or(7);
            let limit = arguments.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize;
            Ok(json!(store.work_retrieval_context(
                &query,
                stale_after_days,
                limit
            )?))
        }
        "work_consolidate" => {
            let run_id = required_string(&arguments, "run_id")?;
            let write_project_status = optional_bool(&arguments, "write_project_status", false);
            Ok(json!(
                store.consolidate_work_run(&run_id, write_project_status)?
            ))
        }
        "procedure_propose_from_work_run" => {
            let run_id = required_string(&arguments, "run_id")?;
            let auto_approve = optional_bool(&arguments, "auto_approve", false);
            Ok(json!(
                store.propose_procedure_from_work_run(&run_id, auto_approve)?
            ))
        }
        "procedure_candidate_create" => {
            let operation = required_string(&arguments, "operation")?;
            let title = required_string(&arguments, "title")?;
            let method = required_string(&arguments, "method")?;
            Ok(json!(
                store.create_procedure_candidate(ProcedureCandidateInput {
                    operation,
                    procedure_id: arguments
                        .get("procedure_id")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    base_version: arguments.get("base_version").and_then(Value::as_i64),
                    title,
                    trigger_context: optional_string(
                        &arguments,
                        "trigger_context",
                        "MCP procedure candidate"
                    ),
                    problem: optional_string(&arguments, "problem", "MCP procedure candidate"),
                    preconditions: string_array_argument(&arguments, "preconditions")?,
                    method,
                    tools: string_array_argument(&arguments, "tools")?,
                    validation_commands: string_array_argument(&arguments, "validation_commands")?,
                    known_risks: string_array_argument(&arguments, "known_risks")?,
                    source_run_ids: string_array_argument(&arguments, "source_run_ids")?,
                    provenance: arguments
                        .get("provenance")
                        .cloned()
                        .unwrap_or_else(|| json!({ "source": "mcp" })),
                    sensitivity: optional_string(&arguments, "sensitivity", "normal"),
                    reason: optional_string(&arguments, "reason", "MCP procedure candidate"),
                })?
            ))
        }
        "procedure_candidate_list" => {
            let status = optional_string(&arguments, "status", "pending");
            Ok(json!(store.list_procedure_candidates(&status)?))
        }
        "procedure_candidate_apply" => {
            let id = required_string(&arguments, "id")?;
            Ok(json!(store.approve_procedure_candidate(&id)?))
        }
        "procedure_candidate_reject" => {
            let id = required_string(&arguments, "id")?;
            let reason = arguments.get("reason").and_then(Value::as_str);
            Ok(
                json!({ "ok": store.reject_procedure_candidate(&id, reason)?, "id": id, "status": "rejected" }),
            )
        }
        "procedure_search" => {
            let query = arguments.get("query").and_then(Value::as_str);
            let status = arguments.get("status").and_then(Value::as_str);
            let limit = arguments.get("limit").and_then(Value::as_u64).unwrap_or(25) as usize;
            Ok(json!(store.search_procedures(query, status, limit)?))
        }
        "procedure_read" => {
            let id = required_string(&arguments, "id")?;
            Ok(json!(store.read_procedure(&id)?))
        }
        "procedure_retrieval_context" => {
            let query = required_string(&arguments, "query")?;
            let limit = arguments.get("limit").and_then(Value::as_u64).unwrap_or(5) as usize;
            Ok(json!(store.procedure_retrieval_context(&query, limit)?))
        }
        "procedure_export_skill" => {
            let id = required_string(&arguments, "id")?;
            let skill_name = required_string(&arguments, "skill_name")?;
            Ok(json!(
                store.export_procedure_to_codex_skill(&id, &skill_name)?
            ))
        }
        "procedure_curate" => Ok(json!(store.curate_procedures()?)),
        "channel_record" => {
            let channel = required_string(&arguments, "channel")?;
            let direction = optional_string(&arguments, "direction", "incoming");
            let sender = required_string(&arguments, "sender")?;
            let body = required_string(&arguments, "body")?;
            let project_id = arguments.get("project_id").and_then(Value::as_str);
            let source_event_id = arguments.get("source_event_id").and_then(Value::as_str);
            Ok(json!(store.record_channel_message(
                &channel,
                &direction,
                &sender,
                &body,
                project_id,
                source_event_id
            )?))
        }
        "channel_list" => Ok(json!(store.list_channel_messages()?)),
        "channel_authorize" => {
            let channel = required_string(&arguments, "channel")?;
            let subject = required_string(&arguments, "subject")?;
            let can_read_projects = optional_bool(&arguments, "can_read_projects", false);
            let can_write_projects = optional_bool(&arguments, "can_write_projects", false);
            let can_send = optional_bool(&arguments, "can_send", false);
            Ok(json!(store.authorize_channel_subject(
                &channel,
                &subject,
                can_read_projects,
                can_write_projects,
                can_send
            )?))
        }
        "channel_authorizations" => Ok(json!(store.list_channel_authorizations()?)),
        "channel_delivery_list" => {
            let message_id = arguments.get("message_id").and_then(Value::as_str);
            Ok(json!(store.list_channel_delivery_attempts(message_id)?))
        }
        "telegram_drain_edge_events" => {
            let max_events = arguments
                .get("max_events")
                .and_then(Value::as_u64)
                .unwrap_or(25) as usize;
            Ok(json!(store.drain_telegram_edge_events(max_events)?))
        }
        "telegram_send_message" => {
            let chat_id = required_string(&arguments, "chat_id")?;
            let text = required_string(&arguments, "text")?;
            let explicit_token = arguments.get("bot_token").and_then(Value::as_str);
            let api_base = arguments.get("api_base").and_then(Value::as_str);
            let token = telegram_bot_token(&store, explicit_token)?;
            Ok(json!(store.send_telegram_message(
                &token, &chat_id, &text, api_base
            )?))
        }
        "email_drain_edge_events" => {
            let max_events = arguments
                .get("max_events")
                .and_then(Value::as_u64)
                .unwrap_or(25) as usize;
            Ok(json!(store.drain_email_edge_events(max_events)?))
        }
        "email_poll_edge" => {
            let max_events = arguments
                .get("max_events")
                .and_then(Value::as_u64)
                .unwrap_or(25) as usize;
            let url = arguments.get("url").and_then(Value::as_str);
            let secret = arguments.get("secret").and_then(Value::as_str);
            let url = edge_remote_url(&store, url)?;
            let secret = edge_remote_secret(&store, secret)?;
            let remote = store.drain_remote_edge_inbox(&url, &secret, max_events)?;
            let email = store.drain_email_edge_events(max_events)?;
            Ok(json!({
                "ok": true,
                "remote": remote,
                "email": email
            }))
        }
        "email_send_message" => {
            let to = required_string(&arguments, "to")?;
            let subject = required_string(&arguments, "subject")?;
            let text = required_string(&arguments, "text")?;
            let from = arguments
                .get("from")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .or_else(|| agent_email_from(&store).ok())
                .unwrap_or_else(|| "agent@example.com".to_string());
            let html = arguments.get("html").and_then(Value::as_str);
            let account_id = arguments.get("account_id").and_then(Value::as_str);
            let api_token = arguments.get("api_token").and_then(Value::as_str);
            let api_base = arguments.get("api_base").and_then(Value::as_str);
            let account_id = cloudflare_account_id(&store, account_id)?;
            let api_token = cloudflare_api_token(&store, api_token)?;
            Ok(json!(store.send_cloudflare_email(
                &account_id,
                &api_token,
                &from,
                &to,
                &subject,
                &text,
                html,
                None,
                api_base
            )?))
        }
        "email_reply_message" => {
            let message_id = required_string(&arguments, "message_id")?;
            let text = required_string(&arguments, "text")?;
            let subject = optional_string(&arguments, "subject", "Re: Arcwell");
            let from = arguments
                .get("from")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .or_else(|| agent_email_from(&store).ok())
                .unwrap_or_else(|| "agent@example.com".to_string());
            let html = arguments.get("html").and_then(Value::as_str);
            let account_id = arguments.get("account_id").and_then(Value::as_str);
            let api_token = arguments.get("api_token").and_then(Value::as_str);
            let api_base = arguments.get("api_base").and_then(Value::as_str);
            let original = store
                .get_channel_message(&message_id)?
                .with_context(|| format!("channel message not found: {message_id}"))?;
            if original.channel != "email" || original.direction != "incoming" {
                bail!("email reply requires an incoming email channel message");
            }
            let to = email_sender_from_channel_body(&original.body)
                .context("incoming email message does not include a sender")?;
            let original_message_id = email_message_id_from_channel_body(&original.body);
            let account_id = cloudflare_account_id(&store, account_id)?;
            let api_token = cloudflare_api_token(&store, api_token)?;
            Ok(json!(store.send_cloudflare_email(
                &account_id,
                &api_token,
                &from,
                &to,
                &subject,
                &text,
                html,
                original_message_id.as_deref(),
                api_base
            )?))
        }
        "digest_candidate_create" => {
            let topic = required_string(&arguments, "topic")?;
            let source_card_ids = string_array_argument(&arguments, "source_card_ids")?;
            Ok(json!(
                store.create_digest_candidate(&topic, &source_card_ids)?
            ))
        }
        "digest_candidate_list" => Ok(json!(store.list_digest_candidates()?)),
        "digest_candidate_approve" => {
            let id = required_string(&arguments, "id")?;
            let reviewed_by = arguments.get("reviewed_by").and_then(Value::as_str);
            let note = arguments.get("note").and_then(Value::as_str);
            Ok(json!(store.approve_digest_candidate(
                &id,
                reviewed_by,
                note
            )?))
        }
        "digest_candidate_reject" => {
            let id = required_string(&arguments, "id")?;
            let reviewed_by = arguments.get("reviewed_by").and_then(Value::as_str);
            let note = arguments.get("note").and_then(Value::as_str);
            Ok(json!(store.reject_digest_candidate(
                &id,
                reviewed_by,
                note
            )?))
        }
        "digest_candidate_delivery_check" => {
            let id = required_string(&arguments, "id")?;
            let channel = required_string(&arguments, "channel")?;
            let subject = required_string(&arguments, "subject")?;
            let target = arguments.get("target").and_then(Value::as_str);
            Ok(json!(store.check_digest_candidate_delivery(
                &id, &channel, &subject, target
            )?))
        }
        "digest_candidate_deliveries" => {
            let candidate_id = arguments.get("candidate_id").and_then(Value::as_str);
            Ok(json!(store.list_digest_deliveries(candidate_id)?))
        }
        "digest_candidate_deliver_telegram" => {
            let id = required_string(&arguments, "id")?;
            let bot_token = required_string(&arguments, "bot_token")?;
            let chat_id = required_string(&arguments, "chat_id")?;
            let idempotency_key = arguments.get("idempotency_key").and_then(Value::as_str);
            let api_base = arguments.get("api_base").and_then(Value::as_str);
            Ok(json!(store.send_digest_candidate_telegram(
                &id,
                &bot_token,
                &chat_id,
                idempotency_key,
                api_base
            )?))
        }
        "digest_candidate_deliver_email" => {
            let id = required_string(&arguments, "id")?;
            let to = required_string(&arguments, "to")?;
            let from = arguments
                .get("from")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .or_else(|| agent_email_from(&store).ok())
                .unwrap_or_else(|| "agent@example.com".to_string());
            let account_id = arguments.get("account_id").and_then(Value::as_str);
            let api_token = arguments.get("api_token").and_then(Value::as_str);
            let account_id = cloudflare_account_id(&store, account_id)?;
            let api_token = cloudflare_api_token(&store, api_token)?;
            let idempotency_key = arguments.get("idempotency_key").and_then(Value::as_str);
            let api_base = arguments.get("api_base").and_then(Value::as_str);
            Ok(json!(store.send_digest_candidate_email(
                &id,
                &account_id,
                &api_token,
                &from,
                &to,
                idempotency_key,
                api_base
            )?))
        }
        "digest_alert_schedule_create" => {
            let name = required_string(&arguments, "name")?;
            let channel = required_string(&arguments, "channel")?;
            let recipient_ref = required_string(&arguments, "recipient_ref")?;
            let min_score = optional_f64_arg(&arguments, "min_score").unwrap_or(0.75);
            let max_candidates = optional_i64_arg(&arguments, "max_candidates").unwrap_or(5);
            let interval_hours = optional_i64_arg(&arguments, "interval_hours").unwrap_or(24);
            let quiet_hours = arguments.get("quiet_hours").cloned();
            let status = arguments
                .get("status")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            Ok(json!(store.create_digest_alert_schedule(
                DigestAlertScheduleInput {
                    name,
                    channel,
                    recipient_ref,
                    min_score,
                    max_candidates,
                    interval_hours,
                    quiet_hours,
                    status,
                }
            )?))
        }
        "digest_alert_schedules" => Ok(json!(store.list_digest_alert_schedules()?)),
        "digest_alert_ticks" => {
            let schedule_id = arguments.get("schedule_id").and_then(Value::as_str);
            Ok(json!(store.list_digest_alert_ticks(schedule_id)?))
        }
        "radar_profile_create" => {
            let name = required_string(&arguments, "name")?;
            let description = optional_string(&arguments, "description", "");
            let window_hours = optional_i64_arg(&arguments, "window_hours").unwrap_or(24);
            let min_score = optional_f64_arg(&arguments, "min_score").unwrap_or(5.0);
            let max_items = arguments.get("max_items").and_then(Value::as_i64);
            let languages = string_array_argument(&arguments, "languages")?;
            let source_selectors = arguments
                .get("source_selectors")
                .cloned()
                .unwrap_or_else(|| json!([]));
            Ok(json!(
                store.create_radar_profile(RadarProfileInput {
                    name,
                    description,
                    window_hours,
                    min_score,
                    max_items,
                    languages,
                    source_selectors,
                    delivery_policy: arguments
                        .get("delivery_policy")
                        .cloned()
                        .unwrap_or_else(|| json!({ "delivery": "manual_only" })),
                    model_policy: arguments
                        .get("model_policy")
                        .cloned()
                        .unwrap_or_else(|| json!({ "model_scoring": "disabled" })),
                    metadata: arguments
                        .get("metadata")
                        .cloned()
                        .unwrap_or_else(|| json!({ "created_from": "mcp" })),
                })?
            ))
        }
        "radar_profile_list" => Ok(json!(store.list_radar_profiles()?)),
        "radar_profile_read" => {
            let profile = required_string(&arguments, "profile")?;
            Ok(json!(store.read_radar_profile(&profile)?))
        }
        "radar_run" => {
            let profile = required_string(&arguments, "profile")?;
            let window_hours = arguments.get("window_hours").and_then(Value::as_i64);
            let fetch_live = optional_bool(&arguments, "fetch_live", false);
            Ok(json!(store.run_radar_profile_with_options(
                &profile,
                window_hours,
                fetch_live,
            )?))
        }
        "radar_enqueue" => {
            let profile = required_string(&arguments, "profile")?;
            let window_hours = arguments.get("window_hours").and_then(Value::as_i64);
            let fetch_live = optional_bool(&arguments, "fetch_live", false);
            Ok(json!(store.enqueue_radar_run_job(
                &profile,
                window_hours,
                fetch_live
            )?))
        }
        "radar_runs" => Ok(json!(store.list_radar_runs()?)),
        "radar_stage_read" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.read_radar_stage(&run_id)?))
        }
        "radar_model_score" => {
            let run_id = required_string(&arguments, "run_id")?;
            let provider = optional_string(&arguments, "provider", "mock");
            let model = arguments.get("model").and_then(Value::as_str);
            let max_items = arguments
                .get("max_items")
                .and_then(Value::as_u64)
                .unwrap_or(10) as usize;
            let endpoint = arguments.get("endpoint").and_then(Value::as_str);
            let api_key = arguments.get("api_key").and_then(Value::as_str);
            Ok(json!(store.score_radar_run_with_model(
                &run_id, &provider, model, max_items, endpoint, api_key
            )?))
        }
        "radar_summarize" => {
            let run_id = required_string(&arguments, "run_id")?;
            let language = optional_string(&arguments, "language", "en");
            let format = optional_string(&arguments, "format", "markdown");
            Ok(json!(
                store.summarize_radar_run(&run_id, &language, &format)?
            ))
        }
        "radar_summary_read" => {
            let run_id = required_string(&arguments, "run_id")?;
            let language = optional_string(&arguments, "language", "en");
            let format = optional_string(&arguments, "format", "markdown");
            Ok(json!(
                store.read_radar_summary(&run_id, &language, &format)?
            ))
        }
        "radar_deliver_summary" => {
            let run_id = required_string(&arguments, "run_id")?;
            let channel = optional_string(&arguments, "channel", "telegram");
            let recipient_ref = required_string(&arguments, "recipient_ref")?;
            let language = optional_string(&arguments, "language", "en");
            let format = optional_string(&arguments, "format", "markdown");
            let idempotency_key = arguments
                .get("idempotency_key")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            let api_base = arguments
                .get("api_base")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            let channel_normalized = channel.trim().to_ascii_lowercase();
            let telegram_bot_token = if channel_normalized == "telegram" {
                Some(telegram_bot_token(
                    &store,
                    arguments.get("bot_token").and_then(Value::as_str),
                )?)
            } else {
                None
            };
            let email_account_id = if channel_normalized == "email" {
                Some(cloudflare_account_id(
                    &store,
                    arguments.get("account_id").and_then(Value::as_str),
                )?)
            } else {
                None
            };
            let email_api_token = if channel_normalized == "email" {
                Some(cloudflare_api_token(
                    &store,
                    arguments.get("api_token").and_then(Value::as_str),
                )?)
            } else {
                None
            };
            let email_from = if channel_normalized == "email" {
                Some(
                    arguments
                        .get("from")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned)
                        .or_else(|| agent_email_from(&store).ok())
                        .unwrap_or_else(|| "agent@example.com".to_string()),
                )
            } else {
                None
            };
            Ok(json!(store.deliver_radar_summary(RadarDeliveryInput {
                run_id,
                language,
                format,
                channel,
                recipient_ref,
                idempotency_key,
                telegram_bot_token,
                email_account_id,
                email_api_token,
                email_from,
                api_base,
            })?))
        }
        "radar_delivery_list" => {
            let run_id = arguments.get("run_id").and_then(Value::as_str);
            Ok(json!(store.list_radar_deliveries(run_id)?))
        }
        "radar_audit_run" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.audit_radar_run(&run_id)?))
        }
        "radar_source_quality" => {
            let run_id = required_string(&arguments, "run_id")?;
            Ok(json!(store.list_radar_source_quality(&run_id)?))
        }
        "radar_source_quality_trends" => {
            let min_windows = optional_usize(&arguments, "min_windows", 2);
            let limit = optional_usize(&arguments, "limit", 50);
            Ok(json!(
                store.list_radar_source_quality_trends(min_windows, limit)?
            ))
        }
        "radar_rebuild_fts" => {
            let run_id = arguments.get("run_id").and_then(Value::as_str);
            Ok(json!({ "rebuilt": store.rebuild_radar_fts(run_id)? }))
        }
        "librarian_expand_topic" => {
            let topic = required_string(&arguments, "topic")?;
            Ok(json!({ "page_id": store.librarian_expand_topic(&topic)? }))
        }
        "ops_snapshot" => Ok(json!(store.ops_snapshot()?)),
        "secret_value_set" => {
            let name = required_string(&arguments, "name")?;
            let value = required_string(&arguments, "value")?;
            let scope = optional_string(&arguments, "scope", "local");
            let provider = arguments.get("provider").and_then(Value::as_str);
            let expires_at = arguments.get("expires_at").and_then(Value::as_str);
            store
                .set_secret_value_with_policy(&name, &value, &scope, provider, expires_at, "mcp")?;
            Ok(json!({ "ok": true, "name": name }))
        }
        "secret_value_list" => Ok(json!(store.list_secret_values()?)),
        "secret_health" => Ok(json!(store.secret_health()?)),
        "secret_value_delete" => {
            let name = required_string(&arguments, "name")?;
            Ok(json!({ "ok": store.delete_secret_value_with_policy(&name, "mcp")?, "name": name }))
        }
        "cursor_list" => Ok(json!(store.list_cursors()?)),
        "cursor_get" => {
            let key = required_string(&arguments, "key")?;
            Ok(json!(store.get_cursor(&key)?))
        }
        "source_card_add" => {
            let title = required_string(&arguments, "title")?;
            let url = required_string(&arguments, "url")?;
            let summary = required_string(&arguments, "summary")?;
            let source_type = optional_string(&arguments, "source_type", "web");
            let provider = optional_string(&arguments, "provider", "mcp");
            let claims = arguments
                .get("claims")
                .cloned()
                .map(serde_json::from_value)
                .transpose()
                .context("invalid claims")?
                .unwrap_or_default();
            let retrieved_at = arguments
                .get("retrieved_at")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            let metadata = arguments.get("metadata").cloned().unwrap_or(Value::Null);
            let card = store.add_source_card(SourceCardInput {
                title,
                url,
                source_type,
                provider,
                summary,
                claims,
                retrieved_at,
                metadata,
            })?;
            if let Some(run_id) = arguments.get("run_id").and_then(Value::as_str) {
                let source_family = optional_string(&arguments, "source_family", "uncategorized");
                let read_depth = optional_string(&arguments, "read_depth", "full-text");
                let triage_status =
                    optional_string(&arguments, "triage_status", "must-read-primary");
                let notes = arguments.get("notes").and_then(Value::as_str);
                let link = store.link_source_card_to_research_run(
                    run_id,
                    &card.id,
                    &source_family,
                    &read_depth,
                    &triage_status,
                    notes,
                )?;
                Ok(json!({ "source_card": card, "research_link": link }))
            } else {
                Ok(json!(card))
            }
        }
        "source_card_search" => {
            let query = required_string(&arguments, "query")?;
            Ok(json!(store.search_source_cards(&query)?))
        }
        "source_card_read" => {
            let id = required_string(&arguments, "id")?;
            Ok(json!(store.read_source_card(&id)?))
        }
        "wiki_ingest_job" => {
            let path = required_string(&arguments, "path")?;
            Ok(json!(store.run_wiki_ingest_file_job(&PathBuf::from(path))?))
        }
        "wiki_ingest_url" => {
            let url = required_string(&arguments, "url")?;
            Ok(json!(store.run_wiki_ingest_url_job(&url)?))
        }
        "wiki_ingest_rendered_page" => {
            let requested_url = required_string(&arguments, "requested_url")?;
            Ok(json!(
                store.run_wiki_ingest_rendered_page_job(RenderedPageSnapshotInput {
                    requested_url,
                    final_url: arguments
                        .get("final_url")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    title: arguments
                        .get("title")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    rendered_html: arguments
                        .get("rendered_html")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    rendered_text: arguments
                        .get("rendered_text")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    captured_at: arguments
                        .get("captured_at")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    browser: arguments
                        .get("browser")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    screenshot_path: arguments
                        .get("screenshot_path")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                },)?
            ))
        }
        "wiki_ingest_dir" => {
            let path = required_string(&arguments, "path")?;
            Ok(json!(store.ingest_wiki_dir(&PathBuf::from(path))?))
        }
        "wiki_import_codex_swift_sources" => {
            let path = required_string(&arguments, "path")?;
            Ok(json!(
                store.import_codex_swift_sources(&PathBuf::from(path))?
            ))
        }
        "wiki_watch_sources" => Ok(json!(store.list_watch_sources()?)),
        "wiki_compile" => {
            let query = required_string(&arguments, "query")?;
            Ok(json!(store.run_wiki_compile_job(&query)?))
        }
        "wiki_expand_page" => {
            let topic = required_string(&arguments, "topic")?;
            Ok(json!(store.run_wiki_expand_page_job(&topic)?))
        }
        "wiki_job_status" => {
            let id = required_string(&arguments, "id")?;
            Ok(json!(store.get_wiki_job(&id)?))
        }
        "wiki_jobs" => Ok(json!(store.list_wiki_jobs()?)),
        "wiki_enqueue_rss" => {
            let url = required_string(&arguments, "url")?;
            Ok(json!(store.enqueue_rss_job(&url)?))
        }
        "wiki_enqueue_github" => {
            let owner = required_string(&arguments, "owner")?;
            let repo = required_string(&arguments, "repo")?;
            let mode = optional_string(&arguments, "mode", "releases");
            let limit = arguments.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize;
            Ok(json!(
                store.enqueue_github_repo_job(&owner, &repo, &mode, limit)?
            ))
        }
        "wiki_enqueue_github_owner" => {
            let owner = required_string(&arguments, "owner")?;
            let limit = arguments.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize;
            Ok(json!(store.enqueue_github_owner_job(&owner, limit)?))
        }
        "wiki_enqueue_arxiv" => {
            let query = required_string(&arguments, "query")?;
            let limit = arguments.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize;
            Ok(json!(store.enqueue_arxiv_search_job(&query, limit)?))
        }
        "x_import_json_file" => {
            let path = required_string(&arguments, "path")?;
            Ok(json!(store.import_x_json_file(&PathBuf::from(path))?))
        }
        "x_import_archive" => {
            let path = required_string(&arguments, "path")?;
            let select = arguments
                .get("select")
                .and_then(Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .filter_map(Value::as_str)
                        .map(ToOwned::to_owned)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let limit = arguments
                .get("limit")
                .and_then(Value::as_u64)
                .map(|value| value as usize)
                .unwrap_or(10_000);
            Ok(json!(store.import_x_archive(
                &PathBuf::from(path),
                &select,
                limit
            )?))
        }
        "x_discover_archives" => {
            let dirs = arguments
                .get("dirs")
                .and_then(Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .filter_map(Value::as_str)
                        .map(PathBuf::from)
                        .collect::<Vec<_>>()
                })
                .or_else(|| {
                    arguments
                        .get("dir")
                        .and_then(Value::as_str)
                        .map(|dir| vec![PathBuf::from(dir)])
                })
                .unwrap_or_default();
            let limit = arguments
                .get("limit")
                .and_then(Value::as_u64)
                .map(|value| value as usize)
                .unwrap_or(25);
            Ok(json!(store.discover_x_archives(&dirs, limit)?))
        }
        "x_export_portable" => {
            let out = required_string(&arguments, "out")?;
            Ok(json!(store.export_x_portable(&PathBuf::from(out))?))
        }
        "x_validate_portable" => {
            let dir = required_string(&arguments, "dir")?;
            Ok(json!(store.validate_x_portable(&PathBuf::from(dir))?))
        }
        "x_import_portable" => {
            let dir = required_string(&arguments, "dir")?;
            Ok(json!(store.import_x_portable(&PathBuf::from(dir))?))
        }
        "x_recent_search" => {
            let query = required_string(&arguments, "query")?;
            let max_results = arguments
                .get("max_results")
                .and_then(Value::as_u64)
                .unwrap_or(10) as usize;
            Ok(json!(store.x_recent_search(&query, max_results)?))
        }
        "x_enqueue_recent_search" => {
            let query = required_string(&arguments, "query")?;
            let max_results = arguments
                .get("max_results")
                .and_then(Value::as_u64)
                .unwrap_or(10) as usize;
            Ok(json!(
                store.enqueue_x_recent_search_job(&query, max_results)?
            ))
        }
        "x_import_bookmarks" => {
            let bookmark_days = arguments
                .get("bookmark_days")
                .and_then(Value::as_i64)
                .unwrap_or(92);
            let max_bookmarks = arguments
                .get("max_bookmarks")
                .and_then(Value::as_u64)
                .unwrap_or(100) as usize;
            Ok(json!(
                store.x_import_bookmarks(bookmark_days, max_bookmarks)?
            ))
        }
        "x_schedule_bookmarks" => {
            let bookmark_days = arguments
                .get("bookmark_days")
                .and_then(Value::as_i64)
                .unwrap_or(92);
            let max_bookmarks = arguments
                .get("max_bookmarks")
                .and_then(Value::as_u64)
                .unwrap_or(1000) as usize;
            let cadence = arguments
                .get("cadence")
                .and_then(Value::as_str)
                .unwrap_or("warm");
            let status = arguments
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("active");
            Ok(json!(store.schedule_x_bookmark_import(
                bookmark_days,
                max_bookmarks,
                cadence,
                status,
            )?))
        }
        "x_import_following_watch_sources" => {
            let max_users = arguments
                .get("max_users")
                .and_then(Value::as_u64)
                .unwrap_or(1000) as usize;
            Ok(json!(store.x_import_following_watch_sources(max_users)?))
        }
        "x_rebuild_definitive_watch_sources" => {
            let bookmark_days = arguments
                .get("bookmark_days")
                .and_then(Value::as_i64)
                .unwrap_or(92);
            let max_bookmarks = arguments
                .get("max_bookmarks")
                .and_then(Value::as_u64)
                .unwrap_or(1000) as usize;
            let max_recent_follows = arguments
                .get("max_recent_follows")
                .and_then(Value::as_u64)
                .unwrap_or(100) as usize;
            Ok(json!(store.x_rebuild_definitive_watch_sources(
                bookmark_days,
                max_bookmarks,
                max_recent_follows,
            )?))
        }
        "x_monitor_watch_sources" => {
            let max_sources = arguments
                .get("max_sources")
                .and_then(Value::as_u64)
                .unwrap_or(25) as usize;
            let max_results_per_source = arguments
                .get("max_results_per_source")
                .and_then(Value::as_u64)
                .unwrap_or(10) as usize;
            Ok(json!(store.x_monitor_watch_sources(
                max_sources,
                max_results_per_source,
            )?))
        }
        "x_repair_health" => {
            let defer_rate_limited_hours = arguments
                .get("defer_rate_limited_hours")
                .and_then(Value::as_i64)
                .unwrap_or(24);
            let limit = arguments
                .get("limit")
                .and_then(Value::as_u64)
                .unwrap_or(10000) as usize;
            Ok(json!(
                store.x_repair_health(defer_rate_limited_hours, limit)?
            ))
        }
        "x_oauth_probe" => {
            let search_query = arguments.get("search_query").and_then(Value::as_str);
            Ok(json!(store.x_oauth_probe(search_query)?))
        }
        "x_oauth_authorize_url" => {
            let client_id = store
                .resolve_x_oauth_client_id(arguments.get("client_id").and_then(Value::as_str))?;
            let redirect_uri = store.resolve_x_oauth_redirect_uri(
                arguments.get("redirect_uri").and_then(Value::as_str),
            )?;
            let scopes = arguments
                .get("scopes")
                .and_then(Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .filter_map(Value::as_str)
                        .map(ToOwned::to_owned)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            Ok(json!(store.x_oauth_authorize_url(
                &client_id,
                &redirect_uri,
                &scopes
            )?))
        }
        "x_oauth_exchange_code" => {
            let client_id = store
                .resolve_x_oauth_client_id(arguments.get("client_id").and_then(Value::as_str))?;
            let redirect_uri = store.resolve_x_oauth_redirect_uri(
                arguments.get("redirect_uri").and_then(Value::as_str),
            )?;
            let code = required_string(&arguments, "code")?;
            let code_verifier = required_string(&arguments, "code_verifier")?;
            let client_secret = arguments.get("client_secret").and_then(Value::as_str);
            Ok(json!(store.x_oauth_exchange_code(
                &client_id,
                &redirect_uri,
                &code,
                &code_verifier,
                client_secret
            )?))
        }
        "x_oauth_refresh" => {
            let client_id = store
                .resolve_x_oauth_client_id(arguments.get("client_id").and_then(Value::as_str))?;
            let client_secret = arguments.get("client_secret").and_then(Value::as_str);
            Ok(json!(store.x_oauth_refresh(&client_id, client_secret)?))
        }
        "x_oauth_revoke" => {
            let name = arguments
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("X_BEARER_TOKEN");
            let client_id = store
                .resolve_x_oauth_client_id(arguments.get("client_id").and_then(Value::as_str))?;
            let client_secret = arguments.get("client_secret").and_then(Value::as_str);
            let token_type_hint = arguments.get("token_type_hint").and_then(Value::as_str);
            let delete_local = arguments
                .get("delete_local")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            Ok(json!(store.x_oauth_revoke(
                name,
                &client_id,
                client_secret,
                token_type_hint,
                delete_local,
            )?))
        }
        "x_list" => {
            let query = arguments.get("query").and_then(Value::as_str);
            let source = arguments.get("source").and_then(Value::as_str);
            let limit = arguments
                .get("limit")
                .and_then(Value::as_u64)
                .map(|value| value as usize);
            Ok(json!(store.list_x_items_filtered(query, source, limit)?))
        }
        "x_bookmarks" => {
            let query = arguments.get("query").and_then(Value::as_str);
            let limit = arguments
                .get("limit")
                .and_then(Value::as_u64)
                .map(|value| value as usize);
            Ok(json!(store.list_x_items_filtered(
                query,
                Some("bookmark"),
                limit
            )?))
        }
        "x_search_tweets" => {
            let query = required_string(&arguments, "query")?;
            let limit = arguments
                .get("limit")
                .and_then(Value::as_u64)
                .map(|value| value as usize)
                .unwrap_or(20);
            Ok(json!(store.search_x_tweets(&query, limit)?))
        }
        "x_research" => {
            let query = required_string(&arguments, "query")?;
            let limit = arguments
                .get("limit")
                .and_then(Value::as_u64)
                .map(|value| value as usize)
                .unwrap_or(10);
            Ok(json!(store.x_research_brief(&query, limit)?))
        }
        "x_thread" => {
            let x_id = required_string(&arguments, "x_id")?;
            let max_depth = arguments
                .get("max_depth")
                .and_then(Value::as_u64)
                .map(|value| value as usize)
                .unwrap_or(50);
            Ok(json!(store.x_thread(&x_id, max_depth)?))
        }
        "x_extract_links" => {
            let limit = arguments
                .get("limit")
                .and_then(Value::as_u64)
                .map(|value| value as usize)
                .unwrap_or(1000);
            Ok(json!(store.x_extract_links(limit)?))
        }
        "x_expand_links" => {
            let limit = arguments
                .get("limit")
                .and_then(Value::as_u64)
                .map(|value| value as usize)
                .unwrap_or(100);
            Ok(json!(store.x_expand_links(limit)?))
        }
        "x_links" => {
            let query = arguments.get("query").and_then(Value::as_str);
            let limit = arguments
                .get("limit")
                .and_then(Value::as_u64)
                .map(|value| value as usize)
                .unwrap_or(100);
            Ok(json!(store.x_links(query, limit)?))
        }
        "x_repair_projections" => {
            let limit = arguments
                .get("limit")
                .and_then(Value::as_u64)
                .map(|value| value as usize)
                .unwrap_or(1000);
            Ok(json!(store.x_repair_projections(limit)?))
        }
        "x_stats" => Ok(json!(store.x_stats()?)),
        "x_report" => {
            let query = arguments.get("query").and_then(Value::as_str);
            Ok(json!(store.x_report(query)?))
        }
        "wiki_ingest_file" => {
            let path = required_string(&arguments, "path")?;
            let id = store.ingest_wiki_file(&PathBuf::from(path))?;
            Ok(json!({ "ok": true, "id": id }))
        }
        "wiki_search" => {
            let query = required_string(&arguments, "query")?;
            Ok(json!(store.search_wiki_pages(&query)?))
        }
        "wiki_read" => {
            let id = required_string(&arguments, "id")?;
            Ok(json!(store.read_wiki_page(&id)?))
        }
        _ => bail!("unknown tool: {name}"),
    }
}
