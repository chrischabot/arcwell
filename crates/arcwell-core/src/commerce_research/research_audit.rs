use super::*;

#[derive(Debug, Clone, Default)]
pub(crate) struct ResearchFamilyCoverage {
    pub(crate) linked_sources: usize,
    pub(crate) source_cards: usize,
    pub(crate) primary_cards: usize,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ResearchCoverageSummary {
    pub(crate) linked_sources: usize,
    pub(crate) source_cards: usize,
    pub(crate) full_text_sources: usize,
    pub(crate) primary_cards: usize,
    pub(crate) secondary_cards: usize,
    pub(crate) generated_cards: usize,
    pub(crate) model_answer_cards: usize,
    pub(crate) high_trust_cards: usize,
    pub(crate) medium_trust_cards: usize,
    pub(crate) low_trust_cards: usize,
    pub(crate) untrusted_cards: usize,
    pub(crate) families: BTreeMap<String, ResearchFamilyCoverage>,
}

pub(crate) fn summarize_research_coverage(
    sources: &[ResearchRunSourceRecord],
) -> ResearchCoverageSummary {
    let mut summary = ResearchCoverageSummary {
        linked_sources: sources.len(),
        ..Default::default()
    };
    let mut seen_cards = BTreeSet::new();
    for record in sources {
        if record.link.read_depth == "full-text" {
            summary.full_text_sources += 1;
        }
        let family = if record.source.source_family.trim().is_empty() {
            "uncategorized".to_string()
        } else {
            record.source.source_family.clone()
        };
        let family_stats = summary.families.entry(family).or_default();
        family_stats.linked_sources += 1;
        let Some(card) = &record.source_card else {
            continue;
        };
        family_stats.source_cards += 1;
        if !seen_cards.insert(card.id.clone()) {
            continue;
        }
        summary.source_cards += 1;
        let role = infer_source_role_from_card(card);
        match role.as_str() {
            "primary" => {
                summary.primary_cards += 1;
                family_stats.primary_cards += 1;
            }
            "model_answer" => summary.model_answer_cards += 1,
            "generated_synthesis" => summary.generated_cards += 1,
            _ => summary.secondary_cards += 1,
        }
        let trust = source_card_metadata_string(&card.metadata, "trust_level")
            .unwrap_or_else(|| "medium".to_string());
        match trust.as_str() {
            "high" => summary.high_trust_cards += 1,
            "low" => summary.low_trust_cards += 1,
            "untrusted" => summary.untrusted_cards += 1,
            _ => summary.medium_trust_cards += 1,
        }
    }
    summary
}

pub(crate) fn corpus_finding(
    severity: &str,
    code: &str,
    message: &str,
    evidence: impl Into<String>,
) -> ResearchAuditFinding {
    ResearchAuditFinding {
        severity: severity.to_string(),
        code: code.to_string(),
        source_card_id: None,
        message: message.to_string(),
        evidence: excerpt(&evidence.into(), 500),
    }
}

pub(crate) fn audit_research_run_corpus(
    sources: &[ResearchRunSourceRecord],
    claims: &[ResearchClaimRecord],
) -> Vec<ResearchAuditFinding> {
    let coverage = summarize_research_coverage(sources);
    let mut findings = Vec::new();
    if coverage.linked_sources == 0 {
        findings.push(corpus_finding(
            "error",
            "no_linked_sources",
            "Research run has no linked source ledger entries.",
            "linked_sources=0",
        ));
    } else if coverage.linked_sources < 25 {
        findings.push(corpus_finding(
            "warning",
            "thin_source_corpus",
            "Deep research corpus has fewer than 25 linked sources.",
            format!("linked_sources={}", coverage.linked_sources),
        ));
    }
    if coverage.source_cards == 0 {
        findings.push(corpus_finding(
            "error",
            "no_source_cards",
            "Research run has no source cards, so claims cannot be audited against typed evidence.",
            "source_cards=0",
        ));
    } else if coverage.linked_sources >= 50
        && (coverage.source_cards as f64 / coverage.linked_sources as f64) < 0.15
    {
        findings.push(corpus_finding(
            "warning",
            "low_carded_source_ratio",
            "Large corpus has too few source-carded sources for confident synthesis.",
            format!(
                "source_cards={} linked_sources={}",
                coverage.source_cards, coverage.linked_sources
            ),
        ));
    }
    if coverage.primary_cards == 0 {
        findings.push(corpus_finding(
            "error",
            "missing_primary_source",
            "Research run has no primary source cards.",
            "primary_cards=0",
        ));
    } else if coverage.primary_cards < 3 {
        findings.push(corpus_finding(
            "warning",
            "thin_primary_source_coverage",
            "Research run has fewer than three primary source cards.",
            format!("primary_cards={}", coverage.primary_cards),
        ));
    }
    if coverage.linked_sources >= 20 && coverage.families.len() < 4 {
        findings.push(corpus_finding(
            "warning",
            "narrow_source_family_coverage",
            "Deep research corpus covers fewer than four source families.",
            format!("families={}", coverage.families.len()),
        ));
    }
    if coverage.full_text_sources == 0 && coverage.source_cards > 0 {
        findings.push(corpus_finding(
            "warning",
            "no_full_text_sources",
            "No linked source was marked full-text/read deeply.",
            "full_text_sources=0",
        ));
    }
    if claims.is_empty() {
        findings.push(corpus_finding(
            "warning",
            "no_extracted_claims",
            "Research run has no structured claims, so the report will be source-led rather than claim-led.",
            "claims=0",
        ));
    } else if coverage.source_cards >= 10 && claims.len() < 10 {
        findings.push(corpus_finding(
            "warning",
            "low_extracted_claim_density",
            "Research run has a sizable carded corpus but fewer than ten extracted claims.",
            format!(
                "source_cards={} claims={}",
                coverage.source_cards,
                claims.len()
            ),
        ));
    }
    findings
}

pub(crate) fn audit_research_host_search_proof(
    sources: &[ResearchRunSourceRecord],
    searches: &[ResearchHostSearchRecord],
) -> Vec<ResearchAuditFinding> {
    let mut findings = Vec::new();
    let has_host_native_sources = sources.iter().any(|record| {
        let provider = record.source.provider.to_ascii_lowercase();
        provider == "host-native"
            || provider == "host"
            || provider == "native"
            || record.source.metadata.get("origin").and_then(Value::as_str)
                == Some("host_search_record")
    });
    if has_host_native_sources && searches.is_empty() {
        findings.push(corpus_finding(
            "error",
            "missing_host_search_proof",
            "Run has host-native source rows but no recorded host search proof.",
            "provider=host-native without research_host_searches",
        ));
    }
    for record in searches {
        if record.results.is_empty() {
            findings.push(corpus_finding(
                "error",
                "host_search_without_results",
                "Recorded host search has no result rows.",
                format!("host_search_id={}", record.search.id),
            ));
            continue;
        }
        let selected: Vec<&ResearchHostSearchResult> = record
            .results
            .iter()
            .filter(|result| result.selected_for_ingest)
            .collect();
        if selected.is_empty() {
            findings.push(corpus_finding(
                "warning",
                "host_search_no_selected_results",
                "Recorded host search did not select any results for ingestion.",
                format!(
                    "host_search_id={} query={}",
                    record.search.id, record.search.query
                ),
            ));
        }
        for result in &selected {
            if result.research_source_id.is_none() {
                findings.push(corpus_finding(
                    "error",
                    "host_search_selected_result_unlinked",
                    "Selected host search result is not linked to a research source.",
                    format!(
                        "host_search_id={} rank={} url={}",
                        record.search.id, result.rank, result.canonical_url
                    ),
                ));
            }
        }
        if !selected.is_empty()
            && selected
                .iter()
                .all(|result| result.research_source_id.is_none())
        {
            findings.push(corpus_finding(
                "error",
                "host_search_zero_linked_sources",
                "Recorded host search selected results but none became run-linked sources.",
                format!("host_search_id={}", record.search.id),
            ));
        }
        let domains: BTreeSet<String> = record
            .results
            .iter()
            .filter_map(|result| host_from_url(&result.canonical_url))
            .collect();
        if record.results.len() >= 5 && domains.len() <= 1 {
            findings.push(corpus_finding(
                "warning",
                "host_search_single_domain_results",
                "Broad host search result set came from only one domain.",
                format!("host_search_id={} domains={:?}", record.search.id, domains),
            ));
        }
    }
    findings
}

pub(crate) fn audit_research_document_anchors(
    claims: &[ResearchClaimRecord],
    documents: &[ResearchDocumentRecord],
) -> Vec<ResearchAuditFinding> {
    let mut findings = Vec::new();
    let document_by_id: BTreeMap<&str, &ResearchDocumentRecord> = documents
        .iter()
        .map(|document| (document.document.id.as_str(), document))
        .collect();
    for record in claims {
        if record.claim.kind == "measurement" && record.document_anchors.is_empty() {
            findings.push(corpus_finding(
                "warning",
                "measurement_claim_without_document_anchor",
                "Measurement claim has no document/table/span anchor.",
                format!("claim_id={}", record.claim.id),
            ));
        }
        for anchor in &record.document_anchors {
            let Some(document) = document_by_id.get(anchor.document_id.as_str()) else {
                findings.push(corpus_finding(
                    "error",
                    "document_anchor_missing_document",
                    "Claim document anchor points at a missing document artifact.",
                    format!(
                        "claim_id={} anchor={} document_id={}",
                        record.claim.id, anchor.anchor_label, anchor.document_id
                    ),
                ));
                continue;
            };
            if !document.document.warning_flags.is_empty() {
                findings.push(corpus_finding(
                    "warning",
                    "document_anchor_warned_extraction",
                    "Claim document anchor uses a document extraction with warnings.",
                    format!(
                        "claim_id={} anchor={} warnings={}",
                        record.claim.id,
                        anchor.anchor_label,
                        document.document.warning_flags.join(",")
                    ),
                ));
            }
            let anchored_table = anchor.table_id.as_ref().and_then(|table_db_id| {
                document
                    .tables
                    .iter()
                    .find(|table| table.table.id == *table_db_id)
            });
            if let Some(table) = anchored_table {
                if table.table.confidence < 0.85
                    || table
                        .table
                        .warning_flags
                        .iter()
                        .any(|flag| flag == "pdf_layout_table_heuristic")
                {
                    findings.push(corpus_finding(
                        "warning",
                        "document_anchor_low_confidence_table",
                        "Claim document anchor uses a low-confidence or heuristic table extraction.",
                        format!(
                            "claim_id={} anchor={} table_confidence={:.2} warnings={}",
                            record.claim.id,
                            anchor.anchor_label,
                            table.table.confidence,
                            table.table.warning_flags.join(",")
                        ),
                    ));
                }
                if let Some(cell_id) = &anchor.table_cell_id
                    && let Some(cell) = table.cells.iter().find(|cell| cell.id == *cell_id)
                    && cell.confidence < 0.85
                {
                    findings.push(corpus_finding(
                        "warning",
                        "document_anchor_low_confidence_cell",
                        "Claim document anchor uses a low-confidence table cell.",
                        format!(
                            "claim_id={} anchor={} cell_confidence={:.2}",
                            record.claim.id, anchor.anchor_label, cell.confidence
                        ),
                    ));
                }
            }
        }
    }
    findings
}

pub(crate) fn audit_research_editorial_gates(
    runs: &[ResearchEditorialRun],
) -> Vec<ResearchAuditFinding> {
    let mut findings = Vec::new();
    if runs.is_empty() {
        return findings;
    }

    for run in runs {
        if matches!(run.status.as_str(), "failed" | "rejected") {
            findings.push(corpus_finding(
                "warning",
                "editorial_stage_failed_or_rejected",
                "A model-backed editorial/eval stage failed or rejected its input.",
                format!(
                    "editorial_run_id={} stage={} status={} error={}",
                    run.id,
                    run.stage,
                    run.status,
                    run.error_message_redacted.as_deref().unwrap_or("")
                ),
            ));
        }
        if editorial_stage_passed(run) && run.output_artifact_id.is_none() {
            findings.push(corpus_finding(
                "error",
                "editorial_stage_missing_output_artifact",
                "A completed editorial/eval stage has no inspectable output artifact.",
                format!("editorial_run_id={} stage={}", run.id, run.stage),
            ));
        }
    }

    let drafts: Vec<&ResearchEditorialRun> = runs
        .iter()
        .filter(|run| run.stage == "editorial_drafter" && editorial_stage_passed(run))
        .collect();
    if drafts.is_empty() {
        let has_eval = runs.iter().any(|run| {
            matches!(
                run.stage.as_str(),
                "citation_verifier" | "adversarial_evaluator"
            )
        });
        if has_eval {
            findings.push(corpus_finding(
                "error",
                "editorial_eval_without_completed_draft",
                "Editorial verification/evaluation exists without a completed synthesis draft.",
                format!("editorial_runs={}", runs.len()),
            ));
        }
        return findings;
    }

    let verifiers: Vec<&ResearchEditorialRun> = runs
        .iter()
        .filter(|run| run.stage == "citation_verifier" && editorial_stage_passed(run))
        .collect();
    let evaluators: Vec<&ResearchEditorialRun> = runs
        .iter()
        .filter(|run| run.stage == "adversarial_evaluator" && editorial_stage_passed(run))
        .collect();

    for draft in drafts {
        if draft.input_artifact_id.is_none() {
            findings.push(corpus_finding(
                "error",
                "editorial_draft_missing_evidence_pack",
                "Completed editorial draft is not linked to an evidence-pack input artifact.",
                format!("editorial_run_id={}", draft.id),
            ));
        }
        let Some(draft_output_id) = draft.output_artifact_id.as_deref() else {
            continue;
        };

        let draft_verifiers: Vec<&ResearchEditorialRun> = verifiers
            .iter()
            .copied()
            .filter(|run| run.input_artifact_id.as_deref() == Some(draft_output_id))
            .collect();
        if draft_verifiers.is_empty() {
            findings.push(corpus_finding(
                "error",
                "missing_citation_verifier",
                "Completed editorial draft has no completed citation-verifier run against its output.",
                format!("draft_editorial_run_id={} draft_artifact_id={draft_output_id}", draft.id),
            ));
        }
        for verifier in draft_verifiers {
            findings.extend(audit_citation_verifier_score(verifier));
        }

        let verifier_outputs: BTreeSet<&str> = verifiers
            .iter()
            .filter(|run| run.input_artifact_id.as_deref() == Some(draft_output_id))
            .filter_map(|run| run.output_artifact_id.as_deref())
            .collect();
        let draft_evaluators: Vec<&ResearchEditorialRun> = evaluators
            .iter()
            .copied()
            .filter(|run| {
                let input_id = run.input_artifact_id.as_deref();
                input_id == Some(draft_output_id)
                    || input_id
                        .map(|id| verifier_outputs.contains(id))
                        .unwrap_or(false)
            })
            .collect();
        if draft_evaluators.is_empty() {
            findings.push(corpus_finding(
                "error",
                "missing_adversarial_evaluator",
                "Completed editorial draft has no completed adversarial-evaluator run against the draft or verified artifact.",
                format!("draft_editorial_run_id={} draft_artifact_id={draft_output_id}", draft.id),
            ));
        }
        for evaluator in draft_evaluators {
            findings.extend(audit_adversarial_evaluator_score(evaluator));
        }
    }

    findings
}

pub(crate) fn editorial_stage_passed(run: &ResearchEditorialRun) -> bool {
    matches!(run.status.as_str(), "completed" | "accepted")
}

pub(crate) fn audit_citation_verifier_score(
    run: &ResearchEditorialRun,
) -> Vec<ResearchAuditFinding> {
    let mut findings = Vec::new();
    let unsupported_count = editorial_score_number(
        &run.score,
        &[
            "unsupported_factual_sentences",
            "unsupported_claims",
            "uncited_claims",
            "unsupported_count",
        ],
    );
    if unsupported_count.unwrap_or(0.0) > 0.0 {
        findings.push(corpus_finding(
            "error",
            "unsupported_factual_sentences",
            "Citation verifier found unsupported or uncited factual content.",
            format!("editorial_run_id={} score={}", run.id, run.score),
        ));
    }
    let unsupported_rate = editorial_score_number(
        &run.score,
        &[
            "unsupported_factual_sentence_rate",
            "unsupported_claim_rate",
            "uncited_claim_rate",
            "unsupported_rate",
        ],
    );
    if unsupported_rate.unwrap_or(0.0) > 0.0 {
        findings.push(corpus_finding(
            "error",
            "unsupported_factual_sentence_rate",
            "Citation verifier found a non-zero unsupported factual sentence rate.",
            format!("editorial_run_id={} score={}", run.id, run.score),
        ));
    }
    if editorial_score_bool(&run.score, &["valid_citations", "citation_integrity"]) == Some(false) {
        findings.push(corpus_finding(
            "error",
            "invalid_editorial_citations",
            "Citation verifier rejected the draft's citation integrity.",
            format!("editorial_run_id={} score={}", run.id, run.score),
        ));
    }
    if unsupported_count.is_none()
        && unsupported_rate.is_none()
        && editorial_score_bool(&run.score, &["valid_citations", "citation_integrity"]).is_none()
    {
        findings.push(corpus_finding(
            "warning",
            "citation_verifier_missing_score",
            "Citation verifier completed without structured citation-integrity scores.",
            format!("editorial_run_id={} score={}", run.id, run.score),
        ));
    }
    findings
}

pub(crate) fn audit_adversarial_evaluator_score(
    run: &ResearchEditorialRun,
) -> Vec<ResearchAuditFinding> {
    let mut findings = Vec::new();
    if editorial_score_bool(&run.score, &["passed", "ok", "accepted"]) == Some(false) {
        findings.push(corpus_finding(
            "error",
            "editorial_evaluator_rejected",
            "Adversarial evaluator rejected the model-backed research draft.",
            format!("editorial_run_id={} score={}", run.id, run.score),
        ));
    }
    let issue_count_present = editorial_score_any_number(
        &run.score,
        &[
            "unsupported_conclusions",
            "missing_caveats",
            "narrative_overreach",
            "overclaim_count",
            "blocking_issue_count",
        ],
    );
    if editorial_score_any_positive_number(
        &run.score,
        &[
            "unsupported_conclusions",
            "missing_caveats",
            "narrative_overreach",
            "overclaim_count",
            "blocking_issue_count",
        ],
    ) {
        findings.push(corpus_finding(
            "error",
            "editorial_evaluator_found_blocking_issues",
            "Adversarial evaluator found unsupported conclusions, weak evidence, missing caveats, or narrative overreach.",
            format!("editorial_run_id={} score={}", run.id, run.score),
        ));
    }
    if editorial_score_any_positive_number(&run.score, &["weak_evidence"]) {
        findings.push(corpus_finding(
            "warning",
            "editorial_evaluator_weak_evidence",
            "Adversarial evaluator found weak evidence that must remain caveated.",
            format!("editorial_run_id={} score={}", run.id, run.score),
        ));
    }
    let score = editorial_score_number(
        &run.score,
        &[
            "score",
            "analyst_usefulness_score",
            "quality_score",
            "final_score",
        ],
    );
    if score.map(|value| value < 0.75).unwrap_or(false) {
        findings.push(corpus_finding(
            "error",
            "editorial_evaluator_score_below_gate",
            "Adversarial evaluator score is below the analyst-grade acceptance gate.",
            format!("editorial_run_id={} score={}", run.id, run.score),
        ));
    }
    if score.is_none()
        && !issue_count_present
        && editorial_score_bool(&run.score, &["passed", "ok", "accepted"]).is_none()
    {
        findings.push(corpus_finding(
            "warning",
            "adversarial_evaluator_missing_score",
            "Adversarial evaluator completed without a structured pass/fail or quality score.",
            format!("editorial_run_id={} score={}", run.id, run.score),
        ));
    }
    findings
}

pub(crate) fn citation_verifier_passed(invocation: &ResearchEditorialInvocation) -> bool {
    editorial_stage_passed(&invocation.editorial_run)
        && invocation.editorial_run.output_artifact_id.is_some()
        && audit_citation_verifier_score(&invocation.editorial_run).is_empty()
}

pub(crate) fn adversarial_evaluator_passed(invocation: &ResearchEditorialInvocation) -> bool {
    editorial_stage_passed(&invocation.editorial_run)
        && invocation.editorial_run.output_artifact_id.is_some()
        && audit_adversarial_evaluator_score(&invocation.editorial_run)
            .into_iter()
            .all(|finding| finding.severity != "error")
}

pub(crate) fn editorial_score_number(score: &Value, keys: &[&str]) -> Option<f64> {
    keys.iter()
        .find_map(|key| score.get(*key).and_then(Value::as_f64))
}

pub(crate) fn editorial_score_any_number(score: &Value, keys: &[&str]) -> bool {
    keys.iter()
        .any(|key| score.get(*key).and_then(Value::as_f64).is_some())
}

pub(crate) fn editorial_score_any_positive_number(score: &Value, keys: &[&str]) -> bool {
    keys.iter()
        .filter_map(|key| score.get(*key).and_then(Value::as_f64))
        .any(|value| value > 0.0)
}

pub(crate) fn editorial_score_bool(score: &Value, keys: &[&str]) -> Option<bool> {
    keys.iter()
        .find_map(|key| score.get(*key).and_then(Value::as_bool))
}

pub(crate) fn merge_json_objects(left: Value, right: Value) -> Value {
    let mut merged = left.as_object().cloned().unwrap_or_default();
    if let Some(right) = right.as_object() {
        for (key, value) in right {
            merged.insert(key.clone(), value.clone());
        }
    }
    Value::Object(merged)
}

pub(crate) fn host_from_url(raw: &str) -> Option<String> {
    Url::parse(raw)
        .ok()
        .and_then(|url| url.host_str().map(str::to_ascii_lowercase))
}

pub(crate) fn research_audit_checklist(findings: &[ResearchAuditFinding]) -> Vec<String> {
    let has_error = findings.iter().any(|finding| finding.severity == "error");
    let has_stale = findings
        .iter()
        .any(|finding| finding.code == "stale_source");
    let has_contradiction = findings
        .iter()
        .any(|finding| finding.code.contains("contradict"));
    let has_untrusted = findings
        .iter()
        .any(|finding| finding.code == "untrusted_evidence");
    let has_thin_corpus = findings.iter().any(|finding| {
        matches!(
            finding.code.as_str(),
            "thin_source_corpus"
                | "low_carded_source_ratio"
                | "narrow_source_family_coverage"
                | "thin_primary_source_coverage"
        )
    });
    let has_claim_gap = findings.iter().any(|finding| {
        matches!(
            finding.code.as_str(),
            "no_extracted_claims" | "low_extracted_claim_density"
        )
    });
    vec![
        format!(
            "{} primary evidence is not generated or uncited model output",
            if has_error { "FAIL" } else { "PASS" }
        ),
        format!(
            "{} corpus depth and source-family coverage meet deep-research expectations",
            if has_thin_corpus { "WARN" } else { "PASS" }
        ),
        format!(
            "{} structured claims exist at a useful density for synthesis",
            if has_claim_gap { "WARN" } else { "PASS" }
        ),
        format!(
            "{} contradictions are surfaced explicitly",
            if has_contradiction { "FAIL" } else { "PASS" }
        ),
        format!(
            "{} stale source dates are flagged",
            if has_stale { "WARN" } else { "PASS" }
        ),
        format!(
            "{} untrusted/prompt-injection/SEO evidence is labeled",
            if has_untrusted { "WARN" } else { "PASS" }
        ),
        "CHECK cite source-card URLs and primary source links for every externally used claim"
            .to_string(),
    ]
}
