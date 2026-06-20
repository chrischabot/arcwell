# Arcwell Codex Plugin

This plugin connects Codex to local `arcwell` through:

- `arcwell mcp` for live tools and resources
- skills for `$...` workflows
- slash-command prompts for quick human actions

Install from the repo root:

```sh
cargo install --path crates/arcwell-cli
codex plugin marketplace add /Users/chabotc/Projects/arcwell
codex plugin add arcwell-codex@arcwell-local
```

Start a new Codex thread after install or update.

See `../../docs/codex-plugin-commands.md` for the complete slash-command and skill catalog.
