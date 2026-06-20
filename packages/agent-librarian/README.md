# agent-librarian

Wiki librarian and interestingness package.

Current first-pass implementation:

- Digest candidates can be created from source-card ids.
- Candidates are scored with transparent rule-based signals.
- Topics can be expanded into wiki pages through `librarian_expand_topic`.

MCP tools:

- `digest_candidate_create`
- `digest_candidate_list`
- `librarian_expand_topic`
- `source_card_add`
- `source_card_search`
- `wiki_expand_page`

Remaining work:

- Cluster related source cards across RSS, GitHub, arXiv, X, and web search.
- Add contradiction detection.
- Add delivery routing to Telegram/email.
- Add model-backed page synthesis with source-grounded citations.
