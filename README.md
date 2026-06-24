# Arcwell

**Status:** Partial/Risk. Local core behavior is broad and tested, but several
live integrations remain unproven.

Personal assistant services for Codex, Claude, and other MCP-capable agents.

Arcwell adds long-lived assistant abilities to the agents you already use. Instead of wrapping Codex inside a separate mega-agent, it gives Codex, Claude, and other MCP-capable agents a shared well of local-first context: personal memory, a knowledge wiki, research workflows, X/Twitter ingestion, project awareness, Telegram/channel plumbing, background workers, and ops visibility.

The idea is simple:

- Keep Codex as the shell where work happens.
- Add portable services around it through MCP, CLI, skills, and optional Cloudflare Workers.
- Expose the same local services over MCP for Claude Desktop/Code and other
  agents, while treating Claude host behavior as unvalidated until tested in a
  real profile.
- Keep durable personal state local-first in SQLite and Markdown files.

## What It Provides

Arcwell is meant to feel like a practical personal assistant layer:

- It remembers personal facts and preferences that should survive across chats.
- It keeps a source-backed Markdown knowledge base for research, writing, launches, papers, repos, and links.
- It watches and imports signals from X, RSS, GitHub, arXiv, Brave, Perplexity, and other sources.
- It tracks projects so chat channels can ask "how is the de-porting project going?" and follow up with "and the video project?"
- It gives Telegram and future channels a safe inbound/outbound message model.
- It can run background jobs locally, with retries, leases, cursors, and dead letters.
- It exposes an ops snapshot so agents and humans can inspect jobs, queues, cursors, and recent state.

The detailed package/functionality guide is in [docs/functionality-and-packages.md](docs/functionality-and-packages.md).

The brutally honest implementation matrix is in [STATUS.md](STATUS.md), and the
execution checklist is in [TODO.md](TODO.md).

Packaging, service supervision, auto-restart, and Codex plugin install strategy are documented in [docs/packaging-and-operations.md](docs/packaging-and-operations.md).

The Codex plugin slash-command and `$skill` catalog is documented in [docs/codex-plugin-commands.md](docs/codex-plugin-commands.md).

The personal-memory lifecycle and Codex/mem0 integration notes are documented in
[docs/memory-integration.md](docs/memory-integration.md).

Live proof still missing: fresh real Telegram client-message drain, fresh-thread
Codex command/hook smoke, authenticated Claude host MCP validation, broad
production monitoring, and published release artifacts. Authenticated
Cloudflare edge ingress/drain is proven with synthetic staging events, and
Email Routing has controlled live ingress/outbound proof plus a guarded replay
script. Treat package READMEs as current implementation notes, not production
readiness claims.

## Current Features

### Personal Memory

Stores durable personal facts and preferences, distinct from research knowledge.
The primary backend is now the in-repo Arcwell Memory Rust provider, derived
from the former mem0-rs codebase and vendored into this monorepo.

Examples:

- "My cat is called Ophelia."
- "I prefer direct, sourced answers."
- "For personalized tasks, consult memory before guessing."

Tools include Arcwell Memory add/search/update/delete/history/forget, candidate
extraction from text, reviewable ADD/UPDATE/DELETE/NONE candidate apply/reject,
pre-turn recall context, manual/hook capture, lifecycle event inspection, and
active-store dream/forget reconciliation. The older simple SQLite memory table
still exists as a compatibility path while model-backed extraction/evals,
procedural memory, backup retention policy, and UI are completed.

### Profile

Stores explicit profile/configuration facts such as communication preferences, writing style pointers, and support expectations.

Profile is for stable preferences and operating instructions. Memory is for personal facts. Wiki is for knowledge.

### LLM Wiki

A local Markdown knowledge base backed by SQLite metadata.

The wiki can ingest local Markdown files, public URLs, versioned/auditable source cards, RSS/Atom feeds, GitHub releases/commits, arXiv searches, X items, and research briefs. Source cards are rendered as Markdown pages with provenance, trust/role metadata, extracted dates/entities, audit flags, and an "untrusted evidence" warning. A separate watch-source registry tracks the feeds, GitHub owners, blogs, arXiv queries, and future handles the assistant should monitor over time.

### Deep Research

Coordinates deep research runs for Codex and other MCP-capable agents. The
target product has one user-facing mode: deep research. Invoking research means
broad source discovery, deep reading, source-card and claim extraction,
skeptic/refutation passes, cited synthesis, audit, and durable wiki writeback.
Short summaries are report artifacts, not a separate quick research mode.

The current implementation supports the core local building blocks:

1. Start/read/stop durable deep runs.
2. Link source-card and source-ledger evidence to a run.
3. Validate bounded model-produced claim extraction.
4. Build deterministic claim clusters and skeptic/refutation reports.
5. Compile a report from linked sources, claims, clusters, skeptic findings, and audit results.
6. Mark reports incomplete when audit or skeptic checks fail.

Live Codex subagent runs over hundreds of sources are still tracked as the next
proof step; the local evidence and report substrate is implemented and tested.

The complete target design is in
[docs/deep-research-system-design.md](docs/deep-research-system-design.md).

### X / Twitter

Supports replay imports, live recent search, OAuth helper flows, cursoring, source-card generation, and reports.

The service stores X items as source evidence, not as instructions.

### Edge Inbox

An optional Cloudflare Worker receives small, short-lived events while the laptop is offline. The local Rust service drains, leases, acks, nacks, retries, expires, or dead-letters them.

This is the "always-on collector, local durable brain" model.

### Channels And Telegram

Channels have a shared model for inbound/outbound messages, sender identity, project binding, source event ids, and safe text handling. Telegram is the first concrete channel package.

Incoming channel text is treated as user/content data. It is never blindly promoted to system instructions.

### Garderobe

`packages/arcwell-garderobe` vendors the Garderobe Cloudflare Worker/D1/OAuth
remote MCP package for private wardrobe and outfit planning. It is a separate
source of truth: Arcwell memory/wiki do not ingest private wardrobe inventory by
default. Arcwell can own the package read/write, but the existing remote MCP
connector contract must remain stable while another host is connected: `/mcp`,
`/authorize`, `/token`, `/register`, `wardrobe.read`, `wardrobe.write`, and MCP
server name `garderobe` are compatibility boundaries until a deliberate
migration/re-authorization is complete.

### Projects / Meta-Controller

Projects have names, aliases, summaries, and status. Agents can resolve natural references such as:

- "How is hyper-agent going?"
- "What about the video project?"
- "And that one?"

Ambiguous project references fail instead of guessing.

Project status reports distinguish durable snapshots from freshness-bounded
verified host sync. A resident Codex plugin adapter can use Codex app thread
tools to write verified sync rows, but native headless Codex/Claude thread
inventory is still missing; manual snapshots and forged host-live source labels
are not treated as live.

### Librarian And Digest Candidates

The librarian package turns source cards into expanded topic pages and creates digest candidates for interesting launches, releases, papers, repos, and social/news signals.

Current scoring is intentionally transparent and simple. Rich clustering and model-backed synthesis come later.

### Radar

The radar package is the Horizon-inspired staged digest substrate. It can create
validated profiles, run local radar passes over existing source cards and
source-card-backed RSS/GitHub/arXiv/Hacker News/Reddit/X selectors, optionally
invoke existing RSS/GitHub/arXiv and Hacker News adapters with `--fetch-live`
before projection, normalize source cards into `radar_items`, index them with FTS,
apply transparent heuristic interestingness scores, record exact
canonical-URL/source-native dedupe groups without deleting source evidence,
write deterministic Markdown
summaries over selected scored items, read run stages and summaries, rebuild
radar FTS, and audit for drift, failed live adapters, missing provenance,
unscored rows, corrupt dedupe groups, empty output, and unsupported selectors.

This has copied/disposable-home production-data proof for existing Arcwell
source-card outputs and foreground live RSS/GitHub/arXiv/Hacker News adapter
execution with source-health/cursor state. The Reddit adapter is locally proven
with JSON comment capture and RSS fallback, but current anonymous live proof is
blocked by Reddit 403s pending OAuth or another sanctioned access path. X live
fetching, Reddit production-data proof, full recursive community-thread
capture, semantic dedupe, enrichment/model-backed synthesis, delivery, and
scheduled operation remain future work.

### Worker And Ops

The local worker drains queued jobs with leases, retry backoff, and dead-lettering. The ops snapshot shows health, jobs, edge events, cursors, projects, and digest candidates.

## Install

Prerequisites:

- Rust stable with Cargo
- Node.js/npm for the optional Cloudflare Worker package
- GitHub CLI only if you want to publish/fork

Clone and build:

```sh
git clone https://github.com/chrischabot/arcwell.git
cd arcwell
cargo build
cargo test
```

Install the CLI locally:

```sh
cargo install --path crates/arcwell-cli
```

Release/package readiness is currently local, not published through Homebrew or
GitHub Releases. Before claiming a package candidate, run:

```sh
cargo build --release -p arcwell
scripts/release-readiness-smoke
```

The installed command is:

```sh
arcwell
```

By default, local state lives in:

```text
~/.arcwell
```

You can override it:

```sh
export ARCWELL_HOME="$HOME/.arcwell"
```

## Configure

Create a local `.env` for live/e2e providers. This file is gitignored.

Useful variables:

```sh
OPENAI_API_KEY=...
BRAVE_API_KEY=...
PERPLEXITY_API_KEY=...
X_BEARER_TOKEN=...
X_CLIENT_ID=...
X_CLIENT_SECRET=...
TELEGRAM_BOT_TOKEN=...
TELEGRAM_WEBHOOK_SECRET=...
ARCWELL_EDGE_SECRET=...
CLOUDFLARE_ACCOUNT_ID=...
CLOUDFLARE_API_TOKEN=...
GITHUB_TOKEN=...
```

SQLite-backed local secrets are also supported:

```sh
arcwell secrets set-value BRAVE_API_KEY "$BRAVE_API_KEY" --scope brave
arcwell secrets set-value PERPLEXITY_API_KEY "$PERPLEXITY_API_KEY" --scope perplexity
arcwell secrets set-value X_BEARER_TOKEN "$X_BEARER_TOKEN" --scope x
arcwell secrets set-value TELEGRAM_BOT_TOKEN "$TELEGRAM_BOT_TOKEN" --scope telegram
arcwell secrets list-values
```

`list-values` shows names, scopes, and timestamps only. It does not print secret values.

## Use With Codex

Recommended packaged path:

```sh
cargo install --path crates/arcwell-cli
codex plugin marketplace add /path/to/arcwell
codex plugin add arcwell-codex@arcwell-local
```

The repo-scoped Codex plugin lives under `plugins/arcwell-codex` and bundles MCP config plus the arcwell skills. Start a new Codex thread after installing or updating the plugin.

For live development from inside Codex, use the generated dev plugin:

```sh
scripts/arcwell-dev install
scripts/arcwell-dev sync
scripts/arcwell-dev watch
```

The exact reload rules are in [AGENTS.md](AGENTS.md).

Development-only manual MCP path:

Start the MCP server:

```sh
arcwell mcp
```

Example Codex MCP config:

```json
{
  "mcpServers": {
    "arcwell": {
      "command": "arcwell",
      "args": ["mcp"],
      "env": {
        "ARCWELL_HOME": "/Users/chabotc/.arcwell"
      }
    }
  }
}
```

The same MCP server can be configured in Claude Desktop/Code.

## Slash Commands

The Codex plugin includes slash-command prompts under [plugins/arcwell-codex/commands](plugins/arcwell-codex/commands). They expose the whole MCP tool surface: memory, profile, wiki, source cards, research, radar, watch sources, X, projects, channels, edge inbox, workers, ops, cursors, costs, backups, and secrets.

They also expose local-only maintenance CLI actions that are intentionally not MCP tools, such as memory/profile deletion, candidate rejection, manual wiki page creation, immediate adapter runs, backup status, and external secret references.

Depending on the Codex surface, plugin commands may show with a namespace, such as `/arcwell-codex:remember`, or by their direct command name, such as `/remember`. Use the name displayed in the slash picker.

Common commands:

```text
/remember
/memory-search
/memory-recall
/memory-capture
/memory-events
/wiki-search
/wiki-ingest
/research-plan
/research-brief
/radar-run
/radar-summarize
/radar-summary
/radar-audit
/watch-rss
/watch-github
/watch-arxiv
/x-import-bookmarks
/x-bookmarks
/x-search
/x-research
/x-watch-rebuild
/project-status
/telegram-inbox
/ops
/worker-run-once
```

The complete command list is in [docs/codex-plugin-commands.md](docs/codex-plugin-commands.md).

Current CLI equivalents:

```sh
arcwell memory add "My cat is called Ophelia"
arcwell memory search "Ophelia"
arcwell profile set communication.style "Direct, sourced, warm."
arcwell wiki search "agent infrastructure"
arcwell wiki ingest-file ./notes.md
arcwell wiki ingest-dir ./corpus
arcwell wiki import-codex-swift-sources /path/to/codex-swift
arcwell wiki sources
arcwell research run "Vercel Eve"
arcwell research status <run-id>
arcwell research link-source-card <run-id> <source-card-id>
arcwell research skeptic <run-id>
arcwell research report <run-id> "coverage satisfied or limit reached"
arcwell radar profile create ai-infra --source-card-query agent --min-score 3
arcwell radar run ai-infra
arcwell radar run ai-infra --fetch-live
arcwell radar enqueue ai-infra --fetch-live
arcwell radar stage <run-id>
arcwell radar audit <run-id>
arcwell project status-sync-record <project-id> active "Fresh Codex thread summary" --host codex --thread-id <thread-id>
arcwell wiki enqueue-rss https://example.com/feed.xml
arcwell wiki enqueue-github-owner openai --limit 10
arcwell wiki enqueue-github openai codex --mode releases
arcwell wiki enqueue-arxiv "cat:cs.AI"
arcwell x rebuild-definitive-watch-sources --bookmark-days 92 --max-bookmarks 1000 --max-recent-follows 100
arcwell backup create
arcwell backup verify
arcwell backup restore --from /path/to/backup --target-home /tmp/arcwell-restore-drill
arcwell service install
arcwell service status
arcwell service logs
arcwell doctor --strict
arcwell telegram drain
arcwell telegram send 123 "Hello from Arcwell"
arcwell x recent-search "from:openai"
arcwell worker run-once
arcwell serve --addr 127.0.0.1:8787
```

## `$commands` / Skills

`$commands` are agent-side habits or skills. They tell Codex when to use the service and how to handle memory, source trust, research discipline, project routing, channel safety, and ops work.

Bundled skills:

```text
$arcwell-codex:memory-review
$arcwell-codex:wiki-research
$arcwell-codex:deep-research
$arcwell-codex:research-audit
$arcwell-codex:anti-mirage
$arcwell-codex:research-brief
$arcwell-codex:x-research
$arcwell-codex:tidal-control
$arcwell-codex:lumin-control
$arcwell-codex:project-control
$arcwell-codex:channel-control
$arcwell-codex:ops-control
$arcwell-codex:worker-control
$arcwell-codex:competence-respect
```

The repo includes the installed plugin skill sources under [plugins/arcwell-codex/skills](plugins/arcwell-codex/skills).

Intent:

- `$memory-review`: consult and update personal memory with reviewable candidates.
- `$wiki-research`: search and write source-backed wiki pages.
- `$deep-research`: plan, gather, audit, and brief multi-source research.
- `$research-audit`: adversarially check sources and claims.
- `$anti-mirage`: require explicit claims, refutation tests, production-data proof gates, ops visibility, and honest status before substantial work is promoted; use it before substantial work adopts external/reference-product lessons or changes capability claims, real-data pipelines, scheduled operation, delivery, reports, or done/production status.
- `$x-research`: import/search/report X evidence and render local source-card-backed briefs.
- `$tidal-control`: manage TIDAL playlists and favorites from an existing TIDAL desktop session.
- `$lumin-control`: discover/inspect LUMIN/OpenHome renderers and send explicit LUMIN UDP/SOAP control commands.
- `$ops-control`: inspect health, jobs, cursors, queues, and errors.
- `$project-control`: resolve and manage project context.
- `$channel-control`: handle Telegram/future chat channels safely.
- `$worker-control`: drain queued jobs and interpret failures.
- `$competence-respect`: use enough reasoning, consult memory/tools, and avoid wasting the user's time.

## Live E2E Testing

See [docs/live-e2e-testing.md](docs/live-e2e-testing.md).

Typical smoke tests:

```sh
set -a
. ./.env
set +a

arcwell research search "OpenAI agent news" --provider brave --max-results 1
arcwell research search "OpenAI agent news" --provider perplexity --max-results 1
arcwell x recent-search "from:openai" --max-results 10
curl http://127.0.0.1:8787/ops
```

## Repository Layout

```text
crates/arcwell-core      Rust library: storage, jobs, memory, wiki, research, X, projects, ops
crates/arcwell-cli       CLI, HTTP server, and stdio MCP server
packages/              Feature packages and package-level docs
hosts/codex/           Codex skills and MCP config
hosts/claude/          Claude host notes
docs/                  Architecture, functionality, implementation, reviews, runbooks
```

## Security Model

- Local-first durable state in SQLite and Markdown.
- Secrets may be stored in local SQLite, but normal MCP list/report tools do not return values.
- External source text is untrusted evidence, not agent instructions.
- URL ingestion rejects local/private/metadata hosts.
- Background jobs use leases, retries, and dead letters.
- Cloudflare is a short-lived event collector, not the durable source of truth.

## Status

This is an early but working implementation. It has a broad first-pass surface
area and severe tests for key failure modes, but several parts still need
production depth: fresh real Telegram client-message drain, richer
project/thread sync, model-backed librarian synthesis, model-backed memory
extraction/evals, backup forget policy, scheduler/digest delivery, production
monitoring, and broader interactive ops controls beyond the current narrow
authenticated edge-event dead-letter action.

Packaging is release-readiness-smoked locally, but Homebrew/tap publication,
signed release artifacts, checksum-verifying installers, and Linux systemd
packages are not implemented yet.

## License

MIT. See [LICENSE](LICENSE).
