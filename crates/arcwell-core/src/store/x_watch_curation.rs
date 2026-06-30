use super::*;

const X_WATCH_CURATION_CLASSIFIER_VERSION: &str = "x-watch-curation-v1-local-conservative";

#[derive(Debug, Clone)]
struct XWatchCandidate {
    source: WatchSource,
    profile_present: bool,
    display_name: String,
    description: String,
    local_tweets: usize,
    tech_tweet_hits: usize,
    bookmarked_items: usize,
    manual_rule: Option<XWatchManualRule>,
}

#[derive(Debug, Clone)]
struct XWatchManualRule {
    decision: String,
    category: String,
    reason: String,
}

impl Store {
    pub fn import_x_watch_manual_rules(
        &self,
        rules: Vec<XWatchManualRuleInput>,
        reviewed_by: &str,
        dry_run: bool,
    ) -> Result<XWatchManualRuleImportReport> {
        validate_notes(reviewed_by)?;
        if rules.len() > 5_000 {
            bail!("too many X watch manual rules");
        }
        let mut items = Vec::new();
        let mut valid_rules = Vec::new();
        let mut rejected = 0_usize;
        let mut seen_handles = BTreeSet::new();
        for rule in rules {
            match normalize_x_watch_manual_rule(rule) {
                Ok(rule) => {
                    if !seen_handles.insert(rule.handle.clone()) {
                        rejected += 1;
                        items.push(XWatchManualRuleImportItem {
                            handle: rule.handle,
                            decision: rule.decision,
                            category: rule.category,
                            status: "rejected".to_string(),
                            error: Some("duplicate handle in manual-rule import".to_string()),
                        });
                        continue;
                    }
                    if self.read_x_watch_source_by_handle(&rule.handle)?.is_none() {
                        rejected += 1;
                        items.push(XWatchManualRuleImportItem {
                            handle: rule.handle,
                            decision: rule.decision,
                            category: rule.category,
                            status: "rejected".to_string(),
                            error: Some("watch source does not exist for handle".to_string()),
                        });
                        continue;
                    }
                    items.push(XWatchManualRuleImportItem {
                        handle: rule.handle.clone(),
                        decision: rule.decision.clone(),
                        category: rule.category.clone(),
                        status: if dry_run {
                            "validated".to_string()
                        } else {
                            "pending_write".to_string()
                        },
                        error: None,
                    });
                    valid_rules.push(rule);
                }
                Err(error) => {
                    rejected += 1;
                    items.push(XWatchManualRuleImportItem {
                        handle: String::new(),
                        decision: String::new(),
                        category: String::new(),
                        status: "rejected".to_string(),
                        error: Some(error.to_string()),
                    });
                }
            }
        }

        let mut imported = 0_usize;
        let mut updated = 0_usize;
        let blocked_by_rejections = !dry_run && rejected > 0;
        if blocked_by_rejections {
            for item in &mut items {
                if item.status == "pending_write" {
                    item.status = "blocked_by_rejection".to_string();
                }
            }
        }

        if !dry_run && !blocked_by_rejections && !valid_rules.is_empty() {
            let written_at = now();
            let write_result = (|| -> Result<()> {
                self.conn.execute("BEGIN IMMEDIATE", [])?;
                for rule in &valid_rules {
                    let existing = self
                        .conn
                        .query_row(
                            "SELECT 1 FROM x_watch_manual_rules WHERE handle = ?1",
                            params![rule.handle],
                            |row| row.get::<_, i64>(0),
                        )
                        .optional()?
                        .is_some();
                    let metadata_json = canonical_json(&json!({
                        "reviewed_by": reviewed_by,
                        "import_metadata": rule.metadata,
                        "source_text_untrusted": true
                    }))?;
                    self.conn.execute(
                        r#"
                        INSERT INTO x_watch_manual_rules
                          (handle, decision, category, reason, metadata_json, created_at, updated_at)
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)
                        ON CONFLICT(handle) DO UPDATE SET
                          decision = excluded.decision,
                          category = excluded.category,
                          reason = excluded.reason,
                          metadata_json = excluded.metadata_json,
                          updated_at = excluded.updated_at
                        "#,
                        params![
                            rule.handle,
                            rule.decision,
                            rule.category,
                            rule.reason,
                            metadata_json,
                            written_at,
                        ],
                    )?;
                    if existing {
                        updated += 1;
                    } else {
                        imported += 1;
                    }
                }
                self.conn.execute("COMMIT", [])?;
                Ok(())
            })();
            if let Err(error) = write_result {
                let _ = self.conn.execute("ROLLBACK", []);
                return Err(error);
            }
            for item in &mut items {
                if item.status == "pending_write" {
                    item.status = "written".to_string();
                }
            }
        }

        Ok(XWatchManualRuleImportReport {
            proof_level: if dry_run {
                "local_manual_rule_dry_run".to_string()
            } else if blocked_by_rejections {
                "local_manual_rule_import_blocked".to_string()
            } else {
                "local_reviewed_manual_rule_import".to_string()
            },
            reviewed_by: reviewed_by.to_string(),
            dry_run,
            seen: items.len(),
            imported,
            updated,
            rejected,
            items,
            non_claims: x_watch_manual_rule_import_non_claims(),
        })
    }

    pub fn x_curate_watch_sources(&self, mode: &str) -> Result<XWatchCurationReport> {
        let mode = normalize_x_watch_curation_mode(mode)?;
        let candidates = self.x_watch_curation_candidates()?;
        let mut decisions = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            decisions.push(classify_x_watch_candidate(candidate));
        }
        self.record_x_watch_curation_run(&mode, decisions)
    }

    pub fn latest_x_watch_curation_report(&self) -> Result<Option<XWatchCurationReport>> {
        let run_id = self
            .conn
            .query_row(
                "SELECT id FROM x_watch_curation_runs ORDER BY created_at DESC LIMIT 1",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        run_id
            .as_deref()
            .map(|id| self.x_watch_curation_report(id))
            .transpose()
    }

    pub fn x_watch_curation_report(&self, run_id: &str) -> Result<XWatchCurationReport> {
        validate_id(run_id)?;
        let run = self
            .read_x_watch_curation_run(run_id)?
            .with_context(|| format!("X watch curation run not found: {run_id}"))?;
        let decisions = self.list_x_watch_curation_decisions(run_id)?;
        Ok(XWatchCurationReport {
            proof_level: "local_proof_ledger".to_string(),
            counts: x_watch_curation_counts(&decisions),
            run,
            decisions,
            non_claims: x_watch_curation_non_claims(),
        })
    }

    pub fn restore_x_watch_curation_run(
        &self,
        run_id: &str,
    ) -> Result<XWatchCurationRestoreReport> {
        validate_id(run_id)?;
        let pending: Vec<(String, String, String, String, String)> = {
            let mut stmt = self.conn.prepare(
                r#"
                SELECT watch_source_id, previous_status, previous_label, previous_cadence, previous_metadata_json
                FROM x_watch_restore_snapshots
                WHERE run_id = ?1 AND restored_at IS NULL
                ORDER BY watch_source_id
                "#,
            )?;
            rows(stmt.query_map(params![run_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })?)?
        };
        if pending.is_empty() {
            bail!("X watch curation run has no pending restore snapshots: {run_id}");
        }

        let restored_at = now();
        let restore_result = (|| -> Result<Vec<String>> {
            self.conn.execute("BEGIN IMMEDIATE", [])?;
            let mut restored = Vec::with_capacity(pending.len());
            for (watch_source_id, status, label, cadence, metadata_json) in &pending {
                validate_id(watch_source_id)?;
                validate_watch_source_status(status)?;
                validate_watch_source_cadence(cadence)?;
                parse_json_column(metadata_json, 0)
                    .context("restore snapshot metadata_json was invalid")?;
                let updated = self.conn.execute(
                    r#"
                    UPDATE watch_sources
                    SET status = ?2, label = ?3, cadence = ?4, metadata_json = ?5, updated_at = ?6
                    WHERE id = ?1
                    "#,
                    params![
                        watch_source_id,
                        status,
                        label,
                        cadence,
                        metadata_json,
                        restored_at
                    ],
                )?;
                if updated != 1 {
                    bail!("restore snapshot watch source missing: {watch_source_id}");
                }
                self.conn.execute(
                    r#"
                    UPDATE x_watch_restore_snapshots
                    SET restored_at = ?3
                    WHERE run_id = ?1 AND watch_source_id = ?2
                    "#,
                    params![run_id, watch_source_id, restored_at],
                )?;
                restored.push(watch_source_id.clone());
            }
            self.conn.execute(
                r#"
                UPDATE x_watch_curation_runs
                SET restored_count = restored_count + ?2, completed_at = COALESCE(completed_at, ?3)
                WHERE id = ?1
                "#,
                params![run_id, restored.len(), restored_at],
            )?;
            self.conn.execute("COMMIT", [])?;
            Ok(restored)
        })();
        let restored_watch_source_ids = match restore_result {
            Ok(restored) => restored,
            Err(error) => {
                let _ = self.conn.execute("ROLLBACK", []);
                return Err(error);
            }
        };

        Ok(XWatchCurationRestoreReport {
            proof_level: "local_restore_proof".to_string(),
            run_id: run_id.to_string(),
            restored_count: restored_watch_source_ids.len(),
            restored_watch_source_ids,
            non_claims: x_watch_curation_non_claims(),
        })
    }

    fn x_watch_curation_candidates(&self) -> Result<Vec<XWatchCandidate>> {
        let mut candidates = Vec::new();
        for source in self
            .list_watch_sources()?
            .into_iter()
            .filter(|source| source.source_kind == "x_handle")
        {
            let (profile_present, display_name, description) =
                self.x_watch_profile_summary(&source)?;
            let local_tweets = self.count_x_items_by_author(&source.locator)?;
            let tech_tweet_hits = self.count_x_tech_tweet_hits(&source.locator)?;
            let bookmarked_items = self.count_x_bookmarks_by_author(&source.locator)?;
            let manual_rule = self.read_x_watch_manual_rule(&source.locator)?;
            candidates.push(XWatchCandidate {
                source,
                profile_present,
                display_name,
                description,
                local_tweets,
                tech_tweet_hits,
                bookmarked_items,
                manual_rule,
            });
        }
        Ok(candidates)
    }

    fn x_watch_profile_summary(&self, source: &WatchSource) -> Result<(bool, String, String)> {
        let profile = self
            .conn
            .query_row(
                r#"
                SELECT display_name, description
                FROM x_profiles
                WHERE lower(handle) = lower(?1)
                ORDER BY updated_at DESC
                LIMIT 1
                "#,
                params![source.locator],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()?;
        if let Some((display_name, description)) = profile {
            return Ok((true, display_name, description));
        }
        Ok((
            false,
            source
                .metadata
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            source
                .metadata
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
        ))
    }

    fn count_x_items_by_author(&self, handle: &str) -> Result<usize> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM x_items WHERE lower(author) = lower(?1)",
                params![handle],
                |row| row.get::<_, i64>(0),
            )
            .map(nonnegative_usize)
            .map_err(Into::into)
    }

    fn count_x_tech_tweet_hits(&self, handle: &str) -> Result<usize> {
        let mut stmt = self.conn.prepare(
            "SELECT text FROM x_items WHERE lower(author) = lower(?1) ORDER BY imported_at DESC LIMIT 200",
        )?;
        let texts = rows(stmt.query_map(params![handle], |row| row.get::<_, String>(0))?)?;
        Ok(texts
            .iter()
            .filter(|text| x_watch_text_has_tech_signal(text))
            .count())
    }

    fn count_x_bookmarks_by_author(&self, handle: &str) -> Result<usize> {
        self.conn
            .query_row(
                r#"
                SELECT COUNT(*)
                FROM x_collections c
                JOIN x_items xi ON xi.x_id = c.tweet_x_id
                WHERE c.collection_kind = 'bookmark' AND lower(xi.author) = lower(?1)
                "#,
                params![handle],
                |row| row.get::<_, i64>(0),
            )
            .map(nonnegative_usize)
            .map_err(Into::into)
    }

    fn read_x_watch_manual_rule(&self, handle: &str) -> Result<Option<XWatchManualRule>> {
        self.conn
            .query_row(
                r#"
                SELECT decision, category, reason
                FROM x_watch_manual_rules
                WHERE lower(handle) = lower(?1)
                "#,
                params![handle],
                |row| {
                    Ok(XWatchManualRule {
                        decision: row.get(0)?,
                        category: row.get(1)?,
                        reason: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    fn record_x_watch_curation_run(
        &self,
        mode: &str,
        mut decisions: Vec<XWatchCurationDecision>,
    ) -> Result<XWatchCurationReport> {
        let run_id = format!(
            "xwcur-{}",
            &sha256(
                format!(
                    "{}\n{}\n{}\n{}",
                    X_WATCH_CURATION_CLASSIFIER_VERSION,
                    mode,
                    now(),
                    Uuid::new_v4()
                )
                .as_bytes()
            )[..24]
        );
        for decision in &mut decisions {
            decision.run_id = run_id.clone();
            decision.id = x_watch_curation_decision_id(&run_id, &decision.watch_source_id);
        }
        let counts = x_watch_curation_counts(&decisions);
        let created_at = now();
        let completed_at = now();
        let mut paused_count = 0_usize;

        let run_result = (|| -> Result<()> {
            self.conn.execute("BEGIN IMMEDIATE", [])?;
            self.conn.execute(
                r#"
                INSERT INTO x_watch_curation_runs
                  (id, classifier_version, mode, status, input_count,
                   keep_count, review_keep_leaning_count, review_drop_leaning_count,
                   needs_profile_enrichment_count, pause_candidate_count, paused_count,
                   restored_count, metadata_json, created_at, completed_at)
                VALUES (?1, ?2, ?3, 'completed', ?4, ?5, ?6, ?7, ?8, ?9, 0, 0, ?10, ?11, ?12)
                "#,
                params![
                    run_id,
                    X_WATCH_CURATION_CLASSIFIER_VERSION,
                    mode,
                    decisions.len(),
                    counts.get("keep").copied().unwrap_or(0),
                    counts.get("review_keep_leaning").copied().unwrap_or(0),
                    counts.get("review_drop_leaning").copied().unwrap_or(0),
                    counts.get("needs_profile_enrichment").copied().unwrap_or(0),
                    counts.get("paused_excluded").copied().unwrap_or(0),
                    canonical_json(&json!({
                        "source": "arcwell x curate-watch-sources",
                        "source_text_untrusted": true,
                        "non_destructive_by_default": true
                    }))?,
                    created_at,
                    completed_at,
                ],
            )?;
            for decision in &decisions {
                self.conn.execute(
                    r#"
                    INSERT INTO x_watch_curation_decisions
                      (id, run_id, watch_source_id, handle, previous_status, proposed_status,
                       recommendation, category, score, confidence, reason, evidence_json, created_at, applied_at)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, NULL)
                    "#,
                    params![
                        decision.id,
                        run_id,
                        decision.watch_source_id,
                        decision.handle,
                        decision.previous_status,
                        decision.proposed_status,
                        decision.recommendation,
                        decision.category,
                        decision.score,
                        decision.confidence,
                        decision.reason,
                        canonical_json(&decision.evidence)?,
                        created_at,
                    ],
                )?;
                self.insert_x_watch_curation_evidence_rows(decision)?;
                if mode == "pause_only"
                    && decision.previous_status == "active"
                    && decision.recommendation == "paused_excluded"
                    && decision.proposed_status == "paused"
                {
                    self.snapshot_and_pause_x_watch_source(&run_id, decision, &completed_at)?;
                    paused_count += 1;
                }
            }
            if paused_count > 0 {
                self.conn.execute(
                    "UPDATE x_watch_curation_runs SET paused_count = ?2 WHERE id = ?1",
                    params![run_id, paused_count],
                )?;
            }
            self.conn.execute("COMMIT", [])?;
            Ok(())
        })();
        if let Err(error) = run_result {
            let _ = self.conn.execute("ROLLBACK", []);
            return Err(error);
        }

        let run = self
            .read_x_watch_curation_run(&run_id)?
            .with_context(|| format!("created X watch curation run not found: {run_id}"))?;
        let decisions = self.list_x_watch_curation_decisions(&run_id)?;
        Ok(XWatchCurationReport {
            proof_level: if mode == "pause_only" {
                "local_pause_only_proof".to_string()
            } else {
                "local_dry_run_proof".to_string()
            },
            counts,
            run,
            decisions,
            non_claims: x_watch_curation_non_claims(),
        })
    }

    fn insert_x_watch_curation_evidence_rows(
        &self,
        decision: &XWatchCurationDecision,
    ) -> Result<()> {
        let Some(items) = decision.evidence.get("items").and_then(Value::as_array) else {
            return Ok(());
        };
        for (index, item) in items.iter().enumerate() {
            let kind = item
                .get("kind")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let value = item
                .get("value")
                .and_then(Value::as_str)
                .unwrap_or("")
                .chars()
                .take(1_000)
                .collect::<String>();
            let weight = item.get("weight").and_then(Value::as_i64).unwrap_or(0);
            let id = format!(
                "xwce-{}",
                &sha256(format!("{}\n{}\n{}", decision.id, index, kind).as_bytes())[..24]
            );
            self.conn.execute(
                r#"
                INSERT INTO x_watch_curation_evidence
                  (id, decision_id, evidence_kind, evidence_value, weight, created_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                "#,
                params![id, decision.id, kind, value, weight, decision.created_at],
            )?;
        }
        Ok(())
    }

    fn snapshot_and_pause_x_watch_source(
        &self,
        run_id: &str,
        decision: &XWatchCurationDecision,
        applied_at: &str,
    ) -> Result<()> {
        let source = self
            .read_watch_source(&decision.watch_source_id)?
            .with_context(|| format!("watch source not found: {}", decision.watch_source_id))?;
        self.conn.execute(
            r#"
            INSERT INTO x_watch_restore_snapshots
              (run_id, watch_source_id, previous_status, previous_label, previous_cadence,
               previous_metadata_json, restored_at, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, ?7)
            "#,
            params![
                run_id,
                source.id,
                source.status,
                source.label,
                source.cadence,
                canonical_json(&source.metadata)?,
                applied_at,
            ],
        )?;
        self.conn.execute(
            "UPDATE watch_sources SET status = 'paused', updated_at = ?2 WHERE id = ?1",
            params![source.id, applied_at],
        )?;
        self.conn.execute(
            "UPDATE x_watch_curation_decisions SET applied_at = ?2 WHERE id = ?1",
            params![decision.id, applied_at],
        )?;
        Ok(())
    }

    fn read_x_watch_curation_run(&self, run_id: &str) -> Result<Option<XWatchCurationRun>> {
        self.conn
            .query_row(
                r#"
                SELECT id, classifier_version, mode, status, input_count,
                       keep_count, review_keep_leaning_count, review_drop_leaning_count,
                       needs_profile_enrichment_count, pause_candidate_count, paused_count,
                       restored_count, error, metadata_json, created_at, completed_at
                FROM x_watch_curation_runs
                WHERE id = ?1
                "#,
                params![run_id],
                x_watch_curation_run_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    fn list_x_watch_curation_decisions(&self, run_id: &str) -> Result<Vec<XWatchCurationDecision>> {
        validate_id(run_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, run_id, watch_source_id, handle, previous_status, proposed_status,
                   recommendation, category, score, confidence, reason, evidence_json,
                   created_at, applied_at
            FROM x_watch_curation_decisions
            WHERE run_id = ?1
            ORDER BY recommendation, score DESC, handle
            "#,
        )?;
        rows(stmt.query_map(params![run_id], x_watch_curation_decision_from_row)?)
    }

    fn read_x_watch_source_by_handle(&self, handle: &str) -> Result<Option<WatchSource>> {
        validate_x_handle(handle)?;
        self.conn
            .query_row(
                r#"
                SELECT id, source_kind, locator, label, cadence, status, metadata_json, created_at, updated_at
                FROM watch_sources
                WHERE source_kind = 'x_handle' AND lower(locator) = lower(?1)
                LIMIT 1
                "#,
                params![handle],
                watch_source_from_row,
            )
            .optional()
            .map_err(Into::into)
    }
}

fn normalize_x_watch_manual_rule(mut rule: XWatchManualRuleInput) -> Result<XWatchManualRuleInput> {
    rule.handle = rule
        .handle
        .trim()
        .trim_start_matches('@')
        .to_ascii_lowercase();
    validate_x_handle(&rule.handle)?;
    rule.decision = rule.decision.trim().to_string();
    match rule.decision.as_str() {
        "manual_always_keep" | "manual_always_exclude" => {}
        other => bail!("unsupported X watch manual-rule decision: {other}"),
    }
    rule.category = rule.category.trim().to_string();
    validate_key(&rule.category)?;
    validate_notes(&rule.reason)?;
    if rule.reason.len() < 12 {
        bail!("manual-rule reason is too short");
    }
    validate_json_size_for_x_watch_rule(&rule.metadata)?;
    Ok(rule)
}

fn validate_json_size_for_x_watch_rule(value: &Value) -> Result<()> {
    let text = canonical_json(value)?;
    if text.len() > 20_000 {
        bail!("manual-rule metadata JSON is too large");
    }
    Ok(())
}

fn x_watch_manual_rule_import_non_claims() -> Vec<String> {
    vec![
        "Manual rules are reviewed curation inputs, not provider evidence.".to_string(),
        "Importing a manual exclude rule does not pause a watch source until pause-only apply runs.".to_string(),
        "Manual-rule reasons are untrusted text and never instructions.".to_string(),
    ]
}

fn classify_x_watch_candidate(candidate: XWatchCandidate) -> XWatchCurationDecision {
    let created_at = now();
    let mut score = 0_i64;
    let mut evidence_items = Vec::new();
    let mut category = "unknown_review".to_string();
    let mut recommendation = "review_drop_leaning".to_string();
    let mut proposed_status = candidate.source.status.clone();
    let mut reasons = Vec::new();

    if let Some(rule) = &candidate.manual_rule {
        evidence_items.push(json!({
            "kind": "manual_rule",
            "value": rule.reason,
            "weight": 100,
            "source_text_untrusted": false
        }));
        category = rule.category.clone();
        if rule.decision == "manual_always_keep" {
            score = 100;
            recommendation = "keep".to_string();
            proposed_status = "active".to_string();
            reasons.push(format!("manual keep rule: {}", rule.reason));
        } else if rule.decision == "manual_always_exclude" {
            score = -100;
            recommendation = "paused_excluded".to_string();
            proposed_status = "paused".to_string();
            reasons.push(format!("manual exclude rule: {}", rule.reason));
        }
    }

    if candidate.manual_rule.is_none() {
        let combined = format!(
            "{} {} {} {}",
            candidate.source.locator,
            candidate.source.label,
            candidate.display_name,
            candidate.description
        );
        if let Some(seed_category) = x_watch_seed_allowlist_category(&candidate.source.locator) {
            score += 10;
            category = seed_category.to_string();
            reasons.push(format!(
                "seed allowlist tech/devrel account: {}",
                candidate.source.locator
            ));
            evidence_items.push(json!({
                "kind": "seed_allowlist",
                "value": candidate.source.locator,
                "weight": 10,
                "source_text_untrusted": false
            }));
        }
        let profile_score = x_watch_profile_score(&combined);
        score += profile_score;
        if profile_score > 0 {
            reasons.push(format!("profile/handle tech signal score {profile_score}"));
            evidence_items.push(json!({
                "kind": "profile_text",
                "value": excerpt_preserving_whitespace(&combined, 1000),
                "weight": profile_score,
                "source_text_untrusted": true
            }));
        }
        if candidate.tech_tweet_hits > 0 {
            let tweet_score = (candidate.tech_tweet_hits.min(5) as i64) * 2;
            score += tweet_score;
            reasons.push(format!(
                "{} local tech tweet hits",
                candidate.tech_tweet_hits
            ));
            evidence_items.push(json!({
                "kind": "local_tweet_hits",
                "value": candidate.tech_tweet_hits.to_string(),
                "weight": tweet_score,
                "source_text_untrusted": true
            }));
        }
        if candidate.bookmarked_items > 0 {
            let bookmark_score = (candidate.bookmarked_items.min(3) as i64) * 3;
            score += bookmark_score;
            reasons.push(format!(
                "{} bookmarked local items",
                candidate.bookmarked_items
            ));
            evidence_items.push(json!({
                "kind": "bookmark_engagement",
                "value": candidate.bookmarked_items.to_string(),
                "weight": bookmark_score,
                "source_text_untrusted": true
            }));
        }
        if category == "unknown_review" {
            category = x_watch_category(&combined).to_string();
        }
        let lacks_profile_evidence = !candidate.profile_present
            && candidate.description.trim().is_empty()
            && candidate.local_tweets == 0
            && candidate.bookmarked_items == 0;
        if lacks_profile_evidence && score < 4 {
            recommendation = "needs_profile_enrichment".to_string();
            proposed_status = candidate.source.status.clone();
            reasons.push("insufficient local profile/tweet/bookmark evidence".to_string());
        } else if score >= 7 {
            recommendation = "keep".to_string();
            proposed_status = "active".to_string();
        } else if score >= 4 {
            recommendation = "review_keep_leaning".to_string();
            proposed_status = candidate.source.status.clone();
        } else {
            recommendation = "review_drop_leaning".to_string();
            proposed_status = candidate.source.status.clone();
        }
    }

    if evidence_items.is_empty() {
        evidence_items.push(json!({
            "kind": "absence",
            "value": "no positive local evidence",
            "weight": 0,
            "source_text_untrusted": false
        }));
    }

    let confidence = match recommendation.as_str() {
        "keep" if score >= 10 => 0.9,
        "keep" => 0.75,
        "review_keep_leaning" => 0.55,
        "review_drop_leaning" => 0.35,
        "needs_profile_enrichment" => 0.2,
        "paused_excluded" => 0.95,
        _ => 0.25,
    };

    XWatchCurationDecision {
        id: String::new(),
        run_id: String::new(),
        watch_source_id: candidate.source.id,
        handle: candidate.source.locator,
        previous_status: candidate.source.status,
        proposed_status,
        recommendation,
        category,
        score,
        confidence,
        reason: if reasons.is_empty() {
            "no local signal; review required".to_string()
        } else {
            reasons.join("; ")
        },
        evidence: json!({
            "schema_version": 1,
            "source_text_untrusted": true,
            "profile_present": candidate.profile_present,
            "description_present": !candidate.description.trim().is_empty(),
            "local_tweets": candidate.local_tweets,
            "tech_tweet_hits": candidate.tech_tweet_hits,
            "bookmarked_items": candidate.bookmarked_items,
            "items": evidence_items
        }),
        created_at,
        applied_at: None,
    }
}

fn normalize_x_watch_curation_mode(mode: &str) -> Result<String> {
    match mode.trim().replace('-', "_").as_str() {
        "" | "dry_run" => Ok("dry_run".to_string()),
        "pause_only" => Ok("pause_only".to_string()),
        other => bail!("unsupported X watch curation mode: {other}"),
    }
}

fn x_watch_curation_counts(decisions: &[XWatchCurationDecision]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for decision in decisions {
        *counts.entry(decision.recommendation.clone()).or_insert(0) += 1;
    }
    counts
}

fn x_watch_curation_non_claims() -> Vec<String> {
    vec![
        "This is not proof that the X watch list is fully curated.".to_string(),
        "This is not live profile enrichment proof.".to_string(),
        "This does not delete watch sources.".to_string(),
        "Unknown sparse accounts are enrichment work, not safe drop candidates.".to_string(),
    ]
}

fn x_watch_curation_decision_id(run_id: &str, watch_source_id: &str) -> String {
    format!(
        "xwcd-{}",
        &sha256(format!("{run_id}\n{watch_source_id}").as_bytes())[..24]
    )
}

fn x_watch_curation_run_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<XWatchCurationRun> {
    let metadata_json: String = row.get(13)?;
    Ok(XWatchCurationRun {
        id: row.get(0)?,
        classifier_version: row.get(1)?,
        mode: row.get(2)?,
        status: row.get(3)?,
        input_count: nonnegative_usize(row.get(4)?),
        keep_count: nonnegative_usize(row.get(5)?),
        review_keep_leaning_count: nonnegative_usize(row.get(6)?),
        review_drop_leaning_count: nonnegative_usize(row.get(7)?),
        needs_profile_enrichment_count: nonnegative_usize(row.get(8)?),
        pause_candidate_count: nonnegative_usize(row.get(9)?),
        paused_count: nonnegative_usize(row.get(10)?),
        restored_count: nonnegative_usize(row.get(11)?),
        error: row.get(12)?,
        metadata: parse_json_column(&metadata_json, 13)?,
        created_at: row.get(14)?,
        completed_at: row.get(15)?,
    })
}

fn x_watch_curation_decision_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<XWatchCurationDecision> {
    let evidence_json: String = row.get(11)?;
    Ok(XWatchCurationDecision {
        id: row.get(0)?,
        run_id: row.get(1)?,
        watch_source_id: row.get(2)?,
        handle: row.get(3)?,
        previous_status: row.get(4)?,
        proposed_status: row.get(5)?,
        recommendation: row.get(6)?,
        category: row.get(7)?,
        score: row.get(8)?,
        confidence: row.get(9)?,
        reason: row.get(10)?,
        evidence: parse_json_column(&evidence_json, 11)?,
        created_at: row.get(12)?,
        applied_at: row.get(13)?,
    })
}

fn x_watch_text_has_tech_signal(text: &str) -> bool {
    let lower = x_watch_normalized_signal_text(text);
    x_watch_tech_terms()
        .iter()
        .any(|term| x_watch_contains_signal(&lower, term))
}

fn x_watch_profile_score(text: &str) -> i64 {
    let lower = x_watch_normalized_signal_text(text);
    let mut score = 0_i64;
    for term in x_watch_tech_terms() {
        if x_watch_contains_signal(&lower, term) {
            score += 2;
        }
    }
    for term in [
        "openai",
        "anthropic",
        "deepmind",
        "huggingface",
        "vercel",
        "cloudflare",
        "cursor",
        "claude",
        "codex",
        "nvidia",
        "github",
        "linux",
        "kubernetes",
        "postgres",
        "dialogflow",
        "asahi linux",
    ] {
        if x_watch_contains_signal(&lower, term) {
            score += 3;
        }
    }
    score.min(20)
}

fn x_watch_category(text: &str) -> &'static str {
    let lower = x_watch_normalized_signal_text(text);
    if ["openai", "anthropic", "deepmind", "nvidia", "model"]
        .iter()
        .any(|term| x_watch_contains_signal(&lower, term))
    {
        "ai_model_lab"
    } else if ["research", "paper", "benchmark", "eval"]
        .iter()
        .any(|term| x_watch_contains_signal(&lower, term))
    {
        "ai_research"
    } else if [
        "devrel",
        "developer relations",
        "dx",
        "developer experience",
    ]
    .iter()
    .any(|term| x_watch_contains_signal(&lower, term))
    {
        "devrel_dx"
    } else if [
        "sdk",
        "api",
        "tool",
        "agent",
        "cursor",
        "codex",
        "devtools",
        "developer tools",
        "cli",
        "terminal",
        "framework",
        "compiler",
        "runtime",
    ]
    .iter()
    .any(|term| x_watch_contains_signal(&lower, term))
    {
        "developer_tools"
    } else if [
        "cloud",
        "infrastructure",
        "security",
        "linux",
        "kubernetes",
        "docker",
        "postgres",
        "database",
        "platform",
    ]
    .iter()
    .any(|term| x_watch_contains_signal(&lower, term))
    {
        "cloud_infra"
    } else if [
        "software",
        "engineer",
        "engineering",
        "code",
        "programming",
        "frontend",
        "backend",
        "ios",
        "macos",
        "swiftui",
    ]
    .iter()
    .any(|term| x_watch_contains_signal(&lower, term))
    {
        "software_engineering"
    } else {
        "unknown_review"
    }
}

fn x_watch_normalized_signal_text(text: &str) -> String {
    let mut normalized = String::with_capacity(text.len() + 16);
    let mut previous: Option<char> = None;
    for ch in text.chars() {
        if let Some(prev) = previous {
            if prev.is_ascii_lowercase() && ch.is_ascii_uppercase() {
                normalized.push(' ');
            }
        }
        normalized.push(ch.to_ascii_lowercase());
        previous = Some(ch);
    }
    normalized.push(' ');
    normalized.push_str(&text.to_ascii_lowercase());
    normalized
}

fn x_watch_contains_signal(lower_text: &str, term: &str) -> bool {
    if term.contains(' ') {
        return lower_text.contains(term);
    }
    lower_text
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .any(|token| token == term)
}

fn x_watch_tech_terms() -> &'static [&'static str] {
    &[
        "ai",
        "artificial intelligence",
        "machine learning",
        "ml",
        "llm",
        "model",
        "agent",
        "agents",
        "developer",
        "developers",
        "devrel",
        "devtools",
        "developer tools",
        "developer platform",
        "developer experience",
        "sdk",
        "api",
        "github",
        "software",
        "engineer",
        "engineering",
        "programming",
        "code",
        "coding",
        "rust",
        "python",
        "typescript",
        "javascript",
        "swift",
        "swiftui",
        "ios",
        "macos",
        "linux",
        "cli",
        "terminal",
        "database",
        "postgres",
        "kubernetes",
        "docker",
        "wasm",
        "browser",
        "frontend",
        "backend",
        "framework",
        "compiler",
        "runtime",
        "platform",
        "cloud",
        "infrastructure",
        "security",
        "benchmark",
        "eval",
        "research",
        "gpu",
        "open source",
        "oss",
    ]
}

fn x_watch_seed_allowlist_category(handle: &str) -> Option<&'static str> {
    match handle
        .trim()
        .trim_start_matches('@')
        .to_ascii_lowercase()
        .as_str()
    {
        "asahilinux" => Some("cloud_infra"),
        "dialogflow" => Some("developer_tools"),
        "engineering" => Some("software_engineering"),
        "github" => Some("developer_tools"),
        "githubnext" => Some("developer_tools"),
        "githubengineering" => Some("software_engineering"),
        "linuxfoundation" => Some("cloud_infra"),
        "nodejs" => Some("developer_tools"),
        "reactjs" => Some("developer_tools"),
        "rustlang" => Some("developer_tools"),
        "sveltejs" => Some("developer_tools"),
        "typescript" => Some("developer_tools"),
        "vercel" => Some("developer_tools"),
        _ => None,
    }
}
