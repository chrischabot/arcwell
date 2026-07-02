use super::*;

pub(crate) fn normalize_commerce_run_config_input(
    mut input: CommerceRunConfigInput,
) -> Result<CommerceRunConfigInput> {
    validate_id(&input.run_id)?;
    input.domain_profile = normalize_research_key(input.domain_profile, "commerce domain profile")?;
    if !(1..=COMMERCE_MAX_TARGET_QUALIFIED).contains(&input.target_qualified_count) {
        bail!(
            "commerce target_qualified_count must be between 1 and {COMMERCE_MAX_TARGET_QUALIFIED}"
        );
    }
    input.geography = normalize_optional_research_text(input.geography, "geography", 200)?;
    input.freshness_window =
        normalize_research_key(input.freshness_window, "commerce freshness window")?;
    input.allowed_private_context_sources = normalize_commerce_key_list(
        input.allowed_private_context_sources,
        "private context source",
    )?;
    input.allowed_public_source_families =
        normalize_commerce_key_list(input.allowed_public_source_families, "public source family")?;
    if let Some(max_provider_calls) = input.max_provider_calls
        && max_provider_calls > COMMERCE_MAX_PROVIDER_CALLS
    {
        bail!("commerce max_provider_calls is too large");
    }
    if let Some(max_browser_pages) = input.max_browser_pages
        && max_browser_pages > COMMERCE_MAX_BROWSER_PAGES
    {
        bail!("commerce max_browser_pages is too large");
    }
    if let Some(cost) = input.max_cost_usd {
        validate_non_negative_cost(cost, "commerce max_cost_usd")?;
    }
    input.stop_rules = sanitize_work_json(input.stop_rules)?;
    Ok(input)
}

pub(crate) fn normalize_commerce_candidate_input(
    mut input: CommerceCandidateInput,
) -> Result<CommerceCandidateInput> {
    validate_id(&input.run_id)?;
    input.domain = normalize_research_key(input.domain, "commerce domain")?;
    validate_fetch_url(input.source_url.trim())?;
    input.source_url = canonical_source_url(input.source_url.trim())?;
    input.retailer_or_provider =
        sanitize_required_commerce_text(&input.retailer_or_provider, "retailer_or_provider", 300)?;
    input.title = sanitize_required_commerce_text(&input.title, "title", 500)?;
    input.normalized_item_key =
        sanitize_required_commerce_text(&input.normalized_item_key, "normalized_item_key", 1_000)?;
    input.variant_key = sanitize_required_commerce_text(&input.variant_key, "variant_key", 1_000)?;
    input.price = normalize_optional_research_text(input.price, "price", 100)?;
    input.currency = normalize_optional_research_text(input.currency, "currency", 20)?;
    input.geography = normalize_optional_research_text(input.geography, "geography", 200)?;
    input.candidate_status = normalize_commerce_candidate_status(&input.candidate_status)?;
    if let Some(score) = input.score {
        validate_score_range(score, "commerce candidate score")?;
    }
    input.score_reasons = sanitize_work_json(input.score_reasons)?;
    input.disqualification_reasons = sanitize_work_json(input.disqualification_reasons)?;
    input.metadata = sanitize_work_json(input.metadata)?;
    Ok(input)
}

pub(crate) fn normalize_commerce_availability_proof_input(
    mut input: CommerceAvailabilityProofInput,
) -> Result<CommerceAvailabilityProofInput> {
    validate_id(&input.run_id)?;
    validate_id(&input.candidate_id)?;
    input.proof_method = normalize_commerce_proof_method(&input.proof_method)?;
    input.variant_key = sanitize_required_commerce_text(&input.variant_key, "variant_key", 1_000)?;
    input.variant_label =
        sanitize_required_commerce_text(&input.variant_label, "variant_label", 300)?;
    input.availability_state = normalize_commerce_availability_state(&input.availability_state)?;
    input.visible_evidence = normalize_optional_research_text(
        input.visible_evidence,
        "visible_evidence",
        COMMERCE_MAX_EVIDENCE_TEXT,
    )?;
    if input.availability_state == "available" && input.visible_evidence.is_none() {
        bail!("available commerce proof requires visible evidence");
    }
    input.selector_or_dom_hint = normalize_optional_research_text(
        input.selector_or_dom_hint,
        "selector_or_dom_hint",
        COMMERCE_MAX_OPTIONAL_TEXT,
    )?;
    input.screenshot_artifact_id =
        normalize_optional_id(input.screenshot_artifact_id, "screenshot_artifact_id")?;
    input.page_snapshot_artifact_id =
        normalize_optional_id(input.page_snapshot_artifact_id, "page_snapshot_artifact_id")?;
    validate_score_range(input.confidence, "commerce proof confidence")?;
    input.caveats = sanitize_work_json(input.caveats)?;
    input.checked_at = normalize_optional_research_text(input.checked_at, "checked_at", 100)?;
    Ok(input)
}

pub(crate) fn normalize_commerce_context_fact_input(
    mut input: CommerceContextFactInput,
) -> Result<CommerceContextFactInput> {
    validate_id(&input.run_id)?;
    input.fact_key = normalize_research_key(input.fact_key, "commerce context fact key")?;
    input.fact_kind = normalize_commerce_fact_kind(&input.fact_kind)?;
    input.redacted_value =
        sanitize_required_commerce_text(&input.redacted_value, "redacted_value", 2_000)?;
    input.source_family = normalize_research_key(input.source_family, "commerce source family")?;
    input.source_ref = normalize_optional_research_text(input.source_ref, "source_ref", 1_000)?;
    validate_score_range(input.confidence, "commerce context confidence")?;
    input.metadata = sanitize_work_json(input.metadata)?;
    Ok(input)
}

pub(crate) fn normalize_commerce_verification_attempt_input(
    mut input: CommerceVerificationAttemptInput,
) -> Result<CommerceVerificationAttemptInput> {
    validate_id(&input.run_id)?;
    validate_id(&input.candidate_id)?;
    input.method = normalize_commerce_proof_method(&input.method)?;
    input.result = normalize_commerce_verification_result(&input.result)?;
    input.error_kind = input
        .error_kind
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            validate_key(value)?;
            Ok::<String, anyhow::Error>(value.to_string())
        })
        .transpose()?;
    input.final_url = input
        .final_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            validate_fetch_url(value)?;
            canonical_source_url(value)
        })
        .transpose()?;
    if let Some(status) = input.http_status
        && !(100..=599).contains(&status)
    {
        bail!("commerce verification http_status must be between 100 and 599");
    }
    if input.result == "blocked" && input.next_action.as_deref().unwrap_or("").trim().is_empty() {
        bail!("blocked commerce verification requires a next_action");
    }
    if input.artifact_ids.len() > COMMERCE_MAX_LIST_ITEMS {
        bail!("too many commerce verification artifacts");
    }
    let mut artifact_ids = Vec::new();
    for artifact_id in input.artifact_ids {
        validate_id(&artifact_id)?;
        if !artifact_ids.contains(&artifact_id) {
            artifact_ids.push(artifact_id);
        }
    }
    input.artifact_ids = artifact_ids;
    input.next_action = normalize_optional_research_text(
        input.next_action,
        "next_action",
        COMMERCE_MAX_OPTIONAL_TEXT,
    )?;
    input.attempted_at = normalize_optional_research_text(input.attempted_at, "attempted_at", 100)?;
    Ok(input)
}

pub(crate) fn normalize_commerce_report_judgment_input(
    mut input: CommerceReportJudgmentInput,
) -> Result<CommerceReportJudgmentInput> {
    validate_id(&input.run_id)?;
    input.decision = normalize_commerce_report_decision(&input.decision)?;
    input.blocking_findings = sanitize_work_json(input.blocking_findings)?;
    input.non_blocking_findings = sanitize_work_json(input.non_blocking_findings)?;
    input.claims_checked = sanitize_work_json(input.claims_checked)?;
    input.availability_proofs_checked = sanitize_work_json(input.availability_proofs_checked)?;
    input.privacy_review = sanitize_work_json(input.privacy_review)?;
    input.remaining_risks = sanitize_work_json(input.remaining_risks)?;
    if input.decision == "accept"
        && input
            .blocking_findings
            .as_array()
            .map(|items| !items.is_empty())
            .unwrap_or(false)
    {
        bail!("accepted commerce report judgment cannot include blocking findings");
    }
    Ok(input)
}

pub(crate) fn normalize_commerce_rendered_page_check_input(
    mut input: CommerceRenderedPageCheckInput,
) -> Result<CommerceRenderedPageCheckInput> {
    validate_id(&input.run_id)?;
    validate_id(&input.candidate_id)?;
    input.variant_key = sanitize_required_commerce_text(
        &input.variant_key,
        "variant_key",
        COMMERCE_MAX_OPTIONAL_TEXT,
    )?;
    input.variant_label = sanitize_required_commerce_text(
        &input.variant_label,
        "variant_label",
        COMMERCE_MAX_OPTIONAL_TEXT,
    )?;
    input.selector_or_dom_hint = normalize_optional_research_text(
        input.selector_or_dom_hint,
        "selector_or_dom_hint",
        COMMERCE_MAX_OPTIONAL_TEXT,
    )?;
    validate_rendered_page_snapshot_input(&input.snapshot)?;
    Ok(input)
}

#[derive(Debug, Clone)]
pub(crate) struct CommerceRenderedAvailability {
    pub(crate) availability_state: String,
    pub(crate) visible_evidence: Option<String>,
    pub(crate) confidence: f64,
    pub(crate) caveats: Value,
    pub(crate) next_action: String,
}

#[derive(Debug, Clone)]
pub(crate) struct CommerceRenderedStructuredFields {
    pub(crate) price: Option<String>,
    pub(crate) currency: Option<String>,
    pub(crate) shipping_caveat: Option<String>,
}

pub(crate) fn extract_commerce_rendered_structured_fields(
    readable_text: &str,
) -> CommerceRenderedStructuredFields {
    let text = normalize_readable_text(readable_text);
    let price = extract_visible_price(&text);
    let currency = price.as_ref().and_then(|price| {
        if price.contains('£') || price.to_ascii_uppercase().contains("GBP") {
            Some("GBP".to_string())
        } else if price.contains('$') {
            Some("USD".to_string())
        } else if price.contains('€') || price.to_ascii_uppercase().contains("EUR") {
            Some("EUR".to_string())
        } else {
            None
        }
    });
    let shipping_caveat = extract_shipping_caveat(&text);
    CommerceRenderedStructuredFields {
        price,
        currency,
        shipping_caveat,
    }
}

pub(crate) fn extract_shipping_caveat(text: &str) -> Option<String> {
    [
        "free standard delivery",
        "free delivery",
        "delivery",
        "returns",
        "click & collect",
        "collection",
        "ships to",
        "shipping",
    ]
    .iter()
    .filter_map(|marker| {
        text.to_ascii_lowercase().find(marker).map(|index| {
            excerpt(
                &commerce_text_span(text, index, index + marker.len(), 24, 220),
                300,
            )
        })
    })
    .next()
}

pub(crate) fn extract_visible_price(text: &str) -> Option<String> {
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if let Some(index) = line.find('£') {
            return Some(extract_price_from_line(line, index, "£"));
        }
        if let Some(index) = line.to_ascii_uppercase().find("GBP") {
            return Some(extract_price_from_line(line, index, "GBP"));
        }
        if let Some(index) = line.find('$') {
            return Some(extract_price_from_line(line, index, "$"));
        }
        if let Some(index) = line.find('€') {
            return Some(extract_price_from_line(line, index, "€"));
        }
    }
    None
}

pub(crate) fn extract_price_from_line(line: &str, marker_index: usize, marker: &str) -> String {
    let after = &line[marker_index + marker.len()..];
    let amount: String = after
        .chars()
        .skip_while(|ch| ch.is_whitespace())
        .take_while(|ch| ch.is_ascii_digit() || *ch == '.' || *ch == ',')
        .collect();
    if amount.is_empty() {
        excerpt(line, 80)
    } else if marker == "GBP" {
        format!("GBP {amount}")
    } else {
        format!("{marker}{amount}")
    }
}

pub(crate) fn commerce_rendered_source_card_summary(
    candidate: &CommerceCandidate,
    input: &CommerceRenderedPageCheckInput,
    rendered: &CommerceRenderedAvailability,
    structured: &CommerceRenderedStructuredFields,
    doc: &UrlIngestDocument,
) -> String {
    let mut parts = vec![format!(
        "Host-supplied rendered product/listing page for {} checked exact variant {}.",
        candidate.title, input.variant_label
    )];
    parts.push(format!(
        "Observed availability state: {}.",
        rendered.availability_state
    ));
    if let Some(price) = &structured.price {
        parts.push(format!("Visible price: {price}."));
    }
    if let Some(shipping) = &structured.shipping_caveat {
        parts.push(format!("Shipping/returns caveat: {shipping}."));
    }
    parts.push(format!(
        "Final URL: {}. Arcwell did not perform a browser or network fetch for this evidence; the host supplied the rendered capture.",
        doc.final_url
    ));
    redact_secret_like_text(&parts.join(" "))
}

pub(crate) fn commerce_rendered_source_card_claims(
    candidate: &CommerceCandidate,
    input: &CommerceRenderedPageCheckInput,
    rendered: &CommerceRenderedAvailability,
    structured: &CommerceRenderedStructuredFields,
) -> Vec<SourceClaim> {
    let mut claims = vec![SourceClaim {
        claim: format!(
            "{} exact variant {} was observed as {} on the rendered page.",
            candidate.title, input.variant_label, rendered.availability_state
        ),
        kind: "commerce_availability".to_string(),
        confidence: rendered.confidence,
    }];
    if let Some(price) = &structured.price {
        claims.push(SourceClaim {
            claim: format!("Visible price was {price}."),
            kind: "commerce_price".to_string(),
            confidence: 0.75,
        });
    }
    if let Some(shipping) = &structured.shipping_caveat {
        claims.push(SourceClaim {
            claim: format!("Visible shipping or returns caveat: {shipping}."),
            kind: "commerce_shipping".to_string(),
            confidence: 0.65,
        });
    }
    claims
}

#[derive(Debug, Clone)]
pub(crate) struct CommerceReportCandidate {
    pub(crate) candidate: CommerceCandidate,
    pub(crate) proof: Option<CommerceAvailabilityProof>,
    pub(crate) latest_attempt: Option<CommerceVerificationAttempt>,
}

#[derive(Debug, Clone)]
pub(crate) struct CommerceReportModel {
    pub(crate) recommended: Vec<CommerceReportCandidate>,
    pub(crate) unavailable: Vec<CommerceReportCandidate>,
    pub(crate) blocked: Vec<CommerceReportCandidate>,
    pub(crate) unknown: Vec<CommerceReportCandidate>,
    pub(crate) context_facts: Vec<CommerceContextFact>,
}

pub(crate) fn build_commerce_report_model(
    candidates: &[CommerceCandidate],
    proofs: &[CommerceAvailabilityProof],
    attempts: &[CommerceVerificationAttempt],
    context_facts: &[CommerceContextFact],
) -> CommerceReportModel {
    let mut latest_proofs: BTreeMap<String, CommerceAvailabilityProof> = BTreeMap::new();
    for proof in proofs {
        latest_proofs
            .entry(proof.candidate_id.clone())
            .and_modify(|existing| {
                if proof.checked_at > existing.checked_at {
                    *existing = proof.clone();
                }
            })
            .or_insert_with(|| proof.clone());
    }
    let mut latest_attempts: BTreeMap<String, CommerceVerificationAttempt> = BTreeMap::new();
    for attempt in attempts {
        latest_attempts
            .entry(attempt.candidate_id.clone())
            .and_modify(|existing| {
                if attempt.attempted_at > existing.attempted_at {
                    *existing = attempt.clone();
                }
            })
            .or_insert_with(|| attempt.clone());
    }
    let mut model = CommerceReportModel {
        recommended: Vec::new(),
        unavailable: Vec::new(),
        blocked: Vec::new(),
        unknown: Vec::new(),
        context_facts: context_facts.to_vec(),
    };
    for candidate in candidates {
        let proof = latest_proofs.get(&candidate.id).cloned();
        let latest_attempt = latest_attempts.get(&candidate.id).cloned();
        let row = CommerceReportCandidate {
            candidate: candidate.clone(),
            proof: proof.clone(),
            latest_attempt,
        };
        match proof
            .as_ref()
            .map(|proof| proof.availability_state.as_str())
        {
            Some("available") if candidate.candidate_status != "disqualified" => {
                model.recommended.push(row)
            }
            Some("unavailable") => model.unavailable.push(row),
            Some("blocked") => model.blocked.push(row),
            _ if candidate.candidate_status == "blocked" => model.blocked.push(row),
            _ if candidate.candidate_status == "disqualified" => model.unavailable.push(row),
            _ => model.unknown.push(row),
        }
    }
    model.recommended.sort_by(|left, right| {
        right
            .candidate
            .score
            .partial_cmp(&left.candidate.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(left.candidate.title.cmp(&right.candidate.title))
    });
    model
}

pub(crate) fn commerce_report_blocking_findings(
    model: &CommerceReportModel,
    config: &Option<CommerceRunConfig>,
) -> Vec<String> {
    let mut findings = Vec::new();
    if model.recommended.is_empty() {
        findings.push("No candidate has exact available-variant proof.".to_string());
    }
    if let Some(config) = config
        && !config.allowed_private_context_sources.is_empty()
        && model.context_facts.is_empty()
    {
        findings.push(
            "Run allowed private context sources, but no redacted context facts were recorded."
                .to_string(),
        );
    }
    if model.recommended.iter().any(|row| {
        row.proof
            .as_ref()
            .and_then(|proof| proof.visible_evidence.as_ref())
            .is_none()
    }) {
        findings
            .push("At least one recommended candidate lacks visible evidence text.".to_string());
    }
    if let Some(config) = config
        && model.recommended.len() < config.target_qualified_count
    {
        findings.push(format!(
            "Only {} recommended candidates reached proof, below target {}.",
            model.recommended.len(),
            config.target_qualified_count
        ));
    }
    findings
}

pub(crate) fn commerce_report_non_blocking_findings(
    model: &CommerceReportModel,
    source_links: &[ResearchRunSourceRecord],
) -> Vec<String> {
    let mut findings = Vec::new();
    if source_links.len() < model.recommended.len() {
        findings.push(
            "Fewer run-linked source cards than recommended candidates; source coverage is thin."
                .to_string(),
        );
    }
    if !model.unknown.is_empty() {
        findings.push(format!(
            "{} candidates remain unknown/unverified and are excluded from recommendations.",
            model.unknown.len()
        ));
    }
    if !model.blocked.is_empty() {
        findings.push(format!(
            "{} candidates were blocked during verification and are excluded from recommendations.",
            model.blocked.len()
        ));
    }
    findings
}

pub(crate) fn commerce_report_remaining_risks(
    model: &CommerceReportModel,
    _config: Option<&CommerceRunConfig>,
) -> Vec<String> {
    let mut risks =
        vec!["Availability is point-in-time evidence and may change after checked_at.".to_string()];
    if model.recommended.iter().any(|row| {
        row.proof
            .as_ref()
            .is_some_and(|proof| proof.confidence < 0.8)
    }) {
        risks.push("One or more recommendations have proof confidence below 0.8.".to_string());
    }
    risks
}

pub(crate) fn render_commerce_context_packet(
    run_id: &str,
    facts: &[CommerceContextFact],
) -> String {
    let mut markdown = String::new();
    markdown.push_str("# Qualified Commerce Context Packet\n\n");
    markdown.push_str(&format!(
        "- Run id: `{}`\n",
        escape_research_report_text(run_id)
    ));
    markdown.push_str("- Privacy policy: raw private sources are not copied here; facts are redacted before storage.\n\n");
    if facts.is_empty() {
        markdown.push_str("No context facts were recorded.\n");
        return markdown;
    }
    markdown.push_str("| Fact | Kind | Redacted value | Source | Confidence | User confirmed |\n");
    markdown.push_str("| --- | --- | --- | --- | ---: | --- |\n");
    for fact in facts {
        markdown.push_str(&format!(
            "| {} | {} | {} | {} | {:.2} | {} |\n",
            escape_research_report_text(&fact.fact_key),
            escape_research_report_text(&fact.fact_kind),
            escape_commerce_report_private_text(&fact.redacted_value),
            escape_research_report_text(&fact.source_family),
            fact.confidence,
            fact.user_confirmed
        ));
    }
    markdown
}

pub(crate) fn render_commerce_report(
    run_id: &str,
    config: Option<&CommerceRunConfig>,
    model: &CommerceReportModel,
    source_links: &[ResearchRunSourceRecord],
) -> String {
    let mut markdown = String::new();
    markdown.push_str("# Qualified Commerce Report\n\n");
    markdown.push_str(&format!(
        "- Run id: `{}`\n",
        escape_research_report_text(run_id)
    ));
    if let Some(config) = config {
        markdown.push_str(&format!(
            "- Domain profile: `{}`\n- Geography: `{}`\n- Target qualified count: `{}`\n",
            escape_research_report_text(&config.domain_profile),
            escape_research_report_text(config.geography.as_deref().unwrap_or("unknown")),
            config.target_qualified_count
        ));
    }
    markdown.push_str(&format!(
        "- Recommended with exact available proof: `{}`\n- Unavailable/disqualified: `{}`\n- Blocked: `{}`\n- Unknown/unverified: `{}`\n- Run-linked source records: `{}`\n\n",
        model.recommended.len(),
        model.unavailable.len(),
        model.blocked.len(),
        model.unknown.len(),
        source_links.len()
    ));
    markdown.push_str("## Main Recommendations\n\n");
    if model.recommended.is_empty() {
        markdown.push_str("No candidates are safe to recommend yet because none has exact available-variant proof.\n\n");
    } else {
        markdown.push_str(
            "| Retailer | Item | Price | Variant | Availability | Checked | Evidence | Caveats |\n",
        );
        markdown.push_str("| --- | --- | --- | --- | --- | --- | --- | --- |\n");
        for row in &model.recommended {
            markdown.push_str(&commerce_report_candidate_row(row));
        }
        markdown.push('\n');
    }
    render_commerce_report_section(
        &mut markdown,
        "Unavailable Or Disqualified",
        &model.unavailable,
    );
    render_commerce_report_section(&mut markdown, "Blocked", &model.blocked);
    render_commerce_report_section(&mut markdown, "Unknown Or Unverified", &model.unknown);
    markdown.push_str("## Context Used\n\n");
    if model.context_facts.is_empty() {
        markdown.push_str("No redacted context facts were recorded.\n\n");
    } else {
        for fact in &model.context_facts {
            markdown.push_str(&format!(
                "- `{}` from `{}`: {} (confidence {:.2})\n",
                escape_research_report_text(&fact.fact_key),
                escape_research_report_text(&fact.source_family),
                escape_commerce_report_private_text(&fact.redacted_value),
                fact.confidence
            ));
        }
        markdown.push('\n');
    }
    let blockers = commerce_report_blocking_findings(model, &config.cloned());
    if !blockers.is_empty() {
        markdown.push_str("## Blocking Findings\n\n");
        for finding in blockers {
            markdown.push_str(&format!("- {}\n", escape_research_report_text(&finding)));
        }
    }
    markdown
}

pub(crate) fn render_commerce_report_section(
    markdown: &mut String,
    title: &str,
    rows: &[CommerceReportCandidate],
) {
    markdown.push_str(&format!("## {}\n\n", escape_research_report_text(title)));
    if rows.is_empty() {
        markdown.push_str("None.\n\n");
        return;
    }
    markdown.push_str("| Retailer | Item | Variant | State | Last attempt / reason |\n");
    markdown.push_str("| --- | --- | --- | --- | --- |\n");
    for row in rows {
        let state = row
            .proof
            .as_ref()
            .map(|proof| proof.availability_state.as_str())
            .unwrap_or("unknown");
        let reason = row
            .latest_attempt
            .as_ref()
            .and_then(|attempt| attempt.next_action.as_deref())
            .or_else(|| {
                row.proof
                    .as_ref()
                    .and_then(|proof| proof.visible_evidence.as_deref())
            })
            .unwrap_or("No exact available proof.");
        markdown.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            escape_table_cell(&row.candidate.retailer_or_provider),
            escape_table_cell(&row.candidate.title),
            escape_table_cell(&row.candidate.variant_key),
            escape_table_cell(state),
            escape_table_cell(reason)
        ));
    }
    markdown.push('\n');
}

pub(crate) fn commerce_report_candidate_row(row: &CommerceReportCandidate) -> String {
    let proof = row
        .proof
        .as_ref()
        .expect("recommended commerce candidate has proof");
    format!(
        "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
        escape_table_cell(&row.candidate.retailer_or_provider),
        escape_table_cell(&row.candidate.title),
        escape_table_cell(row.candidate.price.as_deref().unwrap_or("unknown")),
        escape_table_cell(&proof.variant_label),
        escape_table_cell(&proof.availability_state),
        escape_table_cell(&proof.checked_at),
        escape_table_cell(proof.visible_evidence.as_deref().unwrap_or("")),
        escape_table_cell(&proof.caveats.to_string())
    )
}

pub(crate) fn escape_table_cell(input: &str) -> String {
    escape_research_report_text(input).replace('|', "\\|")
}

pub(crate) fn escape_commerce_report_private_text(input: &str) -> String {
    escape_research_report_text(&redact_secret_like_text(input))
}

pub(crate) fn classify_commerce_rendered_availability(
    readable_text: &str,
    variant_label: &str,
    chrome_profile_required: bool,
) -> Result<CommerceRenderedAvailability> {
    let text = sanitize_work_text(readable_text, 20_000)?;
    let lower = text.to_ascii_lowercase();
    let variant = variant_label.trim().to_ascii_lowercase();
    if variant.is_empty() {
        bail!("commerce rendered availability variant label cannot be empty");
    }
    let blocked_cues = [
        "captcha",
        "are you human",
        "access denied",
        "bot detection",
        "blocked",
        "enable javascript",
        "verify you are human",
    ];
    let negative_cues = [
        "sold out",
        "sold",
        "listing ended",
        "out of stock",
        "not available",
        "unavailable",
        "notify me",
        "currently unavailable",
        "no longer available",
        "disabled",
    ];
    let positive_cues = [
        "in stock",
        "available",
        "selectable",
        "add to bag",
        "add to basket",
        "add to cart",
        "buy it now",
        "buy now",
    ];
    if let Some(cue) = blocked_cues.iter().find(|cue| lower.contains(**cue)) {
        return Ok(CommerceRenderedAvailability {
            availability_state: "blocked".to_string(),
            visible_evidence: Some(excerpt_window(&text, cue, 400)),
            confidence: 0.85,
            caveats: json!([format!("Rendered page contains blocker cue: {cue}")]),
            next_action: if chrome_profile_required {
                "Retry with user Chrome profile or manual review.".to_string()
            } else {
                "Retry with rendered browser; escalate to Chrome profile if expected.".to_string()
            },
        });
    }
    let Some(variant_index) = lower.find(&variant) else {
        return Ok(CommerceRenderedAvailability {
            availability_state: "unknown".to_string(),
            visible_evidence: None,
            confidence: 0.2,
            caveats: json!([format!(
                "Variant label {variant_label:?} was not visible in rendered text."
            )]),
            next_action:
                "Inspect the rendered page controls manually; exact variant was not visible."
                    .to_string(),
        });
    };
    let negative_window = commerce_text_window(&text, variant_index, 360);
    let negative_window_lower = negative_window.to_ascii_lowercase();
    let positive_window = commerce_text_window(&text, variant_index, 420);
    let positive_window_lower = positive_window.to_ascii_lowercase();
    let negative = negative_cues
        .iter()
        .find(|cue| commerce_cue_near_first_variant(&negative_window_lower, &variant, cue, 120))
        .copied();
    let positive = positive_cues
        .iter()
        .find(|cue| commerce_cue_near_any_variant(&positive_window_lower, &variant, cue, 320))
        .copied();
    match (positive, negative) {
        (_, Some(cue)) => Ok(CommerceRenderedAvailability {
            availability_state: "unavailable".to_string(),
            visible_evidence: Some(commerce_variant_cue_evidence(
                &negative_window,
                &variant,
                cue,
                500,
            )),
            confidence: 0.85,
            caveats: json!([format!(
                "Negative availability cue near exact variant: {cue}"
            )]),
            next_action: "Do not include in main recommendations unless a fresher exact-variant proof is captured."
                .to_string(),
        }),
        (Some(cue), None) => Ok(CommerceRenderedAvailability {
            availability_state: "available".to_string(),
            visible_evidence: Some(commerce_variant_cue_evidence(
                &positive_window,
                &variant,
                cue,
                500,
            )),
            confidence: 0.82,
            caveats: json!([format!(
                "Positive availability cue near exact variant: {cue}"
            )]),
            next_action: "None; exact rendered-page availability proof is present.".to_string(),
        }),
        (None, None) => Ok(CommerceRenderedAvailability {
            availability_state: "unknown".to_string(),
            visible_evidence: Some(commerce_variant_evidence(&negative_window, &variant, 500)),
            confidence: 0.35,
            caveats: json!([
                "Exact variant label was visible, but no supported availability cue was nearby."
            ]),
            next_action:
                "Inspect page controls manually or provide a selector/visible evidence note."
                    .to_string(),
        }),
    }
}

pub(crate) fn commerce_variant_cue_evidence(
    text: &str,
    variant_label_lower: &str,
    cue_lower: &str,
    max_chars: usize,
) -> String {
    let lower = text.to_ascii_lowercase();
    let Some(variant_index) = lower.find(variant_label_lower) else {
        return excerpt(text, max_chars);
    };
    let Some(cue_index) = lower.find(cue_lower) else {
        return commerce_variant_evidence(text, variant_label_lower, max_chars);
    };
    let start = variant_index.min(cue_index);
    let end = (variant_index + variant_label_lower.len()).max(cue_index + cue_lower.len());
    excerpt(&commerce_text_span(text, start, end, 80, 160), max_chars)
}

pub(crate) fn commerce_cue_near_first_variant(
    text_lower: &str,
    variant_label_lower: &str,
    cue_lower: &str,
    max_distance: usize,
) -> bool {
    let Some(variant_index) = text_lower.find(variant_label_lower) else {
        return false;
    };
    text_lower
        .match_indices(cue_lower)
        .any(|(cue_index, _)| variant_index.abs_diff(cue_index) <= max_distance)
}

pub(crate) fn commerce_cue_near_any_variant(
    text_lower: &str,
    variant_label_lower: &str,
    cue_lower: &str,
    max_distance: usize,
) -> bool {
    text_lower
        .match_indices(variant_label_lower)
        .any(|(variant_index, _)| {
            text_lower
                .match_indices(cue_lower)
                .any(|(cue_index, _)| variant_index.abs_diff(cue_index) <= max_distance)
        })
}

pub(crate) fn commerce_variant_evidence(
    text: &str,
    variant_label_lower: &str,
    max_chars: usize,
) -> String {
    let lower = text.to_ascii_lowercase();
    if let Some(index) = lower.find(variant_label_lower) {
        excerpt(&commerce_text_window(text, index, max_chars / 2), max_chars)
    } else {
        excerpt(text, max_chars)
    }
}

pub(crate) fn commerce_text_window(text: &str, byte_index: usize, radius: usize) -> String {
    let start = text[..byte_index]
        .char_indices()
        .rev()
        .nth(radius)
        .map(|(idx, _)| idx)
        .unwrap_or(0);
    let end = text[byte_index..]
        .char_indices()
        .nth(radius)
        .map(|(idx, _)| byte_index + idx)
        .unwrap_or_else(|| text.len());
    text[start..end].to_string()
}

pub(crate) fn commerce_text_span(
    text: &str,
    start_byte: usize,
    end_byte: usize,
    prefix_chars: usize,
    suffix_chars: usize,
) -> String {
    let start_byte = start_byte.min(text.len());
    let end_byte = end_byte.min(text.len()).max(start_byte);
    let start = text[..start_byte]
        .char_indices()
        .rev()
        .nth(prefix_chars)
        .map(|(idx, _)| idx)
        .unwrap_or(0);
    let end = text[end_byte..]
        .char_indices()
        .nth(suffix_chars)
        .map(|(idx, _)| end_byte + idx)
        .unwrap_or_else(|| text.len());
    text[start..end].to_string()
}

pub(crate) fn excerpt_window(text: &str, needle: &str, max_chars: usize) -> String {
    let lower = text.to_ascii_lowercase();
    let needle = needle.to_ascii_lowercase();
    if let Some(index) = lower.find(&needle) {
        excerpt(&commerce_text_window(text, index, max_chars / 2), max_chars)
    } else {
        excerpt(text, max_chars)
    }
}

pub(crate) fn render_commerce_rendered_page_artifact(
    doc: &UrlIngestDocument,
    input: &CommerceRenderedPageCheckInput,
    rendered: &CommerceRenderedAvailability,
) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!(
        "# Rendered Commerce Page: {}\n\n",
        escape_untrusted_markdown_text(&doc.title)
    ));
    markdown.push_str(untrusted_evidence_notice(
        "Host/browser-rendered page content below",
    ));
    markdown.push_str("## Commerce Check\n\n");
    markdown.push_str(&format!(
        "- Candidate id: `{}`\n",
        escape_untrusted_markdown_text(&input.candidate_id)
    ));
    markdown.push_str(&format!(
        "- Variant key: `{}`\n",
        escape_untrusted_markdown_text(&input.variant_key)
    ));
    markdown.push_str(&format!(
        "- Variant label: `{}`\n",
        escape_untrusted_markdown_text(&input.variant_label)
    ));
    markdown.push_str(&format!(
        "- Availability state: `{}`\n",
        rendered.availability_state
    ));
    markdown.push_str(&format!("- Requested URL: <{}>\n", doc.requested_url));
    markdown.push_str(&format!("- Final URL: <{}>\n", doc.final_url));
    if let Some(captured_at) = &doc.captured_at {
        markdown.push_str(&format!(
            "- Captured at: `{}`\n",
            escape_untrusted_markdown_text(captured_at)
        ));
    }
    if let Some(browser) = &doc.browser {
        markdown.push_str(&format!(
            "- Browser: `{}`\n",
            escape_untrusted_markdown_text(browser)
        ));
    }
    if let Some(screenshot_path) = &doc.screenshot_path {
        markdown.push_str(&format!(
            "- Screenshot path: `{}`\n",
            escape_untrusted_markdown_text(screenshot_path)
        ));
    }
    if let Some(evidence) = &rendered.visible_evidence {
        markdown.push_str("\n## Visible Evidence\n\n");
        markdown.push_str(&format!("> {}\n", escape_untrusted_markdown_text(evidence)));
    }
    markdown.push_str("\n## Rendered Text Excerpt\n\n");
    markdown.push_str(&escape_untrusted_markdown_text(&excerpt(
        &doc.readable_text,
        6_000,
    )));
    markdown.push('\n');
    markdown
}

pub(crate) fn normalize_commerce_key_list(values: Vec<String>, label: &str) -> Result<Vec<String>> {
    if values.len() > COMMERCE_MAX_LIST_ITEMS {
        bail!("too many commerce {label} values");
    }
    let mut out = Vec::new();
    for value in values {
        let value = normalize_research_key(value, label)?;
        if !out.contains(&value) {
            out.push(value);
        }
    }
    Ok(out)
}

pub(crate) fn sanitize_required_commerce_text(
    input: &str,
    label: &str,
    max_chars: usize,
) -> Result<String> {
    let value = sanitize_work_text(input.trim(), max_chars)?;
    if value.trim().is_empty() {
        bail!("commerce {label} cannot be empty");
    }
    Ok(value)
}

pub(crate) fn normalize_optional_id(value: Option<String>, label: &str) -> Result<Option<String>> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            validate_id(value).with_context(|| format!("invalid {label}"))?;
            Ok(value.to_string())
        })
        .transpose()
}

pub(crate) fn validate_score_range(value: f64, label: &str) -> Result<()> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        bail!("{label} must be finite and between 0.0 and 1.0");
    }
    Ok(())
}

pub(crate) fn normalize_commerce_candidate_status(status: &str) -> Result<String> {
    match status.trim() {
        "qualified" | "disqualified" | "maybe" | "blocked" => Ok(status.trim().to_string()),
        other => bail!("unsupported commerce candidate status: {other}"),
    }
}

pub(crate) fn normalize_commerce_proof_method(method: &str) -> Result<String> {
    match method.trim() {
        "static_fetch" | "rendered_browser" | "chrome_profile" | "manual_user" => {
            Ok(method.trim().to_string())
        }
        other => bail!("unsupported commerce proof method: {other}"),
    }
}

pub(crate) fn normalize_commerce_availability_state(state: &str) -> Result<String> {
    match state.trim() {
        "available" | "unavailable" | "unknown" | "blocked" => Ok(state.trim().to_string()),
        other => bail!("unsupported commerce availability state: {other}"),
    }
}

pub(crate) fn normalize_commerce_fact_kind(kind: &str) -> Result<String> {
    match kind.trim() {
        "explicit" | "inferred" | "uncertain" | "missing" => Ok(kind.trim().to_string()),
        other => bail!("unsupported commerce context fact kind: {other}"),
    }
}

pub(crate) fn normalize_commerce_verification_result(result: &str) -> Result<String> {
    match result.trim() {
        "available" | "unavailable" | "unknown" | "blocked" | "error" => {
            Ok(result.trim().to_string())
        }
        other => bail!("unsupported commerce verification result: {other}"),
    }
}

pub(crate) fn normalize_commerce_report_decision(decision: &str) -> Result<String> {
    match decision.trim() {
        "accept" | "hold" | "block" => Ok(decision.trim().to_string()),
        other => bail!("unsupported commerce report judgment decision: {other}"),
    }
}
