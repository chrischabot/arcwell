# Packaging And Operations

This document describes how `arcwell` should be packaged, installed, run, and exposed to Codex and other MCP-capable agents.

## Goals

- Install a fast local Rust binary once, then let agents call it without rebuilding.
- Run local background workers with automatic restart and visible logs.
- Keep the Codex integration installable as a plugin, not a fork or manual prompt paste.
- Keep Claude Desktop/Code support through the same MCP server.
- Keep Cloudflare Workers as optional always-on collectors, not the durable source of truth.

## Recommended Shape

Use three layers:

1. **System package:** installs the `arcwell` binary and optional service files.
2. **Local services:** supervised processes for worker drains and optional HTTP/ops.
3. **Codex plugin:** bundles skills and MCP config so Codex knows how to use `arcwell`.

This keeps responsibilities clean:

- The OS package owns binaries, upgrades, launchd/systemd, logs, and uninstall.
- The Rust service owns SQLite, Markdown wiki pages, jobs, secrets, and MCP/HTTP APIs.
- The Codex plugin owns agent instructions, MCP registration, and user-facing workflows.

## Local Services

There are three useful process modes:

```sh
arcwell worker run --max-jobs-per-tick 10 --idle-sleep-ms 5000
arcwell serve --addr 127.0.0.1:8787
arcwell mcp
```

Recommended defaults:

- `arcwell worker run` should be a supervised user service.
- `arcwell serve` should be optional but useful for the ops UI and local browser inspection.
- `arcwell mcp` should normally be launched by Codex/Claude as needed, not run as a separate daemon.

Rationale:

- The worker is long-lived and should drain queues even when no Codex thread is active.
- The HTTP server is useful for humans, ops UI, and local integrations, but should be localhost-only.
- MCP stdio servers are normally lifecycle-managed by the host agent.

## macOS LaunchAgent

Primary macOS package target should install LaunchAgents under:

```text
~/Library/LaunchAgents/com.chrischabot.arcwell.worker.plist
~/Library/LaunchAgents/com.chrischabot.arcwell.http.plist
```

Recommended worker settings:

- `KeepAlive = true`
- `RunAtLoad = true`
- `ThrottleInterval = 10`
- `StandardOutPath = ~/.arcwell/logs/worker.log`
- `StandardErrorPath = ~/.arcwell/logs/worker.err.log`
- `EnvironmentVariables.ARCWELL_HOME = ~/.arcwell`

Recommended HTTP settings:

- disabled by default or installed but not bootstrapped unless requested
- bind only to `127.0.0.1`
- logs under `~/.arcwell/logs/http.log`

Commands the installer should provide:

```sh
arcwell service install worker
arcwell service install http
arcwell service start worker
arcwell service stop worker
arcwell service restart worker
arcwell service status
arcwell service logs worker
arcwell service uninstall
```

These can be CLI subcommands later; initially they can be shell scripts generated from templates.

## Linux systemd

Linux packaging should use user services:

```text
~/.config/systemd/user/arcwell-worker.service
~/.config/systemd/user/arcwell-http.service
```

Recommended unit behavior:

- `Restart=always`
- `RestartSec=5`
- `Environment=ARCWELL_HOME=%h/.arcwell`
- `ExecStart=/usr/local/bin/arcwell worker run --max-jobs-per-tick 10 --idle-sleep-ms 5000`

Installers should run:

```sh
systemctl --user daemon-reload
systemctl --user enable --now arcwell-worker.service
```

## Windows

Windows should be a second pass. Preferred options:

- install the `arcwell.exe` binary
- run via Task Scheduler at logon for worker mode
- optionally support `winsw` later for a proper user service

## Package Distribution

Recommended package order:

1. **Cargo install:** immediate developer path.
2. **Homebrew tap:** best macOS first-class path.
3. **GitHub Releases:** prebuilt universal macOS, Linux, and Windows artifacts.
4. **Shell installer:** convenience wrapper around releases plus service install.

Homebrew should be the default for macOS:

```sh
brew tap chrischabot/arcwell
brew install arcwell
arcwell service install worker
arcwell service start worker
```

Cargo remains useful for contributors:

```sh
cargo install --git https://github.com/chrischabot/arcwell arcwell
```

The package should install one binary named `arcwell`.

## Codex Plugin

Codex should receive `arcwell` as a plugin, not a pile of loose manual setup.

The repo now includes a repo-scoped plugin package:

```text
.agents/plugins/marketplace.json
plugins/arcwell-codex/
```

The plugin bundles:

- skills for memory, wiki, X, research, audits, and competence-respect behavior
- slash-command prompts for the full MCP tool surface
- slash-command prompts for local-only maintenance CLI actions that should remain explicit
- MCP configuration for the local `arcwell mcp` server
- interface metadata for the Codex plugin picker

The plugin assumes the `arcwell` binary is already installed and on PATH.

Install flow for local development:

```sh
codex plugin marketplace add /Users/chabotc/Projects/arcwell
codex plugin add arcwell-codex@arcwell-local
```

After install, start a new Codex thread so skills and MCP tools are loaded.

The complete slash-command and `$skill` catalog is in [codex-plugin-commands.md](codex-plugin-commands.md).

## Codex MCP Config

The plugin MCP config should be:

```json
{
  "mcpServers": {
    "arcwell": {
      "command": "arcwell",
      "args": ["mcp"],
      "env": {
        "ARCWELL_HOME": "/Users/chabotc/.arcwell"
      },
      "startup_timeout_sec": 10,
      "tool_timeout_sec": 120
    }
  }
}
```

For development only, `hosts/codex/mcp.json` may still point at `cargo run`. The packaged plugin should never do that.

## Skills Versus Services

Use skills for agent behavior:

- how to research
- when to consult memory
- how to audit claims
- how to treat X/source text as untrusted
- how to use the wiki before writing

Use MCP tools for live actions:

- read/write memory
- search/read wiki pages
- queue ingestion jobs
- inspect ops
- import X/bookmarks/follows
- drain workers once

Use services for unattended work:

- job queue draining
- adapter polling
- edge inbox draining
- digest candidate generation

Use Cloudflare for internet-facing always-on capture:

- Telegram webhooks
- OAuth callback capture
- edge inbox buffering
- RSS/webhook capture if local laptop is offline

Do not use Codex automations as the primary service runner. Automations are useful for agent-level recurring review, but the local job worker should keep running independently of a Codex thread.

Use slash-command prompts for quick human entry points into the same MCP-backed actions:

- `/remember`, `/wiki-search`, `/research-brief`, `/x-search`, `/project-status`, `/telegram-inbox`, `/ops`
- plugin-hosted command names may be namespaced by Codex when displayed in the slash picker
- prompts should stay thin and defer real behavior to MCP tools and skills

## Auto-Restart And Reliability

Minimum service reliability:

- worker process auto-restarts
- every job has a lease, bounded retry, and dead-letter path
- service logs are stable and inspectable
- `arcwell doctor` reports health and warnings
- `arcwell ops` or `arcwell://ops` reports jobs, cursors, dead letters, and watch source counts

Future reliability upgrades:

- `arcwell service status` should inspect launchd/systemd and recent logs
- `arcwell doctor --strict` should fail nonzero on missing service, stale worker heartbeat, or excessive dead letters
- worker heartbeat row in SQLite
- backpressure limits per adapter kind
- per-source health table

## Secrets

Local SQLite secrets are acceptable for this project. The installer should:

- create `~/.arcwell` with `0700`
- create logs and wiki directories
- never print secret values
- support importing `.env` values into SQLite secrets

Cloudflare secrets remain in Wrangler/Cloudflare, not local SQLite.

## Upgrade Strategy

Upgrades should be boring:

```sh
brew upgrade arcwell
arcwell migrate
arcwell doctor
launchctl kickstart -k gui/$UID/com.chrischabot.arcwell.worker
```

The binary should preserve backward-compatible SQLite migrations. Destructive migrations must require an explicit backup.

## Roadmap

Phase 1:

- keep `cargo install --path crates/arcwell-cli`
- add the repo-scoped Codex plugin and marketplace
- document LaunchAgent/systemd target design

Phase 2:

- add `arcwell service` subcommands for macOS LaunchAgents
- add generated LaunchAgent templates
- add `arcwell doctor --strict`

Phase 3:

- add GitHub Actions release artifacts
- add Homebrew tap formula
- add install script that downloads a release and optionally installs services

Phase 4:

- split packages into optional feature groups if needed
- add Cloudflare deploy helpers per edge package
- add plugin share/publish workflow for Codex workspace use
