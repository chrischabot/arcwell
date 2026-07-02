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

pub(crate) fn backup_age_seconds(created_at: &str) -> Result<i64> {
    let created_at = DateTime::parse_from_rfc3339(created_at)
        .with_context(|| format!("parsing backup timestamp {created_at}"))?
        .with_timezone(&Utc);
    Ok((Utc::now() - created_at).num_seconds())
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
