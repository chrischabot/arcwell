# Phase 0/1 Implementation Notes

Date: 2026-06-19

## What Exists

This repo now has the first runnable slice of the Phase 0/1 architecture:

- Rust workspace with `crates/agent-core` and `crates/agent-cli`.
- SQLite-backed local store under `AGENT_SERVICES_HOME` or `~/.agent-services`.
- CLI binary named `agent`.
- Lightweight local HTTP daemon via `agent serve`.
- Stdio MCP server via `agent mcp`.
- Phase 1 primitives for:
  - `agent-profile`: set/get/list/search/delete profile items.
  - `agent-memory`: add/list/search/delete simple memories.
  - `agent-conversation-import`: Claude export dry-run and candidate generation.
  - candidate review/apply/reject flow.
  - `agent-backup`: local snapshot with manifest and checksum.
  - `agent-secrets`: secret reference metadata, not secret values.
  - `agent-cost`: simple cost ledger and summary.
- Phase 2 primitives for:
  - `agent-llm-wiki`: Markdown file ingest, SQLite metadata, search, list, and read.
- Phase 3 primitives for:
  - `agent-deep-research`: local research run ledger, wiki-grounded planning, wiki brief write-back, and MCP tools/resources.

## Current Commands

```sh
cargo run -q -- doctor
cargo run -q -- profile set communication.competence_respect "Consult relevant context and use adequate effort."
cargo run -q -- profile search competence
cargo run -q -- memory add "My cat is called Ophelia" --kind fact
cargo run -q -- memory search Ophelia
cargo run -q -- import claude /path/to/conversations.json --dry-run --limit 50
cargo run -q -- import claude /path/to/conversations.json --write-candidates --limit 50
cargo run -q -- candidate list
cargo run -q -- candidate apply <candidate-id>
cargo run -q -- backup create
cargo run -q -- backup verify
cargo run -q -- wiki ingest-file ./some-page.md
cargo run -q -- wiki search "agent infrastructure"
cargo run -q -- wiki read <page-id>
cargo run -q -- research plan "deep research architecture" --max-sources 5
cargo run -q -- research brief "deep research architecture"
cargo run -q -- research brief "deep research architecture" --no-write
cargo run -q -- research runs
cargo run -q -- secrets set-ref OPENAI_API_KEY env:OPENAI_API_KEY model-calls
cargo run -q -- cost add agent-memory job-1 openai gpt-5.5 --estimated-usd 0.01 --actual-usd 0.008
cargo run -q -- mcp
cargo run -q -- serve --addr 127.0.0.1:8787
```

HTTP endpoints:

- `GET /health`
- `GET /profile`
- `GET /memory`
- `GET /wiki`
- `GET /wiki?q=<query>`

MCP:

- `agent mcp` speaks line-delimited JSON-RPC over stdio.
- Implemented methods: `initialize`, `tools/list`, `tools/call`, `resources/list`, and `resources/read`.
- Current tools include health, profile, memory, candidate review, backup create/verify, cost summary, wiki ingest/search/read, and research plan/brief/runs.

## Learnings

- The local HTTP daemon cannot put a `rusqlite::Connection` directly in shared Axum state because the connection is not `Sync`. The first implementation now stores paths in HTTP state and opens a store per request. That is simple and correct for the spike; a later daemon can add a deliberate connection pool.
- Candidate review is the right bridge between import and memory/profile. Even a crude Claude export scanner immediately benefits from not auto-applying private history.
- Backup needs to exist before the system feels trustworthy. The first version is local snapshot + manifest only; encryption and restore planning are still required.
- The CLI and HTTP routes already exercise the same core store APIs. Keep that invariant when adding MCP.

## Deliberate Gaps

- `agent-memory` is mem0-shaped in lifecycle vocabulary, but not yet powered by `mem0-rs` extraction/reconciliation.
- The MCP server is intentionally hand-rolled and minimal. Next step is to validate it against Codex/Claude MCP clients, then decide whether to keep it or move to the official Rust SDK.
- `agent-deep-research` remains host-native-search first, but now has optional daemon-side Brave/OpenAI/Perplexity search adapters with guarded provider endpoints.
- No encryption for backups yet.
- `agent-secrets` stores only references/metadata, not secret values. Actual secret storage should use OS keychain or explicit 0600 files.
- Claude import heuristics are intentionally conservative and crude. The next version should use a model-backed extractor with redaction, sensitivity labels, and a review UI.
- `agent-quality-kit` is represented by docs/skills next; no automated competence fixtures yet beyond planned tests.

## Validation Run

Completed:

- `cargo fmt --all`
- `cargo test`
- CLI smoke test for doctor/profile/memory/import/candidates/backup/secrets/cost.
- CLI smoke test for wiki ingest/search.
- CLI smoke test for research plan/brief/runs.
- MCP smoke test for initialize/tools/list/memory/wiki.
- MCP smoke test for research plan/brief/runs and `agent://research`.
- HTTP smoke test for `/health`, `/profile`, and `/memory`.
- Severe tests for invalid profile keys, SQL-shaped input, invalid candidate targets, wiki path traversal, backup coverage/tamper detection, generated research self-citation, invalid research queries, and MCP misuse.

Finding during severe testing:

- Backup snapshots initially copied only SQLite and omitted wiki Markdown pages. Fixed by copying `wiki/pages` into each snapshot and adding a regression test that verifies both coverage and tamper detection.
