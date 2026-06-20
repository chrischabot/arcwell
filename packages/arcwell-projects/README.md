# arcwell-projects

Project and thread meta-controller package.

Current first-pass implementation:

- Durable project registry in local SQLite.
- Alias-based project resolution with ambiguity detection.
- Follow-up resolution can use an explicit `context_project_id`.
- Channel messages can be bound to a project id.

MCP tools:

- `project_create`
- `project_list`
- `project_resolve`
- `channel_record`
- `channel_list`

Remaining work:

- Integrate with Codex thread inventory APIs when exposed.
- Add project status summaries from live thread/task state.
- Add per-channel authorization rules for project reads and writes.
