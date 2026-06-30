use super::*;

pub(crate) fn commerce_run_config_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<CommerceRunConfig> {
    let target_qualified_count: i64 = row.get(2)?;
    let allowed_private_context_sources_json: String = row.get(5)?;
    let allowed_public_source_families_json: String = row.get(6)?;
    let allow_marketplaces: i64 = row.get(7)?;
    let allow_chrome_profile: i64 = row.get(8)?;
    let max_provider_calls: Option<i64> = row.get(9)?;
    let max_browser_pages: Option<i64> = row.get(10)?;
    let stop_rules_json: String = row.get(12)?;
    Ok(CommerceRunConfig {
        run_id: row.get(0)?,
        domain_profile: row.get(1)?,
        target_qualified_count: target_qualified_count.max(0) as usize,
        geography: row.get(3)?,
        freshness_window: row.get(4)?,
        allowed_private_context_sources: parse_json_string_vec_column(
            &allowed_private_context_sources_json,
            5,
        )?,
        allowed_public_source_families: parse_json_string_vec_column(
            &allowed_public_source_families_json,
            6,
        )?,
        allow_marketplaces: allow_marketplaces != 0,
        allow_chrome_profile: allow_chrome_profile != 0,
        max_provider_calls: max_provider_calls.map(|value| value.max(0) as usize),
        max_browser_pages: max_browser_pages.map(|value| value.max(0) as usize),
        max_cost_usd: row.get(11)?,
        stop_rules: parse_json_column(&stop_rules_json, 12)?,
        created_at: row.get(13)?,
        updated_at: row.get(14)?,
    })
}

pub(crate) fn commerce_candidate_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<CommerceCandidate> {
    let score_reasons_json: String = row.get(13)?;
    let disqualification_reasons_json: String = row.get(14)?;
    let metadata_json: String = row.get(15)?;
    Ok(CommerceCandidate {
        id: row.get(0)?,
        run_id: row.get(1)?,
        domain: row.get(2)?,
        source_url: row.get(3)?,
        retailer_or_provider: row.get(4)?,
        title: row.get(5)?,
        normalized_item_key: row.get(6)?,
        variant_key: row.get(7)?,
        price: row.get(8)?,
        currency: row.get(9)?,
        geography: row.get(10)?,
        candidate_status: row.get(11)?,
        score: row.get(12)?,
        score_reasons: parse_json_column(&score_reasons_json, 13)?,
        disqualification_reasons: parse_json_column(&disqualification_reasons_json, 14)?,
        metadata: parse_json_column(&metadata_json, 15)?,
        created_at: row.get(16)?,
        updated_at: row.get(17)?,
    })
}

pub(crate) fn commerce_availability_proof_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<CommerceAvailabilityProof> {
    let caveats_json: String = row.get(12)?;
    Ok(CommerceAvailabilityProof {
        id: row.get(0)?,
        run_id: row.get(1)?,
        candidate_id: row.get(2)?,
        proof_method: row.get(3)?,
        variant_key: row.get(4)?,
        variant_label: row.get(5)?,
        availability_state: row.get(6)?,
        visible_evidence: row.get(7)?,
        selector_or_dom_hint: row.get(8)?,
        screenshot_artifact_id: row.get(9)?,
        page_snapshot_artifact_id: row.get(10)?,
        confidence: row.get(11)?,
        caveats: parse_json_column(&caveats_json, 12)?,
        checked_at: row.get(13)?,
        created_at: row.get(14)?,
    })
}

pub(crate) fn commerce_context_fact_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<CommerceContextFact> {
    let user_confirmed: i64 = row.get(8)?;
    let may_persist_to_memory: i64 = row.get(9)?;
    let metadata_json: String = row.get(10)?;
    Ok(CommerceContextFact {
        id: row.get(0)?,
        run_id: row.get(1)?,
        fact_key: row.get(2)?,
        fact_kind: row.get(3)?,
        redacted_value: row.get(4)?,
        source_family: row.get(5)?,
        source_ref: row.get(6)?,
        confidence: row.get(7)?,
        user_confirmed: user_confirmed != 0,
        may_persist_to_memory: may_persist_to_memory != 0,
        metadata: parse_json_column(&metadata_json, 10)?,
        created_at: row.get(11)?,
    })
}

pub(crate) fn commerce_verification_attempt_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<CommerceVerificationAttempt> {
    let browser_required: i64 = row.get(9)?;
    let chrome_profile_required: i64 = row.get(10)?;
    let artifact_ids_json: String = row.get(11)?;
    Ok(CommerceVerificationAttempt {
        id: row.get(0)?,
        run_id: row.get(1)?,
        candidate_id: row.get(2)?,
        attempted_at: row.get(3)?,
        method: row.get(4)?,
        result: row.get(5)?,
        error_kind: row.get(6)?,
        final_url: row.get(7)?,
        http_status: row.get(8)?,
        browser_required: browser_required != 0,
        chrome_profile_required: chrome_profile_required != 0,
        artifact_ids: parse_json_string_vec_column(&artifact_ids_json, 11)?,
        next_action: row.get(12)?,
        created_at: row.get(13)?,
    })
}

pub(crate) fn commerce_report_judgment_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<CommerceReportJudgment> {
    let blocking_findings_json: String = row.get(3)?;
    let non_blocking_findings_json: String = row.get(4)?;
    let claims_checked_json: String = row.get(5)?;
    let availability_proofs_checked_json: String = row.get(6)?;
    let privacy_review_json: String = row.get(7)?;
    let remaining_risks_json: String = row.get(8)?;
    Ok(CommerceReportJudgment {
        id: row.get(0)?,
        run_id: row.get(1)?,
        decision: row.get(2)?,
        blocking_findings: parse_json_column(&blocking_findings_json, 3)?,
        non_blocking_findings: parse_json_column(&non_blocking_findings_json, 4)?,
        claims_checked: parse_json_column(&claims_checked_json, 5)?,
        availability_proofs_checked: parse_json_column(&availability_proofs_checked_json, 6)?,
        privacy_review: parse_json_column(&privacy_review_json, 7)?,
        remaining_risks: parse_json_column(&remaining_risks_json, 8)?,
        created_at: row.get(9)?,
    })
}

pub(crate) fn job_candidate_profile_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<JobCandidateProfile> {
    let metadata_json: String = row.get(6)?;
    Ok(JobCandidateProfile {
        id: row.get(0)?,
        label: row.get(1)?,
        current_resume_source: row.get(2)?,
        linkedin_source: row.get(3)?,
        github_profile: row.get(4)?,
        blog_url: row.get(5)?,
        metadata: parse_json_column(&metadata_json, 6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

pub(crate) fn job_evidence_card_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<JobEvidenceCard> {
    let tags_json: String = row.get(10)?;
    let unsafe_terms_json: String = row.get(12)?;
    let metadata_json: String = row.get(13)?;
    Ok(JobEvidenceCard {
        id: row.get(0)?,
        profile_id: row.get(1)?,
        title: row.get(2)?,
        evidence_type: row.get(3)?,
        visibility: row.get(4)?,
        summary: row.get(5)?,
        proof_url: row.get(6)?,
        local_path: row.get(7)?,
        source_date: row.get(8)?,
        confidence: row.get(9)?,
        tags: parse_json_string_vec_column(&tags_json, 10)?,
        safe_application_text: row.get(11)?,
        unsafe_terms: parse_json_string_vec_column(&unsafe_terms_json, 12)?,
        metadata: parse_json_column(&metadata_json, 13)?,
        created_at: row.get(14)?,
        updated_at: row.get(15)?,
    })
}

pub(crate) fn job_evidence_claim_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<JobEvidenceClaim> {
    let can_use_in_resume: i64 = row.get(5)?;
    let can_use_in_outreach: i64 = row.get(6)?;
    let can_use_in_interview: i64 = row.get(7)?;
    Ok(JobEvidenceClaim {
        id: row.get(0)?,
        evidence_card_id: row.get(1)?,
        claim: row.get(2)?,
        claim_kind: row.get(3)?,
        proof_level: row.get(4)?,
        can_use_in_resume: can_use_in_resume != 0,
        can_use_in_outreach: can_use_in_outreach != 0,
        can_use_in_interview: can_use_in_interview != 0,
        created_at: row.get(8)?,
    })
}

pub(crate) fn job_privacy_rule_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<JobPrivacyRule> {
    Ok(JobPrivacyRule {
        id: row.get(0)?,
        pattern: row.get(1)?,
        rule_type: row.get(2)?,
        severity: row.get(3)?,
        replacement_guidance: row.get(4)?,
        created_at: row.get(5)?,
    })
}

pub(crate) fn job_privacy_check_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<JobPrivacyCheck> {
    let findings_json: String = row.get(5)?;
    let findings: Vec<JobPrivacyFinding> =
        serde_json::from_str(&findings_json).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                5,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?;
    Ok(JobPrivacyCheck {
        id: row.get(0)?,
        artifact_type: row.get(1)?,
        artifact_id: row.get(2)?,
        checked_at: row.get(3)?,
        decision: row.get(4)?,
        findings,
        checked_text_hash: row.get(6)?,
    })
}

pub(crate) fn job_source_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<JobSource> {
    let metadata_json: String = row.get(6)?;
    Ok(JobSource {
        id: row.get(0)?,
        source_family: row.get(1)?,
        name: row.get(2)?,
        url: row.get(3)?,
        market_scope: row.get(4)?,
        refresh_policy: row.get(5)?,
        metadata: parse_json_column(&metadata_json, 6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

pub(crate) fn job_source_health_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<JobSourceHealth> {
    let fetched_count: i64 = row.get(6)?;
    let accepted_count: i64 = row.get(7)?;
    let rejected_count: i64 = row.get(8)?;
    Ok(JobSourceHealth {
        id: row.get(0)?,
        source_id: row.get(1)?,
        checked_at: row.get(2)?,
        status: row.get(3)?,
        http_status: row.get(4)?,
        error_code: row.get(5)?,
        fetched_count: fetched_count.max(0) as usize,
        accepted_count: accepted_count.max(0) as usize,
        rejected_count: rejected_count.max(0) as usize,
        note: row.get(9)?,
    })
}

pub(crate) fn job_role_card_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<JobRoleCard> {
    let core_requirements_json: String = row.get(13)?;
    let evidence_card_ids_json: String = row.get(16)?;
    let gaps_or_blockers_json: String = row.get(17)?;
    let metadata_json: String = row.get(20)?;
    Ok(JobRoleCard {
        id: row.get(0)?,
        company: row.get(1)?,
        role_title: row.get(2)?,
        canonical_url: row.get(3)?,
        source_family: row.get(4)?,
        source_url: row.get(5)?,
        source_confidence: row.get(6)?,
        date_accessed: row.get(7)?,
        posting_freshness: row.get(8)?,
        location: row.get(9)?,
        work_mode: row.get(10)?,
        company_stage_or_size: row.get(11)?,
        role_seniority: row.get(12)?,
        core_requirements: parse_json_string_vec_column(&core_requirements_json, 13)?,
        implied_business_problem: row.get(14)?,
        why_they_might_need_user: row.get(15)?,
        evidence_card_ids: parse_json_string_vec_column(&evidence_card_ids_json, 16)?,
        gaps_or_blockers: parse_json_string_vec_column(&gaps_or_blockers_json, 17)?,
        cluster: row.get(18)?,
        current_status: row.get(19)?,
        metadata: parse_json_column(&metadata_json, 20)?,
        created_at: row.get(21)?,
        updated_at: row.get(22)?,
    })
}

pub(crate) fn job_role_source_link_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<JobRoleSourceLink> {
    Ok(JobRoleSourceLink {
        id: row.get(0)?,
        role_id: row.get(1)?,
        source_id: row.get(2)?,
        source_url: row.get(3)?,
        observed_at: row.get(4)?,
        confidence: row.get(5)?,
        evidence_excerpt: row.get(6)?,
    })
}

pub(crate) fn job_fit_score_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<JobFitScore> {
    let blockers_json: String = row.get(14)?;
    let evidence_card_ids_json: String = row.get(15)?;
    Ok(JobFitScore {
        id: row.get(0)?,
        role_id: row.get(1)?,
        profile_id: row.get(2)?,
        scored_at: row.get(3)?,
        scorer: row.get(4)?,
        role_fit: row.get(5)?,
        domain_fit: row.get(6)?,
        evidence_fit: row.get(7)?,
        geo_work_fit: row.get(8)?,
        stage_fit: row.get(9)?,
        practical_odds: row.get(10)?,
        interest_energy: row.get(11)?,
        weighted_score: row.get(12)?,
        tier: row.get(13)?,
        blockers: parse_json_string_vec_column(&blockers_json, 14)?,
        evidence_card_ids: parse_json_string_vec_column(&evidence_card_ids_json, 15)?,
        explanation: row.get(16)?,
    })
}

pub(crate) fn job_skeptic_finding_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<JobSkepticFinding> {
    Ok(JobSkepticFinding {
        id: row.get(0)?,
        role_id: row.get(1)?,
        severity: row.get(2)?,
        finding_type: row.get(3)?,
        finding: row.get(4)?,
        next_action: row.get(5)?,
        created_at: row.get(6)?,
    })
}

pub(crate) fn job_application_packet_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<JobApplicationPacket> {
    let evidence_card_ids_json: String = row.get(5)?;
    let tailored_bullets_json: String = row.get(7)?;
    let proof_links_json: String = row.get(9)?;
    let likely_objections_json: String = row.get(10)?;
    let interview_stories_json: String = row.get(11)?;
    let questions_to_ask_json: String = row.get(12)?;
    Ok(JobApplicationPacket {
        id: row.get(0)?,
        role_id: row.get(1)?,
        profile_id: row.get(2)?,
        generated_at: row.get(3)?,
        status: row.get(4)?,
        evidence_card_ids: parse_json_string_vec_column(&evidence_card_ids_json, 5)?,
        resume_emphasis: row.get(6)?,
        tailored_bullets: parse_json_string_vec_column(&tailored_bullets_json, 7)?,
        outreach_note: row.get(8)?,
        proof_links: parse_json_column(&proof_links_json, 9)?,
        likely_objections: parse_json_string_vec_column(&likely_objections_json, 10)?,
        interview_stories: parse_json_string_vec_column(&interview_stories_json, 11)?,
        questions_to_ask: parse_json_string_vec_column(&questions_to_ask_json, 12)?,
        privacy_check_id: row.get(13)?,
        reviewer_note: row.get(14)?,
    })
}

pub(crate) fn job_company_card_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<JobCompanyCard> {
    let metadata_json: String = row.get(15)?;
    Ok(JobCompanyCard {
        id: row.get(0)?,
        company_name: row.get(1)?,
        website_url: row.get(2)?,
        source_family: row.get(3)?,
        market: row.get(4)?,
        stage: row.get(5)?,
        funding_signal: row.get(6)?,
        product_category: row.get(7)?,
        technical_audience: row.get(8)?,
        developer_facing_score: row.get(9)?,
        london_relevance: row.get(10)?,
        remote_maturity: row.get(11)?,
        hiring_page_url: row.get(12)?,
        founder_or_team_signal: row.get(13)?,
        last_checked_at: row.get(14)?,
        metadata: parse_json_column(&metadata_json, 15)?,
        created_at: row.get(16)?,
        updated_at: row.get(17)?,
    })
}

pub(crate) fn job_contact_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<JobContact> {
    Ok(JobContact {
        id: row.get(0)?,
        name: row.get(1)?,
        company_id: row.get(2)?,
        role_title: row.get(3)?,
        public_profile_url: row.get(4)?,
        source_url: row.get(5)?,
        relationship_status: row.get(6)?,
        relevance: row.get(7)?,
        note: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

pub(crate) fn job_intro_path_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<JobIntroPath> {
    Ok(JobIntroPath {
        id: row.get(0)?,
        role_id: row.get(1)?,
        contact_id: row.get(2)?,
        path_type: row.get(3)?,
        confidence: row.get(4)?,
        next_action: row.get(5)?,
        status: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

pub(crate) fn job_search_run_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<JobSearchRun> {
    let source_count: i64 = row.get(6)?;
    let role_count: i64 = row.get(7)?;
    let new_role_count: i64 = row.get(8)?;
    let stale_role_count: i64 = row.get(9)?;
    let error_count: i64 = row.get(10)?;
    Ok(JobSearchRun {
        id: row.get(0)?,
        profile_id: row.get(1)?,
        scope: row.get(2)?,
        started_at: row.get(3)?,
        completed_at: row.get(4)?,
        proof_level: row.get(5)?,
        source_count: source_count.max(0) as usize,
        role_count: role_count.max(0) as usize,
        new_role_count: new_role_count.max(0) as usize,
        stale_role_count: stale_role_count.max(0) as usize,
        error_count: error_count.max(0) as usize,
        report_artifact_id: row.get(11)?,
    })
}

pub(crate) fn job_role_status_event_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<JobRoleStatusEvent> {
    Ok(JobRoleStatusEvent {
        id: row.get(0)?,
        role_id: row.get(1)?,
        run_id: row.get(2)?,
        status: row.get(3)?,
        previous_tier: row.get(4)?,
        current_tier: row.get(5)?,
        note: row.get(6)?,
        created_at: row.get(7)?,
    })
}

pub(crate) fn job_application_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<JobApplication> {
    Ok(JobApplication {
        id: row.get(0)?,
        role_id: row.get(1)?,
        packet_id: row.get(2)?,
        status: row.get(3)?,
        applied_at: row.get(4)?,
        follow_up_at: row.get(5)?,
        outcome_note: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

pub(crate) fn job_weekly_report_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<JobWeeklyReport> {
    let metadata_json: String = row.get(6)?;
    Ok(JobWeeklyReport {
        id: row.get(0)?,
        profile_id: row.get(1)?,
        scope: row.get(2)?,
        generated_at: row.get(3)?,
        proof_level: row.get(4)?,
        body: row.get(5)?,
        metadata: parse_json_column(&metadata_json, 6)?,
    })
}

pub(crate) fn job_weekly_report_delivery_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<JobWeeklyReportDelivery> {
    Ok(JobWeeklyReportDelivery {
        id: row.get(0)?,
        report_id: row.get(1)?,
        channel: row.get(2)?,
        subject: row.get(3)?,
        target: row.get(4)?,
        status: row.get(5)?,
        privacy_check_id: row.get(6)?,
        channel_message_id: row.get(7)?,
        idempotency_key: row.get(8)?,
        error: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}
