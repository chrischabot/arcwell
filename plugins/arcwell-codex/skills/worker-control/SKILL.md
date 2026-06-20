---
name: worker-control
description: Use when draining local jobs, investigating background worker behavior, retries, leases, or dead-lettered jobs.
---

# Worker Control

Rules:

- Use `worker_run_once` for interactive bounded drains.
- Use the OS service for long-running background drains.
- Do not start an endless `arcwell worker run` loop inside a normal Codex task unless the user explicitly wants a resident process.
- After a drain, report processed, completed, failed, and dead-lettered counts.
- Inspect failed jobs before retrying blindly.

Useful tools:

- `worker_run_once`
- `wiki_jobs`
- `wiki_job_status`
- `ops_snapshot`
