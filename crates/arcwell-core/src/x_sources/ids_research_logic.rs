use super::*;

pub(crate) fn source_card_id(url: &str, provider: &str, source_type: &str) -> String {
    let hash = sha256(format!("{provider}\n{source_type}\n{url}").as_bytes());
    format!("src-{}", &hash[..16])
}

pub(crate) fn research_source_id(canonical_key: &str) -> String {
    let hash = sha256(canonical_key.as_bytes());
    format!("rsrc-{}", &hash[..16])
}

pub(crate) fn research_run_source_link_id(run_id: &str, source_id: &str) -> String {
    let hash = sha256(format!("{run_id}\n{source_id}").as_bytes());
    format!("rrsrc-{}", &hash[..16])
}

pub(crate) fn research_role_run_id() -> String {
    format!("rrole-{}", Uuid::new_v4().simple())
}

pub(crate) fn research_artifact_id(run_id: &str, artifact_type: &str, body: &str) -> String {
    let hash = sha256(format!("{run_id}\n{artifact_type}\n{body}").as_bytes());
    format!("rart-{}", &hash[..16])
}

pub(crate) fn commerce_candidate_id(
    run_id: &str,
    source_url: &str,
    normalized_item_key: &str,
    variant_key: &str,
) -> String {
    let hash =
        sha256(format!("{run_id}\n{source_url}\n{normalized_item_key}\n{variant_key}").as_bytes());
    format!("ccand-{}", &hash[..16])
}

pub(crate) fn commerce_availability_proof_id() -> String {
    format!("cproof-{}", Uuid::new_v4().simple())
}

pub(crate) fn commerce_context_fact_id(
    run_id: &str,
    fact_key: &str,
    source_family: &str,
) -> String {
    let hash = sha256(format!("{run_id}\n{fact_key}\n{source_family}").as_bytes());
    format!("cctx-{}", &hash[..16])
}

pub(crate) fn commerce_verification_attempt_id() -> String {
    format!("cverify-{}", Uuid::new_v4().simple())
}

pub(crate) fn commerce_report_judgment_id() -> String {
    format!("cjudge-{}", Uuid::new_v4().simple())
}

pub(crate) fn job_candidate_profile_id(label: &str) -> String {
    let hash = sha256(label.trim().to_ascii_lowercase().as_bytes());
    format!("jprof-{}", &hash[..16])
}

pub(crate) fn job_evidence_card_id(
    profile_id: &str,
    title: &str,
    evidence_type: &str,
    locator: Option<&str>,
) -> String {
    let hash = sha256(
        format!(
            "{profile_id}\n{}\n{}\n{}",
            title.trim().to_ascii_lowercase(),
            evidence_type.trim().to_ascii_lowercase(),
            locator.unwrap_or("").trim().to_ascii_lowercase()
        )
        .as_bytes(),
    );
    format!("jev-{}", &hash[..16])
}

pub(crate) fn job_evidence_claim_id(evidence_card_id: &str, claim: &str) -> String {
    let hash =
        sha256(format!("{evidence_card_id}\n{}", claim.trim().to_ascii_lowercase()).as_bytes());
    format!("jclaim-{}", &hash[..16])
}

pub(crate) fn job_privacy_rule_id(pattern: &str, rule_type: &str) -> String {
    let hash = sha256(
        format!(
            "{}\n{}",
            pattern.trim().to_ascii_lowercase(),
            rule_type.trim().to_ascii_lowercase()
        )
        .as_bytes(),
    );
    format!("jprule-{}", &hash[..16])
}

pub(crate) fn job_privacy_check_id() -> String {
    format!("jpcheck-{}", Uuid::new_v4().simple())
}

pub(crate) fn job_source_id(url: &str) -> String {
    let hash = sha256(url.trim().to_ascii_lowercase().as_bytes());
    format!("jsrc-{}", &hash[..16])
}

pub(crate) fn job_source_health_id() -> String {
    format!("jshealth-{}", Uuid::new_v4().simple())
}

pub(crate) fn job_role_card_id(company: &str, role_title: &str, source_url: &str) -> String {
    let hash = sha256(
        format!(
            "{}\n{}\n{}",
            company.trim().to_ascii_lowercase(),
            role_title.trim().to_ascii_lowercase(),
            source_url.trim().to_ascii_lowercase()
        )
        .as_bytes(),
    );
    format!("jrole-{}", &hash[..16])
}

pub(crate) fn job_role_source_link_id(role_id: &str, source_url: &str) -> String {
    let hash = sha256(format!("{role_id}\n{}", source_url.trim().to_ascii_lowercase()).as_bytes());
    format!("jrsrc-{}", &hash[..16])
}

pub(crate) fn job_fit_score_id() -> String {
    format!("jfit-{}", Uuid::new_v4().simple())
}

pub(crate) fn job_skeptic_finding_id() -> String {
    format!("jskep-{}", Uuid::new_v4().simple())
}

pub(crate) fn job_application_packet_id() -> String {
    format!("jpacket-{}", Uuid::new_v4().simple())
}

pub(crate) fn job_company_card_id(website_url: &str) -> String {
    let hash = sha256(website_url.trim().to_ascii_lowercase().as_bytes());
    format!("jco-{}", &hash[..16])
}

pub(crate) fn job_contact_id(public_profile_url: &str) -> String {
    let hash = sha256(public_profile_url.trim().to_ascii_lowercase().as_bytes());
    format!("jcontact-{}", &hash[..16])
}

pub(crate) fn job_intro_path_id(role_id: &str, contact_id: &str) -> String {
    let hash = sha256(format!("{role_id}\n{contact_id}").as_bytes());
    format!("jintro-{}", &hash[..16])
}

pub(crate) fn job_search_run_id() -> String {
    format!("jrun-{}", Uuid::new_v4().simple())
}

pub(crate) fn job_role_status_event_id() -> String {
    format!("jrstat-{}", Uuid::new_v4().simple())
}

pub(crate) fn job_application_id(role_id: &str) -> String {
    let hash = sha256(role_id.as_bytes());
    format!("japp-{}", &hash[..16])
}

pub(crate) fn job_weekly_report_id(profile_id: &str, scope: &str, body: &str) -> String {
    let hash = sha256(format!("{profile_id}\n{scope}\n{body}").as_bytes());
    format!("jweek-{}", &hash[..16])
}

pub(crate) fn job_weekly_report_delivery_id() -> String {
    format!("jweekdel-{}", Uuid::new_v4().simple())
}

pub(crate) fn research_host_search_id() -> String {
    format!("rhsearch-{}", Uuid::new_v4().simple())
}

pub(crate) fn research_host_search_result_id(
    host_search_id: &str,
    rank: usize,
    canonical_url: &str,
) -> String {
    let hash = sha256(format!("{host_search_id}\n{rank}\n{canonical_url}").as_bytes());
    format!("rhsres-{}", &hash[..16])
}

pub(crate) fn research_document_id(run_id: &str, path: &Path, byte_sha256: &str) -> String {
    let hash = sha256(format!("{run_id}\n{}\n{byte_sha256}", path.display()).as_bytes());
    format!("rdoc-{}", &hash[..16])
}

pub(crate) fn research_document_span_db_id(document_id: &str, span_id: &str) -> String {
    let hash = sha256(format!("{document_id}\n{span_id}").as_bytes());
    format!("rdspan-{}", &hash[..16])
}

pub(crate) fn research_table_db_id(document_id: &str, table_id: &str) -> String {
    let hash = sha256(format!("{document_id}\n{table_id}").as_bytes());
    format!("rdtable-{}", &hash[..16])
}

pub(crate) fn research_table_cell_id(
    table_id: &str,
    row_index: usize,
    column_index: usize,
) -> String {
    let hash = sha256(format!("{table_id}\n{row_index}\n{column_index}").as_bytes());
    format!("rdcell-{}", &hash[..16])
}

pub(crate) fn research_claim_document_anchor_id(
    claim_source_id: &str,
    anchor_label: &str,
) -> String {
    let hash = sha256(format!("{claim_source_id}\n{anchor_label}").as_bytes());
    format!("rcanchor-{}", &hash[..16])
}

pub(crate) fn research_editorial_run_id() -> String {
    format!("redit-{}", Uuid::new_v4().simple())
}

pub(crate) fn research_iteration_id(run_id: &str, iteration_index: usize) -> String {
    let hash = sha256(format!("{run_id}\n{iteration_index}").as_bytes());
    format!("riter-{}", &hash[..16])
}

pub(crate) fn research_statement_id(run_id: &str, iteration_id: &str, stable_key: &str) -> String {
    let hash = sha256(format!("{run_id}\n{iteration_id}\n{stable_key}").as_bytes());
    format!("rstmt-{}", &hash[..16])
}

pub(crate) fn research_challenge_id(
    run_id: &str,
    iteration_id: &str,
    statement_id: &str,
    challenge_type: &str,
) -> String {
    let hash =
        sha256(format!("{run_id}\n{iteration_id}\n{statement_id}\n{challenge_type}").as_bytes());
    format!("rchlg-{}", &hash[..16])
}

pub(crate) fn research_convergence_host_search_task_id(challenge_id: &str, query: &str) -> String {
    let hash = sha256(
        format!(
            "{challenge_id}\n{}",
            normalized_research_search_query(query)
        )
        .as_bytes(),
    );
    format!("rchst-{}", &hash[..16])
}

pub(crate) fn research_disproof_id(run_id: &str, iteration_id: &str, challenge_id: &str) -> String {
    let hash = sha256(format!("{run_id}\n{iteration_id}\n{challenge_id}").as_bytes());
    format!("rdisp-{}", &hash[..16])
}

pub(crate) fn research_revision_id(
    run_id: &str,
    iteration_id: &str,
    statement_id: &str,
    revision_type: &str,
) -> String {
    let hash =
        sha256(format!("{run_id}\n{iteration_id}\n{statement_id}\n{revision_type}").as_bytes());
    format!("rrev-{}", &hash[..16])
}

pub(crate) fn research_fact_check_id(
    run_id: &str,
    iteration_id: &str,
    statement_id: &str,
) -> String {
    let hash = sha256(format!("{run_id}\n{iteration_id}\n{statement_id}").as_bytes());
    format!("rfact-{}", &hash[..16])
}

pub(crate) fn research_convergence_snapshot_id(run_id: &str, iteration_id: &str) -> String {
    let hash = sha256(format!("{run_id}\n{iteration_id}").as_bytes());
    format!("rcsnap-{}", &hash[..16])
}

pub(crate) fn research_report_judgment_id(run_id: &str, report_id: Option<&str>) -> String {
    let salt = report_id.unwrap_or("convergence-report");
    let hash = sha256(format!("{run_id}\n{salt}").as_bytes());
    format!("rjudge-{}", &hash[..16])
}

pub(crate) fn research_claim_id(run_id: &str, source_card_id: &str, text: &str) -> String {
    let hash = sha256(format!("{run_id}\n{source_card_id}\n{text}").as_bytes());
    format!("rclaim-{}", &hash[..16])
}

pub(crate) fn research_claim_source_id(claim_id: &str, source_card_id: &str) -> String {
    let hash = sha256(format!("{claim_id}\n{source_card_id}").as_bytes());
    format!("rclsrc-{}", &hash[..16])
}

pub(crate) fn research_cluster_id(run_id: &str, theme: &str) -> String {
    let hash = sha256(format!("{run_id}\n{theme}").as_bytes());
    format!("rcluster-{}", &hash[..16])
}

pub(crate) fn research_cluster_claim_id(cluster_id: &str, claim_id: &str) -> String {
    let hash = sha256(format!("{cluster_id}\n{claim_id}").as_bytes());
    format!("rclmem-{}", &hash[..16])
}

pub(crate) fn research_contradiction_id(
    run_id: &str,
    left_claim_id: &str,
    right_claim_id: &str,
) -> String {
    let (left, right) = if left_claim_id <= right_claim_id {
        (left_claim_id, right_claim_id)
    } else {
        (right_claim_id, left_claim_id)
    };
    let hash = sha256(format!("{run_id}\n{left}\n{right}").as_bytes());
    format!("rcontra-{}", &hash[..16])
}

pub(crate) fn research_report_id(run_id: &str) -> String {
    let hash = sha256(run_id.as_bytes());
    format!("rreport-{}", &hash[..16])
}

pub(crate) fn research_claim_theme(claim: &ResearchClaim) -> String {
    claim
        .subject
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|subject| subject.trim().to_ascii_lowercase())
        .unwrap_or_else(|| claim.kind.clone())
}

pub(crate) fn research_cluster_evidence_strength(records: &[ResearchClaimRecord]) -> String {
    let source_count: usize = records.iter().map(|record| record.sources.len()).sum();
    let high_confidence = records
        .iter()
        .filter(|record| record.claim.confidence >= 0.75)
        .count();
    if source_count >= 3 && high_confidence >= 2 {
        "strong".to_string()
    } else if source_count >= 1 {
        "limited".to_string()
    } else {
        "unsupported".to_string()
    }
}

pub(crate) fn research_statement_stable_key(text: &str) -> String {
    normalize_research_stable_key(text).unwrap_or_else(|_| "statement".to_string())
}

pub(crate) fn research_statement_type_from_claim(kind: &str) -> String {
    match kind {
        "fact" | "measurement" | "interpretation" | "recommendation" | "forecast" => {
            kind.to_string()
        }
        "risk" => "hypothesis".to_string(),
        "design" | "architecture" => "design_proposal".to_string(),
        "question" | "open_question" => "open_question".to_string(),
        _ => "fact".to_string(),
    }
}

pub(crate) fn research_certainty_label(confidence: f64) -> String {
    if confidence >= 0.8 {
        "high".to_string()
    } else if confidence >= 0.55 {
        "moderate".to_string()
    } else if confidence >= 0.3 {
        "low".to_string()
    } else {
        "very_low".to_string()
    }
}

pub(crate) fn research_statement_importance(claim: &ResearchClaim) -> String {
    let text = claim.text.to_ascii_lowercase();
    if text.contains("security")
        || text.contains("safety")
        || text.contains("privacy")
        || text.contains("regulatory")
        || text.contains("must ")
        || text.contains("cannot ")
    {
        "critical".to_string()
    } else if claim.confidence >= 0.8
        || matches!(
            claim.kind.as_str(),
            "measurement" | "recommendation" | "interpretation"
        )
    {
        "high".to_string()
    } else if claim.confidence >= 0.5 {
        "medium".to_string()
    } else {
        "low".to_string()
    }
}

pub(crate) fn statement_evidence_claim_ids(statement: &ResearchStatement) -> Vec<String> {
    statement
        .evidence
        .get("claim_ids")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect()
}

pub(crate) fn statement_evidence_source_card_ids(statement: &ResearchStatement) -> Vec<String> {
    statement
        .evidence
        .get("source_card_ids")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect()
}

#[derive(Debug, Clone)]
pub(crate) struct ResearchStatementFactSupport {
    pub(crate) claim_ids: Vec<String>,
    pub(crate) source_card_ids: Vec<String>,
    pub(crate) matched_claim_ids: Vec<String>,
    pub(crate) acceptable_source_card_ids: Vec<String>,
    pub(crate) missing_claim_ids: Vec<String>,
    pub(crate) unacceptable_source_card_ids: Vec<String>,
}

impl ResearchStatementFactSupport {
    pub(crate) fn metadata_json(&self) -> Value {
        json!({
            "claim_ids": self.claim_ids,
            "source_card_ids": self.source_card_ids,
            "matched_claim_ids": self.matched_claim_ids,
            "acceptable_source_card_ids": self.acceptable_source_card_ids,
            "missing_claim_ids": self.missing_claim_ids,
            "unacceptable_source_card_ids": self.unacceptable_source_card_ids,
            "has_acceptable_evidence": !self.acceptable_source_card_ids.is_empty()
                && self.missing_claim_ids.is_empty(),
        })
    }
}

pub(crate) fn research_statement_fact_support(
    statement: &ResearchStatement,
    claims: &[ResearchClaimRecord],
    run_sources: &[ResearchRunSourceRecord],
) -> ResearchStatementFactSupport {
    let claim_ids = statement_evidence_claim_ids(statement);
    let source_card_ids = statement_evidence_source_card_ids(statement);
    let claim_by_id = claims
        .iter()
        .map(|record| (record.claim.id.as_str(), record))
        .collect::<BTreeMap<_, _>>();
    let run_source_card_by_id = run_sources
        .iter()
        .filter_map(|record| {
            record
                .source_card
                .as_ref()
                .map(|card| (card.id.as_str(), card))
        })
        .collect::<BTreeMap<_, _>>();
    let mut matched_claim_ids = Vec::new();
    let mut acceptable_source_card_ids = Vec::new();
    let mut missing_claim_ids = Vec::new();
    let mut unacceptable_source_card_ids = Vec::new();
    for claim_id in &claim_ids {
        let Some(record) = claim_by_id.get(claim_id.as_str()) else {
            missing_claim_ids.push(claim_id.clone());
            continue;
        };
        matched_claim_ids.push(claim_id.clone());
        for source in &record.sources {
            match run_source_card_by_id.get(source.source_card_id.as_str()) {
                Some(card) if source_card_is_primary_evidence(card) => {
                    acceptable_source_card_ids.push(source.source_card_id.clone());
                }
                Some(_) | None => {
                    unacceptable_source_card_ids.push(source.source_card_id.clone());
                }
            }
        }
    }
    for source_card_id in &source_card_ids {
        if !run_source_card_by_id
            .get(source_card_id.as_str())
            .is_some_and(|card| source_card_is_primary_evidence(card))
        {
            unacceptable_source_card_ids.push(source_card_id.clone());
        }
    }
    matched_claim_ids.sort();
    matched_claim_ids.dedup();
    acceptable_source_card_ids.sort();
    acceptable_source_card_ids.dedup();
    missing_claim_ids.sort();
    missing_claim_ids.dedup();
    unacceptable_source_card_ids.sort();
    unacceptable_source_card_ids.dedup();
    ResearchStatementFactSupport {
        claim_ids,
        source_card_ids,
        matched_claim_ids,
        acceptable_source_card_ids,
        missing_claim_ids,
        unacceptable_source_card_ids,
    }
}

pub(crate) fn statement_has_stale_source(
    statement: &ResearchStatement,
    sources: &[ResearchRunSourceRecord],
) -> bool {
    let source_card_ids = statement_evidence_source_card_ids(statement)
        .into_iter()
        .collect::<BTreeSet<_>>();
    sources
        .iter()
        .filter_map(|record| record.source_card.as_ref())
        .filter(|card| source_card_ids.contains(&card.id))
        .any(|card| {
            source_card_metadata_strings(&card.metadata, "quality_flags")
                .iter()
                .any(|flag| flag == "stale_source")
        })
}

pub(crate) fn statement_has_contradiction(
    statement: &ResearchStatement,
    contradictions: &[ResearchContradiction],
) -> bool {
    let claim_ids = statement_evidence_claim_ids(statement)
        .into_iter()
        .collect::<BTreeSet<_>>();
    contradictions.iter().any(|contradiction| {
        claim_ids.contains(&contradiction.left_claim_id)
            || claim_ids.contains(&contradiction.right_claim_id)
    })
}

pub(crate) fn research_challenge_severity(
    statement: &ResearchStatement,
    challenge_type: &str,
) -> String {
    match challenge_type {
        "contradiction" => "critical".to_string(),
        "missing_primary_source" | "citation_gap" => {
            if matches!(statement.importance.as_str(), "critical" | "high") {
                "error".to_string()
            } else {
                "warning".to_string()
            }
        }
        "stale_evidence" => "warning".to_string(),
        "alternative_hypothesis" => "info".to_string(),
        _ => "warning".to_string(),
    }
}

pub(crate) fn research_challenge_rationale(
    statement: &ResearchStatement,
    challenge_type: &str,
) -> String {
    match challenge_type {
        "alternative_hypothesis" => format!(
            "Attempt to find an alternative explanation or boundary condition for: {}",
            statement.text
        ),
        "missing_primary_source" => format!(
            "High-impact statement lacks primary-source coverage and needs official, paper, or first-party evidence: {}",
            statement.text
        ),
        "citation_gap" => format!(
            "Statement lacks extracted claim evidence and must not appear as a conclusion without support: {}",
            statement.text
        ),
        "stale_evidence" => format!(
            "Statement is supported by stale evidence and needs freshness verification or a caveat: {}",
            statement.text
        ),
        "contradiction" => format!(
            "Structured evidence conflicts with this statement and requires resolution before final synthesis: {}",
            statement.text
        ),
        _ => format!(
            "Pressure-test the statement before final synthesis: {}",
            statement.text
        ),
    }
}

pub(crate) fn research_challenge_queries(
    statement: &ResearchStatement,
    challenge_type: &str,
) -> Vec<String> {
    let scope =
        bounded_challenge_search_scope(statement.scope.as_deref().unwrap_or(&statement.text));
    match challenge_type {
        "alternative_hypothesis" => vec![
            bounded_challenge_search_query(&scope, "alternative explanation"),
            bounded_challenge_search_query(&scope, "limitations counterexample"),
        ],
        "missing_primary_source" => vec![
            bounded_challenge_search_query(&scope, "official source"),
            bounded_challenge_search_query(&scope, "primary documentation paper dataset"),
        ],
        "citation_gap" => vec![bounded_challenge_search_query(
            &bounded_challenge_search_scope(&statement.text),
            "evidence",
        )],
        "stale_evidence" => vec![bounded_challenge_search_query(
            &scope,
            "latest update current status",
        )],
        "contradiction" => vec![
            bounded_challenge_search_query(&scope, "conflicting evidence"),
            bounded_challenge_search_query(&scope, "correction erratum"),
        ],
        _ => vec![bounded_challenge_search_query(&scope, "verification")],
    }
}

pub(crate) fn bounded_challenge_search_scope(text: &str) -> String {
    let normalized = text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_matches(|c: char| c.is_ascii_punctuation() && c != '/' && c != '-')
        .to_string();
    let first_sentence = normalized
        .split(['.', '?', '!', '\n'])
        .find(|part| part.trim().chars().count() >= 12)
        .map(str::trim)
        .unwrap_or_else(|| normalized.trim());
    let mut out = String::new();
    for word in first_sentence.split_whitespace() {
        let next_len = out.len() + if out.is_empty() { 0 } else { 1 } + word.len();
        if next_len > 180 {
            break;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(word);
    }
    if out.trim().is_empty() {
        "research claim".to_string()
    } else {
        out
    }
}

pub(crate) fn bounded_challenge_search_query(scope: &str, suffix: &str) -> String {
    let mut query = format!("{} {}", scope.trim(), suffix.trim())
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    while query.len() > 240 {
        let Some(last_space) = query[..240].rfind(' ') else {
            query.truncate(240);
            break;
        };
        query.truncate(last_space);
    }
    query
}

pub(crate) fn research_challenge_source_families(challenge_type: &str) -> Vec<String> {
    match challenge_type {
        "missing_primary_source" => vec![
            "official".to_string(),
            "paper".to_string(),
            "dataset".to_string(),
        ],
        "stale_evidence" => vec![
            "official".to_string(),
            "news".to_string(),
            "release_notes".to_string(),
        ],
        "contradiction" => vec![
            "primary".to_string(),
            "independent_replication".to_string(),
            "audit".to_string(),
        ],
        _ => vec!["primary".to_string(), "secondary".to_string()],
    }
}

pub(crate) fn host_search_proof_for_challenge(
    challenge: &ResearchChallenge,
    host_searches: &[ResearchHostSearchRecord],
) -> Option<Value> {
    let planned_queries = research_challenge_planned_queries(challenge);
    if planned_queries.is_empty() {
        return None;
    }
    let mut matched_search_ids = Vec::new();
    let mut matched_result_ids = Vec::new();
    let mut research_source_ids = Vec::new();
    let mut source_card_ids = Vec::new();
    let mut matched_queries = Vec::new();
    for query in planned_queries {
        let Some(proof) = host_search_proof_for_challenge_query(challenge, &query, host_searches)
        else {
            continue;
        };
        matched_queries.push(proof.normalized_query);
        matched_search_ids.extend(proof.matched_search_ids);
        matched_result_ids.extend(proof.matched_result_ids);
        research_source_ids.extend(proof.research_source_ids);
        source_card_ids.extend(proof.source_card_ids);
    }
    if matched_result_ids.is_empty() {
        return None;
    }
    matched_queries.sort();
    matched_queries.dedup();
    matched_search_ids.sort();
    matched_search_ids.dedup();
    matched_result_ids.sort();
    matched_result_ids.dedup();
    research_source_ids.sort();
    research_source_ids.dedup();
    source_card_ids.sort();
    source_card_ids.dedup();
    let selected_result_count = matched_result_ids.len();
    Some(json!({
        "matched_planned_queries": matched_queries,
        "host_search_ids": matched_search_ids,
        "host_search_result_ids": matched_result_ids,
        "research_source_ids": research_source_ids,
        "source_card_ids": source_card_ids,
        "selected_result_count": selected_result_count,
        "requires_manual_source_read": true
    }))
}

#[derive(Debug, Clone)]
pub(crate) struct ResearchHostSearchProofParts {
    pub(crate) normalized_query: String,
    pub(crate) matched_search_ids: Vec<String>,
    pub(crate) matched_result_ids: Vec<String>,
    pub(crate) research_source_ids: Vec<String>,
    pub(crate) source_card_ids: Vec<String>,
    pub(crate) selected_result_count: usize,
}

pub(crate) fn host_search_proof_for_challenge_query(
    challenge: &ResearchChallenge,
    query: &str,
    host_searches: &[ResearchHostSearchRecord],
) -> Option<ResearchHostSearchProofParts> {
    let normalized_query = normalized_research_search_query(query);
    if normalized_query.is_empty() {
        return None;
    }
    let required_families = challenge
        .required_source_families
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>();
    let mut matched_search_ids = Vec::new();
    let mut matched_result_ids = Vec::new();
    let mut research_source_ids = Vec::new();
    let mut source_card_ids = Vec::new();
    for record in host_searches {
        if normalized_research_search_query(&record.search.query) != normalized_query {
            continue;
        }
        let selected = record
            .results
            .iter()
            .filter(|result| {
                result.selected_for_ingest
                    && result.research_source_id.is_some()
                    && (required_families.is_empty()
                        || result
                            .source_family_guess
                            .as_deref()
                            .map(|family| {
                                required_families.contains(&family.trim().to_ascii_lowercase())
                            })
                            .unwrap_or(true))
            })
            .collect::<Vec<_>>();
        if selected.is_empty() {
            continue;
        }
        matched_search_ids.push(record.search.id.clone());
        for result in selected {
            matched_result_ids.push(result.id.clone());
            if let Some(source_id) = &result.research_source_id {
                research_source_ids.push(source_id.clone());
            }
            if let Some(source_card_id) = &result.source_card_id {
                source_card_ids.push(source_card_id.clone());
            }
        }
    }
    if matched_result_ids.is_empty() {
        return None;
    }
    matched_search_ids.sort();
    matched_search_ids.dedup();
    matched_result_ids.sort();
    matched_result_ids.dedup();
    research_source_ids.sort();
    research_source_ids.dedup();
    source_card_ids.sort();
    source_card_ids.dedup();
    let selected_result_count = matched_result_ids.len();
    Some(ResearchHostSearchProofParts {
        normalized_query,
        matched_search_ids,
        matched_result_ids,
        research_source_ids,
        source_card_ids,
        selected_result_count,
    })
}

pub(crate) fn research_challenge_planned_queries(challenge: &ResearchChallenge) -> Vec<String> {
    let mut queries = challenge
        .search_plan
        .get("queries")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::trim)
        .filter(|query| !query.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    queries.sort_by_key(|query| normalized_research_search_query(query));
    queries.dedup_by(|left, right| {
        normalized_research_search_query(left) == normalized_research_search_query(right)
    });
    queries
}

pub(crate) fn merge_challenge_host_search_proof(
    search_plan: &Value,
    proof: &Value,
    status: &str,
) -> Value {
    let mut plan = search_plan.as_object().cloned().unwrap_or_default();
    plan.insert("status".to_string(), json!(status));
    plan.insert("host_search_proof".to_string(), proof.clone());
    Value::Object(plan)
}

pub(crate) fn normalized_research_search_query(query: &str) -> String {
    query
        .split_whitespace()
        .map(|part| part.trim_matches(|ch: char| !ch.is_ascii_alphanumeric()))
        .filter(|part| !part.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn deterministic_disproof_verdict(
    challenge: &ResearchChallenge,
) -> (String, String, f64, bool, String) {
    match challenge.challenge_type.as_str() {
        "contradiction" => (
            "weakens".to_string(),
            "moderate".to_string(),
            -0.30,
            true,
            "Structured contradiction found; statement must be revised, narrowed, or explicitly caveated."
                .to_string(),
        ),
        "missing_primary_source" => (
            "unknown".to_string(),
            "moderate".to_string(),
            -0.20,
            true,
            "Required primary-source evidence is missing for a high-impact statement.".to_string(),
        ),
        "citation_gap" => (
            "unknown".to_string(),
            "moderate".to_string(),
            -0.25,
            true,
            "Statement has no extracted claim evidence and cannot be treated as supported.".to_string(),
        ),
        "stale_evidence" => (
            "weakens".to_string(),
            "weak".to_string(),
            -0.10,
            true,
            "Evidence is stale; carry a freshness caveat until updated sources are linked.".to_string(),
        ),
        "alternative_hypothesis" => (
            "inconclusive".to_string(),
            "weak".to_string(),
            0.0,
            false,
            "Alternative-hypothesis search is queued for host/provider work; no deterministic refutation found."
                .to_string(),
        ),
        _ if matches!(challenge.severity.as_str(), "critical" | "error") => (
            "unknown".to_string(),
            "moderate".to_string(),
            -0.15,
            true,
            "High-severity challenge remains unresolved.".to_string(),
        ),
        _ => (
            "inconclusive".to_string(),
            "weak".to_string(),
            0.0,
            false,
            "No deterministic disproof was found.".to_string(),
        ),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn convergence_stop_reason(
    iteration_index: usize,
    statements: &[ResearchStatement],
    critical_open_challenges: usize,
    high_open_challenges: usize,
    strong_refutations: usize,
    unknown_high_impact_claims: usize,
    source_count_total: usize,
    no_progress_count: usize,
    _source_novelty_score: f64,
    _mean_confidence_delta: f64,
    elapsed_seconds: i64,
    config: &ResearchConvergenceConfig,
) -> String {
    if statements.is_empty() {
        return "no_analytical_statements".to_string();
    }
    if elapsed_seconds >= config.max_seconds {
        return "max_seconds".to_string();
    }
    if source_count_total >= config.max_sources {
        return "max_sources".to_string();
    }
    let active_fact_check_blocked =
        config.require_active_fact_check && unknown_high_impact_claims > 0;
    let has_blockers = critical_open_challenges > 0
        || high_open_challenges > 0
        || strong_refutations > 0
        || active_fact_check_blocked;
    if !has_blockers && no_progress_count >= config.no_progress_iteration_limit {
        return "settled".to_string();
    }
    if iteration_index >= config.max_iterations {
        return "max_iterations".to_string();
    }
    "continue".to_string()
}

// allow: refactoring this N-arg signature is out of scope for the lint-cleanup pass.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_research_report_judgment(
    run_id: &str,
    report_id: Option<&str>,
    status: &ResearchConvergenceStatus,
    statements: &[ResearchStatement],
    challenges: &[ResearchChallenge],
    disproofs: &[ResearchDisproof],
    fact_checks: &[ResearchFactCheck],
    claims: &[ResearchClaimRecord],
    sources: &[ResearchRunSourceRecord],
) -> Result<ResearchReportJudgment> {
    let current_statement_ids = status
        .current_statements
        .iter()
        .map(|statement| statement.id.clone())
        .collect::<BTreeSet<_>>();
    let current_statements = if status.current_statements.is_empty() {
        statements
    } else {
        status.current_statements.as_slice()
    };
    let high_open_challenges = challenges
        .iter()
        .filter(|challenge| {
            matches!(
                challenge.status.as_str(),
                "open" | "searching" | "unresolved"
            ) && matches!(challenge.severity.as_str(), "critical" | "error")
        })
        .count();
    let strong_refutations = status.strong_refutations.len();
    let wrong_or_unknown_high = fact_checks
        .iter()
        .filter(|check| {
            (current_statement_ids.is_empty()
                || current_statement_ids.contains(&check.statement_id))
                && check.impact == "high"
                && matches!(check.label.as_str(), "wrong" | "unknown")
        })
        .count();
    let unsupported_statements = current_statements
        .iter()
        .filter(|statement| statement_evidence_claim_ids(statement).is_empty())
        .count();
    let claim_by_id = claims
        .iter()
        .map(|record| (record.claim.id.as_str(), record))
        .collect::<BTreeMap<_, _>>();
    let source_card_by_id = sources
        .iter()
        .filter_map(|record| {
            record
                .source_card
                .as_ref()
                .map(|card| (card.id.as_str(), card))
        })
        .collect::<BTreeMap<_, _>>();
    let current_claim_ids = current_statements
        .iter()
        .flat_map(statement_evidence_claim_ids)
        .collect::<BTreeSet<_>>();
    let current_claim_records = current_claim_ids
        .iter()
        .filter_map(|claim_id| claim_by_id.get(claim_id.as_str()).copied())
        .collect::<Vec<_>>();
    let measurement_claims_without_document_anchor = current_claim_records
        .iter()
        .filter(|record| record.claim.kind == "measurement" && record.document_anchors.is_empty())
        .count();
    let claims_without_primary_source_evidence = current_claim_records
        .iter()
        .filter(|record| {
            !record.sources.iter().any(|source| {
                source_card_by_id
                    .get(source.source_card_id.as_str())
                    .is_some_and(|card| source_card_is_primary_evidence(card))
            })
        })
        .count();
    let stale_current_statement_evidence = current_statements
        .iter()
        .filter(|statement| statement_has_stale_source(statement, sources))
        .count();
    let open_host_search_tasks = status
        .host_search_tasks
        .iter()
        .filter(|task| task.status != "recorded")
        .count();
    let mut blocking_findings = Vec::new();
    if current_statements.is_empty() {
        blocking_findings.push(json!({
            "code": "no_current_statements",
            "message": "No analytical statements are available for a final report."
        }));
    }
    if high_open_challenges > 0 {
        blocking_findings.push(json!({
            "code": "high_open_challenges",
            "count": high_open_challenges,
            "message": "Critical/error challenges remain open."
        }));
    }
    if strong_refutations > 0 {
        blocking_findings.push(json!({
            "code": "strong_refutations",
            "count": strong_refutations,
            "message": "Moderate or strong disproofs still require revision."
        }));
    }
    if wrong_or_unknown_high > 0 {
        blocking_findings.push(json!({
            "code": "unresolved_high_impact_fact_checks",
            "count": wrong_or_unknown_high,
            "message": "High-impact statements are wrong or unknown after fact-checking."
        }));
    }
    if measurement_claims_without_document_anchor > 0 {
        blocking_findings.push(json!({
            "code": "measurement_claims_without_document_anchor",
            "count": measurement_claims_without_document_anchor,
            "message": "Measurement or numeric claims need document, table, span, or cell anchors before publication-grade acceptance."
        }));
    }
    if claims_without_primary_source_evidence > 0 {
        blocking_findings.push(json!({
            "code": "claims_without_primary_source_evidence",
            "count": claims_without_primary_source_evidence,
            "message": "Current-position claims must be backed by non-generated primary source-card evidence before publication-grade acceptance."
        }));
    }
    if stale_current_statement_evidence > 0 {
        blocking_findings.push(json!({
            "code": "stale_current_statement_evidence",
            "count": stale_current_statement_evidence,
            "message": "Current-position statements rely on stale source-card evidence and need freshness verification before publication-grade acceptance."
        }));
    }
    let mut non_blocking_findings = Vec::new();
    if unsupported_statements > 0 {
        non_blocking_findings.push(json!({
            "code": "unsupported_low_impact_statements",
            "count": unsupported_statements,
            "message": "Some statements lack claim evidence and should stay out of final conclusions."
        }));
    }
    if open_host_search_tasks > 0 {
        non_blocking_findings.push(json!({
            "code": "pending_host_search_tasks",
            "count": open_host_search_tasks,
            "message": "Challenge-linked host/provider search tasks remain pending; treat settlement as configured-blocker settlement, not publication-final closure."
        }));
    }
    if !status.settled {
        non_blocking_findings.push(json!({
            "code": "not_settled",
            "stop_reason": status.stop_reason,
            "message": "The convergence loop has not reached its settled stop rule."
        }));
    }
    let overall_decision = if !blocking_findings.is_empty() {
        "reject"
    } else if status.settled && non_blocking_findings.is_empty() {
        "accept"
    } else if status.settled {
        "accept_with_caveats"
    } else {
        "incomplete"
    };
    Ok(ResearchReportJudgment {
        id: research_report_judgment_id(run_id, report_id),
        run_id: run_id.to_string(),
        report_id: report_id.map(ToOwned::to_owned),
        judgment_version: "iterated-epistemic-convergence/v1".to_string(),
        overall_decision: overall_decision.to_string(),
        scores: json!({
            "settled": status.settled,
            "statement_count": current_statements.len(),
            "high_open_challenges": high_open_challenges,
            "strong_refutations": strong_refutations,
            "wrong_or_unknown_high_fact_checks": wrong_or_unknown_high,
            "unsupported_statements": unsupported_statements,
            "measurement_claims_without_document_anchor": measurement_claims_without_document_anchor,
            "claims_without_primary_source_evidence": claims_without_primary_source_evidence,
            "stale_current_statement_evidence": stale_current_statement_evidence,
            "pending_host_search_tasks": open_host_search_tasks,
        }),
        blocking_findings: Value::Array(blocking_findings),
        non_blocking_findings: Value::Array(non_blocking_findings),
        evidence_checked: json!({
            "iterations": status.latest_iteration.as_ref().map(|iteration| iteration.iteration_index),
            "current_statement_ids": status.current_statements.iter().map(|statement| statement.id.clone()).collect::<Vec<_>>(),
            "current_claim_ids": current_claim_ids.iter().cloned().collect::<Vec<_>>(),
            "fact_check_ids": fact_checks.iter().map(|check| check.id.clone()).collect::<Vec<_>>(),
        }),
        remaining_risks: json!({
            "requires_live_host_search_for_open_search_plans": challenges.iter().any(|challenge| {
                challenge
                    .search_plan
                    .get("requires_host_search_proof")
                    .and_then(Value::as_bool)
                    == Some(true)
            }),
            "deterministic_only": true
        }),
        commands_or_artifacts_reviewed: json!({
            "research_convergence_status": true,
            "statements_reviewed": current_statements.len(),
            "claims_reviewed": current_claim_records.len(),
            "source_cards_reviewed": source_card_by_id.len(),
            "challenges_reviewed": challenges.len(),
            "challenge_verifier_records_reviewed": disproofs.len(),
            "fact_checks_reviewed": fact_checks.len(),
        }),
        created_at: now(),
    })
}

pub(crate) fn research_claims_conflict(left: &ResearchClaim, right: &ResearchClaim) -> bool {
    let same_subject = normalized_optional_claim_part(left.subject.as_deref())
        .zip(normalized_optional_claim_part(right.subject.as_deref()))
        .is_some_and(|(left, right)| left == right);
    let same_predicate = normalized_optional_claim_part(left.predicate.as_deref())
        .zip(normalized_optional_claim_part(right.predicate.as_deref()))
        .is_some_and(|(left, right)| left == right);
    let different_object = normalized_optional_claim_part(left.object_value.as_deref())
        .zip(normalized_optional_claim_part(
            right.object_value.as_deref(),
        ))
        .is_some_and(|(left, right)| left != right);
    same_subject && same_predicate && different_object
}

pub(crate) fn normalized_optional_claim_part(value: Option<&str>) -> Option<String> {
    value
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
}
