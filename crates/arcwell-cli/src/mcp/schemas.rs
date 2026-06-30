use super::*;

pub(crate) fn commerce_run_config_tool_properties() -> Value {
    json!({
        "run_id": string_schema("Existing research run id."),
        "domain_profile": string_schema("Domain profile, such as uk-fashion-retail."),
        "target_qualified_count": integer_schema("Desired number of qualified final options. Defaults to 20."),
        "geography": string_schema("Optional geography/market, such as UK."),
        "freshness_window": string_schema("Freshness window for evidence, such as 24h."),
        "allowed_private_context_sources": array_schema("Private context source families the user authorized for this run.", string_schema("Source family, such as memory, wardrobe, email, spreadsheet, browser_history, or screenshot.")),
        "allowed_public_source_families": array_schema("Public source families allowed for discovery and corroboration.", string_schema("Source family, such as retailer, marketplace, review, brand, aggregator, rental_listing, or airline.")),
        "allow_marketplaces": boolean_schema("Whether marketplaces such as eBay or Vinted are allowed."),
        "allow_chrome_profile": boolean_schema("Whether the user's Chrome/cookie profile may be used when host browser access is needed."),
        "max_provider_calls": integer_schema("Optional provider-call cap."),
        "max_browser_pages": integer_schema("Optional rendered-browser page cap."),
        "max_cost_usd": number_schema("Optional cost cap in USD."),
        "stop_rules": object_schema("Structured stop rules. Do not put secrets here.", json!({}), &[]),
        "stop_rules_json": string_schema("Optional JSON string equivalent for CLI compatibility.")
    })
}

pub(crate) fn commerce_candidate_tool_properties() -> Value {
    json!({
        "run_id": string_schema("Existing research run id."),
        "domain": string_schema("Domain, such as fashion, rental, or travel."),
        "source_url": string_schema("Canonical candidate URL."),
        "retailer_or_provider": string_schema("Retailer, marketplace seller, landlord/platform, airline, or provider name."),
        "title": string_schema("Visible item/listing/offer title."),
        "normalized_item_key": string_schema("Stable normalized item key without size/variant, used for duplicate control."),
        "variant_key": string_schema("Exact desired variant key, such as shoe_size:UK 8.5 or shirt_size:XXL."),
        "price": string_schema("Optional visible price text."),
        "currency": string_schema("Optional currency code."),
        "geography": string_schema("Optional market/geography."),
        "candidate_status": enum_schema("Candidate status. Defaults to maybe.", &["maybe", "qualified", "disqualified", "blocked"]),
        "score": number_schema("Optional fit score from 0.0 to 1.0."),
        "score_reasons": object_schema("Structured score reasons.", json!({}), &[]),
        "score_reasons_json": string_schema("Optional JSON string equivalent for CLI compatibility."),
        "disqualification_reasons": array_schema("Structured disqualification reasons.", string_schema("Reason.")),
        "disqualification_reasons_json": string_schema("Optional JSON string equivalent for CLI compatibility."),
        "metadata": object_schema("Optional structured metadata. Do not include secrets.", json!({}), &[]),
        "metadata_json": string_schema("Optional JSON string equivalent for CLI compatibility.")
    })
}

pub(crate) fn commerce_availability_proof_tool_properties() -> Value {
    json!({
        "run_id": string_schema("Existing research run id."),
        "candidate_id": string_schema("Commerce candidate id from commerce_candidate_add."),
        "proof_method": enum_schema("Proof method.", &["static_fetch", "rendered_browser", "chrome_profile", "manual_user"]),
        "variant_key": string_schema("Exact variant key checked. Must match the candidate variant_key."),
        "variant_label": string_schema("Visible label for the checked variant, such as UK 8.5."),
        "availability_state": enum_schema("Observed availability state.", &["available", "unavailable", "unknown", "blocked"]),
        "visible_evidence": string_schema("Required for availability_state=available: short visible page evidence, not model inference."),
        "selector_or_dom_hint": string_schema("Optional selector, DOM hint, or visible control description."),
        "screenshot_artifact_id": string_schema("Optional research artifact id for a screenshot record from the same run."),
        "page_snapshot_artifact_id": string_schema("Optional research artifact id for rendered page text/HTML from the same run."),
        "confidence": number_schema("Confidence from 0.0 to 1.0. Defaults to 0.7."),
        "caveats": array_schema("Caveats about the evidence.", string_schema("Caveat.")),
        "caveats_json": string_schema("Optional JSON string equivalent for CLI compatibility."),
        "checked_at": string_schema("Optional RFC3339 timestamp for when the page was checked.")
    })
}

pub(crate) fn commerce_rendered_page_check_tool_properties() -> Value {
    json!({
        "run_id": string_schema("Existing research run id."),
        "candidate_id": string_schema("Commerce candidate id from commerce_candidate_add."),
        "variant_key": string_schema("Exact variant key checked. Must match the candidate variant_key."),
        "variant_label": string_schema("Visible label to find in rendered text, such as UK 8.5 or XXL."),
        "requested_url": string_schema("Public http(s) URL originally requested by the host/browser."),
        "final_url": string_schema("Optional public http(s) URL after redirects."),
        "title": string_schema("Optional visible page title."),
        "rendered_html": string_schema("Optional rendered HTML captured by host/browser. Arcwell treats it as untrusted evidence."),
        "rendered_text": string_schema("Optional visible rendered text captured by host/browser. Arcwell treats it as untrusted evidence."),
        "captured_at": string_schema("Optional RFC3339 timestamp for capture time."),
        "browser": string_schema("Optional browser/tool name that captured the page."),
        "screenshot_path": string_schema("Optional local screenshot path recorded as provenance only."),
        "selector_or_dom_hint": string_schema("Optional selector, DOM hint, or visible control description."),
        "chrome_profile_required": boolean_schema("Whether this check depended on the user's Chrome/cookie profile.")
    })
}

pub(crate) fn commerce_context_fact_tool_properties() -> Value {
    json!({
        "run_id": string_schema("Existing research run id."),
        "fact_key": string_schema("Stable fact key, such as shoe_size_uk or shirt_size."),
        "fact_kind": enum_schema("Evidence status for the fact.", &["explicit", "inferred", "uncertain", "missing"]),
        "redacted_value": string_schema("Redacted value safe for reports/logs."),
        "source_family": string_schema("Source family, such as memory, wardrobe, email, spreadsheet, browser_history, screenshot, or user_prompt."),
        "source_ref": string_schema("Optional source reference id/path/locator."),
        "confidence": number_schema("Confidence from 0.0 to 1.0. Defaults to 0.7."),
        "user_confirmed": boolean_schema("Whether the user explicitly confirmed this fact."),
        "may_persist_to_memory": boolean_schema("Whether this fact may be proposed for memory after the run."),
        "metadata": object_schema("Optional structured metadata. Do not include secrets.", json!({}), &[]),
        "metadata_json": string_schema("Optional JSON string equivalent for CLI compatibility.")
    })
}

pub(crate) fn commerce_verification_attempt_tool_properties() -> Value {
    json!({
        "run_id": string_schema("Existing research run id."),
        "candidate_id": string_schema("Commerce candidate id."),
        "method": enum_schema("Attempt method.", &["static_fetch", "rendered_browser", "chrome_profile", "manual_user"]),
        "result": enum_schema("Attempt result.", &["available", "unavailable", "unknown", "blocked", "error"]),
        "error_kind": string_schema("Optional redacted failure/blocker kind."),
        "final_url": string_schema("Optional final URL reached."),
        "http_status": integer_schema("Optional HTTP status code."),
        "browser_required": boolean_schema("Whether rendered browser access is required to continue."),
        "chrome_profile_required": boolean_schema("Whether logged-in/cookie-backed Chrome is required to continue."),
        "artifact_ids": array_schema("Research artifact ids created during the attempt. Each must belong to the same run.", string_schema("Research artifact id.")),
        "next_action": string_schema("Required when result is blocked or error."),
        "attempted_at": string_schema("Optional RFC3339 timestamp for the attempt.")
    })
}

pub(crate) fn commerce_report_judgment_tool_properties() -> Value {
    json!({
        "run_id": string_schema("Existing research run id."),
        "decision": enum_schema("Report decision.", &["accept", "hold", "block"]),
        "blocking_findings": array_schema("Blocking findings. decision=accept is rejected when this is non-empty.", string_schema("Finding.")),
        "blocking_findings_json": string_schema("Optional JSON string equivalent for CLI compatibility."),
        "non_blocking_findings": array_schema("Non-blocking findings.", string_schema("Finding.")),
        "non_blocking_findings_json": string_schema("Optional JSON string equivalent for CLI compatibility."),
        "claims_checked": array_schema("Claims checked by the report/audit gate.", string_schema("Claim id or description.")),
        "claims_checked_json": string_schema("Optional JSON string equivalent for CLI compatibility."),
        "availability_proofs_checked": array_schema("Availability proof ids checked by the report/audit gate.", string_schema("Availability proof id.")),
        "availability_proofs_checked_json": string_schema("Optional JSON string equivalent for CLI compatibility."),
        "privacy_review": object_schema("Structured privacy review.", json!({}), &[]),
        "privacy_review_json": string_schema("Optional JSON string equivalent for CLI compatibility."),
        "remaining_risks": array_schema("Remaining risks that should be visible to the user.", string_schema("Risk.")),
        "remaining_risks_json": string_schema("Optional JSON string equivalent for CLI compatibility.")
    })
}

pub(crate) fn job_profile_tool_properties() -> Value {
    json!({
        "label": string_schema("Profile label, such as the candidate name."),
        "current_resume_source": string_schema("Optional current resume source label, URL, or local reference."),
        "linkedin_source": string_schema("Optional LinkedIn source URL or label."),
        "github_profile": string_schema("Optional GitHub profile URL or label."),
        "blog_url": string_schema("Optional public blog URL."),
        "metadata": object_schema("Optional structured metadata.", json!({}), &[]),
        "metadata_json": string_schema("Optional JSON string equivalent for CLI compatibility.")
    })
}

pub(crate) fn job_import_batch_tool_properties() -> Value {
    json!({
        "batch": object_schema(
            "Reviewed job-hunting import packet. Supported array fields include evidence_cards, evidence_claims, privacy_rules, sources, source_health, roles, role_source_links, fit_scores, skeptic_findings, packets, companies, contacts, intro_paths, search_runs, role_status_events, and applications. Values use the same shapes as the corresponding add tools.",
            json!({}),
            &[]
        ),
        "batch_json": string_schema("Optional JSON string equivalent for CLI compatibility. Use either batch or batch_json.")
    })
}

pub(crate) fn job_evidence_tool_properties() -> Value {
    json!({
        "profile_id": string_schema("Job candidate profile id."),
        "title": string_schema("Evidence card title."),
        "evidence_type": string_schema("Evidence type, such as resume, github, blog, project, work, standard, talk, or private_safe."),
        "visibility": enum_schema("Public-shelf visibility.", &["public", "private_safe", "private_blocked", "needs_review"]),
        "summary": string_schema("Short evidence summary."),
        "proof_url": string_schema("Optional public HTTP proof URL."),
        "local_path": string_schema("Optional local-only path. This is never treated as public proof."),
        "source_date": string_schema("Optional source date."),
        "confidence": enum_schema("Evidence confidence.", &["verified", "user_claimed", "inferred", "stale"]),
        "tags": array_schema("Evidence tags.", string_schema("Tag.")),
        "safe_application_text": string_schema("Approved application-safe phrasing for this evidence."),
        "unsafe_terms": array_schema("Terms that must not appear in generated public material.", string_schema("Blocked term.")),
        "metadata": object_schema("Optional structured metadata.", json!({}), &[]),
        "metadata_json": string_schema("Optional JSON string equivalent for CLI compatibility.")
    })
}

pub(crate) fn job_privacy_check_tool_properties() -> Value {
    json!({
        "artifact_type": string_schema("Artifact type, such as packet, outreach, resume, report, or evidence_card."),
        "artifact_id": string_schema("Optional durable artifact id."),
        "text": string_schema("Text to check. It is hashed, not stored verbatim."),
        "blocked_terms": array_schema("Additional blocked terms for this check.", string_schema("Blocked term."))
    })
}

pub(crate) fn job_role_tool_properties() -> Value {
    json!({
        "company": string_schema("Company name."),
        "role_title": string_schema("Role title."),
        "canonical_url": string_schema("Official company/ATS URL. Required for canonical_confirmed."),
        "source_family": string_schema("Source family, such as company, ats, job_board, vc_board, founder_post, or manual."),
        "source_url": string_schema("URL where this role was observed."),
        "source_confidence": enum_schema("Source confidence. Tier 1 requires canonical_confirmed.", &["canonical_confirmed", "secondary_confirmed", "aggregator_only", "stale", "unknown"]),
        "date_accessed": string_schema("Optional observed timestamp."),
        "posting_freshness": string_schema("Freshness label, such as same_day, week, old, or unknown."),
        "location": string_schema("Optional location text."),
        "work_mode": string_schema("Optional work-mode text."),
        "company_stage_or_size": string_schema("Optional stage/size text."),
        "role_seniority": string_schema("Optional seniority text."),
        "core_requirements": array_schema("Core role requirements.", string_schema("Requirement.")),
        "implied_business_problem": string_schema("Optional inferred business/engineering problem."),
        "why_they_might_need_user": string_schema("Optional concise fit thesis."),
        "evidence_card_ids": array_schema("Evidence cards linked to the role.", string_schema("Evidence card id.")),
        "gaps_or_blockers": array_schema("Known gaps or blockers.", string_schema("Gap or blocker.")),
        "cluster": string_schema("Optional role cluster label."),
        "current_status": enum_schema("Current role status.", &["live", "stale", "closed", "unknown"]),
        "metadata": object_schema("Optional structured metadata.", json!({}), &[]),
        "metadata_json": string_schema("Optional JSON string equivalent for CLI compatibility.")
    })
}

pub(crate) fn job_score_tool_properties() -> Value {
    json!({
        "role_id": string_schema("Job role card id."),
        "profile_id": string_schema("Job candidate profile id."),
        "scorer": string_schema("Scorer label, such as human, model, or hybrid."),
        "role_fit": number_schema("0-5 role-fit score."),
        "domain_fit": number_schema("0-5 domain-fit score."),
        "evidence_fit": number_schema("0-5 evidence-fit score. Values above 2 require evidence_card_ids."),
        "geo_work_fit": number_schema("0-5 geography/work-mode score."),
        "stage_fit": number_schema("0-5 company stage score."),
        "practical_odds": number_schema("0-5 practical odds score."),
        "interest_energy": number_schema("0-5 interest/energy score."),
        "blockers": array_schema("Hard blockers. Any blocker demotes to blocked.", string_schema("Blocker.")),
        "evidence_card_ids": array_schema("Evidence card ids supporting the evidence_fit score.", string_schema("Evidence card id.")),
        "explanation": string_schema("Short explanation of the score.")
    })
}

pub(crate) fn job_packet_tool_properties() -> Value {
    json!({
        "role_id": string_schema("Job role card id."),
        "profile_id": string_schema("Job candidate profile id."),
        "evidence_card_ids": array_schema("Evidence cards approved for this packet.", string_schema("Evidence card id.")),
        "resume_emphasis": string_schema("Role-specific resume emphasis."),
        "tailored_bullets": array_schema("Tailored resume/application bullets.", string_schema("Bullet.")),
        "outreach_note": string_schema("Short company-specific outreach note."),
        "proof_links": object_schema("Structured proof links. Local paths are rejected.", json!({}), &[]),
        "proof_links_json": string_schema("Optional JSON string equivalent for CLI compatibility."),
        "likely_objections": array_schema("Likely objections or weaknesses to handle.", string_schema("Objection.")),
        "interview_stories": array_schema("Interview stories to prepare.", string_schema("Story.")),
        "questions_to_ask": array_schema("Questions to ask the company.", string_schema("Question.")),
        "reviewer_note": string_schema("Optional reviewer note.")
    })
}

pub(crate) fn job_packet_approve_tool_properties() -> Value {
    json!({
        "packet_id": string_schema("Application packet id."),
        "reviewer_note": string_schema("Human review note explaining approval. Required for approval.")
    })
}

pub(crate) fn job_packet_export_tool_properties() -> Value {
    json!({
        "packet_id": string_schema("Approved application packet id."),
        "out_dir": string_schema("Local output directory for the generated Markdown packet. The filename is generated by Arcwell.")
    })
}

pub(crate) fn job_packet_export_set_tool_properties() -> Value {
    json!({
        "profile_id": string_schema("Job candidate profile id. Every packet must belong to this profile."),
        "packet_ids": array_schema("Approved application packet ids to export as one local review set.", string_schema("Application packet id.")),
        "out_dir": string_schema("Local output directory for the generated Markdown packets and manifest.")
    })
}

pub(crate) fn job_application_tool_properties() -> Value {
    json!({
        "role_id": string_schema("Job role card id."),
        "packet_id": string_schema("Optional application packet id."),
        "status": enum_schema("User-confirmed application status.", &["planned", "applied", "intro_requested", "replied", "interview", "rejected", "offer", "withdrawn"]),
        "applied_at": string_schema("Optional applied date or timestamp."),
        "follow_up_at": string_schema("Optional follow-up date or timestamp."),
        "outcome_note": string_schema("Optional outcome note.")
    })
}

pub(crate) fn job_source_refresh_tool_properties() -> Value {
    json!({
        "source_id": string_schema("Configured job source id."),
        "body": string_schema("Optional caller-supplied page text or HTML. This records a manual/host-captured snapshot and does not claim a live fetch."),
        "fetched_url": string_schema("Optional URL for the supplied body. Defaults to the stored source URL."),
        "fetch_live": boolean_schema("Explicitly opt in to a live network fetch of the stored source URL behind provider-network policy. Do not combine with body.")
    })
}

pub(crate) fn job_radar_schedule_tool_properties() -> Value {
    let mut properties = job_radar_enqueue_tool_properties();
    if let Some(map) = properties.as_object_mut() {
        map.insert(
            "cadence".to_string(),
            enum_schema("Watch-source cadence.", &["hot", "warm", "cold"]),
        );
        map.insert(
            "status".to_string(),
            enum_schema("Watch-source status.", &["active", "paused", "error"]),
        );
    }
    properties
}

pub(crate) fn job_radar_enqueue_tool_properties() -> Value {
    json!({
        "profile_id": string_schema("Job candidate profile id."),
        "scope": string_schema("Refresh scope label."),
        "source_ids": array_schema("Configured job source ids to refresh.", string_schema("Job source id.")),
        "fetch_live": boolean_schema("Explicitly opt in to live network fetches of the stored source URLs behind provider-network policy and cost gates."),
        "source_snapshots": {
            "type": "object",
            "description": "Optional replay snapshots keyed by source id. Each value may be a string body or an object with body and fetched_url. Replay snapshots prove local scheduled behavior, not current live freshness."
        },
        "delivery": {
            "type": "object",
            "description": "Optional prepared-report delivery metadata for the worker, such as {\"channel\":\"email\",\"subject\":\"email:user@example.com\",\"target\":\"email:user@example.com\",\"idempotency_key\":\"stable-key\"}. Provider send remains authorization and policy gated."
        }
    })
}

pub(crate) fn job_company_targets_tool_properties() -> Value {
    json!({
        "profile_id": string_schema("Job candidate profile id."),
        "market": string_schema("Optional normalized company market filter, such as london."),
        "limit": integer_schema("Maximum target entries to return, clamped to 1..100.")
    })
}

pub(crate) fn job_outreach_readiness_tool_properties() -> Value {
    json!({
        "profile_id": string_schema("Job candidate profile id."),
        "limit": integer_schema("Maximum scored role entries to classify, clamped to 1..100. This does not send outreach.")
    })
}

pub(crate) fn job_manual_refresh_tool_properties() -> Value {
    json!({
        "profile_id": string_schema("Job candidate profile id."),
        "scope": string_schema("Refresh scope label."),
        "observed_role_ids": array_schema("Role ids observed in the current manual refresh.", string_schema("Role id.")),
        "stale_role_ids": array_schema("Role ids the refresh marked stale.", string_schema("Role id.")),
        "closed_role_ids": array_schema("Role ids the refresh marked closed.", string_schema("Role id.")),
        "source_health_ids": array_schema("Source-health rows from this refresh. Non-healthy states remain visible in reports.", string_schema("Source health id.")),
        "proof_level": enum_schema("Proof level. Defaults to local_proof unless the caller has a real production-data packet.", &["local_proof", "production_data_proof", "manual_production_data_pass"]),
        "report_artifact_id": string_schema("Optional report/research artifact id tied to this refresh.")
    })
}

pub(crate) fn job_refresh_audit_tool_properties() -> Value {
    json!({
        "profile_id": string_schema("Job candidate profile id."),
        "scope": string_schema("Refresh scope label."),
        "min_elapsed_hours": number_schema("Minimum elapsed hours required between the first and latest completed run. Use 24 for the operational one-day gate.")
    })
}

pub(crate) fn job_operational_audit_tool_properties() -> Value {
    json!({
        "profile_id": string_schema("Job candidate profile id."),
        "scope": string_schema("Operational audit scope label."),
        "min_elapsed_hours": number_schema("Minimum elapsed hours required for the one-day refresh gate. Values below 24 are raised to 24 for this operational audit.")
    })
}

pub(crate) fn job_weekly_report_delivery_prepare_tool_properties() -> Value {
    json!({
        "report_id": string_schema("Job weekly report id."),
        "channel": enum_schema("Prepared delivery channel. No provider send is attempted.", &["email", "telegram"]),
        "subject": string_schema("Authorized channel subject, such as email:user@example.com or telegram:chat:123."),
        "target": string_schema("Delivery target/destination for the prepared message."),
        "idempotency_key": string_schema("Optional stable key for deliberate replays. Defaults from report/channel/subject/target.")
    })
}

pub(crate) fn job_weekly_report_delivery_send_tool_properties() -> Value {
    json!({
        "delivery_id": string_schema("Prepared weekly job report delivery id."),
        "telegram_bot_token": string_schema("Telegram bot token for telegram deliveries."),
        "email_account_id": string_schema("Cloudflare account id for email deliveries."),
        "email_api_token": string_schema("Cloudflare Email API token for email deliveries."),
        "email_from": string_schema("Verified sender address for email deliveries."),
        "api_base": string_schema("Optional provider API base for controlled local proof or configured provider override.")
    })
}

pub(crate) fn job_weekly_report_deliveries_tool_properties() -> Value {
    json!({
        "report_id": string_schema("Optional job weekly report id filter.")
    })
}

pub(crate) fn research_convergence_tool_properties() -> Value {
    json!({
        "run_id": string_schema("Research run id."),
        "max_iterations": integer_schema("Maximum convergence iterations, 1..16."),
        "max_seconds": integer_schema("Wall-clock cap in seconds, 1..86400."),
        "max_sources": integer_schema("Maximum sources allowed before stopping incomplete."),
        "max_provider_calls": integer_schema("Maximum provider/model calls allowed. Model-backed editorial/eval requires at least 2."),
        "cost_cap_usd": {
            "type": "number",
            "description": "Estimated cost cap in USD."
        },
        "source_novelty_threshold": {
            "type": "number",
            "description": "Novel source threshold for stop-rule no-progress checks."
        },
        "confidence_delta_threshold": {
            "type": "number",
            "description": "Confidence delta threshold for stop-rule no-progress checks."
        },
        "no_progress_iteration_limit": integer_schema("Iterations with no progress before settled stop."),
        "require_active_fact_check": boolean_schema("Require active fact-check labels in convergence."),
        "allow_long_run": boolean_schema("Permit convergence runs longer than two hours."),
        "no_write": boolean_schema("Disable report/editorial writes where supported."),
        "editorial_provider": enum_schema("Optional model-backed convergence editorial/evaluator provider.", &["mock", "openai"]),
        "editorial_model_name": string_schema("Optional editorial/evaluator model name."),
        "editorial_endpoint": string_schema("Optional provider endpoint override for tests or compatible endpoints."),
        "editorial_timeout_seconds": integer_schema("Editorial provider timeout, clamped 1..120 seconds.")
    })
}

pub(crate) fn research_convergence_provider_search_tool_properties() -> Value {
    json!({
        "run_id": string_schema("Research run id."),
        "provider": enum_schema("Provider fallback to use for pending convergence host-search tasks.", &["brave", "openai", "perplexity"]),
        "max_tasks": integer_schema("Maximum pending host-search tasks to attempt, 1..50."),
        "max_results": integer_schema("Maximum provider results per task, 1..20."),
        "max_provider_calls": integer_schema("Maximum provider calls for this invocation, 1..50."),
        "enqueue_selected_url_ingest": boolean_schema("When true, enqueue worker ingest_url jobs for selected safe provider results."),
        "max_ingest_jobs": integer_schema("Maximum selected URL ingest jobs to enqueue, 0..100. Required above zero when enqueue_selected_url_ingest is true."),
        "cost_cap_usd": {
            "type": "number",
            "description": "Per-invocation projected cost cap in USD."
        },
        "endpoint": string_schema("Optional provider endpoint override for tests or compatible endpoints."),
        "api_key": string_schema("Optional provider API key; omitted values use provider environment variables."),
        "model": string_schema("Optional provider model name."),
        "timeout_seconds": integer_schema("Provider search timeout, clamped 1..120 seconds.")
    })
}

pub(crate) fn research_active_fact_check_tool_properties() -> Value {
    json!({
        "run_id": string_schema("Research run id."),
        "artifact_id": string_schema("Optional report/generated-synthesis artifact id. Defaults to the latest convergence/report synthesis artifact for the run."),
        "max_sentences": integer_schema("Maximum factual report sentences to check, 1..200."),
        "create_challenges": boolean_schema("Whether unsupported sentences should create citation-gap host-search challenges. Defaults true.")
    })
}

pub(crate) fn research_convergence_close_loop_tool_properties() -> Value {
    let mut properties = research_convergence_tool_properties()
        .as_object()
        .cloned()
        .unwrap_or_default();
    properties.insert(
        "artifact_id".to_string(),
        string_schema("Optional report/generated-synthesis artifact id to actively fact-check."),
    );
    properties.insert(
        "max_sentences".to_string(),
        integer_schema("Maximum factual report sentences to check, 1..200."),
    );
    properties.insert(
        "create_challenges".to_string(),
        boolean_schema("Whether unsupported report sentences should create citation-gap challenges. Defaults true."),
    );
    properties.insert(
        "compile_report_before_check".to_string(),
        boolean_schema("Compile a convergence report before active fact-checking when artifact_id is absent. Defaults true."),
    );
    properties.insert(
        "rerun_after_check".to_string(),
        boolean_schema("Rerun convergence after fact-check/provider proof so blockers can settle or remain explicit. Defaults true."),
    );
    properties.insert(
        "compile_final_report".to_string(),
        boolean_schema(
            "Compile a final convergence report/judgment after the loop attempt. Defaults true.",
        ),
    );
    properties.insert(
        "provider".to_string(),
        enum_schema(
            "Optional provider fallback for pending host-search tasks.",
            &["brave", "openai", "perplexity"],
        ),
    );
    properties.insert(
        "provider_max_tasks".to_string(),
        integer_schema(
            "Maximum pending host-search tasks to attempt through provider fallback, 1..50.",
        ),
    );
    properties.insert(
        "provider_max_results".to_string(),
        integer_schema("Maximum provider results per pending task, 1..20."),
    );
    properties.insert(
        "provider_max_provider_calls".to_string(),
        integer_schema("Maximum provider fallback calls for this close-loop invocation, 1..50."),
    );
    properties.insert(
        "enqueue_selected_url_ingest".to_string(),
        boolean_schema(
            "When true, enqueue worker ingest_url jobs for selected safe provider results.",
        ),
    );
    properties.insert(
        "max_ingest_jobs".to_string(),
        integer_schema("Maximum selected URL ingest jobs to enqueue, 0..100."),
    );
    properties.insert(
        "provider_cost_cap_usd".to_string(),
        json!({
            "type": "number",
            "description": "Provider-search projected cost cap in USD for this close-loop invocation."
        }),
    );
    properties.insert(
        "provider_endpoint".to_string(),
        string_schema("Optional provider endpoint override for tests or compatible endpoints."),
    );
    properties.insert(
        "provider_api_key".to_string(),
        string_schema(
            "Optional provider API key; omitted values use provider environment variables.",
        ),
    );
    properties.insert(
        "provider_model".to_string(),
        string_schema("Optional provider model name."),
    );
    properties.insert(
        "provider_timeout_seconds".to_string(),
        integer_schema("Provider fallback timeout, clamped 1..120 seconds."),
    );
    Value::Object(properties)
}

pub(crate) fn tool<const N: usize>(
    name: &str,
    description: &str,
    props: [(&str, &str, &str); N],
) -> Value {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();
    for (key, kind, desc) in props {
        properties.insert(
            key.to_string(),
            json!({
                "type": kind,
                "description": desc
            }),
        );
        required.push(key);
    }
    json!({
        "name": name,
        "description": description,
        "inputSchema": {
            "type": "object",
            "properties": properties,
            "required": required
        }
    })
}

pub(crate) fn tool_with_schema(
    name: &str,
    description: &str,
    properties: Value,
    required: &[&str],
) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": {
            "type": "object",
            "properties": properties,
            "required": required
        }
    })
}

pub(crate) fn string_schema(description: &str) -> Value {
    json!({
        "type": "string",
        "description": description
    })
}

pub(crate) fn integer_schema(description: &str) -> Value {
    json!({
        "type": "integer",
        "description": description
    })
}

pub(crate) fn number_schema(description: &str) -> Value {
    json!({
        "type": "number",
        "description": description
    })
}

pub(crate) fn boolean_schema(description: &str) -> Value {
    json!({
        "type": "boolean",
        "description": description
    })
}

pub(crate) fn array_schema(description: &str, items: Value) -> Value {
    json!({
        "type": "array",
        "description": description,
        "items": items
    })
}

pub(crate) fn object_schema(description: &str, properties: Value, required: &[&str]) -> Value {
    json!({
        "type": "object",
        "description": description,
        "properties": properties,
        "required": required
    })
}

pub(crate) fn enum_schema(description: &str, values: &[&str]) -> Value {
    json!({
        "type": "string",
        "description": description,
        "enum": values
    })
}
