---
description: Expand indexed X links through the safe URL-ingest path
argument-hint: "[--limit N]"
---

# X Expand Links

The user invoked this command with: $ARGUMENTS

Use `x_expand_links`. This is an explicit network action: it fetches already-indexed X link URLs through Arcwell's URL-ingest safety path with policy/cost gates, redirect validation, content-type and size limits, and untrusted-source rendering. Do not use it when the user only asked to list or extract local links; use `x_extract_links` and `x_links` for local-only work.
