use super::*;

impl Store {
    pub fn create_project(
        &self,
        name: &str,
        summary: &str,
        aliases: &[String],
    ) -> Result<ProjectRecord> {
        validate_query(name)?;
        validate_notes(summary)?;
        for alias in aliases {
            validate_query(alias)?;
        }
        self.policy_guard(PolicyRequest {
            action: "project.write".to_string(),
            package: None,
            provider: None,
            source: Some("project_create".to_string()),
            channel: None,
            subject: None,
            target: None,
            projected_usd: None,
            metadata: json!({ "name": name, "aliases": aliases }),
            untrusted_excerpt: Some(summary.to_string()),
        })?;
        let id = Uuid::new_v4().to_string();
        let timestamp = now();
        self.conn.execute(
            r#"
            INSERT INTO projects (id, name, aliases_json, status, summary, created_at, updated_at)
            VALUES (?1, ?2, ?3, 'active', ?4, ?5, ?5)
            "#,
            params![
                id,
                name,
                serde_json::to_string(aliases)?,
                summary,
                timestamp
            ],
        )?;
        self.get_project(&id)?
            .with_context(|| format!("inserted project not found: {id}"))
    }

    pub fn list_projects(&self) -> Result<Vec<ProjectRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, aliases_json, status, summary, created_at, updated_at FROM projects ORDER BY updated_at DESC",
        )?;
        rows(stmt.query_map([], project_from_row)?)
    }

    pub fn get_project(&self, id: &str) -> Result<Option<ProjectRecord>> {
        validate_id(id)?;
        self.conn
            .query_row(
                "SELECT id, name, aliases_json, status, summary, created_at, updated_at FROM projects WHERE id = ?1",
                params![id],
                project_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn resolve_project(
        &self,
        query: &str,
        context_project_id: Option<&str>,
    ) -> Result<ProjectResolution> {
        validate_query(query)?;
        let normalized = query.to_ascii_lowercase();
        let projects = self.list_projects()?;
        if is_followup_project_query(&normalized)
            && let Some(id) = context_project_id
            && let Some(project) = self.get_project(id)?
        {
            let latest_status = self.latest_project_status(&project.id)?;
            let live_state = project_live_state(latest_status.as_ref());
            return Ok(ProjectResolution {
                project,
                confidence: 0.65,
                matched_alias: Some("context".to_string()),
                latest_status,
                live_state_available: live_state.available,
                live_state_source: live_state.source.clone(),
                live_state,
            });
        }
        let mut matches = Vec::new();
        for project in projects {
            let mut best_alias = None;
            let mut score = 0.0_f64;
            for alias in std::iter::once(&project.name).chain(project.aliases.iter()) {
                let alias_norm = alias.to_ascii_lowercase();
                if normalized.contains(&alias_norm) || alias_norm.contains(&normalized) {
                    score = score.max(if alias_norm == normalized { 1.0 } else { 0.8 });
                    best_alias = Some(alias.clone());
                }
            }
            if score > 0.0 {
                matches.push((project, score, best_alias));
            }
        }
        if matches.is_empty() {
            bail!("no matching project");
        }
        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        if matches.len() > 1 && (matches[0].1 - matches[1].1).abs() < 0.01 {
            bail!("ambiguous project reference");
        }
        let (project, confidence, matched_alias) = matches.remove(0);
        let latest_status = self.latest_project_status(&project.id)?;
        let live_state = project_live_state(latest_status.as_ref());
        Ok(ProjectResolution {
            project,
            confidence,
            matched_alias,
            latest_status,
            live_state_available: live_state.available,
            live_state_source: live_state.source.clone(),
            live_state,
        })
    }

    pub(crate) fn project_resolution_for_id(
        &self,
        project_id: &str,
        confidence: f64,
    ) -> Result<ProjectResolution> {
        validate_id(project_id)?;
        let project = self
            .get_project(project_id)?
            .with_context(|| format!("project not found: {project_id}"))?;
        let latest_status = self.latest_project_status(&project.id)?;
        let live_state = project_live_state(latest_status.as_ref());
        Ok(ProjectResolution {
            project,
            confidence: confidence.clamp(0.0, 1.0),
            matched_alias: Some("channel_message_project".to_string()),
            latest_status,
            live_state_available: live_state.available,
            live_state_source: live_state.source.clone(),
            live_state,
        })
    }

    pub fn record_project_status(
        &self,
        project_id: &str,
        status: &str,
        summary: &str,
        source: &str,
        thread_ref: Option<&str>,
        confidence: f64,
    ) -> Result<ProjectStatusSnapshot> {
        validate_id(project_id)?;
        self.get_project(project_id)?
            .with_context(|| format!("project not found: {project_id}"))?;
        validate_key(status)?;
        validate_notes(summary)?;
        validate_key(source)?;
        if let Some(thread_ref) = thread_ref {
            validate_notes(thread_ref)?;
        }
        validate_manual_project_status_source(source)?;
        self.policy_guard(PolicyRequest {
            action: "project.write".to_string(),
            package: None,
            provider: None,
            source: Some(source.to_string()),
            channel: None,
            subject: None,
            target: Some(project_id.to_string()),
            projected_usd: None,
            metadata: json!({ "status": status, "thread_ref": thread_ref }),
            untrusted_excerpt: Some(summary.to_string()),
        })?;
        let confidence = confidence.clamp(0.0, 1.0);
        let id = Uuid::new_v4().to_string();
        let created_at = now();
        self.conn.execute(
            r#"
            INSERT INTO project_status_snapshots
              (id, project_id, status, summary, source, thread_ref, confidence, created_at,
               live_verified, verified_host, verified_thread_id, verified_at, stale_after_seconds)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, NULL, NULL, NULL, NULL)
            "#,
            params![
                id, project_id, status, summary, source, thread_ref, confidence, created_at
            ],
        )?;
        self.conn.execute(
            r#"
            UPDATE projects
            SET status = ?2, summary = ?3, updated_at = ?4
            WHERE id = ?1
            "#,
            params![project_id, status, summary, created_at],
        )?;
        self.latest_project_status(project_id)?
            .with_context(|| format!("inserted project status not found: {id}"))
    }

    // allow: refactoring this N-arg signature is out of scope for the lint-cleanup pass.
    #[allow(clippy::too_many_arguments)]
    pub fn record_verified_project_status_sync(
        &self,
        project_id: &str,
        status: &str,
        summary: &str,
        host: &str,
        thread_id: &str,
        confidence: f64,
        stale_after_seconds: Option<i64>,
    ) -> Result<ProjectStatusSnapshot> {
        validate_id(project_id)?;
        self.get_project(project_id)?
            .with_context(|| format!("project not found: {project_id}"))?;
        validate_key(status)?;
        validate_notes(summary)?;
        validate_notes(thread_id)?;
        let host = normalize_project_sync_host(host)?;
        let stale_after_seconds = stale_after_seconds
            .unwrap_or(PROJECT_SYNC_DEFAULT_STALE_AFTER_SECONDS)
            .clamp(60, PROJECT_SYNC_MAX_STALE_AFTER_SECONDS);
        let source = project_sync_source(host);
        let thread_ref = format!("{host}:{thread_id}");
        self.policy_guard(PolicyRequest {
            action: "project.write".to_string(),
            package: None,
            provider: None,
            source: Some(source.clone()),
            channel: None,
            subject: None,
            target: Some(project_id.to_string()),
            projected_usd: None,
            metadata: json!({
                "status": status,
                "thread_ref": thread_ref,
                "verified_host": host,
                "verified_thread_id": thread_id,
                "stale_after_seconds": stale_after_seconds
            }),
            untrusted_excerpt: Some(summary.to_string()),
        })?;
        let confidence = confidence.clamp(0.0, 1.0);
        let id = Uuid::new_v4().to_string();
        let created_at = now();
        self.conn.execute(
            r#"
            INSERT INTO project_status_snapshots
              (id, project_id, status, summary, source, thread_ref, confidence, created_at,
               live_verified, verified_host, verified_thread_id, verified_at, stale_after_seconds)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9, ?10, ?8, ?11)
            "#,
            params![
                id,
                project_id,
                status,
                summary,
                source,
                thread_ref,
                confidence,
                created_at,
                host,
                thread_id,
                stale_after_seconds
            ],
        )?;
        self.conn.execute(
            r#"
            UPDATE projects
            SET status = ?2, summary = ?3, updated_at = ?4
            WHERE id = ?1
            "#,
            params![project_id, status, summary, created_at],
        )?;
        self.latest_project_status(project_id)?
            .with_context(|| format!("inserted verified project sync not found: {id}"))
    }

    pub fn latest_project_status(&self, project_id: &str) -> Result<Option<ProjectStatusSnapshot>> {
        validate_id(project_id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, project_id, status, summary, source, thread_ref, confidence, created_at,
                       live_verified, verified_host, verified_thread_id, verified_at, stale_after_seconds
                FROM project_status_snapshots
                WHERE project_id = ?1
                ORDER BY created_at DESC
                LIMIT 1
                "#,
                params![project_id],
                project_status_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn project_status_report(&self, project_id: &str) -> Result<ProjectStatusReport> {
        validate_id(project_id)?;
        let project = self
            .get_project(project_id)?
            .with_context(|| format!("project not found: {project_id}"))?;
        let latest_status = self.latest_project_status(project_id)?;
        let live_state = project_live_state(latest_status.as_ref());
        let provenance = latest_status
            .as_ref()
            .map(project_status_provenance)
            .into_iter()
            .collect();
        Ok(ProjectStatusReport {
            project,
            latest_status,
            live_state,
            provenance,
        })
    }

    pub fn project_status_report_for_channel(
        &self,
        project_id: &str,
        channel: Option<&str>,
        subject: Option<&str>,
    ) -> Result<ProjectStatusReport> {
        match (channel, subject) {
            (Some(channel), Some(subject)) => {
                if !self.channel_subject_can_read_projects(channel, subject)? {
                    bail!("{channel} subject is not authorized to read project state: {subject}");
                }
            }
            (None, None) => {}
            _ => bail!("channel project reads require both channel and subject"),
        }
        self.project_status_report(project_id)
    }

    pub fn list_project_statuses(&self, project_id: &str) -> Result<Vec<ProjectStatusSnapshot>> {
        validate_id(project_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, project_id, status, summary, source, thread_ref, confidence, created_at,
                   live_verified, verified_host, verified_thread_id, verified_at, stale_after_seconds
            FROM project_status_snapshots
            WHERE project_id = ?1
            ORDER BY created_at DESC
            "#,
        )?;
        rows(stmt.query_map(params![project_id], project_status_from_row)?)
    }

    pub fn list_recent_project_statuses(&self, limit: usize) -> Result<Vec<ProjectStatusSnapshot>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, project_id, status, summary, source, thread_ref, confidence, created_at,
                   live_verified, verified_host, verified_thread_id, verified_at, stale_after_seconds
            FROM project_status_snapshots
            ORDER BY created_at DESC
            LIMIT ?1
            "#,
        )?;
        rows(stmt.query_map(params![limit.clamp(1, 200) as i64], project_status_from_row)?)
    }

    // allow: refactoring this N-arg signature is out of scope for the lint-cleanup pass.
    #[allow(clippy::too_many_arguments)]
    pub fn upsert_controller_context(
        &self,
        channel: &str,
        account_id: Option<&str>,
        conversation_id: &str,
        sender: &str,
        trust_tier: &str,
        last_project_id: Option<&str>,
        last_thread_id: Option<&str>,
        last_run_id: Option<&str>,
        last_intent: Option<&str>,
    ) -> Result<ControllerChannelContext> {
        validate_key(channel)?;
        let account_id = account_id.unwrap_or("");
        validate_optional_controller_ref(account_id, "account id")?;
        validate_controller_ref(conversation_id, "conversation id")?;
        validate_query(sender)?;
        validate_key(trust_tier)?;
        if let Some(project_id) = last_project_id {
            validate_id(project_id)?;
            self.get_project(project_id)?
                .with_context(|| format!("project not found: {project_id}"))?;
        }
        if let Some(thread_id) = last_thread_id {
            validate_id(thread_id)?;
        }
        if let Some(run_id) = last_run_id {
            validate_id(run_id)?;
        }
        if let Some(intent) = last_intent {
            validate_key(intent)?;
        }
        let existing_id = self
            .conn
            .query_row(
                r#"
                SELECT id FROM controller_channel_contexts
                WHERE channel = ?1 AND account_id = ?2 AND conversation_id = ?3 AND sender = ?4
                "#,
                params![channel, account_id, conversation_id, sender],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        let id = existing_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let updated_at = now();
        self.conn.execute(
            r#"
            INSERT INTO controller_channel_contexts
              (id, channel, account_id, conversation_id, sender, trust_tier,
               last_project_id, last_thread_id, last_run_id, last_intent, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(channel, account_id, conversation_id, sender) DO UPDATE SET
              trust_tier = excluded.trust_tier,
              last_project_id = COALESCE(excluded.last_project_id, controller_channel_contexts.last_project_id),
              last_thread_id = COALESCE(excluded.last_thread_id, controller_channel_contexts.last_thread_id),
              last_run_id = COALESCE(excluded.last_run_id, controller_channel_contexts.last_run_id),
              last_intent = COALESCE(excluded.last_intent, controller_channel_contexts.last_intent),
              updated_at = excluded.updated_at
            "#,
            params![
                id,
                channel,
                account_id,
                conversation_id,
                sender,
                trust_tier,
                last_project_id,
                last_thread_id,
                last_run_id,
                last_intent,
                updated_at
            ],
        )?;
        self.get_controller_context(channel, Some(account_id), conversation_id, sender)?
            .with_context(|| "controller context was not found after upsert")
    }

    pub fn get_controller_context(
        &self,
        channel: &str,
        account_id: Option<&str>,
        conversation_id: &str,
        sender: &str,
    ) -> Result<Option<ControllerChannelContext>> {
        validate_key(channel)?;
        let account_id = account_id.unwrap_or("");
        validate_optional_controller_ref(account_id, "account id")?;
        validate_controller_ref(conversation_id, "conversation id")?;
        validate_query(sender)?;
        self.conn
            .query_row(
                r#"
                SELECT id, channel, account_id, conversation_id, sender, trust_tier,
                       last_project_id, last_thread_id, last_run_id, last_intent, updated_at
                FROM controller_channel_contexts
                WHERE channel = ?1 AND account_id = ?2 AND conversation_id = ?3 AND sender = ?4
                "#,
                params![channel, account_id, conversation_id, sender],
                controller_context_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upsert_controller_thread(
        &self,
        host: &str,
        host_thread_id: &str,
        project_id: Option<&str>,
        title: Option<&str>,
        cwd: Option<&str>,
        branch: Option<&str>,
        worktree: Option<&str>,
        status: &str,
        active: bool,
        archived: bool,
        current_goal: Option<&str>,
        latest_summary: Option<&str>,
        latest_summary_source: Option<&str>,
        last_activity_at: Option<&str>,
    ) -> Result<ControllerThread> {
        validate_key(host)?;
        validate_controller_ref(host_thread_id, "host thread id")?;
        validate_controller_status(status)?;
        let project_id = if let Some(project_id) = project_id {
            validate_id(project_id)?;
            self.get_project(project_id)?
                .with_context(|| format!("project not found: {project_id}"))?;
            Some(project_id)
        } else {
            None
        };
        let title = sanitize_optional_controller_text(title, WORK_SUMMARY_MAX)?;
        let cwd = sanitize_optional_controller_ref(cwd, "cwd")?;
        let branch = sanitize_optional_controller_ref(branch, "branch")?;
        let worktree = sanitize_optional_controller_ref(worktree, "worktree")?;
        let current_goal = sanitize_optional_controller_text(current_goal, WORK_SUMMARY_MAX)?;
        let latest_summary = sanitize_optional_controller_text(latest_summary, WORK_SUMMARY_MAX)?;
        let latest_summary_source =
            sanitize_optional_controller_ref(latest_summary_source, "latest summary source")?;
        if let Some(last_activity_at) = last_activity_at {
            DateTime::parse_from_rfc3339(last_activity_at).with_context(|| {
                format!("parsing last_activity_at timestamp {last_activity_at}")
            })?;
        }
        self.policy_guard(PolicyRequest {
            action: "controller.write".to_string(),
            package: Some("arcwell-controller".to_string()),
            provider: Some(host.to_string()),
            source: Some("thread_sync".to_string()),
            channel: None,
            subject: None,
            target: project_id.map(ToOwned::to_owned),
            projected_usd: None,
            metadata: json!({ "host_thread_id": host_thread_id, "status": status }),
            untrusted_excerpt: latest_summary.clone().or_else(|| title.clone()),
        })?;
        let id = self
            .conn
            .query_row(
                "SELECT id FROM controller_threads WHERE host = ?1 AND host_thread_id = ?2",
                params![host, host_thread_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let synced_at = now();
        self.conn.execute(
            r#"
            INSERT INTO controller_threads
              (id, host, host_thread_id, project_id, title, cwd, branch, worktree,
               status, active, archived, current_goal, latest_summary,
               latest_summary_source, last_activity_at, last_synced_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
            ON CONFLICT(host, host_thread_id) DO UPDATE SET
              project_id = excluded.project_id,
              title = excluded.title,
              cwd = excluded.cwd,
              branch = excluded.branch,
              worktree = excluded.worktree,
              status = excluded.status,
              active = excluded.active,
              archived = excluded.archived,
              current_goal = excluded.current_goal,
              latest_summary = excluded.latest_summary,
              latest_summary_source = excluded.latest_summary_source,
              last_activity_at = excluded.last_activity_at,
              last_synced_at = excluded.last_synced_at
            "#,
            params![
                id,
                host,
                host_thread_id,
                project_id,
                title,
                cwd,
                branch,
                worktree,
                status,
                bool_to_i64(active),
                bool_to_i64(archived),
                current_goal,
                latest_summary,
                latest_summary_source,
                last_activity_at,
                synced_at
            ],
        )?;
        self.get_controller_thread(&id)?
            .with_context(|| format!("controller thread not found after upsert: {id}"))
    }

    pub fn get_controller_thread(&self, id: &str) -> Result<Option<ControllerThread>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, host, host_thread_id, project_id, title, cwd, branch, worktree,
                       status, active, archived, current_goal, latest_summary,
                       latest_summary_source, last_activity_at, last_synced_at
                FROM controller_threads
                WHERE id = ?1
                "#,
                params![id],
                controller_thread_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_controller_threads(
        &self,
        project_id: Option<&str>,
        status: Option<&str>,
        limit: usize,
    ) -> Result<Vec<ControllerThread>> {
        if let Some(project_id) = project_id {
            validate_id(project_id)?;
        }
        if let Some(status) = status {
            validate_controller_status(status)?;
        }
        let limit = limit.clamp(1, 500) as i64;
        match (project_id, status) {
            (Some(project_id), Some(status)) => {
                let mut stmt = self.conn.prepare(
                    r#"
                    SELECT id, host, host_thread_id, project_id, title, cwd, branch, worktree,
                           status, active, archived, current_goal, latest_summary,
                           latest_summary_source, last_activity_at, last_synced_at
                    FROM controller_threads
                    WHERE project_id = ?1 AND status = ?2
                    ORDER BY COALESCE(last_activity_at, last_synced_at) DESC
                    LIMIT ?3
                    "#,
                )?;
                rows(stmt.query_map(
                    params![project_id, status, limit],
                    controller_thread_from_row,
                )?)
            }
            (Some(project_id), None) => {
                let mut stmt = self.conn.prepare(
                    r#"
                    SELECT id, host, host_thread_id, project_id, title, cwd, branch, worktree,
                           status, active, archived, current_goal, latest_summary,
                           latest_summary_source, last_activity_at, last_synced_at
                    FROM controller_threads
                    WHERE project_id = ?1
                    ORDER BY COALESCE(last_activity_at, last_synced_at) DESC
                    LIMIT ?2
                    "#,
                )?;
                rows(stmt.query_map(params![project_id, limit], controller_thread_from_row)?)
            }
            (None, Some(status)) => {
                let mut stmt = self.conn.prepare(
                    r#"
                    SELECT id, host, host_thread_id, project_id, title, cwd, branch, worktree,
                           status, active, archived, current_goal, latest_summary,
                           latest_summary_source, last_activity_at, last_synced_at
                    FROM controller_threads
                    WHERE status = ?1
                    ORDER BY COALESCE(last_activity_at, last_synced_at) DESC
                    LIMIT ?2
                    "#,
                )?;
                rows(stmt.query_map(params![status, limit], controller_thread_from_row)?)
            }
            (None, None) => {
                let mut stmt = self.conn.prepare(
                    r#"
                    SELECT id, host, host_thread_id, project_id, title, cwd, branch, worktree,
                           status, active, archived, current_goal, latest_summary,
                           latest_summary_source, last_activity_at, last_synced_at
                    FROM controller_threads
                    ORDER BY COALESCE(last_activity_at, last_synced_at) DESC
                    LIMIT ?1
                    "#,
                )?;
                rows(stmt.query_map(params![limit], controller_thread_from_row)?)
            }
        }
    }

    // allow: refactoring this N-arg signature is out of scope for the lint-cleanup pass.
    #[allow(clippy::too_many_arguments)]
    pub fn create_controller_run(
        &self,
        thread_id: Option<&str>,
        project_id: Option<&str>,
        origin_channel_message_id: Option<&str>,
        host: &str,
        host_run_id: Option<&str>,
        kind: &str,
        status: &str,
        requested_action: &str,
    ) -> Result<ControllerRun> {
        if let Some(thread_id) = thread_id {
            validate_id(thread_id)?;
            self.get_controller_thread(thread_id)?
                .with_context(|| format!("controller thread not found: {thread_id}"))?;
        }
        if let Some(project_id) = project_id {
            validate_id(project_id)?;
            self.get_project(project_id)?
                .with_context(|| format!("project not found: {project_id}"))?;
        }
        if let Some(message_id) = origin_channel_message_id {
            validate_id(message_id)?;
            self.get_channel_message(message_id)?
                .with_context(|| format!("channel message not found: {message_id}"))?;
        }
        validate_key(host)?;
        if let Some(host_run_id) = host_run_id {
            validate_controller_ref(host_run_id, "host run id")?;
        }
        validate_key(kind)?;
        validate_controller_run_status(status)?;
        let requested_action = sanitize_work_text(requested_action, WORK_SUMMARY_MAX)?;
        self.policy_guard(PolicyRequest {
            action: "controller.write".to_string(),
            package: Some("arcwell-controller".to_string()),
            provider: Some(host.to_string()),
            source: Some(kind.to_string()),
            channel: None,
            subject: None,
            target: project_id.map(ToOwned::to_owned),
            projected_usd: None,
            metadata: json!({ "thread_id": thread_id, "host_run_id": host_run_id, "status": status }),
            untrusted_excerpt: Some(requested_action.clone()),
        })?;
        let id = Uuid::new_v4().to_string();
        let timestamp = now();
        self.conn.execute(
            r#"
            INSERT INTO controller_runs
              (id, thread_id, project_id, origin_channel_message_id, host, host_run_id,
               kind, status, requested_action, started_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)
            "#,
            params![
                id,
                thread_id,
                project_id,
                origin_channel_message_id,
                host,
                host_run_id,
                kind,
                status,
                requested_action,
                timestamp
            ],
        )?;
        self.get_controller_run(&id)?
            .with_context(|| format!("controller run not found after insert: {id}"))
    }

    pub fn get_controller_run(&self, id: &str) -> Result<Option<ControllerRun>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, thread_id, project_id, origin_channel_message_id, host, host_run_id,
                       kind, status, requested_action, cancel_requested, cancel_reason,
                       started_at, updated_at, finished_at
                FROM controller_runs
                WHERE id = ?1
                "#,
                params![id],
                controller_run_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_controller_runs(
        &self,
        project_id: Option<&str>,
        status: Option<&str>,
        limit: usize,
    ) -> Result<Vec<ControllerRun>> {
        if let Some(project_id) = project_id {
            validate_id(project_id)?;
        }
        if let Some(status) = status {
            validate_controller_run_status(status)?;
        }
        let limit = limit.clamp(1, 500) as i64;
        match (project_id, status) {
            (Some(project_id), Some(status)) => {
                let mut stmt = self.conn.prepare(
                    r#"
                    SELECT id, thread_id, project_id, origin_channel_message_id, host, host_run_id,
                           kind, status, requested_action, cancel_requested, cancel_reason,
                           started_at, updated_at, finished_at
                    FROM controller_runs
                    WHERE project_id = ?1 AND status = ?2
                    ORDER BY updated_at DESC
                    LIMIT ?3
                    "#,
                )?;
                rows(stmt.query_map(params![project_id, status, limit], controller_run_from_row)?)
            }
            (Some(project_id), None) => {
                let mut stmt = self.conn.prepare(
                    r#"
                    SELECT id, thread_id, project_id, origin_channel_message_id, host, host_run_id,
                           kind, status, requested_action, cancel_requested, cancel_reason,
                           started_at, updated_at, finished_at
                    FROM controller_runs
                    WHERE project_id = ?1
                    ORDER BY updated_at DESC
                    LIMIT ?2
                    "#,
                )?;
                rows(stmt.query_map(params![project_id, limit], controller_run_from_row)?)
            }
            (None, Some(status)) => {
                let mut stmt = self.conn.prepare(
                    r#"
                    SELECT id, thread_id, project_id, origin_channel_message_id, host, host_run_id,
                           kind, status, requested_action, cancel_requested, cancel_reason,
                           started_at, updated_at, finished_at
                    FROM controller_runs
                    WHERE status = ?1
                    ORDER BY updated_at DESC
                    LIMIT ?2
                    "#,
                )?;
                rows(stmt.query_map(params![status, limit], controller_run_from_row)?)
            }
            (None, None) => {
                let mut stmt = self.conn.prepare(
                    r#"
                    SELECT id, thread_id, project_id, origin_channel_message_id, host, host_run_id,
                           kind, status, requested_action, cancel_requested, cancel_reason,
                           started_at, updated_at, finished_at
                    FROM controller_runs
                    ORDER BY updated_at DESC
                    LIMIT ?1
                    "#,
                )?;
                rows(stmt.query_map(params![limit], controller_run_from_row)?)
            }
        }
    }

    pub fn update_controller_run_status(
        &self,
        run_id: &str,
        status: &str,
        host_run_id: Option<&str>,
    ) -> Result<ControllerRun> {
        validate_id(run_id)?;
        validate_controller_run_status(status)?;
        if let Some(host_run_id) = host_run_id {
            validate_controller_ref(host_run_id, "host run id")?;
        }
        let run = self
            .get_controller_run(run_id)?
            .with_context(|| format!("controller run not found: {run_id}"))?;
        self.policy_guard(PolicyRequest {
            action: "controller.write".to_string(),
            package: Some("arcwell-controller".to_string()),
            provider: Some(run.host.clone()),
            source: Some("run_status_update".to_string()),
            channel: None,
            subject: None,
            target: run.project_id.clone(),
            projected_usd: None,
            metadata: json!({
                "run_id": run_id,
                "thread_id": run.thread_id,
                "status": status,
                "host_run_id": host_run_id,
            }),
            untrusted_excerpt: Some(run.requested_action.clone()),
        })?;
        let timestamp = now();
        let terminal = matches!(status, "finished" | "failed" | "cancelled");
        self.conn.execute(
            r#"
            UPDATE controller_runs
            SET status = ?2,
                host_run_id = COALESCE(?3, host_run_id),
                updated_at = ?4,
                finished_at = CASE
                  WHEN ?5 = 1 THEN COALESCE(finished_at, ?4)
                  ELSE finished_at
                END
            WHERE id = ?1
            "#,
            params![
                run_id,
                status,
                host_run_id,
                timestamp,
                bool_to_i64(terminal)
            ],
        )?;
        self.get_controller_run(run_id)?
            .with_context(|| format!("controller run not found after status update: {run_id}"))
    }

    pub fn request_controller_stop(&self, run_id: &str, reason: &str) -> Result<ControllerRun> {
        validate_id(run_id)?;
        let run = self
            .get_controller_run(run_id)?
            .with_context(|| format!("controller run not found: {run_id}"))?;
        let reason = sanitize_work_text(reason, WORK_SUMMARY_MAX)?;
        self.policy_guard(PolicyRequest {
            action: "controller.stop".to_string(),
            package: Some("arcwell-controller".to_string()),
            provider: Some(run.host.clone()),
            source: Some(run.kind.clone()),
            channel: None,
            subject: None,
            target: run.project_id.clone(),
            projected_usd: None,
            metadata: json!({ "run_id": run_id, "thread_id": run.thread_id }),
            untrusted_excerpt: Some(reason.clone()),
        })?;
        let timestamp = now();
        self.conn.execute(
            r#"
            UPDATE controller_runs
            SET cancel_requested = 1,
                cancel_reason = ?2,
                status = CASE WHEN status IN ('finished', 'failed', 'cancelled') THEN status ELSE 'stopping' END,
                updated_at = ?3
            WHERE id = ?1
            "#,
            params![run_id, reason, timestamp],
        )?;
        self.record_controller_event(
            Some(run_id),
            run.thread_id.as_deref(),
            run.project_id.as_deref(),
            "stop_requested",
            &reason,
            json!({ "host_adapter_stop_required": true }),
            "controller",
        )?;
        self.get_controller_run(run_id)?
            .with_context(|| format!("controller run not found after stop request: {run_id}"))
    }

    // allow: refactoring this N-arg signature is out of scope for the lint-cleanup pass.
    #[allow(clippy::too_many_arguments)]
    pub fn record_controller_event(
        &self,
        run_id: Option<&str>,
        thread_id: Option<&str>,
        project_id: Option<&str>,
        event_type: &str,
        summary: &str,
        data: Value,
        source: &str,
    ) -> Result<ControllerEvent> {
        if let Some(run_id) = run_id {
            validate_id(run_id)?;
            self.get_controller_run(run_id)?
                .with_context(|| format!("controller run not found: {run_id}"))?;
        }
        if let Some(thread_id) = thread_id {
            validate_id(thread_id)?;
            self.get_controller_thread(thread_id)?
                .with_context(|| format!("controller thread not found: {thread_id}"))?;
        }
        if let Some(project_id) = project_id {
            validate_id(project_id)?;
            self.get_project(project_id)?
                .with_context(|| format!("project not found: {project_id}"))?;
        }
        validate_key(event_type)?;
        validate_key(source)?;
        let summary = sanitize_work_text(summary, WORK_SUMMARY_MAX)?;
        let data = sanitize_work_json(data)?;
        let id = Uuid::new_v4().to_string();
        let timestamp = now();
        self.conn.execute(
            r#"
            INSERT INTO controller_events
              (id, run_id, thread_id, project_id, event_type, summary, data_json, source, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                id,
                run_id,
                thread_id,
                project_id,
                event_type,
                summary,
                serde_json::to_string(&data)?,
                source,
                timestamp
            ],
        )?;
        self.get_controller_event(&id)?
            .with_context(|| format!("controller event not found after insert: {id}"))
    }

    pub fn get_controller_event(&self, id: &str) -> Result<Option<ControllerEvent>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, run_id, thread_id, project_id, event_type, summary, data_json, source, created_at
                FROM controller_events
                WHERE id = ?1
                "#,
                params![id],
                controller_event_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_controller_events(
        &self,
        run_id: Option<&str>,
        project_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<ControllerEvent>> {
        if let Some(run_id) = run_id {
            validate_id(run_id)?;
        }
        if let Some(project_id) = project_id {
            validate_id(project_id)?;
        }
        let limit = limit.clamp(1, 500) as i64;
        match (run_id, project_id) {
            (Some(run_id), _) => {
                let mut stmt = self.conn.prepare(
                    r#"
                    SELECT id, run_id, thread_id, project_id, event_type, summary, data_json, source, created_at
                    FROM controller_events
                    WHERE run_id = ?1
                    ORDER BY created_at DESC
                    LIMIT ?2
                    "#,
                )?;
                rows(stmt.query_map(params![run_id, limit], controller_event_from_row)?)
            }
            (None, Some(project_id)) => {
                let mut stmt = self.conn.prepare(
                    r#"
                    SELECT id, run_id, thread_id, project_id, event_type, summary, data_json, source, created_at
                    FROM controller_events
                    WHERE project_id = ?1
                    ORDER BY created_at DESC
                    LIMIT ?2
                    "#,
                )?;
                rows(stmt.query_map(params![project_id, limit], controller_event_from_row)?)
            }
            (None, None) => {
                let mut stmt = self.conn.prepare(
                    r#"
                    SELECT id, run_id, thread_id, project_id, event_type, summary, data_json, source, created_at
                    FROM controller_events
                    ORDER BY created_at DESC
                    LIMIT ?1
                    "#,
                )?;
                rows(stmt.query_map(params![limit], controller_event_from_row)?)
            }
        }
    }

    // allow: refactoring this N-arg signature is out of scope for the lint-cleanup pass.
    #[allow(clippy::too_many_arguments)]
    pub fn create_controller_pending_action(
        &self,
        channel: &str,
        conversation_id: &str,
        sender: &str,
        action_type: &str,
        project_id: Option<&str>,
        thread_id: Option<&str>,
        run_id: Option<&str>,
        payload: Value,
        reason: &str,
        expires_in_seconds: i64,
    ) -> Result<ControllerPendingAction> {
        validate_key(channel)?;
        validate_controller_ref(conversation_id, "conversation id")?;
        validate_query(sender)?;
        validate_key(action_type)?;
        if let Some(project_id) = project_id {
            validate_id(project_id)?;
            self.get_project(project_id)?
                .with_context(|| format!("project not found: {project_id}"))?;
        }
        if let Some(thread_id) = thread_id {
            validate_id(thread_id)?;
        }
        if let Some(run_id) = run_id {
            validate_id(run_id)?;
        }
        let payload = sanitize_work_json(payload)?;
        let reason = sanitize_work_text(reason, WORK_SUMMARY_MAX)?;
        let id = Uuid::new_v4().to_string();
        let created_at = now();
        let expires_at = now_plus_seconds(expires_in_seconds.clamp(60, 30 * 24 * 60 * 60));
        self.conn.execute(
            r#"
            INSERT INTO controller_pending_actions
              (id, channel, conversation_id, sender, action_type, project_id, thread_id,
               run_id, payload_json, reason, status, expires_at, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'pending', ?11, ?12)
            "#,
            params![
                id,
                channel,
                conversation_id,
                sender,
                action_type,
                project_id,
                thread_id,
                run_id,
                serde_json::to_string(&payload)?,
                reason,
                expires_at,
                created_at
            ],
        )?;
        self.get_controller_pending_action(&id)?
            .with_context(|| format!("controller pending action not found after insert: {id}"))
    }

    pub fn get_controller_pending_action(
        &self,
        id: &str,
    ) -> Result<Option<ControllerPendingAction>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, channel, conversation_id, sender, action_type, project_id, thread_id,
                       run_id, payload_json, reason, status, expires_at, created_at, resolved_at
                FROM controller_pending_actions
                WHERE id = ?1
                "#,
                params![id],
                controller_pending_action_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_controller_pending_actions(
        &self,
        status: Option<&str>,
        limit: usize,
    ) -> Result<Vec<ControllerPendingAction>> {
        if let Some(status) = status {
            validate_controller_pending_status(status)?;
        }
        let limit = limit.clamp(1, 500) as i64;
        if let Some(status) = status {
            let mut stmt = self.conn.prepare(
                r#"
                SELECT id, channel, conversation_id, sender, action_type, project_id, thread_id,
                       run_id, payload_json, reason, status, expires_at, created_at, resolved_at
                FROM controller_pending_actions
                WHERE status = ?1
                ORDER BY created_at DESC
                LIMIT ?2
                "#,
            )?;
            rows(stmt.query_map(params![status, limit], controller_pending_action_from_row)?)
        } else {
            let mut stmt = self.conn.prepare(
                r#"
                SELECT id, channel, conversation_id, sender, action_type, project_id, thread_id,
                       run_id, payload_json, reason, status, expires_at, created_at, resolved_at
                FROM controller_pending_actions
                ORDER BY created_at DESC
                LIMIT ?1
                "#,
            )?;
            rows(stmt.query_map(params![limit], controller_pending_action_from_row)?)
        }
    }

    pub fn resolve_controller_pending_action(
        &self,
        id: &str,
        status: &str,
        thread_id: Option<&str>,
        run_id: Option<&str>,
    ) -> Result<ControllerPendingAction> {
        validate_id(id)?;
        validate_controller_pending_status(status)?;
        let pending = self
            .get_controller_pending_action(id)?
            .with_context(|| format!("controller pending action not found: {id}"))?;
        if let Some(thread_id) = thread_id {
            validate_id(thread_id)?;
            self.get_controller_thread(thread_id)?
                .with_context(|| format!("controller thread not found: {thread_id}"))?;
        }
        if let Some(run_id) = run_id {
            validate_id(run_id)?;
            self.get_controller_run(run_id)?
                .with_context(|| format!("controller run not found: {run_id}"))?;
        }
        self.policy_guard(PolicyRequest {
            action: "controller.write".to_string(),
            package: Some("arcwell-controller".to_string()),
            provider: Some(
                pending
                    .payload
                    .get("host")
                    .and_then(Value::as_str)
                    .unwrap_or("controller")
                    .to_string(),
            ),
            source: Some("pending_action_resolve".to_string()),
            channel: Some(pending.channel.clone()),
            subject: Some(pending.sender.clone()),
            target: pending.project_id.clone(),
            projected_usd: None,
            metadata: json!({
                "pending_action_id": id,
                "action_type": pending.action_type,
                "status": status,
                "thread_id": thread_id,
                "run_id": run_id,
            }),
            untrusted_excerpt: Some(pending.reason.clone()),
        })?;
        let resolved_at = controller_pending_status_is_terminal(status).then(now);
        self.conn.execute(
            r#"
            UPDATE controller_pending_actions
            SET status = ?2,
                thread_id = COALESCE(?3, thread_id),
                run_id = COALESCE(?4, run_id),
                resolved_at = CASE
                  WHEN ?5 IS NOT NULL THEN ?5
                  ELSE resolved_at
                END
            WHERE id = ?1
            "#,
            params![id, status, thread_id, run_id, resolved_at],
        )?;
        self.get_controller_pending_action(id)?
            .with_context(|| format!("controller pending action not found after resolve: {id}"))
    }

    pub fn controller_route_text(
        &self,
        channel: &str,
        account_id: Option<&str>,
        conversation_id: &str,
        sender: &str,
        text: &str,
    ) -> Result<ControllerRouteReport> {
        self.controller_route_text_with_origin(
            channel,
            account_id,
            conversation_id,
            sender,
            text,
            None,
        )
    }

    pub fn controller_route_text_with_origin(
        &self,
        channel: &str,
        account_id: Option<&str>,
        conversation_id: &str,
        sender: &str,
        text: &str,
        origin_channel_message_id: Option<&str>,
    ) -> Result<ControllerRouteReport> {
        validate_key(channel)?;
        validate_controller_ref(conversation_id, "conversation id")?;
        validate_query(sender)?;
        validate_notes(text)?;
        let origin_message = if let Some(message_id) = origin_channel_message_id {
            validate_id(message_id)?;
            Some(
                self.get_channel_message(message_id)?
                    .with_context(|| format!("channel message not found: {message_id}"))?,
            )
        } else {
            None
        };
        let origin_project_id = origin_message
            .as_ref()
            .and_then(|message| message.project_id.as_deref());
        let account_id = account_id.unwrap_or("");
        let subject = format!("{channel}:{sender}");
        let authorized_read = self.channel_subject_can_read_projects(channel, &subject)?;
        let authorized_write =
            self.channel_subject_can_write_projects(channel, std::slice::from_ref(&subject))?;
        let lower = text.to_ascii_lowercase();
        let intent = classify_controller_intent(&lower);
        let mut project = None;
        let mut thread = None;
        let mut run = None;
        let mut pending_action = None;
        let mut active_runs = Vec::new();
        let mut recent_events = Vec::new();
        let mut host_adapter_required = false;
        let host_adapter_available = false;
        let summary;

        match intent.as_str() {
            "project_status" => {
                if !authorized_read {
                    bail!("{channel} subject is not authorized to read project state: {subject}");
                }
                let context = self.get_controller_context(
                    channel,
                    Some(account_id),
                    conversation_id,
                    sender,
                )?;
                let query = controller_project_query(text);
                let context_project_id = context
                    .as_ref()
                    .and_then(|ctx| ctx.last_project_id.as_deref())
                    .or(origin_project_id);
                let resolved = if let Some(project_id) = origin_project_id {
                    self.project_resolution_for_id(project_id, 0.8)?
                } else {
                    self.resolve_project(&query, context_project_id)?
                };
                active_runs =
                    self.list_controller_runs(Some(&resolved.project.id), Some("running"), 20)?;
                recent_events =
                    self.list_controller_events(None, Some(&resolved.project.id), 20)?;
                let threads = self.list_controller_threads(Some(&resolved.project.id), None, 20)?;
                thread = threads.into_iter().next();
                summary =
                    controller_project_status_summary(&resolved, &active_runs, &recent_events);
                project = Some(resolved.project);
            }
            "active_work_status" => {
                if !authorized_read {
                    bail!("{channel} subject is not authorized to read project state: {subject}");
                }
                let context = self.get_controller_context(
                    channel,
                    Some(account_id),
                    conversation_id,
                    sender,
                )?;
                let context_project_id = context
                    .as_ref()
                    .and_then(|ctx| ctx.last_project_id.as_deref())
                    .or(origin_project_id);
                active_runs = if let Some(project_id) = context_project_id {
                    self.list_controller_runs(Some(project_id), Some("running"), 20)?
                } else {
                    self.list_controller_runs(None, Some("running"), 20)?
                };
                recent_events = self.list_controller_events(None, context_project_id, 20)?;
                summary = controller_active_work_summary(&active_runs, &recent_events);
                if let Some(first) = active_runs.first() {
                    run = Some(first.clone());
                }
                if let Some(project_id) = context_project_id {
                    project = Some(
                        self.get_project(project_id)?
                            .with_context(|| format!("project not found: {project_id}"))?,
                    );
                }
            }
            "create_work_thread" => {
                if !authorized_write {
                    bail!("{channel} subject is not authorized to control project work: {subject}");
                }
                let query = controller_project_query(text);
                let resolved = if let Some(project_id) = origin_project_id {
                    self.project_resolution_for_id(project_id, 0.8)?
                } else {
                    self.resolve_project(&query, None)?
                };
                host_adapter_required = true;
                let pending = self.create_controller_pending_action(
                    channel,
                    conversation_id,
                    sender,
                    "create_thread",
                    Some(&resolved.project.id),
                    None,
                    None,
                    json!({
                        "prompt": text,
                        "host": "codex",
                        "origin_channel_message_id": origin_channel_message_id,
                    }),
                    "Codex resident host adapter must create the host thread and record the result.",
                    24 * 60 * 60,
                )?;
                summary = format!(
                    "Queued create-thread request for project {}. Resident Codex host adapter must create the thread.",
                    resolved.project.name
                );
                project = Some(resolved.project);
                pending_action = Some(pending);
            }
            "stop_work" => {
                if !authorized_write {
                    bail!("{channel} subject is not authorized to control project work: {subject}");
                }
                let query = controller_project_query(text);
                let resolved = if let Some(project_id) = origin_project_id {
                    self.project_resolution_for_id(project_id, 0.8)?
                } else {
                    self.resolve_project(&query, None)?
                };
                let mut matches =
                    self.list_controller_runs(Some(&resolved.project.id), Some("running"), 20)?;
                if matches.is_empty() {
                    matches = self.list_controller_runs(
                        Some(&resolved.project.id),
                        Some("stopping"),
                        20,
                    )?;
                }
                if matches.len() == 1 {
                    let stopped = self.request_controller_stop(&matches[0].id, text)?;
                    summary = format!(
                        "Stop requested for {} run {}. Host adapter stop is still required.",
                        resolved.project.name, stopped.id
                    );
                    run = Some(stopped);
                    host_adapter_required = true;
                } else if matches.is_empty() {
                    summary = format!(
                        "No active controller run matched {}.",
                        resolved.project.name
                    );
                } else {
                    summary = format!(
                        "{} active runs matched {}; clarification required.",
                        matches.len(),
                        resolved.project.name
                    );
                    active_runs = matches;
                }
                project = Some(resolved.project);
            }
            "x_bookmark_report_email" => {
                if !authorized_write {
                    bail!(
                        "{channel} subject is not authorized to start report/email workflows: {subject}"
                    );
                }
                host_adapter_required = true;
                let pending = self.create_controller_pending_action(
                    channel,
                    conversation_id,
                    sender,
                    "x_bookmark_report_email",
                    None,
                    None,
                    None,
                    json!({
                        "request": text,
                        "host": "codex",
                        "origin_channel_message_id": origin_channel_message_id,
                    }),
                    "X bookmark report plus email delivery requires a Codex workflow thread.",
                    24 * 60 * 60,
                )?;
                summary = "Queued X bookmark report/email workflow request for the resident Codex host adapter.".to_string();
                pending_action = Some(pending);
            }
            "calendar_today" => {
                if !authorized_read {
                    bail!(
                        "{channel} subject is not authorized to read schedule-backed status: {subject}"
                    );
                }
                host_adapter_required = true;
                let pending = self.create_controller_pending_action(
                    channel,
                    conversation_id,
                    sender,
                    "calendar_today",
                    None,
                    None,
                    None,
                    json!({
                        "request": text,
                        "host": "google-calendar",
                        "origin_channel_message_id": origin_channel_message_id,
                    }),
                    "Calendar request requires the resident host Google Calendar connector.",
                    60 * 60,
                )?;
                summary = "Queued calendar request for the resident host adapter.".to_string();
                pending_action = Some(pending);
            }
            _ => {
                summary = "Controller could not classify the request. No action taken.".to_string();
            }
        }

        let last_project_id = project.as_ref().map(|project| project.id.as_str());
        let last_thread_id = thread.as_ref().map(|thread| thread.id.as_str());
        let last_run_id = run.as_ref().map(|run| run.id.as_str());
        let context = self.upsert_controller_context(
            channel,
            Some(account_id),
            conversation_id,
            sender,
            if authorized_read {
                "owner_or_authorized"
            } else {
                "untrusted"
            },
            last_project_id,
            last_thread_id,
            last_run_id,
            Some(&intent),
        )?;
        Ok(ControllerRouteReport {
            intent,
            confidence: 0.7,
            summary,
            project,
            thread,
            run,
            pending_action,
            context,
            active_runs,
            recent_events,
            host_adapter_required,
            host_adapter_available,
        })
    }
}
