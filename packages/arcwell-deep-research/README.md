# arcwell-deep-research

**Status:** Partial.

Research run coordination for MCP-capable agents.

Current implementation:

```sh
arcwell research plan "topic or question" --max-sources 5
arcwell research search "topic or question" --provider brave --write-wiki
arcwell research workflow "topic or question"
arcwell research tasks <run-id>
arcwell research complete-task <task-id> "notes"
arcwell research brief "topic or question"
arcwell research brief "topic or question" --no-write
arcwell research audit "topic or question"
arcwell research runs
```

MCP tools:

- `research_plan`
- `research_web_search`
- `research_workflow_create`
- `research_tasks`
- `research_task_complete`
- `research_brief_from_wiki`
- `research_audit`
- `research_runs`

MCP resources:

- `arcwell://research`

Boundary:

- The local service records runs, searches local wiki pages, and writes briefs back to `arcwell-llm-wiki`.
- The host agent performs current web search with its native tools when available.
- Optional daemon providers are `brave`, `openai`, and `perplexity`.
- Provider endpoints are guarded to official HTTPS origins or loopback tests unless explicitly overridden.
- Generated research briefs and source-card wiki artifacts are outputs, not source material. Research source selection excludes generated `Research Brief: ...`, `Expanded: ...`, and `Source Card: ...` wiki pages from local-source evidence.
- `research_audit` checks local source cards for schema/version metadata, generated-page recursion, uncited model answers, stale retrieval dates, prompt-injection/SEO-spam indicators, low reliability, robots `noindex`, low-confidence claims, and conflicting launch dates.
- Brief source selection excludes generated/model-answer cards plus explicitly untrusted or low-reliability source cards. Those cards remain stored and auditable as evidence, but they do not ground synthesized briefs.
- Brave, OpenAI, and Perplexity are adapters, not hard dependencies.

Recommended host-agent loop:

1. Call `research_plan`.
2. Create a daemon-tracked workflow with `research_workflow_create` when the work should be split into scout/extractor/skeptic/synthesizer phases.
3. Search current web sources using the host's native web-search capability, or `research_web_search` when a provider key is configured.
4. Ingest source cards into `arcwell-llm-wiki`.
5. Call `research_brief_from_wiki`.
6. Call `research_audit` and audit the brief against sources before using it externally.

Future work:

- Model-backed synthesis, extraction, and contradiction review beyond the current deterministic audit heuristics.
- End-to-end scheduled monitor loops that expand wiki pages when important sources change; current hooks enqueue due watch-source jobs only.
