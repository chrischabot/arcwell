use super::*;

impl Store {
    fn email_delivery_verification_job_input(
        limit: usize,
        verification_state: Option<&str>,
        destination: Option<&str>,
        requested_by: &str,
    ) -> Result<Value> {
        if let Some(verification_state) = verification_state {
            validate_key(verification_state)?;
        }
        if let Some(destination) = destination {
            validate_query(destination)?;
        }
        validate_key(requested_by)?;
        Ok(json!({
            "limit": limit.clamp(1, 100),
            "verification_state": verification_state.unwrap_or("mailbox_unverified"),
            "destination": destination,
            "requested_by": requested_by
        }))
    }

    fn email_delivery_mailbox_repair_job_input(
        limit: usize,
        verification_state: &str,
        destination: Option<&str>,
        requested_by: &str,
    ) -> Result<Value> {
        validate_key(verification_state)?;
        if let Some(destination) = destination {
            validate_query(destination)?;
        }
        validate_key(requested_by)?;
        Ok(json!({
            "limit": limit.clamp(1, 100),
            "verification_state": verification_state,
            "destination": destination,
            "requested_by": requested_by
        }))
    }

    pub fn record_channel_message(
        &self,
        channel: &str,
        direction: &str,
        sender: &str,
        body: &str,
        project_id: Option<&str>,
        source_event_id: Option<&str>,
    ) -> Result<ChannelMessage> {
        self.record_channel_message_with_status(
            channel,
            direction,
            sender,
            body,
            "recorded",
            project_id,
            source_event_id,
        )
    }

    // allow: refactoring this N-arg signature is out of scope for the lint-cleanup pass.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_channel_message_with_status(
        &self,
        channel: &str,
        direction: &str,
        sender: &str,
        body: &str,
        status: &str,
        project_id: Option<&str>,
        source_event_id: Option<&str>,
    ) -> Result<ChannelMessage> {
        validate_key(channel)?;
        validate_channel_direction(direction)?;
        validate_query(sender)?;
        if channel == "email" {
            validate_email_body_text(body)?;
        } else {
            validate_notes(body)?;
        }
        validate_key(status)?;
        if let Some(project_id) = project_id {
            validate_id(project_id)?;
            self.get_project(project_id)?
                .with_context(|| format!("project not found: {project_id}"))?;
        }
        let sanitized_body = if channel == "email" {
            sanitize_email_body(body)?
        } else {
            sanitize_channel_body(body)?
        };
        let id = Uuid::new_v4().to_string();
        let timestamp = now();
        self.conn.execute(
            r#"
            INSERT INTO channel_messages
              (id, channel, direction, project_id, sender, body, status, source_event_id, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                id,
                channel,
                direction,
                project_id,
                sender,
                sanitized_body,
                status,
                source_event_id,
                timestamp
            ],
        )?;
        self.get_channel_message(&id)?
            .with_context(|| format!("inserted channel message not found: {id}"))
    }

    pub(crate) fn update_channel_message_status(
        &self,
        id: &str,
        status: &str,
    ) -> Result<ChannelMessage> {
        validate_id(id)?;
        validate_key(status)?;
        self.conn.execute(
            "UPDATE channel_messages SET status = ?2 WHERE id = ?1",
            params![id, status],
        )?;
        self.get_channel_message(id)?
            .with_context(|| format!("channel message not found after status update: {id}"))
    }

    pub fn list_channel_messages(&self) -> Result<Vec<ChannelMessage>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, channel, direction, project_id, sender, body, status, source_event_id, created_at FROM channel_messages ORDER BY created_at DESC",
        )?;
        rows(stmt.query_map([], channel_message_from_row)?)
    }

    pub fn get_channel_message(&self, id: &str) -> Result<Option<ChannelMessage>> {
        validate_id(id)?;
        self.conn
            .query_row(
                "SELECT id, channel, direction, project_id, sender, body, status, source_event_id, created_at FROM channel_messages WHERE id = ?1",
                params![id],
                channel_message_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn authorize_channel_subject(
        &self,
        channel: &str,
        subject: &str,
        can_read_projects: bool,
        can_write_projects: bool,
        can_send: bool,
    ) -> Result<ChannelAuthorization> {
        validate_key(channel)?;
        validate_query(subject)?;
        let updated_at = now();
        self.conn.execute(
            r#"
            INSERT INTO channel_authorizations
              (channel, subject, can_read_projects, can_write_projects, can_send, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(channel, subject) DO UPDATE SET
              can_read_projects = excluded.can_read_projects,
              can_write_projects = excluded.can_write_projects,
              can_send = excluded.can_send,
              updated_at = excluded.updated_at
            "#,
            params![
                channel,
                subject,
                bool_to_i64(can_read_projects),
                bool_to_i64(can_write_projects),
                bool_to_i64(can_send),
                updated_at
            ],
        )?;
        self.get_channel_authorization(channel, subject)?
            .with_context(|| {
                format!("inserted channel authorization not found: {channel}:{subject}")
            })
    }

    pub fn get_channel_authorization(
        &self,
        channel: &str,
        subject: &str,
    ) -> Result<Option<ChannelAuthorization>> {
        validate_key(channel)?;
        validate_query(subject)?;
        self.conn
            .query_row(
                r#"
                SELECT channel, subject, can_read_projects, can_write_projects, can_send, updated_at
                FROM channel_authorizations
                WHERE channel = ?1 AND subject = ?2
                "#,
                params![channel, subject],
                channel_authorization_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_channel_authorizations(&self) -> Result<Vec<ChannelAuthorization>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT channel, subject, can_read_projects, can_write_projects, can_send, updated_at
            FROM channel_authorizations
            ORDER BY channel ASC, subject ASC
            "#,
        )?;
        rows(stmt.query_map([], channel_authorization_from_row)?)
    }

    pub fn channel_subject_can_write_projects(
        &self,
        channel: &str,
        subjects: &[String],
    ) -> Result<bool> {
        validate_key(channel)?;
        for subject in subjects {
            validate_query(subject)?;
            if self
                .get_channel_authorization(channel, subject)?
                .is_some_and(|authorization| authorization.can_write_projects)
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn channel_subject_can_read_projects(&self, channel: &str, subject: &str) -> Result<bool> {
        validate_key(channel)?;
        validate_query(subject)?;
        Ok(self
            .get_channel_authorization(channel, subject)?
            .is_some_and(|authorization| authorization.can_read_projects))
    }

    pub fn channel_subject_can_send(&self, channel: &str, subject: &str) -> Result<bool> {
        validate_key(channel)?;
        validate_query(subject)?;
        Ok(self
            .get_channel_authorization(channel, subject)?
            .is_some_and(|authorization| authorization.can_send))
    }

    pub(crate) fn email_sender_is_configured_author(&self, sender: &str) -> Result<bool> {
        let sender = normalize_email_address(sender).context("invalid email sender")?;
        Ok(configured_author_emails(self)?
            .iter()
            .any(|author| author == &sender))
    }

    // allow: refactoring this N-arg signature is out of scope for the lint-cleanup pass.
    #[allow(clippy::too_many_arguments)]
    pub fn record_channel_delivery_attempt(
        &self,
        message_id: &str,
        channel: &str,
        destination: &str,
        ok: bool,
        provider_status: i64,
        response: &Value,
        error: Option<&str>,
        retry_at: Option<&str>,
    ) -> Result<ChannelDeliveryAttempt> {
        validate_id(message_id)?;
        self.get_channel_message(message_id)?
            .with_context(|| format!("channel message not found: {message_id}"))?;
        validate_key(channel)?;
        validate_query(destination)?;
        if let Some(error) = error {
            validate_notes(error)?;
        }
        if let Some(retry_at) = retry_at {
            DateTime::parse_from_rfc3339(retry_at)
                .with_context(|| format!("parsing retry_at timestamp {retry_at}"))?;
        }
        let response_json = serde_json::to_string(response)?;
        let id = Uuid::new_v4().to_string();
        let created_at = now();

        // Compute the next attempt number and consume it in one BEGIN IMMEDIATE
        // transaction so two concurrent connections recording an attempt for the
        // same message_id cannot both read the same MAX(attempt) and insert a
        // duplicate attempt number.
        let record_result = (|| -> Result<()> {
            self.conn.execute("BEGIN IMMEDIATE", [])?;
            let attempt: i64 = self.conn.query_row(
                "SELECT COALESCE(MAX(attempt), 0) + 1 FROM channel_delivery_attempts WHERE message_id = ?1",
                params![message_id],
                |row| row.get(0),
            )?;
            self.conn.execute(
                r#"
                INSERT INTO channel_delivery_attempts
                  (id, message_id, channel, destination, attempt, ok, provider_status, response_json, error, retry_at, created_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                "#,
                params![
                    id,
                    message_id,
                    channel,
                    destination,
                    attempt,
                    bool_to_i64(ok),
                    provider_status,
                    response_json,
                    error,
                    retry_at,
                    created_at
                ],
            )?;
            self.conn.execute("COMMIT", [])?;
            Ok(())
        })();
        if let Err(error) = record_result {
            let _ = self.conn.execute("ROLLBACK", []);
            return Err(error);
        }
        self.get_channel_delivery_attempt(&id)?
            .with_context(|| format!("inserted channel delivery attempt not found: {id}"))
    }

    pub fn get_channel_delivery_attempt(&self, id: &str) -> Result<Option<ChannelDeliveryAttempt>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, message_id, channel, destination, attempt, ok, provider_status,
                       response_json, error, retry_at, created_at
                FROM channel_delivery_attempts
                WHERE id = ?1
                "#,
                params![id],
                channel_delivery_attempt_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_channel_delivery_attempts(
        &self,
        message_id: Option<&str>,
    ) -> Result<Vec<ChannelDeliveryAttempt>> {
        if let Some(message_id) = message_id {
            validate_id(message_id)?;
            let mut stmt = self.conn.prepare(
                r#"
                SELECT id, message_id, channel, destination, attempt, ok, provider_status,
                       response_json, error, retry_at, created_at
                FROM channel_delivery_attempts
                WHERE message_id = ?1
                ORDER BY created_at DESC
                "#,
            )?;
            return rows(stmt.query_map(params![message_id], channel_delivery_attempt_from_row)?);
        }
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, message_id, channel, destination, attempt, ok, provider_status,
                   response_json, error, retry_at, created_at
            FROM channel_delivery_attempts
            ORDER BY created_at DESC
            "#,
        )?;
        rows(stmt.query_map([], channel_delivery_attempt_from_row)?)
    }

    // allow: refactoring this N-arg signature is out of scope for the lint-cleanup pass.
    #[allow(clippy::too_many_arguments)]
    pub fn record_channel_delivery_observation(
        &self,
        delivery_attempt_id: &str,
        observation_source: &str,
        observation_status: &str,
        mailbox_message_id: Option<&str>,
        provider_message_id: Option<&str>,
        observed_at: Option<&str>,
        evidence: &Value,
    ) -> Result<ChannelDeliveryObservation> {
        validate_id(delivery_attempt_id)?;
        validate_key(observation_source)?;
        validate_delivery_observation_status(observation_status)?;
        if let Some(mailbox_message_id) = mailbox_message_id {
            validate_query(mailbox_message_id)?;
        }
        if let Some(provider_message_id) = provider_message_id {
            validate_query(provider_message_id)?;
        }
        let observed_at = observed_at.map(str::to_string).unwrap_or_else(now);
        DateTime::parse_from_rfc3339(&observed_at)
            .with_context(|| format!("parsing observed_at timestamp {observed_at}"))?;
        let evidence = sanitize_delivery_observation_evidence(evidence)?;
        let evidence_json = serde_json::to_string(&evidence)?;
        if evidence_json.len() > 16_384 {
            bail!("delivery observation evidence is too large");
        }
        let attempt = self
            .get_channel_delivery_attempt(delivery_attempt_id)?
            .with_context(|| {
                format!("channel delivery attempt not found: {delivery_attempt_id}")
            })?;
        self.record_email_delivery_mailbox_observation_health(
            &attempt,
            observation_status,
            &evidence,
        )?;
        let id = Uuid::new_v4().to_string();
        let created_at = now();
        self.conn.execute(
            r#"
            INSERT INTO channel_delivery_observations
              (id, delivery_attempt_id, message_id, channel, destination, provider_message_id,
               observation_source, observation_status, mailbox_message_id, observed_at, evidence_json, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
            params![
                id,
                delivery_attempt_id,
                attempt.message_id,
                attempt.channel,
                attempt.destination,
                provider_message_id.or(attempt.provider_message_id.as_deref()),
                observation_source,
                observation_status,
                mailbox_message_id,
                observed_at,
                evidence_json,
                created_at
            ],
        )?;
        self.get_channel_delivery_observation(&id)?
            .with_context(|| format!("inserted channel delivery observation not found: {id}"))
    }

    fn record_email_delivery_mailbox_observation_health(
        &self,
        attempt: &ChannelDeliveryAttempt,
        observation_status: &str,
        evidence: &Value,
    ) -> Result<()> {
        if attempt.channel != "email" {
            return Ok(());
        }
        let Some(placement) = mailbox_observation_placement(observation_status, evidence) else {
            return Ok(());
        };
        let key = email_delivery_mailbox_observation_health_key(&attempt.id);
        match placement.as_str() {
            "inbox" => self.record_source_success(SourceHealthUpdate {
                key: &key,
                provider: "gmail",
                source_kind: "email_delivery_mailbox_observation",
                locator: &attempt.id,
                last_item_id: attempt
                    .outbound_message_id
                    .as_deref()
                    .or(attempt.provider_message_id.as_deref()),
                last_item_date: Some(&attempt.created_at),
                cursor_key: None,
                cursor_value: None,
                next_run_at: None,
            }),
            "trash" | "spam" | "not_observed" | "unknown" => self.record_source_failure(
                &key,
                "gmail",
                "email_delivery_mailbox_observation",
                &attempt.id,
                &format!(
                    "email delivery mailbox verification requires attention: delivery_attempt_id={} placement={} status={} destination={}",
                    attempt.id, placement, observation_status, attempt.destination
                ),
            ),
            _ => Ok(()),
        }
    }

    pub fn get_channel_delivery_observation(
        &self,
        id: &str,
    ) -> Result<Option<ChannelDeliveryObservation>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, delivery_attempt_id, message_id, channel, destination, provider_message_id,
                       observation_source, observation_status, mailbox_message_id, observed_at,
                       evidence_json, created_at
                FROM channel_delivery_observations
                WHERE id = ?1
                "#,
                params![id],
                channel_delivery_observation_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_channel_delivery_observations(
        &self,
        delivery_attempt_id: Option<&str>,
    ) -> Result<Vec<ChannelDeliveryObservation>> {
        if let Some(delivery_attempt_id) = delivery_attempt_id {
            validate_id(delivery_attempt_id)?;
            let mut stmt = self.conn.prepare(
                r#"
                SELECT id, delivery_attempt_id, message_id, channel, destination, provider_message_id,
                       observation_source, observation_status, mailbox_message_id, observed_at,
                       evidence_json, created_at
                FROM channel_delivery_observations
                WHERE delivery_attempt_id = ?1
                ORDER BY observed_at DESC, created_at DESC
                "#,
            )?;
            return rows(stmt.query_map(
                params![delivery_attempt_id],
                channel_delivery_observation_from_row,
            )?);
        }
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, delivery_attempt_id, message_id, channel, destination, provider_message_id,
                   observation_source, observation_status, mailbox_message_id, observed_at,
                   evidence_json, created_at
            FROM channel_delivery_observations
            ORDER BY observed_at DESC, created_at DESC
            "#,
        )?;
        rows(stmt.query_map([], channel_delivery_observation_from_row)?)
    }

    pub(crate) fn latest_channel_delivery_observation(
        &self,
        delivery_attempt_id: &str,
    ) -> Result<Option<ChannelDeliveryObservation>> {
        validate_id(delivery_attempt_id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, delivery_attempt_id, message_id, channel, destination, provider_message_id,
                       observation_source, observation_status, mailbox_message_id, observed_at,
                       evidence_json, created_at
                FROM channel_delivery_observations
                WHERE delivery_attempt_id = ?1
                ORDER BY observed_at DESC, created_at DESC
                LIMIT 1
                "#,
                params![delivery_attempt_id],
                channel_delivery_observation_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_email_delivery_verification_gaps(
        &self,
    ) -> Result<Vec<EmailDeliveryVerificationGap>> {
        let mut gaps = Vec::new();
        for attempt in self.list_channel_delivery_attempts(None)? {
            if attempt.channel != "email" || !attempt.ok {
                continue;
            }
            if !attempt.delivery_proof.ends_with("_mailbox_unverified") {
                continue;
            }
            let observation = self.latest_channel_delivery_observation(&attempt.id)?;
            let verification_state = observation
                .as_ref()
                .map(email_delivery_mailbox_verification_state)
                .unwrap_or_else(|| "mailbox_unverified".to_string());
            if verification_state == "mailbox_observed_inbox" {
                continue;
            }
            gaps.push(EmailDeliveryVerificationGap {
                delivery_attempt_id: attempt.id,
                message_id: attempt.message_id,
                destination: attempt.destination,
                provider_message_id: attempt.provider_message_id,
                outbound_message_id: attempt.outbound_message_id,
                provider_status: attempt.provider_status,
                provider_delivery_proof: attempt.delivery_proof,
                latest_observation_status: observation
                    .as_ref()
                    .map(|item| item.observation_status.clone()),
                latest_observation_at: observation.as_ref().map(|item| item.observed_at.clone()),
                verification_state,
                created_at: attempt.created_at,
            });
        }
        Ok(gaps)
    }

    pub fn build_email_delivery_verification_requests(
        &self,
        limit: usize,
        verification_state: Option<&str>,
        destination: Option<&str>,
    ) -> Result<Vec<EmailDeliveryVerificationRequest>> {
        if let Some(verification_state) = verification_state {
            validate_key(verification_state)?;
        }
        if let Some(destination) = destination {
            validate_query(destination)?;
        }
        let mut requests = Vec::new();
        for gap in self.list_email_delivery_verification_gaps()? {
            if verification_state.is_some_and(|state| gap.verification_state.as_str() != state) {
                continue;
            }
            if destination.is_some_and(|target| gap.destination != target) {
                continue;
            }
            let (search_query, ready, reason) = email_delivery_verification_search_query(
                gap.outbound_message_id
                    .as_deref()
                    .or(gap.provider_message_id.as_deref()),
            );
            requests.push(EmailDeliveryVerificationRequest {
                delivery_attempt_id: gap.delivery_attempt_id,
                message_id: gap.message_id,
                destination: gap.destination,
                provider_message_id: gap.provider_message_id,
                outbound_message_id: gap.outbound_message_id,
                provider_status: gap.provider_status,
                provider_delivery_proof: gap.provider_delivery_proof,
                verification_state: gap.verification_state,
                created_at: gap.created_at,
                observation_source: "gmail".to_string(),
                search_query,
                ready,
                reason,
            });
            if requests.len() >= limit.clamp(1, 100) {
                break;
            }
        }
        Ok(requests)
    }

    pub fn enqueue_email_delivery_verification_request_job(
        &self,
        limit: usize,
        verification_state: Option<&str>,
        destination: Option<&str>,
    ) -> Result<WikiJob> {
        let input = Self::email_delivery_verification_job_input(
            limit,
            verification_state,
            destination,
            "manual",
        )?;
        self.enqueue_wiki_job("email_delivery_verification_request", input)
    }

    pub fn enqueue_due_email_delivery_verification_jobs(
        &self,
        limit: usize,
    ) -> Result<EmailDeliveryVerificationEnqueueReport> {
        let limit = limit.clamp(1, 100);
        let minimum_age_seconds = 5 * 60;
        let throttle_seconds = 15 * 60;
        let minimum_created_at = now_plus_seconds(-minimum_age_seconds);
        let gaps = self.list_email_delivery_verification_gaps()?;
        let matching_gap_count = gaps
            .iter()
            .filter(|gap| {
                gap.verification_state == "mailbox_unverified"
                    && gap.created_at <= minimum_created_at
            })
            .count();
        let mut report = EmailDeliveryVerificationEnqueueReport {
            inspected: matching_gap_count,
            enqueued: 0,
            skipped: 0,
            jobs: Vec::new(),
            errors: Vec::new(),
            gap_count: gaps.len(),
            request_count: 0,
            active_job_id: None,
            recent_job_id: None,
            minimum_age_seconds,
            throttle_seconds,
        };
        if matching_gap_count == 0 {
            return Ok(report);
        }

        let mut input = Self::email_delivery_verification_job_input(
            limit,
            Some("mailbox_unverified"),
            None,
            "worker",
        )?;
        if let Value::Object(map) = &mut input {
            map.insert(
                "minimum_age_seconds".to_string(),
                json!(minimum_age_seconds),
            );
        }
        report.request_count = self
            .build_email_delivery_verification_requests(limit, Some("mailbox_unverified"), None)?
            .into_iter()
            .filter(|request| request.created_at <= minimum_created_at)
            .count();
        if let Some(job) =
            self.find_active_duplicate_wiki_job("email_delivery_verification_request", &input)?
        {
            report.skipped += 1;
            report.active_job_id = Some(job.id);
            return Ok(report);
        }
        if let Some(job) = self.recent_email_delivery_verification_request_job(throttle_seconds)? {
            report.skipped += 1;
            report.recent_job_id = Some(job.id);
            return Ok(report);
        }

        match self.enqueue_wiki_job("email_delivery_verification_request", input) {
            Ok(job) => {
                report.enqueued += 1;
                report.jobs.push(job.id);
            }
            Err(error) => {
                report.skipped += 1;
                report.errors.push(error.to_string());
            }
        }
        Ok(report)
    }

    pub(crate) fn recent_email_delivery_verification_request_job(
        &self,
        throttle_seconds: i64,
    ) -> Result<Option<WikiJob>> {
        let cutoff = now_plus_seconds(-throttle_seconds.abs());
        self.conn
            .query_row(
                r#"
                SELECT id, kind, status, input_json, result_json, error,
                       attempts, max_attempts, leased_until, worker_id, next_run_at, dead_lettered_at,
                       created_at, updated_at
                FROM wiki_jobs
                WHERE kind = 'email_delivery_verification_request'
                  AND status IN ('pending', 'running', 'deferred', 'completed')
                  AND updated_at >= ?1
                ORDER BY updated_at DESC, created_at DESC
                LIMIT 1
                "#,
                params![cutoff],
                wiki_job_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn enqueue_email_delivery_mailbox_repair_job(
        &self,
        limit: usize,
        verification_state: Option<&str>,
        destination: Option<&str>,
    ) -> Result<WikiJob> {
        let input = Self::email_delivery_mailbox_repair_job_input(
            limit,
            verification_state.unwrap_or("mailbox_bad_placement_trash"),
            destination,
            "manual",
        )?;
        self.enqueue_wiki_job("email_delivery_mailbox_repair", input)
    }

    pub fn email_delivery_recovery_plan(
        &self,
        limit: usize,
        verification_state: Option<&str>,
        destination: Option<&str>,
    ) -> Result<EmailDeliveryRecoveryPlan> {
        if let Some(verification_state) = verification_state {
            validate_key(verification_state)?;
        }
        if let Some(destination) = destination {
            validate_query(destination)?;
        }
        let limit = limit.clamp(1, 100);
        let mut items = Vec::new();
        let mut counts_by_state = BTreeMap::new();
        let mut automatic_verification_candidates = 0;
        let mut automatic_repair_candidates = 0;
        let mut explicit_resend_review_candidates = 0;
        let mut manual_review_candidates = 0;

        for gap in self.list_email_delivery_verification_gaps()? {
            if verification_state.is_some_and(|state| gap.verification_state != state) {
                continue;
            }
            if destination.is_some_and(|target| gap.destination != target) {
                continue;
            }
            *counts_by_state
                .entry(gap.verification_state.clone())
                .or_insert(0) += 1;
            let (
                recommended_action,
                automatic_worker_action,
                requires_explicit_resend_approval,
                reason,
            ) = match gap.verification_state.as_str() {
                "mailbox_unverified" => {
                    automatic_verification_candidates += 1;
                    (
                        "verify_mailbox",
                        Some("email_delivery_verification_request"),
                        false,
                        "Provider accepted the send, but no mailbox observation exists yet; verify with Gmail or a host mailbox connector before considering resend.",
                    )
                }
                "mailbox_bad_placement_trash" | "mailbox_bad_placement_spam" => {
                    automatic_repair_candidates += 1;
                    (
                        "repair_mailbox_placement",
                        Some("email_delivery_mailbox_repair"),
                        false,
                        "Mailbox proof found the message in a bad placement; repair labels with Gmail modify scope instead of resending a duplicate.",
                    )
                }
                "mailbox_not_observed" | "mailbox_unknown" => {
                    explicit_resend_review_candidates += 1;
                    (
                        "explicit_resend_review",
                        None,
                        true,
                        "Mailbox verification did not prove user-visible receipt; resend is a new user-visible delivery and requires explicit approval after checking content, idempotency, and policy.",
                    )
                }
                _ => {
                    manual_review_candidates += 1;
                    (
                        "manual_review",
                        None,
                        true,
                        "Arcwell does not have an automatic recovery action for this mailbox state; inspect the latest observation and choose an explicit operator action.",
                    )
                }
            };
            items.push(EmailDeliveryRecoveryPlanItem {
                delivery_attempt_id: gap.delivery_attempt_id,
                message_id: gap.message_id,
                destination: gap.destination,
                provider_message_id: gap.provider_message_id,
                outbound_message_id: gap.outbound_message_id,
                verification_state: gap.verification_state,
                recommended_action: recommended_action.to_string(),
                automatic_worker_action: automatic_worker_action.map(str::to_string),
                requires_explicit_resend_approval,
                reason: reason.to_string(),
            });
            if items.len() >= limit {
                break;
            }
        }

        Ok(EmailDeliveryRecoveryPlan {
            inspected: items.len(),
            counts_by_state,
            automatic_verification_candidates,
            automatic_repair_candidates,
            explicit_resend_review_candidates,
            manual_review_candidates,
            items,
            boundary: "Read-only recovery plan. This does not send, resend, repair labels, read message bodies, or mark mailbox observations. It classifies current delivery verification gaps so an operator or worker can choose the least surprising next action.".to_string(),
        })
    }

    pub fn enqueue_due_email_delivery_mailbox_repair_jobs(
        &self,
        limit: usize,
    ) -> Result<EmailMailboxPlacementRepairEnqueueReport> {
        let limit = limit.clamp(1, 100);
        let throttle_seconds = 15 * 60;
        let gaps = self.list_email_delivery_verification_gaps()?;
        let repairable_count = gaps
            .iter()
            .filter(|gap| {
                matches!(
                    gap.verification_state.as_str(),
                    "mailbox_bad_placement_trash" | "mailbox_bad_placement_spam"
                )
            })
            .count();
        let mut report = EmailMailboxPlacementRepairEnqueueReport {
            inspected: repairable_count,
            enqueued: 0,
            skipped: 0,
            jobs: Vec::new(),
            errors: Vec::new(),
            gap_count: gaps.len(),
            repairable_count,
            active_job_id: None,
            recent_job_id: None,
            throttle_seconds,
        };
        if repairable_count == 0 {
            return Ok(report);
        }

        let verification_state = if gaps
            .iter()
            .any(|gap| gap.verification_state == "mailbox_bad_placement_trash")
        {
            "mailbox_bad_placement_trash"
        } else {
            "mailbox_bad_placement_spam"
        };
        let input = Self::email_delivery_mailbox_repair_job_input(
            limit,
            verification_state,
            None,
            "worker",
        )?;
        if let Some(job) =
            self.find_active_duplicate_wiki_job("email_delivery_mailbox_repair", &input)?
        {
            report.skipped += 1;
            report.active_job_id = Some(job.id);
            return Ok(report);
        }
        if let Some(job) = self.recent_email_delivery_mailbox_repair_job(throttle_seconds)? {
            report.skipped += 1;
            report.recent_job_id = Some(job.id);
            return Ok(report);
        }

        match self.enqueue_wiki_job("email_delivery_mailbox_repair", input) {
            Ok(job) => {
                report.enqueued += 1;
                report.jobs.push(job.id);
            }
            Err(error) => {
                report.skipped += 1;
                report.errors.push(error.to_string());
            }
        }
        Ok(report)
    }

    pub(crate) fn recent_email_delivery_mailbox_repair_job(
        &self,
        throttle_seconds: i64,
    ) -> Result<Option<WikiJob>> {
        let cutoff = now_plus_seconds(-throttle_seconds.abs());
        self.conn
            .query_row(
                r#"
                SELECT id, kind, status, input_json, result_json, error,
                       attempts, max_attempts, leased_until, worker_id, next_run_at, dead_lettered_at,
                       created_at, updated_at
                FROM wiki_jobs
                WHERE kind = 'email_delivery_mailbox_repair'
                  AND status IN ('pending', 'running', 'deferred', 'completed')
                  AND updated_at >= ?1
                ORDER BY updated_at DESC, created_at DESC
                LIMIT 1
                "#,
                params![cutoff],
                wiki_job_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub(crate) fn execute_email_delivery_verification_request(
        &self,
        input: &Value,
    ) -> Result<Value> {
        let limit = input.get("limit").and_then(Value::as_u64).unwrap_or(25) as usize;
        let verification_state = input
            .get("verification_state")
            .or_else(|| input.get("state"))
            .and_then(Value::as_str)
            .unwrap_or("mailbox_unverified");
        let destination = input.get("destination").and_then(Value::as_str);
        let minimum_age_seconds = input
            .get("minimum_age_seconds")
            .and_then(Value::as_i64)
            .filter(|seconds| *seconds > 0)
            .unwrap_or(0);
        let minimum_created_at =
            (minimum_age_seconds > 0).then(|| now_plus_seconds(-minimum_age_seconds));
        let mut requests = self.build_email_delivery_verification_requests(
            limit,
            Some(verification_state),
            destination,
        )?;
        if let Some(minimum_created_at) = minimum_created_at.as_ref() {
            requests.retain(|request| request.created_at <= *minimum_created_at);
        }
        let ready_count = requests.iter().filter(|request| request.ready).count();
        if let Some(access_token) = self.configured_gmail_access_token()? {
            let api_base = self.configured_gmail_api_base()?;
            let report = self.verify_email_delivery_requests_with_gmail(
                requests,
                &access_token,
                api_base.as_deref(),
            )?;
            return Ok(json!({
                "action": "email_delivery_mailbox_verify",
                "status": "mailbox_verification_complete",
                "boundary": "This job used the configured Gmail API credential to search the mailbox and record mailbox_observed or mailbox_not_found observations. Provider acceptance and mailbox observation remain separate proof layers.",
                "verification_state": verification_state,
                "destination": destination,
                "minimum_age_seconds": minimum_age_seconds,
                "report": report
            }));
        }
        if !requests.is_empty() {
            self.record_gmail_mailbox_verifier_missing_credential()?;
        }
        Ok(json!({
            "action": "email_delivery_verification_request",
            "status": if requests.is_empty() { "no_matching_gaps" } else { "requests_ready" },
            "boundary": "This job prepares host mailbox verification requests only because GMAIL_ACCESS_TOKEN is not configured. It does not read Gmail or mark mailbox observations; a host verifier must run the search_query values and record results with email_delivery_observation_batch_add or email observe-delivery-batch.",
            "verification_state": verification_state,
            "destination": destination,
            "minimum_age_seconds": minimum_age_seconds,
            "request_count": requests.len(),
            "ready_count": ready_count,
            "requests": requests
        }))
    }

    pub(crate) fn execute_email_delivery_mailbox_repair(&self, input: &Value) -> Result<Value> {
        let limit = input.get("limit").and_then(Value::as_u64).unwrap_or(25) as usize;
        let verification_state = input
            .get("verification_state")
            .or_else(|| input.get("state"))
            .and_then(Value::as_str)
            .unwrap_or("mailbox_bad_placement_trash");
        let destination = input.get("destination").and_then(Value::as_str);
        let api_base = self.configured_gmail_api_base()?;
        let report = self.repair_email_delivery_mailbox_placement_with_gmail(
            limit,
            Some(verification_state),
            destination,
            None,
            api_base.as_deref(),
        )?;
        Ok(json!({
            "action": "email_delivery_mailbox_repair",
            "status": if report.missing_credential {
                "missing_credential"
            } else if report.repaired > 0 {
                "mailbox_repair_complete"
            } else if report.inspected == 0 {
                "no_matching_gaps"
            } else {
                "mailbox_repair_incomplete"
            },
            "boundary": "This job repairs only already observed bad-placement Arcwell email deliveries by using configured Gmail modify credentials to add INBOX and remove TRASH/SPAM, then records fresh mailbox metadata. It does not resend mail, inspect message bodies, or treat provider acceptance as mailbox proof.",
            "verification_state": verification_state,
            "destination": destination,
            "report": report
        }))
    }

    pub fn verify_email_delivery_mailbox_with_gmail(
        &self,
        limit: usize,
        verification_state: Option<&str>,
        destination: Option<&str>,
        access_token: Option<&str>,
        api_base: Option<&str>,
    ) -> Result<EmailMailboxVerificationReport> {
        if let Some(verification_state) = verification_state {
            validate_key(verification_state)?;
        }
        if let Some(destination) = destination {
            validate_query(destination)?;
        }
        let access_token = match access_token {
            Some(access_token) => Some(access_token.to_string()),
            None => self.configured_gmail_access_token()?,
        };
        let Some(access_token) = access_token else {
            let requests = self.build_email_delivery_verification_requests(
                limit,
                verification_state.or(Some("mailbox_unverified")),
                destination,
            )?;
            if !requests.is_empty() {
                self.record_gmail_mailbox_verifier_missing_credential()?;
            }
            return Ok(EmailMailboxVerificationReport {
                inspected: requests.len(),
                ready: requests.iter().filter(|request| request.ready).count(),
                observed: 0,
                not_found: 0,
                skipped: requests.iter().filter(|request| !request.ready).count(),
                errors: vec!["GMAIL_ACCESS_TOKEN is not configured".to_string()],
                observations: Vec::new(),
                missing_credential: true,
                provider: "gmail".to_string(),
                source_health_key: gmail_mailbox_verifier_source_health_key().to_string(),
            });
        };
        let api_base = match api_base {
            Some(api_base) => Some(api_base.to_string()),
            None => self.configured_gmail_api_base()?,
        };
        let requests = self.build_email_delivery_verification_requests(
            limit,
            verification_state.or(Some("mailbox_unverified")),
            destination,
        )?;
        self.verify_email_delivery_requests_with_gmail(requests, &access_token, api_base.as_deref())
    }

    pub fn repair_email_delivery_mailbox_placement_with_gmail(
        &self,
        limit: usize,
        verification_state: Option<&str>,
        destination: Option<&str>,
        access_token: Option<&str>,
        api_base: Option<&str>,
    ) -> Result<EmailMailboxPlacementRepairReport> {
        let verification_state = verification_state.unwrap_or("mailbox_bad_placement_trash");
        validate_key(verification_state)?;
        if let Some(destination) = destination {
            validate_query(destination)?;
        }
        let source_health_key = gmail_mailbox_repair_source_health_key();
        let limit = limit.clamp(1, 100);
        let gaps = self.list_email_delivery_verification_gaps()?;
        let mut selected = Vec::new();
        for gap in gaps {
            if gap.verification_state != verification_state {
                continue;
            }
            if destination.is_some_and(|target| gap.destination != target) {
                continue;
            }
            selected.push(gap);
            if selected.len() >= limit {
                break;
            }
        }
        let eligible = selected.len();
        let Some(access_token) = access_token
            .map(ToOwned::to_owned)
            .or(self.configured_gmail_access_token()?)
        else {
            if eligible > 0 {
                self.record_source_failure(
                    source_health_key,
                    "gmail",
                    "gmail_mailbox_repair",
                    "me",
                    "GMAIL_ACCESS_TOKEN is not configured; reauthorize with Gmail modify scope before mailbox placement repair can run.",
                )?;
            }
            return Ok(EmailMailboxPlacementRepairReport {
                inspected: eligible,
                eligible,
                repaired: 0,
                skipped: eligible,
                errors: if eligible == 0 {
                    Vec::new()
                } else {
                    vec!["GMAIL_ACCESS_TOKEN is not configured".to_string()]
                },
                observations: Vec::new(),
                missing_credential: eligible > 0,
                provider: "gmail".to_string(),
                source_health_key: source_health_key.to_string(),
            });
        };
        self.repair_email_delivery_mailbox_placements_with_gmail(
            selected,
            &access_token,
            api_base
                .map(ToOwned::to_owned)
                .or(self.configured_gmail_api_base()?)
                .as_deref(),
            source_health_key,
        )
    }

    fn repair_email_delivery_mailbox_placements_with_gmail(
        &self,
        gaps: Vec<EmailDeliveryVerificationGap>,
        access_token: &str,
        api_base: Option<&str>,
        source_health_key: &str,
    ) -> Result<EmailMailboxPlacementRepairReport> {
        validate_notes(access_token)?;
        let api_base = api_base.unwrap_or("https://gmail.googleapis.com");
        let api_base_url = validate_public_http_url(api_base)?;
        let projected_usd = estimated_network_fetch_cost(gaps.len().max(1) * 2);
        let result = (|| -> Result<EmailMailboxPlacementRepairReport> {
            self.policy_guard(PolicyRequest {
                action: "provider.network".to_string(),
                package: Some("arcwell-email".to_string()),
                provider: Some("gmail".to_string()),
                source: Some("email_delivery_mailbox_repair".to_string()),
                channel: Some("email".to_string()),
                subject: None,
                target: Some(api_base_url.as_str().trim_end_matches('/').to_string()),
                projected_usd: Some(projected_usd),
                metadata: json!({
                    "gap_count": gaps.len(),
                    "source_health_key": source_health_key,
                    "operation": "remove_bad_placement_labels_add_inbox"
                }),
                untrusted_excerpt: None,
            })?;
            self.require_cost_budget(
                "arcwell-email",
                "email_delivery_mailbox_repair",
                "gmail",
                "mailbox_repair",
                Some("email_delivery_mailbox_repair"),
                projected_usd,
                "Gmail mailbox placement repair",
            )?;
            let client = Client::builder().timeout(Duration::from_secs(20)).build()?;
            let mut report = EmailMailboxPlacementRepairReport {
                inspected: gaps.len(),
                eligible: gaps.len(),
                repaired: 0,
                skipped: 0,
                errors: Vec::new(),
                observations: Vec::new(),
                missing_credential: false,
                provider: "gmail".to_string(),
                source_health_key: source_health_key.to_string(),
            };
            for gap in gaps {
                if !matches!(
                    gap.verification_state.as_str(),
                    "mailbox_bad_placement_trash" | "mailbox_bad_placement_spam"
                ) {
                    report.skipped += 1;
                    report.errors.push(format!(
                        "{}: unsupported repair state {}",
                        gap.delivery_attempt_id, gap.verification_state
                    ));
                    continue;
                }
                let Some(latest_observation) =
                    self.latest_channel_delivery_observation(&gap.delivery_attempt_id)?
                else {
                    report.skipped += 1;
                    report.errors.push(format!(
                        "{}: latest mailbox observation missing",
                        gap.delivery_attempt_id
                    ));
                    continue;
                };
                let Some(mailbox_message_id) = latest_observation.mailbox_message_id.as_deref()
                else {
                    report.skipped += 1;
                    report.errors.push(format!(
                        "{}: latest mailbox observation has no Gmail message id",
                        gap.delivery_attempt_id
                    ));
                    continue;
                };
                gmail_modify_message_labels(
                    &client,
                    api_base_url.as_str(),
                    access_token,
                    mailbox_message_id,
                    &["INBOX"],
                    &["TRASH", "SPAM"],
                )?;
                let metadata = gmail_fetch_message_metadata(
                    &client,
                    api_base_url.as_str(),
                    access_token,
                    &[mailbox_message_id.to_string()],
                )?;
                let evidence = json!({
                    "matched_by": "gmail_api_mailbox_placement_repair",
                    "repair_action": "remove_bad_placement_labels_add_inbox",
                    "previous_verification_state": gap.verification_state,
                    "previous_observation_id": latest_observation.id,
                    "gmail_message_id": mailbox_message_id,
                    "gmail_message_metadata": metadata,
                    "api_base": api_base_url.as_str().trim_end_matches('/'),
                    "boundary": "Gmail API label repair removed bad placement labels and added INBOX for an already observed Arcwell delivery; records metadata only, not message body or secrets."
                });
                let observation = self.record_channel_delivery_observation(
                    &gap.delivery_attempt_id,
                    "gmail_api_repair",
                    "mailbox_observed",
                    Some(mailbox_message_id),
                    gap.outbound_message_id
                        .as_deref()
                        .or(gap.provider_message_id.as_deref()),
                    None,
                    &evidence,
                )?;
                if email_delivery_mailbox_verification_state(&observation)
                    == "mailbox_observed_inbox"
                {
                    report.repaired += 1;
                } else {
                    report.skipped += 1;
                    report.errors.push(format!(
                        "{}: Gmail metadata after repair did not show INBOX placement",
                        gap.delivery_attempt_id
                    ));
                }
                report.observations.push(observation);
            }
            self.record_source_success(SourceHealthUpdate {
                key: source_health_key,
                provider: "gmail",
                source_kind: "gmail_mailbox_repair",
                locator: "me",
                last_item_id: report
                    .observations
                    .last()
                    .map(|observation| observation.id.as_str()),
                last_item_date: report
                    .observations
                    .last()
                    .map(|observation| observation.observed_at.as_str()),
                cursor_key: None,
                cursor_value: None,
                next_run_at: None,
            })?;
            Ok(report)
        })();
        if let Err(error) = &result {
            let _ = self.record_source_failure(
                source_health_key,
                "gmail",
                "gmail_mailbox_repair",
                "me",
                &error.to_string(),
            );
        }
        result
    }

    fn verify_email_delivery_requests_with_gmail(
        &self,
        requests: Vec<EmailDeliveryVerificationRequest>,
        access_token: &str,
        api_base: Option<&str>,
    ) -> Result<EmailMailboxVerificationReport> {
        validate_notes(access_token)?;
        let api_base = api_base.unwrap_or("https://gmail.googleapis.com");
        let api_base_url = validate_public_http_url(api_base)?;
        let ready = requests.iter().filter(|request| request.ready).count();
        let source_health_key = gmail_mailbox_verifier_source_health_key();
        let projected_usd = estimated_network_fetch_cost(ready.max(1));
        let result = (|| -> Result<EmailMailboxVerificationReport> {
            self.policy_guard(PolicyRequest {
                action: "provider.network".to_string(),
                package: Some("arcwell-email".to_string()),
                provider: Some("gmail".to_string()),
                source: Some("email_delivery_mailbox_verify".to_string()),
                channel: Some("email".to_string()),
                subject: None,
                target: Some(api_base_url.as_str().trim_end_matches('/').to_string()),
                projected_usd: Some(projected_usd),
                metadata: json!({
                    "request_count": requests.len(),
                    "ready_count": ready,
                    "source_health_key": source_health_key
                }),
                untrusted_excerpt: None,
            })?;
            self.require_cost_budget(
                "arcwell-email",
                "email_delivery_mailbox_verify",
                "gmail",
                "mailbox_verify",
                Some("email_delivery_mailbox_verify"),
                projected_usd,
                "Gmail mailbox verification",
            )?;
            let client = Client::builder().timeout(Duration::from_secs(20)).build()?;
            let mut report = EmailMailboxVerificationReport {
                inspected: requests.len(),
                ready,
                observed: 0,
                not_found: 0,
                skipped: 0,
                errors: Vec::new(),
                observations: Vec::new(),
                missing_credential: false,
                provider: "gmail".to_string(),
                source_health_key: source_health_key.to_string(),
            };
            for request in requests {
                if !request.ready {
                    report.skipped += 1;
                    continue;
                }
                let search_query = request.search_query.as_deref().unwrap_or_default();
                let search = gmail_search_message_ids(
                    &client,
                    api_base_url.as_str(),
                    access_token,
                    search_query,
                )?;
                let metadata = gmail_fetch_message_metadata(
                    &client,
                    api_base_url.as_str(),
                    access_token,
                    &search.message_ids,
                )?;
                let (observation_status, mailbox_message_id) =
                    if let Some(first) = search.message_ids.first() {
                        report.observed += 1;
                        ("mailbox_observed", Some(first.as_str()))
                    } else {
                        report.not_found += 1;
                        ("mailbox_not_found", None)
                    };
                let evidence = json!({
                    "matched_by": "gmail_api_message_id_search",
                    "query": search_query,
                    "result_count": search.result_count,
                    "gmail_message_ids": search.message_ids,
                    "gmail_thread_ids": search.thread_ids,
                    "gmail_message_metadata": metadata,
                    "api_base": api_base_url.as_str().trim_end_matches('/'),
                    "boundary": "Gmail API search and metadata result; records Gmail placement labels such as INBOX, SPAM, or TRASH, but does not expose Gmail message body or secrets."
                });
                let observation = self.record_channel_delivery_observation(
                    &request.delivery_attempt_id,
                    "gmail_api",
                    observation_status,
                    mailbox_message_id,
                    request
                        .outbound_message_id
                        .as_deref()
                        .or(request.provider_message_id.as_deref()),
                    None,
                    &evidence,
                )?;
                report.observations.push(observation);
            }
            self.record_source_success(SourceHealthUpdate {
                key: source_health_key,
                provider: "gmail",
                source_kind: "gmail_mailbox_verifier",
                locator: "me",
                last_item_id: report
                    .observations
                    .last()
                    .map(|observation| observation.id.as_str()),
                last_item_date: report
                    .observations
                    .last()
                    .map(|observation| observation.observed_at.as_str()),
                cursor_key: None,
                cursor_value: None,
                next_run_at: None,
            })?;
            Ok(report)
        })();
        if let Err(error) = &result {
            let _ = self.record_source_failure(
                source_health_key,
                "gmail",
                "gmail_mailbox_verifier",
                "me",
                &error.to_string(),
            );
        }
        result
    }

    fn record_gmail_mailbox_verifier_missing_credential(&self) -> Result<()> {
        self.record_source_failure(
            gmail_mailbox_verifier_source_health_key(),
            "gmail",
            "gmail_mailbox_verifier",
            "me",
            "GMAIL_ACCESS_TOKEN is not configured; run `arcwell email oauth-reauthorize` or store a usable Gmail OAuth access token before daemon-owned mailbox verification can run.",
        )
    }

    pub fn drain_telegram_edge_events(&self, max_events: usize) -> Result<TelegramDrainReport> {
        let mut processed = 0;
        let mut acked = 0;
        let mut nacked = 0;
        let mut messages = Vec::new();
        let mut controller_routes = Vec::new();
        let mut controller_route_errors = Vec::new();
        for _ in 0..max_events.clamp(1, 100) {
            let Some(event) = self.lease_edge_event_for_source("telegram")? else {
                break;
            };
            processed += 1;
            match self.record_telegram_event(&event) {
                Ok(recorded) => {
                    match self.controller_route_text_with_origin(
                        "telegram",
                        None,
                        &recorded.conversation_id,
                        &recorded.controller_sender,
                        &recorded.message.body,
                        Some(&recorded.message.id),
                    ) {
                        Ok(report) => controller_routes.push(report),
                        Err(error) => controller_route_errors
                            .push(format!("{}: {}", recorded.message.id, error)),
                    }
                    self.ack_edge_event(&event.id)?;
                    acked += 1;
                    messages.push(recorded.message);
                }
                Err(error) => {
                    self.nack_edge_event(&event.id, &error.to_string())?;
                    nacked += 1;
                }
            }
        }
        Ok(TelegramDrainReport {
            processed,
            acked,
            nacked,
            messages,
            controller_routes,
            controller_route_errors,
        })
    }

    pub fn drain_email_edge_events(&self, max_events: usize) -> Result<EmailDrainReport> {
        let mut processed = 0;
        let mut acked = 0;
        let mut nacked = 0;
        let mut messages = Vec::new();
        let mut source_cards = Vec::new();
        for _ in 0..max_events.clamp(1, 100) {
            let Some(event) = self.lease_edge_event_for_source("email")? else {
                break;
            };
            processed += 1;
            match self.record_email_event(&event) {
                Ok((message, source_card)) => {
                    self.ack_edge_event(&event.id)?;
                    acked += 1;
                    messages.push(message);
                    source_cards.push(source_card);
                }
                Err(error) => {
                    self.nack_edge_event(&event.id, &error.to_string())?;
                    nacked += 1;
                }
            }
        }
        Ok(EmailDrainReport {
            processed,
            acked,
            nacked,
            messages,
            source_cards,
        })
    }

    pub(crate) fn record_email_event(
        &self,
        event: &EdgeEvent,
    ) -> Result<(ChannelMessage, SourceCard)> {
        let payload = &event.payload_json;
        let trusted_sender = payload
            .get("trustedSender")
            .or_else(|| payload.get("trusted_sender"))
            .and_then(Value::as_str)
            .and_then(normalize_email_address)
            .context("email event missing trusted sender")?;
        let recipient = payload
            .get("recipient")
            .and_then(Value::as_str)
            .and_then(normalize_email_address)
            .context("email event missing recipient")?;
        let subject = payload
            .get("subject")
            .and_then(Value::as_str)
            .unwrap_or("(no subject)");
        let text = payload
            .get("sanitizedText")
            .or_else(|| payload.get("sanitized_text"))
            .and_then(Value::as_str)
            .context("email event missing sanitized text")?;
        let message_id = payload
            .get("messageId")
            .or_else(|| payload.get("message_id"))
            .and_then(Value::as_str)
            .context("email event missing message id")?;
        let auth = payload.get("auth").cloned().unwrap_or_else(|| json!({}));
        let is_author = self.email_sender_is_configured_author(&trusted_sender)?;
        let trust_label = if is_author {
            "TRUSTED_AUTHOR_EMAIL_INSTRUCTIONS"
        } else {
            "UNTRUSTED_CHANNEL_EVIDENCE"
        };
        let body = format!(
            "{trust_label}\nFrom: {trusted_sender}\nTo: {recipient}\nSubject: {}\nMessage-ID: {}\n\n{}",
            excerpt(subject, 240),
            excerpt(message_id, 240),
            text
        );
        let project_id = payload.get("projectId").and_then(Value::as_str);
        let message = self.record_channel_message(
            "email",
            "incoming",
            &format!("email:{trusted_sender}"),
            &body,
            project_id,
            Some(&event.id),
        )?;
        let card = self.add_source_card(SourceCardInput {
            title: format!("Email: {}", excerpt(subject, 160)),
            url: email_source_card_url(message_id),
            source_type: "email".to_string(),
            provider: "cloudflare_email_routing".to_string(),
            summary: body.clone(),
            claims: vec![],
            retrieved_at: payload
                .get("receivedAt")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
            metadata: json!({
                "trust": if is_author { "trusted_author_instruction" } else { "untrusted_email_evidence" },
                "body_instruction_policy": if is_author {
                    "configured_author_email_is_allowed_to_instruct"
                } else {
                    "email_body_is_evidence_never_instructions"
                },
                "trusted_sender": trusted_sender,
                "recipient": recipient,
                "message_id_hash": sha256(message_id.as_bytes()),
                "source_event_id": event.id,
                "route_id": payload.get("routeId").cloned().unwrap_or(Value::Null),
                "warnings": payload.get("warnings").cloned().unwrap_or_else(|| json!([])),
                "auth": auth,
            }),
        })?;
        Ok((message, card))
    }

    pub(crate) fn record_telegram_event(&self, event: &EdgeEvent) -> Result<RecordedTelegramEvent> {
        let payload = &event.payload_json;
        let text = payload
            .get("text")
            .and_then(Value::as_str)
            .context("telegram event missing text")?;
        let chat_id = payload
            .get("chatId")
            .or_else(|| payload.get("chat_id"))
            .and_then(value_as_string)
            .context("telegram event missing chat id")?;
        let username = payload
            .get("username")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .map(|username| username.trim().to_string());
        let sender_id = payload
            .get("senderId")
            .or_else(|| payload.get("sender_id"))
            .and_then(value_as_string);
        let mut authorization_subjects = vec![format!("telegram:chat:{chat_id}")];
        let mut controller_sender_candidates = vec![format!("chat:{chat_id}")];
        if let Some(username) = &username {
            authorization_subjects.push(format!("telegram:@{username}"));
            controller_sender_candidates.push(format!("@{username}"));
        }
        if let Some(sender_id) = &sender_id {
            authorization_subjects.push(format!("telegram:user:{sender_id}"));
            controller_sender_candidates.push(format!("user:{sender_id}"));
        }
        let sender = username
            .as_ref()
            .map(|username| format!("telegram:@{username}"))
            .or_else(|| {
                sender_id
                    .as_ref()
                    .map(|sender_id| format!("telegram:user:{sender_id}"))
            })
            .unwrap_or_else(|| format!("telegram:chat:{chat_id}"));
        let explicit_project_id = payload.get("projectId").and_then(Value::as_str);
        let can_write_projects =
            self.channel_subject_can_write_projects("telegram", &authorization_subjects)?;
        if explicit_project_id.is_some() && !can_write_projects {
            bail!(
                "telegram subject is not authorized to bind project state: {}",
                authorization_subjects.join(", ")
            );
        }
        let resolved_project_id = if explicit_project_id.is_none() && can_write_projects {
            self.resolve_project(text, None)
                .ok()
                .map(|resolution| resolution.project.id)
        } else {
            None
        };
        let project_id = explicit_project_id.or(resolved_project_id.as_deref());
        let message = self.record_channel_message(
            "telegram",
            "incoming",
            &sender,
            text,
            project_id,
            Some(&event.id),
        )?;
        let controller_sender =
            self.select_telegram_controller_sender(&controller_sender_candidates)?;
        Ok(RecordedTelegramEvent {
            message,
            conversation_id: format!("chat:{chat_id}"),
            controller_sender,
        })
    }

    pub(crate) fn select_telegram_controller_sender(
        &self,
        candidates: &[String],
    ) -> Result<String> {
        let first = candidates
            .first()
            .context("telegram controller sender candidates cannot be empty")?;
        for candidate in candidates {
            validate_controller_ref(candidate, "telegram controller sender")?;
            let subject = format!("telegram:{candidate}");
            let can_read = self.channel_subject_can_read_projects("telegram", &subject)?;
            let can_write = self.channel_subject_can_write_projects("telegram", &[subject])?;
            if can_read || can_write {
                return Ok(candidate.clone());
            }
        }
        Ok(first.clone())
    }

    pub fn send_telegram_message(
        &self,
        bot_token: &str,
        chat_id: &str,
        text: &str,
        api_base: Option<&str>,
    ) -> Result<TelegramSendReport> {
        validate_notes(bot_token)?;
        validate_key(chat_id)?;
        validate_notes(text)?;
        let subject = format!("telegram:chat:{chat_id}");
        if !self.channel_subject_can_send("telegram", &subject)? {
            bail!("telegram subject is not authorized to send: {subject}");
        }
        self.policy_guard(PolicyRequest {
            action: "channel.send".to_string(),
            package: None,
            provider: Some("telegram".to_string()),
            source: Some("telegram_send".to_string()),
            channel: Some("telegram".to_string()),
            subject: Some(subject.clone()),
            target: Some(chat_id.to_string()),
            projected_usd: None,
            metadata: json!({ "parse_mode": "MarkdownV2" }),
            untrusted_excerpt: Some(text.to_string()),
        })?;
        self.require_cost_budget(
            "arcwell-telegram",
            "telegram_send",
            "telegram",
            "send_message",
            Some("telegram_send"),
            estimated_channel_send_cost(),
            "Telegram send",
        )?;
        self.send_telegram_message_preflighted(bot_token, chat_id, text, api_base)
    }

    pub(crate) fn send_telegram_message_preflighted(
        &self,
        bot_token: &str,
        chat_id: &str,
        text: &str,
        api_base: Option<&str>,
    ) -> Result<TelegramSendReport> {
        validate_notes(bot_token)?;
        validate_key(chat_id)?;
        validate_notes(text)?;
        let message = self.record_channel_message_with_status(
            "telegram",
            "outgoing",
            &format!("telegram:chat:{chat_id}"),
            text,
            "pending",
            None,
            None,
        )?;
        self.send_existing_telegram_message_preflighted(
            &message.id,
            bot_token,
            chat_id,
            text,
            api_base,
        )
    }

    pub(crate) fn send_existing_telegram_message_preflighted(
        &self,
        message_id: &str,
        bot_token: &str,
        chat_id: &str,
        text: &str,
        api_base: Option<&str>,
    ) -> Result<TelegramSendReport> {
        validate_id(message_id)?;
        validate_notes(bot_token)?;
        validate_key(chat_id)?;
        validate_notes(text)?;
        let subject = format!("telegram:chat:{chat_id}");
        let base = api_base.unwrap_or("https://api.telegram.org");
        let url = format!(
            "{}/bot{}/sendMessage",
            base.trim_end_matches('/'),
            bot_token
        );
        let client = Client::builder().timeout(Duration::from_secs(20)).build()?;
        let response = client
            .post(&url)
            .json(&json!({
                "chat_id": chat_id,
                "text": escape_telegram_markdown_v2(text),
                "parse_mode": "MarkdownV2"
            }))
            .send();
        let (status, response_json, error, retry_at) = match response {
            Ok(response) => {
                let status = response.status().as_u16();
                let retry_at = telegram_retry_at(status, response.headers());
                let response_json = response.json::<Value>().unwrap_or_else(|_| json!({}));
                (status, response_json, None, retry_at)
            }
            Err(error) => (
                0,
                json!({ "ok": false, "error": "request_failed" }),
                Some(telegram_request_error_summary(&error)),
                Some((Utc::now() + chrono::Duration::seconds(60)).to_rfc3339()),
            ),
        };
        let ok = (200..300).contains(&status)
            && response_json
                .get("ok")
                .and_then(Value::as_bool)
                .unwrap_or(false);
        let delivery = self.record_channel_delivery_attempt(
            message_id,
            "telegram",
            &subject,
            ok,
            i64::from(status),
            &response_json,
            error.as_deref(),
            retry_at.as_deref(),
        )?;
        let message =
            self.update_channel_message_status(message_id, if ok { "sent" } else { "failed" })?;
        Ok(TelegramSendReport {
            ok,
            status,
            response: response_json,
            message,
            delivery,
        })
    }

    // allow: refactoring this N-arg signature is out of scope for the lint-cleanup pass.
    #[allow(clippy::too_many_arguments)]
    pub fn send_cloudflare_email(
        &self,
        account_id: &str,
        api_token: &str,
        from: &str,
        to: &str,
        subject: &str,
        text: &str,
        html: Option<&str>,
        reply_to_message_id: Option<&str>,
        api_base: Option<&str>,
    ) -> Result<EmailSendReport> {
        validate_key(account_id)?;
        validate_notes(api_token)?;
        let from = normalize_email_address(from).context("invalid email from address")?;
        let to = normalize_email_address(to).context("invalid email to address")?;
        validate_notes(subject)?;
        validate_email_body_text(text)?;
        if let Some(html) = html {
            validate_email_html(html)?;
        }
        if let Some(message_id) = reply_to_message_id {
            validate_notes(message_id)?;
        }
        let subject_key = format!("email:{to}");
        if !self.channel_subject_can_send("email", &subject_key)? {
            bail!("email subject is not authorized to send: {subject_key}");
        }
        self.policy_guard(PolicyRequest {
            action: "channel.send".to_string(),
            package: Some("arcwell-email".to_string()),
            provider: Some("cloudflare_email".to_string()),
            source: Some("email_send".to_string()),
            channel: Some("email".to_string()),
            subject: Some(subject_key.clone()),
            target: Some(to.clone()),
            projected_usd: None,
            metadata: json!({
                "from": from,
                "reply_to_message_id": reply_to_message_id,
                "rich_html": true,
                "html_source": if html.is_some() { "caller" } else { "markdown_auto_render" },
            }),
            untrusted_excerpt: Some(excerpt(&format!("{subject}\n\n{text}"), 4_000)),
        })?;
        self.require_cost_budget(
            "arcwell-email",
            "email_send",
            "cloudflare_email",
            "send",
            Some("email_send"),
            estimated_channel_send_cost(),
            "Cloudflare Email send",
        )?;
        self.send_cloudflare_email_preflighted(
            account_id,
            api_token,
            &from,
            &to,
            subject,
            text,
            html,
            reply_to_message_id,
            api_base,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn send_cloudflare_email_preflighted(
        &self,
        account_id: &str,
        api_token: &str,
        from: &str,
        to: &str,
        subject: &str,
        text: &str,
        html: Option<&str>,
        reply_to_message_id: Option<&str>,
        api_base: Option<&str>,
    ) -> Result<EmailSendReport> {
        validate_key(account_id)?;
        validate_notes(api_token)?;
        let from = normalize_email_address(from).context("invalid email from address")?;
        let to = normalize_email_address(to).context("invalid email to address")?;
        validate_notes(subject)?;
        validate_email_body_text(text)?;
        let rendered_html = match html {
            Some(html) => {
                validate_email_html(html)?;
                Some(html.to_string())
            }
            None => Some(render_email_html_from_markdown(subject, text)?),
        };
        if let Some(message_id) = reply_to_message_id {
            validate_notes(message_id)?;
        }
        let subject_key = format!("email:{to}");
        let message = self.record_channel_message_with_status(
            "email",
            "outgoing",
            &format!("email:{to}"),
            text,
            "pending",
            None,
            None,
        )?;
        let outbound_message_id = arcwell_outbound_email_message_id(&message.id, &from)?;
        let endpoint = format!(
            "{}/accounts/{}/email/sending/send",
            api_base
                .unwrap_or("https://api.cloudflare.com/client/v4")
                .trim_end_matches('/'),
            account_id
        );
        let mut headers = Map::new();
        headers.insert("X-Arcwell-Message-Id".to_string(), json!(message.id));
        if let Some(message_id) = reply_to_message_id {
            headers.insert("In-Reply-To".to_string(), json!(message_id));
            headers.insert("References".to_string(), json!(message_id));
        }
        let mut body = json!({
            "from": from,
            "to": to,
            "subject": subject,
            "text": text,
        });
        if let Some(html) = rendered_html.as_deref() {
            body["html"] = json!(html);
        }
        if !headers.is_empty() {
            body["headers"] = Value::Object(headers);
        }
        let client = Client::builder().timeout(Duration::from_secs(20)).build()?;
        let response = client
            .post(endpoint)
            .header(AUTHORIZATION, format!("Bearer {api_token}"))
            .json(&body)
            .send();
        let (status, response_json, error, retry_at) = match response {
            Ok(response) => {
                let status = response.status().as_u16();
                let retry_at = if (200..300).contains(&status) {
                    None
                } else {
                    Some((Utc::now() + chrono::Duration::seconds(60)).to_rfc3339())
                };
                let response_json = response.json::<Value>().unwrap_or_else(|_| json!({}));
                (status, response_json, None, retry_at)
            }
            Err(error) => (
                0,
                json!({ "success": false, "error": "request_failed" }),
                Some(email_request_error_summary(&error)),
                Some((Utc::now() + chrono::Duration::seconds(60)).to_rfc3339()),
            ),
        };
        let ok = (200..300).contains(&status)
            && response_json
                .get("success")
                .and_then(Value::as_bool)
                .unwrap_or(true);
        let response_json = attach_arcwell_email_delivery_metadata(
            response_json,
            &message.id,
            &outbound_message_id,
        );
        let delivery = self.record_channel_delivery_attempt(
            &message.id,
            "email",
            &subject_key,
            ok,
            i64::from(status),
            &redact_email_send_response(response_json),
            error.as_deref(),
            retry_at.as_deref(),
        )?;
        let message =
            self.update_channel_message_status(&message.id, if ok { "sent" } else { "failed" })?;
        Ok(EmailSendReport {
            ok,
            status,
            response: delivery.response.clone(),
            message,
            delivery,
        })
    }

    pub fn retry_due_email_deliveries(
        &self,
        account_id: &str,
        api_token: &str,
        from: &str,
        api_base: Option<&str>,
        max_attempts: usize,
    ) -> Result<EmailRetryReport> {
        validate_key(account_id)?;
        validate_notes(api_token)?;
        let from = normalize_email_address(from).context("invalid email from address")?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT m.id, m.sender, m.body
            FROM channel_messages m
            JOIN channel_delivery_attempts d ON d.message_id = m.id
            WHERE m.channel = 'email'
              AND m.direction = 'outgoing'
              AND m.status = 'failed'
              AND d.ok = 0
              AND d.retry_at IS NOT NULL
              AND d.retry_at <= ?1
              AND d.attempt = (
                SELECT max(d2.attempt)
                FROM channel_delivery_attempts d2
                WHERE d2.message_id = m.id
              )
              AND NOT EXISTS (
                SELECT 1
                FROM digest_deliveries dd
                JOIN digest_candidates dc ON dc.id = dd.candidate_id
                WHERE dd.channel_message_id = m.id
                  AND (dc.status != 'approved' OR dc.review_status != 'approved')
              )
            ORDER BY d.retry_at ASC, d.created_at ASC
            LIMIT ?2
            "#,
        )?;
        let due = rows(
            stmt.query_map(params![now(), max_attempts.clamp(1, 100)], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?,
        )?;
        let mut reports = Vec::new();
        for (message_id, sender, body) in due {
            let to = sender.strip_prefix("email:").with_context(|| {
                format!("email message {message_id} has unsupported destination {sender}")
            })?;
            reports.push(self.send_existing_cloudflare_email_message(
                account_id,
                api_token,
                &from,
                &message_id,
                to,
                "Arcwell email retry",
                &body,
                api_base,
            )?);
        }
        let sent = reports.iter().filter(|report| report.ok).count();
        let failed = reports.len().saturating_sub(sent);
        Ok(EmailRetryReport {
            attempted: reports.len(),
            sent,
            failed,
            reports,
        })
    }

    pub(crate) fn retry_due_email_deliveries_for_worker(
        &self,
        max_attempts: usize,
    ) -> Result<(Option<EmailRetryReport>, Vec<String>)> {
        let due_count = self.due_email_delivery_count()?;
        if due_count == 0 {
            return Ok((None, Vec::new()));
        }
        let Some(account_id) = self.configured_cloudflare_account_id()? else {
            return Ok((
                None,
                vec![format!(
                    "{due_count} email delivery retry item(s) are due, but CLOUDFLARE_ACCOUNT_ID is not configured"
                )],
            ));
        };
        let Some(api_token) = self.configured_cloudflare_email_api_token()? else {
            return Ok((
                None,
                vec![format!(
                    "{due_count} email delivery retry item(s) are due, but CLOUDFLARE_EMAIL_API_TOKEN or CLOUDFLARE_API_TOKEN is not configured"
                )],
            ));
        };
        let Some(from) = self.configured_agent_email_from()? else {
            return Ok((
                None,
                vec![format!(
                    "{due_count} email delivery retry item(s) are due, but ARCWELL_AGENT_EMAIL_FROM or ARCWELL_AGENT_EMAIL is not configured"
                )],
            ));
        };
        let api_base = self.configured_cloudflare_email_api_base()?;
        let report = self.retry_due_email_deliveries(
            &account_id,
            &api_token,
            &from,
            api_base.as_deref(),
            max_attempts.min(due_count as usize),
        )?;
        Ok((Some(report), Vec::new()))
    }

    pub(crate) fn due_email_delivery_count(&self) -> Result<i64> {
        self.conn
            .query_row(
                r#"
                SELECT COUNT(*)
                FROM channel_messages m
                JOIN channel_delivery_attempts d ON d.message_id = m.id
                WHERE m.channel = 'email'
                  AND m.direction = 'outgoing'
                  AND m.status = 'failed'
                  AND d.ok = 0
                  AND d.retry_at IS NOT NULL
                  AND d.retry_at <= ?1
                  AND d.attempt = (
                    SELECT max(d2.attempt)
                    FROM channel_delivery_attempts d2
                    WHERE d2.message_id = m.id
                  )
                  AND NOT EXISTS (
                    SELECT 1
                    FROM digest_deliveries dd
                    JOIN digest_candidates dc ON dc.id = dd.candidate_id
                    WHERE dd.channel_message_id = m.id
                      AND (dc.status != 'approved' OR dc.review_status != 'approved')
                  )
                "#,
                params![now()],
                |row| row.get(0),
            )
            .map_err(Into::into)
    }

    // allow: refactoring this N-arg signature is out of scope for the lint-cleanup pass.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn send_existing_cloudflare_email_message(
        &self,
        account_id: &str,
        api_token: &str,
        from: &str,
        message_id: &str,
        to: &str,
        subject: &str,
        text: &str,
        api_base: Option<&str>,
    ) -> Result<EmailSendReport> {
        self.send_existing_cloudflare_email_message_with_context(
            account_id,
            api_token,
            from,
            message_id,
            to,
            subject,
            text,
            api_base,
            "email_retry",
            "Cloudflare Email retry",
            json!({ "retry": true }),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn send_existing_cloudflare_email_message_with_context(
        &self,
        account_id: &str,
        api_token: &str,
        from: &str,
        message_id: &str,
        to: &str,
        subject: &str,
        text: &str,
        api_base: Option<&str>,
        source: &str,
        cost_label: &str,
        metadata: Value,
    ) -> Result<EmailSendReport> {
        validate_key(account_id)?;
        validate_notes(api_token)?;
        let from = normalize_email_address(from).context("invalid email from address")?;
        validate_id(message_id)?;
        validate_key(source)?;
        validate_notes(cost_label)?;
        let mut message = self
            .get_channel_message(message_id)?
            .with_context(|| format!("channel message not found: {message_id}"))?;
        if message.channel != "email" || message.direction != "outgoing" {
            bail!("message {message_id} is not an outgoing email message");
        }
        let to = normalize_email_address(to).context("invalid email to address")?;
        validate_notes(subject)?;
        validate_email_body_text(text)?;
        let subject_key = format!("email:{to}");
        if message.sender != subject_key {
            bail!(
                "email message {message_id} destination mismatch: expected {}, found {}",
                message.sender,
                subject_key
            );
        }
        if !self.channel_subject_can_send("email", &subject_key)? {
            bail!("email subject is not authorized to send: {subject_key}");
        }
        self.policy_guard(PolicyRequest {
            action: "channel.send".to_string(),
            package: Some("arcwell-email".to_string()),
            provider: Some("cloudflare_email".to_string()),
            source: Some(source.to_string()),
            channel: Some("email".to_string()),
            subject: Some(subject_key.clone()),
            target: Some(to.clone()),
            projected_usd: None,
            metadata: json!({
                "message_id": message_id,
                "from": from.clone(),
                "rich_html": true,
                "html_source": "markdown_auto_render",
                "context": metadata,
            }),
            untrusted_excerpt: Some(excerpt(&format!("{subject}\n\n{text}"), 4_000)),
        })?;
        self.require_cost_budget(
            "arcwell-email",
            message_id,
            "cloudflare_email",
            "send",
            Some(source),
            estimated_channel_send_cost(),
            cost_label,
        )?;
        let endpoint = format!(
            "{}/accounts/{}/email/sending/send",
            api_base
                .unwrap_or("https://api.cloudflare.com/client/v4")
                .trim_end_matches('/'),
            account_id
        );
        let html = render_email_html_from_markdown(subject, text)?;
        let outbound_message_id = arcwell_outbound_email_message_id(message_id, &from)?;
        let body = json!({
            "from": from,
            "to": to,
            "subject": subject,
            "text": text,
            "html": html,
            "headers": {
                "X-Arcwell-Message-Id": message_id
            },
        });
        let client = Client::builder().timeout(Duration::from_secs(20)).build()?;
        let response = client
            .post(endpoint)
            .header(AUTHORIZATION, format!("Bearer {api_token}"))
            .json(&body)
            .send();
        let (status, response_json, error, retry_at) = match response {
            Ok(response) => {
                let status = response.status().as_u16();
                let retry_at = if (200..300).contains(&status) {
                    None
                } else {
                    Some((Utc::now() + chrono::Duration::seconds(60)).to_rfc3339())
                };
                let response_json = response.json::<Value>().unwrap_or_else(|_| json!({}));
                (status, response_json, None, retry_at)
            }
            Err(error) => (
                0,
                json!({ "success": false, "error": "request_failed" }),
                Some(email_request_error_summary(&error)),
                Some((Utc::now() + chrono::Duration::seconds(60)).to_rfc3339()),
            ),
        };
        let ok = (200..300).contains(&status)
            && response_json
                .get("success")
                .and_then(Value::as_bool)
                .unwrap_or(true);
        let response_json =
            attach_arcwell_email_delivery_metadata(response_json, message_id, &outbound_message_id);
        let delivery = self.record_channel_delivery_attempt(
            message_id,
            "email",
            &subject_key,
            ok,
            i64::from(status),
            &redact_email_send_response(response_json),
            error.as_deref(),
            retry_at.as_deref(),
        )?;
        message =
            self.update_channel_message_status(message_id, if ok { "sent" } else { "failed" })?;
        Ok(EmailSendReport {
            ok,
            status,
            response: delivery.response.clone(),
            message,
            delivery,
        })
    }

    pub fn retry_due_telegram_deliveries(
        &self,
        bot_token: &str,
        api_base: Option<&str>,
        max_attempts: usize,
    ) -> Result<TelegramRetryReport> {
        validate_notes(bot_token)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT m.id, m.sender, m.body
            FROM channel_messages m
            JOIN channel_delivery_attempts d ON d.message_id = m.id
            WHERE m.channel = 'telegram'
              AND m.direction = 'outgoing'
              AND m.status = 'failed'
              AND d.ok = 0
              AND d.retry_at IS NOT NULL
              AND d.retry_at <= ?1
              AND d.attempt = (
                SELECT max(d2.attempt)
                FROM channel_delivery_attempts d2
                WHERE d2.message_id = m.id
              )
              AND NOT EXISTS (
                SELECT 1
                FROM digest_deliveries dd
                JOIN digest_candidates dc ON dc.id = dd.candidate_id
                WHERE dd.channel_message_id = m.id
                  AND (dc.status != 'approved' OR dc.review_status != 'approved')
              )
            ORDER BY d.retry_at ASC, d.created_at ASC
            LIMIT ?2
            "#,
        )?;
        let due = rows(
            stmt.query_map(params![now(), max_attempts.clamp(1, 100)], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?,
        )?;
        let mut reports = Vec::new();
        for (message_id, sender, body) in due {
            let chat_id = sender.strip_prefix("telegram:chat:").with_context(|| {
                format!("telegram message {message_id} has unsupported destination {sender}")
            })?;
            reports.push(self.send_existing_telegram_message(
                &message_id,
                bot_token,
                chat_id,
                &body,
                api_base,
            )?);
        }
        let sent = reports.iter().filter(|report| report.ok).count();
        let failed = reports.len().saturating_sub(sent);
        Ok(TelegramRetryReport {
            attempted: reports.len(),
            sent,
            failed,
            reports,
        })
    }

    pub(crate) fn retry_due_telegram_deliveries_for_worker(
        &self,
        max_attempts: usize,
    ) -> Result<(Option<TelegramRetryReport>, Vec<String>)> {
        let due_count = self.due_telegram_delivery_count()?;
        if due_count == 0 {
            return Ok((None, Vec::new()));
        }
        let Some(bot_token) = self.configured_telegram_bot_token()? else {
            return Ok((
                None,
                vec![format!(
                    "{due_count} Telegram delivery retry item(s) are due, but TELEGRAM_BOT_TOKEN is not configured"
                )],
            ));
        };
        let api_base = self.configured_telegram_api_base()?;
        let report = self.retry_due_telegram_deliveries(
            &bot_token,
            api_base.as_deref(),
            max_attempts.min(due_count as usize),
        )?;
        Ok((Some(report), Vec::new()))
    }

    pub(crate) fn due_telegram_delivery_count(&self) -> Result<i64> {
        self.conn
            .query_row(
                r#"
                SELECT COUNT(*)
                FROM channel_messages m
                JOIN channel_delivery_attempts d ON d.message_id = m.id
                WHERE m.channel = 'telegram'
                  AND m.direction = 'outgoing'
                  AND m.status = 'failed'
                  AND d.ok = 0
                  AND d.retry_at IS NOT NULL
                  AND d.retry_at <= ?1
                  AND d.attempt = (
                    SELECT max(d2.attempt)
                    FROM channel_delivery_attempts d2
                    WHERE d2.message_id = m.id
                  )
                  AND NOT EXISTS (
                    SELECT 1
                    FROM digest_deliveries dd
                    JOIN digest_candidates dc ON dc.id = dd.candidate_id
                    WHERE dd.channel_message_id = m.id
                      AND (dc.status != 'approved' OR dc.review_status != 'approved')
                  )
                "#,
                params![now()],
                |row| row.get(0),
            )
            .map_err(Into::into)
    }

    pub(crate) fn configured_telegram_bot_token(&self) -> Result<Option<String>> {
        self.get_usable_secret_value("TELEGRAM_BOT_TOKEN")
            .map(|secret| secret.or_else(|| std::env::var("TELEGRAM_BOT_TOKEN").ok()))
    }

    pub(crate) fn configured_telegram_api_base(&self) -> Result<Option<String>> {
        let value = self
            .get_usable_secret_value("TELEGRAM_API_BASE")?
            .or_else(|| std::env::var("ARCWELL_TELEGRAM_API_BASE").ok());
        if let Some(value) = &value {
            validate_public_http_url(value)?;
        }
        Ok(value)
    }

    pub(crate) fn configured_cloudflare_account_id(&self) -> Result<Option<String>> {
        self.get_usable_secret_value("CLOUDFLARE_ACCOUNT_ID")
            .map(|secret| secret.or_else(|| std::env::var("CLOUDFLARE_ACCOUNT_ID").ok()))
    }

    pub(crate) fn configured_cloudflare_email_api_token(&self) -> Result<Option<String>> {
        self.get_usable_secret_value("CLOUDFLARE_EMAIL_API_TOKEN")
            .and_then(|secret| {
                if secret.is_some() {
                    Ok(secret)
                } else {
                    self.get_usable_secret_value("CLOUDFLARE_API_TOKEN")
                }
            })
            .map(|secret| {
                secret
                    .or_else(|| std::env::var("CLOUDFLARE_EMAIL_API_TOKEN").ok())
                    .or_else(|| std::env::var("CLOUDFLARE_API_TOKEN").ok())
            })
    }

    pub(crate) fn configured_cloudflare_email_api_base(&self) -> Result<Option<String>> {
        let value = self
            .get_usable_secret_value("CLOUDFLARE_EMAIL_API_BASE")?
            .or_else(|| std::env::var("CLOUDFLARE_EMAIL_API_BASE").ok())
            .or_else(|| std::env::var("ARCWELL_CLOUDFLARE_EMAIL_API_BASE").ok());
        if let Some(value) = &value {
            validate_public_http_url(value)?;
        }
        Ok(value)
    }

    pub(crate) fn configured_gmail_access_token(&self) -> Result<Option<String>> {
        self.configured_gmail_access_token_with_oauth_base(None)
    }

    pub(crate) fn configured_gmail_access_token_with_oauth_base(
        &self,
        oauth_base: Option<&str>,
    ) -> Result<Option<String>> {
        if let Some(token) = std::env::var("GMAIL_ACCESS_TOKEN")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
        {
            validate_notes(&token)?;
            return Ok(Some(token));
        }

        let local_access = self.get_secret_value("GMAIL_ACCESS_TOKEN")?;
        if let Some(access_token) = local_access.as_deref()
            && !self.local_secret_value_is_expired("GMAIL_ACCESS_TOKEN")?
        {
            validate_notes(access_token)?;
            return Ok(Some(access_token.to_string()));
        }

        let refresh_ready = self.get_secret_value("GMAIL_REFRESH_TOKEN")?.is_some();
        if refresh_ready && let Ok(client_id) = self.resolve_gmail_oauth_client_id(None) {
            let refresh = match oauth_base {
                Some(endpoint) => self.gmail_oauth_refresh_with_base(&client_id, None, endpoint),
                None => self.gmail_oauth_refresh(&client_id, None),
            };
            match refresh {
                Ok(_) => return self.get_usable_secret_value("GMAIL_ACCESS_TOKEN"),
                Err(error) => {
                    let redacted = redact_secret_like_text(&error.to_string());
                    if local_access.is_some() {
                        bail!(
                            "GMAIL_ACCESS_TOKEN is expired and Gmail OAuth refresh failed: {redacted}"
                        );
                    }
                    bail!(
                        "GMAIL_ACCESS_TOKEN is not configured and Gmail OAuth refresh failed: {redacted}"
                    );
                }
            }
        }

        if local_access.is_some() {
            return self.get_usable_secret_value("GMAIL_ACCESS_TOKEN");
        }
        Ok(None)
    }

    fn local_secret_value_is_expired(&self, name: &str) -> Result<bool> {
        validate_key(name)?;
        let metadata = self
            .conn
            .query_row(
                "SELECT name, scope, provider, expires_at, updated_at FROM secret_values WHERE name = ?1",
                params![name],
                secret_value_from_row,
            )
            .optional()?;
        Ok(metadata
            .map(secret_value_health)
            .transpose()?
            .is_some_and(|health| health.status == "expired"))
    }

    pub fn gmail_oauth_authorize_url(
        &self,
        client_id: &str,
        redirect_uri: &str,
        scopes: &[String],
    ) -> Result<GmailOAuthStart> {
        validate_query(client_id)?;
        validate_public_http_url(redirect_uri)?;
        let scopes = if scopes.is_empty() {
            default_gmail_oauth_scopes()
        } else {
            scopes.to_vec()
        };
        for scope in &scopes {
            validate_query(scope)?;
        }
        let state = Uuid::new_v4().to_string();
        let code_verifier = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        let code_challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()));
        let mut url = Url::parse("https://accounts.google.com/o/oauth2/v2/auth")?;
        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", client_id)
            .append_pair("redirect_uri", redirect_uri)
            .append_pair("scope", &scopes.join(" "))
            .append_pair("state", &state)
            .append_pair("code_challenge", &code_challenge)
            .append_pair("code_challenge_method", "S256")
            .append_pair("access_type", "offline")
            .append_pair("prompt", "consent");
        Ok(GmailOAuthStart {
            authorization_url: url.to_string(),
            state,
            code_verifier,
            code_challenge,
            scopes,
        })
    }

    pub fn resolve_gmail_oauth_client_id(&self, explicit: Option<&str>) -> Result<String> {
        if let Some(client_id) = explicit.map(str::trim).filter(|value| !value.is_empty()) {
            validate_query(client_id)?;
            return Ok(client_id.to_string());
        }
        self.get_usable_secret_value("GMAIL_CLIENT_ID")?
            .or_else(|| self.get_usable_secret_value("GOOGLE_CLIENT_ID").ok().flatten())
            .or_else(|| std::env::var("GMAIL_CLIENT_ID").ok())
            .or_else(|| std::env::var("GOOGLE_CLIENT_ID").ok())
            .context("GMAIL_CLIENT_ID or GOOGLE_CLIENT_ID is required; store it with `arcwell secrets set-value GMAIL_CLIENT_ID ... --scope gmail` or pass --client-id")
    }

    pub fn resolve_gmail_oauth_redirect_uri(&self, explicit: Option<&str>) -> Result<String> {
        let redirect_uri = explicit
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .or_else(|| std::env::var("GMAIL_REDIRECT_URI").ok())
            .or_else(|| std::env::var("GOOGLE_REDIRECT_URI").ok())
            .or_else(|| {
                self.get_usable_secret_value("GMAIL_REDIRECT_URI")
                    .ok()
                    .flatten()
            })
            .or_else(|| {
                self.get_usable_secret_value("GOOGLE_REDIRECT_URI")
                    .ok()
                    .flatten()
            })
            .unwrap_or_else(|| "http://127.0.0.1:8766/callback".to_string());
        validate_public_http_url(&redirect_uri)?;
        Ok(redirect_uri)
    }

    pub fn gmail_oauth_reauthorize_preflight(
        &self,
        redirect_uri: &str,
        scopes: &[String],
    ) -> Result<GmailOAuthReauthorizePreflightReport> {
        validate_public_http_url(redirect_uri)?;
        let scopes = if scopes.is_empty() {
            default_gmail_oauth_scopes()
        } else {
            scopes.to_vec()
        };
        for scope in &scopes {
            validate_query(scope)?;
        }
        self.policy_guard(PolicyRequest {
            action: "provider.oauth".to_string(),
            package: Some("arcwell-email".to_string()),
            provider: Some("gmail".to_string()),
            source: Some("gmail_oauth".to_string()),
            channel: Some("email".to_string()),
            subject: None,
            target: Some("https://accounts.google.com/o/oauth2/v2/auth".to_string()),
            projected_usd: None,
            metadata: json!({
                "operation": "reauthorize_browser",
                "redirect_uri": redirect_uri,
                "scopes": scopes,
            }),
            untrusted_excerpt: None,
        })?;
        Ok(GmailOAuthReauthorizePreflightReport {
            status: "ready".to_string(),
            redirect_uri: redirect_uri.to_string(),
            scopes,
            policy: "allowed".to_string(),
        })
    }

    pub fn gmail_oauth_exchange_code(
        &self,
        client_id: &str,
        redirect_uri: &str,
        code: &str,
        code_verifier: &str,
        client_secret: Option<&str>,
    ) -> Result<GmailOAuthTokenStoreReport> {
        let endpoint = std::env::var("ARCWELL_GMAIL_OAUTH_BASE")
            .unwrap_or_else(|_| "https://oauth2.googleapis.com".to_string());
        self.gmail_oauth_exchange_code_with_base(
            client_id,
            redirect_uri,
            code,
            code_verifier,
            client_secret,
            &endpoint,
        )
    }

    pub(crate) fn gmail_oauth_exchange_code_with_base(
        &self,
        client_id: &str,
        redirect_uri: &str,
        code: &str,
        code_verifier: &str,
        client_secret: Option<&str>,
        endpoint: &str,
    ) -> Result<GmailOAuthTokenStoreReport> {
        validate_query(client_id)?;
        validate_public_http_url(redirect_uri)?;
        validate_oauth_param(code, "authorization code")?;
        validate_oauth_param(code_verifier, "code verifier")?;
        let endpoint_url = validate_public_http_url(endpoint)?;
        self.policy_guard(PolicyRequest {
            action: "provider.oauth".to_string(),
            package: Some("arcwell-email".to_string()),
            provider: Some("gmail".to_string()),
            source: Some("gmail_oauth".to_string()),
            channel: Some("email".to_string()),
            subject: None,
            target: Some(endpoint_url.as_str().trim_end_matches('/').to_string()),
            projected_usd: Some(estimated_network_fetch_cost(1)),
            metadata: json!({
                "operation": "exchange_code",
                "redirect_uri": redirect_uri,
                "has_explicit_client_secret": client_secret.is_some()
            }),
            untrusted_excerpt: None,
        })?;
        self.require_cost_budget(
            "arcwell-email",
            "gmail_oauth_exchange",
            "gmail",
            "oauth_exchange",
            Some("gmail_oauth"),
            estimated_network_fetch_cost(1),
            "Gmail OAuth exchange",
        )?;
        let client_secret = self.resolve_gmail_oauth_client_secret(client_secret)?;
        let mut form = vec![
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("code_verifier", code_verifier),
        ];
        if client_secret.is_none() {
            form.push(("client_id", client_id));
        }
        let value = post_gmail_oauth_form(
            endpoint_url.as_str(),
            client_id,
            client_secret.as_deref(),
            &form,
        )?;
        self.store_gmail_token_response(&value)
    }

    pub fn gmail_oauth_refresh(
        &self,
        client_id: &str,
        client_secret: Option<&str>,
    ) -> Result<GmailOAuthTokenStoreReport> {
        let endpoint = self.configured_gmail_oauth_base()?;
        self.gmail_oauth_refresh_with_base(client_id, client_secret, &endpoint)
    }

    pub(crate) fn configured_gmail_oauth_base(&self) -> Result<String> {
        let endpoint = self
            .get_usable_secret_value("GMAIL_OAUTH_BASE")?
            .or_else(|| {
                self.get_usable_secret_value("GOOGLE_OAUTH_BASE")
                    .ok()
                    .flatten()
            })
            .or_else(|| std::env::var("ARCWELL_GMAIL_OAUTH_BASE").ok())
            .unwrap_or_else(|| "https://oauth2.googleapis.com".to_string());
        validate_public_http_url(&endpoint)?;
        Ok(endpoint)
    }

    pub(crate) fn gmail_oauth_refresh_with_base(
        &self,
        client_id: &str,
        client_secret: Option<&str>,
        endpoint: &str,
    ) -> Result<GmailOAuthTokenStoreReport> {
        validate_query(client_id)?;
        let endpoint_url = validate_public_http_url(endpoint)?;
        self.policy_guard(PolicyRequest {
            action: "provider.oauth".to_string(),
            package: Some("arcwell-email".to_string()),
            provider: Some("gmail".to_string()),
            source: Some("gmail_oauth".to_string()),
            channel: Some("email".to_string()),
            subject: None,
            target: Some(endpoint_url.as_str().trim_end_matches('/').to_string()),
            projected_usd: Some(estimated_network_fetch_cost(1)),
            metadata: json!({
                "operation": "refresh",
                "has_explicit_client_secret": client_secret.is_some()
            }),
            untrusted_excerpt: None,
        })?;
        self.require_cost_budget(
            "arcwell-email",
            "gmail_oauth_refresh",
            "gmail",
            "oauth_refresh",
            Some("gmail_oauth"),
            estimated_network_fetch_cost(1),
            "Gmail OAuth refresh",
        )?;
        let refresh_token = self
            .get_usable_secret_value("GMAIL_REFRESH_TOKEN")?
            .context("GMAIL_REFRESH_TOKEN is required")?;
        validate_oauth_param(&refresh_token, "refresh token")?;
        let client_secret = self.resolve_gmail_oauth_client_secret(client_secret)?;
        let mut form = vec![
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token.as_str()),
        ];
        if client_secret.is_none() {
            form.push(("client_id", client_id));
        }
        let value = post_gmail_oauth_form(
            endpoint_url.as_str(),
            client_id,
            client_secret.as_deref(),
            &form,
        )
        .map_err(|error| {
            anyhow::anyhow!(
                "{}",
                redact_secret_like_text(&error.to_string()).replace(&refresh_token, "[REDACTED]")
            )
        })?;
        self.store_gmail_token_response(&value)
    }

    pub(crate) fn resolve_gmail_oauth_client_secret(
        &self,
        explicit: Option<&str>,
    ) -> Result<Option<String>> {
        let secret = explicit
            .map(ToOwned::to_owned)
            .or_else(|| std::env::var("GMAIL_CLIENT_SECRET").ok())
            .or_else(|| std::env::var("GOOGLE_CLIENT_SECRET").ok())
            .or_else(|| {
                self.get_usable_secret_value("GMAIL_CLIENT_SECRET")
                    .ok()
                    .flatten()
            })
            .or_else(|| {
                self.get_usable_secret_value("GOOGLE_CLIENT_SECRET")
                    .ok()
                    .flatten()
            });
        if let Some(secret) = &secret
            && (secret.is_empty() || secret.len() > 20_000)
        {
            bail!("Gmail client secret is invalid");
        }
        Ok(secret)
    }

    pub(crate) fn store_gmail_token_response(
        &self,
        value: &Value,
    ) -> Result<GmailOAuthTokenStoreReport> {
        let mut stored = Vec::new();
        let expires_at = value
            .get("expires_in")
            .and_then(Value::as_i64)
            .filter(|seconds| *seconds > 0)
            .map(now_plus_seconds);
        if let Some(access_token) = value.get("access_token").and_then(Value::as_str) {
            self.set_secret_value_with_metadata(
                "GMAIL_ACCESS_TOKEN",
                access_token,
                "gmail",
                Some("gmail"),
                expires_at.as_deref(),
            )?;
            stored.push("GMAIL_ACCESS_TOKEN".to_string());
        }
        if let Some(refresh_token) = value.get("refresh_token").and_then(Value::as_str) {
            self.set_secret_value_with_metadata(
                "GMAIL_REFRESH_TOKEN",
                refresh_token,
                "gmail",
                Some("gmail"),
                None,
            )?;
            stored.push("GMAIL_REFRESH_TOKEN".to_string());
        }
        if stored.is_empty() {
            bail!("Gmail OAuth response did not include an access_token or refresh_token");
        }
        Ok(GmailOAuthTokenStoreReport {
            stored,
            token_type: value
                .get("token_type")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
            expires_in: value.get("expires_in").and_then(Value::as_i64),
            scope: value
                .get("scope")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
        })
    }

    pub(crate) fn configured_gmail_api_base(&self) -> Result<Option<String>> {
        let value = self
            .get_usable_secret_value("GMAIL_API_BASE")?
            .or_else(|| std::env::var("GMAIL_API_BASE").ok())
            .or_else(|| std::env::var("ARCWELL_GMAIL_API_BASE").ok());
        if let Some(value) = &value {
            validate_public_http_url(value)?;
        }
        Ok(value)
    }

    pub(crate) fn configured_agent_email_from(&self) -> Result<Option<String>> {
        let value = match self.get_usable_secret_value("ARCWELL_AGENT_EMAIL_FROM")? {
            Some(value) => Some(value),
            None => self
                .get_usable_secret_value("ARCWELL_AGENT_EMAIL")?
                .or_else(|| std::env::var("ARCWELL_AGENT_EMAIL_FROM").ok())
                .or_else(|| std::env::var("ARCWELL_AGENT_EMAIL").ok()),
        };
        value
            .map(|email| {
                normalize_email_address(&email)
                    .context("invalid configured Arcwell agent email sender")
            })
            .transpose()
    }

    pub(crate) fn send_existing_telegram_message(
        &self,
        message_id: &str,
        bot_token: &str,
        chat_id: &str,
        text: &str,
        api_base: Option<&str>,
    ) -> Result<TelegramSendReport> {
        validate_id(message_id)?;
        validate_notes(bot_token)?;
        validate_key(chat_id)?;
        validate_notes(text)?;
        let subject = format!("telegram:chat:{chat_id}");
        if !self.channel_subject_can_send("telegram", &subject)? {
            bail!("telegram subject is not authorized to send: {subject}");
        }
        self.policy_guard(PolicyRequest {
            action: "channel.send".to_string(),
            package: None,
            provider: Some("telegram".to_string()),
            source: Some("telegram_retry".to_string()),
            channel: Some("telegram".to_string()),
            subject: Some(subject.clone()),
            target: Some(chat_id.to_string()),
            projected_usd: None,
            metadata: json!({ "message_id": message_id, "retry": true }),
            untrusted_excerpt: Some(text.to_string()),
        })?;
        self.require_cost_budget(
            "arcwell-telegram",
            message_id,
            "telegram",
            "send_message",
            Some("telegram_retry"),
            estimated_channel_send_cost(),
            "Telegram retry",
        )?;
        let base = api_base.unwrap_or("https://api.telegram.org");
        let url = format!(
            "{}/bot{}/sendMessage",
            base.trim_end_matches('/'),
            bot_token
        );
        let client = Client::builder().timeout(Duration::from_secs(20)).build()?;
        let response = client
            .post(&url)
            .json(&json!({
                "chat_id": chat_id,
                "text": escape_telegram_markdown_v2(text),
                "parse_mode": "MarkdownV2"
            }))
            .send();
        let (status, response_json, error, retry_at) = match response {
            Ok(response) => {
                let status = response.status().as_u16();
                let retry_at = telegram_retry_at(status, response.headers());
                let response_json = response.json::<Value>().unwrap_or_else(|_| json!({}));
                (status, response_json, None, retry_at)
            }
            Err(error) => (
                0,
                json!({ "ok": false, "error": "request_failed" }),
                Some(telegram_request_error_summary(&error)),
                Some((Utc::now() + chrono::Duration::seconds(60)).to_rfc3339()),
            ),
        };
        let ok = (200..300).contains(&status)
            && response_json
                .get("ok")
                .and_then(Value::as_bool)
                .unwrap_or(false);
        let delivery = self.record_channel_delivery_attempt(
            message_id,
            "telegram",
            &subject,
            ok,
            i64::from(status),
            &response_json,
            error.as_deref(),
            retry_at.as_deref(),
        )?;
        let message =
            self.update_channel_message_status(message_id, if ok { "sent" } else { "failed" })?;
        Ok(TelegramSendReport {
            ok,
            status,
            response: response_json,
            message,
            delivery,
        })
    }
}

fn validate_delivery_observation_status(status: &str) -> Result<()> {
    validate_key(status)?;
    match status {
        "mailbox_observed" | "mailbox_not_found" | "mailbox_unknown" => Ok(()),
        _ => bail!(
            "delivery observation status must be mailbox_observed, mailbox_not_found, or mailbox_unknown"
        ),
    }
}

fn email_delivery_mailbox_observation_health_key(delivery_attempt_id: &str) -> String {
    format!("email:delivery:{delivery_attempt_id}:mailbox")
}

pub(crate) fn mailbox_observation_placement(
    observation_status: &str,
    evidence: &Value,
) -> Option<String> {
    match observation_status {
        "mailbox_not_found" => return Some("not_observed".to_string()),
        "mailbox_unknown" => return Some("unknown".to_string()),
        "mailbox_observed" => {}
        _ => return None,
    }
    if let Some(placement) = evidence.get("placement").and_then(Value::as_str) {
        return Some(placement.trim().to_ascii_lowercase());
    }
    if let Some(metadata_items) = evidence
        .get("gmail_message_metadata")
        .and_then(Value::as_array)
    {
        for metadata in metadata_items {
            if let Some(placement) = metadata.get("placement").and_then(Value::as_str) {
                return Some(placement.trim().to_ascii_lowercase());
            }
            if let Some(labels) = json_string_array(metadata.get("label_ids")) {
                return Some(gmail_message_placement(&labels).to_string());
            }
        }
    }
    json_string_array(evidence.get("label_ids").or_else(|| evidence.get("labels")))
        .map(|labels| gmail_message_placement(&labels).to_string())
}

fn email_delivery_mailbox_verification_state(observation: &ChannelDeliveryObservation) -> String {
    match observation.observation_status.as_str() {
        "mailbox_not_found" => "mailbox_not_observed".to_string(),
        "mailbox_unknown" => "mailbox_unknown".to_string(),
        "mailbox_observed" => match mailbox_observation_placement(
            &observation.observation_status,
            &observation.evidence,
        )
        .as_deref()
        {
            Some("inbox") => "mailbox_observed_inbox".to_string(),
            Some("trash") => "mailbox_bad_placement_trash".to_string(),
            Some("spam") => "mailbox_bad_placement_spam".to_string(),
            Some("sent") => "mailbox_bad_placement_sent".to_string(),
            Some("other") => "mailbox_bad_placement_other".to_string(),
            Some(_) => "mailbox_bad_placement_unknown".to_string(),
            None => "mailbox_observed_placement_unknown".to_string(),
        },
        _ => "mailbox_verification_unknown".to_string(),
    }
}

fn json_string_array(value: Option<&Value>) -> Option<Vec<String>> {
    let labels = value
        .and_then(Value::as_array)?
        .iter()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    (!labels.is_empty()).then_some(labels)
}

fn sanitize_delivery_observation_evidence(value: &Value) -> Result<Value> {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) => Ok(value.clone()),
        Value::String(text) => {
            validate_notes(text)?;
            Ok(Value::String(redact_secret_like_text(text)))
        }
        Value::Array(items) => items
            .iter()
            .map(sanitize_delivery_observation_evidence)
            .collect::<Result<Vec<_>>>()
            .map(Value::Array),
        Value::Object(map) => {
            let mut sanitized = serde_json::Map::with_capacity(map.len());
            for (key, value) in map {
                validate_key(key)?;
                sanitized.insert(key.clone(), sanitize_delivery_observation_evidence(value)?);
            }
            Ok(Value::Object(sanitized))
        }
    }
}

fn email_delivery_verification_search_query(
    message_id: Option<&str>,
) -> (Option<String>, bool, Option<String>) {
    let Some(message_id) = message_id else {
        return (None, false, Some("message_id_missing".to_string()));
    };
    (Some(format!("rfc822msgid:{message_id}")), true, None)
}

fn arcwell_outbound_email_message_id(message_id: &str, from: &str) -> Result<String> {
    validate_id(message_id)?;
    let domain = from
        .rsplit_once('@')
        .map(|(_, domain)| domain)
        .filter(|domain| !domain.trim().is_empty())
        .context("email sender must include a domain")?;
    validate_query(domain)?;
    Ok(format!("<arcwell-{message_id}@{}>", domain.trim()))
}

fn attach_arcwell_email_delivery_metadata(
    mut response: Value,
    channel_message_id: &str,
    outbound_message_id: &str,
) -> Value {
    let provider_message_id = response
        .get("result")
        .and_then(|result| result.get("message_id"))
        .or_else(|| response.get("result").and_then(|result| result.get("id")))
        .and_then(Value::as_str)
        .unwrap_or(outbound_message_id)
        .to_string();
    if let Value::Object(map) = &mut response {
        map.insert(
            "arcwell".to_string(),
            json!({
                "channel_message_id": channel_message_id,
                "requested_outbound_message_id": outbound_message_id,
                "outbound_message_id": provider_message_id,
                "verification_search_query": format!("rfc822msgid:{provider_message_id}")
            }),
        );
    }
    response
}

fn default_gmail_oauth_scopes() -> Vec<String> {
    vec![
        "https://www.googleapis.com/auth/gmail.readonly".to_string(),
        "https://www.googleapis.com/auth/gmail.modify".to_string(),
        "https://www.googleapis.com/auth/userinfo.email".to_string(),
    ]
}

fn post_gmail_oauth_form(
    endpoint: &str,
    client_id: &str,
    client_secret: Option<&str>,
    form: &[(&str, &str)],
) -> Result<Value> {
    let base = validate_public_http_url(endpoint)?;
    let url = base.join("/token")?;
    let client = Client::builder().timeout(Duration::from_secs(20)).build()?;
    let mut request = client
        .post(url)
        .header(ACCEPT, "application/json")
        .header("user-agent", "arcwell/0.1")
        .form(form);
    if let Some(client_secret) = client_secret {
        request = request.basic_auth(client_id, Some(client_secret));
    }
    let response = request.send().context("Gmail OAuth token request failed")?;
    let status = response.status();
    let text = response.text().unwrap_or_default();
    if !status.is_success() {
        bail!(
            "Gmail OAuth token endpoint failed with HTTP {}: {}",
            status.as_u16(),
            excerpt(&redact_secret_like_text(&text), 300)
        );
    }
    serde_json::from_str(&text).context("Gmail OAuth token endpoint returned invalid JSON")
}

struct GmailSearchResult {
    message_ids: Vec<String>,
    thread_ids: Vec<String>,
    result_count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct GmailMessageMetadata {
    id: String,
    thread_id: Option<String>,
    label_ids: Vec<String>,
    placement: String,
}

fn gmail_mailbox_verifier_source_health_key() -> &'static str {
    "email:gmail-mailbox-verifier"
}

fn gmail_mailbox_repair_source_health_key() -> &'static str {
    "email:gmail-mailbox-repair"
}

fn gmail_search_message_ids(
    client: &Client,
    api_base: &str,
    access_token: &str,
    search_query: &str,
) -> Result<GmailSearchResult> {
    validate_notes(access_token)?;
    validate_query(search_query)?;
    let mut url = Url::parse(&format!(
        "{}/gmail/v1/users/me/messages",
        api_base.trim_end_matches('/')
    ))
    .with_context(|| format!("building Gmail messages URL from {api_base}"))?;
    url.query_pairs_mut()
        .append_pair("q", search_query)
        .append_pair("maxResults", "10");
    let response = client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {access_token}"))
        .send()
        .context("searching Gmail mailbox for delivery verification")?;
    let status = response.status();
    let body = response.text().unwrap_or_default();
    if !status.is_success() {
        bail!(
            "Gmail mailbox verification search failed with HTTP {}: {}",
            status.as_u16(),
            excerpt(&redact_secret_like_text(&body), 300)
        );
    }
    let value: Value = serde_json::from_str(&body).context("parsing Gmail search response")?;
    let messages = value
        .get("messages")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut message_ids = Vec::new();
    let mut thread_ids = Vec::new();
    for item in messages {
        if let Some(id) = item.get("id").and_then(Value::as_str) {
            validate_query(id)?;
            message_ids.push(id.to_string());
        }
        if let Some(thread_id) = item.get("threadId").and_then(Value::as_str) {
            validate_query(thread_id)?;
            thread_ids.push(thread_id.to_string());
        }
    }
    Ok(GmailSearchResult {
        result_count: message_ids.len(),
        message_ids,
        thread_ids,
    })
}

fn gmail_modify_message_labels(
    client: &Client,
    api_base: &str,
    access_token: &str,
    message_id: &str,
    add_label_ids: &[&str],
    remove_label_ids: &[&str],
) -> Result<Value> {
    validate_notes(access_token)?;
    validate_query(message_id)?;
    for label in add_label_ids.iter().chain(remove_label_ids.iter()) {
        validate_key(label)?;
    }
    let url = Url::parse(&format!(
        "{}/gmail/v1/users/me/messages/{}/modify",
        api_base.trim_end_matches('/'),
        message_id
    ))
    .with_context(|| format!("building Gmail message modify URL from {api_base}"))?;
    let response = client
        .post(url)
        .header(AUTHORIZATION, format!("Bearer {access_token}"))
        .header(CONTENT_TYPE, "application/json")
        .body(
            json!({
                "addLabelIds": add_label_ids,
                "removeLabelIds": remove_label_ids
            })
            .to_string(),
        )
        .send()
        .with_context(|| format!("modifying Gmail labels for message {message_id}"))?;
    let status = response.status();
    let body = response.text().unwrap_or_default();
    if !status.is_success() {
        bail!(
            "Gmail mailbox placement repair failed with HTTP {}: {}",
            status.as_u16(),
            excerpt(&redact_secret_like_text(&body), 300)
        );
    }
    serde_json::from_str(&body).context("parsing Gmail modify response")
}

fn gmail_fetch_message_metadata(
    client: &Client,
    api_base: &str,
    access_token: &str,
    message_ids: &[String],
) -> Result<Vec<GmailMessageMetadata>> {
    validate_notes(access_token)?;
    let mut metadata = Vec::new();
    for message_id in message_ids.iter().take(10) {
        validate_query(message_id)?;
        let mut url = Url::parse(&format!(
            "{}/gmail/v1/users/me/messages/{}",
            api_base.trim_end_matches('/'),
            message_id
        ))
        .with_context(|| format!("building Gmail message metadata URL from {api_base}"))?;
        url.query_pairs_mut().append_pair("format", "metadata");
        let response = client
            .get(url)
            .header(AUTHORIZATION, format!("Bearer {access_token}"))
            .send()
            .with_context(|| format!("fetching Gmail metadata for message {message_id}"))?;
        let status = response.status();
        let body = response.text().unwrap_or_default();
        if !status.is_success() {
            bail!(
                "Gmail mailbox metadata fetch failed with HTTP {}: {}",
                status.as_u16(),
                excerpt(&redact_secret_like_text(&body), 300)
            );
        }
        let value: Value =
            serde_json::from_str(&body).context("parsing Gmail metadata response")?;
        let label_ids = value
            .get("labelIds")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(|label| {
                validate_key(label)?;
                Ok(label.to_string())
            })
            .collect::<Result<Vec<_>>>()?;
        let placement = gmail_message_placement(&label_ids).to_string();
        metadata.push(GmailMessageMetadata {
            id: value
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(message_id)
                .to_string(),
            thread_id: value
                .get("threadId")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
            label_ids,
            placement,
        });
    }
    Ok(metadata)
}

fn gmail_message_placement(label_ids: &[String]) -> &'static str {
    if label_ids.iter().any(|label| label == "TRASH") {
        "trash"
    } else if label_ids.iter().any(|label| label == "SPAM") {
        "spam"
    } else if label_ids.iter().any(|label| label == "INBOX") {
        "inbox"
    } else if label_ids.iter().any(|label| label == "SENT") {
        "sent"
    } else {
        "other"
    }
}
