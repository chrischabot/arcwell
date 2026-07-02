use super::*;

const DIGEST_CANDIDATE_DELIVERY_TEXT_MAX_BYTES: usize = 12_000;
const DIGEST_CANDIDATE_DELIVERY_OMISSION_NOTE: &str =
    "\n\n_Additional source details were omitted to keep this notification deliverable._";

fn issue_schedule_job_kind(schedule: &IssueSchedule) -> Result<&'static str> {
    match schedule.kind.as_str() {
        "knowledge_daily_briefing" => Ok("knowledge_daily_briefing"),
        other => bail!("unsupported issue schedule kind: {other}"),
    }
}

impl Store {
    pub fn create_digest_candidate(
        &self,
        topic: &str,
        source_card_ids: &[String],
    ) -> Result<DigestCandidate> {
        validate_query(topic)?;
        let source_card_ids = source_card_ids
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        if source_card_ids.is_empty() {
            bail!("digest candidate requires at least one source card");
        }
        for id in &source_card_ids {
            validate_id(id)?;
            self.read_source_card(id)?
                .with_context(|| format!("source card not found: {id}"))?;
        }
        let source_card_ids_json = serde_json::to_string(&source_card_ids)?;
        if let Some(existing) = self
            .conn
            .query_row(
                r#"
                SELECT id, topic, score, reason, status, source_card_ids_json,
                       review_status, reviewed_at, reviewed_by, review_note,
                       created_at, updated_at
                FROM digest_candidates
                WHERE topic = ?1 AND source_card_ids_json = ?2
                ORDER BY updated_at DESC
                LIMIT 1
                "#,
                params![topic, source_card_ids_json],
                digest_candidate_from_row,
            )
            .optional()?
        {
            return Ok(existing);
        }
        let (score, reason) = score_digest_candidate(topic, source_card_ids.len());
        let status = if score >= 0.75 { "ready" } else { "pending" };
        let id = Uuid::new_v4().to_string();
        let timestamp = now();
        self.conn.execute(
            r#"
            INSERT INTO digest_candidates
              (id, topic, score, reason, status, source_card_ids_json, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
            "#,
            params![
                id,
                topic,
                score,
                reason,
                status,
                source_card_ids_json,
                timestamp
            ],
        )?;
        self.get_digest_candidate(&id)?
            .with_context(|| format!("inserted digest candidate not found: {id}"))
    }

    pub fn list_digest_candidates(&self) -> Result<Vec<DigestCandidate>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, topic, score, reason, status, source_card_ids_json,
                   review_status, reviewed_at, reviewed_by, review_note,
                   created_at, updated_at
            FROM digest_candidates
            ORDER BY score DESC, updated_at DESC
            "#,
        )?;
        rows(stmt.query_map([], digest_candidate_from_row)?)
    }

    pub fn get_digest_candidate(&self, id: &str) -> Result<Option<DigestCandidate>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, topic, score, reason, status, source_card_ids_json,
                       review_status, reviewed_at, reviewed_by, review_note,
                       created_at, updated_at
                FROM digest_candidates
                WHERE id = ?1
                "#,
                params![id],
                digest_candidate_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn review_digest_candidate(
        &self,
        id: &str,
        review_status: &str,
        reviewed_by: Option<&str>,
        review_note: Option<&str>,
    ) -> Result<DigestCandidate> {
        validate_id(id)?;
        validate_digest_review_status(review_status)?;
        if let Some(reviewed_by) = reviewed_by {
            validate_notes(reviewed_by)?;
        }
        if let Some(review_note) = review_note {
            validate_notes(review_note)?;
        }
        self.get_digest_candidate(id)?
            .with_context(|| format!("digest candidate not found: {id}"))?;
        let timestamp = now();
        let status = match review_status {
            "approved" => "approved",
            "rejected" => "rejected",
            other => bail!("unsupported digest candidate review transition: {other}"),
        };
        self.conn.execute(
            r#"
            UPDATE digest_candidates
            SET status = ?2,
                review_status = ?3,
                reviewed_at = ?4,
                reviewed_by = ?5,
                review_note = ?6,
                updated_at = ?4
            WHERE id = ?1
            "#,
            params![
                id,
                status,
                review_status,
                timestamp,
                reviewed_by,
                review_note
            ],
        )?;
        self.get_digest_candidate(id)?
            .with_context(|| format!("reviewed digest candidate not found: {id}"))
    }

    pub fn approve_digest_candidate(
        &self,
        id: &str,
        reviewed_by: Option<&str>,
        review_note: Option<&str>,
    ) -> Result<DigestCandidate> {
        self.review_digest_candidate(id, "approved", reviewed_by, review_note)
    }

    pub fn reject_digest_candidate(
        &self,
        id: &str,
        reviewed_by: Option<&str>,
        review_note: Option<&str>,
    ) -> Result<DigestCandidate> {
        self.review_digest_candidate(id, "rejected", reviewed_by, review_note)
    }

    pub fn check_digest_candidate_delivery(
        &self,
        id: &str,
        channel: &str,
        subject: &str,
        target: Option<&str>,
    ) -> Result<DigestCandidateDeliveryGate> {
        validate_id(id)?;
        validate_key(channel)?;
        validate_query(subject)?;
        if let Some(target) = target {
            validate_query(target)?;
        }
        let candidate = self
            .get_digest_candidate(id)?
            .with_context(|| format!("digest candidate not found: {id}"))?;
        let (policy_package, policy_source) =
            self.digest_candidate_delivery_policy_context(&candidate)?;
        let mut gate_reason = None;
        if candidate.review_status != "approved" || candidate.status != "approved" {
            gate_reason = Some(format!(
                "digest candidate delivery requires approved review; status={}, review_status={}",
                candidate.status, candidate.review_status
            ));
        }
        let decision = self.policy_check(PolicyRequest {
            action: "digest_candidate.deliver".to_string(),
            package: Some(policy_package.to_string()),
            provider: None,
            source: Some(policy_source.to_string()),
            channel: Some(channel.to_string()),
            subject: Some(subject.to_string()),
            target: target.map(ToOwned::to_owned),
            projected_usd: None,
            metadata: json!({
                "candidate_id": candidate.id.clone(),
                "candidate_status": candidate.status.clone(),
                "review_status": candidate.review_status.clone(),
                "source_card_count": candidate.source_card_ids.len(),
                "policy_package": policy_package,
                "policy_source": policy_source,
                "gate_block_reason": gate_reason.clone(),
            }),
            untrusted_excerpt: Some(candidate.topic.clone()),
        })?;
        let allowed = gate_reason.is_none() && decision.allowed;
        let reason = gate_reason.unwrap_or_else(|| decision.reason.clone());
        Ok(DigestCandidateDeliveryGate {
            candidate,
            allowed,
            reason,
            policy_decision: decision,
        })
    }

    pub(crate) fn digest_candidate_delivery_policy_context(
        &self,
        candidate: &DigestCandidate,
    ) -> Result<(&'static str, &'static str)> {
        let mut has_x_origin = false;
        let mut has_credential_reminder = false;
        for source_card_id in &candidate.source_card_ids {
            let source_card = self
                .read_source_card(source_card_id)?
                .with_context(|| format!("digest source card not found: {source_card_id}"))?;
            if digest_source_card_is_knowledge_daily_briefing(&source_card) {
                return Ok(("arcwell-knowledge", "knowledge_daily_briefing_delivery"));
            }
            if digest_source_card_is_x_origin(&source_card) {
                has_x_origin = true;
            }
            if digest_source_card_is_credential_reminder(&source_card) {
                has_credential_reminder = true;
            }
        }
        if has_credential_reminder {
            return Ok(("arcwell-ops", "credential_reminder_delivery"));
        }
        if has_x_origin {
            return Ok(("arcwell-x", "x_digest_delivery"));
        }
        Ok(("arcwell-librarian", "digest_candidate_delivery"))
    }

    pub fn require_digest_candidate_delivery_allowed(
        &self,
        id: &str,
        channel: &str,
        subject: &str,
        target: Option<&str>,
    ) -> Result<DigestCandidateDeliveryGate> {
        let gate = self.check_digest_candidate_delivery(id, channel, subject, target)?;
        if !gate.allowed {
            bail!("digest candidate delivery denied: {}", gate.reason);
        }
        Ok(gate)
    }

    pub fn send_digest_candidate_telegram(
        &self,
        id: &str,
        bot_token: &str,
        chat_id: &str,
        idempotency_key: Option<&str>,
        api_base: Option<&str>,
    ) -> Result<DigestCandidateTelegramDeliveryReport> {
        validate_id(id)?;
        validate_notes(bot_token)?;
        validate_key(chat_id)?;
        let subject = format!("telegram:chat:{chat_id}");
        let idempotency_key =
            digest_delivery_idempotency_key(id, "telegram", &subject, &subject, idempotency_key)?;
        let delivery = self.get_or_create_digest_delivery(
            id,
            "telegram",
            &subject,
            &subject,
            &idempotency_key,
        )?;
        if let Some(attempt_id) = delivery.channel_delivery_attempt_id.as_deref() {
            let attempt = self
                .get_channel_delivery_attempt(attempt_id)?
                .with_context(|| format!("channel delivery attempt not found: {attempt_id}"))?;
            let message = self
                .get_channel_message(&attempt.message_id)?
                .with_context(|| format!("channel message not found: {}", attempt.message_id))?;
            let gate =
                self.check_digest_candidate_delivery(id, "telegram", &subject, Some(&subject))?;
            return Ok(DigestCandidateTelegramDeliveryReport {
                gate,
                digest_delivery: delivery,
                telegram: Some(TelegramSendReport {
                    ok: attempt.ok,
                    status: attempt.provider_status.clamp(0, u16::MAX as i64) as u16,
                    response: attempt.response.clone(),
                    message,
                    delivery: attempt,
                }),
                replayed: true,
            });
        }
        let gate =
            self.check_digest_candidate_delivery(id, "telegram", &subject, Some(&subject))?;
        if !gate.allowed {
            let delivery = self.update_digest_delivery(
                &delivery.id,
                "blocked",
                Some(&gate.policy_decision.id),
                None,
                None,
                Some(&gate.reason),
                None,
            )?;
            bail!(
                "digest candidate delivery denied: {} (digest_delivery_id={})",
                gate.reason,
                delivery.id
            );
        }
        let text = self.digest_candidate_delivery_text(&gate.candidate)?;
        match self.send_telegram_message(bot_token, chat_id, &text, api_base) {
            Ok(telegram) => {
                let status = if telegram.ok { "sent" } else { "failed" };
                let digest_delivery = self.update_digest_delivery(
                    &delivery.id,
                    status,
                    Some(&gate.policy_decision.id),
                    Some(&telegram.message.id),
                    Some(&telegram.delivery.id),
                    telegram.delivery.error.as_deref(),
                    telegram.delivery.retry_at.as_deref(),
                )?;
                Ok(DigestCandidateTelegramDeliveryReport {
                    gate,
                    digest_delivery,
                    telegram: Some(telegram),
                    replayed: false,
                })
            }
            Err(error) => {
                let error_text = redact_secret_like_text(&error.to_string());
                let delivery = self.update_digest_delivery(
                    &delivery.id,
                    "blocked",
                    Some(&gate.policy_decision.id),
                    None,
                    None,
                    Some(&error_text),
                    None,
                )?;
                bail!(
                    "digest candidate Telegram delivery blocked: {} (digest_delivery_id={})",
                    error_text,
                    delivery.id
                );
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn send_digest_candidate_email(
        &self,
        id: &str,
        account_id: &str,
        api_token: &str,
        from: &str,
        to: &str,
        idempotency_key: Option<&str>,
        api_base: Option<&str>,
    ) -> Result<DigestCandidateEmailDeliveryReport> {
        validate_id(id)?;
        validate_key(account_id)?;
        validate_notes(api_token)?;
        let from = normalize_email_address(from).context("invalid email from address")?;
        let to = normalize_email_address(to).context("invalid email to address")?;
        let subject = format!("email:{to}");
        let idempotency_key =
            digest_delivery_idempotency_key(id, "email", &subject, &subject, idempotency_key)?;
        let delivery =
            self.get_or_create_digest_delivery(id, "email", &subject, &subject, &idempotency_key)?;
        if let Some(attempt_id) = delivery.channel_delivery_attempt_id.as_deref() {
            let attempt = self
                .get_channel_delivery_attempt(attempt_id)?
                .with_context(|| format!("channel delivery attempt not found: {attempt_id}"))?;
            let message = self
                .get_channel_message(&attempt.message_id)?
                .with_context(|| format!("channel message not found: {}", attempt.message_id))?;
            let gate =
                self.check_digest_candidate_delivery(id, "email", &subject, Some(&subject))?;
            self.reconcile_issue_schedule_tick_for_digest_delivery(&delivery)?;
            return Ok(DigestCandidateEmailDeliveryReport {
                gate,
                digest_delivery: delivery,
                email: Some(EmailSendReport {
                    ok: attempt.ok,
                    status: attempt.provider_status.clamp(0, u16::MAX as i64) as u16,
                    response: attempt.response.clone(),
                    message,
                    delivery: attempt,
                }),
                replayed: true,
            });
        }
        let gate = self.check_digest_candidate_delivery(id, "email", &subject, Some(&subject))?;
        if !gate.allowed {
            let delivery = self.update_digest_delivery(
                &delivery.id,
                "blocked",
                Some(&gate.policy_decision.id),
                None,
                None,
                Some(&gate.reason),
                None,
            )?;
            bail!(
                "digest candidate delivery denied: {} (digest_delivery_id={})",
                gate.reason,
                delivery.id
            );
        }
        let subject_line = digest_candidate_email_subject(&gate.candidate);
        let (text, html) = Self::digest_candidate_email_body(
            &subject_line,
            self.digest_candidate_delivery_text(&gate.candidate)?,
        )?;
        match self.send_cloudflare_email(
            account_id,
            api_token,
            &from,
            &to,
            &subject_line,
            &text,
            Some(&html),
            None,
            api_base,
        ) {
            Ok(email) => {
                let status = if email.ok { "sent" } else { "failed" };
                let digest_delivery = self.update_digest_delivery(
                    &delivery.id,
                    status,
                    Some(&gate.policy_decision.id),
                    Some(&email.message.id),
                    Some(&email.delivery.id),
                    email.delivery.error.as_deref(),
                    email.delivery.retry_at.as_deref(),
                )?;
                self.reconcile_issue_schedule_tick_for_digest_delivery(&digest_delivery)?;
                Ok(DigestCandidateEmailDeliveryReport {
                    gate,
                    digest_delivery,
                    email: Some(email),
                    replayed: false,
                })
            }
            Err(error) => {
                let error_text = redact_secret_like_text(&error.to_string());
                let delivery = self.update_digest_delivery(
                    &delivery.id,
                    "blocked",
                    Some(&gate.policy_decision.id),
                    None,
                    None,
                    Some(&error_text),
                    None,
                )?;
                self.reconcile_issue_schedule_tick_for_digest_delivery(&delivery)?;
                bail!(
                    "digest candidate Email delivery blocked: {} (digest_delivery_id={})",
                    error_text,
                    delivery.id
                );
            }
        }
    }

    pub(crate) fn reconcile_issue_schedule_tick_for_digest_delivery(
        &self,
        delivery: &DigestDelivery,
    ) -> Result<()> {
        let Some(tick_key) = delivery.idempotency_key.strip_prefix("issue-schedule-") else {
            return Ok(());
        };
        let tick_status = match delivery.status.as_str() {
            "sent" => "sent",
            "blocked" => "blocked",
            "failed" => "failed",
            _ => return Ok(()),
        };
        self.conn.execute(
            r#"
            UPDATE issue_schedule_ticks
            SET status = ?2,
                candidate_id = ?3,
                delivery_id = ?4,
                error = ?5,
                updated_at = ?6
            WHERE tick_key = ?1
            "#,
            params![
                tick_key,
                tick_status,
                delivery.candidate_id,
                delivery.id,
                delivery.error,
                now()
            ],
        )?;
        Ok(())
    }

    pub(crate) fn digest_candidate_delivery_text(
        &self,
        candidate: &DigestCandidate,
    ) -> Result<String> {
        if digest_topic_is_knowledge_daily_briefing(&candidate.topic) {
            for source_card_id in &candidate.source_card_ids {
                let card = self
                    .read_source_card(source_card_id)?
                    .with_context(|| format!("digest source card not found: {source_card_id}"))?;
                if digest_source_card_is_knowledge_daily_briefing(&card) {
                    return Self::knowledge_daily_briefing_delivery_text(candidate, &card);
                }
            }
        }
        let mut cards = Vec::new();
        for source_card_id in candidate.source_card_ids.iter().take(12) {
            let card = self
                .read_source_card(source_card_id)?
                .with_context(|| format!("digest source card not found: {source_card_id}"))?;
            cards.push(card);
        }
        if cards.iter().any(digest_source_card_is_credential_reminder) {
            return Ok(Self::cap_digest_candidate_delivery_text(
                Self::credential_reminder_delivery_text(candidate, &cards),
            ));
        }
        let topic = Self::digest_human_topic(&candidate.topic);
        let mut lines = vec![
            format!("X bookmark report: {topic}"),
            String::new(),
            "Bottom line".to_string(),
            format!(
                "Your saved X items are pointing toward {}. {}",
                topic,
                Self::digest_signal_sentence(&cards)
            ),
            String::new(),
            "What happened".to_string(),
        ];
        for card in cards.iter().take(7) {
            lines.push(format!(
                "- [{}]({}) - {}",
                Self::digest_source_label(card),
                card.url,
                excerpt(&Self::digest_card_evidence_text(card), 260)
            ));
        }
        lines.extend([
            String::new(),
            "Why it matters".to_string(),
            Self::digest_why_it_matters(&topic, &cards),
            String::new(),
            "Reception and context".to_string(),
            Self::digest_reception_context(&cards),
            String::new(),
            "What to watch".to_string(),
            "- Later updates should be tied back to the same story: primary-source confirmation, repeated independent mentions, availability changes, benchmark movement, and reaction shifts.".to_string(),
            String::new(),
            "Further reading".to_string(),
        ]);
        for (index, card) in cards.iter().enumerate() {
            lines.push(format!(
                "{}. [{}]({})",
                index + 1,
                Self::digest_source_label(card),
                card.url
            ));
        }
        if candidate.source_card_ids.len() > cards.len() {
            lines.push(format!(
                "... {} more saved sources omitted from notification text.",
                candidate.source_card_ids.len().saturating_sub(cards.len())
            ));
        }
        Ok(Self::cap_digest_candidate_delivery_text(lines.join("\n")))
    }

    pub(crate) fn knowledge_daily_briefing_delivery_text(
        candidate: &DigestCandidate,
        briefing_card: &SourceCard,
    ) -> Result<String> {
        let body = briefing_card.summary.trim();
        let text = if body.starts_with("# ") || body.starts_with("## ") {
            body.to_string()
        } else {
            format!(
                "# {}\n\n{}",
                Self::digest_human_topic(&candidate.topic),
                body
            )
        };
        let forbidden_reader_terms = daily_briefing_forbidden_reader_terms(&text);
        if !forbidden_reader_terms.is_empty() {
            bail!(
                "knowledge daily briefing delivery text contains internal pipeline language: {}",
                forbidden_reader_terms.join(", ")
            );
        }
        Ok(Self::cap_digest_candidate_delivery_text(text))
    }

    pub(crate) fn credential_reminder_delivery_text(
        candidate: &DigestCandidate,
        cards: &[SourceCard],
    ) -> String {
        let topic = Self::digest_human_topic(&candidate.topic);
        let warning_count = cards
            .iter()
            .filter_map(|card| {
                card.metadata
                    .get("warning_count")
                    .and_then(Value::as_u64)
                    .map(|value| value as usize)
            })
            .sum::<usize>()
            .max(cards.len());
        let mut lines = vec![
            format!("Arcwell credential reminder: {topic}"),
            String::new(),
            "Bottom line".to_string(),
            format!(
                "Arcwell found {warning_count} credential health warning(s) that can break scheduled ingestion, provider refresh, or outbound delivery if left unresolved."
            ),
            String::new(),
            "What needs attention".to_string(),
        ];
        let mut shown_warnings = 0usize;
        for (index, card) in cards.iter().take(8).enumerate() {
            let label = Self::digest_source_label(card);
            for claim in card.claims.iter().take(10) {
                shown_warnings += 1;
                lines.push(format!(
                    "- [S{}] {}: {}",
                    index + 1,
                    label,
                    excerpt(
                        &html_unescape_basic(&escape_markdown_line(&claim.claim)),
                        360
                    )
                ));
            }
        }
        if shown_warnings == 0 {
            for (index, card) in cards.iter().take(8).enumerate() {
                lines.push(format!(
                    "- [S{}] {}",
                    index + 1,
                    Self::digest_card_takeaway(card, 320)
                ));
            }
        }
        lines.extend([
            String::new(),
            "Why it matters".to_string(),
            "Scheduled workers only stay useful when the credentials behind provider reads, refreshes, and delivery channels are valid before the next run. This reminder is generated from local secret-health metadata and does not include credential values.".to_string(),
            String::new(),
            "Arcwell action and escalation".to_string(),
            "- Credential names, scopes, providers, expiry metadata, and warnings are evidence; raw secret values are intentionally omitted.".to_string(),
            "- Scheduled ingestion and delivery should use refresh/probe paths where available and keep stale or failed credentials visible in ops until recovered.".to_string(),
            "- Human action is required only where the provider forces re-authorization, rotation, or a new consent grant.".to_string(),
            String::new(),
            "Evidence appendix".to_string(),
        ]);
        for (index, card) in cards.iter().enumerate() {
            lines.push(format!(
                "[S{}] {} - {}",
                index + 1,
                Self::digest_source_label(card),
                excerpt(&card.url, 180)
            ));
        }
        if candidate.source_card_ids.len() > cards.len() {
            lines.push(format!(
                "... {} more source cards omitted from notification text.",
                candidate.source_card_ids.len().saturating_sub(cards.len())
            ));
        }
        lines.join("\n")
    }

    pub(crate) fn cap_digest_candidate_delivery_text(text: String) -> String {
        Self::cap_digest_candidate_delivery_text_to(text, DIGEST_CANDIDATE_DELIVERY_TEXT_MAX_BYTES)
    }

    pub(crate) fn digest_candidate_email_body(
        subject: &str,
        text: String,
    ) -> Result<(String, String)> {
        let mut max_bytes = text.len().min(DIGEST_CANDIDATE_DELIVERY_TEXT_MAX_BYTES);
        loop {
            let current = Self::cap_digest_candidate_delivery_text_to(text.clone(), max_bytes);
            match render_email_html_from_markdown(subject, &current) {
                Ok(html) => return Ok((current, html)),
                Err(error)
                    if error.to_string().contains("notes are too long") && max_bytes > 2_000 =>
                {
                    max_bytes = (max_bytes * 3 / 4).max(2_000);
                }
                Err(error) => return Err(error).context("digest candidate email body is invalid"),
            }
        }
    }

    pub(crate) fn cap_digest_candidate_delivery_text_to(text: String, max_bytes: usize) -> String {
        if text.len() <= max_bytes {
            return text;
        }
        let note = DIGEST_CANDIDATE_DELIVERY_OMISSION_NOTE;
        let budget = max_bytes.saturating_sub(note.len());
        let mut capped = excerpt_preserving_whitespace(&text, budget);
        capped.truncate(capped.trim_end().len());
        capped.push_str(note);
        if capped.len() <= max_bytes {
            capped
        } else {
            excerpt_preserving_whitespace(&capped, max_bytes)
        }
    }

    pub(crate) fn digest_human_topic(topic: &str) -> String {
        let topic = topic
            .strip_prefix("X bookmark trend:")
            .or_else(|| topic.strip_prefix("x bookmark trend:"))
            .unwrap_or(topic)
            .trim();
        if topic.is_empty() {
            "a saved X evidence cluster".to_string()
        } else {
            topic.to_string()
        }
    }

    pub(crate) fn digest_card_takeaway(card: &SourceCard, max_chars: usize) -> String {
        let label = Self::digest_source_label(card);
        let evidence = Self::digest_card_evidence_text(card);
        format!("{label}: {}", excerpt(&evidence, max_chars))
    }

    pub(crate) fn digest_card_evidence_text(card: &SourceCard) -> String {
        let claim = card
            .claims
            .iter()
            .find(|claim| claim.kind != "source_text" && claim.claim.trim().len() >= 20)
            .or_else(|| {
                card.claims
                    .iter()
                    .find(|claim| claim.claim.trim().len() >= 20)
            })
            .map(|claim| claim.claim.as_str())
            .unwrap_or(&card.summary);
        let text = if claim.trim().is_empty() {
            &card.summary
        } else {
            claim
        };
        html_unescape_basic(&escape_markdown_line(text))
    }

    pub(crate) fn digest_source_label(card: &SourceCard) -> String {
        if card.provider == "x" || card.source_type.contains("x") {
            let title = card.title.trim();
            if let Some(rest) = title.strip_prefix("X: ") {
                let mut parts = rest.split_whitespace();
                if let Some(handle) = parts.next() {
                    return format!("@{handle}");
                }
            }
        }
        excerpt(&html_unescape_basic(&card.title), 90)
    }

    pub(crate) fn digest_signal_sentence(cards: &[SourceCard]) -> String {
        if cards.is_empty() {
            return "No readable evidence was available, so this should not be delivered."
                .to_string();
        }
        let launch = Self::digest_signal_count(
            cards,
            &[
                "launch",
                "launched",
                "release",
                "released",
                "upgrade",
                "announced",
                "ships",
                "rolled out",
            ],
        );
        let tooling = Self::digest_signal_count(
            cards,
            &[
                "mcp",
                "xcode",
                "agent",
                "agents",
                "tool",
                "workflow",
                "runtime",
                "computer-use",
            ],
        );
        let model = Self::digest_signal_count(
            cards,
            &[
                "model",
                "multimodal",
                "gemma",
                "claude",
                "deepmind",
                "openai",
                "video generation",
            ],
        );
        let mut signals = Vec::new();
        if launch > 0 {
            signals.push(format!("{launch} launch/release signals"));
        }
        if tooling > 0 {
            signals.push(format!("{tooling} agent-tooling signals"));
        }
        if model > 0 {
            signals.push(format!("{model} model-capability signals"));
        }
        if signals.is_empty() {
            format!(
                "The story is based on {} saved items, but it needs editorial review before a stronger claim.",
                cards.len()
            )
        } else {
            format!(
                "The strongest evidence is {} across {} saved items.",
                signals.join(", "),
                cards.len()
            )
        }
    }

    pub(crate) fn digest_signal_count(cards: &[SourceCard], terms: &[&str]) -> usize {
        cards
            .iter()
            .filter(|card| {
                let haystack = format!("{} {}", card.title, Self::digest_card_evidence_text(card))
                    .to_ascii_lowercase();
                terms.iter().any(|term| haystack.contains(term))
            })
            .count()
    }

    pub(crate) fn digest_reception_context(cards: &[SourceCard]) -> String {
        let mut labels = Vec::new();
        if Self::digest_signal_count(cards, &["reddit", "comment", "reaction", "reception"]) > 0 {
            labels.push("community reaction");
        }
        if Self::digest_signal_count(cards, &["benchmark", "bench", "eval", "score"]) > 0 {
            labels.push("benchmark scrutiny");
        }
        if Self::digest_signal_count(cards, &["security", "red team", "safety", "policy"]) > 0 {
            labels.push("safety and policy framing");
        }
        if Self::digest_signal_count(cards, &["github", "release", "repo", "sdk", "mcp"]) > 0 {
            labels.push("developer adoption signals");
        }
        if labels.is_empty() {
            "The saved evidence is still mostly first-order material. Treat this as an early signal until follow-on sources show how developers, researchers, or customers react.".to_string()
        } else {
            format!(
                "The saved evidence includes {}. That makes the story more useful than a single link, but the claim strength should still rise only when independent sources repeat or challenge it.",
                labels.join(", ")
            )
        }
    }

    pub(crate) fn digest_why_it_matters(topic: &str, cards: &[SourceCard]) -> String {
        let lower_topic = topic.to_ascii_lowercase();
        let mcp_or_tooling = lower_topic.contains("mcp")
            || Self::digest_signal_count(cards, &["mcp", "tool", "xcode", "agent"]) > 0;
        let launch_or_model =
            Self::digest_signal_count(cards, &["launch", "released", "model"]) > 0;
        match (mcp_or_tooling, launch_or_model) {
            (true, true) => "Product launches are converging with agent and tool interfaces. The useful output is an explanation of the pattern and its practical consequences, not a notification that merely lists tweets.".to_string(),
            (true, false) => "The saved sources point at agent or tooling behavior that may affect workflows, integrations, and future monitoring priorities.".to_string(),
            (false, true) => "Several saved sources describe launches or capability changes that may deserve follow-up once corroborated outside X.".to_string(),
            (false, false) => "This is useful only if the evidence survives editorial review; the current signal is not a settled conclusion.".to_string(),
        }
    }

    pub(crate) fn get_or_create_digest_delivery(
        &self,
        candidate_id: &str,
        channel: &str,
        subject: &str,
        target: &str,
        idempotency_key: &str,
    ) -> Result<DigestDelivery> {
        validate_id(candidate_id)?;
        validate_key(channel)?;
        validate_query(subject)?;
        validate_query(target)?;
        validate_query(idempotency_key)?;
        self.get_digest_candidate(candidate_id)?
            .with_context(|| format!("digest candidate not found: {candidate_id}"))?;

        // Guard the find-then-INSERT against a concurrent connection racing the
        // same (candidate_id, channel, subject, target, idempotency_key). A plain
        // read-then-write can lose the race and hit the UNIQUE constraint; running
        // the re-check and INSERT inside BEGIN IMMEDIATE serializes the write path.
        let delivery_id = (|| -> Result<String> {
            self.conn.execute("BEGIN IMMEDIATE", [])?;
            if let Some(existing) =
                self.find_digest_delivery(candidate_id, channel, subject, target, idempotency_key)?
            {
                self.conn.execute("COMMIT", [])?;
                return Ok(existing.id);
            }
            let id = Uuid::new_v4().to_string();
            let timestamp = now();
            self.conn.execute(
                r#"
                INSERT INTO digest_deliveries
                  (id, candidate_id, channel, subject, target, idempotency_key, status, created_at, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending', ?7, ?7)
                "#,
                params![id, candidate_id, channel, subject, target, idempotency_key, timestamp],
            )?;
            self.conn.execute("COMMIT", [])?;
            Ok(id)
        })();
        let delivery_id = match delivery_id {
            Ok(id) => id,
            Err(error) => {
                let _ = self.conn.execute("ROLLBACK", []);
                return Err(error);
            }
        };
        self.get_digest_delivery(&delivery_id)?
            .with_context(|| format!("inserted digest delivery not found: {delivery_id}"))
    }

    pub(crate) fn find_digest_delivery(
        &self,
        candidate_id: &str,
        channel: &str,
        subject: &str,
        target: &str,
        idempotency_key: &str,
    ) -> Result<Option<DigestDelivery>> {
        self.conn
            .query_row(
                r#"
                SELECT id, candidate_id, channel, subject, target, idempotency_key, status,
                       policy_decision_id, channel_message_id, channel_delivery_attempt_id,
                       error, retry_at, created_at, updated_at
                FROM digest_deliveries
                WHERE candidate_id = ?1
                  AND channel = ?2
                  AND subject = ?3
                  AND target = ?4
                  AND idempotency_key = ?5
                LIMIT 1
                "#,
                params![candidate_id, channel, subject, target, idempotency_key],
                digest_delivery_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn get_digest_delivery(&self, id: &str) -> Result<Option<DigestDelivery>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, candidate_id, channel, subject, target, idempotency_key, status,
                       policy_decision_id, channel_message_id, channel_delivery_attempt_id,
                       error, retry_at, created_at, updated_at
                FROM digest_deliveries
                WHERE id = ?1
                "#,
                params![id],
                digest_delivery_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_digest_deliveries(
        &self,
        candidate_id: Option<&str>,
    ) -> Result<Vec<DigestDelivery>> {
        if let Some(candidate_id) = candidate_id {
            validate_id(candidate_id)?;
            let mut stmt = self.conn.prepare(
                r#"
                SELECT id, candidate_id, channel, subject, target, idempotency_key, status,
                       policy_decision_id, channel_message_id, channel_delivery_attempt_id,
                       error, retry_at, created_at, updated_at
                FROM digest_deliveries
                WHERE candidate_id = ?1
                ORDER BY updated_at DESC
                "#,
            )?;
            return rows(stmt.query_map(params![candidate_id], digest_delivery_from_row)?);
        }
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, candidate_id, channel, subject, target, idempotency_key, status,
                   policy_decision_id, channel_message_id, channel_delivery_attempt_id,
                   error, retry_at, created_at, updated_at
            FROM digest_deliveries
            ORDER BY updated_at DESC
            "#,
        )?;
        rows(stmt.query_map([], digest_delivery_from_row)?)
    }

    pub fn create_digest_alert_schedule(
        &self,
        input: DigestAlertScheduleInput,
    ) -> Result<DigestAlertSchedule> {
        validate_query(&input.name)?;
        let status = input
            .status
            .as_deref()
            .unwrap_or("active")
            .to_ascii_lowercase();
        match status.as_str() {
            "active" | "paused" => {}
            other => bail!("digest alert schedule status must be active or paused, got {other}"),
        }
        let channel = normalize_radar_delivery_channel(&input.channel)?;
        let recipient_ref = normalize_radar_delivery_recipient(&channel, &input.recipient_ref)?;
        let policy = DigestAlertSchedulePolicy {
            channel,
            recipient_ref,
            min_score: input.min_score,
            max_candidates: input.max_candidates,
            interval_hours: input.interval_hours,
            quiet_hours: input
                .quiet_hours
                .as_ref()
                .map(parse_scheduled_radar_quiet_hours)
                .transpose()?,
        };
        validate_digest_alert_schedule_policy(&policy)?;
        let quiet_hours_json = input
            .quiet_hours
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let id = Uuid::new_v4().to_string();
        let timestamp = now();
        self.conn.execute(
            r#"
            INSERT INTO digest_alert_schedules
              (id, name, status, channel, recipient_ref, min_score, max_candidates,
               interval_hours, quiet_hours_json, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)
            "#,
            params![
                id,
                input.name,
                status,
                policy.channel,
                policy.recipient_ref,
                policy.min_score,
                policy.max_candidates,
                policy.interval_hours,
                quiet_hours_json,
                timestamp
            ],
        )?;
        self.get_digest_alert_schedule(&id)?
            .with_context(|| format!("inserted digest alert schedule not found: {id}"))
    }

    pub fn list_digest_alert_schedules(&self) -> Result<Vec<DigestAlertSchedule>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, name, status, channel, recipient_ref, min_score, max_candidates,
                   interval_hours, quiet_hours_json, created_at, updated_at
            FROM digest_alert_schedules
            ORDER BY updated_at DESC
            "#,
        )?;
        rows(stmt.query_map([], digest_alert_schedule_from_row)?)
    }

    pub fn get_digest_alert_schedule(&self, id: &str) -> Result<Option<DigestAlertSchedule>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, name, status, channel, recipient_ref, min_score, max_candidates,
                       interval_hours, quiet_hours_json, created_at, updated_at
                FROM digest_alert_schedules
                WHERE id = ?1
                "#,
                params![id],
                digest_alert_schedule_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_digest_alert_ticks(
        &self,
        schedule_id: Option<&str>,
    ) -> Result<Vec<DigestAlertTick>> {
        if let Some(schedule_id) = schedule_id {
            validate_id(schedule_id)?;
            let mut stmt = self.conn.prepare(
                r#"
                SELECT id, schedule_id, tick_key, due_at, status, job_id,
                       candidate_ids_json, delivery_ids_json, error, created_at, updated_at
                FROM digest_alert_ticks
                WHERE schedule_id = ?1
                ORDER BY due_at DESC, updated_at DESC
                "#,
            )?;
            return rows(stmt.query_map(params![schedule_id], digest_alert_tick_from_row)?);
        }
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, schedule_id, tick_key, due_at, status, job_id,
                   candidate_ids_json, delivery_ids_json, error, created_at, updated_at
            FROM digest_alert_ticks
            ORDER BY due_at DESC, updated_at DESC
            "#,
        )?;
        rows(stmt.query_map([], digest_alert_tick_from_row)?)
    }

    pub fn enqueue_due_digest_alert_schedule_jobs(
        &self,
        max_schedules: usize,
    ) -> Result<DigestAlertScheduleEnqueueReport> {
        let mut report = DigestAlertScheduleEnqueueReport {
            inspected: 0,
            enqueued: 0,
            skipped: 0,
            jobs: Vec::new(),
            errors: Vec::new(),
        };
        for schedule in self
            .list_digest_alert_schedules()?
            .into_iter()
            .take(max_schedules.clamp(1, 100))
        {
            report.inspected += 1;
            if schedule.status != "active" {
                report.skipped += 1;
                continue;
            }
            let policy = match digest_alert_schedule_policy(&schedule) {
                Ok(policy) => policy,
                Err(error) => {
                    report.skipped += 1;
                    report.errors.push(format!("{}: {error}", schedule.name));
                    continue;
                }
            };
            if self.digest_alert_schedule_has_active_job(&schedule.id)? {
                report.skipped += 1;
                continue;
            }
            if let Some(latest_due_at) = self.latest_digest_alert_due_at(&schedule.id)?
                && !radar_schedule_interval_elapsed(&latest_due_at, policy.interval_hours)
            {
                report.skipped += 1;
                continue;
            }
            let due_at = radar_schedule_due_slot(policy.interval_hours);
            let tick_key = digest_alert_schedule_tick_key(&schedule.id, &due_at, &policy);
            if self.get_digest_alert_tick_by_key(&tick_key)?.is_some() {
                report.skipped += 1;
                continue;
            }
            match self.create_digest_alert_tick(&schedule.id, &tick_key, &due_at) {
                Ok(tick) => match self
                    .enqueue_wiki_job("digest_scheduled_alert", json!({ "tick_id": tick.id }))
                {
                    Ok(job) => {
                        self.attach_digest_alert_job(&tick.id, &job.id)?;
                        report.enqueued += 1;
                        report.jobs.push(job.id);
                    }
                    Err(error) => {
                        self.update_digest_alert_tick(
                            &tick.id,
                            "blocked",
                            &[],
                            &[],
                            Some(&error.to_string()),
                        )?;
                        report.skipped += 1;
                        report.errors.push(format!("{}: {error}", schedule.name));
                    }
                },
                Err(error) => {
                    report.skipped += 1;
                    report.errors.push(format!("{}: {error}", schedule.name));
                }
            }
        }
        Ok(report)
    }

    pub(crate) fn digest_alert_schedule_has_active_job(&self, schedule_id: &str) -> Result<bool> {
        validate_id(schedule_id)?;
        self.conn
            .query_row(
                r#"
                SELECT COUNT(*)
                FROM digest_alert_ticks tick
                JOIN wiki_jobs job ON job.id = tick.job_id
                WHERE tick.schedule_id = ?1
                  AND job.kind = 'digest_scheduled_alert'
                  AND job.status IN ('pending', 'running', 'deferred')
                "#,
                params![schedule_id],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count > 0)
            .map_err(Into::into)
    }

    pub(crate) fn latest_digest_alert_due_at(&self, schedule_id: &str) -> Result<Option<String>> {
        validate_id(schedule_id)?;
        self.conn
            .query_row(
                r#"
                SELECT due_at
                FROM digest_alert_ticks
                WHERE schedule_id = ?1
                  AND status IN ('sent', 'partial', 'empty', 'blocked', 'failed', 'deferred')
                ORDER BY due_at DESC
                LIMIT 1
                "#,
                params![schedule_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(Into::into)
    }

    pub(crate) fn create_digest_alert_tick(
        &self,
        schedule_id: &str,
        tick_key: &str,
        due_at: &str,
    ) -> Result<DigestAlertTick> {
        validate_id(schedule_id)?;
        validate_query(tick_key)?;
        validate_timestamp(due_at)?;
        let id = Uuid::new_v4().to_string();
        let timestamp = now();
        self.conn.execute(
            r#"
            INSERT INTO digest_alert_ticks
              (id, schedule_id, tick_key, due_at, status, candidate_ids_json,
               delivery_ids_json, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, 'pending', '[]', '[]', ?5, ?5)
            "#,
            params![id, schedule_id, tick_key, due_at, timestamp],
        )?;
        self.get_digest_alert_tick(&id)?
            .with_context(|| format!("inserted digest alert tick not found: {id}"))
    }

    pub(crate) fn attach_digest_alert_job(&self, tick_id: &str, job_id: &str) -> Result<()> {
        validate_id(tick_id)?;
        validate_id(job_id)?;
        self.conn.execute(
            "UPDATE digest_alert_ticks SET job_id = ?2, updated_at = ?3 WHERE id = ?1",
            params![tick_id, job_id, now()],
        )?;
        Ok(())
    }

    pub(crate) fn update_digest_alert_tick(
        &self,
        tick_id: &str,
        status: &str,
        candidate_ids: &[String],
        delivery_ids: &[String],
        error: Option<&str>,
    ) -> Result<DigestAlertTick> {
        validate_id(tick_id)?;
        validate_key(status)?;
        for id in candidate_ids {
            validate_id(id)?;
        }
        for id in delivery_ids {
            validate_id(id)?;
        }
        if let Some(error) = error {
            validate_notes(error)?;
        }
        let candidate_ids_json = serde_json::to_string(candidate_ids)?;
        let delivery_ids_json = serde_json::to_string(delivery_ids)?;
        self.conn.execute(
            r#"
            UPDATE digest_alert_ticks
            SET status = ?2,
                candidate_ids_json = ?3,
                delivery_ids_json = ?4,
                error = ?5,
                updated_at = ?6
            WHERE id = ?1
            "#,
            params![
                tick_id,
                status,
                candidate_ids_json,
                delivery_ids_json,
                error,
                now()
            ],
        )?;
        self.get_digest_alert_tick(tick_id)?
            .with_context(|| format!("updated digest alert tick not found: {tick_id}"))
    }

    pub(crate) fn get_digest_alert_tick(&self, id: &str) -> Result<Option<DigestAlertTick>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, schedule_id, tick_key, due_at, status, job_id,
                       candidate_ids_json, delivery_ids_json, error, created_at, updated_at
                FROM digest_alert_ticks
                WHERE id = ?1
                "#,
                params![id],
                digest_alert_tick_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub(crate) fn get_digest_alert_tick_by_key(
        &self,
        tick_key: &str,
    ) -> Result<Option<DigestAlertTick>> {
        validate_query(tick_key)?;
        self.conn
            .query_row(
                r#"
                SELECT id, schedule_id, tick_key, due_at, status, job_id,
                       candidate_ids_json, delivery_ids_json, error, created_at, updated_at
                FROM digest_alert_ticks
                WHERE tick_key = ?1
                "#,
                params![tick_key],
                digest_alert_tick_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn upsert_issue_schedule(&self, input: IssueScheduleInput) -> Result<IssueSchedule> {
        validate_query(&input.name)?;
        validate_issue_schedule_kind(&input.kind)?;
        let status = input
            .status
            .as_deref()
            .unwrap_or("active")
            .trim()
            .to_ascii_lowercase();
        validate_issue_schedule_status(&status)?;
        let channel = normalize_radar_delivery_channel(&input.channel)?;
        let recipient_ref = normalize_radar_delivery_recipient(&channel, &input.recipient_ref)?;
        let time_zone = normalize_issue_schedule_time_zone(&input.time_zone)?;
        let hour = input.hour.clamp(0, 23);
        let minute = input.minute.clamp(0, 59);
        let catch_up_hours = input.catch_up_hours.clamp(1, 24 * 14);
        let metadata = sanitize_work_json(input.metadata)?;
        validate_issue_schedule_metadata(&metadata)?;
        let metadata_json = serde_json::to_string(&metadata)?;
        let id = issue_schedule_id(&input.kind, &input.name);
        let timestamp = now();
        self.conn.execute(
            r#"
            INSERT INTO issue_schedules
              (id, name, status, kind, channel, recipient_ref, time_zone, hour, minute,
               catch_up_hours, metadata_json, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?12)
            ON CONFLICT(kind, name) DO UPDATE SET
              status = excluded.status,
              channel = excluded.channel,
              recipient_ref = excluded.recipient_ref,
              time_zone = excluded.time_zone,
              hour = excluded.hour,
              minute = excluded.minute,
              catch_up_hours = excluded.catch_up_hours,
              metadata_json = excluded.metadata_json,
              updated_at = excluded.updated_at
            "#,
            params![
                id,
                input.name,
                status,
                input.kind,
                channel,
                recipient_ref,
                time_zone,
                hour,
                minute,
                catch_up_hours,
                metadata_json,
                timestamp
            ],
        )?;
        self.get_issue_schedule(&id)?
            .with_context(|| format!("issue schedule not found after upsert: {id}"))
    }

    pub fn list_issue_schedules(&self) -> Result<Vec<IssueSchedule>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, name, status, kind, channel, recipient_ref, time_zone, hour, minute,
                   catch_up_hours, metadata_json, created_at, updated_at
            FROM issue_schedules
            ORDER BY updated_at DESC
            "#,
        )?;
        rows(stmt.query_map([], issue_schedule_from_row)?)
    }

    pub fn get_issue_schedule(&self, id: &str) -> Result<Option<IssueSchedule>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, name, status, kind, channel, recipient_ref, time_zone, hour, minute,
                       catch_up_hours, metadata_json, created_at, updated_at
                FROM issue_schedules
                WHERE id = ?1
                "#,
                params![id],
                issue_schedule_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_issue_schedule_ticks(
        &self,
        schedule_id: Option<&str>,
    ) -> Result<Vec<IssueScheduleTick>> {
        if let Some(schedule_id) = schedule_id {
            validate_id(schedule_id)?;
            let mut stmt = self.conn.prepare(
                r#"
                SELECT id, schedule_id, tick_key, due_at, status, job_id, candidate_id,
                       delivery_id, error, created_at, updated_at
                FROM issue_schedule_ticks
                WHERE schedule_id = ?1
                ORDER BY due_at DESC, updated_at DESC
                "#,
            )?;
            return rows(stmt.query_map(params![schedule_id], issue_schedule_tick_from_row)?);
        }
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, schedule_id, tick_key, due_at, status, job_id, candidate_id,
                   delivery_id, error, created_at, updated_at
            FROM issue_schedule_ticks
            ORDER BY due_at DESC, updated_at DESC
            "#,
        )?;
        rows(stmt.query_map([], issue_schedule_tick_from_row)?)
    }

    pub fn enqueue_due_issue_schedule_jobs(
        &self,
        max_schedules: usize,
    ) -> Result<IssueScheduleEnqueueReport> {
        let mut report = IssueScheduleEnqueueReport {
            inspected: 0,
            enqueued: 0,
            skipped: 0,
            jobs: Vec::new(),
            errors: Vec::new(),
        };
        let max_schedules = max_schedules.clamp(1, 100);
        let max_jobs = max_schedules;
        for schedule in self.list_issue_schedules()?.into_iter().take(max_schedules) {
            report.inspected += 1;
            if schedule.status != "active" {
                report.skipped += 1;
                continue;
            }
            if self.issue_schedule_has_active_job(&schedule.id)? {
                report.skipped += 1;
                continue;
            }
            let due_slots = match self.issue_schedule_due_slots(&schedule, Utc::now()) {
                Ok(slots) => slots,
                Err(error) => {
                    report.skipped += 1;
                    report.errors.push(format!("{}: {error}", schedule.name));
                    continue;
                }
            };
            if due_slots.is_empty() {
                report.skipped += 1;
                continue;
            }
            for due_at in due_slots {
                if report.enqueued >= max_jobs {
                    break;
                }
                let tick_key = issue_schedule_tick_key(&schedule.id, &due_at, &schedule);
                let tick = match self.get_issue_schedule_tick_by_key(&tick_key)? {
                    Some(existing) if existing.status == "pending" => existing,
                    Some(_) => {
                        report.skipped += 1;
                        continue;
                    }
                    None => match self.create_issue_schedule_tick(&schedule.id, &tick_key, &due_at)
                    {
                        Ok(tick) => tick,
                        Err(error) => {
                            report.skipped += 1;
                            report.errors.push(format!("{}: {error}", schedule.name));
                            continue;
                        }
                    },
                };
                let job_kind = match issue_schedule_job_kind(&schedule) {
                    Ok(kind) => kind,
                    Err(error) => {
                        self.update_issue_schedule_tick(
                            &tick.id,
                            "blocked",
                            None,
                            None,
                            Some(&error.to_string()),
                        )?;
                        report.skipped += 1;
                        report.errors.push(format!("{}: {error}", schedule.name));
                        continue;
                    }
                };
                if self.issue_schedule_tick_has_active_job(&tick)? {
                    report.skipped += 1;
                    continue;
                }
                match self.enqueue_wiki_job(job_kind, json!({ "tick_id": tick.id })) {
                    Ok(job) => {
                        self.attach_issue_schedule_job(&tick.id, &job.id)?;
                        report.enqueued += 1;
                        report.jobs.push(job.id);
                    }
                    Err(error) => {
                        self.update_issue_schedule_tick(
                            &tick.id,
                            "blocked",
                            None,
                            None,
                            Some(&error.to_string()),
                        )?;
                        report.skipped += 1;
                        report.errors.push(format!("{}: {error}", schedule.name));
                    }
                }
            }
        }
        Ok(report)
    }

    pub(crate) fn issue_schedule_has_active_job(&self, schedule_id: &str) -> Result<bool> {
        validate_id(schedule_id)?;
        self.conn
            .query_row(
                r#"
                SELECT COUNT(*)
                FROM issue_schedule_ticks tick
                JOIN wiki_jobs job ON job.id = tick.job_id
                WHERE tick.schedule_id = ?1
                  AND job.status IN ('pending', 'running', 'deferred')
                "#,
                params![schedule_id],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count > 0)
            .map_err(Into::into)
    }

    pub(crate) fn issue_schedule_tick_has_active_job(
        &self,
        tick: &IssueScheduleTick,
    ) -> Result<bool> {
        let Some(job_id) = tick.job_id.as_deref() else {
            return Ok(false);
        };
        Ok(self
            .get_wiki_job(job_id)?
            .is_some_and(|job| matches!(job.status.as_str(), "pending" | "running" | "deferred")))
    }

    pub(crate) fn issue_schedule_due_slots(
        &self,
        schedule: &IssueSchedule,
        now_utc: DateTime<Utc>,
    ) -> Result<Vec<String>> {
        let latest_due_at = self.latest_issue_schedule_scheduled_due_at(&schedule.id)?;
        issue_schedule_due_slots_with_metadata(
            latest_due_at.as_deref(),
            &schedule.created_at,
            schedule.hour,
            schedule.minute,
            schedule.catch_up_hours,
            &schedule.time_zone,
            now_utc,
            schedule
                .metadata
                .get("max_catch_up_ticks")
                .and_then(Value::as_u64)
                .unwrap_or(3) as usize,
            &schedule.metadata,
        )
    }

    pub(crate) fn latest_issue_schedule_scheduled_due_at(
        &self,
        schedule_id: &str,
    ) -> Result<Option<String>> {
        validate_id(schedule_id)?;
        self.conn
            .query_row(
                r#"
                SELECT due_at
                FROM issue_schedule_ticks
                WHERE schedule_id = ?1
                  AND tick_key LIKE 'issue-%'
                  AND status IN ('sent', 'partial', 'empty', 'blocked', 'failed', 'deferred')
                ORDER BY due_at DESC
                LIMIT 1
                "#,
                params![schedule_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(Into::into)
    }

    pub(crate) fn create_issue_schedule_tick(
        &self,
        schedule_id: &str,
        tick_key: &str,
        due_at: &str,
    ) -> Result<IssueScheduleTick> {
        validate_id(schedule_id)?;
        validate_query(tick_key)?;
        validate_timestamp(due_at)?;
        let id = Uuid::new_v4().to_string();
        let timestamp = now();
        self.conn.execute(
            r#"
            INSERT INTO issue_schedule_ticks
              (id, schedule_id, tick_key, due_at, status, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, 'pending', ?5, ?5)
            "#,
            params![id, schedule_id, tick_key, due_at, timestamp],
        )?;
        self.get_issue_schedule_tick(&id)?
            .with_context(|| format!("inserted issue schedule tick not found: {id}"))
    }

    pub(crate) fn attach_issue_schedule_job(&self, tick_id: &str, job_id: &str) -> Result<()> {
        validate_id(tick_id)?;
        validate_id(job_id)?;
        self.conn.execute(
            "UPDATE issue_schedule_ticks SET job_id = ?2, updated_at = ?3 WHERE id = ?1",
            params![tick_id, job_id, now()],
        )?;
        Ok(())
    }

    pub(crate) fn update_issue_schedule_tick(
        &self,
        tick_id: &str,
        status: &str,
        candidate_id: Option<&str>,
        delivery_id: Option<&str>,
        error: Option<&str>,
    ) -> Result<IssueScheduleTick> {
        validate_id(tick_id)?;
        validate_issue_schedule_tick_status(status)?;
        if let Some(candidate_id) = candidate_id {
            validate_id(candidate_id)?;
        }
        if let Some(delivery_id) = delivery_id {
            validate_id(delivery_id)?;
        }
        let error = error.map(sanitize_radar_delivery_error).transpose()?;
        self.conn.execute(
            r#"
            UPDATE issue_schedule_ticks
            SET status = ?2,
                candidate_id = COALESCE(?3, candidate_id),
                delivery_id = COALESCE(?4, delivery_id),
                error = ?5,
                updated_at = ?6
            WHERE id = ?1
            "#,
            params![tick_id, status, candidate_id, delivery_id, error, now()],
        )?;
        self.get_issue_schedule_tick(tick_id)?
            .with_context(|| format!("updated issue schedule tick not found: {tick_id}"))
    }

    pub(crate) fn get_issue_schedule_tick(&self, id: &str) -> Result<Option<IssueScheduleTick>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, schedule_id, tick_key, due_at, status, job_id, candidate_id,
                       delivery_id, error, created_at, updated_at
                FROM issue_schedule_ticks
                WHERE id = ?1
                "#,
                params![id],
                issue_schedule_tick_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub(crate) fn get_issue_schedule_tick_by_key(
        &self,
        tick_key: &str,
    ) -> Result<Option<IssueScheduleTick>> {
        validate_query(tick_key)?;
        self.conn
            .query_row(
                r#"
                SELECT id, schedule_id, tick_key, due_at, status, job_id, candidate_id,
                       delivery_id, error, created_at, updated_at
                FROM issue_schedule_ticks
                WHERE tick_key = ?1
                "#,
                params![tick_key],
                issue_schedule_tick_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn reconcile_digest_delivery_attempts(
        &self,
        max_attempts_per_message: i64,
    ) -> Result<DigestDeliveryReconcileReport> {
        let max_attempts_per_message = max_attempts_per_message.clamp(1, 20);
        let due = {
            let mut stmt = self.conn.prepare(
                r#"
                SELECT dd.id,
                       latest.id,
                       latest.message_id,
                       latest.ok,
                       latest.attempt,
                       latest.provider_status,
                       latest.error,
                       latest.retry_at,
                       candidate.status,
                       candidate.review_status
                FROM digest_deliveries dd
                JOIN digest_candidates candidate
                  ON candidate.id = dd.candidate_id
                JOIN channel_delivery_attempts linked
                  ON linked.id = dd.channel_delivery_attempt_id
                JOIN channel_delivery_attempts latest
                  ON latest.message_id = linked.message_id
                WHERE dd.status IN ('pending', 'failed')
                  AND latest.attempt = (
                    SELECT MAX(d2.attempt)
                    FROM channel_delivery_attempts d2
                    WHERE d2.message_id = linked.message_id
                  )
                  AND (
                    dd.channel_delivery_attempt_id != latest.id
                    OR latest.ok = 1
                    OR latest.attempt >= ?1
                  )
                ORDER BY dd.updated_at ASC, dd.id ASC
                "#,
            )?;
            rows(stmt.query_map(params![max_attempts_per_message], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)? != 0,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, String>(9)?,
                ))
            })?)?
        };

        let mut report = DigestDeliveryReconcileReport {
            inspected: due.len(),
            sent: 0,
            failed: 0,
            dead_lettered: 0,
            updated: Vec::new(),
        };
        for (
            delivery_id,
            latest_attempt_id,
            message_id,
            ok,
            attempt,
            provider_status,
            error,
            retry_at,
            candidate_status,
            candidate_review_status,
        ) in due
        {
            let (status, error_text, retry_at) = if candidate_status != "approved"
                || candidate_review_status != "approved"
            {
                report.failed += 1;
                (
                    "blocked",
                    Some(sanitize_radar_delivery_error(&format!(
                        "digest candidate no longer approved; status={candidate_status}, review_status={candidate_review_status}"
                    ))?),
                    None,
                )
            } else if ok {
                report.sent += 1;
                ("sent", None, None)
            } else if attempt >= max_attempts_per_message {
                report.dead_lettered += 1;
                self.update_channel_message_status(&message_id, "dead_lettered")?;
                (
                    "dead_lettered",
                    Some(sanitize_radar_delivery_error(&format!(
                        "delivery retry exhausted after {attempt} attempt(s): {}",
                        error.unwrap_or_else(|| format!("provider status {provider_status}"))
                    ))?),
                    None,
                )
            } else {
                report.failed += 1;
                (
                    "failed",
                    Some(sanitize_radar_delivery_error(&error.unwrap_or_else(
                        || format!("provider status {provider_status}"),
                    ))?),
                    retry_at,
                )
            };
            let delivery = self.update_digest_delivery(
                &delivery_id,
                status,
                None,
                Some(&message_id),
                Some(&latest_attempt_id),
                error_text.as_deref(),
                retry_at.as_deref(),
            )?;
            report.updated.push(delivery);
        }
        Ok(report)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn update_digest_delivery(
        &self,
        id: &str,
        status: &str,
        policy_decision_id: Option<&str>,
        channel_message_id: Option<&str>,
        channel_delivery_attempt_id: Option<&str>,
        error: Option<&str>,
        retry_at: Option<&str>,
    ) -> Result<DigestDelivery> {
        validate_id(id)?;
        validate_key(status)?;
        if let Some(policy_decision_id) = policy_decision_id {
            validate_id(policy_decision_id)?;
        }
        if let Some(channel_message_id) = channel_message_id {
            validate_id(channel_message_id)?;
        }
        if let Some(channel_delivery_attempt_id) = channel_delivery_attempt_id {
            validate_id(channel_delivery_attempt_id)?;
        }
        if let Some(error) = error {
            validate_notes(error)?;
        }
        if let Some(retry_at) = retry_at {
            DateTime::parse_from_rfc3339(retry_at)
                .with_context(|| format!("parsing retry_at timestamp {retry_at}"))?;
        }
        let timestamp = now();
        self.conn.execute(
            r#"
            UPDATE digest_deliveries
            SET status = ?2,
                policy_decision_id = COALESCE(?3, policy_decision_id),
                channel_message_id = ?4,
                channel_delivery_attempt_id = ?5,
                error = ?6,
                retry_at = ?7,
                updated_at = ?8
            WHERE id = ?1
            "#,
            params![
                id,
                status,
                policy_decision_id,
                channel_message_id,
                channel_delivery_attempt_id,
                error,
                retry_at,
                timestamp
            ],
        )?;
        self.get_digest_delivery(id)?
            .with_context(|| format!("updated digest delivery not found: {id}"))
    }
}
