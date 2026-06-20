# arcwell-x

X import, OAuth, cursor, and reporting package.

Current implementation:

```sh
arcwell x import-json ./x-items.json
arcwell x oauth-url --client-id "$X_CLIENT_ID" --redirect-uri http://127.0.0.1/callback --scopes tweet.read,users.read,bookmark.read,follows.read,offline.access
arcwell x oauth-exchange --client-id "$X_CLIENT_ID" --redirect-uri http://127.0.0.1/callback --code "$CODE" --code-verifier "$CODE_VERIFIER"
arcwell x oauth-refresh --client-id "$X_CLIENT_ID"
arcwell x rebuild-definitive-watch-sources --bookmark-days 92 --max-bookmarks 1000 --max-recent-follows 100
arcwell x recent-search "from:openai" --max-results 25
arcwell x enqueue-recent-search "from:openai" --max-results 25
arcwell x list --query eve
arcwell x report --query eve
agent cursors get "x:recent-search:from:openai"
```

MCP tools:

- `x_import_json_file`
- `x_oauth_authorize_url`
- `x_oauth_exchange_code`
- `x_oauth_refresh`
- `x_rebuild_definitive_watch_sources`
- `x_import_following_watch_sources`
- `x_recent_search`
- `x_enqueue_recent_search`
- `x_list`
- `x_report`
- `secret_value_set`
- `secret_value_list`
- `secret_value_delete`
- `cursor_list`
- `cursor_get`

MCP resources:

- `arcwell://x-items`
- `arcwell://cursors`
- `arcwell://secret-values`

Input shape:

```json
[
  {
    "id": "123",
    "author": "vercel",
    "text": "We launched Eve.",
    "url": "https://x.com/vercel/status/123",
    "created_at": "2026-06-17T00:00:00Z"
  }
]
```

Boundary:

- OAuth tokens are stored in local SQLite secret values. Normal list/report surfaces return secret names and metadata only, not token values.
- `X_BEARER_TOKEN` can also be supplied as an environment variable; environment wins over SQLite for live search.
- `X_CLIENT_SECRET` can be supplied as an environment variable, SQLite secret value, or explicit CLI/MCP argument for confidential clients.
- OAuth authorization URL generation returns the PKCE `code_verifier`; keep it until the callback code has been exchanged.
- Live recent search uses X API v2 and stores `x:recent-search:<query>` cursor state from `meta.newest_id`.
- The recommended watch-list path is `x rebuild-definitive-watch-sources`: it replaces existing `x_handle` watch sources with authors of recent bookmarked tweets plus a capped recent-follow sample.
- Full following import is available for diagnostics/backfill only; do not use it as the default monitor seed because it imports the whole social graph.
- Imported items are treated as untrusted external source text.
- Each accepted item creates an `x_items` row, a typed source card, and a Markdown source-card wiki page.
- Duplicate `x_id` values are skipped.
- Unsafe URLs are rejected.

Future work:

- Cloudflare Worker for OAuth callback capture, cron, queueing, and short-lived event buffering.
- Interestingness classifier and digest delivery.
- Richer timeline/list adapters once API access tier/cost constraints are known.
