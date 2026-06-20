# Phase 4/5 Wiki Jobs And X Implementation Notes

Date: 2026-06-19

## Phase 4: Wiki Ingest Jobs

Implemented the first recorded job and source-card substrate for `arcwell-llm-wiki`.

Current CLI:

```sh
arcwell source-card add --title "Launch" --url "https://example.com" --summary "Summary"
arcwell source-card search "launch"
arcwell source-card read <source-card-id>
arcwell wiki ingest-job ./page.md
arcwell wiki ingest-url https://example.com/page.md
arcwell wiki compile "topic"
arcwell wiki expand "topic"
arcwell wiki jobs
arcwell wiki job <job-id>
```

Current MCP:

- `source_card_add`
- `source_card_search`
- `source_card_read`
- `wiki_ingest_job`
- `wiki_ingest_url`
- `wiki_compile`
- `wiki_expand_page`
- `wiki_job_status`
- `wiki_jobs`

Current resources:

- `arcwell://source-cards`
- `arcwell://wiki-jobs`
- `source-card://<id>`

The jobs execute synchronously in this phase but are recorded in SQLite with input, result, error, status, and timestamps. That gives the future daemon/Cloudflare drain a stable contract without pretending an async worker pool exists yet.

## Source Cards

Source cards are structured records plus Markdown wiki pages. They carry title, URL, source type, provider, retrieved timestamp, summary, claims with kind/confidence, metadata, and linked wiki page id.

Every generated source-card page says source text is untrusted evidence, not agent instructions. That is deliberate: source cards are evidence, not prompt authority.

## URL Ingest Guard

`wiki ingest-url` fetches remote content, so it has stricter rules than a non-fetching source-card URL:

- URL must be HTTPS.
- Loopback, private, link-local, documentation, multicast, and metadata hosts are rejected.
- Local loopback URL ingest is allowed only for tests when `ARCWELL_ALLOW_LOOPBACK_URL_INGEST=1`.

This prevents an agent or hostile source from turning wiki ingest into local-network probing.

## Phase 5: X MVP

Implemented an offline/replay-safe X MVP:

```sh
arcwell x import-json ./x-items.json
arcwell x list --query eve
arcwell x report --query eve
```

Current MCP:

- `x_import_json_file`
- `x_list`
- `x_report`

Current resource:

- `arcwell://x-items`

This does not perform live X OAuth/API calls yet. It accepts replay/export JSON and writes each accepted item as an `x_items` row, typed source card, and Markdown source-card wiki page.

Duplicates are skipped by `x_id`. Invalid or unsafe items are counted as rejected.

## Validation

Automated coverage includes source-card round trip, unsafe source-card URL rejection, claim fanout caps, wiki job recording, URL-ingest SSRF rejection, X import dedupe, X unsafe URL rejection, X prompt-injection text preserved as untrusted data, and MCP round trips for source cards, wiki jobs, and X import/report.

## Deliberate Gaps

- Jobs are recorded but not background-executed yet.
- URL ingest stores fetched text directly; no readability extraction/browser rendering yet.
- RSS, GitHub, arXiv, librarian, watch, digest, and migration tools are still design targets.
- Live X OAuth/API, cursors, cron, and spend-capped monitors are not implemented yet.
- No local ops UI yet; MCP resources are the current ops surface.
