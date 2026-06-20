# agent-services

Personal assistant services for Codex, Claude, and other MCP-capable agents.

`agent-services` adds long-lived assistant abilities to the agent you already use. Instead of wrapping Codex inside a separate mega-agent, it gives Codex a set of local and edge services it can call: memory, a knowledge wiki, research workflows, X/Twitter ingestion, project awareness, Telegram/channel plumbing, background workers, and ops visibility.

The idea is simple:

- Keep Codex as the shell where work happens.
- Add portable services around it through MCP, CLI, skills, and optional Cloudflare Workers.
- Make the same services usable from Claude Desktop/Code and other agents.
- Keep durable personal state local-first in SQLite and Markdown files.

## What It Provides

`agent-services` is meant to feel like a practical personal assistant layer:

- It remembers personal facts and preferences that should survive across chats.
- It keeps a source-backed Markdown knowledge base for research, writing, launches, papers, repos, and links.
- It watches and imports signals from X, RSS, GitHub, arXiv, Brave, Perplexity, and other sources.
- It tracks projects so chat channels can ask "how is the de-porting project going?" and follow up with "and the video project?"
- It gives Telegram and future channels a safe inbound/outbound message model.
- It can run background jobs locally, with retries, leases, cursors, and dead letters.
- It exposes an ops snapshot so agents and humans can inspect jobs, queues, cursors, and recent state.

The detailed package/functionality guide is in [docs/functionality-and-packages.md](docs/functionality-and-packages.md).

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

The wiki can ingest local Markdown files, public URLs, source cards, RSS/Atom feeds, GitHub releases/commits, arXiv searches, X items, and research briefs. Source cards are rendered as Markdown pages with provenance and an "untrusted evidence" warning.

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
git clone https://github.com/chrischabot/agent-services.git
cd agent-services
cargo build
cargo test
```

Install the CLI locally:

```sh
cargo install --path crates/agent-cli
```

The installed command is:

```sh
agent
```

By default, local state lives in:

```text
~/.agent-services
```

You can override it:

```sh
export AGENT_SERVICES_HOME="$HOME/.agent-services"
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
AGENT_EDGE_SECRET=...
CLOUDFLARE_ACCOUNT_ID=...
CLOUDFLARE_API_TOKEN=...
GITHUB_TOKEN=...
```

SQLite-backed local secrets are also supported:

```sh
agent secrets set-value BRAVE_API_KEY "$BRAVE_API_KEY" --scope brave
agent secrets set-value PERPLEXITY_API_KEY "$PERPLEXITY_API_KEY" --scope perplexity
agent secrets set-value X_BEARER_TOKEN "$X_BEARER_TOKEN" --scope x
agent secrets set-value TELEGRAM_BOT_TOKEN "$TELEGRAM_BOT_TOKEN" --scope telegram
agent secrets list-values
```

`list-values` shows names, scopes, and timestamps only. It does not print secret values.

## Use With Codex

Start the MCP server:

```sh
agent mcp
```

Example Codex MCP config:

```json
{
  "mcpServers": {
    "agent-services": {
      "command": "agent",
      "args": ["mcp"],
      "env": {
        "AGENT_SERVICES_HOME": "/Users/chabotc/.agent-services"
      }
    }
  }
}
```

The same MCP server can be configured in Claude Desktop/Code.

## Slash Commands

Slash commands are the human-facing actions a chat host can expose. They map to CLI/MCP tools.

Suggested commands:

```text
/remember <fact>
/memory search <query>
/profile set <key> <value>
/wiki search <query>
/wiki ingest <path-or-url>
/research plan <topic>
/research brief <topic>
/watch rss <feed-url>
/watch github <owner/repo>
/watch arxiv <query>
/x search <query>
/project create <name>
/project status <reference>
/telegram inbox
/ops
/worker run-once
```

Current CLI equivalents:

```sh
agent memory add "My cat is called Ophelia"
agent memory search "Ophelia"
agent profile set communication.style "Direct, sourced, warm."
agent wiki search "agent infrastructure"
agent wiki ingest-file ./notes.md
agent research plan "Vercel Eve"
agent research brief "Vercel Eve"
agent wiki enqueue-rss https://example.com/feed.xml
agent wiki enqueue-github openai codex --mode releases
agent wiki enqueue-arxiv "cat:cs.AI"
agent x recent-search "from:openai"
agent worker run-once
agent serve --addr 127.0.0.1:8787
```

## `$commands` / Skills

`$commands` are agent-side habits or skills. They tell Codex/Claude when to use the service.

Suggested commands:

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

The repo includes Codex skill drafts under [hosts/codex/skills](hosts/codex/skills).

Intent:

- `$memory-review`: consult and update personal memory with reviewable candidates.
- `$wiki-research`: search and write source-backed wiki pages.
- `$deep-research`: plan, gather, audit, and brief multi-source research.
- `$research-audit`: adversarially check sources and claims.
- `$x-research`: import/search/report X evidence.
- `$ops-ui`: inspect health, jobs, cursors, queues, and errors.
- `$project-controller`: resolve and manage project context.
- `$channel-router`: handle Telegram/future chat channels safely.

## Live E2E Testing

See [docs/live-e2e-testing.md](docs/live-e2e-testing.md).

Typical smoke tests:

```sh
set -a
. ./.env
set +a

agent research search "OpenAI agent news" --provider brave --max-results 1
agent research search "OpenAI agent news" --provider perplexity --max-results 1
agent x recent-search "from:openai" --max-results 10
curl http://127.0.0.1:8787/ops
```

## Repository Layout

```text
crates/agent-core      Rust library: storage, jobs, memory, wiki, research, X, projects, ops
crates/agent-cli       CLI, HTTP server, and stdio MCP server
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
