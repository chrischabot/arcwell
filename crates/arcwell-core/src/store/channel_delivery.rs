use super::*;

impl Store {
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
        validate_notes(body)?;
        validate_key(status)?;
        if let Some(project_id) = project_id {
            validate_id(project_id)?;
            self.get_project(project_id)?
                .with_context(|| format!("project not found: {project_id}"))?;
        }
        let sanitized_body = sanitize_channel_body(body)?;
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
        let attempt: i64 = self.conn.query_row(
            "SELECT COALESCE(MAX(attempt), 0) + 1 FROM channel_delivery_attempts WHERE message_id = ?1",
            params![message_id],
            |row| row.get(0),
        )?;
        let id = Uuid::new_v4().to_string();
        let created_at = now();
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
                serde_json::to_string(response)?,
                error,
                retry_at,
                created_at
            ],
        )?;
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
        validate_notes(text)?;
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
            untrusted_excerpt: Some(format!("{subject}\n\n{text}")),
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
        validate_notes(text)?;
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
        let endpoint = format!(
            "{}/accounts/{}/email/sending/send",
            api_base
                .unwrap_or("https://api.cloudflare.com/client/v4")
                .trim_end_matches('/'),
            account_id
        );
        let mut headers = Map::new();
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
        validate_notes(text)?;
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
                "context": metadata,
            }),
            untrusted_excerpt: Some(format!("{subject}\n\n{text}")),
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
        let body = json!({
            "from": from,
            "to": to,
            "subject": subject,
            "text": text,
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
