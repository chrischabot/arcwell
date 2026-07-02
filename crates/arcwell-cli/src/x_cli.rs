use crate::*;

#[derive(Debug, Serialize)]
pub(crate) struct XOAuthReauthorizeCliReport {
    status: String,
    redirect_uri: String,
    scopes: Vec<String>,
    opened_browser: bool,
    callback_received: bool,
    token_store: Value,
    probe: Value,
}

#[derive(Debug)]
pub(crate) struct XOAuthCallback {
    pub(crate) code: String,
}

#[derive(Debug)]
pub(crate) struct LoopbackRedirect {
    pub(crate) bind_addr: String,
    pub(crate) path: String,
}

// allow: refactoring this N-arg signature is out of scope for the lint-cleanup pass.
#[allow(clippy::too_many_arguments)]
pub(crate) fn x_oauth_reauthorize(
    store: &Store,
    client_id: Option<&str>,
    redirect_uri: Option<&str>,
    client_secret: Option<&str>,
    public_client: bool,
    scopes: &[String],
    timeout_seconds: u64,
    probe_search_query: &str,
    open_browser: bool,
) -> Result<XOAuthReauthorizeCliReport> {
    let client_id = store.resolve_x_oauth_client_id(client_id)?;
    let redirect_uri = store.resolve_x_oauth_redirect_uri(redirect_uri)?;
    let preflight = store.x_oauth_reauthorize_preflight(&redirect_uri, scopes)?;
    let loopback = parse_loopback_redirect_uri(&redirect_uri)?;
    let listener = TcpListener::bind(&loopback.bind_addr)
        .with_context(|| format!("binding OAuth callback listener at {}", loopback.bind_addr))?;
    listener
        .set_nonblocking(true)
        .context("configuring OAuth callback listener")?;
    let start = store.x_oauth_authorize_url(&client_id, &redirect_uri, &preflight.scopes)?;
    eprintln!(
        "Arcwell X OAuth reauthorize pending: redirect_uri={} scopes={} authorization_endpoint={}",
        redirect_uri,
        preflight.scopes.join(","),
        oauth_authorization_endpoint(&start.authorization_url)
    );
    if open_browser {
        open_browser_url(&start.authorization_url)?;
    } else {
        eprintln!("{}", start.authorization_url);
    }
    let callback = wait_for_x_oauth_callback(
        &listener,
        &loopback.path,
        &start.state,
        timeout_seconds.max(1),
    )
    .with_context(|| x_oauth_callback_timeout_context(&start.authorization_url, &redirect_uri))?;
    let token_store = store.x_oauth_exchange_code(
        &client_id,
        &redirect_uri,
        &callback.code,
        &start.code_verifier,
        client_secret,
        public_client,
    )?;
    let probe = store.x_oauth_probe(Some(probe_search_query))?;
    let status = if probe.status == "passed" {
        "passed"
    } else {
        "partial"
    };
    Ok(XOAuthReauthorizeCliReport {
        status: status.to_string(),
        redirect_uri,
        scopes: preflight.scopes,
        opened_browser: open_browser,
        callback_received: true,
        token_store: serde_json::to_value(token_store)?,
        probe: serde_json::to_value(probe)?,
    })
}

pub(crate) fn parse_loopback_redirect_uri(redirect_uri: &str) -> Result<LoopbackRedirect> {
    let rest = redirect_uri
        .strip_prefix("http://")
        .context("OAuth reauthorize redirect URI must be an http loopback URL")?;
    let (authority, path_and_query) = rest.split_once('/').unwrap_or((rest, ""));
    let (host, port) = authority
        .rsplit_once(':')
        .context("OAuth reauthorize redirect URI must include an explicit loopback port")?;
    if !matches!(host, "127.0.0.1" | "localhost" | "[::1]") {
        bail!("OAuth reauthorize redirect URI must use 127.0.0.1, localhost, or [::1]");
    }
    let port: u16 = port
        .parse()
        .context("OAuth reauthorize redirect URI has invalid port")?;
    if port == 0 {
        bail!("OAuth reauthorize redirect URI must use a fixed nonzero port registered with X");
    }
    let path = format!("/{}", path_and_query.split('?').next().unwrap_or(""));
    if path == "/" {
        bail!("OAuth reauthorize redirect URI must include a callback path");
    }
    let bind_host = if host == "[::1]" { "::1" } else { "127.0.0.1" };
    Ok(LoopbackRedirect {
        bind_addr: format!("{bind_host}:{port}"),
        path,
    })
}

pub(crate) fn wait_for_x_oauth_callback(
    listener: &TcpListener,
    expected_path: &str,
    expected_state: &str,
    timeout_seconds: u64,
) -> Result<XOAuthCallback> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_seconds);
    let mut stream = loop {
        match listener.accept() {
            Ok((stream, _)) => break stream,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                if std::time::Instant::now() >= deadline {
                    bail!("timed out waiting for X OAuth browser callback");
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(error) => return Err(error).context("accepting X OAuth browser callback"),
        }
    };
    let mut buffer = [0_u8; 16 * 1024];
    let read = stream
        .read(&mut buffer)
        .context("reading X OAuth callback request")?;
    let request = String::from_utf8_lossy(&buffer[..read]);
    let result = parse_x_oauth_callback_request(&request, expected_path, expected_state);
    let (status_line, body) = match &result {
        Ok(_) => (
            "HTTP/1.1 200 OK",
            "Arcwell captured the X authorization code. You can close this tab.",
        ),
        Err(_) => (
            "HTTP/1.1 400 Bad Request",
            "Arcwell could not accept this X OAuth callback. Return to Codex for details.",
        ),
    };
    let response = format!(
        "{status_line}\r\ncontent-type: text/plain; charset=utf-8\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
    result
}

pub(crate) fn x_oauth_callback_timeout_context(
    authorization_url: &str,
    redirect_uri: &str,
) -> String {
    format!(
        "X OAuth browser callback did not complete after opening authorization_endpoint={}; Chrome may still be on the login page, the X app may not accept redirect_uri={redirect_uri}, or the browser session may require an interactive account challenge",
        oauth_authorization_endpoint(authorization_url)
    )
}

pub(crate) fn oauth_authorization_endpoint(authorization_url: &str) -> &str {
    authorization_url
        .split_once('?')
        .map(|(endpoint, _)| endpoint)
        .unwrap_or(authorization_url)
}

pub(crate) fn parse_x_oauth_callback_request(
    request: &str,
    expected_path: &str,
    expected_state: &str,
) -> Result<XOAuthCallback> {
    let request_line = request
        .lines()
        .next()
        .context("OAuth callback request was empty")?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let target = parts.next().unwrap_or_default();
    if method != "GET" {
        bail!("OAuth callback must use GET");
    }
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    if path != expected_path {
        bail!("OAuth callback path mismatch");
    }
    let params = parse_query_params(query)?;
    if let Some(error) = params.get("error") {
        bail!(
            "X OAuth authorization failed: {}",
            redact_secret_like_text_for_cli(error)
        );
    }
    let state = params
        .get("state")
        .context("OAuth callback missing state")?;
    if state != expected_state {
        bail!("OAuth callback state mismatch");
    }
    let code = params.get("code").context("OAuth callback missing code")?;
    if code.trim().is_empty() || code.len() > 20_000 {
        bail!("OAuth callback code is invalid");
    }
    Ok(XOAuthCallback {
        code: code.to_string(),
    })
}

pub(crate) fn parse_query_params(query: &str) -> Result<BTreeMap<String, String>> {
    let mut params = BTreeMap::new();
    for pair in query.split('&').filter(|value| !value.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        params.insert(
            percent_decode_component(key)?,
            percent_decode_component(value)?,
        );
    }
    Ok(params)
}

pub(crate) fn percent_decode_component(value: &str) -> Result<String> {
    let mut bytes = Vec::with_capacity(value.len());
    let raw = value.as_bytes();
    let mut index = 0;
    while index < raw.len() {
        match raw[index] {
            b'+' => {
                bytes.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < raw.len() => {
                let hex = std::str::from_utf8(&raw[index + 1..index + 3])
                    .context("invalid percent escape")?;
                let byte = u8::from_str_radix(hex, 16).context("invalid percent escape")?;
                bytes.push(byte);
                index += 3;
            }
            b'%' => bail!("truncated percent escape"),
            byte => {
                bytes.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(bytes).context("percent-decoded query component was not UTF-8")
}

pub(crate) fn open_browser_url(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        if ProcessCommand::new("osascript")
            .env("ARCWELL_OAUTH_URL", url)
            .args([
                "-e",
                r#"tell application "Google Chrome" to activate"#,
                "-e",
                r#"tell application "Google Chrome" to open location (system attribute "ARCWELL_OAUTH_URL")"#,
            ])
            .status()
            .is_ok_and(|status| status.success())
        {
            return Ok(());
        }
    }
    #[cfg(target_os = "macos")]
    let attempts: Vec<Vec<&str>> =
        vec![vec!["open", "-a", "Google Chrome", url], vec!["open", url]];
    #[cfg(target_os = "linux")]
    let attempts: Vec<Vec<&str>> = vec![vec!["xdg-open", url]];
    #[cfg(target_os = "windows")]
    let attempts: Vec<Vec<&str>> = vec![vec!["cmd", "/C", "start", "", url]];
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    let attempts: Vec<Vec<&str>> = Vec::new();

    for attempt in attempts {
        let Some((program, args)) = attempt.split_first() else {
            continue;
        };
        if ProcessCommand::new(program)
            .args(args)
            .status()
            .is_ok_and(|status| status.success())
        {
            return Ok(());
        }
    }
    bail!("failed to open browser for X OAuth reauthorization")
}

pub(crate) fn redact_secret_like_text_for_cli(value: &str) -> String {
    if value.len() > 24 {
        "[REDACTED]".to_string()
    } else {
        value.to_string()
    }
}

pub(crate) fn x_command(store: Store, args: XCommand) -> Result<()> {
    match args.command {
        XSubcommand::ImportJson { path } => print_json(&store.import_x_json_file(&path)?),
        XSubcommand::DiscoverArchives { dirs, limit } => {
            print_json(&store.discover_x_archives(&dirs, limit)?)
        }
        XSubcommand::ImportArchive {
            path,
            select,
            limit,
        } => print_json(&store.import_x_archive(&path, &select, limit)?),
        XSubcommand::ExportPortable { out } => print_json(&store.export_x_portable(&out)?),
        XSubcommand::ValidatePortable { dir } => print_json(&store.validate_x_portable(&dir)?),
        XSubcommand::ImportPortable { dir } => print_json(&store.import_x_portable(&dir)?),
        XSubcommand::RecentSearch {
            query,
            max_results,
            transport,
        } => print_json(&store.x_recent_search_with_transport(
            &query,
            max_results,
            transport.as_deref(),
        )?),
        XSubcommand::EnqueueRecentSearch {
            query,
            max_results,
            transport,
        } => print_json(&store.enqueue_x_recent_search_job_with_transport(
            &query,
            max_results,
            transport.as_deref(),
        )?),
        XSubcommand::ImportBookmarks {
            bookmark_days,
            max_bookmarks,
            transport,
        } => print_json(&store.x_import_bookmarks_with_transport(
            bookmark_days,
            max_bookmarks,
            transport.as_deref(),
        )?),
        XSubcommand::ScheduleBookmarks {
            bookmark_days,
            max_bookmarks,
            cadence,
            status,
            transport,
        } => print_json(&store.schedule_x_bookmark_import_with_transport(
            bookmark_days,
            max_bookmarks,
            &cadence,
            &status,
            transport.as_deref(),
        )?),
        XSubcommand::ClusterRadarRun {
            run_id,
            max_source_cards,
        } => print_json(
            &store.create_x_knowledge_clusters_from_radar_run(&run_id, max_source_cards)?,
        ),
        XSubcommand::EditorialDecide { cluster_id } => {
            print_json(&store.run_x_editorial_decision_for_cluster(&cluster_id)?)
        }
        XSubcommand::ImportFollowingWatchSources { max_users } => {
            print_json(&store.x_import_following_watch_sources(max_users)?)
        }
        XSubcommand::RebuildDefinitiveWatchSources {
            bookmark_days,
            max_bookmarks,
            max_recent_follows,
        } => print_json(&store.x_rebuild_definitive_watch_sources(
            bookmark_days,
            max_bookmarks,
            max_recent_follows,
        )?),
        XSubcommand::CurateWatchSources {
            dry_run: _,
            apply,
            mode,
        } => {
            let mode = if apply { mode.as_str() } else { "dry-run" };
            print_json(&store.x_curate_watch_sources(mode)?)
        }
        XSubcommand::RestoreWatchCuration { run_id } => {
            print_json(&store.restore_x_watch_curation_run(&run_id)?)
        }
        XSubcommand::WatchCurationReport { run_id } => {
            let report = if let Some(run_id) = run_id {
                Some(store.x_watch_curation_report(&run_id)?)
            } else {
                store.latest_x_watch_curation_report()?
            };
            print_json(&report)
        }
        XSubcommand::ImportWatchManualRules {
            path,
            rules_json,
            reviewed_by,
            dry_run: _,
            apply,
        } => {
            let raw = if let Some(path) = path {
                fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?
            } else {
                rules_json
            };
            let rules: Vec<XWatchManualRuleInput> =
                serde_json::from_str(&raw).context("parsing X watch manual rules JSON")?;
            print_json(&store.import_x_watch_manual_rules(rules, &reviewed_by, !apply)?)
        }
        XSubcommand::EnrichWatchProfiles {
            run_id,
            handles,
            limit,
        } => print_json(&store.x_enrich_watch_profiles(run_id.as_deref(), &handles, limit)?),
        XSubcommand::MonitorWatchSources {
            max_sources,
            max_results_per_source,
        } => print_json(&store.x_monitor_watch_sources(max_sources, max_results_per_source)?),
        XSubcommand::MonitorWatchSource {
            handle,
            max_results_per_source,
        } => print_json(&store.x_monitor_watch_source(&handle, max_results_per_source)?),
        XSubcommand::RepairHealth {
            defer_rate_limited_hours,
            limit,
        } => print_json(&store.x_repair_health(defer_rate_limited_hours, limit)?),
        XSubcommand::OauthProbe { search_query } => {
            print_json(&store.x_oauth_probe(search_query.as_deref())?)
        }
        XSubcommand::OauthUrl {
            client_id,
            redirect_uri,
            scopes,
        } => {
            let client_id = store.resolve_x_oauth_client_id(client_id.as_deref())?;
            let redirect_uri = store.resolve_x_oauth_redirect_uri(redirect_uri.as_deref())?;
            print_json(&store.x_oauth_authorize_url(&client_id, &redirect_uri, &scopes)?)
        }
        XSubcommand::OauthExchange {
            client_id,
            redirect_uri,
            code,
            code_verifier,
            client_secret,
            public_client,
        } => {
            let client_id = store.resolve_x_oauth_client_id(client_id.as_deref())?;
            let redirect_uri = store.resolve_x_oauth_redirect_uri(redirect_uri.as_deref())?;
            print_json(&store.x_oauth_exchange_code(
                &client_id,
                &redirect_uri,
                &code,
                &code_verifier,
                client_secret.as_deref(),
                public_client,
            )?)
        }
        XSubcommand::OauthReauthorize {
            client_id,
            redirect_uri,
            client_secret,
            public_client,
            scopes,
            timeout_seconds,
            probe_search_query,
            no_open_browser,
        } => print_json(&x_oauth_reauthorize(
            &store,
            client_id.as_deref(),
            redirect_uri.as_deref(),
            client_secret.as_deref(),
            public_client,
            &scopes,
            timeout_seconds,
            &probe_search_query,
            !no_open_browser,
        )?),
        XSubcommand::OauthRefresh {
            client_id,
            client_secret,
            public_client,
        } => {
            let client_id = store.resolve_x_oauth_client_id(client_id.as_deref())?;
            print_json(&store.x_oauth_refresh(
                &client_id,
                client_secret.as_deref(),
                public_client,
            )?)
        }
        XSubcommand::OauthRevoke {
            name,
            client_id,
            client_secret,
            public_client,
            token_type_hint,
            delete_local,
        } => {
            let client_id = store.resolve_x_oauth_client_id(client_id.as_deref())?;
            print_json(&store.x_oauth_revoke(
                &name,
                &client_id,
                client_secret.as_deref(),
                public_client,
                token_type_hint.as_deref(),
                delete_local,
            )?)
        }
        XSubcommand::List {
            query,
            source,
            limit,
        } => print_json(&store.list_x_items_filtered(
            query.as_deref(),
            source.as_deref(),
            Some(limit),
        )?),
        XSubcommand::Bookmarks { query, limit } => print_json(&store.list_x_items_filtered(
            query.as_deref(),
            Some("bookmark"),
            Some(limit),
        )?),
        XSubcommand::SearchTweets { query, limit } => {
            print_json(&store.search_x_tweets(&query, limit)?)
        }
        XSubcommand::Research { query, limit } => {
            print_json(&store.x_research_brief(&query, limit)?)
        }
        XSubcommand::Thread { x_id, max_depth } => print_json(&store.x_thread(&x_id, max_depth)?),
        XSubcommand::ExtractLinks { limit } => print_json(&store.x_extract_links(limit)?),
        XSubcommand::ExpandLinks { limit } => print_json(&store.x_expand_links(limit)?),
        XSubcommand::Links { query, limit } => print_json(&store.x_links(query.as_deref(), limit)?),
        XSubcommand::RebuildFts => print_json(&store.x_rebuild_fts()?),
        XSubcommand::RepairProjections { limit } => print_json(&store.x_repair_projections(limit)?),
        XSubcommand::Stats => print_json(&store.x_stats()?),
        XSubcommand::Report { query } => print_json(&store.x_report(query.as_deref())?),
    }
}
