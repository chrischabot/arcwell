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

pub(crate) fn text_contains_case_insensitive(haystack: &str, needle: &str) -> bool {
    haystack
        .to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

pub(crate) struct JobReportRoleGroup<'a> {
    pub(crate) primary: &'a JobShortlistEntry,
    pub(crate) role_ids: BTreeSet<String>,
    pub(crate) location_labels: BTreeSet<String>,
}

pub(crate) struct JobReportRoleEventGroup<'a> {
    pub(crate) primary: &'a JobShortlistEntry,
    pub(crate) role_ids: BTreeSet<String>,
    pub(crate) statuses: BTreeSet<String>,
    pub(crate) location_labels: BTreeSet<String>,
}

pub(crate) const JOB_REPORT_EMAIL_SCORE_FLOOR_PERCENT: f64 = 50.0;

pub(crate) fn collect_job_report_role_groups<'a, I>(entries: I) -> Vec<JobReportRoleGroup<'a>>
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
