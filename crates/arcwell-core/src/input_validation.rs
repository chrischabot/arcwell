use crate::*;

pub(crate) fn validate_query(query: &str) -> Result<()> {
    if query.trim().is_empty() {
        bail!("query cannot be empty");
    }
    if query.len() > 500 {
        bail!("query is too long");
    }
    Ok(())
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
