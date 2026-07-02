# Arcwell Agent Notes

This repo builds Arcwell: local-first assistant services exposed to Codex,
Claude, and other MCP-capable agents.

## Root Cause Rule

Do not paper over failures. If a command, hook, plugin, MCP server, worker, or
test fails, find the root cause and either fix it or document the exact blocker.
Do not call a capability done because a prompt, README, or scaffold exists.

## Validation Baseline

For meaningful Rust changes:

```sh
cargo fmt -- --check
cargo test --all --all-features
```

For Cloudflare worker changes:

```sh
cd packages/arcwell-edge-inbox/worker
npm run typecheck
npm test
```

For Codex plugin/dev-loop changes:

```sh
scripts/arcwell-dev smoke
scripts/arcwell-dev sync
```

CI (`.github/workflows/ci.yml`) runs these on every push and pull request.

## Codex Plugin Dev Loop

Arcwell has two plugin modes:

- `arcwell-codex`: stable local plugin. It calls `arcwell` on `PATH`.
- `arcwell-codex-dev`: generated development plugin. It calls this checkout's
  `target/debug/arcwell` through `.arcwell-dev/bin/arcwell-dev`.

Use the dev plugin while changing Arcwell from inside Codex. Do not edit the
Codex plugin cache directly; regenerate/sync it from the repo.

### First-Time Dev Install

From repo root:

```sh
scripts/arcwell-dev install
```

Then start a new Codex thread and select/install/use `Arcwell Dev`
(`arcwell-codex-dev@arcwell-local`) if needed.

### Normal Inner Loop

After changing Rust, plugin skills, slash commands, hooks, docs used by skills,
or MCP tool descriptions:

```sh
scripts/arcwell-dev sync
```

This builds `target/debug/arcwell`, regenerates `.arcwell-dev/plugins`, and
syncs the installed dev plugin cache when present.

Use:

```sh
scripts/arcwell-dev watch
```

for a continuous loop. It uses `fswatch` if installed, otherwise a polling
fingerprint.

### Reload Expectations

- CLI behavior changes apply immediately after rebuild.
- MCP implementation changes apply when Codex starts/reconnects the MCP server;
  a new thread is the reliable path.
- Skill text changes can usually be picked up with **Cmd+K -> Force Reload
  Skills** in the Codex app.
- New/removed slash commands, new/removed MCP tools, changed MCP schemas, hook
  changes, and plugin manifest changes should be tested in a new Codex thread.
- Hook behavior must be verified with `/memory-events` or
  `arcwell memory events --limit 20`; do not assume hook config ran just because
  the file exists.

### Smoke Checks

Run:

```sh
scripts/arcwell-dev smoke
```

This uses a disposable `ARCWELL_HOME` and mock memory provider to prove the dev
wrapper, memory capture, recall, dream, and lifecycle events work.

Use:

```sh
scripts/arcwell-dev status
```

to inspect the debug binary, generated plugin, Codex cache, and Codex CLI
availability.

## Packaging Boundaries

- The Rust binary owns durable behavior: SQLite, wiki files, memory provider,
  workers, MCP tools, HTTP, backup, costs, secrets, and adapters.
- The Codex plugin owns instructions: skills, slash commands, hooks, and MCP
  registration.
- The worker service owns unattended local jobs and should be supervised outside
  Codex.
- Cloudflare Workers own always-on internet-facing capture, not durable local
  truth.

## Documentation Discipline

Update `STATUS.md` and `TODO.md` in the same change as capability work. If a
feature is only scaffolded, say so. If live provider behavior was not tested,
say so. Keep `README.md` and package docs user-facing and honest.
