# Deep Research

Use this skill for multi-source research, trend reports, launch analysis, technical scans, and questions where freshness or contradictions matter.

Rules:

- Start with `research_plan` to get local wiki context and suggested searches.
- Use `research_workflow_create` for substantial research so scout, extractor, skeptic, and synthesizer work is tracked.
- Use host-native web search for current claims. Do not rely only on local wiki pages when the topic may have changed.
- If native search is unavailable or insufficient, use `research_web_search` with `provider=openai`, `provider=brave`, or `provider=perplexity` when API keys are configured.
- Prefer primary sources first: official docs, release notes, source repos, papers, company blogs, and named-person posts.
- Use `wiki_enqueue_github` for GitHub repo releases/commits, `wiki_enqueue_arxiv` for papers, and `wiki_enqueue_rss` for feeds when those sources should become durable source cards.
- Use `x_recent_search` or `x_enqueue_recent_search` when X is a relevant primary/near-primary signal.
- Use secondary analysis to find controversy, missing context, and implications.
- Write durable source cards or notes into `arcwell-llm-wiki` before producing a final brief.
- Use typed source cards for external evidence; do not bury source provenance only in prose.
- Call `research_brief_from_wiki` after source cards are in place.
- Treat generated `Research Brief:` pages as outputs, not evidence.
- Record retrieval date in source cards for current or fast-moving topics.

MCP tools:

- `research_plan`
- `research_web_search`
- `research_workflow_create`
- `research_tasks`
- `research_task_complete`
- `research_brief_from_wiki`
- `research_runs`
- `wiki_ingest_file`
- `wiki_enqueue_rss`
- `wiki_enqueue_github`
- `wiki_enqueue_arxiv`
- `worker_run_once`
- `x_recent_search`
- `x_enqueue_recent_search`
- `source_card_add`
- `source_card_search`
- `source_card_read`
- `wiki_expand_page`
- `wiki_search`
- `wiki_read`

Suggested subagent roles when available:

- `research-scout`: finds primary and high-signal secondary sources.
- `source-extractor`: converts sources into compact source cards with claims, dates, links, and caveats.
- `skeptic`: searches for contradictions, stale assumptions, security/privacy issues, and missing sources.
- `synthesizer`: creates the final brief from source cards and audit notes.

Minimum output discipline:

- Answer with sourced claims.
- Separate confirmed facts, interpretation, and open questions.
- Include links for external sources and wiki page ids for local sources.
- Say plainly when the local wiki has no matching context.
