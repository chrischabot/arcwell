# Arcwell Remaining Work

Last updated: 2026-06-24

This file is intentionally only unfinished work. Completed historical checklist
items were removed. Existing unchecked items from the prior `TODO.md` have been
preserved here and grouped under the real-user readiness plan in `PLAN.md`.

Do not mark an item complete because a command, scaffold, prompt, package, or
README exists. Mark it complete only when code, tests, severe review, live proof
where relevant, docs, `STATUS.md`, and this file agree.

## Global Execution Gates

- [ ] Every implementation PR/change updates this file and `STATUS.md`.
- [ ] Every meaningful feature names its behavioral claim before coding.
- [ ] Every feature has at least one test that tries to refute that claim.
- [ ] Every P0/P1 feature has a severe/adversarial test gate before completion.
- [ ] Every P0/P1 feature has an explicit "mirage check" listing what would
      make the work look complete while still being hollow.
- [ ] Every P0/P1 feature has a proof packet before completion: claim ledger,
      changed files, test commands, live-smoke commands when relevant,
      adversarial review notes, performance/resource notes, and remaining risk.
- [ ] Every feature that writes durable state has idempotency, duplicate input,
      partial failure, migration/backfill, backup/export, and recovery behavior
      stated before completion.
- [ ] Every feature that reads external/untrusted content has malicious input,
      prompt-injection-as-data, malformed payload, oversized payload, Unicode,
      and escaping/redaction tests before completion.
- [ ] Every feature that touches a provider, model, worker, delivery path, or
      remote write has policy, cost, secret-redaction, retry, rate-limit, and
      source-health/ops visibility gates before completion.
- [ ] Every external integration has one local/mock test and one documented live
      smoke test.
- [ ] Every agent-facing command or skill must fail honestly when the capability
      is partial, scaffolded, or unavailable.
- [ ] Do not silently convert "manual foreground command works" into "service is
      installed and reliable."
- [ ] Do not call generated summaries "research" or "memory" unless source,
      provenance, and uncertainty are inspectable.
- [ ] Do not call a migration complete until an old-schema fixture, a populated
      database, an empty database, and a rerun/idempotency case are all tested.
- [ ] Do not call a sync complete until cursor advancement, all-items-rejected,
      partial-provider-error, provider 401/403/429/5xx, and duplicate-page cases
      are tested.
- [ ] Do not call a UI complete until browser desktop/mobile smoke, XSS/escaping
      fixtures, empty states, stale states, and clipped/overlapping content have
      been inspected.
- [ ] Do not call a model-backed feature complete until deterministic fixtures,
      malformed model output, evidence grounding, cost records, and adversarial
      evaluator gates are all in place.
- [ ] Do not call an ops-visible feature complete unless healthy, stale,
      failed, blocked, partial, retrying, and unknown states are distinguishable.
- [ ] Do not call a slash command/MCP feature complete unless CLI, MCP schema,
      slash prompt, skill docs, package README, and verifier coverage agree.

### Proof Packet Template

Before checking off any P0/P1 item, attach or record this packet in the issue,
PR, implementation note, or final report:

- [ ] Feature name and status: Missing, Scaffold, Partial, Local Proof, Live
      Proof, Operational, or Done.
- [ ] User-visible claim in one sentence.
- [ ] Exact inputs accepted and exact outputs promised.
- [ ] Durable rows/files/remote state written.
- [ ] Failure semantics for invalid input, provider errors, partial writes,
      interrupted execution, stale credentials, and policy denial.
- [ ] Idempotency rule for repeats, duplicates, retries, and reruns.
- [ ] Security/privacy boundary: auth, policy, cost, secret redaction, prompt
      injection, SSRF/path handling, and export/backup behavior.
- [ ] Completeness measures: row counts, fixture coverage, command parity,
      source-health state, ops visibility, and live proof where relevant.
- [ ] Severe tests added, including which broken/scaffold behavior they would
      catch.
- [ ] Performance/resource budget or explicit reason no budget is needed.
- [ ] Commands run and pass/fail result.
- [ ] Adversarial review judgement by the implementer: promote, hold, or block.
- [ ] Remaining risks and next action for each risk.

## 1. Live Telegram And Mobile Channel Loop

- [ ] Live-smoke real Telegram webhook -> Cloudflare -> local drain ->
      `channel_messages` and controller route report from a fresh real Telegram
      client message.
- [ ] Add safe follow-up context carryover for authorized Telegram chats.
- [ ] Add production monitoring for Telegram webhook freshness, drain lag,
      repeated nacks, and failed delivery retries before treating Telegram as a
      critical alert path.
- [ ] Add Miniflare coverage if future local Node tests miss another
      deployed-worker failure mode.

## 2. Codex And Claude Host Integration Proof

- [ ] Fresh-thread smoke `arc` inside the Codex app.
- [ ] Live-smoke Codex plugin hooks and Claude degraded memory workflow.
- [ ] Add Codex plugin prompts or hooks for task start/finish capture where the
      host can support them.
- [ ] Add a native host adapter for Codex thread inventory if a stable API
      becomes available.
- [ ] Record an interactive MCP Inspector run against `arcwell mcp`.
- [ ] Validate Claude Desktop/Code config in an authenticated local profile.
- [ ] Live-smoke the resident Codex host-adapter flow against a disposable Codex
      thread and record the freshness/provenance behavior.
- [ ] Keep degraded/manual host-sync state explicitly labeled so stale snapshots
      cannot masquerade as live thread state.

## 3. Packaging, Release, Install, And Upgrade

- [ ] Publish signed or checksummed GitHub release artifacts.
- [ ] Render and test a Homebrew formula/tap from real release artifact
      checksums.
- [ ] Run Linux `systemctl --user` live proof for install, status, restart,
      journal/logs, strict doctor, and uninstall.
- [ ] Add release gating so archive traversal, checksum mismatch, interrupted
      upgrade, stale `PATH`, old schema, service rendering, plugin PATH, and
      uninstall preservation all fail closed before publication.
- [ ] Document the exact public install, upgrade, backup-before-migration,
      service, plugin, and uninstall paths after the public artifact smoke
      passes.

## 4. Ops, Monitoring, And Human Control Surface

- [ ] Decide whether to keep server-rendered HTML or split out a small frontend
      package before adding richer controls.
- [ ] Add manual job requeue/cancel controls only after safe public core APIs
      exist; do not fake unsupported remediation.
- [ ] Add safe controls for retry delivery, apply/reject candidate, run doctor,
      create/verify backup, drain once, and inspect policy denial reasons.
- [ ] Add charts and stale-state summaries for queue age, failed deliveries,
      backup freshness, source health, credential health, costs, work runs, and
      pending reviews.
- [ ] Add live-provider probe summaries to ops only where probes are cheap,
      safe, redacted, and policy/cost aware.
- [ ] Keep Obsidian/Markdown as the wiki editing surface; do not duplicate wiki
      authoring unless needed.

## 5. Proactive Delivery: Email, Telegram, Librarian, And X

- [ ] Implement the unified knowledge pipeline plan in
      `docs/unified-knowledge-pipeline-implementation-plan.md`. This is the
      source-agnostic architecture for watch sources -> source cards -> events
      -> clusters -> editorial decisions -> research fanout -> rich wiki pages
      -> digest/report delivery. Do not mark it operational until the proof
      packet shows scheduled real or copied-production source ingestion from at
      least three source families, durable events/clusters/decisions,
      source-backed wiki writing, digest routing, external delivery ledger,
      ops visibility, and wall-clock recurrence without manual intervention.
      First local-proof slice now exists in `arcwell-core`: durable
      `knowledge_events`, `knowledge_event_sources`, `knowledge_clusters`,
      `knowledge_editorial_decisions`, and `knowledge_reports`, with source-card
      evidence gates, link-dump report rejection, ops snapshot visibility, and
      severe tests. Remaining work is the adapter contract, entities/relations,
      investigation jobs, wiki/digest worker integration, live provider proofs,
      scheduled recurrence, and ops UI controls.
- [ ] Complete the Arcwell X anti-mirage plan in
      `docs/arcwell-x-architecture-implementation-plan.md` before marking X
      beyond `Partial`.
- [ ] Keep `arcwell-x` status honest: every checked X item must state whether
      it is only local proof, copied-home live proof, real-home live proof, or
      operational scheduled proof.
- [ ] Treat the 2026-06-25 X knowledge-system proof as the current baseline, not
      the finish line. Latest repeatable proof saw 1,010 bookmark collections,
      5,228 X source cards, three deterministic clusters (`model-launches`,
      `computer-use-agents`, `agent-tooling-mcp`), editorial decision
      `xed-17b46142bbec4dd7`, editorial wiki page
      `x-knowledge-x-bookmark-trend-model-launches-for-agents-mcp-and-coding-tools-and--66364db3`,
      editorial-created digest candidate
      `cd8af9bc-97b8-4b5c-9b92-905e2f127470`, active alert schedule
      `a21a483d-c0b4-40ea-b0fc-66b457b8cbc1`, controlled-provider delivery
      `ef5c0e93-c191-4b50-bca5-d8e3b7096341`, and readable email report text
      instead of the old internal metadata/link dump. Local tests now prove
      resident `x_bookmarks` watch-source scheduling, worker bookmark import,
      completeness metadata, source-health backoff, due-time recurrence after
      `next_run_at`, `/ops/ui` X cluster/editorial visibility, and authenticated
      policy-checked X controls for schedule/enqueue/run-worker. Real home also
      has a prior live Cloudflare email delivery and a running resident worker
      heartbeat. Repeatable proof `scripts/x-knowledge-system-proof` passed at
      `.arcwell-dev/proofs/x-knowledge-system-proof-20260625T114921Z-87295/artifacts/proof-packet.json`,
      proving copied real X corpus -> radar scoring -> non-authorizing model
      overlay -> durable multi-cluster buckets -> editorial wiki quality gate ->
      editorial-created digest candidate -> reviewed scheduled delivery ->
      duplicate suppression and ops visibility. Remaining work before calling
      this operationally done: refresh/reauthorize X OAuth credentials for
      current live provider proof, semantic/model-assisted topic clustering,
      scheduled real external recurrence proof over wall-clock time, and
      multi-day monitoring.
- [ ] Expand canonical X storage beyond the first local-search stage. Local
      JSON/X API imports now write conversation/reply/quote/retweet fields and
      `x_tweet_refs`; local archive import now records an `import_archive`
      sync run for supported tweets/bookmarks/likes; remaining work is
      source-health summaries by account/stream, portable/scoring sync ledgers,
      and rollback tests for projection/FTS failures.
- [ ] Add a canonical X proof packet for Phase 1: schema version, migration
      fixture, backfill counts, canonical/compatibility count parity, source
      card/wiki projection links, FTS row count, and rollback behavior.
- [ ] Add canonical X dual-write tests that would fail if a command only writes
      `x_items` and does not write canonical profiles/tweets/edges/collections.
- [ ] Add canonical X read-path tests that would fail if CLI reads canonical
      rows but MCP, slash commands, package docs, or report generation still use
      stale compatibility assumptions.
- [ ] Extend X sync-run ledgers beyond the implemented import-json,
      import-archive, recent-search, bookmark-import, and watch-source monitor
      streams to portable import and future scoring jobs before any of those
      streams are described as operational.
- [ ] Add X sync-run tests for started/completed/failed/superseded statuses,
      count accuracy, previous/new cursor recording, cost decision linkage, and
      redacted error storage.
- [ ] Add X cursor-safety tests for malformed provider payloads, all rows
      rejected, duplicate newest ids, older newest ids, source-card projection
      failure, FTS failure, process interruption, and quota/rate limits.
- [ ] Extend X repair beyond the implemented local CLI/MCP layer: `x stats`,
      `/ops`, `/ops/ui`, `ops_snapshot`, and strict `doctor` now surface FTS
      drift, failed projections, non-healthy X source-health, and failed X sync
      runs; `x repair-projections` / `x_repair_projections` repair missing or
      failed source-card/wiki projections idempotently; next add explicit
      doctor/ops repair guidance or authenticated ops controls for projection
      repair and FTS rebuild.
- [ ] Extend X archive import beyond the implemented local tweets/bookmarks/
      likes MVP. Current `x import-archive` / `x_import_archive` supports local
      directories or zip files, explicit `--select`, JavaScript wrapper parsing
      as data, zip-slip rejection before writes, file/byte limits, no network
      calls, `import_archive` sync runs, canonical writes/FTS/projections, and
      MCP round-trip tests. `x discover-archives` / `x_discover_archives` now
      performs no-write, shallow candidate discovery with bounded ZIP member
      inspection, unsafe-member and unsupported-slice warnings, and MCP
      round-trip coverage. Import reports now list unsupported slice counts and
      sample files without reading private/unsupported payload bytes. Remaining
      work is old/new archive fixture corpus breadth and archive account-slice
      identity validation. Current tests also prove reimport idempotency for the
      local tweet archive path, fail malformed selected slices before writes,
      reject compressed archive bombs plus nested archives before earlier rows
      survive, skip unselected malformed/private slices without reading their
      payload text, and reject same-handle/different-author identity conflicts
      before tweet rows are written.
- [ ] Add X archive apply coverage for authored tweets, note tweets, profiles,
      followers, following, media metadata, malformed slices, richer selected
      import fixtures, and explicit proof that no secret values are read.
      Likes/bookmarks/tweets have a first local fixture path only, and selected
      tweet imports now prove unselected malformed/private slices are skipped
      without payload reads.
- [ ] Extend the implemented X URL/link index beyond the current local
      extraction and explicit expansion layers. `x extract-links` /
      `x_extract_links` index safe URL occurrences without fetching; `x links`
      / `x_links` list the local index; `x expand-links` / `x_expand_links`
      fetch indexed links through URL-ingest safety with policy/cost gates,
      redirect/private-host checks, content-type and size limits, and durable
      expansion status rows. `x report` / `x_report` now includes typed link
      expansion provenance and Markdown status lines for returned items.
      Remaining work is richer cache freshness, expansion provenance in
      digests, large-corpus performance fixtures, and downstream research/digest
      integration.
- [ ] Extend X thread expansion beyond the implemented local-only CLI/MCP
      layer. `x thread` / `x_thread` now expand already-imported conversation,
      reply, quote, and retweet refs with cycle detection, depth caps, stable
      ordering, and missing-context labels; remaining work is optional
      policy/cost-gated live mode, larger performance fixtures, archive-thread
      fixtures, and report/digest integration.
- [ ] Add X digest candidate hardening: canonical tweet/thread id linkage,
      source-card linkage, review states, score freshness, delivery-denial
      audit, delivery-attempt integration, quiet-hours schedule, and no
      model-score-only sending. Generic digest candidate creation now normalizes
      and dedupes exact same-topic/source-card sets so repeated X/watch flows do
      not inflate the queue. X monitor-created candidates now also write
      idempotent `x_projections` rows linking canonical tweet ids, source-card
      ids, and digest candidate ids, and `/ops/ui` surfaces linked X digest
      queue counts. Digest candidates now carry durable `review_status`,
      reviewer, review note, approve/reject MCP/slash surfaces, and a
      fail-closed delivery check that records policy-decision audit metadata
      while refusing unreviewed/rejected/model-score-only delivery. Approved
      Telegram delivery now writes an idempotent `digest_deliveries` ledger row,
      links the generic channel message and channel delivery attempt, records
      blocked review/policy/auth rows without provider calls, exposes MCP/slash
      delivery/list surfaces, and replays the same idempotency key without
      duplicate sends. Severe tests also prove provider failures record failed
      ledger rows with retry metadata, and monitor-created X digest candidates
      can be traced from `x_projections` to `digest_deliveries` to the channel
      delivery attempt, with ledger rows visible through ops snapshots. Email
      digest delivery now uses the same review/policy/channel-auth gated
      `digest_deliveries` ledger and generic channel delivery-attempt path. Due
      generic retries now reconcile digest ledger rows to `sent`, `failed`, or
      `dead_lettered`; score freshness, quiet-hours scheduling, and live
      external delivery proof remain open.
- [ ] Add X heuristic scoring before model scoring, with score rows as overlays,
      stale-score labels, schema-validated model output, eval fixtures,
      cost-decision rows, private-content exclusion, and proof that scores never
      mutate canonical truth or authorize delivery.
- [ ] Extend X ops/doctor visibility beyond the implemented stats/drift layer
      to source-health freshness, projection backlog repair actions, digest
      queues, credential scope/expiry detail, richer archive import run
      summaries, portable export freshness, monitor staleness, and future failed/superseded
      archive/export/scoring syncs.
- [ ] Add scheduled-backup policy for optional automatic portable X export to a
      known backup-adjacent path before backup creation.
- [ ] Add X follow graph only as snapshots/current edges/events with complete
      vs partial snapshot semantics, duplicate snapshot idempotency, account
      scoping, profile-entity extraction as data, and no silent switch to full
      following graph as the default watch seed.
- [ ] Add X media cache as metadata-first, archive-byte extraction optional,
      live fetch opt-in, media-root path safety, content-type/size/pacing/retry
      limits, dry-run, ops stats, and default portable export without bytes.
- [ ] Add X DM support only after explicit retention opt-in, default-off import,
      default-off FTS/export, prompt-injection-as-data tests, participant
      scoping, malformed event handling, and forget/retention behavior exist.
- [ ] Add X moderation/social writes only after read substrate is operational,
      with account-scoped confirmation, exact action preview, policy approval,
      audit-before-remote-call, idempotent retry, target-spoofing tests, and
      disposable-target live proof.
- [ ] Extend X worker/scheduled sync beyond implemented watch-source monitor
      jobs and locally proven `x_bookmarks` scheduled import, with
      heartbeat-specific health, bounded retries/dead letters, explicit config
      for any default schedule, live cron/callback proof, current live X OAuth
      credentials, wall-clock external recurrence proof, and delivery
      integration.
- [ ] Add X performance/stress fixtures: large archive, many duplicate tweets,
      large follow graph, FTS rebuild over a large corpus, export/import over
      large shards, bounded URL expansion, and ops UI row limits.
- [ ] Add X malicious-input corpus covering SQL-ish strings, shell metacharacters,
      markdown/HTML/script tags, prompt-injection text, control characters,
      RTL/Unicode normalization, huge strings, duplicate ids, stale cursors,
      malformed JSON/XML-ish payloads, bad URLs, and hostile filenames.
- [ ] Add X live proof discipline: rebuild fresh binary, use copied/disposable
      home when possible, distinguish app bearer from user-context OAuth,
      record scopes, inspect source-health/cursors after the smoke, redact
      artifacts, and never call local replay a live provider proof.
- [ ] Add X adversarial review report before every X phase status promotion,
      using the score rubric in the architecture plan and ending with a clear
      judgment: promote, hold, or block.
- [ ] Complete scheduled librarian digest alerts with live external delivery
      proof. Local resident-worker routing now selects already-approved
      candidates above a threshold, records durable ticks/delivery ids, suppresses
      duplicate immediate ticks, and defers active UTC quiet-hours before
      provider sends. Local severe tests also prove retry after a blocked
      delivery policy row, resume after quiet-hours deferral, failed tick
      marking when job execution errors, and generic digest candidates not
      borrowing X-only delivery policy. Manual and scheduled reviewed
      Telegram/email digest
      delivery uses policy/cost checks, recipient authorization, durable delivery
      attempts, and retry reconciliation.
      Controlled-provider email proof over copied real source cards passed at
      `.arcwell-dev/proofs/digest-email-production-proof-20260624T143355Z-46300`;
      scheduled alert controlled-provider proof over copied real source cards
      passed at
      `.arcwell-dev/proofs/digest-alert-scheduled-production-proof-20260624T201946Z-74509/artifacts/proof-packet.json`;
      real-home X manual live Cloudflare digest delivery passed for candidate
      `7cfec561-3827-417a-8e93-957ee84ff69a`; broader scheduled real external
      recurrence remains unproven.
- [ ] Add production monitoring for email ingress/outbound if email becomes a
      critical alert path.
- [ ] Add Cloudflare callback/cron event capture after edge inbox is durable and
      monitored enough for production use.
- [ ] Add model-backed interestingness for X/source/digest candidates behind
      explicit config, policy, cost gates, and eval coverage.
- [ ] Add live production proof for X/watch-source digest delivery through the
      same email/Telegram delivery-attempt infrastructure. Telegram and email now
      have review/policy/channel-auth gated, idempotent `digest_deliveries`
      ledger paths over the generic channel delivery-attempt table; scheduled
      digest alerts route approved candidates through that ledger with
      quiet-hours deferral; due generic retries reconcile digest rows. Live
      external digest-delivery proof has passed once for a real-home approved X
      candidate through Cloudflare Email; scheduled watch-source recurrence and
      Telegram live delivery remain open.
- [x] Decide and record the Reddit release-ready claim before promotion. There
      are two valid release paths, and they must not be collapsed into one:
      supervised browser-capture release versus unattended Reddit production
      source. Current release-candidate proof:
      `.arcwell-dev/proofs/reddit-browser-production-proof-20260625T064407Z-87932/artifacts/proof-packet.json`
      proves the supervised path only from a staged candidate binary. The
      original main-Chrome capture proof
      `.arcwell-dev/proofs/reddit-browser-production-proof-20260624T161717Z/artifacts/proof-packet.json`
      remains the production-data capture source. The unattended daemon/RSS proof
      `.arcwell-dev/proofs/radar-reddit-production-proof-20260624T150229Z-29771`
      remains blocked by Reddit HTTP 403 before source-card projection.
  - [x] **Supervised Browser-Capture Release definition:** Arcwell supports
        Reddit when an agent/user supplies sanitized browser-captured Reddit
        listing JSON. This is the release-ready Reddit claim: supervised and
        browser-assisted, not unattended.
    - [x] Add repeatable `scripts/reddit-browser-production-proof` so the
          current proof is not an ad hoc shell sequence. The script should use
          a disposable `ARCWELL_HOME`, accept a sanitized listing artifact,
          ingest it through `arcwell source-card ingest-reddit-browser-listing`,
          run a Reddit radar profile, summarize, audit, inspect ops, and write
          a proof packet with source-card/wiki/radar/cursor/source-health counts.
          Passing packet:
          `.arcwell-dev/proofs/reddit-browser-production-proof-20260625T064407Z-87932/artifacts/proof-packet.json`.
    - [x] Add artifact redaction gates that fail if persisted Reddit artifacts
          contain `modhash`, account-specific fields, cookies, tokens, local
          storage, browser profile paths, raw browser storage, or unredacted raw
          response payloads. Keep the current boundary: no browser cookie,
          local-storage, password, or profile database inspection. The proof
          script allow-lists persisted Reddit fields, scans JSON keys/text in
          artifacts, and records `redaction_scan_passed=true`.
    - [x] Add severe browser-listing ingestion tests for duplicate listing
          replay, malformed listing, oversized listing, empty listing, partial
          write failure, stale capture, hostile source text, unsafe URLs, and
          cursor-not-advanced-on-failure. Core Reddit severe tests cover replay,
          malformed/empty/unsafe listing failure, hostile source text as
          evidence, partial-failure no cursor/source-health advance, duplicate
          suppression, and bearer-token request plumbing. CLI/proof gates cover
          oversized listing files and stale capture artifacts.
    - [x] Decide surface parity for sanitized browser artifacts: either add
          MCP/slash/skill support for ingesting an already-sanitized browser
          artifact, or explicitly document that this remains CLI-only by design
          because capture itself belongs to the host/browser boundary. Decision:
          CLI-only for ingestion; Codex skill text now warns that Reddit
          browser-capture is supervised host/browser-supplied evidence and not
          unattended Reddit support.
    - [x] Add operator docs covering the exact capture boundary, accepted JSON
          shape, persisted fields, rejected/redacted fields, trust model,
          source-health/cursor inspection, radar-stage inspection, and proof
          artifact layout. See `docs/reddit-browser-ingestion.md`.
    - [x] Prove fresh-thread Codex plugin visibility after
          `scripts/arcwell-dev sync`; if no MCP/slash surface is added, prove
          the relevant skill/docs wording is visible and does not imply
          unattended Reddit support. `scripts/arcwell-dev sync` and
          `scripts/verify-codex-plugin-docs` pass, and the installed Codex cache
          contains the CLI-only Reddit boundary text in
          `skills/wiki-research/SKILL.md`. A live already-running thread still
          requires skill reload or a new thread, as normal for Codex plugin
          updates.
    - [x] Add a release proof packet that starts from a clean install or
          candidate binary, not just the dev checkout, and records install path,
          binary version, plugin/cache state, proof command, artifacts, and
          remaining boundaries. Candidate-binary proof passed at
          `.arcwell-dev/proofs/reddit-browser-production-proof-20260625T064407Z-87932/artifacts/proof-packet.json`;
          Arcwell has no `--version`, so the packet records binary path, SHA256,
          and CLI help output instead.
  - [ ] **Unattended Reddit Production Source definition:** Arcwell can monitor
        Reddit on schedule without a browser. This is not close yet because
        daemon/RSS remains blocked by Reddit HTTP 403 and OAuth/sanctioned API
        access is unproven.
    - [ ] Implement Reddit OAuth or another sanctioned non-browser access path
          with scoped secrets, policy/cost gates, provider error classification,
          token redaction, and refresh/revocation failure tests. Groundwork:
          `REDDIT_BEARER_TOKEN` now actually attaches bearer auth to both
          listing and bounded-comment JSON requests and has severe request
          capture coverage; this is not full OAuth, refresh, revocation, or
          sanctioned live access proof.
    - [ ] Prove daemon-side Reddit fetch writes source cards, bounded comments,
          cursor, source-health, radar items/FTS/scores, summary, and audit-ok
          output on real Reddit data in a disposable or copied home.
    - [ ] Add scheduled worker proof with retries, backoff, duplicate
          suppression, stale/blocked/failed/healthy ops states, and no cursor
          corruption across failures or partial writes.
    - [ ] Add bounded top-comment capture for daemon/browser paths or
          explicitly keep comment capture out of the release claim; do not imply
          recursive comment coverage unless it is proven.
    - [ ] Run multi-source production breadth proof over multiple subreddits,
          multiple sorts, and enough volume to catch duplicate, cursor,
          ranking, category-balance, and source-health problems.
    - [ ] Wire Reddit through digest candidate creation, model-backed synthesis
          quality gates, review approval, quiet-hours routing, and live external
          delivery with delivery-attempt ledger proof.
    - [ ] Prove long-running service behavior, not just foreground CLI:
          resident worker/service scheduling, restart recovery, stale-source
          visibility, retry reconciliation, and release-candidate binary proof.
- [ ] Extend radar live execution to authenticated X watch/recent-search data
      with copied/disposable-home source-health/cursor proof before promotion.
      `scripts/radar-x-production-proof` now provides a guarded disposable-home
      harness with OAuth refresh, source-health/cursor, audit, summary, ops,
      artifact redaction checks, and a blocked proof packet when live auth
      fails. Latest local run
      `.arcwell-dev/proofs/radar-x-production-proof-20260624T150151Z-29198`
      is not a pass: OAuth refresh failed, app-bearer fallback returned 401,
      and the proof packet kept existing local X projection separate from
      current authenticated live fetch proof.
- [ ] Add live production delivery proof, live external scheduled
      delivery/service proof, production cross-channel delivery proof,
      arbitrary/model-generated taxonomy quality review, operational wall-clock
      seven-day source-quality decay proof, broader ops controls, and status
      promotion only after real-data gates pass. Fresh live OpenAI model-score
      proof passed at
      `.arcwell-dev/proofs/radar-model-score-production-proof-20260624T150127Z-28610/artifacts/proof-packet.json`;
      it is a non-authorizing scoring overlay proof.
- [x] Add bounded model-written synthesis quality proof over real production
      Arcwell source-card data. Fresh live OpenAI proof passed at
      `.arcwell-dev/proofs/research-synthesis-completion-proof-20260624T153639Z-75610/artifacts/proof-packet.json`:
      six copied production source cards, 12 structured claims, live drafter,
      citation verifier with zero unsupported count/rate, adversarial evaluator
      with zero blocking issues, and final run-scoped audit ok. This does not
      replace a saturated fresh report acceptance proof or digest-specific
      ranking/synthesis quality.

## 6. Deep Research Quality And Host-Native Execution

- [ ] Add page expansion that actively gathers related docs/blogs/repos/social
      sources before writing a topic page.
- [ ] Implement iterated epistemic convergence from
      `docs/iterated-epistemic-convergence-design.md`. Do not mark complete
      until schema, CLI, MCP, skill docs, severe tests, full proof runs, report
      judgments, `STATUS.md`, and this file agree.
- [ ] Add convergence run config: max iterations, wall time, cost cap, source
      cap, provider-call cap, freshness needs, privacy/no-write flags, and
      stop-rule serialization.
- [ ] Severe-test convergence run config with missing limits, huge limits,
      negative/NaN/infinite values, no-write propagation, user stop before the
      next expensive action, and long-run requests without approval.
- [ ] Severe-test iterations for cross-run artifact rejection, duplicate
      iteration indexes, failed iteration preservation, long error redaction,
      parent lineage validation, database reopen, and 1000-iteration listing.
- [ ] Severe-test statements with empty/overlong text, invalid enums,
      invalid confidence, cross-run evidence ids, duplicate stable keys,
      prompt-injection text, HTML/Markdown escaping, SQL metacharacters,
      Unicode spoofing, and cross-run statement attachment attempts.
- [ ] Build the statement compiler from source cards, claims, clusters, skeptic
      notes, and prior iterations, preserving temporal scope and filtering
      source/corpus bookkeeping and generated-output recursion.
- [ ] Severe-test the statement compiler with compound-statement splitting,
      metadata-only corpora, conflicting claims, unsupported model prose,
      currentness-sensitive statements, SEO spam, vendor-only evidence, and
      contradictory benchmark claims.
- [ ] Add `research_challenges` schema plus deterministic challenge templates
      by statement type/domain, expected-information-gain ranking, required
      source-family output, and challenge lifecycle states.
- [ ] Severe-test challenges for missing/cross-run statement ids, unknown
      challenge types, missing severity, empty search plans for high-severity
      challenges, prompt injection that tries to waive challenges, and duplicate
      ids.
- [ ] Add challenge-ranked disproof retrieval that records host-native search
      proof before reliance, falls back to configured provider search only
      through policy/cost gates, links search/source cards to challenges, and
      records blocked searches honestly.
- [ ] Severe-test disproof retrieval with known contradiction discovery,
      duplicate source novelty suppression, blocked-search unresolved status,
      low-reliability contradictions, SSRF URLs, redirects to localhost or
      metadata IPs, search-snippet prompt injection, and local-file URL abuse.
- [ ] Extend evidence extraction/claim ingest so source cards, claims, document
      spans, tables, and table cells can be linked to challenge/disproof ids
      with same-run validation and extractor warnings.
- [ ] Severe-test iterative evidence extraction with malformed claim JSON,
      uncertainty-loss rejection, cross-run anchors, nonexistent span/table/cell
      anchors, unsupported formats, PDF prompt injection, XLSX/CSV formula
      payloads, oversized files, and report-rendering injection.
- [ ] Severe-test disproof verdicts with direct contradictions, partial-scope
      mismatches, irrelevant evidence, missing evidence, stale vs official
      corrections, generated-synthesis misuse, low-quality source overreach,
      and numeric unit/date mismatches.
- [ ] Severe-test revisions so refuted statements cannot remain final without
      caveat/replacement, rewording cannot hide a refuted stable key,
      confidence cannot increase after weakening without new evidence, and
      dropped statements remain visible in appendices.
- [ ] Performance-test convergence metrics with 10,000 candidate sources,
      2,000 source cards, 50,000 claims, 5,000 statements, 20,000
      challenges/disproofs, 100 iterations, large report appendices, and
      bounded status reads.
- [ ] Add `research_report_judgment` artifacts written by Codex/main agent with
      scores for source coverage, primary-source depth, citation support,
      contradiction handling, uncertainty preservation, narrative clarity,
      decision usefulness, novelty/design quality, safety reasoning, and
      cost/time proportionality.
- [ ] Add long-running convergence execution with resumable worker state,
      leases, heartbeats, idempotency keys, progress snapshots, cost
      reservations, stale-lease reclaim, dead-letter behavior, and user stop.
- [ ] Severe-test long-running execution with crashes after statement compile,
      challenge generation, provider call, and revision; duplicate-write
      prevention; stale lease reclaim; stop during sleep/search; retry-storm
      cost blocking; and bounded worker memory over many iterations.
- [ ] Update `$deep-research` skill and role prompts for position compiler,
      red teamer, disproof scout, verifier, reviser, convergence auditor,
      output-artifact requirements, host-search proof per challenge, and
      stale-schema reload caveats.
- [ ] Severe-test Codex/subagent orchestration with completed roles missing
      output artifacts, cross-run ids, source prompt injection, missing caveats,
      unavailable host search, accepted/rejected proposals, and a fresh Codex
      thread live smoke.
- [ ] Severe-test CLI/MCP parity with unknown ids, malformed JSON, missing
      required fields, large bounded responses, stale schema detection, and
      equivalent CLI/MCP state transitions.
- [ ] Add invention/new-tech branch handling for `design_proposal` statements,
      prior-art challenges, feasibility challenges, threat-model challenges,
      benchmark/experiment-plan artifacts, "not proven" report language, and
      explicit graduation criteria.
- [ ] Severe-test invention runs with known prior art, feasibility gaps,
      security flaws, proposed-design/fact separation, missing experiment
      evidence, and report language that refuses "proven" claims.
- [ ] Run an accepted live host-search technical proof on image compression
      algorithms with papers, codec docs, benchmarks, dissenting analyses,
      numeric/table anchors or caveats, and model-backed report acceptance.
      Latest saturated proof home
      `.arcwell-dev/proofs/deep-research-production-proof-20260624T170158Z`
      ran 12 live Brave queries, deduped 81 candidates, linked 121 source cards,
      promoted 40 full-source cards, recorded 108 host-search proofs, executed
      90 exact challenge host-search tasks with selected results, ran 4 worker
      convergence jobs, and recorded live OpenAI model-backed editorial. It
      correctly remains blocking with `closure_status: stopped_incomplete`,
      unaccepted model-backed judgment, 2 unknown high-impact fact checks, and a
      rejected final report judgment. A follow-up close-loop on the same proof
      after the stale-editorial gate fix re-ran OpenAI over the cleaned state
      with 0 pending search tasks, 101 valid citations, and 20 unsupported
      claims. Earlier saturated proof home
      `.arcwell-dev/proofs/deep-research-production-proof-20260623T155121Z`
      produced 12 live Brave queries, 131 deduped candidates, 80 linked source
      cards, 80 structured claims, 18 host-search records, `closure_status:
      closed`, and live OpenAI verifier/evaluator records. It correctly remains
      blocking because the adversarial evaluator found snippet-derived
      medium-confidence evidence, unsupported/overreaching conclusions, missing
      caveats, and 474 pending challenge-search tasks. Earlier failed attempts
      caught and fixed HTTPS URL filtering, source-card-vs-structured-claim
      confusion, active fact-check recursion over generated convergence report
      sections, a too-low convergence source cap, model-backed judgment
      overwrite, bodyless structured provider responses, pending-search prompt
      scoring, evaluator routing over synthesized score artifacts, proof-script
      omission of close-loop editorial flags, stale rejected-editorial rerun
      gating, title/page-dump narrative promotion, and provider-search
      URL-ingest jobs that wrote wiki pages without promoting them into
      run-linked source-card/claim evidence.
- [ ] Run live market/ecosystem proof on London AI startups with official,
      company, funding, job, news, and social source families, hype downgrades,
      currentness labels, contradiction handling, and report judgment average
      score at least 4.
- [ ] Run live security/architecture proof on safe cloud code execution with
      standards, sandbox literature, policy engines, compiler/static-analysis
      papers, platform docs, threat-model challenges, and no unsafe overclaims.
- [ ] Run invention/new-tech proof proposing a verified cloud code execution
      architecture, with prior-art search, feasibility attacks, threat-model
      review, experiment plan, and report separation of facts/proposal/risks.
- [ ] Run long-running resume proof that simulates interruption, resumes
      without duplicate statements/challenges/disproofs, preserves cost/time
      limits, and honors user stop before the next expensive action.
- [ ] Maintain a production-readiness scorecard for iterated convergence:
      data model, CLI/MCP parity, skill clarity, statement quality, challenge
      quality, disproof retrieval, revision correctness, stop rules,
      fact-checking, report quality, malicious input resistance, invalid input
      handling, restart/resume, performance, cost/policy, live proof coverage,
      and documentation honesty. No category below 4 before production-ready.
- [ ] Add native host-search pathway for Claude where available and finish
      full-report host-search orchestration for Codex/OpenAI.
- [ ] Regenerate fresh hundred-source reports through the narrative-filtered
      compiler and evaluate them for analyst-grade judgment quality, not just
      structural completion.
- [ ] Expand difficult-document fixture coverage for PDFs, XLSX, precise table
      extraction, formula/cell handling, and publication-grade citation links.
      Current local coverage includes CSV/XLSX formulas, malformed inputs, PDF
      heuristic tables, and a severe wrapped-header/irregular-column/footnoted
      PDF table fixture that lowers confidence and preserves footnote refs, plus
      a severe XLSX fixture for hidden/very-hidden sheet skipping,
      merged-cell metadata/lowered confidence, and date-time normalization; the
      broader external difficult-document matrix remains open.
- [ ] Run fresh reference-topic deep-research live runs after host search,
      subagent orchestration, and provider-backed evals are proven.
- [ ] Add browser-rendered JavaScript readability extraction for pages that
      require rendering, including actual browser capture orchestration,
      screenshot/page-snapshot artifact storage, blocked-state reporting, and
      live proof against JS-heavy pages.

## 6A. Qualified Commerce Research

- [x] Move qualified commerce from `Scaffold` through `Partial/Local Proof` to
      bounded `Partial/Production Data Proof` only for the proved slice: durable
      ledger, host-supplied rendered-page checks, source-card linkage,
      structured extraction, context/report compilation, CLI/MCP surfaces,
      capability disclosure, severe local/MCP tests, and a two-item live M&S
      UK proof packet.
- [x] Keep qualified commerce below operational/full-autonomous status while
      promoting only the bounded production-data slice that has proof: host
      browser capture replay, context packet compilation, report rendering,
      source-card linkage, structured extraction, and a two-item live UK M&S
      proof packet.
- [x] Add local host-supplied rendered-page commerce checking: rendered DOM/page
      text, URL after redirects, timestamp, visible title, selected variant,
      availability signal, screenshot/page snapshot provenance, blocked-state
      reporting, source text as untrusted evidence, and conservative exact
      variant proof classification.
- [x] Extend rendered-page commerce extraction with structured price/currency,
      delivery/shipping caveat extraction, source-card linkage, and bounded
      live browser proof packets.
- [x] Add commerce candidate and availability-proof tables with same-run
      validation, exact variant keys, checked timestamps, proof methods,
      confidence/caveats, and CLI/MCP read/write surfaces.
- [x] Add qualified-commerce report rendering over the local ledger without
      allowing unverified candidates into the main recommendation list.
- [x] Add a bounded private context packet compiler for commerce runs with raw
      private data excluded from public report/source-card outputs by default.
- [ ] Connect the context packet compiler to Arcwell memory/profile,
      Garderobe, and later approved browser history, screenshots,
      spreadsheets, and emails instead of only recording redacted facts supplied
      by the calling agent/user.
- [ ] Implement the first `$qualified-commerce-research` skill profile for UK
      fashion retail: broad search, 20+ target qualified candidates when the
      market supports it, exact size availability proof, comfort/style/quality
      scoring, review evidence where available, and disqualified near-miss
      reporting. A bounded 2026-06-25 M&S UK loafer proof passed with 24 exact
      UK 8.5/8½ recommendations at
      `.arcwell-dev/proofs/commerce-uk-fashion-20-live-20260625T052635Z-94892/harness/artifacts/proof-packet.json`;
      this does not yet prove autonomous cross-retailer discovery or denim-shirt
      breadth.
- [ ] Severe-test commerce research with disabled/crossed-out sizes, wrong-size
      availability, variant-specific price changes, region/shipping caveats,
      JS-only pages, sold marketplace listings, size-system ambiguity, stale
      search results, retailer and wardrobe prompt injection, blocked pages,
      private-context leakage, and unverified candidates appearing in the main
      recommendation list.
- [x] Add a preserved commerce proof packet script that exits non-zero when
      blockers remain and records feature status, user-visible claims, request,
      privacy/context sources, search providers, cost/policy decisions, raw and
      checked candidate counts, availability-proof methods, blocked/unknown/
      disqualified counts, artifacts, audit result, surfaces exercised, and
      promotion judgment. The first local replay proof for the harness passed
      with `scripts/commerce-research-production-proof --sample
      --target-qualified 2 --min-recommended 2`; production-data manifest
      gates remain separate.
- [x] Run a bounded live proof packet for one UK loafer in UK 8.5/8½ and one
      denim shirt in 2XL with browser-rendered M&S pages, exact variant
      availability, source cards, context packet, and compiled report.
- [ ] Run preserved broad live proofs for UK loafers in UK 8.5 and a denim
      shirt search, with browser-verified availability, context-derived
      preferences, review evidence where available, disqualified near misses,
      and final report audit before claiming the workflow works end to end. The
      loafer side has a bounded M&S-only pass packet; the denim-shirt broad
      proof and multi-retailer breadth remain open.
- [ ] Add an autonomous 20+ shopping manifest generator for UK fashion that
      drives configured Brave/Perplexity/OpenAI search, dedupes retailer pages,
      records search/provider proof, queues browser checks, and feeds
      `scripts/commerce-research-production-proof --manifest ...` until the
      report has at least 20 exact-variant recommendations or an explicit market
      scarcity blocker.
- [ ] Prove marketplace coverage with at least one eBay and one Vinted-style
      listing path when marketplaces are allowed, including sold/ended listing
      rejection, condition/seller fields, short-lived freshness labeling, and
      source-card/report separation from standard retailer stock. A narrow
      Vinted-style exact-size marketplace coverage proof passed at
      `.arcwell-dev/proofs/commerce-marketplace-live-20260625T053405Z-39997/harness-vinted-coverage/artifacts/proof-packet.json`;
      the stricter eBay+Vinted gate is still blocked because the live eBay
      fetch returned 403/no exact evidence at
      `.arcwell-dev/proofs/commerce-marketplace-live-20260625T053405Z-39997/harness/artifacts/proof-packet.json`.
- [ ] Prove logged-in Chrome-profile coverage in a supervised run that requires
      user/browser consent, records `chrome_profile` verification methods
      without copying private page data into public artifacts, and passes
      `scripts/commerce-research-production-proof --manifest ... --require-chrome-profile`.
      Current release gate is blocked at
      `.arcwell-dev/proofs/commerce-uk-fashion-20-live-20260625T052635Z-94892/harness-chrome-profile-gate/artifacts/proof-packet.json`
      because no authenticated Chrome-profile availability check was proven.
- [ ] Add rental and flight domain profiles only after generic field extraction
      supports their exact availability semantics: rental move-in/location/
      price/deposit/contact checks, and flight route/date/fare/baggage/refund
      checks. Each domain needs its own manifest proof and no cross-domain
      recommendation claim before proof. Current rental/travel release gates
      are blocked local replay packets under
      `.arcwell-dev/proofs/commerce-release-blocked-gates-20260625T053438Z-41247/`.
- [ ] Add ops/recovery requirements before any operational claim: worker leases
      or resumable state for long runs, retry/dead-letter behavior, source or
      provider health, cost caps, idempotent reruns, user-stop handling, and ops
      visibility for healthy, stale, blocked, failed, partial, retrying, and
      unknown states.
- [ ] Promote to operational only after a worker-drained commerce proof records
      queued discovery, queued browser checks or host-capture handoff,
      resumable report compilation, retry/dead-letter behavior, cost/policy
      decisions, and ops visibility, then passes
      `scripts/commerce-research-production-proof --manifest ... --require-worker-proof`.
      Current worker release gate is blocked at
      `.arcwell-dev/proofs/commerce-release-blocked-gates-20260625T053438Z-41247/worker/artifacts/proof-packet.json`
      because no real worker-drained commerce run has produced passed worker
      proof metadata.

## 7. Memory, Work Graph, And Procedural Retrieval Loop

- [ ] Add consolidation job that can surface unresolved risks, recurring
      failures, stale runs, pending follow-ups, and reusable lessons.
- [ ] Add optional model-backed procedure extraction behind explicit config and
      cost policy.
- [ ] Add plugin prompts that retrieve approved procedures before relevant
      tasks.
- [ ] Live-smoke Codex/Claude procedure retrieval in a host task and prove the
      procedure is retrieved because of task relevance, not manual prompting.
- [ ] Add human review UI for memory, procedure, and project-status candidates.
- [ ] Add live model-backed memory extraction quality evals with explicit
      provider/cost opt-in.
- [ ] Implement retained-backup erasure or rotation for forgotten memory data,
      or keep the limitation visible in strict doctor and ops until implemented.

## 8. Policy, Cost, Secrets, And Provider Safety

- [ ] Inventory every sensitive operation in CLI, MCP, worker jobs, HTTP, edge
      drain, memory, project, channel, source ingestion, and provider adapters.
- [ ] Add missing policy guards found by the sensitive-operation inventory
      before credentials, provider calls, local mutation, worker enqueue, or
      outbound delivery.
- [ ] Record provider-reported actual costs where provider APIs return reliable
      usage/cost data.
- [ ] Add provider-specific live credential probes for configured providers
      without leaking secret values.
- [ ] Add provider-side revocation/rotation helpers where provider APIs make
      that safe and useful.
- [ ] Add scheduled credential rotation reminders and stale-scope warnings.
- [ ] Add ops UI burn-down and override controls for budgets only after
      idempotency, policy, and audit behavior are tested.

## 9. Backup, Forget, Recovery, And Retention

- [ ] Add scheduled local backup jobs through the worker/service.
- [ ] Add encrypted backup archive support and key-management documentation.
- [ ] Add off-machine backup target configuration with at least one tested
      target.
- [ ] Add automated restore drills into disposable homes and expose last drill
      result in ops/doctor.
- [ ] Add retained-backup erasure or rotation implementation for forget
      requests and document exact remaining limits.
- [ ] Add ops controls for create backup, verify backup, and run restore drill
      once safe action APIs exist.

## 10. Garderobe Deployment And Provenance Boundary

- [ ] Import the current live Garderobe deployment config into ignored local
      files such as `packages/arcwell-garderobe/wrangler.live.jsonc` without
      committing real D1/KV ids, owner email, route, or secrets.
- [ ] Preserve existing MCP connector compatibility while another agent is
      connected: keep `/mcp`, `/authorize`, `/token`, `/register`, S256 PKCE,
      scopes `wardrobe.read` / `wardrobe.write`, and MCP server name
      `garderobe` stable until deliberate migration/re-authorization.
- [ ] Run guarded read-only live smoke with the approved deployed Garderobe base
      URL.
- [ ] Add authenticated/write-capable Garderobe MCP live evidence using
      disposable fixture rows or staging data, not private wardrobe seed data,
      and do not clear OAuth KV or force the connected host to reconnect.
- [ ] Record a host OAuth/MCP handshake proof if Garderobe is meant to be used
      from Claude/Codex directly.
- [ ] Resolve and document top-level license/provenance for vendored Garderobe
      code before public redistribution.
- [ ] Keep Garderobe inventory out of Arcwell memory/profile/wiki by default and
      add tests for explicit opt-in sync only.

## 11. External Assistant Utilities

- [ ] Decide whether TIDAL control should remain a Codex plugin skill/script or
      be promoted to a durable Arcwell CLI/MCP package with policy gates,
      tests, ops visibility, and documented live-smoke expectations.
- [ ] If promoted, add explicit confirmation/policy handling for destructive
      TIDAL actions such as deleting playlists, removing playlist items, or
      unfavoriting collection items.
- [ ] Capture live LUMIN P1 device XML/service descriptors, then decide whether
      `lumin-control` should remain a Codex plugin skill/script or become a
      durable Arcwell CLI/MCP package with policy gates and live-smoke
      expectations.
- [ ] If promoted, add stable tests and policy handling for LUMIN writes such as
      standby, source/input selection, volume changes, and playlist mutation.

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
