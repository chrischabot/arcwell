# arcwell-edge-inbox

**Status:** Partial/Risk. Local worker code, severe tests, deploy, remote D1 reachability, and authenticated deployed ingress/drain smoke pass against `https://arcwell-edge-inbox.chabotc.workers.dev`. The live smoke proves current/next secret rotation, D1 persistence, duplicate idempotency, lease expiry/retry, ack/nack behavior, local Rust drain sync, local-persistence-failure nack without ack, and source rate limiting.

Cloudflare edge inbox package for short-lived always-on event capture.

Purpose:

- Accept webhook/OAuth/email/channel events while the local machine is offline.
- Enforce source, idempotency key, payload size, and TTL at the edge.
- Let the local Rust service drain events into SQLite through the MCP/local API contract.

Local durable owner:

- `edge_event_enqueue`
- `edge_event_lease`
- `edge_event_ack`
- `edge_event_nack`
- `edge_event_dead_letter`
- `edge_event_list`
- `arcwell://edge-events`

Cloudflare worker:

- `POST /events` accepts JSON with `source`, `idempotencyKey`, `payload`, and optional `maxAgeSeconds`.
- `GET /health` returns a small status object.
- `POST /drain/lease` leases the next pending/failed/expired-lease event.
- `POST /drain/ack` marks a drained event as acknowledged.
- `POST /drain/nack` records a drain failure and retries or dead-letters after the attempt budget.
- `GET /events` lists bounded event state for debugging.
- Cloudflare Email Routing `email()` events can enqueue bounded `email` source events when `EMAIL_ROUTES_JSON` and sender allow rules are configured.
- Events are stored in D1 through the `EDGE_DB` binding. The worker remains a bounded ingress/drain buffer, not the local durable brain.
- Local tests use an in-memory store that exercises the same lease/ack/nack, auth-rotation, and rate-limit semantics.

Security rules:

- Require `x-arcwell-edge-secret`; `ARCWELL_EDGE_NEXT_SECRET` can be configured during rotation so old and new secrets are both accepted briefly.
- Reject payloads over 64 KB.
- Clamp TTL to 60 seconds through 24 hours.
- Preserve idempotency key as the replay boundary.
- Enforce a per-source fixed-window rate limit with `RATE_LIMIT_WINDOW_SECONDS` and `RATE_LIMIT_MAX_EVENTS`.
- Do not execute or interpret event payload text at the edge.
- Treat email envelope sender and configured route policy as authority; preserve display `From:` and MIME body as untrusted evidence only.
- Reject oversized email raw MIME, unauthorized recipients/senders, missing `Message-ID`, and failing DMARC by default.

Live smoke:

- Run `scripts/edge-live-smoke` from the repo root against a staging worker with
  `EDGE_URL`, `ARCWELL_EDGE_SECRET`, and `ARCWELL_EDGE_NEXT_SECRET`.
- The script uses a temporary `ARCWELL_HOME` and synthetic events to prove
  forged-secret rejection, current/next rotation, offline persistence,
  duplicate idempotency, lease expiry/retry, nack-without-ack on local
  persistence failure, and bounded replay/rate-limit behavior.
- For a compact rate-limit proof, set staging `RATE_LIMIT_MAX_EVENTS` low enough
  for `EDGE_SMOKE_RATE_LIMIT_PROBES` to exceed it.
- After edge staging is available, run `scripts/telegram-live-smoke` with a
  disposable Telegram bot/chat to prove Telegram webhook ingress, remote drain,
  local Telegram drain, and outgoing delivery recording without production data.

Remaining work:

- Add Miniflare or another deployed-runtime emulator only if future D1 behavior
  diverges from local tests again.
- Replace the simple fixed-window limiter if production traffic needs a stricter global abuse model.
