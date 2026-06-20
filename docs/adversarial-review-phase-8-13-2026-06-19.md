# Adversarial Review Phases 8-13

Date: 2026-06-19

Scope: edge inbox, channel framework, Telegram boundary, project meta-controller, librarian/digest pipeline, memory pipeline, ops snapshot, and package scaffolds.

## Findings

No blocking issues remain in the first-pass implementation after the current test pass.

## Issues Covered By Tests

- Edge events are idempotent by key and replay does not replace the original payload.
- Oversized edge payloads are rejected.
- Edge lease/ack works.
- Nacked edge events respect retry backoff.
- Repeated edge failures become `dead_lettered`.
- Expired edge events are not leased.
- Project references resolve through aliases.
- Ambiguous project references error instead of guessing.
- Follow-up project references can use explicit context.
- Channel prompt-injection-like text is preserved as data.
- Channel control characters are stripped.
- Invalid channel directions and missing project ids are rejected.
- Digest candidates require real source cards.
- Launch/watch-topic signals produce ready digest candidates.
- Librarian expansion writes an auditable wiki page referencing source cards.
- Memory extraction creates review candidates, suppresses duplicates, and dream/reconcile removes exact duplicate memories.
- MCP tools for edge, project, channel, memory, and ops work through the real tool surface.

## Residual Risks

- The Cloudflare Worker scaffold currently validates and echoes accepted events; wiring to Durable Objects/Queues is still needed for deployed buffering.
- Telegram has a package boundary and local channel framework, but not a deployed webhook transform yet.
- Project resolution is string/alias based; it does not yet inspect real Codex/Claude thread inventories.
- Interestingness scoring is transparent but simple and rule-based.
- Librarian page generation currently uses local source-card/wiki expansion scaffolding, not model-backed synthesis.
- Memory extraction uses deterministic patterns as a safe first pass, not a full mem0-style extractor.
- Ops UI is JSON-first; a browser UI with controls remains.

## Validation

```sh
cargo fmt --all -- --check
cargo test
cargo clippy --all-targets -- -D warnings
```

Cloudflare worker validation:

```sh
cd packages/arcwell-edge-inbox/worker
npm install
npm run typecheck
```
