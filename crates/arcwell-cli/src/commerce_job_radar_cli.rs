use crate::*;

pub(crate) fn commerce(store: Store, args: CommerceCommand) -> Result<()> {
    match args.command {
        CommerceSubcommand::Capabilities => print_json(&commerce_capabilities(store.paths())),
        CommerceSubcommand::ConfigSet {
            run_id,
            domain_profile,
            target_qualified_count,
            geography,
            freshness_window,
            allowed_private_context_sources,
            allowed_public_source_families,
            allow_marketplaces,
            allow_chrome_profile,
            max_provider_calls,
            max_browser_pages,
            max_cost_usd,
            stop_rules_json,
        } => print_json(&store.record_commerce_run_config(CommerceRunConfigInput {
            run_id,
            domain_profile,
            target_qualified_count,
            geography,
            freshness_window,
            allowed_private_context_sources,
            allowed_public_source_families,
            allow_marketplaces,
            allow_chrome_profile,
            max_provider_calls,
            max_browser_pages,
            max_cost_usd,
            stop_rules: parse_json_arg(&stop_rules_json, "--stop-rules-json")?,
        })?),
        CommerceSubcommand::Config { run_id } => {
            print_json(&store.read_commerce_run_config(&run_id)?)
        }
        CommerceSubcommand::CandidateAdd {
            run_id,
            domain,
            source_url,
            retailer_or_provider,
            title,
            normalized_item_key,
            variant_key,
            price,
            currency,
            geography,
            candidate_status,
            score,
            score_reasons_json,
            disqualification_reasons_json,
            metadata_json,
        } => print_json(&store.record_commerce_candidate(CommerceCandidateInput {
            run_id,
            domain,
            source_url,
            retailer_or_provider,
            title,
            normalized_item_key,
            variant_key,
            price,
            currency,
            geography,
            candidate_status,
            score,
            score_reasons: parse_json_arg(&score_reasons_json, "--score-reasons-json")?,
            disqualification_reasons: parse_json_arg(
                &disqualification_reasons_json,
                "--disqualification-reasons-json",
            )?,
            metadata: parse_json_arg(&metadata_json, "--metadata-json")?,
        })?),
        CommerceSubcommand::Candidates { run_id } => {
            print_json(&store.list_commerce_candidates(&run_id)?)
        }
        CommerceSubcommand::AvailabilityProofAdd {
            run_id,
            candidate_id,
            proof_method,
            variant_key,
            variant_label,
            availability_state,
            visible_evidence,
            selector_or_dom_hint,
            screenshot_artifact_id,
            page_snapshot_artifact_id,
            confidence,
            caveats_json,
            checked_at,
        } => print_json(&store.record_commerce_availability_proof(
            CommerceAvailabilityProofInput {
                run_id,
                candidate_id,
                proof_method,
                variant_key,
                variant_label,
                availability_state,
                visible_evidence,
                selector_or_dom_hint,
                screenshot_artifact_id,
                page_snapshot_artifact_id,
                confidence,
                caveats: parse_json_arg(&caveats_json, "--caveats-json")?,
                checked_at,
            },
        )?),
        CommerceSubcommand::AvailabilityProofs { run_id } => {
            print_json(&store.list_commerce_availability_proofs(&run_id)?)
        }
        CommerceSubcommand::RenderedPageCheck {
            run_id,
            candidate_id,
            variant_key,
            variant_label,
            requested_url,
            final_url,
            title,
            rendered_html,
            rendered_html_file,
            rendered_text,
            rendered_text_file,
            captured_at,
            browser,
            screenshot_path,
            selector_or_dom_hint,
            chrome_profile_required,
        } => print_json(&store.record_commerce_rendered_page_check(
            CommerceRenderedPageCheckInput {
                run_id,
                candidate_id,
                variant_key,
                variant_label,
                snapshot: RenderedPageSnapshotInput {
                    requested_url,
                    final_url,
                    title,
                    rendered_html: optional_inline_or_file(rendered_html, rendered_html_file)?,
                    rendered_text: optional_inline_or_file(rendered_text, rendered_text_file)?,
                    captured_at,
                    browser,
                    screenshot_path,
                },
                selector_or_dom_hint,
                chrome_profile_required,
            },
        )?),
        CommerceSubcommand::ContextFactAdd {
            run_id,
            fact_key,
            fact_kind,
            redacted_value,
            source_family,
            source_ref,
            confidence,
            user_confirmed,
            may_persist_to_memory,
            metadata_json,
        } => print_json(
            &store.record_commerce_context_fact(CommerceContextFactInput {
                run_id,
                fact_key,
                fact_kind,
                redacted_value,
                source_family,
                source_ref,
                confidence,
                user_confirmed,
                may_persist_to_memory,
                metadata: parse_json_arg(&metadata_json, "--metadata-json")?,
            })?,
        ),
        CommerceSubcommand::ContextFacts { run_id } => {
            print_json(&store.list_commerce_context_facts(&run_id)?)
        }
        CommerceSubcommand::ContextPacket { run_id } => {
            print_json(&store.compile_commerce_context_packet(&run_id)?)
        }
        CommerceSubcommand::VerificationAttemptAdd {
            run_id,
            candidate_id,
            method,
            result,
            error_kind,
            final_url,
            http_status,
            browser_required,
            chrome_profile_required,
            artifact_ids,
            next_action,
            attempted_at,
        } => print_json(&store.record_commerce_verification_attempt(
            CommerceVerificationAttemptInput {
                run_id,
                candidate_id,
                method,
                result,
                error_kind,
                final_url,
                http_status,
                browser_required,
                chrome_profile_required,
                artifact_ids,
                next_action,
                attempted_at,
            },
        )?),
        CommerceSubcommand::VerificationAttempts { run_id } => {
            print_json(&store.list_commerce_verification_attempts(&run_id)?)
        }
        CommerceSubcommand::ReportJudgmentAdd {
            run_id,
            decision,
            blocking_findings_json,
            non_blocking_findings_json,
            claims_checked_json,
            availability_proofs_checked_json,
            privacy_review_json,
            remaining_risks_json,
        } => print_json(
            &store.record_commerce_report_judgment(CommerceReportJudgmentInput {
                run_id,
                decision,
                blocking_findings: parse_json_arg(
                    &blocking_findings_json,
                    "--blocking-findings-json",
                )?,
                non_blocking_findings: parse_json_arg(
                    &non_blocking_findings_json,
                    "--non-blocking-findings-json",
                )?,
                claims_checked: parse_json_arg(&claims_checked_json, "--claims-checked-json")?,
                availability_proofs_checked: parse_json_arg(
                    &availability_proofs_checked_json,
                    "--availability-proofs-checked-json",
                )?,
                privacy_review: parse_json_arg(&privacy_review_json, "--privacy-review-json")?,
                remaining_risks: parse_json_arg(&remaining_risks_json, "--remaining-risks-json")?,
            })?,
        ),
        CommerceSubcommand::ReportJudgments { run_id } => {
            print_json(&store.list_commerce_report_judgments(&run_id)?)
        }
        CommerceSubcommand::Report { run_id } => {
            print_json(&store.compile_commerce_report(&run_id)?)
        }
    }
}

pub(crate) fn job(store: Store, args: JobCommand) -> Result<()> {
    match args.command {
        JobSubcommand::ProfileAdd {
            label,
            current_resume_source,
            linkedin_source,
            github_profile,
            blog_url,
            metadata_json,
        } => print_json(
            &store.record_job_candidate_profile(JobCandidateProfileInput {
                label,
                current_resume_source,
                linkedin_source,
                github_profile,
                blog_url,
                metadata: parse_json_arg(&metadata_json, "--metadata-json")?,
            })?,
        ),
        JobSubcommand::Profiles => print_json(&store.list_job_candidate_profiles()?),
        JobSubcommand::Profile { profile_id } => {
            print_json(&store.read_job_candidate_profile(&profile_id)?)
        }
        JobSubcommand::Import { path } => {
            let raw = fs::read_to_string(&path)
                .with_context(|| format!("reading job import packet {}", path.display()))?;
            let input: JobImportBatchInput = serde_json::from_str(&raw)
                .with_context(|| format!("parsing job import packet {}", path.display()))?;
            print_json(&store.import_job_batch(input)?)
        }
        JobSubcommand::EvidenceAdd {
            profile_id,
            title,
            evidence_type,
            visibility,
            summary,
            proof_url,
            local_path,
            source_date,
            confidence,
            tags,
            safe_application_text,
            unsafe_terms,
            metadata_json,
        } => print_json(&store.record_job_evidence_card(JobEvidenceCardInput {
            profile_id,
            title,
            evidence_type,
            visibility,
            summary,
            proof_url,
            local_path,
            source_date,
            confidence,
            tags,
            safe_application_text,
            unsafe_terms,
            metadata: parse_json_arg(&metadata_json, "--metadata-json")?,
        })?),
        JobSubcommand::Evidence { evidence_id } => {
            print_json(&store.read_job_evidence_card(&evidence_id)?)
        }
        JobSubcommand::EvidenceList { profile_id } => {
            print_json(&store.list_job_evidence_cards(&profile_id)?)
        }
        JobSubcommand::EvidenceReview { profile_id } => {
            print_json(&store.compile_job_evidence_review_report(&profile_id)?)
        }
        JobSubcommand::EvidenceClaimAdd {
            evidence_card_id,
            claim,
            claim_kind,
            proof_level,
            can_use_in_resume,
            can_use_in_outreach,
            can_use_in_interview,
        } => print_json(&store.record_job_evidence_claim(JobEvidenceClaimInput {
            evidence_card_id,
            claim,
            claim_kind,
            proof_level,
            can_use_in_resume,
            can_use_in_outreach,
            can_use_in_interview,
        })?),
        JobSubcommand::PrivacyRuleAdd {
            pattern,
            rule_type,
            severity,
            replacement_guidance,
        } => print_json(&store.record_job_privacy_rule(JobPrivacyRuleInput {
            pattern,
            rule_type,
            severity,
            replacement_guidance,
        })?),
        JobSubcommand::PrivacyCheck {
            artifact_type,
            artifact_id,
            text,
            blocked_terms,
        } => print_json(&store.check_job_privacy_text(
            &artifact_type,
            artifact_id.as_deref(),
            &text,
            &blocked_terms,
        )?),
        JobSubcommand::SourceAdd {
            source_family,
            name,
            url,
            market_scope,
            refresh_policy,
            metadata_json,
        } => print_json(&store.record_job_source(JobSourceInput {
            source_family,
            name,
            url,
            market_scope,
            refresh_policy,
            metadata: parse_json_arg(&metadata_json, "--metadata-json")?,
        })?),
        JobSubcommand::SourceHealthAdd {
            source_id,
            status,
            http_status,
            error_code,
            fetched_count,
            accepted_count,
            rejected_count,
            note,
        } => print_json(&store.record_job_source_health(JobSourceHealthInput {
            source_id,
            status,
            http_status,
            error_code,
            fetched_count,
            accepted_count,
            rejected_count,
            note,
        })?),
        JobSubcommand::SourceRefresh {
            source_id,
            body,
            body_path,
            fetched_url,
            fetch_live,
        } => {
            let body = match (body, body_path) {
                (Some(_), Some(_)) => bail!("use either --body or --body-path, not both"),
                (Some(body), None) => Some(body),
                (None, Some(path)) => Some(fs::read_to_string(&path).with_context(|| {
                    format!("reading job source refresh body from {}", path.display())
                })?),
                (None, None) => None,
            };
            print_json(&store.run_job_source_refresh(JobSourceRefreshInput {
                source_id,
                body,
                fetched_url,
                fetch_live,
            })?)
        }
        JobSubcommand::RadarSchedule {
            profile_id,
            scope,
            source_ids,
            fetch_live,
            source_snapshots_json,
            source_snapshots_path,
            cadence,
            status,
            email_to,
            delivery_idempotency_key,
        } => {
            let source_snapshots =
                parse_json_arg_or_file(&source_snapshots_json, source_snapshots_path.as_ref())?;
            let delivery = job_radar_email_delivery(email_to.as_deref(), delivery_idempotency_key)?;
            print_json(&store.schedule_job_radar_refresh_with_delivery(
                &profile_id,
                &scope,
                source_ids,
                fetch_live,
                source_snapshots,
                &cadence,
                &status,
                delivery,
            )?)
        }
        JobSubcommand::RadarEnqueue {
            profile_id,
            scope,
            source_ids,
            fetch_live,
            source_snapshots_json,
            source_snapshots_path,
            email_to,
            delivery_idempotency_key,
        } => {
            let source_snapshots =
                parse_json_arg_or_file(&source_snapshots_json, source_snapshots_path.as_ref())?;
            let delivery = job_radar_email_delivery(email_to.as_deref(), delivery_idempotency_key)?;
            print_json(&store.enqueue_job_radar_refresh_job_with_delivery(
                &profile_id,
                &scope,
                source_ids,
                fetch_live,
                source_snapshots,
                delivery,
            )?)
        }
        JobSubcommand::RoleAdd {
            company,
            role_title,
            canonical_url,
            source_family,
            source_url,
            source_confidence,
            date_accessed,
            posting_freshness,
            location,
            work_mode,
            company_stage_or_size,
            role_seniority,
            core_requirements,
            implied_business_problem,
            why_they_might_need_user,
            evidence_card_ids,
            gaps_or_blockers,
            cluster,
            current_status,
            metadata_json,
        } => print_json(&store.record_job_role_card(JobRoleCardInput {
            company,
            role_title,
            canonical_url,
            source_family,
            source_url,
            source_confidence,
            date_accessed,
            posting_freshness,
            location,
            work_mode,
            company_stage_or_size,
            role_seniority,
            core_requirements,
            implied_business_problem,
            why_they_might_need_user,
            evidence_card_ids,
            gaps_or_blockers,
            cluster,
            current_status,
            metadata: parse_json_arg(&metadata_json, "--metadata-json")?,
        })?),
        JobSubcommand::Role { role_id } => print_json(&store.read_job_role_card(&role_id)?),
        JobSubcommand::Roles => print_json(&store.list_job_role_cards()?),
        JobSubcommand::RoleSourceLinkAdd {
            role_id,
            source_id,
            source_url,
            confidence,
            evidence_excerpt,
        } => print_json(&store.record_job_role_source_link(JobRoleSourceLinkInput {
            role_id,
            source_id,
            source_url,
            confidence,
            evidence_excerpt,
        })?),
        JobSubcommand::ScoreAdd {
            role_id,
            profile_id,
            scorer,
            role_fit,
            domain_fit,
            evidence_fit,
            geo_work_fit,
            stage_fit,
            practical_odds,
            interest_energy,
            blockers,
            evidence_card_ids,
            explanation,
        } => print_json(&store.record_job_fit_score(JobFitScoreInput {
            role_id,
            profile_id,
            scorer,
            role_fit,
            domain_fit,
            evidence_fit,
            geo_work_fit,
            stage_fit,
            practical_odds,
            interest_energy,
            blockers,
            evidence_card_ids,
            explanation,
        })?),
        JobSubcommand::Shortlist { profile_id } => {
            print_json(&store.compile_job_shortlist(&profile_id)?)
        }
        JobSubcommand::OutreachReadiness { profile_id, limit } => {
            print_json(&store.compile_job_outreach_readiness_report(&profile_id, limit)?)
        }
        JobSubcommand::CompanyTargets {
            profile_id,
            market,
            limit,
        } => print_json(&store.compile_job_company_target_report(
            &profile_id,
            market.as_deref(),
            limit,
        )?),
        JobSubcommand::SkepticFindingAdd {
            role_id,
            severity,
            finding_type,
            finding,
            next_action,
        } => print_json(&store.record_job_skeptic_finding(JobSkepticFindingInput {
            role_id,
            severity,
            finding_type,
            finding,
            next_action,
        })?),
        JobSubcommand::PacketCreate {
            role_id,
            profile_id,
            evidence_card_ids,
            resume_emphasis,
            tailored_bullets,
            outreach_note,
            proof_links_json,
            likely_objections,
            interview_stories,
            questions_to_ask,
            reviewer_note,
        } => print_json(
            &store.create_job_application_packet(JobApplicationPacketInput {
                role_id,
                profile_id,
                evidence_card_ids,
                resume_emphasis,
                tailored_bullets,
                outreach_note,
                proof_links: parse_json_arg(&proof_links_json, "--proof-links-json")?,
                likely_objections,
                interview_stories,
                questions_to_ask,
                reviewer_note,
            })?,
        ),
        JobSubcommand::Packet { packet_id } => {
            print_json(&store.read_job_application_packet(&packet_id)?)
        }
        JobSubcommand::PacketApprove {
            packet_id,
            reviewer_note,
        } => print_json(&store.update_job_application_packet_status(
            JobApplicationPacketStatusInput {
                packet_id,
                status: "approved".to_string(),
                reviewer_note: Some(reviewer_note),
            },
        )?),
        JobSubcommand::PacketExport { packet_id, out } => {
            print_json(&store.export_job_application_packet(&packet_id, &out)?)
        }
        JobSubcommand::PacketExportSet {
            profile_id,
            packet_ids,
            out,
        } => print_json(&store.export_job_application_packet_set(&profile_id, packet_ids, &out)?),
        JobSubcommand::CompanyAdd {
            company_name,
            website_url,
            source_family,
            market,
            stage,
            funding_signal,
            product_category,
            technical_audience,
            developer_facing_score,
            london_relevance,
            remote_maturity,
            hiring_page_url,
            founder_or_team_signal,
            metadata_json,
        } => print_json(&store.record_job_company_card(JobCompanyCardInput {
            company_name,
            website_url,
            source_family,
            market,
            stage,
            funding_signal,
            product_category,
            technical_audience,
            developer_facing_score,
            london_relevance,
            remote_maturity,
            hiring_page_url,
            founder_or_team_signal,
            metadata: parse_json_arg(&metadata_json, "--metadata-json")?,
        })?),
        JobSubcommand::ContactAdd {
            name,
            company_id,
            role_title,
            public_profile_url,
            source_url,
            relationship_status,
            relevance,
            note,
        } => print_json(&store.record_job_contact(JobContactInput {
            name,
            company_id,
            role_title,
            public_profile_url,
            source_url,
            relationship_status,
            relevance,
            note,
        })?),
        JobSubcommand::IntroAdd {
            role_id,
            contact_id,
            path_type,
            confidence,
            next_action,
            status,
        } => print_json(&store.record_job_intro_path(JobIntroPathInput {
            role_id,
            contact_id,
            path_type,
            confidence,
            next_action,
            status,
        })?),
        JobSubcommand::SearchRunAdd {
            profile_id,
            scope,
            proof_level,
            source_count,
            role_count,
            new_role_count,
            stale_role_count,
            error_count,
            report_artifact_id,
            completed_at,
        } => print_json(&store.record_job_search_run(JobSearchRunInput {
            profile_id,
            scope,
            proof_level,
            source_count,
            role_count,
            new_role_count,
            stale_role_count,
            error_count,
            report_artifact_id,
            completed_at,
        })?),
        JobSubcommand::RoleStatusAdd {
            role_id,
            run_id,
            status,
            previous_tier,
            current_tier,
            note,
        } => print_json(
            &store.record_job_role_status_event(JobRoleStatusEventInput {
                role_id,
                run_id,
                status,
                previous_tier,
                current_tier,
                note,
            })?,
        ),
        JobSubcommand::ApplicationRecord {
            role_id,
            packet_id,
            status,
            applied_at,
            follow_up_at,
            outcome_note,
        } => print_json(&store.record_job_application(JobApplicationInput {
            role_id,
            packet_id,
            status,
            applied_at,
            follow_up_at,
            outcome_note,
        })?),
        JobSubcommand::Refresh {
            profile_id,
            scope,
            observed_role_ids,
            stale_role_ids,
            closed_role_ids,
            source_health_ids,
            proof_level,
            report_artifact_id,
        } => print_json(&store.run_job_manual_refresh(JobManualRefreshInput {
            profile_id,
            scope,
            observed_role_ids,
            stale_role_ids,
            closed_role_ids,
            source_health_ids,
            proof_level,
            report_artifact_id,
        })?),
        JobSubcommand::RefreshAudit {
            profile_id,
            scope,
            min_elapsed_hours,
        } => print_json(&store.audit_job_refresh_history(
            &profile_id,
            &scope,
            Some(min_elapsed_hours),
        )?),
        JobSubcommand::OperationalAudit {
            profile_id,
            scope,
            min_elapsed_hours,
        } => print_json(&store.audit_job_operational_readiness(
            &profile_id,
            &scope,
            Some(min_elapsed_hours),
        )?),
        JobSubcommand::WeeklyReport { profile_id, scope } => {
            print_json(&store.compile_job_weekly_report(&profile_id, &scope)?)
        }
        JobSubcommand::WeeklyReportDeliveryPrepare {
            report_id,
            channel,
            subject,
            target,
            idempotency_key,
        } => print_json(&store.prepare_job_weekly_report_delivery(
            JobWeeklyReportDeliveryInput {
                report_id,
                channel,
                subject,
                target,
                idempotency_key,
            },
        )?),
        JobSubcommand::WeeklyReportDeliverySend {
            delivery_id,
            telegram_bot_token,
            email_account_id,
            email_api_token,
            email_from,
            api_base,
        } => print_json(&store.send_job_weekly_report_delivery(
            JobWeeklyReportDeliverySendInput {
                delivery_id,
                telegram_bot_token,
                email_account_id,
                email_api_token,
                email_from,
                api_base,
            },
        )?),
        JobSubcommand::WeeklyReportDeliveries { report_id } => {
            print_json(&store.list_job_weekly_report_deliveries(report_id.as_deref())?)
        }
    }
}

fn job_radar_email_delivery(
    email_to: Option<&str>,
    idempotency_key: Option<String>,
) -> Result<Option<Value>> {
    let Some(email_to) = email_to else {
        return Ok(None);
    };
    let email = normalize_cli_email(email_to)?;
    let subject = format!("email:{email}");
    Ok(Some(json!({
        "channel": "email",
        "subject": subject,
        "target": subject,
        "idempotency_key": idempotency_key,
    })))
}

pub(crate) fn radar(store: Store, args: RadarCommand) -> Result<()> {
    match args.command {
        RadarSubcommand::Profile { command } => match command {
            RadarProfileSubcommand::Create {
                name,
                description,
                window_hours,
                min_score,
                max_items,
                language,
                source_card_query,
                selector_json,
                delivery_policy_json,
                model_policy_json,
                metadata_json,
            } => {
                let mut selectors: Vec<Value> = source_card_query
                    .into_iter()
                    .map(|query| json!({ "kind": "source_card_query", "query": query }))
                    .collect();
                for raw in selector_json {
                    let selector: Value = serde_json::from_str(&raw)
                        .with_context(|| format!("invalid selector JSON: {raw}"))?;
                    selectors.push(selector);
                }
                if selectors.is_empty() {
                    bail!("radar profile requires at least one selector");
                }
                let delivery_policy = delivery_policy_json
                    .as_deref()
                    .map(serde_json::from_str)
                    .transpose()
                    .context("invalid delivery policy JSON")?
                    .unwrap_or_else(|| json!({ "delivery": "manual_only" }));
                let model_policy = model_policy_json
                    .as_deref()
                    .map(serde_json::from_str)
                    .transpose()
                    .context("invalid model policy JSON")?
                    .unwrap_or_else(|| json!({ "model_scoring": "disabled" }));
                let mut metadata = metadata_json
                    .as_deref()
                    .map(serde_json::from_str::<Value>)
                    .transpose()
                    .context("invalid metadata JSON")?
                    .unwrap_or_else(|| json!({ "created_from": "cli" }));
                let Some(metadata_object) = metadata.as_object_mut() else {
                    bail!("radar profile metadata JSON must be an object");
                };
                metadata_object
                    .entry("created_from".to_string())
                    .or_insert_with(|| json!("cli"));
                print_json(&store.create_radar_profile(RadarProfileInput {
                    name,
                    description,
                    window_hours,
                    min_score,
                    max_items,
                    languages: language,
                    source_selectors: Value::Array(selectors),
                    delivery_policy,
                    model_policy,
                    metadata,
                })?)
            }
            RadarProfileSubcommand::List => print_json(&store.list_radar_profiles()?),
            RadarProfileSubcommand::Read { profile } => {
                print_json(&store.read_radar_profile(&profile)?)
            }
        },
        RadarSubcommand::Run {
            profile,
            window_hours,
            fetch_live,
        } => {
            print_json(&store.run_radar_profile_with_options(&profile, window_hours, fetch_live)?)
        }
        RadarSubcommand::Enqueue {
            profile,
            window_hours,
            fetch_live,
        } => print_json(&store.enqueue_radar_run_job(&profile, window_hours, fetch_live)?),
        RadarSubcommand::Runs => print_json(&store.list_radar_runs()?),
        RadarSubcommand::Stage { run_id } => print_json(&store.read_radar_stage(&run_id)?),
        RadarSubcommand::Summarize {
            run_id,
            language,
            format,
        } => print_json(&store.summarize_radar_run(&run_id, &language, &format)?),
        RadarSubcommand::Summary {
            run_id,
            language,
            format,
        } => print_json(&store.read_radar_summary(&run_id, &language, &format)?),
        RadarSubcommand::Deliver {
            run_id,
            channel,
            recipient,
            language,
            format,
            idempotency_key,
            bot_token,
            account_id,
            api_token,
            from,
            api_base,
        } => {
            let channel_normalized = channel.trim().to_ascii_lowercase();
            let telegram_bot_token = if channel_normalized == "telegram" {
                Some(telegram_bot_token(&store, bot_token.as_deref())?)
            } else {
                None
            };
            let email_account_id = if channel_normalized == "email" {
                Some(cloudflare_account_id(&store, account_id.as_deref())?)
            } else {
                None
            };
            let email_api_token = if channel_normalized == "email" {
                Some(cloudflare_api_token(&store, api_token.as_deref())?)
            } else {
                None
            };
            let email_from = if channel_normalized == "email" {
                Some(
                    from.as_deref()
                        .map(ToOwned::to_owned)
                        .or_else(|| agent_email_from(&store).ok())
                        .unwrap_or_else(|| "agent@example.com".to_string()),
                )
            } else {
                None
            };
            print_json(&store.deliver_radar_summary(RadarDeliveryInput {
                run_id,
                language,
                format,
                channel,
                recipient_ref: recipient,
                idempotency_key,
                telegram_bot_token,
                email_account_id,
                email_api_token,
                email_from,
                api_base,
            })?)
        }
        RadarSubcommand::Deliveries { run_id } => {
            print_json(&store.list_radar_deliveries(run_id.as_deref())?)
        }
        RadarSubcommand::Audit { run_id } => print_json(&store.audit_radar_run(&run_id)?),
        RadarSubcommand::SourceQuality { run_id } => {
            print_json(&store.list_radar_source_quality(&run_id)?)
        }
        RadarSubcommand::SourceQualityTrends { min_windows, limit } => {
            print_json(&store.list_radar_source_quality_trends(min_windows, limit)?)
        }
        RadarSubcommand::ModelScore {
            run_id,
            provider,
            model,
            max_items,
            endpoint,
            api_key,
        } => print_json(&store.score_radar_run_with_model(
            &run_id,
            &provider,
            model.as_deref(),
            max_items,
            endpoint.as_deref(),
            api_key.as_deref(),
        )?),
        RadarSubcommand::RepairFts { run_id } => {
            print_json(&json!({ "rebuilt": store.rebuild_radar_fts(run_id.as_deref())? }))
        }
    }
}
