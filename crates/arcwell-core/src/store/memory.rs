use super::*;

impl Store {
    pub fn set_profile(
        &self,
        key: &str,
        value: &str,
        sensitivity: &str,
        source: &str,
    ) -> Result<()> {
        validate_key(key)?;
        let now = now();
        self.conn.execute(
            r#"
            INSERT INTO profile_items (key, value, sensitivity, source, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(key) DO UPDATE SET
              value = excluded.value,
              sensitivity = excluded.sensitivity,
              source = excluded.source,
              updated_at = excluded.updated_at
            "#,
            params![key, value, sensitivity, source, now],
        )?;
        Ok(())
    }

    pub fn get_profile(&self, key: &str) -> Result<Option<ProfileItem>> {
        self.conn
            .query_row(
                "SELECT key, value, sensitivity, source, updated_at FROM profile_items WHERE key = ?1",
                params![key],
                profile_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_profile(&self) -> Result<Vec<ProfileItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT key, value, sensitivity, source, updated_at FROM profile_items ORDER BY key",
        )?;
        rows(stmt.query_map([], profile_from_row)?)
    }

    pub fn search_profile(&self, query: &str) -> Result<Vec<ProfileItem>> {
        let needle = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            r#"
            SELECT key, value, sensitivity, source, updated_at
            FROM profile_items
            WHERE key LIKE ?1 OR value LIKE ?1
            ORDER BY key
            "#,
        )?;
        rows(stmt.query_map(params![needle], profile_from_row)?)
    }

    pub fn delete_profile(&self, key: &str) -> Result<bool> {
        Ok(self
            .conn
            .execute("DELETE FROM profile_items WHERE key = ?1", params![key])?
            > 0)
    }

    pub fn add_memory(
        &self,
        text: &str,
        kind: &str,
        sensitivity: &str,
        source: &str,
        confidence: f64,
    ) -> Result<String> {
        self.add_memory_for_user(text, kind, sensitivity, source, confidence, None)
    }

    pub fn add_memory_for_user(
        &self,
        text: &str,
        kind: &str,
        sensitivity: &str,
        source: &str,
        confidence: f64,
        user_id: Option<&str>,
    ) -> Result<String> {
        let user_id = self.mem0_user_id(user_id)?;
        let id = Uuid::new_v4().to_string();
        let now = now();
        self.conn.execute(
            r#"
            INSERT INTO memories
              (id, text, kind, sensitivity, source, user_id, confidence, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)
            "#,
            params![
                id,
                text,
                kind,
                sensitivity,
                source,
                user_id,
                confidence,
                now
            ],
        )?;
        Ok(id)
    }

    pub fn search_memories(&self, query: &str) -> Result<Vec<MemoryItem>> {
        let needle = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, text, kind, sensitivity, source, user_id, confidence, created_at, updated_at
            FROM memories
            WHERE text LIKE ?1 OR kind LIKE ?1
            ORDER BY updated_at DESC
            "#,
        )?;
        rows(stmt.query_map(params![needle], memory_from_row)?)
    }

    pub fn list_memories(&self, limit: u32) -> Result<Vec<MemoryItem>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, text, kind, sensitivity, source, user_id, confidence, created_at, updated_at
            FROM memories
            ORDER BY updated_at DESC
            LIMIT ?1
            "#,
        )?;
        rows(stmt.query_map(params![limit], memory_from_row)?)
    }

    pub fn delete_memory(&self, id: &str) -> Result<bool> {
        Ok(self
            .conn
            .execute("DELETE FROM memories WHERE id = ?1", params![id])?
            > 0)
    }

    pub fn mem0_add_memory(
        &self,
        text: &str,
        user_id: Option<&str>,
        source: &str,
        sensitivity: &str,
        infer: bool,
    ) -> Result<Mem0AddReport> {
        validate_notes(text)?;
        validate_key(source)?;
        validate_key(sensitivity)?;
        let user_id = self.mem0_user_id(user_id)?;
        let (provider, memory) = self.mem0_memory()?;
        let mut metadata = arcwell_memory::JsonMap::new();
        metadata.insert("source".to_string(), json!(source));
        metadata.insert("sensitivity".to_string(), json!(sensitivity));
        metadata.insert("created_by".to_string(), json!("arcwell"));
        let results = memory
            .add(
                text,
                arcwell_memory::AddOptions {
                    user_id: Some(user_id.clone()),
                    metadata: Some(metadata),
                    infer: Some(infer),
                    ..Default::default()
                },
            )
            .map_err(|error| anyhow::anyhow!("mem0 add failed: {error}"))?;
        Ok(Mem0AddReport {
            provider,
            user_id,
            infer,
            results: serde_json::to_value(results)?,
        })
    }

    pub fn mem0_search_memories(
        &self,
        query: &str,
        user_id: Option<&str>,
        limit: usize,
    ) -> Result<Mem0SearchReport> {
        validate_query(query)?;
        let user_id = self.mem0_user_id(user_id)?;
        let (provider, memory) = self.mem0_memory()?;
        let mut filters = arcwell_memory::JsonMap::new();
        filters.insert("user_id".to_string(), json!(user_id.clone()));
        let results = memory
            .search(
                query,
                &filters,
                arcwell_memory::SearchOptions {
                    top_k: limit.clamp(1, 100),
                    ..Default::default()
                },
            )
            .map_err(|error| anyhow::anyhow!("mem0 search failed: {error}"))?;
        Ok(Mem0SearchReport {
            provider,
            user_id,
            query: query.to_string(),
            results,
        })
    }

    pub fn mem0_update_memory(
        &self,
        memory_id: &str,
        text: &str,
        user_id: Option<&str>,
    ) -> Result<Mem0MutationReport> {
        validate_id(memory_id)?;
        validate_notes(text)?;
        let user_id = self.mem0_user_id(user_id)?;
        let (provider, memory) = self.mem0_memory()?;
        let response = memory
            .update(memory_id, text, None)
            .map_err(|error| anyhow::anyhow!("mem0 update failed: {error}"))?;
        Ok(Mem0MutationReport {
            ok: true,
            provider,
            user_id,
            response,
        })
    }

    pub fn mem0_delete_memory(
        &self,
        memory_id: &str,
        user_id: Option<&str>,
    ) -> Result<Mem0MutationReport> {
        validate_id(memory_id)?;
        let user_id = self.mem0_user_id(user_id)?;
        let (provider, memory) = self.mem0_memory()?;
        let response = memory
            .delete(memory_id)
            .map_err(|error| anyhow::anyhow!("mem0 delete failed: {error}"))?;
        Ok(Mem0MutationReport {
            ok: true,
            provider,
            user_id,
            response,
        })
    }

    pub fn mem0_forget_user(&self, user_id: Option<&str>) -> Result<MemoryForgetReport> {
        let user_id = self.mem0_user_id(user_id)?;
        let (provider, memory) = self.mem0_memory()?;
        let before = self.mem0_get_all_memories_for_user(&memory, &user_id, 10_000)?;
        let provider_memory_ids: std::collections::HashSet<String> = mem0_hit_summaries(&before)
            .into_iter()
            .filter_map(|hit| hit.id)
            .collect();
        let response = memory
            .delete_all(Some(&user_id), None, None)
            .map_err(|error| anyhow::anyhow!("mem0 delete_all failed: {error}"))?;
        let (candidates_deleted, legacy_unscoped_candidates_deleted) =
            self.delete_memory_candidates_for_forget(&user_id, &provider_memory_ids)?;
        let (compatibility_memories_deleted, legacy_unscoped_compatibility_deleted) =
            self.delete_compatibility_memories_for_forget(&user_id)?;
        let lifecycle_events_deleted = self.conn.execute(
            "DELETE FROM memory_lifecycle_events WHERE user_id = ?1",
            params![user_id],
        )?;
        let decision_ledger_deleted = self.conn.execute(
            "DELETE FROM memory_decision_ledger WHERE user_id = ?1",
            params![user_id],
        )?;
        let tombstone = self.record_memory_forget_tombstone(
            &user_id,
            &provider,
            provider_memory_ids.len(),
            candidates_deleted + legacy_unscoped_candidates_deleted,
            compatibility_memories_deleted + legacy_unscoped_compatibility_deleted,
            lifecycle_events_deleted,
            decision_ledger_deleted,
        )?;
        self.record_memory_lifecycle_event(
            "forget",
            Some("manual_or_mcp"),
            Some(&user_id),
            None,
            None,
            &json!({
                "provider_memories_deleted": provider_memory_ids.len(),
                "candidates_deleted": candidates_deleted,
                "legacy_unscoped_candidates_deleted": legacy_unscoped_candidates_deleted,
                "compatibility_memories_deleted": compatibility_memories_deleted,
                "legacy_unscoped_compatibility_deleted": legacy_unscoped_compatibility_deleted,
                "lifecycle_events_deleted": lifecycle_events_deleted,
                "decision_ledger_deleted": decision_ledger_deleted,
                "tombstone_id": tombstone.id
            }),
            "completed",
        )?;
        Ok(MemoryForgetReport {
            ok: true,
            provider,
            user_id,
            provider_memories_deleted: provider_memory_ids.len(),
            provider_response: response,
            candidates_deleted,
            legacy_unscoped_candidates_deleted,
            compatibility_memories_deleted,
            legacy_unscoped_compatibility_deleted,
            lifecycle_events_deleted,
            decision_ledger_deleted,
            tombstone_id: tombstone.id,
        })
    }

    pub(crate) fn mem0_get_all_memories_for_user(
        &self,
        memory: &arcwell_memory::blocking::Memory,
        user_id: &str,
        limit: usize,
    ) -> Result<Value> {
        let mut filters = arcwell_memory::JsonMap::new();
        filters.insert("user_id".to_string(), json!(user_id));
        memory
            .get_all(&filters, limit.clamp(1, 10_000))
            .map_err(|error| anyhow::anyhow!("mem0 get_all failed: {error}"))
    }

    pub(crate) fn delete_memory_candidates_for_forget(
        &self,
        user_id: &str,
        provider_memory_ids: &std::collections::HashSet<String>,
    ) -> Result<(usize, usize)> {
        let default_user = self.mem0_user_id(None)?;
        let delete_legacy_unscoped = user_id == default_user;
        let candidates = self.list_memory_candidates()?;
        let mut deleted = 0;
        let mut legacy_unscoped_deleted = 0;
        for candidate in candidates
            .into_iter()
            .filter(|candidate| candidate.target == "memory")
        {
            let scoped_match = candidate.user_id.as_deref() == Some(user_id)
                || candidate
                    .memory_id
                    .as_ref()
                    .is_some_and(|id| provider_memory_ids.contains(id));
            let legacy_match = delete_legacy_unscoped && candidate.user_id.is_none();
            if scoped_match || legacy_match {
                deleted += self.conn.execute(
                    "DELETE FROM candidates WHERE id = ?1 AND target = 'memory'",
                    params![candidate.id],
                )?;
                if legacy_match && !scoped_match {
                    legacy_unscoped_deleted += 1;
                }
            }
        }
        Ok((deleted, legacy_unscoped_deleted))
    }

    pub(crate) fn list_memory_candidates(&self) -> Result<Vec<Candidate>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, target, kind, content, sensitivity, source_ref, status, created_at,
                   operation, memory_id, user_id, metadata_json, applied_result_json,
                   applied_at, rejected_reason
            FROM candidates
            WHERE target = 'memory'
            ORDER BY created_at DESC
            "#,
        )?;
        rows(stmt.query_map([], candidate_from_row)?)
    }

    pub(crate) fn delete_compatibility_memories_for_forget(
        &self,
        user_id: &str,
    ) -> Result<(usize, usize)> {
        let default_user = self.mem0_user_id(None)?;
        let scoped = self
            .conn
            .execute("DELETE FROM memories WHERE user_id = ?1", params![user_id])?;
        let legacy_unscoped = if user_id == default_user {
            self.conn
                .execute("DELETE FROM memories WHERE user_id IS NULL", [])?
        } else {
            0
        };
        Ok((scoped + legacy_unscoped, legacy_unscoped))
    }

    pub fn mem0_history(&self, memory_id: &str) -> Result<Value> {
        validate_id(memory_id)?;
        let (_provider, memory) = self.mem0_memory()?;
        let history = memory
            .history(memory_id)
            .map_err(|error| anyhow::anyhow!("mem0 history failed: {error}"))?;
        Ok(serde_json::to_value(history)?)
    }

    pub fn memory_recall_context(
        &self,
        query: &str,
        user_id: Option<&str>,
        limit: usize,
    ) -> Result<MemoryRecallReport> {
        validate_query(query)?;
        let user_id = self.mem0_user_id(user_id)?;
        let profile_matches = self.search_profile_terms(query)?;
        let memory = self.mem0_search_memories(query, Some(&user_id), limit)?;
        let context = build_memory_context(&profile_matches, &memory.results);
        let report = MemoryRecallReport {
            query: query.to_string(),
            user_id,
            profile_matches,
            memory,
            context,
        };
        self.record_memory_lifecycle_event(
            "recall",
            Some("manual_or_hook"),
            Some(&report.user_id),
            None,
            Some(query),
            &json!({
                "profile_matches": report.profile_matches.len(),
                "memory_hits": mem0_results_array(&report.memory.results).len()
            }),
            "completed",
        )?;
        Ok(report)
    }

    pub(crate) fn search_profile_terms(&self, query: &str) -> Result<Vec<ProfileItem>> {
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        let mut terms = vec![query.to_string()];
        terms.extend(
            query
                .split(|c: char| !c.is_alphanumeric() && c != '.' && c != '_' && c != '-')
                .map(str::trim)
                .filter(|term| term.len() >= 4)
                .map(ToOwned::to_owned),
        );
        for term in terms {
            for item in self.search_profile(&term)? {
                if seen.insert(item.key.clone()) {
                    out.push(item);
                }
            }
        }
        Ok(out)
    }

    pub fn capture_memory_from_text(
        &self,
        text: &str,
        source_ref: &str,
        user_id: Option<&str>,
        auto_apply: bool,
        infer: bool,
    ) -> Result<MemoryCaptureReport> {
        validate_notes(text)?;
        validate_notes(source_ref)?;
        self.policy_guard(PolicyRequest {
            action: "memory.capture".to_string(),
            package: Some("arcwell-memory".to_string()),
            provider: None,
            source: Some("capture_memory".to_string()),
            channel: None,
            subject: user_id.map(ToOwned::to_owned),
            target: Some(excerpt(source_ref, 240)),
            projected_usd: None,
            metadata: json!({
                "auto_apply": auto_apply,
                "infer": infer,
                "text_len": text.len()
            }),
            untrusted_excerpt: Some(text.to_string()),
        })?;
        let user_id = user_id.map(ToOwned::to_owned);
        let mut report = self.extract_memory_candidates_from_text_for_user(
            text,
            source_ref,
            user_id.as_deref(),
        )?;
        let created_ids: std::collections::HashSet<String> = report
            .candidates
            .iter()
            .map(|candidate| candidate.id.clone())
            .collect();
        let mut applied = Vec::new();
        let mut sensitive_pending = 0;
        if auto_apply {
            for candidate in report.candidates.clone() {
                if memory_candidate_requires_review(&candidate) {
                    sensitive_pending += 1;
                    continue;
                }
                let apply_report = self.apply_candidate(&candidate.id)?;
                applied.push(apply_report);
            }
        } else {
            sensitive_pending = report
                .candidates
                .iter()
                .filter(|candidate| candidate.sensitivity == "sensitive")
                .count();
        }

        report.candidates = self
            .list_candidates("pending")?
            .into_iter()
            .filter(|candidate| created_ids.contains(&candidate.id))
            .collect();
        let capture = MemoryCaptureReport {
            mode: if auto_apply {
                "auto_apply_non_sensitive".to_string()
            } else {
                "review".to_string()
            },
            user_id,
            candidates_created: report.candidates_created,
            duplicates_suppressed: report.duplicates_suppressed,
            sensitive_pending,
            auto_applied: applied.len(),
            candidates: report.candidates,
            applied,
        };
        self.record_memory_lifecycle_event(
            "capture",
            Some("manual_or_hook"),
            capture.user_id.as_deref(),
            Some(source_ref),
            Some(text),
            &json!({
                "mode": capture.mode,
                "candidates_created": capture.candidates_created,
                "auto_applied": capture.auto_applied,
                "sensitive_pending": capture.sensitive_pending,
                "infer_requested": infer
            }),
            "completed",
        )?;
        Ok(capture)
    }

    pub fn list_memory_lifecycle_events(&self, limit: u32) -> Result<Vec<MemoryLifecycleEvent>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, event_type, hook, user_id, source_ref, input, result_json, status, created_at
            FROM memory_lifecycle_events
            ORDER BY created_at DESC
            LIMIT ?1
            "#,
        )?;
        rows(stmt.query_map(params![limit], memory_lifecycle_event_from_row)?)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_memory_lifecycle_event(
        &self,
        event_type: &str,
        hook: Option<&str>,
        user_id: Option<&str>,
        source_ref: Option<&str>,
        input: Option<&str>,
        result: &Value,
        status: &str,
    ) -> Result<String> {
        validate_key(event_type)?;
        if let Some(hook) = hook {
            validate_key(hook)?;
        }
        if let Some(user_id) = user_id {
            validate_key(user_id)?;
        }
        if let Some(source_ref) = source_ref {
            validate_notes(source_ref)?;
        }
        if let Some(input) = input {
            validate_notes(input)?;
        }
        validate_key(status)?;
        let id = Uuid::new_v4().to_string();
        self.conn.execute(
            r#"
            INSERT INTO memory_lifecycle_events
              (id, event_type, hook, user_id, source_ref, input, result_json, status, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                id,
                event_type,
                hook,
                user_id,
                source_ref,
                input,
                serde_json::to_string(result)?,
                status,
                now()
            ],
        )?;
        Ok(id)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_memory_decision(
        &self,
        user_id: Option<&str>,
        source_ref: &str,
        observation: &str,
        operation: &str,
        memory_id: Option<&str>,
        candidate_id: Option<&str>,
        confidence: f64,
        reason: &str,
        metadata: &Value,
    ) -> Result<String> {
        if let Some(user_id) = user_id {
            validate_key(user_id)?;
        }
        validate_notes(source_ref)?;
        validate_notes(observation)?;
        validate_candidate_operation(operation)?;
        if let Some(memory_id) = memory_id {
            validate_id(memory_id)?;
        }
        if let Some(candidate_id) = candidate_id {
            validate_id(candidate_id)?;
        }
        validate_confidence(confidence)?;
        validate_notes(reason)?;
        let id = Uuid::new_v4().to_string();
        self.conn.execute(
            r#"
            INSERT INTO memory_decision_ledger
              (id, user_id, source_ref, observation, operation, memory_id, candidate_id,
               confidence, reason, metadata_json, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            params![
                id,
                user_id,
                source_ref,
                observation,
                operation,
                memory_id,
                candidate_id,
                confidence,
                reason,
                serde_json::to_string(metadata)?,
                now()
            ],
        )?;
        Ok(id)
    }

    pub fn list_memory_decisions(&self, limit: u32) -> Result<Vec<MemoryDecisionLedgerEntry>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, user_id, source_ref, observation, operation, memory_id, candidate_id,
                   confidence, reason, metadata_json, created_at
            FROM memory_decision_ledger
            ORDER BY created_at DESC
            LIMIT ?1
            "#,
        )?;
        rows(stmt.query_map(params![limit], memory_decision_from_row)?)
    }

    // allow: refactoring this N-arg signature is out of scope for the lint-cleanup pass.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_memory_forget_tombstone(
        &self,
        user_id: &str,
        provider: &str,
        provider_memories_deleted: usize,
        candidates_deleted: usize,
        compatibility_memories_deleted: usize,
        lifecycle_events_deleted: usize,
        decision_ledger_deleted: usize,
    ) -> Result<MemoryForgetTombstone> {
        validate_key(user_id)?;
        validate_key(provider)?;
        let id = Uuid::new_v4().to_string();
        let created_at = now();
        let user_id_hash = sha256(user_id.as_bytes());
        let policy = "active_store_purged;historical_backups_retained_until_backup_retention;backups_not_rewritten_by_forget;tombstone_records_active_purge_only".to_string();
        self.conn.execute(
            r#"
            INSERT INTO memory_forget_tombstones
              (id, user_id_hash, provider, provider_memories_deleted, candidates_deleted,
               compatibility_memories_deleted, lifecycle_events_deleted, decision_ledger_deleted,
               policy, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                id,
                user_id_hash,
                provider,
                provider_memories_deleted as i64,
                candidates_deleted as i64,
                compatibility_memories_deleted as i64,
                lifecycle_events_deleted as i64,
                decision_ledger_deleted as i64,
                policy,
                created_at
            ],
        )?;
        self.list_memory_forget_tombstones(1)?
            .into_iter()
            .find(|tombstone| tombstone.id == id)
            .with_context(|| format!("inserted memory tombstone not found: {id}"))
    }

    pub fn list_memory_forget_tombstones(&self, limit: u32) -> Result<Vec<MemoryForgetTombstone>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, user_id_hash, provider, provider_memories_deleted, candidates_deleted,
                   compatibility_memories_deleted, lifecycle_events_deleted, decision_ledger_deleted,
                   policy, created_at
            FROM memory_forget_tombstones
            ORDER BY created_at DESC
            LIMIT ?1
            "#,
        )?;
        rows(stmt.query_map(params![limit], memory_forget_tombstone_from_row)?)
    }

    pub(crate) fn mem0_user_id(&self, explicit: Option<&str>) -> Result<String> {
        let user_id = explicit
            .map(ToOwned::to_owned)
            .or_else(|| std::env::var("ARCWELL_MEMORY_USER_ID").ok())
            .or_else(|| std::env::var("ARCWELL_MEM0_USER_ID").ok())
            .unwrap_or_else(|| "default".to_string());
        validate_key(&user_id)?;
        Ok(user_id)
    }

    pub(crate) fn mem0_memory(&self) -> Result<(String, arcwell_memory::blocking::Memory)> {
        self.paths.ensure()?;
        let mem0_dir = self.paths.home.join("mem0");
        fs::create_dir_all(&mem0_dir)
            .with_context(|| format!("creating {}", mem0_dir.display()))?;
        let config = if let Ok(config) = std::env::var("ARCWELL_MEMORY_CONFIG") {
            config
        } else if let Ok(config) = std::env::var("ARCWELL_MEM0_CONFIG") {
            config
        } else {
            self.mem0_default_config(&mem0_dir)?
        };
        let parsed: Value =
            serde_json::from_str(&config).context("parsing Arcwell memory config")?;
        let provider = parsed
            .pointer("/llm/provider")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let memory = arcwell_memory::blocking::Memory::from_json(&config)
            .map_err(|error| anyhow::anyhow!("building Arcwell memory failed: {error}"))?;
        Ok((provider, memory))
    }

    pub(crate) fn mem0_default_config(&self, mem0_dir: &Path) -> Result<String> {
        let forced_provider = std::env::var("ARCWELL_MEMORY_PROVIDER")
            .ok()
            .or_else(|| std::env::var("ARCWELL_MEM0_PROVIDER").ok());
        let openai_env_key = std::env::var("OPENAI_API_KEY").ok();
        let local_openai_secret_present = openai_env_key.is_none()
            && forced_provider.as_deref() != Some("mock")
            && self
                .list_secret_values()?
                .into_iter()
                .any(|secret| secret.name == "OPENAI_API_KEY");
        let openai_available = openai_env_key.is_some() || local_openai_secret_present;
        let wants_openai = forced_provider.as_deref() == Some("openai")
            || (forced_provider.is_none() && !cfg!(test) && openai_available);
        if wants_openai {
            self.require_cost_budget(
                "arcwell-memory",
                "memory_provider",
                "openai",
                "memory_llm_embed",
                Some("memory_provider"),
                estimated_memory_provider_cost(),
                "Arcwell Memory OpenAI provider",
            )?;
        }
        let openai_key = if wants_openai {
            openai_env_key.or_else(|| {
                self.get_usable_secret_value("OPENAI_API_KEY")
                    .ok()
                    .flatten()
            })
        } else {
            None
        };
        let provider =
            forced_provider
                .as_deref()
                .unwrap_or(if wants_openai { "openai" } else { "mock" });
        let vector_path = mem0_dir.join("vectors");
        let history_path = mem0_dir.join("history.sqlite3");
        fs::create_dir_all(&vector_path)
            .with_context(|| format!("creating {}", vector_path.display()))?;
        let config = if provider == "openai" {
            let api_key = openai_key
                .context("OPENAI_API_KEY is required for ARCWELL_MEMORY_PROVIDER=openai")?;
            json!({
                "embedder": {
                    "provider": "openai",
                    "config": {
                        "api_key": api_key,
                        "model": std::env::var("ARCWELL_MEMORY_EMBEDDING_MODEL")
                            .or_else(|_| std::env::var("ARCWELL_MEM0_EMBEDDING_MODEL"))
                            .unwrap_or_else(|_| "text-embedding-3-small".to_string())
                    }
                },
                "llm": {
                    "provider": "openai",
                    "config": {
                        "api_key": api_key,
                        "model": std::env::var("ARCWELL_MEMORY_LLM_MODEL")
                            .or_else(|_| std::env::var("ARCWELL_MEM0_LLM_MODEL"))
                            .unwrap_or_else(|_| "gpt-5-mini".to_string()),
                        "reasoning_effort": std::env::var("ARCWELL_MEMORY_REASONING_EFFORT")
                            .or_else(|_| std::env::var("ARCWELL_MEM0_REASONING_EFFORT"))
                            .unwrap_or_else(|_| "low".to_string())
                    }
                },
                "vector_store": {
                    "provider": "embedded",
                    "config": { "path": vector_path, "collection_name": "arcwell_memory" }
                },
                "history_db_path": history_path
            })
        } else {
            json!({
                "embedder": { "provider": "mock" },
                "llm": { "provider": "mock" },
                "vector_store": {
                    "provider": "embedded",
                    "config": { "path": vector_path, "collection_name": "arcwell_memory" }
                },
                "history_db_path": history_path
            })
        };
        Ok(serde_json::to_string(&config)?)
    }

    pub fn add_candidate(
        &self,
        target: &str,
        kind: &str,
        content: &str,
        sensitivity: &str,
        source_ref: &str,
    ) -> Result<String> {
        self.add_candidate_with_operation(
            target,
            kind,
            content,
            sensitivity,
            source_ref,
            "ADD",
            None,
            None,
            json!({}),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_candidate_with_operation(
        &self,
        target: &str,
        kind: &str,
        content: &str,
        sensitivity: &str,
        source_ref: &str,
        operation: &str,
        memory_id: Option<&str>,
        user_id: Option<&str>,
        metadata: Value,
    ) -> Result<String> {
        validate_candidate_operation(operation)?;
        validate_notes(content)?;
        validate_key(target)?;
        validate_key(kind)?;
        validate_key(sensitivity)?;
        validate_notes(source_ref)?;
        if let Some(memory_id) = memory_id {
            validate_id(memory_id)?;
        }
        if let Some(user_id) = user_id {
            validate_key(user_id)?;
        }
        let id = Uuid::new_v4().to_string();
        let now = now();
        self.conn.execute(
            r#"
            INSERT INTO candidates
              (id, target, kind, content, sensitivity, source_ref, status, created_at,
               operation, memory_id, user_id, metadata_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending', ?7, ?8, ?9, ?10, ?11)
            "#,
            params![
                id,
                target,
                kind,
                content,
                sensitivity,
                source_ref,
                now,
                operation,
                memory_id,
                user_id,
                serde_json::to_string(&metadata)?
            ],
        )?;
        Ok(id)
    }

    pub fn list_candidates(&self, status: &str) -> Result<Vec<Candidate>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, target, kind, content, sensitivity, source_ref, status, created_at,
                   operation, memory_id, user_id, metadata_json, applied_result_json,
                   applied_at, rejected_reason
            FROM candidates
            WHERE status = ?1
            ORDER BY created_at DESC
            "#,
        )?;
        rows(stmt.query_map(params![status], candidate_from_row)?)
    }

    pub fn start_import_run(
        &self,
        source_kind: &str,
        source_path: &str,
        mode: &str,
        metadata: Value,
    ) -> Result<String> {
        validate_key(source_kind)?;
        validate_notes(source_path)?;
        validate_key(mode)?;
        let metadata = sanitize_work_json(metadata)?;
        let id = Uuid::new_v4().to_string();
        let now = now();
        self.conn.execute(
            r#"
            INSERT INTO import_runs
              (id, source_kind, source_path, mode, status, metadata_json, started_at)
            VALUES (?1, ?2, ?3, ?4, 'running', ?5, ?6)
            "#,
            params![
                id,
                source_kind,
                redact_secret_like_text(source_path),
                mode,
                serde_json::to_string(&metadata)?,
                now
            ],
        )?;
        Ok(id)
    }

    pub fn finish_import_run(&self, id: &str, finish: ImportRunFinish) -> Result<ImportRunRecord> {
        validate_id(id)?;
        validate_key(&finish.status)?;
        if let Some(error) = &finish.error {
            validate_notes(error)?;
        }
        let metadata = sanitize_work_json(finish.metadata)?;
        let now = now();
        self.conn.execute(
            r#"
            UPDATE import_runs
            SET status = ?2,
                conversations_seen = ?3,
                conversations_sampled = ?4,
                candidates_seen = ?5,
                candidates_sampled = ?6,
                candidates_written = ?7,
                duplicates_suppressed = ?8,
                error = ?9,
                metadata_json = ?10,
                finished_at = ?11
            WHERE id = ?1
            "#,
            params![
                id,
                finish.status,
                finish.conversations_seen as i64,
                finish.conversations_sampled as i64,
                finish.candidates_seen as i64,
                finish.candidates_sampled as i64,
                finish.candidates_written as i64,
                finish.duplicates_suppressed as i64,
                finish.error.as_deref().map(redact_secret_like_text),
                serde_json::to_string(&metadata)?,
                now
            ],
        )?;
        self.get_import_run(id)
    }

    pub fn get_import_run(&self, id: &str) -> Result<ImportRunRecord> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, source_kind, source_path, mode, status,
                       conversations_seen, conversations_sampled,
                       candidates_seen, candidates_sampled,
                       candidates_written, duplicates_suppressed,
                       error, metadata_json, started_at, finished_at
                FROM import_runs
                WHERE id = ?1
                "#,
                params![id],
                import_run_from_row,
            )
            .optional()?
            .with_context(|| format!("import run not found: {id}"))
    }

    pub fn list_import_runs(&self, limit: usize) -> Result<Vec<ImportRunRecord>> {
        let limit = limit.clamp(1, 500) as i64;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, source_kind, source_path, mode, status,
                   conversations_seen, conversations_sampled,
                   candidates_seen, candidates_sampled,
                   candidates_written, duplicates_suppressed,
                   error, metadata_json, started_at, finished_at
            FROM import_runs
            ORDER BY started_at DESC
            LIMIT ?1
            "#,
        )?;
        rows(stmt.query_map(params![limit], import_run_from_row)?)
    }

    pub fn apply_candidate(&self, id: &str) -> Result<MemoryCandidateApplyReport> {
        let candidate = self
            .conn
            .query_row(
                r#"
                SELECT id, target, kind, content, sensitivity, source_ref, status, created_at,
                       operation, memory_id, user_id, metadata_json, applied_result_json,
                       applied_at, rejected_reason
                FROM candidates
                WHERE id = ?1
                "#,
                params![id],
                candidate_from_row,
            )
            .optional()?
            .with_context(|| format!("candidate not found: {id}"))?;

        if candidate.status != "pending" {
            bail!("candidate {id} is not pending");
        }

        self.policy_guard(PolicyRequest {
            action: match candidate.target.as_str() {
                "memory" => "memory.apply".to_string(),
                "profile" => "profile.write".to_string(),
                other => format!("{other}.apply"),
            },
            package: None,
            provider: None,
            source: Some(candidate.source_ref.clone()),
            channel: None,
            subject: candidate.user_id.clone(),
            target: Some(candidate.target.clone()),
            projected_usd: None,
            metadata: json!({
                "candidate_id": candidate.id.clone(),
                "operation": candidate.operation.clone(),
                "sensitivity": candidate.sensitivity.clone()
            }),
            untrusted_excerpt: Some(candidate.content.clone()),
        })?;

        let result = match candidate.target.as_str() {
            "profile" => {
                let key = candidate.kind.trim();
                self.set_profile(
                    key,
                    &candidate.content,
                    &candidate.sensitivity,
                    &candidate.source_ref,
                )?;
                MemoryCandidateApplyReport {
                    ok: true,
                    candidate_id: candidate.id.clone(),
                    operation: "ADD".to_string(),
                    user_id: None,
                    memory_id: None,
                    result: json!({ "profile_key": key }),
                }
            }
            "memory" => self.apply_memory_candidate(&candidate)?,
            other => bail!("unsupported candidate target: {other}"),
        };

        self.conn.execute(
            r#"
            UPDATE candidates
            SET status = 'applied', applied_result_json = ?2, applied_at = ?3
            WHERE id = ?1
            "#,
            params![id, serde_json::to_string(&result.result)?, now()],
        )?;
        Ok(result)
    }

    pub(crate) fn apply_memory_candidate(
        &self,
        candidate: &Candidate,
    ) -> Result<MemoryCandidateApplyReport> {
        let user_id = candidate.user_id.as_deref();
        let operation = candidate.operation.to_ascii_uppercase();
        let result = match operation.as_str() {
            "ADD" => json!(self.mem0_add_memory(
                &candidate.content,
                user_id,
                &candidate.source_ref,
                &candidate.sensitivity,
                false,
            )?),
            "UPDATE" => {
                let memory_id = candidate
                    .memory_id
                    .as_deref()
                    .context("UPDATE memory candidate requires memory_id")?;
                json!(self.mem0_update_memory(memory_id, &candidate.content, user_id)?)
            }
            "DELETE" => {
                let memory_id = candidate
                    .memory_id
                    .as_deref()
                    .context("DELETE memory candidate requires memory_id")?;
                json!(self.mem0_delete_memory(memory_id, user_id)?)
            }
            "NONE" => json!({ "ok": true, "noop": true }),
            other => bail!("unsupported memory candidate operation: {other}"),
        };
        Ok(MemoryCandidateApplyReport {
            ok: true,
            candidate_id: candidate.id.clone(),
            operation,
            user_id: candidate.user_id.clone(),
            memory_id: candidate.memory_id.clone(),
            result,
        })
    }

    pub fn reject_candidate(&self, id: &str, reason: Option<&str>) -> Result<bool> {
        if let Some(reason) = reason {
            validate_notes(reason)?;
        }
        Ok(self.conn.execute(
            "UPDATE candidates SET status = 'rejected', rejected_reason = ?2 WHERE id = ?1 AND status = 'pending'",
            params![id, reason],
        )? > 0)
    }
}
