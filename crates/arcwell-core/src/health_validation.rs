use crate::*;

pub(crate) fn worker_heartbeat_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<WorkerHeartbeat> {
    Ok(WorkerHeartbeat {
        worker_id: row.get(0)?,
        started_at: row.get(1)?,
        last_seen_at: row.get(2)?,
        processed_jobs: row.get(3)?,
        last_error: row.get(4)?,
    })
}

pub(crate) fn worker_heartbeat_event_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<WorkerHeartbeatEvent> {
    Ok(WorkerHeartbeatEvent {
        id: row.get(0)?,
        worker_id: row.get(1)?,
        seen_at: row.get(2)?,
        processed_jobs: row.get(3)?,
        last_error: row.get(4)?,
    })
}

pub(crate) fn worker_heartbeat_segment_span_seconds(
    events: &[WorkerHeartbeatEvent],
) -> Result<i64> {
    let Some(first) = events.first() else {
        return Ok(0);
    };
    let Some(last) = events.last() else {
        return Ok(0);
    };
    let first_at = DateTime::parse_from_rfc3339(&first.seen_at)
        .with_context(|| format!("parsing heartbeat event {}", first.seen_at))?
        .with_timezone(&Utc);
    let last_at = DateTime::parse_from_rfc3339(&last.seen_at)
        .with_context(|| format!("parsing heartbeat event {}", last.seen_at))?
        .with_timezone(&Utc);
    Ok((last_at - first_at).num_seconds().max(0))
}

pub(crate) fn heartbeat_age_seconds(heartbeat: &WorkerHeartbeat) -> Result<i64> {
    let last_seen = DateTime::parse_from_rfc3339(&heartbeat.last_seen_at)
        .with_context(|| format!("parsing heartbeat timestamp {}", heartbeat.last_seen_at))?
        .with_timezone(&Utc);
    Ok((Utc::now() - last_seen).num_seconds())
}

pub(crate) fn backup_age_seconds(created_at: &str) -> Result<i64> {
    let created_at = DateTime::parse_from_rfc3339(created_at)
        .with_context(|| format!("parsing backup timestamp {created_at}"))?
        .with_timezone(&Utc);
    Ok((Utc::now() - created_at).num_seconds())
}

pub(crate) fn parse_optional_expiry(expires_at: Option<&str>) -> Result<Option<DateTime<Utc>>> {
    expires_at
        .map(|value| {
            DateTime::parse_from_rfc3339(value)
                .with_context(|| format!("parsing secret expiry timestamp {value}"))
                .map(|parsed| parsed.with_timezone(&Utc))
        })
        .transpose()
}

pub(crate) fn secret_ref_health(secret: &SecretRef, has_local_value: bool) -> SecretHealth {
    let mut warnings = Vec::new();
    let mut status = "configured".to_string();
    match parse_optional_expiry(secret.expires_at.as_deref()) {
        Ok(Some(expires_at)) if expires_at <= Utc::now() => {
            status = "expired".to_string();
            warnings.push(format!(
                "secret {} expired at {}",
                secret.name,
                secret.expires_at.clone().unwrap_or_default()
            ));
        }
        Ok(Some(expires_at))
            if expires_at
                <= Utc::now() + ChronoDuration::seconds(SECRET_EXPIRY_WARNING_WINDOW_SECONDS) =>
        {
            status = "expiring_soon".to_string();
            warnings.push(format!(
                "secret {} expires soon at {}",
                secret.name,
                secret.expires_at.clone().unwrap_or_default()
            ));
        }
        Err(error) => {
            status = "invalid_expiry".to_string();
            warnings.push(format!(
                "secret {} has invalid expiry metadata: {error}",
                secret.name
            ));
        }
        _ => {}
    }
    if secret.location.trim().is_empty() && !has_local_value {
        status = "missing".to_string();
        warnings.push(format!(
            "secret {} has no location or local value",
            secret.name
        ));
    }
    SecretHealth {
        name: secret.name.clone(),
        scope: secret.scope.clone(),
        provider: None,
        source: "ref".to_string(),
        present: has_local_value || !secret.location.trim().is_empty(),
        status,
        expires_at: secret.expires_at.clone(),
        updated_at: secret.updated_at.clone(),
        warnings,
    }
}

pub(crate) fn secret_value_health(secret: SecretValue) -> Result<SecretHealth> {
    let mut warnings = Vec::new();
    let mut status = "present".to_string();
    if let Some(expires_at) = parse_optional_expiry(secret.expires_at.as_deref())? {
        if expires_at <= Utc::now() {
            status = "expired".to_string();
            warnings.push(format!(
                "secret {} expired at {}",
                secret.name,
                secret.expires_at.clone().unwrap_or_default()
            ));
        } else if expires_at
            <= Utc::now() + ChronoDuration::seconds(SECRET_EXPIRY_WARNING_WINDOW_SECONDS)
        {
            status = "expiring_soon".to_string();
            warnings.push(format!(
                "secret {} expires soon at {}",
                secret.name,
                secret.expires_at.clone().unwrap_or_default()
            ));
        }
    }
    Ok(SecretHealth {
        name: secret.name,
        scope: secret.scope,
        provider: secret.provider,
        source: "local_sqlite".to_string(),
        present: true,
        status,
        expires_at: secret.expires_at,
        updated_at: secret.updated_at,
        warnings,
    })
}

pub(crate) fn missing_secret_health(
    name: &str,
    scope: &str,
    provider: Option<&str>,
    warning: &str,
) -> SecretHealth {
    SecretHealth {
        name: name.to_string(),
        scope: scope.to_string(),
        provider: provider.map(ToOwned::to_owned),
        source: "required".to_string(),
        present: false,
        status: "missing".to_string(),
        expires_at: None,
        updated_at: now(),
        warnings: vec![warning.to_string()],
    }
}

pub(crate) fn push_secret_warning(
    health: &mut BTreeMap<String, SecretHealth>,
    name: &str,
    scope: &str,
    provider: Option<&str>,
    warning: &str,
) {
    health
        .entry(name.to_string())
        .and_modify(|item| {
            if !item.warnings.iter().any(|existing| existing == warning) {
                item.warnings.push(warning.to_string());
            }
        })
        .or_insert_with(|| missing_secret_health(name, scope, provider, warning));
}

pub(crate) fn parse_json_column(raw: &str, index: usize) -> rusqlite::Result<Value> {
    serde_json::from_str(raw).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            index,
            rusqlite::types::Type::Text,
            Box::new(error),
        )
    })
}

pub(crate) fn parse_optional_json_column(
    raw: Option<&str>,
    index: usize,
) -> rusqlite::Result<Option<Value>> {
    raw.map(|raw| parse_json_column(raw, index)).transpose()
}

pub(crate) fn parse_json_string_vec_column(
    raw: &str,
    index: usize,
) -> rusqlite::Result<Vec<String>> {
    serde_json::from_str(raw).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            index,
            rusqlite::types::Type::Text,
            Box::new(error),
        )
    })
}

pub(crate) fn research_task_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ResearchTask> {
    Ok(ResearchTask {
        id: row.get(0)?,
        run_id: row.get(1)?,
        role: row.get(2)?,
        status: row.get(3)?,
        instructions: row.get(4)?,
        notes: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

pub(crate) fn markdown_title(content: &str) -> Option<String> {
    content.lines().find_map(|line| {
        line.strip_prefix("# ")
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToOwned::to_owned)
    })
}

pub(crate) fn is_markdown_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "md" | "markdown"))
        .unwrap_or(false)
}

pub(crate) fn wiki_id(title: &str, source: &str) -> String {
    let slug = slugify(title);
    let hash = sha256(format!("{title}\n{source}").as_bytes());
    format!("{slug}-{}", &hash[..8])
}

pub(crate) fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in input.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "untitled".to_string()
    } else {
        trimmed.chars().take(80).collect()
    }
}

pub(crate) fn validate_query(query: &str) -> Result<()> {
    if query.trim().is_empty() {
        bail!("query cannot be empty");
    }
    if query.len() > 500 {
        bail!("query is too long");
    }
    Ok(())
}

pub(crate) fn normalize_knowledge_model_cluster_query(query: &str) -> Result<String> {
    validate_query(query)?;
    if knowledge_model_cluster_query_is_broad(query) {
        Ok("source-cards".to_string())
    } else {
        Ok(query.trim().to_string())
    }
}

pub(crate) fn knowledge_model_cluster_query_is_broad(query: &str) -> bool {
    matches!(
        query.trim().to_ascii_lowercase().as_str(),
        "*" | "source-cards" | "all-source-cards"
    )
}

pub(crate) fn wiki_fts_query(query: &str) -> Option<String> {
    let tokens: Vec<String> = query
        .split(|ch: char| !ch.is_alphanumeric())
        .filter_map(|token| {
            let cleaned = token.trim().to_lowercase();
            if cleaned.len() < 2 {
                None
            } else {
                Some(format!("{cleaned}*"))
            }
        })
        .take(12)
        .collect();
    if tokens.is_empty() {
        None
    } else {
        Some(tokens.join(" "))
    }
}

pub(crate) fn x_fts_query(query: &str) -> Option<String> {
    wiki_fts_query(query)
}

pub(crate) fn validate_id(id: &str) -> Result<()> {
    if id.trim().is_empty() {
        bail!("id cannot be empty");
    }
    if id.len() > 120 {
        bail!("id is too long");
    }
    Ok(())
}

pub(crate) fn validate_notes(notes: &str) -> Result<()> {
    if notes.trim().is_empty() {
        bail!("notes cannot be empty");
    }
    if notes.len() > 20_000 {
        bail!("notes are too long");
    }
    Ok(())
}

pub(crate) fn validate_public_http_url(raw: &str) -> Result<Url> {
    let url = Url::parse(raw).with_context(|| format!("invalid URL: {raw}"))?;
    if !matches!(url.scheme(), "http" | "https") {
        bail!("URL must use http or https");
    }
    if url.host_str().is_none() {
        bail!("URL must include a host");
    }
    Ok(url)
}

pub(crate) fn validate_indexable_x_link_url(raw: &str) -> Result<Url> {
    let url = validate_public_http_url(raw)?;
    if is_blocked_fetch_host(&url) {
        bail!("X link URL host is not allowed");
    }
    Ok(url)
}

pub(crate) fn validate_fetch_url(raw: &str) -> Result<Url> {
    let url = validate_public_http_url(raw)?;
    if url.scheme() != "https" {
        if is_loopback_host(&url)
            && std::env::var("ARCWELL_ALLOW_LOOPBACK_URL_INGEST").as_deref() == Ok("1")
        {
            return Ok(url);
        }
        bail!("fetch URL must use https");
    }
    if is_blocked_fetch_host(&url) {
        bail!("fetch URL host is not allowed");
    }
    Ok(url)
}

pub(crate) fn validated_x_api_base(raw: &str) -> Result<Url> {
    let url = Url::parse(raw).with_context(|| format!("invalid X API base URL: {raw}"))?;
    if is_loopback_host(&url) {
        return Ok(url);
    }
    if url.scheme() != "https" || url.host_str() != Some("api.x.com") {
        bail!("X API base must be https://api.x.com or loopback for tests");
    }
    Ok(url)
}

pub(crate) fn validate_github_segment(segment: &str) -> Result<()> {
    validate_key(segment)?;
    if !segment
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        bail!("invalid GitHub owner/repo segment");
    }
    Ok(())
}

pub(crate) fn validate_github_mode(mode: &str) -> Result<()> {
    match mode {
        "releases" | "commits" => Ok(()),
        other => bail!("unsupported GitHub mode: {other}"),
    }
}

pub(crate) fn is_blocked_fetch_host(url: &Url) -> bool {
    let Some(host) = url.host_str() else {
        return true;
    };
    let host_lower = host.to_ascii_lowercase();
    if matches!(
        host_lower.as_str(),
        "localhost" | "metadata.google.internal"
    ) {
        return true;
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        return match ip {
            IpAddr::V4(ip) => {
                ip.is_private()
                    || ip.is_loopback()
                    || ip.is_link_local()
                    || ip.is_broadcast()
                    || ip.is_documentation()
                    || ip.octets()[0] == 0
                    || ip.octets()[0] >= 224
            }
            IpAddr::V6(ip) => ip.is_loopback() || ip.is_unspecified() || ip.is_unique_local(),
        };
    }
    false
}

pub(crate) fn validate_source_card_input(input: &SourceCardInput) -> Result<()> {
    validate_query(&input.title)?;
    validate_public_http_url(&input.url)?;
    validate_key(&input.source_type)?;
    validate_key(&input.provider)?;
    validate_notes(&input.summary)?;
    validate_source_card_metadata(&input.metadata)?;
    if source_card_metadata_string(&input.metadata, "source_role").as_deref() == Some("primary")
        && is_generated_source_card_input(input)
    {
        bail!("generated research output cannot be primary source-card evidence");
    }
    if input.claims.len() > 50 {
        bail!("too many source claims");
    }
    for claim in &input.claims {
        validate_notes(&claim.claim)?;
        validate_key(&claim.kind)?;
        if !(0.0..=1.0).contains(&claim.confidence) {
            bail!("claim confidence must be between 0 and 1");
        }
    }
    Ok(())
}

pub(crate) fn normalize_research_source_input(
    mut input: ResearchSourceInput,
) -> Result<ResearchSourceInput> {
    input.title = input.title.trim().to_string();
    validate_query(&input.title)?;
    input.source_family = input.source_family.trim().to_string();
    input.source_type = input.source_type.trim().to_string();
    input.provider = input.provider.trim().to_string();
    input.fetch_status = input.fetch_status.trim().to_string();
    input.read_depth = input.read_depth.trim().to_string();
    validate_key(&input.source_family)?;
    validate_key(&input.source_type)?;
    validate_key(&input.provider)?;
    validate_key(&input.fetch_status)?;
    validate_key(&input.read_depth)?;
    if !(0..=10_000).contains(&input.priority) {
        bail!("research source priority must be between 0 and 10000");
    }
    input.reason = input.reason.trim().to_string();
    validate_notes(&input.reason)?;
    validate_research_metadata(&input.metadata)?;
    input.url = input
        .url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            validate_fetch_url(value)?;
            canonical_source_url(value)
        })
        .transpose()?;
    input.local_ref = normalize_optional_research_text(input.local_ref, "local_ref", 500)?;
    input.author = normalize_optional_research_text(input.author, "author", 300)?;
    input.published_at = normalize_optional_research_text(input.published_at, "published_at", 100)?;
    input.language = normalize_optional_research_text(input.language, "language", 80)?;
    if input.url.is_none() && input.local_ref.is_none() {
        bail!("research source needs a url or local_ref");
    }
    let canonical_key = input
        .canonical_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| input.url.as_ref().map(|url| format!("url:{url}")))
        .or_else(|| {
            input
                .local_ref
                .as_ref()
                .map(|local_ref| format!("local:{local_ref}"))
        })
        .context("research source canonical key missing")?;
    if canonical_key.len() > 1_000 {
        bail!("research source canonical key is too long");
    }
    input.canonical_key = Some(canonical_key);
    Ok(input)
}

pub(crate) fn normalize_optional_research_text(
    value: Option<String>,
    label: &str,
    max_len: usize,
) -> Result<Option<String>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.len() > max_len {
        bail!("research source {label} is too long");
    }
    Ok(Some(trimmed.to_string()))
}

pub(crate) fn validate_research_source_link_input(
    triage_status: &str,
    read_depth: &str,
    notes: Option<&str>,
) -> Result<()> {
    validate_key(triage_status)?;
    validate_key(read_depth)?;
    if let Some(notes) = notes {
        validate_notes(notes)?;
    }
    Ok(())
}

pub(crate) fn validate_research_metadata(metadata: &Value) -> Result<()> {
    if metadata.is_null() {
        return Ok(());
    }
    if !metadata.is_object() {
        bail!("research source metadata must be an object");
    }
    Ok(())
}

const RESEARCH_ARTIFACT_BODY_MAX: usize = 120_000;
const RESEARCH_ARTIFACT_TITLE_MAX: usize = 300;
const RESEARCH_ARTIFACT_METADATA_MAX: usize = 60_000;
const RESEARCH_ROLE_INPUT_ARTIFACT_MAX: usize = 50;

pub(crate) fn normalize_research_role_run_start(
    mut input: ResearchRoleRunStart,
) -> Result<ResearchRoleRunStart> {
    validate_id(&input.run_id)?;
    input.role = normalize_research_key(input.role, "research role")?;
    input.host = normalize_research_key(input.host, "research host")?;
    input.prompt_version = normalize_research_key(input.prompt_version, "prompt version")?;
    input.execution_mode = normalize_research_role_execution_mode(&input.execution_mode)?;
    input.host_thread_id =
        normalize_optional_research_text(input.host_thread_id, "host_thread_id", 500)?;
    input.host_subagent_id =
        normalize_optional_research_text(input.host_subagent_id, "host_subagent_id", 500)?;
    input.tool_surface = normalize_optional_research_text(input.tool_surface, "tool_surface", 500)?;
    input.prompt_hash = normalize_optional_research_text(input.prompt_hash, "prompt_hash", 256)?;
    if input.input_artifact_ids.len() > RESEARCH_ROLE_INPUT_ARTIFACT_MAX {
        bail!("too many input artifacts for research role run");
    }
    let mut deduped = Vec::new();
    for artifact_id in input.input_artifact_ids {
        validate_id(&artifact_id)?;
        if !deduped.contains(&artifact_id) {
            deduped.push(artifact_id);
        }
    }
    input.input_artifact_ids = deduped;
    Ok(input)
}

pub(crate) fn normalize_research_key(value: String, label: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        bail!("{label} cannot be empty");
    }
    validate_key(trimmed)?;
    Ok(trimmed.to_string())
}

pub(crate) fn normalize_research_role_execution_mode(mode: &str) -> Result<String> {
    let mode = mode.trim();
    match mode {
        "codex_subagent_live" | "host_sequential" | "simulated_test" => Ok(mode.to_string()),
        other => bail!("unsupported research role execution mode: {other}"),
    }
}

pub(crate) fn validate_research_role_run_finish(
    status: &str,
    error_kind: Option<&str>,
    error_message: Option<&str>,
) -> Result<()> {
    match status {
        "completed" => {
            if error_kind.is_some() || error_message.is_some() {
                bail!("completed research role run cannot include an error");
            }
        }
        "failed" | "rejected" | "cancelled" => {
            if error_kind.is_none() && error_message.is_none() {
                bail!("{status} research role run requires an error kind or message");
            }
            if let Some(kind) = error_kind {
                validate_key(kind.trim())?;
            }
            if let Some(message) = error_message {
                validate_notes(message)?;
            }
        }
        other => bail!("unsupported research role run status: {other}"),
    }
    Ok(())
}

pub(crate) fn normalize_research_artifact_input(
    mut input: ResearchArtifactInput,
) -> Result<ResearchArtifactInput> {
    validate_id(&input.run_id)?;
    input.role_run_id = input
        .role_run_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| -> Result<String> {
            validate_id(value)?;
            Ok(value.to_string())
        })
        .transpose()?;
    input.artifact_type = normalize_research_key(input.artifact_type, "research artifact type")?;
    input.title = sanitize_work_text(&input.title, RESEARCH_ARTIFACT_TITLE_MAX)?;
    if input.title.trim().is_empty() {
        bail!("research artifact title cannot be empty");
    }
    input.body = sanitize_work_text(&input.body, RESEARCH_ARTIFACT_BODY_MAX)?;
    if input.body.trim().is_empty() {
        bail!("research artifact body cannot be empty");
    }
    input.metadata = sanitize_work_json(input.metadata)?;
    if serde_json::to_string(&input.metadata)?.len() > RESEARCH_ARTIFACT_METADATA_MAX {
        bail!("research artifact metadata is too large after redaction");
    }
    Ok(input)
}

pub(crate) const COMMERCE_MAX_LIST_ITEMS: usize = 50;
pub(crate) const COMMERCE_MAX_OPTIONAL_TEXT: usize = 1_000;
pub(crate) const COMMERCE_MAX_EVIDENCE_TEXT: usize = 4_000;
pub(crate) const COMMERCE_MAX_TARGET_QUALIFIED: usize = 200;
pub(crate) const COMMERCE_MAX_PROVIDER_CALLS: usize = 10_000;
pub(crate) const COMMERCE_MAX_BROWSER_PAGES: usize = 10_000;
pub(crate) const JOB_MAX_LIST_ITEMS: usize = 100;
pub(crate) const JOB_MAX_IMPORT_ITEMS: usize = 1_000;
pub(crate) const JOB_MAX_SHORT_TEXT: usize = 500;
pub(crate) const JOB_MAX_TEXT: usize = 4_000;
pub(crate) const JOB_MAX_PACKET_TEXT: usize = 12_000;
pub(crate) const JOB_MAX_SOURCE_REFRESH_BODY_CHARS: usize = 1_000_000;
pub(crate) const JOB_SOURCE_REFRESH_NEW_ROLE_EVENT_NOTE: &str =
    "Job source refresh observed this role for the first time.";

pub(crate) fn normalize_job_candidate_profile_input(
    mut input: JobCandidateProfileInput,
) -> Result<JobCandidateProfileInput> {
    input.label = sanitize_required_job_text(&input.label, "profile label", JOB_MAX_SHORT_TEXT)?;
    input.current_resume_source =
        normalize_optional_job_text(input.current_resume_source, "current_resume_source", 1_000)?;
    input.linkedin_source =
        normalize_optional_job_text(input.linkedin_source, "linkedin_source", 1_000)?;
    input.github_profile =
        normalize_optional_job_text(input.github_profile, "github_profile", 1_000)?;
    input.blog_url = normalize_optional_job_url(input.blog_url, "blog_url")?;
    input.metadata = sanitize_work_json(input.metadata)?;
    Ok(input)
}

pub(crate) fn normalize_job_evidence_card_input(
    mut input: JobEvidenceCardInput,
) -> Result<JobEvidenceCardInput> {
    validate_id(&input.profile_id)?;
    input.title = sanitize_required_job_text(&input.title, "evidence title", JOB_MAX_SHORT_TEXT)?;
    input.evidence_type = normalize_research_key(input.evidence_type, "evidence type")?;
    input.visibility = normalize_job_visibility(&input.visibility)?;
    input.summary = sanitize_required_job_text(&input.summary, "evidence summary", JOB_MAX_TEXT)?;
    input.proof_url = normalize_optional_job_url(input.proof_url, "proof_url")?;
    input.local_path = normalize_optional_job_text(input.local_path, "local_path", 1_000)?;
    input.source_date = normalize_optional_job_text(input.source_date, "source_date", 100)?;
    input.confidence = normalize_job_evidence_confidence(&input.confidence)?;
    input.tags = normalize_job_key_list(input.tags, "evidence tag")?;
    input.safe_application_text = sanitize_required_job_text(
        &input.safe_application_text,
        "safe_application_text",
        JOB_MAX_TEXT,
    )?;
    input.unsafe_terms = normalize_job_string_list(input.unsafe_terms, "unsafe term", 300)?;
    input.metadata = sanitize_work_json(input.metadata)?;
    Ok(input)
}

pub(crate) fn normalize_job_evidence_claim_input(
    mut input: JobEvidenceClaimInput,
) -> Result<JobEvidenceClaimInput> {
    validate_id(&input.evidence_card_id)?;
    input.claim = sanitize_required_job_text(&input.claim, "evidence claim", JOB_MAX_TEXT)?;
    input.claim_kind = normalize_research_key(input.claim_kind, "claim kind")?;
    input.proof_level = normalize_job_claim_proof_level(&input.proof_level)?;
    Ok(input)
}

pub(crate) fn normalize_job_privacy_rule_input(
    mut input: JobPrivacyRuleInput,
) -> Result<JobPrivacyRuleInput> {
    input.pattern = sanitize_required_job_text(&input.pattern, "privacy pattern", 500)?;
    input.rule_type = normalize_job_privacy_rule_type(&input.rule_type)?;
    input.severity = normalize_job_privacy_severity(&input.severity)?;
    input.replacement_guidance =
        normalize_optional_job_text(input.replacement_guidance, "replacement_guidance", 1_000)?;
    Ok(input)
}

pub(crate) fn normalize_job_source_input(mut input: JobSourceInput) -> Result<JobSourceInput> {
    input.source_family = normalize_research_key(input.source_family, "source family")?;
    input.name = sanitize_required_job_text(&input.name, "source name", JOB_MAX_SHORT_TEXT)?;
    input.url = canonical_source_url(input.url.trim())?;
    input.market_scope = normalize_research_key(input.market_scope, "market scope")?;
    input.refresh_policy = normalize_research_key(input.refresh_policy, "refresh policy")?;
    input.metadata = sanitize_work_json(input.metadata)?;
    Ok(input)
}

pub(crate) fn normalize_job_source_health_input(
    mut input: JobSourceHealthInput,
) -> Result<JobSourceHealthInput> {
    validate_id(&input.source_id)?;
    input.status = normalize_job_source_health_status(&input.status)?;
    if let Some(http_status) = input.http_status {
        if !(100..=599).contains(&http_status) {
            bail!("job source health http_status must be between 100 and 599");
        }
    }
    input.error_code = input
        .error_code
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| normalize_research_key(value.to_string(), "error_code"))
        .transpose()?;
    input.note = normalize_optional_job_text(input.note, "source health note", JOB_MAX_TEXT)?;
    Ok(input)
}

pub(crate) fn normalize_job_role_card_input(
    mut input: JobRoleCardInput,
) -> Result<JobRoleCardInput> {
    input.company = sanitize_required_job_text(&input.company, "company", JOB_MAX_SHORT_TEXT)?;
    input.role_title =
        sanitize_required_job_text(&input.role_title, "role title", JOB_MAX_SHORT_TEXT)?;
    input.canonical_url = normalize_optional_job_url(input.canonical_url, "canonical_url")?;
    input.source_family = normalize_research_key(input.source_family, "role source family")?;
    input.source_url = canonical_source_url(input.source_url.trim())?;
    input.source_confidence = normalize_job_source_confidence(&input.source_confidence)?;
    if input.source_confidence == "canonical_confirmed" && input.canonical_url.is_none() {
        bail!("canonical-confirmed job role requires canonical_url");
    }
    input.date_accessed = normalize_optional_job_text(input.date_accessed, "date_accessed", 100)?;
    input.posting_freshness = normalize_research_key(input.posting_freshness, "posting freshness")?;
    input.location = normalize_optional_job_text(input.location, "location", JOB_MAX_SHORT_TEXT)?;
    input.work_mode =
        normalize_optional_job_text(input.work_mode, "work_mode", JOB_MAX_SHORT_TEXT)?;
    input.company_stage_or_size = normalize_optional_job_text(
        input.company_stage_or_size,
        "company_stage_or_size",
        JOB_MAX_SHORT_TEXT,
    )?;
    input.role_seniority =
        normalize_optional_job_text(input.role_seniority, "role_seniority", JOB_MAX_SHORT_TEXT)?;
    input.core_requirements =
        normalize_job_string_list(input.core_requirements, "core requirement", JOB_MAX_TEXT)?;
    input.implied_business_problem = normalize_optional_job_text(
        input.implied_business_problem,
        "implied_business_problem",
        JOB_MAX_TEXT,
    )?;
    input.why_they_might_need_user = normalize_optional_job_text(
        input.why_they_might_need_user,
        "why_they_might_need_user",
        JOB_MAX_TEXT,
    )?;
    input.evidence_card_ids = normalize_job_id_list(input.evidence_card_ids, "evidence_card_id")?;
    input.gaps_or_blockers =
        normalize_job_string_list(input.gaps_or_blockers, "gap or blocker", JOB_MAX_TEXT)?;
    input.cluster = normalize_optional_job_text(input.cluster, "cluster", JOB_MAX_SHORT_TEXT)?;
    input.current_status = normalize_job_role_status(&input.current_status)?;
    input.metadata = sanitize_work_json(input.metadata)?;
    Ok(input)
}

pub(crate) fn normalize_job_role_source_link_input(
    mut input: JobRoleSourceLinkInput,
) -> Result<JobRoleSourceLinkInput> {
    validate_id(&input.role_id)?;
    input.source_id = normalize_optional_id(input.source_id, "source_id")?;
    input.source_url = canonical_source_url(input.source_url.trim())?;
    input.confidence = normalize_job_source_confidence(&input.confidence)?;
    input.evidence_excerpt =
        normalize_optional_job_text(input.evidence_excerpt, "evidence_excerpt", JOB_MAX_TEXT)?;
    Ok(input)
}

pub(crate) fn normalize_job_fit_score_input(
    mut input: JobFitScoreInput,
) -> Result<JobFitScoreInput> {
    validate_id(&input.role_id)?;
    validate_id(&input.profile_id)?;
    input.scorer = normalize_research_key(input.scorer, "job score scorer")?;
    validate_job_fit_dimension(input.role_fit, "role_fit")?;
    validate_job_fit_dimension(input.domain_fit, "domain_fit")?;
    validate_job_fit_dimension(input.evidence_fit, "evidence_fit")?;
    validate_job_fit_dimension(input.geo_work_fit, "geo_work_fit")?;
    validate_job_fit_dimension(input.stage_fit, "stage_fit")?;
    validate_job_fit_dimension(input.practical_odds, "practical_odds")?;
    validate_job_fit_dimension(input.interest_energy, "interest_energy")?;
    input.blockers = normalize_job_string_list(input.blockers, "score blocker", 500)?;
    input.evidence_card_ids = normalize_job_id_list(input.evidence_card_ids, "evidence_card_id")?;
    if input.evidence_fit > 2.0 && input.evidence_card_ids.is_empty() {
        bail!("job evidence_fit above 2 requires linked evidence cards");
    }
    input.explanation =
        sanitize_required_job_text(&input.explanation, "score explanation", JOB_MAX_TEXT)?;
    Ok(input)
}

pub(crate) fn normalize_job_skeptic_finding_input(
    mut input: JobSkepticFindingInput,
) -> Result<JobSkepticFindingInput> {
    validate_id(&input.role_id)?;
    input.severity = normalize_job_privacy_severity(&input.severity)?;
    input.finding_type = normalize_research_key(input.finding_type, "skeptic finding type")?;
    input.finding = sanitize_required_job_text(&input.finding, "skeptic finding", JOB_MAX_TEXT)?;
    input.next_action =
        normalize_optional_job_text(input.next_action, "next_action", JOB_MAX_TEXT)?;
    Ok(input)
}

pub(crate) fn normalize_job_application_packet_input(
    mut input: JobApplicationPacketInput,
) -> Result<JobApplicationPacketInput> {
    validate_id(&input.role_id)?;
    validate_id(&input.profile_id)?;
    input.evidence_card_ids = normalize_job_id_list(input.evidence_card_ids, "evidence_card_id")?;
    input.resume_emphasis = sanitize_required_job_text(
        &input.resume_emphasis,
        "resume_emphasis",
        JOB_MAX_PACKET_TEXT,
    )?;
    input.tailored_bullets =
        normalize_job_string_list(input.tailored_bullets, "tailored bullet", JOB_MAX_TEXT)?;
    input.outreach_note =
        sanitize_required_job_text(&input.outreach_note, "outreach_note", JOB_MAX_TEXT)?;
    input.proof_links = sanitize_work_json(input.proof_links)?;
    input.likely_objections =
        normalize_job_string_list(input.likely_objections, "likely objection", JOB_MAX_TEXT)?;
    input.interview_stories =
        normalize_job_string_list(input.interview_stories, "interview story", JOB_MAX_TEXT)?;
    input.questions_to_ask =
        normalize_job_string_list(input.questions_to_ask, "question to ask", JOB_MAX_TEXT)?;
    input.reviewer_note =
        normalize_optional_job_text(input.reviewer_note, "reviewer_note", JOB_MAX_TEXT)?;
    Ok(input)
}

pub(crate) fn normalize_job_application_packet_status_input(
    mut input: JobApplicationPacketStatusInput,
) -> Result<JobApplicationPacketStatusInput> {
    validate_id(&input.packet_id)?;
    input.status = normalize_job_application_packet_status(&input.status)?;
    input.reviewer_note =
        normalize_optional_job_text(input.reviewer_note, "reviewer_note", JOB_MAX_TEXT)?;
    Ok(input)
}

pub(crate) fn normalize_job_company_card_input(
    mut input: JobCompanyCardInput,
) -> Result<JobCompanyCardInput> {
    input.company_name =
        sanitize_required_job_text(&input.company_name, "company_name", JOB_MAX_SHORT_TEXT)?;
    input.website_url = canonical_source_url(input.website_url.trim())?;
    input.source_family = normalize_research_key(input.source_family, "company source family")?;
    input.market = normalize_research_key(input.market, "company market")?;
    input.stage = normalize_optional_job_text(input.stage, "stage", JOB_MAX_SHORT_TEXT)?;
    input.funding_signal =
        normalize_optional_job_text(input.funding_signal, "funding_signal", JOB_MAX_TEXT)?;
    input.product_category = normalize_optional_job_text(
        input.product_category,
        "product_category",
        JOB_MAX_SHORT_TEXT,
    )?;
    input.technical_audience =
        normalize_optional_job_text(input.technical_audience, "technical_audience", JOB_MAX_TEXT)?;
    validate_job_fit_dimension(input.developer_facing_score, "developer_facing_score")?;
    input.london_relevance =
        sanitize_required_job_text(&input.london_relevance, "london_relevance", JOB_MAX_TEXT)?;
    input.remote_maturity =
        normalize_optional_job_text(input.remote_maturity, "remote_maturity", JOB_MAX_TEXT)?;
    input.hiring_page_url = normalize_optional_job_url(input.hiring_page_url, "hiring_page_url")?;
    input.founder_or_team_signal = normalize_optional_job_text(
        input.founder_or_team_signal,
        "founder_or_team_signal",
        JOB_MAX_TEXT,
    )?;
    input.metadata = sanitize_work_json(input.metadata)?;
    Ok(input)
}

pub(crate) fn normalize_job_contact_input(mut input: JobContactInput) -> Result<JobContactInput> {
    input.name = sanitize_required_job_text(&input.name, "contact name", JOB_MAX_SHORT_TEXT)?;
    input.company_id = normalize_optional_id(input.company_id, "company_id")?;
    input.role_title =
        normalize_optional_job_text(input.role_title, "role_title", JOB_MAX_SHORT_TEXT)?;
    input.public_profile_url = canonical_source_url(input.public_profile_url.trim())?;
    input.source_url = canonical_source_url(input.source_url.trim())?;
    input.relationship_status = normalize_job_relationship_status(&input.relationship_status)?;
    input.relevance = normalize_job_contact_relevance(&input.relevance)?;
    input.note = normalize_optional_job_text(input.note, "contact note", JOB_MAX_TEXT)?;
    validate_job_contact_relevance_evidence(&input)?;
    Ok(input)
}

pub(crate) fn normalize_job_intro_path_input(
    mut input: JobIntroPathInput,
) -> Result<JobIntroPathInput> {
    validate_id(&input.role_id)?;
    validate_id(&input.contact_id)?;
    input.path_type = normalize_job_intro_path_type(&input.path_type)?;
    input.confidence = normalize_job_intro_confidence(&input.confidence)?;
    input.next_action =
        normalize_optional_job_text(input.next_action, "next_action", JOB_MAX_TEXT)?;
    input.status = normalize_job_intro_status(&input.status)?;
    Ok(input)
}

pub(crate) fn normalize_job_search_run_input(
    mut input: JobSearchRunInput,
) -> Result<JobSearchRunInput> {
    validate_id(&input.profile_id)?;
    input.scope = sanitize_required_job_text(&input.scope, "search scope", JOB_MAX_TEXT)?;
    input.proof_level = normalize_job_proof_level(&input.proof_level)?;
    input.report_artifact_id =
        normalize_optional_id(input.report_artifact_id, "report_artifact_id")?;
    input.completed_at = normalize_optional_job_text(input.completed_at, "completed_at", 100)?;
    Ok(input)
}

pub(crate) fn normalize_job_role_status_event_input(
    mut input: JobRoleStatusEventInput,
) -> Result<JobRoleStatusEventInput> {
    validate_id(&input.role_id)?;
    input.run_id = normalize_optional_id(input.run_id, "run_id")?;
    input.status = normalize_job_role_event_status(&input.status)?;
    input.previous_tier = normalize_optional_job_text(input.previous_tier, "previous_tier", 100)?;
    input.current_tier = normalize_optional_job_text(input.current_tier, "current_tier", 100)?;
    input.note = normalize_optional_job_text(input.note, "status note", JOB_MAX_TEXT)?;
    Ok(input)
}

pub(crate) fn normalize_job_application_input(
    mut input: JobApplicationInput,
) -> Result<JobApplicationInput> {
    validate_id(&input.role_id)?;
    input.packet_id = normalize_optional_id(input.packet_id, "packet_id")?;
    input.status = normalize_job_application_status(&input.status)?;
    input.applied_at = normalize_optional_job_text(input.applied_at, "applied_at", 100)?;
    input.follow_up_at = normalize_optional_job_text(input.follow_up_at, "follow_up_at", 100)?;
    input.outcome_note =
        normalize_optional_job_text(input.outcome_note, "outcome_note", JOB_MAX_TEXT)?;
    Ok(input)
}

pub(crate) fn normalize_job_weekly_report_delivery_input(
    mut input: JobWeeklyReportDeliveryInput,
) -> Result<JobWeeklyReportDeliveryInput> {
    validate_id(&input.report_id)?;
    input.channel = normalize_radar_delivery_channel(&input.channel)?;
    input.subject = normalize_radar_delivery_recipient(&input.channel, &input.subject)?;
    input.target = normalize_radar_delivery_recipient(&input.channel, &input.target)?;
    input.idempotency_key = Some(match input.idempotency_key.as_deref() {
        Some(explicit) => {
            validate_query(explicit)?;
            explicit.trim().to_string()
        }
        None => format!(
            "job-weekly-report-delivery-{}",
            &sha256(
                format!(
                    "{}\n{}\n{}\n{}",
                    input.report_id, input.channel, input.subject, input.target
                )
                .as_bytes()
            )[..32]
        ),
    });
    Ok(input)
}

pub(crate) fn normalize_job_weekly_report_delivery_send_input(
    input: JobWeeklyReportDeliverySendInput,
) -> Result<JobWeeklyReportDeliverySendInput> {
    validate_id(&input.delivery_id)?;
    if let Some(value) = input.telegram_bot_token.as_deref() {
        validate_notes(value)?;
    }
    if let Some(value) = input.email_account_id.as_deref() {
        validate_key(value)?;
    }
    if let Some(value) = input.email_api_token.as_deref() {
        validate_notes(value)?;
    }
    if let Some(value) = input.email_from.as_deref() {
        normalize_email_address(value).context("invalid email from address")?;
    }
    if let Some(value) = input.api_base.as_deref() {
        let url = validate_public_http_url(value)?;
        if url.scheme() == "http" && !is_loopback_host(&url) {
            bail!("provider API base must use https or loopback http");
        }
    }
    Ok(input)
}

pub(crate) fn normalize_job_manual_refresh_input(
    mut input: JobManualRefreshInput,
) -> Result<JobManualRefreshInput> {
    validate_id(&input.profile_id)?;
    input.scope = sanitize_required_job_text(&input.scope, "refresh scope", JOB_MAX_TEXT)?;
    input.observed_role_ids =
        normalize_job_refresh_id_list(input.observed_role_ids, "observed_role_id")?;
    input.stale_role_ids = normalize_job_refresh_id_list(input.stale_role_ids, "stale_role_id")?;
    input.closed_role_ids = normalize_job_refresh_id_list(input.closed_role_ids, "closed_role_id")?;
    input.source_health_ids =
        normalize_job_refresh_id_list(input.source_health_ids, "source_health_id")?;
    input.proof_level = normalize_job_proof_level(&input.proof_level)?;
    input.report_artifact_id =
        normalize_optional_id(input.report_artifact_id, "report_artifact_id")?;
    let mut terminal = BTreeSet::new();
    for role_id in input
        .stale_role_ids
        .iter()
        .chain(input.closed_role_ids.iter())
    {
        if !terminal.insert(role_id.clone()) {
            bail!("job refresh role cannot be both stale and closed");
        }
    }
    Ok(input)
}

pub(crate) fn normalize_job_source_refresh_input(
    mut input: JobSourceRefreshInput,
) -> Result<JobSourceRefreshInput> {
    validate_id(&input.source_id)?;
    input.body = input
        .body
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(sanitize_job_source_refresh_body_input)
        .transpose()?;
    input.fetched_url = input
        .fetched_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(canonical_source_url)
        .transpose()?;
    if input.body.is_none() && !input.fetch_live {
        bail!("job source refresh requires body or fetch_live=true");
    }
    if input.body.is_some() && input.fetch_live {
        bail!("job source refresh cannot mix caller-supplied body with fetch_live=true");
    }
    Ok(input)
}

pub(crate) fn sanitize_job_source_refresh_body_input(input: &str) -> Result<String> {
    let without_controls = input
        .chars()
        .filter(|ch| *ch == '\n' || *ch == '\t' || !ch.is_control())
        .collect::<String>();
    let mut output = without_controls
        .chars()
        .take(JOB_MAX_SOURCE_REFRESH_BODY_CHARS)
        .collect::<String>();
    if without_controls.chars().count() > JOB_MAX_SOURCE_REFRESH_BODY_CHARS {
        output.push_str(" [TRUNCATED]");
    }
    if job_source_refresh_body_is_json(&output) {
        Ok(output)
    } else {
        sanitize_work_text(input, JOB_MAX_SOURCE_REFRESH_BODY_CHARS)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct JobSourceSnapshot {
    pub(crate) body: String,
    pub(crate) fetched_url: Option<String>,
}

pub(crate) fn job_source_ids_from_value(input: Option<&Value>) -> Result<Vec<String>> {
    let value = input.context("job radar refresh requires source_ids")?;
    let Some(items) = value.as_array() else {
        bail!("job radar refresh source_ids must be an array");
    };
    let mut source_ids = Vec::new();
    for item in items {
        let Some(source_id) = item.as_str() else {
            bail!("job radar refresh source_ids must contain strings");
        };
        source_ids.push(source_id.to_string());
    }
    let source_ids = normalize_job_id_list(source_ids, "job source id")?;
    if source_ids.is_empty() {
        bail!("job radar refresh requires at least one source id");
    }
    if source_ids.len() > 50 {
        bail!("job radar refresh has too many source ids");
    }
    Ok(source_ids)
}

pub(crate) fn job_source_snapshot_for(
    source_snapshots: &Value,
    source_id: &str,
) -> Result<Option<JobSourceSnapshot>> {
    validate_id(source_id)?;
    if source_snapshots.is_null() {
        return Ok(None);
    }
    let Some(map) = source_snapshots.as_object() else {
        bail!("job radar source_snapshots must be an object keyed by source id");
    };
    let Some(snapshot) = map.get(source_id) else {
        return Ok(None);
    };
    if let Some(body) = snapshot.as_str() {
        return Ok(Some(JobSourceSnapshot {
            body: sanitize_work_text(body, JOB_MAX_SOURCE_REFRESH_BODY_CHARS)?,
            fetched_url: None,
        }));
    }
    let Some(snapshot) = snapshot.as_object() else {
        bail!("job radar source snapshot for {source_id} must be a string or object");
    };
    let body = snapshot
        .get("body")
        .and_then(Value::as_str)
        .context("job radar source snapshot requires body")?;
    let fetched_url = snapshot
        .get("fetched_url")
        .or_else(|| snapshot.get("url"))
        .and_then(Value::as_str)
        .map(str::to_string);
    Ok(Some(JobSourceSnapshot {
        body: sanitize_work_text(body, JOB_MAX_SOURCE_REFRESH_BODY_CHARS)?,
        fetched_url,
    }))
}

pub(crate) fn job_radar_refresh_derived_proof_level(
    fetch_live: bool,
    source_ids: &[String],
    source_snapshots: &Value,
) -> Result<String> {
    if !fetch_live {
        return Ok("local_proof".to_string());
    }
    for source_id in source_ids {
        if job_source_snapshot_for(source_snapshots, source_id)?.is_none() {
            return Ok("production_data_proof".to_string());
        }
    }
    Ok("local_proof".to_string())
}

pub(crate) fn job_import_batch_item_count(input: &JobImportBatchInput) -> usize {
    usize::from(input.profile.is_some())
        + input.evidence_cards.len()
        + input.evidence_claims.len()
        + input.privacy_rules.len()
        + input.sources.len()
        + input.source_health.len()
        + input.roles.len()
        + input.role_source_links.len()
        + input.fit_scores.len()
        + input.skeptic_findings.len()
        + input.packets.len()
        + input.companies.len()
        + input.contacts.len()
        + input.intro_paths.len()
        + input.search_runs.len()
        + input.role_status_events.len()
        + input.applications.len()
}

#[derive(Debug)]
pub(crate) struct ParsedJobSourceRefresh {
    pub(crate) roles: Vec<JobRoleCardInput>,
    pub(crate) companies: Vec<JobCompanyCardInput>,
    pub(crate) fetched_count: usize,
    pub(crate) rejected_count: usize,
    pub(crate) no_openings_signal: bool,
    pub(crate) warnings: Vec<String>,
    pub(crate) readable_text: String,
    pub(crate) direct_role_title: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct JobRefreshAnchor {
    pub(crate) text: String,
    pub(crate) url: String,
}

pub(crate) fn parse_job_source_refresh_body(
    source: &JobSource,
    body: &str,
    fetched_url: &str,
    proof_level: &str,
) -> Result<ParsedJobSourceRefresh> {
    if let Some(parsed) =
        parse_job_source_refresh_structured_json(source, body, fetched_url, proof_level)?
    {
        return Ok(parsed);
    }
    let body = sanitize_work_text(body, JOB_MAX_SOURCE_REFRESH_BODY_CHARS)?;
    let html_like = job_source_refresh_body_is_html(&body);
    let readable = if html_like {
        html_to_readable_text(&body).text
    } else {
        normalize_readable_text(&body)
    };
    let no_openings_signal = job_refresh_has_no_openings_signal(&readable);
    let anchors = if html_like {
        job_html_anchor_links(&body, fetched_url)?
    } else {
        Vec::new()
    };
    let mut warnings = Vec::new();
    if anchors.is_empty() && html_like {
        warnings.push("No links were extracted from the job source snapshot.".to_string());
    }

    let mut roles = Vec::new();
    let mut companies = Vec::new();
    let mut seen_role_keys = BTreeSet::new();
    let mut seen_company_urls = BTreeSet::new();
    let mut rejected_count = 0usize;

    if let Some(company) = job_source_self_company_card(source, fetched_url, &readable) {
        seen_company_urls.insert(company.website_url.clone());
        companies.push(company);
    }

    let direct_job_detail_page = html_like
        && job_source_family_is_canonical_role_source(source)
        && job_refresh_url_looks_like_job_detail(fetched_url);
    let mut direct_role_title = None;

    if direct_job_detail_page {
        if let Some(title) = job_role_title_from_html_metadata(&body) {
            direct_role_title = Some(title.clone());
            if let Some(role) = job_role_input_from_observed_title(
                source,
                &title,
                fetched_url,
                Some(&title),
                proof_level,
            )? {
                let key = job_role_refresh_key(&role.company, &role.role_title, &role.source_url);
                if seen_role_keys.insert(key) {
                    roles.push(role);
                }
            }
        } else {
            warnings.push(
                "Direct job-detail page did not expose a usable page-level role title.".to_string(),
            );
        }
    }

    if !direct_job_detail_page {
        for anchor in anchors.iter().take(250) {
            if let Some(role) = job_role_input_from_refresh_anchor(source, anchor, proof_level)? {
                let key = job_role_refresh_key(&role.company, &role.role_title, &role.source_url);
                if seen_role_keys.insert(key) {
                    roles.push(role);
                }
                continue;
            }
            if let Some(company) = job_company_input_from_refresh_anchor(source, anchor, &readable)?
            {
                if seen_company_urls.insert(company.website_url.clone()) {
                    companies.push(company);
                }
                continue;
            }
            if job_refresh_anchor_looks_like_weak_job_lead(anchor) {
                rejected_count += 1;
            }
        }
    }

    if roles.is_empty() && !html_like && !no_openings_signal {
        for line in job_plaintext_role_lines(&readable).into_iter().take(25) {
            let Some(role) =
                job_role_input_from_plaintext_line(source, &line, fetched_url, proof_level)?
            else {
                continue;
            };
            let key = job_role_refresh_key(&role.company, &role.role_title, &role.source_url);
            if seen_role_keys.insert(key) {
                roles.push(role);
            }
        }
    }

    let fetched_count = roles.len() + companies.len() + rejected_count;
    if fetched_count == 0 && no_openings_signal {
        warnings.push("Source text says there are no current openings.".to_string());
    }
    Ok(ParsedJobSourceRefresh {
        roles,
        companies,
        fetched_count,
        rejected_count,
        no_openings_signal,
        warnings,
        readable_text: readable,
        direct_role_title,
    })
}

#[derive(Debug)]
struct ParsedStructuredJobPayload {
    roles: Vec<JobRoleCardInput>,
    fetched_posting_count: usize,
    rejected_count: usize,
    warnings: Vec<String>,
    direct_role_title: Option<String>,
    no_openings_signal: bool,
}

pub(crate) fn parse_job_source_refresh_structured_json(
    source: &JobSource,
    body: &str,
    fetched_url: &str,
    proof_level: &str,
) -> Result<Option<ParsedJobSourceRefresh>> {
    if !job_source_refresh_body_is_json(body) {
        return Ok(None);
    }
    let Ok(value) = serde_json::from_str::<Value>(body) else {
        return Ok(None);
    };
    let source_url_lower = source.url.to_ascii_lowercase();
    let payload = if source_url_lower.contains("lever.co") || job_json_looks_like_lever(&value) {
        parse_lever_job_source_json(source, &value, proof_level)?
    } else if source_url_lower.contains("ashbyhq.com") || value.get("jobs").is_some() {
        parse_ashby_job_source_json(source, &value, proof_level)?
    } else {
        return Ok(None);
    };
    let readable = normalize_readable_text(&job_json_readable_text(&value));
    let mut warnings = payload.warnings;
    let mut companies = Vec::new();
    if let Some(company) = job_source_self_company_card(source, fetched_url, &readable) {
        companies.push(company);
    }
    let no_openings_signal =
        payload.no_openings_signal || job_refresh_has_no_openings_signal(&readable);
    if payload.roles.is_empty() && !no_openings_signal {
        warnings
            .push("Structured ATS payload did not contain any accepted role postings.".to_string());
    }
    let fetched_count = payload.fetched_posting_count + companies.len() + payload.rejected_count;
    Ok(Some(ParsedJobSourceRefresh {
        roles: payload.roles,
        companies,
        fetched_count,
        rejected_count: payload.rejected_count,
        no_openings_signal,
        warnings,
        readable_text: readable,
        direct_role_title: payload.direct_role_title,
    }))
}

pub(crate) fn job_source_refresh_body_is_json(body: &str) -> bool {
    matches!(body.trim_start().chars().next(), Some('{') | Some('['))
}

pub(crate) fn job_source_refresh_live_fetch_url(raw: &str) -> Result<String> {
    let url = validate_fetch_url(raw)?;
    if let Some(api_url) = job_source_refresh_ats_api_url(&url) {
        validate_fetch_url(&api_url)?;
        return Ok(api_url);
    }
    Ok(url.to_string())
}

pub(crate) fn job_source_refresh_ats_api_url(url: &Url) -> Option<String> {
    let host = url.host_str()?.to_ascii_lowercase();
    let segments = url
        .path_segments()
        .map(|segments| {
            segments
                .filter(|segment| !segment.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if host == "jobs.lever.co" || host.ends_with(".jobs.lever.co") {
        let company = segments.first()?;
        if let Some(posting_id) = segments.get(1) {
            return Some(format!(
                "https://api.lever.co/v0/postings/{company}/{posting_id}"
            ));
        }
        return Some(format!(
            "https://api.lever.co/v0/postings/{company}?mode=json"
        ));
    }
    if host == "jobs.ashbyhq.com" || host.ends_with(".jobs.ashbyhq.com") {
        let organization = segments.first()?;
        return Some(format!(
            "https://api.ashbyhq.com/posting-api/job-board/{organization}"
        ));
    }
    None
}

fn parse_lever_job_source_json(
    source: &JobSource,
    value: &Value,
    proof_level: &str,
) -> Result<ParsedStructuredJobPayload> {
    let Some(postings) = lever_job_postings(value) else {
        return Ok(ParsedStructuredJobPayload {
            roles: Vec::new(),
            fetched_posting_count: 0,
            rejected_count: 0,
            warnings: vec!["Lever JSON payload did not contain postings.".to_string()],
            direct_role_title: None,
            no_openings_signal: false,
        });
    };
    let mut roles = Vec::new();
    let mut seen_role_keys = BTreeSet::new();
    let mut rejected_count = 0usize;
    let mut direct_role_title = None;
    for posting in &postings {
        if job_json_bool_at(posting, &["isListed"]) == Some(false)
            || matches!(
                job_json_string_at(posting, &["state"]).as_deref(),
                Some("closed") | Some("archived")
            )
        {
            rejected_count += 1;
            continue;
        }
        let Some(title) = job_json_string_at(posting, &["text"]) else {
            rejected_count += 1;
            continue;
        };
        let role_url = job_json_string_at(posting, &["hostedUrl"])
            .or_else(|| job_json_string_at(posting, &["applyUrl"]))
            .unwrap_or_else(|| source.url.clone());
        let apply_url = job_json_string_at(posting, &["applyUrl"]);
        let mut locations = job_json_string_list_at(posting, &["categories", "allLocations"]);
        if let Some(location) = job_json_string_at(posting, &["categories", "location"]) {
            locations.push(location);
        }
        let location_text = job_join_unique_strings(locations);
        let work_mode =
            job_json_string_at(posting, &["workplaceType"]).map(|value| value.to_ascii_lowercase());
        let description = job_join_unique_optional_strings([
            job_json_string_at(posting, &["descriptionPlain"]),
            job_json_string_at(posting, &["additionalPlain"]),
            job_lever_lists_text(posting),
        ])
        .unwrap_or_default();
        if let Some(mut role) = job_role_input_from_structured_posting(
            source,
            &title,
            &role_url,
            apply_url.as_deref(),
            location_text.as_deref(),
            work_mode.as_deref(),
            &description,
            "lever",
            proof_level,
        )? {
            role.metadata["lever_id"] = posting
                .get("id")
                .cloned()
                .unwrap_or_else(|| Value::String(String::new()));
            if let Some(team) = job_json_string_at(posting, &["categories", "team"]) {
                role.metadata["team"] = json!(team);
            }
            if let Some(commitment) = job_json_string_at(posting, &["categories", "commitment"]) {
                role.metadata["commitment"] = json!(commitment);
            }
            let key = job_role_refresh_key(&role.company, &role.role_title, &role.source_url);
            if seen_role_keys.insert(key) {
                roles.push(role);
            }
        } else {
            rejected_count += 1;
        }
    }
    if postings.len() == 1 {
        direct_role_title = roles.first().map(|role| role.role_title.clone());
    }
    Ok(ParsedStructuredJobPayload {
        roles,
        fetched_posting_count: postings.len(),
        rejected_count,
        warnings: Vec::new(),
        direct_role_title,
        no_openings_signal: postings.is_empty(),
    })
}

fn parse_ashby_job_source_json(
    source: &JobSource,
    value: &Value,
    proof_level: &str,
) -> Result<ParsedStructuredJobPayload> {
    let jobs = value
        .get("jobs")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut roles = Vec::new();
    let mut seen_role_keys = BTreeSet::new();
    let mut rejected_count = 0usize;
    for job in &jobs {
        if job_json_bool_at(job, &["isListed"]) == Some(false) {
            rejected_count += 1;
            continue;
        }
        let Some(title) = job_json_string_at(job, &["title"]) else {
            rejected_count += 1;
            continue;
        };
        let role_url = job_json_string_at(job, &["jobUrl"])
            .or_else(|| job_json_string_at(job, &["applyUrl"]))
            .unwrap_or_else(|| source.url.clone());
        let apply_url = job_json_string_at(job, &["applyUrl"]);
        let mut locations = Vec::new();
        if let Some(location) = ashby_location_name(job.get("location")) {
            locations.push(location);
        }
        if let Some(secondary) = job.get("secondaryLocations").and_then(Value::as_array) {
            for location in secondary {
                if let Some(name) = ashby_location_name(Some(location)) {
                    locations.push(name);
                }
            }
        }
        let location_text = job_join_unique_strings(locations);
        let work_mode = if job_json_bool_at(job, &["isRemote"]) == Some(true) {
            Some("remote".to_string())
        } else {
            job_json_string_at(job, &["workplaceType"]).map(|value| value.to_ascii_lowercase())
        };
        let description = job_json_string_at(job, &["descriptionPlain"])
            .or_else(|| job_json_string_at(job, &["description"]))
            .or_else(|| {
                job_json_string_at(job, &["descriptionHtml"])
                    .map(|html| html_fragment_to_text(&html))
            })
            .unwrap_or_default();
        if let Some(mut role) = job_role_input_from_structured_posting(
            source,
            &title,
            &role_url,
            apply_url.as_deref(),
            location_text.as_deref(),
            work_mode.as_deref(),
            &description,
            "ashby",
            proof_level,
        )? {
            role.metadata["ashby_id"] = job
                .get("id")
                .cloned()
                .unwrap_or_else(|| Value::String(String::new()));
            if let Some(department) = job_json_string_at(job, &["department"]) {
                role.metadata["department"] = json!(department);
            }
            if let Some(team) = job_json_string_at(job, &["team"]) {
                role.metadata["team"] = json!(team);
            }
            let key = job_role_refresh_key(&role.company, &role.role_title, &role.source_url);
            if seen_role_keys.insert(key) {
                roles.push(role);
            }
        } else {
            rejected_count += 1;
        }
    }
    let direct_role_title = (jobs.len() == 1)
        .then(|| roles.first().map(|role| role.role_title.clone()))
        .flatten();
    Ok(ParsedStructuredJobPayload {
        roles,
        fetched_posting_count: jobs.len(),
        rejected_count,
        warnings: Vec::new(),
        direct_role_title,
        no_openings_signal: jobs.is_empty(),
    })
}

fn lever_job_postings(value: &Value) -> Option<Vec<&Value>> {
    if let Some(items) = value.as_array() {
        return Some(items.iter().collect());
    }
    if value.get("id").is_some() && value.get("text").is_some() {
        return Some(vec![value]);
    }
    value
        .get("postings")
        .and_then(Value::as_array)
        .map(|items| items.iter().collect())
}

fn job_json_looks_like_lever(value: &Value) -> bool {
    value.as_array().is_some_and(|items| {
        items
            .iter()
            .any(|item| item.get("hostedUrl").is_some() && item.get("text").is_some())
    }) || (value.get("hostedUrl").is_some() && value.get("text").is_some())
}

fn job_role_input_from_structured_posting(
    source: &JobSource,
    title: &str,
    role_url: &str,
    apply_url: Option<&str>,
    location: Option<&str>,
    work_mode: Option<&str>,
    description: &str,
    ats: &str,
    proof_level: &str,
) -> Result<Option<JobRoleCardInput>> {
    let readable_description = normalize_readable_text(description);
    let excerpt_text = job_join_unique_optional_strings([
        Some(title.to_string()),
        location.map(ToOwned::to_owned),
        work_mode.map(ToOwned::to_owned),
        (!readable_description.is_empty()).then(|| excerpt(&readable_description, 2000)),
    ]);
    let Some(mut role) = job_role_input_from_observed_title(
        source,
        title,
        role_url,
        excerpt_text.as_deref(),
        proof_level,
    )?
    else {
        return Ok(None);
    };
    role.canonical_url = Some(role_url.to_string());
    role.source_url = role_url.to_string();
    role.location = location
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| excerpt(value, JOB_MAX_SHORT_TEXT))
        .or_else(|| excerpt_text.as_deref().and_then(job_infer_location));
    role.work_mode = work_mode
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| excerpt(&value.to_ascii_lowercase(), JOB_MAX_SHORT_TEXT))
        .or_else(|| excerpt_text.as_deref().and_then(job_infer_work_mode));
    role.core_requirements =
        job_requirements_from_role_description(&role.role_title, &readable_description);
    role.implied_business_problem =
        job_implied_business_problem_from_description(&role.role_title, &readable_description);
    role.why_they_might_need_user =
        job_why_they_might_need_user_from_description(&role.role_title, &readable_description);
    role.metadata = json!({
        "adapter": "job_source_refresh",
        "ats": ats,
        "proof_level": proof_level,
        "source_id": source.id,
        "source_name": source.name,
        "source_market_scope": source.market_scope,
        "apply_url": apply_url,
        "observed_excerpt": excerpt(excerpt_text.as_deref().unwrap_or(title), 500),
        "description_excerpt": excerpt(&readable_description, 1500)
    });
    Ok(Some(role))
}

fn job_requirements_from_role_description(title: &str, description: &str) -> Vec<String> {
    let mut requirements = job_requirements_from_role_title(title)
        .into_iter()
        .filter(|requirement| requirement != "role requirements require manual review")
        .collect::<Vec<_>>();
    let lower = format!("{} {}", title, description).to_ascii_lowercase();
    for (keyword, requirement) in [
        ("developer relations", "developer relations"),
        ("developer advocacy", "developer advocacy"),
        ("developer advocate", "developer advocacy"),
        ("devrel", "developer relations"),
        ("developer education", "developer education"),
        ("technical writing", "technical writing"),
        ("public speaking", "technical talks and workshops"),
        ("sample code", "sample code and demos"),
        ("demo", "sample code and demos"),
        ("python", "python"),
        ("api", "APIs"),
        ("sdk", "SDKs"),
        ("open-source", "open source ecosystems"),
        ("open source", "open source ecosystems"),
        ("enterprise", "enterprise developer adoption"),
        ("llm", "AI systems"),
        ("model", "AI systems"),
        ("machine learning", "AI systems"),
        ("infrastructure", "infrastructure"),
        ("platform", "platform engineering"),
    ] {
        if lower.contains(keyword) {
            requirements.push(requirement.to_string());
        }
    }
    job_dedupe_strings(&mut requirements);
    if requirements.is_empty() {
        requirements.push("role requirements require manual review".to_string());
    }
    requirements.truncate(8);
    requirements
}

fn job_implied_business_problem_from_description(title: &str, description: &str) -> Option<String> {
    let lower = format!("{} {}", title, description).to_ascii_lowercase();
    if lower.contains("developer advocate")
        || lower.contains("developer advocacy")
        || lower.contains("developer relations")
        || lower.contains("devrel")
    {
        Some("Grow developer adoption through technical education, demos, sample code, community support, and field feedback while translating AI or platform capabilities into practical developer workflows.".to_string())
    } else if lower.contains("open source") || lower.contains("community") {
        Some("Build trust and adoption in a technical ecosystem through open-source collaboration, documentation, examples, and direct developer feedback loops.".to_string())
    } else if lower.contains("platform") || lower.contains("infrastructure") {
        Some("Build and operate reliable platform infrastructure for technical users, internal engineering teams, or developer-facing product surfaces.".to_string())
    } else if lower.contains("ai") || lower.contains("llm") || lower.contains("model") {
        Some("Turn advanced AI capabilities into dependable products, tools, workflows, or research systems that technical users can trust.".to_string())
    } else {
        job_implied_business_problem(title)
    }
}

fn job_why_they_might_need_user_from_description(title: &str, description: &str) -> Option<String> {
    let lower = format!("{} {}", title, description).to_ascii_lowercase();
    if lower.contains("developer advocate")
        || lower.contains("developer advocacy")
        || lower.contains("developer relations")
        || lower.contains("devrel")
        || lower.contains("developer education")
    {
        Some("The posting emphasizes developer trust, technical communication, demos, ecosystem feedback, and adoption strategy.".to_string())
    } else if lower.contains("platform") || lower.contains("infrastructure") {
        Some("The posting emphasizes reliable technical systems and product judgment around engineering workflows.".to_string())
    } else {
        None
    }
}

fn job_lever_lists_text(posting: &Value) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(lists) = posting.get("lists").and_then(Value::as_array) {
        for list in lists {
            if let Some(text) = job_json_string_at(list, &["text"]) {
                parts.push(text);
            }
            if let Some(content) = job_json_string_at(list, &["content"]) {
                parts.push(html_fragment_to_text(&content));
            }
        }
    }
    job_join_unique_strings(parts)
}

fn ashby_location_name(value: Option<&Value>) -> Option<String> {
    let value = value?;
    if let Some(name) = value.as_str() {
        return Some(name.to_string());
    }
    job_json_string_at(value, &["name"])
        .or_else(|| job_json_string_at(value, &["city"]))
        .or_else(|| job_json_string_at(value, &["region"]))
        .or_else(|| job_json_string_at(value, &["country"]))
}

fn job_json_string_at(value: &Value, path: &[&str]) -> Option<String> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current
        .as_str()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn job_json_bool_at(value: &Value, path: &[&str]) -> Option<bool> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_bool()
}

fn job_json_string_list_at(value: &Value, path: &[&str]) -> Vec<String> {
    let mut current = value;
    for key in path {
        let Some(next) = current.get(*key) else {
            return Vec::new();
        };
        current = next;
    }
    current
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn job_json_readable_text(value: &Value) -> String {
    let mut parts = Vec::new();
    job_json_collect_strings(value, &mut parts, 300);
    job_join_unique_strings(parts).unwrap_or_default()
}

fn job_json_collect_strings(value: &Value, out: &mut Vec<String>, max_parts: usize) {
    if out.len() >= max_parts {
        return;
    }
    match value {
        Value::String(text) => {
            let text = text.trim();
            if !text.is_empty() {
                out.push(html_fragment_to_text(text));
            }
        }
        Value::Array(items) => {
            for item in items {
                job_json_collect_strings(item, out, max_parts);
                if out.len() >= max_parts {
                    break;
                }
            }
        }
        Value::Object(map) => {
            for item in map.values() {
                job_json_collect_strings(item, out, max_parts);
                if out.len() >= max_parts {
                    break;
                }
            }
        }
        _ => {}
    }
}

fn job_join_unique_strings<I>(parts: I) -> Option<String>
where
    I: IntoIterator<Item = String>,
{
    let mut values = parts
        .into_iter()
        .map(|value| normalize_readable_text(&value))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    job_dedupe_strings(&mut values);
    if values.is_empty() {
        None
    } else {
        Some(values.join("; "))
    }
}

fn job_join_unique_optional_strings<I>(parts: I) -> Option<String>
where
    I: IntoIterator<Item = Option<String>>,
{
    let mut values = parts
        .into_iter()
        .flatten()
        .map(|value| normalize_readable_text(&value))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    job_dedupe_strings(&mut values);
    if values.is_empty() {
        None
    } else {
        Some(values.join("; "))
    }
}

fn job_dedupe_strings(values: &mut Vec<String>) {
    let mut seen = BTreeSet::new();
    values.retain(|value| seen.insert(value.to_ascii_lowercase()));
}

pub(crate) fn job_source_refresh_body_is_html(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    lower.contains("<html")
        || lower.contains("<body")
        || lower.contains("<main")
        || lower.contains("<a ")
        || lower.contains("</")
}

pub(crate) fn job_html_anchor_links(html: &str, base_url: &str) -> Result<Vec<JobRefreshAnchor>> {
    let base = validate_fetch_url(base_url)?;
    let mut links = Vec::new();
    let mut remaining = html;
    loop {
        let lower = remaining.to_ascii_lowercase();
        let Some(start) = lower.find("<a") else {
            break;
        };
        let Some(tag_end_offset) = lower[start..].find('>') else {
            break;
        };
        let tag_end = start + tag_end_offset + 1;
        let tag = &remaining[start..tag_end];
        let after = &remaining[tag_end..];
        let Some(close_offset) = after.to_ascii_lowercase().find("</a>") else {
            remaining = after;
            continue;
        };
        let inner = &after[..close_offset];
        remaining = &after[(close_offset + "</a>".len())..];
        let Some(href) = html_attr_value(tag, "href") else {
            continue;
        };
        let Ok(url) = base.join(&href) else {
            continue;
        };
        if validate_fetch_url(url.as_str()).is_err() {
            continue;
        }
        let text = sanitize_work_text(&html_fragment_to_text(inner), JOB_MAX_SHORT_TEXT)?;
        if text.trim().is_empty() {
            continue;
        }
        links.push(JobRefreshAnchor {
            text,
            url: canonical_source_url(url.as_str())?,
        });
    }
    Ok(links)
}

pub(crate) fn job_role_input_from_refresh_anchor(
    source: &JobSource,
    anchor: &JobRefreshAnchor,
    proof_level: &str,
) -> Result<Option<JobRoleCardInput>> {
    let Some(title) = job_role_title_from_anchor(anchor) else {
        return Ok(None);
    };
    job_role_input_from_observed_title(source, &title, &anchor.url, Some(&anchor.text), proof_level)
}

pub(crate) fn job_role_input_from_plaintext_line(
    source: &JobSource,
    line: &str,
    fetched_url: &str,
    proof_level: &str,
) -> Result<Option<JobRoleCardInput>> {
    if !job_source_family_is_canonical_role_source(source) {
        return Ok(None);
    }
    if job_refresh_generic_anchor_text(line) || !job_refresh_text_is_role_like(line) {
        return Ok(None);
    }
    job_role_input_from_observed_title(source, line, fetched_url, Some(line), proof_level)
}

pub(crate) fn job_role_input_from_observed_title(
    source: &JobSource,
    title: &str,
    role_url: &str,
    excerpt_text: Option<&str>,
    proof_level: &str,
) -> Result<Option<JobRoleCardInput>> {
    let title = job_clean_role_title(title);
    if title.is_empty() || !job_refresh_text_is_role_like(&title) {
        return Ok(None);
    }
    let source_confidence = job_source_confidence_for_refresh(source);
    let canonical_url = (source_confidence == "canonical_confirmed"
        || source_confidence == "secondary_confirmed")
        .then(|| role_url.to_string());
    let excerpt_text = excerpt_text.unwrap_or(&title);
    Ok(Some(JobRoleCardInput {
        company: job_company_name_from_source(source),
        role_title: title.clone(),
        canonical_url,
        source_family: source.source_family.clone(),
        source_url: role_url.to_string(),
        source_confidence,
        date_accessed: Some(now()),
        posting_freshness: if proof_level == "live_fetch" {
            "same_day".to_string()
        } else {
            "captured_snapshot".to_string()
        },
        location: job_infer_location(excerpt_text),
        work_mode: job_infer_work_mode(excerpt_text),
        company_stage_or_size: None,
        role_seniority: job_infer_seniority(&title),
        core_requirements: job_requirements_from_role_title(&title),
        implied_business_problem: job_implied_business_problem(&title),
        why_they_might_need_user: None,
        evidence_card_ids: Vec::new(),
        gaps_or_blockers: if proof_level == "manual_snapshot" {
            vec!["Role was observed from a caller-supplied page capture; live confirmation is still required before claiming current coverage.".to_string()]
        } else {
            Vec::new()
        },
        cluster: job_role_cluster(&title),
        current_status: "live".to_string(),
        metadata: json!({
            "adapter": "job_source_refresh",
            "proof_level": proof_level,
            "source_id": source.id,
            "source_name": source.name,
            "source_market_scope": source.market_scope,
            "observed_excerpt": excerpt(excerpt_text, 500)
        }),
    }))
}

pub(crate) fn job_company_input_from_refresh_anchor(
    source: &JobSource,
    anchor: &JobRefreshAnchor,
    readable: &str,
) -> Result<Option<JobCompanyCardInput>> {
    if !matches!(
        source.source_family.as_str(),
        "vc_board" | "founder_post" | "funding_signal" | "london_startup"
    ) {
        return Ok(None);
    }
    if job_refresh_generic_anchor_text(&anchor.text)
        || job_refresh_text_is_role_like(&anchor.text)
        || job_refresh_url_looks_like_job(&anchor.url)
    {
        return Ok(None);
    }
    let name = job_clean_company_name(&anchor.text);
    if !job_refresh_text_is_company_like(&name) {
        return Ok(None);
    }
    Ok(Some(JobCompanyCardInput {
        company_name: name,
        website_url: anchor.url.clone(),
        source_family: source.source_family.clone(),
        market: source.market_scope.clone(),
        stage: None,
        funding_signal: Some(format!("Observed from {}", source.name)),
        product_category: job_infer_product_category(readable),
        technical_audience: job_infer_technical_audience(readable),
        developer_facing_score: job_developer_facing_score(readable),
        london_relevance: job_london_relevance(source, readable),
        remote_maturity: job_infer_remote_maturity(readable),
        hiring_page_url: None,
        founder_or_team_signal: (source.source_family == "founder_post")
            .then(|| format!("Observed from founder/team source {}", source.name)),
        metadata: json!({
            "adapter": "job_source_refresh",
            "source_id": source.id,
            "source_name": source.name,
            "observed_anchor_text": anchor.text
        }),
    }))
}

pub(crate) fn job_source_self_company_card(
    source: &JobSource,
    fetched_url: &str,
    readable: &str,
) -> Option<JobCompanyCardInput> {
    if !job_source_family_is_canonical_role_source(source) && source.source_family != "founder_post"
    {
        return None;
    }
    Some(JobCompanyCardInput {
        company_name: job_company_name_from_source(source),
        website_url: job_source_origin_url(fetched_url).unwrap_or_else(|| source.url.clone()),
        source_family: source.source_family.clone(),
        market: source.market_scope.clone(),
        stage: None,
        funding_signal: None,
        product_category: job_infer_product_category(readable),
        technical_audience: job_infer_technical_audience(readable),
        developer_facing_score: job_developer_facing_score(readable),
        london_relevance: job_london_relevance(source, readable),
        remote_maturity: job_infer_remote_maturity(readable),
        hiring_page_url: Some(source.url.clone()),
        founder_or_team_signal: (source.source_family == "founder_post")
            .then(|| format!("Observed from founder/team source {}", source.name)),
        metadata: json!({
            "adapter": "job_source_refresh",
            "source_id": source.id,
            "source_name": source.name,
            "source_refresh_policy": source.refresh_policy
        }),
    })
}

pub(crate) fn job_role_title_from_anchor(anchor: &JobRefreshAnchor) -> Option<String> {
    if job_refresh_url_looks_like_job_category(&anchor.url) {
        return None;
    }
    if !job_refresh_url_looks_like_job(&anchor.url) {
        return None;
    }
    let text = job_clean_role_title(&anchor.text);
    if job_refresh_text_is_role_like(&text) && !job_refresh_generic_anchor_text(&text) {
        return Some(text);
    }
    job_role_title_from_url(&anchor.url)
}

pub(crate) fn job_role_title_from_url(url: &str) -> Option<String> {
    let parsed = Url::parse(url).ok()?;
    let segment = parsed
        .path_segments()?
        .rev()
        .find(|segment| segment.len() >= 4 && !segment.chars().all(|ch| ch.is_ascii_digit()))?;
    let without_id = segment
        .split('?')
        .next()
        .unwrap_or(segment)
        .split('#')
        .next()
        .unwrap_or(segment);
    let title = without_id
        .split(['-', '_', '+'])
        .filter(|part| !part.is_empty() && !part.chars().all(|ch| ch.is_ascii_digit()))
        .map(job_title_word)
        .collect::<Vec<_>>()
        .join(" ");
    if job_refresh_text_is_role_like(&title) {
        Some(title)
    } else {
        None
    }
}

pub(crate) fn job_role_title_from_html_metadata(html: &str) -> Option<String> {
    let mut candidates = Vec::new();
    if let Some(title) = html_title(html) {
        candidates.push(title);
    }
    for tag in html_start_tags(html, "meta") {
        let name = html_attr_value(&tag, "name").unwrap_or_default();
        let property = html_attr_value(&tag, "property").unwrap_or_default();
        if !name.eq_ignore_ascii_case("title")
            && !property.eq_ignore_ascii_case("og:title")
            && !property.eq_ignore_ascii_case("twitter:title")
        {
            continue;
        }
        if let Some(content) = html_attr_value(&tag, "content") {
            candidates.push(content);
        }
    }
    candidates.into_iter().find_map(|candidate| {
        let title = job_clean_role_title(&candidate);
        if job_refresh_text_is_role_like(&title)
            && !job_refresh_generic_anchor_text(&title)
            && !job_refresh_text_looks_like_markup_or_code(&title)
        {
            Some(title)
        } else {
            None
        }
    })
}

pub(crate) fn job_title_word(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    format!(
        "{}{}",
        first.to_uppercase().collect::<String>(),
        chars.as_str().to_ascii_lowercase()
    )
}

pub(crate) fn job_clean_role_title(value: &str) -> String {
    let mut text = value
        .replace("Job Application for", "")
        .replace("Apply for", "")
        .replace("Apply to", "")
        .replace("View role", "")
        .replace("View job", "")
        .replace("Open role", "")
        .trim()
        .trim_matches(['-', '|', ':', '*', '.'])
        .trim()
        .to_string();
    for separator in [" at ", " @ ", " - ", " | "] {
        if let Some((left, right)) = text.split_once(separator) {
            let left_like = job_refresh_text_is_role_like(left);
            let right_like = job_refresh_text_is_role_like(right);
            if left_like && !right_like {
                text = left.trim().to_string();
            } else if right_like && !left_like {
                text = right.trim().to_string();
            }
        }
    }
    excerpt(&text, JOB_MAX_SHORT_TEXT)
}

pub(crate) fn job_clean_company_name(value: &str) -> String {
    excerpt(
        value.trim().trim_matches(['-', '|', ':', '*', '.']).trim(),
        JOB_MAX_SHORT_TEXT,
    )
}

pub(crate) fn job_company_name_from_source(source: &JobSource) -> String {
    let mut name = source.name.clone();
    for suffix in [
        " careers",
        " jobs",
        " hiring",
        " ats",
        " greenhouse",
        " lever",
        " ashby",
        " workday",
    ] {
        if name.to_ascii_lowercase().ends_with(suffix) {
            let new_len = name.len().saturating_sub(suffix.len());
            name.truncate(new_len);
            break;
        }
    }
    let name = name.trim();
    if name.is_empty() {
        "Unknown company".to_string()
    } else {
        name.to_string()
    }
}

pub(crate) fn job_refresh_text_is_role_like(text: &str) -> bool {
    if job_refresh_text_looks_like_markup_or_code(text) {
        return false;
    }
    let lower = text.to_ascii_lowercase();
    [
        "engineer",
        "developer",
        "technical specialist",
        "architect",
        "devrel",
        "developer relations",
        "advocate",
        "platform",
        "product manager",
        "researcher",
        "scientist",
        "security",
        "infrastructure",
        "frontend",
        "backend",
        "full stack",
        "staff",
        "principal",
        "senior",
        "lead ",
        "head of",
    ]
    .iter()
    .any(|keyword| lower.contains(keyword))
}

pub(crate) fn job_refresh_text_looks_like_markup_or_code(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    text.contains('<')
        || text.contains('>')
        || lower.contains("function (")
        || lower.contains("=>")
        || lower.contains("rel=")
        || lower.contains("class=")
        || lower.contains("xmlns=")
        || lower.contains("setattribute")
        || lower.contains("fetch(")
}

pub(crate) fn job_refresh_url_looks_like_job(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    [
        "/job",
        "/jobs",
        "/careers",
        "/positions",
        "/openings",
        "/roles",
        "greenhouse.io",
        "lever.co",
        "ashbyhq.com",
        "workable.com",
        "workdayjobs.com",
    ]
    .iter()
    .any(|keyword| lower.contains(keyword))
}

pub(crate) fn job_refresh_url_looks_like_job_detail(url: &str) -> bool {
    let Ok(parsed) = Url::parse(url) else {
        return false;
    };
    let host = parsed.host_str().unwrap_or_default().to_ascii_lowercase();
    let segments = parsed
        .path_segments()
        .map(|segments| segments.collect::<Vec<_>>())
        .unwrap_or_default();
    let last_segment = segments.last().copied().unwrap_or_default();
    if host.contains("greenhouse.io") && job_path_has_segment_after(&segments, "jobs") {
        return true;
    }
    if host.contains("ashbyhq.com")
        && segments.len() >= 2
        && (last_segment.contains('-') || last_segment.len() >= 16)
    {
        return true;
    }
    if host.contains("lever.co") && job_path_has_segment_after(&segments, "jobs") {
        return true;
    }
    if host.contains("lever.co") && segments.len() >= 2 && last_segment.len() >= 16 {
        return true;
    }
    if host.contains("workable.com") && job_path_has_segment_after(&segments, "jobs") {
        return true;
    }
    if host.contains("welcometothejungle.com") && job_path_has_segment_after(&segments, "jobs") {
        return true;
    }
    false
}

pub(crate) fn job_refresh_url_looks_like_job_category(url: &str) -> bool {
    let Ok(parsed) = Url::parse(url) else {
        return false;
    };
    let segments = parsed
        .path_segments()
        .map(|segments| segments.collect::<Vec<_>>())
        .unwrap_or_default();
    segments
        .windows(2)
        .any(|pair| pair[0].eq_ignore_ascii_case("jobs") && pair[1].eq_ignore_ascii_case("role"))
}

pub(crate) fn job_path_has_segment_after(segments: &[&str], marker: &str) -> bool {
    segments
        .windows(2)
        .any(|pair| pair[0].eq_ignore_ascii_case(marker) && !pair[1].trim().is_empty())
}

pub(crate) fn job_refresh_anchor_looks_like_weak_job_lead(anchor: &JobRefreshAnchor) -> bool {
    job_refresh_url_looks_like_job(&anchor.url)
        || job_refresh_text_is_role_like(&anchor.text)
        || job_refresh_generic_anchor_text(&anchor.text)
}

pub(crate) fn job_refresh_generic_anchor_text(text: &str) -> bool {
    let lower = text.trim().to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "careers"
            | "jobs"
            | "open roles"
            | "view roles"
            | "view jobs"
            | "apply"
            | "apply now"
            | "learn more"
            | "read more"
            | "see more"
            | "team"
            | "about"
            | "contact"
    ) || lower.ends_with(" jobs")
        || lower.contains(" jobs in ")
}

pub(crate) fn job_refresh_text_is_company_like(text: &str) -> bool {
    let text = text.trim();
    if !(2..=120).contains(&text.len()) {
        return false;
    }
    if text.split_whitespace().count() > 8 {
        return false;
    }
    !text.contains('@') && !text.to_ascii_lowercase().contains("privacy")
}

pub(crate) fn job_plaintext_role_lines(body: &str) -> Vec<String> {
    body.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| line.len() <= JOB_MAX_SHORT_TEXT)
        .filter(|line| job_refresh_text_is_role_like(line))
        .map(ToOwned::to_owned)
        .collect()
}

pub(crate) fn job_refresh_has_no_openings_signal(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    [
        "no open roles",
        "no open positions",
        "no current openings",
        "no vacancies",
        "not currently hiring",
        "no roles currently available",
        "position has been filled",
        "role has been closed",
        "job is no longer available",
    ]
    .iter()
    .any(|phrase| lower.contains(phrase))
}

pub(crate) fn job_source_confidence_for_refresh(source: &JobSource) -> String {
    match source.source_family.as_str() {
        "company" | "ats" | "company_ats" | "company_job_page" | "company_careers" => {
            "canonical_confirmed"
        }
        "vc_board" | "founder_post" | "funding_signal" | "london_startup" => "secondary_confirmed",
        "job_board" => "aggregator_only",
        _ => "unknown",
    }
    .to_string()
}

pub(crate) fn job_source_family_is_canonical_role_source(source: &JobSource) -> bool {
    matches!(
        source.source_family.as_str(),
        "company" | "ats" | "company_ats" | "company_job_page" | "company_careers"
    )
}

pub(crate) fn job_source_refresh_directly_confirms_existing_role(
    source: &JobSource,
    fetched_url: &str,
    readable: &str,
    existing: &JobRoleCard,
    direct_role_title: Option<&str>,
) -> bool {
    if !job_source_family_is_canonical_role_source(source) {
        return false;
    }
    if existing.current_status != "live" || job_refresh_has_no_openings_signal(readable) {
        return false;
    }
    let fetched = canonical_source_url(fetched_url).ok();
    let source_url = canonical_source_url(&source.url).ok();
    let existing_source_url = canonical_source_url(&existing.source_url).ok();
    let existing_canonical_url = existing
        .canonical_url
        .as_deref()
        .and_then(|url| canonical_source_url(url).ok());
    let direct_url_match = [
        source_url.as_deref(),
        existing_source_url.as_deref(),
        existing_canonical_url.as_deref(),
    ]
    .into_iter()
    .flatten()
    .any(|url| Some(url) == fetched.as_deref());
    if !direct_url_match {
        return false;
    }
    if job_refresh_url_looks_like_job_detail(fetched_url) {
        return direct_role_title
            .map(|title| job_role_titles_match(&existing.role_title, title))
            .unwrap_or(false);
    }
    let readable_lower = readable.to_ascii_lowercase();
    let title_lower = existing.role_title.to_ascii_lowercase();
    let company_lower = existing.company.to_ascii_lowercase();
    readable_lower.contains(&title_lower)
        || readable_lower.contains(&company_lower)
        || fetched.as_deref() == existing_source_url.as_deref()
        || fetched.as_deref() == existing_canonical_url.as_deref()
}

pub(crate) fn job_role_titles_match(left: &str, right: &str) -> bool {
    let left = job_clean_role_title(left).to_ascii_lowercase();
    let right = job_clean_role_title(right).to_ascii_lowercase();
    !left.is_empty() && left == right
}

pub(crate) fn job_source_refresh_health_status(
    role_count: usize,
    company_count: usize,
    rejected_count: usize,
    no_openings_signal: bool,
    stale_role_count: usize,
) -> String {
    if no_openings_signal || stale_role_count > 0 && role_count == 0 {
        "stale"
    } else if role_count > 0 && rejected_count == 0 && stale_role_count == 0 {
        "healthy"
    } else if role_count > 0 || company_count > 0 || rejected_count > 0 || stale_role_count > 0 {
        "partial"
    } else {
        "unknown"
    }
    .to_string()
}

pub(crate) fn job_source_refresh_health_note(
    proof_level: &str,
    role_count: usize,
    company_count: usize,
    stale_role_count: usize,
    no_openings_signal: bool,
) -> String {
    let mut parts = vec![format!(
        "{proof_level} job source refresh accepted {role_count} roles and {company_count} companies."
    )];
    if stale_role_count > 0 {
        parts.push(format!(
            "{stale_role_count} previously linked live roles were not observed and were marked stale."
        ));
    }
    if no_openings_signal {
        parts.push("Source text indicated no current openings.".to_string());
    }
    parts.join(" ")
}

pub(crate) fn job_source_refresh_error_code(error: &str) -> String {
    let lower = error.to_ascii_lowercase();
    if lower.contains("policy denied") {
        "policy_denied"
    } else if lower.contains("too large") {
        "body_too_large"
    } else if lower.contains("invalid") {
        "invalid_response"
    } else {
        "provider_network_failure"
    }
    .to_string()
}

pub(crate) fn job_role_refresh_key(company: &str, title: &str, source_url: &str) -> String {
    format!(
        "{}\n{}\n{}",
        company.trim().to_ascii_lowercase(),
        title.trim().to_ascii_lowercase(),
        canonical_source_url(source_url).unwrap_or_else(|_| source_url.to_string())
    )
}

pub(crate) fn job_source_origin_url(url: &str) -> Option<String> {
    let parsed = Url::parse(url).ok()?;
    let host = parsed.host_str()?;
    let mut out = format!("{}://{}", parsed.scheme(), host);
    if let Some(port) = parsed.port() {
        out.push_str(&format!(":{port}"));
    }
    Some(out)
}

pub(crate) fn job_infer_location(text: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    if lower.contains("london") {
        Some("London".to_string())
    } else if lower.contains("berlin") {
        Some("Berlin".to_string())
    } else if lower.contains("united kingdom") || lower.contains(" uk") {
        Some("United Kingdom".to_string())
    } else if lower.contains("europe") || lower.contains("emea") {
        Some("Europe".to_string())
    } else if lower.contains("remote") {
        Some("Remote".to_string())
    } else {
        None
    }
}

pub(crate) fn job_infer_work_mode(text: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    if lower.contains("hybrid") {
        Some("hybrid".to_string())
    } else if lower.contains("remote") {
        Some("remote".to_string())
    } else if lower.contains("onsite") || lower.contains("on-site") {
        Some("onsite".to_string())
    } else {
        None
    }
}

pub(crate) fn job_infer_seniority(title: &str) -> Option<String> {
    let lower = title.to_ascii_lowercase();
    if lower.contains("principal") {
        Some("principal".to_string())
    } else if lower.contains("staff") {
        Some("staff".to_string())
    } else if lower.contains("senior") {
        Some("senior".to_string())
    } else if lower.contains("lead") || lower.contains("head of") {
        Some("lead".to_string())
    } else {
        None
    }
}

pub(crate) fn job_requirements_from_role_title(title: &str) -> Vec<String> {
    let lower = title.to_ascii_lowercase();
    let mut requirements = Vec::new();
    for (keyword, requirement) in [
        ("agent", "agent systems"),
        ("ai", "ai systems"),
        ("platform", "platform engineering"),
        ("developer", "developer-facing systems"),
        ("devrel", "developer relations"),
        ("security", "security"),
        ("rust", "rust"),
        ("swift", "swift"),
        ("cloud", "cloud infrastructure"),
        ("data", "data systems"),
    ] {
        if lower.contains(keyword) {
            requirements.push(requirement.to_string());
        }
    }
    if requirements.is_empty() {
        requirements.push("role requirements require manual review".to_string());
    }
    requirements
}

pub(crate) fn job_implied_business_problem(title: &str) -> Option<String> {
    let lower = title.to_ascii_lowercase();
    if lower.contains("developer") || lower.contains("devrel") || lower.contains("advocate") {
        Some("Improve developer adoption, trust, and technical enablement.".to_string())
    } else if lower.contains("platform") || lower.contains("infrastructure") {
        Some("Build reliable internal or external platform systems.".to_string())
    } else if lower.contains("agent") || lower.contains("ai") {
        Some("Build or operate AI systems that need strong tool and product judgment.".to_string())
    } else {
        None
    }
}

pub(crate) fn job_role_cluster(title: &str) -> Option<String> {
    let lower = title.to_ascii_lowercase();
    if lower.contains("agent") || lower.contains("ai") {
        Some("agent-platform".to_string())
    } else if lower.contains("developer") || lower.contains("devrel") || lower.contains("advocate")
    {
        Some("developer-tools".to_string())
    } else if lower.contains("platform") || lower.contains("infrastructure") {
        Some("platform-engineering".to_string())
    } else {
        None
    }
}

pub(crate) fn job_infer_product_category(text: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    if lower.contains("developer") || lower.contains("api") || lower.contains("sdk") {
        Some("developer tools".to_string())
    } else if lower.contains("agent") || lower.contains("ai") || lower.contains("model") {
        Some("ai".to_string())
    } else if lower.contains("security") {
        Some("security".to_string())
    } else if lower.contains("data") {
        Some("data".to_string())
    } else {
        None
    }
}

pub(crate) fn job_infer_technical_audience(text: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    let hits = [
        "developers",
        "engineers",
        "api",
        "sdk",
        "platform",
        "open source",
    ]
    .iter()
    .filter(|keyword| lower.contains(**keyword))
    .copied()
    .collect::<Vec<_>>();
    if hits.is_empty() {
        None
    } else {
        Some(format!("Technical audience signals: {}", hits.join(", ")))
    }
}

pub(crate) fn job_developer_facing_score(text: &str) -> f64 {
    let lower = text.to_ascii_lowercase();
    let hits = [
        "developer",
        "engineer",
        "api",
        "sdk",
        "platform",
        "open source",
        "agent",
        "mcp",
        "infrastructure",
    ]
    .iter()
    .filter(|keyword| lower.contains(**keyword))
    .count();
    (2.0 + hits as f64 * 0.4).clamp(0.0, 5.0)
}

pub(crate) fn job_london_relevance(source: &JobSource, text: &str) -> String {
    let lower = text.to_ascii_lowercase();
    if source.market_scope.contains("london") || lower.contains("london") {
        "London or London-relevant source.".to_string()
    } else if source.market_scope.contains("uk") || lower.contains("united kingdom") {
        "UK-relevant source; London relevance needs confirmation.".to_string()
    } else if source.market_scope.contains("europe") || lower.contains("europe") {
        "Europe-relevant source; London relevance needs confirmation.".to_string()
    } else {
        "London relevance not proven by this source.".to_string()
    }
}

pub(crate) fn job_infer_remote_maturity(text: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    if lower.contains("remote-first") {
        Some("remote-first".to_string())
    } else if lower.contains("remote") {
        Some("remote mentioned".to_string())
    } else if lower.contains("hybrid") {
        Some("hybrid mentioned".to_string())
    } else {
        None
    }
}

pub(crate) fn sanitize_required_job_text(
    input: &str,
    label: &str,
    max_chars: usize,
) -> Result<String> {
    let value = sanitize_work_text(input.trim(), max_chars)?;
    if value.trim().is_empty() {
        bail!("job {label} cannot be empty");
    }
    Ok(value)
}

pub(crate) fn normalize_optional_job_text(
    value: Option<String>,
    label: &str,
    max_chars: usize,
) -> Result<Option<String>> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| sanitize_required_job_text(value, label, max_chars))
        .transpose()
}

pub(crate) fn normalize_optional_job_url(
    value: Option<String>,
    label: &str,
) -> Result<Option<String>> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| canonical_source_url(value).with_context(|| format!("invalid job {label}")))
        .transpose()
}

pub(crate) fn normalize_job_key_list(values: Vec<String>, label: &str) -> Result<Vec<String>> {
    if values.len() > JOB_MAX_LIST_ITEMS {
        bail!("too many job {label} values");
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

pub(crate) fn normalize_job_id_list(values: Vec<String>, label: &str) -> Result<Vec<String>> {
    if values.len() > JOB_MAX_LIST_ITEMS {
        bail!("too many job {label} values");
    }
    let mut out = Vec::new();
    for value in values {
        validate_id(&value).with_context(|| format!("invalid job {label}"))?;
        if !out.contains(&value) {
            out.push(value);
        }
    }
    Ok(out)
}

pub(crate) fn normalize_job_refresh_id_list(
    values: Vec<String>,
    label: &str,
) -> Result<Vec<String>> {
    if values.len() > JOB_MAX_IMPORT_ITEMS {
        bail!("too many job {label} values");
    }
    let mut out = Vec::new();
    for value in values {
        validate_id(&value).with_context(|| format!("invalid job {label}"))?;
        if !out.contains(&value) {
            out.push(value);
        }
    }
    Ok(out)
}

pub(crate) fn normalize_job_string_list(
    values: Vec<String>,
    label: &str,
    max_chars: usize,
) -> Result<Vec<String>> {
    if values.len() > JOB_MAX_LIST_ITEMS {
        bail!("too many job {label} values");
    }
    let mut out = Vec::new();
    for value in values {
        let value = sanitize_required_job_text(&value, label, max_chars)?;
        if !out.contains(&value) {
            out.push(value);
        }
    }
    Ok(out)
}

pub(crate) fn normalize_job_visibility(value: &str) -> Result<String> {
    match value.trim() {
        "public" | "private_safe" | "private_blocked" | "needs_review" => {
            Ok(value.trim().to_string())
        }
        other => bail!("unsupported job evidence visibility: {other}"),
    }
}

pub(crate) fn normalize_job_evidence_confidence(value: &str) -> Result<String> {
    match value.trim() {
        "verified" | "user_claimed" | "inferred" | "stale" => Ok(value.trim().to_string()),
        other => bail!("unsupported job evidence confidence: {other}"),
    }
}

pub(crate) fn normalize_job_claim_proof_level(value: &str) -> Result<String> {
    match value.trim() {
        "public" | "resume" | "private_safe" | "private" | "unverified" => {
            Ok(value.trim().to_string())
        }
        other => bail!("unsupported job claim proof level: {other}"),
    }
}

pub(crate) fn normalize_job_privacy_rule_type(value: &str) -> Result<String> {
    match value.trim() {
        "blocked_term" | "sensitive_claim" | "needs_review" | "public_ok" | "unsupported_claim"
        | "private_path" => Ok(value.trim().to_string()),
        other => bail!("unsupported job privacy rule type: {other}"),
    }
}

pub(crate) fn normalize_job_privacy_severity(value: &str) -> Result<String> {
    match value.trim() {
        "block" | "warn" | "note" => Ok(value.trim().to_string()),
        other => bail!("unsupported job privacy severity: {other}"),
    }
}

pub(crate) fn normalize_job_source_health_status(value: &str) -> Result<String> {
    match value.trim() {
        "healthy" | "stale" | "blocked" | "failed" | "partial" | "unknown" => {
            Ok(value.trim().to_string())
        }
        other => bail!("unsupported job source health status: {other}"),
    }
}

pub(crate) fn normalize_job_source_confidence(value: &str) -> Result<String> {
    match value.trim() {
        "canonical_confirmed" | "secondary_confirmed" | "aggregator_only" | "stale" | "unknown" => {
            Ok(value.trim().to_string())
        }
        other => bail!("unsupported job source confidence: {other}"),
    }
}

pub(crate) fn normalize_job_role_status(value: &str) -> Result<String> {
    match value.trim() {
        "live" | "stale" | "closed" | "unknown" => Ok(value.trim().to_string()),
        other => bail!("unsupported job role status: {other}"),
    }
}

pub(crate) fn normalize_job_relationship_status(value: &str) -> Result<String> {
    match value.trim() {
        "unknown" | "public_only" | "possible_mutual" | "known" | "contacted" => {
            Ok(value.trim().to_string())
        }
        other => bail!("unsupported job relationship status: {other}"),
    }
}

pub(crate) fn normalize_job_contact_relevance(value: &str) -> Result<String> {
    match value.trim() {
        "unknown" | "hiring_manager" | "recruiter" | "founder" | "devrel_lead" | "engineer"
        | "investor" => Ok(value.trim().to_string()),
        other => bail!("unsupported job contact relevance: {other}"),
    }
}

pub(crate) fn validate_job_contact_relevance_evidence(input: &JobContactInput) -> Result<()> {
    if input.relevance == "unknown" {
        return Ok(());
    }
    if input.relevance == "hiring_manager" && input.role_title.is_none() {
        bail!("hiring-manager contact relevance requires a source-backed role title");
    }
    let Some(note) = input.note.as_deref() else {
        bail!("job contact relevance requires a note naming source evidence or user confirmation");
    };
    let lower = note.to_ascii_lowercase();
    let has_public_source_basis = [
        "source evidence",
        "source lists",
        "listed",
        "public careers",
        "careers route",
        "public ats",
        "role page",
        "public team",
        "team/about",
        "about route",
        "user-confirmed",
        "user confirmed",
        "confirmed by user",
    ]
    .iter()
    .any(|needle| lower.contains(needle));
    if !has_public_source_basis {
        bail!("job contact relevance requires a note naming source evidence or user confirmation");
    }
    Ok(())
}

pub(crate) fn normalize_job_intro_path_type(value: &str) -> Result<String> {
    match value.trim() {
        "direct" | "mutual" | "recruiter" | "investor" | "community" | "unknown" => {
            Ok(value.trim().to_string())
        }
        other => bail!("unsupported job intro path type: {other}"),
    }
}

pub(crate) fn normalize_job_intro_confidence(value: &str) -> Result<String> {
    match value.trim() {
        "confirmed" | "plausible" | "weak" => Ok(value.trim().to_string()),
        other => bail!("unsupported job intro confidence: {other}"),
    }
}

pub(crate) fn normalize_job_intro_status(value: &str) -> Result<String> {
    match value.trim() {
        "identify" | "ask" | "sent" | "replied" | "declined" | "stale" => {
            Ok(value.trim().to_string())
        }
        other => bail!("unsupported job intro status: {other}"),
    }
}

pub(crate) fn normalize_job_proof_level(value: &str) -> Result<String> {
    match value.trim() {
        "missing"
        | "scaffold"
        | "local_proof"
        | "production_data_proof"
        | "operational"
        | "manual_production_data_pass" => Ok(value.trim().to_string()),
        other => bail!("unsupported job proof level: {other}"),
    }
}

pub(crate) fn normalize_job_role_event_status(value: &str) -> Result<String> {
    match value.trim() {
        "new" | "unchanged" | "promoted" | "demoted" | "stale" | "closed" | "applied" => {
            Ok(value.trim().to_string())
        }
        other => bail!("unsupported job role event status: {other}"),
    }
}

pub(crate) fn normalize_job_application_status(value: &str) -> Result<String> {
    match value.trim() {
        "planned" | "applied" | "intro_requested" | "replied" | "interview" | "rejected"
        | "offer" | "withdrawn" => Ok(value.trim().to_string()),
        other => bail!("unsupported job application status: {other}"),
    }
}

pub(crate) fn normalize_job_application_packet_status(value: &str) -> Result<String> {
    match value.trim() {
        "draft" | "approved" | "rejected" | "archived" => Ok(value.trim().to_string()),
        other => bail!("unsupported job application packet status: {other}"),
    }
}

pub(crate) fn normalize_job_weekly_report_delivery_status(value: &str) -> Result<String> {
    match value.trim() {
        "blocked" | "prepared" | "sent" | "failed" => Ok(value.trim().to_string()),
        other => bail!("unsupported job weekly report delivery status: {other}"),
    }
}

pub(crate) fn job_application_status_requires_approved_packet(status: &str) -> bool {
    matches!(
        status,
        "applied" | "intro_requested" | "replied" | "interview" | "offer"
    )
}

pub(crate) fn validate_job_fit_dimension(value: f64, label: &str) -> Result<()> {
    if !value.is_finite() || !(0.0..=5.0).contains(&value) {
        bail!("job {label} must be finite and between 0 and 5");
    }
    Ok(())
}

pub(crate) fn job_weighted_score(input: &JobFitScoreInput) -> f64 {
    let weighted = input.role_fit * 1.4
        + input.domain_fit * 1.3
        + input.evidence_fit * 1.5
        + input.geo_work_fit * 1.2
        + input.stage_fit
        + input.practical_odds * 1.2
        + input.interest_energy;
    let max_weighted = 5.0 * (1.4 + 1.3 + 1.5 + 1.2 + 1.0 + 1.2 + 1.0);
    ((weighted / max_weighted) * 1000.0).round() / 10.0
}

pub(crate) fn job_score_tier(
    weighted_score: f64,
    source_confidence: &str,
    blockers: &[String],
) -> String {
    if !blockers.is_empty() {
        return "blocked".to_string();
    }
    match source_confidence {
        "canonical_confirmed" if weighted_score >= 85.0 => "tier_1".to_string(),
        "canonical_confirmed" | "secondary_confirmed" if weighted_score >= 70.0 => {
            "tier_2".to_string()
        }
        "canonical_confirmed" | "secondary_confirmed" if weighted_score >= 55.0 => {
            "tier_3".to_string()
        }
        _ => "pass".to_string(),
    }
}

pub(crate) fn job_tier_sort_rank(tier: &str) -> usize {
    match tier {
        "tier_1" => 0,
        "tier_2" => 1,
        "tier_3" => 2,
        "pass" => 3,
        "blocked" => 4,
        _ => 9,
    }
}

pub(crate) fn job_effective_score_for_role(
    role: &JobRoleCard,
    mut score: JobFitScore,
) -> JobFitScore {
    if role.current_status != "live" && score.tier != "blocked" {
        let blocker = format!("role source status is {}", role.current_status);
        if !score.blockers.iter().any(|existing| existing == &blocker) {
            score.blockers.push(blocker);
        }
        score.tier = "blocked".to_string();
    }
    score
}

pub(crate) fn job_company_target_evidence_tags(cards: &[JobEvidenceCard]) -> BTreeSet<String> {
    let mut tags = BTreeSet::new();
    for card in cards {
        if card.visibility == "private_blocked" || card.confidence == "unverified" {
            continue;
        }
        for tag in &card.tags {
            if !tag.trim().is_empty() {
                tags.insert(tag.trim().to_ascii_lowercase());
            }
        }
    }
    tags
}

pub(crate) fn job_company_target_entry(
    company: &JobCompanyCard,
    evidence_tags: &BTreeSet<String>,
) -> JobCompanyTargetEntry {
    let mut score = (company.developer_facing_score * 12.0).clamp(0.0, 60.0);
    let mut reasons = vec![format!(
        "developer-facing score {:.1}/5.0",
        company.developer_facing_score
    )];
    let mut warnings = vec![
        "No current role is implied by this company target; verify a canonical role source before application work."
            .to_string(),
    ];
    let search_text = job_company_target_search_text(company);
    let london_text = format!(
        "{} {}",
        company.market.to_ascii_lowercase(),
        company.london_relevance.to_ascii_lowercase()
    );
    if london_text.contains("london")
        || london_text.contains("uk")
        || london_text.contains("strong")
        || london_text.contains("high")
    {
        score += 12.0;
        reasons.push("London or UK relevance is explicit.".to_string());
    }
    if let Some(remote_maturity) = &company.remote_maturity {
        let remote_maturity = remote_maturity.to_ascii_lowercase();
        if remote_maturity.contains("remote")
            || remote_maturity.contains("hybrid")
            || remote_maturity.contains("europe")
            || remote_maturity.contains("distributed")
        {
            score += 8.0;
            reasons
                .push("Remote, hybrid, or Europe-friendly working pattern is visible.".to_string());
        }
    }
    if company.hiring_page_url.is_some() {
        score += 8.0;
        reasons.push("Hiring page URL is captured.".to_string());
    } else {
        warnings.push("No hiring page URL is captured; find a canonical careers page before creating role cards.".to_string());
    }
    if company.founder_or_team_signal.is_some() {
        score += 5.0;
        reasons.push("Founder or team signal is captured for outreach research.".to_string());
    }
    if company.stage.is_some() || company.funding_signal.is_some() {
        score += 4.0;
        reasons.push("Stage or funding signal is available.".to_string());
    }

    let mut matched_evidence_tags = Vec::new();
    for tag in evidence_tags {
        if job_company_target_text_matches_tag(&search_text, tag) {
            matched_evidence_tags.push(tag.clone());
        }
    }
    matched_evidence_tags.truncate(5);
    if !matched_evidence_tags.is_empty() {
        let evidence_bonus = (matched_evidence_tags.len() as f64 * 4.0).min(12.0);
        score += evidence_bonus;
        reasons.push(format!(
            "matches public evidence tags: {}",
            matched_evidence_tags.join(", ")
        ));
    }

    if company.developer_facing_score < 2.5 {
        warnings.push("Low developer-facing score; keep as a weak monitoring lead unless new evidence appears.".to_string());
    }
    if !matches!(
        company.source_family.as_str(),
        "company" | "company_careers" | "founder_site" | "official"
    ) {
        warnings.push(format!(
            "Source family '{}' is scouting-level; verify against an official company or careers source.",
            company.source_family
        ));
    }

    score = (score.min(100.0) * 10.0).round() / 10.0;
    let tier = job_company_target_tier(score);
    let next_action = if company.hiring_page_url.is_some() {
        "Review the hiring page for current roles, then create role cards only for confirmed openings."
            .to_string()
    } else if tier == "target_now" || tier == "research_next" {
        "Find an official careers page and founder/team context before outreach.".to_string()
    } else {
        "Monitor passively until stronger company or role evidence appears.".to_string()
    };

    JobCompanyTargetEntry {
        company: company.clone(),
        score,
        tier,
        reasons,
        warnings,
        matched_evidence_tags,
        next_action,
    }
}

pub(crate) fn job_company_target_search_text(company: &JobCompanyCard) -> String {
    let metadata = serde_json::to_string(&company.metadata).unwrap_or_default();
    [
        company.company_name.as_str(),
        company.website_url.as_str(),
        company.source_family.as_str(),
        company.market.as_str(),
        company.stage.as_deref().unwrap_or_default(),
        company.funding_signal.as_deref().unwrap_or_default(),
        company.product_category.as_deref().unwrap_or_default(),
        company.technical_audience.as_deref().unwrap_or_default(),
        company.london_relevance.as_str(),
        company.remote_maturity.as_deref().unwrap_or_default(),
        company.hiring_page_url.as_deref().unwrap_or_default(),
        company
            .founder_or_team_signal
            .as_deref()
            .unwrap_or_default(),
        metadata.as_str(),
    ]
    .join(" ")
    .to_ascii_lowercase()
}

pub(crate) fn job_company_target_text_matches_tag(search_text: &str, tag: &str) -> bool {
    let normalized = tag.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return false;
    }
    search_text.contains(&normalized)
        || search_text.contains(&normalized.replace('-', " "))
        || search_text.contains(&normalized.replace('_', " "))
}

pub(crate) fn job_company_target_tier(score: f64) -> String {
    if score >= 75.0 {
        "target_now".to_string()
    } else if score >= 60.0 {
        "research_next".to_string()
    } else if score >= 45.0 {
        "monitor".to_string()
    } else {
        "hold".to_string()
    }
}

pub(crate) fn job_privacy_decision(findings: &[JobPrivacyFinding]) -> String {
    if findings.iter().any(|finding| finding.severity == "block") {
        "block".to_string()
    } else if findings.iter().any(|finding| finding.severity == "warn") {
        "warn".to_string()
    } else {
        "pass".to_string()
    }
}

pub(crate) fn text_contains_case_insensitive(haystack: &str, needle: &str) -> bool {
    haystack
        .to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

pub(crate) fn job_value_contains_local_reference(value: &Value) -> bool {
    match value {
        Value::String(text) => job_text_looks_local_reference(text),
        Value::Array(items) => items.iter().any(job_value_contains_local_reference),
        Value::Object(map) => map.iter().any(|(key, value)| {
            job_text_looks_local_reference(key) || job_value_contains_local_reference(value)
        }),
        _ => false,
    }
}

pub(crate) fn job_text_looks_local_reference(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.starts_with("file://")
        || trimmed.starts_with("/Users/")
        || trimmed.starts_with("~/")
        || trimmed.starts_with("../")
        || trimmed.starts_with("./")
        || trimmed.contains("local_path")
}

pub(crate) fn job_application_packet_text(
    role: &JobRoleCard,
    input: &JobApplicationPacketInput,
) -> String {
    format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}",
        role.company,
        role.role_title,
        input.resume_emphasis,
        input.tailored_bullets.join("\n"),
        input.outreach_note,
        input.likely_objections.join("\n"),
        input.interview_stories.join("\n")
    )
}

pub(crate) fn render_job_application_packet_export_markdown(
    role: &JobRoleCard,
    packet: &JobApplicationPacket,
    evidence: &[JobEvidenceCard],
) -> String {
    let bullets = render_job_markdown_list(&packet.tailored_bullets);
    let objections = render_job_markdown_list(&packet.likely_objections);
    let stories = render_job_markdown_list(&packet.interview_stories);
    let questions = render_job_markdown_list(&packet.questions_to_ask);
    let proof_links =
        serde_json::to_string_pretty(&packet.proof_links).unwrap_or_else(|_| "{}".to_string());
    let evidence_lines = evidence
        .iter()
        .map(|card| {
            let proof = card
                .proof_url
                .as_deref()
                .map(|url| format!(" Proof: {url}."))
                .unwrap_or_default();
            format!(
                "- {} ({}, {}): {}{}",
                card.title, card.visibility, card.confidence, card.safe_application_text, proof
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let reviewer_note = packet
        .reviewer_note
        .as_deref()
        .unwrap_or("No reviewer note recorded.");
    format!(
        "# Application Packet: {} - {}\n\nPacket id: {}\nRole id: {}\nProfile id: {}\nPacket status: {}\nExport proof level: local_proof\nDelivery status: not_sent\nApplication status changed by export: false\n\n## Role\n\nCompany: {}\nRole: {}\nCanonical URL: {}\nSource confidence: {}\nCurrent role status: {}\n\n## Resume Emphasis\n\n{}\n\n## Tailored Bullets\n\n{}\n\n## Outreach Note\n\n{}\n\n## Proof Links\n\n```json\n{}\n```\n\n## Evidence Used\n\n{}\n\n## Likely Objections\n\n{}\n\n## Interview Stories\n\n{}\n\n## Questions To Ask\n\n{}\n\n## Reviewer Note\n\n{}\n\n## Privacy\n\nPacket privacy check: {}\nExport privacy check: recorded separately during export.\n\n## Boundary\n\nThis file is a local reviewed application packet export. It is not proof that an application was sent, delivered, or recorded as applied.\n",
        role.company,
        role.role_title,
        packet.id,
        packet.role_id,
        packet.profile_id,
        packet.status,
        role.company,
        role.role_title,
        role.canonical_url.as_deref().unwrap_or("not recorded"),
        role.source_confidence,
        role.current_status,
        packet.resume_emphasis,
        bullets,
        packet.outreach_note,
        proof_links,
        if evidence_lines.is_empty() {
            "- No evidence cards recorded.".to_string()
        } else {
            evidence_lines
        },
        objections,
        stories,
        questions,
        reviewer_note,
        packet.privacy_check_id,
    )
}

pub(crate) fn render_job_markdown_list(items: &[String]) -> String {
    if items.is_empty() {
        "- none".to_string()
    } else {
        items
            .iter()
            .map(|item| format!("- {item}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub(crate) fn job_intro_claims_warm_path(input: &JobIntroPathInput) -> bool {
    matches!(
        input.path_type.as_str(),
        "direct" | "mutual" | "community" | "investor"
    ) || input.confidence == "confirmed"
        || matches!(input.status.as_str(), "ask" | "sent" | "replied")
}

pub(crate) fn render_job_weekly_report(
    shortlist: &JobShortlist,
    applications: &[JobApplication],
    source_health: &[JobSourceHealth],
    intro_paths: &[JobIntroPath],
    contacts: &[JobContact],
    role_events: &[JobRoleStatusEvent],
    event_since: Option<&str>,
) -> String {
    let mut role_labels: BTreeMap<String, String> = BTreeMap::new();
    let mut role_entries: BTreeMap<String, &JobShortlistEntry> = BTreeMap::new();
    for entry in &shortlist.entries {
        role_labels.insert(
            entry.role.id.clone(),
            format!(
                "{} - {}",
                entry.role.company,
                job_report_display_role_title(&entry.role)
            ),
        );
        role_entries.insert(entry.role.id.clone(), entry);
    }
    let mut status_counts: BTreeMap<String, usize> = BTreeMap::new();
    for application in applications {
        *status_counts.entry(application.status.clone()).or_insert(0) += 1;
    }
    let mut health_counts: BTreeMap<String, usize> = BTreeMap::new();
    for health in source_health {
        *health_counts.entry(health.status.clone()).or_insert(0) += 1;
    }
    let contact_names = contacts
        .iter()
        .map(|contact| (contact.id.clone(), contact.name.clone()))
        .collect::<BTreeMap<_, _>>();
    let latest_role_statuses = job_report_latest_role_statuses(role_events);
    let grouped_open_roles = job_report_open_role_groups(&shortlist.entries, &latest_role_statuses);
    let scored_open_role_groups = grouped_open_roles
        .iter()
        .filter(|group| group.primary.score.is_some())
        .collect::<Vec<_>>();
    let mut tier_counts: BTreeMap<String, usize> = BTreeMap::new();
    for group in &scored_open_role_groups {
        let tier = group.primary.score.as_ref().unwrap().tier.clone();
        *tier_counts.entry(tier).or_insert(0) += 1;
    }
    let top_roles = scored_open_role_groups
        .iter()
        .take(10)
        .map(|group| {
            let entry = group.primary;
            let score = entry
                .score
                .as_ref()
                .map(|score| format!("{} ({:.1})", score.tier, score.weighted_score))
                .unwrap();
            let outcome_warnings = if entry.outcome_warnings.is_empty() {
                String::new()
            } else {
                format!("\n  Outcome notes: {}", entry.outcome_warnings.join(" "))
            };
            format!(
                "- {} - {}: {}",
                entry.role.company,
                job_report_display_role_title(&entry.role),
                score
            ) + &outcome_warnings
        })
        .collect::<Vec<_>>()
        .join("\n");
    let intro_status = render_job_intro_status(intro_paths);
    let next_actions =
        render_job_weekly_next_actions(intro_paths, &contact_names, &role_labels, applications);
    let role_changes = render_job_weekly_role_changes(role_events, &role_labels);
    let new_openings = render_job_opening_events(
        role_events,
        &role_entries,
        &role_labels,
        &["new"],
        event_since,
    );
    let open_roles = render_job_current_open_roles_from_groups(&grouped_open_roles);
    let removed_roles = render_job_opening_events(
        role_events,
        &role_entries,
        &role_labels,
        &["closed", "stale"],
        event_since,
    );
    format!(
        "# Job Weekly Report\n\nProfile: {}\nGenerated: {}\n\n## New openings found\n\n{}\n\n## Currently open roles\n\n{}\n\n## Roles removed\n\n{}\n\n## Shortlist\n\n{}\n\n## Tier Counts\n\n{}\n\n## Role Changes\n\n{}\n\n## Applications\n\n{}\n\n## Intro Status\n\n{}\n\n## Next Actions\n\n{}\n\n## Source Health\n\n{}\n",
        shortlist.profile_id,
        shortlist.generated_at,
        new_openings,
        open_roles,
        removed_roles,
        if top_roles.is_empty() {
            "- No scored roles recorded.".to_string()
        } else {
            top_roles
        },
        render_job_count_map(&tier_counts),
        role_changes,
        render_job_count_map(&status_counts),
        intro_status,
        next_actions,
        render_job_count_map(&health_counts),
    )
}

pub(crate) fn render_job_weekly_report_delivery_body(body: &str) -> String {
    let delivery_source = body.split("\n\n## Shortlist\n").next().unwrap_or(body);
    let mut sections = Vec::<(String, Vec<String>)>::new();
    let mut current_title: Option<String> = None;
    let mut current_lines = Vec::<String>::new();
    for line in delivery_source.lines() {
        if line.starts_with("# ") {
            continue;
        }
        if let Some(title) = line.strip_prefix("## ") {
            if let Some(previous_title) = current_title.take() {
                sections.push((previous_title, std::mem::take(&mut current_lines)));
            }
            current_title = Some(title.trim().to_string());
            continue;
        }
        if current_title.is_some() {
            current_lines.push(line.to_string());
        }
    }
    if let Some(previous_title) = current_title {
        sections.push((previous_title, current_lines));
    }

    let mut delivery_body = String::from("# Job Scan");
    for (title, lines) in sections {
        let content = lines.join("\n").trim().to_string();
        if matches!(title.as_str(), "New openings found" | "Roles removed")
            && job_report_delivery_section_is_empty(&content)
        {
            continue;
        }
        delivery_body.push_str("\n\n## ");
        delivery_body.push_str(&title);
        delivery_body.push_str("\n\n");
        if content.is_empty() {
            delivery_body.push_str("- none");
        } else {
            delivery_body.push_str(&content);
        }
    }
    delivery_body
}

fn job_report_delivery_section_is_empty(content: &str) -> bool {
    matches!(content.trim(), "" | "- none" | "none")
}

struct JobReportRoleGroup<'a> {
    primary: &'a JobShortlistEntry,
    role_ids: BTreeSet<String>,
    location_labels: BTreeSet<String>,
}

struct JobReportRoleEventGroup<'a> {
    primary: &'a JobShortlistEntry,
    role_ids: BTreeSet<String>,
    statuses: BTreeSet<String>,
    location_labels: BTreeSet<String>,
}

const JOB_REPORT_EMAIL_SCORE_FLOOR_PERCENT: f64 = 50.0;

fn job_report_open_role_groups<'a>(
    entries: &'a [JobShortlistEntry],
    latest_role_statuses: &BTreeMap<String, String>,
) -> Vec<JobReportRoleGroup<'a>> {
    collect_job_report_role_groups(entries.iter().filter(|entry| {
        entry.role.current_status == "live"
            && !latest_role_statuses
                .get(&entry.role.id)
                .map(|status| matches!(status.as_str(), "closed" | "stale"))
                .unwrap_or(false)
            && entry
                .score
                .as_ref()
                .map(|score| !matches!(score.tier.as_str(), "pass" | "blocked"))
                .unwrap_or(true)
            && job_report_role_is_uk_plausible(&entry.role)
            && job_report_entry_meets_email_score_floor(entry)
    }))
}

fn job_report_latest_role_statuses(role_events: &[JobRoleStatusEvent]) -> BTreeMap<String, String> {
    let mut latest = BTreeMap::<String, (String, String, String)>::new();
    for event in role_events {
        let key = (
            event.created_at.clone(),
            event.id.clone(),
            event.status.clone(),
        );
        if latest
            .get(&event.role_id)
            .map(|previous| key > *previous)
            .unwrap_or(true)
        {
            latest.insert(event.role_id.clone(), key);
        }
    }
    latest
        .into_iter()
        .map(|(role_id, (_, _, status))| (role_id, status))
        .collect()
}

fn render_job_current_open_roles_from_groups(groups: &[JobReportRoleGroup<'_>]) -> String {
    let mut groups = groups.iter().collect::<Vec<_>>();
    groups.sort_by(|left, right| job_report_group_order(left.primary, right.primary));
    let roles = groups
        .iter()
        .map(|group| render_job_role_group_digest_entry(group))
        .collect::<Vec<_>>();
    if roles.is_empty() {
        "- none".to_string()
    } else {
        roles.join("\n\n")
    }
}

pub(crate) fn render_job_opening_events(
    role_events: &[JobRoleStatusEvent],
    role_entries: &BTreeMap<String, &JobShortlistEntry>,
    role_labels: &BTreeMap<String, String>,
    statuses: &[&str],
    event_since: Option<&str>,
) -> String {
    let mut seen_role_status = BTreeSet::new();
    let mut groups = Vec::<JobReportRoleEventGroup<'_>>::new();
    let mut group_indexes = BTreeMap::<String, usize>::new();
    let mut fallback_lines = Vec::new();
    for event in role_events {
        if !statuses.iter().any(|status| *status == event.status) {
            continue;
        }
        if event_since.is_some_and(|since| event.created_at.as_str() <= since) {
            continue;
        }
        if !seen_role_status.insert((event.role_id.clone(), event.status.clone())) {
            continue;
        }
        if let Some(entry) = role_entries.get(&event.role_id) {
            if !job_report_role_is_uk_plausible(&entry.role) {
                continue;
            }
            if !job_report_entry_meets_email_score_floor(entry) {
                continue;
            }
            let key = job_report_role_family_key(&entry.role);
            if let Some(index) = group_indexes.get(&key).copied() {
                let group = &mut groups[index];
                group.role_ids.insert(entry.role.id.clone());
                group.statuses.insert(event.status.clone());
                if let Some(label) = job_report_location_label(&entry.role) {
                    group.location_labels.insert(label);
                }
                if job_report_entry_score(entry) > job_report_entry_score(group.primary) {
                    group.primary = entry;
                }
            } else {
                let mut role_ids = BTreeSet::new();
                role_ids.insert(entry.role.id.clone());
                let mut status_set = BTreeSet::new();
                status_set.insert(event.status.clone());
                let mut location_labels = BTreeSet::new();
                if let Some(label) = job_report_location_label(&entry.role) {
                    location_labels.insert(label);
                }
                group_indexes.insert(key, groups.len());
                groups.push(JobReportRoleEventGroup {
                    primary: entry,
                    role_ids,
                    statuses: status_set,
                    location_labels,
                });
            }
        } else {
            let label = role_labels
                .get(&event.role_id)
                .cloned()
                .unwrap_or_else(|| event.role_id.clone());
            fallback_lines.push(format!("- {label} ({})", event.status));
        }
    }
    let mut lines = groups.iter().collect::<Vec<_>>();
    lines.sort_by(|left, right| job_report_group_order(left.primary, right.primary));
    let mut lines = lines
        .iter()
        .map(|group| render_job_role_event_group(group))
        .collect::<Vec<_>>();
    lines.extend(fallback_lines);
    if lines.is_empty() {
        "- none".to_string()
    } else {
        lines.join("\n\n")
    }
}

fn render_job_role_group_digest_entry(group: &JobReportRoleGroup<'_>) -> String {
    render_job_role_digest_entry_with_locations(
        group.primary,
        Some(&group.location_labels),
        group.role_ids.len(),
    )
}

fn render_job_role_event_group(group: &JobReportRoleEventGroup<'_>) -> String {
    if group.statuses.len() == 1 && group.statuses.contains("new") {
        return render_job_role_digest_entry_with_locations(
            group.primary,
            Some(&group.location_labels),
            group.role_ids.len(),
        );
    }
    let statuses = group
        .statuses
        .iter()
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    let locations = if group.location_labels.is_empty() {
        String::new()
    } else {
        format!(
            "; locations: {}",
            group
                .location_labels
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    let grouped = if group.role_ids.len() > 1 {
        format!("; grouped {} postings", group.role_ids.len())
    } else {
        String::new()
    };
    format!(
        "- {} at {} ({}{locations}{grouped})",
        job_report_display_role_title(&group.primary.role),
        group.primary.role.company,
        statuses
    )
}

fn render_job_role_digest_entry_with_locations(
    entry: &JobShortlistEntry,
    location_labels: Option<&BTreeSet<String>>,
    grouped_count: usize,
) -> String {
    let role = &entry.role;
    let location = location_labels
        .filter(|labels| !labels.is_empty())
        .map(|labels| labels.iter().cloned().collect::<Vec<_>>().join(", "))
        .or_else(|| job_report_location_label(role));
    let title = job_report_linked_role_title(role);
    let title_line = match location {
        Some(location) if !location.is_empty() => format!("{title}  {location}"),
        _ => title,
    };
    let summary = job_report_role_summary_paragraph(entry);
    let mut status = Vec::new();
    if let Some(score) = &entry.score {
        status.push(format!("Score: {:.0}%.", score.weighted_score));
    } else {
        status.push(format!(
            "Score: {:.0}% estimated.",
            job_report_role_heuristic_score(role)
        ));
    }
    if grouped_count > 1 {
        status.push(format!("Grouped {grouped_count} location postings."));
    }
    format!("{title_line}\n\n{summary}\n\n{}", status.join(" "))
}

fn job_report_linked_role_title(role: &JobRoleCard) -> String {
    let label = format!(
        "{} at {}",
        job_report_display_role_title(role),
        role.company
    );
    if let Some(url) = job_report_role_url(role) {
        format!("**[{label}]({url})**")
    } else {
        format!("**{label}**")
    }
}

fn job_report_role_url(role: &JobRoleCard) -> Option<&str> {
    role.canonical_url
        .as_deref()
        .or(Some(role.source_url.as_str()))
        .filter(|url| {
            Url::parse(url)
                .map(|parsed| matches!(parsed.scheme(), "http" | "https"))
                .unwrap_or(false)
        })
}

fn job_report_role_summary_paragraph(entry: &JobShortlistEntry) -> String {
    let role = &entry.role;
    let category = job_report_role_category(role);
    let article = job_report_role_category_article(category);
    let mut sentences = Vec::new();
    sentences.push(format!(
        "This is {article} {category} role at {}.",
        role.company
    ));
    if let Some(signal) = job_report_role_signal(role) {
        sentences.push(format!("Stands out: {signal}."));
    }
    sentences.join(" ")
}

fn job_report_role_signal(role: &JobRoleCard) -> Option<String> {
    role.implied_business_problem
        .as_deref()
        .filter(|value| job_report_role_text_is_useful_signal(value))
        .map(job_report_clean_sentence)
        .or_else(|| {
            role.why_they_might_need_user
                .as_deref()
                .filter(|value| job_report_role_text_is_useful_signal(value))
                .map(job_report_clean_sentence)
        })
        .or_else(|| {
            let requirements = role
                .core_requirements
                .iter()
                .filter(|value| job_report_role_text_is_useful_signal(value))
                .take(3)
                .cloned()
                .collect::<Vec<_>>();
            if requirements.is_empty() {
                None
            } else {
                Some(format!(
                    "the stored requirements emphasize {}",
                    requirements.join(", ")
                ))
            }
        })
}

fn job_report_role_text_is_useful_signal(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && !trimmed.eq_ignore_ascii_case("role requirements require manual review")
        && !trimmed.eq_ignore_ascii_case("manual review")
}

fn job_report_clean_sentence(value: &str) -> String {
    value.trim().trim_end_matches('.').to_string()
}

fn job_report_role_category(role: &JobRoleCard) -> &'static str {
    let normalized = job_report_canonical_key_text(&role.role_title);
    if normalized.contains("developer relations")
        || normalized.contains("developer advocate")
        || normalized.contains("devrel")
        || normalized.contains("developer experience")
    {
        "developer-facing engineering"
    } else if normalized.contains("research engineer")
        || normalized.contains("machine learning")
        || normalized.contains(" ai ")
        || normalized.contains("pretraining")
        || normalized.contains("post training")
    {
        "AI research engineering"
    } else if normalized.contains("security") {
        "security engineering"
    } else if normalized.contains("infrastructure")
        || normalized.contains("platform")
        || normalized.contains("backend")
        || normalized.contains("frontend")
        || normalized.contains("networking")
    {
        "engineering"
    } else if normalized.contains("solutions engineer")
        || normalized.contains("customer engineering")
    {
        "technical customer engineering"
    } else if normalized.contains("product engineer") {
        "product engineering"
    } else {
        "open"
    }
}

fn job_report_role_category_article(category: &str) -> &'static str {
    match category.chars().next().map(|ch| ch.to_ascii_lowercase()) {
        Some('a' | 'e' | 'i' | 'o' | 'u') => "an",
        _ => "a",
    }
}

fn collect_job_report_role_groups<'a, I>(entries: I) -> Vec<JobReportRoleGroup<'a>>
where
    I: IntoIterator<Item = &'a JobShortlistEntry>,
{
    let mut groups = Vec::<JobReportRoleGroup<'a>>::new();
    let mut indexes = BTreeMap::<String, usize>::new();
    for entry in entries {
        let key = job_report_role_family_key(&entry.role);
        if let Some(index) = indexes.get(&key).copied() {
            let group = &mut groups[index];
            group.role_ids.insert(entry.role.id.clone());
            if let Some(label) = job_report_location_label(&entry.role) {
                group.location_labels.insert(label);
            }
            if job_report_entry_score(entry) > job_report_entry_score(group.primary) {
                group.primary = entry;
            }
            continue;
        }
        let mut role_ids = BTreeSet::new();
        role_ids.insert(entry.role.id.clone());
        let mut location_labels = BTreeSet::new();
        if let Some(label) = job_report_location_label(&entry.role) {
            location_labels.insert(label);
        }
        indexes.insert(key, groups.len());
        groups.push(JobReportRoleGroup {
            primary: entry,
            role_ids,
            location_labels,
        });
    }
    groups
}

fn job_report_entry_score(entry: &JobShortlistEntry) -> f64 {
    entry
        .score
        .as_ref()
        .map(|score| score.weighted_score)
        .unwrap_or_else(|| job_report_role_heuristic_score(&entry.role))
}

fn job_report_entry_meets_email_score_floor(entry: &JobShortlistEntry) -> bool {
    job_report_entry_score(entry) >= JOB_REPORT_EMAIL_SCORE_FLOOR_PERCENT
}

fn job_report_group_order(
    left: &JobShortlistEntry,
    right: &JobShortlistEntry,
) -> std::cmp::Ordering {
    job_report_entry_score(right)
        .partial_cmp(&job_report_entry_score(left))
        .unwrap_or(std::cmp::Ordering::Equal)
        .then_with(|| left.role.company.cmp(&right.role.company))
        .then_with(|| {
            job_report_display_role_title(&left.role)
                .cmp(&job_report_display_role_title(&right.role))
        })
}

pub(crate) fn job_report_role_heuristic_score(role: &JobRoleCard) -> f64 {
    let text = job_report_canonical_key_text(
        &[
            role.role_title.as_str(),
            role.cluster.as_deref().unwrap_or_default(),
            role.implied_business_problem.as_deref().unwrap_or_default(),
            role.why_they_might_need_user.as_deref().unwrap_or_default(),
            &role.core_requirements.join(" "),
        ]
        .join(" "),
    );
    let mut score: f64 =
        if text.contains("developer relations") || text.contains("developer advocate") {
            88.0
        } else if text.contains("devrel")
            || text.contains("developer advocacy")
            || text.contains("developer education")
            || text.contains("developer marketer")
            || text.contains("developer marketing")
            || text.contains("developer who")
            || text.contains("technical evangelist")
        {
            84.0
        } else if text.contains("solutions architect")
            || text.contains("solutions engineer")
            || text.contains("forward deployed")
            || text.contains("developer experience")
            || text.contains("devex")
        {
            79.0
        } else if text.contains("research engineer")
            || text.contains("machine learning")
            || text.contains(" ai ")
            || text.contains(" llm ")
            || text.contains(" model ")
        {
            77.0
        } else if text.contains("platform")
            || text.contains("infrastructure")
            || text.contains("distributed systems")
            || text.contains("backend")
            || text.contains("sdk")
            || text.contains("api")
            || text.contains("open source")
        {
            73.0
        } else if text.contains("security") {
            66.0
        } else if text.contains("product manager") {
            58.0
        } else {
            50.0
        };
    if text.contains("principal")
        || text.contains("staff")
        || text.contains("senior")
        || text.contains("lead")
    {
        score += 3.0;
    }
    if text.contains("intern") || text.contains("graduate") {
        score -= 35.0;
    }
    if text.contains("manager") && !text.contains("developer relations") {
        score -= 5.0;
    }
    let geo_text = job_report_normalized_geo_text(
        &[role.location.as_deref(), role.work_mode.as_deref()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" "),
    );
    if job_report_geo_text_has_positive_uk_relevance(&geo_text)
        || job_report_geo_text_mentions_remote(&geo_text)
    {
        score += 3.0;
    }
    if role.source_confidence == "canonical_confirmed" {
        score += 2.0;
    }
    score.clamp(35.0, 92.0)
}

fn job_report_location_label(role: &JobRoleCard) -> Option<String> {
    let mut locations = job_report_positive_geo_labels(&role.role_title);
    if let Some(location) = role.location.as_deref().filter(|value| !value.is_empty()) {
        let location_labels = job_report_positive_geo_labels(location);
        if location_labels.is_empty() && !location.eq_ignore_ascii_case("remote") {
            locations.push(location.to_string());
        } else {
            locations.extend(location_labels);
        }
    }
    locations.sort();
    locations.dedup();
    let work_mode = role.work_mode.as_deref().filter(|value| !value.is_empty());
    if locations.is_empty() {
        match (role.location.as_deref(), work_mode) {
            (Some(location), Some(work_mode))
                if !location.is_empty()
                    && job_report_canonical_key_text(location)
                        == job_report_canonical_key_text(work_mode) =>
            {
                Some(work_mode.to_string())
            }
            (Some(location), Some(work_mode)) if !location.is_empty() => {
                Some(format!("{location}; {work_mode}"))
            }
            (Some(location), _) if !location.is_empty() => Some(location.to_string()),
            (_, Some(work_mode)) => Some(work_mode.to_string()),
            _ => None,
        }
    } else {
        let location = locations.join("/");
        Some(match work_mode {
            Some(work_mode) => format!("{location}; {work_mode}"),
            None => location,
        })
    }
}

fn job_report_role_family_key(role: &JobRoleCard) -> String {
    format!(
        "{}::{}",
        job_report_canonical_key_text(&role.company),
        job_report_canonical_key_text(&job_report_display_role_title(role))
    )
}

fn job_report_display_role_title(role: &JobRoleCard) -> String {
    let title = job_report_strip_trailing_geo_suffix(&role.role_title);
    if title.is_empty() {
        role.role_title.clone()
    } else {
        title
    }
}

fn job_report_strip_trailing_geo_suffix(title: &str) -> String {
    let mut current = title.trim().to_string();
    loop {
        let mut changed = false;
        if let Some((first, _rest)) = current.split_once(';') {
            let stripped = job_report_strip_trailing_geo_suffix(first);
            if !stripped.is_empty() {
                current = stripped;
                changed = true;
            }
        }
        if let Some(start) = current.rfind(" (") {
            if current.ends_with(')') {
                let suffix = current[start + 2..current.len() - 1].trim();
                if job_report_geo_suffix_is_location(suffix) {
                    current.truncate(start);
                    changed = true;
                }
            }
        }
        if !changed {
            for suffix in [
                " Remote-Friendly",
                " Remote-friendly",
                " remote-friendly",
                " Remote First",
                " Remote-First",
                " Remote",
                " remote",
                " London, UK",
                " London, United Kingdom",
                " London",
                " United Kingdom",
                " Ontario, CAN",
                " Ontario",
                " Toronto, Canada",
                " Toronto",
                " Vancouver, Canada",
                " Vancouver",
                " San Francisco, CA",
                " San Francisco",
                " New York, NY",
                " New York",
                " Seattle, WA",
                " Seattle",
                " United States",
                " Canada",
                " Europe",
                " EMEA",
                " Global",
                " Worldwide",
            ] {
                if current.ends_with(suffix) {
                    current.truncate(current.len().saturating_sub(suffix.len()));
                    current = current.trim().to_string();
                    changed = true;
                    break;
                }
            }
        }
        if !changed {
            for separator in [" - ", " | ", " / ", ", "] {
                if let Some((prefix, suffix)) = current.rsplit_once(separator) {
                    if !prefix.trim().is_empty() && job_report_geo_suffix_is_location(suffix) {
                        current = prefix.trim().to_string();
                        changed = true;
                        break;
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
    current.trim().to_string()
}

fn job_report_canonical_key_text(text: &str) -> String {
    let mut out = String::new();
    let mut previous_space = false;
    for ch in text.to_ascii_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            previous_space = false;
        } else if !previous_space {
            out.push(' ');
            previous_space = true;
        }
    }
    out.trim().to_string()
}

pub(crate) fn job_report_role_is_uk_plausible(role: &JobRoleCard) -> bool {
    let geo_text = [
        Some(role.role_title.as_str()),
        role.location.as_deref(),
        role.work_mode.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ");
    if geo_text.trim().is_empty() {
        return true;
    }
    let normalized = job_report_normalized_geo_text(&geo_text);
    if job_report_geo_text_has_positive_uk_relevance(&normalized) {
        return true;
    }
    if job_report_geo_text_has_excluded_region(&normalized) {
        return false;
    }
    if job_report_geo_text_mentions_remote(&normalized) {
        return true;
    }
    true
}

fn job_report_geo_suffix_is_location(text: &str) -> bool {
    let normalized = job_report_normalized_geo_text(text);
    job_report_geo_text_has_positive_uk_relevance(&normalized)
        || job_report_geo_text_has_excluded_region(&normalized)
        || job_report_geo_text_mentions_remote(&normalized)
}

fn job_report_normalized_geo_text(text: &str) -> String {
    let mut out = String::from(" ");
    let mut previous_space = true;
    for ch in text.to_lowercase().chars() {
        let normalized = match ch {
            'á' | 'à' | 'â' | 'ä' | 'å' | 'ã' => 'a',
            'ç' => 'c',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ñ' => 'n',
            'ó' | 'ò' | 'ô' | 'ö' | 'õ' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            _ => ch,
        };
        if normalized.is_ascii_alphanumeric() {
            out.push(normalized);
            previous_space = false;
        } else if !previous_space {
            out.push(' ');
            previous_space = true;
        }
    }
    if !out.ends_with(' ') {
        out.push(' ');
    }
    out
}

fn job_report_geo_text_mentions_remote(text: &str) -> bool {
    text.contains(" remote ") || text.contains(" remote first ") || text.contains(" distributed ")
}

fn job_report_geo_text_has_positive_uk_relevance(text: &str) -> bool {
    [
        " london ",
        " united kingdom ",
        " uk ",
        " england ",
        " scotland ",
        " wales ",
        " ireland ",
        " europe ",
        " european ",
        " eu ",
        " emea ",
        " global ",
        " worldwide ",
        " anywhere ",
        " gmt ",
        " utc ",
        " cet ",
        " berlin ",
        " amsterdam ",
        " paris ",
        " dublin ",
        " lisbon ",
        " madrid ",
        " barcelona ",
        " stockholm ",
        " copenhagen ",
        " helsinki ",
        " vienna ",
        " austria ",
        " munich ",
        " germany ",
        " france ",
        " spain ",
        " netherlands ",
        " poland ",
        " warsaw ",
        " belgium ",
        " brussels ",
        " luxembourg ",
        " italy ",
        " sweden ",
        " denmark ",
        " finland ",
        " norway ",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

fn job_report_positive_geo_labels(text: &str) -> Vec<String> {
    let normalized = job_report_normalized_geo_text(text);
    let mut labels = Vec::new();
    for (needle, label) in [
        (" london ", "London"),
        (" united kingdom ", "United Kingdom"),
        (" uk ", "United Kingdom"),
        (" england ", "United Kingdom"),
        (" scotland ", "United Kingdom"),
        (" wales ", "United Kingdom"),
        (" ireland ", "Ireland"),
        (" europe ", "Europe"),
        (" european ", "Europe"),
        (" eu ", "Europe"),
        (" emea ", "EMEA"),
        (" global ", "Global"),
        (" worldwide ", "Worldwide"),
        (" anywhere ", "Global"),
        (" berlin ", "Europe"),
        (" amsterdam ", "Europe"),
        (" paris ", "Europe"),
        (" dublin ", "Ireland"),
        (" lisbon ", "Europe"),
        (" madrid ", "Europe"),
        (" barcelona ", "Europe"),
        (" stockholm ", "Europe"),
        (" copenhagen ", "Europe"),
        (" helsinki ", "Europe"),
        (" vienna ", "Europe"),
        (" austria ", "Europe"),
        (" munich ", "Europe"),
        (" germany ", "Europe"),
        (" france ", "Europe"),
        (" spain ", "Europe"),
        (" netherlands ", "Europe"),
        (" poland ", "Europe"),
        (" warsaw ", "Europe"),
        (" belgium ", "Europe"),
        (" brussels ", "Europe"),
        (" luxembourg ", "Europe"),
        (" italy ", "Europe"),
        (" sweden ", "Europe"),
        (" denmark ", "Europe"),
        (" finland ", "Europe"),
        (" norway ", "Europe"),
    ] {
        if normalized.contains(needle) {
            labels.push(label.to_string());
        }
    }
    labels.sort();
    labels.dedup();
    labels
}

fn job_report_geo_text_has_excluded_region(text: &str) -> bool {
    [
        " united states ",
        " us ",
        " usa ",
        " u s ",
        " us only ",
        " est timezone ",
        " est time zone ",
        " eastern time ",
        " dallas ",
        " tx ",
        " utah ",
        " north america ",
        " canada ",
        " can ",
        " ontario ",
        " montreal ",
        " amer ",
        " americas ",
        " latin america ",
        " apac ",
        " australia ",
        " sydney ",
        " melbourne ",
        " canberra ",
        " new zealand ",
        " india ",
        " singapore ",
        " seoul ",
        " japan ",
        " tokyo ",
        " abu dhabi ",
        " casablanca ",
        " switzerland ",
        " swiss ",
        " zurich ",
        " geneva ",
        " lausanne ",
        " ch ",
        " san francisco ",
        " new york ",
        " nyc ",
        " seattle ",
        " austin ",
        " boston ",
        " charlotte ",
        " chicago ",
        " denver ",
        " las vegas ",
        " los angeles ",
        " miami ",
        " phoenix ",
        " portland ",
        " raleigh ",
        " salt lake city ",
        " washington dc ",
        " washington d c ",
        " district of columbia ",
        " atlanta ",
        " california ",
        " texas ",
        " massachusetts ",
        " illinois ",
        " colorado ",
        " oregon ",
        " washington state ",
        " nc ",
        " nv ",
        " az ",
        " ut ",
        " ga ",
        " pacific timezone ",
        " pacific time ",
        " pst ",
        " pdt ",
        " toronto ",
        " vancouver ",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

pub(crate) fn job_source_health_status_counts_as_error(status: &str) -> bool {
    !matches!(status, "healthy" | "partial")
}

pub(crate) fn render_job_count_map(counts: &BTreeMap<String, usize>) -> String {
    if counts.is_empty() {
        return "- none".to_string();
    }
    counts
        .iter()
        .map(|(key, count)| format!("- {key}: {count}"))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn job_intro_path_is_warm_ready(path: &JobIntroPath) -> bool {
    matches!(
        path.path_type.as_str(),
        "direct" | "mutual" | "community" | "investor"
    ) || path.confidence == "confirmed"
        || matches!(path.status.as_str(), "ask" | "sent" | "replied")
}

pub(crate) fn render_job_intro_status(intro_paths: &[JobIntroPath]) -> String {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    counts.insert(
        "warm_intro_ready".to_string(),
        intro_paths
            .iter()
            .filter(|path| job_intro_path_is_warm_ready(path))
            .count(),
    );
    for path in intro_paths {
        *counts.entry(path.status.clone()).or_insert(0) += 1;
    }
    render_job_count_map(&counts)
}

pub(crate) fn render_job_weekly_role_changes(
    role_events: &[JobRoleStatusEvent],
    role_labels: &BTreeMap<String, String>,
) -> String {
    if role_events.is_empty() {
        return "- none".to_string();
    }
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for event in role_events {
        *counts.entry(event.status.clone()).or_insert(0) += 1;
    }
    let details = role_events
        .iter()
        .take(10)
        .map(|event| {
            let role = role_labels
                .get(&event.role_id)
                .cloned()
                .unwrap_or_else(|| event.role_id.clone());
            let tier_change = match (&event.previous_tier, &event.current_tier) {
                (Some(previous), Some(current)) => format!(" {previous} -> {current}"),
                (None, Some(current)) => format!(" -> {current}"),
                (Some(previous), None) => format!(" {previous} -> unknown"),
                (None, None) => String::new(),
            };
            let note = event
                .note
                .as_ref()
                .filter(|value| !value.is_empty())
                .map(|value| format!(" - {value}"))
                .unwrap_or_default();
            format!("- {role}: {}{}{}", event.status, tier_change, note)
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("{}\n\n{}", render_job_count_map(&counts), details)
}

pub(crate) fn render_job_weekly_next_actions(
    intro_paths: &[JobIntroPath],
    contact_names: &BTreeMap<String, String>,
    role_labels: &BTreeMap<String, String>,
    applications: &[JobApplication],
) -> String {
    let mut actions = Vec::new();
    for path in intro_paths {
        let Some(next_action) = path.next_action.as_ref().filter(|value| !value.is_empty()) else {
            continue;
        };
        let role = role_labels
            .get(&path.role_id)
            .cloned()
            .unwrap_or_else(|| path.role_id.clone());
        let contact = contact_names
            .get(&path.contact_id)
            .cloned()
            .unwrap_or_else(|| path.contact_id.clone());
        actions.push(format!(
            "- Intro: {role}: {next_action} (contact: {contact}; path: {}/{}/{})",
            path.path_type, path.confidence, path.status
        ));
    }
    for application in applications {
        let Some(follow_up_at) = application
            .follow_up_at
            .as_ref()
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        let role = role_labels
            .get(&application.role_id)
            .cloned()
            .unwrap_or_else(|| application.role_id.clone());
        actions.push(format!(
            "- Application: {role}: follow up by {follow_up_at} (status: {})",
            application.status
        ));
    }
    if actions.is_empty() {
        "- none".to_string()
    } else {
        actions.into_iter().take(5).collect::<Vec<_>>().join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manual_refresh_input(observed_role_ids: Vec<String>) -> JobManualRefreshInput {
        JobManualRefreshInput {
            profile_id: "jprof-test".to_string(),
            scope: "scheduled job radar refresh".to_string(),
            observed_role_ids,
            stale_role_ids: Vec::new(),
            closed_role_ids: Vec::new(),
            source_health_ids: Vec::new(),
            proof_level: "production_data_proof".to_string(),
            report_artifact_id: None,
        }
    }

    #[test]
    fn job_manual_refresh_allows_scheduled_radar_sized_role_lists() {
        let observed_role_ids = (0..101)
            .map(|index| format!("role-{index:03}"))
            .collect::<Vec<_>>();

        let normalized =
            normalize_job_manual_refresh_input(manual_refresh_input(observed_role_ids))
                .expect("scheduled radar refresh role list should normalize");

        assert_eq!(normalized.observed_role_ids.len(), 101);
    }

    #[test]
    fn job_manual_refresh_still_rejects_oversized_role_lists() {
        let observed_role_ids = (0..=JOB_MAX_IMPORT_ITEMS)
            .map(|index| format!("role-{index:04}"))
            .collect::<Vec<_>>();

        let error = normalize_job_manual_refresh_input(manual_refresh_input(observed_role_ids))
            .expect_err("oversized scheduled radar refresh role list should fail");

        assert!(
            error
                .to_string()
                .contains("too many job observed_role_id values"),
            "{error:?}"
        );
    }
}
