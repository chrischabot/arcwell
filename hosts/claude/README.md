# Claude Host Adapter

Claude Desktop/Code should consume `arcwell` primarily through MCP once the MCP wrapper exists.

Current status:

- The shared Rust CLI and local store work today.
- Claude can use CLI commands manually through a local shell-capable host.
- `arcwell mcp` exposes a stdio MCP server for profile, memory, candidates, backup, cost, wiki read/search/ingest, source adapters, worker draining, cursors, and X import/live search.
- `arcwell mcp` also exposes deep-research tools for planning, provider web search, role task tracking, wiki-grounded briefs, and run history.
- SQLite local secrets can be set/listed/deleted through MCP, but values are not exposed by list resources or tools.
- Full lifecycle parity with Codex is not assumed. Hooks, skills, automations, and Codex app-server thread control are Codex-specific unless Claude exposes equivalent surfaces.

Example MCP config:

```json
{
  "mcpServers": {
    "arcwell": {
      "command": "cargo",
      "args": [
        "run",
        "-q",
        "--manifest-path",
        "/Users/chabotc/Projects/arcwell/Cargo.toml",
        "--",
        "mcp"
      ],
      "env": {
        "ARCWELL_HOME": "/Users/chabotc/.arcwell"
      }
    }
  }
}
```

Degraded-mode rule:

- Claude may read/query/profile/memory manually.
- Auto pre-turn recall and post-turn capture should be treated as unavailable until a real Claude host integration proves otherwise.
- In an isolated temp `HOME`, Claude accepted the MCP server definition but could not health-check it because the temp profile was unauthenticated. Use the official MCP Inspector or an authenticated Claude profile for connection validation.
