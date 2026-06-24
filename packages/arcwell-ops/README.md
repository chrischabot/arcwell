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
  summary health scoring, queue/source/radar-run/radar-quality/credential
  summaries, and one narrow authenticated edge-event dead-letter control with
  token auth, hostile-origin rejection, CSRF/idempotency checks, policy
  enforcement, and replay tests.
- `scripts/ops-ui-browser-smoke` runs browser-backed desktop, detail, and
  mobile validation against a seeded authenticated local `/ops/ui`, preserving
  screenshots and a proof packet under `.arcwell-dev/proofs/`.
- `scripts/ops-ui-x-browser-smoke` runs browser-backed desktop, filtered, and
  mobile validation for hostile X tweet/link/provider-error rows, including
  token-like provider-error redaction, local dummy-secret non-rendering, row-focused screenshots, and no body overflow.
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
- More browser fixtures for future mutating controls.
- Error charts, watchdog summaries, and recent failures.
