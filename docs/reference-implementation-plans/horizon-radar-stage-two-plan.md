# Horizon Radar Stage Two Plan

Date: 2026-06-24

Status: design and implementation plan only. No Arcwell feature is implemented
by this file.

Reference repo: https://github.com/Thysrael/Horizon

Reference commit inspected: `c822780`

Local inspection path: `/tmp/arcwell-reference-repos/Horizon`

## Claim Boundary

This plan can claim that Horizon source code was inspected and that this file
maps the useful pipeline ideas onto Arcwell radar/source-card/wiki surfaces.

This plan cannot claim that Arcwell's existing Horizon-inspired radar is
complete or that any new stage-two capability has shipped.

## Source And Code Inspected

- `src/orchestrator.py`
- `src/models.py`
- `src/ai/analyzer.py`
- `src/ai/summarizer.py`
- `src/storage/manager.py`
- `src/mcp/service.py`
- `src/mcp/run_store.py`
- `src/setup/ai_recommend.py`
- `src/scrapers/base.py`
- `tests/test_balanced_digest.py`

## What Horizon Does Well

Horizon is a multi-source digest pipeline. Arcwell has already borrowed from
this family of ideas, but the inspected source still has several strong
patterns worth making explicit:

- `ContentItem` is a unified item shape for GitHub, Hacker News, RSS, Reddit,
  Telegram, Twitter, OpenBB, and OSS Insight.
- The orchestrator has distinct stages: fetch, cross-source merge, AI analysis,
  threshold filtering, semantic topic dedup, optional reply expansion,
  balanced digest selection, enrichment, summary, and delivery.
- URL dedup normalizes host/path and merges metadata rather than dropping the
  poorer duplicate silently.
- Semantic dedup is allowed to fail closed to unchanged items if AI fails.
- Balanced digest quotas prevent one source/category from taking the whole
  report.
- The summarizer renders deterministic markdown rather than asking the model to
  invent the final document shape.
- MCP exposes staged operations and stage reads instead of only a single opaque
  `run_pipeline` button.
- `RunStore` keeps raw/scored/filtered/enriched artifacts with path validation.
- Setup can recommend sources from user interests.

The balanced digest tests are a good example of a refuting test shape: they
would fail if balancing happened after enrichment, if categories duplicated
incorrectly, or if default groups swallowed the whole digest.

## Arcwell-Native Shape

Arcwell should turn its radar pipeline into a staged, inspectable, source-card
first workflow:

- Source adapters fetch durable rows.
- Source cards retain provenance and raw evidence.
- Radar runs produce stage artifacts visible by CLI/MCP/slash.
- Scoring, filtering, deduping, balancing, enrichment, and digest rendering are
  separately inspectable.
- Delivery is another stage with delivery-attempt rows.

Working name: `radar stage two`

This is an expansion of Arcwell radar, not a new digest app.

## Proposed Data Model

Use existing radar/source-card tables where possible, but add stage-level
observability if not already present:

- `radar_stage_artifacts`
  - `id`
  - `run_id`
  - `stage`
  - `artifact_kind`
  - `item_count`
  - `storage_ref`
  - `summary_json`
  - `created_at`

- `radar_item_scores`
  - `run_id`
  - `source_card_id`
  - `score`
  - `score_reason`
  - `score_model`
  - `score_error`

- `radar_item_dedup_groups`
  - `run_id`
  - `group_id`
  - `dedup_kind`
  - `canonical_source_card_id`
  - `member_source_card_ids`
  - `reason`

- `radar_balancing_rules`
  - `profile_id`
  - `category`
  - `max_items`
  - `min_items`
  - `source_weight_json`

- `radar_delivery_attempts`
  - `id`
  - `run_id`
  - `delivery_kind`
  - `recipient_ref`
  - `status`
  - `error`
  - `created_at`
  - `completed_at`

## CLI, MCP, Slash, And Ops Surfaces

CLI:

- `arcwell radar run --profile <id> --stage fetch|score|filter|digest|deliver`
- `arcwell radar stage <run-id> <stage>`
- `arcwell radar explain <run-id> --source-card <id>`
- `arcwell radar balance-preview <run-id>`
- `arcwell radar recommend-sources --interests <text>`

MCP:

- `radar_fetch_items`
- `radar_score_items`
- `radar_filter_items`
- `radar_stage_read`
- `radar_generate_digest`
- `radar_delivery_status`

Slash/plugin:

- `/radar-run`
- `/radar-stage`
- `/radar-summary`

Ops:

- Per-source fetch status, cursor health, item counts, score failures, dedup
  groups, balance decisions, delivery attempts.

## Implementation Plan

1. Inventory current Arcwell radar stages.
   - Do not duplicate existing tables or commands.
   - Add missing observability only where the current pipeline is opaque.

2. Make stage artifacts first-class.
   - Every stage writes an artifact row or explicit skipped/error row.
   - Stage reads never rely on generated prose as evidence.

3. Harden fetch and source-card linkage.
   - Cursor advances only after durable source cards and indexes.
   - Failed source does not poison the entire run.

4. Add URL and semantic dedup.
   - URL dedup deterministic first.
   - Semantic dedup optional and model-output validated.
   - Preserve all member provenance.

5. Add balanced digest rules.
   - Rules are profile configuration, not prompt text.
   - Tests cover quotas, default category, duplicates, and source diversity.

6. Add deterministic digest renderer.
   - Model may summarize an item.
   - Program code owns the final report structure, citations, and sectioning.

7. Add source recommendation.
   - Recommendations are candidates, not active watchers.
   - User approval required before creating watch sources.

8. Add delivery proof.
   - Email/webhook/channel delivery writes attempts.
   - A rendered digest is not a delivery.

## Anti-Mirage Traps

- Fetched items are not scored items.
- Scored items are not filtered items.
- Filtered items are not a delivered digest.
- Generated summaries are not citations.
- A run artifact file is not source-card proof.
- A live fetch without source-health/cursor rows is not "current."
- Source recommendation is not watch-source activation.

## Proof Gates

- Missing: production-data balance review, source recommendation, model
  enrichment, live external delivery proof, production cross-channel scheduled
  delivery, and scheduled/recurring service operation remain absent or
  unproven.
- Scaffold: stage-two command names and data-model ideas exist in planning
  docs.
- Local Proof: current Arcwell radar writes inspectable `radar_runs`,
  `radar_items`, FTS rows, score rows, exact URL/source-native dedupe groups,
  deterministic semantic/topic dedupe groups, local source/category quota
  statuses, summary artifacts, and source-card/wiki links; severe tests cover
  foreground and queued worker execution, provider-denial blocked runs, invalid
  enqueue, malformed balance caps, source/category dominance, semantic dedupe
  evidence preservation, same-product/different-event separation, dedupe score
  drift, generated-summary/no-delivery boundaries, manual radar delivery
  authorization/idempotency/provider-failure boundaries, local scheduled
  Telegram/email delivery through the resident worker, email authorization
  blocking, quiet-hours deferral, raw secret rejection, FTS drift, corrupt
  dedupe groups, and
  prompt-injection-as-evidence rendering.
- Production Data Proof: copied-home source-card projection, foreground public
  RSS/GitHub/arXiv/Hacker News live fetch, worker-drained public
  RSS/GitHub/arXiv/Hacker News runs, copied-home semantic/topic dedupe review,
  repeated live-run source-quality ranking, and scheduled live public-source
  ingestion with controlled Telegram provider delivery have passed real-data
  proof packets.
- Partial: Reddit has local proof but anonymous live attempts hit HTTP 403; X
  is limited to existing local source-card/canonical projections until
  authenticated live proof passes.
- Operational: long-running scheduled service execution, live external
  notification routing, production cross-channel scheduled retry/dead-letter,
  stale/failed source-health recovery, and ops controls still need proof.
- Done: every claimed source family satisfies the real-data gate and the docs
  distinguish fetch, digest, and delivery claims.

## Severe Tests

- Invalid run ID or language cannot traverse artifact paths.
- One source fails while other sources complete and run status is partial.
- AI scoring returns invalid JSON; item gets fallback state and error record.
- Dedup model returns malformed groups; original items survive.
- URL duplicates merge metadata without losing source provenance.
- Balance rules with duplicate categories warn or reject deterministically.
- Enrichment cannot run before balancing if the profile requires balancing
  first.
- Digest citations point to source cards, not generated summaries.
- Delivery retry uses idempotency and does not send duplicates.
- Cursor does not advance if source-card writes fail.
- Prompt injection in source text is rendered as quoted content only.

## First Slice

The first slice is now partially implemented through existing Arcwell tables
rather than a separate `radar_stage_artifacts` table: `arcwell radar stage`
and `radar_stage_read` expose the run, items, scores, and dedupe groups, with
source-card/wiki links preserved. Remaining first-slice work is to add any
missing explicit skipped/error stage records only where the current run counts,
metadata, source-health rows, and audit findings are too indirect.
