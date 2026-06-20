# arcwell-memory

Personal memory service.

Current implementation is a SQLite-backed MVP:

```sh
arcwell memory add "My cat is called Ophelia" --kind fact
arcwell memory search Ophelia
arcwell memory list
arcwell memory delete <id>
```

Next step:

- Replace the simple add/search path with `mem0-rs` extraction, ADD/UPDATE/DELETE/NONE reconciliation, dream/consolidation jobs, and provenance-aware memory history.

