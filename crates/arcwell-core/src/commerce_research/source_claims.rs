use super::*;

pub(crate) fn parse_research_claim_candidate(
    value: &Value,
    source_text: &str,
    source_card_id: &str,
) -> Result<ResearchClaimCandidate> {
    let object = value
        .as_object()
        .context("each research extraction claim must be an object")?;
    let text = required_json_string(object, "text")?;
    validate_notes(&text)?;
    if contains_prompt_injection_text(&text.to_ascii_lowercase()) {
        bail!("research claim contains prompt-injection instruction text");
    }
    let kind = required_json_string(object, "kind")?;
    validate_research_claim_kind(&kind)?;
    let confidence = object
        .get("confidence")
        .and_then(Value::as_f64)
        .context("research claim confidence must be a number")?;
    if !(0.0..=1.0).contains(&confidence) {
        bail!("research claim confidence must be between 0 and 1");
    }
    let caveats = optional_json_string_array(object.get("caveats"))?;
    for caveat in &caveats {
        validate_notes(caveat)?;
        if contains_prompt_injection_text(&caveat.to_ascii_lowercase()) {
            bail!("research claim caveat contains prompt-injection instruction text");
        }
    }
    if source_text_contains_uncertainty(source_text)
        && !claim_text_preserves_uncertainty(&text)
        && caveats.is_empty()
    {
        bail!("uncertain source text cannot be extracted as a definitive claim without caveats");
    }
    let quote = optional_json_string(object.get("quote"), "quote", 1_000)?;
    let source_anchor = optional_json_string(object.get("source_anchor"), "source_anchor", 500)?;
    let evidence_anchors = parse_research_evidence_anchors(
        object
            .get("document_anchors")
            .or_else(|| object.get("evidence_anchors")),
    )?;
    Ok(ResearchClaimCandidate {
        text,
        kind,
        subject: optional_json_string(object.get("subject"), "subject", 500)?,
        predicate: optional_json_string(object.get("predicate"), "predicate", 500)?,
        object_value: optional_json_string(object.get("object"), "object", 1_000)?,
        temporal_scope: optional_json_string(object.get("temporal_scope"), "temporal_scope", 500)?,
        confidence,
        caveats,
        quote,
        source_anchor,
        evidence_anchors,
        metadata: json!({ "source_card_id": source_card_id }),
    })
}

pub(crate) fn parse_research_evidence_anchors(
    value: Option<&Value>,
) -> Result<Vec<ResearchEvidenceAnchor>> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    if value.is_null() {
        return Ok(Vec::new());
    }
    let anchors = value
        .as_array()
        .context("research claim document_anchors must be an array")?;
    if anchors.len() > 20 {
        bail!("research claim has too many document anchors");
    }
    let mut out = Vec::new();
    for anchor in anchors {
        let object = anchor
            .as_object()
            .context("research claim document anchor must be an object")?;
        let document_id = required_json_string(object, "document_id")?;
        validate_id(&document_id)?;
        let span_id = optional_json_string(object.get("span_id"), "span_id", 200)?;
        if let Some(span_id) = &span_id {
            validate_research_anchor_label(span_id, "span_id")?;
        }
        let table_id = optional_json_string(object.get("table_id"), "table_id", 200)?;
        if let Some(table_id) = &table_id {
            validate_research_anchor_label(table_id, "table_id")?;
        }
        let row_index = optional_json_usize(object.get("row_index"), "row_index")?;
        let column_index = optional_json_usize(object.get("column_index"), "column_index")?;
        let quote = optional_json_string(object.get("quote"), "anchor quote", 1_000)?;
        match (&span_id, &table_id, row_index, column_index) {
            (Some(_), None, None, None)
            | (None, Some(_), None, None)
            | (None, Some(_), Some(_), Some(_)) => {}
            _ => bail!(
                "research claim document anchor must reference exactly one span, table, or table cell"
            ),
        }
        out.push(ResearchEvidenceAnchor {
            document_id,
            span_id,
            table_id,
            row_index,
            column_index,
            quote,
        });
    }
    Ok(out)
}

pub(crate) fn optional_json_usize(value: Option<&Value>, label: &str) -> Result<Option<usize>> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let Some(number) = value.as_u64() else {
        bail!("research claim {label} must be a non-negative integer");
    };
    if number > 1_000_000 {
        bail!("research claim {label} is too large");
    }
    Ok(Some(number as usize))
}

pub(crate) fn validate_research_anchor_label(value: &str, label: &str) -> Result<()> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > 200 {
        bail!("research document anchor {label} must be 1-200 characters");
    }
    if trimmed
        .chars()
        .any(|ch| ch.is_control() || matches!(ch, '[' | ']' | '(' | ')' | '<' | '>' | '`'))
    {
        bail!("research document anchor {label} contains unsupported characters");
    }
    Ok(())
}

pub(crate) fn sanitize_optional_anchor_quote(value: Option<&str>) -> Result<Option<String>> {
    value
        .map(|quote| sanitize_work_text(quote, 1_000))
        .transpose()
        .map(|value| value.filter(|quote| !quote.trim().is_empty()))
}

pub(crate) fn find_document_table<'a>(
    document: &'a ResearchDocumentRecord,
    table_id: &str,
) -> Result<&'a ResearchTableRecord> {
    document
        .tables
        .iter()
        .find(|table| table.table.table_id == table_id)
        .with_context(|| {
            format!(
                "document table anchor not found: document={} table={}",
                document.document.id, table_id
            )
        })
}

pub(crate) fn required_json_string(object: &Map<String, Value>, key: &str) -> Result<String> {
    let value = object
        .get(key)
        .and_then(Value::as_str)
        .with_context(|| format!("research claim {key} must be a string"))?
        .trim()
        .to_string();
    if value.is_empty() {
        bail!("research claim {key} cannot be empty");
    }
    Ok(value)
}

pub(crate) fn optional_json_string(
    value: Option<&Value>,
    label: &str,
    max_len: usize,
) -> Result<Option<String>> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let Some(text) = value.as_str() else {
        bail!("research claim {label} must be a string");
    };
    let text = text.trim();
    if text.is_empty() {
        return Ok(None);
    }
    if text.len() > max_len {
        bail!("research claim {label} is too long");
    }
    Ok(Some(text.to_string()))
}

pub(crate) fn optional_json_string_array(value: Option<&Value>) -> Result<Vec<String>> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    if value.is_null() {
        return Ok(Vec::new());
    }
    if let Some(text) = value.as_str() {
        return Ok(if text.trim().is_empty() {
            Vec::new()
        } else {
            vec![text.trim().to_string()]
        });
    }
    let array = value
        .as_array()
        .context("research claim caveats must be a string array")?;
    let mut out = Vec::new();
    for item in array {
        let Some(text) = item.as_str() else {
            bail!("research claim caveat must be a string");
        };
        let text = text.trim();
        if !text.is_empty() {
            out.push(text.to_string());
        }
    }
    Ok(out)
}

pub(crate) fn validate_research_claim_kind(kind: &str) -> Result<()> {
    match kind {
        "fact" | "interpretation" | "prediction" | "rumor" | "measurement" | "recommendation" => {
            Ok(())
        }
        other => bail!("unsupported research claim kind: {other}"),
    }
}

pub(crate) fn source_card_text_for_uncertainty_checks(card: &SourceCard) -> String {
    let mut text = card.summary.clone();
    for claim in &card.claims {
        text.push('\n');
        text.push_str(&claim.claim);
    }
    text
}

pub(crate) fn source_text_contains_uncertainty(text: &str) -> bool {
    let lower = format!(" {} ", text.to_ascii_lowercase());
    [
        " may ",
        " might ",
        " could ",
        " possibly ",
        " alleged",
        " claims ",
        " suggests ",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

pub(crate) fn claim_text_preserves_uncertainty(text: &str) -> bool {
    source_text_contains_uncertainty(&format!(" {text} "))
}

pub(crate) fn research_extraction_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["claims"],
        "properties": {
            "claims": {
                "type": "array",
                "maxItems": 50,
                "items": {
                    "type": "object",
                    "required": ["text", "kind", "confidence"],
                    "properties": {
                        "text": { "type": "string" },
                        "kind": { "enum": ["fact", "interpretation", "prediction", "rumor", "measurement", "recommendation"] },
                        "subject": { "type": ["string", "null"] },
                        "predicate": { "type": ["string", "null"] },
                        "object": { "type": ["string", "null"] },
                        "temporal_scope": { "type": ["string", "null"] },
                        "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
                        "caveats": { "type": "array", "items": { "type": "string" } },
                        "quote": { "type": ["string", "null"] },
                        "source_anchor": { "type": ["string", "null"] },
                        "document_anchors": {
                            "type": "array",
                            "maxItems": 20,
                            "items": {
                                "type": "object",
                                "additionalProperties": false,
                                "required": ["document_id"],
                                "properties": {
                                    "document_id": { "type": "string" },
                                    "span_id": { "type": ["string", "null"] },
                                    "table_id": { "type": ["string", "null"] },
                                    "row_index": { "type": ["integer", "null"], "minimum": 0 },
                                    "column_index": { "type": ["integer", "null"], "minimum": 0 },
                                    "quote": { "type": ["string", "null"] }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}

pub(crate) fn validate_source_card_metadata(metadata: &Value) -> Result<()> {
    let Value::Object(object) = metadata else {
        if metadata.is_null() {
            return Ok(());
        }
        bail!("source-card metadata must be an object");
    };
    if let Some(version) = object.get("schema_version")
        && version.as_u64() != Some(SOURCE_CARD_SCHEMA_VERSION)
    {
        bail!("unsupported source-card schema version");
    }
    if let Some(role) = object.get("source_role").and_then(Value::as_str) {
        validate_source_role(role)?;
    }
    if let Some(trust) = object.get("trust_level").and_then(Value::as_str) {
        validate_source_trust_level(trust)?;
    }
    if let Some(score) = object.get("reliability_score") {
        let Some(score) = score.as_f64() else {
            bail!("source-card metadata reliability_score must be a number");
        };
        if !(0.0..=1.0).contains(&score) {
            bail!("source-card metadata reliability_score must be between 0 and 1");
        }
    }
    if let Some(strength) = object.get("provenance_strength").and_then(Value::as_str) {
        validate_provenance_strength(strength)?;
    }
    for key in ["source_owner", "robots_meta", "crawl_rate_policy"] {
        if object
            .get(key)
            .is_some_and(|value| value.as_str().is_none())
        {
            bail!("source-card metadata {key} must be a string");
        }
    }
    for key in ["robots_noindex", "robots_nofollow"] {
        if object
            .get(key)
            .is_some_and(|value| value.as_bool().is_none())
        {
            bail!("source-card metadata {key} must be a boolean");
        }
    }
    if let Some(delay) = object.get("crawl_delay_seconds") {
        if delay.as_u64().is_none() {
            bail!("source-card metadata crawl_delay_seconds must be an integer");
        }
        if delay.as_u64().unwrap_or_default() > 86_400 {
            bail!("source-card metadata crawl_delay_seconds is too large");
        }
    }
    for key in ["quality_flags", "extracted_entities", "extracted_dates"] {
        if let Some(value) = object.get(key) {
            let Some(items) = value.as_array() else {
                bail!("source-card metadata {key} must be an array");
            };
            if items.len() > 100 {
                bail!("source-card metadata {key} has too many entries");
            }
            if items.iter().any(|item| item.as_str().is_none()) {
                bail!("source-card metadata {key} must contain strings");
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_source_role(role: &str) -> Result<()> {
    match role {
        "primary" | "secondary" | "generated_synthesis" | "model_answer" => Ok(()),
        other => bail!("unsupported source-card source_role: {other}"),
    }
}

pub(crate) fn validate_source_trust_level(trust: &str) -> Result<()> {
    match trust {
        "high" | "medium" | "low" | "untrusted" => Ok(()),
        other => bail!("unsupported source-card trust_level: {other}"),
    }
}

pub(crate) fn validate_provenance_strength(strength: &str) -> Result<()> {
    match strength {
        "direct" | "syndicated" | "aggregated" | "generated" | "unknown" => Ok(()),
        other => bail!("unsupported source-card provenance_strength: {other}"),
    }
}

pub(crate) fn normalize_source_card_metadata(
    input: &SourceCardInput,
    retrieved_at: &str,
) -> Result<Value> {
    let mut object: Map<String, Value> = match &input.metadata {
        Value::Null => Map::new(),
        Value::Object(object) => object.clone(),
        _ => bail!("source-card metadata must be an object"),
    };
    validate_source_card_metadata(&Value::Object(object.clone()))?;

    let source_role = object
        .get("source_role")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| infer_source_role(input));
    validate_source_role(&source_role)?;
    if source_role == "primary" && is_generated_source_card_input(input) {
        bail!("generated research output cannot be primary source-card evidence");
    }

    let mut flags: BTreeSet<String> = object
        .get("quality_flags")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect();
    for flag in infer_source_quality_flags(input, retrieved_at) {
        flags.insert(flag);
    }

    let trust_level = object
        .get("trust_level")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| infer_source_trust_level(input, &flags, &source_role));
    validate_source_trust_level(&trust_level)?;

    let text = source_card_text_for_extraction(input);
    object.insert(
        "schema_version".to_string(),
        json!(SOURCE_CARD_SCHEMA_VERSION),
    );
    object.insert("source_role".to_string(), json!(source_role));
    object.insert("trust_level".to_string(), json!(trust_level));
    object
        .entry("reliability_score".to_string())
        .or_insert_with(|| json!(infer_source_reliability_score(input, &flags)));
    object
        .entry("provenance_strength".to_string())
        .or_insert_with(|| json!(infer_provenance_strength(input)));
    if let Ok(url) = Url::parse(&input.url)
        && let Some(host) = url.host_str()
    {
        object
            .entry("source_owner".to_string())
            .or_insert_with(|| json!(host.to_ascii_lowercase()));
    }
    object
        .entry("crawl_rate_policy".to_string())
        .or_insert_with(|| json!(infer_crawl_rate_policy(input)));
    object.insert(
        "quality_flags".to_string(),
        json!(flags.into_iter().collect::<Vec<_>>()),
    );
    object.insert(
        "extracted_entities".to_string(),
        json!(extract_source_entities(&text)),
    );
    object.insert(
        "extracted_dates".to_string(),
        json!(extract_date_mentions(&text)),
    );
    Ok(Value::Object(object))
}

pub(crate) fn source_card_metadata_string(metadata: &Value, key: &str) -> Option<String> {
    metadata
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

pub(crate) fn source_card_metadata_strings(metadata: &Value, key: &str) -> Vec<String> {
    metadata
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect()
}

pub(crate) fn infer_source_role(input: &SourceCardInput) -> String {
    let source_type = input.source_type.to_ascii_lowercase();
    let provider = input.provider.to_ascii_lowercase();
    if is_generated_source_card_input(input) {
        "generated_synthesis".to_string()
    } else if matches!(
        source_type.as_str(),
        "model_answer" | "llm_answer" | "answer"
    ) {
        "model_answer".to_string()
    } else if provider.contains("github")
        || provider.contains("arxiv")
        || matches!(
            source_type.as_str(),
            "github_release"
                | "github_commit"
                | "github_repo"
                | "arxiv"
                | "paper"
                | "release"
                | "doc"
                | "docs"
                | "documentation"
                | "gov"
                | "government"
                | "regulator"
                | "city"
                | "university"
                | "official"
                | "company"
                | "accelerator"
                | "vc"
        )
    {
        "primary".to_string()
    } else {
        "secondary".to_string()
    }
}

pub(crate) fn infer_source_trust_level(
    input: &SourceCardInput,
    flags: &BTreeSet<String>,
    source_role: &str,
) -> String {
    if source_role == "generated_synthesis"
        || flags.contains("prompt_injection_text")
        || flags.contains("seo_spam_indicators")
    {
        "untrusted".to_string()
    } else if flags.contains("model_answer_without_citations") || flags.contains("stale_source") {
        "low".to_string()
    } else if source_role == "primary" && !input.url.contains("example.com") {
        "high".to_string()
    } else {
        "medium".to_string()
    }
}

pub(crate) fn infer_source_reliability_score(
    input: &SourceCardInput,
    flags: &BTreeSet<String>,
) -> f64 {
    let mut score: f64 = match infer_source_role(input).as_str() {
        "primary" => 0.85,
        "secondary" => 0.65,
        "model_answer" => 0.35,
        "generated_synthesis" => 0.2,
        _ => 0.5,
    };
    if flags.contains("prompt_injection_text") {
        score -= 0.25;
    }
    if flags.contains("seo_spam_indicators") {
        score -= 0.25;
    }
    if flags.contains("stale_source") {
        score -= 0.15;
    }
    if flags.contains("model_answer_without_citations") {
        score -= 0.2;
    }
    if input.url.contains("example.com") {
        score -= 0.1;
    }
    score.clamp(0.0, 1.0)
}

pub(crate) fn infer_provenance_strength(input: &SourceCardInput) -> &'static str {
    let source_type = input.source_type.to_ascii_lowercase();
    let provider = input.provider.to_ascii_lowercase();
    if is_generated_source_card_input(input) {
        "generated"
    } else if source_type == "rss" {
        "syndicated"
    } else if matches!(
        source_type.as_str(),
        "model_answer" | "llm_answer" | "answer"
    ) {
        "generated"
    } else if provider.contains("brave") || provider.contains("perplexity") {
        "aggregated"
    } else if matches!(
        source_type.as_str(),
        "github_release"
            | "github_commit"
            | "github_repo"
            | "arxiv"
            | "paper"
            | "release"
            | "blog"
            | "doc"
            | "docs"
            | "documentation"
            | "gov"
            | "government"
            | "regulator"
            | "city"
            | "university"
            | "official"
            | "company"
            | "accelerator"
            | "vc"
    ) {
        "direct"
    } else {
        "unknown"
    }
}

pub(crate) fn infer_crawl_rate_policy(input: &SourceCardInput) -> String {
    if let Some(policy) = input
        .metadata
        .get("crawl_rate_policy")
        .and_then(Value::as_str)
    {
        return excerpt(policy, 500);
    }
    match input.source_type.to_ascii_lowercase().as_str() {
        "rss" => {
            "rss poller default: no more than hourly unless source health backs off".to_string()
        }
        "github_release" | "github_commit" | "github_repo" => {
            "github poller default: no more than hourly; rate-limit responses back off".to_string()
        }
        "arxiv" | "paper" => "arxiv poller default: no more than hourly".to_string(),
        "x" | "tweet" => {
            "x monitor default: no more than every 15 minutes; quota responses back off".to_string()
        }
        _ => "manual or one-shot source; no scheduled crawl claimed".to_string(),
    }
}

pub(crate) fn is_generated_source_card_input(input: &SourceCardInput) -> bool {
    is_generated_title(&input.title)
        || input.provider.to_ascii_lowercase().contains("generated")
        || matches!(
            input.source_type.to_ascii_lowercase().as_str(),
            "generated" | "research_brief" | "expanded_page" | "generated_summary"
        )
        || input
            .metadata
            .get("generated")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        || input
            .metadata
            .get("origin")
            .and_then(Value::as_str)
            .map(|origin| origin.starts_with("generated:") || origin.starts_with("research:"))
            .unwrap_or(false)
}

pub(crate) fn is_generated_source_card(card: &SourceCard) -> bool {
    is_generated_title(&card.title)
        || card.provider.to_ascii_lowercase().contains("generated")
        || matches!(
            card.source_type.to_ascii_lowercase().as_str(),
            "generated" | "research_brief" | "expanded_page" | "generated_summary"
        )
        || source_card_metadata_string(&card.metadata, "source_role").as_deref()
            == Some("generated_synthesis")
        || card
            .metadata
            .get("generated")
            .and_then(Value::as_bool)
            .unwrap_or(false)
}

pub(crate) fn is_generated_title(title: &str) -> bool {
    let normalized = title.trim_start().to_ascii_lowercase();
    normalized.starts_with("research brief:")
        || normalized.starts_with("deep research report:")
        || normalized.starts_with("expanded:")
        || normalized.starts_with("source card:")
        || normalized.starts_with("source card: research brief:")
        || normalized.starts_with("source card: deep research report:")
        || normalized.starts_with("source card: expanded:")
}

pub(crate) fn source_card_is_primary_evidence(card: &SourceCard) -> bool {
    let role = source_card_metadata_string(&card.metadata, "source_role")
        .unwrap_or_else(|| infer_source_role_from_card(card));
    let trust = source_card_metadata_string(&card.metadata, "trust_level")
        .unwrap_or_else(|| "medium".to_string());
    let reliability = card
        .metadata
        .get("reliability_score")
        .and_then(Value::as_f64)
        .unwrap_or(0.5);
    !is_generated_source_card(card)
        && role != "generated_synthesis"
        && role != "model_answer"
        && trust != "untrusted"
        && reliability >= 0.4
}

pub(crate) fn infer_source_role_from_card(card: &SourceCard) -> String {
    if let Some(role) = source_card_metadata_string(&card.metadata, "source_role") {
        return role;
    }
    infer_source_role(&SourceCardInput {
        title: card.title.clone(),
        url: card.url.clone(),
        source_type: card.source_type.clone(),
        provider: card.provider.clone(),
        summary: card.summary.clone(),
        claims: card.claims.clone(),
        retrieved_at: Some(card.retrieved_at.clone()),
        metadata: card.metadata.clone(),
    })
}

pub(crate) fn source_card_text_for_extraction(input: &SourceCardInput) -> String {
    let mut text = format!("{} {}", input.title, input.summary);
    for claim in &input.claims {
        text.push(' ');
        text.push_str(&claim.claim);
    }
    text
}

pub(crate) fn source_card_text(card: &SourceCard) -> String {
    let mut text = format!("{} {}", card.title, card.summary);
    for claim in &card.claims {
        text.push(' ');
        text.push_str(&claim.claim);
    }
    text
}

pub(crate) fn extract_source_claims_from_summary(summary: &str) -> Vec<SourceClaim> {
    summary
        .split(['.', '\n'])
        .map(str::trim)
        .filter(|sentence| sentence.len() >= 20)
        .take(5)
        .map(|sentence| SourceClaim {
            claim: sentence.to_string(),
            kind: infer_claim_kind(sentence).to_string(),
            confidence: 0.55,
        })
        .collect()
}

pub(crate) fn infer_claim_kind(claim: &str) -> &'static str {
    let lower = claim.to_ascii_lowercase();
    if lower.contains("launch") || lower.contains("released") {
        "launch"
    } else if lower.contains("date") || lower.contains("announced") {
        "timeline"
    } else {
        "fact"
    }
}

pub(crate) fn extract_source_entities(text: &str) -> Vec<String> {
    let stop = [
        "The",
        "This",
        "That",
        "Source",
        "Card",
        "Research",
        "Brief",
        "Expanded",
        "Generated",
        "According",
    ];
    let mut entities = BTreeSet::new();
    let mut current = Vec::new();
    for token in text.split_whitespace() {
        let cleaned = token.trim_matches(|ch: char| !ch.is_alphanumeric() && ch != '-');
        let is_entity = cleaned
            .chars()
            .next()
            .map(char::is_uppercase)
            .unwrap_or(false)
            && cleaned.len() > 2
            && !stop.contains(&cleaned);
        if is_entity {
            current.push(cleaned.to_string());
        } else if !current.is_empty() {
            entities.insert(current.join(" "));
            current.clear();
        }
    }
    if !current.is_empty() {
        entities.insert(current.join(" "));
    }
    entities.into_iter().take(25).collect()
}

pub(crate) fn extract_date_mentions(text: &str) -> Vec<String> {
    let mut dates = BTreeSet::new();
    for token in text.split_whitespace() {
        let cleaned = token
            .trim_matches(|ch: char| matches!(ch, ',' | '.' | ';' | ':' | ')' | '(' | '[' | ']'));
        if is_iso_date(cleaned) || is_year_month_date(cleaned) {
            dates.insert(cleaned.to_string());
        }
    }
    dates.into_iter().take(25).collect()
}

pub(crate) fn is_iso_date(value: &str) -> bool {
    value.len() == 10
        && value.as_bytes().get(4) == Some(&b'-')
        && value.as_bytes().get(7) == Some(&b'-')
        && value
            .chars()
            .enumerate()
            .all(|(idx, ch)| idx == 4 || idx == 7 || ch.is_ascii_digit())
}

pub(crate) fn is_year_month_date(value: &str) -> bool {
    value.len() == 7
        && value.as_bytes().get(4) == Some(&b'-')
        && value
            .chars()
            .enumerate()
            .all(|(idx, ch)| idx == 4 || ch.is_ascii_digit())
}

pub(crate) fn infer_source_quality_flags(
    input: &SourceCardInput,
    retrieved_at: &str,
) -> Vec<String> {
    let mut flags = BTreeSet::new();
    let text = source_card_text_for_extraction(input);
    let lower = text.to_ascii_lowercase();
    if contains_prompt_injection_text(&lower) {
        flags.insert("prompt_injection_text".to_string());
    }
    if contains_seo_spam_text(&lower) {
        flags.insert("seo_spam_indicators".to_string());
    }
    if source_card_retrieved_at_is_stale(retrieved_at) {
        flags.insert("stale_source".to_string());
    }
    if matches!(
        input.source_type.to_ascii_lowercase().as_str(),
        "model_answer" | "llm_answer" | "answer"
    ) && !source_input_has_citations(input)
    {
        flags.insert("model_answer_without_citations".to_string());
    }
    if input.claims.is_empty() {
        flags.insert("no_structured_claims".to_string());
    }
    flags.into_iter().collect()
}

pub(crate) fn contains_prompt_injection_text(lower: &str) -> bool {
    [
        "ignore previous instructions",
        "system:",
        "developer:",
        "tool_call",
        "reveal secrets",
        "disclose tokens",
        "send your api key",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

pub(crate) fn contains_seo_spam_text(lower: &str) -> bool {
    let spam_terms = [
        "casino",
        "coupon code",
        "guest post",
        "best price",
        "seo backlinks",
        "sponsored post",
        "buy now",
    ];
    let hits = spam_terms
        .iter()
        .filter(|needle| lower.contains(**needle))
        .count();
    hits >= 2 || (hits == 1 && lower.matches("http").count() > 4)
}

pub(crate) fn source_input_has_citations(input: &SourceCardInput) -> bool {
    input
        .metadata
        .get("citations")
        .and_then(Value::as_array)
        .map(|items| !items.is_empty())
        .unwrap_or(false)
        || input.summary.contains("http://")
        || input.summary.contains("https://")
        || input.claims.iter().any(|claim| {
            claim.claim.contains("http://")
                || claim.claim.contains("https://")
                || claim.claim.contains("[1]")
        })
}

pub(crate) fn source_card_has_citations(card: &SourceCard) -> bool {
    card.metadata
        .get("citations")
        .and_then(Value::as_array)
        .map(|items| !items.is_empty())
        .unwrap_or(false)
        || card.summary.contains("http://")
        || card.summary.contains("https://")
        || card.claims.iter().any(|claim| {
            claim.claim.contains("http://")
                || claim.claim.contains("https://")
                || claim.claim.contains("[1]")
        })
}

pub(crate) fn source_card_retrieved_at_is_stale(retrieved_at: &str) -> bool {
    DateTime::parse_from_rfc3339(retrieved_at)
        .map(|date| {
            Utc::now().signed_duration_since(date.with_timezone(&Utc))
                > chrono::Duration::days(SOURCE_CARD_STALE_DAYS)
        })
        .unwrap_or(false)
}

pub(crate) fn audit_source_card(card: &SourceCard) -> Vec<ResearchAuditFinding> {
    let mut findings = Vec::new();
    let role = source_card_metadata_string(&card.metadata, "source_role")
        .unwrap_or_else(|| infer_source_role_from_card(card));
    let trust_level = source_card_metadata_string(&card.metadata, "trust_level")
        .unwrap_or_else(|| "medium".to_string());
    let reliability_score = card
        .metadata
        .get("reliability_score")
        .and_then(Value::as_f64)
        .unwrap_or(0.5);
    let flags = source_card_metadata_strings(&card.metadata, "quality_flags");
    if card.metadata.get("schema_version").and_then(Value::as_u64)
        != Some(SOURCE_CARD_SCHEMA_VERSION)
    {
        findings.push(source_card_finding(
            "warning",
            "legacy_or_missing_schema_version",
            card,
            "Source card does not declare the current schema version.",
            "metadata.schema_version",
        ));
    }
    if is_generated_source_card(card) {
        findings.push(source_card_finding(
            "error",
            "generated_page_recursion",
            card,
            "Generated research/wiki output cannot be primary evidence.",
            &card.title,
        ));
    }
    if role == "primary" && is_generated_source_card(card) {
        findings.push(source_card_finding(
            "error",
            "generated_primary_source",
            card,
            "Generated output was marked as primary source evidence.",
            &card.title,
        ));
    }
    if role == "model_answer" && !source_card_has_citations(card) {
        findings.push(source_card_finding(
            "error",
            "model_answer_without_citations",
            card,
            "Model answer source card has no citations and must not ground research output.",
            &card.summary,
        ));
    }
    if trust_level == "untrusted"
        || flags.iter().any(|flag| {
            matches!(
                flag.as_str(),
                "prompt_injection_text" | "seo_spam_indicators"
            )
        })
    {
        findings.push(source_card_finding(
            "warning",
            "untrusted_evidence",
            card,
            "Source-card text is untrusted evidence and should be quoted, not obeyed.",
            &card.summary,
        ));
    }
    if reliability_score < 0.4 {
        findings.push(source_card_finding(
            "warning",
            "low_reliability_source",
            card,
            "Source-card reliability score is below the research quality gate.",
            &format!("{reliability_score:.2}"),
        ));
    }
    if card
        .metadata
        .get("robots_noindex")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        findings.push(source_card_finding(
            "warning",
            "robots_noindex_source",
            card,
            "Fetched source declares robots noindex; keep it as provenance, not publishable evidence.",
            card.metadata
                .get("robots_meta")
                .and_then(Value::as_str)
                .unwrap_or("robots_noindex=true"),
        ));
    }
    if flags.iter().any(|flag| flag == "stale_source")
        || source_card_retrieved_at_is_stale(&card.retrieved_at)
    {
        findings.push(source_card_finding(
            "warning",
            "stale_source",
            card,
            "Source-card retrieval date is stale for freshness-sensitive research.",
            &card.retrieved_at,
        ));
    }
    if card.claims.is_empty() {
        findings.push(source_card_finding(
            "warning",
            "no_structured_claims",
            card,
            "Source card has no structured claims to audit.",
            &card.title,
        ));
    }
    for claim in &card.claims {
        if claim.confidence < 0.4 {
            findings.push(source_card_finding(
                "warning",
                "low_confidence_claim",
                card,
                "Claim is explicitly uncertain and should be presented with caveats.",
                &claim.claim,
            ));
        }
    }
    findings
}

pub(crate) fn source_card_finding(
    severity: &str,
    code: &str,
    card: &SourceCard,
    message: &str,
    evidence: &str,
) -> ResearchAuditFinding {
    ResearchAuditFinding {
        severity: severity.to_string(),
        code: code.to_string(),
        source_card_id: Some(card.id.clone()),
        message: message.to_string(),
        evidence: excerpt(evidence, 500),
    }
}

pub(crate) fn detect_source_contradictions(cards: &[SourceCard]) -> Vec<ResearchAuditFinding> {
    let mut findings = Vec::new();
    for left_index in 0..cards.len() {
        for right_index in (left_index + 1)..cards.len() {
            let left = &cards[left_index];
            let right = &cards[right_index];
            if !claims_share_launch_subject(left, right) {
                continue;
            }
            let left_dates = source_card_dates(left);
            let right_dates = source_card_dates(right);
            if left_dates.is_empty() || right_dates.is_empty() || left_dates == right_dates {
                continue;
            }
            findings.push(ResearchAuditFinding {
                severity: "error".to_string(),
                code: "contradictory_launch_dates".to_string(),
                source_card_id: None,
                message: format!(
                    "Conflicting launch dates are present across `{}` and `{}`.",
                    left.id, right.id
                ),
                evidence: format!(
                    "{}: {:?}; {}: {:?}",
                    left.title, left_dates, right.title, right_dates
                ),
            });
        }
    }
    findings
}

pub(crate) fn claims_share_launch_subject(left: &SourceCard, right: &SourceCard) -> bool {
    let left_text = source_card_text(left).to_ascii_lowercase();
    let right_text = source_card_text(right).to_ascii_lowercase();
    if !left_text.contains("launch") || !right_text.contains("launch") {
        return false;
    }
    let left_entities: BTreeSet<String> =
        source_card_metadata_strings(&left.metadata, "extracted_entities")
            .into_iter()
            .map(|entity| entity.to_ascii_lowercase())
            .collect();
    let right_entities: BTreeSet<String> =
        source_card_metadata_strings(&right.metadata, "extracted_entities")
            .into_iter()
            .map(|entity| entity.to_ascii_lowercase())
            .collect();
    !left_entities.is_empty()
        && left_entities
            .iter()
            .any(|entity| right_entities.contains(entity))
}

pub(crate) fn source_card_dates(card: &SourceCard) -> BTreeSet<String> {
    let mut dates: BTreeSet<String> =
        source_card_metadata_strings(&card.metadata, "extracted_dates")
            .into_iter()
            .collect();
    for date in extract_date_mentions(&source_card_text(card)) {
        dates.insert(date);
    }
    dates
}
