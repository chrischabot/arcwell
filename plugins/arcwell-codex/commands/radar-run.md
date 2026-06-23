---
description: Run a radar profile through source-card projection, optionally after live adapter fetch
argument-hint: <profile-id-or-name> [--window-hours N] [--fetch-live]
---

# Radar Run

The user invoked this command with: $ARGUMENTS

Use `radar_run`. By default, treat this as the locally proven source-card-backed
radar pipeline: normalized items, FTS, and heuristic scores.

Only pass `fetch_live: true` when the user explicitly requests live/current
source fetching. With `fetch_live`, the tool invokes existing Arcwell
RSS/GitHub/arXiv/X adapters before projection, then inspect the returned
adapter jobs, source-health state, run status, and audit findings before
calling the result healthy. Do not imply HN/Reddit fetch, enrichment, model
summary, scheduled operation, or delivery unless a later proof packet shows
those stages passed.
