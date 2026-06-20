# Functionality And Packages

This document is the detailed map of what `arcwell` provides, what each package owns, and how a host agent should use it.

## Architectural Approach

Most assistant projects wrap a coding agent inside a larger agent framework. `arcwell` takes the opposite route:

- Codex stays the main workbench.
- Services expose capabilities to Codex through MCP, CLI, skills, and optional HTTP.
- Claude and other MCP-capable agents can use the same services.
- Durable state stays local by default.
- Cloudflare is used only where always-on internet-facing collection helps.

The result is "add assistant infrastructure to Codex" rather than "put Codex inside a new assistant."

## Surfaces

### Rust CLI

The binary is `arcwell`.

It supports:

- `arcwell doctor`
- `arcwell profile ...`
- `arcwell memory ...`
- `arcwell wiki ...`
- `arcwell source-card ...`
- `arcwell research ...`
- `arcwell x ...`
- `arcwell worker ...`
- `arcwell secrets ...`
- `agent cursors ...`
- `arcwell serve`
- `arcwell mcp`

### MCP Server

`arcwell mcp` is the primary agent control plane.

Core resources:

- `arcwell://health`
- `arcwell://profile`
- `arcwell://memory`
- `arcwell://wiki`
- `arcwell://source-cards`
- `arcwell://wiki-jobs`
- `arcwell://cursors`
- `arcwell://secret-values`
- `arcwell://x-items`
- `arcwell://research`
- `arcwell://edge-events`
- `arcwell://channels`
- `arcwell://projects`
- `arcwell://digest-candidates`
- `arcwell://ops`

### HTTP Server

`arcwell serve --addr 127.0.0.1:8787`

Current endpoints:

- `GET /health`
- `GET /profile`
- `GET /memory`
- `GET /wiki`
- `GET /ops`

### Cloudflare Workers

Cloudflare is used for always-on capture:

- webhooks
- OAuth callback capture
- channel events
- short-lived queues

The local service remains the durable source of truth.

## Package: `arcwell-profile`

Intent: store explicit user profile and preference records.

Examples:

- communication preferences
- writing style pointers
- accessibility/support needs
- preferred defaults

Main tools:

- `profile_set`
- `profile_search`
- `profile_list`

CLI:

```sh
arcwell profile set communication.style "Direct, sourced, warm."
arcwell profile search communication
arcwell profile list
```

Data store:

- SQLite table `profile_items`

## Package: `arcwell-memory`

Intent: personal memory for facts/preferences about the person, separate from wiki knowledge.

Examples:

- "My cat is called Ophelia."
- "I have ADHD."
- "I prefer short progress updates."

Main tools:

- `memory_add`
- `memory_search`
- `memory_extract_candidates`
- `memory_dream_reconcile`
- `candidate_list`
- `candidate_apply`

CLI:

```sh
arcwell memory add "My cat is called Ophelia"
arcwell memory search Ophelia
arcwell candidate list
arcwell candidate apply <id>
```

Data store:

- SQLite `memories`
- SQLite `candidates`

Safety:

- Extracted memories become candidates first.
- Applying candidates is explicit.
- Duplicate suppression prevents easy write amplification.

## Package: `arcwell-llm-wiki`

Intent: a source-backed Markdown knowledge base.

This is for external knowledge, not personal memory.

Examples:

- AI agent papers
- developer relations notes
- launch analysis
- GitHub repo changes
- blog post source cards
- X posts as evidence

Main tools:

- `wiki_search`
- `wiki_read`
- `wiki_ingest_file`
- `wiki_ingest_url`
- `wiki_import_codex_swift_sources`
- `wiki_watch_sources`
- `wiki_enqueue_rss`
- `wiki_enqueue_github`
- `wiki_enqueue_arxiv`
- `wiki_expand_page`
- `source_card_add`
- `source_card_search`
- `source_card_read`

CLI:

```sh
arcwell wiki ingest-file ./notes.md
arcwell wiki ingest-dir ./corpus
arcwell wiki import-codex-swift-sources /path/to/codex-swift
arcwell wiki sources
arcwell wiki search "MCP"
arcwell wiki enqueue-github-owner openai --limit 10
arcwell source-card add --title "Launch" --url "https://example.com" --summary "Summary"
arcwell wiki expand "Vercel Eve"
```

Data store:

- Markdown files under `ARCWELL_HOME/wiki/pages`
- SQLite `wiki_pages`
- SQLite `wiki_pages_fts`
- SQLite `source_cards`
- SQLite `watch_sources`
- SQLite `wiki_jobs`

Safety:

- Source cards mark external material as untrusted evidence.
- Watch sources are monitor configuration, not retrieved evidence; Codex Swift seed imports merge duplicates idempotently and reject unsafe URLs/invalid handles.
- URL ingest blocks local/private/metadata hosts.

## Package: `arcwell-deep-research`

Intent: coordinate multi-source research.

Main tools:

- `research_plan`
- `research_web_search`
- `research_workflow_create`
- `research_tasks`
- `research_task_complete`
- `research_brief_from_wiki`
- `research_runs`

CLI:

```sh
arcwell research plan "Vercel Eve"
arcwell research workflow "Vercel Eve"
arcwell research search "Vercel Eve" --provider brave --write-wiki
arcwell research brief "Vercel Eve"
```

Providers:

- host-native search by the calling agent
- Brave
- OpenAI web search
- Perplexity

Workflow roles:

- research scout
- source extractor
- skeptic
- synthesizer

## Package: `arcwell-x`

Intent: import, search, and report on X/Twitter material as source evidence.

Main tools:

- `x_import_json_file`
- `x_rebuild_definitive_watch_sources`
- `x_import_following_watch_sources`
- `x_oauth_authorize_url`
- `x_oauth_exchange_code`
- `x_oauth_refresh`
- `x_recent_search`
- `x_enqueue_recent_search`
- `x_list`
- `x_report`

CLI:

```sh
arcwell x import-json ./x-items.json
arcwell x rebuild-definitive-watch-sources --bookmark-days 92 --max-bookmarks 1000 --max-recent-follows 100
arcwell x recent-search "from:openai" --max-results 10
arcwell x list --query agents
arcwell x report --query agents
```

Data store:

- SQLite `x_items`
- source cards and wiki pages for imported items
- SQLite `watch_sources` for followed-account monitor handles
- cursor keys such as `x:recent-search:<query>`

Safety:

- X text is untrusted source text.
- Definitive watch rebuild replaces the previous `x_handle` registry instead of appending to it.
- Following imports validate handles and preserve profile descriptions as metadata only.
- Duplicate tweet ids are skipped.
- Unsafe URLs are rejected.

## Package: `arcwell-edge-inbox`

Intent: always-on short-lived event capture.

Main tools:

- `edge_event_enqueue`
- `edge_event_lease`
- `edge_event_ack`
- `edge_event_nack`
- `edge_event_dead_letter`
- `edge_event_list`

Cloudflare Worker:

- `packages/arcwell-edge-inbox/worker`

Worker endpoints:

- `GET /health`
- `POST /events`

Data store:

- SQLite `edge_events` locally
- future Cloudflare Queue/Durable Object at the edge

Semantics:

- idempotency key blocks replay replacement
- payload cap
- TTL/expiry
- leases
- retry/backoff
- dead-lettering

## Package: `arcwell-telegram`

Intent: Telegram channel integration.

Current package shape:

- Telegram webhook updates should land in edge inbox.
- Local service records messages through `channel_record`.
- Project references resolve through `project_resolve`.

Main tools:

- `edge_event_enqueue`
- `edge_event_lease`
- `channel_record`
- `channel_list`
- `project_resolve`

Safety:

- Telegram text is user/content data.
- Formatting should be normalized before outbound sends.
- Ambiguous project references stop instead of guessing.

## Package: `arcwell-projects`

Intent: project and thread meta-controller.

Main tools:

- `project_create`
- `project_list`
- `project_resolve`
- `channel_record`
- `channel_list`

Data store:

- SQLite `projects`
- SQLite `channel_messages`

Abilities:

- aliases
- summaries
- active status
- follow-up resolution via explicit context
- ambiguity detection

Future depth:

- Codex thread inventory
- Claude thread/context inventory
- status summaries from live work

## Package: `arcwell-librarian`

Intent: turn incoming source cards into useful knowledge and digest candidates.

Main tools:

- `digest_candidate_create`
- `digest_candidate_list`
- `librarian_expand_topic`
- `wiki_expand_page`

Data store:

- SQLite `digest_candidates`
- generated wiki pages

Current scoring:

- launch/release signals
- watched org/person signals
- agent/MCP topic signals
- source count

Future depth:

- clustering
- contradiction detection
- model-backed synthesis
- digest delivery

## Package: `arcwell-ops`

Intent: inspect and operate the local assistant system.

Main tools/resources:

- `ops_snapshot`
- `arcwell://ops`
- `GET /ops`

Snapshot includes:

- health
- wiki jobs
- edge events
- cursors
- projects
- digest candidates

Future depth:

- browser UI
- retry/cancel/requeue controls
- recent errors
- source health
- memory candidate review

## Package: `arcwell-conversation-import`

Intent: import conversation exports and propose profile/memory candidates.

Current CLI:

```sh
arcwell import claude ./conversations.json --dry-run
arcwell import claude ./conversations.json --write-candidates
```

This is used to migrate personal context from previous assistant conversations.

## Worker System

The local worker drains jobs from `wiki_jobs`.

CLI:

```sh
arcwell worker run-once --max-jobs 10
arcwell worker run --max-jobs-per-tick 10 --idle-sleep-ms 5000
```

Semantics:

- pending jobs are leased before execution
- failed jobs retry after backoff
- stale leases can be reclaimed
- repeated failures become `dead_lettered`

## Secrets

Secrets can come from:

- environment variables
- local `.env`
- local SQLite `secret_values`

MCP exposes set/list/delete for secret values, but no general read tool.

CLI:

```sh
arcwell secrets set-value X_BEARER_TOKEN "$X_BEARER_TOKEN" --scope x
arcwell secrets list-values
arcwell secrets delete-value X_BEARER_TOKEN
```

## Suggested Slash Commands

Slash commands are the human UI layer a host app can expose:

```text
/remember <fact>
/memory search <query>
/wiki search <query>
/wiki ingest <path-or-url>
/research plan <topic>
/research brief <topic>
/watch rss <url>
/watch github <owner/repo>
/watch arxiv <query>
/x search <query>
/project create <name>
/project status <reference>
/ops
```

## Suggested `$commands`

`$commands` are skills/habits for agents:

```text
$memory-review
$wiki-research
$deep-research
$research-audit
$x-research
$ops-ui
$project-controller
$channel-router
```

They map to skill docs under `hosts/codex/skills`.

## Validation

Rust:

```sh
cargo fmt --all -- --check
cargo test
cargo clippy --all-targets -- -D warnings
```

Cloudflare Worker:

```sh
cd packages/arcwell-edge-inbox/worker
npm install
npm run typecheck
```
