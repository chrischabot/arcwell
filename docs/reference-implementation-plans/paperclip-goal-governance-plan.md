# Paperclip Goal Governance Plan

Date: 2026-06-24

Status: design and implementation plan only. No Arcwell feature is implemented
by this file.

Reference repo: https://github.com/paperclipai/paperclip

Reference commit inspected: `ef37203`

Local inspection path: `/tmp/arcwell-reference-repos/paperclip`

## Claim Boundary

This plan can claim that Paperclip source code was inspected and that its goal,
approval, budget, finance, portability, heartbeat, workspace-sync, watchdog,
and secret patterns were mapped to Arcwell design options.

This plan cannot claim that Arcwell has Paperclip-style companies, budgets,
approval workflows, portable agent packages, or task watchdogs.

## Source And Code Inspected

- `README.md`
- `doc/DATABASE.md`
- `doc/TASK-WATCHDOG.md`
- `docs/companies/companies-spec.md`
- `server/src/services/goals.ts`
- `server/src/services/approvals.ts`
- `server/src/services/finance.ts`
- `server/src/services/budgets.ts`
- `server/src/services/access.ts`
- `server/src/services/company-portability.ts`
- `server/src/services/task-watchdog-scope.ts`
- `packages/adapter-utils/src/billing.ts`
- `packages/adapter-utils/src/command-managed-runtime.ts`
- `packages/adapter-utils/src/runtime-progress.ts`
- `packages/adapter-utils/src/git-workspace-sync.ts`
- `packages/adapter-utils/src/session-compaction.ts`
- `packages/adapter-utils/src/sandbox-managed-runtime.ts`
- `cli/src/commands/client/goal.ts`
- `cli/src/commands/client/approval.ts`
- `cli/src/commands/client/cost.ts`

## What Paperclip Does Well

Paperclip is a broad control plane for teams of agents. The "company" metaphor
is not the part Arcwell should copy first. The strongest transferable features
are the governance primitives underneath it:

- Goals as durable, hierarchical, owner-scoped objects.
- Approvals with pending, approved, rejected, revision-requested, and resubmit
  states.
- Approval decisions are idempotent and only certain statuses are resolvable.
- Approval comments are redacted according to instance settings.
- Budget policies by company/agent/project scope, with monthly/lifetime
  windows, soft warnings, hard stops, incidents, and approval rows.
- Cost/finance events are linked to company/agent/project/goal/issue/run/cost
  rows with company-boundary checks.
- Secret storage has metadata, versions, bindings, access events, encrypted
  local provider, strict mode for inline sensitive env values, and permission
  health checks.
- Markdown-first agent company packages preserve source refs, license,
  attribution, secrets as declarations, and vendor-specific sidecars.
- Safe import rejects process/http adapters, setup/cleanup commands, unsafe
  project workspace policies, and non-schedule triggers in restricted mode.
- Task watchdogs verify stopped issue trees, compute stop fingerprints, create
  exactly one review task, and enforce mutation scope server-side.
- Workspace sync uses shallow git refs, temporary refs, bundles, bounded remote
  transfer, progress reporting, chunking, and merge/retry handling.
- Session compaction policies are adapter-aware instead of one-size-fits-all.

The most Arcwell-relevant "oh yes" feature is the watchdog: a second-pass agent
that treats stopped work as a claim needing evidence and can restore a live path
without granting itself approval or cross-scope authority.

## Arcwell-Native Shape

Arcwell should adapt Paperclip into project/run governance, not an org chart.

Working name: `arcwell governance`

Core Arcwell concepts:

- Goals attach to projects, research runs, radar profiles, memory/wiki work,
  channel workflows, and Codex tasks.
- Approvals gate publish/send/delete/spend/schedule/import/export operations.
- Budgets and cost policy already exist; strengthen them with incidents,
  hard-stop behavior, and approval-backed overrides.
- Portable packages should be Arcwell project/profile/wiki/source-card bundles,
  not "companies."
- Watchdogs should detect fake-stopped Arcwell work trees and request evidence
  or reopen a live path.

## Proposed Data Model

- `governance_goals`
  - `id`
  - `scope_kind`
  - `scope_id`
  - `parent_id`
  - `title`
  - `description`
  - `status`
  - `owner_ref`
  - `created_at`
  - `updated_at`

- `governance_approvals`
  - `id`
  - `scope_kind`
  - `scope_id`
  - `approval_type`
  - `status`
  - `requested_by`
  - `decided_by`
  - `payload_hash`
  - `payload_json`
  - `decision_note`
  - `created_at`
  - `decided_at`

- `governance_approval_comments`
  - `id`
  - `approval_id`
  - `author_ref`
  - `body_redacted`
  - `created_at`

- `budget_incidents`
  - `id`
  - `policy_id`
  - `scope_kind`
  - `scope_id`
  - `threshold_type`
  - `amount_limit`
  - `amount_observed`
  - `status`
  - `approval_id`
  - `created_at`
  - `resolved_at`

- `finance_events`
  - `id`
  - `scope_kind`
  - `scope_id`
  - `goal_id`
  - `run_id`
  - `cost_event_id`
  - `biller`
  - `event_kind`
  - `direction`
  - `amount_cents`
  - `estimated`
  - `occurred_at`

- `governance_watchdogs`
  - `id`
  - `watched_scope_kind`
  - `watched_scope_id`
  - `watchdog_agent_ref`
  - `instructions`
  - `status`
  - `last_reviewed_fingerprint`
  - `review_task_ref`
  - `created_at`

- `portable_bundles`
  - `id`
  - `bundle_kind`
  - `manifest_hash`
  - `source_ref_json`
  - `license_summary`
  - `status`
  - `created_at`

## CLI, MCP, Slash, And Ops Surfaces

CLI:

- `arcwell goal list|create|update|delete`
- `arcwell approval list|get|request|approve|reject|request-revision|resubmit`
- `arcwell budget incidents`
- `arcwell finance summary`
- `arcwell watchdog set|status|scan`
- `arcwell portable export|preview|import`

MCP:

- `goals_list`
- `approval_request`
- `approval_decide`
- `budget_incidents`
- `watchdog_scan`
- `portable_export_preview`

Slash/plugin:

- `/cost-check`
- `/cost-summary`
- Future `/approval-list`
- Future `/watchdog-scan`

Ops:

- Open approvals, budget incidents, stopped-work watchdog findings, portable
  import warnings, finance rollups, secret provider health.

## Implementation Plan

1. Goals.
   - Add durable goals scoped to Arcwell projects/runs.
   - Link goals to existing project status and proof packets.
   - Avoid replacing TODO/status files in the first slice.

2. Approvals.
   - Introduce generic approval rows with payload hash.
   - Gate risky operations: send/publish/delete/import/export/spend override.
   - Require reapproval if payload hash changes.

3. Budget incidents.
   - Extend existing Arcwell cost policy with incidents.
   - Soft threshold creates warning event.
   - Hard threshold pauses/cancels eligible work and creates approval request.

4. Finance events.
   - Link cost events to projects/goals/runs.
   - Preserve estimated vs actual.
   - Company terminology should become Arcwell scope terminology.

5. Watchdogs.
   - Define watched scopes: project tree, research run, radar run, controller
     session, or task list.
   - Compute stop fingerprint from leaves, statuses, blockers, validations, and
     watchdog config.
   - Create or reuse one review task.
   - Enforce scope at mutation route/command layer.

6. Portable bundles.
   - Start with export preview of Arcwell project/wiki/source-card profile.
   - Markdown is canonical.
   - Vendor-specific Arcwell fidelity lives in `.arcwell.yaml`.
   - Preserve source refs, license, attribution, hashes, and secret
     declarations.

7. Runtime/workspace sync patterns.
   - Borrow bounded transfer/progress ideas only if Arcwell adds remote runtime
     workspace sync.
   - Keep local-first current behavior as the default.

## Anti-Mirage Traps

- A goal row is not progress tracking.
- An approval prompt is not an approval unless it binds an immutable payload.
- Budget policy is not enforcement unless hard-stop work is paused/cancelled.
- Cost summary is not finance unless events are linked and scoped.
- Portable export is unsafe if it includes secret values or local paths.
- Watchdog is dangerous unless server-side scope enforcement exists.
- A stopped-work detector is not proof that the stop was valid.

## Proof Gates

- Missing: no governance rows or approval gates.
- Scaffold: goal/approval commands exist with local fixtures.
- Partial: approvals exist but do not gate real operations.
- Local Proof: payload hash, status transitions, reapproval, budget incidents,
  hard-stop pause, secret redaction, portable preview, and watchdog scope tests
  pass.
- Production Data Proof: an authorized Arcwell operation is blocked by an
  approval or budget incident, then proceeds only after the correct decision,
  with durable audit/proof.
- Operational: ops shows open approvals/incidents/watchdogs, blocked work,
  resolved work, and remaining risks.
- Done: risky operations consistently enforce approvals/cost policy, portable
  bundles are reproducible and secret-free, and watchdogs restore or validate
  stopped work without scope escape.

## Severe Tests

- Approval approve/reject is idempotent.
- Approved payload is changed before execution; operation is rejected.
- Revision-requested approval can be resubmitted and then approved.
- Agent/user without permission cannot approve.
- Approval comments redact current-user/secrets according to policy.
- Budget observed spend crosses soft threshold; incident created once.
- Hard threshold pauses/cancels work and creates override approval.
- Budget lowered below observed spend triggers hard stop.
- Finance event linked to a run in another scope is rejected.
- Portable export contains no secret values, local absolute paths, or transient
  database IDs.
- Pinned source hash mismatch blocks import.
- Safe import rejects setup command, process/http adapter, unsafe trigger, and
  workspace policy.
- Watchdog cannot mutate outside watched subtree/scope.
- Watchdog cannot approve spend or security-sensitive interaction.
- Stop fingerprint suppresses duplicate wakes and changes when leaves/config
  change.
- Workspace transfer progress reports failure instead of dangling percentage.

## First Slice

Implement generic approval rows with payload hashing and use them to gate one
existing Arcwell risky operation, preferably an outbound channel send or future
social publish. The watchdog is the highest-upside follow-up, but it should not
land until scoped mutation enforcement is real.

