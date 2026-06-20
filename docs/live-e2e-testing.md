# Live E2E Testing

Local live-test secrets are imported into:

- `.env` for shell-loaded local runs.
- `.dev.vars` for Cloudflare Worker local development.
- `/Users/chabotc/.arcwell/arcwell.sqlite3` `secret_values` for MCP/daemon use.

These files contain secrets and are intentionally gitignored.

Provider aliases added during import:

- `BRAVE_API_KEY` from `BRAVE_SEARCH_API_KEY`
- `PERPLEXITY_API_KEY` from `PERPLEXITYAI_API_KEY`
- `X_BEARER_TOKEN` from `TWITTER_BEARER_TOKEN`
- `X_CLIENT_ID` from `TWITTER_OAUTH2_CLIENT_ID`
- `X_CLIENT_SECRET` from `TWITTER_OAUTH2_CLIENT_SECRET`
- `ARCWELL_EDGE_SECRET` from `TELEGRAM_WEBHOOK_SECRET`

Smoke commands:

```sh
set -a
. ./.env
set +a

arcwell research search "OpenAI agent news" --provider brave --max-results 1
arcwell research search "OpenAI agent news" --provider perplexity --max-results 1
arcwell x recent-search "from:openai" --max-results 10
arcwell secrets list-values
```

Telegram `getMe` should be checked with a script that reads `TELEGRAM_BOT_TOKEN` from the environment rather than putting the token in shell history.
