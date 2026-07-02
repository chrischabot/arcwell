use super::*;

impl Store {
    pub fn add_cost(
        &self,
        package: &str,
        job_id: &str,
        provider: &str,
        model: &str,
        estimated_usd: f64,
        actual_usd: f64,
    ) -> Result<String> {
        self.add_cost_for_source(
            package,
            job_id,
            provider,
            model,
            None,
            estimated_usd,
            actual_usd,
        )
    }

    // allow: refactoring this N-arg signature is out of scope for the lint-cleanup pass.
    #[allow(clippy::too_many_arguments)]
    pub fn add_cost_for_source(
        &self,
        package: &str,
        job_id: &str,
        provider: &str,
        model: &str,
        source: Option<&str>,
        estimated_usd: f64,
        actual_usd: f64,
    ) -> Result<String> {
        validate_key(package)?;
        validate_key(job_id)?;
        validate_key(provider)?;
        validate_key(model)?;
        if let Some(source) = source {
            validate_key(source)?;
        }
        validate_non_negative_cost(estimated_usd, "estimated_usd")?;
        validate_non_negative_cost(actual_usd, "actual_usd")?;
        self.insert_cost_entry(
            package,
            job_id,
            provider,
            model,
            source,
            estimated_usd,
            actual_usd,
        )
    }

    // allow: refactoring this N-arg signature is out of scope for the lint-cleanup pass.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn insert_cost_entry(
        &self,
        package: &str,
        job_id: &str,
        provider: &str,
        model: &str,
        source: Option<&str>,
        estimated_usd: f64,
        actual_usd: f64,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = now();
        self.conn.execute(
            r#"
            INSERT INTO cost_entries
              (id, package, job_id, provider, model, source, estimated_usd, actual_usd, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                id,
                package,
                job_id,
                provider,
                model,
                source,
                estimated_usd,
                actual_usd,
                now
            ],
        )?;
        Ok(id)
    }

    pub fn cost_summary(&self) -> Result<(f64, f64, i64)> {
        Ok(self.conn.query_row(
            "SELECT COALESCE(sum(estimated_usd), 0), COALESCE(sum(actual_usd), 0), count(*) FROM cost_entries",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?)
    }

    pub fn set_cost_policy(
        &self,
        scope: &str,
        key: &str,
        limit_usd: Option<f64>,
        kill_switch: bool,
        override_until: Option<&str>,
    ) -> Result<CostPolicy> {
        validate_cost_scope(scope)?;
        validate_key(key)?;
        if let Some(limit) = limit_usd {
            validate_non_negative_cost(limit, "limit_usd")?;
        }
        if let Some(override_until) = override_until {
            DateTime::parse_from_rfc3339(override_until)
                .with_context(|| format!("parsing override_until timestamp {override_until}"))?;
        }
        let updated_at = now();
        self.conn.execute(
            r#"
            INSERT INTO cost_policies
              (scope, key, limit_usd, kill_switch, override_until, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(scope, key) DO UPDATE SET
              limit_usd = excluded.limit_usd,
              kill_switch = excluded.kill_switch,
              override_until = excluded.override_until,
              updated_at = excluded.updated_at
            "#,
            params![
                scope,
                key,
                limit_usd,
                bool_to_i64(kill_switch),
                override_until,
                updated_at
            ],
        )?;
        self.get_cost_policy(scope, key)?
            .with_context(|| format!("inserted cost policy not found: {scope}:{key}"))
    }

    pub fn get_cost_policy(&self, scope: &str, key: &str) -> Result<Option<CostPolicy>> {
        validate_cost_scope(scope)?;
        validate_key(key)?;
        self.conn
            .query_row(
                r#"
                SELECT scope, key, limit_usd, kill_switch, override_until, updated_at
                FROM cost_policies
                WHERE scope = ?1 AND key = ?2
                "#,
                params![scope, key],
                cost_policy_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_cost_policies(&self) -> Result<Vec<CostPolicy>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT scope, key, limit_usd, kill_switch, override_until, updated_at
            FROM cost_policies
            ORDER BY scope ASC, key ASC
            "#,
        )?;
        rows(stmt.query_map([], cost_policy_from_row)?)
    }

    pub fn cost_decision(
        &self,
        package: &str,
        provider: &str,
        source: Option<&str>,
        projected_usd: f64,
    ) -> Result<CostDecision> {
        let mut decision = self.evaluate_cost_decision(package, provider, source, projected_usd)?;
        let id = self.record_cost_decision(
            package,
            "cost_check",
            provider,
            "projected",
            source,
            &decision,
        )?;
        decision.decision_id = Some(id);
        Ok(decision)
    }

    pub fn reserve_cost_budget(
        &self,
        package: &str,
        job_id: &str,
        provider: &str,
        model: &str,
        source: Option<&str>,
        projected_usd: f64,
    ) -> Result<CostDecision> {
        validate_key(package)?;
        validate_key(job_id)?;
        validate_key(provider)?;
        validate_key(model)?;
        if let Some(source) = source {
            validate_key(source)?;
        }
        validate_non_negative_cost(projected_usd, "projected_usd")?;
        self.conn.execute("BEGIN IMMEDIATE", [])?;
        let result = (|| -> Result<CostDecision> {
            let mut decision =
                self.evaluate_cost_decision(package, provider, source, projected_usd)?;
            let decision_id =
                self.record_cost_decision(package, job_id, provider, model, source, &decision)?;
            decision.decision_id = Some(decision_id);
            if decision.allowed {
                self.insert_cost_entry(
                    package,
                    job_id,
                    provider,
                    model,
                    source,
                    projected_usd,
                    0.0,
                )?;
            }
            Ok(decision)
        })();
        match result {
            Ok(decision) => {
                self.conn.execute("COMMIT", [])?;
                Ok(decision)
            }
            Err(error) => {
                let _ = self.conn.execute("ROLLBACK", []);
                Err(error)
            }
        }
    }

    // allow: refactoring this N-arg signature is out of scope for the lint-cleanup pass.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn require_cost_budget(
        &self,
        package: &str,
        job_id: &str,
        provider: &str,
        model: &str,
        source: Option<&str>,
        projected_usd: f64,
        label: &str,
    ) -> Result<CostDecision> {
        let decision =
            self.reserve_cost_budget(package, job_id, provider, model, source, projected_usd)?;
        if !decision.allowed {
            bail!("budget blocked {label}: {}", decision.reason);
        }
        Ok(decision)
    }

    pub(crate) fn release_cost_reservation(
        &self,
        package: &str,
        job_id: &str,
        provider: &str,
        model: &str,
        source: Option<&str>,
    ) -> Result<usize> {
        validate_key(package)?;
        validate_key(job_id)?;
        validate_key(provider)?;
        validate_key(model)?;
        if let Some(source) = source {
            validate_key(source)?;
        }
        self.conn
            .execute(
                r#"
                DELETE FROM cost_entries
                WHERE package = ?1
                  AND job_id = ?2
                  AND provider = ?3
                  AND model = ?4
                  AND ((source IS NULL AND ?5 IS NULL) OR source = ?5)
                "#,
                params![package, job_id, provider, model, source],
            )
            .map_err(Into::into)
    }

    pub(crate) fn evaluate_cost_decision(
        &self,
        package: &str,
        provider: &str,
        source: Option<&str>,
        projected_usd: f64,
    ) -> Result<CostDecision> {
        validate_key(package)?;
        validate_key(provider)?;
        if let Some(source) = source {
            validate_key(source)?;
        }
        validate_non_negative_cost(projected_usd, "projected_usd")?;
        let candidates = [
            source.map(|source| ("source", source.to_string())),
            Some(("provider", provider.to_string())),
            Some(("package", package.to_string())),
            Some(("global", "*".to_string())),
        ];
        for (scope, key) in candidates.into_iter().flatten() {
            let Some(policy) = self.get_cost_policy(scope, &key)? else {
                continue;
            };
            if cost_override_active(policy.override_until.as_deref())? {
                continue;
            }
            let spent = self.cost_spent_for_policy(&policy)?;
            if policy.kill_switch {
                return Ok(CostDecision {
                    decision_id: None,
                    allowed: false,
                    reason: format!("cost policy {scope}:{key} kill switch is enabled"),
                    matched_policy: Some(policy),
                    projected_usd,
                    spent_usd: spent,
                    remaining_usd: None,
                });
            }
            if let Some(limit) = policy.limit_usd {
                let remaining = (limit - spent).max(0.0);
                if spent + projected_usd > limit {
                    return Ok(CostDecision {
                        decision_id: None,
                        allowed: false,
                        reason: format!("cost policy {scope}:{key} would exceed limit ${limit:.4}"),
                        matched_policy: Some(policy),
                        projected_usd,
                        spent_usd: spent,
                        remaining_usd: Some(remaining),
                    });
                }
            }
        }
        Ok(CostDecision {
            decision_id: None,
            allowed: true,
            reason: "allowed".to_string(),
            matched_policy: None,
            projected_usd,
            spent_usd: 0.0,
            remaining_usd: None,
        })
    }

    pub(crate) fn record_cost_decision(
        &self,
        package: &str,
        job_id: &str,
        provider: &str,
        model: &str,
        source: Option<&str>,
        decision: &CostDecision,
    ) -> Result<String> {
        validate_key(package)?;
        validate_key(job_id)?;
        validate_key(provider)?;
        validate_key(model)?;
        if let Some(source) = source {
            validate_key(source)?;
        }
        validate_non_negative_cost(decision.projected_usd, "projected_usd")?;
        validate_non_negative_cost(decision.spent_usd, "spent_usd")?;
        if let Some(remaining) = decision.remaining_usd {
            validate_non_negative_cost(remaining, "remaining_usd")?;
        }
        let id = Uuid::new_v4().to_string();
        let created_at = now();
        let (matched_scope, matched_key) = decision
            .matched_policy
            .as_ref()
            .map(|policy| (Some(policy.scope.as_str()), Some(policy.key.as_str())))
            .unwrap_or((None, None));
        self.conn.execute(
            r#"
            INSERT INTO cost_decisions
              (id, allowed, reason, package, job_id, provider, model, source,
               projected_usd, spent_usd, remaining_usd, matched_scope, matched_key, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            "#,
            params![
                id,
                bool_to_i64(decision.allowed),
                decision.reason,
                package,
                job_id,
                provider,
                model,
                source,
                decision.projected_usd,
                decision.spent_usd,
                decision.remaining_usd,
                matched_scope,
                matched_key,
                created_at
            ],
        )?;
        Ok(id)
    }

    pub fn list_cost_decisions(&self, limit: usize) -> Result<Vec<CostDecisionRecord>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, allowed, reason, package, job_id, provider, model, source,
                   projected_usd, spent_usd, remaining_usd, matched_scope, matched_key, created_at
            FROM cost_decisions
            ORDER BY created_at DESC
            LIMIT ?1
            "#,
        )?;
        rows(stmt.query_map(params![limit.clamp(1, 500)], cost_decision_from_row)?)
    }

    pub fn policy_check(&self, request: PolicyRequest) -> Result<PolicyDecisionRecord> {
        validate_policy_request(&request)?;
        let rules = self.load_policy_rules()?;
        let matched = best_policy_rule(&rules, &request)?;
        let (effect, matched_rule_id, reason) = match matched {
            Some(rule) => (
                rule.effect.clone(),
                Some(rule.id.clone()),
                rule.reason.clone(),
            ),
            None => (
                "defer".to_string(),
                None,
                "no matching policy rule; defer to explicit user or higher-level policy"
                    .to_string(),
            ),
        };
        let allowed = effect == "allow";
        let id = Uuid::new_v4().to_string();
        let created_at = now();
        let metadata = policy_decision_metadata(&request);
        self.conn.execute(
            r#"
            INSERT INTO policy_decisions
              (id, action, effect, allowed, reason, matched_rule_id, approval_id,
               package, provider, source, channel, subject, target, metadata_json, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            "#,
            params![
                id,
                request.action,
                effect,
                bool_to_i64(allowed),
                reason,
                matched_rule_id,
                request.package,
                request.provider,
                request.source,
                request.channel,
                request.subject,
                request.target,
                serde_json::to_string(&metadata)?,
                created_at
            ],
        )?;
        let mut decision = self
            .get_policy_decision(&id)?
            .with_context(|| format!("inserted policy decision not found: {id}"))?;
        if decision.effect == "require_approval" {
            let approval_id = Uuid::new_v4().to_string();
            self.conn.execute(
                r#"
                INSERT INTO policy_approvals
                  (id, decision_id, action, status, reason, created_at, resolved_at)
                VALUES (?1, ?2, ?3, 'pending', ?4, ?5, NULL)
                "#,
                params![
                    approval_id,
                    decision.id,
                    decision.action,
                    decision.reason,
                    decision.created_at
                ],
            )?;
            self.conn.execute(
                "UPDATE policy_decisions SET approval_id = ?2 WHERE id = ?1",
                params![decision.id, approval_id],
            )?;
            decision = self
                .get_policy_decision(&id)?
                .with_context(|| format!("policy decision not found after approval link: {id}"))?;
        }
        Ok(decision)
    }

    pub fn policy_explain(&self, request: PolicyRequest) -> Result<PolicyExplanation> {
        validate_policy_request(&request)?;
        let rules = self.load_policy_rules()?;
        let matching_rules = matching_policy_rules(&rules, &request)?;
        let matched_rule = matching_rules.first().cloned();
        let (effect, allowed, reason) = match &matched_rule {
            Some(rule) => (
                rule.effect.clone(),
                rule.effect == "allow",
                rule.reason.clone(),
            ),
            None => (
                "defer".to_string(),
                false,
                "no matching policy rule; defer to explicit user or higher-level policy"
                    .to_string(),
            ),
        };
        Ok(PolicyExplanation {
            request,
            effect,
            allowed,
            reason,
            matched_rule,
            matching_rules,
        })
    }

    pub fn list_policy_rules(&self) -> Result<Vec<PolicyRule>> {
        self.load_policy_rules()
    }

    pub fn ensure_local_ops_ui_policy_rules(&self) -> Result<Vec<String>> {
        let path = self.paths.home.join("arcwell-policy.toml");
        let mut policy = if path.exists() {
            let body =
                fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
            toml::from_str::<PolicyFile>(&body)
                .with_context(|| format!("parsing policy file {}", path.display()))?
        } else {
            PolicyFile {
                rules: default_policy_rules(),
            }
        };
        let seed_rules = local_ops_ui_policy_rules();
        let existing_ids = policy
            .rules
            .iter()
            .map(|rule| rule.id.clone())
            .collect::<BTreeSet<_>>();
        let mut added = Vec::new();
        for rule in seed_rules {
            if existing_ids.contains(&rule.id) {
                continue;
            }
            validate_policy_rule(&rule)?;
            added.push(rule.id.clone());
            policy.rules.push(rule);
        }
        if !added.is_empty() {
            let body = toml::to_string_pretty(&policy)?;
            fs::write(&path, body).with_context(|| format!("writing {}", path.display()))?;
        }
        self.load_policy_rules()?;
        Ok(added)
    }

    pub fn create_policy_allow_override(
        &self,
        mut request: PolicyRequest,
        reason: &str,
        expires_at: &str,
    ) -> Result<PolicyOverrideReport> {
        validate_policy_request(&request)?;
        validate_notes(reason)?;
        let expires_at = DateTime::parse_from_rfc3339(expires_at)
            .with_context(|| format!("parsing policy override expires_at timestamp {expires_at}"))?
            .with_timezone(&Utc)
            .to_rfc3339();
        if DateTime::parse_from_rfc3339(&expires_at)?.with_timezone(&Utc) <= Utc::now() {
            bail!("policy override expires_at must be in the future");
        }
        request.metadata = Value::Null;
        request.untrusted_excerpt = None;
        let path = self.paths.home.join("arcwell-policy.toml");
        let mut policy = if path.exists() {
            let body =
                fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
            toml::from_str::<PolicyFile>(&body)
                .with_context(|| format!("parsing policy file {}", path.display()))?
        } else {
            PolicyFile {
                rules: default_policy_rules(),
            }
        };
        let rule = PolicyRule {
            id: format!("override-{}", Uuid::new_v4()),
            effect: "allow".to_string(),
            action: request.action,
            reason: reason.to_string(),
            package: request.package,
            provider: request.provider,
            source: request.source,
            channel: request.channel,
            subject: request.subject,
            target: request.target,
            priority: 100,
            expires_at: Some(expires_at),
        };
        validate_policy_rule(&rule)?;
        policy.rules.push(rule.clone());
        let body = toml::to_string_pretty(&policy)?;
        fs::write(&path, body).with_context(|| format!("writing {}", path.display()))?;
        Ok(PolicyOverrideReport {
            policy_path: path,
            rule,
        })
    }

    pub fn list_policy_decisions(&self, limit: usize) -> Result<Vec<PolicyDecisionRecord>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, action, effect, allowed, reason, matched_rule_id, approval_id,
                   package, provider, source, channel, subject, target, metadata_json, created_at
            FROM policy_decisions
            ORDER BY created_at DESC
            LIMIT ?1
            "#,
        )?;
        rows(stmt.query_map(params![limit.clamp(1, 500)], policy_decision_from_row)?)
    }

    pub fn list_policy_approvals(&self, status: Option<&str>) -> Result<Vec<PolicyApprovalRecord>> {
        if let Some(status) = status {
            validate_key(status)?;
            let mut stmt = self.conn.prepare(
                r#"
                SELECT id, decision_id, action, status, reason, created_at, resolved_at
                FROM policy_approvals
                WHERE status = ?1
                ORDER BY created_at DESC
                "#,
            )?;
            return rows(stmt.query_map(params![status], policy_approval_from_row)?);
        }
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, decision_id, action, status, reason, created_at, resolved_at
            FROM policy_approvals
            ORDER BY created_at DESC
            "#,
        )?;
        rows(stmt.query_map([], policy_approval_from_row)?)
    }

    pub fn approve_policy_approval(
        &self,
        approval_id: &str,
        reason: Option<&str>,
    ) -> Result<PolicyApprovalRecord> {
        self.resolve_policy_approval(approval_id, "approved", reason)
    }

    pub fn reject_policy_approval(
        &self,
        approval_id: &str,
        reason: Option<&str>,
    ) -> Result<PolicyApprovalRecord> {
        self.resolve_policy_approval(approval_id, "rejected", reason)
    }

    pub(crate) fn resolve_policy_approval(
        &self,
        approval_id: &str,
        status: &str,
        reason: Option<&str>,
    ) -> Result<PolicyApprovalRecord> {
        validate_id(approval_id)?;
        match status {
            "approved" | "rejected" => {}
            other => bail!("unsupported policy approval resolution: {other}"),
        }
        if let Some(reason) = reason {
            validate_notes(reason)?;
        }
        let approval = self
            .get_policy_approval(approval_id)?
            .with_context(|| format!("policy approval not found: {approval_id}"))?;
        if approval.status != "pending" {
            bail!(
                "policy approval {approval_id} is already {} and cannot be resolved again",
                approval.status
            );
        }
        let resolved_at = now();
        let reason = reason.unwrap_or(&approval.reason);
        self.conn.execute(
            r#"
            UPDATE policy_approvals
            SET status = ?2, reason = ?3, resolved_at = ?4
            WHERE id = ?1
            "#,
            params![approval_id, status, reason, resolved_at],
        )?;
        self.get_policy_approval(approval_id)?
            .with_context(|| format!("policy approval not found after update: {approval_id}"))
    }

    pub(crate) fn get_policy_decision(&self, id: &str) -> Result<Option<PolicyDecisionRecord>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, action, effect, allowed, reason, matched_rule_id, approval_id,
                       package, provider, source, channel, subject, target, metadata_json, created_at
                FROM policy_decisions
                WHERE id = ?1
                "#,
                params![id],
                policy_decision_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub(crate) fn get_policy_approval(&self, id: &str) -> Result<Option<PolicyApprovalRecord>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, decision_id, action, status, reason, created_at, resolved_at
                FROM policy_approvals
                WHERE id = ?1
                "#,
                params![id],
                policy_approval_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub(crate) fn policy_guard(&self, request: PolicyRequest) -> Result<PolicyDecisionRecord> {
        let decision = self.policy_check(request)?;
        match decision.effect.as_str() {
            "allow" => Ok(decision),
            "deny" => bail!("policy denied {}: {}", decision.action, decision.reason),
            "require_approval" => bail!(
                "policy requires approval for {}: {} (approval_id: {})",
                decision.action,
                decision.reason,
                decision.approval_id.as_deref().unwrap_or("unknown")
            ),
            "defer" => bail!("policy deferred {}: {}", decision.action, decision.reason),
            other => bail!("policy produced unsupported effect {other}"),
        }
    }

    pub(crate) fn load_policy_rules(&self) -> Result<Vec<PolicyRule>> {
        let path = self.paths.home.join("arcwell-policy.toml");
        if !path.exists() {
            return Ok(default_policy_rules());
        }
        let raw =
            fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        let parsed: PolicyFile = toml::from_str(&raw)
            .with_context(|| format!("policy file {} is invalid TOML", path.display()))?;
        if parsed.rules.is_empty() {
            bail!(
                "policy file {} must contain at least one [[rules]] entry",
                path.display()
            );
        }
        for rule in &parsed.rules {
            validate_policy_rule(rule)
                .with_context(|| format!("invalid policy rule {}", rule.id))?;
        }
        Ok(parsed.rules)
    }

    pub(crate) fn cost_spent_for_policy(&self, policy: &CostPolicy) -> Result<f64> {
        let sql = match policy.scope.as_str() {
            "global" => {
                "SELECT COALESCE(sum(CASE WHEN actual_usd > 0 THEN actual_usd ELSE estimated_usd END), 0) FROM cost_entries"
            }
            "package" => {
                "SELECT COALESCE(sum(CASE WHEN actual_usd > 0 THEN actual_usd ELSE estimated_usd END), 0) FROM cost_entries WHERE package = ?1"
            }
            "provider" => {
                "SELECT COALESCE(sum(CASE WHEN actual_usd > 0 THEN actual_usd ELSE estimated_usd END), 0) FROM cost_entries WHERE provider = ?1"
            }
            "source" => {
                "SELECT COALESCE(sum(CASE WHEN actual_usd > 0 THEN actual_usd ELSE estimated_usd END), 0) FROM cost_entries WHERE source = ?1"
            }
            other => bail!("unsupported cost policy scope: {other}"),
        };
        if policy.scope == "global" {
            self.conn
                .query_row(sql, [], |row| row.get(0))
                .map_err(Into::into)
        } else {
            self.conn
                .query_row(sql, params![policy.key], |row| row.get(0))
                .map_err(Into::into)
        }
    }

    pub fn set_secret_ref(
        &self,
        name: &str,
        location: &str,
        scope: &str,
        expires_at: Option<&str>,
    ) -> Result<()> {
        validate_key(name)?;
        validate_key(scope)?;
        parse_optional_expiry(expires_at)?;
        let now = now();
        self.conn.execute(
            r#"
            INSERT INTO secret_refs (name, location, scope, expires_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(name) DO UPDATE SET
              location = excluded.location,
              scope = excluded.scope,
              expires_at = excluded.expires_at,
              updated_at = excluded.updated_at
            "#,
            params![name, location, scope, expires_at, now],
        )?;
        Ok(())
    }

    pub fn set_secret_ref_with_policy(
        &self,
        name: &str,
        location: &str,
        scope: &str,
        expires_at: Option<&str>,
        source: &str,
    ) -> Result<()> {
        self.policy_guard(PolicyRequest {
            action: "secret.write".to_string(),
            package: None,
            provider: None,
            source: Some(source.to_string()),
            channel: None,
            subject: None,
            target: Some(name.to_string()),
            projected_usd: None,
            metadata: json!({
                "operation": "set_ref",
                "scope": scope,
                "has_expires_at": expires_at.is_some(),
                "location_kind": secret_ref_location_kind(location),
            }),
            untrusted_excerpt: None,
        })?;
        self.set_secret_ref(name, location, scope, expires_at)
    }

    pub fn list_secret_refs(&self) -> Result<Vec<SecretRef>> {
        let mut stmt = self.conn.prepare(
            "SELECT name, location, scope, expires_at, updated_at FROM secret_refs ORDER BY name",
        )?;
        rows(stmt.query_map([], secret_from_row)?)
    }

    pub fn set_secret_value(&self, name: &str, value: &str, scope: &str) -> Result<()> {
        self.set_secret_value_with_metadata(name, value, scope, None, None)
    }

    pub fn set_secret_value_with_metadata(
        &self,
        name: &str,
        value: &str,
        scope: &str,
        provider: Option<&str>,
        expires_at: Option<&str>,
    ) -> Result<()> {
        validate_key(name)?;
        validate_key(scope)?;
        if let Some(provider) = provider {
            validate_key(provider)?;
        }
        if let Some(expires_at) = expires_at {
            parse_optional_expiry(Some(expires_at))?;
        }
        if value.is_empty() {
            bail!("secret value cannot be empty");
        }
        if value.len() > 20_000 {
            bail!("secret value is too long");
        }
        let now = now();
        self.conn.execute(
            r#"
            INSERT INTO secret_values (name, value, scope, provider, expires_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(name) DO UPDATE SET
              value = excluded.value,
              scope = excluded.scope,
              provider = excluded.provider,
              expires_at = excluded.expires_at,
              updated_at = excluded.updated_at
            "#,
            params![name, value, scope, provider, expires_at, now],
        )?;
        Ok(())
    }

    pub fn set_secret_value_with_policy(
        &self,
        name: &str,
        value: &str,
        scope: &str,
        provider: Option<&str>,
        expires_at: Option<&str>,
        source: &str,
    ) -> Result<()> {
        self.policy_guard(PolicyRequest {
            action: "secret.write".to_string(),
            package: None,
            provider: provider.map(ToOwned::to_owned),
            source: Some(source.to_string()),
            channel: None,
            subject: None,
            target: Some(name.to_string()),
            projected_usd: None,
            metadata: json!({
                "operation": "set_value",
                "scope": scope,
                "has_expires_at": expires_at.is_some(),
            }),
            untrusted_excerpt: None,
        })?;
        self.set_secret_value_with_metadata(name, value, scope, provider, expires_at)
    }

    pub fn get_secret_value(&self, name: &str) -> Result<Option<String>> {
        validate_key(name)?;
        self.conn
            .query_row(
                "SELECT value FROM secret_values WHERE name = ?1",
                params![name],
                |row| row.get(0),
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn get_secret_value_with_policy(&self, name: &str, source: &str) -> Result<Option<String>> {
        self.policy_guard(PolicyRequest {
            action: "secret.read".to_string(),
            package: None,
            provider: None,
            source: Some(source.to_string()),
            channel: None,
            subject: None,
            target: Some(name.to_string()),
            projected_usd: None,
            metadata: json!({ "operation": "get_value" }),
            untrusted_excerpt: None,
        })?;
        self.get_secret_value(name)
    }

    pub fn list_secret_values(&self) -> Result<Vec<SecretValue>> {
        let mut stmt = self.conn.prepare(
            "SELECT name, scope, provider, expires_at, updated_at FROM secret_values ORDER BY name",
        )?;
        rows(stmt.query_map([], secret_value_from_row)?)
    }

    pub fn secret_health(&self) -> Result<Vec<SecretHealth>> {
        let mut by_name = BTreeMap::new();
        for secret in self.list_secret_refs()? {
            let has_local_value = self.get_secret_value(&secret.name)?.is_some();
            by_name.insert(
                secret.name.clone(),
                secret_ref_health(&secret, has_local_value),
            );
        }
        for value in self.list_secret_values()? {
            by_name.insert(value.name.clone(), secret_value_health(value)?);
        }
        let x_bookmark_schedule_active = self
            .list_watch_sources()?
            .into_iter()
            .any(|source| source.status == "active" && source.source_kind == "x_bookmarks");
        let refresh_ready = by_name
            .get("X_REFRESH_TOKEN")
            .is_some_and(|item| item.present && item.status != "expired");
        let client_id_ready = by_name
            .get("X_CLIENT_ID")
            .is_some_and(|item| item.present && item.status != "expired");
        if x_bookmark_schedule_active {
            let x_scope_warning = "X bookmark ingestion is scheduled; stored X OAuth credentials must include user-context tweet.read, users.read, bookmark.read, follows.read, and offline.access scopes.";
            if !refresh_ready {
                push_secret_warning(
                    &mut by_name,
                    "X_REFRESH_TOKEN",
                    "x",
                    Some("x"),
                    &format!(
                        "{x_scope_warning} X_REFRESH_TOKEN is missing or expired, so scheduled bookmark ingestion cannot refresh stale bearer credentials."
                    ),
                );
            }
            if !client_id_ready {
                push_secret_warning(
                    &mut by_name,
                    "X_CLIENT_ID",
                    "x",
                    Some("x"),
                    &format!(
                        "{x_scope_warning} X_CLIENT_ID is missing or expired, so scheduled bookmark ingestion cannot perform OAuth refresh."
                    ),
                );
            }
            if let Some(bearer) = by_name.get_mut("X_BEARER_TOKEN")
                && bearer.status == "expiring_soon"
                && (!refresh_ready || !client_id_ready)
            {
                bearer.warnings.push(
                    "X_BEARER_TOKEN expires soon and scheduled X bookmark ingestion lacks complete stored refresh material; refresh credentials before the next scheduled run.".to_string(),
                );
            }
        }
        if refresh_ready && client_id_ready {
            let refresh_auth_failure =
                self.get_source_health("x:oauth-refresh")?.filter(|health| {
                    health.status == "auth_failed"
                        && health.source_kind.eq_ignore_ascii_case("x_oauth")
                });
            if let Some(refresh_health) = refresh_auth_failure {
                let warning = format!(
                    "Stored X_REFRESH_TOKEN was rejected during X OAuth refresh at {}; reauthorize X OAuth before scheduled bookmark/watch reads can recover. Last error: {}",
                    refresh_health
                        .last_failure_at
                        .as_deref()
                        .unwrap_or(&refresh_health.updated_at),
                    refresh_health
                        .last_error
                        .as_deref()
                        .unwrap_or("provider rejected stored refresh material")
                );
                push_secret_warning(&mut by_name, "X_REFRESH_TOKEN", "x", Some("x"), &warning);
                if let Some(bearer) = by_name.get_mut("X_BEARER_TOKEN") {
                    bearer.warnings.push(warning);
                }
            } else if let Some(warning) = self.x_oauth_refresh_policy_warning() {
                push_secret_warning(
                    &mut by_name,
                    "X_OAUTH_REFRESH_POLICY",
                    "x",
                    Some("x"),
                    &warning,
                );
                if let Some(bearer) = by_name.get_mut("X_BEARER_TOKEN") {
                    bearer.warnings.push(warning);
                }
            } else if let Some(bearer) = by_name.get_mut("X_BEARER_TOKEN")
                && matches!(bearer.status.as_str(), "expired" | "expiring_soon")
            {
                let secret_name = bearer.name.clone();
                bearer.warnings.retain(|warning| {
                    !(warning.starts_with(&format!("secret {secret_name} "))
                        && (warning.contains(" expired at ")
                            || warning.contains(" expires soon at ")))
                });
                bearer.status = "refreshable".to_string();
            }
        }
        let gmail_issue_schedule_dependency_count = self
            .list_issue_schedules()?
            .into_iter()
            .filter(|schedule| {
                schedule.status.eq_ignore_ascii_case("active")
                    && schedule.channel.eq_ignore_ascii_case("email")
                    && schedule
                        .recipient_ref
                        .trim()
                        .strip_prefix("email:")
                        .is_some_and(|address| address.to_ascii_lowercase().ends_with("@gmail.com"))
            })
            .count();
        let gmail_gaps = self.list_email_delivery_verification_gaps()?;
        let gmail_unverified_count = gmail_gaps
            .iter()
            .filter(|gap| gap.verification_state == "mailbox_unverified")
            .count();
        let gmail_repairable_count = gmail_gaps
            .iter()
            .filter(|gap| {
                matches!(
                    gap.verification_state.as_str(),
                    "mailbox_bad_placement_trash" | "mailbox_bad_placement_spam"
                )
            })
            .count();
        if gmail_unverified_count > 0
            || gmail_repairable_count > 0
            || gmail_issue_schedule_dependency_count > 0
        {
            let access_ready = by_name
                .get("GMAIL_ACCESS_TOKEN")
                .is_some_and(|item| item.present && item.status != "expired");
            let refresh_ready = by_name
                .get("GMAIL_REFRESH_TOKEN")
                .is_some_and(|item| item.present && item.status != "expired");
            let client_ready = by_name
                .get("GMAIL_CLIENT_ID")
                .or_else(|| by_name.get("GOOGLE_CLIENT_ID"))
                .is_some_and(|item| item.present && item.status != "expired");
            let gmail_scope_warning = format!(
                "Gmail mailbox verification has {gmail_unverified_count} unverified gap(s), {gmail_repairable_count} Trash/Spam repairable gap(s), and {gmail_issue_schedule_dependency_count} active Gmail-recipient email issue schedule(s); stored Gmail OAuth credentials must include https://www.googleapis.com/auth/gmail.readonly for verification and https://www.googleapis.com/auth/gmail.modify for placement repair."
            );
            if !(access_ready || refresh_ready && client_ready) {
                push_secret_warning(
                    &mut by_name,
                    "GMAIL_ACCESS_TOKEN",
                    "gmail",
                    Some("gmail"),
                    &format!(
                        "{gmail_scope_warning} GMAIL_ACCESS_TOKEN is missing or expired and Arcwell lacks complete stored refresh material."
                    ),
                );
            }
            if !refresh_ready {
                push_secret_warning(
                    &mut by_name,
                    "GMAIL_REFRESH_TOKEN",
                    "gmail",
                    Some("gmail"),
                    &format!(
                        "{gmail_scope_warning} GMAIL_REFRESH_TOKEN is missing or expired, so daemon-owned Gmail mailbox verification/repair cannot recover after access-token expiry."
                    ),
                );
            }
            if !client_ready {
                push_secret_warning(
                    &mut by_name,
                    "GMAIL_CLIENT_ID",
                    "gmail",
                    Some("gmail"),
                    &format!(
                        "{gmail_scope_warning} GMAIL_CLIENT_ID or GOOGLE_CLIENT_ID is missing or expired, so daemon-owned Gmail OAuth refresh cannot run."
                    ),
                );
            }
            if refresh_ready && client_ready {
                if let Some(warning) = self.gmail_oauth_refresh_policy_warning() {
                    push_secret_warning(
                        &mut by_name,
                        "GMAIL_OAUTH_REFRESH_POLICY",
                        "gmail",
                        Some("gmail"),
                        &warning,
                    );
                    if let Some(access) = by_name.get_mut("GMAIL_ACCESS_TOKEN") {
                        access.warnings.push(warning);
                    }
                } else if let Some(access) = by_name.get_mut("GMAIL_ACCESS_TOKEN")
                    && matches!(access.status.as_str(), "expired" | "expiring_soon")
                {
                    let secret_name = access.name.clone();
                    access.warnings.retain(|warning| {
                        !(warning.starts_with(&format!("secret {secret_name} "))
                            && (warning.contains(" expired at ")
                                || warning.contains(" expires soon at ")))
                    });
                    access.status = "refreshable".to_string();
                }
            }
        }
        Ok(by_name.into_values().collect())
    }

    pub(crate) fn x_oauth_refresh_policy_warning(&self) -> Option<String> {
        match self.policy_explain(PolicyRequest {
            action: "provider.oauth".to_string(),
            package: Some("arcwell-x".to_string()),
            provider: Some("x".to_string()),
            source: Some("x_oauth".to_string()),
            channel: None,
            subject: None,
            target: Some("https://api.x.com".to_string()),
            projected_usd: Some(estimated_network_fetch_cost(1)),
            metadata: json!({ "operation": "refresh", "health_check": true }),
            untrusted_excerpt: None,
        }) {
            Ok(explanation) if explanation.allowed => None,
            Ok(explanation) => Some(format!(
                "Arcwell cannot auto-refresh expired X_BEARER_TOKEN because provider.oauth policy for arcwell-x/x_oauth is {}; reason: {}",
                explanation.effect, explanation.reason
            )),
            Err(error) => Some(format!(
                "Arcwell cannot verify provider.oauth policy for X token refresh: {}",
                redact_secret_like_text(&error.to_string())
            )),
        }
    }

    pub(crate) fn gmail_oauth_refresh_policy_warning(&self) -> Option<String> {
        match self.policy_explain(PolicyRequest {
            action: "provider.oauth".to_string(),
            package: Some("arcwell-email".to_string()),
            provider: Some("gmail".to_string()),
            source: Some("gmail_oauth".to_string()),
            channel: Some("email".to_string()),
            subject: None,
            target: Some("https://oauth2.googleapis.com".to_string()),
            projected_usd: Some(estimated_network_fetch_cost(1)),
            metadata: json!({ "operation": "refresh", "health_check": true }),
            untrusted_excerpt: None,
        }) {
            Ok(explanation) if explanation.allowed => None,
            Ok(explanation) => Some(format!(
                "Arcwell cannot auto-refresh expired GMAIL_ACCESS_TOKEN because provider.oauth policy for arcwell-email/gmail_oauth is {}; reason: {}",
                explanation.effect, explanation.reason
            )),
            Err(error) => Some(format!(
                "Arcwell cannot verify provider.oauth policy for Gmail token refresh: {}",
                redact_secret_like_text(&error.to_string())
            )),
        }
    }

    pub(crate) fn get_usable_secret_value(&self, name: &str) -> Result<Option<String>> {
        validate_key(name)?;
        let metadata = self
            .conn
            .query_row(
                "SELECT name, scope, provider, expires_at, updated_at FROM secret_values WHERE name = ?1",
                params![name],
                secret_value_from_row,
            )
            .optional()?;
        if let Some(metadata) = metadata {
            let health = secret_value_health(metadata)?;
            if health.status == "expired" {
                bail!("{name} is expired; rotate or revoke the credential before use");
            }
        }
        self.get_secret_value(name)
    }

    pub fn delete_secret_value(&self, name: &str) -> Result<bool> {
        validate_key(name)?;
        Ok(self
            .conn
            .execute("DELETE FROM secret_values WHERE name = ?1", params![name])?
            > 0)
    }

    pub fn delete_secret_value_with_policy(&self, name: &str, source: &str) -> Result<bool> {
        self.policy_guard(PolicyRequest {
            action: "secret.write".to_string(),
            package: None,
            provider: None,
            source: Some(source.to_string()),
            channel: None,
            subject: None,
            target: Some(name.to_string()),
            projected_usd: None,
            metadata: json!({ "operation": "delete_value" }),
            untrusted_excerpt: None,
        })?;
        self.delete_secret_value(name)
    }
}
