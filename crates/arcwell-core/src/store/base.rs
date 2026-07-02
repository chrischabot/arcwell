use super::*;

impl Store {
    pub fn open(paths: AppPaths) -> Result<Self> {
        paths.ensure()?;
        let conn = Connection::open(&paths.db)
            .with_context(|| format!("opening sqlite database {}", paths.db.display()))?;
        conn.busy_timeout(std::time::Duration::from_millis(5000))?;
        let store = Self { paths, conn };
        store.migrate()?;
        Ok(store)
    }

    pub fn paths(&self) -> &AppPaths {
        &self.paths
    }

    pub(crate) fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = WAL;
            PRAGMA busy_timeout = 5000;

            CREATE TABLE IF NOT EXISTS meta (
              key TEXT PRIMARY KEY,
              value TEXT NOT NULL
            );

            INSERT INTO meta (key, value)
            VALUES ('schema_version', '1')
            ON CONFLICT(key) DO NOTHING;

            CREATE TABLE IF NOT EXISTS schema_migrations (
              version INTEGER PRIMARY KEY,
              name TEXT NOT NULL,
              destructive INTEGER NOT NULL DEFAULT 0,
              backup_id TEXT,
              applied_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS profile_items (
              key TEXT PRIMARY KEY,
              value TEXT NOT NULL,
              sensitivity TEXT NOT NULL DEFAULT 'normal',
              source TEXT NOT NULL DEFAULT 'manual',
              updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS memories (
              id TEXT PRIMARY KEY,
              text TEXT NOT NULL,
              kind TEXT NOT NULL DEFAULT 'fact',
              sensitivity TEXT NOT NULL DEFAULT 'normal',
              source TEXT NOT NULL DEFAULT 'manual',
              user_id TEXT,
              confidence REAL NOT NULL DEFAULT 0.8,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS candidates (
              id TEXT PRIMARY KEY,
              target TEXT NOT NULL,
              kind TEXT NOT NULL,
              content TEXT NOT NULL,
              sensitivity TEXT NOT NULL DEFAULT 'normal',
              source_ref TEXT NOT NULL,
              status TEXT NOT NULL DEFAULT 'pending',
              created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS import_runs (
              id TEXT PRIMARY KEY,
              source_kind TEXT NOT NULL,
              source_path TEXT NOT NULL,
              mode TEXT NOT NULL,
              status TEXT NOT NULL,
              conversations_seen INTEGER NOT NULL DEFAULT 0,
              conversations_sampled INTEGER NOT NULL DEFAULT 0,
              candidates_seen INTEGER NOT NULL DEFAULT 0,
              candidates_sampled INTEGER NOT NULL DEFAULT 0,
              candidates_written INTEGER NOT NULL DEFAULT 0,
              duplicates_suppressed INTEGER NOT NULL DEFAULT 0,
              error TEXT,
              metadata_json TEXT NOT NULL DEFAULT '{}',
              started_at TEXT NOT NULL,
              finished_at TEXT
            );

            CREATE TABLE IF NOT EXISTS memory_lifecycle_events (
              id TEXT PRIMARY KEY,
              event_type TEXT NOT NULL,
              hook TEXT,
              user_id TEXT,
              source_ref TEXT,
              input TEXT,
              result_json TEXT NOT NULL DEFAULT '{}',
              status TEXT NOT NULL,
              created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS memory_decision_ledger (
              id TEXT PRIMARY KEY,
              user_id TEXT,
              source_ref TEXT NOT NULL,
              observation TEXT NOT NULL,
              operation TEXT NOT NULL,
              memory_id TEXT,
              candidate_id TEXT,
              confidence REAL NOT NULL,
              reason TEXT NOT NULL,
              metadata_json TEXT NOT NULL DEFAULT '{}',
              created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS memory_forget_tombstones (
              id TEXT PRIMARY KEY,
              user_id_hash TEXT NOT NULL,
              provider TEXT NOT NULL,
              provider_memories_deleted INTEGER NOT NULL DEFAULT 0,
              candidates_deleted INTEGER NOT NULL DEFAULT 0,
              compatibility_memories_deleted INTEGER NOT NULL DEFAULT 0,
              lifecycle_events_deleted INTEGER NOT NULL DEFAULT 0,
              decision_ledger_deleted INTEGER NOT NULL DEFAULT 0,
              policy TEXT NOT NULL,
              created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS cost_entries (
              id TEXT PRIMARY KEY,
              package TEXT NOT NULL,
              job_id TEXT NOT NULL,
              provider TEXT NOT NULL,
              model TEXT NOT NULL,
              source TEXT,
              estimated_usd REAL NOT NULL DEFAULT 0,
              actual_usd REAL NOT NULL DEFAULT 0,
              created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS cost_policies (
              scope TEXT NOT NULL,
              key TEXT NOT NULL,
              limit_usd REAL,
              kill_switch INTEGER NOT NULL DEFAULT 0,
              override_until TEXT,
              updated_at TEXT NOT NULL,
              PRIMARY KEY(scope, key)
            );

            CREATE TABLE IF NOT EXISTS cost_decisions (
              id TEXT PRIMARY KEY,
              allowed INTEGER NOT NULL,
              reason TEXT NOT NULL,
              package TEXT NOT NULL,
              job_id TEXT NOT NULL,
              provider TEXT NOT NULL,
              model TEXT NOT NULL,
              source TEXT,
              projected_usd REAL NOT NULL DEFAULT 0,
              spent_usd REAL NOT NULL DEFAULT 0,
              remaining_usd REAL,
              matched_scope TEXT,
              matched_key TEXT,
              created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS policy_decisions (
              id TEXT PRIMARY KEY,
              action TEXT NOT NULL,
              effect TEXT NOT NULL,
              allowed INTEGER NOT NULL,
              reason TEXT NOT NULL,
              matched_rule_id TEXT,
              approval_id TEXT,
              package TEXT,
              provider TEXT,
              source TEXT,
              channel TEXT,
              subject TEXT,
              target TEXT,
              metadata_json TEXT NOT NULL DEFAULT '{}',
              created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS policy_approvals (
              id TEXT PRIMARY KEY,
              decision_id TEXT NOT NULL,
              action TEXT NOT NULL,
              status TEXT NOT NULL,
              reason TEXT NOT NULL,
              created_at TEXT NOT NULL,
              resolved_at TEXT,
              FOREIGN KEY(decision_id) REFERENCES policy_decisions(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS secret_refs (
              name TEXT PRIMARY KEY,
              location TEXT NOT NULL,
              scope TEXT NOT NULL,
              expires_at TEXT,
              updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS secret_values (
              name TEXT PRIMARY KEY,
              value TEXT NOT NULL,
              scope TEXT NOT NULL,
              provider TEXT,
              expires_at TEXT,
              updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS backups (
              id TEXT PRIMARY KEY,
              path TEXT NOT NULL,
              manifest_sha256 TEXT NOT NULL,
              created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS worker_heartbeats (
              worker_id TEXT PRIMARY KEY,
              started_at TEXT NOT NULL,
              last_seen_at TEXT NOT NULL,
              processed_jobs INTEGER NOT NULL DEFAULT 0,
              last_error TEXT
            );

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

            CREATE TABLE IF NOT EXISTS wiki_pages (
              id TEXT PRIMARY KEY,
              title TEXT NOT NULL,
              path TEXT NOT NULL,
              content_sha256 TEXT NOT NULL,
              source TEXT NOT NULL DEFAULT 'unknown',
              status TEXT NOT NULL DEFAULT 'active',
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS wiki_pages_fts
            USING fts5(id UNINDEXED, title, content);

            CREATE TABLE IF NOT EXISTS wiki_editorial_decision_ledger (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              page_id TEXT NOT NULL,
              page_title TEXT NOT NULL DEFAULT '',
              decision TEXT NOT NULL CHECK (decision IN ('hold', 'reject')),
              reviewed_source_card_ids TEXT NOT NULL,
              source_set_hash TEXT NOT NULL,
              source_count INTEGER NOT NULL DEFAULT 0,
              rationale TEXT NOT NULL DEFAULT '',
              follow_up TEXT NOT NULL DEFAULT '',
              reviewed_at TEXT NOT NULL,
              first_seen_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              source_file TEXT NOT NULL DEFAULT '',
              UNIQUE(page_id, source_set_hash)
            );

            CREATE INDEX IF NOT EXISTS idx_wiki_editorial_decision_ledger_page
            ON wiki_editorial_decision_ledger(page_id);

            CREATE TABLE IF NOT EXISTS source_cards (
              id TEXT PRIMARY KEY,
              title TEXT NOT NULL,
              url TEXT NOT NULL,
              source_type TEXT NOT NULL,
              provider TEXT NOT NULL,
              summary TEXT NOT NULL,
              claims_json TEXT NOT NULL,
              retrieved_at TEXT NOT NULL,
              wiki_page_id TEXT NOT NULL,
              content_sha256 TEXT NOT NULL,
              metadata_json TEXT NOT NULL DEFAULT '{}',
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS source_health (
              key TEXT PRIMARY KEY,
              provider TEXT NOT NULL,
              source_kind TEXT NOT NULL,
              locator TEXT NOT NULL,
              status TEXT NOT NULL,
              last_success_at TEXT,
              last_failure_at TEXT,
              last_error TEXT,
              last_item_id TEXT,
              last_item_date TEXT,
              cursor_key TEXT,
              cursor_value TEXT,
              next_run_at TEXT,
              updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS watch_sources (
              id TEXT PRIMARY KEY,
              source_kind TEXT NOT NULL,
              locator TEXT NOT NULL,
              label TEXT NOT NULL,
              cadence TEXT NOT NULL,
              status TEXT NOT NULL,
              metadata_json TEXT NOT NULL,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              UNIQUE(source_kind, locator)
            );

            CREATE TABLE IF NOT EXISTS wiki_jobs (
              id TEXT PRIMARY KEY,
              kind TEXT NOT NULL,
              status TEXT NOT NULL,
              input_json TEXT NOT NULL,
              result_json TEXT,
              error TEXT,
              attempts INTEGER NOT NULL DEFAULT 0,
              max_attempts INTEGER NOT NULL DEFAULT 3,
              leased_until TEXT,
              worker_id TEXT,
              next_run_at TEXT,
              dead_lettered_at TEXT,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS cursors (
              key TEXT PRIMARY KEY,
              value TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS research_runs (
              id TEXT PRIMARY KEY,
              query TEXT NOT NULL,
              status TEXT NOT NULL,
              result_page_id TEXT,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS research_tasks (
              id TEXT PRIMARY KEY,
              run_id TEXT NOT NULL,
              role TEXT NOT NULL,
              status TEXT NOT NULL,
              instructions TEXT NOT NULL,
              notes TEXT,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              FOREIGN KEY(run_id) REFERENCES research_runs(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS research_role_runs (
              id TEXT PRIMARY KEY,
              run_id TEXT NOT NULL,
              role TEXT NOT NULL,
              host TEXT NOT NULL,
              host_thread_id TEXT,
              host_subagent_id TEXT,
              tool_surface TEXT,
              prompt_version TEXT NOT NULL,
              prompt_hash TEXT,
              execution_mode TEXT NOT NULL,
              input_artifact_ids_json TEXT NOT NULL DEFAULT '[]',
              output_artifact_id TEXT,
              status TEXT NOT NULL,
              started_at TEXT NOT NULL,
              finished_at TEXT,
              error_kind TEXT,
              error_message_redacted TEXT,
              FOREIGN KEY(run_id) REFERENCES research_runs(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS research_artifacts (
              id TEXT PRIMARY KEY,
              run_id TEXT NOT NULL,
              role_run_id TEXT,
              artifact_type TEXT NOT NULL,
              title TEXT NOT NULL,
              body TEXT NOT NULL,
              body_sha256 TEXT NOT NULL,
              metadata_json TEXT NOT NULL DEFAULT '{}',
              created_at TEXT NOT NULL,
              FOREIGN KEY(run_id) REFERENCES research_runs(id) ON DELETE CASCADE,
              FOREIGN KEY(role_run_id) REFERENCES research_role_runs(id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS research_host_searches (
              id TEXT PRIMARY KEY,
              run_id TEXT NOT NULL,
              role_run_id TEXT,
              host TEXT NOT NULL,
              tool_surface TEXT NOT NULL,
              query TEXT NOT NULL,
              query_intent TEXT,
              requested_recency INTEGER,
              requested_domains_json TEXT NOT NULL DEFAULT '[]',
              executed_at TEXT NOT NULL,
              retrieved_at TEXT NOT NULL,
              cost_decision_id TEXT,
              result_count INTEGER NOT NULL,
              status TEXT NOT NULL,
              error_kind TEXT,
              error_message_redacted TEXT,
              FOREIGN KEY(run_id) REFERENCES research_runs(id) ON DELETE CASCADE,
              FOREIGN KEY(role_run_id) REFERENCES research_role_runs(id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS research_host_search_results (
              id TEXT PRIMARY KEY,
              host_search_id TEXT NOT NULL,
              rank INTEGER NOT NULL,
              title TEXT NOT NULL,
              url TEXT NOT NULL,
              canonical_url TEXT NOT NULL,
              snippet TEXT,
              published_at TEXT,
              source_family_guess TEXT,
              provider_metadata_json TEXT NOT NULL DEFAULT '{}',
              selected_for_ingest INTEGER NOT NULL DEFAULT 0,
              research_source_id TEXT,
              source_card_id TEXT,
              UNIQUE(host_search_id, rank, canonical_url),
              FOREIGN KEY(host_search_id) REFERENCES research_host_searches(id) ON DELETE CASCADE,
              FOREIGN KEY(research_source_id) REFERENCES research_sources(id) ON DELETE SET NULL,
              FOREIGN KEY(source_card_id) REFERENCES source_cards(id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS research_documents (
              id TEXT PRIMARY KEY,
              run_id TEXT NOT NULL,
              research_source_id TEXT,
              source_card_id TEXT,
              url TEXT,
              local_path TEXT,
              media_type TEXT NOT NULL,
              byte_sha256 TEXT NOT NULL,
              byte_len INTEGER NOT NULL,
              retrieved_at TEXT NOT NULL,
              extractor_name TEXT NOT NULL,
              extractor_version TEXT NOT NULL,
              extraction_status TEXT NOT NULL,
              page_count INTEGER NOT NULL,
              sheet_count INTEGER NOT NULL,
              table_count INTEGER NOT NULL,
              warning_flags_json TEXT NOT NULL DEFAULT '[]',
              error_message_redacted TEXT,
              FOREIGN KEY(run_id) REFERENCES research_runs(id) ON DELETE CASCADE,
              FOREIGN KEY(research_source_id) REFERENCES research_sources(id) ON DELETE SET NULL,
              FOREIGN KEY(source_card_id) REFERENCES source_cards(id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS research_document_spans (
              id TEXT PRIMARY KEY,
              document_id TEXT NOT NULL,
              span_id TEXT NOT NULL,
              page_number INTEGER,
              section_label TEXT,
              char_start INTEGER NOT NULL,
              char_end INTEGER NOT NULL,
              text_sha256 TEXT NOT NULL,
              text_excerpt TEXT NOT NULL,
              bbox_json TEXT,
              confidence REAL NOT NULL,
              warning_flags_json TEXT NOT NULL DEFAULT '[]',
              UNIQUE(document_id, span_id),
              FOREIGN KEY(document_id) REFERENCES research_documents(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS research_tables (
              id TEXT PRIMARY KEY,
              document_id TEXT NOT NULL,
              table_id TEXT NOT NULL,
              page_number INTEGER,
              sheet_name TEXT,
              caption TEXT,
              bbox_json TEXT,
              row_count INTEGER NOT NULL,
              column_count INTEGER NOT NULL,
              extraction_method TEXT NOT NULL,
              confidence REAL NOT NULL,
              warning_flags_json TEXT NOT NULL DEFAULT '[]',
              UNIQUE(document_id, table_id),
              FOREIGN KEY(document_id) REFERENCES research_documents(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS research_table_cells (
              id TEXT PRIMARY KEY,
              table_id TEXT NOT NULL,
              row_index INTEGER NOT NULL,
              column_index INTEGER NOT NULL,
              row_header TEXT,
              column_header TEXT,
              raw_text TEXT NOT NULL,
              normalized_text TEXT NOT NULL,
              numeric_value REAL,
              unit TEXT,
              footnote_refs_json TEXT NOT NULL DEFAULT '[]',
              bbox_json TEXT,
              confidence REAL NOT NULL,
              UNIQUE(table_id, row_index, column_index),
              FOREIGN KEY(table_id) REFERENCES research_tables(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS research_editorial_runs (
              id TEXT PRIMARY KEY,
              run_id TEXT NOT NULL,
              stage TEXT NOT NULL,
              model_provider TEXT NOT NULL,
              model_name TEXT NOT NULL,
              prompt_version TEXT NOT NULL,
              input_artifact_hash TEXT,
              input_artifact_id TEXT,
              output_artifact_id TEXT,
              cost_decision_id TEXT,
              status TEXT NOT NULL,
              score_json TEXT NOT NULL DEFAULT '{}',
              error_message_redacted TEXT,
              created_at TEXT NOT NULL,
              FOREIGN KEY(run_id) REFERENCES research_runs(id) ON DELETE CASCADE,
              FOREIGN KEY(input_artifact_id) REFERENCES research_artifacts(id) ON DELETE SET NULL,
              FOREIGN KEY(output_artifact_id) REFERENCES research_artifacts(id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS research_sources (
              id TEXT PRIMARY KEY,
              url TEXT,
              local_ref TEXT,
              title TEXT NOT NULL,
              source_family TEXT NOT NULL,
              source_type TEXT NOT NULL,
              provider TEXT NOT NULL,
              author TEXT,
              published_at TEXT,
              language TEXT,
              priority INTEGER NOT NULL,
              reason TEXT NOT NULL,
              canonical_key TEXT NOT NULL UNIQUE,
              fetch_status TEXT NOT NULL,
              read_depth TEXT NOT NULL,
              metadata_json TEXT NOT NULL DEFAULT '{}',
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS research_run_sources (
              id TEXT PRIMARY KEY,
              run_id TEXT NOT NULL,
              source_id TEXT NOT NULL,
              source_card_id TEXT,
              triage_status TEXT NOT NULL,
              read_depth TEXT NOT NULL,
              notes TEXT,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              UNIQUE(run_id, source_id),
              FOREIGN KEY(run_id) REFERENCES research_runs(id) ON DELETE CASCADE,
              FOREIGN KEY(source_id) REFERENCES research_sources(id) ON DELETE CASCADE,
              FOREIGN KEY(source_card_id) REFERENCES source_cards(id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS research_claims (
              id TEXT PRIMARY KEY,
              run_id TEXT NOT NULL,
              text TEXT NOT NULL,
              kind TEXT NOT NULL,
              subject TEXT,
              predicate TEXT,
              object_value TEXT,
              temporal_scope TEXT,
              confidence REAL NOT NULL,
              caveats_json TEXT NOT NULL,
              extraction_provider TEXT NOT NULL,
              extraction_model TEXT NOT NULL,
              extracted_at TEXT NOT NULL,
              metadata_json TEXT NOT NULL DEFAULT '{}',
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              FOREIGN KEY(run_id) REFERENCES research_runs(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS research_claim_sources (
              id TEXT PRIMARY KEY,
              claim_id TEXT NOT NULL,
              source_card_id TEXT NOT NULL,
              quote TEXT,
              source_anchor TEXT,
              created_at TEXT NOT NULL,
              UNIQUE(claim_id, source_card_id),
              FOREIGN KEY(claim_id) REFERENCES research_claims(id) ON DELETE CASCADE,
              FOREIGN KEY(source_card_id) REFERENCES source_cards(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS research_claim_document_anchors (
              id TEXT PRIMARY KEY,
              claim_source_id TEXT NOT NULL,
              document_id TEXT NOT NULL,
              anchor_kind TEXT NOT NULL,
              document_span_id TEXT,
              table_id TEXT,
              table_cell_id TEXT,
              anchor_label TEXT NOT NULL,
              quote TEXT,
              created_at TEXT NOT NULL,
              CHECK(anchor_kind IN ('span', 'table', 'cell')),
              CHECK(
                (anchor_kind = 'span' AND document_span_id IS NOT NULL AND table_id IS NULL AND table_cell_id IS NULL)
                OR (anchor_kind = 'table' AND document_span_id IS NULL AND table_id IS NOT NULL AND table_cell_id IS NULL)
                OR (anchor_kind = 'cell' AND document_span_id IS NULL AND table_id IS NOT NULL AND table_cell_id IS NOT NULL)
              ),
              FOREIGN KEY(claim_source_id) REFERENCES research_claim_sources(id) ON DELETE CASCADE,
              FOREIGN KEY(document_id) REFERENCES research_documents(id) ON DELETE CASCADE,
              FOREIGN KEY(document_span_id) REFERENCES research_document_spans(id) ON DELETE CASCADE,
              FOREIGN KEY(table_id) REFERENCES research_tables(id) ON DELETE CASCADE,
              FOREIGN KEY(table_cell_id) REFERENCES research_table_cells(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS research_clusters (
              id TEXT PRIMARY KEY,
              run_id TEXT NOT NULL,
              theme TEXT NOT NULL,
              summary TEXT NOT NULL,
              claim_count INTEGER NOT NULL,
              evidence_strength TEXT NOT NULL,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              UNIQUE(run_id, theme),
              FOREIGN KEY(run_id) REFERENCES research_runs(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS research_cluster_claims (
              id TEXT PRIMARY KEY,
              cluster_id TEXT NOT NULL,
              claim_id TEXT NOT NULL,
              created_at TEXT NOT NULL,
              UNIQUE(cluster_id, claim_id),
              FOREIGN KEY(cluster_id) REFERENCES research_clusters(id) ON DELETE CASCADE,
              FOREIGN KEY(claim_id) REFERENCES research_claims(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS research_contradictions (
              id TEXT PRIMARY KEY,
              run_id TEXT NOT NULL,
              left_claim_id TEXT NOT NULL,
              right_claim_id TEXT NOT NULL,
              severity TEXT NOT NULL,
              notes TEXT NOT NULL,
              created_at TEXT NOT NULL,
              UNIQUE(run_id, left_claim_id, right_claim_id),
              FOREIGN KEY(run_id) REFERENCES research_runs(id) ON DELETE CASCADE,
              FOREIGN KEY(left_claim_id) REFERENCES research_claims(id) ON DELETE CASCADE,
              FOREIGN KEY(right_claim_id) REFERENCES research_claims(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS research_reports (
              id TEXT PRIMARY KEY,
              run_id TEXT NOT NULL,
              status TEXT NOT NULL,
              wiki_page_id TEXT,
              saturation_reason TEXT NOT NULL,
              markdown TEXT NOT NULL,
              created_at TEXT NOT NULL,
              FOREIGN KEY(run_id) REFERENCES research_runs(id) ON DELETE CASCADE,
              FOREIGN KEY(wiki_page_id) REFERENCES wiki_pages(id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS research_iterations (
              id TEXT PRIMARY KEY,
              run_id TEXT NOT NULL,
              iteration_index INTEGER NOT NULL,
              parent_iteration_id TEXT,
              status TEXT NOT NULL,
              objective TEXT NOT NULL,
              position_artifact_id TEXT,
              statement_set_artifact_id TEXT,
              challenge_pack_artifact_id TEXT,
              disproof_pack_artifact_id TEXT,
              revision_artifact_id TEXT,
              convergence_snapshot_id TEXT,
              cost_decision_id TEXT,
              started_at TEXT NOT NULL,
              completed_at TEXT,
              stop_reason TEXT,
              error_message_redacted TEXT,
              UNIQUE(run_id, iteration_index),
              FOREIGN KEY(run_id) REFERENCES research_runs(id) ON DELETE CASCADE,
              FOREIGN KEY(parent_iteration_id) REFERENCES research_iterations(id) ON DELETE SET NULL,
              FOREIGN KEY(position_artifact_id) REFERENCES research_artifacts(id) ON DELETE SET NULL,
              FOREIGN KEY(statement_set_artifact_id) REFERENCES research_artifacts(id) ON DELETE SET NULL,
              FOREIGN KEY(challenge_pack_artifact_id) REFERENCES research_artifacts(id) ON DELETE SET NULL,
              FOREIGN KEY(disproof_pack_artifact_id) REFERENCES research_artifacts(id) ON DELETE SET NULL,
              FOREIGN KEY(revision_artifact_id) REFERENCES research_artifacts(id) ON DELETE SET NULL,
              FOREIGN KEY(convergence_snapshot_id) REFERENCES research_convergence_snapshots(id) ON DELETE SET NULL
            );

            CREATE INDEX IF NOT EXISTS idx_research_iterations_run ON research_iterations(run_id, iteration_index);
            CREATE INDEX IF NOT EXISTS idx_research_iterations_status ON research_iterations(status);

            CREATE TABLE IF NOT EXISTS research_statements (
              id TEXT PRIMARY KEY,
              run_id TEXT NOT NULL,
              iteration_id TEXT NOT NULL,
              parent_statement_id TEXT,
              stable_key TEXT NOT NULL,
              statement_type TEXT NOT NULL,
              text TEXT NOT NULL,
              scope TEXT,
              temporal_scope TEXT,
              confidence REAL NOT NULL,
              certainty_label TEXT NOT NULL,
              status TEXT NOT NULL,
              importance TEXT NOT NULL,
              evidence_json TEXT NOT NULL DEFAULT '[]',
              counterevidence_json TEXT NOT NULL DEFAULT '[]',
              assumptions_json TEXT NOT NULL DEFAULT '[]',
              caveats_json TEXT NOT NULL DEFAULT '[]',
              created_by_role TEXT NOT NULL,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              UNIQUE(run_id, iteration_id, stable_key),
              FOREIGN KEY(run_id) REFERENCES research_runs(id) ON DELETE CASCADE,
              FOREIGN KEY(iteration_id) REFERENCES research_iterations(id) ON DELETE CASCADE,
              FOREIGN KEY(parent_statement_id) REFERENCES research_statements(id) ON DELETE SET NULL
            );

            CREATE INDEX IF NOT EXISTS idx_research_statements_run_status ON research_statements(run_id, status);
            CREATE INDEX IF NOT EXISTS idx_research_statements_stable_key ON research_statements(run_id, stable_key);

            CREATE TABLE IF NOT EXISTS research_challenges (
              id TEXT PRIMARY KEY,
              run_id TEXT NOT NULL,
              iteration_id TEXT NOT NULL,
              statement_id TEXT NOT NULL,
              challenge_type TEXT NOT NULL,
              severity TEXT NOT NULL,
              rationale TEXT NOT NULL,
              would_change_answer_if_true INTEGER NOT NULL,
              search_plan_json TEXT NOT NULL DEFAULT '{}',
              required_source_families_json TEXT NOT NULL DEFAULT '[]',
              status TEXT NOT NULL,
              created_by_role TEXT NOT NULL,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              UNIQUE(run_id, iteration_id, statement_id, challenge_type),
              FOREIGN KEY(run_id) REFERENCES research_runs(id) ON DELETE CASCADE,
              FOREIGN KEY(iteration_id) REFERENCES research_iterations(id) ON DELETE CASCADE,
              FOREIGN KEY(statement_id) REFERENCES research_statements(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_research_challenges_run_status ON research_challenges(run_id, status);
            CREATE INDEX IF NOT EXISTS idx_research_challenges_statement ON research_challenges(statement_id);

            CREATE TABLE IF NOT EXISTS research_disproofs (
              id TEXT PRIMARY KEY,
              run_id TEXT NOT NULL,
              iteration_id TEXT NOT NULL,
              challenge_id TEXT NOT NULL,
              statement_id TEXT NOT NULL,
              verdict TEXT NOT NULL,
              strength TEXT NOT NULL,
              evidence_json TEXT NOT NULL DEFAULT '[]',
              reasoning_summary TEXT NOT NULL,
              confidence_delta REAL NOT NULL,
              requires_revision INTEGER NOT NULL,
              created_by_role TEXT NOT NULL,
              created_at TEXT NOT NULL,
              UNIQUE(run_id, iteration_id, challenge_id),
              FOREIGN KEY(run_id) REFERENCES research_runs(id) ON DELETE CASCADE,
              FOREIGN KEY(iteration_id) REFERENCES research_iterations(id) ON DELETE CASCADE,
              FOREIGN KEY(challenge_id) REFERENCES research_challenges(id) ON DELETE CASCADE,
              FOREIGN KEY(statement_id) REFERENCES research_statements(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_research_disproofs_run_verdict ON research_disproofs(run_id, verdict);
            CREATE INDEX IF NOT EXISTS idx_research_disproofs_statement ON research_disproofs(statement_id);

            CREATE TABLE IF NOT EXISTS research_revisions (
              id TEXT PRIMARY KEY,
              run_id TEXT NOT NULL,
              iteration_id TEXT NOT NULL,
              from_statement_id TEXT NOT NULL,
              to_statement_id TEXT,
              revision_type TEXT NOT NULL,
              rationale TEXT NOT NULL,
              trigger_disproof_ids_json TEXT NOT NULL DEFAULT '[]',
              evidence_delta_json TEXT NOT NULL DEFAULT '[]',
              created_at TEXT NOT NULL,
              UNIQUE(run_id, iteration_id, from_statement_id, revision_type),
              FOREIGN KEY(run_id) REFERENCES research_runs(id) ON DELETE CASCADE,
              FOREIGN KEY(iteration_id) REFERENCES research_iterations(id) ON DELETE CASCADE,
              FOREIGN KEY(from_statement_id) REFERENCES research_statements(id) ON DELETE CASCADE,
              FOREIGN KEY(to_statement_id) REFERENCES research_statements(id) ON DELETE SET NULL
            );

            CREATE INDEX IF NOT EXISTS idx_research_revisions_run ON research_revisions(run_id, iteration_id);

            CREATE TABLE IF NOT EXISTS research_fact_checks (
              id TEXT PRIMARY KEY,
              run_id TEXT NOT NULL,
              iteration_id TEXT NOT NULL,
              statement_id TEXT NOT NULL,
              label TEXT NOT NULL,
              impact TEXT NOT NULL,
              evidence_json TEXT NOT NULL DEFAULT '[]',
              notes TEXT NOT NULL,
              created_at TEXT NOT NULL,
              UNIQUE(run_id, iteration_id, statement_id),
              FOREIGN KEY(run_id) REFERENCES research_runs(id) ON DELETE CASCADE,
              FOREIGN KEY(iteration_id) REFERENCES research_iterations(id) ON DELETE CASCADE,
              FOREIGN KEY(statement_id) REFERENCES research_statements(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_research_fact_checks_run_label ON research_fact_checks(run_id, label);

            CREATE TABLE IF NOT EXISTS research_convergence_snapshots (
              id TEXT PRIMARY KEY,
              run_id TEXT NOT NULL,
              iteration_id TEXT NOT NULL UNIQUE,
              source_count_total INTEGER NOT NULL,
              source_count_new INTEGER NOT NULL,
              primary_source_count_new INTEGER NOT NULL,
              claim_count_total INTEGER NOT NULL,
              statement_count_current INTEGER NOT NULL,
              statement_count_changed INTEGER NOT NULL,
              critical_open_challenges INTEGER NOT NULL,
              high_open_challenges INTEGER NOT NULL,
              strong_refutations INTEGER NOT NULL,
              unknown_high_impact_claims INTEGER NOT NULL,
              mean_confidence_delta REAL NOT NULL,
              max_confidence_delta REAL NOT NULL,
              source_novelty_score REAL NOT NULL,
              claim_novelty_score REAL NOT NULL,
              position_edit_distance REAL NOT NULL,
              citation_support_score REAL NOT NULL,
              active_fact_check_score REAL NOT NULL,
              evaluator_score REAL NOT NULL,
              cost_usd_estimated REAL NOT NULL,
              elapsed_seconds INTEGER NOT NULL,
              stop_rule_json TEXT NOT NULL DEFAULT '{}',
              settled INTEGER NOT NULL,
              created_at TEXT NOT NULL,
              FOREIGN KEY(run_id) REFERENCES research_runs(id) ON DELETE CASCADE,
              FOREIGN KEY(iteration_id) REFERENCES research_iterations(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_research_convergence_run ON research_convergence_snapshots(run_id, created_at);

            CREATE TABLE IF NOT EXISTS research_report_judgments (
              id TEXT PRIMARY KEY,
              run_id TEXT NOT NULL,
              report_id TEXT,
              judgment_version TEXT NOT NULL,
              overall_decision TEXT NOT NULL,
              scores_json TEXT NOT NULL DEFAULT '{}',
              blocking_findings_json TEXT NOT NULL DEFAULT '[]',
              non_blocking_findings_json TEXT NOT NULL DEFAULT '[]',
              evidence_checked_json TEXT NOT NULL DEFAULT '[]',
              remaining_risks_json TEXT NOT NULL DEFAULT '[]',
              commands_or_artifacts_reviewed_json TEXT NOT NULL DEFAULT '[]',
              created_at TEXT NOT NULL,
              FOREIGN KEY(run_id) REFERENCES research_runs(id) ON DELETE CASCADE,
              FOREIGN KEY(report_id) REFERENCES research_reports(id) ON DELETE SET NULL
            );

            CREATE INDEX IF NOT EXISTS idx_research_report_judgments_run ON research_report_judgments(run_id, created_at);

            CREATE TABLE IF NOT EXISTS x_items (
              id TEXT PRIMARY KEY,
              x_id TEXT NOT NULL UNIQUE,
              author TEXT NOT NULL,
              text TEXT NOT NULL,
              url TEXT NOT NULL,
              created_at TEXT,
              imported_at TEXT NOT NULL,
              retrieved_at TEXT,
              metrics_json TEXT NOT NULL DEFAULT '{}',
              raw_json TEXT NOT NULL DEFAULT '{}',
              source_card_id TEXT,
              wiki_page_id TEXT
            );

            CREATE TABLE IF NOT EXISTS x_item_sources (
              id TEXT PRIMARY KEY,
              x_id TEXT NOT NULL,
              source_kind TEXT NOT NULL,
              source_detail TEXT,
              seen_at TEXT NOT NULL,
              metadata_json TEXT NOT NULL DEFAULT '{}',
              UNIQUE(x_id, source_kind, source_detail)
            );

            CREATE INDEX IF NOT EXISTS idx_x_item_sources_x_id ON x_item_sources(x_id);
            CREATE INDEX IF NOT EXISTS idx_x_item_sources_kind ON x_item_sources(source_kind);

            CREATE TABLE IF NOT EXISTS edge_events (
              id TEXT PRIMARY KEY,
              source TEXT NOT NULL,
              idempotency_key TEXT NOT NULL UNIQUE,
              status TEXT NOT NULL,
              payload_json TEXT NOT NULL,
              attempts INTEGER NOT NULL DEFAULT 0,
              max_attempts INTEGER NOT NULL DEFAULT 3,
              leased_until TEXT,
              next_run_at TEXT,
              error TEXT,
              received_at TEXT NOT NULL,
              expires_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS channel_messages (
              id TEXT PRIMARY KEY,
              channel TEXT NOT NULL,
              direction TEXT NOT NULL,
              project_id TEXT,
              sender TEXT NOT NULL,
              body TEXT NOT NULL,
              status TEXT NOT NULL,
              source_event_id TEXT,
              created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS channel_authorizations (
              channel TEXT NOT NULL,
              subject TEXT NOT NULL,
              can_read_projects INTEGER NOT NULL DEFAULT 0,
              can_write_projects INTEGER NOT NULL DEFAULT 0,
              can_send INTEGER NOT NULL DEFAULT 0,
              updated_at TEXT NOT NULL,
              PRIMARY KEY(channel, subject)
            );

            CREATE TABLE IF NOT EXISTS channel_delivery_attempts (
              id TEXT PRIMARY KEY,
              message_id TEXT NOT NULL,
              channel TEXT NOT NULL,
              destination TEXT NOT NULL,
              attempt INTEGER NOT NULL,
              ok INTEGER NOT NULL,
              provider_status INTEGER NOT NULL,
              response_json TEXT NOT NULL,
              error TEXT,
              retry_at TEXT,
              created_at TEXT NOT NULL,
              FOREIGN KEY(message_id) REFERENCES channel_messages(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS channel_delivery_observations (
              id TEXT PRIMARY KEY,
              delivery_attempt_id TEXT NOT NULL,
              message_id TEXT NOT NULL,
              channel TEXT NOT NULL,
              destination TEXT NOT NULL,
              provider_message_id TEXT,
              observation_source TEXT NOT NULL,
              observation_status TEXT NOT NULL,
              mailbox_message_id TEXT,
              observed_at TEXT NOT NULL,
              evidence_json TEXT NOT NULL,
              created_at TEXT NOT NULL,
              FOREIGN KEY(delivery_attempt_id) REFERENCES channel_delivery_attempts(id) ON DELETE CASCADE,
              FOREIGN KEY(message_id) REFERENCES channel_messages(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS projects (
              id TEXT PRIMARY KEY,
              name TEXT NOT NULL,
              aliases_json TEXT NOT NULL,
              status TEXT NOT NULL,
              summary TEXT NOT NULL,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS project_status_snapshots (
              id TEXT PRIMARY KEY,
              project_id TEXT NOT NULL,
              status TEXT NOT NULL,
              summary TEXT NOT NULL,
              source TEXT NOT NULL,
              thread_ref TEXT,
              confidence REAL NOT NULL DEFAULT 0.5,
              created_at TEXT NOT NULL,
              live_verified INTEGER NOT NULL DEFAULT 0,
              verified_host TEXT,
              verified_thread_id TEXT,
              verified_at TEXT,
              stale_after_seconds INTEGER
            );

            CREATE TABLE IF NOT EXISTS controller_channel_contexts (
              id TEXT PRIMARY KEY,
              channel TEXT NOT NULL,
              account_id TEXT NOT NULL DEFAULT '',
              conversation_id TEXT NOT NULL,
              sender TEXT NOT NULL,
              trust_tier TEXT NOT NULL,
              last_project_id TEXT,
              last_thread_id TEXT,
              last_run_id TEXT,
              last_intent TEXT,
              updated_at TEXT NOT NULL,
              UNIQUE(channel, account_id, conversation_id, sender)
            );

            CREATE TABLE IF NOT EXISTS controller_threads (
              id TEXT PRIMARY KEY,
              host TEXT NOT NULL,
              host_thread_id TEXT NOT NULL,
              project_id TEXT,
              title TEXT,
              cwd TEXT,
              branch TEXT,
              worktree TEXT,
              status TEXT NOT NULL,
              active INTEGER NOT NULL DEFAULT 0,
              archived INTEGER NOT NULL DEFAULT 0,
              current_goal TEXT,
              latest_summary TEXT,
              latest_summary_source TEXT,
              last_activity_at TEXT,
              last_synced_at TEXT NOT NULL,
              UNIQUE(host, host_thread_id)
            );

            CREATE TABLE IF NOT EXISTS controller_runs (
              id TEXT PRIMARY KEY,
              thread_id TEXT,
              project_id TEXT,
              origin_channel_message_id TEXT,
              host TEXT NOT NULL,
              host_run_id TEXT,
              kind TEXT NOT NULL,
              status TEXT NOT NULL,
              requested_action TEXT NOT NULL,
              cancel_requested INTEGER NOT NULL DEFAULT 0,
              cancel_reason TEXT,
              started_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              finished_at TEXT
            );

            CREATE TABLE IF NOT EXISTS controller_events (
              id TEXT PRIMARY KEY,
              run_id TEXT,
              thread_id TEXT,
              project_id TEXT,
              event_type TEXT NOT NULL,
              summary TEXT NOT NULL,
              data_json TEXT NOT NULL DEFAULT '{}',
              source TEXT NOT NULL,
              created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS controller_pending_actions (
              id TEXT PRIMARY KEY,
              channel TEXT NOT NULL,
              conversation_id TEXT NOT NULL,
              sender TEXT NOT NULL,
              action_type TEXT NOT NULL,
              project_id TEXT,
              thread_id TEXT,
              run_id TEXT,
              payload_json TEXT NOT NULL DEFAULT '{}',
              reason TEXT NOT NULL,
              status TEXT NOT NULL,
              expires_at TEXT NOT NULL,
              created_at TEXT NOT NULL,
              resolved_at TEXT
            );

            CREATE TABLE IF NOT EXISTS controller_outbox (
              id TEXT PRIMARY KEY,
              channel TEXT NOT NULL,
              target TEXT NOT NULL,
              related_message_id TEXT,
              run_id TEXT,
              body TEXT NOT NULL,
              status TEXT NOT NULL,
              idempotency_key TEXT NOT NULL UNIQUE,
              created_at TEXT NOT NULL,
              delivered_at TEXT
            );

            CREATE TABLE IF NOT EXISTS work_runs (
              id TEXT PRIMARY KEY,
              goal TEXT NOT NULL,
              project_id TEXT,
              host_id TEXT,
              thread_id TEXT,
              agent_surface TEXT NOT NULL,
              status TEXT NOT NULL,
              outcome TEXT,
              validation_summary TEXT,
              follow_ups_json TEXT NOT NULL DEFAULT '[]',
              reusable_lessons_json TEXT NOT NULL DEFAULT '[]',
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              completed_at TEXT
            );

            CREATE TABLE IF NOT EXISTS work_events (
              id TEXT PRIMARY KEY,
              run_id TEXT NOT NULL,
              event_type TEXT NOT NULL,
              summary TEXT NOT NULL,
              data_json TEXT NOT NULL DEFAULT '{}',
              created_at TEXT NOT NULL,
              FOREIGN KEY(run_id) REFERENCES work_runs(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS work_artifacts (
              id TEXT PRIMARY KEY,
              run_id TEXT NOT NULL,
              artifact_type TEXT NOT NULL,
              locator TEXT NOT NULL,
              role TEXT NOT NULL,
              metadata_json TEXT NOT NULL DEFAULT '{}',
              created_at TEXT NOT NULL,
              FOREIGN KEY(run_id) REFERENCES work_runs(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS work_links (
              id TEXT PRIMARY KEY,
              run_id TEXT NOT NULL,
              target_type TEXT NOT NULL,
              target_id TEXT NOT NULL,
              role TEXT NOT NULL,
              generated_summary INTEGER NOT NULL DEFAULT 0,
              created_at TEXT NOT NULL,
              FOREIGN KEY(run_id) REFERENCES work_runs(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS procedures (
              id TEXT PRIMARY KEY,
              title TEXT NOT NULL,
              trigger_context TEXT NOT NULL,
              problem TEXT NOT NULL,
              preconditions_json TEXT NOT NULL DEFAULT '[]',
              tools_json TEXT NOT NULL DEFAULT '[]',
              validation_commands_json TEXT NOT NULL DEFAULT '[]',
              known_risks_json TEXT NOT NULL DEFAULT '[]',
              confidence REAL NOT NULL DEFAULT 0.7,
              freshness_days INTEGER NOT NULL DEFAULT 90,
              last_reviewed_at TEXT NOT NULL DEFAULT '',
              status TEXT NOT NULL,
              current_version INTEGER NOT NULL DEFAULT 0,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              archived_at TEXT
            );

            CREATE TABLE IF NOT EXISTS procedure_versions (
              id TEXT PRIMARY KEY,
              procedure_id TEXT NOT NULL,
              version INTEGER NOT NULL,
              method TEXT NOT NULL,
              source_run_ids_json TEXT NOT NULL DEFAULT '[]',
              provenance_json TEXT NOT NULL DEFAULT '{}',
              artifact_path TEXT NOT NULL,
              content_sha256 TEXT NOT NULL,
              created_at TEXT NOT NULL,
              UNIQUE(procedure_id, version),
              FOREIGN KEY(procedure_id) REFERENCES procedures(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS procedure_candidates (
              id TEXT PRIMARY KEY,
              operation TEXT NOT NULL,
              procedure_id TEXT,
              base_version INTEGER,
              title TEXT NOT NULL,
              trigger_context TEXT NOT NULL,
              problem TEXT NOT NULL,
              preconditions_json TEXT NOT NULL DEFAULT '[]',
              method TEXT NOT NULL,
              tools_json TEXT NOT NULL DEFAULT '[]',
              validation_commands_json TEXT NOT NULL DEFAULT '[]',
              known_risks_json TEXT NOT NULL DEFAULT '[]',
              source_run_ids_json TEXT NOT NULL DEFAULT '[]',
              provenance_json TEXT NOT NULL DEFAULT '{}',
              sensitivity TEXT NOT NULL,
              status TEXT NOT NULL,
              reason TEXT NOT NULL,
              content_sha256 TEXT NOT NULL,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              applied_at TEXT,
              rejected_reason TEXT,
              applied_result_json TEXT
            );

            CREATE TABLE IF NOT EXISTS digest_candidates (
              id TEXT PRIMARY KEY,
              topic TEXT NOT NULL,
              score REAL NOT NULL,
              reason TEXT NOT NULL,
              status TEXT NOT NULL,
              source_card_ids_json TEXT NOT NULL,
              review_status TEXT NOT NULL DEFAULT 'unreviewed',
              reviewed_at TEXT,
              reviewed_by TEXT,
              review_note TEXT,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS digest_deliveries (
              id TEXT PRIMARY KEY,
              candidate_id TEXT NOT NULL,
              channel TEXT NOT NULL,
              subject TEXT NOT NULL,
              target TEXT NOT NULL,
              idempotency_key TEXT NOT NULL,
              status TEXT NOT NULL,
              policy_decision_id TEXT,
              channel_message_id TEXT,
              channel_delivery_attempt_id TEXT,
              error TEXT,
              retry_at TEXT,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              UNIQUE(candidate_id, channel, subject, target, idempotency_key),
              FOREIGN KEY(candidate_id) REFERENCES digest_candidates(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS digest_alert_schedules (
              id TEXT PRIMARY KEY,
              name TEXT NOT NULL,
              status TEXT NOT NULL,
              channel TEXT NOT NULL,
              recipient_ref TEXT NOT NULL,
              min_score REAL NOT NULL,
              max_candidates INTEGER NOT NULL,
              interval_hours INTEGER NOT NULL,
              quiet_hours_json TEXT,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS digest_alert_ticks (
              id TEXT PRIMARY KEY,
              schedule_id TEXT NOT NULL,
              tick_key TEXT NOT NULL UNIQUE,
              due_at TEXT NOT NULL,
              status TEXT NOT NULL,
              job_id TEXT,
              candidate_ids_json TEXT NOT NULL DEFAULT '[]',
              delivery_ids_json TEXT NOT NULL DEFAULT '[]',
              error TEXT,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              FOREIGN KEY(schedule_id) REFERENCES digest_alert_schedules(id) ON DELETE CASCADE,
              FOREIGN KEY(job_id) REFERENCES wiki_jobs(id) ON DELETE SET NULL
            );

            CREATE INDEX IF NOT EXISTS idx_digest_alert_ticks_schedule_due
            ON digest_alert_ticks(schedule_id, due_at);

            CREATE INDEX IF NOT EXISTS idx_digest_alert_ticks_status
            ON digest_alert_ticks(status);

            CREATE TABLE IF NOT EXISTS issue_schedules (
              id TEXT PRIMARY KEY,
              name TEXT NOT NULL,
              status TEXT NOT NULL,
              kind TEXT NOT NULL,
              channel TEXT NOT NULL,
              recipient_ref TEXT NOT NULL,
              time_zone TEXT NOT NULL,
              hour INTEGER NOT NULL,
              minute INTEGER NOT NULL,
              catch_up_hours INTEGER NOT NULL,
              metadata_json TEXT NOT NULL DEFAULT '{}',
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              UNIQUE(kind, name)
            );

            CREATE TABLE IF NOT EXISTS issue_schedule_ticks (
              id TEXT PRIMARY KEY,
              schedule_id TEXT NOT NULL,
              tick_key TEXT NOT NULL UNIQUE,
              due_at TEXT NOT NULL,
              status TEXT NOT NULL,
              job_id TEXT,
              candidate_id TEXT,
              delivery_id TEXT,
              error TEXT,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              FOREIGN KEY(schedule_id) REFERENCES issue_schedules(id) ON DELETE CASCADE,
              FOREIGN KEY(job_id) REFERENCES wiki_jobs(id) ON DELETE SET NULL,
              FOREIGN KEY(candidate_id) REFERENCES digest_candidates(id) ON DELETE SET NULL,
              FOREIGN KEY(delivery_id) REFERENCES digest_deliveries(id) ON DELETE SET NULL
            );

            CREATE INDEX IF NOT EXISTS idx_issue_schedule_ticks_schedule_due
            ON issue_schedule_ticks(schedule_id, due_at);

            CREATE INDEX IF NOT EXISTS idx_issue_schedule_ticks_status
            ON issue_schedule_ticks(status);
            "#,
        )?;
        self.ensure_column(
            "wiki_jobs",
            "attempts",
            "ALTER TABLE wiki_jobs ADD COLUMN attempts INTEGER NOT NULL DEFAULT 0",
        )?;
        self.ensure_column(
            "wiki_jobs",
            "max_attempts",
            "ALTER TABLE wiki_jobs ADD COLUMN max_attempts INTEGER NOT NULL DEFAULT 3",
        )?;
        self.ensure_column(
            "wiki_jobs",
            "leased_until",
            "ALTER TABLE wiki_jobs ADD COLUMN leased_until TEXT",
        )?;
        self.ensure_column(
            "wiki_jobs",
            "worker_id",
            "ALTER TABLE wiki_jobs ADD COLUMN worker_id TEXT",
        )?;
        self.ensure_column(
            "wiki_jobs",
            "next_run_at",
            "ALTER TABLE wiki_jobs ADD COLUMN next_run_at TEXT",
        )?;
        self.ensure_column(
            "wiki_jobs",
            "dead_lettered_at",
            "ALTER TABLE wiki_jobs ADD COLUMN dead_lettered_at TEXT",
        )?;
        self.ensure_column(
            "wiki_pages",
            "source",
            "ALTER TABLE wiki_pages ADD COLUMN source TEXT NOT NULL DEFAULT 'unknown'",
        )?;
        self.ensure_column(
            "wiki_pages",
            "status",
            "ALTER TABLE wiki_pages ADD COLUMN status TEXT NOT NULL DEFAULT 'active'",
        )?;
        self.ensure_column(
            "memories",
            "user_id",
            "ALTER TABLE memories ADD COLUMN user_id TEXT",
        )?;
        self.ensure_column(
            "candidates",
            "operation",
            "ALTER TABLE candidates ADD COLUMN operation TEXT NOT NULL DEFAULT 'ADD'",
        )?;
        self.ensure_column(
            "candidates",
            "memory_id",
            "ALTER TABLE candidates ADD COLUMN memory_id TEXT",
        )?;
        self.ensure_column(
            "candidates",
            "user_id",
            "ALTER TABLE candidates ADD COLUMN user_id TEXT",
        )?;
        self.ensure_column(
            "candidates",
            "metadata_json",
            "ALTER TABLE candidates ADD COLUMN metadata_json TEXT NOT NULL DEFAULT '{}'",
        )?;
        self.ensure_column(
            "candidates",
            "applied_result_json",
            "ALTER TABLE candidates ADD COLUMN applied_result_json TEXT",
        )?;
        self.ensure_column(
            "candidates",
            "applied_at",
            "ALTER TABLE candidates ADD COLUMN applied_at TEXT",
        )?;
        self.ensure_column(
            "candidates",
            "rejected_reason",
            "ALTER TABLE candidates ADD COLUMN rejected_reason TEXT",
        )?;
        self.ensure_column(
            "memory_forget_tombstones",
            "decision_ledger_deleted",
            "ALTER TABLE memory_forget_tombstones ADD COLUMN decision_ledger_deleted INTEGER NOT NULL DEFAULT 0",
        )?;
        self.ensure_column(
            "cost_entries",
            "source",
            "ALTER TABLE cost_entries ADD COLUMN source TEXT",
        )?;
        self.ensure_column(
            "source_cards",
            "metadata_json",
            "ALTER TABLE source_cards ADD COLUMN metadata_json TEXT NOT NULL DEFAULT '{}'",
        )?;
        self.ensure_column(
            "digest_candidates",
            "review_status",
            "ALTER TABLE digest_candidates ADD COLUMN review_status TEXT NOT NULL DEFAULT 'unreviewed'",
        )?;
        self.ensure_column(
            "digest_candidates",
            "reviewed_at",
            "ALTER TABLE digest_candidates ADD COLUMN reviewed_at TEXT",
        )?;
        self.ensure_column(
            "digest_candidates",
            "reviewed_by",
            "ALTER TABLE digest_candidates ADD COLUMN reviewed_by TEXT",
        )?;
        self.ensure_column(
            "digest_candidates",
            "review_note",
            "ALTER TABLE digest_candidates ADD COLUMN review_note TEXT",
        )?;
        self.ensure_column(
            "x_items",
            "retrieved_at",
            "ALTER TABLE x_items ADD COLUMN retrieved_at TEXT",
        )?;
        self.ensure_column(
            "x_items",
            "metrics_json",
            "ALTER TABLE x_items ADD COLUMN metrics_json TEXT NOT NULL DEFAULT '{}'",
        )?;
        self.ensure_column(
            "x_items",
            "raw_json",
            "ALTER TABLE x_items ADD COLUMN raw_json TEXT NOT NULL DEFAULT '{}'",
        )?;
        self.ensure_x_canonical_schema()?;
        self.ensure_column(
            "procedures",
            "confidence",
            "ALTER TABLE procedures ADD COLUMN confidence REAL NOT NULL DEFAULT 0.7",
        )?;
        self.ensure_column(
            "procedures",
            "freshness_days",
            "ALTER TABLE procedures ADD COLUMN freshness_days INTEGER NOT NULL DEFAULT 90",
        )?;
        self.ensure_column(
            "procedures",
            "last_reviewed_at",
            "ALTER TABLE procedures ADD COLUMN last_reviewed_at TEXT NOT NULL DEFAULT ''",
        )?;
        self.ensure_column(
            "secret_values",
            "provider",
            "ALTER TABLE secret_values ADD COLUMN provider TEXT",
        )?;
        self.ensure_column(
            "secret_values",
            "expires_at",
            "ALTER TABLE secret_values ADD COLUMN expires_at TEXT",
        )?;
        self.ensure_column(
            "project_status_snapshots",
            "live_verified",
            "ALTER TABLE project_status_snapshots ADD COLUMN live_verified INTEGER NOT NULL DEFAULT 0",
        )?;
        self.ensure_column(
            "project_status_snapshots",
            "verified_host",
            "ALTER TABLE project_status_snapshots ADD COLUMN verified_host TEXT",
        )?;
        self.ensure_column(
            "project_status_snapshots",
            "verified_thread_id",
            "ALTER TABLE project_status_snapshots ADD COLUMN verified_thread_id TEXT",
        )?;
        self.ensure_column(
            "project_status_snapshots",
            "verified_at",
            "ALTER TABLE project_status_snapshots ADD COLUMN verified_at TEXT",
        )?;
        self.ensure_column(
            "project_status_snapshots",
            "stale_after_seconds",
            "ALTER TABLE project_status_snapshots ADD COLUMN stale_after_seconds INTEGER",
        )?;
        self.ensure_wiki_search_index()?;
        self.apply_schema_migration(1, "initial_core_schema", false, None, |_| Ok(()))?;
        self.apply_schema_migration(
            2,
            "compatibility_columns_deep_research_and_x_provenance",
            false,
            None,
            |_| Ok(()),
        )?;
        self.apply_schema_migration(3, "conversation_import_ledger", false, None, |_| Ok(()))?;
        self.apply_schema_migration(4, "controller_registry", false, None, |_| Ok(()))?;
        self.apply_schema_migration(
            5,
            "research_claim_document_anchors",
            false,
            None,
            |conn| {
                conn.execute_batch(
                    r#"
                    CREATE TABLE IF NOT EXISTS research_claim_document_anchors (
                      id TEXT PRIMARY KEY,
                      claim_source_id TEXT NOT NULL,
                      document_id TEXT NOT NULL,
                      anchor_kind TEXT NOT NULL,
                      document_span_id TEXT,
                      table_id TEXT,
                      table_cell_id TEXT,
                      anchor_label TEXT NOT NULL,
                      quote TEXT,
                      created_at TEXT NOT NULL,
                      CHECK(anchor_kind IN ('span', 'table', 'cell')),
                      CHECK(
                        (anchor_kind = 'span' AND document_span_id IS NOT NULL AND table_id IS NULL AND table_cell_id IS NULL)
                        OR (anchor_kind = 'table' AND document_span_id IS NULL AND table_id IS NOT NULL AND table_cell_id IS NULL)
                        OR (anchor_kind = 'cell' AND document_span_id IS NULL AND table_id IS NOT NULL AND table_cell_id IS NOT NULL)
                      ),
                      FOREIGN KEY(claim_source_id) REFERENCES research_claim_sources(id) ON DELETE CASCADE,
                      FOREIGN KEY(document_id) REFERENCES research_documents(id) ON DELETE CASCADE,
                      FOREIGN KEY(document_span_id) REFERENCES research_document_spans(id) ON DELETE CASCADE,
                      FOREIGN KEY(table_id) REFERENCES research_tables(id) ON DELETE CASCADE,
                      FOREIGN KEY(table_cell_id) REFERENCES research_table_cells(id) ON DELETE CASCADE
                    );
                    "#,
                )?;
                Ok(())
            },
        )?;
        self.apply_schema_migration(6, "canonical_x_schema", false, None, |conn| {
            ensure_x_canonical_schema_on(conn)?;
            backfill_x_canonical_from_compatibility_on(conn)?;
            rebuild_x_tweets_fts_on(conn)?;
            Ok(())
        })?;
        self.apply_schema_migration(8, "radar_core_schema", false, None, |conn| {
            ensure_radar_schema_on(conn)?;
            Ok(())
        })?;
        self.apply_schema_migration(9, "radar_dedup_groups", false, None, |conn| {
            ensure_radar_schema_on(conn)?;
            Ok(())
        })?;
        self.apply_schema_migration(10, "x_profile_identity_events", false, None, |conn| {
            ensure_x_canonical_schema_on(conn)?;
            Ok(())
        })?;
        self.apply_schema_migration(11, "radar_source_quality_windows", false, None, |conn| {
            migrate_radar_source_quality_windows_on(conn)
        })?;
        self.apply_schema_migration(12, "qualified_commerce_research", false, None, |conn| {
            ensure_commerce_schema_on(conn)
        })?;
        self.apply_schema_migration(13, "x_knowledge_clusters", false, None, |conn| {
            ensure_x_knowledge_schema_on(conn)
        })?;
        self.apply_schema_migration(14, "unified_knowledge_pipeline", false, None, |conn| {
            ensure_knowledge_schema_on(conn)
        })?;
        self.apply_schema_migration(15, "knowledge_entities_relations", false, None, |conn| {
            ensure_knowledge_schema_on(conn)
        })?;
        self.apply_schema_migration(
            16,
            "knowledge_adapter_contract_entity_resolution",
            false,
            None,
            |conn| ensure_knowledge_schema_on(conn),
        )?;
        self.apply_schema_migration(17, "worker_heartbeat_events", false, None, |conn| {
            ensure_worker_heartbeat_events_schema_on(conn)
        })?;
        self.apply_schema_migration(18, "issue_schedules", false, None, |conn| {
            ensure_issue_schedule_schema_on(conn)
        })?;
        self.apply_schema_migration(19, "job_hunting_intelligence", false, None, |conn| {
            ensure_job_hunting_schema_on(conn)
        })?;
        self.apply_schema_migration(
            20,
            "job_weekly_report_delivery_preparation",
            false,
            None,
            |conn| ensure_job_hunting_schema_on(conn),
        )?;
        self.apply_schema_migration(21, "x_watch_curation", false, None, |conn| {
            ensure_x_watch_curation_schema_on(conn)
        })?;
        self.apply_schema_migration(22, "proof_packet_ledger", false, None, |conn| {
            ensure_proof_packet_schema_on(conn)
        })?;
        self.apply_schema_migration(23, "guard_review_ledger", false, None, |conn| {
            ensure_guard_schema_on(conn)
        })?;
        self.apply_schema_migration(24, "adversarial_review_ledger", false, None, |conn| {
            ensure_proof_packet_schema_on(conn)
        })?;
        self.apply_schema_migration(
            25,
            "x_watch_curation_audit_retention",
            false,
            None,
            |conn| migrate_x_watch_curation_audit_retention_on(conn),
        )?;
        repair_radar_source_quality_run_scope_on(&self.conn)?;
        self.conn.execute(
            "UPDATE meta SET value = ?1 WHERE key = 'schema_version'",
            params![SCHEMA_VERSION.to_string()],
        )?;
        Ok(())
    }

    pub(crate) fn apply_schema_migration<F>(
        &self,
        version: i64,
        name: &str,
        destructive: bool,
        backup_id: Option<&str>,
        apply: F,
    ) -> Result<()>
    where
        F: FnOnce(&Connection) -> Result<()>,
    {
        if destructive && backup_id.is_none() {
            bail!("destructive migration {version} ({name}) requires a verified backup id");
        }
        let already_applied: Option<i64> = self
            .conn
            .query_row(
                "SELECT version FROM schema_migrations WHERE version = ?1",
                params![version],
                |row| row.get(0),
            )
            .optional()?;
        if already_applied.is_some() {
            return Ok(());
        }
        apply(&self.conn)?;
        self.conn.execute(
            r#"
            INSERT INTO schema_migrations (version, name, destructive, backup_id, applied_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            params![
                version,
                name,
                if destructive { 1 } else { 0 },
                backup_id,
                now()
            ],
        )?;
        Ok(())
    }

    pub(crate) fn ensure_column(&self, table: &str, column: &str, alter_sql: &str) -> Result<()> {
        ensure_column_on(&self.conn, table, column, alter_sql)
    }
}
