# arcwell-librarian

**Status:** Scaffold/Partial.

Repository tracking: [STATUS.md](../../STATUS.md) and [TODO.md](../../TODO.md).

Wiki librarian and interestingness package.

Current first-pass implementation:

- Digest candidates can be created from source-card ids.
- Candidates are scored with transparent rule-based signals.
- Topics can be expanded into wiki pages through `librarian_expand_topic`.
- Expanded pages include deterministic source-card audit notes and exclude generated/model-answer, untrusted, and low-reliability source cards from primary evidence.
- Email send/reply exists in `arcwell-email` after recipient authorization, but
  librarian digest scheduling and delivery routing are not wired yet.

MCP tools:

- `digest_candidate_create`
- `digest_candidate_list`
- `librarian_expand_topic`
- `source_card_add`
- `source_card_search`
- `source_card_read`
- `wiki_expand_page`

Remaining work:

- Cluster related source cards across RSS, GitHub, arXiv, X, and web search.
- Add richer contradiction detection beyond deterministic source-card audit heuristics.
- Add model-backed extraction/synthesis only behind explicit config and source-grounded citation checks; no model-backed librarian synthesis is claimed today.
- Add delivery routing to Telegram/email with explicit recipient authorization,
  dedupe, quiet hours, loop prevention, policy/cost checks, and delivery attempt
  records.
- Add model-backed page synthesis with source-grounded citations.
