use super::*;

pub(crate) fn mcp_tools() -> Vec<Value> {
    vec![
        tool("arcwell_health", "Read local arcwell health.", []),
        tool(
            "provider_credential_probe",
            "Probe configured provider credentials with cheap policy/cost-gated live endpoint checks and write source-health rows.",
            [(
                "providers",
                "string",
                "Optional comma-separated providers. Defaults to github, openai, brave, cloudflare.",
            )],
        ),
        tool("profile_list", "List profile items.", []),
        tool(
            "profile_search",
            "Search profile items.",
            [("query", "string", "Search query.")],
        ),
        tool(
            "profile_set",
            "Set a profile item.",
            [
                ("key", "string", "Profile key."),
                ("value", "string", "Profile value."),
            ],
        ),
        tool(
            "memory_search",
            "Search personal memories.",
            [("query", "string", "Search query.")],
        ),
        tool(
            "memory_add",
            "Add a simple memory.",
            [("text", "string", "Memory text.")],
        ),
        tool(
            "mem0_add",
            "Add a personal memory through Arcwell Memory with optional inference.",
            [("text", "string", "Memory text or conversation snippet.")],
        ),
        tool(
            "mem0_search",
            "Search personal memory through Arcwell Memory hybrid retrieval.",
            [("query", "string", "Memory search query.")],
        ),
        tool(
            "mem0_update",
            "Update an Arcwell Memory entry by id.",
            [
                ("id", "string", "Memory id."),
                ("text", "string", "New memory text."),
            ],
        ),
        tool(
            "mem0_delete",
            "Delete an Arcwell Memory entry by id.",
            [("id", "string", "Memory id.")],
        ),
        tool(
            "mem0_history",
            "Read Arcwell Memory history for a memory id.",
            [("id", "string", "Memory id.")],
        ),
        tool(
            "mem0_forget_user",
            "Delete all Arcwell Memory entries for the configured or supplied user id.",
            [],
        ),
        tool(
            "memory_recall_context",
            "Retrieve concise profile and Arcwell Memory context for a prompt or hook.",
            [("query", "string", "Prompt, task, or recall query.")],
        ),
        tool(
            "memory_capture",
            "Capture text into reviewable Arcwell Memory candidates or auto-apply non-sensitive facts.",
            [("text", "string", "Conversation or note text.")],
        ),
        tool(
            "memory_lifecycle_events",
            "List Arcwell Memory lifecycle recall/capture events.",
            [],
        ),
        tool(
            "memory_extract_candidates",
            "Extract reviewable personal-memory candidates from text.",
            [("text", "string", "Conversation or note text.")],
        ),
        tool(
            "memory_dream_reconcile",
            "Run a local memory reconciliation pass that removes exact duplicates and creates reviewable conflict candidates.",
            [],
        ),
        tool("candidate_list", "List review candidates.", []),
        tool(
            "candidate_apply",
            "Apply a review candidate.",
            [("id", "string", "Candidate id.")],
        ),
        tool(
            "backup_create",
            "Create a local backup snapshot with an explicit X recovery/portable-export summary in the manifest.",
            [],
        ),
        tool(
            "backup_verify",
            "Verify the latest local backup snapshot, including the recorded X recovery/portable-export manifest summary.",
            [],
        ),
        tool(
            "worker_run_once",
            "Process pending local wiki/source adapter jobs once.",
            [],
        ),
        tool(
            "edge_event_enqueue",
            "Add a bounded Cloudflare/edge inbox event for local draining.",
            [
                ("source", "string", "Event source."),
                ("idempotency_key", "string", "Replay/idempotency key."),
            ],
        ),
        tool("edge_event_lease", "Lease the next edge inbox event.", []),
        tool(
            "edge_event_ack",
            "Acknowledge a leased edge inbox event.",
            [("id", "string", "Edge event id.")],
        ),
        tool(
            "edge_event_nack",
            "Reject a leased edge inbox event for retry or dead-letter.",
            [
                ("id", "string", "Edge event id."),
                ("error", "string", "Failure reason."),
            ],
        ),
        tool(
            "edge_event_dead_letter",
            "Mark an unrecoverable edge inbox event as dead-lettered.",
            [
                ("id", "string", "Edge event id."),
                ("error", "string", "Failure reason."),
            ],
        ),
        tool("edge_event_list", "List edge inbox events.", []),
        tool("cost_summary", "Read model/tool cost summary.", []),
        tool(
            "cost_policy_set",
            "Set a global, package, provider, or source cost policy.",
            [
                ("scope", "string", "global, package, provider, or source."),
                ("key", "string", "Policy key, or * for global."),
            ],
        ),
        tool("cost_policy_list", "List cost policies.", []),
        tool(
            "cost_check",
            "Check whether a projected provider operation is allowed by cost policy.",
            [
                ("package", "string", "Arcwell package name."),
                ("provider", "string", "Provider name."),
            ],
        ),
        tool(
            "policy_check",
            "Evaluate and audit an Arcwell policy request.",
            [(
                "action",
                "string",
                "Policy action, such as provider.network.",
            )],
        ),
        tool(
            "policy_explain",
            "Explain matching Arcwell policy rules without writing a decision record.",
            [(
                "action",
                "string",
                "Policy action, such as provider.network.",
            )],
        ),
        tool(
            "policy_decision_list",
            "List recent Arcwell policy decisions.",
            [],
        ),
        tool("policy_rule_list", "List active Arcwell policy rules.", []),
        tool(
            "policy_override_allow",
            "Create a temporary allow rule in arcwell-policy.toml.",
            [
                ("action", "string", "Policy action to allow."),
                ("reason", "string", "Human reason for the override."),
                ("expires_at", "string", "RFC3339 expiration timestamp."),
            ],
        ),
        tool(
            "policy_approval_list",
            "List pending or resolved Arcwell policy approvals.",
            [],
        ),
        tool(
            "policy_approval_approve",
            "Mark a pending Arcwell policy approval as approved.",
            [("id", "string", "Policy approval id.")],
        ),
        tool(
            "policy_approval_reject",
            "Mark a pending Arcwell policy approval as rejected.",
            [("id", "string", "Policy approval id.")],
        ),
        tool(
            "research_capabilities",
            "Read the agent-facing deep-research capability contract, including rich extraction support, host-search proof flow, role artifact requirements, and editorial provider boundaries.",
            [],
        ),
        tool(
            "commerce_research_capabilities",
            "Read the anti-mirage qualified-commerce capability contract. Current status is bounded production-data proof for a small supervised browser packet; it explicitly does not claim autonomous broad live browser shopping.",
            [],
        ),
        tool_with_schema(
            "commerce_run_config_set",
            "Record or update a qualified-commerce run config for an existing research_run. This is local durable config only, not proof that live search ran.",
            commerce_run_config_tool_properties(),
            &["run_id", "domain_profile"],
        ),
        tool(
            "commerce_run_config",
            "Read the qualified-commerce run config for an existing research_run.",
            [("run_id", "string", "Research run id.")],
        ),
        tool_with_schema(
            "commerce_candidate_add",
            "Record one qualified-commerce candidate with an exact normalized item key and variant key. Availability is not proven until commerce_availability_proof_add records visible page evidence with artifact provenance.",
            commerce_candidate_tool_properties(),
            &[
                "run_id",
                "domain",
                "source_url",
                "retailer_or_provider",
                "title",
                "normalized_item_key",
                "variant_key",
            ],
        ),
        tool(
            "commerce_candidates",
            "List qualified-commerce candidates for one research run.",
            [("run_id", "string", "Research run id.")],
        ),
        tool_with_schema(
            "commerce_availability_proof_add",
            "Record visible page evidence for one candidate's exact variant availability. This rejects wrong-run candidates, wrong variants, and available claims without visible evidence or artifact provenance.",
            commerce_availability_proof_tool_properties(),
            &[
                "run_id",
                "candidate_id",
                "proof_method",
                "variant_key",
                "variant_label",
                "availability_state",
            ],
        ),
        tool(
            "commerce_availability_proofs",
            "List exact-variant availability proofs for one qualified-commerce run.",
            [("run_id", "string", "Research run id.")],
        ),
        tool_with_schema(
            "commerce_rendered_page_check",
            "Record a host-supplied rendered page snapshot for one candidate and conservatively classify selector-backed exact-variant availability. Arcwell performs no browser or network fetch for this tool.",
            commerce_rendered_page_check_tool_properties(),
            &[
                "run_id",
                "candidate_id",
                "variant_key",
                "variant_label",
                "requested_url",
            ],
        ),
        tool_with_schema(
            "commerce_context_fact_add",
            "Record one redacted private-context fact used by qualified-commerce ranking, including source family and confidence.",
            commerce_context_fact_tool_properties(),
            &[
                "run_id",
                "fact_key",
                "fact_kind",
                "redacted_value",
                "source_family",
            ],
        ),
        tool(
            "commerce_context_facts",
            "List redacted private-context facts recorded for one qualified-commerce run.",
            [("run_id", "string", "Research run id.")],
        ),
        tool(
            "commerce_context_packet_compile",
            "Compile a redacted qualified-commerce context packet artifact from recorded private-context facts.",
            [("run_id", "string", "Research run id.")],
        ),
        tool_with_schema(
            "commerce_verification_attempt_add",
            "Record a browser/search verification attempt, including blocked/manual states. This is attempt evidence, not an availability proof by itself.",
            commerce_verification_attempt_tool_properties(),
            &["run_id", "candidate_id", "method", "result"],
        ),
        tool(
            "commerce_verification_attempts",
            "List verification attempts for one qualified-commerce run.",
            [("run_id", "string", "Research run id.")],
        ),
        tool_with_schema(
            "commerce_report_judgment_add",
            "Record an acceptance/revision/rejection judgment for a qualified-commerce report. Accept is rejected while blocking findings remain.",
            commerce_report_judgment_tool_properties(),
            &["run_id", "decision"],
        ),
        tool(
            "commerce_report_judgments",
            "List report judgments for one qualified-commerce run.",
            [("run_id", "string", "Research run id.")],
        ),
        tool(
            "commerce_report_compile",
            "Compile a gated qualified-commerce report artifact and judgment from recorded candidates, exact-variant proofs, source cards, and context facts.",
            [("run_id", "string", "Research run id.")],
        ),
        tool_with_schema(
            "job_profile_add",
            "Record or update a local job-hunting candidate profile. This is durable profile context, not a market search.",
            job_profile_tool_properties(),
            &["label"],
        ),
        tool(
            "job_profiles",
            "List local job-hunting candidate profiles.",
            [],
        ),
        tool_with_schema(
            "job_import_batch",
            "Import a reviewed job-hunting JSON packet into the durable local ledger. This records supplied facts only; it is not live discovery or freshness proof.",
            job_import_batch_tool_properties(),
            &["batch"],
        ),
        tool_with_schema(
            "job_evidence_add",
            "Record one candidate evidence card with explicit visibility and confidence. Private-blocked cards are not usable in application packets.",
            job_evidence_tool_properties(),
            &[
                "profile_id",
                "title",
                "evidence_type",
                "visibility",
                "summary",
                "confidence",
                "safe_application_text",
            ],
        ),
        tool(
            "job_evidence_list",
            "List candidate evidence cards for one profile.",
            [("profile_id", "string", "Job candidate profile id.")],
        ),
        tool(
            "job_evidence_review_report",
            "Compile a local evidence-readiness report for one candidate profile, including privacy, visibility, proof, and claim-mapping blockers before application material is trusted.",
            [("profile_id", "string", "Job candidate profile id.")],
        ),
        tool_with_schema(
            "job_privacy_check",
            "Run and persist a privacy check over proposed resume, outreach, packet, or report text.",
            job_privacy_check_tool_properties(),
            &["artifact_type", "text"],
        ),
        tool_with_schema(
            "job_role_add",
            "Record one job role card with source confidence. This does not claim broad search coverage or freshness beyond the supplied source.",
            job_role_tool_properties(),
            &[
                "company",
                "role_title",
                "source_family",
                "source_url",
                "source_confidence",
                "posting_freshness",
                "current_status",
            ],
        ),
        tool("job_roles", "List durable job role cards.", []),
        tool_with_schema(
            "job_score_add",
            "Record an auditable numeric fit score for a role. High evidence_fit requires linked evidence cards and source confidence gates the tier.",
            job_score_tool_properties(),
            &[
                "role_id",
                "profile_id",
                "role_fit",
                "domain_fit",
                "evidence_fit",
                "geo_work_fit",
                "stage_fit",
                "practical_odds",
                "interest_energy",
                "explanation",
            ],
        ),
        tool(
            "job_shortlist",
            "Compile a local shortlist from durable role cards and latest fit scores. Stale/closed roles are blocked in the effective shortlist even if an older score was high.",
            [("profile_id", "string", "Job candidate profile id.")],
        ),
        tool_with_schema(
            "job_outreach_readiness",
            "Classify scored roles for outreach readiness from approved packets, privacy rechecks, and warm-intro/contact paths. Public-only contacts are not warm intros; this does not send outreach.",
            job_outreach_readiness_tool_properties(),
            &["profile_id"],
        ),
        tool_with_schema(
            "job_company_targets",
            "Compile a local-proof company scouting report from durable company cards and public evidence tags. This does not claim current openings or create application-ready role cards.",
            job_company_targets_tool_properties(),
            &["profile_id"],
        ),
        tool_with_schema(
            "job_packet_create",
            "Create a privacy-checked draft application packet for a confirmed role. This rejects private terms, local proof links, and unsupported evidence.",
            job_packet_tool_properties(),
            &[
                "role_id",
                "profile_id",
                "evidence_card_ids",
                "resume_emphasis",
                "outreach_note",
            ],
        ),
        tool_with_schema(
            "job_packet_approve",
            "Approve a privacy-passing draft application packet for later application or outreach recording. This records human review intent; it does not send anything.",
            job_packet_approve_tool_properties(),
            &["packet_id", "reviewer_note"],
        ),
        tool_with_schema(
            "job_packet_export",
            "Export an approved privacy-passing application packet to a local Markdown file. This does not send or record an application.",
            job_packet_export_tool_properties(),
            &["packet_id", "out_dir"],
        ),
        tool_with_schema(
            "job_packet_export_set",
            "Export approved privacy-passing application packets for one profile to local Markdown files plus a manifest. This does not create Google Docs, send, submit, or record applications.",
            job_packet_export_set_tool_properties(),
            &["profile_id", "packet_ids", "out_dir"],
        ),
        tool_with_schema(
            "job_application_record",
            "Record a user-confirmed application status for one role.",
            job_application_tool_properties(),
            &["role_id", "status"],
        ),
        tool_with_schema(
            "job_source_refresh",
            "Refresh one configured job source from caller-supplied page text/html or explicit fetch_live network access. Writes source health, accepted roles/company cards, and stale events for missing previously linked roles.",
            job_source_refresh_tool_properties(),
            &["source_id"],
        ),
        tool_with_schema(
            "job_radar_schedule",
            "Schedule recurring job radar refresh for configured source ids, optionally carrying report-delivery metadata to the worker. Replay snapshots provide local proof; fetch_live=true is provider-policy and cost gated. This is local scheduled proof until wall-clock/live recurrence is proven.",
            job_radar_schedule_tool_properties(),
            &["profile_id", "scope", "source_ids"],
        ),
        tool_with_schema(
            "job_radar_enqueue",
            "Enqueue one job radar refresh job for configured source ids, optionally carrying report-delivery metadata to the worker. Replay snapshots provide local proof; fetch_live=true is provider-policy and cost gated.",
            job_radar_enqueue_tool_properties(),
            &["profile_id", "scope", "source_ids"],
        ),
        tool_with_schema(
            "job_refresh_manual",
            "Reconcile a caller-supplied manual job refresh into new/unchanged/stale/closed/promoted/demoted role events. This is not a source crawler.",
            job_manual_refresh_tool_properties(),
            &["profile_id", "scope"],
        ),
        tool_with_schema(
            "job_refresh_audit",
            "Audit durable job refresh runs for the two-refresh/elapsed-time transition proof gate. This reads existing rows only and does not fetch sources.",
            job_refresh_audit_tool_properties(),
            &["profile_id", "scope"],
        ),
        tool_with_schema(
            "job_operational_audit",
            "Compile a read-only job-hunting operational readiness audit from durable state. It reports blockers for recurrence, provider delivery, warm routes, source freshness, packets, applications, and privacy; it does not fetch, send, or submit.",
            job_operational_audit_tool_properties(),
            &["profile_id", "scope"],
        ),
        tool(
            "job_weekly_report",
            "Compile a local-proof weekly report from durable role, score, source-health, and application state.",
            [
                ("profile_id", "string", "Job candidate profile id."),
                ("scope", "string", "Report scope label."),
            ],
        ),
        tool_with_schema(
            "job_weekly_report_delivery_prepare",
            "Prepare a weekly job report for an authorized channel subject after privacy checks. This writes a prepared channel message only; it does not call provider APIs or send.",
            job_weekly_report_delivery_prepare_tool_properties(),
            &["report_id", "channel", "subject", "target"],
        ),
        tool_with_schema(
            "job_weekly_report_delivery_send",
            "Send a prepared weekly job report through an authorized provider channel after rechecking authorization and privacy. This records a provider delivery attempt; it does not schedule recurrence or submit applications.",
            job_weekly_report_delivery_send_tool_properties(),
            &["delivery_id"],
        ),
        tool_with_schema(
            "job_weekly_report_deliveries",
            "List weekly job report delivery rows and their prepared/sent/failed/blocked state.",
            job_weekly_report_deliveries_tool_properties(),
            &[],
        ),
        tool(
            "research_plan",
            "Create a research plan using local wiki context and suggested host-native searches.",
            [("query", "string", "Research question or topic.")],
        ),
        tool(
            "research_web_search",
            "Run optional daemon-side web search with provider=brave, openai, or perplexity. provider=host returns an instruction error.",
            [("query", "string", "Search query.")],
        ),
        tool(
            "research_workflow_create",
            "Create a daemon-tracked deep research workflow. Compatibility alias for research_run.",
            [("query", "string", "Research question or topic.")],
        ),
        tool(
            "research_run",
            "Start a daemon-tracked deep research run with orchestrator, scout, corpus, extractor, skeptic, synthesizer, and auditor tasks.",
            [("query", "string", "Research question or topic.")],
        ),
        tool(
            "research_status",
            "Read durable status and task counts for one deep research run.",
            [("run_id", "string", "Research run id.")],
        ),
        tool(
            "research_read",
            "Read one deep research run, its tasks, and its final result page when present.",
            [("run_id", "string", "Research run id.")],
        ),
        tool(
            "research_audit_run",
            "Audit a deep research run by id using its persisted query.",
            [("run_id", "string", "Research run id.")],
        ),
        tool(
            "research_stop",
            "Stop a deep research run and cancel pending role tasks.",
            [("run_id", "string", "Research run id.")],
        ),
        tool(
            "research_sources",
            "List source-ledger records linked to one deep research run.",
            [("run_id", "string", "Research run id.")],
        ),
        tool(
            "research_source_add",
            "Add or update a research source candidate and link it to a deep research run.",
            [
                ("run_id", "string", "Research run id."),
                ("title", "string", "Source title."),
                ("url", "string", "Canonical source URL when available."),
            ],
        ),
        tool(
            "research_source_card_link",
            "Link an existing source card to a deep research run so retrieval and audit work by run id.",
            [
                ("run_id", "string", "Research run id."),
                ("source_card_id", "string", "Source card id."),
            ],
        ),
        tool(
            "research_extraction_prompt",
            "Build a bounded claim-extraction prompt and JSON schema for a run-linked source card.",
            [
                ("run_id", "string", "Research run id."),
                ("source_card_id", "string", "Source card id."),
            ],
        ),
        tool(
            "research_claims_ingest",
            "Validate and ingest model-produced structured claims for a run-linked source card.",
            [
                ("run_id", "string", "Research run id."),
                ("source_card_id", "string", "Source card id."),
                ("output_json", "string", "Model output JSON."),
            ],
        ),
        tool(
            "research_claims",
            "List structured claims extracted for a deep research run.",
            [("run_id", "string", "Research run id.")],
        ),
        tool(
            "research_clusters",
            "Build deterministic thematic clusters from extracted research claims.",
            [("run_id", "string", "Research run id.")],
        ),
        tool(
            "research_skeptic_pass",
            "Run mandatory skeptic checks over linked sources, extracted claims, clusters, and contradictions.",
            [("run_id", "string", "Research run id.")],
        ),
        tool(
            "research_report_compile",
            "Compile a deep research report from linked sources, extracted claims, clusters, skeptic findings, and audit results.",
            [
                ("run_id", "string", "Research run id."),
                (
                    "saturation_reason",
                    "string",
                    "Why the run stopped or is ready to report.",
                ),
            ],
        ),
        tool_with_schema(
            "research_convergence_start",
            "Start the iterated epistemic convergence loop with one inspectable iteration.",
            research_convergence_tool_properties(),
            &["run_id"],
        ),
        tool_with_schema(
            "research_convergence_step",
            "Run exactly one convergence iteration: compile statements, pressure-test, disprove, revise, fact-check, and snapshot.",
            research_convergence_tool_properties(),
            &["run_id"],
        ),
        tool_with_schema(
            "research_convergence_run",
            "Run convergence iterations until the configured stop rule settles or stops incomplete. Optional editorial_provider runs a model-backed citation/evaluator gate after terminal convergence.",
            research_convergence_tool_properties(),
            &["run_id"],
        ),
        tool_with_schema(
            "research_convergence_enqueue",
            "Queue a resumable worker-run convergence job for long-running research. Optional editorial_provider runs the model-backed citation/evaluator gate from the worker.",
            research_convergence_tool_properties(),
            &["run_id"],
        ),
        tool(
            "research_convergence_status",
            "Read current convergence status, latest snapshot, current statements, open challenges, and strong refutations.",
            [("run_id", "string", "Research run id.")],
        ),
        tool(
            "research_iterations",
            "List convergence iterations for a research run.",
            [("run_id", "string", "Research run id.")],
        ),
        tool(
            "research_iteration_read",
            "Read one convergence iteration by id.",
            [("id", "string", "Research iteration id.")],
        ),
        tool(
            "research_statements",
            "List convergence statements for a research run.",
            [("run_id", "string", "Research run id.")],
        ),
        tool(
            "research_challenges",
            "List red-team challenges generated for convergence statements.",
            [("run_id", "string", "Research run id.")],
        ),
        tool(
            "research_convergence_host_search_tasks",
            "List exact host-native search tasks required by convergence challenges, with pending/recorded proof status.",
            [("run_id", "string", "Research run id.")],
        ),
        tool_with_schema(
            "research_convergence_provider_search",
            "Run policy/cost-gated provider search for pending convergence host-search tasks and record auditable proof.",
            research_convergence_provider_search_tool_properties(),
            &["run_id", "provider"],
        ),
        tool(
            "research_disproofs",
            "List verifier disproof records generated during convergence.",
            [("run_id", "string", "Research run id.")],
        ),
        tool(
            "research_revisions",
            "List revisions applied because of convergence disproofs.",
            [("run_id", "string", "Research run id.")],
        ),
        tool(
            "research_fact_checks",
            "List active fact-check records for convergence statements.",
            [("run_id", "string", "Research run id.")],
        ),
        tool_with_schema(
            "research_active_fact_check",
            "Extract factual sentences from a report artifact, verify them against current convergence statements, and create citation-gap challenges for unsupported high-impact sentences.",
            research_active_fact_check_tool_properties(),
            &["run_id"],
        ),
        tool_with_schema(
            "research_convergence_close_loop",
            "Compile/check a convergence report, run active fact-checking, optionally run provider fallback for pending citation-gap searches, rerun convergence, and return explicit closure blockers.",
            research_convergence_close_loop_tool_properties(),
            &["run_id"],
        ),
        tool(
            "research_convergence_snapshots",
            "List convergence snapshots and stop-rule metrics.",
            [("run_id", "string", "Research run id.")],
        ),
        tool(
            "research_convergence_report_compile",
            "Compile an analyst-readable convergence report artifact plus report judgment.",
            [("run_id", "string", "Research run id.")],
        ),
        tool(
            "research_report_judgments",
            "List final report judgments and blocking/non-blocking findings for a research run.",
            [("run_id", "string", "Research run id.")],
        ),
        tool(
            "research_tasks",
            "List daemon-tracked research tasks for a run.",
            [("run_id", "string", "Research run id.")],
        ),
        tool_with_schema(
            "research_role_start",
            "Record the start of a host or Codex subagent role execution for a deep research run. Optional host_thread_id/host_subagent_id/tool_surface fields make fresh in-app Codex orchestration auditable.",
            json!({
                "run_id": string_schema("Research run id."),
                "role": string_schema("Research role name, such as research-scout, corpus-builder, source-extractor, skeptic, synthesizer, or auditor."),
                "host": string_schema("Host runtime. Defaults to codex."),
                "execution_mode": enum_schema("Execution mode. Defaults to host_sequential; use codex_subagent_live when a real Codex subagent is spawned.", &["host_sequential", "codex_subagent_live", "simulated_test"]),
                "host_thread_id": string_schema("Optional host thread/session id for provenance."),
                "host_subagent_id": string_schema("Optional host subagent id for provenance."),
                "tool_surface": string_schema("Optional surface used by the role, such as mcp, cli, host-search, or codex-subagent."),
                "prompt_version": string_schema("Prompt/instruction version. Defaults to v1."),
                "prompt_hash": string_schema("Optional prompt hash when available."),
                "input_artifact_ids": array_schema("Optional input artifact ids supplied to this role.", string_schema("Research artifact id."))
            }),
            &["run_id", "role"],
        ),
        tool_with_schema(
            "research_role_finish",
            "Record completion, rejection, cancellation, or failure of a research role execution. IMPORTANT: status=completed requires output_artifact_id, and that artifact must be linked to the same role_run_id.",
            json!({
                "role_run_id": string_schema("Research role run id."),
                "status": enum_schema("Role terminal status.", &["completed", "failed", "rejected", "cancelled"]),
                "output_artifact_id": string_schema("Required when status=completed; must identify an artifact created with this role_run_id."),
                "error_kind": string_schema("Failure/rejection category when status is failed, rejected, or cancelled."),
                "error_message": string_schema("Redacted failure/rejection notes when status is failed, rejected, or cancelled.")
            }),
            &["role_run_id", "status"],
        ),
        tool(
            "research_role_runs",
            "List host/subagent role execution records for one deep research run.",
            [("run_id", "string", "Research run id.")],
        ),
        tool_with_schema(
            "research_artifact_add",
            "Record an auditable research artifact such as a source map, role output, rejected proposal, evidence pack, or synthesis draft. Use role_run_id before research_role_finish so completed roles can point at their accepted output.",
            json!({
                "run_id": string_schema("Research run id."),
                "artifact_type": string_schema("Artifact type, such as source_map, role_output, evidence_pack, synthesis_draft, evaluator_report, or rejected_proposal."),
                "title": string_schema("Artifact title."),
                "body": string_schema("Artifact body, normally Markdown or JSON text."),
                "role_run_id": string_schema("Optional research role run id this artifact belongs to."),
                "metadata": object_schema("Optional structured metadata object.", json!({}), &[]),
                "metadata_json": string_schema("Optional metadata JSON string for CLI parity; metadata object is preferred.")
            }),
            &["run_id", "artifact_type", "title", "body"],
        ),
        tool(
            "research_artifacts",
            "List research artifacts for one deep research run.",
            [("run_id", "string", "Research run id.")],
        ),
        tool(
            "research_artifact_read",
            "Read one research artifact by id.",
            [("id", "string", "Research artifact id.")],
        ),
        tool_with_schema(
            "research_host_search_record",
            "Record auditable host-native search proof and link selected results into the research source ledger. Use after running Codex/web host search; results must be objects, not strings. Example result: {\"rank\":1,\"title\":\"Official docs\",\"url\":\"https://example.com\",\"snippet\":\"Relevant passage\",\"selected_for_ingest\":true,\"source_family_guess\":\"official-docs\"}.",
            json!({
                "run_id": string_schema("Research run id."),
                "query": string_schema("Host-native search query."),
                "host": string_schema("Host runtime. Defaults to codex."),
                "tool_surface": string_schema("Host search surface. Defaults to host-native."),
                "role_run_id": string_schema("Optional research role run id that performed the search."),
                "query_intent": string_schema("Optional purpose of the search, such as source-discovery, contradiction-check, or freshness-check."),
                "requested_recency": integer_schema("Optional requested recency window in days."),
                "requested_domains": array_schema("Optional requested domain filters.", string_schema("Domain name.")),
                "cost_decision_id": string_schema("Optional cost/policy decision id."),
                "results": array_schema(
                    "Structured host search results in ranked order.",
                    object_schema(
                        "One host search result.",
                        json!({
                            "rank": integer_schema("1-based rank from the host search result list."),
                            "title": string_schema("Result title."),
                            "url": string_schema("Result URL."),
                            "snippet": string_schema("Search snippet or short host-provided summary."),
                            "published_at": string_schema("Optional publication/update date when visible."),
                            "source_family_guess": string_schema("Optional source family guess, such as official-docs, paper, company-blog, news, forum, or repository."),
                            "provider_metadata": object_schema("Optional host/provider metadata. Do not include secrets.", json!({}), &[]),
                            "selected_for_ingest": boolean_schema("Whether Arcwell should create/link a source-ledger candidate for this result.")
                        }),
                        &["rank", "title", "url", "snippet", "selected_for_ingest"]
                    )
                )
            }),
            &["run_id", "query", "results"],
        ),
        tool(
            "research_host_searches",
            "List host-native search proof records for one deep research run.",
            [("run_id", "string", "Research run id.")],
        ),
        tool(
            "research_host_search_read",
            "Read one host-native search proof record by id.",
            [("id", "string", "Host search id.")],
        ),
        tool_with_schema(
            "research_document_extract",
            "Extract a local CSV, TSV, XLSX/XLSM, or PDF into auditable document/table/span artifacts with byte hashes and anchors. PDF tables are layout heuristics unless manually corroborated; XLSX formulas are preserved as untrusted text and not evaluated.",
            json!({
                "run_id": string_schema("Research run id."),
                "path": string_schema("Local document path."),
                "media_type": string_schema("Optional media type override: text/csv, text/tab-separated-values, application/pdf, application/vnd.openxmlformats-officedocument.spreadsheetml.sheet, or application/xlsx."),
                "research_source_id": string_schema("Optional linked research source id."),
                "source_card_id": string_schema("Optional linked source card id.")
            }),
            &["run_id", "path"],
        ),
        tool(
            "research_documents",
            "List extracted document/table/span artifacts for one deep research run.",
            [("run_id", "string", "Research run id.")],
        ),
        tool(
            "research_document_read",
            "Read one extracted research document artifact by id.",
            [("id", "string", "Research document id.")],
        ),
        tool(
            "research_evidence_pack",
            "Build a deterministic evidence-pack artifact for model-backed editorial drafting and evaluation.",
            [("run_id", "string", "Research run id.")],
        ),
        tool_with_schema(
            "research_editorial_invoke",
            "Invoke a live OpenAI or mock model-backed editorial/eval stage and record its inspectable output artifact. Use mock for deterministic tests; OpenAI requires OPENAI_API_KEY or api_key plus policy and cost approval.",
            json!({
                "run_id": string_schema("Research run id."),
                "stage": enum_schema("Editorial/eval stage.", &["evidence_pack", "editorial_drafter", "citation_verifier", "adversarial_evaluator", "final_audit"]),
                "model_provider": enum_schema("Provider. Defaults to openai; use mock for deterministic local tests.", &["openai", "mock"]),
                "model_name": string_schema("Optional model name. Defaults to ARCWELL_RESEARCH_EDITORIAL_MODEL or gpt-5.5 for OpenAI, mock-editorial for mock."),
                "prompt_version": string_schema("Prompt version. Defaults to v1."),
                "input_artifact_id": string_schema("Optional input artifact id. If omitted, Arcwell builds an evidence pack."),
                "endpoint": string_schema("Optional OpenAI-compatible endpoint override."),
                "api_key": string_schema("Optional API key for live provider invocation. Prefer environment/secret configuration."),
                "timeout_seconds": integer_schema("Optional timeout, clamped by Arcwell.")
            }),
            &["run_id", "stage"],
        ),
        tool_with_schema(
            "research_editorial_record",
            "Record one externally produced model-backed editorial, citation-verifier, or adversarial-evaluator run. Prefer research_editorial_invoke when Arcwell should call the provider itself.",
            json!({
                "run_id": string_schema("Research run id."),
                "stage": enum_schema("Editorial/eval stage.", &["evidence_pack", "editorial_drafter", "citation_verifier", "adversarial_evaluator", "final_audit"]),
                "model_provider": string_schema("Provider name. Defaults to openai."),
                "model_name": string_schema("Model name used for the editorial/eval stage."),
                "prompt_version": string_schema("Prompt version. Defaults to v1."),
                "input_artifact_id": string_schema("Optional input artifact id."),
                "output_artifact_id": string_schema("Optional output artifact id."),
                "cost_decision_id": string_schema("Optional cost/policy decision id."),
                "status": enum_schema("Editorial run status.", &["completed", "accepted", "failed", "rejected"]),
                "score": object_schema("Optional structured score object.", json!({}), &[]),
                "error_message": string_schema("Optional redacted failure/rejection message.")
            }),
            &["run_id", "stage", "model_name"],
        ),
        tool(
            "research_editorial_runs",
            "List model-backed editorial/eval run records for one deep research run.",
            [("run_id", "string", "Research run id.")],
        ),
        tool(
            "research_editorial_read",
            "Read one model-backed editorial/eval run record by id.",
            [("id", "string", "Editorial run id.")],
        ),
        tool(
            "research_task_complete",
            "Complete a daemon-tracked research task with notes.",
            [
                ("task_id", "string", "Research task id."),
                ("notes", "string", "Completion notes."),
            ],
        ),
        tool(
            "research_brief_from_wiki",
            "Create a wiki-grounded research brief. By default writes the brief back to the wiki.",
            [("query", "string", "Research question or topic.")],
        ),
        tool(
            "research_audit",
            "Audit local source cards and wiki sources for generated recursion, stale evidence, contradictions, and untrusted text.",
            [("query", "string", "Research question or topic.")],
        ),
        tool("research_runs", "List local research runs.", []),
        tool(
            "project_create",
            "Create a local project record with aliases and summary.",
            [
                ("name", "string", "Project name."),
                ("summary", "string", "Project summary."),
            ],
        ),
        tool("project_list", "List local projects.", []),
        tool(
            "project_resolve",
            "Resolve a natural-language project reference.",
            [("query", "string", "Project reference.")],
        ),
        tool(
            "project_status_record",
            "Record a timestamped manual/durable project status snapshot with provenance. Reserved live-sync sources are rejected.",
            [
                ("project_id", "string", "Project id."),
                ("status", "string", "Project status label."),
                ("summary", "string", "Status summary."),
            ],
        ),
        tool(
            "project_status_sync_record",
            "Record an explicit verified host-thread sync snapshot with a freshness marker after the host has listed/read a matching thread.",
            [
                ("project_id", "string", "Project id."),
                ("status", "string", "Project status label."),
                ("summary", "string", "Status summary."),
                ("host", "string", "Host name: codex or claude."),
                ("thread_id", "string", "Verified host thread id."),
            ],
        ),
        tool(
            "project_status_get",
            "Read a project status report with latest snapshot, live-state availability, and provenance.",
            [("project_id", "string", "Project id.")],
        ),
        tool(
            "controller_route_text",
            "Route an incoming channel message into project status, work control, or queued workflow actions.",
            [
                (
                    "conversation_id",
                    "string",
                    "Channel-local conversation id.",
                ),
                ("sender", "string", "Channel-local sender subject."),
                ("text", "string", "Incoming message text."),
            ],
        ),
        tool(
            "controller_thread_upsert",
            "Sync or register a host thread for controller status and routing.",
            [
                ("host", "string", "Host name, for example codex."),
                ("host_thread_id", "string", "Host-native thread id."),
            ],
        ),
        tool(
            "controller_thread_list",
            "List known controller host threads, optionally filtered by project or status.",
            [],
        ),
        tool(
            "controller_thread_get",
            "Read one known controller host-thread row by Arcwell controller thread id.",
            [("id", "string", "Arcwell controller thread id.")],
        ),
        tool(
            "controller_run_create",
            "Register a controller run for a requested host action.",
            [("requested_action", "string", "Requested action text.")],
        ),
        tool(
            "controller_run_list",
            "List controller runs, optionally filtered by project or status.",
            [],
        ),
        tool(
            "controller_run_get",
            "Read one controller run row by id.",
            [("id", "string", "Controller run id.")],
        ),
        tool(
            "controller_run_update",
            "Update a controller run status after a host adapter creates, sends, stops, or observes work.",
            [
                ("run_id", "string", "Controller run id."),
                ("status", "string", "New controller run status."),
            ],
        ),
        tool(
            "controller_stop",
            "Request cancellation of a controller run; host adapter still must deliver the stop.",
            [
                ("run_id", "string", "Controller run id."),
                ("reason", "string", "Stop reason."),
            ],
        ),
        tool(
            "controller_event_record",
            "Record controller activity from a host adapter or worker.",
            [
                ("event_type", "string", "Controller event type."),
                ("summary", "string", "Event summary."),
            ],
        ),
        tool(
            "controller_event_list",
            "List recent controller events, optionally filtered by run or project.",
            [],
        ),
        tool(
            "controller_pending_list",
            "List queued controller actions waiting for a host adapter or approval.",
            [],
        ),
        tool(
            "controller_pending_resolve",
            "Mark a queued controller action processing, completed, failed, cancelled, expired, or deferred.",
            [
                ("id", "string", "Controller pending action id."),
                ("status", "string", "New pending action status."),
            ],
        ),
        tool(
            "work_run_start",
            "Start a compact work-memory trace for a substantial task.",
            [("goal", "string", "Work goal.")],
        ),
        tool(
            "work_event_record",
            "Append a redacted work event such as command, source, failure, root_cause, validation, or lesson.",
            [
                ("run_id", "string", "Work run id."),
                ("event_type", "string", "Work event type."),
                ("summary", "string", "Compact event summary."),
            ],
        ),
        tool(
            "work_artifact_add",
            "Link a file, command, output, or source locator to a work run.",
            [
                ("run_id", "string", "Work run id."),
                ("artifact_type", "string", "Artifact type."),
                (
                    "locator",
                    "string",
                    "File path, URL, command summary, or other locator.",
                ),
            ],
        ),
        tool(
            "work_link_add",
            "Link a work run to project, source card, wiki page, memory event, cost entry, backup, or generated summary evidence.",
            [
                ("run_id", "string", "Work run id."),
                ("target_type", "string", "Target type."),
                ("target_id", "string", "Target id."),
            ],
        ),
        tool(
            "work_run_finish",
            "Finish a work run. Successful runs require validation evidence.",
            [
                ("run_id", "string", "Work run id."),
                (
                    "status",
                    "string",
                    "success, failed, blocked, or cancelled.",
                ),
                ("outcome", "string", "Final outcome summary."),
            ],
        ),
        tool("work_run_search", "Search work-memory traces.", []),
        tool(
            "work_run_read",
            "Read a work-memory trace with events, artifacts, and links.",
            [("run_id", "string", "Work run id.")],
        ),
        tool(
            "work_run_stale",
            "List active work-memory runs whose updated_at is stale for host follow-up.",
            [],
        ),
        tool(
            "work_follow_up_list",
            "List recorded follow-up items from completed work-memory runs.",
            [],
        ),
        tool(
            "work_consolidation_candidates",
            "List validated project-bound work runs ready for consolidation.",
            [],
        ),
        tool(
            "work_retrieval_context",
            "Build host prompt context for stale work, consolidation candidates, and follow-ups.",
            [("query", "string", "Host retrieval query.")],
        ),
        tool(
            "work_consolidate",
            "Create a project status proposal from work trace evidence without generated-summary-only citations.",
            [("run_id", "string", "Work run id.")],
        ),
        tool(
            "procedure_propose_from_work_run",
            "Create a pending reviewed procedure candidate from validated work-run reusable lessons.",
            [("run_id", "string", "Work run id.")],
        ),
        tool(
            "procedure_candidate_create",
            "Create a pending procedure candidate for explicit review.",
            [
                ("operation", "string", "ADD, UPDATE, or ARCHIVE."),
                ("title", "string", "Procedure title."),
                ("method", "string", "Procedure method text."),
            ],
        ),
        tool(
            "procedure_candidate_list",
            "List reviewable procedure candidates.",
            [],
        ),
        tool(
            "procedure_candidate_apply",
            "Apply an explicitly reviewed procedure candidate.",
            [("id", "string", "Procedure candidate id.")],
        ),
        tool(
            "procedure_candidate_reject",
            "Reject a pending procedure candidate.",
            [("id", "string", "Procedure candidate id.")],
        ),
        tool(
            "procedure_search",
            "Search approved procedural memory. Procedures are not factual source evidence.",
            [],
        ),
        tool(
            "procedure_read",
            "Read a versioned approved procedure and provenance.",
            [("id", "string", "Procedure id.")],
        ),
        tool(
            "procedure_retrieval_context",
            "Build host prompt context from approved procedural memory with freshness/confidence warnings.",
            [("query", "string", "Procedure retrieval query.")],
        ),
        tool(
            "procedure_export_skill",
            "Export an active approved procedure into Arcwell's Codex skill export directory.",
            [
                ("id", "string", "Procedure id."),
                (
                    "skill_name",
                    "string",
                    "Lowercase hyphenated Codex skill name.",
                ),
            ],
        ),
        tool(
            "procedure_curate",
            "Create reviewable merge/no-op candidates for duplicate or stale procedures.",
            [],
        ),
        tool(
            "channel_record",
            "Record an incoming or outgoing channel message with optional project binding.",
            [
                ("channel", "string", "Channel name."),
                ("sender", "string", "Sender identity."),
                ("body", "string", "Message body."),
            ],
        ),
        tool("channel_list", "List recorded channel messages.", []),
        tool(
            "channel_authorize",
            "Authorize a channel subject for project reads, project writes, or sending.",
            [
                ("channel", "string", "Channel name."),
                (
                    "subject",
                    "string",
                    "Channel subject, such as telegram:chat:123.",
                ),
            ],
        ),
        tool(
            "channel_authorizations",
            "List channel authorization policy entries.",
            [],
        ),
        tool(
            "channel_delivery_list",
            "List channel delivery attempts, optionally filtered by message_id.",
            [],
        ),
        tool(
            "telegram_drain_edge_events",
            "Drain Telegram edge inbox events into local channel messages.",
            [],
        ),
        tool(
            "telegram_send_message",
            "Send a Telegram message with MarkdownV2 escaping and record the outgoing channel message.",
            [
                ("chat_id", "string", "Telegram chat id."),
                ("text", "string", "Message text."),
            ],
        ),
        tool(
            "email_drain_edge_events",
            "Drain Cloudflare Email Routing edge events into local email channel messages and source cards.",
            [],
        ),
        tool(
            "email_poll_edge",
            "Poll the remote edge inbox and then drain Cloudflare Email Routing events into local email channel messages and source cards.",
            [],
        ),
        tool(
            "email_send_message",
            "Send a rich or plain email through Cloudflare Email Service and record delivery state.",
            [
                ("to", "string", "Recipient email address."),
                ("subject", "string", "Email subject."),
                ("text", "string", "Plain-text email body."),
            ],
        ),
        tool(
            "email_reply_message",
            "Reply to a recorded incoming email channel message through Cloudflare Email Service.",
            [
                ("message_id", "string", "Incoming email channel message id."),
                ("text", "string", "Plain-text reply body."),
            ],
        ),
        tool(
            "digest_candidate_create",
            "Create an interestingness/digest candidate from source cards.",
            [
                ("topic", "string", "Candidate topic."),
                (
                    "source_card_ids",
                    "array",
                    "Source card ids supporting the candidate.",
                ),
            ],
        ),
        tool(
            "digest_candidate_list",
            "List interestingness/digest candidates.",
            [],
        ),
        tool(
            "digest_candidate_approve",
            "Mark a sourced digest candidate as human-reviewed and approved for later delivery gating.",
            [
                ("id", "string", "Digest candidate id."),
                ("reviewed_by", "string", "Reviewer label."),
                ("note", "string", "Review note or rationale."),
            ],
        ),
        tool(
            "digest_candidate_reject",
            "Reject a sourced digest candidate and keep the review decision durable.",
            [
                ("id", "string", "Digest candidate id."),
                ("reviewed_by", "string", "Reviewer label."),
                ("note", "string", "Review note or rejection rationale."),
            ],
        ),
        tool(
            "digest_candidate_delivery_check",
            "Check whether a digest candidate passes review and policy gates before any delivery attempt.",
            [
                ("id", "string", "Digest candidate id."),
                (
                    "channel",
                    "string",
                    "Delivery channel such as telegram or email.",
                ),
                (
                    "subject",
                    "string",
                    "Authorized delivery subject, such as telegram:chat:123.",
                ),
                ("target", "string", "Optional delivery target/destination."),
            ],
        ),
        tool(
            "digest_candidate_deliveries",
            "List durable digest delivery ledger rows, optionally filtered by digest candidate id.",
            [(
                "candidate_id",
                "string",
                "Optional digest candidate id filter.",
            )],
        ),
        tool(
            "digest_candidate_deliver_telegram",
            "Deliver an approved digest candidate to Telegram after review, policy, channel authorization, cost, and provider-send gates.",
            [
                ("id", "string", "Digest candidate id."),
                ("bot_token", "string", "Telegram bot token."),
                ("chat_id", "string", "Telegram chat id."),
                (
                    "idempotency_key",
                    "string",
                    "Optional idempotency key for deliberate replays.",
                ),
            ],
        ),
        tool(
            "digest_candidate_deliver_email",
            "Deliver an approved digest candidate to email after review, policy, channel authorization, cost, and provider-send gates.",
            [
                ("id", "string", "Digest candidate id."),
                ("to", "string", "Recipient email address."),
                ("from", "string", "Optional sender email address."),
                (
                    "account_id",
                    "string",
                    "Optional Cloudflare account id; falls back to configured secret/env.",
                ),
                (
                    "api_token",
                    "string",
                    "Optional Cloudflare API token; falls back to configured secret/env.",
                ),
                (
                    "idempotency_key",
                    "string",
                    "Optional idempotency key for deliberate replays.",
                ),
                (
                    "api_base",
                    "string",
                    "Optional Cloudflare API base for tests or controlled providers.",
                ),
            ],
        ),
        tool(
            "digest_alert_schedule_create",
            "Create a resident worker schedule that selects approved digest candidates above a threshold and routes them through the digest delivery ledger.",
            [
                ("name", "string", "Schedule name."),
                (
                    "channel",
                    "string",
                    "Delivery channel, currently telegram or email.",
                ),
                (
                    "recipient_ref",
                    "string",
                    "Recipient reference such as telegram:chat:123 or email:user@example.com.",
                ),
                (
                    "min_score",
                    "number",
                    "Minimum approved digest candidate score required for alerting.",
                ),
                (
                    "max_candidates",
                    "integer",
                    "Maximum approved unsent candidates delivered per tick.",
                ),
                ("interval_hours", "integer", "Schedule cadence in hours."),
                (
                    "quiet_hours",
                    "object",
                    "Optional UTC quiet-hours object with start and end HH:MM.",
                ),
            ],
        ),
        tool(
            "digest_alert_schedules",
            "List scheduled digest alert routes.",
            [],
        ),
        tool(
            "digest_alert_ticks",
            "List scheduled digest alert worker ticks, optionally for one schedule.",
            [(
                "schedule_id",
                "string",
                "Optional digest alert schedule id filter.",
            )],
        ),
        tool(
            "radar_profile_create",
            "Create a Horizon-style radar profile over configured selectors.",
            [
                ("name", "string", "Profile name."),
                (
                    "source_selectors",
                    "array",
                    "Selector objects; source_card_query is locally implemented.",
                ),
            ],
        ),
        tool("radar_profile_list", "List radar profiles.", []),
        tool(
            "radar_profile_read",
            "Read a radar profile by id or name.",
            [("profile", "string", "Radar profile id or name.")],
        ),
        tool_with_schema(
            "radar_run",
            "Run a radar profile. By default this uses the locally proven source-card projection, FTS, and heuristic scoring stages; fetch_live=true first invokes existing Arcwell RSS/GitHub/arXiv/Hacker News/Reddit/X adapters and records adapter jobs/source health.",
            json!({
                "profile": string_schema("Radar profile id or name."),
                "window_hours": integer_schema("Optional run window override in hours."),
                "fetch_live": boolean_schema("Opt in to live adapter fetches before source-card projection.")
            }),
            &["profile"],
        ),
        tool_with_schema(
            "radar_enqueue",
            "Enqueue a radar profile run for the local worker. The worker writes the same radar_runs/items/FTS/scores state as radar_run and records blocked/partial status when live adapters fail.",
            json!({
                "profile": string_schema("Radar profile id or name."),
                "window_hours": integer_schema("Optional run window override in hours."),
                "fetch_live": boolean_schema("Opt in to live adapter fetches during worker execution.")
            }),
            &["profile"],
        ),
        tool("radar_runs", "List radar runs.", []),
        tool(
            "radar_stage_read",
            "Read normalized radar items, score overlays, and dedupe groups for a run.",
            [("run_id", "string", "Radar run id.")],
        ),
        tool_with_schema(
            "radar_model_score",
            "Write model-backed radar interestingness score overlays for an audit-ok run. These rows are non-authorizing and do not replace heuristic selected rows used for summaries or delivery.",
            json!({
                "run_id": string_schema("Radar run id."),
                "provider": enum_schema("Model provider. Use mock for deterministic local proof or openai for live provider attempt.", &["mock", "openai"]),
                "model": string_schema("Optional model name."),
                "max_items": integer_schema("Maximum heuristic-selected/over-limit candidates to score, default 10, max 25."),
                "endpoint": string_schema("Optional OpenAI-compatible endpoint override for authorized tests."),
                "api_key": string_schema("Optional API key; prefer local secret configuration.")
            }),
            &["run_id"],
        ),
        tool(
            "radar_summarize",
            "Write a deterministic local Markdown radar summary artifact over selected scored items. This does not deliver messages or run model summarization.",
            [
                ("run_id", "string", "Radar run id."),
                ("language", "string", "Language code, default en."),
                (
                    "format",
                    "string",
                    "Summary format; only markdown is supported.",
                ),
            ],
        ),
        tool(
            "radar_summary_read",
            "Read a deterministic local radar summary artifact for a run.",
            [
                ("run_id", "string", "Radar run id."),
                ("language", "string", "Language code, default en."),
                (
                    "format",
                    "string",
                    "Summary format; only markdown is supported.",
                ),
            ],
        ),
        tool_with_schema(
            "radar_deliver_summary",
            "Deliver an existing audit-ok radar summary through authorized Telegram or Cloudflare Email send paths and record a durable radar delivery row linked to the channel delivery attempt. This is a manual delivery attempt, not scheduled operation.",
            json!({
                "run_id": string_schema("Radar run id."),
                "recipient_ref": string_schema("Telegram chat id or email address."),
                "channel": string_schema("telegram or email; default telegram."),
                "language": string_schema("Language code, default en."),
                "format": string_schema("Summary format, default markdown."),
                "idempotency_key": string_schema("Optional stable key to prevent duplicate delivery."),
                "bot_token": string_schema("Optional Telegram bot token; otherwise env/local secret is used."),
                "account_id": string_schema("Optional Cloudflare account id for email delivery."),
                "api_token": string_schema("Optional Cloudflare Email/API token for email delivery."),
                "from": string_schema("Optional email sender address."),
                "api_base": string_schema("Optional provider API base for authorized local/staging tests.")
            }),
            &["run_id", "recipient_ref"],
        ),
        tool(
            "radar_delivery_list",
            "List durable radar delivery rows, optionally filtered by run id.",
            [("run_id", "string", "Optional radar run id.")],
        ),
        tool(
            "radar_audit_run",
            "Audit a radar run for FTS drift, missing provenance, unscored items, missing source-quality windows, corrupt dedupe groups, empty output, and unsupported selectors.",
            [("run_id", "string", "Radar run id.")],
        ),
        tool(
            "radar_source_quality",
            "List source-quality windows materialized for one scored radar run, including accepted counts, score percentiles, duplicate rate, and source-health failure contribution.",
            [("run_id", "string", "Radar run id.")],
        ),
        tool(
            "radar_source_quality_trends",
            "Rank local radar source-quality history across runs. This uses only durable local windows and does not claim global/community quality or seven-day decay proof.",
            [
                (
                    "min_windows",
                    "number",
                    "Minimum windows per source, default 2.",
                ),
                (
                    "limit",
                    "number",
                    "Maximum rows to return, default 50, max 500.",
                ),
            ],
        ),
        tool(
            "radar_rebuild_fts",
            "Rebuild radar item FTS rows globally or for one run.",
            [],
        ),
        tool(
            "librarian_expand_topic",
            "Ask the wiki librarian to expand a topic from source cards and wiki pages.",
            [("topic", "string", "Topic to expand.")],
        ),
        tool("ops_snapshot", "Read local ops snapshot.", []),
        tool(
            "secret_value_set",
            "Store a local SQLite-backed secret value for provider clients.",
            [
                ("name", "string", "Secret name."),
                ("value", "string", "Secret value."),
                ("scope", "string", "Secret scope."),
                ("provider", "string", "Optional provider name."),
                ("expires_at", "string", "Optional RFC3339 expiry timestamp."),
            ],
        ),
        tool(
            "secret_value_list",
            "List local SQLite-backed secret names without values.",
            [],
        ),
        tool(
            "secret_health",
            "List redacted credential presence, scope, provider, and expiry health.",
            [],
        ),
        tool(
            "secret_value_delete",
            "Delete a local SQLite-backed secret value.",
            [("name", "string", "Secret name.")],
        ),
        tool("cursor_list", "List adapter cursor state.", []),
        tool(
            "cursor_get",
            "Read one adapter cursor by key.",
            [("key", "string", "Cursor key.")],
        ),
        tool(
            "source_card_add",
            "Add a typed source card, write its Markdown page to the wiki, and optionally link it to a research run with run_id.",
            [
                ("title", "string", "Source title."),
                ("url", "string", "Source URL."),
                ("summary", "string", "Short source summary."),
                ("run_id", "string", "Optional research run id."),
            ],
        ),
        tool(
            "source_card_search",
            "Search typed source cards.",
            [("query", "string", "Search query.")],
        ),
        tool(
            "source_card_read",
            "Read a typed source card by id.",
            [("id", "string", "Source card id.")],
        ),
        tool(
            "wiki_ingest_job",
            "Run a recorded wiki ingest job for a Markdown/text file.",
            [("path", "string", "Path to ingest.")],
        ),
        tool(
            "wiki_ingest_url",
            "Run a recorded wiki ingest job for a public HTTP(S) URL.",
            [("url", "string", "URL to ingest.")],
        ),
        tool_with_schema(
            "wiki_ingest_rendered_page",
            "Run a recorded no-network wiki ingest job for host/browser-rendered page DOM or visible text.",
            json!({
                "requested_url": string_schema("Original page URL."),
                "final_url": string_schema("Optional post-render/redirect URL."),
                "title": string_schema("Optional rendered page title."),
                "rendered_html": string_schema("Optional rendered DOM/HTML."),
                "rendered_text": string_schema("Optional visible text from the rendered page."),
                "captured_at": string_schema("Optional capture timestamp."),
                "browser": string_schema("Optional host/browser surface."),
                "screenshot_path": string_schema("Optional screenshot or snapshot path.")
            }),
            &["requested_url"],
        ),
        tool(
            "wiki_ingest_dir",
            "Bulk ingest Markdown files from a local directory into the wiki index.",
            [("path", "string", "Directory path to ingest.")],
        ),
        tool(
            "wiki_import_codex_swift_sources",
            "Import Codex Swift wiki watch-source seeds into the local watch registry.",
            [("path", "string", "Path to a codex-swift checkout.")],
        ),
        tool(
            "wiki_watch_sources",
            "List configured wiki watch sources.",
            [],
        ),
        tool(
            "wiki_compile",
            "Compile matching wiki context into a recorded brief job.",
            [("query", "string", "Compile topic.")],
        ),
        tool(
            "wiki_expand_page",
            "Create an expanded wiki page from matching source cards and local pages.",
            [("topic", "string", "Page/topic to expand.")],
        ),
        tool(
            "wiki_job_status",
            "Read a wiki job by id.",
            [("id", "string", "Wiki job id.")],
        ),
        tool("wiki_jobs", "List wiki jobs.", []),
        tool(
            "wiki_decision_ledger_summary",
            "Summarize reviewed wiki editorial decisions from the durable DB ledger.",
            [],
        ),
        tool(
            "wiki_decision_ledger_list",
            "List reviewed wiki editorial decisions from the durable DB ledger.",
            [(
                "limit",
                "integer",
                "Maximum rows to return. Defaults to 50.",
            )],
        ),
        tool(
            "wiki_enqueue_rss",
            "Enqueue an RSS/Atom fetch job.",
            [("url", "string", "RSS/Atom URL.")],
        ),
        tool(
            "wiki_enqueue_github",
            "Enqueue a GitHub repo adapter job.",
            [
                ("owner", "string", "GitHub owner."),
                ("repo", "string", "GitHub repo."),
            ],
        ),
        tool(
            "wiki_enqueue_github_owner",
            "Enqueue a GitHub owner adapter job that discovers recent public repos.",
            [("owner", "string", "GitHub owner.")],
        ),
        tool(
            "wiki_enqueue_arxiv",
            "Enqueue an arXiv search adapter job.",
            [("query", "string", "arXiv query.")],
        ),
        tool(
            "x_import_json_file",
            "Import replayed X items from a local JSON file into source cards and wiki pages.",
            [("path", "string", "Path to X JSON export/replay fixture.")],
        ),
        tool(
            "x_import_archive",
            "Import supported Twitter/X archive tweets, bookmarks, and likes from a local directory or zip without network access, while reporting unsupported slices without reading them.",
            [
                (
                    "path",
                    "string",
                    "Path to a Twitter/X archive directory or zip.",
                ),
                (
                    "select",
                    "array",
                    "Optional selectors: tweets, bookmarks, likes, or all.",
                ),
                ("limit", "integer", "Maximum archive records to import."),
            ],
        ),
        tool(
            "x_discover_archives",
            "Find likely local Twitter/X archive directories or zip files without importing or writing state.",
            [
                ("dirs", "array", "Optional directories or files to inspect."),
                ("limit", "integer", "Maximum candidates to return."),
            ],
        ),
        tool(
            "x_export_portable",
            "Export canonical local X data as deterministic portable JSONL shards with a hashed manifest, token-like value checks, and an export_portable freshness ledger.",
            [(
                "out",
                "string",
                "Output directory for the portable X bundle.",
            )],
        ),
        tool(
            "x_validate_portable",
            "Validate a portable X bundle manifest, shard hashes, JSONL rows, and token-like content before import.",
            [("dir", "string", "Portable X bundle directory.")],
        ),
        tool(
            "x_import_portable",
            "Validate and import a portable X bundle into canonical local X storage.",
            [("dir", "string", "Portable X bundle directory.")],
        ),
        tool(
            "x_recent_search",
            "Run live X recent search using X_BEARER_TOKEN from env or local SQLite secrets.",
            [("query", "string", "X search query.")],
        ),
        tool(
            "x_enqueue_recent_search",
            "Enqueue a live X recent search job.",
            [("query", "string", "X search query.")],
        ),
        tool(
            "x_import_bookmarks",
            "Import authenticated X bookmarks as full X items with source provenance and public metrics.",
            [
                (
                    "bookmark_days",
                    "integer",
                    "Only import bookmarked tweets newer than this many days.",
                ),
                ("max_bookmarks", "integer", "Maximum bookmarks to scan."),
            ],
        ),
        tool(
            "x_schedule_bookmarks",
            "Create or update the resident worker watch source that periodically imports authenticated X bookmarks.",
            [
                (
                    "bookmark_days",
                    "integer",
                    "Only import bookmarked tweets newer than this many days.",
                ),
                ("max_bookmarks", "integer", "Maximum bookmarks to scan."),
                ("cadence", "string", "Watch cadence: hot, warm, or cold."),
                ("status", "string", "Watch status: active or paused."),
            ],
        ),
        tool(
            "x_import_following_watch_sources",
            "Import authenticated X following accounts into the wiki watch-source registry.",
            [(
                "max_users",
                "integer",
                "Maximum followed accounts to import.",
            )],
        ),
        tool(
            "x_rebuild_definitive_watch_sources",
            "Replace X watch sources with bookmark authors from the recent window plus recent follows.",
            [
                (
                    "bookmark_days",
                    "integer",
                    "Bookmark tweet age window in days.",
                ),
                ("max_bookmarks", "integer", "Maximum bookmarks to scan."),
                (
                    "max_recent_follows",
                    "integer",
                    "Maximum recent follows to include.",
                ),
            ],
        ),
        tool(
            "x_monitor_watch_sources",
            "Poll the definitive X watch-source list, ingest new watched-source tweets as source cards, and create digest candidates.",
            [
                (
                    "max_sources",
                    "integer",
                    "Maximum active x_handle watch sources to poll.",
                ),
                (
                    "max_results_per_source",
                    "integer",
                    "Maximum recent tweets to request per watched source.",
                ),
            ],
        ),
        tool(
            "x_repair_health",
            "Reconcile stale X source-health rows after later successful syncs and defer currently rate-limited rows without marking them healthy.",
            [
                (
                    "defer_rate_limited_hours",
                    "integer",
                    "Hours to defer stale rate-limited X source-health rows.",
                ),
                (
                    "limit",
                    "integer",
                    "Maximum stale rate-limited rows to defer.",
                ),
            ],
        ),
        tool(
            "x_oauth_probe",
            "Probe current X OAuth credentials against provider endpoints for users.read, bookmark.read, follows.read, and tweet.read without importing source data.",
            [(
                "search_query",
                "string",
                "Optional recent-search query used to prove tweet.read; defaults to from:openai.",
            )],
        ),
        tool(
            "x_oauth_authorize_url",
            "Create an X OAuth 2.0 PKCE authorization URL, resolving stored X_CLIENT_ID and default redirect URI when omitted.",
            [
                ("client_id", "string", "Optional X OAuth client id."),
                ("redirect_uri", "string", "Optional OAuth redirect URI."),
            ],
        ),
        tool(
            "x_oauth_exchange_code",
            "Exchange an X OAuth 2.0 authorization code and store returned tokens in local SQLite secrets, resolving stored X client metadata when omitted.",
            [
                ("client_id", "string", "Optional X OAuth client id."),
                ("redirect_uri", "string", "Optional OAuth redirect URI."),
                ("code", "string", "Authorization code."),
                ("code_verifier", "string", "PKCE code verifier."),
            ],
        ),
        tool(
            "x_oauth_refresh",
            "Refresh an X OAuth token from the stored X_REFRESH_TOKEN and store the new token response; resolves stored X_CLIENT_ID when omitted.",
            [("client_id", "string", "Optional X OAuth client id.")],
        ),
        tool(
            "x_oauth_revoke",
            "Revoke a stored X OAuth token through the X revoke endpoint; optionally delete the local secret only after provider success.",
            [
                (
                    "name",
                    "string",
                    "Stored secret name, either X_BEARER_TOKEN or X_REFRESH_TOKEN.",
                ),
                ("client_id", "string", "Optional X OAuth client id."),
                (
                    "token_type_hint",
                    "string",
                    "Optional token hint: access_token or refresh_token.",
                ),
                (
                    "delete_local",
                    "boolean",
                    "Delete the local secret after provider revocation succeeds.",
                ),
            ],
        ),
        tool(
            "x_list",
            "List imported X items, optionally filtered by source such as bookmark.",
            [
                ("query", "string", "Optional text query."),
                (
                    "source",
                    "string",
                    "Optional source kind, for example bookmark.",
                ),
                ("limit", "integer", "Maximum items to return."),
            ],
        ),
        tool(
            "x_bookmarks",
            "List imported X bookmark items.",
            [
                ("query", "string", "Optional text query."),
                ("limit", "integer", "Maximum items to return."),
            ],
        ),
        tool(
            "x_search_tweets",
            "Search canonical local X tweet text, authors, and URLs with FTS.",
            [
                ("query", "string", "Search query."),
                ("limit", "integer", "Maximum items to return."),
            ],
        ),
        tool(
            "x_research",
            "Render a local-only X research brief from already-imported tweets with source-card IDs and local thread context. Empty or unprojected evidence fails honestly; no live fetch, model synthesis, or writes are performed.",
            [
                ("query", "string", "Local X search query."),
                ("limit", "integer", "Maximum matching tweets to include."),
            ],
        ),
        tool(
            "x_thread",
            "Expand a local-only X thread around a known tweet, with bounded depth, quote/retweet distinctions, missing-context labels, and cycle detection.",
            [
                ("x_id", "string", "Root X tweet id already present locally."),
                (
                    "max_depth",
                    "integer",
                    "Maximum local reference depth to follow.",
                ),
            ],
        ),
        tool(
            "x_extract_links",
            "Extract safe local URL occurrences from already-imported X tweets without fetching or expanding them.",
            [("limit", "integer", "Maximum tweets to scan.")],
        ),
        tool(
            "x_expand_links",
            "Fetch and ingest indexed X link URLs through the explicit URL-ingest safety path, with policy/cost gates and expansion status rows.",
            [("limit", "integer", "Maximum indexed links to expand.")],
        ),
        tool(
            "x_links",
            "List locally indexed X URL occurrences.",
            [
                (
                    "query",
                    "string",
                    "Optional URL, display URL, or tweet id filter.",
                ),
                ("limit", "integer", "Maximum link occurrences to return."),
            ],
        ),
        tool(
            "x_stats",
            "Inspect canonical X counts, compatibility drift, FTS drift, projections, sync runs, source health, watch-source status, and portable export freshness.",
            [],
        ),
        tool(
            "x_repair_projections",
            "Repair missing or failed canonical X tweet source-card/wiki projections idempotently.",
            [(
                "limit",
                "integer",
                "Maximum candidate projections to repair.",
            )],
        ),
        tool("x_report", "Render a report from imported X items.", []),
        tool(
            "wiki_ingest_file",
            "Ingest a Markdown file into the local wiki.",
            [("path", "string", "Absolute or relative markdown path.")],
        ),
        tool(
            "wiki_search",
            "Search the local Markdown wiki.",
            [("query", "string", "Search query.")],
        ),
        tool(
            "wiki_read",
            "Read a local wiki page by id.",
            [("id", "string", "Wiki page id.")],
        ),
    ]
}
