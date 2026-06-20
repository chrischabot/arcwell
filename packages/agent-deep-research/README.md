# agent-deep-research

Research run coordination for MCP-capable agents.

Current implementation:

```sh
agent research plan "topic or question" --max-sources 5
agent research search "topic or question" --provider brave --write-wiki
agent research workflow "topic or question"
agent research tasks <run-id>
agent research complete-task <task-id> "notes"
agent research brief "topic or question"
agent research brief "topic or question" --no-write
agent research runs
```

MCP tools:

- `research_plan`
- `research_web_search`
- `research_workflow_create`
- `research_tasks`
- `research_task_complete`
- `research_brief_from_wiki`
- `research_runs`

MCP resources:

- `agent://research`

Boundary:

- The local service records runs, searches local wiki pages, and writes briefs back to `agent-llm-wiki`.
- The host agent performs current web search with its native tools when available.
- Optional daemon providers are `brave`, `openai`, and `perplexity`.
- Provider endpoints are guarded to official HTTPS origins or loopback tests unless explicitly overridden.
- Generated research briefs are outputs, not source material. Research source selection excludes pages titled `Research Brief: ...`.
- Brave, OpenAI, and Perplexity are adapters, not hard dependencies.

Recommended host-agent loop:

1. Call `research_plan`.
2. Create a daemon-tracked workflow with `research_workflow_create` when the work should be split into scout/extractor/skeptic/synthesizer phases.
3. Search current web sources using the host's native web-search capability, or `research_web_search` when a provider key is configured.
4. Ingest source cards into `agent-llm-wiki`.
5. Call `research_brief_from_wiki`.
6. Audit the brief against sources before using it externally.

Future work:

- Typed source-card schema.
- Model-backed synthesis and contradiction extraction.
- Scheduled monitors that expand wiki pages when important sources change.
