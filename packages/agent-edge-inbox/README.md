# agent-edge-inbox

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
- `agent://edge-events`

Cloudflare first-pass worker:

- `POST /events` accepts JSON with `source`, `idempotencyKey`, `payload`, and optional `maxAgeSeconds`.
- `GET /health` returns a small status object.
- The worker is intentionally a bounded ingress shim. It is not the durable brain.

Security rules:

- Require `x-agent-edge-secret`.
- Reject payloads over 64 KB.
- Clamp TTL to 60 seconds through 24 hours.
- Preserve idempotency key as the replay boundary.
- Do not execute or interpret event payload text at the edge.
