# Adversarial Review Phase 6

Date: 2026-06-19

Scope: background job workers, RSS/GitHub/arXiv adapters, live X OAuth/API/cursors, SQLite local secret values, and MCP exposure.

## Findings

No blocking issues remain in the implemented surface after the current test pass.

## Issues Found And Fixed

- OAuth codes, PKCE verifiers, and refresh tokens were initially routed through the generic key validator, which capped values at 200 bytes. This would reject legitimate provider tokens. Fixed by adding a dedicated OAuth parameter validator with a 20 KB limit.
- X OAuth exchange existed only as authorization URL generation. Fixed by adding code exchange, refresh, token response storage, and no-token-response rejection.
- Cursor state existed in core but was not agent-visible. Fixed by adding CLI, MCP tools, and MCP resources for cursor inspection.
- SQLite secret values were acceptable for local use, but a generic read tool would make accidental leakage easy. Fixed by exposing set/list/delete only through MCP; list/resources omit values.

## Attack Cases Covered By Tests

- Unknown worker job kind such as `shell_exec` cannot enter the queue.
- Missing queued ingest files are marked failed, not silently ignored.
- URL ingest rejects loopback and cloud metadata targets.
- RSS parser skips unsafe item links.
- GitHub owner/repo path injection is rejected.
- arXiv parser accepts valid Atom entries and authors.
- X imports reject unsafe URLs.
- Prompt-injection text from X is preserved as data inside source cards with an untrusted-source warning.
- X recent search uses SQLite token fallback and advances cursor state.
- OAuth exchange and refresh store long token values without echoing them in reports.
- OAuth responses without tokens fail and do not create local secrets.
- Secret listing does not expose token values.
- MCP does not advertise or implement `secret_value_get`.
- MCP cursor tools and resources expose cursor state for debugging.

## Residual Risks

- `arcwell worker run-once` is deliberately simple. It has no durable lease timeout, exponential retry, retry budget, or dead-letter table yet.
- GitHub/RSS/arXiv adapters may duplicate source cards across repeated runs because provider-specific item-level cursors are not fully implemented.
- X recent search cursoring depends on `meta.newest_id`; query edits create new cursor keys by design.
- Passing secrets through CLI arguments may place them in shell history. Environment variables or SQLite secret set via trusted local workflows are preferable.
- SQLite secrets are local plaintext by design. This matches the current threat model, but backups should be treated as sensitive.
- Live X behavior still depends on API tier, rate limits, and OAuth app configuration that cannot be verified without real credentials.

## Validation

```sh
cargo fmt --all
cargo test
```

Current result after this phase: 46 tests passing.
