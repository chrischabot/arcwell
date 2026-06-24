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
- [ ] Add browser validation for the richer current `/ops/ui` on desktop and
      mobile.
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

- [ ] Complete the Arcwell X anti-mirage plan in
      `docs/arcwell-x-architecture-implementation-plan.md` before marking X
      beyond `Partial`.
- [ ] Keep `arcwell-x` status honest: every checked X item must state whether
      it is only local proof, copied-home live proof, real-home live proof, or
      operational scheduled proof.
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
- [ ] Add X source-health tests for healthy, stale, rate_limited,
      auth_failed, policy_denied, projection_failed, partial, and unknown states.
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
- [x] Add canonical X profile identity alias/conflict storage: same immutable
      `x_author_id` across handle renames preserves one profile with alias
      history, while same-handle/different-author imports record a conflict and
      reject the item before tweet/source-card/projection writes.
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
- [x] Add X research brief generation only when every claim/quote links to
      canonical tweet ids or source cards, empty evidence fails honestly,
      no-write mode performs no writes, and prompt-injection tweets remain
      quoted evidence. Implemented as local-only `x research` / `x_research`
      with severe core tests for empty/unprojected/thread-context failure and
      no-write prompt-injection rendering, plus MCP round-trip coverage.
- [ ] Add X digest candidate hardening: canonical tweet/thread id linkage,
      source-card linkage, review states, score freshness, delivery-denial
      audit, delivery-attempt integration, quiet-hours schedule, and no
      model-score-only sending. Generic digest candidate creation now normalizes
      and dedupes exact same-topic/source-card sets so repeated X/watch flows do
      not inflate the queue.
- [ ] Add X heuristic scoring before model scoring, with score rows as overlays,
      stale-score labels, schema-validated model output, eval fixtures,
      cost-decision rows, private-content exclusion, and proof that scores never
      mutate canonical truth or authorize delivery.
- [ ] Extend X ops/doctor visibility beyond the implemented stats/drift layer
      to source-health freshness, projection backlog repair actions, digest
      queues, credential scope/expiry detail, richer archive import run
      summaries, portable export freshness, monitor staleness, and future failed/superseded
      archive/export/scoring syncs.
- [ ] Add X browser validation for ops UI: hostile tweet/profile/link/error
      strings are escaped, token-like strings are redacted, dense tables do not
      clip/overlap on desktop/mobile, empty states are clear, and POST controls
      require auth/origin/CSRF/idempotency/policy.
- [x] Add X portable export/import/validate for canonical tweet rows with
      deterministic JSONL shards, manifest hashes, row counts, token-like
      content scan, malformed JSONL failure, idempotent import, unsafe shard
      path rejection, FTS/cache exclusion, and raw DM exclusion by default.
      Current tests prove disposable restore searchability and MCP round trip;
      broader entity coverage remains separate from this MVP.
- [x] Add X backup/recovery drill integration proving portable export freshness
      is visible in `x stats`, `/ops`, `/ops/ui`, `ops_snapshot`, strict
      `doctor`, and backup manifests. Current backup manifests explicitly state
      that SQLite backups contain canonical X rows while portable bundles are
      separate unless exported/stored deliberately; disposable restore tests
      prove restored canonical X rows remain searchable with source-card links.
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
- [x] Add X worker/scheduled watch-source monitor parity: due `x_handle`
      sources enqueue executable per-handle monitor jobs with shared manual
      monitor implementation, job input validation, policy/cost guards,
      source-health, `watch_monitor` sync runs, digest candidates, cursor
      safety, and policy-denied no-write tests.
- [ ] Extend X worker/scheduled sync beyond implemented watch-source monitor
      jobs, with heartbeat-specific health, bounded retries/dead letters,
      explicit config for any default schedule, live cron/callback proof, and
      delivery integration.
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
- [ ] Wire email/librarian digest delivery with schedule, threshold, quiet
      hours, dedupe, policy/cost checks, recipient authorization, delivery
      attempts, and retry behavior.
- [ ] Add production monitoring for email ingress/outbound if email becomes a
      critical alert path.
- [ ] Add Cloudflare callback/cron event capture after edge inbox is durable and
      monitored enough for production use.
- [ ] Add model-backed interestingness for X/source/digest candidates behind
      explicit config, policy, cost gates, and eval coverage.
- [ ] Add delivery routing for X/watch-source digest candidates through the same
      email/Telegram delivery-attempt infrastructure.
- [x] Add Horizon-inspired radar substrate with durable profiles, runs,
      normalized source-card-backed items, radar FTS, heuristic score overlays,
      CLI/MCP run/stage/audit surfaces, and severe local-proof tests for
      unsupported selectors, FTS drift, unscored items, provenance links, and
      prompt-injection source text.
- [x] Extend radar projection from local source-card query into existing
      RSS/GitHub/arXiv/X source-card families with copied-home production-data
      proof and unsupported-selector audit visibility.
- [x] Extend radar from projection over existing source cards into opt-in
      foreground live RSS, GitHub, and arXiv adapter execution with
      source-health/cursor safety, CLI/MCP/slash surfaces, severe failure tests,
      and disposable-home production-data proof.
- [x] Extend radar live execution to Hacker News top/frontpage stories through
      the official Firebase API with bounded top-level comment capture,
      source-health/cursor safety, watch-source enqueue support, severe tests,
      and disposable-home production-data proof.
- [x] Extend radar/source-card live execution to Reddit with JSON listing fetch,
      bounded top-comment capture, RSS fallback that does not claim comments,
      source-health/cursor safety, watch-source enqueue support, and severe
      policy/fallback/source-card tests.
- [ ] Add Reddit production-data proof through OAuth or another sanctioned
      access path; current anonymous Arcwell binary proof is blocked by Reddit
      HTTP 403 even when curl can intermittently read RSS.
- [ ] Extend radar live execution to authenticated X watch/recent-search data
      with copied/disposable-home source-health/cursor proof before promotion.
- [x] Add radar exact URL/source-native dedupe groups with preserved source
      evidence, duplicate score statuses, audit drift checks, schema-migration
      coverage, severe local tests, and copied-home production-data proof over
      2,500 real source cards.
- [x] Add deterministic radar Markdown summary artifacts over selected scored
      items, CLI/MCP/slash surfaces, no-delivery/no-source-evidence boundaries,
      severe tests, and copied-home production-data proof.
- [x] Add local queued radar execution with `radar_enqueue` /
      `arcwell radar enqueue`, `worker run-once` processing, result JSON, MCP
      round trip, severe source-card success proof, provider-denial blocked-run
      proof, source-health/cursor assertions, and invalid-enqueue rejection.
- [x] Run worker-drained production-data radar proof in a disposable home:
      `scripts/radar-worker-production-proof` preserved
      `.arcwell-dev/proofs/radar-worker-production-proof-20260624T082527Z-74723`,
      queued `radar_run`, drained it through `worker run-once`, completed RSS,
      GitHub-owner, arXiv, and Hacker News adapter jobs, wrote 45 source
      cards/wiki pages/radar items/FTS rows/scores, recorded four healthy
      source-health rows and four cursors, selected 30 items, passed
      `radar audit`, and wrote a deterministic non-delivery summary.
- [x] Add local radar score freshness/source-health scoring adjustments and
      source-quality window records with audit coverage and direct severe-test
      assertions over source-card projection runs.
- [x] Add local deterministic radar source/category balance caps via explicit
      profile metadata, with quota rejection statuses that keep source-card
      evidence inspectable and severe tests for malformed caps, source
      dominance, category dominance, and audit-clean readback.
- [x] Add radar source-quality ops visibility in `ops_snapshot` and `/ops/ui`
      with non-healthy health warnings, summary scoring, filtered rows, and
      severe HTML escaping coverage.
- [x] Add local radar source-quality trend/ranking over durable historical
      windows, with bounded CLI/MCP/slash surfaces and severe tests for thin
      history filtering, decaying/failing sources, hostile locators, ranking,
      and invalid limits.
- [x] Run source-quality ranking production-data proof over repeated live public
      GitHub/arXiv/Hacker News radar runs:
      `scripts/radar-source-quality-trends-proof` preserved
      `.arcwell-dev/proofs/radar-source-quality-trends-proof-20260624T090251Z-4856`,
      wrote two scored runs, 50 radar items/scores total, six source-quality
      windows, three trend rows, clean audits, and healthy cursors/source-health
      for the three public source families.
- [x] Add local deterministic semantic/topic dedupe after initial scoring, with
      `semantic_topic` dedupe groups, `duplicate_topic` score rows, preserved
      evidence, source-quality duplicate accounting, and audit drift coverage.
- [x] Run copied-home production-data semantic/topic dedupe review over real
      existing source cards:
      `.arcwell-dev/proofs/radar-semantic-production-review-20260624T090459Z`
      projected 16 Copilot source-card items, wrote 3 `semantic_topic` groups,
      marked 5 `duplicate_topic` scores, selected 8 items, produced 16
      source-quality windows, and passed `radar audit` with zero findings.
- [x] Add local/manual radar summary delivery attempts through authorized
      Telegram/email channel send paths, with CLI/MCP/slash surfaces, durable
      `radar_deliveries` rows linked to channel delivery attempts, idempotency
      replay, authorization/policy-denial blocking, provider-failure recording,
      ops snapshot/UI visibility, health warnings, HTML escaping, and
      token-redaction severe tests.
- [x] Reconcile radar delivery retry state through resident Telegram/email
      delivery retry paths: worker-driven successful retries now promote the
      original `radar_deliveries` row and radar run, scheduled email retries
      also promote the linked schedule tick, and exhausted local retry chains
      become `dead_lettered` without continued sends.
- [x] Add local scheduled Telegram/email delivery through the resident worker:
      scheduled profiles write durable ticks, enqueue
      `radar_scheduled_delivery`, run/summarize/audit, deliver through
      configured authorized Telegram or Cloudflare Email, record
      tick/run/summary/delivery lineage, suppress duplicate ticks inside the
      interval, reject raw secrets in profile policy, block unauthorized email
      recipients before provider send, and defer active quiet-hours windows
      without provider sends.
- [x] Add local cross-channel/scheduled retry with bounded attempts and
      dead-letter behavior: due failed email messages retry from local
      Cloudflare Email config without duplicate channel messages, successful
      scheduled email retries reconcile tick/delivery/run state, exhausted
      email retry chains dead-letter the channel message, radar delivery, and
      schedule tick, and severe tests prove token redaction.
- [x] Add optional radar model-score overlays: CLI/MCP/core write
      schema-validated `model_interestingness_v1` score rows and inspectable
      prompt/output artifacts, policy/cost block live OpenAI attempts before
      credentials or score rows, malformed provider output fails closed, model
      scores remain non-authorizing for summaries/delivery, and
      `scripts/radar-model-score-production-proof` proves a live OpenAI
      overlay on a fresh public RSS/GitHub/arXiv/Hacker News worker run without
      mutating heuristic selected rows or source-quality accounting.
- [x] Add repeatable production-data scheduled-delivery proof:
      `scripts/radar-scheduled-delivery-production-proof` creates a disposable
      scheduled profile over real public RSS/GitHub/arXiv/Hacker News sources,
      drains the resident worker, verifies real indexed/scored items and
      healthy cursor/source-health state, sends one audit-ok summary through a
      controlled Telegram endpoint, records tick/run/summary/delivery lineage,
      and proves duplicate suppression on a second worker pass.
- [ ] Add model-backed synthesis, live production delivery proof, live external
      scheduled delivery/service proof, production cross-channel delivery proof,
      production quiet-hours deferral, production-data semantic dedupe breadth
      across more profiles, production-data balance review, live model-scoring
      quality proof, seven-day source-quality trend/decay proof, broader ops
      controls, and status promotion only after real-data gates pass.
- [ ] Preserve tracked email defaults as `agent@example.com` and
      `user@example.com`; keep real local agent/author addresses only in ignored
      env or secret config.

## 6. Deep Research Quality And Host-Native Execution

- [ ] Add page expansion that actively gathers related docs/blogs/repos/social
      sources before writing a topic page.
- [x] Record fresh in-app Codex subagent and host-search proof for the current
      deep-research substrate.
- [x] Prove live OpenAI editorial invocation with cost records and fail-closed
      behavior on insufficient evidence.
- [x] Expose rich deep-research MCP schemas plus `research_capabilities`, and
      smoke the dev wrapper/cache for schema freshness before fresh-thread use.
- [x] Run disposable MCP reference-topic smokes for London AI, image
      compression, and safe cloud code execution with host-search proof,
      role artifacts, document anchors, editorial gates, audit, and reports.
- [x] Filter source/corpus bookkeeping claims out of executive judgment,
      analyst takeaways, and evidence narrative while keeping them inspectable
      in the report appendix.
- [x] Implement the local deterministic convergence substrate: durable
      iterations, statements, challenges, disproofs, revisions, fact-checks,
      snapshots, convergence report artifacts, and report judgments over
      persisted evidence.
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
- [x] Add `research_iterations` schema, indexes, Rust structs, row mappers,
      store methods, CLI read/list, MCP read/list, status embedding, redacted
      error fields, and migration ledger entries.
- [ ] Severe-test iterations for cross-run artifact rejection, duplicate
      iteration indexes, failed iteration preservation, long error redaction,
      parent lineage validation, database reopen, and 1000-iteration listing.
- [x] Add `research_statements` schema with statement type, scope, temporal
      scope, confidence, certainty, status, importance, evidence,
      counterevidence, assumptions, caveats, stable keys, and lineage.
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
- [x] Add initial `research_challenges` schema, deterministic challenge
      templates, required source-family output, and lifecycle states.
- [ ] Severe-test challenges for missing/cross-run statement ids, unknown
      challenge types, missing severity, empty search plans for high-severity
      challenges, prompt injection that tries to waive challenges, and duplicate
      ids.
- [ ] Add challenge-ranked disproof retrieval that records host-native search
      proof before reliance, falls back to configured provider search only
      through policy/cost gates, links search/source cards to challenges, and
      records blocked searches honestly.
- [x] Add initial challenge-linked host-search proof consumption: planned
      challenge queries can be answered only by matching
      `research_host_search_record` entries with selected linked research
      sources, and convergence disproof evidence records the host-search/result
      ids before treating the challenge as answered.
- [x] Add first-class convergence host-search task surface:
      `research_convergence_host_search_tasks` derives exact pending/recorded
      task rows from challenge search plans plus recorded host-search proof,
      appears in convergence status/CLI/MCP/capabilities, refreshes matching
      existing challenges when proof is recorded, and severe-tests wrong-query
      selected results so they cannot satisfy a planned challenge.
- [x] Add policy/cost-gated provider fallback for pending convergence
      host-search tasks: `research_convergence_provider_search` runs
      Brave/OpenAI/Perplexity provider search through existing policy and cost
      gates, records safe selected provider results as auditable search proof,
      can optionally enqueue bounded worker `ingest_url` jobs for selected safe
      results, stores blocked provider attempts as artifacts, and severe-tests
      cost cap, budget denial, unsafe URL filtering, cost-decision linkage,
      ingest enqueue caps, and task proof updates.
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
- [x] Add `research_disproofs` schema with verdict, strength, evidence,
      reasoning summary, confidence delta, and `requires_revision`.
- [ ] Severe-test disproof verdicts with direct contradictions, partial-scope
      mismatches, irrelevant evidence, missing evidence, stale vs official
      corrections, generated-synthesis misuse, low-quality source overreach,
      and numeric unit/date mismatches.
- [x] Add `research_revisions` schema with dropped/narrowed/downgraded/
      upgraded/split/merged/reframed/replaced/caveated lineage and links to
      trigger disproof ids.
- [ ] Severe-test revisions so refuted statements cannot remain final without
      caveat/replacement, rewording cannot hide a refuted stable key,
      confidence cannot increase after weakening without new evidence, and
      dropped statements remain visible in appendices.
- [x] Add `research_convergence_snapshots` with source novelty, claim novelty,
      statement change count, confidence deltas, open critical/high challenges,
      strong refutations, active fact-check summaries, evaluator scores,
      elapsed time, cost, and stop-rule JSON.
- [ ] Severe-test stop rules so critical unresolved challenges, strong
      refutations without revisions, wrong high-impact fact checks, no-progress
      loops, time caps, cost caps, provider-call caps, and user stop all block
      `settled` or stop incomplete as appropriate.
- [ ] Performance-test convergence metrics with 10,000 candidate sources,
      2,000 source cards, 50,000 claims, 5,000 statements, 20,000
      challenges/disproofs, 100 iterations, large report appendices, and
      bounded status reads.
- [x] Add active fact-checking for convergence runs that extracts report
      statements, verifies them against source cards plus fresh retrieval,
      labels right/wrong/unknown/not-checkable, and feeds wrong/unknown
      high-impact facts back into the next iteration. Current implementation:
      `research_active_fact_check` extracts factual report/generated-synthesis
      sentences, matches source-backed convergence statements as `right`,
      creates statement-backed `unknown` checks for unsupported high-impact
      sentences, marks status as resumable `continue`, and creates citation-gap
      host-search tasks for fresh retrieval.
- [x] Add first-class active fact-check closure orchestration:
      `research_convergence_close_loop` compiles/checks a convergence report,
      runs active fact-checking, optionally runs policy/cost-gated provider
      fallback for pending citation-gap searches, reruns convergence, compiles
      a final report judgment, and returns `closure_status` plus blockers.
      Severe fixtures prove unsupported report prose stays `needs_host_search`
      without retrieval proof, while provider-recorded proof plus rerun can
      settle and accept the final judgment.
- [ ] Severe-test active fact-checking with seeded false report sentences,
      uncited true sentences, vague opinions, unknown high-impact claims,
      verifier prompt injection, self-validating source text, cross-run source
      cards, and generated-summary evidence misuse. Current coverage includes
      supported/unsupported/vague sentence extraction plus close-loop blocked
      and provider-proof closure; prompt-injection, cross-run, and generated
      evidence misuse fixtures still need expansion.
- [x] Add convergence-aware report rendering: final position, what changed,
      survived/weakened/refuted statements, unresolved disproofs, source
      coverage, saturation, active fact-check summary, convergence stop reason,
      and statement/disproof/revision appendices. The current renderer now
      includes bottom-line readiness, iteration deltas, source/search
      saturation, host-search proof coverage, residual risks, blockers,
      revisions, evidence ledger, and method notes, with a severe regression
      asserting those analyst-grade sections remain present.
- [x] Add initial convergence report rendering with executive judgment, current
      position, pressure-test results, metrics, blockers, revisions, evidence
      ledger, and method notes.
- [ ] Add `research_report_judgment` artifacts written by Codex/main agent with
      scores for source coverage, primary-source depth, citation support,
      contradiction handling, uncertainty preservation, narrative clarity,
      decision usefulness, novelty/design quality, safety reasoning, and
      cost/time proportionality.
- [x] Add initial `research_report_judgment` ledger with accept/reject/
      incomplete decisions, blocking findings, non-blocking findings, evidence
      checked, remaining risks, and commands/artifacts reviewed.
- [x] Add opt-in model-backed convergence editorial/evaluator gate:
      `editorial_provider` plus `max_provider_calls>=2` on synchronous and
      queued convergence, fail-closed `no_write`/invalid-provider handling,
      citation-verifier plus adversarial-evaluator invocations, persisted
      nested judgment scores, terminal replay idempotency without duplicate
      provider calls, and severe direct/MCP round-trip tests. Regression
      coverage now proves incomplete terminal states such as `max_iterations`
      still invoke and persist the model-backed gate instead of silently
      skipping evaluation.
- [ ] Severe-test report rendering so refuted statements cannot appear as final
      conclusions, unresolved severe disproofs appear in executive caveats,
      metadata-only corpora do not fake judgment, appendices preserve
      traceability, and source Markdown/HTML is escaped.
- [ ] Add long-running convergence execution with resumable worker state,
      leases, heartbeats, idempotency keys, progress snapshots, cost
      reservations, stale-lease reclaim, dead-letter behavior, and user stop.
- [x] Add initial worker-resumable convergence queue execution via
      `research_convergence_enqueue`/`research_convergence_run` jobs with
      worker leases, retry/dead-letter behavior, stopped-run no-op handling,
      deterministic replay idempotency, terminal report/judgment compilation,
      CLI/MCP callability, and severe tests for resume, replay, malformed
      payloads, and user stop.
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
- [x] Update `$deep-research` skill with convergence loop, ledger inspection,
      settled/incomplete semantics, report judgment use, and stale-schema
      caveats.
- [x] Add CLI/MCP parity for convergence start/step/run/enqueue/status,
      iteration read/list, statements, challenges, disproofs, revisions,
      convergence report compile, and `research_capabilities` convergence
      fields, including host-search task, provider fallback, and model-backed
      editorial/evaluator convergence fields.
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
- [x] Run deterministic convergence fixture proof with at least 30 source
      cards, 80 claims, seeded contradictions, stale claims, malicious source
      text, unsupported report sentences, revisions, active fact-check, audit,
      and report judgment. `severe_research_convergence_saturated_fixture_preserves_bad_evidence_and_report_gate`
      seeds 30 linked source cards and 82 claims, preserves a structured
      contradiction, downgrades stale evidence, keeps prompt-injection source
      text out of the analyst narrative, and turns unsupported report prose
      into citation-gap host-search work.
- [x] Add the live saturated production-proof harness for the technical profile
      and make it fail closed when model-backed editorial/evaluator gates reject
      the output. `scripts/deep-research-production-proof --profile
      image-compression` preserves the disposable run home, command log, proof
      packet, report, host-search proof, source-card ledger, structured claims,
      convergence loop output, close-loop result, and OpenAI editorial/evaluator
      records. The harness now also promotes selected URLs into full-text
      source cards where bounded URL ingestion succeeds, executes exact
      convergence challenge-search tasks with `--`-safe query handling,
      dedupes normalized challenge tasks, supports worker-resumable convergence
      mode, and records challenge/full-source/worker settings in the proof
      packet.
- [ ] Run an accepted live host-search technical proof on image compression
      algorithms with papers, codec docs, benchmarks, dissenting analyses,
      numeric/table anchors or caveats, and model-backed report acceptance.
      Latest bounded orchestration proof home
      `.arcwell-dev/proofs/deep-research-production-proof-20260623T181935Z`
      ran 2 live Brave queries, linked 12 source cards, promoted 4 full-source
      cards, executed 20 exact challenge host-search tasks, ran 4 worker
      convergence jobs, and recorded live OpenAI model-backed editorial on a
      `max_iterations` incomplete terminal state. It correctly remains blocking
      with `closure_status: stopped_incomplete`, unaccepted model-backed
      judgment, one unknown high-impact fact check, and rejected final report
      judgment. Earlier saturated proof home
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
      scoring, and evaluator routing over synthesized score artifacts.
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
- [x] Run live provider-backed research/editorial synthesis and adversarial eval
      smokes over a saturated source-card corpus with cost records and artifacts
      for the technical/literature profile, proving provider invocation,
      structured score parsing, judgment persistence, fail-closed routing, and
      rejection behavior. Accepted analyst-grade quality remains open above;
      remaining live market and security profiles stay open below.
- [ ] Regenerate fresh hundred-source reports through the narrative-filtered
      compiler and evaluate them for analyst-grade judgment quality, not just
      structural completion.
- [ ] Expand difficult-document fixture coverage for PDFs, XLSX, precise table
      extraction, formula/cell handling, and publication-grade citation links.
      Current local coverage includes CSV/XLSX formulas, malformed inputs, PDF
      heuristic tables, and a severe wrapped-header/irregular-column/footnoted
      PDF table fixture that lowers confidence and preserves footnote refs; the
      broader external difficult-document matrix remains open.
- [x] Add publication-grade claim/report citation-quality checks that block
      completed status when evidence links are missing, stale, generated, or too
      weak, with severe convergence-report judgment tests for missing
      measurement anchors, stale current evidence, untrusted-only evidence, and
      a positive cell-anchored measurement path.
- [ ] Run fresh reference-topic deep-research live runs after host search,
      subagent orchestration, and provider-backed evals are proven.
- [x] Add host-supplied browser-rendered page snapshot ingestion through
      CLI/MCP with no daemon browser/network fetch, public-URL validation,
      rendered HTML/text size bounds, capture metadata, untrusted rendering,
      and severe invalid-input/prompt-injection tests.
- [ ] Add browser-rendered JavaScript readability extraction for pages that
      require rendering, including actual browser capture orchestration,
      screenshot/page-snapshot artifact storage, blocked-state reporting, and
      live proof against JS-heavy pages.

## 6A. Qualified Commerce Research

- [x] Capture the qualified commerce research design in
      `docs/qualified-commerce-research-design.md`, covering retail, rentals,
      travel, private context, exact-variant availability proof, browser
      verification, broad outputs, privacy boundaries, severe tests, and live
      proof gates.
- [x] Expand the qualified commerce design with anti-mirage claim ledger,
      false-done traps, proof levels, dependency order, report acceptance gate,
      proof packet template, and operational promotion requirements.
- [ ] Keep qualified commerce status at `Scaffold` until durable candidate and
      availability-proof storage, browser verification, context packet
      redaction, CLI/MCP read surfaces, severe tests, live proof packets, and
      docs/status agreement exist.
- [ ] Add browser-rendered commerce extraction as a deep-research fetch path:
      rendered DOM/page text, URL after redirects, timestamp, visible title,
      selected variant, availability signal, price, geography/shipping caveat,
      screenshot or page snapshot, blocked-state reporting, and source text as
      untrusted evidence.
- [ ] Add commerce candidate and availability-proof artifacts or tables with
      same-run validation, exact variant keys, checked timestamps, proof
      methods, confidence/caveats, CLI/MCP read surfaces, and report rendering.
- [ ] Add a bounded private context packet compiler for commerce runs using
      Arcwell memory/profile, Garderobe, and later approved browser history,
      screenshots, spreadsheets, and emails, with raw private data excluded from
      public wiki/source-card outputs by default.
- [ ] Implement the first `$qualified-commerce-research` skill profile for UK
      fashion retail: broad search, 20+ target qualified candidates when the
      market supports it, exact size availability proof, comfort/style/quality
      scoring, review evidence where available, and disqualified near-miss
      reporting.
- [ ] Severe-test commerce research with disabled/crossed-out sizes, wrong-size
      availability, variant-specific price changes, region/shipping caveats,
      JS-only pages, sold marketplace listings, size-system ambiguity, stale
      search results, retailer and wardrobe prompt injection, blocked pages,
      private-context leakage, and unverified candidates appearing in the main
      recommendation list.
- [ ] Add a preserved commerce proof packet script that exits non-zero when
      blockers remain and records feature status, user-visible claims, request,
      privacy/context sources, search providers, cost/policy decisions, raw and
      checked candidate counts, availability-proof methods, blocked/unknown/
      disqualified counts, artifacts, audit result, surfaces exercised, and
      promotion judgment.
- [ ] Run preserved live proofs for UK loafers in UK 8.5 and a denim shirt
      search, with browser-verified availability, context-derived preferences,
      review evidence where available, disqualified near misses, and final
      report audit before claiming the workflow works.
- [ ] Add ops/recovery requirements before any operational claim: worker leases
      or resumable state for long runs, retry/dead-letter behavior, source or
      provider health, cost caps, idempotent reruns, user-stop handling, and ops
      visibility for healthy, stale, blocked, failed, partial, retrying, and
      unknown states.

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
