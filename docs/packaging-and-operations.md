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
- The current `/ops/ui` page is mostly an inspection surface, with one narrow
  authenticated edge-event dead-letter action. Installers should not expose
  broader mutating controls until each action has token auth, same-origin/CSRF
  checks, policy enforcement, idempotency/replay handling, and severe tests.
- MCP stdio servers are normally lifecycle-managed by the host agent.

## macOS LaunchAgent

Primary macOS package target should install LaunchAgents under:

```text
~/Library/LaunchAgents/com.arcwell.worker.plist
~/Library/LaunchAgents/com.arcwell.http.plist
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
- set `ARCWELL_HTTP_AUTH_TOKEN` to a long local random token before exposing
  the API to tools beyond the same-user localhost workflow
- keep browser integrations same-origin; the server intentionally rejects
  hostile `Origin` headers and does not enable broad CORS
- logs under `~/.arcwell/logs/http.log`

Commands now provided by the CLI for the macOS worker service:

```sh
arcwell service install
arcwell service status
arcwell service restart
arcwell service logs
arcwell service uninstall
arcwell doctor --strict
```

Current macOS state: this writes and loads a user LaunchAgent plist for the
local worker, records worker heartbeat in SQLite, exposes strict doctor checks
for backup verification, stale backups, dead letters, stale/missing heartbeat,
required local directories, schema drift, missing/non-file LaunchAgent plist,
corrupt service plist metadata, and missing worker binary, and supports
`launchctl kickstart -k` through `arcwell service restart`.

`scripts/service-live-smoke` is the repeatable severe smoke for this surface. It
uses disposable `HOME` and `ARCWELL_HOME` paths containing spaces and hostile
shell/XML characters, refuses to disturb an already-loaded `com.arcwell.worker`
unless explicitly allowed, and exercises:

- no-load install metadata validation;
- strict doctor pass with fresh backup and heartbeat;
- strict doctor failure for stale heartbeat, corrupt plist, and missing binary;
- explicit log-read errors for unreadable log files;
- uninstall cleanup without deleting unrelated Arcwell home data;
- real launchd install, status, restart, killed-worker recovery, logs, and
  uninstall when macOS launchd is available.

Commands the installer should ultimately provide:

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

Linux systemd user-service support now has repo-owned templates and a cautious
installer scaffold, but live user-service behavior is still unproven. The
reason is verification: this wave ran on macOS and cannot honestly prove
`systemctl --user` availability, lingering behavior after logout, journal log
paths, restart semantics, or uninstall cleanup on a Linux user session. Until a
Linux CI/staging host is available, Arcwell should claim only rendered unit
files and disposable-path installer validation, not running Linux service
support.

The target design remains user services:

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

Current scaffold:

```text
packaging/systemd/arcwell-worker.service.in
packaging/systemd/arcwell-http.service.in
scripts/install-systemd-user
```

`scripts/install-systemd-user install --no-systemctl --unit-dir <temp-dir>`
renders worker and optional HTTP units into a caller-provided directory without
starting services. It requires absolute paths, quotes/escapes systemd-sensitive
characters in rendered paths, creates `ARCWELL_HOME/logs`, and never removes
`ARCWELL_HOME`. It only runs `systemctl --user enable --now` when
`--enable-now` is explicitly supplied.

Done criteria for implementing systemd later:

- generate and install the unit under a disposable `$HOME`;
- run `systemctl --user daemon-reload`, `enable --now`, `status`, `restart`,
  and `disable --now`;
- prove `Restart=always` recovers a killed worker;
- prove `doctor --strict` catches stale heartbeat while stopped;
- prove logs are inspectable through `journalctl --user-unit`;
- prove uninstall removes only the unit file and leaves unrelated
  `ARCWELL_HOME` data intact.

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

Current release scaffolds:

```text
packaging/homebrew/arcwell.rb.template
packaging/install.sh
scripts/verify-packaging-artifacts
```

`packaging/homebrew/arcwell.rb.template` is a tap-ready formula template with
per-target URL and SHA-256 placeholders. It is not a published formula and must
be rendered with real release archive URLs and checksums before use.

`packaging/install.sh` installs from either a local archive plus explicit
SHA-256 or a release URL plus `checksums.txt`. It verifies the SHA-256 before
extracting, rejects absolute paths, `..` tar members, and non-regular
symlink/hardlink-style archive members, installs only the `arcwell` binary into
`<prefix>/bin`, and does not install or start services.

## Release Readiness Checklist

Assumptions for the current local release gate:

- The candidate package installs one executable named `arcwell`.
- The stable Codex plugin invokes `arcwell` from `PATH`, not `cargo run`, a
  debug binary, or the generated dev wrapper.
- Local user data remains under `ARCWELL_HOME`; uninstalling a service or
  package must not delete that data unless the user explicitly asks for a data
  wipe.
- Schema version `1` is the only supported SQLite schema in this checkout.
  A future destructive migration must require a fresh backup and an explicit
  migration/restore plan before release.

Behavioral claim:

> A release candidate binary plus the stable Codex plugin can be staged in a
> fresh install prefix, survive interrupted upgrade simulation, expose stale
> `PATH`/plugin-binary mismatches, preserve backup/restore compatibility, catch
> unsupported schema drift, handle duplicate service install, and uninstall
> service state without deleting user data.

Run this before claiming release readiness:

```sh
cargo build --release -p arcwell
scripts/release-readiness-smoke
scripts/service-live-smoke --no-live
scripts/verify-packaging-artifacts
scripts/verify-codex-plugin-docs
```

On a macOS machine where no real `com.arcwell.worker` user service is already
loaded, also run:

```sh
scripts/service-live-smoke --live
```

For Codex plugin/dev-loop packaging changes, also run:

```sh
scripts/arcwell-dev smoke
scripts/arcwell-dev sync
```

For release artifact/template changes, also run the local adversarial fixture
test:

```sh
scripts/verify-packaging-artifacts --self-test
```

`scripts/release-readiness-smoke` uses disposable install, `HOME`, and
`ARCWELL_HOME` paths containing spaces and shell/XML-hostile characters. It
checks:

- release candidate binary starts from a staged package prefix;
- stable plugin `.mcp.json` runs `arcwell mcp`;
- stable plugin hooks call `arcwell` and do not reference `cargo`,
  `target/debug`, `.arcwell-dev`, or `arcwell-dev`;
- a stale `arcwell` earlier on `PATH` is actually detected, then corrected by
  placing the package prefix first;
- interrupted upgrade simulation leaves the existing binary hash unchanged;
- backup create, backup verify, backup restore, and profile readback work from
  temporary homes;
- `doctor --strict` rejects an old/unsupported schema version;
- duplicate `service install --no-load` does not create duplicate plist files;
- `service uninstall --no-unload` is idempotent and preserves unrelated
  `ARCWELL_HOME` data;
- a bad log path and an unwritable home fail explicitly.

Publication blockers:

- No published Homebrew tap/formula exists yet; the repository contains only a
  formula template.
- No GitHub Actions release workflow creates signed or checksummed archives.
- The install script has local checksum/path-traversal fixture coverage, but no
  real GitHub release archive has been installed through it.
- Linux systemd templates and installer rendering exist, but live
  `systemctl --user` behavior is not run or proven.
- Fresh-thread Codex app command/hook smoke is still not recorded.

Ready inputs for a Homebrew formula once a tap exists:

- package name: `arcwell`;
- current version: use the workspace package version in `Cargo.toml`;
- build command: `cargo build --release -p arcwell`;
- installed file: `target/release/arcwell` into `bin`;
- formula test: run `arcwell --help`, then use a temporary `ARCWELL_HOME` to
  run `profile set`, `backup create`, and `backup verify`;
- post-install guidance: run `arcwell service install` on macOS only when the
  user wants the worker LaunchAgent.
- render `packaging/homebrew/arcwell.rb.template` with real per-target archive
  URLs and SHA-256 values; do not publish if any `__ARCWELL_...__` placeholder
  remains.

Ready inputs for GitHub Releases once CI exists:

- archives per supported target, each containing `arcwell`, `LICENSE`, and
  install notes;
- SHA-256 checksums for every archive;
- release notes naming schema version, backup expectations, service restart
  steps, plugin reload expectations, and known blockers;
- smoke evidence from `scripts/release-readiness-smoke` and platform-specific
  service smokes.

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

## Codex Plugin Live Development

Use the generated development plugin while editing Arcwell from inside Codex:

```sh
scripts/arcwell-dev install
```

This creates `.arcwell-dev/plugins/arc` from the stable plugin,
creates `.arcwell-dev/bin/arcwell-dev`, builds `target/debug/arcwell`, and
installs `arc@arcwell-local` through the repo marketplace.

Normal inner loop:

```sh
scripts/arcwell-dev sync
```

Continuous loop:

```sh
scripts/arcwell-dev watch
```

Smoke check:

```sh
scripts/arcwell-dev smoke
```

Reload expectations:

- Rust/CLI behavior changes apply after rebuild.
- MCP implementation changes apply after Codex reconnects the MCP server; a new
  thread is the reliable path.
- Skill text can usually be reloaded with **Cmd+K -> Force Reload Skills**.
- New or removed slash commands, tools, schemas, hooks, and plugin manifest
  changes should be tested in a new Codex thread.

The generated `.arcwell-dev` directory is intentionally gitignored. The stable
plugin remains `plugins/arcwell-codex`; the dev plugin is a generated local
copy with a different plugin name and an MCP command pointed at the local debug
binary wrapper.

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

## Claude MCP Config

Claude Desktop and Claude Code should use the same installed binary shape as
the packaged Codex plugin:

```json
{
  "mcpServers": {
    "arcwell": {
      "type": "stdio",
      "command": "arcwell",
      "args": ["mcp"],
      "env": {
        "ARCWELL_HOME": "/Users/chabotc/.arcwell"
      }
    }
  }
}
```

If a GUI host cannot find `arcwell`, replace the command with the absolute path
returned by `which arcwell`. The development checkout can be inspected with:

```sh
cargo build -p arcwell
scripts/mcp-inspector
scripts/claude-mcp-smoke
```

`scripts/claude-mcp-smoke` is the repeatable local validation for Claude-shaped
stdio behavior. It does not claim that an authenticated Claude Desktop/Code UI
session has loaded Arcwell. `scripts/mcp-inspector` launches the official
`@modelcontextprotocol/inspector` package for interactive protocol inspection;
it is not a replacement for the automated smoke. Use
`scripts/claude-mcp-smoke --require-host-config` plus a real Claude
Desktop/Code session for host-level proof.

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

Implemented reliability checks:

- `arcwell service status` inspects macOS launchd and recent heartbeat state.
- `arcwell service logs` reports explicit per-stream read status.
- `arcwell doctor --strict` fails nonzero on missing service plist, corrupt
  service metadata, missing binary, stale worker heartbeat, excessive dead
  letters, stale or unverifiable backups, schema drift, and missing required
  directories.
- The worker writes a heartbeat row in SQLite.

Future reliability upgrades:

- Implement the same status/restart/log/uninstall contract for Linux systemd
  user services after a Linux user-service runner is available.
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
arcwell backup create
brew upgrade arcwell
arcwell backup verify
arcwell doctor --strict
arcwell service restart
```

The binary should preserve backward-compatible SQLite migrations. Destructive
migrations must require an explicit backup, a migration command or restore
drill, and a release note that names the old/new schema versions.

Package installers should replace the binary atomically:

1. download or build the new candidate into a staging file;
2. verify checksum or local build provenance;
3. move the staging file over the installed `arcwell` only after it is complete;
4. run `arcwell --help`, `arcwell backup verify`, and `arcwell doctor --strict`;
5. restart the worker service if installed.

Rollback expectations:

- restore the previous package version through the package manager when
  possible;
- if a migration changed durable data, restore a backup into a fresh
  `ARCWELL_HOME` first instead of guessing;
- keep the stable Codex plugin unchanged as long as it still invokes
  `arcwell` on `PATH`;
- after rollback, run `arcwell doctor --strict` and restart the worker service.

Uninstall expectations:

```sh
arcwell service uninstall
codex plugin remove arcwell-codex@arcwell-local  # when installed through Codex
```

Then remove the packaged binary through the package manager. Do not delete
`~/.arcwell` by default. If the user explicitly requests data removal, create
or verify a backup first and then remove `ARCWELL_HOME` as a separate action.

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
