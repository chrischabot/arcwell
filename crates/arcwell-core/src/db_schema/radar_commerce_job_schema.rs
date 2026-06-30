use super::*;

pub(crate) fn migrate_radar_source_quality_windows_on(conn: &Connection) -> Result<()> {
    ensure_radar_schema_on(conn)?;
    let columns = table_columns_on(conn, "radar_source_quality")?;
    let has_run_id = columns.iter().any(|column| column == "run_id");
    conn.execute_batch(
        r#"
        DROP TABLE IF EXISTS radar_source_quality_v11;
        CREATE TABLE radar_source_quality_v11 (
          id TEXT PRIMARY KEY,
          run_id TEXT NOT NULL DEFAULT '',
          source_kind TEXT NOT NULL,
          locator TEXT NOT NULL,
          window_start TEXT NOT NULL,
          window_end TEXT NOT NULL,
          raw_count INTEGER NOT NULL DEFAULT 0,
          accepted_count INTEGER NOT NULL DEFAULT 0,
          average_score REAL,
          score_p50 REAL,
          score_p90 REAL,
          signal_to_noise REAL,
          duplicate_rate REAL,
          delivery_contribution_count INTEGER NOT NULL DEFAULT 0,
          failure_count INTEGER NOT NULL DEFAULT 0,
          status TEXT NOT NULL,
          created_at TEXT NOT NULL,
          UNIQUE(run_id, source_kind, locator, window_start, window_end)
        );
        "#,
    )?;
    let insert_sql = if has_run_id {
        r#"
        INSERT OR REPLACE INTO radar_source_quality_v11
          (id, run_id, source_kind, locator, window_start, window_end, raw_count,
           accepted_count, average_score, score_p50, score_p90, signal_to_noise,
           duplicate_rate, delivery_contribution_count, failure_count, status, created_at)
        SELECT id, COALESCE(run_id, ''), source_kind, locator, window_start, window_end, raw_count,
               accepted_count, average_score, score_p50, score_p90, signal_to_noise,
               duplicate_rate, delivery_contribution_count, failure_count, status, created_at
        FROM radar_source_quality
        "#
    } else {
        r#"
        INSERT OR REPLACE INTO radar_source_quality_v11
          (id, run_id, source_kind, locator, window_start, window_end, raw_count,
           accepted_count, average_score, score_p50, score_p90, signal_to_noise,
           duplicate_rate, delivery_contribution_count, failure_count, status, created_at)
        SELECT id, '', source_kind, locator, window_start, window_end, raw_count,
               accepted_count, average_score, score_p50, score_p90, signal_to_noise,
               duplicate_rate, delivery_contribution_count, failure_count, status, created_at
        FROM radar_source_quality
        "#
    };
    conn.execute(insert_sql, [])?;
    conn.execute_batch(
        r#"
        DROP TABLE radar_source_quality;
        ALTER TABLE radar_source_quality_v11 RENAME TO radar_source_quality;
        "#,
    )?;
    Ok(())
}

pub(crate) fn ensure_worker_heartbeat_events_schema_on(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS worker_heartbeat_events (
          id TEXT PRIMARY KEY,
          worker_id TEXT NOT NULL,
          seen_at TEXT NOT NULL,
          processed_jobs INTEGER NOT NULL DEFAULT 0,
          last_error TEXT,
          created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_worker_heartbeat_events_worker_seen
        ON worker_heartbeat_events(worker_id, seen_at);

        INSERT OR IGNORE INTO worker_heartbeat_events
          (id, worker_id, seen_at, processed_jobs, last_error, created_at)
        SELECT
          'backfill:' || worker_id || ':' || last_seen_at,
          worker_id,
          last_seen_at,
          processed_jobs,
          last_error,
          last_seen_at
        FROM worker_heartbeats;
        "#,
    )?;
    Ok(())
}

pub(crate) fn repair_radar_source_quality_run_scope_on(conn: &Connection) -> Result<()> {
    ensure_radar_schema_on(conn)?;
    let create_sql: Option<String> = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'radar_source_quality'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    let Some(create_sql) = create_sql else {
        return Ok(());
    };
    let normalized = create_sql
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    if !normalized.contains("unique(run_id,source_kind,locator,window_start,window_end)") {
        migrate_radar_source_quality_windows_on(conn)?;
    }
    Ok(())
}

pub(crate) fn ensure_radar_schema_on(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS radar_profiles (
          id TEXT PRIMARY KEY,
          name TEXT NOT NULL UNIQUE,
          description TEXT NOT NULL DEFAULT '',
          status TEXT NOT NULL,
          window_hours INTEGER NOT NULL,
          min_score REAL NOT NULL,
          max_items INTEGER,
          languages_json TEXT NOT NULL,
          category_groups_json TEXT NOT NULL,
          source_selectors_json TEXT NOT NULL,
          delivery_policy_json TEXT NOT NULL,
          model_policy_json TEXT NOT NULL,
          metadata_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          CHECK(window_hours > 0),
          CHECK(min_score >= 0 AND min_score <= 10)
        );

        CREATE TABLE IF NOT EXISTS radar_runs (
          id TEXT PRIMARY KEY,
          profile_id TEXT NOT NULL,
          status TEXT NOT NULL,
          window_start TEXT NOT NULL,
          window_end TEXT NOT NULL,
          stage TEXT NOT NULL,
          source_selection_json TEXT NOT NULL,
          raw_count INTEGER NOT NULL DEFAULT 0,
          normalized_count INTEGER NOT NULL DEFAULT 0,
          indexed_count INTEGER NOT NULL DEFAULT 0,
          scored_count INTEGER NOT NULL DEFAULT 0,
          filtered_count INTEGER NOT NULL DEFAULT 0,
          enriched_count INTEGER NOT NULL DEFAULT 0,
          summary_count INTEGER NOT NULL DEFAULT 0,
          delivery_count INTEGER NOT NULL DEFAULT 0,
          error TEXT,
          metadata_json TEXT NOT NULL DEFAULT '{}',
          started_at TEXT NOT NULL,
          finished_at TEXT,
          updated_at TEXT NOT NULL,
          FOREIGN KEY(profile_id) REFERENCES radar_profiles(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_radar_runs_profile_updated ON radar_runs(profile_id, updated_at);
        CREATE INDEX IF NOT EXISTS idx_radar_runs_status ON radar_runs(status);

        CREATE TABLE IF NOT EXISTS radar_items (
          id TEXT PRIMARY KEY,
          run_id TEXT NOT NULL,
          stable_key TEXT NOT NULL,
          source_kind TEXT NOT NULL,
          provider TEXT NOT NULL,
          source_locator TEXT NOT NULL,
          native_id TEXT,
          canonical_url TEXT,
          title TEXT NOT NULL,
          author TEXT,
          published_at TEXT,
          fetched_at TEXT NOT NULL,
          content_text TEXT NOT NULL DEFAULT '',
          content_sha256 TEXT NOT NULL,
          metadata_json TEXT NOT NULL DEFAULT '{}',
          source_card_id TEXT,
          wiki_page_id TEXT,
          canonical_entity_ref TEXT,
          trust_level TEXT NOT NULL DEFAULT 'untrusted_external_evidence',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(run_id, stable_key),
          FOREIGN KEY(run_id) REFERENCES radar_runs(id) ON DELETE CASCADE,
          FOREIGN KEY(source_card_id) REFERENCES source_cards(id) ON DELETE SET NULL
        );

        CREATE INDEX IF NOT EXISTS idx_radar_items_run ON radar_items(run_id);
        CREATE INDEX IF NOT EXISTS idx_radar_items_source ON radar_items(source_kind, provider);
        CREATE INDEX IF NOT EXISTS idx_radar_items_source_card ON radar_items(source_card_id);

        CREATE VIRTUAL TABLE IF NOT EXISTS radar_item_fts
        USING fts5(id UNINDEXED, title, content_text, author, source_kind);

        CREATE TABLE IF NOT EXISTS radar_scores (
          id TEXT PRIMARY KEY,
          run_id TEXT NOT NULL,
          item_id TEXT NOT NULL,
          score_kind TEXT NOT NULL,
          score REAL NOT NULL,
          reason TEXT NOT NULL,
          tags_json TEXT NOT NULL DEFAULT '[]',
          model_provider TEXT,
          model_name TEXT,
          cost_decision_id TEXT,
          input_artifact_id TEXT,
          output_artifact_id TEXT,
          schema_version INTEGER NOT NULL,
          status TEXT NOT NULL,
          error TEXT,
          created_at TEXT NOT NULL,
          UNIQUE(item_id, score_kind, schema_version),
          FOREIGN KEY(run_id) REFERENCES radar_runs(id) ON DELETE CASCADE,
          FOREIGN KEY(item_id) REFERENCES radar_items(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_radar_scores_run_status ON radar_scores(run_id, status);

        CREATE TABLE IF NOT EXISTS radar_dedup_groups (
          id TEXT PRIMARY KEY,
          run_id TEXT NOT NULL,
          dedup_kind TEXT NOT NULL,
          primary_item_id TEXT NOT NULL,
          member_item_ids_json TEXT NOT NULL,
          reason TEXT NOT NULL,
          confidence REAL NOT NULL,
          model_provider TEXT,
          cost_decision_id TEXT,
          created_at TEXT NOT NULL,
          FOREIGN KEY(run_id) REFERENCES radar_runs(id) ON DELETE CASCADE,
          FOREIGN KEY(primary_item_id) REFERENCES radar_items(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_radar_dedup_groups_run ON radar_dedup_groups(run_id, dedup_kind);

        CREATE TABLE IF NOT EXISTS radar_summaries (
          id TEXT PRIMARY KEY,
          run_id TEXT NOT NULL,
          language TEXT NOT NULL,
          format TEXT NOT NULL,
          title TEXT NOT NULL,
          body_markdown TEXT NOT NULL,
          item_ids_json TEXT NOT NULL,
          source_card_ids_json TEXT NOT NULL,
          audit_status TEXT NOT NULL,
          metadata_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          UNIQUE(run_id, language, format),
          FOREIGN KEY(run_id) REFERENCES radar_runs(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS radar_deliveries (
          id TEXT PRIMARY KEY,
          run_id TEXT NOT NULL,
          summary_id TEXT NOT NULL,
          channel TEXT NOT NULL,
          recipient_ref TEXT NOT NULL,
          status TEXT NOT NULL,
          policy_decision_id TEXT,
          cost_decision_id TEXT,
          delivery_attempt_id TEXT,
          quiet_hours_deferred_until TEXT,
          idempotency_key TEXT NOT NULL UNIQUE,
          error TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          FOREIGN KEY(run_id) REFERENCES radar_runs(id) ON DELETE CASCADE,
          FOREIGN KEY(summary_id) REFERENCES radar_summaries(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS radar_schedule_ticks (
          id TEXT PRIMARY KEY,
          profile_id TEXT NOT NULL,
          tick_key TEXT NOT NULL UNIQUE,
          due_at TEXT NOT NULL,
          status TEXT NOT NULL,
          job_id TEXT,
          run_id TEXT,
          summary_id TEXT,
          delivery_id TEXT,
          error TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          FOREIGN KEY(profile_id) REFERENCES radar_profiles(id) ON DELETE CASCADE,
          FOREIGN KEY(job_id) REFERENCES wiki_jobs(id) ON DELETE SET NULL,
          FOREIGN KEY(run_id) REFERENCES radar_runs(id) ON DELETE SET NULL,
          FOREIGN KEY(summary_id) REFERENCES radar_summaries(id) ON DELETE SET NULL,
          FOREIGN KEY(delivery_id) REFERENCES radar_deliveries(id) ON DELETE SET NULL
        );

        CREATE INDEX IF NOT EXISTS idx_radar_schedule_ticks_profile_due
        ON radar_schedule_ticks(profile_id, due_at);

        CREATE INDEX IF NOT EXISTS idx_radar_schedule_ticks_status
        ON radar_schedule_ticks(status);

        CREATE TABLE IF NOT EXISTS radar_source_quality (
          id TEXT PRIMARY KEY,
          run_id TEXT NOT NULL DEFAULT '',
          source_kind TEXT NOT NULL,
          locator TEXT NOT NULL,
          window_start TEXT NOT NULL,
          window_end TEXT NOT NULL,
          raw_count INTEGER NOT NULL,
          accepted_count INTEGER NOT NULL,
          average_score REAL,
          score_p50 REAL,
          score_p90 REAL,
          signal_to_noise REAL,
          duplicate_rate REAL,
          delivery_contribution_count INTEGER NOT NULL DEFAULT 0,
          failure_count INTEGER NOT NULL DEFAULT 0,
          status TEXT NOT NULL,
          created_at TEXT NOT NULL,
          UNIQUE(run_id, source_kind, locator, window_start, window_end)
        );
        "#,
    )?;
    Ok(())
}

pub(crate) fn ensure_commerce_schema_on(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS commerce_run_configs (
          run_id TEXT PRIMARY KEY,
          domain_profile TEXT NOT NULL,
          target_qualified_count INTEGER NOT NULL,
          geography TEXT,
          freshness_window TEXT NOT NULL,
          allowed_private_context_sources_json TEXT NOT NULL DEFAULT '[]',
          allowed_public_source_families_json TEXT NOT NULL DEFAULT '[]',
          allow_marketplaces INTEGER NOT NULL DEFAULT 0,
          allow_chrome_profile INTEGER NOT NULL DEFAULT 0,
          max_provider_calls INTEGER,
          max_browser_pages INTEGER,
          max_cost_usd REAL,
          stop_rules_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          FOREIGN KEY(run_id) REFERENCES research_runs(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS commerce_candidates (
          id TEXT PRIMARY KEY,
          run_id TEXT NOT NULL,
          domain TEXT NOT NULL,
          source_url TEXT NOT NULL,
          retailer_or_provider TEXT NOT NULL,
          title TEXT NOT NULL,
          normalized_item_key TEXT NOT NULL,
          variant_key TEXT NOT NULL,
          price TEXT,
          currency TEXT,
          geography TEXT,
          candidate_status TEXT NOT NULL,
          score REAL,
          score_reasons_json TEXT NOT NULL DEFAULT '{}',
          disqualification_reasons_json TEXT NOT NULL DEFAULT '[]',
          metadata_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          FOREIGN KEY(run_id) REFERENCES research_runs(id) ON DELETE CASCADE,
          UNIQUE(run_id, source_url, normalized_item_key, variant_key)
        );
        CREATE INDEX IF NOT EXISTS idx_commerce_candidates_run_status
          ON commerce_candidates(run_id, candidate_status);
        CREATE INDEX IF NOT EXISTS idx_commerce_candidates_variant
          ON commerce_candidates(run_id, variant_key);

        CREATE TABLE IF NOT EXISTS commerce_availability_proofs (
          id TEXT PRIMARY KEY,
          run_id TEXT NOT NULL,
          candidate_id TEXT NOT NULL,
          proof_method TEXT NOT NULL,
          variant_key TEXT NOT NULL,
          variant_label TEXT NOT NULL,
          availability_state TEXT NOT NULL,
          visible_evidence TEXT,
          selector_or_dom_hint TEXT,
          screenshot_artifact_id TEXT,
          page_snapshot_artifact_id TEXT,
          confidence REAL NOT NULL,
          caveats_json TEXT NOT NULL DEFAULT '[]',
          checked_at TEXT NOT NULL,
          created_at TEXT NOT NULL,
          FOREIGN KEY(run_id) REFERENCES research_runs(id) ON DELETE CASCADE,
          FOREIGN KEY(candidate_id) REFERENCES commerce_candidates(id) ON DELETE CASCADE,
          FOREIGN KEY(screenshot_artifact_id) REFERENCES research_artifacts(id) ON DELETE SET NULL,
          FOREIGN KEY(page_snapshot_artifact_id) REFERENCES research_artifacts(id) ON DELETE SET NULL
        );
        CREATE INDEX IF NOT EXISTS idx_commerce_proofs_run_state
          ON commerce_availability_proofs(run_id, availability_state);
        CREATE INDEX IF NOT EXISTS idx_commerce_proofs_candidate
          ON commerce_availability_proofs(candidate_id, checked_at);

        CREATE TABLE IF NOT EXISTS commerce_context_facts (
          id TEXT PRIMARY KEY,
          run_id TEXT NOT NULL,
          fact_key TEXT NOT NULL,
          fact_kind TEXT NOT NULL,
          redacted_value TEXT NOT NULL,
          source_family TEXT NOT NULL,
          source_ref TEXT,
          confidence REAL NOT NULL,
          user_confirmed INTEGER NOT NULL DEFAULT 0,
          may_persist_to_memory INTEGER NOT NULL DEFAULT 0,
          metadata_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          FOREIGN KEY(run_id) REFERENCES research_runs(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_commerce_context_run_kind
          ON commerce_context_facts(run_id, fact_kind);

        CREATE TABLE IF NOT EXISTS commerce_verification_attempts (
          id TEXT PRIMARY KEY,
          run_id TEXT NOT NULL,
          candidate_id TEXT NOT NULL,
          attempted_at TEXT NOT NULL,
          method TEXT NOT NULL,
          result TEXT NOT NULL,
          error_kind TEXT,
          final_url TEXT,
          http_status INTEGER,
          browser_required INTEGER NOT NULL DEFAULT 0,
          chrome_profile_required INTEGER NOT NULL DEFAULT 0,
          artifact_ids_json TEXT NOT NULL DEFAULT '[]',
          next_action TEXT,
          created_at TEXT NOT NULL,
          FOREIGN KEY(run_id) REFERENCES research_runs(id) ON DELETE CASCADE,
          FOREIGN KEY(candidate_id) REFERENCES commerce_candidates(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_commerce_verification_run_result
          ON commerce_verification_attempts(run_id, result);

        CREATE TABLE IF NOT EXISTS commerce_report_judgments (
          id TEXT PRIMARY KEY,
          run_id TEXT NOT NULL,
          decision TEXT NOT NULL,
          blocking_findings_json TEXT NOT NULL DEFAULT '[]',
          non_blocking_findings_json TEXT NOT NULL DEFAULT '[]',
          claims_checked_json TEXT NOT NULL DEFAULT '[]',
          availability_proofs_checked_json TEXT NOT NULL DEFAULT '[]',
          privacy_review_json TEXT NOT NULL DEFAULT '{}',
          remaining_risks_json TEXT NOT NULL DEFAULT '[]',
          created_at TEXT NOT NULL,
          FOREIGN KEY(run_id) REFERENCES research_runs(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_commerce_report_judgments_run
          ON commerce_report_judgments(run_id, created_at);
        "#,
    )?;
    Ok(())
}

pub(crate) fn ensure_job_hunting_schema_on(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS job_candidate_profiles (
          id TEXT PRIMARY KEY,
          label TEXT NOT NULL UNIQUE,
          current_resume_source TEXT,
          linkedin_source TEXT,
          github_profile TEXT,
          blog_url TEXT,
          metadata_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS job_evidence_cards (
          id TEXT PRIMARY KEY,
          profile_id TEXT NOT NULL,
          title TEXT NOT NULL,
          evidence_type TEXT NOT NULL,
          visibility TEXT NOT NULL,
          summary TEXT NOT NULL,
          proof_url TEXT,
          local_path TEXT,
          source_date TEXT,
          confidence TEXT NOT NULL,
          tags_json TEXT NOT NULL DEFAULT '[]',
          safe_application_text TEXT NOT NULL,
          unsafe_terms_json TEXT NOT NULL DEFAULT '[]',
          metadata_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          FOREIGN KEY(profile_id) REFERENCES job_candidate_profiles(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_job_evidence_cards_profile_visibility
          ON job_evidence_cards(profile_id, visibility);
        CREATE INDEX IF NOT EXISTS idx_job_evidence_cards_type
          ON job_evidence_cards(evidence_type, confidence);

        CREATE TABLE IF NOT EXISTS job_evidence_claims (
          id TEXT PRIMARY KEY,
          evidence_card_id TEXT NOT NULL,
          claim TEXT NOT NULL,
          claim_kind TEXT NOT NULL,
          proof_level TEXT NOT NULL,
          can_use_in_resume INTEGER NOT NULL DEFAULT 0,
          can_use_in_outreach INTEGER NOT NULL DEFAULT 0,
          can_use_in_interview INTEGER NOT NULL DEFAULT 0,
          created_at TEXT NOT NULL,
          FOREIGN KEY(evidence_card_id) REFERENCES job_evidence_cards(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_job_evidence_claims_card
          ON job_evidence_claims(evidence_card_id);

        CREATE TABLE IF NOT EXISTS job_privacy_rules (
          id TEXT PRIMARY KEY,
          pattern TEXT NOT NULL,
          rule_type TEXT NOT NULL,
          severity TEXT NOT NULL,
          replacement_guidance TEXT,
          created_at TEXT NOT NULL,
          UNIQUE(pattern, rule_type)
        );
        CREATE INDEX IF NOT EXISTS idx_job_privacy_rules_severity
          ON job_privacy_rules(severity);

        CREATE TABLE IF NOT EXISTS job_privacy_checks (
          id TEXT PRIMARY KEY,
          artifact_type TEXT NOT NULL,
          artifact_id TEXT,
          checked_at TEXT NOT NULL,
          decision TEXT NOT NULL,
          findings_json TEXT NOT NULL DEFAULT '[]',
          checked_text_hash TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_job_privacy_checks_artifact
          ON job_privacy_checks(artifact_type, artifact_id, checked_at);

        CREATE TABLE IF NOT EXISTS job_sources (
          id TEXT PRIMARY KEY,
          source_family TEXT NOT NULL,
          name TEXT NOT NULL,
          url TEXT NOT NULL UNIQUE,
          market_scope TEXT NOT NULL,
          refresh_policy TEXT NOT NULL,
          metadata_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_job_sources_family_scope
          ON job_sources(source_family, market_scope);

        CREATE TABLE IF NOT EXISTS job_source_health (
          id TEXT PRIMARY KEY,
          source_id TEXT NOT NULL,
          checked_at TEXT NOT NULL,
          status TEXT NOT NULL,
          http_status INTEGER,
          error_code TEXT,
          fetched_count INTEGER NOT NULL DEFAULT 0,
          accepted_count INTEGER NOT NULL DEFAULT 0,
          rejected_count INTEGER NOT NULL DEFAULT 0,
          note TEXT,
          FOREIGN KEY(source_id) REFERENCES job_sources(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_job_source_health_source_checked
          ON job_source_health(source_id, checked_at);
        CREATE INDEX IF NOT EXISTS idx_job_source_health_status
          ON job_source_health(status);

        CREATE TABLE IF NOT EXISTS job_role_cards (
          id TEXT PRIMARY KEY,
          company TEXT NOT NULL,
          role_title TEXT NOT NULL,
          canonical_url TEXT,
          source_family TEXT NOT NULL,
          source_url TEXT NOT NULL,
          source_confidence TEXT NOT NULL,
          date_accessed TEXT NOT NULL,
          posting_freshness TEXT NOT NULL,
          location TEXT,
          work_mode TEXT,
          company_stage_or_size TEXT,
          role_seniority TEXT,
          core_requirements_json TEXT NOT NULL DEFAULT '[]',
          implied_business_problem TEXT,
          why_they_might_need_user TEXT,
          evidence_card_ids_json TEXT NOT NULL DEFAULT '[]',
          gaps_or_blockers_json TEXT NOT NULL DEFAULT '[]',
          cluster TEXT,
          current_status TEXT NOT NULL,
          metadata_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(company, role_title, source_url)
        );
        CREATE INDEX IF NOT EXISTS idx_job_role_cards_status_confidence
          ON job_role_cards(current_status, source_confidence);
        CREATE INDEX IF NOT EXISTS idx_job_role_cards_company
          ON job_role_cards(company);

        CREATE TABLE IF NOT EXISTS job_role_source_links (
          id TEXT PRIMARY KEY,
          role_id TEXT NOT NULL,
          source_id TEXT,
          source_url TEXT NOT NULL,
          observed_at TEXT NOT NULL,
          confidence TEXT NOT NULL,
          evidence_excerpt TEXT,
          FOREIGN KEY(role_id) REFERENCES job_role_cards(id) ON DELETE CASCADE,
          FOREIGN KEY(source_id) REFERENCES job_sources(id) ON DELETE SET NULL,
          UNIQUE(role_id, source_url)
        );
        CREATE INDEX IF NOT EXISTS idx_job_role_source_links_role
          ON job_role_source_links(role_id);

        CREATE TABLE IF NOT EXISTS job_fit_scores (
          id TEXT PRIMARY KEY,
          role_id TEXT NOT NULL,
          profile_id TEXT NOT NULL,
          scored_at TEXT NOT NULL,
          scorer TEXT NOT NULL,
          role_fit REAL NOT NULL,
          domain_fit REAL NOT NULL,
          evidence_fit REAL NOT NULL,
          geo_work_fit REAL NOT NULL,
          stage_fit REAL NOT NULL,
          practical_odds REAL NOT NULL,
          interest_energy REAL NOT NULL,
          weighted_score REAL NOT NULL,
          tier TEXT NOT NULL,
          blockers_json TEXT NOT NULL DEFAULT '[]',
          evidence_card_ids_json TEXT NOT NULL DEFAULT '[]',
          explanation TEXT NOT NULL,
          FOREIGN KEY(role_id) REFERENCES job_role_cards(id) ON DELETE CASCADE,
          FOREIGN KEY(profile_id) REFERENCES job_candidate_profiles(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_job_fit_scores_profile_tier
          ON job_fit_scores(profile_id, tier, weighted_score);
        CREATE INDEX IF NOT EXISTS idx_job_fit_scores_role_profile
          ON job_fit_scores(role_id, profile_id, scored_at);

        CREATE TABLE IF NOT EXISTS job_skeptic_findings (
          id TEXT PRIMARY KEY,
          role_id TEXT NOT NULL,
          severity TEXT NOT NULL,
          finding_type TEXT NOT NULL,
          finding TEXT NOT NULL,
          next_action TEXT,
          created_at TEXT NOT NULL,
          FOREIGN KEY(role_id) REFERENCES job_role_cards(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_job_skeptic_findings_role
          ON job_skeptic_findings(role_id, severity);

        CREATE TABLE IF NOT EXISTS job_application_packets (
          id TEXT PRIMARY KEY,
          role_id TEXT NOT NULL,
          profile_id TEXT NOT NULL,
          generated_at TEXT NOT NULL,
          status TEXT NOT NULL,
          evidence_card_ids_json TEXT NOT NULL DEFAULT '[]',
          resume_emphasis TEXT NOT NULL,
          tailored_bullets_json TEXT NOT NULL DEFAULT '[]',
          outreach_note TEXT NOT NULL,
          proof_links_json TEXT NOT NULL DEFAULT '{}',
          likely_objections_json TEXT NOT NULL DEFAULT '[]',
          interview_stories_json TEXT NOT NULL DEFAULT '[]',
          questions_to_ask_json TEXT NOT NULL DEFAULT '[]',
          privacy_check_id TEXT NOT NULL,
          reviewer_note TEXT,
          FOREIGN KEY(role_id) REFERENCES job_role_cards(id) ON DELETE CASCADE,
          FOREIGN KEY(profile_id) REFERENCES job_candidate_profiles(id) ON DELETE CASCADE,
          FOREIGN KEY(privacy_check_id) REFERENCES job_privacy_checks(id) ON DELETE RESTRICT
        );
        CREATE INDEX IF NOT EXISTS idx_job_application_packets_role_status
          ON job_application_packets(role_id, status);

        CREATE TABLE IF NOT EXISTS job_company_cards (
          id TEXT PRIMARY KEY,
          company_name TEXT NOT NULL,
          website_url TEXT NOT NULL UNIQUE,
          source_family TEXT NOT NULL,
          market TEXT NOT NULL,
          stage TEXT,
          funding_signal TEXT,
          product_category TEXT,
          technical_audience TEXT,
          developer_facing_score REAL NOT NULL,
          london_relevance TEXT NOT NULL,
          remote_maturity TEXT,
          hiring_page_url TEXT,
          founder_or_team_signal TEXT,
          last_checked_at TEXT NOT NULL,
          metadata_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_job_company_cards_market_score
          ON job_company_cards(market, developer_facing_score);

        CREATE TABLE IF NOT EXISTS job_contacts (
          id TEXT PRIMARY KEY,
          name TEXT NOT NULL,
          company_id TEXT,
          role_title TEXT,
          public_profile_url TEXT NOT NULL,
          source_url TEXT NOT NULL,
          relationship_status TEXT NOT NULL,
          relevance TEXT NOT NULL,
          note TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          FOREIGN KEY(company_id) REFERENCES job_company_cards(id) ON DELETE SET NULL,
          UNIQUE(public_profile_url)
        );
        CREATE INDEX IF NOT EXISTS idx_job_contacts_company_relevance
          ON job_contacts(company_id, relevance);

        CREATE TABLE IF NOT EXISTS job_intro_paths (
          id TEXT PRIMARY KEY,
          role_id TEXT NOT NULL,
          contact_id TEXT NOT NULL,
          path_type TEXT NOT NULL,
          confidence TEXT NOT NULL,
          next_action TEXT,
          status TEXT NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          FOREIGN KEY(role_id) REFERENCES job_role_cards(id) ON DELETE CASCADE,
          FOREIGN KEY(contact_id) REFERENCES job_contacts(id) ON DELETE CASCADE,
          UNIQUE(role_id, contact_id)
        );
        CREATE INDEX IF NOT EXISTS idx_job_intro_paths_role_status
          ON job_intro_paths(role_id, status);

        CREATE TABLE IF NOT EXISTS job_search_runs (
          id TEXT PRIMARY KEY,
          profile_id TEXT NOT NULL,
          scope TEXT NOT NULL,
          started_at TEXT NOT NULL,
          completed_at TEXT,
          proof_level TEXT NOT NULL,
          source_count INTEGER NOT NULL DEFAULT 0,
          role_count INTEGER NOT NULL DEFAULT 0,
          new_role_count INTEGER NOT NULL DEFAULT 0,
          stale_role_count INTEGER NOT NULL DEFAULT 0,
          error_count INTEGER NOT NULL DEFAULT 0,
          report_artifact_id TEXT,
          FOREIGN KEY(profile_id) REFERENCES job_candidate_profiles(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_job_search_runs_profile_started
          ON job_search_runs(profile_id, started_at);

        CREATE TABLE IF NOT EXISTS job_role_status_events (
          id TEXT PRIMARY KEY,
          role_id TEXT NOT NULL,
          run_id TEXT,
          status TEXT NOT NULL,
          previous_tier TEXT,
          current_tier TEXT,
          note TEXT,
          created_at TEXT NOT NULL,
          FOREIGN KEY(role_id) REFERENCES job_role_cards(id) ON DELETE CASCADE,
          FOREIGN KEY(run_id) REFERENCES job_search_runs(id) ON DELETE SET NULL
        );
        CREATE INDEX IF NOT EXISTS idx_job_role_status_events_role
          ON job_role_status_events(role_id, created_at);

        CREATE TABLE IF NOT EXISTS job_applications (
          id TEXT PRIMARY KEY,
          role_id TEXT NOT NULL,
          packet_id TEXT,
          status TEXT NOT NULL,
          applied_at TEXT,
          follow_up_at TEXT,
          outcome_note TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          FOREIGN KEY(role_id) REFERENCES job_role_cards(id) ON DELETE CASCADE,
          FOREIGN KEY(packet_id) REFERENCES job_application_packets(id) ON DELETE SET NULL
        );
        CREATE INDEX IF NOT EXISTS idx_job_applications_status
          ON job_applications(status, follow_up_at);

        CREATE TABLE IF NOT EXISTS job_weekly_reports (
          id TEXT PRIMARY KEY,
          profile_id TEXT NOT NULL,
          scope TEXT NOT NULL,
          generated_at TEXT NOT NULL,
          proof_level TEXT NOT NULL,
          body TEXT NOT NULL,
          metadata_json TEXT NOT NULL DEFAULT '{}',
          FOREIGN KEY(profile_id) REFERENCES job_candidate_profiles(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_job_weekly_reports_profile_generated
          ON job_weekly_reports(profile_id, generated_at);

        CREATE TABLE IF NOT EXISTS job_weekly_report_deliveries (
          id TEXT PRIMARY KEY,
          report_id TEXT NOT NULL,
          channel TEXT NOT NULL,
          subject TEXT NOT NULL,
          target TEXT NOT NULL,
          status TEXT NOT NULL,
          privacy_check_id TEXT,
          channel_message_id TEXT,
          idempotency_key TEXT NOT NULL,
          error TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          FOREIGN KEY(report_id) REFERENCES job_weekly_reports(id) ON DELETE CASCADE,
          FOREIGN KEY(privacy_check_id) REFERENCES job_privacy_checks(id) ON DELETE SET NULL,
          FOREIGN KEY(channel_message_id) REFERENCES channel_messages(id) ON DELETE SET NULL,
          UNIQUE(report_id, channel, subject, target, idempotency_key)
        );
        CREATE INDEX IF NOT EXISTS idx_job_weekly_report_deliveries_report
          ON job_weekly_report_deliveries(report_id, updated_at);
        CREATE INDEX IF NOT EXISTS idx_job_weekly_report_deliveries_status
          ON job_weekly_report_deliveries(status, updated_at);
        "#,
    )?;
    Ok(())
}
