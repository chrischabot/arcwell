# Codex Plugin Commands And Skills

`arcwell` exposes Codex integration through one plugin:

```text
plugins/arcwell-codex/
```

The plugin registers:

- MCP tools through `arcwell mcp`
- `$commands` as Codex skills under `skills/`
- slash-command prompts under `commands/`

Restart Codex or start a new thread after installing or updating the plugin so the command catalog and skills reload.

## Install

```sh
cargo install --path crates/arcwell-cli
codex plugin marketplace add /Users/chabotc/Projects/arcwell
codex plugin add arcwell-codex@arcwell-local
```

The plugin assumes `arcwell` is on `PATH`. The MCP server uses the default local home, `~/.arcwell`, unless `ARCWELL_HOME` is set in the host environment.

## Slash Commands

Codex plugin slash commands are prompt files. Depending on the host surface and namespace collision handling, they may appear as `/arcwell-codex:remember`, `/remember`, or through the slash menu after typing part of the name. Use the displayed command name from the picker.

### Health, Ops, And Services

- `/arcwell-health` uses `arcwell_health`.
- `/ops` uses `ops_snapshot`.
- `/worker-run-once` uses `worker_run_once`.
- `/backup-create` uses `backup_create`.
- `/backup-status` uses `arcwell backup status`.
- `/backup-verify` uses `backup_verify`.
- `/cost-add` uses `arcwell cost add`.
- `/cost-summary` uses `cost_summary`.

### Personal Memory And Profile

- `/remember` uses `memory_add` or `memory_extract_candidates`.
- `/memory-search` uses `memory_search`.
- `/memory-list` uses `arcwell memory list`.
- `/memory-extract` uses `memory_extract_candidates`.
- `/memory-candidates` uses `candidate_list` or `candidate_apply`.
- `/memory-reject` uses `arcwell candidate reject`.
- `/memory-dream` uses `memory_dream_reconcile`.
- `/memory-delete` uses `arcwell memory delete`.
- `/profile-list` uses `profile_list`.
- `/profile-get` uses `arcwell profile get`.
- `/profile-search` uses `profile_search`.
- `/profile-set` uses `profile_set`.
- `/profile-delete` uses `arcwell profile delete`.

### Wiki, Source Cards, And Librarian

- `/wiki-search` uses `wiki_search` and optionally `wiki_read`.
- `/wiki-list` uses `arcwell wiki list`.
- `/wiki-add` uses `arcwell wiki add`.
- `/wiki-read` uses `wiki_read`.
- `/wiki-ingest` uses `wiki_ingest_file`, `wiki_ingest_dir`, `wiki_ingest_job`, or `wiki_ingest_url`.
- `/wiki-import-codex-swift-sources` uses `wiki_import_codex_swift_sources`.
- `/wiki-sources` uses `wiki_watch_sources`.
- `/wiki-jobs` uses `wiki_jobs`.
- `/wiki-job` uses `wiki_job_status`.
- `/wiki-compile` uses `wiki_compile`.
- `/wiki-expand` uses `wiki_expand_page`.
- `/wiki-run-rss` uses `arcwell wiki run-rss`.
- `/wiki-run-github` uses `arcwell wiki run-github-owner` or `arcwell wiki run-github`.
- `/wiki-run-arxiv` uses `arcwell wiki run-arxiv`.
- `/librarian-expand` uses `librarian_expand_topic` or `wiki_expand_page`.
- `/source-card-add` uses `source_card_add`.
- `/source-card-search` uses `source_card_search`.
- `/source-card-read` uses `source_card_read`.
- `/digest-candidate-create` uses `digest_candidate_create`.
- `/digest-candidates` uses `digest_candidate_list`.

### Research

- `/research-plan` uses `research_plan`.
- `/research-search` uses `research_web_search`.
- `/research-workflow` uses `research_workflow_create` and `research_tasks`.
- `/research-runs` uses `research_runs`.
- `/research-tasks` uses `research_tasks`.
- `/research-task-complete` uses `research_task_complete`.
- `/research-brief` uses `research_brief_from_wiki`.
- `/import-claude` uses `arcwell import claude`.

### Watch Sources And Adapters

- `/watch-rss` uses `wiki_enqueue_rss`.
- `/watch-github` uses `wiki_enqueue_github_owner` or `wiki_enqueue_github`.
- `/watch-arxiv` uses `wiki_enqueue_arxiv`.
- `/cursor-list` uses `cursor_list`.
- `/cursor-get` uses `cursor_get`.

### X / Twitter

- `/x-search` uses `x_recent_search`.
- `/x-enqueue-search` uses `x_enqueue_recent_search`.
- `/x-watch-rebuild` uses `x_rebuild_definitive_watch_sources`.
- `/x-import-following-watch-sources` uses `x_import_following_watch_sources`.
- `/x-import-json` uses `x_import_json_file`.
- `/x-list` uses `x_list`.
- `/x-report` uses `x_report`.
- `/x-oauth` uses `x_oauth_authorize_url`, `x_oauth_exchange_code`, or `x_oauth_refresh`.

### Projects And Channels

- `/project-create` uses `project_create`.
- `/project-list` uses `project_list`.
- `/project-status` uses `project_resolve` and `project_list`.
- `/channel-list` uses `channel_list`.
- `/channel-record` uses `channel_record`.
- `/telegram-inbox` uses `channel_list` with Telegram-focused handling.

### Edge Inbox

- `/edge-events` uses `edge_event_list`.
- `/edge-enqueue` uses `edge_event_enqueue`.
- `/edge-lease` uses `edge_event_lease`.
- `/edge-ack` uses `edge_event_ack`.
- `/edge-nack` uses `edge_event_nack`.
- `/edge-dead-letter` uses `edge_event_dead_letter`.

### Secrets

- `/secret-list` uses `secret_value_list`.
- `/secret-set` uses `secret_value_set`.
- `/secret-delete` uses `secret_value_delete`.
- `/secret-ref-list` uses `arcwell secrets list`.
- `/secret-ref-set` uses `arcwell secrets set-ref`.

Secret commands must never print secret values back into the chat transcript. The `secret-ref-*` commands store references to external secret locations; the `secret-*` commands manage local SQLite-backed secret values.

## `$commands` / Skills

Skills are the primary reusable behavior surface. In Codex they appear as `$...` capabilities, usually plugin-prefixed if there are name collisions. Use the skill picker or type the displayed name, such as `$arcwell-codex:wiki-research`.

- `$arcwell-codex:memory-review`: consult personal memory, extract reviewable candidates, and keep memory separate from wiki knowledge.
- `$arcwell-codex:wiki-research`: search and write source-backed wiki pages.
- `$arcwell-codex:deep-research`: plan, gather, audit, and brief substantial research.
- `$arcwell-codex:research-audit`: adversarially check sources, claims, provenance, and uncertainty.
- `$arcwell-codex:research-brief`: produce concise wiki-grounded research briefs.
- `$arcwell-codex:x-research`: import, search, report, and evaluate X evidence safely.
- `$arcwell-codex:project-control`: resolve and manage project context across threads and channels.
- `$arcwell-codex:channel-control`: handle Telegram and future channel messages without prompt-injection leakage.
- `$arcwell-codex:ops-control`: inspect jobs, queues, cursors, edge events, and service health.
- `$arcwell-codex:worker-control`: drain workers safely and interpret job failures.
- `$arcwell-codex:competence-respect`: use enough reasoning, consult memory/tools, and avoid wasting the user’s time.

## Design Notes

Slash commands are intentionally thin. They route a human request to the right MCP tool and capture the handling discipline for ambiguity, secrets, prompt injection, and source trust. Durable behavior belongs in skills and MCP tools; unattended work belongs in the worker service.

The slash prompts do not use restrictive `allowed-tools` front matter. Current Codex plugin examples support that metadata for built-in tools, but MCP tool allow-list naming is host-sensitive. Leaving prompts unrestricted avoids registration failures while the actual operations remain governed by MCP schemas, service code, and host sandbox settings.
