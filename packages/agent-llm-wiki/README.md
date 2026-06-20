# agent-llm-wiki

Local source-backed Markdown wiki.

Current implementation:

```sh
agent wiki ingest-file ./some-page.md
agent wiki ingest-job ./some-page.md
agent wiki ingest-url https://example.com/page.md
agent wiki enqueue-rss https://example.com/feed.xml
agent wiki enqueue-github openai codex --mode releases --limit 10
agent wiki enqueue-github openai codex --mode commits --limit 10
agent wiki enqueue-arxiv "cat:cs.AI" --limit 10
agent worker run-once --max-jobs 10
agent worker run --max-jobs-per-tick 10 --idle-sleep-ms 5000
agent wiki run-rss https://example.com/feed.xml
agent wiki run-github openai codex --mode releases
agent wiki run-arxiv "cat:cs.AI"
agent wiki compile "agent infrastructure"
agent wiki expand "agent infrastructure"
agent wiki jobs
agent source-card add --title "Source" --url "https://example.com" --summary "Summary"
agent wiki search "agent infrastructure"
agent wiki list
agent wiki read <page-id>
```

MCP tools:

- `wiki_ingest_file`
- `wiki_ingest_job`
- `wiki_ingest_url`
- `wiki_enqueue_rss`
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
- Pages are Markdown files under `AGENT_SERVICES_HOME/wiki/pages`.
- SQLite stores metadata and checksums.
- Source cards are structured records plus Markdown wiki pages.
- Wiki jobs can be queued as `pending` and drained by `agent worker run-once`, the resident `agent worker run` loop, or the MCP `worker_run_once` tool.
- Worker claims use leases. Failed jobs retry with bounded backoff and become `dead_lettered` after their attempt budget is exhausted.
- RSS/Atom, GitHub releases/commits, and arXiv search adapters write source cards rather than directly rewriting topic pages.
- Adapter cursor state lives in SQLite and is exposed through cursor CLI/MCP reads for debugging.
- Current search is simple title/body substring search. Hybrid/vector search comes later.
- URL ingest is HTTPS-only and rejects local/private/metadata hosts.
- External source text is treated as evidence, not instructions; generated source-card pages include an untrusted-source warning.

Cloudflare boundary:

- Always-on feed polling, webhook capture, and queue buffering still belong in the optional Cloudflare component.
- The local Rust service is the durable owner of wiki pages, source cards, cursor state, and final ingestion.
- Cloudflare should hold short-lived events with max-age and size limits, then let the local service drain them over MCP/HTTP when online.
