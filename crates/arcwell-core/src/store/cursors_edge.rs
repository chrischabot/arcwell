use super::*;

impl Store {
    pub fn get_cursor(&self, key: &str) -> Result<Option<CursorState>> {
        validate_key(key)?;
        self.conn
            .query_row(
                "SELECT key, value, updated_at FROM cursors WHERE key = ?1",
                params![key],
                cursor_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn set_cursor(&self, key: &str, value: &str) -> Result<()> {
        validate_key(key)?;
        validate_key(value)?;
        let now = now();
        self.conn.execute(
            r#"
            INSERT INTO cursors (key, value, updated_at)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
            "#,
            params![key, value, now],
        )?;
        Ok(())
    }

    pub(crate) fn delete_cursor(&self, key: &str) -> Result<()> {
        validate_key(key)?;
        self.conn
            .execute("DELETE FROM cursors WHERE key = ?1", params![key])?;
        Ok(())
    }

    pub fn list_cursors(&self) -> Result<Vec<CursorState>> {
        let mut stmt = self
            .conn
            .prepare("SELECT key, value, updated_at FROM cursors ORDER BY key")?;
        rows(stmt.query_map([], cursor_from_row)?)
    }

    pub fn get_source_health(&self, key: &str) -> Result<Option<SourceHealth>> {
        validate_key(key)?;
        self.conn
            .query_row(
                r#"
                SELECT key, provider, source_kind, locator, status, last_success_at, last_failure_at,
                       last_error, last_item_id, last_item_date, cursor_key, cursor_value,
                       next_run_at, updated_at
                FROM source_health
                WHERE key = ?1
                "#,
                params![key],
                source_health_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_source_health(&self) -> Result<Vec<SourceHealth>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT key, provider, source_kind, locator, status, last_success_at, last_failure_at,
                   last_error, last_item_id, last_item_date, cursor_key, cursor_value,
                   next_run_at, updated_at
            FROM source_health
            ORDER BY updated_at DESC, key
            "#,
        )?;
        rows(stmt.query_map([], source_health_from_row)?)
    }

    pub(crate) fn record_source_success(&self, update: SourceHealthUpdate<'_>) -> Result<()> {
        let updated_at = now();
        self.conn.execute(
            r#"
            INSERT INTO source_health
              (key, provider, source_kind, locator, status, last_success_at, last_failure_at,
               last_error, last_item_id, last_item_date, cursor_key, cursor_value, next_run_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, 'healthy', ?5, NULL, NULL, ?6, ?7, ?8, ?9, ?10, ?5)
            ON CONFLICT(key) DO UPDATE SET
              provider = excluded.provider,
              source_kind = excluded.source_kind,
              locator = excluded.locator,
              status = excluded.status,
              last_success_at = excluded.last_success_at,
              last_error = NULL,
              last_item_id = COALESCE(excluded.last_item_id, source_health.last_item_id),
              last_item_date = COALESCE(excluded.last_item_date, source_health.last_item_date),
              cursor_key = excluded.cursor_key,
              cursor_value = excluded.cursor_value,
              next_run_at = excluded.next_run_at,
              updated_at = excluded.updated_at
            "#,
            params![
                update.key,
                update.provider,
                update.source_kind,
                update.locator,
                updated_at,
                update.last_item_id,
                update.last_item_date,
                update.cursor_key,
                update.cursor_value,
                update.next_run_at,
            ],
        )?;
        Ok(())
    }

    pub(crate) fn watch_source_next_run_seconds(
        &self,
        source_kind: &str,
        locator: &str,
        fallback_seconds: i64,
    ) -> i64 {
        let id = watch_source_id(source_kind, locator);
        self.read_watch_source(&id)
            .ok()
            .flatten()
            .and_then(|source| watch_source_cadence_seconds(&source.cadence))
            .unwrap_or(fallback_seconds)
    }

    pub(crate) fn record_source_failure(
        &self,
        key: &str,
        provider: &str,
        source_kind: &str,
        locator: &str,
        error: &str,
    ) -> Result<()> {
        let updated_at = now();
        let error = redact_secret_like_text(error);
        let classification = classify_provider_failure(&error);
        let next_run_at = now_plus_seconds(classification.backoff_seconds);
        self.conn.execute(
            r#"
            INSERT INTO source_health
              (key, provider, source_kind, locator, status, last_success_at, last_failure_at,
               last_error, last_item_id, last_item_date, cursor_key, cursor_value, next_run_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6, ?7, NULL, NULL, NULL, NULL, ?8, ?6)
            ON CONFLICT(key) DO UPDATE SET
              provider = excluded.provider,
              source_kind = excluded.source_kind,
              locator = excluded.locator,
              status = excluded.status,
              last_failure_at = excluded.last_failure_at,
              last_error = excluded.last_error,
              next_run_at = excluded.next_run_at,
              updated_at = excluded.updated_at
            "#,
            params![
                key,
                provider,
                source_kind,
                locator,
                classification.status,
                updated_at,
                excerpt(&error, 2000),
                next_run_at,
            ],
        )?;
        Ok(())
    }

    pub fn enqueue_edge_event(
        &self,
        source: &str,
        idempotency_key: &str,
        payload: Value,
        max_age_seconds: i64,
    ) -> Result<EdgeEvent> {
        validate_key(source)?;
        validate_key(idempotency_key)?;
        let payload_json = serde_json::to_string(&payload)?;
        if payload_json.len() > 64_000 {
            bail!("edge event payload is too large");
        }
        let max_age_seconds = max_age_seconds.clamp(60, 86_400);
        let existing = self
            .conn
            .query_row(
                r#"
                SELECT id, source, idempotency_key, status, payload_json, attempts, max_attempts,
                       leased_until, next_run_at, error, received_at, expires_at, updated_at
                FROM edge_events
                WHERE idempotency_key = ?1
                "#,
                params![idempotency_key],
                edge_event_from_row,
            )
            .optional()?;
        if let Some(existing) = existing {
            return Ok(existing);
        }
        let id = Uuid::new_v4().to_string();
        let received_at = now();
        let expires_at = now_plus_seconds(max_age_seconds);
        self.conn.execute(
            r#"
            INSERT INTO edge_events
              (id, source, idempotency_key, status, payload_json, attempts, max_attempts,
               leased_until, next_run_at, error, received_at, expires_at, updated_at)
            VALUES (?1, ?2, ?3, 'pending', ?4, 0, 3, NULL, NULL, NULL, ?5, ?6, ?5)
            "#,
            params![
                id,
                source,
                idempotency_key,
                payload_json,
                received_at,
                expires_at
            ],
        )?;
        self.get_edge_event(&id)?
            .with_context(|| format!("inserted edge event not found: {id}"))
    }

    pub fn list_edge_events(&self) -> Result<Vec<EdgeEvent>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, source, idempotency_key, status, payload_json, attempts, max_attempts,
                   leased_until, next_run_at, error, received_at, expires_at, updated_at
            FROM edge_events
            ORDER BY updated_at DESC
            "#,
        )?;
        rows(stmt.query_map([], edge_event_from_row)?)
    }

    pub fn get_edge_event(&self, id: &str) -> Result<Option<EdgeEvent>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, source, idempotency_key, status, payload_json, attempts, max_attempts,
                       leased_until, next_run_at, error, received_at, expires_at, updated_at
                FROM edge_events
                WHERE id = ?1
                "#,
                params![id],
                edge_event_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn drain_remote_edge_inbox(
        &self,
        base_url: &str,
        secret: &str,
        max_events: usize,
    ) -> Result<EdgeRemoteDrainReport> {
        validate_notes(base_url)?;
        validate_notes(secret)?;
        let base = Url::parse(base_url).with_context(|| format!("invalid edge URL: {base_url}"))?;
        if base.scheme() != "https" && !is_loopback_host(&base) {
            bail!("remote edge URL must use https unless it is loopback");
        }
        self.require_cost_budget(
            "arcwell-edge-inbox",
            "edge_remote_drain",
            "edge",
            "remote_drain",
            Some("edge_remote_drain"),
            estimated_network_fetch_cost(max_events.clamp(1, 100)),
            "remote edge drain",
        )?;
        let client = Client::builder().timeout(Duration::from_secs(20)).build()?;
        let mut imported_events = Vec::new();
        let mut attempted = 0;
        let mut imported = 0;
        let mut acked = 0;
        let mut nacked = 0;
        let mut empty = false;
        for _ in 0..max_events.clamp(1, 100) {
            let lease_url = base.join("/drain/lease")?;
            let lease: Value = client
                .post(lease_url)
                .header("x-arcwell-edge-secret", secret)
                .json(&json!({ "leaseSeconds": 120 }))
                .send()
                .context("remote edge lease request failed")?
                .error_for_status()
                .context("remote edge lease returned an error status")?
                .json()
                .context("remote edge lease returned invalid JSON")?;
            let Some(remote_event) = lease.get("event").filter(|event| !event.is_null()) else {
                empty = true;
                break;
            };
            attempted += 1;
            let source = remote_event
                .get("source")
                .and_then(Value::as_str)
                .context("remote edge event missing source")?;
            let idempotency_key = remote_event
                .get("idempotencyKey")
                .or_else(|| remote_event.get("idempotency_key"))
                .and_then(Value::as_str)
                .context("remote edge event missing idempotency key")?;
            let payload = remote_event.get("payload").cloned().unwrap_or(Value::Null);
            match self.enqueue_edge_event(source, idempotency_key, payload, 86_400) {
                Ok(local) => {
                    imported += 1;
                    imported_events.push(local);
                    let ack_url = base.join("/drain/ack")?;
                    client
                        .post(ack_url)
                        .header("x-arcwell-edge-secret", secret)
                        .json(&json!({ "idempotencyKey": idempotency_key }))
                        .send()
                        .context("remote edge ack request failed")?
                        .error_for_status()
                        .context("remote edge ack returned an error status")?;
                    acked += 1;
                }
                Err(error) => {
                    let nack_url = base.join("/drain/nack")?;
                    client
                        .post(nack_url)
                        .header("x-arcwell-edge-secret", secret)
                        .json(&json!({
                            "idempotencyKey": idempotency_key,
                            "error": error.to_string(),
                            "retrySeconds": 60
                        }))
                        .send()
                        .context("remote edge nack request failed")?
                        .error_for_status()
                        .context("remote edge nack returned an error status")?;
                    nacked += 1;
                }
            }
        }
        Ok(EdgeRemoteDrainReport {
            attempted,
            imported,
            acked,
            nacked,
            empty,
            events: imported_events,
        })
    }

    pub fn lease_edge_event(&self) -> Result<Option<EdgeEvent>> {
        self.lease_edge_event_matching(None)
    }

    pub fn lease_edge_event_for_source(&self, source: &str) -> Result<Option<EdgeEvent>> {
        validate_key(source)?;
        self.lease_edge_event_matching(Some(source))
    }

    pub(crate) fn lease_edge_event_matching(
        &self,
        source: Option<&str>,
    ) -> Result<Option<EdgeEvent>> {
        let timestamp = now();
        self.mark_expired_edge_events(&timestamp)?;
        let event = if let Some(source) = source {
            self.conn.query_row(
                r#"
                SELECT id, source, idempotency_key, status, payload_json, attempts, max_attempts,
                       leased_until, next_run_at, error, received_at, expires_at, updated_at
                FROM edge_events
                WHERE source = ?2
                AND (
                    status = 'pending'
                    OR (status = 'failed' AND (next_run_at IS NULL OR next_run_at <= ?1))
                    OR (status = 'leased' AND leased_until IS NOT NULL AND leased_until <= ?1)
                )
                AND attempts < max_attempts
                AND expires_at > ?1
                ORDER BY received_at ASC
                LIMIT 1
                "#,
                params![timestamp, source],
                edge_event_from_row,
            )
        } else {
            self.conn.query_row(
                r#"
                SELECT id, source, idempotency_key, status, payload_json, attempts, max_attempts,
                       leased_until, next_run_at, error, received_at, expires_at, updated_at
                FROM edge_events
                WHERE (
                    status = 'pending'
                    OR (status = 'failed' AND (next_run_at IS NULL OR next_run_at <= ?1))
                    OR (status = 'leased' AND leased_until IS NOT NULL AND leased_until <= ?1)
                )
                AND attempts < max_attempts
                AND expires_at > ?1
                ORDER BY received_at ASC
                LIMIT 1
                "#,
                params![timestamp],
                edge_event_from_row,
            )
        }
        .optional()?;
        let Some(event) = event else {
            return Ok(None);
        };
        let changed = self.conn.execute(
            r#"
            UPDATE edge_events
            SET status = 'leased',
                attempts = attempts + 1,
                leased_until = ?2,
                next_run_at = NULL,
                updated_at = ?3
            WHERE id = ?1
              AND (
                status = 'pending'
                OR (status = 'failed' AND (next_run_at IS NULL OR next_run_at <= ?4))
                OR (status = 'leased' AND leased_until IS NOT NULL AND leased_until <= ?4)
              )
              AND attempts < max_attempts
              AND expires_at > ?4
            "#,
            params![event.id, now_plus_seconds(300), now(), timestamp],
        )?;
        if changed == 0 {
            return Ok(None);
        }
        self.get_edge_event(&event.id)
    }

    pub fn ack_edge_event(&self, id: &str) -> Result<EdgeEvent> {
        validate_id(id)?;
        self.conn.execute(
            r#"
            UPDATE edge_events
            SET status = 'acked', leased_until = NULL, next_run_at = NULL, error = NULL, updated_at = ?2
            WHERE id = ?1
            "#,
            params![id, now()],
        )?;
        self.get_edge_event(id)?
            .with_context(|| format!("acked edge event not found: {id}"))
    }

    pub fn nack_edge_event(&self, id: &str, error: &str) -> Result<EdgeEvent> {
        validate_id(id)?;
        validate_notes(error)?;
        let event = self
            .get_edge_event(id)?
            .with_context(|| format!("edge event not found: {id}"))?;
        let dead_letter = event.attempts >= event.max_attempts;
        let status = if dead_letter {
            "dead_lettered"
        } else {
            "failed"
        };
        let next_run_at = if dead_letter {
            None
        } else {
            Some(now_plus_seconds(retry_backoff_seconds(event.attempts)))
        };
        self.conn.execute(
            r#"
            UPDATE edge_events
            SET status = ?2,
                leased_until = NULL,
                next_run_at = ?3,
                error = ?4,
                updated_at = ?5
            WHERE id = ?1
            "#,
            params![id, status, next_run_at, excerpt(error, 2000), now()],
        )?;
        self.get_edge_event(id)?
            .with_context(|| format!("nacked edge event not found: {id}"))
    }

    pub fn dead_letter_edge_event(&self, id: &str, error: &str) -> Result<EdgeEvent> {
        validate_id(id)?;
        validate_notes(error)?;
        self.conn.execute(
            r#"
            UPDATE edge_events
            SET status = 'dead_lettered',
                leased_until = NULL,
                next_run_at = NULL,
                error = ?2,
                updated_at = ?3
            WHERE id = ?1
            "#,
            params![id, excerpt(error, 2000), now()],
        )?;
        self.get_edge_event(id)?
            .with_context(|| format!("dead-lettered edge event not found: {id}"))
    }
}
