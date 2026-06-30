use super::*;

impl Store {
    pub fn x_oauth_authorize_url(
        &self,
        client_id: &str,
        redirect_uri: &str,
        scopes: &[String],
    ) -> Result<XOAuthStart> {
        validate_key(client_id)?;
        validate_public_http_url(redirect_uri)?;
        let scopes = if scopes.is_empty() {
            default_x_oauth_scopes()
        } else {
            scopes.to_vec()
        };
        for scope in &scopes {
            validate_key(scope)?;
        }
        let state = Uuid::new_v4().to_string();
        let code_verifier = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        let code_challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()));
        let mut url = Url::parse("https://x.com/i/oauth2/authorize")?;
        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", client_id)
            .append_pair("redirect_uri", redirect_uri)
            .append_pair("scope", &scopes.join(" "))
            .append_pair("state", &state)
            .append_pair("code_challenge", &code_challenge)
            .append_pair("code_challenge_method", "S256");
        Ok(XOAuthStart {
            authorization_url: url.to_string(),
            state,
            code_verifier,
            code_challenge,
            scopes,
        })
    }

    pub fn resolve_x_oauth_client_id(&self, explicit: Option<&str>) -> Result<String> {
        if let Some(client_id) = explicit.map(str::trim).filter(|value| !value.is_empty()) {
            validate_key(client_id)?;
            return Ok(client_id.to_string());
        }
        self.resolve_x_client_id()?
            .context("X_CLIENT_ID is required; store it with `arcwell secrets set-value X_CLIENT_ID ... --scope x` or pass --client-id")
    }

    pub fn resolve_x_oauth_redirect_uri(&self, explicit: Option<&str>) -> Result<String> {
        let redirect_uri = explicit
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .or_else(|| {
                std::env::var("X_REDIRECT_URI")
                    .ok()
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
            })
            .or_else(|| {
                self.get_usable_secret_value("X_REDIRECT_URI")
                    .ok()
                    .flatten()
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
            })
            .unwrap_or_else(|| "http://127.0.0.1:8765/callback".to_string());
        validate_public_http_url(&redirect_uri)?;
        Ok(redirect_uri)
    }

    pub fn x_oauth_reauthorize_preflight(
        &self,
        redirect_uri: &str,
        scopes: &[String],
    ) -> Result<XOAuthReauthorizePreflightReport> {
        validate_public_http_url(redirect_uri)?;
        let scopes = if scopes.is_empty() {
            default_x_oauth_scopes()
        } else {
            scopes.to_vec()
        };
        for scope in &scopes {
            validate_key(scope)?;
        }
        self.policy_guard(PolicyRequest {
            action: "provider.oauth".to_string(),
            package: Some("arcwell-x".to_string()),
            provider: Some("x".to_string()),
            source: Some("x_oauth".to_string()),
            channel: None,
            subject: None,
            target: Some(excerpt("https://x.com/i/oauth2/authorize", 240)),
            projected_usd: None,
            metadata: json!({
                "operation": "reauthorize_browser",
                "redirect_uri": redirect_uri,
                "scopes": scopes,
            }),
            untrusted_excerpt: None,
        })?;
        Ok(XOAuthReauthorizePreflightReport {
            status: "ready".to_string(),
            redirect_uri: redirect_uri.to_string(),
            scopes,
            policy: "allowed".to_string(),
        })
    }

    pub fn x_oauth_exchange_code(
        &self,
        client_id: &str,
        redirect_uri: &str,
        code: &str,
        code_verifier: &str,
        client_secret: Option<&str>,
    ) -> Result<XOAuthTokenStoreReport> {
        let endpoint =
            std::env::var("ARCWELL_X_API_BASE").unwrap_or_else(|_| "https://api.x.com".to_string());
        self.x_oauth_exchange_code_with_base(
            client_id,
            redirect_uri,
            code,
            code_verifier,
            client_secret,
            &endpoint,
        )
    }

    pub(crate) fn x_oauth_exchange_code_with_base(
        &self,
        client_id: &str,
        redirect_uri: &str,
        code: &str,
        code_verifier: &str,
        client_secret: Option<&str>,
        endpoint: &str,
    ) -> Result<XOAuthTokenStoreReport> {
        validate_key(client_id)?;
        validate_public_http_url(redirect_uri)?;
        validate_oauth_param(code, "authorization code")?;
        validate_oauth_param(code_verifier, "code verifier")?;
        self.policy_guard(PolicyRequest {
            action: "provider.oauth".to_string(),
            package: Some("arcwell-x".to_string()),
            provider: Some("x".to_string()),
            source: Some("x_oauth".to_string()),
            channel: None,
            subject: None,
            target: Some(excerpt(endpoint, 240)),
            projected_usd: Some(estimated_network_fetch_cost(1)),
            metadata: json!({
                "operation": "exchange_code",
                "redirect_uri": redirect_uri,
                "has_explicit_client_secret": client_secret.is_some()
            }),
            untrusted_excerpt: None,
        })?;
        self.require_cost_budget(
            "arcwell-x",
            "x_oauth_exchange",
            "x",
            "oauth_exchange",
            Some("x_oauth"),
            estimated_network_fetch_cost(1),
            "X OAuth exchange",
        )?;
        let client_secret = self.resolve_x_client_secret(client_secret)?;
        let mut form = vec![
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("code_verifier", code_verifier),
        ];
        if client_secret.is_none() {
            form.push(("client_id", client_id));
        }
        let value = post_x_oauth_form(endpoint, client_id, client_secret.as_deref(), &form)?;
        self.store_x_token_response(&value)
    }

    pub fn x_oauth_refresh(
        &self,
        client_id: &str,
        client_secret: Option<&str>,
    ) -> Result<XOAuthTokenStoreReport> {
        let endpoint =
            std::env::var("ARCWELL_X_API_BASE").unwrap_or_else(|_| "https://api.x.com".to_string());
        self.x_oauth_refresh_with_base(client_id, client_secret, &endpoint)
    }

    pub fn x_oauth_revoke(
        &self,
        name: &str,
        client_id: &str,
        client_secret: Option<&str>,
        token_type_hint: Option<&str>,
        delete_local: bool,
    ) -> Result<XOAuthRevocationReport> {
        let endpoint =
            std::env::var("ARCWELL_X_API_BASE").unwrap_or_else(|_| "https://api.x.com".to_string());
        self.x_oauth_revoke_with_base(
            name,
            client_id,
            client_secret,
            token_type_hint,
            delete_local,
            &endpoint,
        )
    }

    pub(crate) fn x_oauth_revoke_with_base(
        &self,
        name: &str,
        client_id: &str,
        client_secret: Option<&str>,
        token_type_hint: Option<&str>,
        delete_local: bool,
        endpoint: &str,
    ) -> Result<XOAuthRevocationReport> {
        validate_x_oauth_secret_name(name)?;
        validate_key(client_id)?;
        if let Some(hint) = token_type_hint {
            validate_x_oauth_token_type_hint(hint)?;
        }
        self.policy_guard(PolicyRequest {
            action: "provider.oauth".to_string(),
            package: Some("arcwell-x".to_string()),
            provider: Some("x".to_string()),
            source: Some("x_oauth".to_string()),
            channel: None,
            subject: None,
            target: Some(excerpt(endpoint, 240)),
            projected_usd: Some(estimated_network_fetch_cost(1)),
            metadata: json!({
                "operation": "revoke",
                "secret_name": name,
                "token_type_hint": token_type_hint,
                "delete_local": delete_local,
                "has_explicit_client_secret": client_secret.is_some()
            }),
            untrusted_excerpt: None,
        })?;
        self.require_cost_budget(
            "arcwell-x",
            "x_oauth_revoke",
            "x",
            "oauth_revoke",
            Some("x_oauth"),
            estimated_network_fetch_cost(1),
            "X OAuth revoke",
        )?;
        let token = self
            .get_secret_value(name)?
            .with_context(|| format!("{name} is required"))?;
        validate_oauth_param(&token, "token")?;
        let client_secret = self.resolve_x_client_secret(client_secret)?;
        let mut form = vec![("token", token.as_str())];
        if let Some(hint) = token_type_hint {
            form.push(("token_type_hint", hint));
        }
        if client_secret.is_none() {
            form.push(("client_id", client_id));
        }
        let provider_status =
            post_x_oauth_revoke_form(endpoint, client_id, client_secret.as_deref(), &form)
                .map_err(|error| {
                    anyhow::anyhow!(
                        "{}",
                        redact_secret_like_text(&error.to_string()).replace(&token, "[REDACTED]")
                    )
                })?;
        let deleted_local_secret = if delete_local {
            self.delete_secret_value(name)?
        } else {
            false
        };
        Ok(XOAuthRevocationReport {
            secret_name: name.to_string(),
            token_type_hint: token_type_hint.map(ToOwned::to_owned),
            provider_status,
            revoked_provider_side: true,
            deleted_local_secret,
        })
    }

    pub fn x_oauth_probe(&self, search_query: Option<&str>) -> Result<XOAuthScopeProbeReport> {
        let endpoint =
            std::env::var("ARCWELL_X_API_BASE").unwrap_or_else(|_| "https://api.x.com".to_string());
        self.x_oauth_probe_with_base(search_query.unwrap_or("from:openai"), &endpoint)
    }

    pub(crate) fn x_oauth_probe_with_base(
        &self,
        search_query: &str,
        endpoint: &str,
    ) -> Result<XOAuthScopeProbeReport> {
        validate_query(search_query)?;
        let source_key = "x:oauth-scope-probe";
        let started_at = now();
        let required_scopes = vec![
            "users.read".to_string(),
            "bookmark.read".to_string(),
            "follows.read".to_string(),
            "tweet.read".to_string(),
        ];
        self.policy_guard(PolicyRequest {
            action: "provider.network".to_string(),
            package: Some("arcwell-x".to_string()),
            provider: Some("x".to_string()),
            source: Some("x_oauth_probe".to_string()),
            channel: None,
            subject: None,
            target: Some(excerpt(endpoint, 240)),
            projected_usd: Some(estimated_network_fetch_cost(4)),
            metadata: json!({
                "operation": "oauth_scope_probe",
                "search_query": search_query,
                "required_scopes": required_scopes,
            }),
            untrusted_excerpt: None,
        })?;
        self.require_cost_budget(
            "arcwell-x",
            "x_oauth_probe",
            "x",
            "oauth_probe",
            Some("x_oauth_probe"),
            estimated_network_fetch_cost(4),
            "X OAuth scope probe",
        )?;

        let mut endpoints = Vec::new();
        let mut account_id = None;
        let mut username = None;
        let base = validated_x_api_base(endpoint)?;
        let token = match self.x_bearer_token_for_endpoint(endpoint) {
            Ok(token) => token,
            Err(error) => {
                let redacted = redact_secret_like_text(&error.to_string());
                endpoints.push(XOAuthScopeProbeEndpoint {
                    name: "bearer_token".to_string(),
                    required_scope: "oauth_material".to_string(),
                    path: "local_secret_or_refresh".to_string(),
                    status: "failed".to_string(),
                    classification: classify_x_probe_error(&redacted),
                    evidence: "could not acquire usable X bearer token before provider probes"
                        .to_string(),
                    error: Some(excerpt(&redacted, 1000)),
                });
                let completed_at = now();
                let report = self.finish_x_oauth_probe_report(
                    source_key,
                    started_at,
                    completed_at,
                    account_id,
                    username,
                    endpoints,
                    required_scopes,
                )?;
                return Ok(report);
            }
        };

        let me_path = "/2/users/me?user.fields=username,name";
        match fetch_x_json(base.join(me_path)?.as_str(), Some(&token)) {
            Ok(value) => {
                let id = value.pointer("/data/id").and_then(Value::as_str);
                let handle = value.pointer("/data/username").and_then(Value::as_str);
                if let Some(id) = id {
                    validate_key(id)?;
                    account_id = Some(id.to_string());
                    username = handle.map(ToOwned::to_owned);
                    endpoints.push(XOAuthScopeProbeEndpoint {
                        name: "users_me".to_string(),
                        required_scope: "users.read".to_string(),
                        path: me_path.to_string(),
                        status: "passed".to_string(),
                        classification: "current_provider_fetch".to_string(),
                        evidence: format!(
                            "provider returned authenticated user id{}",
                            handle
                                .map(|value| format!(" and username {}", excerpt(value, 80)))
                                .unwrap_or_default()
                        ),
                        error: None,
                    });
                } else {
                    endpoints.push(XOAuthScopeProbeEndpoint {
                        name: "users_me".to_string(),
                        required_scope: "users.read".to_string(),
                        path: me_path.to_string(),
                        status: "failed".to_string(),
                        classification: "provider_shape_mismatch".to_string(),
                        evidence: "provider returned 200 but no data.id".to_string(),
                        error: Some("X /2/users/me response missing data.id".to_string()),
                    });
                }
            }
            Err(error) => endpoints.push(x_oauth_probe_failed_endpoint(
                "users_me",
                "users.read",
                me_path,
                error,
            )),
        }

        if let Some(user_id) = account_id.as_deref() {
            let bookmark_path = format!("/2/users/{user_id}/bookmarks?max_results=1");
            match fetch_x_json(base.join(&bookmark_path)?.as_str(), Some(&token)) {
                Ok(value) if x_probe_collection_response_is_valid(&value) => {
                    endpoints.push(XOAuthScopeProbeEndpoint {
                        name: "bookmarks".to_string(),
                        required_scope: "bookmark.read".to_string(),
                        path: "/2/users/:id/bookmarks?max_results=1".to_string(),
                        status: "passed".to_string(),
                        classification: "current_provider_fetch".to_string(),
                        evidence: "provider accepted authenticated bookmarks endpoint".to_string(),
                        error: None,
                    });
                }
                Ok(_) => endpoints.push(XOAuthScopeProbeEndpoint {
                    name: "bookmarks".to_string(),
                    required_scope: "bookmark.read".to_string(),
                    path: "/2/users/:id/bookmarks?max_results=1".to_string(),
                    status: "failed".to_string(),
                    classification: "provider_shape_mismatch".to_string(),
                    evidence: "provider returned 200 but no collection metadata".to_string(),
                    error: Some(
                        "X bookmarks response missing data array or meta object".to_string(),
                    ),
                }),
                Err(error) => endpoints.push(x_oauth_probe_failed_endpoint(
                    "bookmarks",
                    "bookmark.read",
                    "/2/users/:id/bookmarks?max_results=1",
                    error,
                )),
            }

            let following_path = format!("/2/users/{user_id}/following?max_results=1");
            match fetch_x_json(base.join(&following_path)?.as_str(), Some(&token)) {
                Ok(value) if x_probe_collection_response_is_valid(&value) => {
                    endpoints.push(XOAuthScopeProbeEndpoint {
                        name: "following".to_string(),
                        required_scope: "follows.read".to_string(),
                        path: "/2/users/:id/following?max_results=1".to_string(),
                        status: "passed".to_string(),
                        classification: "current_provider_fetch".to_string(),
                        evidence: "provider accepted authenticated following endpoint".to_string(),
                        error: None,
                    });
                }
                Ok(_) => endpoints.push(XOAuthScopeProbeEndpoint {
                    name: "following".to_string(),
                    required_scope: "follows.read".to_string(),
                    path: "/2/users/:id/following?max_results=1".to_string(),
                    status: "failed".to_string(),
                    classification: "provider_shape_mismatch".to_string(),
                    evidence: "provider returned 200 but no collection metadata".to_string(),
                    error: Some(
                        "X following response missing data array or meta object".to_string(),
                    ),
                }),
                Err(error) => endpoints.push(x_oauth_probe_failed_endpoint(
                    "following",
                    "follows.read",
                    "/2/users/:id/following?max_results=1",
                    error,
                )),
            }
        } else {
            for (name, scope, path) in [
                (
                    "bookmarks",
                    "bookmark.read",
                    "/2/users/:id/bookmarks?max_results=1",
                ),
                (
                    "following",
                    "follows.read",
                    "/2/users/:id/following?max_results=1",
                ),
            ] {
                endpoints.push(XOAuthScopeProbeEndpoint {
                    name: name.to_string(),
                    required_scope: scope.to_string(),
                    path: path.to_string(),
                    status: "skipped".to_string(),
                    classification: "dependency_failed".to_string(),
                    evidence: "skipped because /2/users/me did not produce an account id"
                        .to_string(),
                    error: None,
                });
            }
        }

        let mut search_url = base.join("/2/tweets/search/recent")?;
        search_url
            .query_pairs_mut()
            .append_pair("query", search_query)
            .append_pair("max_results", "10")
            .append_pair("tweet.fields", "created_at,author_id")
            .append_pair("user.fields", "username");
        match fetch_x_json(search_url.as_str(), Some(&token)) {
            Ok(value) if x_probe_collection_response_is_valid(&value) => {
                endpoints.push(XOAuthScopeProbeEndpoint {
                    name: "recent_search".to_string(),
                    required_scope: "tweet.read".to_string(),
                    path: "/2/tweets/search/recent".to_string(),
                    status: "passed".to_string(),
                    classification: "current_provider_fetch".to_string(),
                    evidence: format!(
                        "provider accepted recent search query {}",
                        excerpt(search_query, 120)
                    ),
                    error: None,
                });
            }
            Ok(_) => endpoints.push(XOAuthScopeProbeEndpoint {
                name: "recent_search".to_string(),
                required_scope: "tweet.read".to_string(),
                path: "/2/tweets/search/recent".to_string(),
                status: "failed".to_string(),
                classification: "provider_shape_mismatch".to_string(),
                evidence: "provider returned 200 but no search metadata".to_string(),
                error: Some(
                    "X recent search response missing data array or meta object".to_string(),
                ),
            }),
            Err(error) => endpoints.push(x_oauth_probe_failed_endpoint(
                "recent_search",
                "tweet.read",
                "/2/tweets/search/recent",
                error,
            )),
        }

        let completed_at = now();
        self.finish_x_oauth_probe_report(
            source_key,
            started_at,
            completed_at,
            account_id,
            username,
            endpoints,
            required_scopes,
        )
    }

    pub fn provider_credential_probe(
        &self,
        providers: &[String],
    ) -> Result<ProviderCredentialProbeReport> {
        let specs = provider_credential_probe_specs(providers)?;
        self.provider_credential_probe_with_specs(specs)
    }

    pub(crate) fn provider_credential_probe_with_specs(
        &self,
        specs: Vec<ProviderCredentialProbeSpec>,
    ) -> Result<ProviderCredentialProbeReport> {
        if specs.is_empty() {
            bail!("at least one provider must be selected");
        }
        let checked_at = now();
        let providers_requested = specs
            .iter()
            .map(|spec| spec.provider.clone())
            .collect::<Vec<_>>();
        let mut endpoints = Vec::new();
        for spec in specs {
            let source_key = format!("provider:{}:credential-probe", spec.provider);
            let endpoint_label = provider_probe_endpoint_label(&spec.url);
            let mut finish = |status: &str,
                              classification: &str,
                              evidence: String,
                              error: Option<String>,
                              secret_name: Option<String>|
             -> Result<()> {
                let error = error.map(|value| excerpt(&redact_secret_like_text(&value), 1000));
                if status == "passed" {
                    self.record_source_success(SourceHealthUpdate {
                        key: &source_key,
                        provider: &spec.provider,
                        source_kind: "provider_credential_probe",
                        locator: &endpoint_label,
                        last_item_id: secret_name.as_deref(),
                        last_item_date: None,
                        cursor_key: None,
                        cursor_value: None,
                        next_run_at: Some(&now_plus_seconds(6 * 60 * 60)),
                    })?;
                } else {
                    let summary = format!("{classification}: {evidence}");
                    self.record_source_failure(
                        &source_key,
                        &spec.provider,
                        "provider_credential_probe",
                        &endpoint_label,
                        error.as_deref().unwrap_or(&summary),
                    )?;
                }
                endpoints.push(ProviderCredentialProbeEndpoint {
                    provider: spec.provider.clone(),
                    secret_name,
                    endpoint: endpoint_label.clone(),
                    status: status.to_string(),
                    classification: classification.to_string(),
                    evidence,
                    error,
                    source_health_key: source_key.clone(),
                });
                Ok(())
            };

            if let Err(error) = self.policy_guard(PolicyRequest {
                action: "provider.network".to_string(),
                package: Some("arcwell-provider-probe".to_string()),
                provider: Some(spec.provider.clone()),
                source: Some("provider_credential_probe".to_string()),
                channel: None,
                subject: None,
                target: Some(endpoint_label.clone()),
                projected_usd: Some(estimated_network_fetch_cost(1)),
                metadata: json!({
                    "operation": "credential_probe",
                    "provider": spec.provider.clone(),
                    "secret_names": spec.secret_names.clone(),
                }),
                untrusted_excerpt: None,
            }) {
                finish(
                    "failed",
                    "policy_denied",
                    "provider-network policy denied before secret read or provider request"
                        .to_string(),
                    Some(error.to_string()),
                    None,
                )?;
                continue;
            }
            if let Err(error) = self.require_cost_budget(
                "arcwell-provider-probe",
                "provider_credential_probe",
                &spec.provider,
                "credential_probe",
                Some("provider_credential_probe"),
                estimated_network_fetch_cost(1),
                &format!("{} credential probe", spec.provider),
            ) {
                finish(
                    "failed",
                    "cost_denied",
                    "cost budget denied before secret read or provider request".to_string(),
                    Some(error.to_string()),
                    None,
                )?;
                continue;
            }

            let secret = match self.first_usable_provider_probe_secret(&spec) {
                Ok(Some(secret)) => secret,
                Ok(None) => {
                    finish(
                        "failed",
                        "missing_secret",
                        format!(
                            "no usable local secret found; checked {}",
                            spec.secret_names.join(", ")
                        ),
                        None,
                        None,
                    )?;
                    continue;
                }
                Err(error) => {
                    finish(
                        "failed",
                        "secret_unusable",
                        "stored credential metadata is unusable before provider request"
                            .to_string(),
                        Some(error.to_string()),
                        None,
                    )?;
                    continue;
                }
            };

            match fetch_provider_probe_json(&spec, &secret.value) {
                Ok(value) if provider_probe_evidence_passes(&spec, &value) => {
                    finish(
                        "passed",
                        "current_provider_fetch",
                        provider_probe_success_evidence(&spec, &value),
                        None,
                        Some(secret.name),
                    )?;
                }
                Ok(_value) => {
                    finish(
                        "failed",
                        "provider_shape_mismatch",
                        "provider returned 200 but the response did not match the expected proof shape"
                            .to_string(),
                        Some(format!(
                            "{} credential probe response did not match expected shape",
                            spec.provider
                        )),
                        Some(secret.name),
                    )?;
                }
                Err(error) => {
                    let redacted = redact_secret_like_text(&error.to_string());
                    finish(
                        "failed",
                        &classify_provider_probe_error(&redacted),
                        "provider rejected or failed the credential probe".to_string(),
                        Some(redacted),
                        Some(secret.name),
                    )?;
                }
            }
        }
        let missing_or_failed_providers = endpoints
            .iter()
            .filter(|endpoint| endpoint.status != "passed")
            .map(|endpoint| endpoint.provider.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let passed_any = endpoints.iter().any(|endpoint| endpoint.status == "passed");
        let status = if missing_or_failed_providers.is_empty() {
            "passed"
        } else if passed_any {
            "partial"
        } else {
            "failed"
        }
        .to_string();
        let source_health_keys = endpoints
            .iter()
            .map(|endpoint| endpoint.source_health_key.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        Ok(ProviderCredentialProbeReport {
            status,
            checked_at,
            providers_requested,
            endpoints,
            source_health_keys,
            missing_or_failed_providers,
        })
    }

    pub(crate) fn first_usable_provider_probe_secret(
        &self,
        spec: &ProviderCredentialProbeSpec,
    ) -> Result<Option<ProviderProbeSecret>> {
        for name in &spec.secret_names {
            if let Some(value) = self.get_usable_secret_value(name)? {
                return Ok(Some(ProviderProbeSecret {
                    name: name.clone(),
                    value,
                }));
            }
        }
        Ok(None)
    }

    pub(crate) fn finish_x_oauth_probe_report(
        &self,
        source_key: &str,
        started_at: String,
        completed_at: String,
        account_id: Option<String>,
        username: Option<String>,
        endpoints: Vec<XOAuthScopeProbeEndpoint>,
        required_scopes: Vec<String>,
    ) -> Result<XOAuthScopeProbeReport> {
        let passed_scopes = endpoints
            .iter()
            .filter(|endpoint| endpoint.status == "passed")
            .map(|endpoint| endpoint.required_scope.clone())
            .collect::<BTreeSet<_>>();
        let missing_or_unproven_scopes = required_scopes
            .iter()
            .filter(|scope| !passed_scopes.contains(*scope))
            .cloned()
            .collect::<Vec<_>>();
        let status = if missing_or_unproven_scopes.is_empty() {
            "passed"
        } else if endpoints.iter().any(|endpoint| endpoint.status == "passed") {
            "partial"
        } else {
            "failed"
        }
        .to_string();
        if status == "passed" {
            self.record_source_success(SourceHealthUpdate {
                key: source_key,
                provider: "x",
                source_kind: "x_oauth_probe",
                locator: "oauth_scope_probe",
                last_item_id: account_id.as_deref(),
                last_item_date: None,
                cursor_key: None,
                cursor_value: None,
                next_run_at: Some(&now_plus_seconds(6 * 60 * 60)),
            })?;
        } else {
            let summary = endpoints
                .iter()
                .filter(|endpoint| endpoint.status != "passed")
                .map(|endpoint| {
                    format!(
                        "{}:{}:{}",
                        endpoint.name, endpoint.classification, endpoint.evidence
                    )
                })
                .collect::<Vec<_>>()
                .join("; ");
            self.record_source_failure(
                source_key,
                "x",
                "x_oauth_probe",
                "oauth_scope_probe",
                &summary,
            )?;
        }
        let failed_count = endpoints
            .iter()
            .filter(|endpoint| endpoint.status != "passed")
            .count();
        let sync_error = (status != "passed").then(|| {
            format!(
                "unproven X OAuth scope(s): {}",
                missing_or_unproven_scopes.join(", ")
            )
        });
        let sync_run_id = self.record_x_sync_run(XSyncRunInsert {
            account_id: account_id.as_deref(),
            stream: "oauth_scope_probe",
            transport: "x_api",
            status: if status == "passed" {
                "completed"
            } else {
                "failed"
            },
            started_at: &started_at,
            completed_at: &completed_at,
            seen: endpoints.len(),
            inserted: 0,
            updated: 0,
            skipped_duplicates: 0,
            rejected: failed_count,
            cursor_key: Some(source_key),
            previous_cursor: None,
            new_cursor: None,
            error: sync_error.as_deref(),
            metadata: json!({
                "endpoints": endpoints.clone(),
                "required_scopes": required_scopes.clone(),
                "missing_or_unproven_scopes": missing_or_unproven_scopes.clone(),
            }),
        })?;
        Ok(XOAuthScopeProbeReport {
            status,
            account_id,
            username,
            endpoints,
            required_scopes,
            missing_or_unproven_scopes,
            source_health_key: source_key.to_string(),
            sync_run_id,
        })
    }

    pub(crate) fn x_oauth_refresh_with_base(
        &self,
        client_id: &str,
        client_secret: Option<&str>,
        endpoint: &str,
    ) -> Result<XOAuthTokenStoreReport> {
        validate_key(client_id)?;
        self.policy_guard(PolicyRequest {
            action: "provider.oauth".to_string(),
            package: Some("arcwell-x".to_string()),
            provider: Some("x".to_string()),
            source: Some("x_oauth".to_string()),
            channel: None,
            subject: None,
            target: Some(excerpt(endpoint, 240)),
            projected_usd: Some(estimated_network_fetch_cost(1)),
            metadata: json!({
                "operation": "refresh",
                "has_explicit_client_secret": client_secret.is_some()
            }),
            untrusted_excerpt: None,
        })?;
        self.require_cost_budget(
            "arcwell-x",
            "x_oauth_refresh",
            "x",
            "oauth_refresh",
            Some("x_oauth"),
            estimated_network_fetch_cost(1),
            "X OAuth refresh",
        )?;
        let refresh_token = self
            .get_usable_secret_value("X_REFRESH_TOKEN")?
            .context("X_REFRESH_TOKEN is required")?;
        validate_oauth_param(&refresh_token, "refresh token")?;
        let client_secret = self.resolve_x_client_secret(client_secret)?;
        let mut form = vec![
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token.as_str()),
        ];
        if client_secret.is_none() {
            form.push(("client_id", client_id));
        }
        let value = post_x_oauth_form(endpoint, client_id, client_secret.as_deref(), &form)
            .map_err(|error| {
                anyhow::anyhow!(
                    "{}",
                    redact_secret_like_text(&error.to_string())
                        .replace(&refresh_token, "[REDACTED]")
                )
            })?;
        self.store_x_token_response(&value)
    }

    pub(crate) fn resolve_x_client_secret(&self, explicit: Option<&str>) -> Result<Option<String>> {
        let secret = explicit
            .map(ToOwned::to_owned)
            .or_else(|| std::env::var("X_CLIENT_SECRET").ok())
            .or_else(|| std::env::var("TWITTER_OAUTH2_CLIENT_SECRET").ok())
            .or_else(|| {
                self.get_usable_secret_value("X_CLIENT_SECRET")
                    .ok()
                    .flatten()
                    .or_else(|| {
                        self.get_usable_secret_value("TWITTER_OAUTH2_CLIENT_SECRET")
                            .ok()
                            .flatten()
                    })
            });
        if let Some(secret) = &secret
            && (secret.is_empty() || secret.len() > 20_000)
        {
            bail!("X client secret is invalid");
        }
        Ok(secret)
    }

    pub(crate) fn store_x_token_response(&self, value: &Value) -> Result<XOAuthTokenStoreReport> {
        let mut stored = Vec::new();
        let expires_at = value
            .get("expires_in")
            .and_then(Value::as_i64)
            .filter(|seconds| *seconds > 0)
            .map(now_plus_seconds);
        if let Some(access_token) = value.get("access_token").and_then(Value::as_str) {
            self.set_secret_value_with_metadata(
                "X_BEARER_TOKEN",
                access_token,
                "x",
                Some("x"),
                expires_at.as_deref(),
            )?;
            stored.push("X_BEARER_TOKEN".to_string());
        }
        if let Some(refresh_token) = value.get("refresh_token").and_then(Value::as_str) {
            self.set_secret_value_with_metadata(
                "X_REFRESH_TOKEN",
                refresh_token,
                "x",
                Some("x"),
                None,
            )?;
            stored.push("X_REFRESH_TOKEN".to_string());
        }
        if stored.is_empty() {
            bail!("X OAuth response did not include an access_token or refresh_token");
        }
        Ok(XOAuthTokenStoreReport {
            stored,
            token_type: value
                .get("token_type")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
            expires_in: value.get("expires_in").and_then(Value::as_i64),
            scope: value
                .get("scope")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
        })
    }

    pub fn x_recent_search(&self, query: &str, max_results: usize) -> Result<XImportReport> {
        let endpoint =
            std::env::var("ARCWELL_X_API_BASE").unwrap_or_else(|_| "https://api.x.com".to_string());
        self.x_recent_search_with_base(query, max_results, &endpoint)
    }

    pub(crate) fn x_recent_search_with_base(
        &self,
        query: &str,
        max_results: usize,
        endpoint: &str,
    ) -> Result<XImportReport> {
        self.x_recent_search_with_base_and_job_id(query, max_results, endpoint, None)
    }

    pub(crate) fn x_recent_search_with_base_and_job_id(
        &self,
        query: &str,
        max_results: usize,
        endpoint: &str,
        job_id: Option<&str>,
    ) -> Result<XImportReport> {
        validate_query(query)?;
        let cursor_key = format!("x:recent-search:{query}");
        let source_key = cursor_key.clone();
        let job_id = job_id.unwrap_or("x_recent_search");
        let projected = estimated_x_recent_search_cost(max_results);
        self.policy_guard(PolicyRequest {
            action: "provider.network".to_string(),
            package: Some("arcwell-x".to_string()),
            provider: Some("x".to_string()),
            source: Some("x_recent_search".to_string()),
            channel: None,
            subject: None,
            target: Some(endpoint.to_string()),
            projected_usd: Some(projected),
            metadata: json!({ "query": query, "max_results": max_results.clamp(10, 100) }),
            untrusted_excerpt: None,
        })?;
        self.require_cost_budget(
            "arcwell-x",
            job_id,
            "x",
            "recent_search",
            Some("x_recent_search"),
            projected,
            "X recent search",
        )?;
        let started_at = now();
        let mut previous_cursor_for_run: Option<String> = None;
        let mut new_cursor_for_run: Option<String> = None;
        let result = (|| -> Result<XImportReport> {
            let token = self.x_bearer_token_for_endpoint(endpoint)?;
            let previous_cursor = self.get_cursor(&cursor_key)?.map(|cursor| cursor.value);
            previous_cursor_for_run = previous_cursor.clone();
            let base = validated_x_api_base(endpoint)?;
            let mut url = base.join("/2/tweets/search/recent")?;
            {
                let mut pairs = url.query_pairs_mut();
                pairs
                    .append_pair("query", query)
                    .append_pair("max_results", &max_results.clamp(10, 100).to_string())
                    .append_pair("tweet.fields", "created_at,author_id,public_metrics")
                    .append_pair("expansions", "author_id")
                    .append_pair("user.fields", "username,name");
                if let Some(since_id) = &previous_cursor {
                    pairs.append_pair("since_id", since_id);
                }
            }
            let value = fetch_x_json(url.as_str(), Some(&token))?;
            x_fail_on_response_errors(&value)?;
            let import_value =
                x_search_response_to_import_items(&value, "recent_search", Some(query))?;
            let report = self.import_x_json_value_without_sync_run(&import_value)?;
            if report.rejected > 0 {
                let first_error = report
                    .rejected_errors
                    .first()
                    .map(|error| format!("; first rejection: {error}"))
                    .unwrap_or_default();
                bail!(
                    "X recent search returned {rejected} malformed item(s){first_error}; cursor was not advanced",
                    rejected = report.rejected
                );
            }
            let newest_id = value.pointer("/meta/newest_id").and_then(Value::as_str);
            let effective_cursor = x_effective_cursor(previous_cursor.as_deref(), newest_id);
            new_cursor_for_run = effective_cursor.clone();
            if effective_cursor.as_deref() != previous_cursor.as_deref()
                && let Some(cursor) = &effective_cursor
            {
                self.set_cursor(&cursor_key, cursor)?;
            }
            self.record_source_success(SourceHealthUpdate {
                key: &source_key,
                provider: "x",
                source_kind: "x_recent_search",
                locator: query,
                last_item_id: report.items.first().map(|item| item.x_id.as_str()),
                last_item_date: report
                    .items
                    .first()
                    .and_then(|item| item.created_at.as_deref()),
                cursor_key: Some(&cursor_key),
                cursor_value: effective_cursor.as_deref().or(newest_id),
                next_run_at: Some(&now_plus_seconds(900)),
            })?;
            Ok(report)
        })();
        let completed_at = now();
        match &result {
            Ok(report) => {
                self.record_x_sync_run(XSyncRunInsert {
                    account_id: None,
                    stream: "recent_search",
                    transport: "x_api",
                    status: "completed",
                    started_at: &started_at,
                    completed_at: &completed_at,
                    seen: report.seen,
                    inserted: report.imported,
                    updated: 0,
                    skipped_duplicates: report.skipped_duplicates,
                    rejected: report.rejected,
                    cursor_key: Some(&cursor_key),
                    previous_cursor: previous_cursor_for_run.as_deref(),
                    new_cursor: new_cursor_for_run.as_deref(),
                    error: None,
                    metadata: json!({ "query": query, "max_results": max_results.clamp(10, 100) }),
                })?;
            }
            Err(error) => {
                let error_text = error.to_string();
                if x_failure_should_release_budget(error) {
                    let _ = self.release_cost_reservation(
                        "arcwell-x",
                        job_id,
                        "x",
                        "recent_search",
                        Some("x_recent_search"),
                    );
                }
                let _ = self.record_source_failure(
                    &source_key,
                    "x",
                    "x_recent_search",
                    query,
                    &error_text,
                );
                let _ = self.record_x_sync_run(XSyncRunInsert {
                    account_id: None,
                    stream: "recent_search",
                    transport: "x_api",
                    status: "failed",
                    started_at: &started_at,
                    completed_at: &completed_at,
                    seen: 0,
                    inserted: 0,
                    updated: 0,
                    skipped_duplicates: 0,
                    rejected: 0,
                    cursor_key: Some(&cursor_key),
                    previous_cursor: previous_cursor_for_run.as_deref(),
                    new_cursor: new_cursor_for_run.as_deref(),
                    error: Some(&error_text),
                    metadata: json!({ "query": query, "max_results": max_results.clamp(10, 100) }),
                });
            }
        }
        result
    }

    pub fn x_import_bookmarks(
        &self,
        bookmark_days: i64,
        max_bookmarks: usize,
    ) -> Result<XImportReport> {
        let endpoint =
            std::env::var("ARCWELL_X_API_BASE").unwrap_or_else(|_| "https://api.x.com".to_string());
        self.x_import_bookmarks_with_base(bookmark_days, max_bookmarks, &endpoint)
    }

    pub(crate) fn x_import_bookmarks_with_base(
        &self,
        bookmark_days: i64,
        max_bookmarks: usize,
        endpoint: &str,
    ) -> Result<XImportReport> {
        let bookmark_days = bookmark_days.clamp(1, 36_500);
        let max_bookmarks = max_bookmarks.clamp(1, 100_000);
        let projected = estimated_x_definitive_watch_cost(max_bookmarks, 0);
        self.policy_guard(PolicyRequest {
            action: "provider.network".to_string(),
            package: Some("arcwell-x".to_string()),
            provider: Some("x".to_string()),
            source: Some("x_import_bookmarks".to_string()),
            channel: None,
            subject: None,
            target: Some(endpoint.to_string()),
            projected_usd: Some(projected),
            metadata: json!({ "bookmark_days": bookmark_days, "max_bookmarks": max_bookmarks }),
            untrusted_excerpt: None,
        })?;
        self.require_cost_budget(
            "arcwell-x",
            "x_import_bookmarks",
            "x",
            "bookmarks",
            Some("x_import_bookmarks"),
            projected,
            "X bookmark import",
        )?;

        let started_at = now();
        let mut account_id_for_run: Option<String> = None;
        let result = (|| -> Result<XImportReport> {
            let token = self.x_bearer_token_for_endpoint(endpoint)?;
            let base = validated_x_api_base(endpoint)?;
            let user_id = self.x_user_id(&base, &token)?;
            account_id_for_run = Some(user_id.clone());
            let cutoff = Utc::now() - chrono::Duration::days(bookmark_days);
            let mut seen = 0;
            let mut imported = 0;
            let mut skipped_duplicates = 0;
            let mut rejected = 0;
            let mut rejected_errors = Vec::new();
            let mut imported_items = Vec::new();
            let mut pagination_token: Option<String> = None;
            let mut pages_fetched = 0;
            let mut exhausted = false;
            let mut stop_reason = "not_started".to_string();

            while seen < max_bookmarks {
                let page_size = (max_bookmarks - seen).clamp(1, 100);
                let mut url = base.join(&format!("/2/users/{user_id}/bookmarks"))?;
                {
                    let mut pairs = url.query_pairs_mut();
                    pairs
                        .append_pair("max_results", &page_size.to_string())
                        .append_pair(
                            "tweet.fields",
                            "created_at,author_id,public_metrics,lang,entities,conversation_id,referenced_tweets",
                        )
                        .append_pair("expansions", "author_id")
                        .append_pair(
                            "user.fields",
                            "username,name,description,verified,verified_type",
                        );
                    if let Some(token) = &pagination_token {
                        pairs.append_pair("pagination_token", token);
                    }
                }
                let value = fetch_x_json(url.as_str(), Some(&token))?;
                x_fail_on_response_errors(&value)?;
                pages_fetched += 1;
                let tweets = value
                    .get("data")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                if tweets.is_empty() {
                    exhausted = true;
                    stop_reason = "empty_page".to_string();
                    break;
                }
                let users = x_users_by_id(&value);
                for tweet in tweets {
                    if seen >= max_bookmarks {
                        break;
                    }
                    seen += 1;
                    match x_bookmark_tweet_to_item_input(&tweet, &users, cutoff) {
                        Ok(Some(input)) => match self.insert_x_item(input) {
                            Ok(Some(item)) => {
                                imported += 1;
                                imported_items.push(item);
                            }
                            Ok(None) => skipped_duplicates += 1,
                            Err(error) => {
                                rejected += 1;
                                if rejected_errors.len() < 10 {
                                    rejected_errors.push(excerpt(
                                        &redact_secret_like_text(&error.to_string()),
                                        500,
                                    ));
                                }
                            }
                        },
                        Ok(None) => {}
                        Err(error) => {
                            rejected += 1;
                            if rejected_errors.len() < 10 {
                                rejected_errors.push(excerpt(
                                    &redact_secret_like_text(&error.to_string()),
                                    500,
                                ));
                            }
                        }
                    }
                }
                pagination_token = value
                    .pointer("/meta/next_token")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned);
                if pagination_token.is_none() {
                    exhausted = true;
                    stop_reason = "provider_exhausted".to_string();
                    break;
                }
            }
            if !exhausted {
                stop_reason = if seen >= max_bookmarks {
                    "requested_limit_reached".to_string()
                } else {
                    "stopped_before_exhaustion".to_string()
                };
            }
            let source_card_projections = imported_items
                .iter()
                .filter(|item| item.source_card_id.is_some())
                .count();
            let mut drift_warnings = Vec::new();
            if source_card_projections < imported {
                drift_warnings.push(format!(
                    "{} imported bookmark rows lacked source-card projection",
                    imported - source_card_projections
                ));
            }

            Ok(XImportReport {
                seen,
                imported,
                skipped_duplicates,
                rejected,
                rejected_errors,
                pages_fetched: Some(pages_fetched),
                requested_limit: Some(max_bookmarks),
                exhausted: Some(exhausted),
                stop_reason: Some(stop_reason),
                next_token: pagination_token,
                source_card_projections: Some(source_card_projections),
                drift_warnings,
                items: imported_items,
            })
        })();
        let completed_at = now();
        match &result {
            Ok(report) => {
                self.record_source_success(SourceHealthUpdate {
                    key: "x:bookmarks",
                    provider: "x",
                    source_kind: "x_import_bookmarks",
                    locator: "bookmarks",
                    last_item_id: report.items.first().map(|item| item.x_id.as_str()),
                    last_item_date: report
                        .items
                        .first()
                        .and_then(|item| item.created_at.as_deref()),
                    cursor_key: None,
                    cursor_value: report.next_token.as_deref(),
                    next_run_at: Some(&now_plus_seconds(6 * 60 * 60)),
                })?;
                self.record_x_sync_run(XSyncRunInsert {
                    account_id: account_id_for_run.as_deref(),
                    stream: "bookmarks",
                    transport: "x_api",
                    status: "completed",
                    started_at: &started_at,
                    completed_at: &completed_at,
                    seen: report.seen,
                    inserted: report.imported,
                    updated: 0,
                    skipped_duplicates: report.skipped_duplicates,
                    rejected: report.rejected,
                    cursor_key: None,
                    previous_cursor: None,
                    new_cursor: None,
                    error: None,
                    metadata: json!({
                        "bookmark_days": bookmark_days,
                        "max_bookmarks": max_bookmarks,
                        "pages_fetched": report.pages_fetched,
                        "requested_limit": report.requested_limit,
                        "exhausted": report.exhausted,
                        "stop_reason": report.stop_reason,
                        "next_token_present": report.next_token.is_some(),
                        "source_card_projections": report.source_card_projections,
                        "drift_warnings": report.drift_warnings,
                    }),
                })?;
            }
            Err(error) => {
                let error_text = error.to_string();
                if x_failure_should_release_budget(error) {
                    let _ = self.release_cost_reservation(
                        "arcwell-x",
                        "x_import_bookmarks",
                        "x",
                        "bookmarks",
                        Some("x_import_bookmarks"),
                    );
                }
                let _ = self.record_source_failure(
                    "x:bookmarks",
                    "x",
                    "x_import_bookmarks",
                    "bookmarks",
                    &error_text,
                );
                let _ = self.record_x_sync_run(XSyncRunInsert {
                    account_id: account_id_for_run.as_deref(),
                    stream: "bookmarks",
                    transport: "x_api",
                    status: "failed",
                    started_at: &started_at,
                    completed_at: &completed_at,
                    seen: 0,
                    inserted: 0,
                    updated: 0,
                    skipped_duplicates: 0,
                    rejected: 0,
                    cursor_key: None,
                    previous_cursor: None,
                    new_cursor: None,
                    error: Some(&error_text),
                    metadata: json!({
                        "bookmark_days": bookmark_days,
                        "max_bookmarks": max_bookmarks,
                        "stop_reason": "failed",
                    }),
                });
            }
        }
        result
    }

    pub fn x_import_following_watch_sources(
        &self,
        max_users: usize,
    ) -> Result<XFollowingWatchImportReport> {
        let endpoint =
            std::env::var("ARCWELL_X_API_BASE").unwrap_or_else(|_| "https://api.x.com".to_string());
        self.x_import_following_watch_sources_with_base(max_users, &endpoint)
    }
}
