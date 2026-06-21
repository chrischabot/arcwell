# Arcwell Memory Integration

Arcwell Memory is the local-first personal memory layer for Arcwell. It is
separate from the source-backed LLM Wiki and from explicit profile/preferences.

## Design References

Current implementation follows the shape recommended by:

- Codex MCP servers, plugin-bundled MCP, and plugin packaging guidance:
  https://developers.openai.com/codex/mcp.md
- Codex hooks for `SessionStart`, `UserPromptSubmit`, `PreCompact`, and `Stop`:
  https://developers.openai.com/codex/hooks.md
- Codex built-in Memories cautions: local generated state, secret redaction,
  background generation, and "helpful recall" rather than mandatory rules:
  https://developers.openai.com/codex/memories.md
- Mem0's Codex integration model: MCP server plus lifecycle hooks and a skill:
  https://docs.mem0.ai/integrations/codex
- Mem0 MCP tool shape for add/search/get/update/delete/history/events:
  https://docs.mem0.ai/platform/mem0-mcp
- Mem0 agent-memory guidance: store, retrieve, update, forget, and balance
  token/cost/latency/safety:
  https://mem0.ai/blog/how-to-add-memory-to-autonomous-ai-agents
- OpenMemory's local shared MCP memory model:
  https://mem0.ai/blog/introducing-openmemory-mcp

## Boundaries

- `arcwell-profile`: explicit operating preferences and durable profile facts.
- `arcwell-memory`: compact personal facts and learned preferences.
- `arcwell-llm-wiki`: cited external knowledge, source cards, research, and
  Markdown corpus pages.

Memory is allowed to help personalize behavior. It is not a source of truth for
current external facts, legal/medical claims, or cited research.

## Implemented Surfaces

### CLI

```sh
arcwell memory mem0-add "My cat is called Ophelia" --user-id chris
arcwell memory mem0-search Ophelia --user-id chris
arcwell memory mem0-update <memory-id> "My cat is called Ophelia Blue"
arcwell memory mem0-delete <memory-id>
arcwell memory mem0-history <memory-id>
arcwell memory mem0-forget-user --user-id chris

arcwell memory recall "personal preferences for this task"
arcwell memory capture "My cat is called Ophelia." --source manual-note
arcwell memory events --limit 20
arcwell memory decisions --limit 20
arcwell memory tombstones --limit 20
arcwell memory eval-corpus
```

Local hook and eval scaffolds:

```sh
scripts/codex-hook-smoke --arcwell-bin target/debug/arcwell
scripts/memory-model-eval-gate --arcwell-bin target/debug/arcwell
```

### MCP Tools

- `mem0_add`
- `mem0_search`
- `mem0_update`
- `mem0_delete`
- `mem0_history`
- `mem0_forget_user`
- `memory_recall_context`
- `memory_capture`
- `memory_lifecycle_events`
- `memory_extract_candidates`
- `candidate_list`
- `candidate_apply`
- `candidate_reject`

### MCP Resources

- `arcwell://memory`
- `arcwell://memory-events`
- `arcwell://ops`

`arcwell://ops` includes recent memory lifecycle events so agents and humans can
check whether recall/capture actually ran.

### Codex Plugin Hooks

The plugin includes `plugins/arcwell-codex/hooks/hooks.json`:

- `SessionStart`: recall stable personal preferences and project context.
- `UserPromptSubmit`: recall context for the current prompt.
- `PreCompact`: capture reviewable memory candidates before compaction.
- `Stop`: capture reviewable memory candidates after a response.

Hook capture defaults to review mode. Non-sensitive auto-apply is only enabled
when `ARCWELL_MEMORY_HOOK_AUTO_APPLY=1` or a command explicitly opts in.
`ARCWELL_MEMORY_HOOK_INFER=1` records that inference was requested, but capture
does not directly write raw provider-inferred text. Model-backed capture quality
is unclaimed until an explicit provider/cost-gated eval with a reviewed
candidate oracle exists.

`scripts/codex-hook-smoke` proves the hook config and commands against a
disposable local process. Live Codex hook execution still needs an end-to-end
fresh-thread plugin smoke test; this repository should not claim the installed
host has accepted and run hooks until that smoke is recorded.

## Candidate Lifecycle

Candidates now carry:

- operation: `ADD`, `UPDATE`, `DELETE`, or `NONE`
- target memory id when updating/deleting
- user id
- sensitivity
- source reference
- metadata
- applied result
- applied/rejected audit state

Applying a memory candidate uses Arcwell Memory provider operations:

- `ADD` calls `mem0_add`
- `UPDATE` calls `mem0_update`
- `DELETE` calls `mem0_delete`
- `NONE` records a no-op

Candidates include extractor confidence, reason metadata, matched-memory
context, and whether review is required. Sensitive facts and non-ADD operations
stay pending for review in capture flows even when non-sensitive auto-apply is
enabled. This keeps contradictory identity/preference updates reviewable instead
of silently replacing active memory.

The decision ledger records ADD/UPDATE/DELETE/NONE decisions with confidence,
reasoning, source reference, user scope, and candidate/memory ids where
available. Use `arcwell memory decisions` or `/ops/ui` to inspect it. The
deterministic personal-memory eval corpus is available with
`arcwell memory eval-corpus`; it includes durable facts/preferences, sensitive
medical/secret review cases, prompt-injection-as-data, and false-positive cases
for task-local implementation prose.

## Dream/Reconcile

`memory_dream_reconcile` now reconciles the active stores instead of only the
legacy compatibility table:

- exact duplicate Arcwell Memory provider entries are deleted automatically
- exact duplicate compatibility memories are deleted automatically
- compatibility rows that exactly duplicate provider memories are deleted
- same-subject conflicting provider memories create reviewable delete
  candidates rather than silently choosing truth
- every run records a `dream_reconcile` lifecycle event

Conflict candidates keep the newer/provider-kept memory in metadata so the user
or agent can inspect what would be removed before applying it.

## Forget Cascade

`mem0_forget_user` now performs an active-store cascade:

- deletes provider memories for the user
- purges provider history rows for those memory ids
- deletes memory candidates scoped to the user or those memory ids
- deletes legacy unscoped candidates when forgetting the default local user
- deletes compatibility memories scoped to the user
- deletes legacy unscoped compatibility memories when forgetting the default
  local user
- deletes prior memory lifecycle events for the user
- deletes user-scoped memory decision-ledger observations so forgotten facts do
  not survive through ops/audit views
- records one new `forget` lifecycle event with counts only
- records one tombstone with a hashed user id, deletion counts, and the explicit
  policy that active stores were purged while historical backups were not
  rewritten by forget

## Still Not Complete

- Live model-backed extraction quality is not proven; current eval coverage is
  deterministic and local. `scripts/memory-model-eval-gate` blocks live quality
  claims behind explicit provider/cost gates and currently reports the model
  candidate oracle as missing.
- Model-backed dream synthesis, confidence aging, stale preference review, and
  procedural-memory creation are not complete.
- Historical backup snapshots are not rewritten by forget. Forget now writes an
  explicit tombstone, but privacy erasure across retained backup snapshots
  remains unclaimed until backup retention/encryption/rotation is implemented.
- Live Codex and Claude host behavior is not proven in this audit. Local hook
  commands are covered by `scripts/codex-hook-smoke` and
  `scripts/arcwell-dev smoke`; fresh-thread Codex and Claude Desktop/Code
  validation remain live host smokes.
- There is no human UI for reviewing memory candidates or lifecycle events yet.
