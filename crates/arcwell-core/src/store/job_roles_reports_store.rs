use super::*;

struct PreparedJobApplicationPacketExport {
    packet: JobApplicationPacket,
    markdown: String,
    findings: Vec<JobPrivacyFinding>,
    filename: String,
}

impl Store {
    pub fn record_job_source(&self, input: JobSourceInput) -> Result<JobSource> {
        let input = normalize_job_source_input(input)?;
        let id = job_source_id(&input.url);
        let metadata_json = serde_json::to_string(&input.metadata)?;
        let timestamp = now();
        self.conn.execute(
            r#"
            INSERT INTO job_sources
              (id, source_family, name, url, market_scope, refresh_policy, metadata_json, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)
            ON CONFLICT(url) DO UPDATE SET
              source_family = excluded.source_family,
              name = excluded.name,
              market_scope = excluded.market_scope,
              refresh_policy = excluded.refresh_policy,
              metadata_json = excluded.metadata_json,
              updated_at = excluded.updated_at
            "#,
            params![
                id,
                input.source_family,
                input.name,
                input.url,
                input.market_scope,
                input.refresh_policy,
                metadata_json,
                timestamp,
            ],
        )?;
        self.read_job_source(&id)?
            .with_context(|| format!("job source not found: {id}"))
    }

    pub fn read_job_source(&self, id: &str) -> Result<Option<JobSource>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, source_family, name, url, market_scope, refresh_policy, metadata_json, created_at, updated_at
                FROM job_sources
                WHERE id = ?1
                "#,
                params![id],
                job_source_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn record_job_source_health(&self, input: JobSourceHealthInput) -> Result<JobSourceHealth> {
        let input = normalize_job_source_health_input(input)?;
        self.read_job_source(&input.source_id)?
            .with_context(|| format!("job source not found: {}", input.source_id))?;
        let id = job_source_health_id();
        self.conn.execute(
            r#"
            INSERT INTO job_source_health
              (id, source_id, checked_at, status, http_status, error_code, fetched_count, accepted_count, rejected_count, note)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                id,
                input.source_id,
                now(),
                input.status,
                input.http_status,
                input.error_code,
                input.fetched_count as i64,
                input.accepted_count as i64,
                input.rejected_count as i64,
                input.note,
            ],
        )?;
        self.read_job_source_health(&id)?
            .with_context(|| format!("job source health not found: {id}"))
    }

    pub fn read_job_source_health(&self, id: &str) -> Result<Option<JobSourceHealth>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, source_id, checked_at, status, http_status, error_code, fetched_count, accepted_count, rejected_count, note
                FROM job_source_health
                WHERE id = ?1
                "#,
                params![id],
                job_source_health_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn record_job_role_card(&self, input: JobRoleCardInput) -> Result<JobRoleCard> {
        let input = normalize_job_role_card_input(input)?;
        self.validate_job_evidence_card_ids(&input.evidence_card_ids, None, false)?;
        let id = job_role_card_id(&input.company, &input.role_title, &input.source_url);
        let core_requirements_json = serde_json::to_string(&input.core_requirements)?;
        let evidence_card_ids_json = serde_json::to_string(&input.evidence_card_ids)?;
        let gaps_or_blockers_json = serde_json::to_string(&input.gaps_or_blockers)?;
        let metadata_json = serde_json::to_string(&input.metadata)?;
        let timestamp = now();
        self.conn.execute(
            r#"
            INSERT INTO job_role_cards
              (id, company, role_title, canonical_url, source_family, source_url, source_confidence, date_accessed, posting_freshness, location, work_mode, company_stage_or_size, role_seniority, core_requirements_json, implied_business_problem, why_they_might_need_user, evidence_card_ids_json, gaps_or_blockers_json, cluster, current_status, metadata_json, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?22)
            ON CONFLICT(company, role_title, source_url) DO UPDATE SET
              canonical_url = excluded.canonical_url,
              source_family = excluded.source_family,
              source_confidence = excluded.source_confidence,
              date_accessed = excluded.date_accessed,
              posting_freshness = excluded.posting_freshness,
              location = excluded.location,
              work_mode = excluded.work_mode,
              company_stage_or_size = excluded.company_stage_or_size,
              role_seniority = excluded.role_seniority,
              core_requirements_json = excluded.core_requirements_json,
              implied_business_problem = excluded.implied_business_problem,
              why_they_might_need_user = excluded.why_they_might_need_user,
              evidence_card_ids_json = excluded.evidence_card_ids_json,
              gaps_or_blockers_json = excluded.gaps_or_blockers_json,
              cluster = excluded.cluster,
              current_status = excluded.current_status,
              metadata_json = excluded.metadata_json,
              updated_at = excluded.updated_at
            "#,
            params![
                id,
                input.company,
                input.role_title,
                input.canonical_url,
                input.source_family,
                input.source_url,
                input.source_confidence,
                input.date_accessed.unwrap_or_else(now),
                input.posting_freshness,
                input.location,
                input.work_mode,
                input.company_stage_or_size,
                input.role_seniority,
                core_requirements_json,
                input.implied_business_problem,
                input.why_they_might_need_user,
                evidence_card_ids_json,
                gaps_or_blockers_json,
                input.cluster,
                input.current_status,
                metadata_json,
                timestamp,
            ],
        )?;
        let role = self
            .read_job_role_card(&id)?
            .with_context(|| format!("job role card not found: {id}"))?;
        self.record_job_role_source_link(JobRoleSourceLinkInput {
            role_id: role.id.clone(),
            source_id: None,
            source_url: role
                .canonical_url
                .clone()
                .unwrap_or_else(|| role.source_url.clone()),
            confidence: role.source_confidence.clone(),
            evidence_excerpt: Some("Primary role source recorded with role card.".to_string()),
        })?;
        Ok(role)
    }

    pub fn read_job_role_card(&self, id: &str) -> Result<Option<JobRoleCard>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, company, role_title, canonical_url, source_family, source_url, source_confidence, date_accessed, posting_freshness, location, work_mode, company_stage_or_size, role_seniority, core_requirements_json, implied_business_problem, why_they_might_need_user, evidence_card_ids_json, gaps_or_blockers_json, cluster, current_status, metadata_json, created_at, updated_at
                FROM job_role_cards
                WHERE id = ?1
                "#,
                params![id],
                job_role_card_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_job_role_cards(&self) -> Result<Vec<JobRoleCard>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, company, role_title, canonical_url, source_family, source_url, source_confidence, date_accessed, posting_freshness, location, work_mode, company_stage_or_size, role_seniority, core_requirements_json, implied_business_problem, why_they_might_need_user, evidence_card_ids_json, gaps_or_blockers_json, cluster, current_status, metadata_json, created_at, updated_at
            FROM job_role_cards
            ORDER BY created_at ASC
            "#,
        )?;
        rows(stmt.query_map([], job_role_card_from_row)?)
    }

    pub fn record_job_role_source_link(
        &self,
        input: JobRoleSourceLinkInput,
    ) -> Result<JobRoleSourceLink> {
        let input = normalize_job_role_source_link_input(input)?;
        self.read_job_role_card(&input.role_id)?
            .with_context(|| format!("job role card not found: {}", input.role_id))?;
        if let Some(source_id) = &input.source_id {
            self.read_job_source(source_id)?
                .with_context(|| format!("job source not found: {source_id}"))?;
        }
        let id = job_role_source_link_id(&input.role_id, &input.source_url);
        self.conn.execute(
            r#"
            INSERT INTO job_role_source_links
              (id, role_id, source_id, source_url, observed_at, confidence, evidence_excerpt)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(role_id, source_url) DO UPDATE SET
              source_id = excluded.source_id,
              observed_at = excluded.observed_at,
              confidence = excluded.confidence,
              evidence_excerpt = excluded.evidence_excerpt
            "#,
            params![
                id,
                input.role_id,
                input.source_id,
                input.source_url,
                now(),
                input.confidence,
                input.evidence_excerpt,
            ],
        )?;
        self.read_job_role_source_link(&id)?
            .with_context(|| format!("job role source link not found: {id}"))
    }

    pub fn read_job_role_source_link(&self, id: &str) -> Result<Option<JobRoleSourceLink>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, role_id, source_id, source_url, observed_at, confidence, evidence_excerpt
                FROM job_role_source_links
                WHERE id = ?1
                "#,
                params![id],
                job_role_source_link_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_job_role_source_links(&self, role_id: &str) -> Result<Vec<JobRoleSourceLink>> {
        validate_id(role_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, role_id, source_id, source_url, observed_at, confidence, evidence_excerpt
            FROM job_role_source_links
            WHERE role_id = ?1
            ORDER BY observed_at ASC
            "#,
        )?;
        rows(stmt.query_map(params![role_id], job_role_source_link_from_row)?)
    }

    pub(crate) fn list_job_roles_for_source(&self, source_id: &str) -> Result<Vec<JobRoleCard>> {
        validate_id(source_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT DISTINCT r.id, r.company, r.role_title, r.canonical_url, r.source_family, r.source_url, r.source_confidence, r.date_accessed, r.posting_freshness, r.location, r.work_mode, r.company_stage_or_size, r.role_seniority, r.core_requirements_json, r.implied_business_problem, r.why_they_might_need_user, r.evidence_card_ids_json, r.gaps_or_blockers_json, r.cluster, r.current_status, r.metadata_json, r.created_at, r.updated_at
            FROM job_role_cards r
            JOIN job_role_source_links l ON l.role_id = r.id
            WHERE l.source_id = ?1
            ORDER BY r.updated_at DESC
            "#,
        )?;
        rows(stmt.query_map(params![source_id], job_role_card_from_row)?)
    }

    pub fn record_job_fit_score(&self, input: JobFitScoreInput) -> Result<JobFitScore> {
        let input = normalize_job_fit_score_input(input)?;
        let role = self
            .read_job_role_card(&input.role_id)?
            .with_context(|| format!("job role card not found: {}", input.role_id))?;
        self.require_job_profile(&input.profile_id)?;
        self.validate_job_evidence_card_ids(
            &input.evidence_card_ids,
            Some(&input.profile_id),
            false,
        )?;
        let mut blockers = input.blockers.clone();
        if role.current_status != "live" {
            blockers.push(format!("role source status is {}", role.current_status));
        }
        if role.source_confidence == "stale" || role.source_confidence == "unknown" {
            blockers.push(format!(
                "role source confidence is {}",
                role.source_confidence
            ));
        }
        if role.source_confidence == "aggregator_only" {
            blockers.push("aggregator-only source cannot support apply-now tier".to_string());
        }
        blockers = normalize_job_string_list(blockers, "job score blocker", 500)?;
        let weighted_score = job_weighted_score(&input);
        let tier = job_score_tier(weighted_score, &role.source_confidence, &blockers);
        let id = job_fit_score_id();
        self.conn.execute(
            r#"
            INSERT INTO job_fit_scores
              (id, role_id, profile_id, scored_at, scorer, role_fit, domain_fit, evidence_fit, geo_work_fit, stage_fit, practical_odds, interest_energy, weighted_score, tier, blockers_json, evidence_card_ids_json, explanation)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
            "#,
            params![
                id,
                input.role_id,
                input.profile_id,
                now(),
                input.scorer,
                input.role_fit,
                input.domain_fit,
                input.evidence_fit,
                input.geo_work_fit,
                input.stage_fit,
                input.practical_odds,
                input.interest_energy,
                weighted_score,
                tier,
                serde_json::to_string(&blockers)?,
                serde_json::to_string(&input.evidence_card_ids)?,
                input.explanation,
            ],
        )?;
        self.read_job_fit_score(&id)?
            .with_context(|| format!("job fit score not found: {id}"))
    }

    pub fn read_job_fit_score(&self, id: &str) -> Result<Option<JobFitScore>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, role_id, profile_id, scored_at, scorer, role_fit, domain_fit, evidence_fit, geo_work_fit, stage_fit, practical_odds, interest_energy, weighted_score, tier, blockers_json, evidence_card_ids_json, explanation
                FROM job_fit_scores
                WHERE id = ?1
                "#,
                params![id],
                job_fit_score_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn compile_job_shortlist(&self, profile_id: &str) -> Result<JobShortlist> {
        self.require_job_profile(profile_id)?;
        let applications = self.list_job_applications()?;
        let mut entries = Vec::new();
        for role in self.list_job_role_cards()? {
            let score = self
                .latest_job_fit_score(&role.id, profile_id)?
                .map(|score| job_effective_score_for_role(&role, score));
            let outcome_warnings = self.job_outcome_warnings_for_role(&role, &applications)?;
            entries.push(JobShortlistEntry {
                role,
                score,
                outcome_warnings,
            });
        }
        entries.sort_by(|left, right| {
            let left_rank = left
                .score
                .as_ref()
                .map(|score| job_tier_sort_rank(&score.tier))
                .unwrap_or(99);
            let right_rank = right
                .score
                .as_ref()
                .map(|score| job_tier_sort_rank(&score.tier))
                .unwrap_or(99);
            left_rank
                .cmp(&right_rank)
                .then_with(|| {
                    let left_score = left.score.as_ref().map(|s| s.weighted_score).unwrap_or(0.0);
                    let right_score = right
                        .score
                        .as_ref()
                        .map(|s| s.weighted_score)
                        .unwrap_or(0.0);
                    right_score
                        .partial_cmp(&left_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .then_with(|| left.role.company.cmp(&right.role.company))
        });
        Ok(JobShortlist {
            profile_id: profile_id.to_string(),
            generated_at: now(),
            entries,
        })
    }

    pub fn compile_job_outreach_readiness_report(
        &self,
        profile_id: &str,
        limit: usize,
    ) -> Result<JobOutreachReadinessReport> {
        self.require_job_profile(profile_id)?;
        let limit = limit.clamp(1, 100);
        let shortlist = self.compile_job_shortlist(profile_id)?;
        let all_intro_paths = self.list_job_intro_paths()?;
        let contacts = self.list_job_contacts()?;
        let contacts_by_id = contacts
            .into_iter()
            .map(|contact| (contact.id.clone(), contact))
            .collect::<BTreeMap<_, _>>();
        let mut entries = Vec::new();
        for shortlist_entry in shortlist
            .entries
            .into_iter()
            .filter(|entry| entry.score.is_some())
            .take(limit)
        {
            let role = shortlist_entry.role;
            let score = shortlist_entry.score;
            let mut blockers = Vec::new();
            if role.current_status != "live" {
                blockers.push(format!("role status is `{}`", role.current_status));
            }
            if let Some(score) = &score {
                if !matches!(score.tier.as_str(), "tier_1" | "tier_2") {
                    blockers.push(format!(
                        "fit tier is `{}`; outreach readiness is limited to Tier 1 or Tier 2 roles",
                        score.tier
                    ));
                }
                for blocker in &score.blockers {
                    blockers.push(format!("fit blocker: {blocker}"));
                }
            }
            for warning in &shortlist_entry.outcome_warnings {
                blockers.push(format!("pipeline warning: {warning}"));
            }

            let packet =
                self.latest_job_application_packet_for_role_profile(&role.id, profile_id)?;
            let mut packet_id = None;
            let mut packet_status = None;
            let mut privacy_check_id = None;
            if let Some(packet) = packet {
                packet_id = Some(packet.id.clone());
                packet_status = Some(packet.status.clone());
                if packet.status != "approved" {
                    blockers.push(format!(
                        "latest application packet is `{}`; approved packet required",
                        packet.status
                    ));
                } else {
                    let check = self.check_job_privacy_text(
                        "job_outreach_readiness",
                        Some(&packet.id),
                        &packet.outreach_note,
                        &[],
                    )?;
                    privacy_check_id = Some(check.id.clone());
                    if check.decision != "pass" {
                        blockers.push(format!(
                            "approved packet outreach note privacy decision is `{}`",
                            check.decision
                        ));
                    }
                }
            } else {
                blockers.push("no application packet exists for this role".to_string());
            }

            let role_intro_paths = all_intro_paths
                .iter()
                .filter(|path| path.role_id == role.id)
                .cloned()
                .collect::<Vec<_>>();
            let intro_path_ids = role_intro_paths
                .iter()
                .map(|path| path.id.clone())
                .collect::<Vec<_>>();
            let contact_ids = role_intro_paths
                .iter()
                .map(|path| path.contact_id.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            let public_only_count = contact_ids
                .iter()
                .filter(|contact_id| {
                    contacts_by_id
                        .get(*contact_id)
                        .map(|contact| contact.relationship_status == "public_only")
                        .unwrap_or(false)
                })
                .count();
            let warm_intro_ready_count = role_intro_paths
                .iter()
                .filter(|path| {
                    let warm_contact = contacts_by_id
                        .get(&path.contact_id)
                        .map(|contact| {
                            matches!(
                                contact.relationship_status.as_str(),
                                "known" | "possible_mutual"
                            )
                        })
                        .unwrap_or(false);
                    warm_contact && job_intro_path_is_warm_ready(path)
                })
                .count();
            if role_intro_paths.is_empty() {
                blockers.push("no intro or outreach path recorded for this role".to_string());
            } else if warm_intro_ready_count == 0 {
                blockers.push(
                    "no user-confirmed warm intro or outreach route is ready; public-only paths remain identify/monitor work"
                        .to_string(),
                );
            }

            let decision = if blockers.is_empty() {
                "ready".to_string()
            } else {
                "blocked".to_string()
            };
            let next_action = if decision == "ready" {
                "Use the approved packet to ask for the warm intro or user-confirmed outreach route; Arcwell has not sent anything.".to_string()
            } else {
                format!("Resolve before outreach: {}", blockers.join("; "))
            };
            entries.push(JobOutreachReadinessEntry {
                role,
                score,
                packet_id,
                packet_status,
                privacy_check_id,
                intro_path_ids,
                contact_ids,
                warm_intro_ready_count,
                public_only_count,
                decision,
                blockers,
                next_action,
            });
        }
        let ready_count = entries
            .iter()
            .filter(|entry| entry.decision == "ready")
            .count();
        let blocked_count = entries.len().saturating_sub(ready_count);
        Ok(JobOutreachReadinessReport {
            profile_id: profile_id.to_string(),
            generated_at: now(),
            proof_level: "local_proof".to_string(),
            ready_count,
            blocked_count,
            entries,
            non_claims: vec![
                "This report does not send outreach or applications.".to_string(),
                "Public-only contacts are not warm intros.".to_string(),
                "A ready row means local packet/privacy/path gates passed, not that a person agreed to introduce or reply.".to_string(),
            ],
        })
    }

    pub(crate) fn job_outcome_warnings_for_role(
        &self,
        role: &JobRoleCard,
        applications: &[JobApplication],
    ) -> Result<Vec<String>> {
        let mut warnings = BTreeSet::new();
        for application in applications {
            let Some(application_role) = self.read_job_role_card(&application.role_id)? else {
                continue;
            };
            if application.role_id == role.id {
                match application.status.as_str() {
                    "planned" => {
                        warnings.insert(
                            "This role already has a planned application; avoid duplicate packet work."
                                .to_string(),
                        );
                    }
                    "applied" | "intro_requested" | "replied" | "interview" | "offer" => {
                        warnings.insert(format!(
                            "This role already has application status `{}`; do not treat it as a fresh lead.",
                            application.status
                        ));
                    }
                    "rejected" | "withdrawn" => {
                        warnings.insert(format!(
                            "This role already has outcome `{}`; keep follow-up user-confirmed.",
                            application.status
                        ));
                    }
                    _ => {}
                }
                continue;
            }

            if !application_role.company.eq_ignore_ascii_case(&role.company) {
                continue;
            }
            match application.status.as_str() {
                "rejected" => {
                    warnings.insert(format!(
                        "Previous application to {} was rejected; treat that as one data point, not a scoring rule.",
                        role.company
                    ));
                }
                "withdrawn" => {
                    warnings.insert(format!(
                        "Previous application to {} was withdrawn; check whether the same blocker still applies.",
                        role.company
                    ));
                }
                "replied" | "interview" | "offer" => {
                    warnings.insert(format!(
                        "Previous application to {} reached `{}`; use that history in follow-up notes, not automatic tier promotion.",
                        role.company, application.status
                    ));
                }
                _ => {}
            }
        }
        Ok(warnings.into_iter().collect())
    }

    pub fn compile_job_company_target_report(
        &self,
        profile_id: &str,
        market: Option<&str>,
        limit: usize,
    ) -> Result<JobCompanyTargetReport> {
        self.require_job_profile(profile_id)?;
        let market = market
            .map(|value| normalize_research_key(value.to_string(), "company target market"))
            .transpose()?;
        let limit = limit.clamp(1, 100);
        let evidence_tags =
            job_company_target_evidence_tags(&self.list_job_evidence_cards(profile_id)?);
        let mut entries = Vec::new();
        for company in self.list_job_company_cards()? {
            if market
                .as_deref()
                .is_some_and(|target_market| company.market != target_market)
            {
                continue;
            }
            entries.push(job_company_target_entry(&company, &evidence_tags));
        }
        entries.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.company.company_name.cmp(&right.company.company_name))
        });
        entries.truncate(limit);

        let mut warnings = vec![
            "Company targets are scouting leads from company cards, not current role cards."
                .to_string(),
            "Create or refresh canonical role cards before treating any entry as apply-now."
                .to_string(),
        ];
        if entries.is_empty() {
            warnings.push("No company cards matched the requested market.".to_string());
        }

        Ok(JobCompanyTargetReport {
            profile_id: profile_id.to_string(),
            market,
            generated_at: now(),
            proof_level: "local_proof".to_string(),
            entries,
            warnings,
        })
    }

    pub fn record_job_skeptic_finding(
        &self,
        input: JobSkepticFindingInput,
    ) -> Result<JobSkepticFinding> {
        let input = normalize_job_skeptic_finding_input(input)?;
        self.read_job_role_card(&input.role_id)?
            .with_context(|| format!("job role card not found: {}", input.role_id))?;
        let id = job_skeptic_finding_id();
        self.conn.execute(
            r#"
            INSERT INTO job_skeptic_findings
              (id, role_id, severity, finding_type, finding, next_action, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                id,
                input.role_id,
                input.severity,
                input.finding_type,
                input.finding,
                input.next_action,
                now(),
            ],
        )?;
        self.read_job_skeptic_finding(&id)?
            .with_context(|| format!("job skeptic finding not found: {id}"))
    }

    pub fn read_job_skeptic_finding(&self, id: &str) -> Result<Option<JobSkepticFinding>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, role_id, severity, finding_type, finding, next_action, created_at
                FROM job_skeptic_findings
                WHERE id = ?1
                "#,
                params![id],
                job_skeptic_finding_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn create_job_application_packet(
        &self,
        input: JobApplicationPacketInput,
    ) -> Result<JobApplicationPacket> {
        let input = normalize_job_application_packet_input(input)?;
        let role = self
            .read_job_role_card(&input.role_id)?
            .with_context(|| format!("job role card not found: {}", input.role_id))?;
        self.require_job_profile(&input.profile_id)?;
        if role.current_status != "live" {
            bail!("cannot create application packet for non-live job role");
        }
        if !matches!(
            role.source_confidence.as_str(),
            "canonical_confirmed" | "secondary_confirmed"
        ) {
            bail!("application packet requires canonical or secondary-confirmed role source");
        }
        let evidence = self.validate_job_evidence_card_ids(
            &input.evidence_card_ids,
            Some(&input.profile_id),
            false,
        )?;
        if evidence.is_empty() {
            bail!("application packet requires evidence cards");
        }
        if !text_contains_case_insensitive(&input.outreach_note, &role.company) {
            bail!("application packet outreach note must include a company-specific sentence");
        }
        if job_value_contains_local_reference(&input.proof_links) {
            bail!("application packet proof links cannot include local files or private paths");
        }
        let mut extra_blocked_terms = Vec::new();
        for card in &evidence {
            extra_blocked_terms.extend(card.unsafe_terms.clone());
        }
        let packet_text = job_application_packet_text(&role, &input);
        let findings = self.evaluate_job_privacy_text(&packet_text, &extra_blocked_terms)?;
        let decision = job_privacy_decision(&findings);
        if decision == "block" || decision == "warn" {
            bail!("application packet failed privacy check with decision {decision}");
        }
        let id = job_application_packet_id();
        let privacy_check = self.record_job_privacy_check_result(
            "packet",
            Some(&id),
            &decision,
            findings,
            &packet_text,
        )?;
        self.conn.execute(
            r#"
            INSERT INTO job_application_packets
              (id, role_id, profile_id, generated_at, status, evidence_card_ids_json, resume_emphasis, tailored_bullets_json, outreach_note, proof_links_json, likely_objections_json, interview_stories_json, questions_to_ask_json, privacy_check_id, reviewer_note)
            VALUES (?1, ?2, ?3, ?4, 'draft', ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            "#,
            params![
                id,
                input.role_id,
                input.profile_id,
                now(),
                serde_json::to_string(&input.evidence_card_ids)?,
                input.resume_emphasis,
                serde_json::to_string(&input.tailored_bullets)?,
                input.outreach_note,
                serde_json::to_string(&input.proof_links)?,
                serde_json::to_string(&input.likely_objections)?,
                serde_json::to_string(&input.interview_stories)?,
                serde_json::to_string(&input.questions_to_ask)?,
                privacy_check.id,
                input.reviewer_note,
            ],
        )?;
        self.read_job_application_packet(&id)?
            .with_context(|| format!("job application packet not found: {id}"))
    }

    pub fn read_job_application_packet(&self, id: &str) -> Result<Option<JobApplicationPacket>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, role_id, profile_id, generated_at, status, evidence_card_ids_json, resume_emphasis, tailored_bullets_json, outreach_note, proof_links_json, likely_objections_json, interview_stories_json, questions_to_ask_json, privacy_check_id, reviewer_note
                FROM job_application_packets
                WHERE id = ?1
                "#,
                params![id],
                job_application_packet_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    fn latest_job_application_packet_for_role_profile(
        &self,
        role_id: &str,
        profile_id: &str,
    ) -> Result<Option<JobApplicationPacket>> {
        validate_id(role_id)?;
        validate_id(profile_id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, role_id, profile_id, generated_at, status, evidence_card_ids_json, resume_emphasis, tailored_bullets_json, outreach_note, proof_links_json, likely_objections_json, interview_stories_json, questions_to_ask_json, privacy_check_id, reviewer_note
                FROM job_application_packets
                WHERE role_id = ?1
                  AND profile_id = ?2
                ORDER BY generated_at DESC, id DESC
                LIMIT 1
                "#,
                params![role_id, profile_id],
                job_application_packet_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn update_job_application_packet_status(
        &self,
        input: JobApplicationPacketStatusInput,
    ) -> Result<JobApplicationPacket> {
        let input = normalize_job_application_packet_status_input(input)?;
        let packet = self
            .read_job_application_packet(&input.packet_id)?
            .with_context(|| format!("job application packet not found: {}", input.packet_id))?;
        let reviewer_note = input
            .reviewer_note
            .or(packet.reviewer_note.clone())
            .unwrap_or_default();
        if input.status == "approved" {
            if reviewer_note.trim().is_empty() {
                bail!("approved application packet requires reviewer_note");
            }
            let privacy_check = self
                .read_job_privacy_check(&packet.privacy_check_id)?
                .with_context(|| {
                    format!(
                        "job application packet privacy check not found: {}",
                        packet.privacy_check_id
                    )
                })?;
            if privacy_check.decision != "pass" {
                bail!(
                    "cannot approve application packet with privacy decision {}",
                    privacy_check.decision
                );
            }
            let role = self
                .read_job_role_card(&packet.role_id)?
                .with_context(|| format!("job role card not found: {}", packet.role_id))?;
            if role.current_status != "live" {
                bail!("cannot approve application packet for non-live job role");
            }
        }
        self.conn.execute(
            r#"
            UPDATE job_application_packets
            SET status = ?2, reviewer_note = ?3
            WHERE id = ?1
            "#,
            params![&input.packet_id, &input.status, &reviewer_note],
        )?;
        self.read_job_application_packet(&input.packet_id)?
            .with_context(|| format!("job application packet not found: {}", input.packet_id))
    }

    pub fn export_job_application_packet(
        &self,
        packet_id: &str,
        out_dir: &Path,
    ) -> Result<JobApplicationPacketExport> {
        let prepared = self.prepare_job_application_packet_export(packet_id)?;
        if out_dir.exists() && !out_dir.is_dir() {
            bail!("application packet export output path is not a directory");
        }
        fs::create_dir_all(out_dir).with_context(|| {
            format!(
                "creating application packet export directory {}",
                out_dir.display()
            )
        })?;
        self.write_prepared_job_application_packet_export(prepared, out_dir)
    }

    fn prepare_job_application_packet_export(
        &self,
        packet_id: &str,
    ) -> Result<PreparedJobApplicationPacketExport> {
        validate_id(packet_id)?;
        let packet = self
            .read_job_application_packet(packet_id)?
            .with_context(|| format!("job application packet not found: {packet_id}"))?;
        if packet.status != "approved" {
            bail!(
                "job application packet export requires approved status, found {}",
                packet.status
            );
        }
        let role = self
            .read_job_role_card(&packet.role_id)?
            .with_context(|| format!("job role card not found: {}", packet.role_id))?;
        if role.current_status != "live" {
            bail!("cannot export application packet for non-live job role");
        }
        let privacy_check = self
            .read_job_privacy_check(&packet.privacy_check_id)?
            .with_context(|| {
                format!(
                    "job application packet privacy check not found: {}",
                    packet.privacy_check_id
                )
            })?;
        if privacy_check.decision != "pass" {
            bail!(
                "cannot export application packet with privacy decision {}",
                privacy_check.decision
            );
        }
        let evidence = self.validate_job_evidence_card_ids(
            &packet.evidence_card_ids,
            Some(&packet.profile_id),
            false,
        )?;
        if evidence.is_empty() {
            bail!("application packet export requires evidence cards");
        }
        let markdown = render_job_application_packet_export_markdown(&role, &packet, &evidence);
        let mut blocked_terms = Vec::new();
        for card in &evidence {
            blocked_terms.extend(card.unsafe_terms.clone());
        }
        let findings = self.evaluate_job_privacy_text(&markdown, &blocked_terms)?;
        let decision = job_privacy_decision(&findings);
        if decision != "pass" {
            bail!("application packet export failed privacy check with decision {decision}");
        }
        let slug = slugify(&format!("{} {}", role.company, role.role_title));
        let filename = format!("{slug}-{}.md", packet.id);
        Ok(PreparedJobApplicationPacketExport {
            packet,
            markdown,
            findings,
            filename,
        })
    }

    fn write_prepared_job_application_packet_export(
        &self,
        prepared: PreparedJobApplicationPacketExport,
        out_dir: &Path,
    ) -> Result<JobApplicationPacketExport> {
        let export_privacy_check = self.record_job_privacy_check_result(
            "packet_export",
            Some(&prepared.packet.id),
            "pass",
            prepared.findings,
            &prepared.markdown,
        )?;
        let path = out_dir.join(prepared.filename);
        fs::write(&path, prepared.markdown.as_bytes())
            .with_context(|| format!("writing application packet export {}", path.display()))?;
        Ok(JobApplicationPacketExport {
            packet_id: prepared.packet.id,
            role_id: prepared.packet.role_id,
            profile_id: prepared.packet.profile_id,
            path: path.to_string_lossy().to_string(),
            byte_len: prepared.markdown.len(),
            sha256: sha256(prepared.markdown.as_bytes()),
            privacy_check_id: export_privacy_check.id,
            proof_level: "local_proof".to_string(),
            delivery_status: "not_sent".to_string(),
            application_status_changed: false,
            warnings: vec![
                "Local Markdown export only; no application was sent or recorded.".to_string(),
                "User must review the exported artifact before external use.".to_string(),
            ],
        })
    }

    pub fn export_job_application_packet_set(
        &self,
        profile_id: &str,
        packet_ids: Vec<String>,
        out_dir: &Path,
    ) -> Result<JobApplicationPacketSetExport> {
        validate_id(profile_id)?;
        self.require_job_profile(profile_id)?;
        let packet_ids = normalize_job_id_list(packet_ids, "job application packet id")?;
        if packet_ids.is_empty() {
            bail!("job application packet set export requires at least one packet id");
        }
        if packet_ids.len() > 50 {
            bail!("job application packet set export has too many packet ids");
        }
        let mut prepared_exports = Vec::new();
        for packet_id in &packet_ids {
            let prepared = self.prepare_job_application_packet_export(packet_id)?;
            if prepared.packet.profile_id != profile_id {
                bail!(
                    "job application packet {} belongs to profile {}, not {}",
                    prepared.packet.id,
                    prepared.packet.profile_id,
                    profile_id
                );
            }
            prepared_exports.push(prepared);
        }
        if out_dir.exists() && !out_dir.is_dir() {
            bail!("application packet set export output path is not a directory");
        }
        fs::create_dir_all(out_dir).with_context(|| {
            format!(
                "creating application packet set export directory {}",
                out_dir.display()
            )
        })?;

        let mut exports = Vec::new();
        for prepared in prepared_exports {
            exports.push(self.write_prepared_job_application_packet_export(prepared, out_dir)?);
        }
        let total_byte_len = exports.iter().map(|export| export.byte_len).sum::<usize>();
        let export_fingerprint = exports
            .iter()
            .map(|export| format!("{}\t{}\t{}", export.packet_id, export.path, export.sha256))
            .collect::<Vec<_>>()
            .join("\n");
        let export_set_sha256 = sha256(export_fingerprint.as_bytes());
        let packet_set_hash = sha256(packet_ids.join("\n").as_bytes());
        let manifest_path = out_dir.join(format!(
            "job-packet-export-set-{}-{}.json",
            profile_id,
            &packet_set_hash[..16]
        ));
        let report = JobApplicationPacketSetExport {
            profile_id: profile_id.to_string(),
            packet_ids,
            out_dir: out_dir.to_string_lossy().to_string(),
            manifest_path: manifest_path.to_string_lossy().to_string(),
            exported_count: exports.len(),
            total_byte_len,
            export_set_sha256,
            exports,
            proof_level: "local_proof".to_string(),
            delivery_status: "not_sent".to_string(),
            application_status_changed: false,
            warnings: vec![
                "Local Markdown packet-set export only; no application was sent or recorded."
                    .to_string(),
                "User must review every exported artifact before external use.".to_string(),
            ],
            non_claims: vec![
                "This is not Google Docs draft creation.".to_string(),
                "This is not email, browser, ATS, or provider delivery.".to_string(),
                "This is not application submission proof.".to_string(),
                "This is not proof that the user approved these packets for sending.".to_string(),
                "This is not operational-home proof unless intentionally run in that home."
                    .to_string(),
            ],
        };
        let manifest = serde_json::to_vec_pretty(&report)?;
        fs::write(&manifest_path, manifest).with_context(|| {
            format!(
                "writing application packet set export manifest {}",
                manifest_path.display()
            )
        })?;
        Ok(report)
    }

    pub fn record_job_company_card(&self, input: JobCompanyCardInput) -> Result<JobCompanyCard> {
        let input = normalize_job_company_card_input(input)?;
        let id = job_company_card_id(&input.website_url);
        let metadata_json = serde_json::to_string(&input.metadata)?;
        let timestamp = now();
        self.conn.execute(
            r#"
            INSERT INTO job_company_cards
              (id, company_name, website_url, source_family, market, stage, funding_signal, product_category, technical_audience, developer_facing_score, london_relevance, remote_maturity, hiring_page_url, founder_or_team_signal, last_checked_at, metadata_json, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?17)
            ON CONFLICT(website_url) DO UPDATE SET
              company_name = excluded.company_name,
              source_family = excluded.source_family,
              market = excluded.market,
              stage = excluded.stage,
              funding_signal = excluded.funding_signal,
              product_category = excluded.product_category,
              technical_audience = excluded.technical_audience,
              developer_facing_score = excluded.developer_facing_score,
              london_relevance = excluded.london_relevance,
              remote_maturity = excluded.remote_maturity,
              hiring_page_url = excluded.hiring_page_url,
              founder_or_team_signal = excluded.founder_or_team_signal,
              last_checked_at = excluded.last_checked_at,
              metadata_json = excluded.metadata_json,
              updated_at = excluded.updated_at
            "#,
            params![
                id,
                input.company_name,
                input.website_url,
                input.source_family,
                input.market,
                input.stage,
                input.funding_signal,
                input.product_category,
                input.technical_audience,
                input.developer_facing_score,
                input.london_relevance,
                input.remote_maturity,
                input.hiring_page_url,
                input.founder_or_team_signal,
                timestamp,
                metadata_json,
                timestamp,
            ],
        )?;
        self.read_job_company_card(&id)?
            .with_context(|| format!("job company card not found: {id}"))
    }

    pub fn read_job_company_card(&self, id: &str) -> Result<Option<JobCompanyCard>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, company_name, website_url, source_family, market, stage, funding_signal, product_category, technical_audience, developer_facing_score, london_relevance, remote_maturity, hiring_page_url, founder_or_team_signal, last_checked_at, metadata_json, created_at, updated_at
                FROM job_company_cards
                WHERE id = ?1
                "#,
                params![id],
                job_company_card_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_job_company_cards(&self) -> Result<Vec<JobCompanyCard>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, company_name, website_url, source_family, market, stage, funding_signal, product_category, technical_audience, developer_facing_score, london_relevance, remote_maturity, hiring_page_url, founder_or_team_signal, last_checked_at, metadata_json, created_at, updated_at
            FROM job_company_cards
            ORDER BY developer_facing_score DESC, company_name ASC
            "#,
        )?;
        rows(stmt.query_map([], job_company_card_from_row)?)
    }

    pub fn record_job_contact(&self, input: JobContactInput) -> Result<JobContact> {
        let input = normalize_job_contact_input(input)?;
        if let Some(company_id) = &input.company_id {
            self.read_job_company_card(company_id)?
                .with_context(|| format!("job company card not found: {company_id}"))?;
        }
        let id = job_contact_id(&input.public_profile_url);
        let timestamp = now();
        self.conn.execute(
            r#"
            INSERT INTO job_contacts
              (id, name, company_id, role_title, public_profile_url, source_url, relationship_status, relevance, note, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)
            ON CONFLICT(public_profile_url) DO UPDATE SET
              name = excluded.name,
              company_id = excluded.company_id,
              role_title = excluded.role_title,
              source_url = excluded.source_url,
              relationship_status = excluded.relationship_status,
              relevance = excluded.relevance,
              note = excluded.note,
              updated_at = excluded.updated_at
            "#,
            params![
                id,
                input.name,
                input.company_id,
                input.role_title,
                input.public_profile_url,
                input.source_url,
                input.relationship_status,
                input.relevance,
                input.note,
                timestamp,
            ],
        )?;
        self.read_job_contact(&id)?
            .with_context(|| format!("job contact not found: {id}"))
    }

    pub fn read_job_contact(&self, id: &str) -> Result<Option<JobContact>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, name, company_id, role_title, public_profile_url, source_url, relationship_status, relevance, note, created_at, updated_at
                FROM job_contacts
                WHERE id = ?1
                "#,
                params![id],
                job_contact_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_job_contacts(&self) -> Result<Vec<JobContact>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, name, company_id, role_title, public_profile_url, source_url, relationship_status, relevance, note, created_at, updated_at
            FROM job_contacts
            ORDER BY updated_at DESC, name ASC, id ASC
            "#,
        )?;
        rows(stmt.query_map([], job_contact_from_row)?)
    }

    pub fn record_job_intro_path(&self, input: JobIntroPathInput) -> Result<JobIntroPath> {
        let input = normalize_job_intro_path_input(input)?;
        self.read_job_role_card(&input.role_id)?
            .with_context(|| format!("job role card not found: {}", input.role_id))?;
        let contact = self
            .read_job_contact(&input.contact_id)?
            .with_context(|| format!("job contact not found: {}", input.contact_id))?;
        if job_intro_claims_warm_path(&input)
            && !matches!(
                contact.relationship_status.as_str(),
                "known" | "possible_mutual"
            )
        {
            bail!("public-only contact discovery is not a warm intro path");
        }
        let id = job_intro_path_id(&input.role_id, &input.contact_id);
        let timestamp = now();
        self.conn.execute(
            r#"
            INSERT INTO job_intro_paths
              (id, role_id, contact_id, path_type, confidence, next_action, status, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)
            ON CONFLICT(role_id, contact_id) DO UPDATE SET
              path_type = excluded.path_type,
              confidence = excluded.confidence,
              next_action = excluded.next_action,
              status = excluded.status,
              updated_at = excluded.updated_at
            "#,
            params![
                id,
                input.role_id,
                input.contact_id,
                input.path_type,
                input.confidence,
                input.next_action,
                input.status,
                timestamp,
            ],
        )?;
        self.read_job_intro_path(&id)?
            .with_context(|| format!("job intro path not found: {id}"))
    }

    pub fn read_job_intro_path(&self, id: &str) -> Result<Option<JobIntroPath>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, role_id, contact_id, path_type, confidence, next_action, status, created_at, updated_at
                FROM job_intro_paths
                WHERE id = ?1
                "#,
                params![id],
                job_intro_path_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_job_intro_paths(&self) -> Result<Vec<JobIntroPath>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, role_id, contact_id, path_type, confidence, next_action, status, created_at, updated_at
            FROM job_intro_paths
            ORDER BY updated_at DESC, id ASC
            "#,
        )?;
        rows(stmt.query_map([], job_intro_path_from_row)?)
    }

    pub fn record_job_search_run(&self, input: JobSearchRunInput) -> Result<JobSearchRun> {
        let input = normalize_job_search_run_input(input)?;
        self.require_job_profile(&input.profile_id)?;
        let id = job_search_run_id();
        self.conn.execute(
            r#"
            INSERT INTO job_search_runs
              (id, profile_id, scope, started_at, completed_at, proof_level, source_count, role_count, new_role_count, stale_role_count, error_count, report_artifact_id)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
            params![
                id,
                input.profile_id,
                input.scope,
                now(),
                input.completed_at,
                input.proof_level,
                input.source_count as i64,
                input.role_count as i64,
                input.new_role_count as i64,
                input.stale_role_count as i64,
                input.error_count as i64,
                input.report_artifact_id,
            ],
        )?;
        self.read_job_search_run(&id)?
            .with_context(|| format!("job search run not found: {id}"))
    }

    pub fn read_job_search_run(&self, id: &str) -> Result<Option<JobSearchRun>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, profile_id, scope, started_at, completed_at, proof_level, source_count, role_count, new_role_count, stale_role_count, error_count, report_artifact_id
                FROM job_search_runs
                WHERE id = ?1
                "#,
                params![id],
                job_search_run_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub(crate) fn list_job_search_runs_for_scope(
        &self,
        profile_id: &str,
        scope: &str,
    ) -> Result<Vec<JobSearchRun>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, profile_id, scope, started_at, completed_at, proof_level, source_count, role_count, new_role_count, stale_role_count, error_count, report_artifact_id
            FROM job_search_runs
            WHERE profile_id = ?1 AND scope = ?2 AND completed_at IS NOT NULL
            ORDER BY started_at ASC, id ASC
            "#,
        )?;
        rows(stmt.query_map(params![profile_id, scope], job_search_run_from_row)?)
    }

    pub fn record_job_role_status_event(
        &self,
        input: JobRoleStatusEventInput,
    ) -> Result<JobRoleStatusEvent> {
        let input = normalize_job_role_status_event_input(input)?;
        self.read_job_role_card(&input.role_id)?
            .with_context(|| format!("job role card not found: {}", input.role_id))?;
        if let Some(run_id) = &input.run_id {
            self.read_job_search_run(run_id)?
                .with_context(|| format!("job search run not found: {run_id}"))?;
        }
        let id = job_role_status_event_id();
        self.conn.execute(
            r#"
            INSERT INTO job_role_status_events
              (id, role_id, run_id, status, previous_tier, current_tier, note, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                id,
                input.role_id,
                input.run_id,
                input.status,
                input.previous_tier,
                input.current_tier,
                input.note,
                now(),
            ],
        )?;
        self.read_job_role_status_event(&id)?
            .with_context(|| format!("job role status event not found: {id}"))
    }

    pub fn read_job_role_status_event(&self, id: &str) -> Result<Option<JobRoleStatusEvent>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, role_id, run_id, status, previous_tier, current_tier, note, created_at
                FROM job_role_status_events
                WHERE id = ?1
                "#,
                params![id],
                job_role_status_event_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub(crate) fn list_job_role_status_events_for_run(
        &self,
        run_id: &str,
    ) -> Result<Vec<JobRoleStatusEvent>> {
        validate_id(run_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, role_id, run_id, status, previous_tier, current_tier, note, created_at
            FROM job_role_status_events
            WHERE run_id = ?1
            ORDER BY created_at ASC, id ASC
            "#,
        )?;
        rows(stmt.query_map(params![run_id], job_role_status_event_from_row)?)
    }

    pub(crate) fn list_job_role_status_events_for_roles_recent(
        &self,
        role_ids: &BTreeSet<String>,
        limit: usize,
    ) -> Result<Vec<JobRoleStatusEvent>> {
        if role_ids.is_empty() {
            return Ok(Vec::new());
        }
        let limit = limit.clamp(1, 500) as i64;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, role_id, run_id, status, previous_tier, current_tier, note, created_at
            FROM job_role_status_events
            WHERE role_id = ?1
            ORDER BY created_at DESC, id ASC
            LIMIT ?2
            "#,
        )?;
        let mut events = Vec::new();
        for role_id in role_ids {
            validate_id(role_id)?;
            events.extend(rows(stmt.query_map(
                params![role_id, limit],
                job_role_status_event_from_row,
            )?)?);
        }
        events.sort_by(|left, right| {
            right
                .created_at
                .cmp(&left.created_at)
                .then_with(|| left.id.cmp(&right.id))
        });
        events.truncate(limit as usize);
        Ok(events)
    }

    pub fn record_job_application(&self, input: JobApplicationInput) -> Result<JobApplication> {
        let input = normalize_job_application_input(input)?;
        self.read_job_role_card(&input.role_id)?
            .with_context(|| format!("job role card not found: {}", input.role_id))?;
        if let Some(packet_id) = &input.packet_id {
            let packet = self
                .read_job_application_packet(packet_id)?
                .with_context(|| format!("job application packet not found: {packet_id}"))?;
            if packet.role_id != input.role_id {
                bail!("job application packet belongs to a different role");
            }
            if job_application_status_requires_approved_packet(&input.status)
                && packet.status != "approved"
            {
                bail!(
                    "job application status {} requires an approved application packet",
                    input.status
                );
            }
        }
        let id = job_application_id(&input.role_id);
        let timestamp = now();
        self.conn.execute(
            r#"
            INSERT INTO job_applications
              (id, role_id, packet_id, status, applied_at, follow_up_at, outcome_note, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)
            ON CONFLICT(id) DO UPDATE SET
              packet_id = excluded.packet_id,
              status = excluded.status,
              applied_at = excluded.applied_at,
              follow_up_at = excluded.follow_up_at,
              outcome_note = excluded.outcome_note,
              updated_at = excluded.updated_at
            "#,
            params![
                id,
                input.role_id,
                input.packet_id,
                input.status,
                input.applied_at,
                input.follow_up_at,
                input.outcome_note,
                timestamp,
            ],
        )?;
        self.read_job_application(&id)?
            .with_context(|| format!("job application not found: {id}"))
    }

    pub fn read_job_application(&self, id: &str) -> Result<Option<JobApplication>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, role_id, packet_id, status, applied_at, follow_up_at, outcome_note, created_at, updated_at
                FROM job_applications
                WHERE id = ?1
                "#,
                params![id],
                job_application_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn compile_job_weekly_report(
        &self,
        profile_id: &str,
        scope: &str,
    ) -> Result<JobWeeklyReport> {
        self.require_job_profile(profile_id)?;
        let scope = sanitize_required_job_text(scope, "scope", 500)?;
        let shortlist = self.compile_job_shortlist(profile_id)?;
        let scored_role_ids = shortlist
            .entries
            .iter()
            .filter(|entry| entry.score.is_some())
            .map(|entry| entry.role.id.clone())
            .collect::<BTreeSet<_>>();
        let applications = self
            .list_job_applications()?
            .into_iter()
            .filter(|application| scored_role_ids.contains(&application.role_id))
            .collect::<Vec<_>>();
        let health = self.list_job_source_health_recent(50)?;
        let intro_paths = self
            .list_job_intro_paths()?
            .into_iter()
            .filter(|path| scored_role_ids.contains(&path.role_id))
            .collect::<Vec<_>>();
        let contacts = self.list_job_contacts()?;
        let role_events =
            self.list_job_role_status_events_for_roles_recent(&scored_role_ids, 50)?;
        let body = render_job_weekly_report(
            &shortlist,
            &applications,
            &health,
            &intro_paths,
            &contacts,
            &role_events,
        );
        let id = job_weekly_report_id(profile_id, &scope, &body);
        let metadata = json!({
            "role_count": shortlist.entries.len(),
            "application_count": applications.len(),
            "intro_path_count": intro_paths.len(),
            "role_status_event_count": role_events.len(),
            "source_health_count": health.len(),
            "proof_level": "local_proof"
        });
        self.conn.execute(
            r#"
            INSERT INTO job_weekly_reports
              (id, profile_id, scope, generated_at, proof_level, body, metadata_json)
            VALUES (?1, ?2, ?3, ?4, 'local_proof', ?5, ?6)
            "#,
            params![
                id,
                profile_id,
                scope,
                now(),
                body,
                serde_json::to_string(&metadata)?,
            ],
        )?;
        self.read_job_weekly_report(&id)?
            .with_context(|| format!("job weekly report not found: {id}"))
    }

    pub fn read_job_weekly_report(&self, id: &str) -> Result<Option<JobWeeklyReport>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, profile_id, scope, generated_at, proof_level, body, metadata_json
                FROM job_weekly_reports
                WHERE id = ?1
                "#,
                params![id],
                job_weekly_report_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn prepare_job_weekly_report_delivery(
        &self,
        input: JobWeeklyReportDeliveryInput,
    ) -> Result<JobWeeklyReportDeliveryReport> {
        let input = normalize_job_weekly_report_delivery_input(input)?;
        let weekly_report = self
            .read_job_weekly_report(&input.report_id)?
            .with_context(|| format!("job weekly report not found: {}", input.report_id))?;
        let idempotency_key = input
            .idempotency_key
            .as_deref()
            .context("normalized job weekly report delivery missing idempotency key")?;
        let existing_delivery = self.find_job_weekly_report_delivery(
            &input.report_id,
            &input.channel,
            &input.subject,
            &input.target,
            idempotency_key,
        )?;
        if !self.channel_subject_can_send(&input.channel, &input.subject)? {
            if let Some(message_id) = existing_delivery
                .as_ref()
                .and_then(|delivery| delivery.channel_message_id.as_deref())
            {
                let _ = self.update_channel_message_status(message_id, "blocked");
            }
            let delivery = self.record_job_weekly_report_delivery_state(
                &input,
                "blocked",
                None,
                None,
                Some(format!(
                    "{} subject is not authorized to send: {}",
                    input.channel, input.subject
                )),
            )?;
            return self.hydrate_job_weekly_report_delivery_report(delivery, weekly_report, false);
        }

        self.conn
            .execute_batch("SAVEPOINT job_weekly_report_delivery_prepare")?;
        let prepared_result = (|| -> Result<JobWeeklyReportDeliveryReport> {
            let privacy_check = self.check_job_privacy_text(
                "job_weekly_report_delivery",
                Some(&weekly_report.id),
                &weekly_report.body,
                &[],
            )?;
            if privacy_check.decision == "block" {
                if let Some(message_id) = existing_delivery
                    .as_ref()
                    .and_then(|delivery| delivery.channel_message_id.as_deref())
                {
                    let _ = self.update_channel_message_status(message_id, "blocked");
                }
                let delivery = self.record_job_weekly_report_delivery_state(
                    &input,
                    "blocked",
                    Some(&privacy_check.id),
                    None,
                    Some("job weekly report delivery privacy check blocked".to_string()),
                )?;
                return Ok(JobWeeklyReportDeliveryReport {
                    delivery,
                    weekly_report: weekly_report.clone(),
                    privacy_check: Some(privacy_check),
                    channel_message: None,
                    idempotent_replay: false,
                });
            }

            if let Some(existing) = existing_delivery.as_ref() {
                if existing.status == "prepared" {
                    let delivery = self.record_job_weekly_report_delivery_state(
                        &input,
                        "prepared",
                        Some(&privacy_check.id),
                        existing.channel_message_id.as_deref(),
                        None,
                    )?;
                    return self.hydrate_job_weekly_report_delivery_report(
                        delivery,
                        weekly_report.clone(),
                        true,
                    );
                }
            }

            let message = self.record_channel_message_with_status(
                &input.channel,
                "outgoing",
                &input.target,
                &weekly_report.body,
                "prepared",
                None,
                Some(&weekly_report.id),
            )?;
            let delivery = self.record_job_weekly_report_delivery_state(
                &input,
                "prepared",
                Some(&privacy_check.id),
                Some(&message.id),
                None,
            )?;
            Ok(JobWeeklyReportDeliveryReport {
                delivery,
                weekly_report: weekly_report.clone(),
                privacy_check: Some(privacy_check),
                channel_message: Some(message),
                idempotent_replay: false,
            })
        })();
        match prepared_result {
            Ok(report) => {
                self.conn
                    .execute_batch("RELEASE SAVEPOINT job_weekly_report_delivery_prepare")?;
                Ok(report)
            }
            Err(error) => {
                let _ = self.conn.execute_batch(
                    "ROLLBACK TO SAVEPOINT job_weekly_report_delivery_prepare;\
                     RELEASE SAVEPOINT job_weekly_report_delivery_prepare;",
                );
                Err(error)
            }
        }
    }

    pub fn send_job_weekly_report_delivery(
        &self,
        input: JobWeeklyReportDeliverySendInput,
    ) -> Result<JobWeeklyReportDeliverySendReport> {
        let input = normalize_job_weekly_report_delivery_send_input(input)?;
        let delivery = self
            .read_job_weekly_report_delivery(&input.delivery_id)?
            .with_context(|| {
                format!(
                    "job weekly report delivery not found: {}",
                    input.delivery_id
                )
            })?;
        let weekly_report = self
            .read_job_weekly_report(&delivery.report_id)?
            .with_context(|| format!("job weekly report not found: {}", delivery.report_id))?;
        let message_id = delivery
            .channel_message_id
            .as_deref()
            .context("job weekly report delivery has no prepared channel message")?;
        if delivery.status == "blocked" {
            return self.hydrate_job_weekly_report_delivery_send_report(
                delivery,
                weekly_report,
                None,
                true,
            );
        }

        if let Some(successful_attempt) = self
            .list_channel_delivery_attempts(Some(message_id))?
            .into_iter()
            .find(|attempt| attempt.ok)
        {
            return self.hydrate_job_weekly_report_delivery_send_report(
                delivery,
                weekly_report,
                Some(successful_attempt),
                true,
            );
        }

        let delivery_input = JobWeeklyReportDeliveryInput {
            report_id: delivery.report_id.clone(),
            channel: delivery.channel.clone(),
            subject: delivery.subject.clone(),
            target: delivery.target.clone(),
            idempotency_key: Some(delivery.idempotency_key.clone()),
        };

        if !self.channel_subject_can_send(&delivery.channel, &delivery.subject)? {
            let _ = self.update_channel_message_status(message_id, "blocked");
            let blocked = self.record_job_weekly_report_delivery_state(
                &delivery_input,
                "blocked",
                delivery.privacy_check_id.as_deref(),
                Some(message_id),
                Some(format!(
                    "{} subject is not authorized to send: {}",
                    delivery.channel, delivery.subject
                )),
            )?;
            return self.hydrate_job_weekly_report_delivery_send_report(
                blocked,
                weekly_report,
                None,
                false,
            );
        }

        let privacy_check = self.check_job_privacy_text(
            "job_weekly_report_provider_delivery",
            Some(&weekly_report.id),
            &weekly_report.body,
            &[],
        )?;
        if privacy_check.decision == "block" {
            let _ = self.update_channel_message_status(message_id, "blocked");
            let blocked = self.record_job_weekly_report_delivery_state(
                &delivery_input,
                "blocked",
                Some(&privacy_check.id),
                Some(message_id),
                Some("job weekly report provider delivery privacy check blocked".to_string()),
            )?;
            return self.hydrate_job_weekly_report_delivery_send_report(
                blocked,
                weekly_report,
                None,
                false,
            );
        }

        let send_result: Result<(bool, u16, ChannelMessage, ChannelDeliveryAttempt)> =
            (|| match delivery.channel.as_str() {
                "telegram" => {
                    let token = input.telegram_bot_token.as_deref().context(
                        "telegram_bot_token is required for telegram weekly report delivery",
                    )?;
                    let chat_id = delivery
                        .target
                        .strip_prefix("telegram:chat:")
                        .unwrap_or(&delivery.target);
                    self.policy_guard(PolicyRequest {
                        action: "channel.send".to_string(),
                        package: Some("arcwell-job-hunting".to_string()),
                        provider: Some("telegram".to_string()),
                        source: Some("job_weekly_report_delivery".to_string()),
                        channel: Some("telegram".to_string()),
                        subject: Some(delivery.subject.clone()),
                        target: Some(chat_id.to_string()),
                        projected_usd: None,
                        metadata: json!({
                            "delivery_id": delivery.id,
                            "report_id": weekly_report.id,
                            "parse_mode": "MarkdownV2",
                        }),
                        untrusted_excerpt: Some(weekly_report.body.clone()),
                    })?;
                    self.require_cost_budget(
                        "arcwell-job-hunting",
                        &delivery.id,
                        "telegram",
                        "send_message",
                        Some("job_weekly_report_delivery"),
                        estimated_channel_send_cost(),
                        "Job weekly report Telegram delivery",
                    )?;
                    let report = self.send_existing_telegram_message_preflighted(
                        message_id,
                        token,
                        chat_id,
                        &weekly_report.body,
                        input.api_base.as_deref(),
                    )?;
                    Ok((report.ok, report.status, report.message, report.delivery))
                }
                "email" => {
                    let account_id = input
                        .email_account_id
                        .as_deref()
                        .context("email_account_id is required for email weekly report delivery")?;
                    let api_token = input
                        .email_api_token
                        .as_deref()
                        .context("email_api_token is required for email weekly report delivery")?;
                    let from = input
                        .email_from
                        .as_deref()
                        .context("email_from is required for email weekly report delivery")?;
                    let to = delivery
                        .target
                        .strip_prefix("email:")
                        .unwrap_or(&delivery.target);
                    let subject = format!("Arcwell job weekly report: {}", weekly_report.scope);
                    let report = self.send_existing_cloudflare_email_message_with_context(
                        account_id,
                        api_token,
                        from,
                        message_id,
                        to,
                        &subject,
                        &weekly_report.body,
                        input.api_base.as_deref(),
                        "job_weekly_report_delivery",
                        "Job weekly report email delivery",
                        json!({
                            "delivery_id": delivery.id,
                            "report_id": weekly_report.id,
                            "scope": weekly_report.scope,
                        }),
                    )?;
                    Ok((report.ok, report.status, report.message, report.delivery))
                }
                _ => bail!(
                    "unsupported job weekly report delivery channel: {}",
                    delivery.channel
                ),
            })();

        match send_result {
            Ok((ok, status, _message, attempt)) => {
                let delivery = self.record_job_weekly_report_delivery_state(
                    &delivery_input,
                    if ok { "sent" } else { "failed" },
                    Some(&privacy_check.id),
                    Some(message_id),
                    if ok {
                        None
                    } else {
                        Some(format!(
                            "provider send failed with status {}{}",
                            status,
                            attempt
                                .error
                                .as_deref()
                                .map(|error| format!(": {error}"))
                                .unwrap_or_default()
                        ))
                    },
                )?;
                self.hydrate_job_weekly_report_delivery_send_report(
                    delivery,
                    weekly_report,
                    Some(attempt),
                    false,
                )
            }
            Err(error) => {
                let _ = self.update_channel_message_status(message_id, "blocked");
                let blocked = self.record_job_weekly_report_delivery_state(
                    &delivery_input,
                    "blocked",
                    Some(&privacy_check.id),
                    Some(message_id),
                    Some(error.to_string()),
                )?;
                self.hydrate_job_weekly_report_delivery_send_report(
                    blocked,
                    weekly_report,
                    None,
                    false,
                )
            }
        }
    }

    pub fn read_job_weekly_report_delivery(
        &self,
        id: &str,
    ) -> Result<Option<JobWeeklyReportDelivery>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, report_id, channel, subject, target, status, privacy_check_id, channel_message_id, idempotency_key, error, created_at, updated_at
                FROM job_weekly_report_deliveries
                WHERE id = ?1
                "#,
                params![id],
                job_weekly_report_delivery_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_job_weekly_report_deliveries(
        &self,
        report_id: Option<&str>,
    ) -> Result<Vec<JobWeeklyReportDelivery>> {
        if let Some(report_id) = report_id {
            validate_id(report_id)?;
            let mut stmt = self.conn.prepare(
                r#"
                SELECT id, report_id, channel, subject, target, status, privacy_check_id, channel_message_id, idempotency_key, error, created_at, updated_at
                FROM job_weekly_report_deliveries
                WHERE report_id = ?1
                ORDER BY updated_at DESC, id ASC
                "#,
            )?;
            return rows(stmt.query_map(params![report_id], job_weekly_report_delivery_from_row)?);
        }
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, report_id, channel, subject, target, status, privacy_check_id, channel_message_id, idempotency_key, error, created_at, updated_at
            FROM job_weekly_report_deliveries
            ORDER BY updated_at DESC, id ASC
            "#,
        )?;
        rows(stmt.query_map([], job_weekly_report_delivery_from_row)?)
    }

    fn hydrate_job_weekly_report_delivery_report(
        &self,
        delivery: JobWeeklyReportDelivery,
        weekly_report: JobWeeklyReport,
        idempotent_replay: bool,
    ) -> Result<JobWeeklyReportDeliveryReport> {
        let privacy_check = delivery
            .privacy_check_id
            .as_deref()
            .map(|id| {
                self.read_job_privacy_check(id)?
                    .with_context(|| format!("job privacy check not found: {id}"))
            })
            .transpose()?;
        let channel_message = delivery
            .channel_message_id
            .as_deref()
            .map(|id| {
                self.get_channel_message(id)?
                    .with_context(|| format!("channel message not found: {id}"))
            })
            .transpose()?;
        Ok(JobWeeklyReportDeliveryReport {
            delivery,
            weekly_report,
            privacy_check,
            channel_message,
            idempotent_replay,
        })
    }

    fn hydrate_job_weekly_report_delivery_send_report(
        &self,
        delivery: JobWeeklyReportDelivery,
        weekly_report: JobWeeklyReport,
        channel_delivery_attempt: Option<ChannelDeliveryAttempt>,
        idempotent_replay: bool,
    ) -> Result<JobWeeklyReportDeliverySendReport> {
        let privacy_check = delivery
            .privacy_check_id
            .as_deref()
            .map(|id| {
                self.read_job_privacy_check(id)?
                    .with_context(|| format!("job privacy check not found: {id}"))
            })
            .transpose()?;
        let channel_message = delivery
            .channel_message_id
            .as_deref()
            .map(|id| {
                self.get_channel_message(id)?
                    .with_context(|| format!("channel message not found: {id}"))
            })
            .transpose()?;
        Ok(JobWeeklyReportDeliverySendReport {
            delivery,
            weekly_report,
            privacy_check,
            channel_message,
            channel_delivery_attempt,
            idempotent_replay,
            proof_level: "controlled_provider_delivery".to_string(),
            non_claims: vec![
                "This is not proof of wall-clock recurrence or unattended scheduling.".to_string(),
                "A mock or local API base proves the provider path shape, not live external delivery."
                    .to_string(),
                "This does not submit applications or contact employers.".to_string(),
            ],
        })
    }

    fn find_job_weekly_report_delivery(
        &self,
        report_id: &str,
        channel: &str,
        subject: &str,
        target: &str,
        idempotency_key: &str,
    ) -> Result<Option<JobWeeklyReportDelivery>> {
        validate_id(report_id)?;
        validate_key(channel)?;
        validate_query(subject)?;
        validate_query(target)?;
        validate_query(idempotency_key)?;
        self.conn
            .query_row(
                r#"
                SELECT id, report_id, channel, subject, target, status, privacy_check_id, channel_message_id, idempotency_key, error, created_at, updated_at
                FROM job_weekly_report_deliveries
                WHERE report_id = ?1
                  AND channel = ?2
                  AND subject = ?3
                  AND target = ?4
                  AND idempotency_key = ?5
                "#,
                params![report_id, channel, subject, target, idempotency_key],
                job_weekly_report_delivery_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    fn record_job_weekly_report_delivery_state(
        &self,
        input: &JobWeeklyReportDeliveryInput,
        status: &str,
        privacy_check_id: Option<&str>,
        channel_message_id: Option<&str>,
        error: Option<String>,
    ) -> Result<JobWeeklyReportDelivery> {
        let status = normalize_job_weekly_report_delivery_status(status)?;
        let idempotency_key = input
            .idempotency_key
            .as_deref()
            .context("normalized job weekly report delivery missing idempotency key")?;
        if let Some(privacy_check_id) = privacy_check_id {
            validate_id(privacy_check_id)?;
        }
        if let Some(channel_message_id) = channel_message_id {
            validate_id(channel_message_id)?;
        }
        let error = error
            .as_deref()
            .map(|value| sanitize_required_job_text(value, "delivery error", 1_000))
            .transpose()?;
        let timestamp = now();
        self.conn.execute(
            r#"
            INSERT INTO job_weekly_report_deliveries
              (id, report_id, channel, subject, target, status, privacy_check_id, channel_message_id, idempotency_key, error, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)
            ON CONFLICT(report_id, channel, subject, target, idempotency_key) DO UPDATE SET
              status = excluded.status,
              privacy_check_id = excluded.privacy_check_id,
              channel_message_id = excluded.channel_message_id,
              error = excluded.error,
              updated_at = excluded.updated_at
            "#,
            params![
                job_weekly_report_delivery_id(),
                input.report_id,
                input.channel,
                input.subject,
                input.target,
                status,
                privacy_check_id,
                channel_message_id,
                idempotency_key,
                error,
                timestamp,
            ],
        )?;
        self.find_job_weekly_report_delivery(
            &input.report_id,
            &input.channel,
            &input.subject,
            &input.target,
            idempotency_key,
        )?
        .with_context(|| {
            format!(
                "job weekly report delivery not found after upsert for report {}",
                input.report_id
            )
        })
    }
}
