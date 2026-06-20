# Remaining Implementation Plan

Date: 2026-06-19

This is the executable backlog for turning the local MVP into an always-on agent services system.

## Phase 7: Worker Daemon Reliability

Status: implemented in the first pass. Remaining enhancements are manual requeue/cancel, policy-specific backoff, and process-supervisor install support.

Success criteria:

- Jobs are leased before execution.
- Failed jobs retry with bounded backoff.
- Repeated failures become `dead_lettered`.
- Stale `running` jobs can be reclaimed after lease expiry.
- CLI and MCP expose enough state to debug worker progress.
- Severe tests cover malformed jobs, failure retries, stale leases, and dead-lettering.

## Phase 8: Cloudflare Edge Inbox

Status: first pass implemented. Local drain protocol, MCP tools, and worker scaffold exist. Durable Cloudflare Queue/Durable Object deployment remains.

Success criteria:

- TypeScript Worker package accepts bounded inbound events.
- Events have source, idempotency key, received time, max age, and payload size caps.
- Local Rust service can drain, ack, nack, and dead-letter events.
- OAuth callback capture is represented as a short-lived inbox event, not durable edge memory.
- Severe tests cover replay, oversized payloads, stale events, sender validation, and queue overflow.

## Phase 9: Channel Framework And Telegram

Status: first pass implemented. Shared local channel message model and Telegram package boundary exist. Real Telegram webhook transform remains.

Success criteria:

- Shared channel envelope supports inbound/outbound messages, attachments, formatting, parse mode, sender identity, and permissions.
- Telegram package implements webhook capture at Cloudflare and local processing through MCP/daemon APIs.
- Prompt injection in channel messages is treated as user/content data unless explicitly trusted.
- Severe tests cover spoofing, formatting injection, unauthorized project switching, replay, and delivery failure.

## Phase 10: Project Meta-Controller

Status: first pass implemented. Project registry, aliases, ambiguity detection, and context follow-up resolution exist. Live Codex/Claude thread inventory remains.

Success criteria:

- Local project/thread registry tracks known projects, aliases, active work, summaries, and last-seen status.
- Channels can resolve references such as "de-porting of codex swift" and follow-ups like "and the video project?"
- Actions that create or modify projects are recorded with provenance.
- Severe tests cover ambiguous references, stale summaries, unauthorized cross-project reads, and malicious channel prompts.

## Phase 11: Librarian And Interestingness Pipeline

Status: first pass implemented. Digest candidates and librarian topic expansion exist. Model-backed synthesis, clustering, and delivery remain.

Success criteria:

- New source cards can create/update topic pages.
- Related source cards cluster into one launch/news/research item.
- Interestingness scoring produces digest candidates with evidence and suppression reasons.
- Digest delivery can target Telegram/email later without coupling to one channel.
- Severe tests cover duplicate events, contradictory sources, prompt injection, over-alerting, and stale-source synthesis.

## Phase 12: Personal Memory Pipeline

Status: first pass implemented. Candidate extraction, duplicate suppression, and exact duplicate reconcile exist. Full mem0-style hooks/dream/conflict machinery remains.

Success criteria:

- Memory extraction candidates are generated from conversations/events.
- Review, apply, reject, dream/reconcile, confidence, aging, and conflict flows are explicit.
- Personal memory remains distinct from wiki knowledge.
- Severe tests cover sensitive data, contradiction resolution, accidental write amplification, and unauthorized memory reads.

## Phase 13: Ops UI And Packaging

Status: first pass implemented. Ops snapshot, HTTP `/ops`, MCP resource/tool, and package READMEs exist. Browser UI and install automation remain.

Success criteria:

- Local UI shows health, jobs, dead letters, cursors, secrets metadata, source health, recent errors, and memory candidates.
- MCP remains the primary agent control plane; UI is for inspection and manual ops.
- Install docs work for Codex and Claude.
- Repo is ready for open-source publication.
