# NanoClaw Controller Isolation Plan

Date: 2026-06-24

Status: design and implementation plan only. No Arcwell feature is implemented
by this file.

Reference repo: https://github.com/nanocoai/nanoclaw

Reference commit inspected: `add6145`

Local inspection path: `/tmp/arcwell-reference-repos/nanoclaw`

## Claim Boundary

This plan can claim that NanoClaw source code was inspected and that its
controller isolation, routing, delivery, and runner patterns were mapped to an
Arcwell design.

This plan cannot claim that Arcwell has NanoClaw-style container isolation,
two-ledger session exchange, or multi-agent chat routing implemented.

## Source And Code Inspected

- `README.md`
- `docs/architecture.md`
- `docs/isolation-model.md`
- `src/router.ts`
- `src/delivery.ts`
- `src/session-manager.ts`
- `src/container-runner.ts`
- `src/db/session-db.ts`
- `src/host-sweep.ts`
- `src/channels/adapter.ts`
- `container/agent-runner/src/index.ts`
- `container/agent-runner/src/poll-loop.ts`

## What NanoClaw Does Well

NanoClaw is one of the strongest direct references for Arcwell controller work.
It has a small, explicit host/container split:

```text
messaging apps -> host router -> inbound.db -> container agent ->
outbound.db -> delivery -> messaging apps
```

The most important design decision is two SQLite files per session:

- `inbound.db` is host-owned.
- `outbound.db` is container-owned.
- The host tracks delivery receipts in inbound state.
- Each database has one writer.
- Cross-mount behavior relies on simple file ownership and short write windows.

Other strong source-level patterns:

- Entity model separates agent groups, messaging groups, and wirings.
- Session modes support shared, agent-shared, and per-thread isolation.
- Channel adapters know platform IDs/thread IDs but not agent/session IDs.
- Router only auto-creates messaging groups on mention/DM, not passive chatter.
- Access gate, sender-scope gate, message interceptor, and channel request gate
  are explicit hooks.
- Dropped messages are audited.
- Engagement modes include pattern, mention, and mention-sticky.
- Ignored-message policy can accumulate silent context but does not accumulate
  messages rejected by access/scope gates.
- Host-side command gate denies admin commands before agent execution.
- Delivery polls active sessions, prevents concurrent duplicate delivery with
  `inflightDeliveries`, checks destination ACLs, retries, and records failures.
- Attachments are extracted with basename, symlink checks, realpath containment,
  and exclusive create.
- Runner wake deduplication avoids spawning duplicate containers.
- Host sweep handles stale acks, due wakeups, backoff, heartbeat ceilings, and
  retry limits.

## Arcwell-Native Shape

Arcwell should borrow the controller exchange pattern, not necessarily
NanoClaw's container runtime. The core Arcwell problem is routing durable,
auditable work between channels and long-lived agents without letting any one
transport own controller truth.

Working name: `arcwell controller exchange`

Key adaptation:

- Host/controller owns routing, auth, channel policy, secrets, and delivery.
- Agent runner owns reasoning/execution.
- Exchange ledger separates inbound work, outbound replies, wakeups, acks, and
  delivery receipts.
- Channels remain platform-level adapters.
- Arcwell source cards/wiki/jobs receive projections after durable exchange
  writes, not as the primary channel truth.

## Proposed Data Model

Arcwell can implement this as SQLite tables first. Two physical DB files are an
option for high isolation, but the first Arcwell slice can enforce one-writer
semantics in the existing local store.

- `controller_agent_groups`
  - `id`
  - `name`
  - `runner_kind`
  - `isolation_mode`
  - `status`
  - `created_at`

- `controller_messaging_groups`
  - `id`
  - `channel_kind`
  - `platform_group_ref`
  - `thread_ref`
  - `status`
  - `created_at`

- `controller_wirings`
  - `id`
  - `agent_group_id`
  - `messaging_group_id`
  - `engage_mode`
  - `ignored_message_policy`
  - `created_at`

- `controller_sessions`
  - `id`
  - `agent_group_id`
  - `messaging_group_id`
  - `session_mode`
  - `status`
  - `last_heartbeat_at`
  - `created_at`

- `controller_inbound`
  - `id`
  - `session_id`
  - `channel_message_id`
  - `sender_ref`
  - `body`
  - `attachments_json`
  - `trigger`
  - `access_decision`
  - `seq`
  - `created_at`

- `controller_outbound`
  - `id`
  - `session_id`
  - `body`
  - `destination_json`
  - `seq`
  - `created_at`

- `controller_delivery_receipts`
  - `outbound_id`
  - `destination_ref`
  - `status`
  - `attempt_count`
  - `provider_message_id`
  - `error`
  - `updated_at`

- `controller_wakeups`
  - `id`
  - `session_id`
  - `wake_reason`
  - `idempotency_key`
  - `status`
  - `attempt_count`
  - `next_attempt_at`

- `controller_dropped_messages`
  - `id`
  - `channel_kind`
  - `platform_message_id`
  - `reason`
  - `policy_json`
  - `created_at`

## CLI, MCP, Slash, And Ops Surfaces

CLI:

- `arcwell controller groups`
- `arcwell controller wire`
- `arcwell controller inbound <session-id>`
- `arcwell controller outbound <session-id>`
- `arcwell controller wake <session-id>`
- `arcwell controller sweep`
- `arcwell controller deliveries <session-id>`

MCP:

- `controller_list_sessions`
- `controller_read_exchange`
- `controller_enqueue_inbound`
- `controller_delivery_status`
- `controller_wake_status`

Slash/plugin:

- `/channel-record`
- `/channel-deliveries`
- `/edge-events`
- Future `/controller-sessions`

Ops:

- Pending inbound, pending outbound, stuck acks, wake retries, delivery
  failures, dropped-message reasons, ACL denials, runner health.

## Implementation Plan

1. Define channel adapter contract.
   - Normalize platform IDs and thread IDs.
   - Keep adapters ignorant of agent/session IDs.
   - Add capability flag for thread support.

2. Implement host router.
   - Mention/DM creation rules.
   - Access gate and sender-scope gate.
   - Dropped-message audit.
   - Engage modes.
   - Accumulate-only context separate from trigger messages.

3. Implement exchange ledger.
   - Inbound append.
   - Outbound append.
   - Delivery receipts.
   - Sequence invariants.
   - Optional future split into inbound/outbound physical DBs.

4. Implement wake dedup and runner adapter.
   - `wakePromises`-style dedup.
   - Runner failure leaves inbound pending.
   - Secrets and mounts are resolved host-side.

5. Implement delivery loop.
   - Destination ACL.
   - Idempotency.
   - Retry and failure state.
   - Best-effort outbox cleanup never causes duplicate delivery.

6. Implement host sweep.
   - Stale acks.
   - Due messages.
   - Heartbeat ceiling.
   - Backoff/max attempts.
   - UTC timestamp parsing.

7. Add source-card/wiki projections.
   - Only after ledger writes are durable.
   - Projections are derived views, not the controller source of truth.

## Anti-Mirage Traps

- A channel message table is not a controller.
- A worker job is not delivery unless destination receipts exist.
- A runner spawn is not proof that inbound was processed.
- Accumulated ignored messages must not include access-denied messages.
- Container isolation cannot be claimed without actual runtime proof.
- Delivery success must mean provider/platform accepted the message.

## Proof Gates

- Missing: no exchange ledger.
- Scaffold: tables and mock router exist.
- Partial: inbound/outbound write locally but no runner/delivery proof.
- Local Proof: routing, gates, dropped audit, wake dedup, delivery idempotency,
  attachment path safety, ACLs, and sweep tests pass.
- Production Data Proof: an authorized real channel records inbound, wakes a
  runner, writes outbound, delivers once, and records provider receipt.
- Operational: stale ack, retry, disabled channel, runner failure, delivery
  failure, and ops views are proven.
- Done: all claimed channel/runner combinations satisfy exchange, wake,
  delivery, projection, and recovery proof.

## Severe Tests

- Plain group chatter does not create a messaging group.
- Mention/DM creates or attaches the correct group exactly once.
- Access-denied message is dropped and audited, not accumulated.
- Trigger `0`/accumulate-only context does not wake the agent.
- Duplicate wake requests coalesce.
- Concurrent delivery loop sends outbound once.
- Destination outside ACL is rejected.
- Attachment filename traversal and symlink escape are blocked.
- Runner spawn failure leaves inbound pending with retry state.
- Stale ack is cleared and retried according to backoff.
- Admin command is denied host-side before agent sees it.
- Corrupt outbound read marks session unhealthy without losing inbound.
- Provider delivery succeeds but cleanup fails; retry does not duplicate send.

## First Slice

Build the controller exchange ledger and router against one existing Arcwell
channel in mock/local mode. Do not claim container isolation. The first "oh yes"
deliverable is durable inbound -> wake -> outbound -> delivery receipt with
audit and ops visibility.

