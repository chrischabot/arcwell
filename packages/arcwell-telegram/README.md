# arcwell-telegram

Telegram channel package.

Current first-pass implementation:

- Cloudflare webhook ingress should post Telegram updates into `arcwell-edge-inbox`.
- Local processing stores messages with `channel_record`.
- Project-aware routing uses `project_resolve`.
- Agent-facing operations happen through MCP, not direct Telegram-specific local tools.

Channel safety rules:

- Telegram text is untrusted user/content data.
- Formatting must be normalized before delivery.
- Incoming update ids are idempotency keys.
- Project switching must resolve through the project registry, and ambiguity must stop the action.

Relevant MCP tools:

- `edge_event_enqueue`
- `edge_event_lease`
- `edge_event_ack`
- `edge_event_nack`
- `channel_record`
- `channel_list`
- `project_resolve`
- `ops_snapshot`
