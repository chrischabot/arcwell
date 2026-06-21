# arcwell-llm-wiki

**Status:** Partial.

Local source-backed Markdown wiki.

Current implementation:

```sh
arcwell wiki ingest-file ./some-page.md
arcwell wiki ingest-dir ./corpus
arcwell wiki import-codex-swift-sources /path/to/codex-swift
arcwell wiki sources
arcwell wiki ingest-job ./some-page.md
arcwell wiki ingest-url https://example.com/page.md
arcwell wiki enqueue-rss https://example.com/feed.xml
arcwell wiki enqueue-github-owner openai --limit 10
arcwell wiki enqueue-github openai codex --mode releases --limit 10
arcwell wiki enqueue-github openai codex --mode commits --limit 10
arcwell wiki enqueue-arxiv "cat:cs.AI" --limit 10
arcwell worker run-once --max-jobs 10
arcwell worker run --max-jobs-per-tick 10 --idle-sleep-ms 5000
arcwell wiki run-rss https://example.com/feed.xml
arcwell wiki run-github-owner openai --limit 10
arcwell wiki run-github openai codex --mode releases
arcwell wiki run-arxiv "cat:cs.AI"
arcwell wiki compile "agent infrastructure"
arcwell wiki expand "agent infrastructure"
arcwell wiki jobs
arcwell source-card add --title "Source" --url "https://example.com" --summary "Summary"
arcwell wiki search "agent infrastructure"
arcwell wiki list
arcwell wiki read <page-id>
```

MCP tools:

- `wiki_ingest_file`
- `wiki_ingest_dir`
- `wiki_import_codex_swift_sources`
- `wiki_watch_sources`
- `wiki_ingest_job`
- `wiki_ingest_url`
- `wiki_enqueue_rss`
- `wiki_enqueue_github_owner`
- `wiki_enqueue_github`
- `wiki_enqueue_arxiv`
- `wiki_compile`
- `wiki_expand_page`
- `wiki_job_status`
- `wiki_jobs`
- `worker_run_once`
- `cursor_list`
- `cursor_get`
- `wiki_search`
- `wiki_read`
- `source_card_add`
- `source_card_search`
- `source_card_read`

Boundary:

- The wiki is knowledge/corpus state, not personal mem0 memory.
- Pages are Markdown files under `ARCWELL_HOME/wiki/pages`.
- SQLite stores metadata and checksums.
- SQLite FTS indexes wiki page title/body content for local search; each page write updates the index.
- Source cards are structured records plus Markdown wiki pages with schema version metadata, evidence role, trust level, extracted dates/entities, and deterministic audit flags.
- Source cards also carry deterministic reliability metadata: reliability score, provenance strength, inferred source owner, crawl-rate policy, and robots metadata when available from URL ingest. These are quality gates, not a model-backed credibility verdict.
- Watch sources are durable monitor configuration in SQLite, separate from source-card evidence.
- `arcwell wiki import-codex-swift-sources` imports the old Codex Swift seed registry from `llm-wiki.md` and `scripts/wiki-sources-restore.sh`, merging duplicate sources idempotently.
- Wiki jobs can be queued as `pending` and drained by `arcwell worker run-once`, the resident `arcwell worker run` loop, or the MCP `worker_run_once` tool.
- Worker claims use leases. Failed jobs retry with bounded backoff and become `dead_lettered` after their attempt budget is exhausted.
- RSS/Atom, GitHub owner/repo, GitHub releases/commits, and arXiv search adapters write source cards rather than directly rewriting topic pages.
- Adapter cursor state lives in SQLite and is exposed through cursor CLI/MCP reads for debugging.
- Source-health state records last success, last failure, last item id/date, cursor key/value, and next run hints for RSS, GitHub, arXiv, and X recent search. Rate-limit/quota failures are classified as `rate_limited` with longer backoff instead of generic failure.
- Scheduled polling hooks can enqueue due active watch sources while respecting source-health `next_run_at`; the local worker still has to drain the queued jobs.
- Source cards are keyed by canonical URL/provider/type so repeated adapter runs update existing cards instead of flooding source-card rows or wiki artifacts.
- `sync_wiki_dir` performs incremental Markdown sync and tombstones pages whose source Markdown file disappeared so deleted local files stop appearing in list/search evidence.
- Current search uses a local SQLite FTS title/body index over active pages. Hybrid/vector search comes later.
- URL ingest is HTTPS-only, rejects local/private/metadata hosts, validates redirects, enforces content type and size bounds, and writes provenance separately from cleaned readable text. HTML extraction is deterministic/readability-like (`article`/`main`/`body` preference plus boilerplate removal), not browser-rendered or model-backed.
- External source text is treated as evidence, not instructions; generated source-card pages include an untrusted-source warning and are excluded from local-source evidence for later research briefs.

Cloudflare boundary:

- Always-on feed polling, webhook capture, and queue buffering still belong in the optional Cloudflare component.
- The local Rust service is the durable owner of wiki pages, source cards, cursor state, and final ingestion.
- Cloudflare should hold short-lived events with max-age and size limits, then let the local service drain them over MCP/HTTP when online.
