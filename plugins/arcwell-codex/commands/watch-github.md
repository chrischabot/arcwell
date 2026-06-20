---
description: Queue a GitHub owner or repository watch fetch
argument-hint: OWNER[/REPO] [mode=releases|commits]
---

# Watch GitHub

The user invoked this command with: $ARGUMENTS

Use `wiki_enqueue_github_owner` when only an owner is supplied. Use `wiki_enqueue_github` when `OWNER/REPO` is supplied. Default repo mode is `releases` unless the user asks for commits. Run `worker_run_once` only when immediate processing is requested.
