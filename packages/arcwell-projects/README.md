# arcwell-projects

**Status:** Partial. Manual/local project state, evidence-backed work-run
consolidation, and an explicit verified host-sync protocol exist. Live Codex/Claude thread inventory is missing because Codex and Claude host inventory/read adapters are not connected.

Project and thread meta-controller package.

Current first-pass implementation:

- Durable project registry in local SQLite.
- Alias-based project resolution with ambiguity detection.
- Follow-up resolution can use an explicit `context_project_id`.
- Channel messages can be bound to a project id.
- Channel authorization policy controls which channel subjects may bind or mutate project state.
- Authorized Telegram chats can auto-bind project-ish messages to a uniquely resolved project; unauthorized messages stay unbound unless they attempt an explicit project id, which fails closed.
- Timestamped project status snapshots can record status, summary, source,
  thread reference, confidence, created time, and whether the row came from the
  explicit host-sync protocol.
- Generic manual snapshots reject reserved live-sync source labels such as
  `codex-host` and `codex-verified-sync`; this prevents stale/manual rows from
  masquerading as live thread state.
- `project_status_get` returns a status report envelope with the project,
  latest snapshot, timestamp/source/confidence provenance, and a live-state
  availability matrix. Fresh explicit host sync can be marked available until
  its freshness window expires. Native Codex and Claude inventory/read adapters
  are still reported unavailable; ordinary thread refs are provenance only, not
  verified live handles.
- Work-memory consolidation can record project status snapshots from trace
  evidence, including validation and work-run provenance. Generated summaries
  alone cannot support consolidation.
- Channel-scoped project status reads can enforce `can_read_projects`; direct
  project id reads from untrusted channel subjects fail closed when channel
  context is supplied.
- CLI fallback for manual sync:
  - `arcwell project create <name> <summary> --alias <alias>`
  - `arcwell project list`
  - `arcwell project resolve <query>`
  - `arcwell project status-record <project-id> <status> <summary> --source manual --thread-ref <ref>`
  - `arcwell project status-sync-record <project-id> <status> <summary> --host codex --thread-id <id> --stale-after-seconds 21600`
  - `arcwell project status-get <project-id> [--channel telegram --subject telegram:chat:<id>]`

Degraded live-state capability matrix:

| Host | Live inventory | Live thread read | Manual snapshot | Current blocker |
| --- | --- | --- | --- | --- |
| Codex | Unavailable | Unavailable | Supported via CLI/MCP/plugin prompt; explicit verified sync is freshness-bounded | No stable Arcwell-owned Codex thread inventory/read API is wired into the Rust core. |
| Claude | Unavailable | Unavailable | Supported via CLI/MCP | Claude lifecycle/thread inventory hooks are unavailable or unproven. |

MCP tools:

- `project_create`
- `project_list`
- `project_resolve`
- `project_status_record`
- `project_status_sync_record`
- `project_status_get`
- `channel_record`
- `channel_list`
- `channel_authorize`
- `channel_authorizations`

Remaining work:

- Integrate with Codex thread inventory APIs when exposed; local tool discovery
  in this audit did not expose usable Codex `list_threads`/`read_thread` tools
  to Arcwell.
- Add automatic project status summaries from live thread/task state.
- Add channel context carryover for follow-up messages like "and the video project?" from real chat history.
