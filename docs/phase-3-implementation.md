# Phase 3 Deep Research Implementation Notes

Date: 2026-06-19

## What Exists

Phase 3 adds the first local `arcwell-deep-research` slice:

- SQLite `research_runs` table for planned, completed, and no-write research runs.
- SQLite `research_tasks` table for daemon-tracked scout, extractor, skeptic, and synthesizer work.
- CLI commands:
  - `arcwell research plan <query> --max-sources 5`
  - `arcwell research search <query> --provider brave|openai|perplexity`
  - `arcwell research search <query> --provider brave --write-wiki`
  - `arcwell research workflow <query>`
  - `arcwell research tasks <run-id>`
  - `arcwell research complete-task <task-id> <notes>`
  - `arcwell research brief <query>`
  - `arcwell research brief <query> --no-write`
  - `arcwell research runs`
- MCP tools:
  - `research_plan`
  - `research_web_search`
  - `research_workflow_create`
  - `research_tasks`
  - `research_task_complete`
  - `research_brief_from_wiki`
  - `research_runs`
- MCP resource:
  - `arcwell://research`
- Wiki write-back for generated research briefs.
- Severe regression coverage for invalid queries and generated-brief self-citation.

## Boundary

`arcwell-deep-research` is the run ledger and artifact writer. It does not pretend to be a full web research engine yet.

- Local service: stores runs, finds local wiki context, writes generated briefs back into `arcwell-llm-wiki`.
- Host skill: performs host-native web search, launches subagents when available, checks contradictions, and decides what to ingest.
- Optional providers: Brave, OpenAI web search, and Perplexity can run daemon-side when API keys are configured. The portable default is still "use whatever native search the host agent already has."
- Cloudflare: future always-on collectors can queue source cards, but Phase 3 remains local-first.

This keeps the local daemon portable to Codex, Claude Desktop/Code, and other MCP clients without binding it to one model vendor's web-search API.

## Current Workflow

1. Ask the MCP tool for a plan:

```sh
arcwell research plan "OpenAI agent platform changes"
```

2. The host agent searches current sources using its native search tools and existing connectors.
3. Or, when a daemon provider is configured, run guarded provider search:

```sh
arcwell research search "OpenAI agent platform changes" --provider openai --write-wiki
arcwell research search "OpenAI agent platform changes" --provider brave --write-wiki
arcwell research search "OpenAI agent platform changes" --provider perplexity --write-wiki
```

4. The host agent ingests source cards or Markdown notes into the wiki:

```sh
arcwell wiki ingest-file ./source-card.md
```

5. The host agent asks for a local brief:

```sh
arcwell research brief "OpenAI agent platform changes"
```

6. The brief is written to the wiki as `Research Brief: <query>`.

## Provider Guards

Daemon-side web search is intentionally constrained:

- `provider=host` returns an instruction error. The daemon cannot pretend to use a host-native browser/search tool.
- Brave requires `BRAVE_API_KEY`.
- OpenAI requires `OPENAI_API_KEY` and defaults to `gpt-5.5` with the hosted `web_search` tool.
- Perplexity requires `PERPLEXITY_API_KEY` and defaults to `sonar-pro`.
- Provider endpoints must be HTTPS official-provider origins or loopback test endpoints.
- Custom non-loopback endpoints require `ARCWELL_ALLOW_CUSTOM_SEARCH_ENDPOINTS=1`.
- Search result URLs must be `http` or `https`; `javascript:`, `data:`, and similar schemes are dropped before wiki write-back.

## Self-Citation Guard

Generated research pages are useful outputs, but they should not become primary sources for the next run. The research source selector excludes wiki pages whose title begins with `Research Brief:`.

This came from a real smoke-test failure: after creating a brief for `Deep research`, a later no-write MCP brief cited the generated brief as if it were source material. The fix filters generated briefs from source selection and has a regression test:

```sh
cargo test severe_research_brief_does_not_cite_prior_generated_briefs
```

## Validation Run

Completed:

- `cargo fmt --all -- --check`
- `cargo test`
- CLI smoke test for `research plan`, `research workflow`, `research tasks`, `research complete-task`, `research search`, `research brief`, and `research runs`.
- MCP smoke test for `initialize`, `ping`, `resources/templates/list`, `research_workflow_create`, `research_web_search`, and `arcwell://research`.
- Official MCP Inspector smoke for `tools/list` and `tools/call arcwell_health`.
- Isolated Codex MCP config add/get smoke.
- Severe self-citation smoke: after writing a brief, a no-write MCP brief still reports only the original source page.
- Severe endpoint guard tests for non-HTTPS, custom HTTPS, unsafe result URLs, host-native misuse, oversized task notes, and MCP misuse.

Findings during severe testing:

- The first CLI web-search smoke panicked because `reqwest::blocking` was called under the global Tokio runtime created by `#[tokio::main]`. Fixed by making the CLI synchronous and creating a Tokio runtime only for `arcwell serve`.
- MCP callers could pass arbitrary HTTPS provider endpoints. Fixed by allowing only official provider origins or loopback endpoints unless `ARCWELL_ALLOW_CUSTOM_SEARCH_ENDPOINTS=1` is set.

## Deliberate Gaps

- No streaming search/provider responses yet.
- No model-backed long-form synthesis inside the Rust service yet; current brief synthesis is deterministic and wiki-grounded.
- No autonomous worker pool that executes research tasks without a host agent yet.
- No source-card schema beyond Markdown wiki pages yet.
- Claude Code isolated config accepted the server definition but could not health-check under a temp `HOME` because that temp profile is unauthenticated; MCP Inspector and Codex config checks passed.

The next useful phase is to add a source-card schema and a host skill that performs the full research discipline: native search, source extraction, contradiction checks, synthesis, and wiki write-back.
