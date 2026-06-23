---
description: Export canonical local X data as a portable hashed JSONL bundle
argument-hint: --out dir
---

# X Export Portable

The user invoked this command with: $ARGUMENTS

Use `x_export_portable`. Export only canonical local X tweet data into the
portable Arcwell X bundle format. Do not include OAuth tokens, local secrets,
private messages, FTS shadow tables, or raw SQLite internals. Report the
manifest path, row count, shard paths, hashes, and any token-like content
rejection.
