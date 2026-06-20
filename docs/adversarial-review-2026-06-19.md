# Adversarial Review, 2026-06-19

## Scope

Reviewed and tested the new residual-gap work:

- Daemon-side provider web search.
- Research workflow task orchestration.
- MCP compatibility probes.
- Severe failure modes around provider endpoints, unsafe URLs, task state, and runtime behavior.

## Findings Fixed

### Blocking HTTP Inside Tokio Runtime

Score: 100.

Evidence: the disposable CLI smoke for `agent research search ... --write-wiki` panicked with:

```text
Cannot drop a runtime in a context where blocking is not allowed.
```

Root cause: the CLI used `#[tokio::main]` for every command, but daemon-side web search uses `reqwest::blocking`. Dropping the blocking client inside the Tokio runtime triggered the panic.

Fix: made the CLI entrypoint synchronous and construct a Tokio runtime only for `agent serve`.

Validation: full smoke now passes for research workflow, search, wiki write-back, brief, and MCP probes.

### Arbitrary Provider Endpoint Egress

Score: 75.

Evidence: inspection showed MCP/CLI callers could provide any HTTPS search endpoint. That is unnecessary for normal provider use and creates an avoidable SSRF-shaped egress path.

Fix: custom endpoints are now limited to official provider origins or loopback test endpoints unless `AGENT_SERVICES_ALLOW_CUSTOM_SEARCH_ENDPOINTS=1` is explicitly set.

Validation: severe tests reject non-HTTPS non-loopback endpoints and custom HTTPS endpoints by default.

## Severe Tests Added

- Host-native provider misuse returns an instruction error instead of pretending daemon search happened.
- Non-HTTPS non-loopback endpoint rejection.
- Custom HTTPS endpoint rejection without explicit override.
- Unsafe search result schemes are skipped before wiki write-back.
- Research workflow task creation and completion.
- Empty, missing, and oversized task notes are rejected.
- OpenAI citation extraction handles nested annotations.
- MCP `ping`, `resources/templates/list`, and `prompts/list` probes return clean responses.
- MCP research workflow round trip.
- MCP host-native web search misuse returns an error.

## Validation Commands

```sh
cargo fmt --all -- --check
cargo test
```

Runtime smoke:

- CLI wiki ingest.
- CLI research workflow/tasks/complete-task.
- CLI research plan/brief.
- CLI Brave-compatible search against loopback mock server with wiki write-back.
- MCP initialize/ping/workflow/web-search-error/templates-list.
- MCP Inspector `tools/list`.
- MCP Inspector `tools/call agent_health`.
- Isolated Codex `mcp add` and `mcp get --json`.

## Remaining Risk

- Provider schemas can drift. Current adapters are guarded and tested against representative shapes, but live API regression tests require real keys.
- OpenAI/Perplexity answer synthesis is parsed conservatively; missing citations degrade to warnings instead of fabricated source links.
- Claude Code connection validation was blocked in a temp `HOME` because the isolated profile was unauthenticated. This is a host-auth limitation, not reproduced as an MCP server failure.
- No autonomous worker pool executes research tasks without a host agent yet.
