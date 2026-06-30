use super::*;

#[test]
fn severe_mcp_unknown_tool_returns_error() {
    let paths = test_paths("unknown-tool");
    let error = call_mcp_tool(&paths, "please_escalate_me", json!({}))
        .expect_err("unknown tool must not succeed");
    assert!(error.to_string().contains("unknown tool"));
}

#[test]
fn severe_mcp_missing_required_argument_returns_error() {
    let paths = test_paths("missing-arg");
    let error = call_mcp_tool(&paths, "memory_add", json!({}))
        .expect_err("missing required argument must not succeed");
    assert!(error.to_string().contains("missing string argument"));
}

#[test]
fn severe_mcp_profile_set_uses_parameterized_storage() {
    let paths = test_paths("mcp-injection");
    call_mcp_tool(
        &paths,
        "profile_set",
        json!({
            "key": "x'); DROP TABLE profile_items; --",
            "value": "payload"
        }),
    )
    .unwrap();

    let result = call_mcp_tool(&paths, "profile_list", json!({})).unwrap();
    assert_eq!(result.as_array().unwrap().len(), 1);
}

#[test]
fn severe_mcp_tool_call_structured_content_is_object_for_list_results() {
    let paths = test_paths("mcp-structured-content-list");
    call_mcp_tool(
        &paths,
        "profile_set",
        json!({
            "key": "proof.profile",
            "value": "list-shaped results must be wrapped for Claude Code"
        }),
    )
    .unwrap();

    let result = dispatch_mcp(
        &paths,
        "tools/call",
        json!({
            "name": "profile_list",
            "arguments": {}
        }),
    )
    .unwrap();

    assert!(result["structuredContent"].is_object());
    assert!(result["structuredContent"]["result"].is_array());
    assert_eq!(
        result["structuredContent"]["result"][0]["key"].as_str(),
        Some("proof.profile")
    );
}

#[test]
fn severe_mcp_ops_snapshot_is_compact_for_tool_clients() {
    let paths = test_paths("mcp-compact-ops");
    call_mcp_tool(
        &paths,
        "source_card_add",
        json!({
            "title": "Compact ops fixture",
            "url": "https://example.com/compact-ops",
            "provider": "test",
            "source_kind": "fixture",
            "summary": "Compact ops fixture",
            "raw_text": "This card makes ops contain a list-shaped source_cards entry."
        }),
    )
    .unwrap();

    let result = dispatch_mcp(
        &paths,
        "tools/call",
        json!({
            "name": "ops_snapshot",
            "arguments": {}
        }),
    )
    .unwrap();

    let structured = &result["structuredContent"];
    assert!(structured.is_object());
    assert_eq!(
        structured["summary"].as_str(),
        Some("Compact MCP ops snapshot. Use `arcwell ops` for the full local JSON payload.")
    );
    assert!(structured["health"].is_object());
    assert!(structured["counts"]["source_cards"].as_u64().unwrap_or(0) >= 1);
    assert!(structured.get("source_cards").is_none());
}

#[test]
fn severe_mcp_secret_resources_and_ops_never_expose_values() {
    let paths = test_paths("mcp-secret-redaction");
    let token = format!("sk-{}", "d".repeat(48));
    let expired = (chrono::Utc::now() - chrono::Duration::seconds(30)).to_rfc3339();
    call_mcp_tool(
        &paths,
        "secret_value_set",
        json!({
            "name": "X_BEARER_TOKEN",
            "value": token.clone(),
            "scope": "x",
            "provider": "x",
            "expires_at": expired
        }),
    )
    .unwrap();

    let values = call_mcp_tool(&paths, "secret_value_list", json!({})).unwrap();
    let health = call_mcp_tool(&paths, "secret_health", json!({})).unwrap();
    let ops = call_mcp_tool(&paths, "ops_snapshot", json!({})).unwrap();
    let resource_values = dispatch_mcp(
        &paths,
        "resources/read",
        json!({ "uri": "arcwell://secret-values" }),
    )
    .unwrap();
    let resource_health = dispatch_mcp(
        &paths,
        "resources/read",
        json!({ "uri": "arcwell://secret-health" }),
    )
    .unwrap();
    let serialized = serde_json::to_string(&json!({
        "values": values,
        "health": health,
        "ops": ops,
        "resource_values": resource_values,
        "resource_health": resource_health,
    }))
    .unwrap();
    assert!(serialized.contains("X_BEARER_TOKEN"));
    assert!(serialized.contains("expired"));
    assert!(!serialized.contains(&token));
}

#[test]
fn severe_mcp_policy_admin_and_secret_denial_round_trip() {
    // CLAIM: MCP exposes policy admin tools and secret mutation tools enforce policy before SQLite writes.
    // ORACLE: policy_check records a denial, secret_value_set fails with that denial, and no secret value exists.
    // SEVERITY: Severe because MCP is an agent-facing boundary for policy and credential administration.
    let paths = test_paths("mcp-policy-admin");
    fs::create_dir_all(&paths.home).unwrap();
    fs::write(
        paths.home.join("arcwell-policy.toml"),
        r#"
[[rules]]
id = "deny-mcp-secret"
effect = "deny"
action = "secret.write"
source = "mcp"
target = "BLOCKED_TOKEN"
reason = "MCP secret writes are denied for this token"
"#,
    )
    .unwrap();

    let tools = mcp_tools();
    let tool_names: BTreeSet<_> = tools
        .iter()
        .filter_map(|tool| tool.get("name").and_then(Value::as_str))
        .collect();
    assert!(tool_names.contains("policy_check"));
    assert!(tool_names.contains("policy_explain"));
    assert!(tool_names.contains("policy_override_allow"));
    assert!(tool_names.contains("policy_approval_approve"));
    assert!(tool_names.contains("policy_approval_reject"));

    let decision = call_mcp_tool(
        &paths,
        "policy_check",
        json!({
            "action": "secret.write",
            "source": "mcp",
            "target": "BLOCKED_TOKEN"
        }),
    )
    .unwrap();
    assert_eq!(decision["effect"], "deny");
    assert_eq!(decision["matched_rule_id"], "deny-mcp-secret");

    let error = call_mcp_tool(
        &paths,
        "secret_value_set",
        json!({
            "name": "BLOCKED_TOKEN",
            "value": "blocked-secret-value",
            "scope": "local"
        }),
    )
    .expect_err("denied MCP secret write must fail before mutation")
    .to_string();
    assert!(error.contains("policy denied secret.write"), "{error}");
    assert!(!error.contains("blocked-secret-value"), "{error}");

    let values = call_mcp_tool(&paths, "secret_value_list", json!({})).unwrap();
    assert_eq!(values.as_array().unwrap().len(), 0);
    let decisions = call_mcp_tool(&paths, "policy_decision_list", json!({ "limit": 10 })).unwrap();
    assert!(
        decisions.as_array().unwrap().iter().any(|decision| {
            decision["action"] == "secret.write" && decision["effect"] == "deny"
        })
    );
}

#[test]
fn severe_cli_redacts_command_echo_and_failed_provider_tokens() {
    let token = format!("ghp_{}", "e".repeat(48));
    let message = format!(
        "provider failed access_token={token}&refresh_token={} Authorization: Bearer {token}",
        "f".repeat(48)
    );
    let redacted = redact_secret_like_text(&message);
    assert!(!redacted.contains(&token));
    assert!(!redacted.contains(&"f".repeat(48)));
    assert!(redacted.contains("[REDACTED]"));
}

#[test]
fn mcp_ping_and_template_probe_are_supported() {
    let paths = test_paths("mcp-probes");
    assert_eq!(dispatch_mcp(&paths, "ping", json!({})).unwrap(), json!({}));
    assert_eq!(
        dispatch_mcp(&paths, "resources/templates/list", json!({})).unwrap(),
        json!({ "resourceTemplates": [] })
    );
    assert_eq!(
        dispatch_mcp(&paths, "prompts/list", json!({})).unwrap(),
        json!({ "prompts": [] })
    );
}

#[test]
fn mcp_research_workflow_round_trip() {
    let paths = test_paths("mcp-research-workflow");
    let workflow = call_mcp_tool(
        &paths,
        "research_workflow_create",
        json!({ "query": "agent monitors" }),
    )
    .unwrap();
    let run_id = workflow
        .get("run")
        .and_then(|run| run.get("id"))
        .and_then(Value::as_str)
        .unwrap();
    let tasks = call_mcp_tool(&paths, "research_tasks", json!({ "run_id": run_id })).unwrap();
    assert_eq!(tasks.as_array().unwrap().len(), 7);
}

#[test]
fn mcp_research_deep_run_lifecycle_round_trip() {
    let paths = test_paths("mcp-research-deep-run");
    let workflow =
        call_mcp_tool(&paths, "research_run", json!({ "query": "agent monitors" })).unwrap();
    let run_id = workflow
        .get("run")
        .and_then(|run| run.get("id"))
        .and_then(Value::as_str)
        .unwrap();
    assert_eq!(
        workflow
            .get("run")
            .and_then(|run| run.get("status"))
            .and_then(Value::as_str),
        Some("deep_open")
    );
    assert_eq!(workflow["tasks"].as_array().unwrap().len(), 7);

    let status = call_mcp_tool(&paths, "research_status", json!({ "run_id": run_id })).unwrap();
    assert_eq!(status["task_count"].as_u64(), Some(7));
    assert_eq!(status["pending_task_count"].as_u64(), Some(7));

    let read = call_mcp_tool(&paths, "research_read", json!({ "run_id": run_id })).unwrap();
    assert_eq!(read["run"]["id"].as_str(), Some(run_id));
    assert_eq!(read["tasks"].as_array().unwrap().len(), 7);

    let audit = call_mcp_tool(&paths, "research_audit_run", json!({ "run_id": run_id })).unwrap();
    assert_eq!(audit["run"]["id"].as_str(), Some(run_id));
    assert_eq!(audit["audit"]["query"].as_str(), Some("agent monitors"));

    let stopped = call_mcp_tool(&paths, "research_stop", json!({ "run_id": run_id })).unwrap();
    assert_eq!(stopped["run"]["status"].as_str(), Some("stopped"));
    assert_eq!(stopped["pending_task_count"].as_u64(), Some(0));
    assert_eq!(stopped["cancelled_task_count"].as_u64(), Some(7));
}

#[test]
fn severe_mcp_research_convergence_loop_is_agent_callable_and_inspectable() {
    // CLAIM: agents can invoke and inspect the full convergence loop through MCP, not only internal Rust APIs.
    // ORACLE: source card, claims, convergence, ledgers, status, report, and judgment all round-trip via call_mcp_tool.
    // SEVERITY: Severe because prior failures came from capabilities existing in code but not usable by agents.
    let paths = test_paths("mcp-research-convergence");
    let workflow = call_mcp_tool(
        &paths,
        "research_run",
        json!({ "query": "deterministic sandbox verification" }),
    )
    .unwrap();
    let run_id = workflow["run"]["id"].as_str().unwrap();
    let source = call_mcp_tool(
            &paths,
            "source_card_add",
            json!({
                "run_id": run_id,
                "title": "Sandbox verification note",
                "url": "https://example.com/sandbox-verification-note",
                "source_type": "paper",
                "provider": "test",
                "summary": "The sandbox requires deterministic verification before untrusted execution.",
                "source_family": "papers",
                "metadata": { "source_role": "primary", "trust_level": "high" }
            }),
        )
        .unwrap();
    let source_card_id = source["source_card"]["id"].as_str().unwrap();
    call_mcp_tool(
            &paths,
            "research_claims_ingest",
            json!({
                "run_id": run_id,
                "source_card_id": source_card_id,
                "provider": "test",
                "model": "fixture",
                "output_json": r#"{"claims":[{
                    "text":"The sandbox requires deterministic verification before untrusted execution.",
                    "kind":"fact",
                    "subject":"the sandbox",
                    "predicate":"requires",
                    "object":"deterministic verification before untrusted execution",
                    "confidence":0.88,
                    "caveats":["Fixture source only."],
                    "quote":"requires deterministic verification"
                }]}"#
            }),
        )
        .unwrap();

    let converged = call_mcp_tool(
        &paths,
        "research_convergence_run",
        json!({
            "run_id": run_id,
            "max_iterations": 3,
            "no_progress_iteration_limit": 1
        }),
    )
    .unwrap();
    assert_eq!(converged["status"]["settled"].as_bool(), Some(true));
    assert_eq!(
        converged["snapshot"]["stop_rule"]["stop_reason"].as_str(),
        Some("settled")
    );

    for tool in [
        "research_iterations",
        "research_statements",
        "research_challenges",
        "research_convergence_host_search_tasks",
        "research_disproofs",
        "research_fact_checks",
        "research_convergence_snapshots",
    ] {
        let value = call_mcp_tool(&paths, tool, json!({ "run_id": run_id })).unwrap();
        assert!(
            !value.as_array().unwrap().is_empty(),
            "{tool} returned no convergence records"
        );
    }
    let status = call_mcp_tool(
        &paths,
        "research_convergence_status",
        json!({ "run_id": run_id }),
    )
    .unwrap();
    assert_eq!(status["settled"].as_bool(), Some(true));
    assert!(
        status["host_search_tasks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|task| task["status"].as_str() == Some("pending"))
    );
    let host_search_tasks = call_mcp_tool(
        &paths,
        "research_convergence_host_search_tasks",
        json!({ "run_id": run_id }),
    )
    .unwrap();
    let task = host_search_tasks
        .as_array()
        .unwrap()
        .iter()
        .find(|task| task["status"].as_str() == Some("pending"))
        .expect("convergence should expose pending host-search tasks")
        .clone();
    let task_id = task["id"].as_str().unwrap();
    let task_query = task["query"].as_str().unwrap();
    let recorded_search = call_mcp_tool(
        &paths,
        "research_host_search_record",
        json!({
            "run_id": run_id,
            "host": "codex",
            "tool_surface": "web.run",
            "query": task_query,
            "query_intent": "Resolve exact convergence host-search task.",
            "results": [{
                "rank": 1,
                "title": "Sandbox verification official note",
                "url": "https://example.com/sandbox/official-verification",
                "snippet": "Official note corroborates deterministic verification.",
                "source_family_guess": "primary",
                "selected_for_ingest": true
            }]
        }),
    )
    .unwrap();
    let recorded_tasks = call_mcp_tool(
        &paths,
        "research_convergence_host_search_tasks",
        json!({ "run_id": run_id }),
    )
    .unwrap();
    assert!(recorded_tasks.as_array().unwrap().iter().any(|task| {
        task["id"].as_str() == Some(task_id)
            && task["status"].as_str() == Some("recorded")
            && task["matched_host_search_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|id| id.as_str() == recorded_search["search"]["id"].as_str())
            && task["research_source_ids"].as_array().unwrap().len() == 1
    }));
    let report = call_mcp_tool(
        &paths,
        "research_convergence_report_compile",
        json!({ "run_id": run_id }),
    )
    .unwrap();
    assert_eq!(
        report["judgment"]["overall_decision"].as_str(),
        Some("accept_with_caveats")
    );
    assert!(
        report["artifact"]["body"]
            .as_str()
            .unwrap()
            .contains("Pressure-Test Results")
    );
    let judgments = call_mcp_tool(
        &paths,
        "research_report_judgments",
        json!({ "run_id": run_id }),
    )
    .unwrap();
    assert_eq!(judgments.as_array().unwrap().len(), 1);

    let queued = call_mcp_tool(
        &paths,
        "research_convergence_enqueue",
        json!({ "run_id": run_id, "max_iterations": 3, "no_progress_iteration_limit": 1 }),
    )
    .unwrap();
    assert_eq!(queued["kind"].as_str(), Some("research_convergence_run"));
    let worker = call_mcp_tool(&paths, "worker_run_once", json!({ "max_jobs": 1 })).unwrap();
    assert_eq!(worker["completed"].as_u64(), Some(1));
    assert_eq!(
        worker["jobs"][0]["result_json"]["action"].as_str(),
        Some("already_terminal")
    );
}

#[test]
fn severe_mcp_research_convergence_model_editorial_gate_round_trips() {
    // CLAIM: agents can request the model-backed convergence editorial/evaluator loop through MCP.
    // ORACLE: the convergence result exposes editorial stage outputs and the persisted judgment/evidence can be read back.
    // SEVERITY: Severe because schema-only exposure is insufficient for long-running Codex research orchestration.
    let paths = test_paths("mcp-research-convergence-editorial");
    let workflow = call_mcp_tool(
        &paths,
        "research_run",
        json!({ "query": "research deterministic code sandbox verification" }),
    )
    .unwrap();
    let run_id = workflow["run"]["id"].as_str().unwrap();
    let source = call_mcp_tool(
            &paths,
            "source_card_add",
            json!({
                "run_id": run_id,
                "title": "Deterministic sandbox verification note",
                "url": "https://example.com/deterministic-sandbox-verification",
                "source_type": "paper",
                "provider": "test",
                "summary": "Deterministic verification is required before untrusted code execution in the sandbox.",
                "source_family": "papers",
                "metadata": { "source_role": "primary", "trust_level": "high" }
            }),
        )
        .unwrap();
    let source_card_id = source["source_card"]["id"].as_str().unwrap();
    call_mcp_tool(
            &paths,
            "research_claims_ingest",
            json!({
                "run_id": run_id,
                "source_card_id": source_card_id,
                "provider": "test",
                "model": "fixture",
                "output_json": r#"{"claims":[{
                    "text":"Deterministic verification is required before untrusted code execution in the sandbox.",
                    "kind":"fact",
                    "subject":"deterministic verification",
                    "predicate":"is required before",
                    "object":"untrusted code execution in the sandbox",
                    "confidence":0.91,
                    "caveats":["Fixture source only."],
                    "quote":"Deterministic verification is required"
                }]}"#
            }),
        )
        .unwrap();

    let converged = call_mcp_tool(
        &paths,
        "research_convergence_run",
        json!({
            "run_id": run_id,
            "max_iterations": 3,
            "no_progress_iteration_limit": 1,
            "editorial_provider": "mock",
            "max_provider_calls": 2
        }),
    )
    .unwrap();
    assert_eq!(converged["status"]["settled"].as_bool(), Some(true));
    assert_eq!(converged["editorial"]["status"].as_str(), Some("accepted"));
    assert_eq!(
        converged["editorial"]["citation_verifier"]["editorial_run"]["stage"].as_str(),
        Some("citation_verifier")
    );
    assert_eq!(
        converged["editorial"]["adversarial_evaluator"]["editorial_run"]["stage"].as_str(),
        Some("adversarial_evaluator")
    );

    let editorial_runs = call_mcp_tool(
        &paths,
        "research_editorial_runs",
        json!({ "run_id": run_id }),
    )
    .unwrap();
    assert_eq!(editorial_runs.as_array().unwrap().len(), 2);
    assert!(editorial_runs.as_array().unwrap().iter().all(|run| {
        run["status"].as_str() == Some("completed")
            && run["output_artifact_id"]
                .as_str()
                .is_some_and(|id| !id.is_empty())
    }));

    let judgments = call_mcp_tool(
        &paths,
        "research_report_judgments",
        json!({ "run_id": run_id }),
    )
    .unwrap();
    let gate = judgments
        .as_array()
        .unwrap()
        .iter()
        .find_map(|judgment| judgment["scores"].get("model_backed_convergence_editorial"))
        .expect("model-backed convergence editorial judgment must be present");
    assert_eq!(gate["accepted"].as_bool(), Some(true));
    assert_eq!(
        gate["citation_verifier"]["status"].as_str(),
        Some("completed")
    );
    assert_eq!(
        gate["adversarial_evaluator"]["status"].as_str(),
        Some("completed")
    );
}

#[test]
fn severe_mcp_deep_research_schemas_expose_agent_usable_fields() {
    // CLAIM: MCP discovery exposes the fields an in-app Codex agent needs
    // for deep research without falling back to CLI spelunking.
    // ORACLE: JSON schema properties for the exact logged failure surfaces.
    // SEVERITY: Severe because a thin schema caused real agent misuse in logs.
    let tools = mcp_tools();
    let find_tool = |name: &str| {
        tools
            .iter()
            .find(|tool| tool.get("name").and_then(Value::as_str) == Some(name))
            .unwrap_or_else(|| panic!("missing tool {name}"))
    };

    let capabilities = find_tool("research_capabilities");
    assert!(
        capabilities["description"]
            .as_str()
            .unwrap()
            .contains("capability contract")
    );

    let role_start = find_tool("research_role_start");
    for property in [
        "host",
        "execution_mode",
        "host_thread_id",
        "host_subagent_id",
        "tool_surface",
        "prompt_version",
        "prompt_hash",
        "input_artifact_ids",
    ] {
        assert!(
            role_start
                .pointer(&format!("/inputSchema/properties/{property}"))
                .is_some(),
            "research_role_start missing {property}"
        );
    }

    let role_finish = find_tool("research_role_finish");
    assert!(
        role_finish
            .pointer("/inputSchema/properties/output_artifact_id")
            .is_some()
    );
    assert!(
        role_finish["description"]
            .as_str()
            .unwrap()
            .contains("requires output_artifact_id")
    );

    let artifact = find_tool("research_artifact_add");
    assert!(
        artifact
            .pointer("/inputSchema/properties/role_run_id")
            .is_some()
    );
    assert!(
        artifact
            .pointer("/inputSchema/properties/metadata")
            .is_some()
    );
    assert!(
        artifact
            .pointer("/inputSchema/properties/metadata_json")
            .is_some()
    );

    let host_search = find_tool("research_host_search_record");
    assert_eq!(
        host_search.pointer("/inputSchema/properties/results/items/type"),
        Some(&json!("object"))
    );
    for property in [
        "rank",
        "title",
        "url",
        "snippet",
        "provider_metadata",
        "selected_for_ingest",
    ] {
        assert!(
            host_search
                .pointer(&format!(
                    "/inputSchema/properties/results/items/properties/{property}"
                ))
                .is_some(),
            "research_host_search_record result missing {property}"
        );
    }

    let document = find_tool("research_document_extract");
    assert!(document["description"].as_str().unwrap().contains("XLSX"));
    for property in ["media_type", "research_source_id", "source_card_id"] {
        assert!(
            document
                .pointer(&format!("/inputSchema/properties/{property}"))
                .is_some(),
            "research_document_extract missing {property}"
        );
    }

    for tool_name in [
        "research_convergence_start",
        "research_convergence_step",
        "research_convergence_run",
        "research_convergence_enqueue",
        "research_convergence_status",
        "research_iterations",
        "research_statements",
        "research_challenges",
        "research_convergence_host_search_tasks",
        "research_convergence_provider_search",
        "research_disproofs",
        "research_revisions",
        "research_fact_checks",
        "research_active_fact_check",
        "research_convergence_close_loop",
        "research_convergence_snapshots",
        "research_convergence_report_compile",
        "research_report_judgments",
    ] {
        let tool = find_tool(tool_name);
        assert!(
            tool.pointer("/inputSchema/properties/run_id").is_some()
                || tool.pointer("/inputSchema/properties/id").is_some(),
            "{tool_name} missing id/run_id input"
        );
    }

    for tool_name in ["research_convergence_run", "research_convergence_enqueue"] {
        let tool = find_tool(tool_name);
        for property in [
            "max_provider_calls",
            "editorial_provider",
            "editorial_model_name",
            "editorial_endpoint",
            "editorial_timeout_seconds",
        ] {
            assert!(
                tool.pointer(&format!("/inputSchema/properties/{property}"))
                    .is_some(),
                "{tool_name} missing convergence editorial/eval property {property}"
            );
        }
    }

    let active_fact_check = find_tool("research_active_fact_check");
    for property in ["artifact_id", "max_sentences", "create_challenges"] {
        assert!(
            active_fact_check
                .pointer(&format!("/inputSchema/properties/{property}"))
                .is_some(),
            "research_active_fact_check missing {property}"
        );
    }

    let provider_search = find_tool("research_convergence_provider_search");
    for property in [
        "provider",
        "max_tasks",
        "max_results",
        "max_provider_calls",
        "enqueue_selected_url_ingest",
        "max_ingest_jobs",
        "cost_cap_usd",
        "endpoint",
        "api_key",
        "model",
        "timeout_seconds",
    ] {
        assert!(
            provider_search
                .pointer(&format!("/inputSchema/properties/{property}"))
                .is_some(),
            "research_convergence_provider_search missing {property}"
        );
    }

    let close_loop = find_tool("research_convergence_close_loop");
    for property in [
        "artifact_id",
        "max_sentences",
        "create_challenges",
        "compile_report_before_check",
        "rerun_after_check",
        "compile_final_report",
        "provider",
        "provider_max_tasks",
        "provider_max_results",
        "provider_max_provider_calls",
        "provider_cost_cap_usd",
        "max_provider_calls",
        "editorial_provider",
    ] {
        assert!(
            close_loop
                .pointer(&format!("/inputSchema/properties/{property}"))
                .is_some(),
            "research_convergence_close_loop missing {property}"
        );
    }

    let editorial = find_tool("research_editorial_invoke");
    for property in [
        "model_provider",
        "model_name",
        "prompt_version",
        "input_artifact_id",
        "endpoint",
        "api_key",
        "timeout_seconds",
    ] {
        assert!(
            editorial
                .pointer(&format!("/inputSchema/properties/{property}"))
                .is_some(),
            "research_editorial_invoke missing {property}"
        );
    }
}

#[test]
fn severe_mcp_deep_research_agent_surface_round_trip_without_cli_fallback() {
    // CLAIM: The deep-research MCP surface can run the logged host-search,
    // role, artifact, document, evidence-pack, and editorial flow directly.
    // ORACLE: Every state transition is observed through call_mcp_tool.
    // SEVERITY: Severe because this reproduces the live failure class with
    // structured host results and completed role artifact linkage.
    let paths = test_paths("mcp-research-agent-surface");
    let workflow = call_mcp_tool(
        &paths,
        "research_run",
        json!({ "query": "research the most effective compression algorithms for images" }),
    )
    .unwrap();
    let run_id = workflow["run"]["id"].as_str().unwrap();

    let capabilities = call_mcp_tool(&paths, "research_capabilities", json!({})).unwrap();
    assert_eq!(capabilities["schema_version"].as_u64(), Some(3));
    assert_eq!(
        capabilities["role_orchestration"]["completed_requires_output_artifact_id"].as_bool(),
        Some(true)
    );
    assert_eq!(
        capabilities["iterated_epistemic_convergence"]["status_tool"].as_str(),
        Some("research_convergence_status")
    );

    let role = call_mcp_tool(
        &paths,
        "research_role_start",
        json!({
            "run_id": run_id,
            "role": "research-scout",
            "host": "codex",
            "execution_mode": "codex_subagent_live",
            "host_thread_id": "test-thread",
            "host_subagent_id": "test-subagent",
            "tool_surface": "mcp+host-search",
            "prompt_version": "severe-test-v1"
        }),
    )
    .unwrap();
    let role_run_id = role["id"].as_str().unwrap();

    let string_result_error = call_mcp_tool(
        &paths,
        "research_host_search_record",
        json!({
            "run_id": run_id,
            "query": "image compression codec benchmark official paper",
            "results": ["https://example.com/not-an-object"]
        }),
    )
    .expect_err("string search results must not be accepted");
    assert!(
        string_result_error
            .to_string()
            .contains("parsing host search results")
    );

    let search = call_mcp_tool(
        &paths,
        "research_host_search_record",
        json!({
            "run_id": run_id,
            "role_run_id": role_run_id,
            "query": "image compression codec benchmark official paper",
            "query_intent": "source-discovery",
            "requested_recency": 30,
            "requested_domains": ["example.com"],
            "results": [
                {
                    "rank": 1,
                    "title": "Codec benchmark paper",
                    "url": "https://example.com/codec-benchmark",
                    "snippet": "A benchmark compares modern image compression methods.",
                    "published_at": "2026-01-02",
                    "source_family_guess": "paper",
                    "provider_metadata": { "fixture": true },
                    "selected_for_ingest": true
                }
            ]
        }),
    )
    .unwrap();
    assert_eq!(search["results"].as_array().unwrap().len(), 1);
    assert!(
        search["results"][0]["research_source_id"]
            .as_str()
            .is_some()
    );

    let missing_output_error = call_mcp_tool(
        &paths,
        "research_role_finish",
        json!({
            "role_run_id": role_run_id,
            "status": "completed"
        }),
    )
    .expect_err("completed role without output artifact must fail");
    assert!(
        missing_output_error
            .to_string()
            .contains("requires an output artifact")
    );

    let artifact = call_mcp_tool(
        &paths,
        "research_artifact_add",
        json!({
            "run_id": run_id,
            "role_run_id": role_run_id,
            "artifact_type": "source_map",
            "title": "Scout source map",
            "body": "Selected a benchmark paper and recorded host-native proof.",
            "metadata_json": "{\"fixture\":true,\"schema\":\"mcp\"}"
        }),
    )
    .unwrap();
    let artifact_id = artifact["id"].as_str().unwrap();
    assert_eq!(artifact["role_run_id"].as_str(), Some(role_run_id));
    assert_eq!(artifact["metadata"]["schema"].as_str(), Some("mcp"));

    let finished = call_mcp_tool(
        &paths,
        "research_role_finish",
        json!({
            "role_run_id": role_run_id,
            "status": "completed",
            "output_artifact_id": artifact_id
        }),
    )
    .unwrap();
    assert_eq!(finished["status"].as_str(), Some("completed"));

    fs::create_dir_all(&paths.home).unwrap();
    let csv_path = paths.home.join("codec-benchmarks.csv");
    fs::write(
        &csv_path,
        "codec,ratio,notes\nAVIF,0.72,high quality\nJPEG XL,0.69,fast decode\n",
    )
    .unwrap();
    let document = call_mcp_tool(
        &paths,
        "research_document_extract",
        json!({
            "run_id": run_id,
            "path": csv_path.to_string_lossy(),
            "media_type": "text/csv"
        }),
    )
    .unwrap();
    assert_eq!(
        document["document"]["extraction_status"].as_str(),
        Some("extracted")
    );
    assert_eq!(document["tables"].as_array().unwrap().len(), 1);
    assert_eq!(
        document["tables"][0]["cells"][0]["column_header"].as_str(),
        Some("codec")
    );

    let evidence = call_mcp_tool(
        &paths,
        "research_evidence_pack",
        json!({ "run_id": run_id }),
    )
    .unwrap();
    assert_eq!(evidence["artifact_type"].as_str(), Some("evidence_pack"));
    let editorial = call_mcp_tool(
        &paths,
        "research_editorial_invoke",
        json!({
            "run_id": run_id,
            "stage": "editorial_drafter",
            "model_provider": "mock",
            "input_artifact_id": evidence["id"].as_str().unwrap(),
            "prompt_version": "severe-test-v1",
            "timeout_seconds": 5
        }),
    )
    .unwrap();
    assert_eq!(
        editorial["editorial_run"]["model_provider"].as_str(),
        Some("mock")
    );
    assert!(
        editorial["output_artifact"]["id"].as_str().is_some(),
        "{editorial}"
    );
}

#[test]
fn severe_mcp_commerce_schemas_expose_local_only_boundaries() {
    // CLAIM: MCP discovery exposes exact commerce evidence fields without implying live shopping works.
    // ORACLE: capability and tool schemas include variant/proof/context/judgment fields plus false live-proof gates.
    // SEVERITY: Severe because thin schemas make agents hallucinate availability and skip exact-size proof.
    let tools = mcp_tools();
    let find_tool = |name: &str| {
        tools
            .iter()
            .find(|tool| tool.get("name").and_then(Value::as_str) == Some(name))
            .unwrap_or_else(|| panic!("missing tool {name}"))
    };

    let capabilities_tool = find_tool("commerce_research_capabilities");
    assert!(
        capabilities_tool["description"]
            .as_str()
            .unwrap()
            .contains("bounded production-data proof")
    );

    let candidate = find_tool("commerce_candidate_add");
    for property in [
        "source_url",
        "retailer_or_provider",
        "normalized_item_key",
        "variant_key",
        "score_reasons",
        "disqualification_reasons",
    ] {
        assert!(
            candidate
                .pointer(&format!("/inputSchema/properties/{property}"))
                .is_some(),
            "commerce_candidate_add missing {property}"
        );
    }

    let proof = find_tool("commerce_availability_proof_add");
    for property in [
        "proof_method",
        "variant_key",
        "variant_label",
        "availability_state",
        "visible_evidence",
        "screenshot_artifact_id",
        "page_snapshot_artifact_id",
    ] {
        assert!(
            proof
                .pointer(&format!("/inputSchema/properties/{property}"))
                .is_some(),
            "commerce_availability_proof_add missing {property}"
        );
    }
    assert!(
        proof["description"]
            .as_str()
            .unwrap()
            .contains("wrong variants")
    );

    let rendered_check = find_tool("commerce_rendered_page_check");
    for property in [
        "requested_url",
        "rendered_html",
        "rendered_text",
        "variant_key",
        "variant_label",
        "chrome_profile_required",
    ] {
        assert!(
            rendered_check
                .pointer(&format!("/inputSchema/properties/{property}"))
                .is_some(),
            "commerce_rendered_page_check missing {property}"
        );
    }
    assert!(
        rendered_check["description"]
            .as_str()
            .unwrap()
            .contains("performs no browser or network fetch")
    );

    let context_packet = find_tool("commerce_context_packet_compile");
    assert!(
        context_packet["description"]
            .as_str()
            .unwrap()
            .contains("redacted")
    );

    let report_compile = find_tool("commerce_report_compile");
    assert!(
        report_compile["description"]
            .as_str()
            .unwrap()
            .contains("gated")
    );
    let judgment = find_tool("commerce_report_judgment_add");
    assert!(
        judgment["description"]
            .as_str()
            .unwrap()
            .contains("blocking findings")
    );

    let paths = test_paths("mcp-commerce-capabilities");
    let capabilities = call_mcp_tool(&paths, "commerce_research_capabilities", json!({})).unwrap();
    assert_eq!(
        capabilities["status"],
        json!("partial_bounded_production_data_proof")
    );
    assert_eq!(
        capabilities["proof_boundaries"]["browser_rendered_extraction"],
        json!("host_supplied_local_check_proven_no_daemon_browse")
    );
    assert_eq!(
        capabilities["proof_boundaries"]["source_card_linkage"],
        json!("locally_proven_for_host_supplied_rendered_pages")
    );
    assert_eq!(
        capabilities["proof_boundaries"]["bounded_live_uk_fashion_packet"],
        json!("production_data_proven_for_two_mands_pages")
    );
    assert_eq!(
        capabilities["proof_boundaries"]["broad_production_data_proof"],
        json!(false)
    );
}

#[test]
fn severe_mcp_commerce_ledger_round_trips_and_rejects_fake_availability() {
    // CLAIM: MCP writes/readbacks use the durable commerce ledger and reject common fake-proof shapes.
    // ORACLE: exact variant availability succeeds once, wrong variant and accept-with-blockers fail.
    // SEVERITY: Severe because unavailable sizes and overaccepted reports are the core user-harm path.
    let paths = test_paths("mcp-commerce-ledger");
    let run = call_mcp_tool(
        &paths,
        "research_run",
        json!({ "query": "Find UK loafers with softer soles in UK 8.5" }),
    )
    .unwrap();
    let run_id = run["run"]["id"].as_str().unwrap();

    call_mcp_tool(
        &paths,
        "commerce_run_config_set",
        json!({
            "run_id": run_id,
            "domain_profile": "uk-fashion-retail",
            "target_qualified_count": 1,
            "geography": "UK",
            "freshness_window": "24h",
            "allowed_private_context_sources": ["memory", "wardrobe"],
            "allowed_public_source_families": ["retailer", "marketplace", "review"],
            "allow_marketplaces": true,
            "allow_chrome_profile": false,
            "max_provider_calls": 12,
            "max_browser_pages": 80,
            "max_cost_usd": 4.5,
            "stop_rules": { "min_available_exact_variant": 1 }
        }),
    )
    .unwrap();

    let context = call_mcp_tool(
        &paths,
        "commerce_context_fact_add",
        json!({
            "run_id": run_id,
            "fact_key": "shoe_size_uk",
            "fact_kind": "explicit",
            "redacted_value": "UK 8.5",
            "source_family": "memory",
            "confidence": 0.95,
            "user_confirmed": true,
            "may_persist_to_memory": true,
            "metadata": { "semantic_kind": "size", "raw_value": "[redacted]" }
        }),
    )
    .unwrap();
    assert_eq!(context["fact_key"], json!("shoe_size_uk"));

    let candidate = call_mcp_tool(
        &paths,
        "commerce_candidate_add",
        json!({
            "run_id": run_id,
            "domain": "fashion",
            "source_url": "https://example.test/loafers/soft-sole",
            "retailer_or_provider": "Example Shoes",
            "title": "Soft Sole Penny Loafer",
            "normalized_item_key": "example-shoes-soft-sole-penny-loafer",
            "variant_key": "category=shoe;size_system=UK;size=8.5",
            "price": "185.00",
            "currency": "GBP",
            "geography": "UK",
            "candidate_status": "qualified",
            "score": 0.84,
            "score_reasons": { "comfort": "visible cushioned sole claim" },
            "disqualification_reasons": [],
            "metadata": { "source": "mcp-test" }
        }),
    )
    .unwrap();
    let candidate_id = candidate["id"].as_str().unwrap();

    let rendered_check = call_mcp_tool(
            &paths,
            "commerce_rendered_page_check",
            json!({
                "run_id": run_id,
                "candidate_id": candidate_id,
                "variant_key": "category=shoe;size_system=UK;size=8.5",
                "variant_label": "UK 8.5",
                "requested_url": "https://example.test/loafers/soft-sole",
                "final_url": "https://example.test/loafers/soft-sole?size=8.5",
                "title": "Soft Sole Penny Loafer",
                "rendered_text": "Soft Sole Penny Loafer\nPrice GBP 185\nSize UK 8.5 available - add to bag",
                "captured_at": "2026-06-24T10:00:00Z",
                "browser": "codex-in-app-browser",
                "selector_or_dom_hint": "button[data-size='8.5']",
                "chrome_profile_required": false
            }),
        )
        .unwrap();
    assert_eq!(rendered_check["availability_state"], json!("available"));
    let proof_id = rendered_check["availability_proof"]["id"].as_str().unwrap();
    assert_eq!(
        rendered_check["source_card"]["metadata"]["commerce_availability_state"],
        json!("available")
    );
    assert!(
        rendered_check["research_source_link"]["link"]["id"]
            .as_str()
            .is_some()
    );

    let context_packet = call_mcp_tool(
        &paths,
        "commerce_context_packet_compile",
        json!({ "run_id": run_id }),
    )
    .unwrap();
    assert_eq!(context_packet["fact_count"], json!(1));
    assert!(
        context_packet["artifact"]["body"]
            .as_str()
            .unwrap()
            .contains("shoe_size_uk")
    );

    let compiled_report = call_mcp_tool(
        &paths,
        "commerce_report_compile",
        json!({ "run_id": run_id }),
    )
    .unwrap();
    assert_eq!(compiled_report["judgment"]["decision"], json!("accept"));
    assert_eq!(compiled_report["recommended_count"], json!(1));
    assert_eq!(compiled_report["source_card_count"], json!(1));
    assert!(
        compiled_report["artifact"]["body"]
            .as_str()
            .unwrap()
            .contains("Main Recommendations")
    );

    let wrong_variant = call_mcp_tool(
        &paths,
        "commerce_availability_proof_add",
        json!({
            "run_id": run_id,
            "candidate_id": candidate_id,
            "proof_method": "rendered_browser",
            "variant_key": "category=shoe;size_system=UK;size=9",
            "variant_label": "UK 9",
            "availability_state": "available",
            "visible_evidence": "Size UK 9 is available"
        }),
    )
    .unwrap_err()
    .to_string();
    assert!(wrong_variant.contains("variant does not match"));

    call_mcp_tool(
        &paths,
        "commerce_verification_attempt_add",
        json!({
            "run_id": run_id,
            "candidate_id": candidate_id,
            "method": "rendered_browser",
            "result": "blocked",
            "error_kind": "cookie_wall",
            "browser_required": true,
            "chrome_profile_required": true,
            "next_action": "retry in user Chrome profile"
        }),
    )
    .unwrap();

    let blocked_accept = call_mcp_tool(
        &paths,
        "commerce_report_judgment_add",
        json!({
            "run_id": run_id,
            "decision": "accept",
            "blocking_findings": ["Only one exact-size proof exists."],
            "availability_proofs_checked": [proof_id],
            "privacy_review": { "redacted_context": true },
            "remaining_risks": ["fixture-only proof"]
        }),
    )
    .unwrap_err()
    .to_string();
    assert!(blocked_accept.contains("cannot include blocking findings"));

    call_mcp_tool(
        &paths,
        "commerce_report_judgment_add",
        json!({
            "run_id": run_id,
            "decision": "hold",
            "blocking_findings": ["Need production-data browser proof."],
            "claims_checked": ["availability is fixture-only"],
            "availability_proofs_checked": [proof_id],
            "privacy_review": { "redacted_context": true },
            "remaining_risks": ["no live retailer page was checked"]
        }),
    )
    .unwrap();

    assert_eq!(
        call_mcp_tool(&paths, "commerce_candidates", json!({ "run_id": run_id }))
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        call_mcp_tool(
            &paths,
            "commerce_availability_proofs",
            json!({ "run_id": run_id })
        )
        .unwrap()
        .as_array()
        .unwrap()
        .len(),
        1
    );
    assert_eq!(
        call_mcp_tool(
            &paths,
            "commerce_report_judgments",
            json!({ "run_id": run_id })
        )
        .unwrap()
        .as_array()
        .unwrap()
        .len(),
        2
    );
}

#[test]
fn severe_mcp_job_hunting_round_trips_local_proof_workflow() {
    // CLAIM: MCP exposes the local-proof job-hunting workflow as durable
    // typed records with privacy and source-confidence gates.
    // ORACLE: profile, evidence, role, score, packet approval/export,
    // application, and weekly report round-trip through MCP; privacy
    // checks can still block text independently.
    // SEVERITY: Severe because tool discovery without real writes would
    // make agents claim a job-search system that does not exist.
    let tools = mcp_tools();
    for tool_name in [
        "job_profile_add",
        "job_import_batch",
        "job_evidence_add",
        "job_evidence_review_report",
        "job_role_add",
        "job_score_add",
        "job_packet_create",
        "job_packet_approve",
        "job_packet_export",
        "job_packet_export_set",
        "job_outreach_readiness",
        "job_application_record",
        "job_source_refresh",
        "job_radar_schedule",
        "job_radar_enqueue",
        "job_refresh_manual",
        "job_refresh_audit",
        "job_operational_audit",
        "job_company_targets",
        "job_weekly_report",
        "job_weekly_report_delivery_prepare",
        "job_weekly_report_delivery_send",
        "job_weekly_report_deliveries",
    ] {
        assert!(
            tools
                .iter()
                .any(|tool| tool.get("name").and_then(Value::as_str) == Some(tool_name)),
            "missing MCP job tool {tool_name}"
        );
    }
    let role_tool = tools
        .iter()
        .find(|tool| tool.get("name").and_then(Value::as_str) == Some("job_role_add"))
        .unwrap();
    assert!(
        role_tool["description"]
            .as_str()
            .unwrap()
            .contains("does not claim broad search coverage")
    );
    assert!(
        role_tool
            .pointer("/inputSchema/properties/source_confidence")
            .is_some()
    );
    let source_refresh_tool = tools
        .iter()
        .find(|tool| tool.get("name").and_then(Value::as_str) == Some("job_source_refresh"))
        .unwrap();
    assert!(
        source_refresh_tool["description"]
            .as_str()
            .unwrap()
            .contains("caller-supplied page text/html")
    );
    assert!(
        source_refresh_tool
            .pointer("/inputSchema/properties/fetch_live")
            .is_some()
    );
    let radar_schedule_tool = tools
        .iter()
        .find(|tool| tool.get("name").and_then(Value::as_str) == Some("job_radar_schedule"))
        .unwrap();
    assert!(
        radar_schedule_tool["description"]
            .as_str()
            .unwrap()
            .contains("local scheduled proof")
    );
    assert!(
        radar_schedule_tool
            .pointer("/inputSchema/properties/source_snapshots")
            .is_some()
    );
    let report_delivery_tool = tools
        .iter()
        .find(|tool| {
            tool.get("name").and_then(Value::as_str) == Some("job_weekly_report_delivery_prepare")
        })
        .unwrap();
    assert!(
        report_delivery_tool["description"]
            .as_str()
            .unwrap()
            .contains("does not call provider APIs or send")
    );
    assert!(
        report_delivery_tool
            .pointer("/inputSchema/properties/idempotency_key")
            .is_some()
    );
    let report_delivery_send_tool = tools
        .iter()
        .find(|tool| {
            tool.get("name").and_then(Value::as_str) == Some("job_weekly_report_delivery_send")
        })
        .unwrap();
    assert!(
        report_delivery_send_tool["description"]
            .as_str()
            .unwrap()
            .contains("records a provider delivery attempt")
    );
    assert!(
        report_delivery_send_tool
            .pointer("/inputSchema/properties/api_base")
            .is_some()
    );
    let outreach_tool = tools
        .iter()
        .find(|tool| tool.get("name").and_then(Value::as_str) == Some("job_outreach_readiness"))
        .unwrap();
    assert!(
        outreach_tool["description"]
            .as_str()
            .unwrap()
            .contains("does not send")
    );
    assert!(
        outreach_tool
            .pointer("/inputSchema/properties/limit")
            .is_some()
    );
    let operational_audit_tool = tools
        .iter()
        .find(|tool| tool.get("name").and_then(Value::as_str) == Some("job_operational_audit"))
        .unwrap();
    assert!(
        operational_audit_tool["description"]
            .as_str()
            .unwrap()
            .contains("does not fetch, send, or submit")
    );
    assert!(
        operational_audit_tool
            .pointer("/inputSchema/properties/min_elapsed_hours")
            .is_some()
    );
    let packet_export_set_tool = tools
        .iter()
        .find(|tool| tool.get("name").and_then(Value::as_str) == Some("job_packet_export_set"))
        .unwrap();
    assert!(
        packet_export_set_tool["description"]
            .as_str()
            .unwrap()
            .contains("does not create Google Docs, send, submit, or record applications")
    );
    assert!(
        packet_export_set_tool
            .pointer("/inputSchema/properties/packet_ids")
            .is_some()
    );

    let paths = test_paths("mcp-job-hunting-roundtrip");
    let imported = call_mcp_tool(
        &paths,
        "job_import_batch",
        json!({
            "batch": {
                "profile": {
                    "label": "Imported Candidate",
                    "github_profile": "https://github.com/chrischabot"
                }
            }
        }),
    )
    .unwrap();
    assert_eq!(imported["profile_ids"].as_array().unwrap().len(), 1);
    assert!(
        imported["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| warning
                .as_str()
                .unwrap()
                .contains("does not prove live source discovery"))
    );
    let source_import = call_mcp_tool(
        &paths,
        "job_import_batch",
        json!({
            "batch": {
                "sources": [{
                    "source_family": "company",
                    "name": "Example Careers",
                    "url": "https://example.com/careers",
                    "market_scope": "london",
                    "refresh_policy": "manual"
                }]
            }
        }),
    )
    .unwrap();
    let source_id = source_import["source_ids"][0].as_str().unwrap();
    let source_refresh = call_mcp_tool(
            &paths,
            "job_source_refresh",
            json!({
                "source_id": source_id,
                "fetched_url": "https://example.com/careers",
                "body": r#"
                    <main>
                      <h1>Example Careers</h1>
                      <p>Developer platform and agent infrastructure roles in London.</p>
                      <a href="/careers/staff-agent-platform-engineer">Staff Agent Platform Engineer - London hybrid</a>
                    </main>
                "#
            }),
        )
        .unwrap();
    assert_eq!(source_refresh["source_health"]["status"], json!("healthy"));
    assert_eq!(source_refresh["roles"].as_array().unwrap().len(), 1);
    assert_eq!(
        source_refresh["roles"][0]["source_confidence"],
        json!("canonical_confirmed")
    );
    let refreshed_role_id = source_refresh["roles"][0]["id"].as_str().unwrap();
    assert_eq!(
        source_refresh["role_source_links"][0]["source_id"],
        json!(source_id)
    );
    let stale_refresh = call_mcp_tool(
        &paths,
        "job_source_refresh",
        json!({
            "source_id": source_id,
            "fetched_url": "https://example.com/careers",
            "body": "<main><h1>Example Careers</h1><p>No current openings.</p></main>"
        }),
    )
    .unwrap();
    assert_eq!(stale_refresh["source_health"]["status"], json!("stale"));
    assert_eq!(
        stale_refresh["stale_role_events"][0]["role_id"],
        json!(refreshed_role_id)
    );

    let profile = call_mcp_tool(
        &paths,
        "job_profile_add",
        json!({
            "label": "Chris Chabot",
            "github_profile": "https://github.com/chrischabot",
            "blog_url": "https://chabot.dev"
        }),
    )
    .unwrap();
    let profile_id = profile["id"].as_str().unwrap();

    let mut schedule_snapshots = serde_json::Map::new();
    schedule_snapshots.insert(
            source_id.to_string(),
            json!({
                "fetched_url": "https://example.com/careers",
                "body": "<main><a href='/careers/principal-agent-engineer'>Principal Agent Engineer - London</a></main>"
            }),
        );
    let radar_schedule = call_mcp_tool(
        &paths,
        "job_radar_schedule",
        json!({
            "profile_id": profile_id,
            "scope": "London agent platform roles",
            "source_ids": [source_id],
            "source_snapshots": Value::Object(schedule_snapshots),
            "cadence": "warm",
            "status": "active"
        }),
    )
    .unwrap();
    assert_eq!(radar_schedule["source_kind"], json!("job_radar"));
    let mut enqueue_snapshots = serde_json::Map::new();
    enqueue_snapshots.insert(
            source_id.to_string(),
            json!("<main><a href='/careers/principal-agent-engineer'>Principal Agent Engineer - London</a></main>"),
        );
    let radar_job = call_mcp_tool(
        &paths,
        "job_radar_enqueue",
        json!({
            "profile_id": profile_id,
            "scope": "London agent platform roles",
            "source_ids": [source_id],
            "source_snapshots": Value::Object(enqueue_snapshots)
        }),
    )
    .unwrap();
    assert_eq!(radar_job["kind"], json!("job_radar_refresh"));
    assert_eq!(radar_job["status"], json!("pending"));

    let evidence = call_mcp_tool(
        &paths,
        "job_evidence_add",
        json!({
            "profile_id": profile_id,
            "title": "Open Cloud",
            "evidence_type": "github",
            "visibility": "public",
            "summary": "Public developer tooling evidence.",
            "proof_url": "https://github.com/chrischabot/opencloud",
            "confidence": "verified",
            "tags": ["developer-tools", "cloud"],
            "safe_application_text": "Built public cloud developer tooling.",
            "unsafe_terms": []
        }),
    )
    .unwrap();
    let evidence_id = evidence["id"].as_str().unwrap();

    let company_import = call_mcp_tool(
        &paths,
        "job_import_batch",
        json!({
            "batch": {
                "companies": [{
                    "company_name": "Orbital Cloud",
                    "website_url": "https://orbital.example",
                    "source_family": "company",
                    "market": "london",
                    "stage": "seed",
                    "funding_signal": "technical founder-led developer tools company",
                    "product_category": "cloud developer tools",
                    "technical_audience": "developer-tools and platform teams",
                    "developer_facing_score": 4.8,
                    "london_relevance": "high London relevance",
                    "remote_maturity": "remote Europe and London hybrid",
                    "hiring_page_url": "https://orbital.example/careers",
                    "founder_or_team_signal": "founders write about cloud developer workflows",
                    "metadata": { "note": "scouting target, not a role" }
                }, {
                    "company_name": "Consumer Garden",
                    "website_url": "https://garden.example",
                    "source_family": "directory",
                    "market": "berlin",
                    "developer_facing_score": 1.5,
                    "london_relevance": "low",
                    "metadata": {}
                }]
            }
        }),
    )
    .unwrap();
    let company_id = company_import["company_ids"][0].as_str().unwrap();

    let company_targets = call_mcp_tool(
        &paths,
        "job_company_targets",
        json!({
            "profile_id": profile_id,
            "market": "london",
            "limit": 10
        }),
    )
    .unwrap();
    assert_eq!(company_targets["proof_level"], json!("local_proof"));
    assert!(
        company_targets["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| warning.as_str().unwrap().contains("not current role cards"))
    );
    assert_eq!(
        company_targets["entries"][0]["company"]["id"],
        json!(company_id)
    );
    assert_eq!(company_targets["entries"][0]["tier"], json!("target_now"));
    assert!(
        company_targets["entries"][0]["matched_evidence_tags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tag| tag.as_str() == Some("developer-tools"))
    );
    assert!(
        company_targets["entries"][0]["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| warning
                .as_str()
                .unwrap()
                .contains("No current role is implied"))
    );

    let evidence_report = call_mcp_tool(
        &paths,
        "job_evidence_review_report",
        json!({ "profile_id": profile_id }),
    )
    .unwrap();
    assert_eq!(evidence_report["decision"], json!("warn"));
    assert_eq!(evidence_report["evidence_card_count"], json!(1));
    assert!(
        evidence_report["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["finding_type"] == json!("thin_evidence_set"))
    );

    let role = call_mcp_tool(
        &paths,
        "job_role_add",
        json!({
            "company": "Example AI",
            "role_title": "Staff Agent Platform Engineer",
            "canonical_url": "https://example.com/careers/staff-agent-platform-engineer",
            "source_family": "company",
            "source_url": "https://example.com/careers/staff-agent-platform-engineer",
            "source_confidence": "canonical_confirmed",
            "posting_freshness": "same_day",
            "location": "London or remote Europe",
            "work_mode": "hybrid_or_remote",
            "company_stage_or_size": "startup",
            "role_seniority": "staff",
            "core_requirements": ["agent systems", "developer tooling"],
            "evidence_card_ids": [evidence_id],
            "current_status": "live"
        }),
    )
    .unwrap();
    let role_id = role["id"].as_str().unwrap();

    let score = call_mcp_tool(
        &paths,
        "job_score_add",
        json!({
            "role_id": role_id,
            "profile_id": profile_id,
            "scorer": "human",
            "role_fit": 5.0,
            "domain_fit": 5.0,
            "evidence_fit": 5.0,
            "geo_work_fit": 5.0,
            "stage_fit": 4.5,
            "practical_odds": 4.5,
            "interest_energy": 5.0,
            "evidence_card_ids": [evidence_id],
            "explanation": "Strong match across agent systems and developer tooling."
        }),
    )
    .unwrap();
    assert_eq!(score["tier"], json!("tier_1"));

    let shortlist =
        call_mcp_tool(&paths, "job_shortlist", json!({ "profile_id": profile_id })).unwrap();
    assert!(
        shortlist["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["role"]["id"].as_str() == Some(role_id))
    );

    let blocked = call_mcp_tool(
        &paths,
        "job_privacy_check",
        json!({
            "artifact_type": "outreach",
            "artifact_id": role_id,
            "text": "This mentions private-name.",
            "blocked_terms": ["private-name"]
        }),
    )
    .unwrap();
    assert_eq!(blocked["decision"], json!("block"));

    let packet = call_mcp_tool(
        &paths,
        "job_packet_create",
        json!({
            "role_id": role_id,
            "profile_id": profile_id,
            "evidence_card_ids": [evidence_id],
            "resume_emphasis": "Lead with public developer tooling and agent systems.",
            "tailored_bullets": ["Built public cloud developer tooling."],
            "outreach_note": "Example AI appears to need reliable agent tooling discipline.",
            "proof_links": { "github": "https://github.com/chrischabot/opencloud" },
            "likely_objections": ["No direct company-specific evidence yet."],
            "interview_stories": ["Public project technical story."],
            "questions_to_ask": ["Where do agent workflows fail today?"]
        }),
    )
    .unwrap();
    let packet_id = packet["id"].as_str().unwrap();
    assert_eq!(packet["status"], json!("draft"));

    let draft_readiness = call_mcp_tool(
        &paths,
        "job_outreach_readiness",
        json!({
            "profile_id": profile_id,
            "limit": 5
        }),
    )
    .unwrap();
    assert_eq!(draft_readiness["ready_count"], json!(0));
    assert_eq!(
        draft_readiness["entries"][0]["packet_status"],
        json!("draft")
    );
    assert!(
        draft_readiness["entries"][0]["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|blocker| blocker
                .as_str()
                .unwrap()
                .contains("approved packet required"))
    );

    let public_contact_import = call_mcp_tool(
        &paths,
        "job_import_batch",
        json!({
            "batch": {
                "contacts": [{
                    "name": "Public Hiring Manager",
                    "role_title": "Engineering Manager",
                    "public_profile_url": "https://example.com/team/public-hiring-manager",
                    "source_url": "https://example.com/team",
                    "relationship_status": "public_only",
                    "relevance": "hiring_manager",
                    "note": "Source evidence: public team page lists this person as engineering manager; no relationship path."
                }]
            }
        }),
    )
    .unwrap();
    let public_contact_id = public_contact_import["contact_ids"][0].as_str().unwrap();
    call_mcp_tool(
        &paths,
        "job_import_batch",
        json!({
            "batch": {
                "intro_paths": [{
                    "role_id": role_id,
                    "contact_id": public_contact_id,
                    "path_type": "unknown",
                    "confidence": "weak",
                    "next_action": "Find a real route before outreach.",
                    "status": "identify"
                }]
            }
        }),
    )
    .unwrap();

    let approved_packet = call_mcp_tool(
        &paths,
        "job_packet_approve",
        json!({
            "packet_id": packet_id,
            "reviewer_note": "Reviewed by user for MCP round-trip."
        }),
    )
    .unwrap();
    assert_eq!(approved_packet["status"], json!("approved"));

    let public_only_readiness = call_mcp_tool(
        &paths,
        "job_outreach_readiness",
        json!({
            "profile_id": profile_id,
            "limit": 5
        }),
    )
    .unwrap();
    assert_eq!(public_only_readiness["ready_count"], json!(0));
    assert_eq!(
        public_only_readiness["entries"][0]["public_only_count"],
        json!(1)
    );
    assert_eq!(
        public_only_readiness["entries"][0]["warm_intro_ready_count"],
        json!(0)
    );
    assert!(
        public_only_readiness["entries"][0]["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|blocker| blocker
                .as_str()
                .unwrap()
                .contains("public-only paths remain identify"))
    );

    let known_contact_import = call_mcp_tool(
        &paths,
        "job_import_batch",
        json!({
            "batch": {
                "contacts": [{
                    "name": "Known Staff Engineer",
                    "role_title": "Staff Engineer",
                    "public_profile_url": "https://example.com/team/known-staff-engineer",
                    "source_url": "https://example.com/team",
                    "relationship_status": "known",
                    "relevance": "engineer",
                    "note": "User confirmed this contact can route a relevant intro."
                }]
            }
        }),
    )
    .unwrap();
    let known_contact_id = known_contact_import["contact_ids"][0].as_str().unwrap();
    call_mcp_tool(
        &paths,
        "job_import_batch",
        json!({
            "batch": {
                "intro_paths": [{
                    "role_id": role_id,
                    "contact_id": known_contact_id,
                    "path_type": "mutual",
                    "confidence": "confirmed",
                    "next_action": "Ask for an intro using the approved packet.",
                    "status": "ask"
                }]
            }
        }),
    )
    .unwrap();
    let ready_outreach = call_mcp_tool(
        &paths,
        "job_outreach_readiness",
        json!({
            "profile_id": profile_id,
            "limit": 5
        }),
    )
    .unwrap();
    assert_eq!(ready_outreach["ready_count"], json!(1));
    assert_eq!(ready_outreach["entries"][0]["decision"], json!("ready"));
    assert_eq!(
        ready_outreach["entries"][0]["warm_intro_ready_count"],
        json!(1)
    );
    assert!(
        ready_outreach["non_claims"]
            .as_array()
            .unwrap()
            .iter()
            .any(|non_claim| non_claim.as_str().unwrap().contains("does not send"))
    );

    let export_dir = paths.home.join("packet-exports");
    let exported_packet = call_mcp_tool(
        &paths,
        "job_packet_export",
        json!({
            "packet_id": packet_id,
            "out_dir": export_dir.to_string_lossy()
        }),
    )
    .unwrap();
    assert_eq!(exported_packet["proof_level"], json!("local_proof"));
    assert_eq!(exported_packet["delivery_status"], json!("not_sent"));
    assert_eq!(exported_packet["application_status_changed"], json!(false));
    let exported_path = PathBuf::from(exported_packet["path"].as_str().unwrap());
    assert!(exported_path.exists());
    let exported_body = fs::read_to_string(&exported_path).unwrap();
    assert!(exported_body.contains("Delivery status: not_sent"));
    assert!(exported_body.contains("not proof that an application was sent"));

    let export_set_dir = paths.home.join("packet-export-sets");
    let exported_set = call_mcp_tool(
        &paths,
        "job_packet_export_set",
        json!({
            "profile_id": profile_id,
            "packet_ids": [packet_id],
            "out_dir": export_set_dir.to_string_lossy()
        }),
    )
    .unwrap();
    assert_eq!(exported_set["proof_level"], json!("local_proof"));
    assert_eq!(exported_set["delivery_status"], json!("not_sent"));
    assert_eq!(exported_set["application_status_changed"], json!(false));
    assert_eq!(exported_set["exported_count"], json!(1));
    assert!(PathBuf::from(exported_set["manifest_path"].as_str().unwrap()).exists());
    assert!(
        exported_set["non_claims"]
            .as_array()
            .unwrap()
            .iter()
            .any(|non_claim| non_claim.as_str().unwrap().contains("not Google Docs"))
    );

    let application = call_mcp_tool(
        &paths,
        "job_application_record",
        json!({
            "role_id": role_id,
            "packet_id": packet_id,
            "status": "applied",
            "applied_at": "2026-06-28",
            "follow_up_at": "2026-07-05"
        }),
    )
    .unwrap();
    assert_eq!(application["status"], json!("applied"));

    let refresh = call_mcp_tool(
        &paths,
        "job_refresh_manual",
        json!({
            "profile_id": profile_id,
            "scope": "London agent platform roles",
            "closed_role_ids": [role_id],
            "proof_level": "local_proof"
        }),
    )
    .unwrap();
    assert_eq!(refresh["closed_role_count"], json!(1));
    assert_eq!(refresh["events"][0]["status"], json!("closed"));

    let refresh_audit = call_mcp_tool(
        &paths,
        "job_refresh_audit",
        json!({
            "profile_id": profile_id,
            "scope": "London agent platform roles"
        }),
    )
    .unwrap();
    assert_eq!(refresh_audit["decision"], json!("block"));
    assert_eq!(refresh_audit["minimum_elapsed_hours"], json!(24));
    assert!(
        refresh_audit["missing_requirements"]
            .as_array()
            .unwrap()
            .iter()
            .any(|missing| missing
                .as_str()
                .unwrap()
                .contains("At least two completed job search runs"))
    );

    let closed_shortlist =
        call_mcp_tool(&paths, "job_shortlist", json!({ "profile_id": profile_id })).unwrap();
    let closed_entry = closed_shortlist["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["role"]["id"].as_str() == Some(role_id))
        .unwrap();
    assert_eq!(closed_entry["score"]["tier"], json!("blocked"));

    let report = call_mcp_tool(
        &paths,
        "job_weekly_report",
        json!({
            "profile_id": profile_id,
            "scope": "London agent platform roles"
        }),
    )
    .unwrap();
    assert!(
        report["body"].as_str().unwrap().contains("applied: 1"),
        "{report}"
    );
    assert_eq!(report["metadata"]["application_count"], json!(1));
    let report_id = report["id"].as_str().unwrap();

    let unauthorized_delivery = call_mcp_tool(
        &paths,
        "job_weekly_report_delivery_prepare",
        json!({
            "report_id": report_id,
            "channel": "email",
            "subject": "job-proof@example.com",
            "target": "job-proof@example.com",
            "idempotency_key": "mcp-job-weekly-delivery-unauthorized"
        }),
    )
    .unwrap();
    assert_eq!(
        unauthorized_delivery["delivery"]["status"],
        json!("blocked")
    );
    assert_eq!(unauthorized_delivery["channel_message"], Value::Null);

    call_mcp_tool(
        &paths,
        "channel_authorize",
        json!({
            "channel": "email",
            "subject": "email:job-proof@example.com",
            "can_send": true
        }),
    )
    .unwrap();
    let prepared_delivery = call_mcp_tool(
        &paths,
        "job_weekly_report_delivery_prepare",
        json!({
            "report_id": report_id,
            "channel": "email",
            "subject": "job-proof@example.com",
            "target": "job-proof@example.com",
            "idempotency_key": "mcp-job-weekly-delivery-prepared"
        }),
    )
    .unwrap();
    assert_eq!(prepared_delivery["delivery"]["status"], json!("prepared"));
    assert_eq!(
        prepared_delivery["privacy_check"]["decision"],
        json!("pass")
    );
    assert_eq!(
        prepared_delivery["channel_message"]["status"],
        json!("prepared")
    );
    let prepared_message_id = prepared_delivery["delivery"]["channel_message_id"]
        .as_str()
        .unwrap();

    let replay_delivery = call_mcp_tool(
        &paths,
        "job_weekly_report_delivery_prepare",
        json!({
            "report_id": report_id,
            "channel": "email",
            "subject": "email:job-proof@example.com",
            "target": "email:job-proof@example.com",
            "idempotency_key": "mcp-job-weekly-delivery-prepared"
        }),
    )
    .unwrap();
    assert_eq!(replay_delivery["idempotent_replay"], json!(true));
    assert_eq!(
        replay_delivery["delivery"]["channel_message_id"],
        json!(prepared_message_id)
    );
    let deliveries = call_mcp_tool(
        &paths,
        "job_weekly_report_deliveries",
        json!({ "report_id": report_id }),
    )
    .unwrap();
    assert_eq!(deliveries.as_array().unwrap().len(), 2);
    let operational_audit = call_mcp_tool(
        &paths,
        "job_operational_audit",
        json!({
            "profile_id": profile_id,
            "scope": "London agent platform roles",
            "min_elapsed_hours": 0
        }),
    )
    .unwrap();
    assert_eq!(operational_audit["decision"], json!("block"));
    assert_eq!(
        operational_audit["refresh_audit"]["minimum_elapsed_hours"],
        json!(24)
    );
    assert!(
        operational_audit["operational_blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|blocker| blocker
                .as_str()
                .unwrap()
                .contains("provider delivery attempt"))
    );
    assert!(
        operational_audit["non_claims"]
            .as_array()
            .unwrap()
            .iter()
            .any(|non_claim| non_claim
                .as_str()
                .unwrap()
                .contains("not wall-clock recurrence proof"))
    );
    let delivery_attempts = call_mcp_tool(&paths, "channel_delivery_list", json!({})).unwrap();
    assert!(delivery_attempts.as_array().unwrap().is_empty());

    fs::write(
        paths.home.join("arcwell-policy.toml"),
        r#"
[[rules]]
id = "allow-mcp-job-weekly-report-email-send"
effect = "allow"
action = "channel.send"
package = "arcwell-email"
provider = "cloudflare_email"
source = "job_weekly_report_delivery"
channel = "email"
subject = "email:job-proof@example.com"
target = "job-proof@example.com"
reason = "allow controlled MCP weekly job report email send"
priority = 10
"#,
    )
    .unwrap();
    let api = mock_base_server(
        r#"{"success":true,"result":{"id":"mcp_job_weekly_email_ok"}}"#,
        "application/json",
    );
    let sent_delivery = call_mcp_tool(
        &paths,
        "job_weekly_report_delivery_send",
        json!({
            "delivery_id": prepared_delivery["delivery"]["id"],
            "email_account_id": "acct123",
            "email_api_token": "TOKEN",
            "email_from": "agent@example.com",
            "api_base": api
        }),
    )
    .unwrap();
    assert_eq!(sent_delivery["delivery"]["status"], json!("sent"));
    assert_eq!(sent_delivery["channel_delivery_attempt"]["ok"], json!(true));
    assert_eq!(sent_delivery["idempotent_replay"], json!(false));
    let delivery_attempts = call_mcp_tool(&paths, "channel_delivery_list", json!({})).unwrap();
    assert_eq!(delivery_attempts.as_array().unwrap().len(), 1);

    let post_send_audit = call_mcp_tool(
        &paths,
        "job_operational_audit",
        json!({
            "profile_id": profile_id,
            "scope": "London agent platform roles",
            "min_elapsed_hours": 0
        }),
    )
    .unwrap();
    assert_eq!(post_send_audit["decision"], json!("block"));
    let provider_gate = post_send_audit["gates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|gate| gate["name"] == json!("provider_delivery"))
        .unwrap();
    assert_eq!(provider_gate["decision"], json!("pass"));
    assert!(
        !post_send_audit["operational_blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|blocker| blocker
                .as_str()
                .unwrap()
                .contains("provider delivery attempt"))
    );
}
