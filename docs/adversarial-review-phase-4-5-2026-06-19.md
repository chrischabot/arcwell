# Adversarial Review Phase 4/5, 2026-06-19

## Scope

Reviewed and tested the new Phase 4/5 work:

- Typed source cards.
- Recorded wiki jobs.
- URL ingest.
- X JSON import/report.
- MCP exposure for all of the above.

## Claims Tested

- External evidence is stored as structured source cards plus inspectable Markdown.
- Source text is framed as untrusted data, not instructions.
- Wiki jobs leave visible status/result/error records.
- URL ingest cannot fetch loopback/private/metadata hosts.
- X imports dedupe by `x_id`, reject unsafe URLs, and preserve hostile text only as evidence.
- MCP tools enforce the same validation as local APIs.

## Findings Fixed

### URL Ingest Could Fetch Internal Hosts

Score: 75.

Evidence: the first implementation reused generic source URL validation for `wiki ingest-url`. That was acceptable for non-fetching source cards, but wrong for an operation that performs network egress.

Root cause: source identity URL validation and fetch-target validation were conflated.

Fix: added a dedicated fetch URL validator. Fetches now require HTTPS and reject loopback, private, link-local, documentation, multicast, and metadata hosts. Loopback is available only under `AGENT_SERVICES_ALLOW_LOOPBACK_URL_INGEST=1` for tests.

Validation: severe tests reject `127.0.0.1`, `169.254.169.254`, and `metadata.google.internal`; MCP URL-ingest rejects loopback too.

## Severe Tests Added

- Source-card round trip writes a wiki artifact with untrusted-evidence framing.
- Source-card rejects unsafe URL schemes.
- Source-card rejects excessive claim fanout.
- Wiki file ingest job records completion.
- Wiki expand job records completion.
- Wiki URL ingest rejects loopback and metadata hosts.
- X import dedupes duplicate `x_id` values.
- X import rejects unsafe URLs.
- X prompt-injection text is preserved as data inside a source-card page.
- MCP source-card add/read and wiki expand round trip.
- MCP URL ingest rejects loopback.
- MCP X import/report round trip.

## Validation Commands

```sh
cargo fmt --all -- --check
cargo test
```

Additional runtime smoke covered:

- CLI source-card add/search.
- CLI wiki ingest-job/expand/jobs.
- CLI X import/list/report.
- MCP source-card, wiki-expand, X import/report.

## Remaining Risk

- Jobs are still synchronous. A real background daemon will need lease/ack/retry/dead-letter semantics.
- URL ingest stores fetched text directly. Browser rendering/readability extraction is not implemented.
- X live OAuth/API and cursors are not implemented.
- X interestingness scoring and delivery are not implemented.
- Source-card schema is stored in SQLite but not yet emitted as versioned JSON Schema.
