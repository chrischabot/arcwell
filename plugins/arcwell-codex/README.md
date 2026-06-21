# Arcwell Codex Plugin

**Status:** Partial. The package, prompts, skills, MCP config, and dev sync loop
exist; fresh-thread Codex command/hook smoke is still missing.

This plugin connects Codex to local `arcwell` through:

- `arcwell mcp` for live tools and resources
- skills for `$...` workflows
- slash-command prompts for quick human actions
- generated command-skill shims for Codex app slash-picker visibility in the dev plugin

Install from the repo root:

```sh
cargo install --path crates/arcwell-cli
codex plugin marketplace add /Users/chabotc/Projects/arcwell
codex plugin add arcwell-codex@arcwell-local
```

Start a new Codex thread after install or update.

See `../../docs/codex-plugin-commands.md` for the complete slash-command and
skill catalog, including the fresh-thread manual smoke matrix that still needs
recorded in-app evidence.
