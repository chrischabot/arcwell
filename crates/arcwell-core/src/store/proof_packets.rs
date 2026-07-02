use super::*;

const PROOF_PACKET_MAX_ITEMS: usize = 500;
const PROOF_PACKET_MAX_TEXT: usize = 20_000;
const PROOF_PACKET_MAX_OUTPUT: usize = 8_000;
const PROOF_PACKET_VERIFY_MAX_BYTES: u64 = 5_000_000;

impl Store {
    pub fn record_proof_packet(&self, input: ProofPacketInput) -> Result<ProofPacketReport> {
        validate_proof_packet_input(&input)?;
        let packet_id = format!(
            "proof-{}",
            &sha256(
                format!(
                    "{}\n{}\n{}\n{}",
                    input.scope,
                    input.title,
                    input.status,
                    Uuid::new_v4()
                )
                .as_bytes()
            )[..24]
        );
        let created_at = now();
        let promoted_at = (input.status == "promoted").then(|| created_at.clone());

        let record_result = (|| -> Result<()> {
            self.conn.execute("BEGIN IMMEDIATE", [])?;
            self.conn.execute(
                r#"
                INSERT INTO proof_packets
                  (id, scope, title, proof_level, status, summary, artifact_root,
                   reviewer, metadata_json, created_at, promoted_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                "#,
                params![
                    packet_id,
                    input.scope,
                    input.title,
                    input.proof_level,
                    input.status,
                    input.summary,
                    input.artifact_root,
                    input.reviewer,
                    canonical_json(&input.metadata)?,
                    created_at,
                    promoted_at,
                ],
            )?;
            for claim in &input.claims {
                let id = proof_child_id("proof-claim", &packet_id, &claim.claim_key);
                self.conn.execute(
                    r#"
                    INSERT INTO proof_claims
                      (id, packet_id, claim_key, claim, status, proof_level,
                       evidence_json, refutation_json, gates_json, created_at)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                    "#,
                    params![
                        id,
                        packet_id,
                        claim.claim_key,
                        claim.claim,
                        claim.status,
                        claim.proof_level,
                        canonical_json(&claim.evidence)?,
                        canonical_json(&claim.refutation)?,
                        canonical_json(&claim.gates)?,
                        created_at,
                    ],
                )?;
            }
            for (index, artifact) in input.artifacts.iter().enumerate() {
                let id = proof_child_id(
                    "proof-artifact",
                    &packet_id,
                    &format!("{}-{index}", artifact.label),
                );
                self.conn.execute(
                    r#"
                    INSERT INTO proof_artifacts
                      (id, packet_id, artifact_kind, label, path, sha256, metadata_json, created_at)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                    "#,
                    params![
                        id,
                        packet_id,
                        artifact.artifact_kind,
                        artifact.label,
                        artifact.path,
                        artifact.sha256,
                        canonical_json(&artifact.metadata)?,
                        created_at,
                    ],
                )?;
            }
            for (index, check) in input.checks.iter().enumerate() {
                let id = proof_child_id(
                    "proof-check",
                    &packet_id,
                    &format!("{}-{index}", check.command),
                );
                self.conn.execute(
                    r#"
                    INSERT INTO proof_checks
                      (id, packet_id, check_kind, command, status, exit_code, duration_ms,
                       output_excerpt, metadata_json, created_at)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                    "#,
                    params![
                        id,
                        packet_id,
                        check.check_kind,
                        check.command,
                        check.status,
                        check.exit_code,
                        check.duration_ms,
                        check.output_excerpt,
                        canonical_json(&check.metadata)?,
                        created_at,
                    ],
                )?;
            }
            self.conn.execute("COMMIT", [])?;
            Ok(())
        })();
        if let Err(error) = record_result {
            let _ = self.conn.execute("ROLLBACK", []);
            return Err(error);
        }

        self.read_proof_packet(&packet_id)?
            .with_context(|| format!("created proof packet not found: {packet_id}"))
    }

    pub fn read_proof_packet(&self, packet_id: &str) -> Result<Option<ProofPacketReport>> {
        validate_id(packet_id)?;
        let packet = self
            .conn
            .query_row(
                r#"
                SELECT id, scope, title, proof_level, status, summary, artifact_root,
                       reviewer, metadata_json, created_at, promoted_at
                FROM proof_packets
                WHERE id = ?1
                "#,
                params![packet_id],
                proof_packet_from_row,
            )
            .optional()?;
        let Some(packet) = packet else {
            return Ok(None);
        };
        let claims = self.list_proof_claims(packet_id)?;
        let artifacts = self.list_proof_artifacts(packet_id)?;
        let checks = self.list_proof_checks(packet_id)?;
        let judgment = judge_proof_packet(&packet, &claims, &artifacts, &checks);
        Ok(Some(ProofPacketReport {
            packet,
            claims,
            artifacts,
            checks,
            judgment,
            non_claims: proof_packet_non_claims(),
        }))
    }

    pub fn list_proof_packets(
        &self,
        scope: Option<&str>,
        limit: usize,
    ) -> Result<Vec<ProofPacketSummary>> {
        let limit = limit.clamp(1, 500);
        if let Some(scope) = scope {
            validate_key(scope)?;
            let mut stmt = self.conn.prepare(
                r#"
                SELECT p.id, p.scope, p.title, p.proof_level, p.status,
                       COUNT(DISTINCT c.id) AS claim_count,
                       SUM(CASE WHEN ck.status = 'passed' THEN 1 ELSE 0 END) AS passed_checks,
                       SUM(CASE WHEN c.status IN ('partial', 'blocked', 'refuted') THEN 1 ELSE 0 END) AS blocker_count,
                       p.created_at, p.promoted_at
                FROM proof_packets p
                LEFT JOIN proof_claims c ON c.packet_id = p.id
                LEFT JOIN proof_checks ck ON ck.packet_id = p.id
                WHERE p.scope = ?1
                GROUP BY p.id
                ORDER BY p.created_at DESC
                LIMIT ?2
                "#,
            )?;
            return rows(stmt.query_map(params![scope, limit], proof_packet_summary_from_row)?);
        }
        let mut stmt = self.conn.prepare(
            r#"
            SELECT p.id, p.scope, p.title, p.proof_level, p.status,
                   COUNT(DISTINCT c.id) AS claim_count,
                   SUM(CASE WHEN ck.status = 'passed' THEN 1 ELSE 0 END) AS passed_checks,
                   SUM(CASE WHEN c.status IN ('partial', 'blocked', 'refuted') THEN 1 ELSE 0 END) AS blocker_count,
                   p.created_at, p.promoted_at
            FROM proof_packets p
            LEFT JOIN proof_claims c ON c.packet_id = p.id
            LEFT JOIN proof_checks ck ON ck.packet_id = p.id
            GROUP BY p.id
            ORDER BY p.created_at DESC
            LIMIT ?1
            "#,
        )?;
        rows(stmt.query_map(params![limit], proof_packet_summary_from_row)?)
    }

    pub fn latest_proof_packet(&self, capability: &str) -> Result<Option<ProofPacketReport>> {
        validate_key(capability)?;
        let packet_id = self
            .conn
            .query_row(
                r#"
                SELECT id
                FROM proof_packets
                WHERE scope = ?1
                   OR json_extract(metadata_json, '$.capability') = ?1
                   OR json_extract(metadata_json, '$.proof_name') = ?1
                ORDER BY created_at DESC, id DESC
                LIMIT 1
                "#,
                params![capability],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        let Some(packet_id) = packet_id else {
            return Ok(None);
        };
        self.read_proof_packet(&packet_id)
    }

    pub fn verify_proof_packet_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<ProofPacketVerificationReport> {
        let path = path.as_ref();
        let path_text = path.to_string_lossy().to_string();
        let mut blockers = Vec::new();
        let mut warnings = Vec::new();
        let mut redaction_findings = Vec::new();
        if !path.exists() {
            return Ok(ProofPacketVerificationReport {
                path: path_text,
                packet_id: None,
                proof_name: None,
                proof_level: None,
                ok: false,
                blockers: vec!["proof packet file does not exist".to_string()],
                warnings,
                checked_artifacts: Vec::new(),
                redaction_findings,
            });
        }
        let packet_bytes = fs::read(path).with_context(|| {
            format!(
                "reading proof packet file for verification: {}",
                path.display()
            )
        })?;
        if packet_bytes.len() as u64 > PROOF_PACKET_VERIFY_MAX_BYTES {
            blockers.push(format!(
                "proof packet file is too large to verify safely: {} bytes",
                packet_bytes.len()
            ));
        }
        let packet_text = String::from_utf8_lossy(&packet_bytes);
        redaction_findings.extend(scan_proof_text_for_secret_like_values(
            &format!("proof_packet:{}", path.display()),
            &packet_text,
        ));
        let packet_json: Value =
            serde_json::from_slice(&packet_bytes).context("proof packet file is not valid JSON")?;
        blockers.extend(semantic_proof_packet_blockers(&packet_json, &packet_text));
        let packet_id = packet_json
            .pointer("/packet/id")
            .and_then(Value::as_str)
            .or_else(|| packet_json.get("id").and_then(Value::as_str))
            .map(ToString::to_string);
        let proof_name = packet_json
            .get("proof_name")
            .and_then(Value::as_str)
            .or_else(|| packet_json.get("name").and_then(Value::as_str))
            .map(ToString::to_string);
        let proof_level = packet_json
            .pointer("/packet/proof_level")
            .and_then(Value::as_str)
            .or_else(|| packet_json.get("proof_level").and_then(Value::as_str))
            .map(ToString::to_string);
        let artifact_root = packet_json
            .pointer("/packet/artifact_root")
            .and_then(Value::as_str)
            .or_else(|| packet_json.get("artifact_root").and_then(Value::as_str))
            .or_else(|| packet_json.get("proof_root").and_then(Value::as_str))
            .map(ToString::to_string);
        let artifact_specs = extract_proof_artifact_specs(&packet_json);
        if artifact_specs.is_empty() {
            blockers.push("proof packet has no verifiable artifacts".to_string());
        }
        let mut checked_artifacts = Vec::new();
        for spec in artifact_specs {
            let checked = verify_proof_artifact_spec(path, artifact_root.as_deref(), spec)?;
            for finding in &checked.redaction_findings {
                redaction_findings.push(finding.clone());
            }
            if !checked.warnings.is_empty() {
                warnings.extend(
                    checked
                        .warnings
                        .iter()
                        .map(|warning| format!("{}: {warning}", checked.label)),
                );
            }
            if !checked.blockers.is_empty() {
                blockers.extend(
                    checked
                        .blockers
                        .iter()
                        .map(|blocker| format!("{}: {blocker}", checked.label)),
                );
            }
            checked_artifacts.push(ProofArtifactVerification {
                label: checked.label,
                path: checked.path,
                resolved_path: checked.resolved_path,
                exists: checked.exists,
                sha256_expected: checked.sha256_expected,
                sha256_actual: checked.sha256_actual,
                sha256_matches: checked.sha256_matches,
                redaction_findings: checked.redaction_findings.len(),
                warnings: checked.warnings,
                blockers: checked.blockers,
            });
        }
        if !redaction_findings.is_empty() {
            blockers.push(format!(
                "{} secret-like or email-like redaction findings detected",
                redaction_findings.len()
            ));
        }
        Ok(ProofPacketVerificationReport {
            path: path_text,
            packet_id,
            proof_name,
            proof_level,
            ok: blockers.is_empty(),
            blockers,
            warnings,
            checked_artifacts,
            redaction_findings,
        })
    }

    pub fn promote_proof_packet(
        &self,
        packet_id: &str,
        reviewer: &str,
    ) -> Result<ProofPacketReport> {
        validate_id(packet_id)?;
        validate_required_text("reviewer", reviewer, 200)?;
        let report = self
            .read_proof_packet(packet_id)?
            .with_context(|| format!("proof packet not found: {packet_id}"))?;
        if !report.judgment.promotable {
            bail!(
                "proof packet is not promotable: {}",
                report.judgment.blockers.join("; ")
            );
        }
        let promoted_at = now();
        self.conn.execute(
            "UPDATE proof_packets SET status = 'promoted', reviewer = ?2, promoted_at = ?3 WHERE id = ?1",
            params![packet_id, reviewer, promoted_at],
        )?;
        self.read_proof_packet(packet_id)?
            .with_context(|| format!("promoted proof packet not found: {packet_id}"))
    }

    pub fn record_adversarial_review(
        &self,
        input: AdversarialReviewRunInput,
    ) -> Result<AdversarialReviewReport> {
        validate_adversarial_review_input(&input)?;
        if let Some(packet_id) = &input.packet_id {
            self.read_proof_packet(packet_id)?.with_context(|| {
                format!("proof packet not found for adversarial review: {packet_id}")
            })?;
        }
        let review_id = format!(
            "arev-{}",
            &sha256(
                format!(
                    "{}\n{}\n{}\n{}",
                    input.scope,
                    input.title,
                    input.judgment,
                    Uuid::new_v4()
                )
                .as_bytes()
            )[..24]
        );
        let created_at = now();
        let record_result = (|| -> Result<()> {
            self.conn.execute("BEGIN IMMEDIATE", [])?;
            self.conn.execute(
                r#"
                INSERT INTO adversarial_review_runs
                  (id, packet_id, scope, title, reviewer, requested_proof_level,
                   judgment, summary, strongest_fake_done_path, refutations_json,
                   skipped_categories_json, metadata_json, created_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                "#,
                params![
                    review_id,
                    input.packet_id,
                    input.scope,
                    input.title,
                    input.reviewer,
                    input.requested_proof_level,
                    input.judgment,
                    input.summary,
                    input.strongest_fake_done_path,
                    canonical_json(&input.refutations)?,
                    canonical_json(&input.skipped_categories)?,
                    canonical_json(&input.metadata)?,
                    created_at,
                ],
            )?;
            for (index, finding) in input.findings.iter().enumerate() {
                let id = proof_child_id(
                    "arev-finding",
                    &review_id,
                    &format!("{}-{index}", finding.title),
                );
                self.conn.execute(
                    r#"
                    INSERT INTO adversarial_review_findings
                      (id, review_id, severity, status, title, body, evidence_json,
                       recommendation, created_at)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                    "#,
                    params![
                        id,
                        review_id,
                        finding.severity,
                        finding.status,
                        finding.title,
                        finding.body,
                        canonical_json(&finding.evidence)?,
                        finding.recommendation,
                        created_at,
                    ],
                )?;
            }
            self.conn.execute("COMMIT", [])?;
            Ok(())
        })();
        if let Err(error) = record_result {
            let _ = self.conn.execute("ROLLBACK", []);
            return Err(error);
        }
        self.read_adversarial_review(&review_id)?
            .with_context(|| format!("created adversarial review not found: {review_id}"))
    }

    pub fn read_adversarial_review(
        &self,
        review_id: &str,
    ) -> Result<Option<AdversarialReviewReport>> {
        validate_id(review_id)?;
        let review = self
            .conn
            .query_row(
                r#"
                SELECT id, packet_id, scope, title, reviewer, requested_proof_level,
                       judgment, summary, strongest_fake_done_path, refutations_json,
                       skipped_categories_json, metadata_json, created_at
                FROM adversarial_review_runs
                WHERE id = ?1
                "#,
                params![review_id],
                adversarial_review_run_from_row,
            )
            .optional()?;
        let Some(review) = review else {
            return Ok(None);
        };
        let findings = self.list_adversarial_review_findings(review_id)?;
        Ok(Some(AdversarialReviewReport {
            review,
            findings,
            non_claims: adversarial_review_non_claims(),
        }))
    }

    pub fn list_adversarial_reviews(
        &self,
        scope: Option<&str>,
        packet_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<AdversarialReviewSummary>> {
        let limit = limit.clamp(1, 500);
        if let Some(scope) = scope {
            validate_key(scope)?;
        }
        if let Some(packet_id) = packet_id {
            validate_id(packet_id)?;
        }
        let mut stmt = self.conn.prepare(
            r#"
            SELECT r.id, r.packet_id, r.scope, r.title, r.reviewer,
                   r.requested_proof_level, r.judgment,
                   COUNT(f.id) AS finding_count,
                   SUM(CASE WHEN f.status = 'blocking' THEN 1 ELSE 0 END) AS blocking_finding_count,
                   r.created_at
            FROM adversarial_review_runs r
            LEFT JOIN adversarial_review_findings f ON f.review_id = r.id
            WHERE (?1 IS NULL OR r.scope = ?1)
              AND (?2 IS NULL OR r.packet_id = ?2)
            GROUP BY r.id
            ORDER BY r.created_at DESC, r.id DESC
            LIMIT ?3
            "#,
        )?;
        rows(stmt.query_map(
            params![scope, packet_id, limit],
            adversarial_review_summary_from_row,
        )?)
    }

    fn list_proof_claims(&self, packet_id: &str) -> Result<Vec<ProofClaim>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, packet_id, claim_key, claim, status, proof_level,
                   evidence_json, refutation_json, gates_json, created_at
            FROM proof_claims
            WHERE packet_id = ?1
            ORDER BY claim_key
            "#,
        )?;
        rows(stmt.query_map(params![packet_id], proof_claim_from_row)?)
    }

    fn list_proof_artifacts(&self, packet_id: &str) -> Result<Vec<ProofArtifact>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, packet_id, artifact_kind, label, path, sha256, metadata_json, created_at
            FROM proof_artifacts
            WHERE packet_id = ?1
            ORDER BY artifact_kind, label
            "#,
        )?;
        rows(stmt.query_map(params![packet_id], proof_artifact_from_row)?)
    }

    fn list_proof_checks(&self, packet_id: &str) -> Result<Vec<ProofCheck>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, packet_id, check_kind, command, status, exit_code,
                   duration_ms, output_excerpt, metadata_json, created_at
            FROM proof_checks
            WHERE packet_id = ?1
            ORDER BY check_kind, command
            "#,
        )?;
        rows(stmt.query_map(params![packet_id], proof_check_from_row)?)
    }

    fn list_adversarial_review_findings(
        &self,
        review_id: &str,
    ) -> Result<Vec<AdversarialReviewFinding>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, review_id, severity, status, title, body, evidence_json,
                   recommendation, created_at
            FROM adversarial_review_findings
            WHERE review_id = ?1
            ORDER BY severity DESC, title
            "#,
        )?;
        rows(stmt.query_map(params![review_id], adversarial_review_finding_from_row)?)
    }
}

fn validate_proof_packet_input(input: &ProofPacketInput) -> Result<()> {
    validate_required_text("scope", &input.scope, 200)?;
    validate_required_text("title", &input.title, 300)?;
    validate_required_text("proof_level", &input.proof_level, 120)?;
    validate_proof_packet_status(&input.status)?;
    validate_required_text("summary", &input.summary, PROOF_PACKET_MAX_TEXT)?;
    if let Some(root) = &input.artifact_root {
        validate_optional_pathish("artifact_root", root)?;
    }
    if let Some(reviewer) = &input.reviewer {
        validate_required_text("reviewer", reviewer, 200)?;
    }
    validate_json_size("metadata", &input.metadata)?;
    if input.claims.len() > PROOF_PACKET_MAX_ITEMS {
        bail!("too many proof claims");
    }
    if input.artifacts.len() > PROOF_PACKET_MAX_ITEMS {
        bail!("too many proof artifacts");
    }
    if input.checks.len() > PROOF_PACKET_MAX_ITEMS {
        bail!("too many proof checks");
    }
    let mut claim_keys = BTreeSet::new();
    for claim in &input.claims {
        validate_required_text("claim_key", &claim.claim_key, 200)?;
        if !claim_keys.insert(claim.claim_key.clone()) {
            bail!("duplicate proof claim key: {}", claim.claim_key);
        }
        validate_required_text("claim", &claim.claim, PROOF_PACKET_MAX_TEXT)?;
        validate_proof_claim_status(&claim.status)?;
        validate_required_text("claim.proof_level", &claim.proof_level, 120)?;
        validate_json_size("claim.evidence", &claim.evidence)?;
        validate_json_size("claim.refutation", &claim.refutation)?;
        validate_json_size("claim.gates", &claim.gates)?;
    }
    for artifact in &input.artifacts {
        validate_required_text("artifact_kind", &artifact.artifact_kind, 120)?;
        validate_required_text("artifact.label", &artifact.label, 300)?;
        if let Some(path) = &artifact.path {
            validate_optional_pathish("artifact.path", path)?;
        }
        if let Some(hash) = &artifact.sha256 {
            validate_sha256_hex(hash)?;
        }
        validate_json_size("artifact.metadata", &artifact.metadata)?;
    }
    for check in &input.checks {
        validate_required_text("check_kind", &check.check_kind, 120)?;
        validate_required_text("check.command", &check.command, 2_000)?;
        validate_proof_check_status(&check.status)?;
        if let Some(duration_ms) = check.duration_ms
            && !(0..=86_400_000).contains(&duration_ms)
        {
            bail!("check.duration_ms out of range");
        }
        if let Some(output) = &check.output_excerpt {
            validate_required_text("check.output_excerpt", output, PROOF_PACKET_MAX_OUTPUT)?;
        }
        validate_json_size("check.metadata", &check.metadata)?;
    }

    if matches!(input.status.as_str(), "passed" | "promoted") {
        let packet = ProofPacket {
            id: "validation-preview".to_string(),
            scope: input.scope.clone(),
            title: input.title.clone(),
            proof_level: input.proof_level.clone(),
            status: input.status.clone(),
            summary: input.summary.clone(),
            artifact_root: input.artifact_root.clone(),
            reviewer: input.reviewer.clone(),
            metadata: input.metadata.clone(),
            created_at: now(),
            promoted_at: None,
        };
        let claims = input
            .claims
            .iter()
            .map(|claim| ProofClaim {
                id: String::new(),
                packet_id: packet.id.clone(),
                claim_key: claim.claim_key.clone(),
                claim: claim.claim.clone(),
                status: claim.status.clone(),
                proof_level: claim.proof_level.clone(),
                evidence: claim.evidence.clone(),
                refutation: claim.refutation.clone(),
                gates: claim.gates.clone(),
                created_at: packet.created_at.clone(),
            })
            .collect::<Vec<_>>();
        let artifacts = input
            .artifacts
            .iter()
            .map(|artifact| ProofArtifact {
                id: String::new(),
                packet_id: packet.id.clone(),
                artifact_kind: artifact.artifact_kind.clone(),
                label: artifact.label.clone(),
                path: artifact.path.clone(),
                sha256: artifact.sha256.clone(),
                metadata: artifact.metadata.clone(),
                created_at: packet.created_at.clone(),
            })
            .collect::<Vec<_>>();
        let checks = input
            .checks
            .iter()
            .map(|check| ProofCheck {
                id: String::new(),
                packet_id: packet.id.clone(),
                check_kind: check.check_kind.clone(),
                command: check.command.clone(),
                status: check.status.clone(),
                exit_code: check.exit_code,
                duration_ms: check.duration_ms,
                output_excerpt: check.output_excerpt.clone(),
                metadata: check.metadata.clone(),
                created_at: packet.created_at.clone(),
            })
            .collect::<Vec<_>>();
        let judgment = judge_proof_packet(&packet, &claims, &artifacts, &checks);
        if !judgment.promotable {
            bail!(
                "passed/promoted proof packet is blocked: {}",
                judgment.blockers.join("; ")
            );
        }
    }
    Ok(())
}

fn judge_proof_packet(
    packet: &ProofPacket,
    claims: &[ProofClaim],
    artifacts: &[ProofArtifact],
    checks: &[ProofCheck],
) -> ProofPacketJudgment {
    let proven_claims = claims
        .iter()
        .filter(|claim| claim.status == "proven")
        .count();
    let partial_claims = claims
        .iter()
        .filter(|claim| claim.status == "partial")
        .count();
    let blocked_claims = claims
        .iter()
        .filter(|claim| claim.status == "blocked")
        .count();
    let refuted_claims = claims
        .iter()
        .filter(|claim| claim.status == "refuted")
        .count();
    let not_claimed = claims
        .iter()
        .filter(|claim| claim.status == "not_claimed")
        .count();
    let passed_checks = checks
        .iter()
        .filter(|check| check.status == "passed")
        .count();
    let failed_checks = checks
        .iter()
        .filter(|check| check.status == "failed")
        .count();
    let mut blockers = Vec::new();
    if claims.is_empty() {
        blockers.push("no proof claims recorded".to_string());
    }
    if proven_claims == 0 {
        blockers.push("no proven claims recorded".to_string());
    }
    if partial_claims > 0 {
        blockers.push(format!("{partial_claims} partial claims remain"));
    }
    if blocked_claims > 0 {
        blockers.push(format!("{blocked_claims} blocked claims remain"));
    }
    if refuted_claims > 0 {
        blockers.push(format!("{refuted_claims} refuted claims remain"));
    }
    if passed_checks == 0 {
        blockers.push("no passing checks recorded".to_string());
    }
    if failed_checks > 0 {
        blockers.push(format!("{failed_checks} failed checks recorded"));
    }
    if artifacts.is_empty() {
        blockers.push("no artifacts recorded".to_string());
    }
    if packet.proof_level.contains("production") && packet.artifact_root.is_none() {
        blockers.push("production proof packet needs an artifact root".to_string());
    }
    ProofPacketJudgment {
        promotable: blockers.is_empty(),
        blockers,
        proven_claims,
        partial_claims,
        blocked_claims,
        refuted_claims,
        not_claimed,
        passed_checks,
        failed_checks,
        artifacts: artifacts.len(),
    }
}

fn proof_packet_non_claims() -> Vec<String> {
    vec![
        "A proof packet proves only the explicit claims listed in the packet.".to_string(),
        "Local proof artifacts are not live external recurrence proof unless the claims and artifacts say so.".to_string(),
        "Generated summaries, model scores, and unchecked source text are not primary evidence by themselves.".to_string(),
    ]
}

fn adversarial_review_non_claims() -> Vec<String> {
    vec![
        "An adversarial review records a human or agent judgment; it does not execute the proof gates by itself.".to_string(),
        "A promote judgment is only valid for the requested proof level and linked packet/scope.".to_string(),
        "Hold and block findings must stay visible until a later proof packet or review resolves them.".to_string(),
    ]
}

#[derive(Debug, Clone)]
struct ProofArtifactSpec {
    label: String,
    path: Option<String>,
    sha256: Option<String>,
}

#[derive(Debug, Clone)]
struct CheckedProofArtifact {
    label: String,
    path: Option<String>,
    resolved_path: Option<String>,
    exists: bool,
    sha256_expected: Option<String>,
    sha256_actual: Option<String>,
    sha256_matches: Option<bool>,
    redaction_findings: Vec<ProofRedactionFinding>,
    warnings: Vec<String>,
    blockers: Vec<String>,
}

fn extract_proof_artifact_specs(packet_json: &Value) -> Vec<ProofArtifactSpec> {
    let sha_map = packet_json
        .get("artifact_sha256")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let mut specs = Vec::new();
    if let Some(items) = packet_json.get("artifacts").and_then(Value::as_array) {
        for item in items {
            let label = item
                .get("label")
                .and_then(Value::as_str)
                .or_else(|| item.get("artifact_kind").and_then(Value::as_str))
                .unwrap_or("artifact")
                .to_string();
            let path = item
                .get("path")
                .and_then(Value::as_str)
                .map(ToString::to_string);
            let sha256 = item
                .get("sha256")
                .and_then(Value::as_str)
                .map(ToString::to_string)
                .or_else(|| sha_for_artifact_path_or_label(&sha_map, path.as_deref(), &label));
            specs.push(ProofArtifactSpec {
                label,
                path,
                sha256,
            });
        }
        return specs;
    }
    if let Some(items) = packet_json.get("artifacts").and_then(Value::as_object) {
        for (label, raw_path) in items {
            let path = raw_path.as_str().map(ToString::to_string);
            let sha256 = sha_for_artifact_path_or_label(&sha_map, path.as_deref(), label);
            specs.push(ProofArtifactSpec {
                label: label.clone(),
                path,
                sha256,
            });
        }
    }
    specs
}

fn sha_for_artifact_path_or_label(
    sha_map: &Map<String, Value>,
    path: Option<&str>,
    label: &str,
) -> Option<String> {
    let by_label = sha_map
        .get(label)
        .and_then(Value::as_str)
        .map(ToString::to_string);
    if by_label.is_some() {
        return by_label;
    }
    let file_name = path
        .and_then(|path| Path::new(path).file_name())
        .and_then(|name| name.to_str())?;
    sha_map
        .get(file_name)
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn verify_proof_artifact_spec(
    packet_path: &Path,
    artifact_root: Option<&str>,
    spec: ProofArtifactSpec,
) -> Result<CheckedProofArtifact> {
    let mut warnings = Vec::new();
    let mut blockers = Vec::new();
    let Some(path_text) = spec.path.clone() else {
        blockers.push("artifact has no path".to_string());
        return Ok(CheckedProofArtifact {
            label: spec.label,
            path: None,
            resolved_path: None,
            exists: false,
            sha256_expected: spec.sha256,
            sha256_actual: None,
            sha256_matches: None,
            redaction_findings: Vec::new(),
            warnings,
            blockers,
        });
    };
    let resolved_path = resolve_proof_artifact_path(packet_path, artifact_root, &path_text);
    let resolved_text = resolved_path.to_string_lossy().to_string();
    if !proof_artifact_path_is_allowed(packet_path, artifact_root, &resolved_path) {
        blockers.push("artifact path escapes proof packet or artifact root".to_string());
        return Ok(CheckedProofArtifact {
            label: spec.label,
            path: Some(path_text),
            resolved_path: Some(resolved_text),
            exists: resolved_path.exists(),
            sha256_expected: spec.sha256,
            sha256_actual: None,
            sha256_matches: None,
            redaction_findings: Vec::new(),
            warnings,
            blockers,
        });
    }
    if !resolved_path.exists() {
        blockers.push("artifact file does not exist".to_string());
        return Ok(CheckedProofArtifact {
            label: spec.label,
            path: Some(path_text),
            resolved_path: Some(resolved_text),
            exists: false,
            sha256_expected: spec.sha256,
            sha256_actual: None,
            sha256_matches: None,
            redaction_findings: Vec::new(),
            warnings,
            blockers,
        });
    }
    let metadata = fs::metadata(&resolved_path).with_context(|| {
        format!(
            "reading proof artifact metadata for verification: {}",
            resolved_path.display()
        )
    })?;
    if !metadata.is_file() {
        blockers.push("artifact path is not a regular file".to_string());
        return Ok(CheckedProofArtifact {
            label: spec.label,
            path: Some(path_text),
            resolved_path: Some(resolved_text),
            exists: true,
            sha256_expected: spec.sha256,
            sha256_actual: None,
            sha256_matches: None,
            redaction_findings: Vec::new(),
            warnings,
            blockers,
        });
    }
    if metadata.len() > PROOF_PACKET_VERIFY_MAX_BYTES {
        blockers.push(format!(
            "artifact is too large to verify safely: {} bytes",
            metadata.len()
        ));
        return Ok(CheckedProofArtifact {
            label: spec.label,
            path: Some(path_text),
            resolved_path: Some(resolved_text),
            exists: true,
            sha256_expected: spec.sha256,
            sha256_actual: None,
            sha256_matches: None,
            redaction_findings: Vec::new(),
            warnings,
            blockers,
        });
    }
    let bytes = fs::read(&resolved_path).with_context(|| {
        format!(
            "reading proof artifact for verification: {}",
            resolved_path.display()
        )
    })?;
    let actual_sha = sha256(&bytes);
    let sha256_matches = spec.sha256.as_ref().map(|expected| expected == &actual_sha);
    if matches!(sha256_matches, Some(false)) {
        blockers.push("artifact sha256 mismatch".to_string());
    }
    if spec.sha256.is_none() {
        warnings.push("artifact has no expected sha256".to_string());
    }
    let text = String::from_utf8_lossy(&bytes);
    let redaction_findings = scan_proof_text_for_secret_like_values(
        &format!("artifact:{}", resolved_path.display()),
        &text,
    );
    Ok(CheckedProofArtifact {
        label: spec.label,
        path: Some(path_text),
        resolved_path: Some(resolved_text),
        exists: true,
        sha256_expected: spec.sha256,
        sha256_actual: Some(actual_sha),
        sha256_matches,
        redaction_findings,
        warnings,
        blockers,
    })
}

fn resolve_proof_artifact_path(
    packet_path: &Path,
    artifact_root: Option<&str>,
    artifact_path: &str,
) -> PathBuf {
    let raw = Path::new(artifact_path);
    if raw.is_absolute() {
        return raw.to_path_buf();
    }
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let packet_parent = packet_path.parent().unwrap_or_else(|| Path::new("."));
    let mut candidates = Vec::new();
    candidates.push(current_dir.join(raw));
    candidates.push(packet_parent.join(raw));
    if let Some(root) = artifact_root {
        let root = Path::new(root);
        if root.is_absolute() {
            candidates.push(root.join(raw));
        } else {
            candidates.push(current_dir.join(root).join(raw));
            candidates.push(packet_parent.join(root).join(raw));
        }
    }
    candidates
        .iter()
        .find(|candidate| candidate.exists())
        .cloned()
        .unwrap_or_else(|| candidates.remove(0))
}

fn proof_artifact_path_is_allowed(
    packet_path: &Path,
    artifact_root: Option<&str>,
    resolved_path: &Path,
) -> bool {
    let Ok(resolved) = normalize_proof_path(resolved_path) else {
        return false;
    };
    let mut roots = Vec::new();
    if let Some(packet_parent) = packet_path.parent() {
        roots.push(packet_parent.to_path_buf());
    }
    if let Some(root) = artifact_root {
        let root = Path::new(root);
        if root.is_absolute() {
            roots.push(root.to_path_buf());
        } else {
            if let Ok(cwd) = std::env::current_dir() {
                roots.push(cwd.join(root));
            }
            if let Some(packet_parent) = packet_path.parent() {
                roots.push(packet_parent.join(root));
            }
        }
    }
    roots
        .into_iter()
        .filter_map(|root| normalize_proof_path(&root).ok())
        .any(|root| resolved.starts_with(root))
}

fn normalize_proof_path(path: &Path) -> Result<PathBuf> {
    let mut normalized = if path.is_absolute() {
        PathBuf::new()
    } else {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    };
    for component in path.components() {
        match component {
            std::path::Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            std::path::Component::RootDir => normalized.push(std::path::MAIN_SEPARATOR.to_string()),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !normalized.pop() {
                    bail!("path escapes root");
                }
            }
            std::path::Component::Normal(part) => normalized.push(part),
        }
    }
    Ok(normalized)
}

fn scan_proof_text_for_secret_like_values(
    location: &str,
    text: &str,
) -> Vec<ProofRedactionFinding> {
    let mut findings = Vec::new();
    for marker in [
        ("authorization_header", "authorization:"),
        ("cookie_header", "cookie:"),
        ("set_cookie_header", "set-cookie:"),
        ("private_key", "-----begin private key-----"),
    ] {
        if text.to_ascii_lowercase().contains(marker.1) {
            findings.push(proof_redaction_finding(location, marker.0, marker.1));
        }
    }
    for fragment in text.split(|ch: char| {
        !(ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '@' | '+'))
    }) {
        if fragment.is_empty() {
            continue;
        }
        if let Some(kind) = proof_secret_fragment_kind(fragment) {
            findings.push(proof_redaction_finding(location, kind, fragment));
        }
    }
    findings
}

fn semantic_proof_packet_blockers(packet_json: &Value, packet_text: &str) -> Vec<String> {
    let mut blockers = Vec::new();
    let lower = packet_text.to_ascii_lowercase();
    let proof_level = packet_json
        .pointer("/packet/proof_level")
        .and_then(Value::as_str)
        .or_else(|| packet_json.get("proof_level").and_then(Value::as_str))
        .unwrap_or("")
        .to_ascii_lowercase();
    let status = packet_json
        .pointer("/packet/status")
        .and_then(Value::as_str)
        .or_else(|| packet_json.get("status").and_then(Value::as_str))
        .unwrap_or("")
        .to_ascii_lowercase();
    if (proof_level.contains("operational") || status == "operational")
        && !contains_any(
            &lower,
            &[
                "recurrence",
                "wall-clock",
                "wall clock",
                "heartbeat",
                "schedule_tick",
                "schedule tick",
                "worker_heartbeat",
            ],
        )
    {
        blockers.push(
            "operational proof claim lacks recurrence or heartbeat evidence marker".to_string(),
        );
    }
    if contains_any(
        &lower,
        &["source freshness", "freshness claim", "latest source"],
    ) && !contains_any(&lower, &["source_health", "source-health", "cursor"])
    {
        blockers.push(
            "source freshness claim lacks source-health or cursor evidence marker".to_string(),
        );
    }
    if contains_any(
        &lower,
        &[
            "model prose",
            "model summary",
            "model-backed report",
            "model backed report",
        ],
    ) && !contains_any(
        &lower,
        &[
            "source_card",
            "source-card",
            "source evidence",
            "citation",
            "cited",
        ],
    ) {
        blockers.push(
            "model/report proof claim lacks source-card or citation evidence marker".to_string(),
        );
    }
    blockers
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn proof_secret_fragment_kind(fragment: &str) -> Option<&'static str> {
    let lower = fragment.to_ascii_lowercase();
    if fragment.starts_with("sk-") && fragment.len() >= 20 {
        return Some("openai_api_key");
    }
    if fragment.starts_with("ghp_") && fragment.len() >= 20 {
        return Some("github_token");
    }
    if fragment.starts_with("github_pat_") && fragment.len() >= 30 {
        return Some("github_fine_grained_token");
    }
    if fragment.starts_with("xoxb-") && fragment.len() >= 20 {
        return Some("slack_bot_token");
    }
    if fragment.starts_with("xoxp-") && fragment.len() >= 20 {
        return Some("slack_user_token");
    }
    if fragment.starts_with("AKIA") && fragment.len() >= 20 {
        return Some("aws_access_key_id");
    }
    if fragment.starts_with("ya29.") && fragment.len() >= 20 {
        return Some("google_oauth_token");
    }
    if looks_like_email_address(fragment) {
        return Some("email_address");
    }
    if lower.contains("bearer") && fragment.len() >= 24 {
        return Some("bearer_token_like_fragment");
    }
    None
}

fn looks_like_email_address(fragment: &str) -> bool {
    if fragment.len() > 320 || !fragment.contains('@') || fragment.starts_with('@') {
        return false;
    }
    let Some((local, domain)) = fragment.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && domain
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-'))
}

fn proof_redaction_finding(location: &str, kind: &str, evidence: &str) -> ProofRedactionFinding {
    ProofRedactionFinding {
        location: location.to_string(),
        kind: kind.to_string(),
        evidence_hash: sha256(evidence.as_bytes())[..16].to_string(),
    }
}

fn proof_child_id(prefix: &str, packet_id: &str, key: &str) -> String {
    format!(
        "{prefix}-{}",
        &sha256(format!("{packet_id}\n{key}").as_bytes())[..24]
    )
}

fn validate_required_text(label: &str, value: &str, max_len: usize) -> Result<()> {
    if value.trim().is_empty() {
        bail!("{label} cannot be empty");
    }
    if value.len() > max_len {
        bail!("{label} is too long");
    }
    if value.contains('\0') {
        bail!("{label} contains a null byte");
    }
    Ok(())
}

fn validate_optional_pathish(label: &str, value: &str) -> Result<()> {
    validate_required_text(label, value, 2_000)?;
    if value.contains('\n') || value.contains('\r') {
        bail!("{label} cannot contain newlines");
    }
    Ok(())
}

fn validate_json_size(label: &str, value: &Value) -> Result<()> {
    let text = canonical_json(value)?;
    if text.len() > 100_000 {
        bail!("{label} JSON is too large");
    }
    Ok(())
}

fn validate_sha256_hex(value: &str) -> Result<()> {
    if value.len() != 64 || !value.chars().all(|ch| ch.is_ascii_hexdigit()) {
        bail!("artifact sha256 must be 64 hex characters");
    }
    Ok(())
}

fn validate_proof_packet_status(status: &str) -> Result<()> {
    match status {
        "draft" | "partial" | "blocked" | "passed" | "promoted" | "rejected" => Ok(()),
        other => bail!("unsupported proof packet status: {other}"),
    }
}

fn validate_adversarial_review_input(input: &AdversarialReviewRunInput) -> Result<()> {
    if let Some(packet_id) = &input.packet_id {
        validate_id(packet_id)?;
    }
    validate_required_text("scope", &input.scope, 200)?;
    validate_required_text("title", &input.title, 300)?;
    validate_required_text("reviewer", &input.reviewer, 200)?;
    validate_required_text("requested_proof_level", &input.requested_proof_level, 120)?;
    validate_adversarial_review_judgment(&input.judgment)?;
    validate_required_text("summary", &input.summary, PROOF_PACKET_MAX_TEXT)?;
    validate_required_text(
        "strongest_fake_done_path",
        &input.strongest_fake_done_path,
        PROOF_PACKET_MAX_TEXT,
    )?;
    validate_json_size("review.refutations", &input.refutations)?;
    validate_json_size("review.skipped_categories", &input.skipped_categories)?;
    validate_json_size("review.metadata", &input.metadata)?;
    if input.findings.len() > PROOF_PACKET_MAX_ITEMS {
        bail!("too many adversarial review findings");
    }
    if matches!(input.judgment.as_str(), "hold" | "block") && input.findings.is_empty() {
        bail!("hold/block adversarial reviews must record at least one finding");
    }
    if input.judgment == "promote"
        && input
            .findings
            .iter()
            .any(|finding| finding.status == "blocking")
    {
        bail!("promote adversarial review cannot contain blocking findings");
    }
    let mut titles = BTreeSet::new();
    for finding in &input.findings {
        if !(0..=3).contains(&finding.severity) {
            bail!("finding.severity must be between 0 and 3");
        }
        validate_adversarial_review_finding_status(&finding.status)?;
        validate_required_text("finding.title", &finding.title, 300)?;
        if !titles.insert(finding.title.clone()) {
            bail!(
                "duplicate adversarial review finding title: {}",
                finding.title
            );
        }
        validate_required_text("finding.body", &finding.body, PROOF_PACKET_MAX_TEXT)?;
        validate_json_size("finding.evidence", &finding.evidence)?;
        if let Some(recommendation) = &finding.recommendation {
            validate_required_text("finding.recommendation", recommendation, 2_000)?;
        }
    }
    Ok(())
}

fn validate_adversarial_review_judgment(judgment: &str) -> Result<()> {
    match judgment {
        "promote" | "hold" | "block" => Ok(()),
        other => bail!("unsupported adversarial review judgment: {other}"),
    }
}

fn validate_adversarial_review_finding_status(status: &str) -> Result<()> {
    match status {
        "blocking" | "non_blocking" | "resolved" => Ok(()),
        other => bail!("unsupported adversarial review finding status: {other}"),
    }
}

fn validate_proof_claim_status(status: &str) -> Result<()> {
    match status {
        "proven" | "partial" | "blocked" | "refuted" | "not_claimed" => Ok(()),
        other => bail!("unsupported proof claim status: {other}"),
    }
}

fn validate_proof_check_status(status: &str) -> Result<()> {
    match status {
        "passed" | "failed" | "skipped" | "blocked" => Ok(()),
        other => bail!("unsupported proof check status: {other}"),
    }
}

fn proof_packet_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProofPacket> {
    let metadata_json: String = row.get(8)?;
    Ok(ProofPacket {
        id: row.get(0)?,
        scope: row.get(1)?,
        title: row.get(2)?,
        proof_level: row.get(3)?,
        status: row.get(4)?,
        summary: row.get(5)?,
        artifact_root: row.get(6)?,
        reviewer: row.get(7)?,
        metadata: parse_json_column(&metadata_json, 8)?,
        created_at: row.get(9)?,
        promoted_at: row.get(10)?,
    })
}

fn proof_claim_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProofClaim> {
    let evidence_json: String = row.get(6)?;
    let refutation_json: String = row.get(7)?;
    let gates_json: String = row.get(8)?;
    Ok(ProofClaim {
        id: row.get(0)?,
        packet_id: row.get(1)?,
        claim_key: row.get(2)?,
        claim: row.get(3)?,
        status: row.get(4)?,
        proof_level: row.get(5)?,
        evidence: parse_json_column(&evidence_json, 6)?,
        refutation: parse_json_column(&refutation_json, 7)?,
        gates: parse_json_column(&gates_json, 8)?,
        created_at: row.get(9)?,
    })
}

fn proof_artifact_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProofArtifact> {
    let metadata_json: String = row.get(6)?;
    Ok(ProofArtifact {
        id: row.get(0)?,
        packet_id: row.get(1)?,
        artifact_kind: row.get(2)?,
        label: row.get(3)?,
        path: row.get(4)?,
        sha256: row.get(5)?,
        metadata: parse_json_column(&metadata_json, 6)?,
        created_at: row.get(7)?,
    })
}

fn proof_check_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProofCheck> {
    let metadata_json: String = row.get(8)?;
    Ok(ProofCheck {
        id: row.get(0)?,
        packet_id: row.get(1)?,
        check_kind: row.get(2)?,
        command: row.get(3)?,
        status: row.get(4)?,
        exit_code: row.get(5)?,
        duration_ms: row.get(6)?,
        output_excerpt: row.get(7)?,
        metadata: parse_json_column(&metadata_json, 8)?,
        created_at: row.get(9)?,
    })
}

fn proof_packet_summary_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProofPacketSummary> {
    Ok(ProofPacketSummary {
        id: row.get(0)?,
        scope: row.get(1)?,
        title: row.get(2)?,
        proof_level: row.get(3)?,
        status: row.get(4)?,
        claim_count: nonnegative_usize(row.get(5)?),
        passed_checks: nonnegative_usize(row.get(6)?),
        blocker_count: nonnegative_usize(row.get(7)?),
        created_at: row.get(8)?,
        promoted_at: row.get(9)?,
    })
}

fn adversarial_review_run_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<AdversarialReviewRun> {
    let refutations_json: String = row.get(9)?;
    let skipped_categories_json: String = row.get(10)?;
    let metadata_json: String = row.get(11)?;
    Ok(AdversarialReviewRun {
        id: row.get(0)?,
        packet_id: row.get(1)?,
        scope: row.get(2)?,
        title: row.get(3)?,
        reviewer: row.get(4)?,
        requested_proof_level: row.get(5)?,
        judgment: row.get(6)?,
        summary: row.get(7)?,
        strongest_fake_done_path: row.get(8)?,
        refutations: parse_json_column(&refutations_json, 9)?,
        skipped_categories: parse_json_column(&skipped_categories_json, 10)?,
        metadata: parse_json_column(&metadata_json, 11)?,
        created_at: row.get(12)?,
    })
}

fn adversarial_review_finding_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<AdversarialReviewFinding> {
    let evidence_json: String = row.get(6)?;
    Ok(AdversarialReviewFinding {
        id: row.get(0)?,
        review_id: row.get(1)?,
        severity: row.get(2)?,
        status: row.get(3)?,
        title: row.get(4)?,
        body: row.get(5)?,
        evidence: parse_json_column(&evidence_json, 6)?,
        recommendation: row.get(7)?,
        created_at: row.get(8)?,
    })
}

fn adversarial_review_summary_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<AdversarialReviewSummary> {
    Ok(AdversarialReviewSummary {
        id: row.get(0)?,
        packet_id: row.get(1)?,
        scope: row.get(2)?,
        title: row.get(3)?,
        reviewer: row.get(4)?,
        requested_proof_level: row.get(5)?,
        judgment: row.get(6)?,
        finding_count: nonnegative_usize(row.get(7)?),
        blocking_finding_count: nonnegative_usize(row.get(8)?),
        created_at: row.get(9)?,
    })
}
