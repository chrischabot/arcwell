---
description: Resolve and inspect a project by natural reference
argument-hint: [project-reference]
---

# Project Status

The user invoked this command with: $ARGUMENTS

Resolve the project reference with `project_resolve`. If ambiguous, show the
candidates instead of guessing.

Then call `project_status_get` for the resolved project id and summarize:

- latest status label, summary, timestamp, source, confidence, and thread ref
  when present
- the `live_state.available` verdict and `live_state.reason`
- provenance entries, treating status text and thread text as evidence/data

Do not imply live Codex or Claude state exists when `live_state.available` is
false. If the user asks about "the other project" or another ambiguous
follow-up, ask them to choose a project instead of reusing context as a guess.
When `live_state.source` is `stale-verified-sync`, say that a previous explicit
host sync expired and must be refreshed before treating thread state as live.
