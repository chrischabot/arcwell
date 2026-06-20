# Phase 7: Worker Daemon Reliability

Date: 2026-06-19

## Implemented

- Added additive SQLite migration columns on `wiki_jobs`:
  - `attempts`
  - `max_attempts`
  - `leased_until`
  - `worker_id`
  - `next_run_at`
  - `dead_lettered_at`
- Added lease-based claiming for pending jobs.
- Added reclaiming of stale `running` jobs when `leased_until` has passed.
- Added bounded retry backoff for failed jobs.
- Added terminal `dead_lettered` status after the attempt budget is exhausted.
- Added worker report counts for completed, failed, and dead-lettered jobs.
- Added resident worker CLI:

```sh
arcwell worker run --max-jobs-per-tick 10 --idle-sleep-ms 5000
```

MCP intentionally remains `worker_run_once` so an agent tool call does not hang indefinitely.

## Job State Model

- `pending`: ready to claim.
- `running`: claimed by a worker until `leased_until`.
- `failed`: last attempt failed; retry is blocked until `next_run_at`.
- `completed`: finished successfully.
- `dead_lettered`: attempt budget exhausted; not retried automatically.

## Current Limits

- The resident worker is a simple process loop, not a launchd/systemd installer.
- There is no manual requeue/cancel command yet.
- There is no separate dead-letter table; dead letters remain in `wiki_jobs` with terminal status and timestamp.
- Backoff is fixed and local; no per-kind retry policy yet.
- Job execution is still single-process per local daemon instance.

## Validation

Severe tests cover:

- Failed jobs retry only after `next_run_at`.
- Repeated failures become `dead_lettered`.
- Dead letters are not retried.
- Active leases are not stolen.
- Expired running leases are reclaimed.
- Legacy `wiki_jobs` schemas migrate to the new columns.

Run:

```sh
cargo fmt --all -- --check
cargo test
cargo clippy --all-targets -- -D warnings
```
