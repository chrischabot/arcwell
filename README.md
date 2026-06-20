# Arcwell

Personal assistant services for Codex, Claude, and other MCP-capable agents.

Arcwell adds long-lived assistant abilities to the agents you already use. Instead of wrapping Codex inside a separate mega-agent, it gives Codex, Claude, and other MCP-capable agents a shared well of local-first context: personal memory, a knowledge wiki, research workflows, X/Twitter ingestion, project awareness, Telegram/channel plumbing, background workers, and ops visibility.

The idea is simple:

- Keep Codex as the shell where work happens.
- Add portable services around it through MCP, CLI, skills, and optional Cloudflare Workers.
- Make the same services usable from Claude Desktop/Code and other agents.
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

Packaging, service supervision, auto-restart, and Codex plugin install strategy are documented in [docs/packaging-and-operations.md](docs/packaging-and-operations.md).

The Codex plugin slash-command and `$skill` catalog is documented in [docs/codex-plugin-commands.md](docs/codex-plugin-commands.md).

## Current Features

### Personal Memory

Stores durable personal facts and preferences, distinct from research knowledge.

Examples:

- "My cat is called Ophelia."
- "I prefer direct, sourced answers."
- "For personalized tasks, consult memory before guessing."

Tools include simple memory add/search, candidate extraction from text, reviewable candidate apply/reject, and duplicate reconciliation.

### Profile

Stores explicit profile/configuration facts such as communication preferences, writing style pointers, and support expectations.

Profile is for stable preferences and operating instructions. Memory is for personal facts. Wiki is for knowledge.

### LLM Wiki

A local Markdown knowledge base backed by SQLite metadata.

The wiki can ingest local Markdown files, public URLs, source cards, RSS/Atom feeds, GitHub releases/commits, arXiv searches, X items, and research briefs. Source cards are rendered as Markdown pages with provenance and an "untrusted evidence" warning. A separate watch-source registry tracks the feeds, GitHub owners, blogs, arXiv queries, and future handles the assistant should monitor over time.

### Deep Research

Coordinates research plans, source gathering, role-based tasks, optional web search through Brave/OpenAI/Perplexity, and wiki-grounded briefs.

The recommended flow is:

1. Make a research plan.
2. Gather current primary sources.
3. Write source cards into the wiki.
4. Run skeptic/audit passes.
5. Produce a brief from local source cards and wiki pages.

### X / Twitter

Supports replay imports, live recent search, OAuth helper flows, cursoring, source-card generation, and reports.

The service stores X items as source evidence, not as instructions.

### Edge Inbox

An optional Cloudflare Worker receives small, short-lived events while the laptop is offline. The local Rust service drains, leases, acks, nacks, retries, expires, or dead-letters them.

This is the "always-on collector, local durable brain" model.

### Channels And Telegram

Channels have a shared model for inbound/outbound messages, sender identity, project binding, source event ids, and safe text handling. Telegram is the first concrete channel package.

Incoming channel text is treated as user/content data. It is never blindly promoted to system instructions.

### Projects / Meta-Controller

Projects have names, aliases, summaries, and status. Agents can resolve natural references such as:

- "How is hyper-agent going?"
- "What about the video project?"
- "And that one?"

Ambiguous project references fail instead of guessing.

### Librarian And Digest Candidates

The librarian package turns source cards into expanded topic pages and creates digest candidates for interesting launches, releases, papers, repos, and social/news signals.

Current scoring is intentionally transparent and simple. Rich clustering and model-backed synthesis come later.

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

The Codex plugin includes slash-command prompts under [plugins/arcwell-codex/commands](plugins/arcwell-codex/commands). They expose the whole MCP tool surface: memory, profile, wiki, source cards, research, watch sources, X, projects, channels, edge inbox, workers, ops, cursors, costs, backups, and secrets.

They also expose local-only maintenance CLI actions that are intentionally not MCP tools, such as memory/profile deletion, candidate rejection, manual wiki page creation, immediate adapter runs, backup status, and external secret references.

Depending on the Codex surface, plugin commands may show with a namespace, such as `/arcwell-codex:remember`, or by their direct command name, such as `/remember`. Use the name displayed in the slash picker.

Common commands:

```text
/remember
/memory-search
/wiki-search
/wiki-ingest
/research-plan
/research-brief
/watch-rss
/watch-github
/watch-arxiv
/x-search
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
arcwell research plan "Vercel Eve"
arcwell research brief "Vercel Eve"
arcwell wiki enqueue-rss https://example.com/feed.xml
arcwell wiki enqueue-github-owner openai --limit 10
arcwell wiki enqueue-github openai codex --mode releases
arcwell wiki enqueue-arxiv "cat:cs.AI"
arcwell x rebuild-definitive-watch-sources --bookmark-days 92 --max-bookmarks 1000 --max-recent-follows 100
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
$arcwell-codex:research-brief
$arcwell-codex:x-research
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
- `$x-research`: import/search/report X evidence.
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

This is an early but working implementation. It has a broad first-pass surface area and severe tests for key failure modes, but several parts still need production depth: deployed Cloudflare queues, Telegram webhook transform, richer project/thread sync, model-backed librarian synthesis, full mem0-style memory reconciliation, and a browser ops UI.

## License

MIT. See [LICENSE](LICENSE).
