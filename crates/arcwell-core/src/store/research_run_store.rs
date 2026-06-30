use super::*;

impl Store {
    pub fn ops_snapshot(&self) -> Result<OpsSnapshot> {
        let health = self.health()?;
        let worker = health.latest_worker_heartbeat.clone();
        Ok(OpsSnapshot {
            health,
            worker,
            x_stats: self.x_stats()?,
            radar_runs: self.list_radar_runs()?.into_iter().take(50).collect(),
            radar_source_quality: self.list_all_radar_source_quality()?,
            radar_deliveries: self.list_radar_deliveries(None)?,
            knowledge_adapter_runs: self.list_knowledge_adapter_runs(50)?,
            knowledge_entities: self.list_knowledge_entities(50)?,
            knowledge_entity_resolutions: self.list_knowledge_entity_resolutions(50)?,
            knowledge_relations: self.list_knowledge_relations(50)?,
            knowledge_events: self.list_knowledge_events(50)?,
            knowledge_clusters: self.list_knowledge_clusters(50)?,
            knowledge_editorial_decisions: self.list_knowledge_editorial_decisions(50)?,
            knowledge_reports: self.list_knowledge_reports(50)?,
            x_knowledge_clusters: self.list_x_knowledge_clusters(50)?,
            x_editorial_decisions: self.list_x_editorial_decisions(50)?,
            jobs: self.list_wiki_jobs()?,
            edge_events: self.list_edge_events()?,
            cursors: self.list_cursors()?,
            source_health: self.list_source_health()?,
            projects: self.list_projects()?,
            project_status_snapshots: self.list_recent_project_statuses(50)?,
            source_cards: self.list_source_cards()?,
            watch_sources: self.list_watch_sources()?,
            channel_messages: self.list_channel_messages()?,
            channel_delivery_attempts: self.list_channel_delivery_attempts(None)?,
            digest_candidates: self.list_digest_candidates()?,
            digest_deliveries: self.list_digest_deliveries(None)?,
            issue_schedules: self.list_issue_schedules()?,
            issue_schedule_ticks: self.list_issue_schedule_ticks(None)?,
            work_runs: self.search_work_runs(None, None, None, 50)?,
            procedures: self.search_procedures(None, Some("active"), 50)?,
            procedure_candidates: self.list_procedure_candidates("pending")?,
            job_hunting: self.job_ops_summary()?,
            memory_candidates: self.list_memory_candidates()?,
            memory_lifecycle_events: self.list_memory_lifecycle_events(50)?,
            memory_decisions: self.list_memory_decisions(50)?,
            memory_forget_tombstones: self.list_memory_forget_tombstones(50)?,
            import_runs: self.list_import_runs(50)?,
            controller_threads: self.list_controller_threads(None, None, 50)?,
            controller_runs: self.list_controller_runs(None, None, 50)?,
            controller_events: self.list_controller_events(None, None, 50)?,
            controller_pending_actions: self
                .list_controller_pending_actions(Some("pending"), 50)?,
            cost_policies: self.list_cost_policies()?,
            cost_decisions: self.list_cost_decisions(50)?,
            policy_decisions: self.list_policy_decisions(50)?,
            policy_approvals: self.list_policy_approvals(Some("pending"))?,
            secrets: self.list_secret_refs()?,
            secret_health: self.secret_health()?,
        })
    }

    pub fn create_research_plan(&self, query: &str, max_sources: usize) -> Result<ResearchPlan> {
        validate_query(query)?;
        let run = self.insert_research_run(query, "planned", None)?;
        let mut local_sources = self.search_wiki_pages_for_research(query)?;
        local_sources.truncate(max_sources);
        let suggested_searches = suggested_searches(query);
        let mut open_questions = vec![
            "What current sources should be checked with host-native web search?".to_string(),
            "Which claims are contradicted or stale in the local wiki?".to_string(),
            "What should be written back as source cards or a final brief?".to_string(),
        ];
        if local_sources.is_empty() {
            open_questions.insert(
                0,
                "No matching local wiki pages were found; web/search work is required.".to_string(),
            );
        }
        Ok(ResearchPlan {
            run,
            local_sources,
            suggested_searches,
            open_questions,
        })
    }

    pub fn create_research_brief_from_wiki(
        &self,
        query: &str,
        write_to_wiki: bool,
    ) -> Result<ResearchBrief> {
        validate_query(query)?;
        let run = self.insert_research_run(query, "drafting", None)?;
        let sources = self.search_wiki_pages_for_research(query)?;
        let source_cards: Vec<SourceCard> = self
            .search_source_cards(query)?
            .into_iter()
            .filter(source_card_is_primary_evidence)
            .collect();
        let markdown = self.render_wiki_research_brief(query, &sources, &source_cards)?;
        let result_page_id = if write_to_wiki {
            let page_id = self.add_wiki_page(
                &format!("Research Brief: {query}"),
                &markdown,
                &format!("research:{}", run.id),
            )?;
            self.update_research_run(&run.id, "completed", Some(&page_id))?;
            Some(page_id)
        } else {
            self.update_research_run(&run.id, "completed_no_write", None)?;
            None
        };
        let run = self
            .get_research_run(&run.id)?
            .context("research run disappeared")?;
        Ok(ResearchBrief {
            run,
            source_count: sources.len() + source_cards.len(),
            result_page_id,
            markdown,
        })
    }

    pub fn audit_research_output(&self, query: &str) -> Result<ResearchAuditReport> {
        validate_query(query)?;
        let source_cards = self.search_source_cards(query)?;
        let local_sources = self.search_wiki_pages_for_research(query)?;
        self.build_research_audit_report(query, source_cards, local_sources)
    }

    pub(crate) fn build_research_audit_report(
        &self,
        query: &str,
        source_cards: Vec<SourceCard>,
        local_sources: Vec<WikiPageSummary>,
    ) -> Result<ResearchAuditReport> {
        let mut findings = Vec::new();
        for card in &source_cards {
            findings.extend(audit_source_card(card));
        }
        findings.extend(detect_source_contradictions(&source_cards));
        if source_cards.is_empty() && local_sources.is_empty() {
            findings.push(ResearchAuditFinding {
                severity: "warning".to_string(),
                code: "no_grounding_sources".to_string(),
                source_card_id: None,
                message: "No local source cards or non-generated wiki sources match this query."
                    .to_string(),
                evidence: query.to_string(),
            });
        }
        let ok = !findings.iter().any(|finding| finding.severity == "error");
        let checklist = research_audit_checklist(&findings);
        Ok(ResearchAuditReport {
            query: query.to_string(),
            checked_at: now(),
            ok,
            source_card_count: source_cards.len(),
            local_source_count: local_sources.len(),
            findings,
            checklist,
        })
    }

    pub fn create_deep_research_run(&self, query: &str) -> Result<ResearchWorkflow> {
        validate_query(query)?;
        let run = self.insert_research_run(query, "deep_open", None)?;
        let tasks = research_role_instructions(query)
            .into_iter()
            .map(|(role, instructions)| self.insert_research_task(&run.id, role, &instructions))
            .collect::<Result<Vec<_>>>()?;
        Ok(ResearchWorkflow { run, tasks })
    }

    pub fn create_research_workflow(&self, query: &str) -> Result<ResearchWorkflow> {
        self.create_deep_research_run(query)
    }

    pub fn research_run_status(&self, run_id: &str) -> Result<ResearchRunStatus> {
        let run = self.require_research_run(run_id)?;
        let tasks = self.list_research_tasks(run_id)?;
        Ok(research_run_status_from_parts(run, &tasks))
    }

    pub fn read_research_run(&self, run_id: &str) -> Result<ResearchRunRead> {
        let run = self.require_research_run(run_id)?;
        let tasks = self.list_research_tasks(run_id)?;
        let role_runs = self.list_research_role_runs(run_id)?;
        let artifacts = self.list_research_artifacts(run_id)?;
        let host_searches = self.list_research_host_searches(run_id)?;
        let documents = self.list_research_documents(run_id)?;
        let editorial_runs = self.list_research_editorial_runs(run_id)?;
        let sources = self.list_research_run_sources(run_id)?;
        let claims = self.list_research_claims(run_id)?;
        let result_page = run
            .result_page_id
            .as_deref()
            .map(|page_id| {
                self.read_wiki_page(page_id)?
                    .with_context(|| format!("research result page not found: {page_id}"))
            })
            .transpose()?;
        Ok(ResearchRunRead {
            run,
            tasks,
            role_runs,
            artifacts,
            host_searches,
            documents,
            editorial_runs,
            sources,
            claims,
            convergence: self.research_convergence_status(run_id).ok(),
            result_page,
        })
    }

    pub fn audit_research_run(&self, run_id: &str) -> Result<ResearchRunAudit> {
        let run = self.require_research_run(run_id)?;
        let run_sources = self.list_research_run_sources(run_id)?;
        let claims = self.list_research_claims(run_id)?;
        let host_searches = self.list_research_host_searches(run_id)?;
        let documents = self.list_research_documents(run_id)?;
        let editorial_runs = self.list_research_editorial_runs(run_id)?;
        let mut source_cards = self.search_source_cards(&run.query)?;
        let mut seen: BTreeSet<String> = source_cards.iter().map(|card| card.id.clone()).collect();
        for card in run_sources
            .iter()
            .filter_map(|record| record.source_card.clone())
        {
            if seen.insert(card.id.clone()) {
                source_cards.push(card);
            }
        }
        let local_sources = self.search_wiki_pages_for_research(&run.query)?;
        let mut audit =
            self.build_research_audit_report(&run.query, source_cards, local_sources)?;
        audit
            .findings
            .extend(audit_research_run_corpus(&run_sources, &claims));
        audit.findings.extend(audit_research_host_search_proof(
            &run_sources,
            &host_searches,
        ));
        audit
            .findings
            .extend(audit_research_document_anchors(&claims, &documents));
        audit
            .findings
            .extend(audit_research_editorial_gates(&editorial_runs));
        audit.ok = !audit
            .findings
            .iter()
            .any(|finding| finding.severity == "error");
        audit.checklist = research_audit_checklist(&audit.findings);
        Ok(ResearchRunAudit { run, audit })
    }

    pub fn stop_research_run(&self, run_id: &str) -> Result<ResearchRunStatus> {
        let run = self.require_research_run(run_id)?;
        if matches!(run.status.as_str(), "completed" | "completed_no_write") {
            bail!("completed research run cannot be stopped: {run_id}");
        }
        self.update_research_run_status(run_id, "stopped")?;
        self.conn.execute(
            r#"
            UPDATE research_tasks
            SET status = 'cancelled', updated_at = ?2
            WHERE run_id = ?1 AND status = 'pending'
            "#,
            params![run_id, now()],
        )?;
        self.research_run_status(run_id)
    }

    pub fn list_research_tasks(&self, run_id: &str) -> Result<Vec<ResearchTask>> {
        validate_id(run_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, run_id, role, status, instructions, notes, created_at, updated_at
            FROM research_tasks
            WHERE run_id = ?1
            ORDER BY created_at ASC
            "#,
        )?;
        rows(stmt.query_map(params![run_id], research_task_from_row)?)
    }

    pub fn complete_research_task(&self, task_id: &str, notes: &str) -> Result<ResearchTask> {
        validate_id(task_id)?;
        validate_notes(notes)?;
        let changed = self.conn.execute(
            r#"
            UPDATE research_tasks
            SET status = 'completed', notes = ?2, updated_at = ?3
            WHERE id = ?1
            "#,
            params![task_id, notes, now()],
        )?;
        if changed == 0 {
            bail!("research task not found: {task_id}");
        }
        self.get_research_task(task_id)?
            .with_context(|| format!("completed research task not found: {task_id}"))
    }

    pub fn start_research_role_run(&self, input: ResearchRoleRunStart) -> Result<ResearchRoleRun> {
        let input = normalize_research_role_run_start(input)?;
        self.require_research_run(&input.run_id)?;
        for artifact_id in &input.input_artifact_ids {
            let artifact = self
                .read_research_artifact(artifact_id)?
                .with_context(|| format!("input artifact not found: {artifact_id}"))?;
            if artifact.run_id != input.run_id {
                bail!("input artifact belongs to a different research run");
            }
        }
        let id = research_role_run_id();
        let now = now();
        let input_artifact_ids_json = serde_json::to_string(&input.input_artifact_ids)?;
        self.conn.execute(
            r#"
            INSERT INTO research_role_runs
              (id, run_id, role, host, host_thread_id, host_subagent_id, tool_surface, prompt_version, prompt_hash, execution_mode, input_artifact_ids_json, status, started_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'running', ?12)
            "#,
            params![
                id,
                input.run_id,
                input.role,
                input.host,
                input.host_thread_id,
                input.host_subagent_id,
                input.tool_surface,
                input.prompt_version,
                input.prompt_hash,
                input.execution_mode,
                input_artifact_ids_json,
                now,
            ],
        )?;
        self.get_research_role_run(&id)?
            .with_context(|| format!("inserted research role run not found: {id}"))
    }

    pub fn finish_research_role_run(
        &self,
        role_run_id: &str,
        status: &str,
        output_artifact_id: Option<&str>,
        error_kind: Option<&str>,
        error_message: Option<&str>,
    ) -> Result<ResearchRoleRun> {
        validate_id(role_run_id)?;
        validate_research_role_run_finish(status, error_kind, error_message)?;
        let current = self
            .get_research_role_run(role_run_id)?
            .with_context(|| format!("research role run not found: {role_run_id}"))?;
        if current.status != "running" {
            bail!("research role run is not running: {role_run_id}");
        }
        if status == "completed" && output_artifact_id.is_none() {
            bail!("completed research role run requires an output artifact");
        }
        if let Some(artifact_id) = output_artifact_id {
            validate_id(artifact_id)?;
            let artifact = self
                .read_research_artifact(artifact_id)?
                .with_context(|| format!("output artifact not found: {artifact_id}"))?;
            if artifact.run_id != current.run_id {
                bail!("output artifact belongs to a different research run");
            }
            if artifact.role_run_id.as_deref() != Some(role_run_id) {
                bail!("output artifact is not linked to this role run");
            }
        }
        let error_kind = error_kind.map(str::trim).filter(|value| !value.is_empty());
        let error_message_redacted = error_message
            .map(|value| sanitize_work_text(value, 2_000))
            .transpose()?;
        self.conn.execute(
            r#"
            UPDATE research_role_runs
            SET status = ?2,
                output_artifact_id = ?3,
                finished_at = ?4,
                error_kind = ?5,
                error_message_redacted = ?6
            WHERE id = ?1
            "#,
            params![
                role_run_id,
                status,
                output_artifact_id,
                now(),
                error_kind,
                error_message_redacted,
            ],
        )?;
        self.get_research_role_run(role_run_id)?
            .with_context(|| format!("finished research role run not found: {role_run_id}"))
    }

    pub fn record_research_artifact(
        &self,
        input: ResearchArtifactInput,
    ) -> Result<ResearchArtifact> {
        let input = normalize_research_artifact_input(input)?;
        self.require_research_run(&input.run_id)?;
        if let Some(role_run_id) = &input.role_run_id {
            let role_run = self
                .get_research_role_run(role_run_id)?
                .with_context(|| format!("research role run not found: {role_run_id}"))?;
            if role_run.run_id != input.run_id {
                bail!("artifact role run belongs to a different research run");
            }
        }
        let id = research_artifact_id(&input.run_id, &input.artifact_type, &input.body);
        let body_sha256 = sha256(input.body.as_bytes());
        let metadata_json = serde_json::to_string(&input.metadata)?;
        self.conn.execute(
            r#"
            INSERT INTO research_artifacts
              (id, run_id, role_run_id, artifact_type, title, body, body_sha256, metadata_json, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(id) DO UPDATE SET
              role_run_id = excluded.role_run_id,
              artifact_type = excluded.artifact_type,
              title = excluded.title,
              body = excluded.body,
              body_sha256 = excluded.body_sha256,
              metadata_json = excluded.metadata_json
            "#,
            params![
                id,
                input.run_id,
                input.role_run_id,
                input.artifact_type,
                input.title,
                input.body,
                body_sha256,
                metadata_json,
                now(),
            ],
        )?;
        self.read_research_artifact(&id)?
            .with_context(|| format!("inserted research artifact not found: {id}"))
    }

    pub fn list_research_role_runs(&self, run_id: &str) -> Result<Vec<ResearchRoleRun>> {
        self.require_research_run(run_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, run_id, role, host, host_thread_id, host_subagent_id, tool_surface, prompt_version, prompt_hash, execution_mode, input_artifact_ids_json, output_artifact_id, status, started_at, finished_at, error_kind, error_message_redacted
            FROM research_role_runs
            WHERE run_id = ?1
            ORDER BY started_at ASC
            "#,
        )?;
        rows(stmt.query_map(params![run_id], research_role_run_from_row)?)
    }

    pub fn get_research_role_run(&self, id: &str) -> Result<Option<ResearchRoleRun>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, run_id, role, host, host_thread_id, host_subagent_id, tool_surface, prompt_version, prompt_hash, execution_mode, input_artifact_ids_json, output_artifact_id, status, started_at, finished_at, error_kind, error_message_redacted
                FROM research_role_runs
                WHERE id = ?1
                "#,
                params![id],
                research_role_run_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_research_artifacts(&self, run_id: &str) -> Result<Vec<ResearchArtifact>> {
        self.require_research_run(run_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, run_id, role_run_id, artifact_type, title, body, body_sha256, metadata_json, created_at
            FROM research_artifacts
            WHERE run_id = ?1
            ORDER BY created_at ASC
            "#,
        )?;
        rows(stmt.query_map(params![run_id], research_artifact_from_row)?)
    }

    pub fn read_research_artifact(&self, id: &str) -> Result<Option<ResearchArtifact>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, run_id, role_run_id, artifact_type, title, body, body_sha256, metadata_json, created_at
                FROM research_artifacts
                WHERE id = ?1
                "#,
                params![id],
                research_artifact_from_row,
            )
            .optional()
            .map_err(Into::into)
    }
}
