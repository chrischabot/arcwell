use super::*;

pub(crate) fn fetch_text(url: &str, bearer_token: Option<&str>) -> Result<String> {
    fetch_text_with_user_agent(url, bearer_token, "arcwell/0.1")
}

pub(crate) fn fetch_text_with_user_agent(
    url: &str,
    bearer_token: Option<&str>,
    user_agent: &str,
) -> Result<String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(20))
        .redirect(Policy::none())
        .build()?;
    let mut request = client
        .get(url)
        .header(
            ACCEPT,
            "application/rss+xml, application/atom+xml, application/xml, text/xml, text/plain, */*",
        )
        .header("user-agent", user_agent);
    if let Some(token) = bearer_token {
        request = request.header(AUTHORIZATION, format!("Bearer {token}"));
    }
    let response = request
        .send()
        .with_context(|| format!("fetch request failed: {url}"))?;
    let status = response.status();
    let retry_after = response
        .headers()
        .get(RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    if !status.is_success() {
        let text = response.text().unwrap_or_default();
        bail!(
            "{}",
            classify_provider_http_error("fetch", status, retry_after.as_deref(), &text)
        );
    }
    if let Some(length) = response
        .headers()
        .get(CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        && length > FETCH_TEXT_MAX_BYTES
    {
        bail!("fetched body is too large");
    }
    let mut bytes = Vec::new();
    let mut limited = response.take(FETCH_TEXT_MAX_BYTES + 1);
    limited
        .read_to_end(&mut bytes)
        .with_context(|| format!("reading fetch response: {url}"))?;
    if bytes.len() > FETCH_TEXT_MAX_BYTES as usize {
        bail!("fetched body is too large");
    }
    String::from_utf8(bytes).with_context(|| format!("fetch returned invalid text: {url}"))
}

pub(crate) fn provider_user_agent(provider: &str) -> String {
    match provider {
        "reddit" => std::env::var("ARCWELL_REDDIT_USER_AGENT")
            .unwrap_or_else(|_| "macos:arcwell-local:v0.1 (by /u/arcwell-local)".to_string()),
        _ => "arcwell/0.1".to_string(),
    }
}

pub(crate) fn default_x_oauth_scopes() -> Vec<String> {
    [
        "tweet.read",
        "users.read",
        "bookmark.read",
        "follows.read",
        "offline.access",
    ]
    .iter()
    .map(|scope| (*scope).to_string())
    .collect()
}

pub(crate) fn provider_credential_probe_specs(
    providers: &[String],
) -> Result<Vec<ProviderCredentialProbeSpec>> {
    let mut selected = providers
        .iter()
        .flat_map(|provider| provider.split(','))
        .map(|provider| provider.trim().to_ascii_lowercase())
        .filter(|provider| !provider.is_empty())
        .collect::<Vec<_>>();
    if selected.is_empty() || selected.iter().any(|provider| provider == "all") {
        selected = vec![
            "github".to_string(),
            "openai".to_string(),
            "brave".to_string(),
            "cloudflare".to_string(),
        ];
    }
    let mut deduped = BTreeSet::new();
    let mut specs = Vec::new();
    for provider in selected {
        if !deduped.insert(provider.clone()) {
            continue;
        }
        let spec = match provider.as_str() {
            "github" => ProviderCredentialProbeSpec {
                provider,
                secret_names: vec!["GITHUB_TOKEN".to_string()],
                url: "https://api.github.com/user".to_string(),
                auth: ProviderProbeAuth::Bearer,
                evidence: ProviderProbeEvidence::GithubUser,
            },
            "openai" => ProviderCredentialProbeSpec {
                provider,
                secret_names: vec!["OPENAI_API_KEY".to_string()],
                url: "https://api.openai.com/v1/models".to_string(),
                auth: ProviderProbeAuth::Bearer,
                evidence: ProviderProbeEvidence::OpenAiModels,
            },
            "brave" => ProviderCredentialProbeSpec {
                provider,
                secret_names: vec![
                    "BRAVE_SEARCH_API_KEY".to_string(),
                    "BRAVE_API_KEY".to_string(),
                ],
                url: "https://api.search.brave.com/res/v1/web/search?q=arcwell&count=1".to_string(),
                auth: ProviderProbeAuth::BraveSearchToken,
                evidence: ProviderProbeEvidence::BraveSearch,
            },
            "cloudflare" => ProviderCredentialProbeSpec {
                provider,
                secret_names: vec!["CLOUDFLARE_API_TOKEN".to_string()],
                url: "https://api.cloudflare.com/client/v4/user/tokens/verify".to_string(),
                auth: ProviderProbeAuth::Bearer,
                evidence: ProviderProbeEvidence::CloudflareTokenVerify,
            },
            _ => bail!("unsupported provider credential probe: {provider}"),
        };
        specs.push(spec);
    }
    Ok(specs)
}

pub(crate) fn provider_probe_endpoint_label(url: &str) -> String {
    Url::parse(url)
        .ok()
        .and_then(|parsed| {
            let host = parsed.host_str()?;
            Some(format!("{host}{}", parsed.path()))
        })
        .unwrap_or_else(|| excerpt(url, 240))
}

pub(crate) fn fetch_provider_probe_json(
    spec: &ProviderCredentialProbeSpec,
    token: &str,
) -> Result<Value> {
    let client = Client::builder().timeout(Duration::from_secs(20)).build()?;
    let mut request = client
        .get(&spec.url)
        .header(ACCEPT, "application/json")
        .header("user-agent", provider_user_agent(&spec.provider));
    request = match spec.auth {
        ProviderProbeAuth::Bearer => request.header(AUTHORIZATION, format!("Bearer {token}")),
        ProviderProbeAuth::BraveSearchToken => request.header("X-Subscription-Token", token),
    };
    let response = request
        .send()
        .with_context(|| format!("{} credential probe request failed", spec.provider))?;
    let status = response.status();
    let retry_after = response
        .headers()
        .get(RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    let text = response
        .text()
        .with_context(|| format!("{} returned unreadable probe response body", spec.provider))?;
    if !status.is_success() {
        bail!(
            "{}",
            classify_provider_http_error(&spec.provider, status, retry_after.as_deref(), &text)
        );
    }
    serde_json::from_str(&text)
        .with_context(|| format!("{} credential probe returned invalid JSON", spec.provider))
}

pub(crate) fn provider_probe_evidence_passes(
    spec: &ProviderCredentialProbeSpec,
    value: &Value,
) -> bool {
    match spec.evidence {
        ProviderProbeEvidence::GithubUser => {
            value.get("login").and_then(Value::as_str).is_some()
                || value.get("id").and_then(Value::as_i64).is_some()
        }
        ProviderProbeEvidence::OpenAiModels => {
            value.get("data").and_then(Value::as_array).is_some()
        }
        ProviderProbeEvidence::BraveSearch => {
            value.get("web").is_some() || value.get("query").is_some()
        }
        ProviderProbeEvidence::CloudflareTokenVerify => value
            .get("success")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        ProviderProbeEvidence::CloudflareAccount => {
            value
                .get("success")
                .and_then(Value::as_bool)
                .unwrap_or(false)
                && value
                    .pointer("/result/id")
                    .and_then(Value::as_str)
                    .is_some()
        }
    }
}

pub(crate) fn provider_probe_success_evidence(
    spec: &ProviderCredentialProbeSpec,
    value: &Value,
) -> String {
    match spec.evidence {
        ProviderProbeEvidence::GithubUser => {
            let login = value
                .get("login")
                .and_then(Value::as_str)
                .map(|value| excerpt(value, 80))
                .unwrap_or_else(|| "authenticated user".to_string());
            format!("provider accepted credential and returned GitHub user {login}")
        }
        ProviderProbeEvidence::OpenAiModels => {
            let count = value
                .get("data")
                .and_then(Value::as_array)
                .map(Vec::len)
                .unwrap_or(0);
            format!(
                "provider accepted credential and returned OpenAI models list with {count} item(s)"
            )
        }
        ProviderProbeEvidence::BraveSearch => {
            "provider accepted credential and returned Brave Search response shape".to_string()
        }
        ProviderProbeEvidence::CloudflareTokenVerify => {
            "provider accepted credential and verified Cloudflare API token".to_string()
        }
        ProviderProbeEvidence::CloudflareAccount => {
            "provider accepted credential and returned Cloudflare account details".to_string()
        }
    }
}

pub(crate) fn classify_provider_probe_error(error: &str) -> String {
    let lower = error.to_ascii_lowercase();
    if lower.contains("policy") || lower.contains("denied") {
        "policy_denied".to_string()
    } else if lower.contains("cost") || lower.contains("budget") {
        "cost_denied".to_string()
    } else if lower.contains("missing") || lower.contains("no usable") {
        "missing_secret".to_string()
    } else if lower.contains("token rejected")
        || lower.contains("expired")
        || lower.contains("unauthorized")
        || lower.contains("http 401")
        || lower.contains("forbidden")
        || lower.contains("http 403")
    {
        "provider_revocation_or_expiry".to_string()
    } else if lower.contains("rate limit")
        || lower.contains("quota")
        || lower.contains("too many requests")
        || lower.contains("http 429")
    {
        "quota_or_rate_limit".to_string()
    } else {
        "provider_network_failure".to_string()
    }
}

pub(crate) fn fetch_json(url: &str, bearer_token: Option<&str>, provider: &str) -> Result<Value> {
    let client = Client::builder().timeout(Duration::from_secs(20)).build()?;
    let mut request = client
        .get(url)
        .header(ACCEPT, "application/json")
        .header("user-agent", provider_user_agent(provider));
    if let Some(token) = bearer_token {
        request = request.header(AUTHORIZATION, format!("Bearer {token}"));
    }
    let response = request
        .send()
        .with_context(|| format!("{provider} request failed"))?;
    let status = response.status();
    let retry_after = response
        .headers()
        .get(RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    let text = response
        .text()
        .with_context(|| format!("{provider} returned unreadable response body"))?;
    if !status.is_success() {
        bail!(
            "{}",
            classify_provider_http_error(provider, status, retry_after.as_deref(), &text)
        );
    }
    serde_json::from_str(&text).with_context(|| format!("{provider} returned invalid JSON"))
}

pub(crate) fn classify_provider_http_error(
    provider: &str,
    status: StatusCode,
    retry_after: Option<&str>,
    body: &str,
) -> String {
    let body = redact_secret_like_text(body);
    let body_excerpt = excerpt(&body, 500);
    let mut reason = match status {
        StatusCode::TOO_MANY_REQUESTS => {
            format!("{provider} rate limit or quota exceeded; HTTP 429")
        }
        StatusCode::UNAUTHORIZED => format!("{provider} token rejected or expired; HTTP 401"),
        StatusCode::FORBIDDEN => format!("{provider} request forbidden; HTTP 403"),
        _ => format!("{provider} returned HTTP {}", status.as_u16()),
    };
    if let Some(retry_after) = retry_after
        && !retry_after.trim().is_empty()
    {
        reason.push_str(&format!("; retry_after={}", excerpt(retry_after, 120)));
    }
    if !body_excerpt.trim().is_empty() {
        reason.push_str(&format!("; provider_error={body_excerpt}"));
    }
    reason
}

pub(crate) fn fetch_x_json(url: &str, bearer_token: Option<&str>) -> Result<Value> {
    let client = Client::builder().timeout(Duration::from_secs(20)).build()?;
    let mut request = client
        .get(url)
        .header(ACCEPT, "application/json")
        .header("user-agent", "arcwell/0.1");
    if let Some(token) = bearer_token {
        request = request.header(AUTHORIZATION, format!("Bearer {token}"));
    }
    let response = request.send().context("x request failed")?;
    let status = response.status();
    let retry_after = response
        .headers()
        .get(RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    let text = response.text().unwrap_or_default();
    if !status.is_success() {
        bail!(
            "{}",
            classify_x_http_error(status, retry_after.as_deref(), &text)
        );
    }
    serde_json::from_str(&text).context("x returned invalid JSON")
}

pub(crate) fn classify_x_http_error(
    status: StatusCode,
    retry_after: Option<&str>,
    body: &str,
) -> String {
    let body = redact_secret_like_text(body);
    let body_excerpt = excerpt(&body, 500);
    let mut reason = match status {
        StatusCode::UNAUTHORIZED => {
            "x token rejected or expired; refresh OAuth token before retry".to_string()
        }
        StatusCode::FORBIDDEN => {
            let lower = body_excerpt.to_ascii_lowercase();
            if lower.contains("client-not-enrolled")
                || lower.contains("unsupported")
                || lower.contains("tier")
                || lower.contains("access")
            {
                "x API access tier does not allow this endpoint".to_string()
            } else {
                "x request forbidden; source may be protected, blocked, deleted, or out of scope"
                    .to_string()
            }
        }
        StatusCode::TOO_MANY_REQUESTS => "x rate limit or quota exceeded".to_string(),
        _ => format!("x returned HTTP {}", status.as_u16()),
    };
    if let Some(retry_after) = retry_after
        && !retry_after.trim().is_empty()
    {
        reason.push_str(&format!("; retry_after={}", excerpt(retry_after, 120)));
    }
    if !body_excerpt.trim().is_empty() {
        reason.push_str(&format!("; provider_error={body_excerpt}"));
    }
    reason
}

pub(crate) fn x_probe_collection_response_is_valid(value: &Value) -> bool {
    value.get("data").and_then(Value::as_array).is_some()
        || value.get("meta").and_then(Value::as_object).is_some()
}

pub(crate) fn classify_x_probe_error(error: &str) -> String {
    let lower = error.to_ascii_lowercase();
    if lower.contains("token rejected")
        || lower.contains("expired")
        || lower.contains("invalid_grant")
        || lower.contains("revok")
        || lower.contains("unauthorized")
        || lower.contains("http 401")
    {
        "provider_revocation_or_expiry".to_string()
    } else if lower.contains("scope")
        || lower.contains("bookmark.read")
        || lower.contains("follows.read")
        || lower.contains("tweet.read")
        || lower.contains("users.read")
    {
        "scope_mismatch".to_string()
    } else if lower.contains("tier")
        || lower.contains("access tier")
        || lower.contains("client-not-enrolled")
        || lower.contains("unsupported authentication")
        || lower.contains("does not allow this endpoint")
        || lower.contains("http 403")
    {
        "provider_tier_or_endpoint_denial".to_string()
    } else if lower.contains("rate limit")
        || lower.contains("quota")
        || lower.contains("too many requests")
        || lower.contains("http 429")
    {
        "quota_tier_denial".to_string()
    } else if lower.contains("missing") || lower.contains("required") || lower.contains("not found")
    {
        "missing_refresh_material".to_string()
    } else {
        "provider_network_failure".to_string()
    }
}

pub(crate) fn x_oauth_probe_failed_endpoint(
    name: &str,
    required_scope: &str,
    path: &str,
    error: anyhow::Error,
) -> XOAuthScopeProbeEndpoint {
    let error = redact_secret_like_text(&error.to_string());
    XOAuthScopeProbeEndpoint {
        name: name.to_string(),
        required_scope: required_scope.to_string(),
        path: path.to_string(),
        status: "failed".to_string(),
        classification: classify_x_probe_error(&error),
        evidence: "provider did not accept this endpoint with the current bearer token".to_string(),
        error: Some(excerpt(&error, 1000)),
    }
}

pub(crate) fn x_fail_on_response_errors(value: &Value) -> Result<()> {
    let Some(errors) = value.get("errors").and_then(Value::as_array) else {
        return Ok(());
    };
    if errors.is_empty() {
        return Ok(());
    }
    let error_text = errors
        .iter()
        .take(5)
        .map(|error| {
            let title = error
                .get("title")
                .and_then(Value::as_str)
                .or_else(|| error.get("type").and_then(Value::as_str))
                .unwrap_or("x partial error");
            let detail = error
                .get("detail")
                .and_then(Value::as_str)
                .or_else(|| error.get("message").and_then(Value::as_str))
                .unwrap_or("");
            format!("{title}: {detail}")
        })
        .collect::<Vec<_>>()
        .join("; ");
    bail!(
        "x response contained blocked/protected/deleted or partial-error items; cursor was not advanced: {}",
        excerpt(&redact_secret_like_text(&error_text), 1000)
    )
}

pub(crate) fn x_effective_cursor(previous: Option<&str>, newest: Option<&str>) -> Option<String> {
    match (previous, newest) {
        (None, None) => None,
        (Some(previous), None) => Some(previous.to_string()),
        (None, Some(newest)) => Some(newest.to_string()),
        (Some(previous), Some(newest)) => {
            if x_id_is_newer(newest, previous) {
                Some(newest.to_string())
            } else {
                Some(previous.to_string())
            }
        }
    }
}

pub(crate) fn x_id_is_newer(candidate: &str, previous: &str) -> bool {
    match (candidate.parse::<u128>(), previous.parse::<u128>()) {
        (Ok(candidate), Ok(previous)) => candidate > previous,
        _ => candidate > previous,
    }
}

pub(crate) fn x_failure_should_release_budget(error: &anyhow::Error) -> bool {
    let text = error.to_string().to_ascii_lowercase();
    text.contains("x_bearer_token is required")
        || text.contains("x_refresh_token is required")
        || text.contains("x_client_id is required")
        || text.contains("refreshing expired x_bearer_token failed")
        || text.contains("budget blocked x oauth refresh")
        || text.contains("policy denied provider.oauth")
        || text.contains("expired")
        || text.contains("token rejected")
        || text.contains("rate limit")
        || text.contains("quota exceeded")
        || text.contains("access tier")
        || text.contains("does not allow this endpoint")
}

pub(crate) fn post_x_oauth_form(
    endpoint: &str,
    client_id: &str,
    client_secret: Option<&str>,
    form: &[(&str, &str)],
) -> Result<Value> {
    let base = validated_x_api_base(endpoint)?;
    let url = base.join("/2/oauth2/token")?;
    post_x_oauth_json_form(url, client_id, client_secret, form)
}

pub(crate) fn post_x_oauth_json_form(
    url: Url,
    client_id: &str,
    client_secret: Option<&str>,
    form: &[(&str, &str)],
) -> Result<Value> {
    let client = Client::builder().timeout(Duration::from_secs(20)).build()?;
    let mut request = client
        .post(url)
        .header(ACCEPT, "application/json")
        .header("user-agent", "arcwell/0.1")
        .form(form);
    if let Some(client_secret) = client_secret {
        request = request.basic_auth(client_id, Some(client_secret));
    }
    let response = request.send().context("X OAuth token request failed")?;
    let status = response.status();
    let text = response.text().unwrap_or_default();
    if !status.is_success() {
        bail!(
            "X OAuth token endpoint failed: {}",
            classify_x_http_error(status, None, &text)
        );
    }
    serde_json::from_str(&text).context("X OAuth token endpoint returned invalid JSON")
}

pub(crate) fn post_x_oauth_revoke_form(
    endpoint: &str,
    client_id: &str,
    client_secret: Option<&str>,
    form: &[(&str, &str)],
) -> Result<u16> {
    let base = validated_x_api_base(endpoint)?;
    let url = base.join("/2/oauth2/revoke")?;
    let client = Client::builder().timeout(Duration::from_secs(20)).build()?;
    let mut request = client
        .post(url)
        .header(ACCEPT, "application/json")
        .header("user-agent", "arcwell/0.1")
        .form(form);
    if let Some(client_secret) = client_secret {
        request = request.basic_auth(client_id, Some(client_secret));
    }
    let response = request.send().context("X OAuth revoke request failed")?;
    let status = response.status();
    let text = response.text().unwrap_or_default();
    if !status.is_success() {
        bail!(
            "X OAuth revoke endpoint failed: {}",
            classify_x_http_error(status, None, &text)
        );
    }
    Ok(status.as_u16())
}
