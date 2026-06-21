# arcwell-telegram

**Status:** Partial/Risk. Local drain/send/auth behavior exists in code and
tests; real Telegram bot/webhook behavior is still unproven.

Repository tracking: [STATUS.md](../../STATUS.md) and [TODO.md](../../TODO.md).

Telegram channel package.

Current implementation:

- Cloudflare worker `POST /telegram/webhook` normalizes Telegram text/caption updates into `arcwell-edge-inbox` events.
- Non-text Telegram updates are intentionally out of scope for the current
  package. Photos, videos, documents, stickers, reactions, edits, callbacks,
  polls, joins/leaves, and other update kinds are rejected as unsupported
  instead of being partially interpreted.
- Local `arcwell telegram drain` leases Telegram edge events, records them with `channel_record`, and acks/nacks the source event.
- Local `arcwell telegram send <chat-id> <text>` requires an explicit `telegram:chat:<chat-id>` authorization with `--send`, sends through Telegram `sendMessage`, escapes MarkdownV2 for the API call, records the outgoing channel message, and persists a delivery attempt with provider response, failed status, and retry hint when applicable.
- The resident worker path (`arcwell worker run` / `worker run-once`) retries
  due failed Telegram deliveries when `TELEGRAM_BOT_TOKEN` is available as a
  local Arcwell secret or environment variable. `TELEGRAM_API_BASE` can be set
  as a local secret for tests; otherwise the worker uses Telegram's production
  API endpoint.
- Local `arcwell telegram authorize <subject> --write-projects --send` grants project-write/binding and send rights to subjects such as `telegram:chat:123`, `telegram:user:456`, or `telegram:@username`.
- Local `arcwell telegram deliveries [--message-id <id>]` lists persisted delivery attempts.
- MCP tools `telegram_drain_edge_events` and `telegram_send_message` expose the same behavior to agents.
- Project-aware routing can bind an explicit `projectId` in payloads only for authorized subjects. Authorized chats can also auto-bind a Telegram message to a uniquely resolved project from the message text. Ambiguous or missing matches remain unbound.
- `scripts/telegram-live-smoke` runs local authorization checks in a preserved smoke home and, when live credentials are supplied, sets the Telegram webhook, sends a safe outgoing reply, drains Cloudflare edge events locally, and asserts the exact incoming message is recorded exactly once. Failure artifacts and mismatch/duplicate diagnostics are kept under the smoke home.

Channel safety rules:

- Telegram text is untrusted user/content data.
- Captions are treated as text-only message content; attached media bytes and
  file references are not stored or fetched.
- Formatting must be normalized before delivery.
- Incoming update ids are idempotency keys.
- Project switching must resolve through the project registry, and ambiguity must stop the action.
- Sender/chat authorization is required before Telegram events can mutate or bind project state.
- Chat send authorization is required before outgoing Telegram sends or retries.
- Telegram provider transport errors are stored as classified retryable errors, not raw provider URLs, because Telegram API URLs include the bot token.

Relevant MCP tools:

- `edge_event_enqueue`
- `edge_event_lease`
- `edge_event_ack`
- `edge_event_nack`
- `channel_record`
- `channel_list`
- `channel_authorize`
- `channel_authorizations`
- `channel_delivery_list`
- `telegram_drain_edge_events`
- `telegram_send_message`

Live exact incoming proof:

```sh
set -a
. ./.env
set +a
ARCWELL_TELEGRAM_LIVE_CONFIRM=disposable scripts/telegram-live-smoke
```

When prompted, send the exact phrase printed by the script to the disposable
Telegram chat. If the run times out, re-run with the printed
`TELEGRAM_SMOKE_EXPECT_TEXT=... ARCWELL_SMOKE_HOME=...` command so the same
assertion can be repeated against the preserved local state. Do not paste bot
tokens into logs; the smoke prints names, paths, and sanitized diagnostics only.
- `project_resolve`
- `ops_snapshot`
