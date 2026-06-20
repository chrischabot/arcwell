# Phases 8-13 Implementation Pass

Date: 2026-06-19

This pass implements first durable surfaces for every remaining plan item.

## Phase 8: Cloudflare Edge Inbox

Implemented:

- Local SQLite `edge_events` inbox.
- Idempotent enqueue by `idempotency_key`.
- Payload size cap.
- TTL/expiry handling.
- Lease, ack, nack, dead-letter, list operations.
- MCP tools and `arcwell://edge-events`.
- Cloudflare Worker scaffold in `packages/arcwell-edge-inbox/worker`.

## Phase 9: Channel Framework And Telegram

Implemented:

- Local `channel_messages` table.
- `channel_record` and `channel_list` MCP tools.
- Channel direction validation.
- Control-character stripping while preserving prompt-injection-like text as data.
- Telegram package boundary in `packages/arcwell-telegram`.

## Phase 10: Project Meta-Controller

Implemented:

- Local `projects` table with aliases, status, and summary.
- `project_create`, `project_list`, and `project_resolve` MCP tools.
- Ambiguity detection.
- Explicit `context_project_id` support for follow-up references.
- Project package boundary in `packages/arcwell-projects`.

## Phase 11: Librarian And Interestingness Pipeline

Implemented:

- Local `digest_candidates` table.
- `digest_candidate_create` and `digest_candidate_list` MCP tools.
- Transparent rule-based interestingness scoring.
- `librarian_expand_topic` MCP tool.
- Librarian package boundary in `packages/arcwell-librarian`.

## Phase 12: Personal Memory Pipeline

Implemented:

- `memory_extract_candidates` MCP tool for reviewable memory extraction candidates.
- Duplicate suppression against existing memories and pending candidates.
- `memory_dream_reconcile` MCP tool for exact duplicate reconciliation.
- Personal memory remains separate from wiki knowledge.

## Phase 13: Ops UI And Packaging

Implemented:

- `ops_snapshot` MCP tool.
- `arcwell://ops` resource.
- HTTP `GET /ops`.
- Ops package boundary in `packages/arcwell-ops`.
- Package READMEs for edge inbox, Telegram, projects, librarian, and ops.

## Remaining Depth Work

- Production Cloudflare persistence bindings and deployment secrets.
- Telegram webhook worker that transforms real Telegram updates into edge inbox events.
- Live Codex thread inventory integration for project status.
- Model-backed librarian synthesis and contradiction detection.
- Rich memory extraction hooks and conflict resolution.
- Browser ops UI with controls, not just JSON snapshots.
