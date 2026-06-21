---
description: Record a timestamped Arcwell project status snapshot
argument-hint: PROJECT_ID STATUS SUMMARY [SOURCE] [THREAD_REF]
---

# Project Status Record

The user invoked this command with: $ARGUMENTS

Use `project_status_record`. Preserve the source and thread reference when known.
Do not imply this is live Codex or Claude state. Reserved host-live source names
are rejected by the core; use `project_status_sync_record` only after a host
inventory/read tool has verified the exact thread and freshness window.
`project_status_get` is the source of truth for whether Arcwell can currently
treat a snapshot as fresh verified sync or stale durable evidence.
