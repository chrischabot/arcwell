use crate::*;

#[derive(Debug, Serialize)]
pub(crate) struct GmailOAuthReauthorizeCliReport {
    status: String,
    redirect_uri: String,
    scopes: Vec<String>,
    opened_browser: bool,
    callback_received: bool,
    token_store: Value,
    mailbox_verification: Value,
}

// allow: refactoring this N-arg signature is out of scope for the lint-cleanup pass.
#[allow(clippy::too_many_arguments)]
pub(crate) fn gmail_oauth_reauthorize(
    store: &Store,
    client_id: Option<&str>,
    redirect_uri: Option<&str>,
    client_secret: Option<&str>,
    public_client: bool,
    scopes: &[String],
    timeout_seconds: u64,
    verify_limit: usize,
    open_browser: bool,
) -> Result<GmailOAuthReauthorizeCliReport> {
    if public_client && client_secret.is_some_and(|value| !value.trim().is_empty()) {
        bail!("public Gmail OAuth client mode cannot use a client secret");
    }
    let client_id = store.resolve_gmail_oauth_client_id(client_id)?;
    let redirect_uri = store.resolve_gmail_oauth_redirect_uri(redirect_uri)?;
    let preflight = store.gmail_oauth_reauthorize_preflight(&redirect_uri, scopes)?;
    let loopback = parse_loopback_redirect_uri(&redirect_uri)?;
    let listener = TcpListener::bind(&loopback.bind_addr)
        .with_context(|| format!("binding OAuth callback listener at {}", loopback.bind_addr))?;
    listener
        .set_nonblocking(true)
        .context("configuring OAuth callback listener")?;
    let start = store.gmail_oauth_authorize_url(&client_id, &redirect_uri, &preflight.scopes)?;
    eprintln!(
        "Arcwell Gmail OAuth reauthorize pending: redirect_uri={} scopes={} authorization_endpoint={}",
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
    .with_context(|| {
        gmail_oauth_callback_timeout_context(&start.authorization_url, &redirect_uri)
    })?;
    let token_store = store.gmail_oauth_exchange_code(
        &client_id,
        &redirect_uri,
        &callback.code,
        &start.code_verifier,
        if public_client { None } else { client_secret },
    )?;
    let mailbox_verification = store.verify_email_delivery_mailbox_with_gmail(
        verify_limit,
        Some("mailbox_unverified"),
        None,
        None,
        None,
    )?;
    let status = if mailbox_verification.missing_credential {
        "credential_missing"
    } else if mailbox_verification.errors.is_empty() {
        "ready"
    } else {
        "partial"
    };
    Ok(GmailOAuthReauthorizeCliReport {
        status: status.to_string(),
        redirect_uri,
        scopes: preflight.scopes,
        opened_browser: open_browser,
        callback_received: true,
        token_store: serde_json::to_value(token_store)?,
        mailbox_verification: serde_json::to_value(mailbox_verification)?,
    })
}

pub(crate) fn gmail_oauth_callback_timeout_context(
    authorization_url: &str,
    redirect_uri: &str,
) -> String {
    format!(
        "Timed out waiting for Gmail OAuth callback at {redirect_uri} after opening authorization_endpoint={}. Complete consent in the browser, then retry if the browser did not return to Arcwell.",
        oauth_authorization_endpoint(authorization_url)
    )
}

pub(crate) fn telegram(store: Store, args: TelegramCommand) -> Result<()> {
    match args.command {
        TelegramSubcommand::Drain { max_events } => {
            print_json(&store.drain_telegram_edge_events(max_events)?)
        }
        TelegramSubcommand::Authorize {
            subject,
            read_projects,
            write_projects,
            send,
        } => print_json(&store.authorize_channel_subject(
            "telegram",
            &subject,
            read_projects,
            write_projects,
            send,
        )?),
        TelegramSubcommand::Authorizations => print_json(&store.list_channel_authorizations()?),
        TelegramSubcommand::Deliveries { message_id } => {
            print_json(&store.list_channel_delivery_attempts(message_id.as_deref())?)
        }
        TelegramSubcommand::Send {
            chat_id,
            text,
            bot_token,
            api_base,
        } => {
            cost_preflight(
                &store,
                "arcwell-telegram",
                "telegram",
                Some("telegram_send"),
                0.0001,
                "Telegram send",
            )?;
            let token = telegram_bot_token(&store, bot_token.as_deref())?;
            print_json(&store.send_telegram_message(
                &token,
                &chat_id,
                &text,
                api_base.as_deref(),
            )?)
        }
        TelegramSubcommand::RetryDue {
            bot_token,
            api_base,
            max_attempts,
        } => {
            cost_preflight(
                &store,
                "arcwell-telegram",
                "telegram",
                Some("telegram_retry"),
                0.0001 * max_attempts.clamp(1, 100) as f64,
                "Telegram retry",
            )?;
            let token = telegram_bot_token(&store, bot_token.as_deref())?;
            print_json(&store.retry_due_telegram_deliveries(
                &token,
                api_base.as_deref(),
                max_attempts,
            )?)
        }
    }
}

pub(crate) fn email(store: Store, args: EmailCommand) -> Result<()> {
    match args.command {
        EmailSubcommand::Drain { max_events } => {
            print_json(&store.drain_email_edge_events(max_events)?)
        }
        EmailSubcommand::Poll {
            url,
            secret,
            max_events,
        } => {
            cost_preflight(
                &store,
                "arcwell-edge-inbox",
                "edge",
                Some("edge_remote_drain"),
                0.001 + max_events.clamp(1, 100) as f64 * 0.0001,
                "remote email poll",
            )?;
            let url = edge_remote_url(&store, url.as_deref())?;
            let secret = edge_remote_secret(&store, secret.as_deref())?;
            let remote = store.drain_remote_edge_inbox(&url, &secret, max_events)?;
            let email = store.drain_email_edge_events(max_events)?;
            print_json(&json!({
                "ok": true,
                "remote": remote,
                "email": email
            }))
        }
        EmailSubcommand::Authorize {
            address,
            read_projects,
            write_projects,
            send,
        } => print_json(&store.authorize_channel_subject(
            "email",
            &format!("email:{}", normalize_cli_email(&address)?),
            read_projects,
            write_projects,
            send,
        )?),
        EmailSubcommand::Send {
            to,
            subject,
            text,
            from,
            html,
            account_id,
            api_token,
            api_base,
        } => {
            cost_preflight(
                &store,
                "arcwell-email",
                "cloudflare_email",
                Some("email_send"),
                0.0001,
                "Cloudflare Email send",
            )?;
            let account_id = cloudflare_account_id(&store, account_id.as_deref())?;
            let api_token = cloudflare_api_token(&store, api_token.as_deref())?;
            let from = from
                .as_deref()
                .map(ToOwned::to_owned)
                .or_else(|| agent_email_from(&store).ok())
                .unwrap_or_else(|| "agent@example.com".to_string());
            print_json(&store.send_cloudflare_email(
                &account_id,
                &api_token,
                &from,
                &to,
                &subject,
                &text,
                html.as_deref(),
                None,
                api_base.as_deref(),
            )?)
        }
        EmailSubcommand::Reply {
            message_id,
            text,
            subject,
            html,
            from,
            account_id,
            api_token,
            api_base,
        } => {
            let original = store
                .get_channel_message(&message_id)?
                .with_context(|| format!("channel message not found: {message_id}"))?;
            if original.channel != "email" || original.direction != "incoming" {
                bail!("email reply requires an incoming email channel message");
            }
            let to = email_sender_from_channel_body(&original.body)
                .context("incoming email message does not include a sender")?;
            let original_message_id = email_message_id_from_channel_body(&original.body);
            cost_preflight(
                &store,
                "arcwell-email",
                "cloudflare_email",
                Some("email_send"),
                0.0001,
                "Cloudflare Email reply",
            )?;
            let account_id = cloudflare_account_id(&store, account_id.as_deref())?;
            let api_token = cloudflare_api_token(&store, api_token.as_deref())?;
            let subject = subject.unwrap_or_else(|| "Re: Arcwell".to_string());
            let from = from
                .as_deref()
                .map(ToOwned::to_owned)
                .or_else(|| agent_email_from(&store).ok())
                .unwrap_or_else(|| "agent@example.com".to_string());
            print_json(&store.send_cloudflare_email(
                &account_id,
                &api_token,
                &from,
                &to,
                &subject,
                &text,
                html.as_deref(),
                original_message_id.as_deref(),
                api_base.as_deref(),
            )?)
        }
        EmailSubcommand::ObserveDelivery {
            delivery_attempt_id,
            source,
            status,
            mailbox_message_id,
            provider_message_id,
            observed_at,
            evidence_json,
        } => {
            let evidence: Value = serde_json::from_str(&evidence_json)
                .with_context(|| "parsing --evidence-json as JSON")?;
            print_json(&store.record_channel_delivery_observation(
                &delivery_attempt_id,
                &source,
                &status,
                mailbox_message_id.as_deref(),
                provider_message_id.as_deref(),
                observed_at.as_deref(),
                &evidence,
            )?)
        }
        EmailSubcommand::Observations {
            delivery_attempt_id,
        } => print_json(&store.list_channel_delivery_observations(delivery_attempt_id.as_deref())?),
        EmailSubcommand::VerificationGaps => {
            print_json(&store.list_email_delivery_verification_gaps()?)
        }
        EmailSubcommand::VerificationRequests {
            limit,
            state,
            destination,
        } => print_json(&store.build_email_delivery_verification_requests(
            limit,
            state.as_deref(),
            destination.as_deref(),
        )?),
        EmailSubcommand::EnqueueVerification {
            limit,
            state,
            destination,
        } => print_json(&store.enqueue_email_delivery_verification_request_job(
            limit,
            state.as_deref(),
            destination.as_deref(),
        )?),
        EmailSubcommand::RecoveryPlan {
            limit,
            state,
            destination,
        } => print_json(&store.email_delivery_recovery_plan(
            limit,
            state.as_deref(),
            destination.as_deref(),
        )?),
        EmailSubcommand::VerifyMailbox {
            limit,
            state,
            destination,
            access_token,
            api_base,
        } => print_json(&store.verify_email_delivery_mailbox_with_gmail(
            limit,
            state.as_deref(),
            destination.as_deref(),
            access_token.as_deref(),
            api_base.as_deref(),
        )?),
        EmailSubcommand::RepairMailboxPlacement {
            limit,
            state,
            destination,
            access_token,
            api_base,
        } => print_json(&store.repair_email_delivery_mailbox_placement_with_gmail(
            limit,
            state.as_deref(),
            destination.as_deref(),
            access_token.as_deref(),
            api_base.as_deref(),
        )?),
        EmailSubcommand::OauthUrl {
            client_id,
            redirect_uri,
            scopes,
        } => {
            let client_id = store.resolve_gmail_oauth_client_id(client_id.as_deref())?;
            let redirect_uri = store.resolve_gmail_oauth_redirect_uri(redirect_uri.as_deref())?;
            print_json(&store.gmail_oauth_authorize_url(&client_id, &redirect_uri, &scopes)?)
        }
        EmailSubcommand::OauthExchange {
            client_id,
            redirect_uri,
            code,
            code_verifier,
            client_secret,
            public_client,
        } => {
            if public_client
                && client_secret
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
            {
                bail!("public Gmail OAuth client mode cannot use a client secret");
            }
            let client_id = store.resolve_gmail_oauth_client_id(client_id.as_deref())?;
            let redirect_uri = store.resolve_gmail_oauth_redirect_uri(redirect_uri.as_deref())?;
            print_json(&store.gmail_oauth_exchange_code(
                &client_id,
                &redirect_uri,
                &code,
                &code_verifier,
                if public_client {
                    None
                } else {
                    client_secret.as_deref()
                },
            )?)
        }
        EmailSubcommand::OauthReauthorize {
            client_id,
            redirect_uri,
            client_secret,
            public_client,
            scopes,
            timeout_seconds,
            verify_limit,
            no_open_browser,
        } => print_json(&gmail_oauth_reauthorize(
            &store,
            client_id.as_deref(),
            redirect_uri.as_deref(),
            client_secret.as_deref(),
            public_client,
            &scopes,
            timeout_seconds,
            verify_limit,
            !no_open_browser,
        )?),
        EmailSubcommand::OauthRefresh {
            client_id,
            client_secret,
            public_client,
        } => {
            if public_client
                && client_secret
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
            {
                bail!("public Gmail OAuth client mode cannot use a client secret");
            }
            let client_id = store.resolve_gmail_oauth_client_id(client_id.as_deref())?;
            print_json(&store.gmail_oauth_refresh(
                &client_id,
                if public_client {
                    None
                } else {
                    client_secret.as_deref()
                },
            )?)
        }
        EmailSubcommand::ObserveDeliveryBatch {
            source,
            results_json,
        } => {
            let results: Vec<Value> = serde_json::from_str(&results_json)
                .with_context(|| "parsing --results-json as a JSON array")?;
            let mut observations = Vec::new();
            for result in results {
                let delivery_attempt_id = result
                    .get("delivery_attempt_id")
                    .and_then(Value::as_str)
                    .context("batch result missing delivery_attempt_id")?;
                let observation_status = result
                    .get("observation_status")
                    .and_then(Value::as_str)
                    .unwrap_or("mailbox_observed");
                let observation_source = result
                    .get("observation_source")
                    .and_then(Value::as_str)
                    .unwrap_or(&source);
                let mailbox_message_id = result.get("mailbox_message_id").and_then(Value::as_str);
                let provider_message_id = result.get("provider_message_id").and_then(Value::as_str);
                let observed_at = result.get("observed_at").and_then(Value::as_str);
                let evidence = result.get("evidence").cloned().unwrap_or_else(|| json!({}));
                observations.push(store.record_channel_delivery_observation(
                    delivery_attempt_id,
                    observation_source,
                    observation_status,
                    mailbox_message_id,
                    provider_message_id,
                    observed_at,
                    &evidence,
                )?);
            }
            print_json(&observations)
        }
    }
}

pub(crate) fn edge(store: Store, args: EdgeCommand) -> Result<()> {
    match args.command {
        EdgeSubcommand::DrainRemote {
            url,
            secret,
            max_events,
        } => {
            cost_preflight(
                &store,
                "arcwell-edge-inbox",
                "edge",
                Some("edge_remote_drain"),
                0.001 + max_events.clamp(1, 100) as f64 * 0.0001,
                "remote edge drain",
            )?;
            let url = edge_remote_url(&store, url.as_deref())?;
            let secret = edge_remote_secret(&store, secret.as_deref())?;
            print_json(&store.drain_remote_edge_inbox(&url, &secret, max_events)?)
        }
    }
}

pub(crate) fn project(store: Store, args: ProjectCommand) -> Result<()> {
    match args.command {
        ProjectSubcommand::Create {
            name,
            summary,
            aliases,
        } => print_json(&store.create_project(&name, &summary, &aliases)?),
        ProjectSubcommand::List => print_json(&store.list_projects()?),
        ProjectSubcommand::Resolve {
            query,
            context_project_id,
        } => print_json(&store.resolve_project(&query, context_project_id.as_deref())?),
        ProjectSubcommand::StatusRecord {
            project_id,
            status,
            summary,
            source,
            thread_ref,
            confidence,
        } => print_json(&store.record_project_status(
            &project_id,
            &status,
            &summary,
            &source,
            thread_ref.as_deref(),
            confidence,
        )?),
        ProjectSubcommand::StatusSyncRecord {
            project_id,
            status,
            summary,
            host,
            thread_id,
            confidence,
            stale_after_seconds,
        } => print_json(&store.record_verified_project_status_sync(
            &project_id,
            &status,
            &summary,
            &host,
            &thread_id,
            confidence,
            stale_after_seconds,
        )?),
        ProjectSubcommand::StatusGet {
            project_id,
            channel,
            subject,
        } => print_json(&store.project_status_report_for_channel(
            &project_id,
            channel.as_deref(),
            subject.as_deref(),
        )?),
    }
}

pub(crate) fn controller(store: Store, args: ControllerCommand) -> Result<()> {
    match args.command {
        ControllerSubcommand::Route {
            channel,
            account_id,
            conversation_id,
            sender,
            text,
        } => print_json(&store.controller_route_text(
            &channel,
            account_id.as_deref(),
            &conversation_id,
            &sender,
            &text,
        )?),
        ControllerSubcommand::ThreadUpsert {
            host,
            host_thread_id,
            project_id,
            title,
            cwd,
            branch,
            worktree,
            status,
            active,
            archived,
            current_goal,
            latest_summary,
            latest_summary_source,
            last_activity_at,
        } => print_json(&store.upsert_controller_thread(
            &host,
            &host_thread_id,
            project_id.as_deref(),
            title.as_deref(),
            cwd.as_deref(),
            branch.as_deref(),
            worktree.as_deref(),
            &status,
            active,
            archived,
            current_goal.as_deref(),
            latest_summary.as_deref(),
            latest_summary_source.as_deref(),
            last_activity_at.as_deref(),
        )?),
        ControllerSubcommand::Threads {
            project_id,
            status,
            limit,
        } => print_json(&store.list_controller_threads(
            project_id.as_deref(),
            status.as_deref(),
            limit,
        )?),
        ControllerSubcommand::ThreadGet { id } => print_json(
            &store
                .get_controller_thread(&id)?
                .with_context(|| format!("controller thread not found: {id}"))?,
        ),
        ControllerSubcommand::RunCreate {
            thread_id,
            project_id,
            origin_channel_message_id,
            host,
            host_run_id,
            kind,
            status,
            requested_action,
        } => print_json(&store.create_controller_run(
            thread_id.as_deref(),
            project_id.as_deref(),
            origin_channel_message_id.as_deref(),
            &host,
            host_run_id.as_deref(),
            &kind,
            &status,
            &requested_action,
        )?),
        ControllerSubcommand::Runs {
            project_id,
            status,
            limit,
        } => print_json(&store.list_controller_runs(
            project_id.as_deref(),
            status.as_deref(),
            limit,
        )?),
        ControllerSubcommand::RunGet { id } => print_json(
            &store
                .get_controller_run(&id)?
                .with_context(|| format!("controller run not found: {id}"))?,
        ),
        ControllerSubcommand::RunUpdate {
            run_id,
            status,
            host_run_id,
        } => print_json(&store.update_controller_run_status(
            &run_id,
            &status,
            host_run_id.as_deref(),
        )?),
        ControllerSubcommand::Stop { run_id, reason } => {
            print_json(&store.request_controller_stop(&run_id, &reason)?)
        }
        ControllerSubcommand::Event {
            run_id,
            thread_id,
            project_id,
            event_type,
            summary,
            data,
            source,
        } => {
            let data: Value = serde_json::from_str(&data).context("--data must be JSON")?;
            print_json(&store.record_controller_event(
                run_id.as_deref(),
                thread_id.as_deref(),
                project_id.as_deref(),
                &event_type,
                &summary,
                data,
                &source,
            )?)
        }
        ControllerSubcommand::Events {
            run_id,
            project_id,
            limit,
        } => print_json(&store.list_controller_events(
            run_id.as_deref(),
            project_id.as_deref(),
            limit,
        )?),
        ControllerSubcommand::Pending { status, limit } => {
            print_json(&store.list_controller_pending_actions(status.as_deref(), limit)?)
        }
        ControllerSubcommand::PendingResolve {
            id,
            status,
            thread_id,
            run_id,
        } => print_json(&store.resolve_controller_pending_action(
            &id,
            &status,
            thread_id.as_deref(),
            run_id.as_deref(),
        )?),
    }
}

pub(crate) fn work(store: Store, args: WorkCommand) -> Result<()> {
    match args.command {
        WorkSubcommand::Start {
            goal,
            project_id,
            host_id,
            thread_id,
            agent_surface,
        } => print_json(&store.start_work_run(
            &goal,
            project_id.as_deref(),
            host_id.as_deref(),
            thread_id.as_deref(),
            &agent_surface,
        )?),
        WorkSubcommand::Event {
            run_id,
            event_type,
            summary,
            data,
        } => {
            let data: Value = serde_json::from_str(&data).context("--data must be JSON")?;
            print_json(&store.record_work_event(&run_id, &event_type, &summary, data)?)
        }
        WorkSubcommand::ArtifactAdd {
            run_id,
            artifact_type,
            locator,
            role,
            metadata,
        } => {
            let metadata: Value =
                serde_json::from_str(&metadata).context("--metadata must be JSON")?;
            print_json(&store.add_work_artifact(
                &run_id,
                &artifact_type,
                &locator,
                &role,
                metadata,
            )?)
        }
        WorkSubcommand::LinkAdd {
            run_id,
            target_type,
            target_id,
            role,
            generated_summary,
        } => print_json(&store.add_work_link(
            &run_id,
            &target_type,
            &target_id,
            &role,
            generated_summary,
        )?),
        WorkSubcommand::Finish {
            run_id,
            status,
            outcome,
            validation_summary,
            follow_ups,
            reusable_lessons,
        } => print_json(&store.finish_work_run(
            &run_id,
            &status,
            &outcome,
            validation_summary.as_deref(),
            &follow_ups,
            &reusable_lessons,
        )?),
        WorkSubcommand::Search {
            query,
            project_id,
            status,
            limit,
        } => print_json(&store.search_work_runs(
            query.as_deref(),
            project_id.as_deref(),
            status.as_deref(),
            limit,
        )?),
        WorkSubcommand::Read { run_id } => print_json(&store.read_work_run(&run_id)?),
        WorkSubcommand::Stale {
            max_age_days,
            limit,
        } => print_json(&store.list_stale_work_runs(max_age_days, limit)?),
        WorkSubcommand::FollowUps { limit } => print_json(&store.list_work_follow_ups(limit)?),
        WorkSubcommand::ConsolidationCandidates { limit } => {
            print_json(&store.list_work_consolidation_candidates(limit)?)
        }
        WorkSubcommand::RetrievalContext {
            query,
            stale_after_days,
            limit,
        } => print_json(&store.work_retrieval_context(&query, stale_after_days, limit)?),
        WorkSubcommand::Consolidate {
            run_id,
            write_project_status,
        } => print_json(&store.consolidate_work_run(&run_id, write_project_status)?),
    }
}

pub(crate) fn procedure(store: Store, args: ProcedureCommand) -> Result<()> {
    match args.command {
        ProcedureSubcommand::Propose {
            run_id,
            auto_approve,
        } => print_json(&store.propose_procedure_from_work_run(&run_id, auto_approve)?),
        ProcedureSubcommand::Candidate {
            operation,
            procedure_id,
            base_version,
            title,
            method,
            source_run_id,
            sensitivity,
            reason,
            trigger_context,
            problem,
            precondition,
            tool,
            validation_command,
            known_risk,
        } => print_json(&store.create_procedure_candidate(ProcedureCandidateInput {
            operation,
            procedure_id,
            base_version,
            title,
            trigger_context: if trigger_context.trim().is_empty() {
                "Manual procedure candidate".to_string()
            } else {
                trigger_context
            },
            problem: if problem.trim().is_empty() {
                "Manual procedure candidate".to_string()
            } else {
                problem
            },
            preconditions: precondition,
            method,
            tools: tool,
            validation_commands: validation_command,
            known_risks: known_risk,
            source_run_ids: source_run_id,
            provenance: json!({ "source": "manual-cli" }),
            sensitivity,
            reason,
        })?),
        ProcedureSubcommand::Candidates { status } => {
            print_json(&store.list_procedure_candidates(&status)?)
        }
        ProcedureSubcommand::Apply { id } => print_json(&store.approve_procedure_candidate(&id)?),
        ProcedureSubcommand::Reject { id, reason } => print_json(
            &json!({ "ok": store.reject_procedure_candidate(&id, reason.as_deref())?, "id": id, "status": "rejected" }),
        ),
        ProcedureSubcommand::Search {
            query,
            status,
            limit,
        } => print_json(&store.search_procedures(query.as_deref(), Some(&status), limit)?),
        ProcedureSubcommand::Read { id } => print_json(&store.read_procedure(&id)?),
        ProcedureSubcommand::RetrievalContext { query, limit } => {
            print_json(&store.procedure_retrieval_context(&query, limit)?)
        }
        ProcedureSubcommand::ExportSkill { id, skill_name } => {
            print_json(&store.export_procedure_to_codex_skill(&id, &skill_name)?)
        }
        ProcedureSubcommand::Curate => print_json(&store.curate_procedures()?),
    }
}

pub(crate) fn cost_preflight(
    store: &Store,
    package: &str,
    provider: &str,
    source: Option<&str>,
    projected_usd: f64,
    label: &str,
) -> Result<()> {
    let decision = store.cost_decision(package, provider, source, projected_usd)?;
    if !decision.allowed {
        bail!("budget blocked {label}: {}", decision.reason);
    }
    Ok(())
}

pub(crate) fn edge_base_from_webhook_url(url: String) -> String {
    url.trim_end_matches("/telegram/webhook").to_string()
}

pub(crate) fn edge_remote_url(store: &Store, explicit: Option<&str>) -> Result<String> {
    explicit
        .map(ToOwned::to_owned)
        .or_else(|| std::env::var("ARCWELL_EDGE_URL").ok())
        .or_else(|| {
            std::env::var("TELEGRAM_WEBHOOK_URL")
                .ok()
                .map(edge_base_from_webhook_url)
        })
        .or_else(|| store.get_secret_value("ARCWELL_EDGE_URL").ok().flatten())
        .context("ARCWELL_EDGE_URL or --url is required")
}

pub(crate) fn edge_remote_secret(store: &Store, explicit: Option<&str>) -> Result<String> {
    explicit
        .map(ToOwned::to_owned)
        .or_else(|| std::env::var("ARCWELL_EDGE_SECRET").ok())
        .or_else(|| store.get_secret_value("ARCWELL_EDGE_SECRET").ok().flatten())
        .context("ARCWELL_EDGE_SECRET or --secret is required")
}

pub(crate) fn telegram_bot_token(store: &Store, explicit: Option<&str>) -> Result<String> {
    explicit
        .map(ToOwned::to_owned)
        .or_else(|| std::env::var("TELEGRAM_BOT_TOKEN").ok())
        .or_else(|| store.get_secret_value("TELEGRAM_BOT_TOKEN").ok().flatten())
        .context("TELEGRAM_BOT_TOKEN is required")
}

pub(crate) fn cloudflare_account_id(store: &Store, explicit: Option<&str>) -> Result<String> {
    explicit
        .map(ToOwned::to_owned)
        .or_else(|| std::env::var("CLOUDFLARE_ACCOUNT_ID").ok())
        .or_else(|| {
            store
                .get_secret_value("CLOUDFLARE_ACCOUNT_ID")
                .ok()
                .flatten()
        })
        .context("CLOUDFLARE_ACCOUNT_ID is required")
}

pub(crate) fn cloudflare_api_token(store: &Store, explicit: Option<&str>) -> Result<String> {
    explicit
        .map(ToOwned::to_owned)
        .or_else(|| std::env::var("CLOUDFLARE_EMAIL_API_TOKEN").ok())
        .or_else(|| std::env::var("CLOUDFLARE_API_TOKEN").ok())
        .or_else(|| {
            store
                .get_secret_value("CLOUDFLARE_EMAIL_API_TOKEN")
                .ok()
                .flatten()
        })
        .or_else(|| {
            store
                .get_secret_value("CLOUDFLARE_API_TOKEN")
                .ok()
                .flatten()
        })
        .context("CLOUDFLARE_EMAIL_API_TOKEN or CLOUDFLARE_API_TOKEN is required")
}

pub(crate) fn agent_email_from(store: &Store) -> Result<String> {
    std::env::var("ARCWELL_AGENT_EMAIL_FROM")
        .ok()
        .or_else(|| std::env::var("ARCWELL_AGENT_EMAIL").ok())
        .or_else(|| {
            store
                .get_secret_value("ARCWELL_AGENT_EMAIL_FROM")
                .ok()
                .flatten()
        })
        .or_else(|| store.get_secret_value("ARCWELL_AGENT_EMAIL").ok().flatten())
        .context("ARCWELL_AGENT_EMAIL_FROM or ARCWELL_AGENT_EMAIL is required")
}

pub(crate) fn normalize_cli_email(value: &str) -> Result<String> {
    let value = value
        .trim()
        .trim_matches(['<', '>', '"', '\''])
        .to_ascii_lowercase();
    if value.len() > 254 || value.matches('@').count() != 1 {
        bail!("invalid email address");
    }
    let (local, domain) = value
        .split_once('@')
        .context("email address must include @")?;
    if local.is_empty() || domain.is_empty() {
        bail!("invalid email address");
    }
    Ok(value)
}

pub(crate) fn email_sender_from_channel_body(body: &str) -> Option<String> {
    body.lines()
        .find_map(|line| line.strip_prefix("From: "))
        .map(str::trim)
        .and_then(|value| normalize_cli_email(value).ok())
}

pub(crate) fn email_message_id_from_channel_body(body: &str) -> Option<String> {
    body.lines()
        .find_map(|line| line.strip_prefix("Message-ID: "))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}
