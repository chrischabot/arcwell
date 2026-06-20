# agent-ops

Local operations surface.

Current first-pass implementation:

- HTTP `GET /ops` returns an ops snapshot.
- MCP `ops_snapshot` returns the same durable state through the agent control plane.
- Snapshot includes health, wiki jobs, edge events, cursors, projects, and digest candidates.

MCP resources:

- `agent://ops`
- `agent://edge-events`
- `agent://projects`
- `agent://channels`
- `agent://digest-candidates`

Remaining work:

- Browser UI for filtering, retrying, dead-letter inspection, source health, and memory candidates.
- Manual requeue/cancel controls with confirmation policy.
- Error charts and recent failures.
