# Adversarial Review Phase 7

Date: 2026-06-19

Scope: local worker daemon reliability, job leases, retries, backoff, dead-lettering, and schema migration.

## Findings

No blocking issues remain in the implemented worker reliability surface after the current test pass.

## Issues Found And Fixed

- `run_worker_once` previously claimed only `pending` jobs and had no lease. Fixed by recording worker lease state and reclaiming expired `running` jobs.
- Failed jobs previously stayed `failed` forever with no retry/dead-letter semantics. Fixed by adding attempts, bounded backoff through `next_run_at`, and terminal `dead_lettered` status.
- The old schema had no room for worker diagnostics. Fixed with additive migration columns and a regression test that opens an old-style database.
- A resident worker loop did not exist. Fixed with `agent worker run`, while keeping MCP non-blocking through `worker_run_once`.

## Attack Cases Covered By Tests

- A missing ingest file fails the job and records the root error.
- A failed job cannot be retried immediately before backoff expires.
- Repeated failures stop at `dead_lettered`.
- Dead-lettered jobs are not automatically reprocessed.
- A second worker cannot steal an active lease.
- An expired lease can be reclaimed and completed.
- Existing databases with the old `wiki_jobs` shape are migrated without losing jobs.
- Unknown job kinds such as `shell_exec` cannot enter the queue.

## Residual Risks

- There is not yet a manual `requeue` or `cancel` operation.
- Backoff is fixed rather than policy-driven by job kind or error class.
- There is no external supervisor installer for keeping `agent worker run` alive after reboot.
- There is no cross-process stress test with two OS processes racing on the same SQLite database yet; current severe lease tests exercise the state transition invariants inside one process.
- Long-running adapter calls still depend on provider timeouts and rate limits.

## Validation

```sh
cargo fmt --all -- --check
cargo test
cargo clippy --all-targets -- -D warnings
```

Current expected result after this phase: 49 tests passing.
