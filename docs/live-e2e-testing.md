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
scripts/x-live-smoke
arcwell secrets list-values
arcwell secrets health
```

`arcwell secrets list-values` and `arcwell secrets health` must show names,
scope/provider metadata, and expiry status only. They must not print secret
values. Use `--expires-at <RFC3339>` when importing short-lived local tokens so
ops can warn before a provider path uses a stale credential.

Backup smoke for homes with local SQLite secrets:

```sh
arcwell backup create
arcwell backup verify
```

Expected result: the backup manifest reports
`contains_local_secret_values: true` when local `secret_values` exist and never
prints raw local secret values. Treat those backup directories as sensitive and
encrypt or delete them when rotating/revoking the underlying credential.

## X Live Smoke

Status: local replay/source-card smoke passes. Live X recent search runs with
the available bearer token, but definitive watch rebuild is blocked because the
available token is application-only and X requires OAuth 1.0a User Context or
OAuth 2.0 User Context for the account-data endpoints used by bookmarks/follows.

Use a disposable/current X bearer token with scopes/API tier that allow recent
search, bookmarks, follows, and user lookup. Do not paste tokens into chat
transcripts or shell history.

```sh
X_BEARER_TOKEN=... scripts/x-live-smoke
```

The script uses a disposable `ARCWELL_HOME` by default and verifies:

- local replayed X JSON writes an item, source card, and wiki page with
  prompt-injection text preserved as untrusted evidence;
- secret-health output shows `X_BEARER_TOKEN` presence without printing the
  token;
- live `x recent-search "from:openai"` returns count-shaped output;
- live `x rebuild-definitive-watch-sources` returns watch-list audit counts;
- live `x monitor-watch-sources` polls the definitive watch list and returns
  per-source cursor/failure/digest audit fields;
- cursor output does not expose token values.

Current blocker:

- `x rebuild-definitive-watch-sources` fails with X `403 Unsupported
  Authentication` when run with the available application-only bearer token.
  Provide a user-context token/scope before claiming bookmarks/follows/watch
  monitoring live proof.

Telegram `getMe` should be checked with a script that reads `TELEGRAM_BOT_TOKEN` from the environment rather than putting the token in shell history.

## Claude MCP Validation Smoke

Status: local stdio MCP validation passed on 2026-06-20; authenticated Claude
Desktop/Code host UI validation remains blocked.

Command:

```sh
cargo build -p arcwell
scripts/claude-mcp-smoke
```

Recorded result:

- `arcwell mcp` started as a stdio subprocess against disposable homes.
- `initialize`, `tools/list`, `resources/list`, `arcwell_health`,
  `profile_set`, `resources/read`, `project_create`, and
  `project_status_get` returned valid JSON-RPC responses.
- Expected Claude-relevant tools were listed: `profile_list`,
  `memory_search`, `wiki_search`, `ops_snapshot`, and `project_status_get`.
- Expected resources were listed: `arcwell://profile`, `arcwell://wiki`, and
  `arcwell://ops`.
- A malformed JSON-RPC frame returned a parse error with `id: null`.
- An unsupported MCP method and an unknown resource URI returned explicit MCP
  errors instead of silent success.
- A bounded 256 KiB profile/resource response round-tripped without truncation
  or runaway output amplification.
- A huge missing `wiki://page/...` resource read returned explicit JSON `null`,
  matching the nullable wiki-page read contract.
- A bad `ARCWELL_HOME` path that points at a file returned an explicit MCP
  error.
- Missing `ARCWELL_HOME` fell back to a disposable `HOME/.arcwell`.
- Claude lifecycle/thread inventory remained explicitly unavailable while manual
  project snapshots were reported as supported.

Current blocker:

- The local Claude Desktop config at
  `~/Library/Application Support/Claude/claude_desktop_config.json` did not
  define `mcpServers.arcwell`, so this audit did not prove that an
  authenticated Claude Desktop/Code UI session listed or called Arcwell tools.
  Re-run `scripts/claude-mcp-smoke --require-host-config` after installing the
  host config, then validate in Claude Desktop/Code or the MCP Inspector.

## Service Supervision Live Smoke

Status: passed on 2026-06-20 on macOS
`Darwin MacBook-Pro-3 25.4.0 arm64`.

Command:

```sh
scripts/service-live-smoke
```

Recorded result:

- Disposable no-load checks passed with `HOME` and `ARCWELL_HOME` paths
  containing spaces plus shell/XML-hostile characters.
- `doctor --strict` passed with a generated worker LaunchAgent plist, fresh
  backup, and fresh heartbeat.
- `doctor --strict` failed for a stale worker heartbeat.
- `doctor --strict` failed for corrupt service metadata and for a missing worker
  binary.
- `arcwell service logs` reported explicit per-stream read status for unreadable
  log files instead of silently returning empty output.
- `arcwell service uninstall --no-unload` removed only the disposable plist and
  left unrelated disposable `ARCWELL_HOME` data intact.
- Real launchd bootstrap passed for a disposable user service label
  `com.arcwell.worker`.
- `arcwell service status` saw the loaded launchd service.
- `arcwell service restart` changed the worker pid through
  `launchctl kickstart -k`.
- Killing the worker pid was recovered by launchd.
- `arcwell service logs` read at least one live log stream.
- Live uninstall removed the disposable plist and left unrelated
  `ARCWELL_HOME` data intact.

Safety behavior:

- The script refuses live launchd operations when an existing
  `com.arcwell.worker` service is loaded, unless `--allow-existing` is passed.
- Use `--no-live` to run only the no-load adversarial checks.
- Use `--live` to require launchd coverage and fail if launchd is unavailable.
- Use `--keep-temp` to preserve the disposable home for inspection.

Linux status: systemd user-service support is explicitly deferred. The target
unit shape is documented in `docs/packaging-and-operations.md`, but no Linux
implementation is claimed until a Linux host can run and verify
`systemctl --user` install/status/restart/recovery/logs/uninstall.

## Release/Install Readiness Smoke

Status: passed on 2026-06-20 with a disposable package prefix, `HOME`, and
`ARCWELL_HOME` containing spaces and shell/XML-hostile characters.

Commands:

```sh
cargo build --release -p arcwell
scripts/release-readiness-smoke
```

Recorded result:

- Candidate release binary started from a staged package prefix.
- Stable Codex plugin `.mcp.json` invoked `arcwell mcp`.
- Stable plugin hooks did not reference `cargo`, `target/debug`,
  `.arcwell-dev`, or `arcwell-dev`.
- A stale `arcwell` earlier on `PATH` was detected, and the smoke proved the
  package prefix must be first for the plugin to reach the candidate binary.
- Interrupted upgrade simulation left the installed binary hash unchanged.
- Backup create, verify, restore, and profile readback passed in disposable
  homes.
- `doctor --strict` rejected an old/unsupported schema version.
- Duplicate `service install --no-load` produced one LaunchAgent plist.
- `service uninstall --no-unload` was idempotent and preserved unrelated
  `ARCWELL_HOME` data.
- Bad log path and unwritable home cases failed explicitly.

Publication status:

- Homebrew formula/tap, signed GitHub release artifacts, checksum-verifying
  installer script, Linux systemd package support, and fresh-thread Codex app
  plugin smoke remain blocked/unimplemented. Ready inputs and blockers are in
  `docs/packaging-and-operations.md`.

## Cloudflare Edge Inbox Staging Smoke

Status: passed on 2026-06-20 against
`https://arcwell-edge-inbox.chabotc.workers.dev`.

The first authenticated live run found a real deployed-boundary issue:
`scripts/edge-live-smoke` requested a 1-second lease to prove lease expiry, but
the worker HTTP handler clamped `leaseSeconds` to a 30-second minimum, so the
retry proof returned no event. The worker now accepts 1-second staging leases
and has a regression test for the HTTP lease endpoint.

Prerequisites:

- A staging `ARCWELL_EDGE_NEXT_SECRET` is configured so current/next rotation
  can be tested.
- `EDGE_URL` points at the deployed staging worker.
- A Cloudflare D1 database is created and wired to the worker as `EDGE_DB`.
- `wrangler.jsonc` has the real staging D1 database id, not the placeholder.
- For a small bounded rate-limit proof, configure staging with a low
  `RATE_LIMIT_MAX_EVENTS` such as `5`, or set `EDGE_SMOKE_RATE_LIMIT_PROBES`
  high enough to exceed the configured staging limit.

Commands:

```sh
cd packages/arcwell-edge-inbox/worker
npm run typecheck
npm test
npx wrangler d1 execute arcwell_edge_inbox --remote --command "SELECT 1"
npx wrangler deploy
```

Full severe live smoke:

```sh
export EDGE_URL="https://<worker-host>"
export ARCWELL_EDGE_SECRET="<current staging secret>"
export ARCWELL_EDGE_NEXT_SECRET="<next staging secret>"
scripts/edge-live-smoke
```

The script uses a temporary `ARCWELL_HOME`, unique synthetic ids, and bounded
requests only. It verifies:

- forged `x-arcwell-edge-secret` is rejected;
- current and next edge secrets both work during rotation;
- duplicate idempotency keys are not duplicated locally;
- a synthetic event survives an offline wait and is drained into local SQLite;
- direct lease without ack is retried after lease expiry;
- local drain nacks, and does not ack, when local persistence rejects the event;
- replay/rate-limit probes return `429` before the staging queue is filled.

Manual synthetic event smoke:

```sh
EDGE_URL="https://<worker-host>"
curl -sS -X POST "$EDGE_URL/events" \
  -H "content-type: application/json" \
  -H "x-arcwell-edge-secret: $ARCWELL_EDGE_SECRET" \
  --data '{"source":"smoke","idempotencyKey":"smoke:edge:1","payload":{"ok":true},"maxAgeSeconds":300}'

curl -sS -X POST "$EDGE_URL/drain/lease" \
  -H "content-type: application/json" \
  -H "x-arcwell-edge-secret: $ARCWELL_EDGE_SECRET" \
  --data '{"leaseSeconds":30}'
```

Expected result:

- `/events` returns `accepted: true`.
- `/drain/lease` returns the same `idempotencyKey`.
- Repeating `/events` with the same idempotency key returns `duplicate: true`.
- A request with a bad secret returns `401`.
- A replay storm over `RATE_LIMIT_MAX_EVENTS` returns `429`.
- `arcwell edge drain-remote` acks only after the event is persisted locally and
  nacks persistence failures.

Recorded 2026-06-20 evidence:

```sh
cd packages/arcwell-edge-inbox/worker
npx wrangler deploy
npx wrangler secret list
npx wrangler d1 execute arcwell_edge_inbox --remote --command "SELECT 1 AS ok"
curl -sS https://arcwell-edge-inbox.chabotc.workers.dev/health
curl -sS -o /tmp/arcwell-edge-forged.json -w '%{http_code}\n' \
  -X POST https://arcwell-edge-inbox.chabotc.workers.dev/events \
  -H 'content-type: application/json' \
  -H 'x-arcwell-edge-secret: forged-live-smoke' \
  --data '{"source":"smoke","idempotencyKey":"forged-live-smoke","payload":{},"maxAgeSeconds":60}'
```

Result: deploy passed to
`https://arcwell-edge-inbox.chabotc.workers.dev` version
`1ac44b72-a58a-4f31-81f2-7bf36afa8271`; D1 `SELECT 1` passed against
`arcwell_edge_inbox` (`e467e06a-4623-4b1f-aa0a-866c7f645df6`); `/health`
returned durable `ok: true`; forged `/events` returned `401`. Authenticated
current/next-secret worker ingress, remote lease/ack/nack/expiry/rate-limit,
and local Rust drain were not run because this shell had no current secret
value and `npx wrangler secret list` showed no `ARCWELL_EDGE_NEXT_SECRET`.

Additional authenticated smoke:

```sh
cd packages/arcwell-edge-inbox/worker
npx wrangler deploy
npx wrangler secret put ARCWELL_EDGE_NEXT_SECRET
cd ../../..
EDGE_URL="https://arcwell-edge-inbox.chabotc.workers.dev" \
ARCWELL_EDGE_NEXT_SECRET="<redacted>" \
EDGE_SMOKE_RATE_LIMIT_PROBES=130 \
scripts/edge-live-smoke
```

Result: deploy passed to version
`df79f7e0-574c-483e-8cf8-a67ba95bae12`, `ARCWELL_EDGE_NEXT_SECRET` was present
in `npx wrangler secret list`, and `scripts/edge-live-smoke` passed. The smoke
proved current and next secret auth, forged-secret rejection, D1 persistence,
duplicate idempotency, local Rust drain with ack-after-persist, lease
expiry/retry, nack without ack on local persistence failure, and source rate
limit behavior.

## Garderobe Remote MCP Smoke

Status: package-local severe integration checks exist; a fresh Arcwell-owned
Cloudflare deployment and live Claude/host MCP handshake have not been recorded
from this package.

Private inventory sync is opt-in. Live tests must use disposable fixture rows or
an explicitly approved private wardrobe deployment. Do not copy `.dev.vars`,
seed SQL, live login codes, or real wardrobe CSV exports into this repository or
test artifacts.

Local package checks:

```sh
cd packages/arcwell-garderobe
npm install
npm run typecheck
npm test
```

The local severe integration test verifies:

- OAuth 2.1/DCR wiring is present and plain PKCE is disabled;
- `.dev.vars`, `.wrangler`, node modules, private seed SQL, and live-remote
  severe scripts are absent from the package;
- Arcwell memory/profile/wiki boundaries reject default private inventory sync;
- hostile wardrobe metadata and unsafe prompt instructions in wardrobe notes are
  documented as untrusted data;
- weather lookup fails closed to a manual temperature/conditions fallback.

Live remote MCP smoke, once a disposable or approved deployment exists:

```sh
cd packages/arcwell-garderobe
npm run db:migrate:remote
npm run deploy
```

Then use MCP Inspector or a host connector with OAuth/DCR to verify:

- auth bypass: unauthenticated `/mcp` and invalid/forged bearer calls fail
  before any tool result is returned;
- inventory leakage: exported logs, Arcwell memory/profile/wiki, and host
  transcript summaries do not contain private item rows unless explicitly
  requested for that test;
- hostile item names/prompt injection: item names and notes containing "ignore
  previous instructions", "reveal secrets", or "skip OAuth" appear only as data
  in the tool output and do not change host behavior;
- weather API failure: an unavailable weather source causes a manual fallback
  question, not fabricated weather;
- accidental sync of private inventory into Arcwell memory/wiki: after an
  outfit-planning session, `arcwell memory search wardrobe` and
  `arcwell wiki search wardrobe` show no private inventory rows unless the test
  explicitly opted into that sync.

Current blockers:

- `wrangler.jsonc` contains placeholder D1/KV ids for a new Arcwell deployment.
- The adjacent source project had no explicit top-level license file at
  integration time, so public redistribution remains blocked until provenance is
  settled.
- Live MCP/OAuth proof against Claude.ai or another remote host has not been
  run from `packages/arcwell-garderobe`.

## Telegram Live Smoke

Status: local authorization smoke, Telegram `getMe`, webhook installation,
outgoing provider send, and deployed edge webhook delivery were exercised on
2026-06-20. The strict `scripts/telegram-live-smoke` run did not pass because
the one observed incoming Telegram update did not match the exact expected
smoke phrase before timeout.

Use a disposable test bot/chat. Do not paste bot tokens into chat transcripts or
shell history. The smoke script uses a disposable `ARCWELL_HOME` by default and
refuses to claim live proof unless `ARCWELL_TELEGRAM_LIVE_CONFIRM=disposable`
is set.

```sh
TELEGRAM_BOT_TOKEN=... \
TELEGRAM_TEST_CHAT_ID=... \
TELEGRAM_WEBHOOK_SECRET=... \
ARCWELL_EDGE_SECRET=... \
ARCWELL_EDGE_URL="https://<worker-host>" \
ARCWELL_TELEGRAM_LIVE_CONFIRM=disposable \
scripts/telegram-live-smoke
```

Expected result:

- The local preflight proves an unauthorised chat and a read/write-only chat
  cannot trigger `telegram send`.
- `telegram send` records a channel message with status `sent`.
- `telegram deliveries` records one successful provider delivery attempt.
- A Telegram webhook message appears in the edge inbox, `edge drain-remote`
  imports it, and `arcwell telegram drain` records it once in
  `channel_messages`.
- Re-running remote/local drains does not create a second channel record for
  the same Telegram update.

Manual equivalent:

```sh
arcwell telegram authorize telegram:chat:<chat-id> \
  --read-projects \
  --write-projects \
  --send
arcwell telegram send <chat-id> "Arcwell smoke test"
arcwell telegram deliveries
arcwell edge drain-remote --url "$ARCWELL_EDGE_URL" --secret "$ARCWELL_EDGE_SECRET"
arcwell telegram drain
```

Current blocker:

- This audit has not been run with a disposable live Telegram bot/chat and a
  deployed Cloudflare edge URL. Without those credentials, the live webhook and
  outgoing provider proof must remain unclaimed.

## Email Channel Live Smoke

Status: local adversarial mapper fixtures, edge Email Routing enqueue tests, and
Rust poll/drain/send tests pass. Live Cloudflare Email Routing and
provider-side outbound delivery are not yet proven.

Local package smoke:

```sh
cd packages/arcwell-email
npm test
```

This proves the normalized inbound-email mapper rejects spoofed sender metadata,
malicious HTML/script/CSS, attachment bombs, duplicate `Message-ID` values,
oversized bodies, auto-responder loops, and unauthorized routes. It also proves
accepted messages are mapped as untrusted source-card/channel evidence.

Local Rust smoke:

```sh
cargo test -p arcwell-core email -- --nocapture
```

This proves email edge events drain into local channel/source-card records only
after persistence, malformed email events are nacked before ack, configured
author envelope senders can be treated as instructions, spoofed display `From:`
headers remain untrusted evidence, and outbound Cloudflare Email Service sends
require recipient authorization plus safe rich HTML.

Local one-shot polling after a route has enqueued events:

```sh
arcwell email poll
```

This uses `ARCWELL_EDGE_URL`/`ARCWELL_EDGE_SECRET` or stored Arcwell secrets,
leases remote edge events, persists them locally, then drains email events into
channel/source-card records.

Guarded Cloudflare setup:

```sh
ARCWELL_EMAIL_SETUP_CONFIRM=configure \
ARCWELL_EMAIL_ROUTE_RECIPIENT=agent@example.com \
ARCWELL_AUTHOR_EMAILS=user@example.com \
ARCWELL_AGENT_EMAIL_FROM=agent@example.com \
scripts/setup-email-route
```

Use ignored local env or secret-store values for real addresses. The script
sets `EMAIL_ROUTES_JSON` on the Worker, deploys the current edge inbox Worker,
and then attempts to create/update a Cloudflare Email Routing rule. The routing
rule step requires a Cloudflare API token with Email Routing permissions.
Tracked docs and examples must keep only `agent@example.com` and
`user@example.com`.

Live smoke:

```sh
ARCWELL_EDGE_URL="https://<worker-host>" \
ARCWELL_EDGE_SECRET="..." \
ARCWELL_EMAIL_LIVE_CONFIRM=disposable \
scripts/email-live-smoke
```

Expected result for that future smoke:

- A disposable test address receives one controlled message through Cloudflare
  Email Routing.
- The worker enqueues one `email` edge event with a `Message-ID` idempotency
  key and sanitized metadata.
- `arcwell email poll` persists the edge event locally.
- The email drain path records exactly one channel message and optional source
  card for an authorized sender/route.
- Re-sending or replaying the same `Message-ID` does not create duplicates.
- Spoofed `From:`, failed DMARC, unauthorized sender, oversized body,
  attachment bomb, tracking-link, and auto-responder fixtures fail closed.

Current blockers:

- No live test address/route is configured. On 2026-06-21, guarded no-deploy
  setup uploaded `EMAIL_ROUTES_JSON` to the `arcwell-edge-inbox` Worker, but
  Cloudflare Email Routing rule creation failed with API error
  `10000 Authentication error`, so the current token lacks the needed Email
  Routing rule permission.
- No provider-side live outbound send/reply smoke has been recorded.
- No librarian digest scheduler has been wired to email delivery yet.

## Codex Memory Hook Smoke

Status: local/degraded command smoke exists; fresh-thread Codex host execution
is not yet recorded as passed in this audit.

Local command smoke:

```sh
scripts/arcwell-dev smoke
arcwell memory eval-corpus
arcwell memory decisions --limit 20
arcwell memory tombstones --limit 20
```

This proves the generated dev plugin hook commands, recall/capture command
paths, deterministic personal-memory eval corpus, decision ledger readout, and
tombstone readout against a disposable local `ARCWELL_HOME`. It does not prove
that a currently running Codex app thread accepted and executed the hook config.

```sh
cargo install --path crates/arcwell-cli
codex plugin marketplace add /Users/chabotc/Projects/arcwell
codex plugin add arcwell-codex@arcwell-local
arcwell memory events --limit 20
```

Manual smoke:

1. Start a fresh Codex thread after installing/updating the plugin.
2. Ask a personalized prompt that should trigger recall.
3. End or compact a thread after mentioning a stable personal fact.
4. Run `/memory-events` or `arcwell memory events --limit 20`.

Expected result:

- A `recall` lifecycle event appears for startup or prompt submission.
- A `capture` lifecycle event appears for stop or compaction.
- Sensitive facts are candidate-only unless automatic capture policy is
  explicitly configured.
- Same-subject UPDATE/DELETE/conflict candidates remain pending for review.

## Claude Memory Degraded Smoke

Status: documented as degraded/manual; not recorded as passed with Claude
Desktop or Claude Code in this audit.

Manual smoke:

1. Configure Claude to use `arcwell mcp` from `hosts/claude/README.md`.
2. Call memory search/recall tools manually from Claude.
3. Capture a disposable stable fact in review mode.
4. Run `arcwell memory events --limit 20` and
   `arcwell memory decisions --limit 20`.

Expected result:

- Claude can use MCP/CLI memory read and review-capture paths manually.
- No lifecycle hook parity is claimed; Claude hook/thread inventory remains
  unavailable unless a future host integration proves it.
