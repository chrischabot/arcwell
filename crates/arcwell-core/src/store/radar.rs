use super::*;

impl Store {
    pub fn create_radar_profile(&self, input: RadarProfileInput) -> Result<RadarProfile> {
        let normalized = normalize_radar_profile_input(input)?;
        let id = Uuid::new_v4().to_string();
        let timestamp = now();
        let status = radar_profile_status(&normalized.source_selectors);
        self.conn.execute(
            r#"
            INSERT INTO radar_profiles
              (id, name, description, status, window_hours, min_score, max_items,
               languages_json, category_groups_json, source_selectors_json,
               delivery_policy_json, model_policy_json, metadata_json, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?14)
            "#,
            params![
                id,
                normalized.name,
                normalized.description,
                status,
                normalized.window_hours,
                normalized.min_score,
                normalized.max_items,
                serde_json::to_string(&normalized.languages)?,
                serde_json::to_string(&json!({}))?,
                serde_json::to_string(&normalized.source_selectors)?,
                serde_json::to_string(&normalized.delivery_policy)?,
                serde_json::to_string(&normalized.model_policy)?,
                serde_json::to_string(&normalized.metadata)?,
                timestamp
            ],
        )?;
        self.read_radar_profile(&id)?
            .with_context(|| format!("inserted radar profile not found: {id}"))
    }

    pub fn list_radar_profiles(&self) -> Result<Vec<RadarProfile>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, name, description, status, window_hours, min_score, max_items,
                   languages_json, category_groups_json, source_selectors_json,
                   delivery_policy_json, model_policy_json, metadata_json, created_at, updated_at
            FROM radar_profiles
            ORDER BY updated_at DESC, name ASC
            "#,
        )?;
        rows(stmt.query_map([], radar_profile_from_row)?)
    }

    pub fn read_radar_profile(&self, id_or_name: &str) -> Result<Option<RadarProfile>> {
        validate_id(id_or_name)?;
        self.conn
            .query_row(
                r#"
                SELECT id, name, description, status, window_hours, min_score, max_items,
                       languages_json, category_groups_json, source_selectors_json,
                       delivery_policy_json, model_policy_json, metadata_json, created_at, updated_at
                FROM radar_profiles
                WHERE id = ?1 OR name = ?1
                "#,
                params![id_or_name],
                radar_profile_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn run_radar_profile(
        &self,
        profile_id_or_name: &str,
        window_hours_override: Option<i64>,
    ) -> Result<RadarFetchReport> {
        self.run_radar_profile_with_options(profile_id_or_name, window_hours_override, false)
    }

    pub fn run_radar_profile_with_options(
        &self,
        profile_id_or_name: &str,
        window_hours_override: Option<i64>,
        fetch_live: bool,
    ) -> Result<RadarFetchReport> {
        let profile = self
            .read_radar_profile(profile_id_or_name)?
            .with_context(|| format!("radar profile not found: {profile_id_or_name}"))?;
        let balance_config = radar_balance_config_from_metadata(&profile.metadata)?;
        let window_hours = window_hours_override.unwrap_or(profile.window_hours);
        if window_hours <= 0 {
            bail!("window_hours must be greater than zero");
        }
        let window_end = Utc::now();
        let window_start = window_end - chrono::Duration::hours(window_hours);
        let started_at = now();
        let run_id = Uuid::new_v4().to_string();
        let unsupported_selectors = unsupported_radar_selectors(&profile.source_selectors);
        let mut run_metadata = json!({
            "proof_level": if fetch_live { "Live Adapter Attempt" } else { "Local Fixture Proof" },
            "source_family": if fetch_live { "live_adapter_then_source_card_projection" } else { "source_card_projection" },
            "unsupported_selectors": unsupported_selectors,
            "fetch_live": fetch_live,
            "warning": if fetch_live {
                "This run invokes existing Arcwell source adapters before projecting source cards; enrichment, model synthesis, delivery, and scheduling are not proven by this stage."
            } else {
                "This run projects existing Arcwell source cards only; live network adapters, enrichment, summaries, and delivery are not proven by this stage."
            }
        });
        if balance_config.enabled() {
            run_metadata["balance_config"] = balance_config.to_json();
        }
        self.conn.execute(
            r#"
            INSERT INTO radar_runs
              (id, profile_id, status, window_start, window_end, stage, source_selection_json,
               metadata_json, started_at, updated_at)
            VALUES (?1, ?2, 'fetching', ?3, ?4, 'fetching', ?5, ?6, ?7, ?7)
            "#,
            params![
                run_id,
                profile.id,
                window_start.to_rfc3339(),
                window_end.to_rfc3339(),
                serde_json::to_string(&profile.source_selectors)?,
                serde_json::to_string(&run_metadata)?,
                started_at
            ],
        )?;

        let mut warnings = Vec::new();
        if !unsupported_selectors.is_empty() {
            warnings.push(format!(
                "{} unsupported selector(s) were recorded and skipped",
                unsupported_selectors.len()
            ));
        }
        let mut adapter_jobs = Vec::new();
        let mut live_pre_job_failed = false;
        if fetch_live {
            let (jobs, live_warnings, pre_job_failed) =
                self.run_radar_live_fetches_for_selectors(&profile.source_selectors)?;
            adapter_jobs = jobs;
            live_pre_job_failed = pre_job_failed;
            warnings.extend(live_warnings);
            run_metadata["adapter_jobs"] = json!(
                adapter_jobs
                    .iter()
                    .map(|job| json!({
                        "id": job.id,
                        "kind": job.kind,
                        "status": job.status,
                        "error": job.error,
                        "result": job.result_json
                    }))
                    .collect::<Vec<_>>()
            );
            run_metadata["live_fetch_warnings"] = json!(warnings);
        }
        let mut source_cards = Vec::new();
        let selectors = profile
            .source_selectors
            .as_array()
            .cloned()
            .unwrap_or_default();
        for selector in selectors {
            if !radar_selector_is_source_card_backed(&selector) {
                continue;
            }
            let mut cards = self.radar_source_cards_for_selector(&selector)?;
            source_cards.append(&mut cards);
        }
        source_cards.sort_by(|left, right| left.id.cmp(&right.id));
        source_cards.dedup_by(|left, right| left.id == right.id);

        let mut items_inserted = 0usize;
        for card in &source_cards {
            let item = radar_item_from_source_card(&run_id, card)?;
            self.insert_radar_item(&item)?;
            items_inserted += 1;
        }
        self.rebuild_radar_fts(Some(&run_id))?;
        let exact_dedup_groups = self.dedupe_radar_run(&run_id)?;
        let initial_scores_inserted = self.score_radar_run(&run_id)?;
        let semantic_dedup_groups = self.dedupe_radar_run_semantic_topics(&run_id)?;
        let scores_inserted = if semantic_dedup_groups > 0 {
            self.score_radar_run(&run_id)?
        } else {
            initial_scores_inserted
        };
        let source_quality_windows = self.record_radar_source_quality_window(&run_id)?;
        let radar_scores = self.list_radar_scores(&run_id)?;
        let score_distribution = radar_score_distribution_json(&radar_scores);
        let selected_items = self.count_radar_selected_scores(&run_id)?;
        let raw_count = source_cards.len() as i64;
        let now = now();
        let live_failed =
            live_pre_job_failed || adapter_jobs.iter().any(|job| job.status != "completed");
        let (proof_level, source_family, projection_warning) =
            self.classify_radar_projection_proof(&source_cards, fetch_live, live_failed)?;
        let status = if raw_count == 0 && (live_failed || !unsupported_selectors.is_empty()) {
            "blocked"
        } else if raw_count == 0 {
            "empty"
        } else if live_failed {
            "partial"
        } else {
            "scored"
        };
        let error = if raw_count == 0 && live_failed {
            Some(
                "live adapter fetch failed and no supported selectors produced radar items"
                    .to_string(),
            )
        } else if raw_count == 0 && !unsupported_selectors.is_empty() {
            Some("no supported selectors produced radar items".to_string())
        } else {
            None
        };
        run_metadata["proof_level"] = json!(proof_level);
        run_metadata["source_family"] = json!(source_family);
        run_metadata["warning"] = json!(projection_warning);
        run_metadata["live_fetch_failed"] = json!(live_failed);
        run_metadata["live_fetch_pre_job_failed"] = json!(live_pre_job_failed);
        run_metadata["adapter_job_count"] = json!(adapter_jobs.len());
        let adapter_job_ids = adapter_jobs
            .iter()
            .map(|job| job.id.clone())
            .collect::<BTreeSet<_>>();
        let adapter_runs = if adapter_job_ids.is_empty() {
            Vec::new()
        } else {
            self.list_knowledge_adapter_runs(500)?
                .into_iter()
                .filter(|run| adapter_job_ids.contains(&run.job_id))
                .collect::<Vec<_>>()
        };
        run_metadata["adapter_runs"] = json!(adapter_runs);
        run_metadata["adapter_run_count"] = json!(adapter_runs.len());
        run_metadata["exact_dedup_groups"] = json!(exact_dedup_groups);
        run_metadata["semantic_dedup_groups"] = json!(semantic_dedup_groups);
        run_metadata["source_quality_windows"] = json!(source_quality_windows);
        run_metadata["score_distribution"] = score_distribution;
        self.conn.execute(
            r#"
            UPDATE radar_runs
            SET status = ?2,
                stage = ?3,
                raw_count = ?4,
                normalized_count = ?5,
                indexed_count = ?6,
                scored_count = ?7,
                filtered_count = ?8,
                error = ?9,
                finished_at = ?10,
                updated_at = ?10,
                metadata_json = ?11
            WHERE id = ?1
            "#,
            params![
                run_id,
                status,
                status,
                raw_count,
                items_inserted as i64,
                items_inserted as i64,
                scores_inserted as i64,
                selected_items as i64,
                error,
                now,
                serde_json::to_string(&run_metadata)?
            ],
        )?;

        Ok(RadarFetchReport {
            run: self
                .read_radar_run(&run_id)?
                .with_context(|| format!("radar run not found after insert: {run_id}"))?,
            profile,
            items_inserted,
            scores_inserted,
            selected_items,
            adapter_jobs,
            adapter_runs,
            unsupported_selectors,
            warnings,
        })
    }

    pub(crate) fn classify_radar_projection_proof(
        &self,
        source_cards: &[SourceCard],
        fetch_live: bool,
        live_failed: bool,
    ) -> Result<(String, String, String)> {
        if fetch_live {
            let level = if live_failed {
                "Live Adapter Attempt"
            } else {
                "Production Data Proof"
            };
            return Ok((
                level.to_string(),
                "live_adapter_then_source_card_projection".to_string(),
                "This run invoked Arcwell live source adapters before projecting source cards; enrichment, model synthesis, delivery, and scheduling are not proven by this stage.".to_string(),
            ));
        }
        if !source_cards.is_empty() && self.source_cards_are_healthy_browser_reddit(source_cards)? {
            return Ok((
                "Production Data Proof".to_string(),
                "host_browser_then_source_card_projection".to_string(),
                "This run projects Reddit source cards captured through an authorized host browser session; raw browser storage, comment capture, model synthesis, delivery, and scheduling are not proven by this stage.".to_string(),
            ));
        }
        Ok((
            "Local Fixture Proof".to_string(),
            "source_card_projection".to_string(),
            "This run projects existing Arcwell source cards only; live network adapters, enrichment, summaries, and delivery are not proven by this stage.".to_string(),
        ))
    }

    pub(crate) fn source_cards_are_healthy_browser_reddit(
        &self,
        source_cards: &[SourceCard],
    ) -> Result<bool> {
        let mut source_keys = BTreeSet::new();
        for card in source_cards {
            if card.provider != "reddit" || card.source_type != "reddit_post" {
                return Ok(false);
            }
            if card.metadata.get("transport").and_then(Value::as_str) != Some("host_browser_json") {
                return Ok(false);
            }
            let Some(source_detail) = card.metadata.get("source_detail").and_then(Value::as_str)
            else {
                return Ok(false);
            };
            source_keys.insert(format!("reddit:{source_detail}"));
        }
        if source_keys.is_empty() {
            return Ok(false);
        }
        for source_key in source_keys {
            let Some(health) = self.get_source_health(&source_key)? else {
                return Ok(false);
            };
            if health.status != "healthy" || health.cursor_key.as_deref() != Some(&source_key) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(crate) fn run_radar_live_fetches_for_selectors(
        &self,
        source_selectors: &Value,
    ) -> Result<(Vec<WikiJob>, Vec<String>, bool)> {
        let mut jobs = Vec::new();
        let mut warnings = Vec::new();
        let mut pre_job_failed = false;
        for selector in source_selectors.as_array().cloned().unwrap_or_default() {
            let Some(kind) = radar_selector_kind(&selector) else {
                continue;
            };
            let Some(locator) = radar_selector_locator(&selector) else {
                continue;
            };
            let limit = selector
                .get("limit")
                .and_then(Value::as_u64)
                .unwrap_or(10)
                .clamp(1, 30) as usize;
            let job = match kind.as_str() {
                "source_card_query" => continue,
                "rss" => self.run_rss_fetch_job(&locator),
                "github_owner" => self.run_github_owner_job(&locator, limit),
                "github_release" | "github" => {
                    if let Some((owner, repo)) = parse_github_repo_locator(&locator) {
                        self.run_github_repo_job(&owner, &repo, "releases", limit)
                    } else if kind == "github" {
                        self.run_github_owner_job(&locator, limit)
                    } else {
                        Err(anyhow::anyhow!(
                            "github_release radar selector requires locator owner/repo"
                        ))
                    }
                }
                "arxiv" => self.run_arxiv_search_job(&locator, limit),
                "hackernews" | "hn" => self.run_hackernews_fetch_job(&locator, limit),
                "reddit" => self.run_reddit_fetch_job(&locator, limit),
                "x" | "x_handle" => {
                    let handle = locator.trim().trim_start_matches('@');
                    self.run_x_recent_search_job(&format!("from:{handle}"), limit.max(10))
                }
                _ => continue,
            };
            match job {
                Ok(job) => {
                    if job.status != "completed" {
                        self.record_radar_live_fetch_failure_for_selector(
                            &kind,
                            &locator,
                            job.error.as_deref().unwrap_or("live adapter job failed"),
                        )?;
                        warnings.push(format!(
                            "live adapter job {} ({}) ended with status {}{}",
                            job.id,
                            job.kind,
                            job.status,
                            job.error
                                .as_deref()
                                .map(|error| format!(": {error}"))
                                .unwrap_or_default()
                        ));
                    }
                    jobs.push(job);
                }
                Err(error) => {
                    pre_job_failed = true;
                    self.record_radar_live_fetch_failure_for_selector(
                        &kind,
                        &locator,
                        &error.to_string(),
                    )?;
                    warnings.push(format!(
                        "live adapter {kind}:{locator} failed before job record: {error}"
                    ));
                }
            }
        }
        Ok((jobs, warnings, pre_job_failed))
    }

    pub(crate) fn record_radar_live_fetch_failure_for_selector(
        &self,
        kind: &str,
        locator: &str,
        error: &str,
    ) -> Result<()> {
        match kind {
            "rss" => {
                let key = format!(
                    "rss:{}",
                    canonical_source_url(locator).unwrap_or_else(|_| locator.to_string())
                );
                self.record_source_failure(&key, "rss", "rss", locator, error)?;
            }
            "github_owner" => {
                let key = format!("github-owner:{locator}");
                self.record_source_failure(&key, "github", "github_owner", locator, error)?;
            }
            "github_release" | "github" => {
                if let Some((owner, repo)) = parse_github_repo_locator(locator) {
                    let key = format!("github:{owner}/{repo}:releases");
                    self.record_source_failure(
                        &key,
                        "github",
                        "github_repo",
                        &format!("{owner}/{repo}:releases"),
                        error,
                    )?;
                } else {
                    let key = format!("github-owner:{locator}");
                    self.record_source_failure(&key, "github", "github_owner", locator, error)?;
                }
            }
            "arxiv" => {
                let key = format!("arxiv:{locator}");
                self.record_source_failure(&key, "arxiv", "arxiv_query", locator, error)?;
            }
            "hackernews" | "hn" => {
                let feed =
                    normalize_hackernews_feed(locator).unwrap_or_else(|_| locator.to_string());
                let key = format!("hackernews:{feed}");
                self.record_source_failure(&key, "hackernews", "hackernews", &feed, error)?;
            }
            "reddit" => {
                let locator = normalize_reddit_locator(locator).unwrap_or_else(|_| RedditLocator {
                    subreddit: locator
                        .trim()
                        .trim_start_matches('/')
                        .trim_start_matches("r/")
                        .to_ascii_lowercase(),
                    sort: "hot".to_string(),
                });
                let key = format!("reddit:{}", locator.source_detail());
                self.record_source_failure(
                    &key,
                    "reddit",
                    "reddit",
                    &locator.source_detail(),
                    error,
                )?;
            }
            "x" | "x_handle" => {
                let handle = locator.trim().trim_start_matches('@');
                let query = format!("from:{handle}");
                let key = format!("x:recent-search:{query}");
                self.record_source_failure(&key, "x", "x_recent_search", &query, error)?;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn list_radar_runs(&self) -> Result<Vec<RadarRun>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, profile_id, status, window_start, window_end, stage, source_selection_json,
                   raw_count, normalized_count, indexed_count, scored_count, filtered_count,
                   enriched_count, summary_count, delivery_count, error, metadata_json,
                   started_at, finished_at, updated_at
            FROM radar_runs
            ORDER BY updated_at DESC
            "#,
        )?;
        rows(stmt.query_map([], radar_run_from_row)?)
    }

    pub fn read_radar_run(&self, id: &str) -> Result<Option<RadarRun>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, profile_id, status, window_start, window_end, stage, source_selection_json,
                       raw_count, normalized_count, indexed_count, scored_count, filtered_count,
                       enriched_count, summary_count, delivery_count, error, metadata_json,
                       started_at, finished_at, updated_at
                FROM radar_runs
                WHERE id = ?1
                "#,
                params![id],
                radar_run_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn read_radar_stage(&self, run_id: &str) -> Result<RadarStageReport> {
        let run = self
            .read_radar_run(run_id)?
            .with_context(|| format!("radar run not found: {run_id}"))?;
        Ok(RadarStageReport {
            items: self.list_radar_items(run_id)?,
            scores: self.list_radar_scores(run_id)?,
            dedup_groups: self.list_radar_dedup_groups(run_id)?,
            source_quality: self.list_radar_source_quality(run_id)?,
            run,
        })
    }

    pub fn summarize_radar_run(
        &self,
        run_id: &str,
        language: &str,
        format: &str,
    ) -> Result<RadarSummary> {
        validate_id(run_id)?;
        let language = normalize_radar_summary_language(language)?;
        let format = normalize_radar_summary_format(format)?;
        let run = self
            .read_radar_run(run_id)?
            .with_context(|| format!("radar run not found: {run_id}"))?;
        let profile = self
            .read_radar_profile(&run.profile_id)?
            .with_context(|| format!("radar profile not found: {}", run.profile_id))?;
        let scores = self.list_radar_scores(run_id)?;
        let selected_scores = scores
            .iter()
            .filter(|score| score.status == "selected")
            .cloned()
            .collect::<Vec<_>>();
        if selected_scores.is_empty() {
            bail!("radar summary requires at least one selected score");
        }
        let item_by_id = self
            .list_radar_items(run_id)?
            .into_iter()
            .map(|item| (item.id.clone(), item))
            .collect::<BTreeMap<_, _>>();
        let mut selected = Vec::new();
        for score in selected_scores {
            let item = item_by_id.get(&score.item_id).with_context(|| {
                format!(
                    "selected score references missing radar item: {}",
                    score.item_id
                )
            })?;
            selected.push((score, item.clone()));
        }
        selected.sort_by(|left, right| {
            right
                .0
                .score
                .partial_cmp(&left.0.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.1.title.cmp(&right.1.title))
        });
        let item_ids = selected
            .iter()
            .map(|(_, item)| item.id.clone())
            .collect::<Vec<_>>();
        let source_card_ids = selected
            .iter()
            .filter_map(|(_, item)| item.source_card_id.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let status_counts = radar_score_status_counts(&scores);
        let dedup_groups = self.list_radar_dedup_groups(run_id)?;
        let audit = self.audit_radar_run(run_id)?;
        let existing_summary_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM radar_summaries WHERE run_id = ?1",
            params![run_id],
            |row| row.get(0),
        )?;
        let summary_already_exists: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM radar_summaries WHERE run_id = ?1 AND language = ?2 AND format = ?3",
            params![run_id, &language, &format],
            |row| row.get(0),
        )?;
        let mut render_run = run.clone();
        render_run.summary_count = if summary_already_exists > 0 {
            existing_summary_count
        } else {
            existing_summary_count + 1
        };
        let audit_status = if audit.ok {
            "audit_ok"
        } else {
            "audit_findings"
        }
        .to_string();
        let title = format!("Radar Summary: {}", profile.name);
        let body_markdown = render_radar_summary_markdown(
            &render_run,
            &profile,
            &selected,
            &status_counts,
            dedup_groups.len(),
            &audit,
        );
        let id = format!(
            "radar-summary-{}",
            &sha256(format!("{run_id}\n{language}\n{format}").as_bytes())[..32]
        );
        let created_at = now();
        let semantic_dedup_groups = dedup_groups
            .iter()
            .filter(|group| group.dedup_kind == "semantic_topic")
            .count();
        let metadata = json!({
            "proof_level": "deterministic_local_summary",
            "provenance_boundary": "summary is generated from radar_items and radar_scores; it is not source evidence",
            "selected_items": item_ids.len(),
            "source_cards": source_card_ids.len(),
            "dedup_group_count": dedup_groups.len(),
            "score_status_counts": status_counts,
            "not_delivery": true,
            "not_model_backed": true,
            "semantic_dedupe": if semantic_dedup_groups > 0 { "deterministic_local" } else { "no_groups" },
            "semantic_dedup_group_count": semantic_dedup_groups
        });
        self.conn.execute(
            r#"
            INSERT INTO radar_summaries
              (id, run_id, language, format, title, body_markdown, item_ids_json,
               source_card_ids_json, audit_status, metadata_json, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(run_id, language, format) DO UPDATE SET
              title = excluded.title,
              body_markdown = excluded.body_markdown,
              item_ids_json = excluded.item_ids_json,
              source_card_ids_json = excluded.source_card_ids_json,
              audit_status = excluded.audit_status,
              metadata_json = excluded.metadata_json,
              created_at = excluded.created_at
            "#,
            params![
                id,
                run_id,
                language,
                format,
                title,
                body_markdown,
                serde_json::to_string(&item_ids)?,
                serde_json::to_string(&source_card_ids)?,
                audit_status,
                serde_json::to_string(&metadata)?,
                created_at
            ],
        )?;
        let summary_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM radar_summaries WHERE run_id = ?1",
            params![run_id],
            |row| row.get(0),
        )?;
        let updated_at = now();
        self.conn.execute(
            r#"
            UPDATE radar_runs
            SET status = CASE
                    WHEN status IN ('scored', 'summarized') THEN 'summarized'
                    ELSE status
                END,
                stage = CASE
                    WHEN stage IN ('scored', 'summarized') THEN 'summarized'
                    ELSE stage
                END,
                summary_count = ?2,
                updated_at = ?3
            WHERE id = ?1
            "#,
            params![run_id, summary_count, updated_at],
        )?;
        self.read_radar_summary(run_id, &language, &format)?
            .with_context(|| format!("radar summary not found after write: {run_id}"))
    }

    pub fn read_radar_summary(
        &self,
        run_id: &str,
        language: &str,
        format: &str,
    ) -> Result<Option<RadarSummary>> {
        validate_id(run_id)?;
        let language = normalize_radar_summary_language(language)?;
        let format = normalize_radar_summary_format(format)?;
        self.conn
            .query_row(
                r#"
                SELECT id, run_id, language, format, title, body_markdown,
                       item_ids_json, source_card_ids_json, audit_status,
                       metadata_json, created_at
                FROM radar_summaries
                WHERE run_id = ?1 AND language = ?2 AND format = ?3
                "#,
                params![run_id, language, format],
                radar_summary_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn deliver_radar_summary(&self, input: RadarDeliveryInput) -> Result<RadarDeliveryReport> {
        validate_id(&input.run_id)?;
        let language = normalize_radar_summary_language(&input.language)?;
        let format = normalize_radar_summary_format(&input.format)?;
        let channel = normalize_radar_delivery_channel(&input.channel)?;
        let recipient_ref = normalize_radar_delivery_recipient(&channel, &input.recipient_ref)?;
        let summary = self
            .read_radar_summary(&input.run_id, &language, &format)?
            .with_context(|| format!("radar summary not found for run {}", input.run_id))?;
        if summary.audit_status != "audit_ok" {
            bail!(
                "radar delivery requires audit_ok summary; found {}",
                summary.audit_status
            );
        }
        let idempotency_key = normalize_radar_delivery_idempotency_key(
            input.idempotency_key.as_deref(),
            &summary,
            &channel,
            &recipient_ref,
        )?;
        let delivery =
            if let Some(existing) = self.get_radar_delivery_by_idempotency_key(&idempotency_key)? {
                if matches!(existing.status.as_str(), "sent" | "pending" | "deferred") {
                    return self.radar_delivery_report(existing, summary, true);
                }
                existing
            } else {
                self.insert_radar_delivery_start(
                    &input.run_id,
                    &summary.id,
                    &channel,
                    &recipient_ref,
                    &idempotency_key,
                )?
            };
        let mut cost_decision_id: Option<String> = None;
        let send_result = (|| -> Result<(ChannelMessage, ChannelDeliveryAttempt)> {
            match channel.as_str() {
                "telegram" => {
                    if !self.channel_subject_can_send("telegram", &recipient_ref)? {
                        bail!("telegram subject is not authorized to send: {recipient_ref}");
                    }
                    let bot_token = input
                        .telegram_bot_token
                        .as_deref()
                        .context("telegram radar delivery requires telegram_bot_token")?;
                    let chat_id = recipient_ref
                        .strip_prefix("telegram:chat:")
                        .unwrap_or(&recipient_ref);
                    self.policy_guard(PolicyRequest {
                        action: "channel.send".to_string(),
                        package: None,
                        provider: Some("telegram".to_string()),
                        source: Some("telegram_send".to_string()),
                        channel: Some("telegram".to_string()),
                        subject: Some(recipient_ref.clone()),
                        target: Some(chat_id.to_string()),
                        projected_usd: None,
                        metadata: json!({
                            "radar_delivery_id": delivery.id,
                            "run_id": input.run_id,
                            "parse_mode": "MarkdownV2"
                        }),
                        untrusted_excerpt: Some(summary.body_markdown.clone()),
                    })?;
                    let cost = self.reserve_cost_budget(
                        "arcwell-telegram",
                        &delivery.id,
                        "telegram",
                        "send_message",
                        Some("telegram_send"),
                        estimated_channel_send_cost(),
                    )?;
                    cost_decision_id = cost.decision_id.clone();
                    if !cost.allowed {
                        bail!("budget blocked Telegram radar delivery: {}", cost.reason);
                    }
                    self.send_telegram_message_preflighted(
                        bot_token,
                        chat_id,
                        &summary.body_markdown,
                        input.api_base.as_deref(),
                    )
                    .map(|report| (report.message, report.delivery))
                }
                "email" => {
                    if !self.channel_subject_can_send("email", &recipient_ref)? {
                        bail!("email subject is not authorized to send: {recipient_ref}");
                    }
                    let account_id = input
                        .email_account_id
                        .as_deref()
                        .context("email radar delivery requires email_account_id")?;
                    let api_token = input
                        .email_api_token
                        .as_deref()
                        .context("email radar delivery requires email_api_token")?;
                    let from = input
                        .email_from
                        .as_deref()
                        .context("email radar delivery requires email_from")?;
                    let to = recipient_ref
                        .strip_prefix("email:")
                        .unwrap_or(&recipient_ref);
                    self.policy_guard(PolicyRequest {
                        action: "channel.send".to_string(),
                        package: Some("arcwell-email".to_string()),
                        provider: Some("cloudflare_email".to_string()),
                        source: Some("email_send".to_string()),
                        channel: Some("email".to_string()),
                        subject: Some(recipient_ref.clone()),
                        target: Some(to.to_string()),
                        projected_usd: None,
                        metadata: json!({
                            "from": from,
                            "radar_delivery_id": delivery.id,
                            "run_id": input.run_id,
                            "rich_html": false
                        }),
                        untrusted_excerpt: Some(format!(
                            "{}\n\n{}",
                            summary.title, summary.body_markdown
                        )),
                    })?;
                    let cost = self.reserve_cost_budget(
                        "arcwell-email",
                        &delivery.id,
                        "cloudflare_email",
                        "send",
                        Some("email_send"),
                        estimated_channel_send_cost(),
                    )?;
                    cost_decision_id = cost.decision_id.clone();
                    if !cost.allowed {
                        bail!(
                            "budget blocked Cloudflare Email radar delivery: {}",
                            cost.reason
                        );
                    }
                    self.send_cloudflare_email_preflighted(
                        account_id,
                        api_token,
                        from,
                        to,
                        &summary.title,
                        &summary.body_markdown,
                        None,
                        None,
                        input.api_base.as_deref(),
                    )
                    .map(|report| (report.message, report.delivery))
                }
                other => bail!("unsupported radar delivery channel: {other}"),
            }
        })();

        match send_result {
            Ok((message, attempt)) => {
                let status = if attempt.ok { "sent" } else { "failed" };
                let error =
                    if attempt.ok {
                        None
                    } else {
                        Some(attempt.error.clone().unwrap_or_else(|| {
                            format!("provider status {}", attempt.provider_status)
                        }))
                    };
                let delivery = self.update_radar_delivery_result(
                    &delivery.id,
                    status,
                    Some(&attempt.id),
                    cost_decision_id.as_deref(),
                    error.as_deref(),
                )?;
                self.refresh_radar_delivery_count(&input.run_id, attempt.ok)?;
                Ok(RadarDeliveryReport {
                    delivery,
                    summary,
                    channel_message: Some(message),
                    channel_delivery_attempt: Some(attempt),
                    idempotent_replay: false,
                })
            }
            Err(error) => {
                let error_text = sanitize_radar_delivery_error(&error.to_string())?;
                let delivery = self.update_radar_delivery_result(
                    &delivery.id,
                    "blocked",
                    None,
                    cost_decision_id.as_deref(),
                    Some(&error_text),
                )?;
                self.refresh_radar_delivery_count(&input.run_id, false)?;
                Ok(RadarDeliveryReport {
                    delivery,
                    summary,
                    channel_message: None,
                    channel_delivery_attempt: None,
                    idempotent_replay: false,
                })
            }
        }
    }

    pub fn list_radar_deliveries(&self, run_id: Option<&str>) -> Result<Vec<RadarDelivery>> {
        if let Some(run_id) = run_id {
            validate_id(run_id)?;
            let mut stmt = self.conn.prepare(
                r#"
                SELECT id, run_id, summary_id, channel, recipient_ref, status,
                       policy_decision_id, cost_decision_id, delivery_attempt_id,
                       quiet_hours_deferred_until, idempotency_key, error, created_at, updated_at
                FROM radar_deliveries
                WHERE run_id = ?1
                ORDER BY updated_at DESC
                "#,
            )?;
            return rows(stmt.query_map(params![run_id], radar_delivery_from_row)?);
        }
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, run_id, summary_id, channel, recipient_ref, status,
                   policy_decision_id, cost_decision_id, delivery_attempt_id,
                   quiet_hours_deferred_until, idempotency_key, error, created_at, updated_at
            FROM radar_deliveries
            ORDER BY updated_at DESC
            "#,
        )?;
        rows(stmt.query_map([], radar_delivery_from_row)?)
    }

    pub fn list_radar_schedule_ticks(&self) -> Result<Vec<RadarScheduleTick>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, profile_id, tick_key, due_at, status, job_id, run_id,
                   summary_id, delivery_id, error, created_at, updated_at
            FROM radar_schedule_ticks
            ORDER BY updated_at DESC
            "#,
        )?;
        rows(stmt.query_map([], radar_schedule_tick_from_row)?)
    }

    pub fn reconcile_radar_delivery_attempts(
        &self,
        max_attempts_per_message: i64,
    ) -> Result<RadarDeliveryReconcileReport> {
        let max_attempts_per_message = max_attempts_per_message.clamp(1, 20);
        let due = {
            let mut stmt = self.conn.prepare(
                r#"
                SELECT rd.id,
                       rd.run_id,
                       latest.id,
                       latest.message_id,
                       latest.ok,
                       latest.attempt,
                       latest.provider_status,
                       latest.error
                FROM radar_deliveries rd
                JOIN channel_delivery_attempts linked
                  ON linked.id = rd.delivery_attempt_id
                JOIN channel_delivery_attempts latest
                  ON latest.message_id = linked.message_id
                WHERE rd.status IN ('pending', 'failed')
                  AND latest.attempt = (
                    SELECT MAX(d2.attempt)
                    FROM channel_delivery_attempts d2
                    WHERE d2.message_id = linked.message_id
                  )
                  AND (
                    rd.delivery_attempt_id != latest.id
                    OR latest.ok = 1
                    OR latest.attempt >= ?1
                  )
                ORDER BY rd.updated_at ASC, rd.id ASC
                "#,
            )?;
            rows(stmt.query_map(params![max_attempts_per_message], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)? != 0,
                    row.get::<_, i64>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, Option<String>>(7)?,
                ))
            })?)?
        };

        let mut report = RadarDeliveryReconcileReport {
            inspected: due.len(),
            sent: 0,
            failed: 0,
            dead_lettered: 0,
            updated: Vec::new(),
        };
        for (
            delivery_id,
            run_id,
            latest_attempt_id,
            message_id,
            ok,
            attempt,
            provider_status,
            error,
        ) in due
        {
            let (status, error_text) = if ok {
                report.sent += 1;
                ("sent", None)
            } else if attempt >= max_attempts_per_message {
                report.dead_lettered += 1;
                self.update_channel_message_status(&message_id, "dead_lettered")?;
                (
                    "dead_lettered",
                    Some(sanitize_radar_delivery_error(&format!(
                        "delivery retry exhausted after {attempt} attempt(s): {}",
                        error.unwrap_or_else(|| format!("provider status {provider_status}"))
                    ))?),
                )
            } else {
                report.failed += 1;
                (
                    "failed",
                    Some(sanitize_radar_delivery_error(&error.unwrap_or_else(
                        || format!("provider status {provider_status}"),
                    ))?),
                )
            };
            let delivery = self.update_radar_delivery_result(
                &delivery_id,
                status,
                Some(&latest_attempt_id),
                None,
                error_text.as_deref(),
            )?;
            self.update_radar_schedule_ticks_for_delivery(
                &delivery_id,
                status,
                error_text.as_deref(),
            )?;
            self.refresh_radar_delivery_count(&run_id, ok)?;
            report.updated.push(delivery);
        }
        Ok(report)
    }

    pub(crate) fn get_radar_delivery_by_idempotency_key(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<RadarDelivery>> {
        validate_query(idempotency_key)?;
        self.conn
            .query_row(
                r#"
                SELECT id, run_id, summary_id, channel, recipient_ref, status,
                       policy_decision_id, cost_decision_id, delivery_attempt_id,
                       quiet_hours_deferred_until, idempotency_key, error, created_at, updated_at
                FROM radar_deliveries
                WHERE idempotency_key = ?1
                "#,
                params![idempotency_key],
                radar_delivery_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub(crate) fn get_radar_delivery(&self, id: &str) -> Result<Option<RadarDelivery>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, run_id, summary_id, channel, recipient_ref, status,
                       policy_decision_id, cost_decision_id, delivery_attempt_id,
                       quiet_hours_deferred_until, idempotency_key, error, created_at, updated_at
                FROM radar_deliveries
                WHERE id = ?1
                "#,
                params![id],
                radar_delivery_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub(crate) fn insert_radar_delivery_start(
        &self,
        run_id: &str,
        summary_id: &str,
        channel: &str,
        recipient_ref: &str,
        idempotency_key: &str,
    ) -> Result<RadarDelivery> {
        let id = Uuid::new_v4().to_string();
        let timestamp = now();
        self.conn.execute(
            r#"
            INSERT INTO radar_deliveries
              (id, run_id, summary_id, channel, recipient_ref, status,
               policy_decision_id, cost_decision_id, delivery_attempt_id,
               quiet_hours_deferred_until, idempotency_key, error, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, 'pending', NULL, NULL, NULL, NULL, ?6, NULL, ?7, ?7)
            "#,
            params![
                id,
                run_id,
                summary_id,
                channel,
                recipient_ref,
                idempotency_key,
                timestamp
            ],
        )?;
        self.get_radar_delivery(&id)?
            .with_context(|| format!("inserted radar delivery not found: {id}"))
    }

    pub(crate) fn update_radar_delivery_result(
        &self,
        id: &str,
        status: &str,
        delivery_attempt_id: Option<&str>,
        cost_decision_id: Option<&str>,
        error: Option<&str>,
    ) -> Result<RadarDelivery> {
        validate_id(id)?;
        validate_radar_delivery_status(status)?;
        if let Some(delivery_attempt_id) = delivery_attempt_id {
            validate_id(delivery_attempt_id)?;
        }
        if let Some(cost_decision_id) = cost_decision_id {
            validate_id(cost_decision_id)?;
        }
        if let Some(error) = error {
            validate_notes(error)?;
        }
        let updated_at = now();
        self.conn.execute(
            r#"
            UPDATE radar_deliveries
            SET status = ?2,
                delivery_attempt_id = ?3,
                cost_decision_id = COALESCE(?4, cost_decision_id),
                error = ?5,
                updated_at = ?6
            WHERE id = ?1
            "#,
            params![
                id,
                status,
                delivery_attempt_id,
                cost_decision_id,
                error,
                updated_at
            ],
        )?;
        self.get_radar_delivery(id)?
            .with_context(|| format!("radar delivery not found after update: {id}"))
    }

    pub(crate) fn refresh_radar_delivery_count(&self, run_id: &str, delivered: bool) -> Result<()> {
        validate_id(run_id)?;
        let delivery_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM radar_deliveries WHERE run_id = ?1",
            params![run_id],
            |row| row.get(0),
        )?;
        let updated_at = now();
        self.conn.execute(
            r#"
            UPDATE radar_runs
            SET delivery_count = ?2,
                status = CASE
                    WHEN ?3 = 1 AND status IN ('scored', 'summarized', 'delivered') THEN 'delivered'
                    ELSE status
                END,
                stage = CASE
                    WHEN ?3 = 1 AND stage IN ('scored', 'summarized', 'delivered') THEN 'delivered'
                    ELSE stage
                END,
                updated_at = ?4
            WHERE id = ?1
            "#,
            params![run_id, delivery_count, bool_to_i64(delivered), updated_at],
        )?;
        Ok(())
    }

    pub(crate) fn radar_delivery_report(
        &self,
        delivery: RadarDelivery,
        summary: RadarSummary,
        idempotent_replay: bool,
    ) -> Result<RadarDeliveryReport> {
        let channel_delivery_attempt = delivery
            .delivery_attempt_id
            .as_deref()
            .map(|id| {
                self.get_channel_delivery_attempt(id)?
                    .with_context(|| format!("channel delivery attempt not found: {id}"))
            })
            .transpose()?;
        let channel_message = channel_delivery_attempt
            .as_ref()
            .map(|attempt| {
                self.get_channel_message(&attempt.message_id)?
                    .with_context(|| format!("channel message not found: {}", attempt.message_id))
            })
            .transpose()?;
        Ok(RadarDeliveryReport {
            delivery,
            summary,
            channel_message,
            channel_delivery_attempt,
            idempotent_replay,
        })
    }

    pub fn list_radar_items(&self, run_id: &str) -> Result<Vec<RadarItem>> {
        validate_id(run_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, run_id, stable_key, source_kind, provider, source_locator, native_id,
                   canonical_url, title, author, published_at, fetched_at, content_text,
                   content_sha256, metadata_json, source_card_id, wiki_page_id,
                   canonical_entity_ref, trust_level, created_at, updated_at
            FROM radar_items
            WHERE run_id = ?1
            ORDER BY updated_at DESC, title ASC
            "#,
        )?;
        rows(stmt.query_map(params![run_id], radar_item_from_row)?)
    }

    pub fn list_radar_scores(&self, run_id: &str) -> Result<Vec<RadarScore>> {
        validate_id(run_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, run_id, item_id, score_kind, score, reason, tags_json,
                   model_provider, model_name, cost_decision_id, input_artifact_id,
                   output_artifact_id, schema_version, status, error, created_at
            FROM radar_scores
            WHERE run_id = ?1
            ORDER BY score DESC, created_at DESC
            "#,
        )?;
        rows(stmt.query_map(params![run_id], radar_score_from_row)?)
    }

    pub fn list_radar_dedup_groups(&self, run_id: &str) -> Result<Vec<RadarDedupGroup>> {
        validate_id(run_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, run_id, dedup_kind, primary_item_id, member_item_ids_json,
                   reason, confidence, model_provider, cost_decision_id, created_at
            FROM radar_dedup_groups
            WHERE run_id = ?1
            ORDER BY created_at ASC, dedup_kind ASC, id ASC
            "#,
        )?;
        rows(stmt.query_map(params![run_id], radar_dedup_group_from_row)?)
    }

    pub fn list_radar_source_quality(&self, run_id: &str) -> Result<Vec<RadarSourceQuality>> {
        validate_id(run_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, run_id, source_kind, locator, window_start, window_end, raw_count,
                   accepted_count, average_score, score_p50, score_p90, signal_to_noise,
                   duplicate_rate, delivery_contribution_count, failure_count, status, created_at
            FROM radar_source_quality
            WHERE run_id = ?1
            ORDER BY created_at DESC, source_kind, locator
            "#,
        )?;
        rows(stmt.query_map(params![run_id], radar_source_quality_from_row)?)
    }

    pub fn list_all_radar_source_quality(&self) -> Result<Vec<RadarSourceQuality>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, run_id, source_kind, locator, window_start, window_end, raw_count,
                   accepted_count, average_score, score_p50, score_p90, signal_to_noise,
                   duplicate_rate, delivery_contribution_count, failure_count, status, created_at
            FROM radar_source_quality
            ORDER BY created_at DESC, source_kind, locator
            "#,
        )?;
        rows(stmt.query_map([], radar_source_quality_from_row)?)
    }

    pub fn list_radar_source_quality_trends(
        &self,
        min_windows: usize,
        limit: usize,
    ) -> Result<Vec<RadarSourceQualityTrend>> {
        if !(1..=10_000).contains(&min_windows) {
            bail!("min_windows must be between 1 and 10000");
        }
        if !(1..=500).contains(&limit) {
            bail!("limit must be between 1 and 500");
        }
        let mut grouped: BTreeMap<(String, String), Vec<RadarSourceQuality>> = BTreeMap::new();
        for row in self.list_all_radar_source_quality()? {
            grouped
                .entry((row.source_kind.clone(), row.locator.clone()))
                .or_default()
                .push(row);
        }
        let mut trends = Vec::new();
        for ((source_kind, locator), mut rows) in grouped {
            if rows.len() < min_windows {
                continue;
            }
            rows.sort_by(|left, right| {
                left.window_end
                    .cmp(&right.window_end)
                    .then(left.created_at.cmp(&right.created_at))
                    .then(left.run_id.cmp(&right.run_id))
            });
            trends.push(radar_source_quality_trend_from_rows(
                source_kind,
                locator,
                &rows,
            )?);
        }
        trends.sort_by(|left, right| {
            right
                .quality_score
                .partial_cmp(&left.quality_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(right.last_window_end.cmp(&left.last_window_end))
                .then(left.source_kind.cmp(&right.source_kind))
                .then(left.locator.cmp(&right.locator))
        });
        trends.truncate(limit);
        Ok(trends)
    }

    pub fn rebuild_radar_fts(&self, run_id: Option<&str>) -> Result<usize> {
        if let Some(run_id) = run_id {
            validate_id(run_id)?;
            self.conn.execute(
                "DELETE FROM radar_item_fts WHERE id IN (SELECT id FROM radar_items WHERE run_id = ?1)",
                params![run_id],
            )?;
            let mut stmt = self.conn.prepare(
                "SELECT id, title, content_text, COALESCE(author, ''), source_kind FROM radar_items WHERE run_id = ?1",
            )?;
            let rows = rows(stmt.query_map(params![run_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })?)?;
            for (id, title, content_text, author, source_kind) in &rows {
                self.conn.execute(
                    "INSERT INTO radar_item_fts (id, title, content_text, author, source_kind) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![id, title, content_text, author, source_kind],
                )?;
            }
            Ok(rows.len())
        } else {
            self.conn.execute("DELETE FROM radar_item_fts", [])?;
            let mut stmt = self.conn.prepare(
                "SELECT id, title, content_text, COALESCE(author, ''), source_kind FROM radar_items",
            )?;
            let rows = rows(stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })?)?;
            for (id, title, content_text, author, source_kind) in &rows {
                self.conn.execute(
                    "INSERT INTO radar_item_fts (id, title, content_text, author, source_kind) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![id, title, content_text, author, source_kind],
                )?;
            }
            Ok(rows.len())
        }
    }

    pub fn audit_radar_run(&self, run_id: &str) -> Result<RadarAuditReport> {
        let run = self
            .read_radar_run(run_id)?
            .with_context(|| format!("radar run not found: {run_id}"))?;
        let item_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM radar_items WHERE run_id = ?1",
            params![run_id],
            |row| row.get(0),
        )?;
        let fts_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM radar_item_fts WHERE id IN (SELECT id FROM radar_items WHERE run_id = ?1)",
            params![run_id],
            |row| row.get(0),
        )?;
        let scored_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM radar_scores WHERE run_id = ?1",
            params![run_id],
            |row| row.get(0),
        )?;
        let dedup_group_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM radar_dedup_groups WHERE run_id = ?1",
            params![run_id],
            |row| row.get(0),
        )?;
        let source_quality_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM radar_source_quality WHERE run_id = ?1",
            params![run_id],
            |row| row.get(0),
        )?;
        let duplicate_score_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM radar_scores WHERE run_id = ?1 AND status LIKE 'duplicate_%'",
            params![run_id],
            |row| row.get(0),
        )?;
        let missing_source_cards: i64 = self.conn.query_row(
            r#"
            SELECT COUNT(*)
            FROM radar_items item
            LEFT JOIN source_cards card ON card.id = item.source_card_id
            WHERE item.run_id = ?1 AND item.source_card_id IS NOT NULL AND card.id IS NULL
            "#,
            params![run_id],
            |row| row.get(0),
        )?;
        let radar_items = self.list_radar_items(run_id)?;
        let radar_scores = self.list_radar_scores(run_id)?;
        let mut findings = Vec::new();
        if item_count != fts_count {
            findings.push(RadarAuditFinding {
                severity: "high".to_string(),
                code: "radar_fts_drift".to_string(),
                message: "Radar item FTS rows do not match normalized radar items.".to_string(),
                evidence: format!("items={item_count} fts_rows={fts_count}"),
            });
        }
        if item_count > scored_count {
            findings.push(RadarAuditFinding {
                severity: "high".to_string(),
                code: "radar_unscored_items".to_string(),
                message: "One or more radar items have no score overlay.".to_string(),
                evidence: format!("items={item_count} scores={scored_count}"),
            });
        }
        if missing_source_cards > 0 {
            findings.push(RadarAuditFinding {
                severity: "critical".to_string(),
                code: "radar_missing_source_card".to_string(),
                message: "Radar item provenance points at missing source cards.".to_string(),
                evidence: format!("missing_source_cards={missing_source_cards}"),
            });
        }
        if duplicate_score_count > 0 && dedup_group_count == 0 {
            findings.push(RadarAuditFinding {
                severity: "high".to_string(),
                code: "radar_duplicate_scores_without_groups".to_string(),
                message: "Radar scores mark duplicate items without auditable dedupe groups."
                    .to_string(),
                evidence: format!("duplicate_scores={duplicate_score_count} dedup_groups=0"),
            });
        }
        if item_count > 0 && source_quality_count == 0 {
            findings.push(RadarAuditFinding {
                severity: "high".to_string(),
                code: "radar_source_quality_missing".to_string(),
                message: "Radar run has items and scores but no source-quality window.".to_string(),
                evidence: format!(
                    "window_start={} window_end={} items={item_count} scores={scored_count}",
                    run.window_start, run.window_end
                ),
            });
        }
        let expected_source_quality =
            expected_radar_source_quality_counts(&radar_items, &radar_scores);
        let source_quality_rows = self.list_radar_source_quality(run_id)?;
        if !expected_source_quality.is_empty()
            && expected_source_quality.len() != source_quality_rows.len()
        {
            findings.push(RadarAuditFinding {
                severity: "high".to_string(),
                code: "radar_source_quality_drift".to_string(),
                message: "Radar source-quality row count does not match scored source buckets."
                    .to_string(),
                evidence: format!(
                    "expected_buckets={} source_quality_rows={}",
                    expected_source_quality.len(),
                    source_quality_rows.len()
                ),
            });
        }
        let source_quality_by_key = source_quality_rows
            .iter()
            .map(|row| ((row.source_kind.clone(), row.locator.clone()), row))
            .collect::<BTreeMap<_, _>>();
        for (key, expected) in &expected_source_quality {
            match source_quality_by_key.get(key) {
                Some(row) => {
                    let signal_to_noise = row.signal_to_noise.unwrap_or(-1.0);
                    let duplicate_rate = row.duplicate_rate.unwrap_or(-1.0);
                    let expected_signal_to_noise =
                        expected.accepted_count as f64 / expected.raw_count as f64;
                    let expected_duplicate_rate =
                        expected.duplicate_count as f64 / expected.raw_count as f64;
                    if row.raw_count != expected.raw_count
                        || row.accepted_count != expected.accepted_count
                        || row.raw_count < row.accepted_count
                        || !(0.0..=1.0).contains(&signal_to_noise)
                        || !(0.0..=1.0).contains(&duplicate_rate)
                        || (signal_to_noise - expected_signal_to_noise).abs() > 0.000001
                        || (duplicate_rate - expected_duplicate_rate).abs() > 0.000001
                    {
                        findings.push(RadarAuditFinding {
                            severity: "high".to_string(),
                            code: "radar_source_quality_drift".to_string(),
                            message:
                                "Radar source-quality metrics do not match scored radar items."
                                    .to_string(),
                            evidence: format!(
                                "source_kind={} locator={} expected_raw={} actual_raw={} expected_accepted={} actual_accepted={} expected_signal_to_noise={} signal_to_noise={} expected_duplicate_rate={} duplicate_rate={}",
                                key.0,
                                key.1,
                                expected.raw_count,
                                row.raw_count,
                                expected.accepted_count,
                                row.accepted_count,
                                expected_signal_to_noise,
                                signal_to_noise,
                                expected_duplicate_rate,
                                duplicate_rate
                            ),
                        });
                    }
                }
                None => findings.push(RadarAuditFinding {
                    severity: "high".to_string(),
                    code: "radar_source_quality_drift".to_string(),
                    message: "Radar source-quality is missing a scored source bucket.".to_string(),
                    evidence: format!(
                        "source_kind={} locator={} expected_raw={}",
                        key.0, key.1, expected.raw_count
                    ),
                }),
            }
        }
        if run
            .metadata
            .get("live_fetch_failed")
            .and_then(Value::as_bool)
            == Some(true)
        {
            let severity = if item_count == 0 { "high" } else { "medium" };
            findings.push(RadarAuditFinding {
                severity: severity.to_string(),
                code: "radar_live_fetch_failed".to_string(),
                message: "One or more opt-in live radar adapters failed before or during fetch."
                    .to_string(),
                evidence: serde_json::to_string(&json!({
                    "status": run.status,
                    "items": item_count,
                    "adapter_job_count": run.metadata.get("adapter_job_count"),
                    "warnings": run.metadata.get("live_fetch_warnings"),
                    "pre_job_failed": run.metadata.get("live_fetch_pre_job_failed")
                }))?,
            });
        }
        let item_ids = radar_items
            .iter()
            .map(|item| item.id.clone())
            .collect::<BTreeSet<_>>();
        let score_by_item = radar_scores
            .iter()
            .map(|score| (score.item_id.clone(), score))
            .collect::<BTreeMap<_, _>>();
        let mut expected_duplicate_status_by_item: BTreeMap<String, String> = BTreeMap::new();
        for group in self.list_radar_dedup_groups(run_id)? {
            let expected_status = match group.dedup_kind.as_str() {
                "canonical_url" => Some("duplicate_url"),
                "same_native_id" => Some("duplicate_native_id"),
                "semantic_topic" => Some("duplicate_topic"),
                _ => Some("duplicate_exact"),
            };
            if group.member_item_ids.len() < 2 {
                findings.push(RadarAuditFinding {
                    severity: "high".to_string(),
                    code: "radar_dedup_group_too_small".to_string(),
                    message: "A radar dedupe group has fewer than two members.".to_string(),
                    evidence: format!(
                        "group_id={} member_count={}",
                        group.id,
                        group.member_item_ids.len()
                    ),
                });
            }
            if !group.member_item_ids.contains(&group.primary_item_id) {
                findings.push(RadarAuditFinding {
                    severity: "high".to_string(),
                    code: "radar_dedup_primary_missing_from_group".to_string(),
                    message: "A radar dedupe group primary is not listed as a member.".to_string(),
                    evidence: format!("group_id={} primary={}", group.id, group.primary_item_id),
                });
            }
            for member_id in &group.member_item_ids {
                if !item_ids.contains(member_id) {
                    findings.push(RadarAuditFinding {
                        severity: "high".to_string(),
                        code: "radar_dedup_missing_member".to_string(),
                        message: "A radar dedupe group references a missing radar item."
                            .to_string(),
                        evidence: format!("group_id={} member_id={member_id}", group.id),
                    });
                }
                if member_id != &group.primary_item_id
                    && let Some(status) = expected_status
                {
                    expected_duplicate_status_by_item
                        .entry(member_id.clone())
                        .or_insert_with(|| status.to_string());
                    match score_by_item.get(member_id) {
                        Some(score) if score.status == status => {}
                        Some(score) => findings.push(RadarAuditFinding {
                            severity: "high".to_string(),
                            code: "radar_dedup_score_drift".to_string(),
                            message: "Radar score status does not match dedupe group membership."
                                .to_string(),
                            evidence: format!(
                                "group_id={} member_id={} expected_status={} actual_status={}",
                                group.id, member_id, status, score.status
                            ),
                        }),
                        None => findings.push(RadarAuditFinding {
                            severity: "high".to_string(),
                            code: "radar_dedup_score_missing".to_string(),
                            message: "A non-primary dedupe member has no radar score.".to_string(),
                            evidence: format!(
                                "group_id={} member_id={} expected_status={}",
                                group.id, member_id, status
                            ),
                        }),
                    }
                }
            }
        }
        for score in &radar_scores {
            if score.status.starts_with("duplicate_")
                && expected_duplicate_status_by_item.get(&score.item_id) != Some(&score.status)
            {
                findings.push(RadarAuditFinding {
                    severity: "high".to_string(),
                    code: "radar_duplicate_score_without_matching_group".to_string(),
                    message:
                        "Radar score marks an item duplicate without matching dedupe membership."
                            .to_string(),
                    evidence: format!("item_id={} status={}", score.item_id, score.status),
                });
            }
        }
        let unsupported = unsupported_radar_selectors(&run.source_selection);
        if !unsupported.is_empty() {
            findings.push(RadarAuditFinding {
                severity: "medium".to_string(),
                code: "radar_unsupported_selectors".to_string(),
                message: "The run skipped selectors that are not implemented yet.".to_string(),
                evidence: serde_json::to_string(&unsupported)?,
            });
        }
        if run.status == "empty" || run.status == "blocked" {
            findings.push(RadarAuditFinding {
                severity: "medium".to_string(),
                code: "radar_no_items".to_string(),
                message: "The run did not produce normalized radar items.".to_string(),
                evidence: format!("status={} raw_count={}", run.status, run.raw_count),
            });
        }
        Ok(RadarAuditReport {
            run_id: run.id,
            checked_at: now(),
            ok: findings
                .iter()
                .all(|finding| !matches!(finding.severity.as_str(), "critical" | "high")),
            item_count,
            fts_count,
            scored_count,
            dedup_group_count,
            source_quality_count,
            findings,
        })
    }

    pub(crate) fn insert_radar_item(&self, item: &RadarItem) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO radar_items
              (id, run_id, stable_key, source_kind, provider, source_locator, native_id,
               canonical_url, title, author, published_at, fetched_at, content_text,
               content_sha256, metadata_json, source_card_id, wiki_page_id,
               canonical_entity_ref, trust_level, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
                    ?15, ?16, ?17, ?18, ?19, ?20, ?21)
            ON CONFLICT(run_id, stable_key) DO UPDATE SET
              title = excluded.title,
              content_text = excluded.content_text,
              content_sha256 = excluded.content_sha256,
              metadata_json = excluded.metadata_json,
              source_card_id = excluded.source_card_id,
              wiki_page_id = excluded.wiki_page_id,
              updated_at = excluded.updated_at
            "#,
            params![
                item.id,
                item.run_id,
                item.stable_key,
                item.source_kind,
                item.provider,
                item.source_locator,
                item.native_id,
                item.canonical_url,
                item.title,
                item.author,
                item.published_at,
                item.fetched_at,
                item.content_text,
                item.content_sha256,
                serde_json::to_string(&item.metadata)?,
                item.source_card_id,
                item.wiki_page_id,
                item.canonical_entity_ref,
                item.trust_level,
                item.created_at,
                item.updated_at
            ],
        )?;
        Ok(())
    }

    pub(crate) fn dedupe_radar_run(&self, run_id: &str) -> Result<usize> {
        validate_id(run_id)?;
        self.conn.execute(
            "DELETE FROM radar_dedup_groups WHERE run_id = ?1",
            params![run_id],
        )?;
        let items = self.list_radar_items(run_id)?;
        let mut grouped_item_ids = BTreeSet::new();
        let mut inserted = 0usize;

        let mut url_buckets: BTreeMap<String, Vec<RadarItem>> = BTreeMap::new();
        for item in &items {
            if let Some(url) = item.canonical_url.as_deref().and_then(radar_exact_url_key) {
                url_buckets.entry(url).or_default().push(item.clone());
            }
        }
        for (url, members) in url_buckets {
            if members.len() < 2 {
                continue;
            }
            let group = radar_exact_dedup_group(
                run_id,
                "canonical_url",
                &format!("same canonical URL: {url}"),
                members,
            )?;
            for member_id in &group.member_item_ids {
                grouped_item_ids.insert(member_id.clone());
            }
            self.insert_radar_dedup_group(&group)?;
            inserted += 1;
        }

        let mut native_buckets: BTreeMap<String, Vec<RadarItem>> = BTreeMap::new();
        for item in &items {
            if grouped_item_ids.contains(&item.id) {
                continue;
            }
            let Some(native_id) = item.native_id.as_deref() else {
                continue;
            };
            let native_id = native_id.trim();
            if native_id.is_empty() {
                continue;
            }
            native_buckets
                .entry(format!(
                    "{}\u{001f}{}\u{001f}{}",
                    item.source_kind,
                    item.provider,
                    native_id.to_ascii_lowercase()
                ))
                .or_default()
                .push(item.clone());
        }
        for (_key, members) in native_buckets {
            if members.len() < 2 {
                continue;
            }
            let group = radar_exact_dedup_group(
                run_id,
                "same_native_id",
                "same source/provider/native id",
                members,
            )?;
            self.insert_radar_dedup_group(&group)?;
            inserted += 1;
        }

        Ok(inserted)
    }

    pub(crate) fn dedupe_radar_run_semantic_topics(&self, run_id: &str) -> Result<usize> {
        validate_id(run_id)?;
        self.conn.execute(
            "DELETE FROM radar_dedup_groups WHERE run_id = ?1 AND dedup_kind = 'semantic_topic'",
            params![run_id],
        )?;
        let items_by_id = self
            .list_radar_items(run_id)?
            .into_iter()
            .map(|item| (item.id.clone(), item))
            .collect::<BTreeMap<_, _>>();
        let exact_duplicate_ids = self
            .radar_duplicate_status_by_item(run_id)?
            .into_iter()
            .filter(|(_, (status, _, _))| status != "duplicate_topic")
            .map(|(item_id, _)| item_id)
            .collect::<BTreeSet<_>>();
        let mut topic_clusters: Vec<Vec<(RadarItem, RadarScore)>> = Vec::new();
        let mut topic_signatures: Vec<BTreeSet<String>> = Vec::new();
        let mut topic_candidates = self
            .list_radar_scores(run_id)?
            .into_iter()
            .filter(|score| {
                !matches!(
                    score.status.as_str(),
                    "below_threshold" | "duplicate_url" | "duplicate_native_id" | "duplicate_exact"
                )
            })
            .filter_map(|score| {
                let item = items_by_id.get(&score.item_id)?.clone();
                if exact_duplicate_ids.contains(&item.id) {
                    return None;
                }
                let signature = radar_topic_dedupe_signature(&item)?;
                Some((item, score, signature))
            })
            .collect::<Vec<_>>();
        topic_candidates.sort_by(|left, right| {
            right
                .1
                .score
                .partial_cmp(&left.1.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.0.title.cmp(&right.0.title))
                .then_with(|| left.0.id.cmp(&right.0.id))
        });
        for (item, score, signature) in topic_candidates {
            if let Some(index) = topic_signatures
                .iter()
                .position(|existing| radar_topic_signatures_match(existing, &signature))
            {
                topic_clusters[index].push((item, score));
                topic_signatures[index].extend(signature);
            } else {
                topic_clusters.push(vec![(item, score)]);
                topic_signatures.push(signature);
            }
        }

        let mut inserted = 0usize;
        for members in topic_clusters {
            if members.len() < 2 {
                continue;
            }
            let group = radar_topic_dedup_group(run_id, members)?;
            self.insert_radar_dedup_group(&group)?;
            inserted += 1;
        }
        Ok(inserted)
    }

    pub(crate) fn insert_radar_dedup_group(&self, group: &RadarDedupGroup) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO radar_dedup_groups
              (id, run_id, dedup_kind, primary_item_id, member_item_ids_json,
               reason, confidence, model_provider, cost_decision_id, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                group.id,
                group.run_id,
                group.dedup_kind,
                group.primary_item_id,
                serde_json::to_string(&group.member_item_ids)?,
                group.reason,
                group.confidence,
                group.model_provider,
                group.cost_decision_id,
                group.created_at
            ],
        )?;
        Ok(())
    }

    pub(crate) fn radar_duplicate_status_by_item(
        &self,
        run_id: &str,
    ) -> Result<BTreeMap<String, (String, String, String)>> {
        let mut duplicates = BTreeMap::new();
        for group in self.list_radar_dedup_groups(run_id)? {
            let status = match group.dedup_kind.as_str() {
                "canonical_url" => "duplicate_url",
                "same_native_id" => "duplicate_native_id",
                "semantic_topic" => "duplicate_topic",
                _ => "duplicate_exact",
            };
            for member_id in &group.member_item_ids {
                if member_id != &group.primary_item_id {
                    duplicates.entry(member_id.clone()).or_insert_with(|| {
                        (
                            status.to_string(),
                            group.id.clone(),
                            group.primary_item_id.clone(),
                        )
                    });
                }
            }
        }
        Ok(duplicates)
    }

    pub(crate) fn score_radar_run(&self, run_id: &str) -> Result<usize> {
        let run = self
            .read_radar_run(run_id)?
            .with_context(|| format!("radar run not found: {run_id}"))?;
        let profile = self
            .read_radar_profile(&run.profile_id)?
            .with_context(|| format!("radar profile not found: {}", run.profile_id))?;
        let items = self.list_radar_items(run_id)?;
        let source_health = self.list_source_health()?;
        let mut scored = Vec::new();
        for item in items {
            let health = radar_source_health_for_item(&source_health, &item);
            let (score, reason, tags) = score_radar_item_heuristic_with_health(&item, health);
            scored.push((item, score, reason, tags));
        }
        let duplicate_status_by_item = self.radar_duplicate_status_by_item(run_id)?;
        scored.sort_by(|left, right| {
            right
                .1
                .partial_cmp(&left.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.0.title.cmp(&right.0.title))
        });
        let max_items = profile.max_items.unwrap_or(i64::MAX).max(0) as usize;
        let balance_config = radar_balance_config_from_metadata(&profile.metadata)?;
        let mut selected_ids = BTreeSet::new();
        let mut source_selected_counts: BTreeMap<String, usize> = BTreeMap::new();
        let mut category_selected_counts: BTreeMap<String, usize> = BTreeMap::new();
        let mut balance_rejections: BTreeMap<String, (String, String, String)> = BTreeMap::new();
        for (item, score, _, tags) in &scored {
            if *score < profile.min_score || duplicate_status_by_item.contains_key(&item.id) {
                continue;
            }
            if selected_ids.len() >= max_items {
                continue;
            }
            let source_key = radar_balance_source_key(item);
            if let Some(max_per_source) = balance_config.max_per_source {
                let selected_for_source = source_selected_counts
                    .get(source_key.as_str())
                    .copied()
                    .unwrap_or(0);
                if selected_for_source >= max_per_source {
                    balance_rejections.insert(
                        item.id.clone(),
                        (
                            "source_quota".to_string(),
                            format!("source quota cap {max_per_source} reached for `{source_key}`"),
                            "source-quota".to_string(),
                        ),
                    );
                    continue;
                }
            }
            let category = radar_balance_category_for_item(item, tags, &balance_config);
            if let Some(category) = category.as_deref()
                && let Some(max_per_category) = balance_config.category_quotas.get(category)
            {
                let selected_for_category =
                    category_selected_counts.get(category).copied().unwrap_or(0);
                if selected_for_category >= *max_per_category {
                    balance_rejections.insert(
                        item.id.clone(),
                        (
                            "category_quota".to_string(),
                            format!(
                                "category quota cap {max_per_category} reached for `{category}`"
                            ),
                            format!("category-quota-{category}"),
                        ),
                    );
                    continue;
                }
            }
            selected_ids.insert(item.id.clone());
            *source_selected_counts.entry(source_key).or_default() += 1;
            if let Some(category) = category {
                *category_selected_counts.entry(category).or_default() += 1;
            }
        }
        let timestamp = now();
        let mut inserted = 0usize;
        for (item, score, mut reason, mut tags) in scored {
            let status = if let Some((status, group_id, primary_item_id)) =
                duplicate_status_by_item.get(&item.id)
            {
                tags.push("duplicate".to_string());
                if status == "duplicate_topic" {
                    tags.push("semantic-dedupe".to_string());
                } else {
                    tags.push("exact-dedupe".to_string());
                }
                reason = format!(
                    "{reason}; duplicate suppressed by dedupe group {group_id}; primary item {primary_item_id}"
                );
                status.as_str()
            } else if selected_ids.contains(&item.id) {
                "selected"
            } else if let Some((status, rejection_reason, rejection_tag)) =
                balance_rejections.get(&item.id)
            {
                tags.push(rejection_tag.clone());
                if status == "source_quota" {
                    tags.push("source-quota".to_string());
                } else if status == "category_quota" {
                    tags.push("category-quota".to_string());
                }
                reason = format!("{reason}; {rejection_reason}");
                status.as_str()
            } else if score >= profile.min_score {
                "over_profile_limit"
            } else {
                "below_threshold"
            };
            self.conn.execute(
                r#"
                INSERT INTO radar_scores
                  (id, run_id, item_id, score_kind, score, reason, tags_json,
                   schema_version, status, created_at)
                VALUES (?1, ?2, ?3, 'heuristic_v1', ?4, ?5, ?6, 1, ?7, ?8)
                ON CONFLICT(item_id, score_kind, schema_version) DO UPDATE SET
                  score = excluded.score,
                  reason = excluded.reason,
                  tags_json = excluded.tags_json,
                  status = excluded.status,
                  error = NULL
                "#,
                params![
                    Uuid::new_v4().to_string(),
                    run_id,
                    item.id,
                    score,
                    reason,
                    serde_json::to_string(&tags)?,
                    status,
                    timestamp
                ],
            )?;
            inserted += 1;
        }
        Ok(inserted)
    }

    pub fn score_radar_run_with_model(
        &self,
        run_id: &str,
        provider: &str,
        model_name: Option<&str>,
        max_items: usize,
        endpoint: Option<&str>,
        api_key: Option<&str>,
    ) -> Result<RadarModelScoreReport> {
        validate_id(run_id)?;
        let provider = provider.trim().to_ascii_lowercase();
        if !matches!(provider.as_str(), "mock" | "openai") {
            bail!("unsupported radar model scoring provider: {provider}");
        }
        let run = self
            .read_radar_run(run_id)?
            .with_context(|| format!("radar run not found: {run_id}"))?;
        let profile = self
            .read_radar_profile(&run.profile_id)?
            .with_context(|| format!("radar profile not found: {}", run.profile_id))?;
        let audit = self.audit_radar_run(run_id)?;
        let audit_allows_missing_provenance_block_only = !audit.ok
            && !audit.findings.is_empty()
            && audit
                .findings
                .iter()
                .all(|finding| finding.code == "radar_missing_source_card");
        if !audit.ok && !audit_allows_missing_provenance_block_only {
            bail!(
                "radar model scoring requires an audit-ok run; finding_count={}",
                audit.findings.len()
            );
        }
        let model = model_name
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| {
                if provider == "mock" {
                    "mock-radar-interestingness".to_string()
                } else {
                    std::env::var("ARCWELL_RADAR_MODEL")
                        .unwrap_or_else(|_| "gpt-5.5-mini".to_string())
                }
            });
        validate_key(&provider)?;
        validate_key(&model)?;
        let items_by_id = self
            .list_radar_items(run_id)?
            .into_iter()
            .map(|item| (item.id.clone(), item))
            .collect::<BTreeMap<_, _>>();
        let mut heuristic_scores = self
            .list_radar_scores(run_id)?
            .into_iter()
            .filter(|score| score.score_kind == "heuristic_v1")
            .filter(|score| matches!(score.status.as_str(), "selected" | "over_profile_limit"))
            .filter_map(|score| {
                let item = items_by_id.get(&score.item_id)?.clone();
                Some((item, score))
            })
            .collect::<Vec<_>>();
        heuristic_scores.sort_by(|left, right| {
            right
                .1
                .score
                .partial_cmp(&left.1.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.0.title.cmp(&right.0.title))
                .then_with(|| left.0.id.cmp(&right.0.id))
        });
        let limit = max_items.clamp(1, 25);
        if heuristic_scores.is_empty() {
            bail!("radar model scoring requires selected or over-limit heuristic score rows");
        }
        let mut eligible_scores = Vec::new();
        let mut excluded_scores = Vec::new();
        for (item, score) in heuristic_scores {
            if audit_allows_missing_provenance_block_only {
                excluded_scores.push((
                    item,
                    score,
                    "radar audit failed before model scoring: missing source-card provenance"
                        .to_string(),
                ));
                continue;
            }
            let source_card = if let Some(source_card_id) = item.source_card_id.as_deref() {
                self.read_source_card(source_card_id)?
            } else {
                None
            };
            let missing_source_card_reason = item.source_card_id.as_deref().and_then(|id| {
                if source_card.is_none() {
                    Some(format!(
                        "radar item provenance missing linked source card {id}"
                    ))
                } else {
                    None
                }
            });
            if let Some(reason) = missing_source_card_reason
                .or_else(|| radar_model_prompt_exclusion_reason(&item, source_card.as_ref()))
            {
                excluded_scores.push((item, score, reason));
            } else if eligible_scores.len() < limit {
                eligible_scores.push((item, score));
            }
        }
        let prompt = if eligible_scores.is_empty() {
            format!(
                "Radar model scoring privacy filter found no eligible candidates for run {}. No source titles, excerpts, URLs, or metadata from excluded candidates are included in this artifact.",
                run.id
            )
        } else {
            build_radar_model_score_prompt(&profile, &run, &eligible_scores)?
        };
        let projected_cost =
            estimated_radar_model_score_cost(&model, prompt.len(), eligible_scores.len());
        let invocation_job_id = format!("radar-model-score-{}", Uuid::new_v4().simple());
        let (provider_response, cost_decision_id) = if eligible_scores.is_empty() {
            (
                json!({
                    "scores": [],
                    "status": "privacy_filter_no_eligible_candidates"
                }),
                None::<String>,
            )
        } else if provider == "mock" {
            (
                mock_radar_model_score_response(&eligible_scores),
                None::<String>,
            )
        } else {
            let endpoint = validated_endpoint(endpoint, "https://api.openai.com/v1/responses")?;
            self.policy_guard(PolicyRequest {
                action: "provider.network".to_string(),
                package: Some("arcwell-radar".to_string()),
                provider: Some("openai".to_string()),
                source: Some("radar_model_score".to_string()),
                channel: None,
                subject: None,
                target: Some(endpoint.as_str().to_string()),
                projected_usd: Some(projected_cost),
                metadata: json!({
                    "run_id": run.id,
                    "profile_id": profile.id,
                    "model": model,
                    "candidate_count": eligible_scores.len()
                }),
                untrusted_excerpt: Some(excerpt(&prompt, 1_000)),
            })?;
            let decision = self.require_cost_budget(
                "arcwell-radar",
                &invocation_job_id,
                "openai",
                &model,
                Some("radar_model_score"),
                projected_cost,
                "radar model scoring",
            )?;
            (
                openai_radar_model_score_response(
                    &prompt,
                    &model,
                    endpoint,
                    api_key
                        .map(ToOwned::to_owned)
                        .or_else(|| self.configured_openai_api_key().ok().flatten())
                        .as_deref(),
                    Duration::from_secs(45),
                )?,
                decision.decision_id,
            )
        };
        let input_artifact_id = self.add_wiki_page(
            &format!("Radar Model Score Input: {}", run.id),
            &format!(
                "# Radar Model Score Input\n\n- Run: `{}`\n- Provider: `{}`\n- Model: `{}`\n- Eligible candidate count: `{}`\n- Excluded candidate count: `{}`\n\n```text\n{}\n```\n",
                run.id,
                provider,
                model,
                eligible_scores.len(),
                excluded_scores.len(),
                prompt
            ),
            &format!("radar-model-score-input:{}:{}", run.id, provider),
        )?;
        let output_artifact_id = self.add_wiki_page(
            &format!("Radar Model Score Output: {}", run.id),
            &format!(
                "# Radar Model Score Output\n\n- Run: `{}`\n- Provider: `{}`\n- Model: `{}`\n- Provider status: `{}`\n\n```json\n{}\n```\n",
                run.id,
                provider,
                model,
                provider_response
                    .get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("response_received"),
                canonical_json(&sanitize_work_json(provider_response.clone())?)?
            ),
            &format!("radar-model-score-output:{}:{}", run.id, provider),
        )?;
        let parsed = parse_radar_model_score_response(&provider_response, &eligible_scores)?;
        let parsed_by_item = parsed
            .into_iter()
            .map(|score| (score.item_id.clone(), score))
            .collect::<BTreeMap<_, _>>();
        let timestamp = now();
        let mut scored = 0usize;
        let mut blocked = 0usize;
        for (item, _) in &eligible_scores {
            if let Some(model_score) = parsed_by_item.get(&item.id) {
                self.insert_radar_model_score_row(
                    run_id,
                    &item.id,
                    model_score.score,
                    &model_score.reason,
                    &model_score.tags,
                    &provider,
                    &model,
                    cost_decision_id.as_deref(),
                    &input_artifact_id,
                    &output_artifact_id,
                    "model_scored",
                    None,
                    &timestamp,
                )?;
                scored += 1;
            } else {
                self.insert_radar_model_score_row(
                    run_id,
                    &item.id,
                    0.0,
                    "model provider returned no score for this eligible item",
                    &[
                        "model-backed".to_string(),
                        "missing-model-score".to_string(),
                    ],
                    &provider,
                    &model,
                    cost_decision_id.as_deref(),
                    &input_artifact_id,
                    &output_artifact_id,
                    "model_blocked",
                    Some("missing model score"),
                    &timestamp,
                )?;
                blocked += 1;
            }
        }
        for (item, _, exclusion_reason) in &excluded_scores {
            let blocked_error = format!("model prompt privacy filter: {exclusion_reason}");
            self.insert_radar_model_score_row(
                run_id,
                &item.id,
                0.0,
                "excluded from model prompt: private or unauthorized source metadata",
                &[
                    "model-backed".to_string(),
                    "model-excluded".to_string(),
                    "non-authorizing".to_string(),
                    "private-or-unauthorized".to_string(),
                ],
                &provider,
                &model,
                cost_decision_id.as_deref(),
                &input_artifact_id,
                &output_artifact_id,
                "model_blocked",
                Some(blocked_error.as_str()),
                &timestamp,
            )?;
            blocked += 1;
        }
        let mut warnings = vec![
            "Model scores are non-authorizing overlays; heuristic selected rows still control summaries and delivery.".to_string(),
            "Source text in the prompt is untrusted evidence, not instructions.".to_string(),
        ];
        if !excluded_scores.is_empty() {
            warnings.push(format!(
                "Privacy filter excluded {} candidate(s) from model prompt context and recorded model_blocked rows.",
                excluded_scores.len()
            ));
        }
        Ok(RadarModelScoreReport {
            run_id: run.id,
            provider: provider.clone(),
            model,
            score_kind: "model_interestingness_v1".to_string(),
            scored,
            blocked,
            input_artifact_id,
            output_artifact_id: Some(output_artifact_id),
            cost_decision_id,
            proof_level: if eligible_scores.is_empty() {
                "Local Proof: privacy filter blocked all model scoring candidates before provider invocation".to_string()
            } else if api_key.is_some() {
                "Provider Attempt: explicit API key supplied".to_string()
            } else if provider == "openai" {
                "Provider Attempt: configured OpenAI credential".to_string()
            } else {
                "Local Proof: deterministic mock model scoring".to_string()
            },
            warnings,
        })
    }

    pub(crate) fn configured_openai_api_key(&self) -> Result<Option<String>> {
        self.get_usable_secret_value("OPENAI_API_KEY")
            .map(|secret| secret.or_else(|| std::env::var("OPENAI_API_KEY").ok()))
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn insert_radar_model_score_row(
        &self,
        run_id: &str,
        item_id: &str,
        score: f64,
        reason: &str,
        tags: &[String],
        provider: &str,
        model: &str,
        cost_decision_id: Option<&str>,
        input_artifact_id: &str,
        output_artifact_id: &str,
        status: &str,
        error: Option<&str>,
        timestamp: &str,
    ) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO radar_scores
              (id, run_id, item_id, score_kind, score, reason, tags_json,
               model_provider, model_name, cost_decision_id, input_artifact_id,
               output_artifact_id, schema_version, status, error, created_at)
            VALUES (?1, ?2, ?3, 'model_interestingness_v1', ?4, ?5, ?6,
                    ?7, ?8, ?9, ?10, ?11, 1, ?12, ?13, ?14)
            ON CONFLICT(item_id, score_kind, schema_version) DO UPDATE SET
              score = excluded.score,
              reason = excluded.reason,
              tags_json = excluded.tags_json,
              model_provider = excluded.model_provider,
              model_name = excluded.model_name,
              cost_decision_id = excluded.cost_decision_id,
              input_artifact_id = excluded.input_artifact_id,
              output_artifact_id = excluded.output_artifact_id,
              status = excluded.status,
              error = excluded.error,
              created_at = excluded.created_at
            "#,
            params![
                Uuid::new_v4().to_string(),
                run_id,
                item_id,
                score,
                reason,
                serde_json::to_string(tags)?,
                provider,
                model,
                cost_decision_id,
                input_artifact_id,
                output_artifact_id,
                status,
                error,
                timestamp
            ],
        )?;
        Ok(())
    }

    pub(crate) fn record_radar_source_quality_window(&self, run_id: &str) -> Result<usize> {
        validate_id(run_id)?;
        let run = self
            .read_radar_run(run_id)?
            .with_context(|| format!("radar run not found: {run_id}"))?;
        self.conn.execute(
            "DELETE FROM radar_source_quality WHERE run_id = ?1",
            params![run_id],
        )?;
        let scores = self
            .list_radar_scores(run_id)?
            .into_iter()
            .filter(|score| score.score_kind == "heuristic_v1")
            .map(|score| (score.item_id.clone(), score))
            .collect::<BTreeMap<_, _>>();
        let source_health = self.list_source_health()?;
        let mut buckets: BTreeMap<(String, String), Vec<(RadarItem, RadarScore)>> = BTreeMap::new();
        for item in self.list_radar_items(run_id)? {
            let Some(score) = scores.get(&item.id).cloned() else {
                continue;
            };
            let key = radar_source_quality_key_for_item(&item);
            buckets.entry(key).or_default().push((item, score));
        }

        let timestamp = now();
        let mut written = 0usize;
        for ((source_kind, locator), entries) in buckets {
            if entries.is_empty() {
                continue;
            }
            let raw_count = entries.len() as i64;
            let accepted_count = entries
                .iter()
                .filter(|(_, score)| score.status == "selected")
                .count() as i64;
            let duplicate_count = entries
                .iter()
                .filter(|(_, score)| score.status.starts_with("duplicate_"))
                .count() as i64;
            let mut values = entries
                .iter()
                .map(|(_, score)| score.score)
                .collect::<Vec<_>>();
            values.sort_by(|left, right| {
                left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal)
            });
            let average_score = values.iter().sum::<f64>() / values.len() as f64;
            let score_p50 = percentile_sorted(&values, 0.50);
            let score_p90 = percentile_sorted(&values, 0.90);
            let signal_to_noise = accepted_count as f64 / raw_count as f64;
            let duplicate_rate = duplicate_count as f64 / raw_count as f64;
            let failure_count = radar_source_health_for_quality_key(
                &source_health,
                &source_kind,
                &locator,
                entries
                    .first()
                    .map(|(item, _)| item.provider.as_str())
                    .unwrap_or(""),
            )
            .filter(|health| health.status != "healthy")
            .map(|_| 1)
            .unwrap_or(0);
            let status = if failure_count > 0 && accepted_count == 0 {
                "failed"
            } else if failure_count > 0 {
                "partial"
            } else if accepted_count == 0 {
                "low_signal"
            } else {
                "healthy"
            };
            self.conn.execute(
                r#"
                INSERT INTO radar_source_quality
                  (id, run_id, source_kind, locator, window_start, window_end, raw_count,
                   accepted_count, average_score, score_p50, score_p90, signal_to_noise,
                   duplicate_rate, delivery_contribution_count, failure_count, status, created_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 0, ?14, ?15, ?16)
                ON CONFLICT(run_id, source_kind, locator, window_start, window_end) DO UPDATE SET
                  raw_count = excluded.raw_count,
                  accepted_count = excluded.accepted_count,
                  average_score = excluded.average_score,
                  score_p50 = excluded.score_p50,
                  score_p90 = excluded.score_p90,
                  signal_to_noise = excluded.signal_to_noise,
                  duplicate_rate = excluded.duplicate_rate,
                  failure_count = excluded.failure_count,
                  status = excluded.status,
                  created_at = excluded.created_at
                "#,
                params![
                    Uuid::new_v4().to_string(),
                    run_id,
                    source_kind,
                    locator,
                    run.window_start,
                    run.window_end,
                    raw_count,
                    accepted_count,
                    average_score,
                    score_p50,
                    score_p90,
                    signal_to_noise,
                    duplicate_rate,
                    failure_count,
                    status,
                    timestamp
                ],
            )?;
            written += 1;
        }
        Ok(written)
    }

    pub(crate) fn count_radar_selected_scores(&self, run_id: &str) -> Result<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM radar_scores WHERE run_id = ?1 AND status = 'selected'",
            params![run_id],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    pub(crate) fn radar_source_cards_for_selector(
        &self,
        selector: &Value,
    ) -> Result<Vec<SourceCard>> {
        let kind = radar_selector_kind(selector).context("radar selector requires kind")?;
        let locator = selector
            .get("query")
            .or_else(|| selector.get("locator"))
            .or_else(|| selector.get("handle"))
            .and_then(Value::as_str)
            .unwrap_or("*")
            .trim()
            .trim_start_matches('@')
            .to_string();
        if kind == "source_card_query" {
            return if locator == "*" {
                self.list_source_cards()
            } else {
                self.search_source_cards(&locator)
            };
        }
        let cards = self
            .list_source_cards()?
            .into_iter()
            .filter(|card| radar_source_card_matches_selector(card, &kind, &locator))
            .collect();
        Ok(cards)
    }

    pub fn librarian_expand_topic(&self, topic: &str) -> Result<String> {
        validate_query(topic)?;
        let job = self.run_wiki_expand_page_job(topic)?;
        job.result_json
            .and_then(|value| {
                value
                    .get("page_id")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
            })
            .context("librarian expansion did not produce a page id")
    }

    pub fn extract_memory_candidates_from_text(
        &self,
        text: &str,
        source_ref: &str,
    ) -> Result<MemoryPipelineReport> {
        self.extract_memory_candidates_from_text_for_user(text, source_ref, None)
    }

    pub fn extract_memory_candidates_from_text_for_user(
        &self,
        text: &str,
        source_ref: &str,
        user_id: Option<&str>,
    ) -> Result<MemoryPipelineReport> {
        validate_notes(text)?;
        validate_notes(source_ref)?;
        self.policy_guard(PolicyRequest {
            action: "memory.capture".to_string(),
            package: Some("arcwell-memory".to_string()),
            provider: None,
            source: Some("memory_extract".to_string()),
            channel: None,
            subject: user_id.map(ToOwned::to_owned),
            target: Some(excerpt(source_ref, 240)),
            projected_usd: None,
            metadata: json!({ "mode": "extract_candidates", "text_len": text.len() }),
            untrusted_excerpt: Some(text.to_string()),
        })?;
        let mut created = Vec::new();
        let mut duplicates_suppressed = 0;
        for candidate in memory_candidate_phrases(text) {
            let sensitivity = classify_memory_sensitivity(&candidate);
            let plan = self.plan_memory_candidate(&candidate, user_id)?;
            let duplicate = plan.operation == "NONE"
                || self
                    .list_candidates("pending")?
                    .into_iter()
                    .any(|existing| existing.content.eq_ignore_ascii_case(&candidate));
            if duplicate {
                duplicates_suppressed += 1;
                self.record_memory_decision(
                    user_id,
                    source_ref,
                    &candidate,
                    &plan.operation,
                    plan.memory_id.as_deref(),
                    None,
                    plan.confidence,
                    &plan.reason,
                    &json!({
                        "matched_memory": plan.matched_memory.clone(),
                        "duplicate_suppressed": true,
                        "decision_source": "deterministic-extractor"
                    }),
                )?;
                continue;
            }
            let id = self.add_candidate_with_operation(
                "memory",
                "fact",
                &candidate,
                &sensitivity,
                source_ref,
                &plan.operation,
                plan.memory_id.as_deref(),
                user_id,
                json!({
                    "reason": plan.reason.clone(),
                    "matched_memory": plan.matched_memory.clone(),
                    "confidence": plan.confidence,
                    "review_required": sensitivity == "sensitive" || plan.operation != "ADD",
                    "source": "arcwell-memory-extractor"
                }),
            )?;
            self.record_memory_decision(
                user_id,
                source_ref,
                &candidate,
                &plan.operation,
                plan.memory_id.as_deref(),
                Some(&id),
                plan.confidence,
                &plan.reason,
                &json!({
                    "matched_memory": plan.matched_memory.clone(),
                    "sensitivity": sensitivity,
                    "review_required": sensitivity == "sensitive" || plan.operation != "ADD",
                    "decision_source": "deterministic-extractor"
                }),
            )?;
            let new_candidate = self
                .list_candidates("pending")?
                .into_iter()
                .find(|candidate| candidate.id == id)
                .context("new memory candidate not found")?;
            created.push(new_candidate);
        }
        Ok(MemoryPipelineReport {
            candidates_created: created.len(),
            duplicates_suppressed,
            candidates: created,
        })
    }

    pub(crate) fn plan_memory_candidate(
        &self,
        text: &str,
        user_id: Option<&str>,
    ) -> Result<MemoryCandidatePlan> {
        let delete_query = memory_delete_query(text);
        if let Some(query) = delete_query {
            let search = self.mem0_search_memories(&query, user_id, 5)?;
            if let Some(hit) = first_mem0_hit(&search.results) {
                return Ok(MemoryCandidatePlan {
                    operation: "DELETE".to_string(),
                    memory_id: hit.id,
                    matched_memory: Some(hit.memory),
                    confidence: 0.9,
                    reason: format!("explicit delete/forget request matched {query:?}"),
                });
            }
            return Ok(MemoryCandidatePlan {
                operation: "NONE".to_string(),
                memory_id: None,
                matched_memory: None,
                confidence: 0.55,
                reason: "delete/forget request did not match existing memory".to_string(),
            });
        }

        let search = self.mem0_search_memories(text, user_id, 5)?;
        for hit in mem0_hit_summaries(&search.results) {
            if hit.memory.eq_ignore_ascii_case(text) {
                return Ok(MemoryCandidatePlan {
                    operation: "NONE".to_string(),
                    memory_id: hit.id,
                    matched_memory: Some(hit.memory),
                    confidence: 0.95,
                    reason: "equivalent memory already exists".to_string(),
                });
            }
        }

        if let Some(subject) = memory_subject_key(text) {
            let search = self.mem0_search_memories(&subject, user_id, 10)?;
            for hit in mem0_hit_summaries(&search.results) {
                if memory_subject_key(&hit.memory).as_deref() == Some(subject.as_str())
                    && !hit.memory.eq_ignore_ascii_case(text)
                {
                    return Ok(MemoryCandidatePlan {
                        operation: "UPDATE".to_string(),
                        memory_id: hit.id,
                        matched_memory: Some(hit.memory),
                        confidence: 0.78,
                        reason: format!("same subject changed: {subject}"),
                    });
                }
            }
        }

        Ok(MemoryCandidatePlan {
            operation: "ADD".to_string(),
            memory_id: None,
            matched_memory: None,
            confidence: 0.72,
            reason: "new personal fact or preference".to_string(),
        })
    }

    pub fn dream_reconcile_memories(&self) -> Result<MemoryDreamReport> {
        let user_id = self.mem0_user_id(None)?;
        let (_provider, memory) = self.mem0_memory()?;
        let provider_value = self.mem0_get_all_memories_for_user(&memory, &user_id, 10_000)?;
        let mut provider_hits = mem0_hit_summaries(&provider_value);
        provider_hits.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));

        let mut actions = Vec::new();
        let mut provider_exact_duplicates_deleted = 0;
        let mut deleted_provider_ids = std::collections::HashSet::new();
        let mut exact_seen: std::collections::HashMap<String, Mem0HitSummary> =
            std::collections::HashMap::new();
        for hit in provider_hits.clone() {
            let key = normalized_memory_text(&hit.memory);
            if let Some(kept) = exact_seen.get(&key) {
                if let Some(id) = &hit.id {
                    self.mem0_delete_memory(id, Some(&user_id))?;
                    deleted_provider_ids.insert(id.clone());
                    provider_exact_duplicates_deleted += 1;
                    actions.push(json!({
                        "action": "delete_provider_duplicate",
                        "deleted_memory_id": id,
                        "kept_memory_id": kept.id,
                        "memory": hit.memory
                    }));
                }
            } else {
                exact_seen.insert(key, hit);
            }
        }

        let remaining_provider_hits: Vec<Mem0HitSummary> = provider_hits
            .into_iter()
            .filter(|hit| {
                hit.id
                    .as_ref()
                    .is_none_or(|id| !deleted_provider_ids.contains(id))
            })
            .collect();

        let memories = self.list_memories(10_000)?;
        let mut compatibility_exact_duplicates_deleted = 0;
        let mut compatibility_provider_duplicates_deleted = 0;
        let mut compat_seen = std::collections::HashSet::new();
        for item in memories.clone() {
            let key = normalized_memory_text(&item.text);
            let provider_duplicate = remaining_provider_hits
                .iter()
                .any(|hit| normalized_memory_text(&hit.memory) == key);
            if provider_duplicate {
                if self.delete_memory(&item.id)? {
                    compatibility_provider_duplicates_deleted += 1;
                    actions.push(json!({
                        "action": "delete_compatibility_duplicate_of_provider",
                        "deleted_memory_id": item.id,
                        "memory": item.text
                    }));
                }
                continue;
            }
            if !compat_seen.insert(key) && self.delete_memory(&item.id)? {
                compatibility_exact_duplicates_deleted += 1;
                actions.push(json!({
                    "action": "delete_compatibility_duplicate",
                    "deleted_memory_id": item.id,
                    "memory": item.text
                }));
            }
        }

        let mut conflicts_detected = 0;
        let mut conflict_candidates_created = 0;
        let mut subject_groups: std::collections::HashMap<String, Vec<Mem0HitSummary>> =
            std::collections::HashMap::new();
        for hit in remaining_provider_hits {
            if let Some(subject) = memory_subject_key(&hit.memory) {
                subject_groups.entry(subject).or_default().push(hit);
            }
        }
        for (subject, mut group) in subject_groups {
            let distinct: std::collections::HashSet<String> = group
                .iter()
                .map(|hit| normalized_memory_text(&hit.memory))
                .collect();
            if distinct.len() <= 1 {
                continue;
            }
            conflicts_detected += 1;
            group.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
            let keep = group[0].clone();
            for hit in group.into_iter().skip(1) {
                if let Some(memory_id) = hit.id.as_deref() {
                    if self.pending_memory_candidate_exists(
                        "DELETE",
                        Some(memory_id),
                        &hit.memory,
                    )? {
                        continue;
                    }
                    self.add_candidate_with_operation(
                        "memory",
                        "fact",
                        &hit.memory,
                        &classify_memory_sensitivity(&hit.memory),
                        "dream:reconcile",
                        "DELETE",
                        Some(memory_id),
                        Some(&user_id),
                        json!({
                            "reason": "same memory subject has conflicting values",
                            "subject": subject,
                            "keep_memory_id": keep.id,
                            "keep_memory": keep.memory,
                            "matched_memory": hit.memory,
                            "source": "arcwell-memory-dream"
                        }),
                    )?;
                    conflict_candidates_created += 1;
                    actions.push(json!({
                        "action": "create_conflict_delete_candidate",
                        "subject": subject,
                        "candidate_memory_id": memory_id,
                        "keep_memory_id": keep.id
                    }));
                }
            }
        }

        let report = MemoryDreamReport {
            user_id,
            provider_exact_duplicates_deleted,
            compatibility_exact_duplicates_deleted,
            compatibility_provider_duplicates_deleted,
            conflict_candidates_created,
            conflicts_detected,
            actions,
        };
        self.record_memory_lifecycle_event(
            "dream_reconcile",
            Some("manual_or_mcp"),
            Some(&report.user_id),
            None,
            None,
            &json!(&report),
            "completed",
        )?;
        Ok(report)
    }

    pub(crate) fn pending_memory_candidate_exists(
        &self,
        operation: &str,
        memory_id: Option<&str>,
        content: &str,
    ) -> Result<bool> {
        validate_candidate_operation(operation)?;
        validate_notes(content)?;
        if let Some(memory_id) = memory_id {
            validate_id(memory_id)?;
        }
        Ok(self
            .list_candidates("pending")?
            .into_iter()
            .any(|candidate| {
                candidate.target == "memory"
                    && candidate.operation.eq_ignore_ascii_case(operation)
                    && candidate.memory_id.as_deref() == memory_id
                    && candidate.content.eq_ignore_ascii_case(content)
            }))
    }
}
