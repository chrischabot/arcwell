# arcwell-deep-research

**Status:** Partial.

Deep research coordination for MCP-capable agents.

Target product contract:

- Arcwell has one user-facing research mode: deep.
- Invoking research means broad source discovery, deep source mining, structured
  source cards and claims, skeptic/refutation passes, cited synthesis, audit,
  and durable wiki writeback.
- There is no quick/surface research mode. A short executive summary or brief is
  an output artifact inside a deep run, not a separate product path.
- The normal assistant should not auto-trigger Deep Research for every question;
  it should run only when research is explicitly invoked or unmistakably
  requested.

Design reference: `docs/deep-research-system-design.md`.

Current implementation:

```sh
arcwell research run "topic or question"
arcwell research status <run-id>
arcwell research read <run-id>
arcwell research audit-run <run-id>
arcwell research stop <run-id>
arcwell research sources <run-id>
arcwell research add-source <run-id> --title "Source" --url https://example.com/source
arcwell research link-source-card <run-id> <source-card-id>
arcwell research extraction-prompt <run-id> <source-card-id>
arcwell research ingest-claims <run-id> <source-card-id> --output-json '{"claims":[]}'
arcwell research claims <run-id>
arcwell research clusters <run-id>
arcwell research skeptic <run-id>
arcwell research report <run-id> "source coverage satisfied"
arcwell research plan "topic or question" --max-sources 5
arcwell research search "topic or question" --provider brave --write-wiki
arcwell research workflow "topic or question" # compatibility alias for run
arcwell research tasks <run-id>
arcwell research complete-task <task-id> "notes"
arcwell research brief "topic or question"
arcwell research brief "topic or question" --no-write
arcwell research audit "topic or question" # legacy query audit
arcwell research runs
```

MCP tools:

- `research_plan`
- `research_run`
- `research_status`
- `research_read`
- `research_audit_run`
- `research_stop`
- `research_web_search`
- `research_workflow_create`
- `research_sources`
- `research_source_add`
- `research_source_card_link`
- `research_extraction_prompt`
- `research_claims_ingest`
- `research_claims`
- `research_clusters`
- `research_skeptic_pass`
- `research_report_compile`
- `research_tasks`
- `research_task_complete`
- `research_brief_from_wiki`
- `research_audit`
- `research_runs`

MCP resources:

- `arcwell://research`

Current boundary:

- The local service records deep runs, role tasks, source ledgers, run-source links, extracted claims, clusters, contradictions, skeptic reports, and final report artifacts.
- The host agent performs current web search with its native tools when available.
- Optional daemon providers are `brave`, `openai`, and `perplexity`.
- Provider endpoints are guarded to official HTTPS origins or loopback tests unless explicitly overridden.
- Generated research briefs and source-card wiki artifacts are outputs, not source material. Research source selection excludes generated `Research Brief: ...`, `Expanded: ...`, and `Source Card: ...` wiki pages from local-source evidence.
- `research_audit` checks local source cards for schema/version metadata, generated-page recursion, uncited model answers, stale retrieval dates, prompt-injection/SEO-spam indicators, low reliability, robots `noindex`, low-confidence claims, and conflicting launch dates. `research_audit_run` also includes run-linked source cards even when query text search misses them.
- `research_claims_ingest` validates model-produced JSON and rejects malformed output, prompt-injection text, and uncertainty loss before storing structured claims.
- `research_skeptic_pass` builds deterministic clusters, records structured claim contradictions, and fails runs with missing primary evidence or generated/model-answer evidence loops.
- `research_report_compile` compiles a durable report from linked sources, extracted claims, clusters, skeptic findings, and audit results. It marks reports incomplete when audit or skeptic checks fail.
- Brief/report source selection excludes generated/model-answer cards plus explicitly untrusted or low-reliability source cards. Those cards remain stored and auditable as evidence, but they do not ground synthesized outputs.
- Brave, OpenAI, and Perplexity are adapters, not hard dependencies.

Target host-agent loop:

1. Start a deep run with `research_run`.
2. Call `research_plan` when local context and suggested searches help.
3. Build a source map across primary, secondary, dissenting, technical, historical, and adjacent source families.
4. Search current sources using the host's native web-search capability, or `research_web_search` when a provider key is configured.
5. Ingest and link source cards with `source_card_add` plus `run_id` or `research_source_card_link`.
6. Extract structured claims through `research_extraction_prompt` and `research_claims_ingest`.
7. Run `research_clusters`, `research_skeptic_pass`, and `research_audit_run`.
8. Compile the final report with `research_report_compile`.

Future work:

- Codex-native subagent workflow for scout, corpus builder, extractor, skeptic, synthesizer, and auditor roles.
- Live deep-run proof against the three design-reference topics, including source-count/saturation reporting, cost records, and fresh-provider evidence.
- Model-backed synthesis and contradiction review beyond the current bounded extraction plus deterministic audit/skeptic heuristics.
- End-to-end scheduled monitor loops that expand wiki pages when important sources change; current hooks enqueue due watch-source jobs only.
