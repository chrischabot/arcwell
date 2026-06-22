# arcwell-ops

**Status:** Partial.

Repository tracking: [STATUS.md](../../STATUS.md) and [TODO.md](../../TODO.md).

Local operations surface.

Current first-pass implementation:

- HTTP `GET /ops` returns an ops snapshot.
- HTTP `GET /ops/ui` renders a localhost browser dashboard over the same
  snapshot.
- MCP `ops_snapshot` returns the same durable state through the agent control plane.
- Snapshot includes health, backups, worker heartbeat, wiki jobs, dead letters,
  edge events, cursors, source health, watch sources, projects, project status
  snapshots, channels, Telegram delivery failures, source cards, digest
  candidates, work runs, procedure candidates, memory candidates, policy
  decisions/approvals, costs, and secret health.
- `/ops/ui` includes search/status filters, stable sorting, detail views,
  summary health scoring, queue/source/credential summaries, and one narrow
  authenticated edge-event dead-letter control with token auth, hostile-origin
  rejection, CSRF/idempotency checks, policy enforcement, and replay tests.
- Broader mutating controls remain deferred until each action has explicit
  core support, auth, policy, CSRF/origin, idempotency/replay handling, and
  severe tests.

MCP resources:

- `arcwell://ops`
- `arcwell://edge-events`
- `arcwell://projects`
- `arcwell://channels`
- `arcwell://digest-candidates`

Remaining work:

- Manual requeue/cancel controls with confirmation policy.
- Browser validation for the richer filter/detail/control UI.
- Error charts and recent failures.
