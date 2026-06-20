# agent-memory

Personal memory service.

Current implementation is a SQLite-backed MVP:

```sh
agent memory add "My cat is called Ophelia" --kind fact
agent memory search Ophelia
agent memory list
agent memory delete <id>
```

Next step:

- Replace the simple add/search path with `mem0-rs` extraction, ADD/UPDATE/DELETE/NONE reconciliation, dream/consolidation jobs, and provenance-aware memory history.

