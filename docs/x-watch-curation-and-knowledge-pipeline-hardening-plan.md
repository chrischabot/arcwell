# X Watch Curation And Knowledge Pipeline Hardening Plan

Date: 2026-06-29

## Current Audit Truth

This plan starts from the live Arcwell home at `/Users/chabotc/.arcwell/arcwell.sqlite3`.

The source-ingestion audit created these artifacts:

- `.arcwell-dev/audits/source-ingestion-20260629T101507Z-12236/artifacts/proof-packet.json`
- `.arcwell-dev/audits/source-ingestion-20260629T101507Z-12236/artifacts/source_inventory.csv`
- `docs/reports/2026-06-29-source-ingestion-inventory.csv`
- `docs/reports/2026-06-29-source-downstream-proof.csv`

The X curation preview created:

- `docs/reports/2026-06-29-x-watch-curation-preview.csv`
- `docs/reports/2026-06-29-x-watch-curation-preview-safe.csv`

The local X watch list currently contains the user's broad following import, not a curated AI/devtools/devrel watch list. That is too noisy and too expensive for the intended autonomous knowledge system.

Observed current state:

- X watch sources: 1,553 to 1,554 active `x_handle` rows depending on join shape/query normalization.
- Local profile coverage for watched X handles: 376 profiles, 213 with non-empty descriptions.
- X curation preview using only local evidence:
  - `keep`: 113 accounts
  - `review_keep_leaning`: 70 accounts
  - `review_drop_leaning`: 104 accounts
  - `drop_candidate`: 1,267 accounts
- Safer curation preview after separating evidence-starved rows:
  - `keep`: 113 accounts
  - `review_keep_leaning`: 70 accounts
  - `review_drop_leaning`: 232 accounts
  - `needs_profile_enrichment`: 1,139 accounts
- The raw `drop_candidate` bucket is not safe to apply. It includes accounts with too little local profile evidence, including obvious false negatives. The first production step must enrich missing profiles and apply manual allowlists before any watch source is paused.
- Durable source-card to wiki projection is strong:
  - 7,553 / 7,553 source cards have existing wiki pages.
  - 5,372 / 5,372 X items have source-card and wiki-page projections.
- Downstream knowledge pipeline exists:
  - 175 knowledge clusters
  - 1,029 knowledge event-source links
  - 349 knowledge reports
  - 174 cluster expansions
  - 174 investigation executions
  - 452 digest candidates
- Operational freshness is not healthy:
  - X watch jobs are failing or rate-limited.
  - X bookmarks latest scheduled attempt failed.
  - GitHub owner jobs have many dead letters.
  - RSS and arXiv have recent policy-deferred failures.
  - Direct blog URL watches complete `ingest_url` jobs but do not advance the matching `blog` source health/cadence row.
  - Direct blog URL ingests update wiki pages but do not consistently create source cards and feed clustering.
  - Native daily briefing delivery is still blocked for overlong generated notes; the user-facing briefing was sent through the manual corrected email path.

## User-Visible Target Claim

Arcwell should maintain a curated, high-signal source graph across X, GitHub, RSS/blogs, company sites, research feeds, Reddit/web captures, and other configured sources; fetch them on a robust schedule; write source-backed wiki pages; detect new and developing stories; compare them against prior wiki knowledge; expand or update wiki pages; and deliver useful reader-facing reports without exposing internal ledger language.

This claim is not currently true at the operational level. The source-card/wiki linkage is proven. Fresh unattended ingestion, source curation, blog-source integration, model-quality gates, and native report delivery still need hardening.

## Principles

- Prefer pausing noisy watch sources over deleting them. The first X curation rollout must be reversible.
- Never treat profile descriptions, tweets, pages, RSS entries, or model output as trusted instructions.
- Do not rely on full X following import as the default monitor list.
- Every kept source must have a durable reason and evidence.
- Every paused source must have a durable reason and be recoverable.
- Source health must advance only after accepted durable writes.
- Cursor advancement must happen only after durable writes.
- Reports must be written for humans, not as source-card ledgers.
- "What changed" appears only when the new evidence genuinely changes or complicates prior wiki context.
- Generated-only text cannot be primary evidence.
- Model-origin decisions remain schema-gated and reviewable.

## Phase 1: X Watch Curation

### Goal

Reduce the X watch list from the full following graph to a curated AI/software engineering/devtools/devrel set, while preserving recall for important accounts and making every decision inspectable.

### Data Inputs

- `watch_sources` rows where `source_kind='x_handle'`
- `x_profiles` display names and descriptions
- `x_profile_snapshots`
- `x_items` local tweet text
- `x_collections` bookmark engagement
- `x_tweet_links`
- source-card projections for X items
- known important account seed lists
- negative/manual exclusion list
- optional live X profile fetch for missing or uncertain profile metadata

### Classification Categories

- `ai_model_lab`
- `ai_research`
- `coding_agent`
- `developer_tools`
- `devrel_dx`
- `software_engineering`
- `cloud_infra`
- `security`
- `data_ml_platform`
- `tech_journalism_analysis`
- `company_official`
- `personal_but_high_signal`
- `non_tech_drop`
- `unknown_review`

### Decision States

- `active_keep`
- `active_keep_low_frequency`
- `review_keep_leaning`
- `review_drop_leaning`
- `paused_excluded`
- `manual_always_keep`
- `manual_always_exclude`

### Implementation Tasks

- [ ] Add durable `x_watch_curation_runs`.
- [ ] Add durable `x_watch_curation_decisions`.
- [ ] Store handle, previous watch status, proposed status, category, score, confidence, reason, evidence rows, and classifier version.
- [ ] Add manual seed allowlist for critical handles and orgs.
- [ ] Add manual blocklist for obvious non-tech accounts.
- [ ] Add deterministic classifier over handle, display name, profile description, local tweet text, local link domains, and bookmark engagement.
- [ ] Add model-assisted classifier only for `review_*` accounts, never for automatic destructive action.
- [ ] Add live profile enrichment for accounts missing local descriptions, with rate limits, cursor/source health, and auth refresh.
- [ ] Add `arcwell x curate-watch-sources --dry-run`.
- [ ] Add `arcwell x curate-watch-sources --apply --mode pause-only`.
- [ ] Add `arcwell x curate-watch-sources --restore-run <run_id>`.
- [ ] Add `arcwell x watch-curation-report`.
- [ ] Add ops UI table for curation results with filters by category, decision, confidence, last evidence, and proposed action.
- [ ] Add export CSV for review.
- [ ] Add import/apply from reviewed CSV.

### Safety Rules

- [ ] First production run is dry-run only.
- [ ] Second production run may only pause sources, never delete.
- [ ] Restore command must restore every source status from a curation run.
- [ ] Unknown accounts with recent bookmarks stay in review, not automatic drop.
- [ ] Accounts with high bookmark engagement stay keep/review even if profile text is sparse.
- [ ] Known critical org/person seed handles are always kept unless manually blocked.
- [ ] Model classifier cannot override manual allowlist/blocklist.
- [ ] Prompt-injection text in bios/tweets is quoted evidence only.

### Evaluation

- [ ] Build a gold set of at least 150 handles:
  - 75 must-keep AI/software/devrel accounts.
  - 50 obvious drop accounts.
  - 25 ambiguous accounts requiring review.
- [ ] Measure precision for `active_keep`.
- [ ] Measure recall against must-keep seeds.
- [ ] Measure manual-review burden.
- [ ] Sample 50 automatic drop candidates and confirm fewer than 5% are true important misses before applying.
- [ ] Sample 50 keep candidates and confirm fewer than 10% are irrelevant.
- [ ] Compare result count against target range: likely 250 to 500 accounts after review.
- [ ] Run live watch-source poll after curation and prove fewer jobs, fewer policy failures, and no missing seed accounts.

### Proof Gate

- [ ] Dry-run proof packet records input count, output count, category counts, reasons, and sample evidence.
- [ ] Pause-only proof packet records reversible DB changes and restore command.
- [ ] Restore drill proves the old watch list can be restored.
- [ ] Worker proof shows curated watch list reduces failed X monitor jobs.
- [ ] Ops UI proof shows curation status and source-health impact.

## Phase 2: Provider Policy, Auth, And Source Health Repair

### Goal

Make due source polling reliable enough that source freshness can be trusted.

### Implementation Tasks

- [ ] Audit policy decisions for failing `provider.network` actions.
- [ ] Add narrowly scoped policies for:
  - X recent search
  - X bookmark import
  - X watch monitor
  - GitHub owner/repo fetch
  - RSS fetch
  - arXiv search
  - URL/blog ingest
- [ ] Classify source failures as:
  - `policy_blocked`
  - `auth_expired`
  - `rate_limited`
  - `provider_4xx`
  - `provider_5xx`
  - `parse_error`
  - `content_safety_rejected`
  - `network_timeout`
  - `partial_write`
- [ ] Do not collapse policy failures into generic failed source health.
- [ ] Add dead-letter recovery command with dry-run and scoped requeue.
- [ ] Add source-health cleanup command for superseded stale rows.
- [ ] Fix X OAuth refresh path so expired X tokens are refreshed by the system, not by the user.
- [ ] Add credential probes for providers that support direct probes.
- [ ] Keep X-specific probes separate from generic provider probes.

### Evaluation

- [ ] Unit tests for failure classification.
- [ ] Tests for source-health next-run/backoff per failure type.
- [ ] Tests that policy-denied jobs do not advance cursor.
- [ ] Tests that auth-expired X jobs attempt refresh and redact secrets.
- [ ] Tests that 429 defers without retry storms.
- [ ] Tests that 5xx retries with capped backoff.
- [ ] Tests that partial writes are rolled back or recorded as partial without cursor advancement.

### Proof Gate

- [ ] Run bounded live worker pass after policy repair.
- [ ] Prove no unexpected dead-letter growth.
- [ ] Prove latest source-health status is not dominated by policy-deferred failures.
- [ ] Prove cursors changed only for sources with accepted durable writes.
- [ ] Prove ops UI distinguishes healthy, stale, failed, rate-limited, auth-expired, and policy-blocked.

## Phase 3: Source Adapter Contract Completion

### Goal

All source families should write comparable durable source-card evidence, adapter-run rows, source health, cursors, and downstream backlog triggers.

### Required Contract

Each adapter run must record:

- provider
- source kind
- locator
- status
- error kind
- cursor before
- cursor after
- raw count
- accepted count
- duplicate count
- rejected count
- source-card ids
- source-health update
- next run
- whether backlog clustering was enqueued, skipped, or blocked

### Blog URL Repair

Current problem: `blog` watch sources enqueue `ingest_url`; the job updates wiki pages, but does not advance the `blog` source-health/cadence row and does not consistently create source cards.

Implementation:

- [ ] Replace `blog -> ingest_url` scheduling with a real `blog_fetch` adapter, or pass watch-source lineage into `ingest_url` and promote it into the adapter contract.
- [ ] Create source cards for blog/company page captures.
- [ ] Prefer feed discovery when a blog homepage exposes RSS/Atom links.
- [ ] For company/news pages without RSS, extract recent article links and create source cards per article.
- [ ] Track content hash, ETag, Last-Modified, canonical URL, and final URL.
- [ ] Advance `source_health` after source cards/wiki pages are durable.
- [ ] Chain accepted source cards into knowledge backlog.
- [ ] Avoid re-ingesting unchanged pages.

### GitHub Repair

- [ ] Normalize owner/repo jobs into adapter-run contract.
- [ ] Ensure owner scans mark duplicates and stale repos distinctly.
- [ ] Add optional specific repo watches for high-value repos.
- [ ] Add release/commit/source-card projections with topic metadata.
- [ ] Add per-owner policy failure visibility.

### RSS/arXiv Repair

- [ ] Normalize rejected/duplicate counts.
- [ ] Add parser error samples with redaction.
- [ ] Add per-feed last accepted item.
- [ ] Add stale feed detection.
- [ ] Add feed replacement/supersede flow.

### X Repair

- [ ] Run curated watch list only.
- [ ] Fetch bookmarks and likes where supported.
- [ ] Classify X provider failures separately: auth, quota, GraphQL/browser failure, API rate limit, policy denial.
- [ ] Keep browser/GraphQL fallback explicit and audited.
- [ ] Keep full bookmark import completeness report.
- [ ] Do not advance X cursors until source cards and projections are durable.

### Evaluation

- [ ] Adapter contract tests for every source family.
- [ ] Replay tests for duplicate items.
- [ ] Cursor tests for partial writes.
- [ ] Malformed RSS/arXiv/GitHub/X payload tests.
- [ ] Prompt-injection-as-source-text tests.
- [ ] URL safety tests: SSRF, redirects, content-type, size, private IPs.
- [ ] Restart tests during adapter write.

## Phase 4: Unified Knowledge Pipeline

### Goal

Fresh source cards from every family should become knowledge events, entities, relations, clusters, editorial decisions, reports, wiki pages, and digest candidates where appropriate.

### Implementation Tasks

- [ ] Ensure source-card output from every adapter chains to backlog clustering.
- [ ] Decide whether to populate `x_knowledge_clusters` or formally deprecate it in favor of `knowledge_clusters`.
- [ ] Add source-family diversity to cluster scoring.
- [ ] Add duplicate-group persistence across X/GitHub/RSS/web sources.
- [ ] Add stale-score computation from source health and evidence dates.
- [ ] Add momentum based on fresh source count and source diversity.
- [ ] Add novelty based on prior wiki coverage.
- [ ] Add entity relations for:
  - company launched repo
  - company announced model
  - person reacted to announcement
  - repo implements topic
  - blog/article comments on event
  - benchmark evaluates model/product
- [ ] Add "story development" tracking by linking new clusters to existing story pages.
- [ ] Add contradiction/tension detector:
  - claim conflicts with previous wiki belief
  - claim broadens scope
  - claim narrows availability
  - claim contradicts benchmark/reception
  - claim changes competitive positioning

### Evaluation

- [ ] Golden story set:
  - OpenAI GPT-5.6 Sol/Terra/Luna
  - Vercel agent SDK/Eve-style launch
  - Simon Willison benchmark/research post
  - NVIDIA open model release
  - Karpathy/Claude workflow story
  - Cloudflare agent infrastructure release
- [ ] For each story, verify:
  - sources from at least two families when available
  - entity extraction
  - source-card citation
  - cluster creation or existing cluster update
  - correct story page creation/update
  - no duplicate wiki page
  - "what changed" paragraph appears only when meaningful
  - report links are human-facing URLs, not source ids

## Phase 5: Editorial Decision Loop And Wiki Writing

### Goal

Every important cluster gets the right action: create wiki page, update existing wiki page, create digest candidate, investigate more, monitor only, or ignore.

### Decision Types

- `create_new_wiki_page`
- `update_existing_wiki_page`
- `create_digest_candidate`
- `investigate_more`
- `monitor_only`
- `ignore`
- `block_for_review`

### Writer Requirements

Every generated page/report must:

- tell the story in human language
- explain what happened
- explain why it matters
- explain what is known
- explain what is uncertain
- include reception/context when available
- link source-card IDs internally
- show human-facing source links externally
- connect to prior wiki context only when it adds insight
- avoid source-card/cluster/candidate/internal-ledger terms in reader-facing output
- avoid "What This Changes" unless something actually changed
- avoid "Recommended follow-up" as a user task; follow-up becomes an internal job

### Quality Gates

- [ ] Reject empty generated pages.
- [ ] Reject link dumps.
- [ ] Reject reports with source-card IDs in reader-facing copy.
- [ ] Reject unsupported claims.
- [ ] Reject claims based only on model prose.
- [ ] Reject duplicate wiki page creation when an existing story page should be updated.
- [ ] Reject prompt-injection leakage.
- [ ] Reject stale evidence presented as current.
- [ ] Reject "what changed" sections with no real change.
- [ ] Reject internal phrases:
  - source card
  - cluster
  - local corpus
  - digest candidate
  - review score
  - candidate id
  - metadata
  - filed evidence

### Evaluation

- [ ] Add editorial golden-output evals.
- [ ] Add human-readable report rubric:
  - clear headline
  - context
  - why it matters
  - reception
  - competitive position
  - uncertainty
  - useful links
  - no ledger language
- [ ] Add adversarial report-review job that scores prose before delivery.
- [ ] Add regression corpus from the user's rejected briefing examples.

## Phase 6: Delivery And Recurrence

### Goal

Arcwell should deliver immediate alerts for big stories and a daily 7am briefing, using the native scheduled path rather than manual corrected sends.

### Implementation Tasks

- [ ] Fix native daily briefing body length and quality.
- [ ] Convert Markdown to HTML for email automatically.
- [ ] Convert channel-specific formatting for Telegram.
- [ ] Enforce quiet hours and urgent override policy.
- [ ] Add idempotency keys for story alerts and daily briefings.
- [ ] Add dedupe windows by story, topic, source cluster, and recipient.
- [ ] Add retry/dead-letter behavior with ops visibility.
- [ ] Add morning report inclusion of already-alerted big stories.
- [ ] Add story-development sections in daily reports.
- [ ] Add "no major new story" wording that still reads editorial, not conversational filler.

### Proof Gate

- [ ] Live external email send with native path.
- [ ] Live external recurrence over wall-clock time.
- [ ] Sleep/shutdown/restart catch-up proof.
- [ ] Duplicate-send suppression proof.
- [ ] Quiet-hours deferral proof.
- [ ] Urgent-story immediate-send proof.
- [ ] Next-morning inclusion proof.

## Phase 7: Ops UI And Operator Controls

### Goal

The ops dashboard should make it obvious whether the knowledge system is current, stale, blocked, or producing low-quality output.

### Required Views

- source inventory
- source curation
- source health
- stale sources
- provider policy failures
- auth/credential health
- dead-lettered jobs
- adapter runs
- source-card counts
- wiki projection coverage
- clusters by status
- story pages updated today
- reports generated today
- editorial decisions
- digest candidates
- deliveries
- last successful external alert
- last successful daily briefing
- curation restore controls

### Required Actions

- dry-run curation
- pause/apply curation
- restore curation
- requeue scoped dead letters
- mark source superseded
- run one source now
- run source family now
- run backlog clustering
- run editorial decision
- run report quality audit
- send test digest

All actions need CSRF/auth checks, policy checks, idempotency keys, and audit rows.

## Phase 8: Severe Test Matrix

### X Curation Tests

- [ ] malicious profile description prompt injection
- [ ] false-positive high keyword spam account
- [ ] false-negative important account with sparse bio
- [ ] account rename
- [ ] duplicate handles with case differences
- [ ] missing profile rows
- [ ] huge profile description
- [ ] Unicode/RTL/control characters
- [ ] restore after pause-only apply
- [ ] model classifier returns invalid schema
- [ ] model classifier tries to issue instructions

### Adapter Tests

- [ ] policy denial
- [ ] 401/403/429/5xx
- [ ] token refresh success/failure
- [ ] partial write before cursor
- [ ] duplicate item replay
- [ ] malformed RSS
- [ ] huge RSS item
- [ ] arXiv parse drift
- [ ] GitHub API schema drift
- [ ] X API quota and browser fallback failure
- [ ] URL SSRF/private IP
- [ ] redirect to unsafe URL
- [ ] content-type mismatch
- [ ] content too large

### Knowledge Pipeline Tests

- [ ] source cards without wiki projection cannot proceed
- [ ] generated-only evidence cannot create cluster
- [ ] duplicate source card cannot be reused across incompatible model clusters
- [ ] source family diversity affects confidence
- [ ] stale evidence lowers score
- [ ] story update attaches to existing page
- [ ] duplicate-page creation is rejected
- [ ] contradiction/tension is detected and explained
- [ ] unsupported claims are rejected

### Editorial Tests

- [ ] no ledger language in reader copy
- [ ] no source ids in reader copy
- [ ] no empty report
- [ ] no link dump
- [ ] no meaningless "What This Changes"
- [ ] no user-assigned "Recommended follow-up"
- [ ] HTML email renders headings and links
- [ ] Telegram formatting uses its allowed markdown format
- [ ] report body fits delivery limits

### Recurrence Tests

- [ ] worker catches up after sleep/shutdown
- [ ] repeated worker loop does not duplicate jobs
- [ ] quiet-hours deferral
- [ ] urgent override
- [ ] delivery retry
- [ ] dead-letter visibility
- [ ] no model-score-only sends
- [ ] approved story appears in next daily report after immediate alert

## Phase 9: Production Proof Packet

Before claiming operational readiness, produce a proof packet with:

- curated X source count before/after
- curation precision/recall sample
- source health before/after
- dead letters before/after
- source-card writes by family
- wiki-page writes and updates
- cluster count and examples
- entity/relation count and examples
- story-development examples
- editorial decision examples
- report quality audit results
- delivery attempts
- external email proof
- recurrence proof over wall-clock time
- sleep/restart catch-up proof
- ops UI screenshots
- severe test command output
- remaining risks

## Completion Definition

Do not call this complete until all of these are true:

- [ ] X watch list is curated, reversible, and materially smaller.
- [ ] All active source families have healthy or intentionally deferred source-health rows.
- [ ] Dead-letter queues are either empty or have explicit accepted reasons.
- [ ] Blog/company pages create source cards and feed the knowledge pipeline.
- [ ] Fresh source cards from each family can trigger backlog clustering.
- [ ] Clusters create or update wiki pages with human-readable source-backed prose.
- [ ] Story updates attach to existing wiki pages when appropriate.
- [ ] Reports are editorial and pass no-ledger-language gates.
- [ ] Immediate alerts and daily briefings are sent through native scheduled paths.
- [ ] External recurrence is proven over time.
- [ ] Ops UI shows the whole system's health and lets us repair it safely.
- [ ] Severe tests pass for malicious, invalid, duplicate, stale, partial-write, provider-failure, and delivery-failure scenarios.
