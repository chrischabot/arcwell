# Mission Control Ops Cockpit Plan

Date: 2026-06-24

Status: design and implementation plan only. No Arcwell feature is implemented
by this file.

Reference repo: https://github.com/builderz-labs/mission-control

Reference commit inspected: `d09e608`

Local inspection path: `/tmp/arcwell-reference-repos/mission-control`

## Claim Boundary

This plan can claim that Mission Control source code was inspected and that
selected task, dispatch, RBAC, audit, evaluation, and security-scan patterns
were mapped into Arcwell ops design.

This plan cannot claim that Arcwell has a Mission Control dashboard or any new
ops cockpit implemented.

## Source And Code Inspected

- `README.md`
- `src/lib/schema.sql`
- `src/lib/task-dispatch.ts`
- `src/lib/auth.ts`
- `src/lib/agent-evals.ts`
- `src/lib/security-scan.ts`
- `src/lib/mcp-audit.ts`
- `src/app/api/*` route inventory

## What Mission Control Does Well

Mission Control is a multi-agent operations dashboard. The UI claims are broad,
but several source-level ideas are directly useful:

- A task schema with agents, comments, activities, notifications,
  subscriptions, standup reports, quality reviews, gateway health logs, and
  indexes.
- Task dispatch builds prompts from ticket fields, priority, tags,
  description, and rejection feedback.
- It avoids forcing a model override unless the agent config explicitly asks
  for one.
- Deferred completion reconciliation can wait for gateway state, recover
  transcript by session/agent/task markers, and sync to outbound systems.
- Auth uses hashed sessions, constant-time comparison, dummy hash on missing
  users, progressive rehash, careful proxy auth handling, and agent-scoped API
  keys with scope/expiry/revocation.
- Agent evaluation spans output correctness, trace convergence, component tool
  reliability, and drift detection over rolling baselines.
- Security scan categorizes credentials, network, OpenClaw/runtime, and OS
  risks with weighted severity and fix-safety labels.
- MCP audit logs tool calls and can sign canonical payloads with Ed25519
  receipts.

The strongest transferable idea is not "make a dashboard." It is an ops cockpit
that turns Arcwell's existing queues, jobs, source health, channel deliveries,
costs, secrets, and research runs into one verifiable control surface.

## Arcwell-Native Shape

Working name: `arcwell ops cockpit`

Arcwell should extend the existing `ops`/health surfaces, not build a separate
web app first. The product surface should answer:

- What is running?
- What is blocked?
- What has stale cursors?
- Which deliveries failed?
- Which source adapters are unhealthy?
- Which cost/policy/secret gates are blocking work?
- What claims did agents make, and what proof exists?
- Which MCP/tool calls changed state, and can we audit them?

## Proposed Data Model

Reuse existing job/source/cost/secret/channel tables where possible. Add only
missing cross-cutting records:

- `ops_events`
  - `id`
  - `event_kind`
  - `severity`
  - `entity_kind`
  - `entity_id`
  - `message`
  - `details_json`
  - `created_at`

- `ops_quality_reviews`
  - `id`
  - `subject_kind`
  - `subject_id`
  - `claim`
  - `status`
  - `proof_packet_ref`
  - `reviewer`
  - `created_at`

- `ops_security_findings`
  - `id`
  - `category`
  - `severity`
  - `fix_safety`
  - `finding`
  - `evidence_ref`
  - `status`
  - `created_at`

- `ops_audit_receipts`
  - `id`
  - `action_kind`
  - `entity_kind`
  - `entity_id`
  - `canonical_payload_hash`
  - `signature`
  - `signing_key_ref`
  - `created_at`

- `ops_run_evals`
  - `id`
  - `run_id`
  - `eval_kind`
  - `score`
  - `finding_json`
  - `baseline_ref`
  - `created_at`

## CLI, MCP, Slash, And Ops Surfaces

CLI:

- `arcwell ops`
- `arcwell ops events`
- `arcwell ops security-scan`
- `arcwell ops audit <entity>`
- `arcwell ops eval <run-id>`
- `arcwell ops quality-review <subject>`

MCP:

- `ops_snapshot`
- `ops_events`
- `ops_security_scan`
- `ops_audit_verify`
- `ops_run_eval`

Slash/plugin:

- `/ops`
- `/arcwell-health`
- Future `/ops-security-scan`

UI:

- Add only after CLI/MCP snapshots are proven.
- Start with a dense operational table, not marketing-style cards.

## Implementation Plan

1. Define ops snapshot contract.
   - Jobs, queues, workers, source health, cursors, deliveries, costs, secrets,
     research/radar/X status, channel state.
   - Every row has status, last update, blocker, and next action.

2. Add event normalization.
   - Convert existing subsystem events into `ops_events`.
   - Preserve source subsystem IDs.

3. Add security scan.
   - Check tracked config for secrets.
   - Check secret provider health.
   - Check local key permissions.
   - Check network/listen settings.
   - Check plugin/tool exposure.
   - Label fix safety.

4. Add audit receipts.
   - Canonicalize state-changing tool/action payloads.
   - Hash first.
   - Add signing later if a local signing key exists.
   - Verification command recomputes canonical hash.

5. Add quality review hooks.
   - Tie anti-mirage proof packets to ops subjects.
   - Mark claim status separately from implementation status.

6. Add run evaluation.
   - Start with deterministic checks: completion, proof packet present,
     repeated tool failure, looping, no source citations, skipped validation.
   - Drift/baselines later.

7. Add UI only when CLI/MCP are stable.

## Anti-Mirage Traps

- A dashboard tile is not ops visibility unless backed by real subsystem state.
- A security scan with only static warnings is not a finding.
- Audit logs are not tamper-evident without canonical payload/hash/signature.
- Agent evaluation is not useful if it only scores final prose.
- Gateway health is not task success.
- A green worker process is not proof that queues are draining.

## Proof Gates

- Missing: ops surfaces remain subsystem-specific.
- Scaffold: ops snapshot command exists with partial static data.
- Partial: snapshot reads real data but lacks blockers/actions/proof.
- Local Proof: snapshot, security scan, audit hash, eval rules, and authz tests
  pass on fixtures.
- Production Data Proof: a real Arcwell home produces an ops snapshot that
  correctly identifies at least one healthy, stale, blocked, and failed/partial
  state from durable subsystem rows.
- Operational: recurring scan/eval/audit records, remediation hints, and docs
  match actual status.
- Done: ops cockpit is the trusted status source for claimed Arcwell
  subsystems, with proof packets linked for major claims.

## Severe Tests

- API/CLI route refuses unauthorized workspace/company/entity access.
- Agent-scoped key cannot read admin-only ops data.
- Secret-like values are redacted in security findings and audit payloads.
- Scanner never executes user-controlled command strings.
- Receipt verification fails after payload tampering.
- Worker healthy but queue stale; ops reports stale, not healthy.
- Source cursor stale but last fetch success exists; ops distinguishes stale
  from failed.
- Delivery generated but not sent; ops does not call it delivered.
- Run loops or repeats failed tool calls; eval flags convergence risk.
- Security scan fix marked unsafe cannot be auto-applied.
- Gateway optional/missing state is warning, not fatal, when gateway disabled.

## First Slice

Create an `ops_snapshot` contract that unifies current Arcwell durable state and
explicitly distinguishes healthy, stale, blocked, failed, partial, and unknown.
Only after that should Arcwell add a richer cockpit UI.

