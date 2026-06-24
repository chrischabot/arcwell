---
description: Enqueue a radar profile run for the local worker
argument-hint: <profile-id-or-name> [--window-hours N] [--fetch-live]
---

# Radar Enqueue

The user invoked this command with: $ARGUMENTS

Use `radar_enqueue` to create a local worker job for a radar profile. Treat the
returned job as queued work, not a completed radar run.

Only pass `fetch_live: true` when the user explicitly requests live/current
source fetching. With `fetch_live`, the worker will invoke existing Arcwell
RSS/GitHub/arXiv/Hacker News/Reddit/X adapters before projection, subject to
policy and cost gates.

After enqueueing, inspect the job id/status. If the user asked you to complete
the run now, drain the local worker with a bounded job count, then inspect
`radar_runs`, `radar_stage_read`, and `radar_audit_run` before calling the run
healthy. Do not imply delivery, model enrichment, or Reddit production-data proof
unless a later proof packet shows those stages passed.
