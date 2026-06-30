use super::*;

impl Store {
    pub fn health(&self) -> Result<HealthReport> {
        let profile_items = self.count("profile_items")?;
        let memories = self.count("memories")?;
        let wiki_pages = self.count("wiki_pages")?;
        let source_cards = self.count("source_cards")?;
        let watch_sources = self.count("watch_sources")?;
        let wiki_jobs = self.count("wiki_jobs")?;
        let x_items = self.count("x_items")?;
        let x_tweets = self.count("x_tweets")?;
        let x_profiles = self.count("x_profiles")?;
        let pending_jobs: i64 = self.conn.query_row(
            "SELECT count(*) FROM wiki_jobs WHERE status = 'pending'",
            [],
            |row| row.get(0),
        )?;
        let cursors = self.count("cursors")?;
        let research_runs = self.count("research_runs")?;
        let work_runs = self.count("work_runs")?;
        let pending_candidates: i64 = self.conn.query_row(
            "SELECT count(*) FROM candidates WHERE status = 'pending'",
            [],
            |row| row.get(0),
        )?;
        let failed_jobs: i64 = self.conn.query_row(
            "SELECT count(*) FROM wiki_jobs WHERE status = 'failed'",
            [],
            |row| row.get(0),
        )?;
        let dead_lettered_jobs: i64 = self.conn.query_row(
            "SELECT count(*) FROM wiki_jobs WHERE status = 'dead_lettered'",
            [],
            |row| row.get(0),
        )?;
        let latest_backup: Option<String> = self
            .conn
            .query_row(
                "SELECT created_at FROM backups ORDER BY created_at DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()?;
        let latest_worker_heartbeat = self.latest_worker_heartbeat()?;
        let latest_worker_heartbeat_events = self.list_worker_heartbeat_events(20)?;
        let secret_health = self.secret_health()?;
        let mut warnings = Vec::new();
        if latest_backup.is_none() {
            warnings.push("no backup has been recorded".to_string());
        }
        if dead_lettered_jobs > 0 {
            warnings.push(format!("{dead_lettered_jobs} wiki jobs are dead-lettered"));
        }
        for item in &secret_health {
            warnings.extend(item.warnings.clone());
        }
        let x_stats = self.x_stats()?;
        warnings.extend(Self::x_health_warnings(&x_stats));
        let failing_radar_source_quality = self.count_query(
            "SELECT COUNT(*) FROM radar_source_quality WHERE status IN ('failed', 'partial')",
        )?;
        if failing_radar_source_quality > 0 {
            warnings.push(format!(
                "Radar source quality: {failing_radar_source_quality} failed or partial source-quality window(s)"
            ));
        }
        let non_successful_radar_deliveries = self.count_query(
            "SELECT COUNT(*) FROM radar_deliveries WHERE status IN ('failed', 'blocked')",
        )?;
        if non_successful_radar_deliveries > 0 {
            warnings.push(format!(
                "Radar delivery: {non_successful_radar_deliveries} failed or blocked delivery attempt(s)"
            ));
        }
        Ok(HealthReport {
            ok: warnings.is_empty(),
            home: self.paths.home.clone(),
            db: self.paths.db.clone(),
            schema_version: self.stored_schema_version()?,
            profile_items,
            memories,
            wiki_pages,
            source_cards,
            watch_sources,
            wiki_jobs,
            x_items,
            x_tweets,
            x_profiles,
            pending_jobs,
            cursors,
            research_runs,
            pending_candidates,
            work_runs,
            failed_jobs,
            dead_lettered_jobs,
            latest_backup,
            latest_worker_heartbeat,
            latest_worker_heartbeat_events,
            secret_health,
            warnings,
        })
    }

    pub fn doctor(&self, options: DoctorOptions) -> Result<DoctorReport> {
        let health = self.health()?;
        let mut failures = Vec::new();
        if options.strict {
            failures.extend(
                health
                    .warnings
                    .iter()
                    .filter(|warning| warning.starts_with("X "))
                    .cloned(),
            );
            failures.extend(self.required_directory_failures());
            if health.schema_version != SCHEMA_VERSION {
                failures.push(format!(
                    "schema version mismatch: database has {}, binary expects {}",
                    health.schema_version, SCHEMA_VERSION
                ));
            }
            if let Some(path) = &options.service_plist_path {
                match fs::metadata(path) {
                    Ok(metadata) if metadata.is_file() => {}
                    Ok(_) => failures.push(format!(
                        "service plist path is not a file: {}",
                        path.display()
                    )),
                    Err(error) => failures.push(format!(
                        "service plist is missing or unreadable: {} ({error})",
                        path.display()
                    )),
                }
            }
            let latest_backup = self.verify_latest_backup()?;
            match latest_backup {
                Some(verification) if verification.ok => {
                    let age = backup_age_seconds(&verification.created_at)?;
                    if age > options.max_backup_age_seconds {
                        failures.push(format!(
                            "latest backup is stale: {age}s old, limit is {}s",
                            options.max_backup_age_seconds
                        ));
                    }
                }
                Some(verification) => failures.push(format!(
                    "latest backup verification failed: {}",
                    verification.errors.join("; ")
                )),
                None => failures.push("no backup has been recorded".to_string()),
            }
            if health.dead_lettered_jobs > options.max_dead_lettered_jobs {
                failures.push(format!(
                    "{} dead-lettered wiki jobs exceeds limit {}",
                    health.dead_lettered_jobs, options.max_dead_lettered_jobs
                ));
            }
            match &health.latest_worker_heartbeat {
                Some(heartbeat) => {
                    let age = heartbeat_age_seconds(heartbeat)?;
                    if age > options.max_worker_heartbeat_age_seconds {
                        failures.push(format!(
                            "worker heartbeat is stale: {age}s old, limit is {}s",
                            options.max_worker_heartbeat_age_seconds
                        ));
                    }
                }
                None => failures.push("no worker heartbeat has been recorded".to_string()),
            }
        }
        Ok(DoctorReport {
            ok: health.ok && failures.is_empty(),
            strict: options.strict,
            health,
            failures,
        })
    }

    pub(crate) fn stored_schema_version(&self) -> Result<i64> {
        let value: String = self.conn.query_row(
            "SELECT value FROM meta WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        )?;
        value
            .parse::<i64>()
            .with_context(|| format!("parsing stored schema_version value {value:?}"))
    }

    pub(crate) fn required_directory_failures(&self) -> Vec<String> {
        [
            ("home", &self.paths.home),
            ("backups", &self.paths.backups),
            ("wiki pages", &self.paths.wiki_pages),
            ("mem0", &self.paths.mem0),
            ("procedures", &self.paths.procedures),
        ]
        .into_iter()
        .filter_map(|(label, path)| match fs::metadata(path) {
            Ok(metadata) if metadata.is_dir() => None,
            Ok(_) => Some(format!(
                "required {label} path is not a directory: {}",
                path.display()
            )),
            Err(error) => Some(format!(
                "required {label} directory is missing or unreadable: {} ({error})",
                path.display()
            )),
        })
        .collect()
    }

    pub fn record_worker_heartbeat(
        &self,
        worker_id: &str,
        processed_jobs: i64,
        last_error: Option<&str>,
    ) -> Result<WorkerHeartbeat> {
        validate_key(worker_id)?;
        let now = now();
        self.conn.execute(
            r#"
            INSERT INTO worker_heartbeats
              (worker_id, started_at, last_seen_at, processed_jobs, last_error)
            VALUES (?1, ?2, ?2, ?3, ?4)
            ON CONFLICT(worker_id) DO UPDATE SET
              last_seen_at = excluded.last_seen_at,
              processed_jobs = excluded.processed_jobs,
              last_error = excluded.last_error
            "#,
            params![worker_id, now, processed_jobs, last_error],
        )?;
        self.record_worker_heartbeat_event(worker_id, &now, processed_jobs, last_error)?;
        self.prune_worker_heartbeat_events()?;
        self.latest_worker_heartbeat()?
            .with_context(|| format!("worker heartbeat not found after update: {worker_id}"))
    }

    pub(crate) fn record_worker_heartbeat_event(
        &self,
        worker_id: &str,
        seen_at: &str,
        processed_jobs: i64,
        last_error: Option<&str>,
    ) -> Result<WorkerHeartbeatEvent> {
        validate_key(worker_id)?;
        let id = Uuid::new_v4().to_string();
        self.conn.execute(
            r#"
            INSERT INTO worker_heartbeat_events
              (id, worker_id, seen_at, processed_jobs, last_error, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?3)
            "#,
            params![id, worker_id, seen_at, processed_jobs, last_error],
        )?;
        self.conn
            .query_row(
                r#"
                SELECT id, worker_id, seen_at, processed_jobs, last_error
                FROM worker_heartbeat_events
                WHERE id = ?1
                "#,
                params![id],
                worker_heartbeat_event_from_row,
            )
            .map_err(Into::into)
    }

    pub(crate) fn prune_worker_heartbeat_events(&self) -> Result<usize> {
        let cutoff =
            (Utc::now() - ChronoDuration::days(WORKER_HEARTBEAT_EVENT_RETENTION_DAYS)).to_rfc3339();
        self.conn
            .execute(
                "DELETE FROM worker_heartbeat_events WHERE seen_at < ?1",
                params![cutoff],
            )
            .map_err(Into::into)
    }

    pub fn latest_worker_heartbeat(&self) -> Result<Option<WorkerHeartbeat>> {
        self.conn
            .query_row(
                r#"
                SELECT worker_id, started_at, last_seen_at, processed_jobs, last_error
                FROM worker_heartbeats
                ORDER BY last_seen_at DESC
                LIMIT 1
                "#,
                [],
                worker_heartbeat_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_worker_heartbeat_events(&self, limit: usize) -> Result<Vec<WorkerHeartbeatEvent>> {
        let limit = limit.clamp(1, 10_000);
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, worker_id, seen_at, processed_jobs, last_error
            FROM worker_heartbeat_events
            ORDER BY seen_at DESC, id DESC
            LIMIT ?1
            "#,
        )?;
        rows(stmt.query_map(params![limit as i64], worker_heartbeat_event_from_row)?)
    }

    pub fn audit_worker_recurrence(
        &self,
        min_required_span_seconds: i64,
        max_allowed_gap_seconds: i64,
    ) -> Result<WorkerRecurrenceAudit> {
        if min_required_span_seconds <= 0 {
            bail!("min_required_span_seconds must be greater than zero");
        }
        if max_allowed_gap_seconds <= 0 {
            bail!("max_allowed_gap_seconds must be greater than zero");
        }
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, worker_id, seen_at, processed_jobs, last_error
            FROM worker_heartbeat_events
            ORDER BY seen_at ASC, id ASC
            "#,
        )?;
        let events = rows(stmt.query_map([], worker_heartbeat_event_from_row)?)?;
        let mut failures = Vec::new();
        if events.len() < 2 {
            failures.push(
                "worker recurrence requires at least two retained heartbeat events".to_string(),
            );
        }
        let worker_ids = events
            .iter()
            .map(|event| event.worker_id.clone())
            .collect::<BTreeSet<_>>();
        let worker_ids = worker_ids.into_iter().collect::<Vec<_>>();
        let mut best_segment: Vec<WorkerHeartbeatEvent> = Vec::new();
        let mut current_segment: Vec<WorkerHeartbeatEvent> = Vec::new();
        let mut current_max_gap = None::<i64>;
        let mut best_max_gap = None::<i64>;
        for event in &events {
            if let Some(previous) = current_segment.last() {
                let previous_at = DateTime::parse_from_rfc3339(&previous.seen_at)
                    .with_context(|| format!("parsing heartbeat event {}", previous.seen_at))?
                    .with_timezone(&Utc);
                let event_at = DateTime::parse_from_rfc3339(&event.seen_at)
                    .with_context(|| format!("parsing heartbeat event {}", event.seen_at))?
                    .with_timezone(&Utc);
                let gap = (event_at - previous_at).num_seconds().max(0);
                if gap > max_allowed_gap_seconds {
                    if worker_heartbeat_segment_span_seconds(&current_segment)?
                        > worker_heartbeat_segment_span_seconds(&best_segment)?
                    {
                        best_max_gap = current_max_gap;
                        best_segment = current_segment;
                    }
                    current_segment = vec![event.clone()];
                    current_max_gap = None;
                    continue;
                }
                current_max_gap = Some(current_max_gap.map_or(gap, |current| current.max(gap)));
            }
            current_segment.push(event.clone());
        }
        if worker_heartbeat_segment_span_seconds(&current_segment)?
            > worker_heartbeat_segment_span_seconds(&best_segment)?
        {
            best_max_gap = current_max_gap;
            best_segment = current_segment;
        }
        let first = best_segment.first();
        let last = best_segment.last();
        let observed_span_seconds = worker_heartbeat_segment_span_seconds(&best_segment)?;
        if observed_span_seconds < min_required_span_seconds {
            failures.push(format!(
                "best contiguous worker heartbeat event span is {observed_span_seconds}s, below required {min_required_span_seconds}s"
            ));
        }
        let sample_events = if best_segment.len() <= 10 {
            best_segment.clone()
        } else {
            let mut sample = best_segment.iter().take(5).cloned().collect::<Vec<_>>();
            let tail_start = best_segment.len().saturating_sub(5);
            sample.extend(best_segment[tail_start..].iter().cloned());
            sample
        };
        Ok(WorkerRecurrenceAudit {
            ok: failures.is_empty(),
            worker_id: best_segment.first().map(|event| event.worker_id.clone()),
            worker_ids,
            event_count: best_segment.len(),
            retained_event_count: events.len(),
            first_seen_at: first.map(|event| event.seen_at.clone()),
            last_seen_at: last.map(|event| event.seen_at.clone()),
            observed_span_seconds,
            max_gap_seconds: best_max_gap,
            min_required_span_seconds,
            max_allowed_gap_seconds,
            failures,
            sample_events,
        })
    }

    pub(crate) fn count(&self, table: &str) -> Result<i64> {
        let sql = format!("SELECT count(*) FROM {table}");
        Ok(self.conn.query_row(&sql, [], |row| row.get(0))?)
    }

    pub(crate) fn count_query(&self, sql: &str) -> Result<i64> {
        Ok(self.conn.query_row(sql, [], |row| row.get(0))?)
    }

    pub(crate) fn grouped_counts(&self, sql: &str) -> Result<BTreeMap<String, i64>> {
        let mut stmt = self.conn.prepare(sql)?;
        let rows = rows(stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?)?;
        Ok(rows.into_iter().collect())
    }

    pub(crate) fn record_x_sync_run(&self, input: XSyncRunInsert<'_>) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let metadata_json = serde_json::to_string(&input.metadata)?;
        let error = input.error.map(redact_secret_like_text);
        self.conn.execute(
            r#"
            INSERT INTO x_sync_runs
              (id, account_id, stream, transport, status, started_at, completed_at,
               seen, inserted, updated, skipped_duplicates, rejected, cursor_key,
               previous_cursor, new_cursor, error, metadata_json)
            VALUES
              (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
            "#,
            params![
                id,
                input.account_id,
                input.stream,
                input.transport,
                input.status,
                input.started_at,
                input.completed_at,
                count_to_i64(input.seen),
                count_to_i64(input.inserted),
                count_to_i64(input.updated),
                count_to_i64(input.skipped_duplicates),
                count_to_i64(input.rejected),
                input.cursor_key,
                input.previous_cursor,
                input.new_cursor,
                error,
                metadata_json,
            ],
        )?;
        Ok(id)
    }
}
