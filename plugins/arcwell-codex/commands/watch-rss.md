---
description: Queue an RSS or Atom feed watch fetch
argument-hint: [feed-url]
---

# Watch RSS

The user invoked this command with: $ARGUMENTS

Use `wiki_enqueue_rss`. Queue an RSS/Atom fetch job for the supplied feed URL. If the user asks for immediate ingestion, run `worker_run_once` after enqueueing.
