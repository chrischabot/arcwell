# Arcwell Implementation Plan

Last updated: 2026-06-21

This is the execution plan derived from `STATUS.md`. It is a checklist, not a
vision document. Do not mark an item complete because a command, README, prompt,
or scaffold exists. Mark it complete only when implementation, tests, severe
review, docs, and status updates all agree.

`PLAN.md` is the strategic roadmap for the five largest missing pieces:
live mobile loop, work-memory graph, procedural learning, policy enforcement,
and ops UX. This file is the executable checklist for those plans plus the
existing trust blockers. `TODO_DAG.md` maps this checklist into dependency
waves, subagent ownership, and severe/adversarial review gates.

## Current State Snapshot

- Local Rust CLI/core behavior is implemented for profile, Arcwell Memory,
  wiki/source cards, research, X, worker, backup, costs, secrets, cursors,
  projects, channels, Telegram, edge inbox, MCP, and HTTP.
- Rust tests currently pass across the workspace, including severe tests for
  backup restore, strict doctor, cost blocking, memory lifecycle, edge drain,
  Telegram drain/send/retry, project authorization, and prompt-injection-as-data
  handling.
- The Cloudflare worker package typechecks and has local Node in-memory tests
  for auth, idempotency, leasing, ack/nack/dead-letter, expiry, rate limits, and
  Telegram webhook normalization.
- Still unproven: live Telegram bot/webhook, fresh-thread Codex hook/command
  behavior, Claude Desktop/Code MCP behavior, live provider API flows,
  historical backup erasure for forgotten memory, and production HTTP
  auth/error handling.
- Missing or not product-real: work-memory graph, procedural learning,
  generalized policy engine, product-grade ops UI with controls, automatic
  proactive alert routing, live Codex/Claude thread inventory, and model-backed
  research synthesis.

## Execution Rules

- [ ] Every implementation PR/change updates this file and `STATUS.md`.
- [ ] Every meaningful feature names its behavioral claim before coding.
- [ ] Every feature has at least one test that tries to refute that claim.
- [ ] Every P0/P1 feature has a severe/adversarial test gate before completion.
- [ ] Every external integration has one local/mock test and one documented live
      smoke test.
- [ ] Every agent-facing command or skill must fail honestly when the capability
      is partial, scaffolded, or unavailable.
- [ ] Do not silently convert "manual foreground command works" into "service is
      installed and reliable."
- [ ] Do not call generated summaries "research" or "memory" unless source,
      provenance, and uncertainty are inspectable.

## Done Matrix

| Done requirement | Applies to | Required evidence |
| --- | --- | --- |
| Code implemented | All tasks | Main path and expected failure paths exist in code, not only docs/prompts. |
| Unit tests | All core logic | `cargo test --all --all-features` or package-specific equivalent passes. |
| Integration tests | MCP, CLI, worker, Cloudflare, Telegram, X, research | Real boundary exercised through CLI/MCP/HTTP/worker, not only private helpers. |
| Severe tests | P0/P1 and trust-sensitive P2 | Malicious, malformed, replay, auth, size, timeout, or crash/restart tests as relevant. |
| Live smoke | External APIs and deployed services | Documented command, environment, result, and residual risk in `docs/live-e2e-testing.md`. |
| Ops visibility | Workers, queues, providers, sync | `/ops`, MCP resource, or ops UI shows state, errors, age, and next action. |
| Docs | All user-visible work | README/package docs, `STATUS.md`, and this file reflect exact state. |
| No overclaim | All agent-facing surfaces | Slash commands and skills say what is missing instead of implying completion; `scripts/verify-codex-plugin-docs` checks the Codex plugin/docs slice. |

## P0 Trust Blockers

### 1. Backup Restore And Recovery

- [x] Implement restore command and library API.
- [x] Add `arcwell backup restore --from <backup-id-or-path>`.
- [x] Require an explicit safety flag or interactive confirmation before
      replacing an existing home directory.
- [x] Restore SQLite DB, wiki pages, source-card artifacts, manifest, and
      metadata with checksum verification.
- [x] Support restore into a fresh target home for dry-run drills.
- [x] Add backup manifest versioning.
- [x] Add `doctor --strict` failure when latest backup is stale or unverifiable.
- [x] Document local, off-machine, and encrypted backup policy even if remote
      sync remains future work.

Description:
Backup create, verify, and restore now exist and are covered by severe restore
drills. Remaining backup work is about scheduled/off-machine/encrypted backup
policy and historical retention behavior, not the basic ability to recover a
local Arcwell home.

How to test:
- Maintain a severe restore drill test that creates profile, memory, wiki page,
  source card, job state, cursor, project, and channel data; backs it up; restores
  into a fresh temp home; then compares expected records and files.
- Maintain tampered-manifest and missing-file restore tests.
- Maintain restore refusal test when target home is non-empty and no safety flag is
  supplied.
- Run `cargo test --all --all-features`.

Success looks like:
- A user can restore into a clean `ARCWELL_HOME` and run `arcwell doctor --strict`
  successfully.
- Tampered backups fail closed with actionable errors.
- `STATUS.md` keeps backup marked with exact remaining remote-sync and
  retention limits.

Adversarial/severe gate:
- Simulate partial backup, modified DB, missing wiki file, wrong manifest hash,
  non-empty target, symlink escape in backup path, and corrupted JSON manifest.
- No test may rely on production user data.

### 2. Service Supervision, Heartbeat, And Strict Doctor

- [x] Add `arcwell service install`.
- [x] Add `arcwell service uninstall`.
- [x] Add `arcwell service status`.
- [x] Add `arcwell service restart`.
- [x] Add `arcwell service logs`.
- [x] Generate a macOS LaunchAgent plist for `arcwell worker run`.
- [x] Record worker heartbeat in SQLite.
- [x] Add worker identity, start time, last tick, processed job count, and last
      error to ops snapshot.
- [x] Add `arcwell doctor --strict`.
- [x] Make strict doctor fail nonzero for stale heartbeat, excessive dead
      letters, stale backup, missing service, schema mismatch, or missing
      required directories.
- [x] Document Linux systemd as planned or implement it if cheap after launchd.
- [x] Live-smoke macOS launchd install, status, restart/recovery, logs, and
      uninstall against a real user service.
- [x] Decide and implement or explicitly defer Linux systemd user-service
      support.

Description:
The worker loop, heartbeat, strict doctor checks, macOS LaunchAgent commands,
and disposable live-smoke script exist. On macOS, `scripts/service-live-smoke`
proved install, status, restart, killed-worker recovery, logs, strict stale
heartbeat failure, corrupt plist failure, missing binary failure, unreadable log
reporting, and uninstall cleanup against a real user launchd service on
2026-06-20. Linux systemd user-service support is explicitly deferred until a
Linux CI or staging host can run `systemctl --user` live; the repository keeps
the target unit shape documented but does not claim implementation.

How to test:
- Unit-test plist generation and path escaping.
- Integration-test heartbeat updates with a temporary home and short worker run.
- Add strict doctor tests for healthy, stale worker, dead-letter threshold, no
  backup, and missing service states.
- On macOS, run `scripts/service-live-smoke` for disposable no-load adversarial
  checks plus real launchd install, status, restart, killed-worker recovery,
  logs, and uninstall when no existing `com.arcwell.worker` service is loaded.

Success looks like:
- `arcwell service status` reports whether the worker is installed, running, and
  recently healthy.
- `doctor --strict` catches silent failure instead of returning green.
- Ops surfaces show worker liveness without reading logs manually.

Adversarial/severe gate:
- Test paths with spaces, hostile service labels, missing binary, killed worker,
  stale heartbeat, unwritable log directory, and corrupt service metadata.

### 3. Durable Cloudflare Edge Inbox

- [x] Pick storage design: D1, Queue, Durable Object, or combined minimal model.
- [x] Persist accepted events at the edge instead of echoing payloads only.
- [x] Add drain endpoint for local service with lease/ack/nack semantics.
- [x] Add event retention and max-age cleanup.
- [x] Add replay/idempotency behavior at edge and local drain.
- [x] Add rate limits or abuse controls.
- [x] Add edge auth rotation story.
- [x] Add local worker tests with an in-memory store for auth, idempotency,
      leasing, retries, dead letters, expiry, and Telegram normalization.
- [x] Add deploy docs and live smoke commands.
- [x] Add severe local proof that remote drain acks only after local persistence
      and nacks local persistence failures.
- [x] Add worker regression coverage for nack retry delay and D1 lease candidate
      selection.
- [x] Live-smoke deployed Cloudflare D1 persistence, lease, ack, nack, expiry,
      secret rotation, and rate-limit behavior.
- [x] Live-smoke local Rust `edge drain-remote` against the deployed worker and
      prove ack happens only after local persistence.
- [x] Run `scripts/edge-live-smoke` with staging `ARCWELL_EDGE_SECRET` and
      `ARCWELL_EDGE_NEXT_SECRET`.
- [ ] Add Miniflare coverage if future local Node tests miss another
      deployed-worker failure mode.

Description:
The worker now has durable D1-backed code paths, local severe tests, D1 SQL
candidate-selection coverage, deployed health/forged-secret checks, and a
repeatable authenticated live smoke script. Worker deploy, remote D1 `SELECT 1`,
and `scripts/edge-live-smoke` passed on 2026-06-20. The live run initially
found a real bug: the HTTP lease endpoint clamped `leaseSeconds` to 30 seconds,
so staging could not prove lease expiry/retry. The endpoint now permits
1-second staging leases and has a worker regression test.

How to test:
- Worker tests for unauthorized request, oversized body, invalid JSON, duplicate
  idempotency key, expired event, lease timeout, ack, nack retry delay,
  dead-letter, D1 candidate ordering, and drain ordering.
- Local integration test that drains edge-like events into SQLite, including
  nack-without-ack on local persistence failure.
- Live smoke on Cloudflare staging with disposable synthetic events using
  `scripts/edge-live-smoke`.

Success looks like:
- A synthetic event sent while local Arcwell is offline remains available until
  drained or expired.
- Duplicate sends do not create duplicate local events.
- Expired events are not delivered as fresh work.

Adversarial/severe gate:
- Test replay storms, forged secret, huge payload, many duplicate ids, malformed
  nested JSON, clock skew around expiry, nack retry delay, exhausted failed
  rows before later valid work, local persistence failure, and drain without ack
  followed by retry.

### 4. Telegram End To End

- [x] Implement Telegram webhook transformer in the edge package.
- [x] Map Telegram `update_id` to edge idempotency key.
- [x] Preserve chat id, sender id, username, message id, timestamp, and text.
- [x] Normalize Telegram formatting into safe channel body data.
- [x] Drain Telegram edge events into `channel_messages`.
- [x] Add outgoing Telegram send command/tool.
- [x] Add send status, retries, and delivery error storage.
- [x] Add project resolution path for Telegram messages.
- [x] Add authorization policy for which Telegram senders/chats may read or
      mutate project state.
- [x] Enforce explicit Telegram chat send authorization before outgoing sends
      and delivery retries.
- [x] Store Telegram transport failures as classified retryable errors instead
      of raw provider URLs that can contain bot tokens.
- [x] Add slash/MCP docs that say what is automatic vs manual.
- [x] Add a repeatable `scripts/telegram-live-smoke` runner for preserved-home
      local authorization checks and optional live Telegram/edge proof.
- [ ] Live-smoke real Telegram webhook -> Cloudflare -> local drain ->
      `channel_messages`.
- [x] Live-smoke real outgoing Telegram send and delivery attempt recording.
- [ ] Add automatic worker-driven retry for due Telegram deliveries.
- [ ] Add safe follow-up context carryover for authorized Telegram chats.
- [ ] Add richer media/update support or explicitly document text/caption-only
      scope.

Description:
Telegram has real webhook normalization, local drain, outgoing send with
explicit send authorization, MarkdownV2 escaping, delivery attempts, retry
helpers, retryable transport-error classification, authorization, conservative
project binding, and a repeatable preserved-home live-smoke script. The live
smoke previously proved Telegram `getMe`, webhook setup to the Arcwell edge
worker, outgoing provider send, and one real Telegram webhook reaching the edge.
It is not product-complete until the user sends the exact printed phrase and
the script records exactly one matching local `channel_messages` row after a
second drain.

How to test:
- Unit-test Telegram update parsing for text, edited message, missing fields,
  unsupported payloads, duplicate update id, long text, Markdown injection, and
  commands.
- Integration-test webhook -> edge event -> local drain -> channel record.
- Integration-test outgoing send with mocked Telegram API.
- Run `scripts/telegram-live-smoke` with `TELEGRAM_BOT_TOKEN`,
  `TELEGRAM_TEST_CHAT_ID`, `TELEGRAM_WEBHOOK_SECRET`, `ARCWELL_EDGE_SECRET`,
  `ARCWELL_EDGE_URL`, and `ARCWELL_TELEGRAM_LIVE_CONFIRM=disposable` against a
  disposable test chat. Send the exact phrase printed by the script; on timeout,
  re-run the printed `TELEGRAM_SMOKE_EXPECT_TEXT=... ARCWELL_SMOKE_HOME=...`
  command and inspect the preserved mismatch/duplicate diagnostics.

Success looks like:
- A real Telegram message appears in `channel_messages` once.
- A safe outgoing reply can be sent and its delivery state inspected.
- Unauthorized chat ids cannot read or modify projects.

Adversarial/severe gate:
- Test forged sender/chat ids, replayed update ids, malicious Markdown/HTML,
  prompt-injection text, ambiguous project switch, overlong messages, API
  timeout, 429 retry, and failed delivery.

### 5. Live Project And Thread State

- [x] Define project status data model separate from static project summary.
- [x] Add project status events or snapshots.
- [x] Check whether this local Codex environment exposes usable thread
      inventory/read tools to Arcwell; tool discovery did not expose Codex
      `list_threads`/`read_thread`.
- [ ] Add a native host adapter for Codex thread inventory if a stable API
      becomes available.
- [x] Add a degraded manual sync path when live host APIs are unavailable.
- [x] Add an explicit verified host-sync protocol with a freshness marker so
      manual or stale snapshots cannot masquerade as live thread state.
- [x] Add Claude capability matrix: what can be read, manually updated, or not
      accessed.
- [x] Make `project_resolve` return static match and live-state availability.
- [x] Add `project_status` command/tool if needed, rather than forcing agents to
      infer from `project_list`.
- [x] Add per-channel authorization for project reads/writes.
- [x] Add status provenance: host, thread id, timestamp, summary source.

Description:
Project records, status snapshots, explicit verified host-sync snapshots, and
work-run evidence consolidation exist. Tool discovery in the 2026-06-21 Codex
thread did not expose usable Codex thread inventory/read APIs to Arcwell, so
there is no native adapter yet. `project_resolve`/`project_status_get` say that
explicitly with live-state availability, provenance, freshness expiry, and a
degraded host capability matrix. This prevents "how is that going?" from
becoming a fake answer over stale summaries.

How to test:
- Unit-test ambiguous aliases, follow-up context, stale status, missing host
  integration, and unauthorized channel access.
- Integration-test manual project status update and retrieval.
- If host API exists, live smoke against a disposable Codex thread.

Success looks like:
- Project status answers include timestamp, source, and confidence.
- When live state is unavailable, the tool says so explicitly.
- Fresh explicit host sync is marked available only until its freshness window
  expires.
- Telegram/project follow-ups can carry context without guessing.
- Work-memory graph consolidation can propose project status snapshots from
  trace evidence rather than only manual summaries.

Adversarial/severe gate:
- Test forged channel sender, stale status masquerading as live status, ambiguous
  "the other project", direct project id access without permission, deleted
  thread reference, forged host-live source labels, expired verified sync, and
  injected instructions inside status text.

## P1 Core Product Capabilities

### 6. Arcwell Memory Personal Memory

- [x] Evaluate and choose memory integration boundary.
- [x] Vendor the former `mem0-rs` codebase into the Arcwell monorepo as
      Arcwell Memory crates.
- [x] Wire Arcwell core/CLI/MCP to Arcwell Memory add/search/update/delete,
      history, and user-scoped forget.
- [x] Back up and restore Arcwell Memory vector/history artifacts.
- [x] Add canonical `ARCWELL_MEMORY_*` config names with legacy
      `ARCWELL_MEM0_*` aliases.
- [x] Extend Arcwell review schema for memory operations, target memory ids,
      user scope, metadata, applied results, rejection reasons, and provenance.
- [x] Implement deterministic extraction into ADD/UPDATE/DELETE/NONE candidates.
- [x] Implement sensitivity, source, timestamp, and review state for candidates.
- [x] Implement model confidence and a richer decision ledger for memory
      observations/decisions.
- [x] Implement active-store dream/reconcile job with duplicate cleanup and
      reviewable conflict candidates.
- [x] Add active-store delete/forget cascade across provider memories, provider
      history, candidates, lifecycle inputs, decision observations, and simple
      compatibility rows.
- [x] Add historical backup retention/tombstone policy for forgotten memory data.
- [x] Add pre-turn recall guidance through skills/MCP command prompts for
      Arcwell Memory search.
- [x] Add manual and hook-oriented capture flow that never silently stores
      sensitive facts without review policy.
- [x] Add process-level Codex hook smoke scaffold that executes plugin
      `hooks/hooks.json` commands in a disposable home.
- [x] Add deterministic eval gate script that refuses live/model quality claims
      without explicit provider/cost opt-in and a model-candidate oracle.
- [ ] Live-smoke Codex plugin hooks and Claude degraded memory workflow.
- [x] Add personal-memory eval corpus.

Description:
Arcwell Memory now has a product-complete local lifecycle for the current scope:
provider CRUD/history/forget, reviewed ADD/UPDATE/DELETE/NONE candidates,
confidence/reason decision ledger, deterministic personal-memory eval corpus,
active-store forget cascade, process-level Codex hook command smoke, and
explicit backup tombstones. Capture inference no longer directly writes raw
provider-inferred text when the deterministic extractor finds no candidate.
Live model-backed quality, human review UI, and fresh Codex/Claude host
execution are still separate proof items.

How to test:
- Tests for each operation type: ADD, UPDATE, DELETE, NONE.
- Tests for contradictory facts, changed preferences, sensitive medical/personal
  data, duplicate phrasing, and false positives.
- Tests for "forget this" across Arcwell Memory, candidates, provider history,
  lifecycle inputs, decision observations, and compatibility rows.
- Keep backup tombstone tests explicit and do not claim forgotten memory data is
  erased from retained historical snapshots.
- Run severe eval corpus and report precision/recall tradeoffs.
- Run `scripts/codex-hook-smoke --arcwell-bin target/debug/arcwell` for local
  hook contract proof.
- Run `scripts/memory-model-eval-gate --arcwell-bin target/debug/arcwell`; use
  `--require-live` only when explicit provider and cost gates are configured.

Success looks like:
- Personal facts like "my cat is called Ophelia" are stored cleanly.
- Changed facts update or create reviewable conflict cleanup candidates instead
  of duplicating forever.
- Sensitive candidates are reviewable and deletable.
- Forgetting a user removes active provider memories, provider history, review
  candidates, compatibility memories, old lifecycle inputs, and user-scoped
  decision observations, then writes a backup-retention tombstone.

Adversarial/severe gate:
- Test prompt-injection text asking the agent to store secrets, malicious
  transcript content, contradictory identity facts, mass duplicate memories,
  task-local prose under infer+auto-apply, deletion resurrection, stale history
  leakage, and private data leakage through list/search/errors.

### 7. Research And Librarian Synthesis

Target design:
Arcwell Research has one user-facing mode: Deep Research. Invoking research
means broad source discovery, deep reading, source-card and claim extraction,
clustering, skeptic/refutation passes, cited synthesis, audit, and durable
writeback. A short brief can be a report section or interim artifact, but not a
separate quick/surface mode. See `docs/deep-research-system-design.md`.

- [x] Define a typed source-card schema with versioning.
- [x] Add `research run/status/read/audit/stop` as the target deep-run surface
      over the existing partial plan/workflow/search/report building blocks.
- [x] Add a research source ledger with source-family, read-depth, canonical
      URL, provider, freshness, fetch status, and run-source links.
- [x] Link source cards to research runs so audit/report retrieval is not only
      literal query text search.
- [x] Add bounded model-output source extraction into claims, entities, dates, and links.
- [x] Add a structured claim graph with confidence, caveats, temporal scope,
      source-card evidence links, and contradiction links.
- [x] Add deterministic clustering across linked run sources and extracted
      claims, with adapter/source-family fields ready for RSS, GitHub, arXiv, X,
      web search, local wiki, and fetched sources.
- [x] Add deterministic contradiction, staleness, untrusted-source,
      low-reliability, robots/noindex, and uncertainty audit detection for
      source cards.
- [ ] Add page expansion that actively gathers related docs/blogs/repos/social
      sources before writing a topic page.
- [ ] Add native host-search pathway for Codex/OpenAI and Claude where available.
- [x] Add Codex-native subagent prompts/configs for scout, corpus builder,
      extractor, skeptic, synthesizer, and auditor roles.
- [x] Add mandatory skeptic/refutation passes for important claims before final
      synthesis.
- [x] Add a report compiler that produces final reports with executive summary,
      methodology/source coverage, key findings, evidence tables,
      contradictions, confidence labels, gaps, bibliography, and retrieval date.
- [x] Add saturation reporting that explains why the run stopped: coverage,
      diminishing novelty, unresolved blocker, provider limit, budget, or user
      stop.
- [ ] Keep Brave and Perplexity as optional provider adapters.
- [x] Add research output audit command/checklist.
- [x] Prevent generated research pages from becoming primary sources.

Description:
The current research and librarian flows are still local/deterministic for
orchestration, but the deep-run substrate is implemented: durable run lifecycle,
role tasks, source ledger/run links, source-card run linking, bounded
model-output claim ingestion, structured claims, deterministic clusters,
contradiction records, mandatory skeptic pass, run-aware audit, and report
compilation. Source cards carry schema version, evidence role, trust level,
reliability score, provenance strength, inferred source owner, crawl-rate
policy, extracted dates/entities, and audit flags. `research_audit_run` includes
run-linked source cards even when literal query search misses them. Reports
exclude generated/model-answer, untrusted, and low-reliability source cards from
primary evidence and are marked incomplete when skeptic/audit checks fail.
Codex-native role prompts/config guidance now covers scout, corpus builder,
extractor, skeptic, synthesizer, and auditor handoffs with read-heavy subagent
defaults, adversarial evidence rules, generated-output recursion checks,
uncertainty preservation, and coverage/saturation reporting. The deep-only
target still needs a fresh Codex-thread subagent smoke with Arcwell MCP tools,
real host-native search proof, large-corpus source-count/saturation evidence,
and live runs on the three reference topics.

How to test:
- Source-card schema validation tests.
- Source-ledger/run-link retrieval tests proving a run finds its source cards
  even when literal query search misses them.
- Mock-provider tests for citation extraction, missing citations, conflicting
  sources, stale dates, and source type mixing.
- End-to-end topic expansion fixture: seed one launch source, verify it
  collects related source cards, extracts claims, clusters evidence, runs a
  skeptic pass, and writes a cited report.
- Run live Deep Research through Codex on at least:
  `AI startup scene in London`, `most effective image compression algorithms`,
  and `safe cloud code execution with compile-time security constraint
  verification`.

Success looks like:
- Invoking `$deep-research` produces a deep report, not a quick answer.
- Broad topics can gather hundreds of candidate sources and explain source
  coverage/saturation.
- A launch/repo/paper/market/technical field can become a report with cited
  claims, gaps, uncertainty, and contradiction notes.
- Reports distinguish local wiki evidence from current web findings.
- Contradictions are surfaced instead of smoothed over.

Adversarial/severe gate:
- Test SEO spam source, prompt-injection page, fake citation, dead link,
  conflicting launch dates, generated page recursion, stale source, model answer
  without citations, subagent summary losing caveats, source-card run-link
  misses, and final-report claims absent from source cards.

### 8. Work-Memory Graph

- [x] Define the work trace data model and redaction rules.
- [x] Add `work_runs`, `work_events`, `work_artifacts`, and `work_links` or the
      smallest equivalent schema needed for the first real workflow.
- [x] Add CLI/MCP tools for work run start, event record, finish, search, read,
      and consolidate.
- [x] Add degraded manual commands for hosts that cannot provide lifecycle
      hooks.
- [ ] Add Codex plugin prompts or hooks for task start/finish capture where the
      host can support them.
- [x] Link work traces to projects, source cards, wiki pages, memory lifecycle
      events, costs, and future procedure candidates.
- [x] Add consolidation method/tool that can create project status proposals
      with trace provenance.
- [ ] Add consolidation job that can surface unresolved risks, recurring
      failures, and reusable lessons.
- [x] Add basic ops visibility for recent work runs. Stale-consolidation and
      pending-follow-up views remain ops UI work.
- [x] Prove generated project statuses and briefs cite underlying traces/source
      cards, not prior generated summaries alone.

Description:
Arcwell needs a Brain-like memory of work performed by agents: goals, sources,
decisions, failures, validation, outcomes, and reusable lessons. This is
separate from personal memory and source-backed wiki knowledge. It is the
evidence substrate for project status, procedural learning, research quality,
and cost reduction.

Current state:
Core work traces are implemented in SQLite with manual CLI/MCP commands,
redaction, bounded payloads, search/read, project-status consolidation, and
severe tests. Automatic Codex/Claude lifecycle hooks, scheduled consolidation,
procedure candidate extraction, and a richer ops UI are still follow-on work.

How to test:
- Unit-test trace creation, linking, redaction, search, finish semantics, and
  stale-state handling.
- Integration-test project status generation from traces.
- Add fixtures for tasks with validation, failed commands, source-card links,
  file changes, and follow-up items.

Success looks like:
- A substantial task can be recorded from start to finish with validation and
  outcome.
- Work traces can be searched/read through CLI and MCP.
- Project status snapshots can be generated from trace evidence with timestamp
  and source provenance.
- Sensitive data is redacted or excluded according to policy.

Adversarial/severe gate:
- Test secret leakage, prompt injection in logs/tool output, malformed host or
  thread ids, generated-summary citation loops, missing validation masquerading
  as success, huge traces, and raw terminal transcript over-capture.

### 9. Procedural Learning

- [x] Define procedure schema: trigger context, problem, preconditions, steps,
      tools, validation, risks, and provenance.
- [x] Add explicit procedure confidence and freshness fields/policies.
- [x] Add procedure candidate operation types: add, update, and archive.
- [x] Add richer procedure candidate operation types for merge and no-op.
- [x] Store approved procedures as versioned Markdown artifacts with SQLite
      metadata.
- [x] Add CLI/MCP tools for procedure search, list, read, candidate apply,
      candidate reject, and curator runs.
- [x] Add deterministic first-pass extraction from completed work traces.
- [ ] Add optional model-backed extraction behind explicit config and cost
      policy.
- [x] Add curator behavior for exact duplicate detection and archival
      candidates.
- [x] Add curator behavior for stale review and merge proposals.
- [ ] Add plugin prompts that retrieve approved procedures before relevant
      tasks.
- [ ] Add a reviewed export path from approved procedure to Codex skill text
      where appropriate.
- [x] Ensure untrusted channel/source text cannot become a procedure without
      review.

Description:
Arcwell can now learn a local, reviewed slice of reusable task procedures
without silently modifying skills or polluting prompts. Procedures are
reviewable procedural memory derived from work traces, separate from personal
memory and external knowledge. Remaining work is model-backed extraction/evals,
plugin retrieval, and reviewed skill export.

How to test:
- Unit-test schema validation, candidate apply/reject, versioning, search,
  duplicate detection, archival, and unsafe source handling.
- Build a small eval set of tasks that should create procedure candidates and
  tasks that should not.

Success looks like:
- A completed task can produce a pending procedure candidate with provenance.
- Applying a candidate creates a versioned approved procedure.
- Future relevant tasks retrieve the approved procedure.
- Later evidence can propose an update instead of creating overlapping copies.

Adversarial/severe gate:
- Test prompt injection inside work traces, malicious tool output, overlong
  procedure text, path traversal in generated filenames, conflicting procedure
  updates, stale procedures, and sensitive-source auto-approval attempts.

### 10. Ops UI And Human Visibility

- [x] Choose initial UI shape: server-rendered localhost HTML over the existing
      ops snapshot.
- [x] Add first read-only `/ops/ui` route.
- [x] Escape untrusted channel/source/error text in the initial ops HTML.
- [x] Show health, backups, worker heartbeat, jobs, dead letters, edge events,
      watch/source health, cursors, provider/secret health, costs, projects,
      channels, memory/procedure candidates, work runs, policy decisions and
      approvals, Telegram delivery failures, and project status proposals.
- [x] Add severe ops UI tests for XSS across channel, source card, project,
      procedure, work run, policy denial, and stored error text.
- [x] Browser-validate desktop and mobile read-only rendering for obvious
      overlap/clipping before marking Wave 3 P1.10 complete.
- [ ] Decide whether to keep server-rendered HTML or split out a small frontend
      package before adding richer controls.
- [x] Add filters, sorting, search, and detail views.
- [x] Add a narrow authenticated edge-event dead-letter control with policy,
      CSRF, idempotency, and replay tests.
- [ ] Add manual job requeue/cancel controls only after safe public core APIs
      exist; do not fake unsupported remediation.
- [ ] Add safe controls for retry delivery, apply/reject candidate, run doctor,
      create/verify backup, drain once, and inspect policy denial reasons.
- [x] Add source health and last-success/last-failure view.
- [x] Add local auth token and CSRF/cross-origin stance before mutating controls.
- [x] Add read-only UI smoke tests.
- [ ] Keep Obsidian/Markdown as the wiki editing surface; do not duplicate wiki
      authoring unless needed.

Description:
Ops is no longer only JSON: `/ops/ui` shows the current durable ops snapshot
across health, queues, sources, secrets, costs, projects, channels,
memory/procedure review, work runs, policy, Telegram failures, and project
status. It now includes filters/search/sorting, detail views, health scoring,
queue/source/credential summaries, and one carefully authenticated
policy-checked edge-event dead-letter control. It is not yet a product-grade
control surface. The remaining work is broader charts/browser validation and
only those mutating controls backed by safe core APIs.

How to test:
- Unit-test ops snapshot shape.
- Maintain server-side escaping tests for untrusted channel/source/project/
  procedure/work/policy/error text.
- Browser test for loading UI and rendering empty/healthy/failing states on
  desktop and mobile.
- API tests for requeue/cancel permissions and failure paths once those safe
  APIs exist.

Success looks like:
- A human can open one local page and see what is broken, stale, queued, or
  waiting for review.
- Dangerous controls require auth, CSRF, idempotency, policy checks, and audit
  records.
- Untrusted channel/source/project/procedure/work/policy/error text is escaped
  in the rendered UI.

Adversarial/severe gate:
- Test hostile job errors containing HTML/Markdown/script text, huge error
  payloads, stale state, repeated submissions, unauthorized cross-origin writes,
  policy denial, and keyboard-only navigation for critical controls.

### 11. Cost Controls And Kill Switch

- [x] Define global, package, provider, and source-level budgets.
- [x] Add budget config storage.
- [x] Enforce budgets before daemon-side web search and X recent search.
- [x] Enforce budgets before model synthesis and every scheduled paid/network
      job kind.
- [x] Add kill switch for all paid/network providers.
- [x] Record estimated costs for X recent search, daemon-side web search,
      Arcwell Memory OpenAI provider setup, X OAuth/following/bookmark paths,
      Telegram send/retry, remote edge drain, URL/RSS/GitHub/arXiv scheduled
      network jobs, and blocked cost decisions.
- [ ] Record provider-reported actual costs where provider APIs return reliable
      usage/cost data.
- [x] Add ops visibility for budget burn and blocked jobs.
- [x] Add CLI/MCP commands for budget status and temporary override.

Description:
Always-on monitoring without hard budget limits is not acceptable. Cost policies
now reserve estimated spend before Arcwell-owned paid/provider/network egress
and block current scheduled paid/network job kinds before credentials or network
calls. Blocked decisions are durable in cost/ops state. Actual provider-reported
cost reconciliation remains future work where providers expose reliable usage
data.

How to test:
- Unit-test budget allow/block decisions.
- Integration-test a job blocked before provider call.
- Test override expiry and kill switch behavior.
- Mock provider cost reporting and budget burn.
- Maintain severe tests for retry storms, repeated budget reservations,
  malformed costs, scheduled kill switches before HTTP, and blocked-job ops
  visibility.

Success looks like:
- Misconfigured monitors cannot quietly spend money forever.
- Blocked jobs explain the budget rule that stopped them.
- `/ops` and `cost summary` expose recent allowed/blocked cost decisions.

Adversarial/severe gate:
- Test runaway queue, repeated retries, clock skew around reset window, negative
  cost values, huge costs, unknown provider, override abuse, and concurrent jobs
  racing the same remaining budget.

### 12. Policy Enforcement Outside The Agent

- [ ] Inventory every sensitive operation in CLI, MCP, worker jobs, HTTP, edge
      drain, memory, project, channel, source ingestion, and provider adapters.
- [x] Define the first `arcwell-policy.toml` or YAML schema and default
      conservative policy.
- [x] Add a `PolicyEngine` in `crates/arcwell-core` with explainable decisions:
      allow, deny, require approval, or defer.
- [x] Store policy decisions and approval records in SQLite.
- [x] Integrate existing cost policies and channel authorization into the
      broader policy model without weakening current checks.
- [x] Apply policy checks before web/research provider calls, X calls, source
      ingestion, memory capture/apply, procedure apply, Telegram send, project
      writes, secret access, OAuth token writes, and worker enqueue/execution.
      Severe tests now prove denied source writes, memory capture, worker
      enqueue, queued URL ingest, and X OAuth stop before local/provider side
      effects; the broader sensitive-operation inventory remains open above.
- [x] Add CLI/MCP tools for policy check, explain, list, override, approvals,
      approve, reject, and decision history.
- [x] Add ops visibility for denied actions, pending approvals, and matching
      rules.
- [x] Document the boundary between Arcwell policy enforcement and host/OS
      sandboxing.

Description:
Prompt instructions are not enforcement. Arcwell needs a declarative policy
layer for Arcwell-owned network, paid, mutating, sending, and secret-using
actions. This is the first step toward stronger sandbox policy without claiming
kernel-level isolation. The first implemented slice loads
`ARCWELL_HOME/arcwell-policy.toml` with `[[rules]]`, evaluates allow/deny/
require_approval/defer decisions, writes SQLite decision and approval audit
records, and exposes recent policy state through the ops snapshot. Deny/
approval checks are wired before X recent search, X monitor, daemon web search,
memory candidate apply, procedure candidate apply, Telegram send/retry, local
project writes, and CLI/MCP secret value/ref admin. Existing cost policies still
run after policy allows provider network actions and before credentials/network
calls; existing Telegram/project channel authorization is not weakened or
replaced. CLI/MCP policy administration can check, explain, list rules and
decisions, create temporary allow overrides, list approvals, and mark approvals
approved or rejected.

How to test:
- Unit-test policy parsing, rule priority, malformed policies, allow/deny,
  approval/defer decisions, audit records, and override expiry.
- Integration-test denied provider calls, memory apply, Telegram send, project
  write, and secret access.

Success looks like:
- Denied wired actions do not read credentials, make provider calls, send
  messages, or mutate local state.
- Required-approval actions create pending approval records.
- Policy decisions are inspectable through ops snapshot data and CLI/MCP
  commands explain the matching rule.

Adversarial/severe gate:
- Test prompt attempts to bypass policy, concurrent budget/policy decisions,
  malformed policy files, stale overrides, broad wildcard rules, denied network
  and approval-gated provider paths before credential lookup, guarded secret
  admin access/mutation, and policy-denial text containing untrusted payload
  snippets.

### 13. HTTP Hardening

- [x] Replace basic HTTP handler panics/`expect` paths with structured internal
      error responses for current read endpoints.
- [x] Add a basic structured JSON error shape for current read endpoints.
- [x] Add local auth token for HTTP API.
- [x] Define CORS/CSRF stance for browser UI.
- [x] Limit sensitive endpoints and redact secrets/errors.
- [x] Add request size/time limits where relevant.
- [x] Add richer typed error categories for UI and agents.

Description:
The HTTP server is still a development/local surface, but the current read
routes and `/ops/ui` now share structured redacted errors, optional local bearer
or `x-arcwell-http-token` auth via `--auth-token` or
`ARCWELL_HTTP_AUTH_TOKEN`, hostile-Origin rejection, explicit read-only POST
rejection, URI/body/query limits, and security headers. P1.13 has severe tests
in `crates/arcwell-cli/src/main.rs`, and the focused severe HTTP tests plus the
full Rust suite pass in this checkout. Browser-based UI validation remains for
future interactive controls.

How to test:
- HTTP tests for missing auth, bad token, store error, malformed query, secret
  redaction, and CORS behavior.
- Panic-free test harness around handlers.

Success looks like:
- Bad HTTP requests return useful errors and do not crash the server.
- Browser UI can use the API without exposing broad unauthenticated local state.

Adversarial/severe gate:
- Test hostile Origin, CSRF-like POST, giant query, HTML/script in stored errors,
  missing DB, locked DB, and secret-like strings in error output.

## P1 Data And Source Quality

### 14. Wiki Ingestion Quality

- [x] Add deterministic readability-like extraction for HTML pages.
- [x] Preserve fetch provenance, escaped source excerpt, and cleaned readable text separately.
- [x] Add canonical URL and duplicate detection.
- [x] Add content-type and size policy.
- [x] Add robots/crawl-rate policy notes.
- [x] Add source reliability fields.
- [x] Add incremental local Markdown sync and deleted-file handling.
- [ ] Add browser-rendered JavaScript readability extraction for pages that require rendering.

Description:
URL ingest now writes an untrusted Markdown artifact with provenance, canonical
URL, cleaned readable text, escaped source excerpt, content-type checks, bounded
body reads, redirect validation, deterministic article/main/body extraction,
robots metadata, crawl-rate policy notes, and source reliability fields. It is
still not browser-rendered JavaScript extraction and does not claim
model-backed source extraction.

How to test:
- Fixtures for HTML, Markdown, wrong content type, redirect, huge response,
  duplicate canonical URL, and hostile embedded prompt.
- SSRF and private-network tests stay in place.

Success looks like:
- Wiki pages are readable, source-backed, deduped, and preserve provenance.

Adversarial/severe gate:
- Test metadata IPs, redirect to private IP, HTML script injection, boilerplate
  extraction, robots/noindex metadata, canonical URL collision, enormous page,
  binary response, slow response, deleted Markdown source files, and prompt
  injection.

### 15. Adapter Cursoring And Source Health

- [x] Add item-level cursor semantics for RSS, GitHub, arXiv, and X.
- [x] Track last success, last failure, last item id/date, and next run.
- [x] Add duplicate policy per adapter.
- [x] Add provider rate-limit handling.
- [x] Add source health in ops snapshot.
- [x] Add source health UI.
- [x] Add scheduled polling enqueue hooks.
- [ ] Add full resident scheduled polling through worker service.

Description:
Adapters now update source cards with canonical duplicate suppression, expose
SQLite source-health records, and advance cursor/source-success state only after
durable writes complete. Rate-limit/quota errors are classified as
`rate_limited` with longer backoff, and due active watch sources can be enqueued
while respecting source-health `next_run_at`. Provider-specific pagination/ETag
handling and a full resident scheduled polling loop are still future work.

How to test:
- Mock feeds/repos/searches with reordered, duplicated, deleted, and stale items.
- Test 429/500 provider responses and retry behavior.
- Test cursor update only after successful durable write.

Success looks like:
- Re-running adapters does not flood duplicate source cards.
- Failed sources are visible and retry predictably.

Adversarial/severe gate:
- Test partial writes, cursor write failure, provider pagination loop, duplicate
  IDs across feeds, clock skew, and retry storms.

### 16. X Production Monitoring

- [x] Live-test OAuth user-context token refresh with current credentials.
- [x] Live-test definitive watch rebuild from bookmarks and recent follows with
      OAuth user-context auth.
- [x] Add watch-list audit output with counts and provenance.
- [x] Add rate-limit and tier-aware failures.
- [x] Add X bookmarks ingestion path if not already covered by rebuild internals.
- [x] Store bookmarked tweet bodies, public metrics, and source provenance as
      first-class X items instead of only watch-source metadata.
- [x] Add interestingness/source-card/digest flow.
- [ ] Add Cloudflare callback/cron event capture after edge inbox is durable.

Description:
X production monitoring now has local/mock-proven credential health, definitive
watch rebuild audit counts, API tier/rate/quota failure classification, a
watch-source monitor path that ingests accepted watched-source tweets into
source cards/wiki pages and digest candidates, and cursor/source-health safety
under malformed, blocked, duplicate, and quota responses. The copied-home
`scripts/x-live-smoke` path reached X with OAuth 2.0 User Context after a real
local OAuth refresh, then passed live recent search, bookmark/recent-follow
watch rebuild, and watch-source monitor without writing to the real Arcwell
home.

How to test:
- Mock tests for token expiry, refresh failure, 429, malformed tweet, duplicate
  item, unsafe URL, huge text, and cursor behavior.
- Live smoke with copied user-context credentials, recording counts only, not
  tokens: `X_USER_CONTEXT_SOURCE_HOME="$ARCWELL_HOME" scripts/x-live-smoke`.

Success looks like:
- The definitive watch list is small, explainable, and reproducible.
- API failures are visible and do not burn budget or corrupt cursors.

Adversarial/severe gate:
- Test polluted following list, blocked/protected accounts, deleted tweets,
  prompt injection in tweet text, malicious links, API quota exceeded, and
  duplicate newest_id behavior.

## P2 Portability, Packaging, And Host Integrations

### 17. Codex Plugin Verification

- [x] Add command inventory check: plugin prompts vs MCP tools vs CLI-only
      commands.
- [x] Add stale command detection.
- [x] Add plugin install smoke test or documented manual test.
- [x] Add status wording to commands that wrap partial/scaffold features.
- [x] Verify `$skills` point to correct tools and safety rules.

Description:
The plugin surface is broad. It needs verification so commands do not imply that
unfinished services are complete. `docs/codex-plugin-commands.md` now includes
a fresh-thread manual smoke matrix, but no in-app smoke has been recorded as
passed yet.

How to test:
- `scripts/verify-codex-plugin-docs` compares command files to the catalog,
  MCP tool registrations, tracked CLI-only commands, package README badges, and
  safety wording.
- `scripts/verify-codex-plugin-docs --self-test` proves the checker fails on a
  missing MCP tool, undocumented command, unsafe channel prompt, and README
  overclaim.
- Manual install smoke in a fresh Codex thread using the matrix in
  `docs/codex-plugin-commands.md`.
- MCP call smoke for representative commands.

Success looks like:
- Every slash command either works or honestly routes to a partial/scaffold
  explanation.

Adversarial/severe gate:
- Test missing MCP tool, malformed command args, prompt-injection content passed
  through commands, and commands for unavailable features.

### 18. Claude MCP Validation

- [x] Add repeatable local Claude-style stdio MCP smoke for `arcwell mcp`.
- [x] Add official MCP Inspector wrapper and package-availability preflight.
- [ ] Record an interactive MCP Inspector run against `arcwell mcp`.
- [ ] Validate Claude Desktop/Code config in an authenticated local profile.
- [x] Document exactly which features degrade without Codex hooks/skills.
- [x] Add examples for manual memory/profile/wiki use from Claude.
- [x] Add no-op warnings for lifecycle features Claude cannot host.

Description:
Arcwell should be portable, but Claude is not Codex. This task prevents fake
parity. `scripts/claude-mcp-smoke` now proves the local stdio protocol boundary
with disposable homes and adversarial JSON-RPC cases, but host-level Claude UI
validation is still blocked until an authenticated Claude Desktop/Code profile
has an `arcwell` MCP server configured.

How to test:
- Local process-level JSON-RPC smoke: `scripts/claude-mcp-smoke`.
- Inspector package preflight: `scripts/mcp-inspector --check-only`.
- Inspector protocol test: `scripts/mcp-inspector`, then manually exercise
  capability negotiation, tool schemas, representative tool calls, and resource
  errors in the Inspector UI.
- Real Claude MCP connection smoke.
- Manual tool calls: profile list, memory search, wiki search, ops snapshot.

Success looks like:
- Claude users can use Arcwell through MCP and understand what is unavailable.
- Local smoke must pass even with malformed frames, unsupported methods, a bad
  `ARCWELL_HOME`, missing `ARCWELL_HOME`, bounded large responses, and
  unavailable Claude lifecycle hooks.

Adversarial/severe gate:
- Test unsupported MCP methods, malformed frames, huge tool responses, missing
  env vars, unauthenticated profile, and unavailable lifecycle hooks.

### 19. Packaging And Release

- [x] Add release build instructions.
- [x] Add Homebrew formula template or local tap plan.
- [x] Add checksum-verifying installer scaffold.
- [x] Add Linux systemd unit templates and non-destructive renderer.
- [x] Ensure packaged stable plugin invokes installed `arcwell`, not `cargo run`.
- [x] Add generated Codex dev plugin workflow that invokes the local debug
      binary wrapper and syncs into the installed plugin cache.
- [ ] Fresh-thread smoke `arc` inside the Codex app.
- [x] Add upgrade and uninstall path.
- [x] Add migration backup requirement for destructive schema changes.
- [x] Add smoke script for fresh install minimal path.

Description:
Open-source users need a boring install path. Development commands are not a
product installation. The current local gate is `scripts/release-readiness-smoke`:
it proves the candidate binary and stable plugin contract in disposable
install/home paths, including stale `PATH`, interrupted upgrade, backup/restore,
old schema, duplicate service install, bad permissions, and uninstall cleanup.
Homebrew/GitHub publication remains planned, not shipped; the exact blockers and
formula/release inputs are documented in `docs/packaging-and-operations.md`.
`packaging/homebrew/arcwell.rb.template`, `packaging/install.sh`,
`packaging/systemd/*.service.in`, and `scripts/install-systemd-user` are local
scaffolds with fixture verification, not public package proof.

How to test:
- Fresh temp home install smoke: `cargo build --release -p arcwell`, then
  `scripts/release-readiness-smoke`.
- Release artifact/template smoke: `scripts/verify-packaging-artifacts` and
  `scripts/verify-packaging-artifacts --self-test`.
- macOS service smoke: `scripts/service-live-smoke --no-live`, and
  `scripts/service-live-smoke --live` when no real `com.arcwell.worker` service
  is already loaded.
- Linux rendering smoke: `scripts/install-systemd-user install --no-systemctl
  --unit-dir <temp-dir> --arcwell-bin <absolute-test-bin> --arcwell-home
  <temp-home>`. Full Linux service proof still needs a real
  `systemctl --user` session.
- Plugin/dev-loop packaging: `scripts/verify-codex-plugin-docs`,
  `scripts/arcwell-dev smoke`, and `scripts/arcwell-dev sync` when plugin or
  dev-loop docs/scripts change.

Success looks like:
- A candidate package binary can get profile, backup/restore, service plist,
  and strict doctor behavior working from a disposable install prefix.
- The stable plugin reaches the installed `arcwell` through `PATH`, and stale
  binary ordering is caught before release.
- Publication docs name the missing Homebrew/GitHub signed-checksum/Linux-live
  and Codex fresh-thread blockers instead of implying a public package exists.

Adversarial/severe gate:
- Test paths with spaces, missing permissions, old DB version, interrupted
  upgrade, duplicate service install, tar path traversal, checksum mismatch,
  systemd path escaping, uninstall cleanup, and plugin pointing to a stale
  binary.

### 20. Documentation Honesty Pass

- [x] Add status badges to README package section.
- [x] Add status badge to every package README.
- [x] Link each package README to `STATUS.md` and `TODO.md`.
- [x] Remove or qualify product prose that says "watches", "tracks", or
      "Telegram" where the capability is still partial.
- [x] Add "known missing" sections where needed.

Description:
Docs should sell the direction without lying about the current implementation.

How to test:
- Review README and package READMEs for each `STATUS.md` scaffold/risk item.
- Add a script or checklist to ensure every package has a status line.

Success looks like:
- A new reader cannot reasonably mistake scaffolds for finished services.

Adversarial/severe gate:
- Read docs as a hostile evaluator and list every sentence that implies a
  missing capability is live. Fix or annotate each one.

## P2 Missing/Future Packages

### 21. Garderobe Integration

- [x] Copy or vendor `/Users/chabotc/Projects/garderobe` into the monorepo as
      `packages/arcwell-garderobe` if licensing/structure allows.
- [x] Preserve Cloudflare Worker, auth, and MCP design.
- [x] Add Arcwell package README and status badge.
- [x] Add MCP/host docs for outfit planning with weather/profile/style context.
- [x] Decide what data, if any, should sync into Arcwell memory/profile/wiki.
- [x] Add guarded read-only live smoke script that refuses to run without an
      explicit base URL and readonly confirmation.

Description:
Garderobe is a working adjacent personal-domain MCP app. It should become a
first-class Arcwell package without muddying memory/wiki boundaries.

How to test:
- `cd packages/arcwell-garderobe && npm run typecheck && npm test`.
- `GARDEROBE_READONLY_CONFIRM=readonly GARDEROBE_BASE_URL=https://... scripts/garderobe-readonly-smoke`
  for unauthenticated GET-only surface proof.
- Existing adjacent Garderobe live severe tests were reviewed but not copied or
  run from Arcwell because they target the live deployment and depend on
  `.dev.vars`.
- Future MCP smoke through Arcwell host docs should use disposable fixture rows,
  not real wardrobe seed data.

Success looks like:
- Arcwell can point agents to garderobe cleanly for wardrobe planning.

Adversarial/severe gate:
- Local severe integration test covers auth/DCR/PKCE wiring, copied
  `.dev.vars`/seed leakage, hostile item names, weather API failure fallback,
  unsafe prompt instructions in wardrobe metadata, and accidental private
  inventory sync into Arcwell memory/wiki at the docs/package boundary.
- Live auth bypass and remote inventory-leakage proof remain required after a
  disposable or explicitly approved Cloudflare deployment is provisioned.

### 22. Google Workspace Strategy

- [x] Decide host-native connector only vs Arcwell indexing layer.
- [x] If host-native only, document exact usage and non-goals.
- [x] If indexing, define metadata schema, scopes, retention, and privacy rules.
- [x] Add explicit permission/scoping matrix.

Description:
Google Workspace already exists as host connectors in Codex. Arcwell should not
duplicate it blindly. The current decision is host-native connector first,
narrow Arcwell indexing second: Arcwell stores Workspace-derived metadata or
content only when the user explicitly archives it into projects, wiki/source
cards, work runs, procedures, or memory/profile review queues.

How to test:
- For host-native path, run documented examples through connected tools when a
  live connected host is available.
- For future indexing path, mock Gmail/Drive/Calendar metadata and test scope
  controls before adding provider API code.

Success looks like:
- Agents know whether to use host Google tools or Arcwell storage for each task.
- `docs/google-workspace-strategy.md` is the source of truth for that boundary.

Adversarial/severe gate:
- Test overbroad scopes, unauthorized project access, private email leakage,
  stale calendar state, and prompt injection in docs/email bodies.

### 23. Email Channel/Ingestion

- [x] Define `arcwell-email` package boundary.
- [x] Decide Gmail API vs Cloudflare Email Routing for inbound capture.
- [x] Add source-card and channel-message mapping.
- [x] Add sender authorization and routing rules.
- [x] Add digest delivery option boundary for librarian alerts.
- [x] Add bounded Cloudflare Email Routing Worker handler and severe enqueue
      tests.
- [x] Add local Rust drain/persistence from email edge events into durable
      channel messages/source cards.
- [x] Add one-shot email polling that remote-drains the edge inbox and then
      persists local email channel/source-card records.
- [x] Add outbound Cloudflare Email Service send/reply path with rich HTML
      support, recipient authorization, policy/cost checks, and token-redaction
      tests.
- [x] Add configured-author trust boundary: tracked defaults are
      `agent@example.com` and `user@example.com`; real author/agent addresses
      must live only in local env/secret config.
- [x] Add ops visibility for email edge/channel state through existing edge,
      channel, source-card, delivery, and `/ops/ui` surfaces.
- [x] Add guarded setup script for Worker route secret, deploy, and Email
      Routing rule creation.
- [x] Create live Cloudflare Email Routing rule to the Arcwell edge Worker for
      the locally configured agent address without committing real addresses.
- [x] Run empty-queue local poll against the deployed Worker after route setup.
- [x] Run controlled author-originated live ingress smoke and prove one trusted
      local channel/source-card record.
- [x] Run provider-side outbound email delivery smoke.

Description:
Email is part of the desired proactive assistant loop. The current slice is a
bounded `arcwell-email` package with a tested normalized mapper, explicit live
blockers, an edge inbox Email Routing handler that parses bounded raw MIME into
durable email edge events under configured route/sender policy, local Rust
poll/drain into durable email channel/source-card records, and Cloudflare Email
Service send/reply support. Gmail remains host-native first for interactive
selected-thread work. Tracked defaults stay as `agent@example.com` and
`user@example.com`; real agent/author addresses belong in ignored local config
or secrets. A dashboard-created live Email Routing rule now targets the Arcwell
edge Worker for the locally configured agent address, and local poll against
the deployed Worker succeeds on an empty queue. A controlled author-originated
live message was routed through Cloudflare Email Routing, polled from the
deployed edge inbox, and drained into one trusted local email
channel/source-card record. Cloudflare Email Service outbound delivery was also
smoke-tested through `arcwell email send` after recipient authorization. The
live proof used local-only real addresses and secrets; tracked docs
intentionally retain only placeholders.

How to test:
- Package-local severe fixtures:
  `cd packages/arcwell-email && npm test`.
- Worker severe tests:
  `cd packages/arcwell-edge-inbox/worker && npm test`.
- Rust poll/drain/send tests:
  `cargo test -p arcwell-core email -- --nocapture`.
- Local one-shot polling:
  `arcwell email poll` after `ARCWELL_EDGE_URL`/`ARCWELL_EDGE_SECRET` are
  configured.
- Setup script dry/safe gate:
  `scripts/setup-email-route` must refuse without
  `ARCWELL_EMAIL_SETUP_CONFIRM=configure`.
- Manual live smoke with a controlled message from the configured author
  address to the configured narrow agent address, followed by
  `arcwell email poll`.
- Manual outbound smoke after recipient authorization:
  `arcwell email authorize user@example.com --send` and
  `arcwell email send user@example.com "Arcwell email smoke" "Controlled outbound smoke" --from agent@example.com`.

Success looks like:
- Normalized important inbound email metadata can become a safe event/source
  card/channel message draft without treating email body as instructions.
- The docs make clear that local email ingestion/send paths exist, while the
  completed live Cloudflare ingress and provider outbound smoke are bounded
  manual proofs, not a long-running scheduler or production monitoring claim.

Adversarial/severe gate:
- Local fixtures test spoofed From, malicious HTML, attachment bombs, tracking
  links, prompt injection, duplicate Message-ID, oversized bodies,
  auto-responder loops, and unauthorized routing.
- Worker tests prove MIME normalization, duplicate idempotency, route/sender
  rejection, raw-size rejection, and durable edge enqueue.
- Leak-scan tracked docs so real local-only email addresses never replace
  `agent@example.com` and `user@example.com`.

## Cross-Cutting Required Work

### 24. Schema And Migration Discipline

- [x] Add explicit migration table with numbered migrations.
- [x] Add migration tests from fixture DBs.
- [x] Require backup before destructive migrations.
- [x] Add schema drift check in doctor.

How to test:
- Fixture DB from schema version 1 migrates cleanly.
- Corrupt/unknown future schema fails safely.
- Destructive migration helper refuses to run without a verified backup id and
  leaves no side effects on refusal.

Success looks like:
- Upgrades are boring and reversible enough to trust.

Adversarial/severe gate:
- Test interrupted migration, missing column, unknown future version, readonly DB,
  and partial migration rollback.

### 25. Secrets And Credential Lifecycle

- [x] Add expiry and scope health checks.
- [x] Add cheap provider credential health via stored metadata and required
      credential presence checks; live provider probes remain provider-specific
      smoke work.
- [x] Add secret backup sensitivity policy.
- [x] Add rotation/revocation documentation.
- [x] Add ops warnings for stale/missing credentials.

How to test:
- Severe tests cover list/MCP/ops redaction, expired secrets, missing required
  credential health, provider failure redaction, backup manifests, snapshots,
  command echo, and failed provider-response strings that contain token-like
  material.

Success looks like:
- Health and ops surfaces fail loudly when credentials are missing or expired,
  while values stay available only to local provider clients that explicitly
  need them.

Adversarial/severe gate:
- Test secret leakage through logs, MCP resources, errors, backups, snapshots,
  command echo, and failed provider responses.

### 26. Source Trust And Prompt Injection Policy

- [x] Add explicit trust labels to source cards, channel messages, research
      inputs, and wiki pages.
- [x] Add agent-facing skills that require quoting untrusted text as evidence.
- [x] Add sanitizer/renderer rules for Markdown/HTML surfaces.
- [x] Add tests that prove untrusted text is never elevated into instructions.

Description:
Source-card, URL-ingest, search-result, X report, channel-message, research
brief, and generated-page renderers now label untrusted text as source/channel
evidence and escape Markdown/HTML injection payloads. Research source selection
excludes generated `Research Brief:` and `Expanded:` pages so generated outputs
cannot become primary evidence without source links. Agent-facing wiki,
research, audit, X, and channel skills now require quoted/fenced evidence
handling.

How to test:
- Prompt-injection fixtures across X, Telegram, RSS, GitHub, arXiv, email, URL,
  and conversation import.

Success looks like:
- The agent can use hostile source text as evidence without obeying it.

Adversarial/severe gate:
- Test "ignore previous instructions", tool-call exfiltration attempts, Markdown
  links/images, HTML script tags, quoted system prompts, and nested generated
  summaries.

## Continuous Verification Checklist

Run this before marking any P0/P1 item done:

- [ ] `cargo test --all --all-features`
- [ ] Package-specific typecheck/test commands
- [ ] New severe tests fail on the old broken/scaffold behavior or clearly
      refute a realistic failure mode
- [ ] Live smoke documented when external APIs are involved
- [ ] `STATUS.md` updated
- [ ] `TODO.md` checkbox updated
- [ ] Package README updated
- [ ] Plugin commands/skills updated if the agent-facing behavior changed
- [ ] Ops visibility added for new long-running or failure-prone state
- [ ] Remaining risk explicitly stated
