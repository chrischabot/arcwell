---
name: ops-control
description: Use when checking arcwell health, queues, jobs, cursors, edge events, source health, or recent operational errors.
---

# Ops Control

Rules:

- Start with `ops_snapshot` or `arcwell://ops`.
- Check `arcwell://health` when health is the focus.
- Inspect `wiki_jobs` for pending, failed, or dead-lettered work.
- Inspect cursors when adapter progress or duplicate fetches are in question.
- Report concrete counts and latest errors. Avoid vague "looks fine" summaries.
- If the worker is not running, say that separately from queue health.

Useful tools:

- `ops_snapshot`
- `wiki_jobs`
- `wiki_job_status`
- `cursor_list`
- `cursor_get`
- `worker_run_once`
