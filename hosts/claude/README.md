# Claude Host Adapter

Claude Desktop/Code should consume `arcwell` through the same stdio MCP server
used by other MCP-capable hosts.

Current status:

- The shared Rust CLI and local store work today.
- `arcwell mcp` exposes a stdio MCP server for profile, memory, candidates,
  backup, cost, wiki read/search/ingest, source adapters, worker draining,
  cursors, X import/live search, projects, work runs, procedures, operations,
  and deep-research tools.
- `scripts/claude-mcp-smoke` validates the local Claude-style stdio boundary by
  launching `arcwell mcp`, listing tools/resources, reading resources, calling
  representative tools, and hitting malformed/unsupported/adversarial requests.
- SQLite local secrets can be set/listed/deleted through MCP, but values are
  not exposed by list resources or tools.
- Authenticated Claude Desktop/Code UI validation is not claimed in this audit.
  The local macOS Claude Desktop config at
  `~/Library/Application Support/Claude/claude_desktop_config.json` did not
  define `mcpServers.arcwell` when the smoke was run.
- Full lifecycle parity with Codex is not assumed. Hooks, skills, automations,
  and Codex app-server thread control are Codex-specific unless Claude exposes
  equivalent surfaces.

## Local Validation

Run the process-level smoke after building or installing `arcwell`:

```sh
cargo build -p arcwell
scripts/claude-mcp-smoke
```

The smoke uses disposable homes. It verifies:

- `initialize`, `tools/list`, and `resources/list`;
- expected tools including `profile_list`, `memory_search`, `wiki_search`,
  `ops_snapshot`, and `project_status_get`;
- expected resources including `arcwell://profile`, `arcwell://wiki`, and
  `arcwell://ops`;
- malformed JSON-RPC frames return parse errors;
- unsupported MCP methods and unknown resource URIs return explicit errors;
- a bounded large profile/resource response round-trips without truncation or
  runaway output amplification;
- a huge missing `wiki://page/...` read returns explicit JSON `null`;
- bad `ARCWELL_HOME` returns an MCP error instead of silently passing;
- missing `ARCWELL_HOME` falls back to `HOME/.arcwell`;
- Claude live thread/lifecycle state is reported unavailable while manual
  project snapshots remain supported.

To make the smoke fail when the local Claude Desktop config is missing or points
at the wrong profile:

```sh
scripts/claude-mcp-smoke --require-host-config
```

This checks the config file shape only. It still does not prove that an
authenticated Claude UI session listed or called Arcwell tools.

## Claude Desktop Setup

Claude Desktop reads local MCP server definitions from
`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS and
`%APPDATA%\Claude\claude_desktop_config.json` on Windows. Use an installed
`arcwell` binary or an absolute path returned by `which arcwell`; a GUI app may
not inherit your interactive shell `PATH`.

Example MCP config:

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

After editing, fully quit and restart Claude Desktop. If the Arcwell server
does not appear, first run `scripts/claude-mcp-smoke --require-host-config`,
then run the same command from the config manually:

```sh
ARCWELL_HOME=/Users/chabotc/.arcwell arcwell mcp
```

That command should wait for JSON-RPC on stdin; press `Ctrl-C` to exit.

## Claude Code Setup

Claude Code can add stdio MCP servers from JSON config:

```sh
claude mcp add-json arcwell \
  '{"type":"stdio","command":"arcwell","args":["mcp"],"env":{"ARCWELL_HOME":"/Users/chabotc/.arcwell"}}' \
  --scope user
claude mcp get arcwell
```

If `arcwell` is not on the environment Claude Code uses, replace `"arcwell"`
with an absolute binary path.

## MCP Inspector

For interactive protocol inspection:

```sh
scripts/mcp-inspector
```

Use the Inspector to check capability negotiation, tool schemas, resource
listing, representative tool calls, and error handling. The repeatable
`scripts/claude-mcp-smoke` helper is the automated local regression check.
To preflight whether the official Inspector package is reachable without
opening an interactive session, run:

```sh
scripts/mcp-inspector --check-only
```

## Manual Claude Use

Useful requests from Claude once the MCP server is connected:

- "Use Arcwell to list my profile items."
- "Use Arcwell to set profile key `work.preference` to `<value>`."
- "Use Arcwell to search memory for `<query>`."
- "Use Arcwell to search the wiki for `<query>` and read the most relevant
  page."
- "Use Arcwell ops snapshot and summarize any warnings."
- "Use Arcwell to record a manual project status snapshot."

Degraded-mode rule:

- Claude may read/query/profile/memory manually.
- Auto pre-turn recall and post-turn capture should be treated as unavailable until a real Claude host integration proves otherwise.
- Work-run/procedure/project lifecycle capture is manual unless a future Claude
  hook or explicit sync protocol is implemented and live-smoked.
- Unavailable hooks or host integrations must be reported as unavailable. Do
  not treat a local MCP process smoke as proof of Claude UI hook behavior.

Project/thread state capability matrix:

| Capability | Claude status | Arcwell behavior |
| --- | --- | --- |
| List live Claude conversations/threads | Unavailable/unproven | `project_status_get` reports live state unavailable. |
| Read a live Claude thread for status | Unavailable/unproven | Thread refs are treated as unverified provenance only. |
| Record a manual project snapshot | Available through MCP/CLI | Snapshot includes timestamp, source, confidence, and provenance. |
| Consolidate from work-run trace evidence | Available when traces are recorded manually | Consolidation can write a project status snapshot but does not claim live host state. |
| Automatic lifecycle hooks | Unavailable/unproven | Use manual work-run/status commands until a real host integration proves hooks. |
