use crate::*;

pub fn now() -> String {
    Utc::now().to_rfc3339()
}

pub(crate) fn now_plus_seconds(seconds: i64) -> String {
    (Utc::now() + chrono::Duration::seconds(seconds)).to_rfc3339()
}

pub(crate) struct ProviderFailureClassification {
    pub(crate) status: &'static str,
    pub(crate) backoff_seconds: i64,
}

#[derive(Debug, Clone)]
pub(crate) struct ProviderCredentialProbeSpec {
    pub(crate) provider: String,
    pub(crate) secret_names: Vec<String>,
    pub(crate) url: String,
    pub(crate) auth: ProviderProbeAuth,
    pub(crate) evidence: ProviderProbeEvidence,
}

#[derive(Debug, Clone)]
pub(crate) enum ProviderProbeAuth {
    Bearer,
    BraveSearchToken,
}

#[derive(Debug, Clone)]
pub(crate) enum ProviderProbeEvidence {
    GithubUser,
    OpenAiModels,
    BraveSearch,
    CloudflareTokenVerify,
    CloudflareAccount,
}

pub(crate) struct ProviderProbeSecret {
    pub(crate) name: String,
    pub(crate) value: String,
}

pub(crate) fn classify_provider_failure(error: &str) -> ProviderFailureClassification {
    let lower = error.to_ascii_lowercase();
    if lower.contains("rate limit")
        || lower.contains("quota")
        || lower.contains("too many requests")
        || lower.contains("http 429")
        || lower.contains("status 429")
    {
        ProviderFailureClassification {
            status: "rate_limited",
            backoff_seconds: 3600,
        }
    } else if lower.contains("timeout") || lower.contains("temporarily unavailable") {
        ProviderFailureClassification {
            status: "transient_error",
            backoff_seconds: 900,
        }
    } else {
        ProviderFailureClassification {
            status: "failed",
            backoff_seconds: 300,
        }
    }
}

pub(crate) fn retry_backoff_seconds(attempts: i64) -> i64 {
    match attempts {
        0 | 1 => 5,
        2 => 30,
        3 => 120,
        _ => 300,
    }
}

pub(crate) fn default_worker_id() -> String {
    format!("arcwell-worker-{}", std::process::id())
}

pub fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub(crate) fn validate_key(key: &str) -> Result<()> {
    if key.trim().is_empty() {
        bail!("key cannot be empty");
    }
    if key.len() > 200 {
        bail!("key is too long");
    }
    Ok(())
}
