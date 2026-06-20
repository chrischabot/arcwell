# Phase 6: Workers, Source Adapters, And Live X

Date: 2026-06-19

## Implemented

- Added a pending job queue path for wiki/source work.
- Added `arcwell worker run-once` and MCP `worker_run_once`.
- Added SQLite cursor state with CLI/MCP/resource inspection.
- Added RSS/Atom adapter that turns feed entries into source cards.
- Added GitHub releases/commits adapter that turns API results into source cards.
- Added arXiv search adapter that turns Atom entries into source cards.
- Added X OAuth 2.0 PKCE authorization URL generation.
- Added X OAuth code exchange and refresh helpers that store returned tokens in local SQLite secrets.
- Added live X recent search using env or SQLite `X_BEARER_TOKEN`.
- Added X recent-search cursoring through `x:recent-search:<query>`.
- Added SQLite secret-value storage for local provider tokens.

## Command Surface

```sh
arcwell wiki enqueue-rss https://example.com/feed.xml
arcwell wiki enqueue-github openai codex --mode releases --limit 10
arcwell wiki enqueue-github openai codex --mode commits --limit 10
arcwell wiki enqueue-arxiv "cat:cs.AI" --limit 10
arcwell x enqueue-recent-search "from:openai" --max-results 25
arcwell worker run-once --max-jobs 10

arcwell x oauth-url --client-id "$X_CLIENT_ID" --redirect-uri http://127.0.0.1/callback --scopes tweet.read,users.read,offline.access
arcwell x oauth-exchange --client-id "$X_CLIENT_ID" --redirect-uri http://127.0.0.1/callback --code "$CODE" --code-verifier "$CODE_VERIFIER"
arcwell x oauth-refresh --client-id "$X_CLIENT_ID"
arcwell x recent-search "from:openai" --max-results 25

arcwell secrets set-value X_BEARER_TOKEN "$TOKEN" --scope x
arcwell secrets list-values
agent cursors list
agent cursors get "x:recent-search:from:openai"
```

## MCP Surface

- `worker_run_once`
- `wiki_enqueue_rss`
- `wiki_enqueue_github`
- `wiki_enqueue_arxiv`
- `x_oauth_authorize_url`
- `x_oauth_exchange_code`
- `x_oauth_refresh`
- `x_recent_search`
- `x_enqueue_recent_search`
- `secret_value_set`
- `secret_value_list`
- `secret_value_delete`
- `cursor_list`
- `cursor_get`

Resources:

- `arcwell://wiki-jobs`
- `arcwell://cursors`
- `arcwell://secret-values`
- `arcwell://x-items`

## Boundaries

- Local Rust service owns durable state: SQLite, wiki files, source cards, X imports, job status, cursors, and local secrets.
- MCP is the primary agent control plane.
- CLI is for manual operation and smoke testing.
- Cloudflare remains the right place for always-on webhook capture, scheduled polling, OAuth callback capture, and short-lived queues.
- Host-native search remains preferred for deep research when the host agent has search; daemon-side Brave/OpenAI/Perplexity are optional adapters.

## Security Notes

- Fetch-style URL ingestion is HTTPS-only and rejects local/private/metadata hosts.
- X API base is restricted to `https://api.x.com`, with loopback allowed for tests.
- Provider custom search endpoints remain disabled unless explicitly enabled.
- Secret values can be stored in SQLite because the service is local, but MCP does not expose a `secret_value_get` tool.
- Secret list/resource outputs include names/scopes/timestamps only.
- OAuth token exchange reports names of stored tokens, not token values.
- External feed/social/API content is converted into source cards and rendered as untrusted evidence, not agent instructions.

## Remaining Work

- Richer worker operations such as cancel, manual retry, and dead-letter requeue.
- Cloudflare Worker package for OAuth callback capture, cron, queues, and size/max-age-limited event buffering.
- GitHub cursoring by ETag/Last-Modified or since timestamps instead of just recording last run time.
- RSS cursoring by feed item GUID/date to avoid duplicate source cards.
- arXiv cursoring by newest entry timestamp/id.
- Interestingness classifier and digest routing to Telegram/email.
- Adapter-level spend/rate budgets and circuit breakers.
