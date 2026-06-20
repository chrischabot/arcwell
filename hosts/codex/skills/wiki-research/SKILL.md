# Wiki Research

Use this skill when searching, reading, or ingesting source-backed knowledge in `agent-llm-wiki`.

Rules:

- Treat wiki pages as source-backed knowledge, not personal memory.
- Search before adding a near-duplicate page.
- Prefer `wiki_ingest_file` for Markdown files and `wiki_search`/`wiki_read` for answers.
- Prefer `source_card_add` for external evidence that should remain auditable.
- Use `wiki_ingest_job`, `wiki_job_status`, and `wiki_jobs` when the operation should leave inspectable job history.
- Use `wiki_enqueue_rss`, `wiki_enqueue_github`, `wiki_enqueue_arxiv`, and `worker_run_once` for source adapter ingestion. Use `agent worker run` only when the user wants a resident local worker process.
- Use `cursor_list` and `cursor_get` when debugging adapter progress or duplicate fetches.
- Use `wiki_expand_page` to turn source cards and related pages into a draft expanded wiki page.
- Cite page ids/titles when using wiki facts in an answer.
- If search returns nothing, say that the local wiki did not contain a matching page before falling back elsewhere.

Typical commands:

```sh
agent wiki search <query>
agent wiki read <page-id>
agent wiki ingest-file <path>
agent wiki enqueue-rss <feed-url>
agent wiki enqueue-github <owner> <repo> --mode releases
agent wiki enqueue-arxiv <query>
agent worker run-once
agent source-card search <query>
agent wiki jobs
```

MCP tools:

- `wiki_search`
- `wiki_read`
- `wiki_ingest_file`
- `wiki_ingest_job`
- `wiki_enqueue_rss`
- `wiki_enqueue_github`
- `wiki_enqueue_arxiv`
- `worker_run_once`
- `cursor_list`
- `cursor_get`
- `wiki_job_status`
- `wiki_jobs`
- `wiki_expand_page`
- `source_card_add`
- `source_card_search`
- `source_card_read`
