# Arcwell Radar And Digest Implementation Plan

Date: 2026-06-23

Reference study: temporary clone of `Thysrael/Horizon` at
`/tmp/horizon-arcwell.21dBh1/Horizon`.

Status: design and execution plan only. No implementation is claimed by this
document.

## Objective

Build the full value of Horizon inside Arcwell as a native, inspectable,
source-backed, policy-aware radar and digest system.

The product claim is not "Arcwell can summarize some links." The claim is:

> Arcwell can continuously ingest real production signals from configured
> source families, normalize and index them durably, rank what matters,
> explain why it matters with source-grounded evidence, generate useful reports,
> and send authorized updates through existing Arcwell delivery surfaces without
> hiding uncertainty, skipping provenance, leaking secrets, or silently breaking
> under real data volume.

This plan is intentionally anti-mirage. A feature is not done because a table,
prompt, README, MCP schema, or happy-path smoke exists. It is done only when
the production-data proof gates in this document pass and the status surfaces
agree.

## Horizon Lessons To Absorb

Horizon is not a broad app shell. It is a focused daily information pipeline.
The reusable lessons are structural:

1. Normalize everything into one item shape before ranking.
   Horizon uses `ContentItem` with source type, title, URL, content, author,
   timestamps, metadata, score, reason, summary, and tags.

2. Keep pipeline stages explicit.
   Horizon stages fetch, URL dedupe, score, threshold filter, semantic dedupe,
   category balance, enrich, summarize, and deliver. Its MCP layer exposes
   those as re-enterable stages with run artifacts.

3. Score before spending enrichment effort.
   Horizon enriches only selected high-value items. Arcwell should do the same
   to control cost, latency, and noise.

4. Dedupe twice.
   Horizon first dedupes exact/canonical URLs, then topic-dedupes scored items
   so repeated releases/incidents do not dominate a digest.

5. Balance the final digest.
   Horizon's category quotas prevent one loud source family from drowning out
   everything else.

6. Treat source discovery as part of the product.
   Horizon's presets, wizard, AI recommendation path, and HorizonHub design
   show that good source management is not setup fluff. Source quality,
   signal-to-noise, output volume, and decay detection become product data.

7. Delivery is a pipeline stage, not an afterthought.
   Email, webhooks, MCP summaries, and local artifacts all receive the same
   summary output. Arcwell should route through email, Telegram/channels,
   source cards, wiki pages, digest candidates, and ops records.

## Arcwell Native Shape

Do not vendor Horizon as a Python sidecar. That would bypass Arcwell's existing
strengths:

- local SQLite and Markdown source of truth
- source cards with trust/provenance metadata
- wiki pages and FTS indexing
- watch sources, cursors, source health, and worker jobs
- X canonical rows and source-card projections
- policy, cost, secret redaction, and ops visibility
- Codex plugin slash commands and MCP parity
- email/Telegram/channel delivery boundaries

Implement this as a new native package:

```text
packages/arcwell-radar/
crates/arcwell-core/src/lib.rs      # storage, stages, scoring, reports
crates/arcwell-cli/src/main.rs      # CLI and MCP tools
plugins/arcwell-codex/commands/    # slash prompts
plugins/arcwell-codex/skills/      # operator skill if needed
docs/horizon-radar-digest-implementation-plan.md
```

The package can initially live inside `arcwell-core` like the current wiki,
librarian, X, and research surfaces. Split into a crate only if the code size
or dependency boundary justifies it.

## Status Ladder

Every radar capability must carry one of these labels:

- `Missing`: no code.
- `Scaffold`: schemas, prompts, or docs exist but behavior is not proven.
- `Local Fixture Proof`: deterministic fixtures pass. This is not enough for
  production data claims.
- `Production Data Proof`: real public or authorized private source data has
  passed the relevant gate in a disposable or controlled Arcwell home.
- `Operational`: production-data proof plus scheduled/resumable operation,
  source-health monitoring, delivery records, recovery drills, and docs.
- `Done`: only for stable, maintained, regression-covered operational features.

No item may jump from `Scaffold` or `Local Fixture Proof` to `Done`.

## Proof Log

### 2026-06-23 Source-Card Radar Slice

Status: `Production Data Proof` for copied-home `source_card_query`
projection, existing source-family projection, exact canonical-URL dedupe, and
foreground live RSS/GitHub/arXiv/Hacker News adapter execution; `Partial` for
the overall Horizon-inspired system.

Evidence:

- `cargo test --all --all-features` passed.
- `scripts/verify-codex-plugin-docs` passed with 128 commands, 15 skills, 215
  MCP tools, and 158 docs/prompts checked.
- `scripts/arcwell-dev sync` rebuilt and synced the dev plugin.
- Copied-home proof at `/tmp/arcwell-radar-proof-20260623T184338Z` used a
  SQLite backup of the real local Arcwell home. `arcwell radar run
  horizon-real-source-cards` with `source_card_query=agent` produced 368
  normalized rows, 368 FTS rows, 368 heuristic scores, and 25 selected items.
  `arcwell radar audit` returned `ok=true` with no findings.
- Broader copied-home proof at
  `/tmp/arcwell-radar-family-proof-20260623T184644Z` used a SQLite backup of
  the real local Arcwell home with `rss=*`, `github=*`, `arxiv=*`, and
  `x_handle=sawyerhood`, plus an intentionally unsupported
  `hackernews=frontpage` selector. It produced 1,374 normalized rows, 1,374
  FTS rows, 1,374 heuristic scores, and 50 selected items. `arcwell radar audit`
  returned `ok=true` with only the expected medium unsupported-HN finding.
- Rebuilt copied-home proof at `/tmp/arcwell-radar-dedupe-proof-20260623T`
  used a copy of the real local Arcwell home containing 2,500 source cards.
  `ARCWELL_HOME=/tmp/arcwell-radar-dedupe-proof-20260623T/home target/debug/arcwell radar audit e0a9de84-2c4c-401d-acb9-6da826076a5c`
  returned `ok=true`, 2,500 items, 2,500 FTS rows, 2,500 scored rows, 26
  dedupe groups, and no findings. Direct SQLite status checks showed 50
  `selected`, 26 `duplicate_url`, and 2,424 `over_profile_limit` score rows;
  `radar stage` preserved all 2,500 item rows and listed the 26
  `canonical_url` dedupe groups.
- The same copied-home run wrote deterministic Markdown summary
  `radar-summary-95b92c60cf9bd0c214a2d3864edbf7b4` with `arcwell radar
  summarize`. It covered 50 selected items and 50 source cards, carried forward
  the 26 dedupe groups and score-status counts in metadata, stored
  `not_delivery=true`, advanced `summary_count=1`, kept `delivery_count=0`, and
  `arcwell radar summary` read back the same artifact.
- Disposable-home live adapter proof at
  `/tmp/arcwell-radar-live-proof-20260623T191510Z` used real public
  production sources with `arcwell radar run horizon-live-proof --fetch-live`.
  The profile selected `https://hnrss.org/newest?points=100`, GitHub owner
  `Thysrael`, and arXiv query `cat:cs.AI`. The run completed 3 adapter jobs
  (`rss_fetch`, `github_owner`, `arxiv_search`), wrote 30 source cards, advanced
  3 cursors, recorded 3 healthy source-health rows, projected 30 radar items,
  wrote 30 FTS rows, wrote 30 score rows, selected 20 items, and `radar audit`
  returned `ok=true`. `radar summarize` wrote
  `radar-summary-e655496f9cde91bc80c054f5915e6fad` over 20 selected items /
  20 source cards with `not_delivery=true`, and `radar summary` read back the
  same artifact.
- Disposable Hacker News proof at `/tmp/arcwell-radar-hn-proof-20260623T192659Z`
  used real current HN top stories through a `hackernews=frontpage` selector.
  The foreground `hackernews_fetch` job completed, wrote 5 source cards with
  bounded top-level comment evidence, advanced cursor `hackernews:topstories`,
  recorded healthy source-health, wrote 5 radar items/FTS rows/scores, selected
  5 items, `radar audit` returned `ok=true`, and
  `radar-summary-7276ea4ad8fd0d8d0ae243f65a0a4eb5` read back with
  `not_delivery=true`.
- Reddit local-proof slice added JSON listing fetch with bounded top-comment
  capture, RSS-first unauthenticated fallback that explicitly does not claim
  comment capture, source-card persistence, cursor/source-health safety,
  watch-source enqueue support, and severe policy/fallback/source-card tests.
- Queued `radar_run` local-proof slice added `arcwell radar enqueue`,
  `radar_enqueue`, and worker execution for radar profiles. Severe tests prove
  `worker run-once` can complete a queued source-card-backed radar run into
  durable `radar_runs`, `radar_items`, FTS rows, scores, and audit-clean stage
  state; a queued live run with RSS provider policy denial records a completed
  worker job whose radar result is `blocked`, with failed adapter job summary,
  non-healthy source-health, no cursor advance, and failed radar audit; invalid
  profile/window inputs are rejected before inert jobs are inserted.
- Repeatable worker-drained production-data proof script
  `scripts/radar-worker-production-proof` preserved proof packet
  `.arcwell-dev/proofs/radar-worker-production-proof-20260624T102233Z-28818/artifacts/proof-packet.json`.
  It queued one `radar_run` and completed it through `worker run-once` against
  real public RSS, GitHub owner, arXiv, and Hacker News selectors. The worker
  completed one radar job plus four adapter jobs, wrote 45 source cards, 45
  wiki pages, 45 radar items, 45 FTS rows, 45 score rows, four healthy
  source-health rows, and four cursors; selected 30 items; returned
  `radar audit ok=true`; and wrote
  `radar-summary-78b9272df8503999963e027006b971a8` with
  `not_delivery=true`. The same proof also verifies run metadata and ops expose
  `score_distribution` for `heuristic_v1` rows with `score_count=45`,
  `selected_count=30`, finite `average`, `p50`, and `p90`.
- Reddit disposable live proof is not yet production-data proof. Preserved
  attempts under `/tmp/arcwell-radar-reddit-proof-20260624T075239Z` and
  `/tmp/arcwell-radar-reddit-debug-20260624T075408Z` show the Arcwell binary
  receives Reddit HTTP 403 for anonymous RSS/JSON access, even though `curl`
  intermittently read RSS with the same declared User-Agent. Keep Reddit below
  `Production Data Proof` until OAuth or another sanctioned access path is
  implemented and passes the source-card/cursor/source-health/radar gates.
- Local balance slice added explicit `metadata.balance.max_per_source` and
  `metadata.balance.category_quotas` handling during deterministic scoring.
  Severe tests prove malformed caps fail closed, one source cannot dominate when
  capped, one category cannot exceed its quota, quota-rejected rows remain in
  `radar_scores` as `source_quota` / `category_quota` with reasons/tags, and
  rejected items retain source-card/wiki provenance.
- Production-data source-balance proof now exists through
  `scripts/radar-balance-production-proof`, preserved at
  `.arcwell-dev/proofs/radar-balance-production-proof-20260624T102938Z-92096/artifacts/proof-packet.json`.
  A worker-drained live public RSS/GitHub/arXiv/Hacker News run wrote 52
  normalized/indexed/scored radar items, selected 4 items with at most one per
  source, kept 36 `source_quota` rows inspectable with source-card/wiki
  provenance, passed `radar audit`, recorded healthy source-health/cursors, and
  wrote a non-delivery summary.
- Production-data source-family category-balance proof now exists through
  `scripts/radar-category-balance-production-proof`, preserved at
  `.arcwell-dev/proofs/radar-category-balance-production-proof-20260624T104401Z-30612/artifacts/proof-packet.json`.
  Two worker-drained live public RSS/GitHub/arXiv/Hacker News profiles wrote
  116 normalized/indexed/scored radar items, selected 12 items while respecting
  per-family quotas, kept 76 `category_quota` rows inspectable with
  source-card/wiki provenance across all four configured source-family
  categories, exposed matching run/ops score-distribution counts, advanced all
  four cursors with healthy source-health, passed both audits, and wrote two
  non-delivery summaries. This proves source-family category quotas, not
  arbitrary future topic taxonomies.
- Production-data deterministic semantic/topic dedupe breadth proof now exists
  through `scripts/radar-semantic-dedupe-production-proof`, preserved at
  `.arcwell-dev/proofs/radar-semantic-dedupe-production-proof-20260624T105627Z-44256/artifacts/proof-packet.json`.
  A sanitized copied home from the real 2,500-card source corpus ran `copilot`,
  `codex`, `agent`, and `github` source-card profiles, wrote 1,482
  normalized/indexed/scored radar items, kept 304 `semantic_topic` groups and
  621 `duplicate_topic` rows inspectable with source-card/wiki provenance,
  exposed matching run/ops score-distribution counts, passed four audits, and
  wrote four non-delivery summaries. This proves deterministic local semantic
  dedupe breadth over real source cards, not model semantic dedupe or live
  adapter semantic breadth.

Still not proven by this slice:

- Radar-owned live X/Reddit/public Telegram/OSS/OpenBB fetch.
- Scheduled recurring radar service execution, retry/recovery, and ops UI
  controls.
- Full recursive HN/Reddit community-thread capture.
- Live-adapter semantic/topic dedupe breadth, non-source-family taxonomy
  category-balance review, source-quality decay.
- Model-backed interestingness, enrichment/synthesis, and delivery attempts.
- Full production multi-source proof including authenticated/private sources.

## Product Surfaces

### CLI

Target commands:

```sh
arcwell radar profile create <name> --from-template ai-infra
arcwell radar profile list
arcwell radar profile read <profile-id>
arcwell radar profile update <profile-id> ...
arcwell radar source recommend "agent infrastructure"
arcwell radar source import-presets <file-or-url>
arcwell radar run <profile-id> --window-hours 24
arcwell radar enqueue <profile-id> --window-hours 24
arcwell radar fetch <run-id>
arcwell radar score <run-id>
arcwell radar filter <run-id>
arcwell radar enrich <run-id>
arcwell radar summarize <run-id> --language en
arcwell radar deliver <run-id> --channel telegram --recipient <authorized-subject>
arcwell radar runs
arcwell radar stage <run-id> <raw|normalized|indexed|scored|filtered|enriched>
arcwell radar summary <run-id> --language en
arcwell radar audit <run-id>
arcwell radar source-quality <run-id>
arcwell radar repair <run-id>
```

### MCP

Target tools:

```text
radar_profile_create
radar_profile_list
radar_profile_read
radar_source_recommend
radar_run_create
radar_enqueue
radar_fetch
radar_score
radar_filter
radar_enrich
radar_summarize
radar_deliver
radar_runs
radar_stage_read
radar_summary_read
radar_audit_run
radar_source_quality
```

Target resources:

```text
arcwell://radar
arcwell://radar-runs
arcwell://radar-profiles
arcwell://radar-deliveries
```

### Slash Commands

Target commands:

```text
/radar-run
/radar-enqueue
/radar-runs
/radar-stage
/radar-summary
/radar-deliver
/radar-source-quality
/radar-recommend-sources
```

Every slash command needs a CLI or MCP alias and must be covered by
`scripts/verify-codex-plugin-docs`.

## Data Model

Use additive migrations. Do not mutate existing source-card, watch-source,
source-health, X, or digest-candidate semantics unless explicitly required.

### `radar_profiles`

Purpose: named configuration for a recurring digest/radar view.

Fields:

- `id TEXT PRIMARY KEY`
- `name TEXT NOT NULL UNIQUE`
- `description TEXT NOT NULL DEFAULT ''`
- `status TEXT NOT NULL`
- `window_hours INTEGER NOT NULL`
- `min_score REAL NOT NULL`
- `max_items INTEGER`
- `languages_json TEXT NOT NULL`
- `category_groups_json TEXT NOT NULL`
- `source_selectors_json TEXT NOT NULL`
- `delivery_policy_json TEXT NOT NULL`
- `model_policy_json TEXT NOT NULL`
- `metadata_json TEXT NOT NULL DEFAULT '{}'`
- `created_at TEXT NOT NULL`
- `updated_at TEXT NOT NULL`

Invariants:

- `window_hours > 0`.
- `min_score` is between `0` and `10`.
- `languages_json` is non-empty.
- `source_selectors_json` references source kinds Arcwell understands or marks
  unsupported selectors as disabled with an explicit reason.
- Delivery targets are references to authorized channels/recipients, not raw
  secrets or unverified addresses.

### `radar_runs`

Purpose: one execution ledger from fetch through delivery.

Fields:

- `id TEXT PRIMARY KEY`
- `profile_id TEXT NOT NULL`
- `status TEXT NOT NULL`
- `window_start TEXT NOT NULL`
- `window_end TEXT NOT NULL`
- `stage TEXT NOT NULL`
- `source_selection_json TEXT NOT NULL`
- `raw_count INTEGER NOT NULL DEFAULT 0`
- `normalized_count INTEGER NOT NULL DEFAULT 0`
- `indexed_count INTEGER NOT NULL DEFAULT 0`
- `scored_count INTEGER NOT NULL DEFAULT 0`
- `filtered_count INTEGER NOT NULL DEFAULT 0`
- `enriched_count INTEGER NOT NULL DEFAULT 0`
- `summary_count INTEGER NOT NULL DEFAULT 0`
- `delivery_count INTEGER NOT NULL DEFAULT 0`
- `error TEXT`
- `metadata_json TEXT NOT NULL DEFAULT '{}'`
- `started_at TEXT NOT NULL`
- `finished_at TEXT`
- `updated_at TEXT NOT NULL`

Statuses:

- `created`
- `fetching`
- `fetched`
- `indexing`
- `indexed`
- `scoring`
- `scored`
- `filtering`
- `filtered`
- `enriching`
- `enriched`
- `summarizing`
- `summarized`
- `delivering`
- `completed`
- `failed`
- `blocked`
- `stopped`

Invariants:

- Terminal states preserve all successful prior stage artifacts.
- Failure never deletes prior stages.
- Rerun with the same `run_id` is idempotent for completed stages unless
  explicitly forced.
- A later stage cannot run if required earlier artifacts are missing.

### `radar_items`

Purpose: normalized item rows for all source families.

Fields:

- `id TEXT PRIMARY KEY`
- `run_id TEXT NOT NULL`
- `stable_key TEXT NOT NULL`
- `source_kind TEXT NOT NULL`
- `provider TEXT NOT NULL`
- `source_locator TEXT NOT NULL`
- `native_id TEXT`
- `canonical_url TEXT`
- `title TEXT NOT NULL`
- `author TEXT`
- `published_at TEXT`
- `fetched_at TEXT NOT NULL`
- `content_text TEXT NOT NULL DEFAULT ''`
- `content_sha256 TEXT NOT NULL`
- `metadata_json TEXT NOT NULL DEFAULT '{}'`
- `source_card_id TEXT`
- `wiki_page_id TEXT`
- `canonical_entity_ref TEXT`
- `trust_level TEXT NOT NULL DEFAULT 'untrusted_external_evidence'`
- `created_at TEXT NOT NULL`
- `updated_at TEXT NOT NULL`
- `UNIQUE(run_id, stable_key)`

`canonical_entity_ref` examples:

- `x_tweet:<x_id>`
- `source_card:<source_card_id>`
- `wiki_page:<page_id>`
- `github_release:<owner>/<repo>@<tag>`
- `rss_entry:<feed_hash>:<entry_hash>`

Invariants:

- External text is never instruction text.
- Source text written into Markdown or UI is escaped or fenced.
- Unsafe URLs are rejected before source-card/wiki projection.
- If a source-card projection fails, the item remains with explicit
  `projection_failed` status metadata and ops visibility.

### `radar_item_fts`

Purpose: search over normalized radar rows.

Use SQLite FTS5:

```sql
CREATE VIRTUAL TABLE radar_item_fts
USING fts5(id UNINDEXED, title, content_text, author, source_kind);
```

Invariants:

- Every active `radar_items` row with indexable text has one FTS row.
- `radar audit` reports FTS drift.
- Repair can rebuild FTS from `radar_items`.

### `radar_scores`

Purpose: score overlays, not source truth.

Fields:

- `id TEXT PRIMARY KEY`
- `run_id TEXT NOT NULL`
- `item_id TEXT NOT NULL`
- `score_kind TEXT NOT NULL`
- `score REAL NOT NULL`
- `reason TEXT NOT NULL`
- `tags_json TEXT NOT NULL DEFAULT '[]'`
- `model_provider TEXT`
- `model_name TEXT`
- `cost_decision_id TEXT`
- `input_artifact_id TEXT`
- `output_artifact_id TEXT`
- `schema_version INTEGER NOT NULL`
- `status TEXT NOT NULL`
- `error TEXT`
- `created_at TEXT NOT NULL`
- `UNIQUE(item_id, score_kind, schema_version)`

Score kinds:

- `heuristic_v1`
- `model_interestingness_v1`
- `discussion_quality_v1`
- `source_quality_v1`
- `freshness_v1`
- `user_profile_relevance_v1`

Invariants:

- Scores never mutate source rows.
- Model score output is schema-validated.
- Malformed model output records `failed` and cannot silently become `0` unless
  the reason explicitly says `model_output_invalid`.
- Model scoring requires policy and cost decisions before provider calls.
- Private X/DM-like content is hard-excluded from model scoring today; any
  explicit retention/scoring override would require a separate policy design
  and severe test suite before implementation.

### `radar_dedup_groups`

Purpose: exact URL and semantic topic deduplication trace.

Fields:

- `id TEXT PRIMARY KEY`
- `run_id TEXT NOT NULL`
- `dedup_kind TEXT NOT NULL`
- `primary_item_id TEXT NOT NULL`
- `member_item_ids_json TEXT NOT NULL`
- `reason TEXT NOT NULL`
- `confidence REAL NOT NULL`
- `model_provider TEXT`
- `cost_decision_id TEXT`
- `created_at TEXT NOT NULL`

Dedup kinds:

- `canonical_url`
- `same_native_id`
- `same_x_conversation`
- `semantic_topic`

Invariants:

- Dedup never deletes items.
- Dropped digest members stay inspectable.
- Semantic dedupe requires either deterministic evidence or validated model
  output with the exact compared titles/summaries preserved.

### `radar_enrichments`

Purpose: background/context/discussion enrichment for filtered items.

Fields:

- `id TEXT PRIMARY KEY`
- `run_id TEXT NOT NULL`
- `item_id TEXT NOT NULL`
- `language TEXT NOT NULL`
- `whats_new TEXT NOT NULL`
- `why_it_matters TEXT NOT NULL`
- `key_details TEXT NOT NULL`
- `background TEXT NOT NULL DEFAULT ''`
- `community_discussion TEXT NOT NULL DEFAULT ''`
- `source_card_ids_json TEXT NOT NULL DEFAULT '[]'`
- `research_artifact_ids_json TEXT NOT NULL DEFAULT '[]'`
- `model_provider TEXT`
- `model_name TEXT`
- `cost_decision_id TEXT`
- `status TEXT NOT NULL`
- `error TEXT`
- `created_at TEXT NOT NULL`

Invariants:

- Every factual sentence in enrichment must be grounded in the original item,
  linked source cards, URL ingest artifacts, or research artifacts.
- Generated summaries and generated wiki pages do not count as primary
  evidence.
- If grounding is insufficient, enrichment status is `blocked_weak_evidence`.

### `radar_summaries`

Purpose: final report artifacts.

Fields:

- `id TEXT PRIMARY KEY`
- `run_id TEXT NOT NULL`
- `language TEXT NOT NULL`
- `format TEXT NOT NULL`
- `title TEXT NOT NULL`
- `body_markdown TEXT NOT NULL`
- `item_ids_json TEXT NOT NULL`
- `source_card_ids_json TEXT NOT NULL`
- `audit_status TEXT NOT NULL`
- `metadata_json TEXT NOT NULL DEFAULT '{}'`
- `created_at TEXT NOT NULL`
- `UNIQUE(run_id, language, format)`

Formats:

- `brief_markdown`
- `full_report_markdown`
- `telegram_markdown`
- `email_markdown`
- `json_summary`

Invariants:

- The report includes source counts, source-family coverage, score thresholds,
  dedup removals, enrichment failures, delivery status, and caveats.
- The report cannot claim "today's top stories" unless the source-health state
  shows the configured production sources were actually polled within the
  run window.

### `radar_deliveries`

Purpose: authorized update sending.

Fields:

- `id TEXT PRIMARY KEY`
- `run_id TEXT NOT NULL`
- `summary_id TEXT NOT NULL`
- `channel TEXT NOT NULL`
- `recipient_ref TEXT NOT NULL`
- `status TEXT NOT NULL`
- `policy_decision_id TEXT`
- `cost_decision_id TEXT`
- `delivery_attempt_id TEXT`
- `quiet_hours_deferred_until TEXT`
- `idempotency_key TEXT NOT NULL UNIQUE`
- `error TEXT`
- `created_at TEXT NOT NULL`
- `updated_at TEXT NOT NULL`

Invariants:

- No delivery without recipient authorization.
- No delivery to a raw address/chat id unless it is resolved into a durable
  authorized recipient reference.
- Quiet hours defer, not drop.
- Retries reuse the idempotency key.
- Delivery failure is ops-visible and does not mark the run completed unless
  delivery is optional for that profile.

### `radar_source_quality`

Purpose: local HorizonHub-style source quality telemetry.

Fields:

- `id TEXT PRIMARY KEY`
- `run_id TEXT NOT NULL`
- `source_kind TEXT NOT NULL`
- `locator TEXT NOT NULL`
- `window_start TEXT NOT NULL`
- `window_end TEXT NOT NULL`
- `raw_count INTEGER NOT NULL`
- `accepted_count INTEGER NOT NULL`
- `average_score REAL`
- `score_p50 REAL`
- `score_p90 REAL`
- `signal_to_noise REAL`
- `duplicate_rate REAL`
- `delivery_contribution_count INTEGER NOT NULL DEFAULT 0`
- `failure_count INTEGER NOT NULL DEFAULT 0`
- `status TEXT NOT NULL`
- `created_at TEXT NOT NULL`
- `UNIQUE(source_kind, locator, window_start, window_end)`

Invariants:

- Source quality uses local run evidence, not global claims.
- A source can be recommended only if its quality window is current enough or
  clearly labeled `unproven`.
- Decay detection compares windows and records why quality changed.

## Pipeline

### Stage 0: Profile And Source Selection

Inputs:

- radar profile
- watch sources
- X watch-source registry
- explicit source selectors
- source preset library
- source-quality history

Outputs:

- `radar_runs.source_selection_json`
- policy/cost preflight plan for network/model stages

Checklist:

- [x] Implement `radar_profiles` migration and structs.
- [x] Implement profile create/list/read with validation.
- [x] Add profile source selectors for `rss`, `github_release`,
      `github_owner`, `arxiv`, `x_handle`, `source_card_query`, `hackernews`,
      `reddit`, `telegram_public`, `ossinsight`, and `openbb`, with currently
      supported source-card-backed selectors separated from unsupported future
      live adapters.
- [ ] Add source preset import from local JSON.
- [ ] Add Horizon-style keyword/tag source matching.
- [ ] Add AI source recommendation only behind explicit model config, policy,
      cost gate, and schema validation.
- [ ] Add source recommendation output that proposes watch-source additions but
      does not auto-write them without explicit user action.
- [x] Add profile read output showing unsupported or unproven source selectors.

Anti-mirage gate:

- [x] A profile with unsupported selectors must not appear healthy.
- [ ] Source recommendation must distinguish `recommended`, `already_watched`,
      `requires_secret`, `requires_policy`, `unproven_quality`, and
      `unsupported`.

Production-data proof:

- [ ] Run source recommendation against the user's real Arcwell watch-source
      registry plus real source-quality history.
- [ ] Import at least one real preset set containing RSS, GitHub, Reddit,
      Hacker News, and X selectors.
- [ ] Prove no source is auto-added without explicit user approval.

### Stage 1: Fetch

Inputs:

- selected source selectors
- source-health cursors
- source credentials from env or SQLite secret values
- policy and cost decisions

Outputs:

- raw provider payload artifacts
- `radar_items` candidate rows
- source-health updates
- cursor updates only after durable writes

Source adapters:

- RSS/Atom: reuse current wiki RSS adapter where possible.
- GitHub releases/commits/owner events: reuse current wiki GitHub adapters.
- arXiv: reuse current wiki arXiv adapter.
- X handles: reuse existing X recent-search/watch-source paths where
  authorized, with copied/disposable-home proof before promotion.
- Source-card query: read existing local source cards.
- Hacker News: implemented via the official Firebase API with bounded
  top-level comment evidence.
- Reddit: locally proven adapter inspired by Horizon, including JSON top-comment
  capture and RSS fallback. Production-data proof is blocked until OAuth or
  another sanctioned access path passes live gates.
- Public Telegram channels: optional new adapter inspired by Horizon web preview
  scraping. Keep separate from Arcwell's Telegram bot/channel authority.
- OSS Insight: optional new adapter for trending repos.
- OpenBB: optional new adapter for financial/news watchlists.

Checklist:

- [ ] Define adapter trait returning normalized fetch records plus raw artifact
      references.
- [ ] Add fetch ledger fields to `radar_runs`.
- [x] Implement source-card query adapter first, because it uses existing local
      durable data and proves the staged shape.
- [x] Implement RSS/GitHub/arXiv projection from existing wiki jobs/source
      cards before adding new network adapters.
- [x] Add opt-in foreground live RSS/GitHub/arXiv adapter execution through
      existing Arcwell jobs before source-card projection.
- [x] Implement X projection from source-card projections.
- [ ] Implement X projection directly from canonical X rows.
- [x] Add Hacker News adapter with real Firebase API, bounded top-level comment
      capture, source-card persistence, cursor/source-health safety,
      watch-source enqueue support, and adapter-failure audit visibility.
- [x] Add Reddit adapter with JSON listing fetch, RSS-first unauthenticated
      fallback, bounded top-comment capture, rate-limit/error classification,
      source-card persistence, cursor/source-health safety, watch-source enqueue
      support, User-Agent discipline, and severe local tests.
- [ ] Add public Telegram adapter with explicit `telegram_public` source kind,
      HTML parsing as untrusted evidence, and no confusion with authorized bot
      chats.
- [ ] Add OSS Insight adapter with trending repo metadata.
- [ ] Add OpenBB adapter only if optional dependency/runtime boundary is
      acceptable; otherwise use a separate command path and mark source kind
      `requires_optional_runtime`.
- [ ] Add per-source raw payload size caps and content truncation rules.
- [x] Reuse existing RSS/GitHub/arXiv cursor/source-health advancement for
      opt-in foreground live adapter runs, and record radar-owned source-health
      failures when a live selector fails before adapter execution.
- [ ] Add radar-specific source-health statuses beyond the existing adapter
      classification: `empty`, `stale`, `projection_failed`, and richer
      `partial`/`blocked` semantics.

Anti-mirage gate:

- [ ] A fetch run that only returns mock/local fixture rows cannot satisfy
      production proof.
- [x] A fetch run that writes radar rows but skips source cards, source-health,
      or cursors is only `Scaffold`.
- [x] A source with provider errors cannot be hidden under `completed`.

Production-data proof:

- [x] Run against real current RSS feed data.
- [x] Run against real GitHub owner/repo data.
- [x] Run against real arXiv query data.
- [ ] Run against real X watch sources in a copied/disposable home when
      authenticated user-context data is involved.
- [x] Run against real Hacker News top stories.
- [ ] Run against real Reddit subreddits/users configured by profile.
      Current anonymous Arcwell binary attempts are blocked by Reddit HTTP 403;
      use OAuth or another sanctioned access path before promotion.
- [ ] Run against at least one real public Telegram channel only if that source
      kind is enabled.
- [ ] Run against real OSS Insight data if adapter is included.
- [ ] Run against real OpenBB data if adapter is included and credentials or
      provider settings are configured.
- [ ] Production proof must include at least five source families and enough
      real returned rows to exercise dedupe, scoring, filtering, enrichment,
      summary, and delivery. If production sources return too few rows on a
      quiet day, extend the time window rather than shrinking the proof.

### Stage 2: Normalize, Project, And Index

Inputs:

- raw fetch records
- canonical X rows
- existing source cards
- URL ingest results

Outputs:

- `radar_items`
- source cards
- wiki pages
- FTS rows
- projection status metadata

Checklist:

- [ ] Implement canonical URL normalization shared with source-card dedupe.
- [ ] Add stable keys by source kind and native id.
- [ ] Add source-card projection for every accepted external item unless the
      source kind already has a canonical source-card projection.
- [ ] Preserve raw external text as evidence, not instructions.
- [ ] Store item metadata with source-specific metrics: HN score/comments,
      Reddit score/upvote ratio/comments, GitHub repo/tag/event, X metrics,
      RSS feed category/tags, Telegram public message URL, OSS Insight stars,
      OpenBB tickers/provider.
- [x] Add FTS indexing for title/content/author/source kind.
- [x] Add repair command that rebuilds FTS.
- [x] Add audit command that reports item/source-card/FTS drift.

Anti-mirage gate:

- [x] The run is not indexed if `radar_items` exists but FTS rows are missing.
- [ ] The run is not indexed if accepted external items lack source-card or
      explicit no-projection reason.
- [x] The run is not indexed if hostile Markdown/HTML is rendered as trusted
      text.

Production-data proof:

- [ ] On a real production-data run, randomly sample at least 50 accepted items
      or all items if fewer than 50, and verify item -> source card -> wiki page
      -> FTS searchability.
- [ ] Run `radar audit` after indexing and prove zero unexplained projection or
      FTS drift.
- [ ] Run restart/reopen proof: close process, reopen store, read run stages,
      search indexed rows, and repair no-op.

### Stage 3: Exact And Semantic Dedupe

Inputs:

- indexed radar items
- canonical URLs
- source-specific native ids
- title/summary/metadata
- optional model semantic dedupe

Outputs:

- `radar_dedup_groups`
- filtered candidate set for scoring/reporting

Checklist:

- [x] Implement exact URL dedupe.
- [x] Implement source-native dedupe for same X id, same GitHub release, same
      RSS entry id/link, and same provider/native id where existing projected
      source cards expose it.
- [x] Implement cross-source canonical URL merging while preserving all sources.
- [x] Implement semantic topic dedupe only after initial scoring and only with
      preserved evidence.
- [ ] Add model semantic-dedupe output schema with primary id, duplicate ids,
      reason, confidence.
- [ ] Add model semantic-dedupe cost/policy gate.
- [x] Add deterministic local semantic dedupe that does not depend on model
      output and keeps failures from silently hiding items.

Anti-mirage gate:

- [x] Dedupe cannot delete source evidence.
- [x] Dedupe cannot collapse "same product, different event" without explicit
      evidence.
- [ ] Model semantic dedupe failure cannot silently drop items.

Production-data proof:

- [x] Run on real production sources where repeated canonical URLs exist across
      existing source-card rows; copied-home run recorded 26 exact dedupe groups.
- [ ] Run semantic/topic dedupe on real production sources where the same story
      appears in at least two source families.
- [ ] Inspect dedup groups and verify both kept and duplicate source evidence
      remain readable.
- [ ] Seed production run with real common topics, not synthetic duplicates,
      such as a GitHub release appearing in GitHub, HN, Reddit, RSS, or X.

### Stage 4: Interestingness Ranking

Ranking must not be a single opaque model score.

Score layers:

1. `heuristic_v1`
   - recency
   - source family
   - source quality
   - engagement metrics
   - source-card reliability
   - watched org/person/topic signal
   - novelty vs prior runs
   - duplicate/corroboration signal

2. `discussion_quality_v1`
   - HN/Reddit/X/Telegram public comment quality where available
   - disagreement/concern signal
   - technical specificity

3. `source_quality_v1`
   - rolling source signal-to-noise
   - failure/staleness penalties
   - source decay

4. `user_profile_relevance_v1`
   - explicit radar profile interests
   - Arcwell profile preferences only when safe and relevant
   - no private memory leakage into model prompts without explicit design

5. `model_interestingness_v1`
   - optional
   - schema-validated
   - source-grounded
   - cost/policy gated

Checklist:

- [x] Implement score tables and score read APIs.
- [x] Implement deterministic heuristic score first.
- [x] Add explanation fields that name positive and negative signals.
- [ ] Add stale-score labels when source rows or profile config changed after
      scoring.
- [x] Add category quota and max-item selection after scoring.
- [x] Add per-source cap so one source cannot dominate unless profile says so.
- [x] Add model-backed ranking only after deterministic score and stage
      inspection exist.
- [x] Add model output schema validation and malformed-output severe tests.
- [x] Add heuristic score distribution metrics to run metadata and ops:
      `score_distribution` now records run-level count/status/min/max/average
      and p10/p50/p90 metrics for `heuristic_v1` scores, and `/ops/ui` renders
      recent radar runs with score distribution columns. Production-data proof
      passed in
      `.arcwell-dev/proofs/radar-worker-production-proof-20260624T102233Z-28818/artifacts/proof-packet.json`
      over 45 live public RSS/GitHub/arXiv/Hacker News items.

Anti-mirage gate:

- [x] A model score alone cannot authorize delivery.
- [x] A score without reason is invalid.
- [x] A score that references unavailable evidence is invalid.
- [x] Private or unauthorized content cannot be sent to a model as ranking
      context. Local severe tests now prove source-card privacy/model-prompt
      metadata excludes candidates before prompt construction, records
      `model_blocked` rows, omits private token-shaped sentinels from
      input/output artifacts, and skips provider/cost paths when every
      candidate is excluded. The gate also audits excluded rows beyond the
      prompt `max_items` limit, backfills eligible public rows after private
      candidates, recognizes X/DM/email privacy synonyms, and overwrites stale
      model rows when source-card provenance becomes unverifiable.

Production-data proof:

- [ ] Score a real production run with at least five source families.
- [x] Score a fresh public-source worker run with four source families through
      the live OpenAI model-score overlay:
      `scripts/radar-model-score-production-proof` passed at
      `.arcwell-dev/proofs/radar-model-score-production-proof-20260624T100638Z-66866`
      with 45 heuristic rows, 30 selected heuristic rows, 3
      `model_interestingness_v1` rows, audit-ok after model scoring, and
      unchanged source-quality raw/accepted totals.
- [ ] Compare top 25 and bottom 25 items manually or with an adversarial review
      artifact.
- [ ] Prove score reasons cite actual available fields.
- [ ] Prove category balancing changes at least one real run where a source
      family would otherwise dominate.
- [ ] Record cost decisions and actual provider usage where available for
      model scoring.

### Stage 5: Filter And Balance Digest

Inputs:

- scored rows
- dedup groups
- profile category quotas
- source quality

Outputs:

- final selected item set
- rejected-but-inspectable rows with reasons

Checklist:

- [x] Implement score threshold filter.
- [x] Implement per-category group quotas.
- [x] Implement global max item cap.
- [x] Implement per-source max cap.
- [ ] Implement "must include if critical" override only with explicit profile
      config and audit note.
- [x] Store rejection reasons for `below_threshold`, exact duplicate statuses,
      `category_quota`, `source_quota`, and `over_profile_limit`.
- [x] Store deterministic semantic rejection reason `duplicate_topic`.
- [ ] Store later-stage rejection reasons for `unsafe_source`, `weak_evidence`,
      and `delivery_policy_denied`.

Anti-mirage gate:

- [x] Rejected threshold/duplicate/quota/global-limit items remain inspectable
      through `radar stage` item and score rows.
- [ ] Empty digest must explain whether the day was quiet, sources failed,
      threshold was too high, or scoring failed.
- [ ] A filtered run cannot be marked healthy if all source families failed.

Production-data proof:

- [ ] Run threshold and balancing on a real high-volume window.
- [ ] Prove selected and rejected counts by source family.
- [ ] Prove an empty digest path on real data by using a deliberately high
      threshold, and verify it reports threshold cause rather than pretending
      no sources exist.

### Stage 6: Enrichment

Inputs:

- selected items
- original source cards
- expanded safe URLs
- local wiki pages
- research artifacts where needed

Outputs:

- `radar_enrichments`
- linked source-card/research evidence

Checklist:

- [ ] Extract concepts that need background only from title, content, summary,
      tags, or comments.
- [ ] Use Arcwell URL ingest and local source cards for web grounding instead
      of ad hoc untracked search.
- [ ] Use host-native search or `research_web_search` only when explicitly
      configured for enrichment and record the proof.
- [ ] Store linked source-card ids for every enrichment.
- [ ] Add `blocked_weak_evidence` when background cannot be grounded.
- [ ] Add community discussion summaries only from captured HN/Reddit/X/public
      Telegram discussion text.
- [ ] Add language-specific enrichment fields.
- [ ] Add citation verification pass for model-generated enrichment.

Anti-mirage gate:

- [ ] Enrichment without linked evidence is not complete.
- [ ] Web search snippets alone are not enough for high-confidence background.
- [ ] Generated reports, expanded pages, and summaries cannot recursively
      ground enrichment.

Production-data proof:

- [ ] Enrich at least 25 real selected production items or all selected items
      if fewer than 25 in a high-volume run.
- [ ] For each enriched item in the proof sample, verify at least one source
      card or original source supports the enrichment.
- [ ] Include at least one item with community discussion and one without, and
      prove both render correctly.

### Stage 7: Summary And Report Writing

Outputs:

- executive digest
- detailed report
- item-by-item appendix
- evidence appendix
- source-health appendix
- delivery-ready variants

Report sections:

1. Title and run metadata.
2. Bottom line: what matters today.
3. Top items with score, reason, source, and evidence links.
4. Category-balanced sections.
5. What changed since previous run.
6. Source family coverage.
7. Weak evidence and blocked enrichment.
8. Delivery status.
9. Evidence appendix.
10. Method notes and caveats.

Checklist:

- [x] Implement deterministic Markdown renderer over `radar_summaries`.
- [ ] Implement compact Telegram renderer with safe Markdown and length caps.
- [ ] Implement email renderer with inert Markdown/HTML conversion policy.
- [ ] Implement JSON renderer for programmatic consumers.
- [x] Include source-card ids and URLs for each selected item.
- [x] Include dedupe and rejection/status stats.
- [ ] Include full score distribution and category quotas.
- [ ] Include source-health status at time of run.
- [ ] Include stale/failed/missing source warnings in the executive caveats.
- [ ] Add no-write mode that renders but writes no summary rows.

Anti-mirage gate:

- [ ] A report cannot say "daily" or "latest" if the run window/source health
      does not prove current polling.
- [ ] A report cannot hide failed source families.
- [ ] A report cannot omit caveats when enrichment/audit failed.
- [ ] A report cannot cite generated summaries as source evidence.

Production-data proof:

- [ ] Generate reports from real production runs for at least three profiles:
      `agent-infrastructure`, `security-sandboxing`, and `market-ecosystem`.
- [ ] Each report must include at least five source families unless a profile
      explicitly narrows the source set and says so.
- [ ] Run adversarial report review that scores source coverage, ranking
      usefulness, evidence support, contradiction handling, delivery readiness,
      and caveat honesty. No production promotion if any score is below 4/5.

### Stage 8: Delivery

Delivery routes:

- Telegram authorized subjects through existing Telegram/channel delivery.
- Email authorized recipients through existing email send/reply infrastructure.
- Local wiki/source-card digest page.
- Optional webhook only after an Arcwell-owned webhook delivery model exists.

Checklist:

- [x] Add recipient authorization lookup.
- [x] Add local scheduled-delivery profile config for channel, recipient,
      interval, summary language, and summary format.
- [x] Add policy check before delivery.
- [x] Add cost check where provider delivery has cost.
- [x] Add idempotency keys per run/summary/recipient.
- [x] Add local quiet-hours deferral for scheduled Telegram delivery.
- [x] Add local Telegram retry reconciliation for manual radar deliveries:
      worker-driven successful retries update the original `radar_deliveries`
      row and exhausted local retry chains become `dead_lettered`.
- [x] Add local cross-channel/scheduled retry with bounded attempts and
      dead-letter behavior: due failed email messages retry from configured
      Cloudflare Email secrets without duplicate channel messages, successful
      scheduled email retries reconcile tick/delivery/run state, and exhausted
      retry chains dead-letter the channel message, radar delivery, and
      schedule tick without leaking tokens.
- [x] Add delivery attempt records linked to `radar_deliveries`.
- [x] Add delivery status to ops snapshot and `/ops/ui`.
- [x] Add manual `radar deliver` confirmation path with CLI/MCP/slash surfaces,
      durable `radar_deliveries`, idempotency, authorization/policy gates,
      provider-failure recording, and local severe tests.
- [x] Add local scheduled Telegram/email delivery through the resident worker after
      manual delivery proof: scheduled profiles write durable schedule ticks,
      enqueue `radar_scheduled_delivery`, run/summarize/audit, deliver through
      configured authorized Telegram or Cloudflare Email, link
      tick/run/summary/delivery lineage, suppress duplicate ticks inside the
      configured interval, reject raw secrets in profile policy, block
      unauthorized email recipients before provider sends, and defer active
      quiet-hours without provider sends.
- [x] Add repeatable production-data scheduled-delivery proof with controlled
      Telegram provider delivery: `scripts/radar-scheduled-delivery-production-proof`
      creates a disposable scheduled profile over real public RSS/GitHub/arXiv/
      Hacker News sources, drains the resident worker, verifies real
      indexed/scored items plus healthy cursor/source-health state, sends one
      audit-ok summary through a controlled Telegram endpoint, records
      tick/run/summary/delivery lineage, and proves duplicate suppression on a
      second worker pass.
- [x] Prove local scheduled email delivery with controlled provider success and
      authorization blocking: resident worker enqueues one schedule tick,
      completes `radar_scheduled_delivery`, records run/summary/delivery
      lineage, reaches `sent` through the Cloudflare Email path, avoids
      duplicate ticks inside the interval, redacts configured tokens, and
      writes no channel message/provider attempt for unauthorized recipients.
- [x] Prove local scheduled email retry/dead-letter reconciliation with
      controlled provider failure/success: failed scheduled email delivery
      records a retryable channel attempt, worker retry reuses the same channel
      message, successful retry promotes the schedule tick, radar delivery, and
      run, exhausted retries dead-letter all three ledgers, and serialized
      worker output redacts configured tokens.
- [ ] Add live external scheduled delivery proof, long-running service proof,
      production quiet-hours deferral, and production cross-channel scheduled
      delivery.

Anti-mirage gate:

- [x] Generating a summary is not delivery.
- [ ] Sending to a test chat/address is not production delivery unless it uses
      the real Arcwell authorization, policy, cost, delivery-attempt, and retry
      surfaces.
- [x] A failed delivery cannot be hidden by marking the run completed unless
      delivery was explicitly optional.

Production-data proof:

- [ ] Send one real production-data digest to an authorized Telegram recipient
      or disposable authorized Telegram test chat.
- [ ] Send one real production-data digest to an authorized email recipient or
      disposable authorized email route.
- [x] Prove local quiet-hours deferral with a deferred worker job/tick and no
      provider send.
- [x] Prove retry/dead-letter with controlled provider failure/success, without
      leaking secrets.
- [x] Prove local Telegram retry/dead-letter behavior with controlled provider
      failures and no real provider send.
- [x] Prove local scheduled Telegram delivery with controlled provider success:
      resident worker enqueues one schedule tick, completes the
      `radar_scheduled_delivery` job, records run/summary/delivery lineage,
      reaches `sent`, avoids duplicate ticks inside the interval, rejects raw
      secrets in profile policy, and defers active quiet-hours without provider
      sends.
- [x] Prove local scheduled email delivery with controlled provider success and
      recipient authorization failure: resident worker enqueues one schedule
      tick, completes `radar_scheduled_delivery`, records
      run/summary/delivery lineage, reaches `sent` through the Cloudflare Email
      path, avoids duplicate ticks inside the interval, redacts configured
      tokens, and writes no channel message/provider attempt for unauthorized
      recipients.
- [x] Prove local scheduled email retry/dead-letter behavior: retry uses the
      existing channel message, success reconciles the schedule tick, radar
      delivery, and run to `sent`/`delivered`, and exhausted failure reconciles
      the schedule tick, radar delivery, and channel message to `dead_lettered`
      without another retry storm.
- [x] Prove production-data scheduled delivery with real public-source ingestion
      and controlled Telegram provider delivery using
      `scripts/radar-scheduled-delivery-production-proof`: real items are
      ingested/indexed/scored from RSS/GitHub/arXiv/Hacker News with healthy
      cursor/source-health state, selected items are summarized after audit,
      one provider request is sent to a sanitized controlled Telegram endpoint,
      and duplicate schedule enqueue is suppressed on the second worker pass.

### Stage 9: Source Quality And Recommendation Loop

Inputs:

- production run history
- source health
- score distributions
- duplicate rates
- delivery contributions
- removal/disable reasons

Outputs:

- local source-quality records
- source recommendations
- source decay warnings

Checklist:

- [x] Compute per-source signal-to-noise for local source-quality windows and
      historical trend rankings.
- [ ] Compute p50/p90 scores per source over rolling windows.
- [x] Compute duplicate rate for local source-quality windows and historical
      trend rankings.
- [ ] Compute output frequency.
- [x] Detect local source-quality improvement/decay/failure from durable
      historical windows.
- [ ] Detect source staleness over real scheduled history.
- [ ] Recommend complementary sources based on category gaps and source
      quality.
- [ ] Flag overlapping sources where dedupe shows persistent duplication.
- [ ] Add source removal feedback recording.
- [x] Add source-quality section in ops snapshot and `/ops/ui`, with
      non-healthy health warnings, summary scoring, filtered rows, and escaping
      coverage for hostile source locators.

Anti-mirage gate:

- [ ] Source recommendations cannot claim quality without local quality data.
- [ ] Global/community quality is future work unless an explicit Arcwell Hub is
      built. Do not imply HorizonHub telemetry exists locally.

Production-data proof:

- [x] Local single-run source-quality windows are materialized from scored
      `radar_items` / `radar_scores`, exposed through `radar stage` and
      `radar_source_quality`, and audited for missing/drifted rows.
- [x] Local source-quality windows are exposed to operators through
      `ops_snapshot` and `/ops/ui`; non-healthy windows affect health warnings
      and UI health scoring.
- [x] Local source-quality trend/ranking surfaces aggregate durable historical
      windows through CLI/MCP/slash/resource access and are severe-tested for
      thin history, decay/failure labels, hostile locators, ranking, and invalid
      bounds.
- [x] Repeated live-run source-quality ranking proof passed at
      `.arcwell-dev/proofs/radar-source-quality-trends-proof-20260624T090251Z-4856`:
      two public GitHub/arXiv/Hacker News runs wrote 50 radar items/scores, six
      source-quality windows, three trend rows, clean audits, healthy
      source-health, and cursors for all three public source families.
- [ ] Run at least seven days of real scheduled or manually repeated
      production radar runs before claiming decay/quality trend behavior.
- [x] Show at least one source-quality ranking generated from real local run
      history.
- [ ] Show at least one recommended source and one overlap warning, with the
      local evidence used.

## Worker And Scheduling Design

Worker job kinds:

- `radar_run`
- `radar_fetch`
- `radar_score`
- `radar_filter`
- `radar_enrich`
- `radar_summarize`
- `radar_deliver`
- `radar_source_quality_rollup`

Checklist:

- [x] Add local worker `radar_run` execution for whole-profile runs.
- [ ] Add job input schema validation for every radar job kind.
- [ ] Add worker execution that resumes from durable per-stage runs.
- [ ] Add idempotency keys for stage jobs.
- [ ] Add stale-lease recovery.
- [ ] Add stop/cancel semantics before next expensive action.
- [ ] Add retry/backoff by error class.
- [ ] Add dead-letter records with redacted errors.
- [ ] Add ops visibility for radar jobs and stale runs.

Production-data proof:

- [x] Queue a real production-data radar run and drain it through
      `arcwell worker run-once` without manual stage calls.
- [ ] Kill or interrupt after fetch, resume, and prove no duplicate source
      cards, no cursor corruption, and no duplicate delivery.
- [ ] Let a provider/source fail and prove the run becomes partial/blocked with
      source-health evidence.

## Real Production Data Proof Profiles

Mocks, toy fixtures, and small synthetic datasets are useful for unit tests,
but they do not satisfy completion. The production proof must use real source
data at natural volume.

### Profile A: Agent Infrastructure

Required source families:

- real RSS feeds
- real GitHub releases or owner activity
- real Hacker News stories
- real Reddit discussions
- real X watch-source rows from copied/disposable authenticated home or public
  recent search if credentials permit
- real arXiv or web/research source cards where relevant

Minimum proof:

- [ ] Fetch at least a full 7-day window if 24 hours produces too little data.
- [ ] Include at least 100 raw items or document why the real configured source
      universe produced fewer after a 7-day window.
- [ ] Select at least 15 digest candidates after scoring unless source volume
      genuinely prevents it.
- [ ] Enrich at least 10 items.
- [ ] Produce full report and delivery-ready summary.

### Profile B: Security And Sandboxing

Required source families:

- security RSS feeds
- GitHub repos/releases for sandbox/runtime projects
- HN/Reddit security or programming discussions
- arXiv or standards/document source cards
- X/watch-source rows if configured

Minimum proof:

- [ ] Include source-family caveats because security topics are high-risk.
- [ ] Verify no generated advice is delivered as operational security guidance
      without citations.
- [ ] Include adversarial report review focused on overclaiming and stale
      vulnerability data.

### Profile C: Market Ecosystem

Required source families:

- company/blog RSS
- GitHub/project release data
- X/watch-source rows
- news/search source cards
- optional OpenBB company news
- optional Reddit/HN discussion

Minimum proof:

- [ ] Include currentness labels by item.
- [ ] Separate launch claims, funding claims, hiring claims, product claims,
      and community reaction.
- [ ] Mark weak/company-only evidence clearly.

### Profile D: Source Quality Stress Run

Purpose: prove source-quality and balancing.

Required:

- [x] Run with enough sources that one family would dominate without quotas:
      `scripts/radar-balance-production-proof` selected 4 live items from 52
      scored rows and produced 36 inspectable `source_quota` rows.
- [ ] Show source-quality table.
- [x] Show source-family category quota effects:
      `scripts/radar-category-balance-production-proof` selected 12 live items
      from 116 scored rows and produced 76 inspectable `category_quota` rows
      across all four configured source-family categories in two quota
      configurations.
- [ ] Show overlap/dedupe groups.
- [ ] Show at least one stale/failing source or controlled disabled source
      surfaced honestly.

## Severe Test Matrix

### Schema And Migration

- [ ] Empty database migration.
- [ ] Populated database migration.
- [ ] Old-schema fixture migration.
- [ ] Rerun migration idempotency.
- [ ] Backup/restore drill with radar rows.
- [ ] Corrupt `metadata_json` row reports clear error or repair path.
- [ ] Missing FTS table rebuild.

### Source Fetch

- [ ] RSS malformed XML.
- [ ] RSS duplicate GUID/link.
- [ ] RSS private/metadata redirect rejected when URL ingest is used.
- [ ] GitHub 401/403/429/5xx.
- [ ] HN deleted/dead/comment HTML.
- [ ] Reddit 429 and RSS fallback.
- [ ] Reddit blocked JSON response.
- [ ] Public Telegram hostile HTML/Markdown.
- [ ] X expired token, forbidden tier, rate limit, partial data, malformed row.
- [ ] OSS Insight empty/malformed rows.
- [ ] OpenBB optional dependency missing.
- [ ] All rows rejected does not advance cursor.
- [ ] Partial provider failure records source-health `partial`.

### Normalization And Indexing

- [ ] Duplicate stable keys.
- [ ] Duplicate canonical URLs.
- [ ] Hostile title/content with script tags.
- [ ] Markdown image/link injection.
- [ ] Unicode RTL/control characters.
- [ ] Huge content truncation.
- [ ] Null bytes.
- [ ] FTS drift.
- [ ] Source-card projection failure rollback.
- [ ] Process interruption after source-card write before radar item write.

### Ranking

- [ ] Score bounds reject NaN, infinity, negative, over 10.
- [ ] Missing score reason invalid.
- [ ] Malformed model JSON rejected.
- [ ] Prompt-injection content cannot alter scoring schema.
- [x] Cost denial blocks model scoring but leaves deterministic scoring.
- [ ] Private content excluded from model prompt.
- [ ] Stale score detected after item/profile change.

### Dedupe

- [x] Same URL across source families groups correctly.
- [x] Same product different events stay separate.
- [ ] Model dedupe parse failure keeps items.
- [x] Duplicate group preserves all member evidence.
- [x] Dedup does not affect source-quality raw counts incorrectly.

### Enrichment

- [ ] Enrichment refuses unsupported facts.
- [ ] Search/provider blocked records `blocked_weak_evidence`.
- [ ] Generated summary recursion rejected.
- [ ] Citation URL not in evidence rejected.
- [ ] Community discussion prompt injection quoted as data.
- [ ] Bilingual output validates language fields if multilingual is enabled.

### Report Writing

- [ ] Empty digest report explains cause.
- [ ] Failed source families appear in caveats.
- [ ] Source-health stale state appears in caveats.
- [ ] Refuted/weak evidence cannot appear as confident conclusion.
- [ ] Markdown/HTML escaped in report.
- [ ] Long report bounded.
- [ ] Telegram compact renderer length cap.
- [ ] Email renderer strips active HTML.

### Delivery

- [x] Unauthorized recipient blocked.
- [x] Local quiet-hours deferral.
- [x] Duplicate delivery idempotency.
- [x] Provider failure retry for local Telegram manual delivery.
- [x] Retry exhaustion dead-letters local Telegram manual delivery.
- [x] Local scheduled Telegram/email delivery through resident worker.
- [x] Delivery error redacts tokens/addresses where required.
- [x] Policy denial records decision.
- [x] Cost denial records decision.

### Ops

- [ ] Radar run visible in ops.
- [ ] Radar stale run visible in ops.
- [ ] Radar failed source-health visible in ops.
- [x] Radar delivery failures visible in ops.
- [ ] Ops UI escapes hostile item text.
- [ ] Doctor reports radar drift.

## Implementation Order

### Phase 0: Design Lock And Guardrails

- [ ] Add this plan.
- [ ] Add `packages/arcwell-radar/README.md` with `Status: Missing/Design`.
- [ ] Add `TODO.md` references only after user approves touching the dirty
      planning files.
- [ ] Add `STATUS.md` row only when first code lands.
- [ ] Define proof packet template for radar features.

Exit gate:

- [ ] User and implementer agree that Horizon is a design input, not a runtime
      dependency.
- [ ] No code claims exist yet.

### Phase 1: Staged Local Skeleton

- [ ] Add radar tables.
- [ ] Add structs and row mappers.
- [ ] Add profile CRUD.
- [ ] Add run create/read/list.
- [ ] Add stage read API.
- [ ] Add source-card query adapter only.
- [ ] Add deterministic normalization into `radar_items`.
- [ ] Add FTS indexing.
- [ ] Add audit/repair for item/source-card/wiki/FTS drift.
- [ ] Add CLI/MCP tools for profile, run, stage, audit.
- [ ] Add severe local tests.

Exit gate:

- [ ] Production proof over existing real local source cards, not synthetic
      toy data.
- [ ] Database reopen and repair no-op proof.

### Phase 2: Existing Arcwell Source Integration

- [ ] Integrate RSS/GitHub/arXiv watch-source outputs.
- [ ] Integrate X canonical rows and source-card projections.
- [ ] Integrate digest candidate creation from selected radar rows.
- [ ] Add source-health/cursor propagation.
- [x] Add local worker `radar_run` job.
- [ ] Add resumable worker `radar_fetch` stage job.

Exit gate:

- [ ] Real production run over existing Arcwell watch sources.
- [ ] At least five source families if configured; otherwise document missing
      source families and keep status below `Production Data Proof`.

### Phase 3: Horizon-Inspired New Adapters

- [x] Hacker News foreground adapter for top/new/best/ask/show/jobs feeds with
      bounded top-level comment evidence.
- [x] Reddit local adapter for JSON listing/comment capture plus RSS fallback.
- [ ] Reddit production-data proof through OAuth or sanctioned access.
- [ ] Public Telegram.
- [ ] OSS Insight.
- [ ] OpenBB optional.
- [ ] Source preset import and recommendation.

Exit gate:

- [ ] Each adapter has a real production-data smoke and source-health proof.
- [ ] Each adapter has severe malformed/rate-limit tests.

### Phase 4: Ranking, Dedupe, Balance

- [x] Heuristic scoring.
- [x] Score explanations.
- [x] Exact URL/native dedupe.
- [x] Local deterministic semantic/topic dedupe.
- [x] Copied-home production-data deterministic semantic/topic dedupe breadth
      over the real source-card corpus.
- [x] Local deterministic category/source balancing.
- [x] Optional local/mock model scoring overlays.
- [ ] Optional model semantic dedupe.

Exit gate:

- [ ] Real production ranking review across at least three profiles.
- [ ] Prove category/source balancing changes at least one real production-data
      run where a source family would otherwise dominate.
- [ ] Model scoring cannot promote if deterministic scoring/audit fails.

### Phase 5: Enrichment And Report Writing

- [ ] Evidence-grounded enrichment.
- [ ] Citation verification.
- [x] Deterministic Markdown summaries over selected scored items.
- [ ] Detailed reports.
- [ ] Telegram/email renderers.
- [ ] Report audit.

Exit gate:

- [ ] Real production reports for Agent Infrastructure, Security/Sandboxing,
      and Market Ecosystem profiles.
- [ ] Adversarial review average score at least 4/5, no blocking findings.

### Phase 6: Delivery And Scheduling

- [x] Manual authorized delivery.
- [x] Local quiet-hours deferral.
- [x] Local Telegram retry reconciliation/dead-letter for manual deliveries.
- [x] Local scheduled Telegram/email delivery through `worker run-once` with
  durable ticks, authorized provider attempts, duplicate suppression,
  raw-secret rejection, email authorization blocking, and explicit quiet-hours
  deferral.
- [x] Local cross-channel/scheduled retry/dead-letter through `worker run-once`
  and reconciliation: Telegram and email retry attempts feed radar delivery
  status, and scheduled email retry updates linked schedule ticks.
- [x] Production-data scheduled delivery proof through `worker run-once` with
  live public-source ingestion and controlled Telegram provider delivery.
- [ ] Live external scheduled delivery and production scheduled worker/service
  runs.
- [ ] Source-quality rollups.
- [x] Ops snapshot/UI visibility for manual delivery attempts.
- [ ] Doctor visibility and broader delivery controls.

Exit gate:

- [ ] Real production-data digest delivered to authorized Telegram/email target.
- [ ] Scheduled run completes without manual stage calls.
- [ ] Interrupted run resumes without duplicate delivery.

### Phase 7: Operational Hardening

- [ ] Seven-day source-quality trend proof.
- [ ] Backup/restore drill.
- [ ] Performance/stress proof over large production windows.
- [ ] Ops UI browser validation desktop/mobile.
- [ ] Documentation and plugin parity.
- [ ] Release readiness smoke.

Exit gate:

- [ ] Operational scorecard has no category below 4/5.
- [ ] `STATUS.md`, `TODO.md`, package README, CLI, MCP, slash commands, and
      tests agree.

## Production Proof Commands

Exact scripts should be added as the implementation lands. Target shape:

```sh
cargo fmt -- --check
cargo test --all --all-features
cargo test -p arcwell-core radar_ -- --nocapture
cargo test -p arcwell severe_radar -- --nocapture
scripts/radar-production-proof --profile agent-infrastructure --window-hours 168
scripts/radar-production-proof --profile security-sandboxing --window-hours 168
scripts/radar-production-proof --profile market-ecosystem --window-hours 168
scripts/radar-delivery-proof --run-id <run-id> --channel telegram --recipient <authorized-test-subject>
scripts/radar-delivery-proof --run-id <run-id> --channel email --recipient <authorized-test-recipient>
scripts/arcwell-dev smoke
scripts/arcwell-dev sync
scripts/verify-codex-plugin-docs
```

For X user-context production proof, use a copied/disposable source home and do
not rewrite the real watch list unless the user explicitly requests it:

```sh
set -a
. ./.env
set +a
X_USER_CONTEXT_SOURCE_HOME="$ARCWELL_HOME" scripts/radar-production-proof --profile agent-infrastructure --window-hours 168
```

## Proof Packet For Every Phase

Each phase must produce a packet with:

- [ ] Feature name and status.
- [ ] User-visible claim in one sentence.
- [ ] Exact accepted inputs and promised outputs.
- [ ] Durable rows/files written.
- [ ] Source families used.
- [ ] Real production data window.
- [ ] Row counts by source family.
- [ ] Cursor and source-health before/after.
- [ ] Score distribution.
- [ ] Dedupe groups.
- [ ] Rejected item reasons.
- [ ] Enrichment evidence links.
- [ ] Summary/report artifact paths.
- [ ] Delivery attempt rows, if applicable.
- [ ] Policy and cost decisions.
- [ ] Secret redaction evidence.
- [ ] Commands run.
- [ ] Adversarial review findings.
- [ ] Remaining risks.
- [ ] Promotion decision: promote, hold, or block.

## Things That Must Not Be Claimed

- Do not claim "Horizon integrated" because Horizon was cloned or its docs were
  summarized.
- Do not claim "digest works" because a Markdown string was generated.
- Do not claim "delivery works" because a summary exists.
- Do not claim "current" or "latest" without source-health and cursor proof.
- Do not claim "ranked intelligently" because a model returned scores.
- Do not claim "source quality" without real run history.
- Do not claim "production data proof" from synthetic fixtures.
- Do not claim "scheduled" from a foreground manual command.
- Do not claim "safe" without malicious input and policy/cost/secret tests.

## First Implementation Slice Recommendation

Start with the smallest real slice that still threatens the mirage:

1. `radar_profiles`, `radar_runs`, `radar_items`, `radar_item_fts`.
2. Source-card query adapter over existing real Arcwell source cards.
3. Run create/fetch/index/stage-read/audit.
4. Production proof over the user's current real local source-card corpus.
5. Then integrate existing RSS/GitHub/arXiv/X watch-source outputs.

Do not begin with model scoring or delivery. Those are where a fake shell would
look impressive fastest. The durable ingestion/indexing spine must be real
first.
