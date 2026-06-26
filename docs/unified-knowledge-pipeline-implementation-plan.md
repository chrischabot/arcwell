# Unified Knowledge Pipeline Implementation Plan

Date: 2026-06-25

Status: design plus implemented foreground bridge slices. This document does
not claim that the unified cross-source pipeline is operational. The shared
knowledge substrate and a live source-card/radar projection bridge are now
implemented and tested; bounded scheduled recurrence exists for selected
knowledge paths, while multi-day service recurrence, live external delivery,
broad production-data semantic clustering, and broad provider coverage remain
open.

Skill gate: `arc:anti-mirage`.

## Product Claim

Arcwell should continuously watch configured sources, detect important events
and trend clusters across those sources, investigate promising but incomplete
signals, write rich source-backed wiki pages, and send authorized reports when
something genuinely interesting happens.

Concrete examples this system must support:

- OpenAI publishes a new GitHub package, tweets about it, and other developers
  discuss its implications.
- Andrej Karpathy shares a useful way of using Claude in Slack.
- Simon Willison publishes a new benchmark that replaces or improves on an
  earlier SVG benchmark.
- NVIDIA releases a new open source model with repo, model card, license,
  benchmark, and community reaction evidence.
- Vercel announces Eve or an agent SDK for simplifying agent workflows, and
  the system links announcement, docs, repo, X discussion, and competitive
  context.

The user-facing result is not a log dump. It is a concise but rich report:

- what happened
- why it matters
- who is involved
- what evidence supports it
- what others are saying
- how it connects to prior wiki knowledge
- competitive and historical context
- uncertainties and follow-up questions
- source-card citations and wiki links
- delivery and review state

## Anti-Mirage Boundary

This plan is intentionally strict because the fake-done risk is high.

Arcwell may not claim this system is operational because any one of these exists:

- a source adapter writes rows
- a radar profile selects items
- a model writes a paragraph
- a wiki page is created
- a digest candidate exists
- an email is sent once
- an ops table shows a count
- an MCP tool schema is exposed
- a prompt says "writer" or "editor"

The capability is only operational when real configured sources are scheduled,
fresh data is ingested, events are clustered across source families, research
fanout closes evidence gaps, wiki pages pass quality gates, delivery policy
routes reports without manual intervention, and ops can explain healthy,
stale, blocked, failed, retrying, and sent states.

## Proof Levels

- `Missing`: no code or only notes.
- `Scaffold`: schemas, commands, prompts, or docs exist.
- `Local Proof`: deterministic fixtures and severe tests pass.
- `Production Data Proof`: real source/provider data in a disposable or copied
  Arcwell home passes the named proof packet.
- `Operational`: production-data proof plus recurring scheduling, retries,
  source health, recovery, delivery ledgers, ops visibility, and status docs.
- `Done`: operational, maintained, and no known core proof gate remains.

Mock-only or generated-only proof can never satisfy a freshness, live-provider,
wiki-quality, or delivery claim.

## Current State Summary

Already present ingredients:

- `watch_sources` with source kinds such as RSS/blog, GitHub owner, arXiv,
  Hacker News, Reddit, X handles, and X bookmarks.
- Worker jobs with leases, retries, backoff, and dead-letter behavior.
- Source cards, wiki pages, FTS, provenance metadata, and untrusted evidence
  rendering.
- Radar profiles/runs/items/scores/dedupe/balance/audit/summaries.
- X-specific canonical storage, bookmark/watch-source import, X clusters,
  editorial decisions, wiki quality gate, digest candidate creation, and ops
  visibility.
- Research convergence, host-search proof, source-card linking, evidence packs,
  editorial invocation, report judgments, and active fact-checking.
- Digest candidates, approval/rejection, email/Telegram delivery ledgers,
  quiet-hours scheduling, retry/dead-letter reconciliation, and ops visibility.
- Policy, cost, secret redaction, source health, cursors, and strict doctor
  checks.

Missing or partial:

- Source-agnostic knowledge events and clusters.
- Shared adapter contract across all watched source families.
- General editorial decision loop beyond X.
- Automatic research fanout from clusters.
- General wiki writer/editor quality gate for cross-source pages.
- Unified "interestingness" and alert routing across all sources.
- Multi-day/live-external wall-clock recurrence proof for real alerts.
- Live X freshness now has capped copied-home proof; broad quota/tier and
  long-running recurrence remain open.
- Ops controls are still narrow.

Current implemented bridge slice:

- Durable shared tables for `knowledge_events`, `knowledge_event_sources`,
  `knowledge_clusters`, `knowledge_editorial_decisions`, and
  `knowledge_reports`.
- Durable shared tables for first-pass `knowledge_entities` and
  `knowledge_relations`.
- CLI projection from existing source cards or scored radar runs:
  `arcwell knowledge project-source-card-query` and
  `arcwell knowledge project-radar-run`.
- CLI listing for knowledge events, clusters, reports, entities, and relations.
- Deterministic source-card-to-event mapping with source-card evidence rows,
  canonical keys for common providers, source roles, duplicate grouping,
  provider-native timestamp normalization, and report quality gates.
- Deterministic source-card-backed entity/relation mapping for providers,
  source items, GitHub owners/repos, provider reporting links, GitHub ownership
  links, and cluster co-occurrence links.
- Alias collision checks that fail closed instead of silently merging unrelated
  canonical entities.
- Deterministic semantic entity-resolution proposals plus model-invoked
  entity-resolution suggestions through
  `arcwell knowledge resolve-entity-model`, with policy/cost gates,
  schema-validated output, source-card citation requirements, prompt-injection
  reason rejection, and pending-review-only writes.
- Schema-gated semantic/model cluster proposals through
  `arcwell knowledge propose-clusters`, with policy/cost gates, source-card
  citation requirements, duplicate-source-card rejection across proposed
  clusters, prompt-injection topic/reason rejection, confirmed event backing,
  and candidate-only cluster writes.
- Scheduled model-cluster proposal jobs through
  `arcwell knowledge enqueue-model-clusters` and
  `arcwell knowledge schedule-model-clusters`. The `knowledge_model_clusters`
  watch source searches source cards for a configured query, invokes the same
  schema-gated model proposal path behind policy/cost gates only when evidence
  exists, writes review-only candidate clusters, records source-health state,
  and still refuses wiki/report/digest expansion until explicit promotion.
  Fresh proof
  `.arcwell-dev/proofs/knowledge-model-cluster-scheduled-proof-20260626T090342Z-10737/artifacts/proof-packet.json`
  copied the real local source-card home, ran a 40-tick resident worker loop,
  invoked live OpenAI `gpt-4.1-mini`, wrote six review-only candidate clusters
  from 24 source cards, recorded source health/cost/policy evidence, created no
  report/wiki/digest/expansion side effects, and browser-checked `/ops/ui`.
- Broad model-cluster scheduling can use `source-cards` or `*` as the query to
  sweep the local source-card corpus instead of a narrow text search. The
  worker canonicalizes that scope to `source-cards`, skips already-clustered
  source cards and generated-only evidence before invoking a provider, records
  skip counts in the job result, and remains review-only; local severe tests
  prove replay does not reuse clustered source-card evidence.
- Policy-gated promotion of model-origin cluster proposals through
  `arcwell knowledge promote-cluster`. Unpromoted model-origin candidates are
  refused by foreground expansion, direct expansion enqueue, and due expansion
  recurrence; promotion records a durable `promote_model_cluster` editorial
  decision, flips the cluster to `active`, and still leaves digest delivery
  behind separate review, policy, channel, quiet-hours, idempotency, and retry
  gates.
- Authenticated `/ops/ui` Knowledge Controls can promote a reviewed
  model-origin cluster by id. The HTTP action is CSRF/idempotency protected and
  double-gated: `ops.knowledge_clusters.promote` authorizes the operator
  action, while core `knowledge_cluster.promote` still authorizes activating the
  cluster.
- Authenticated `/ops/ui` Knowledge Controls can queue due shared cluster
  editorial-decision jobs through
  `ops.knowledge_clusters.enqueue_editorial_decisions`. The visible operator
  path runs the writer/editor decision loop before any wiki/report/digest
  expansion; direct expansion enqueue remains a lower-level repair path, not
  the primary UI control.
- Authenticated `/ops/ui` Knowledge Controls can schedule or enqueue
  review-only model-cluster proposals and promoted-cluster model-writer jobs.
  These HTTP actions are CSRF/idempotency protected and policy-gated through
  `ops.knowledge_model_clusters.schedule`,
  `ops.knowledge_model_clusters.enqueue`,
  `ops.knowledge_model_write.schedule`, and
  `ops.knowledge_model_write.enqueue`; they reuse the existing worker paths and
  do not promote model-cluster quality, wiki writes, or delivery readiness.
- `arcwell knowledge enqueue-due-model-writes` and the authenticated
  `/ops/ui` due-writer control can bulk-enqueue model-writer jobs for active
  promoted model-origin clusters. The path skips unpromoted model proposals,
  deterministic/shared clusters, active writer/editorial/expansion jobs, and
  terminal writer/expansion decisions; external digest delivery remains behind
  separate delivery policy and recipient gates.
- Resident `worker run-once` now invokes the same due promoted model-writer
  sweep before shared editorial/expansion recurrence, so promoted model-origin
  clusters can advance without an operator pre-creating a writer job or
  cluster-scoped watch source. Local severe tests prove idempotency and no
  external delivery; live/broad production writer quality still needs a separate
  proof.
- Cluster evidence revision gating now keeps terminal editorial, expansion,
  investigation, and model-writer decisions tied to the exact source-card set
  they evaluated. `add_source_cards_to_knowledge_cluster` merges fresh
  source-card evidence into an existing cluster, updates event evidence and
  cluster scores, stores a source-card-set fingerprint, and due recurrence
  reopens when that fingerprint changes. Severe tests prove fresh evidence
  reopens shared editorial recurrence and promoted model-writer recurrence
  without external delivery. Reopened expansion/model-writer runs now also
  supersede older undelivered digest candidates referenced by stale cluster
  decisions, refresh wiki/report/digest artifacts with the fresh source-card
  citations, and make the stale candidates fail the delivery gate while
  preserving already pending/sent delivery ledger rows. This is local
  stale-evidence and stale-digest protection, not versioned decision history,
  broad semantic merge quality, or live wall-clock recurrence.
- Source-card-gated model cluster writing through
  `arcwell knowledge write-cluster-model`,
  `arcwell knowledge enqueue-cluster-model-write`, and the resident
  `knowledge_cluster_model_write` job. The writer path requires prior model
  cluster promotion, provider policy, cost budget, exact source-card citations,
  uncertainty language, a cluster link, and the existing wiki/report quality
  audit before it can write a wiki page, report, or digest candidate; malformed,
  uncited, delivery-authorizing, and provider-policy-denied outputs fail closed.
  Fresh proof
  `.arcwell-dev/proofs/knowledge-cluster-model-writer-proof-20260626T092354Z-43488/artifacts/proof-packet.json`
  seeded proof source cards, created a review-only model-origin cluster, proved
  pre-promotion denial, promoted cluster `kcl-d05b33585b8fa1ab`, invoked live
  OpenAI `gpt-4.1-mini`, wrote model-backed wiki page
  `knowledge-agent-tooling-and-mcp-infrastructure-model-draft-224af13a`, report
  `krpt-03e5a617cf07c686`, digest candidate
  `ba7b0fe4-d28c-43ab-96f6-a3f3d8ea2e00`, delivered nothing externally, and
  browser-checked `/ops/ui`.
- Explicit cluster-scoped scheduled model writing through
  `arcwell knowledge schedule-cluster-model-write` and the
  `knowledge_model_write` watch-source kind. Local severe tests prove
  unpromoted model-origin clusters cannot be scheduled, a due watch source
  enqueues exactly one `knowledge_cluster_model_write` job only after promotion
  and worker-enqueue policy, deterministic expansion is suppressed while that
  writer job is active and after terminal model-writer decisions, source health
  advances only after durable writer output, terminal writer decisions suppress
  recurrence, and active or provider-policy-denied writer jobs do not create
  retry storms. Fresh scheduled proof
  `.arcwell-dev/proofs/knowledge-model-writer-scheduled-proof-20260626T095734Z-12775/artifacts/proof-packet.json`
  ran a bounded 50-tick resident worker over a proof-scoped promoted cluster,
  detected the due watch source, completed live OpenAI `gpt-4.1-mini` writer
  job `cbec70a7-5e54-4f76-b682-7197f1794c56`, recorded cost decision
  `eafc92b1-56ca-4fb7-932b-013897bb5faf`, wrote wiki page
  `knowledge-agent-tooling-and-mcp-infrastructure-model-draft-782645b5`,
  report `krpt-66297b3e919fa209`, digest candidate
  `98d9a313-408b-428b-a133-1c20a812cad5`, advanced source health only after
  durable output, completed one local investigation-execution follow-through
  job without enqueue-deferral churn, created no deterministic expansion job or
  duplicate active writer job, delivered nothing externally, and
  browser-checked authenticated desktop/mobile `/ops/ui`. This is not a broad
  autonomous model-writing sweep.
- `/ops` and `/ops/ui` visibility for knowledge events, clusters, editorial
  decisions, reports, entities, relations, adapter runs, and entity-resolution
  proposals.
- Preserved production-data foreground proof:
  `.arcwell-dev/proofs/knowledge-live-e2e-proof-20260625T173937Z-37414/artifacts/proof-packet.json`.
- Preserved bounded resident recurrence proof:
  `.arcwell-dev/proofs/knowledge-wall-clock-recurrence-proof-20260626T081914Z-31980/proof-packet.json`.

What the bridge proof showed:

- Live public RSS, GitHub owner, arXiv, and Hacker News adapters completed.
- A scored radar run projected 12 source cards into 12 confirmed knowledge
  events.
- The projection wrote 9 source-backed entities, 19 source-backed relations,
  and 4 shared adapter-run contract rows.
- One source-backed cluster, one completed editorial decision, and one
  human-readable report were written durably.
- First-pass source-backed entities and relations are now part of the
  projection output and live proof harness assertions.
- `scripts/knowledge-entity-resolution-production-proof` invoked the
  entity-resolution model path with both deterministic mock provider and live
  OpenAI `gpt-4.1-mini`; the live proof wrote a cost decision, durable
  `pending_review` resolution, source-card-backed evidence boundary, zero graph
  relations, and authenticated desktop/mobile `/ops/ui` screenshots at
  `.arcwell-dev/proofs/knowledge-entity-resolution-production-proof-20260625T181411Z-84883/artifacts/proof-packet.json`.
- `scripts/knowledge-cluster-proposal-production-proof` invoked model-backed
  cluster proposals with both deterministic mock provider and live OpenAI
  `gpt-4.1-mini`; the live proof
  `.arcwell-dev/proofs/knowledge-cluster-proposal-production-proof-20260626T083551Z-61527/artifacts/proof-packet.json`
  recorded cost decision `d055f61f-a4b4-49e3-aafb-e3e7e31b9b8b`,
  wrote three candidate clusters, confirmed event/source evidence, proved no
  report/wiki/digest side effects at proposal time, denied pre-promotion
  expansion, promoted `kcl-bf3b8decc24a5fe3` through explicit
  `knowledge_cluster.promote` policy decision
  `8e19aae8-9952-40b3-ba40-e72f2db98335`, expanded it into wiki page
  `knowledge-nvidia-open-source-model-release-with-benchmarks-and-evaluations-d92102a7`,
  report `krpt-9ec31fc4ce0bd58b`, digest candidate
  `7f59d663-50e5-4fe5-bebe-4852d4e47f8f`, and browser-checked ops visibility.
- Cursors and ops state were visible after durable writes.
- Authenticated `/ops/ui` rendered desktop and mobile knowledge tables through
  browser automation without horizontal overflow.

What it still does not prove:

- Multi-day launchd/systemd recurrence or live external inbox delivery over
  elapsed time. The bounded copied-home wall-clock proof runs one resident
  worker process for 80 ticks over 21 seconds with a controlled local email
  provider.
- Broad live X freshness beyond the capped copied-home smoke.
- Model-invoked entity resolution over broad production clusters or scheduled
  recurrence; the live proof is a foreground provider attempt over proof
  fixture data.
- Broad production-data semantic/model clustering quality. Local severe tests
  and the copied-home live-provider scheduled proof now prove the scheduled
  worker candidate-only path, while the promotion/expansion proof is still a
  foreground provider attempt over proof fixture data with one policy-gated
  promoted expansion.
- Broad production-corpus model-backed writer/editor synthesis and broad
  automatic model-writing quality. The resident due sweep is locally proven and
  the live foreground/scheduled writer proofs are source-card-gated and accepted
  over proof-scoped promoted clusters, not broad autonomous production analyst
  quality.
- Autonomous approval, broad wiki page update decisions, and live external
  delivery from shared knowledge reports.
- Broad ops repair controls.

## Unified Pipeline

```text
watch source
  -> source adapter fetch
  -> source card projection
  -> entity/event extraction
  -> knowledge event upsert
  -> cross-source cluster update
  -> editorial decision
  -> optional research fanout
  -> wiki write/update with quality gate
  -> digest candidate
  -> delivery policy/router
  -> email/Telegram/ops alert
```

Every stage writes durable rows before the next stage can claim progress.

## Core Concepts

### Source Card

A source card remains the durable evidence primitive. It stores what an external
source said, where it came from, how it was retrieved, and why it is trusted or
untrusted.

Source text is always evidence, never instructions.

### Knowledge Event

A knowledge event is the normalized "thing happened" object.

Examples:

- `github_package_release`
- `model_release`
- `benchmark_release`
- `workflow_pattern`
- `agent_sdk_launch`
- `repo_created`
- `docs_changed`
- `paper_published`
- `community_reaction`
- `company_announcement`
- `security_incident`
- `pricing_or_policy_change`

Events are source-backed, not model-invented.

### Knowledge Cluster

A cluster groups related events and sources into one user-meaningful topic.

Examples:

- OpenAI package launch cluster:
  - GitHub repo or release
  - package registry entry
  - OpenAI X post
  - docs or blog post
  - HN/Reddit/X reactions
  - related prior wiki pages about OpenAI agents/tools

- NVIDIA model release cluster:
  - NVIDIA blog
  - Hugging Face model card
  - GitHub repo
  - paper/arXiv
  - license details
  - benchmark claims
  - community reaction
  - comparison to existing open models

### Editorial Decision

Each cluster gets a durable decision:

- `ignore`
- `monitor`
- `digest_only`
- `update_existing_page`
- `create_new_page`
- `deepen_research`
- `block_for_review`

The decision includes reason, evidence, uncertainty, and the next job to run.

### Investigation

Investigation is bounded research fanout:

- fetch canonical docs
- inspect GitHub README/releases/tags/issues
- expand links from source cards
- search related announcements
- collect reactions
- compare to wiki history
- identify competitors
- identify missing primary evidence

No investigation output is accepted unless it links back to source cards or
recorded host/provider search proof.

## Proposed Data Model

Use additive SQLite migrations. Keep X-specific tables and radar tables intact
while adding shared knowledge tables. Migrate X onto shared tables after the
shared path is locally proven.

### `knowledge_entities`

Purpose: durable people, companies, repos, packages, products, models,
benchmarks, papers, protocols, and communities.

Fields:

- `id TEXT PRIMARY KEY`
- `entity_type TEXT NOT NULL`
- `name TEXT NOT NULL`
- `canonical_key TEXT NOT NULL UNIQUE`
- `aliases_json TEXT NOT NULL DEFAULT '[]'`
- `homepage_url TEXT`
- `source_card_ids_json TEXT NOT NULL DEFAULT '[]'`
- `wiki_page_id TEXT`
- `confidence REAL NOT NULL DEFAULT 0.0`
- `metadata_json TEXT NOT NULL DEFAULT '{}'`
- `created_at TEXT NOT NULL`
- `updated_at TEXT NOT NULL`

Entity types:

- `person`
- `company`
- `github_repo`
- `github_owner`
- `package`
- `model`
- `benchmark`
- `product`
- `paper`
- `protocol`
- `community`
- `topic`

Invariants:

- Alias merge is reviewable.
- Cross-entity merge requires evidence.
- Source text cannot create privileged identity by itself.

### `knowledge_events`

Purpose: normalized event rows that can be clustered, investigated, and
reported.

Fields:

- `id TEXT PRIMARY KEY`
- `event_type TEXT NOT NULL`
- `title TEXT NOT NULL`
- `canonical_key TEXT NOT NULL`
- `status TEXT NOT NULL`
- `primary_entity_id TEXT`
- `event_time TEXT`
- `first_seen_at TEXT NOT NULL`
- `last_seen_at TEXT NOT NULL`
- `novelty_score REAL NOT NULL DEFAULT 0.0`
- `importance_score REAL NOT NULL DEFAULT 0.0`
- `momentum_score REAL NOT NULL DEFAULT 0.0`
- `uncertainty_score REAL NOT NULL DEFAULT 1.0`
- `source_diversity_score REAL NOT NULL DEFAULT 0.0`
- `summary TEXT NOT NULL DEFAULT ''`
- `metadata_json TEXT NOT NULL DEFAULT '{}'`
- `created_at TEXT NOT NULL`
- `updated_at TEXT NOT NULL`
- `UNIQUE(event_type, canonical_key)`

Statuses:

- `candidate`
- `confirmed`
- `needs_more_evidence`
- `stale`
- `duplicate`
- `rejected`

Invariants:

- A confirmed event must cite at least one source-card id.
- Current/fresh claims require source-health and cursor evidence.
- Model output can propose an event but cannot confirm it without evidence.

### `knowledge_event_sources`

Purpose: source-card evidence linked to events.

Fields:

- `event_id TEXT NOT NULL`
- `source_card_id TEXT NOT NULL`
- `role TEXT NOT NULL`
- `confidence REAL NOT NULL`
- `claim_summary TEXT NOT NULL DEFAULT ''`
- `metadata_json TEXT NOT NULL DEFAULT '{}'`
- `created_at TEXT NOT NULL`
- `PRIMARY KEY(event_id, source_card_id, role)`

Roles:

- `primary_announcement`
- `repo_or_package`
- `documentation`
- `model_card`
- `paper`
- `benchmark`
- `reaction`
- `critique`
- `historical_context`
- `competitor_context`
- `correction`

Invariants:

- Primary claims require primary or first-party evidence when available.
- Reactions cannot be promoted to announcement evidence.
- Unsupported reaction summaries stay uncertainty, not fact.

### `knowledge_clusters`

Purpose: shared trend clusters across sources.

Fields:

- `id TEXT PRIMARY KEY`
- `topic TEXT NOT NULL`
- `status TEXT NOT NULL`
- `event_ids_json TEXT NOT NULL DEFAULT '[]'`
- `source_card_ids_json TEXT NOT NULL DEFAULT '[]'`
- `entity_ids_json TEXT NOT NULL DEFAULT '[]'`
- `first_seen_at TEXT NOT NULL`
- `last_seen_at TEXT NOT NULL`
- `novelty_score REAL NOT NULL`
- `momentum_score REAL NOT NULL`
- `importance_score REAL NOT NULL`
- `source_diversity_score REAL NOT NULL`
- `stale_score REAL NOT NULL`
- `confidence REAL NOT NULL`
- `reason TEXT NOT NULL`
- `metadata_json TEXT NOT NULL DEFAULT '{}'`
- `created_at TEXT NOT NULL`
- `updated_at TEXT NOT NULL`

Statuses:

- `candidate`
- `monitoring`
- `needs_research`
- `ready_for_editorial`
- `wiki_written`
- `digest_candidate_created`
- `sent`
- `ignored`
- `blocked`
- `stale`

Invariants:

- Clusters preserve the deduped source-card evidence set.
- Duplicate occurrence signal is recorded in `duplicate_groups_json`, not by
  repeating IDs in `source_card_ids_json`.
- A cluster cannot be sent on model score alone.

### `knowledge_relations`

Purpose: graph edges between events, entities, source cards, wiki pages, and
clusters.

Target fields for the complete operational version:

- `id TEXT PRIMARY KEY`
- `subject_type TEXT NOT NULL`
- `subject_id TEXT NOT NULL`
- `relation_type TEXT NOT NULL`
- `object_type TEXT NOT NULL`
- `object_id TEXT NOT NULL`
- `source_card_ids_json TEXT NOT NULL DEFAULT '[]'`
- `confidence REAL NOT NULL`
- `reason TEXT NOT NULL`
- `created_at TEXT NOT NULL`

Relation types:

- `announced_by`
- `implemented_in`
- `discussed_by`
- `competes_with`
- `replaces`
- `extends`
- `similar_to`
- `uses`
- `benchmarks_against`
- `historical_precedent`
- `community_reaction_to`

### `knowledge_editorial_decisions`

Purpose: durable editorial loop for every cluster.

Fields:

- `id TEXT PRIMARY KEY`
- `cluster_id TEXT NOT NULL`
- `decision TEXT NOT NULL`
- `status TEXT NOT NULL`
- `reason TEXT NOT NULL`
- `source_card_ids_json TEXT NOT NULL DEFAULT '[]'`
- `wiki_page_id TEXT`
- `digest_candidate_id TEXT`
- `research_run_id TEXT`
- `reviewer TEXT NOT NULL DEFAULT 'arcwell-knowledge-editor'`
- `metadata_json TEXT NOT NULL DEFAULT '{}'`
- `created_at TEXT NOT NULL`
- `updated_at TEXT NOT NULL`

Current local-proof slice implements a smaller subset: `id`, `cluster_id`,
`title`, `body_markdown`, `status`, `source_card_ids_json`,
`quality_findings_json`, `metadata_json`, `created_at`, and `updated_at`.
`wiki_page_id`, `digest_candidate_id`, `report_kind`, `summary`, and richer
quality-gate payloads remain part of the operational follow-up.

Invariants:

- Every decision must include evidence ids and a human-readable reason.
- `create_new_page` and `update_existing_page` require quality gates.
- `digest_only` still requires human-readable report body, not link dumps.

### `knowledge_investigation_jobs`

Purpose: bounded follow-up tasks spawned from clusters.

Fields:

- `id TEXT PRIMARY KEY`
- `cluster_id TEXT NOT NULL`
- `job_kind TEXT NOT NULL`
- `status TEXT NOT NULL`
- `input_json TEXT NOT NULL`
- `result_json TEXT`
- `error TEXT`
- `cost_decision_id TEXT`
- `created_at TEXT NOT NULL`
- `updated_at TEXT NOT NULL`

Job kinds:

- `expand_links`
- `fetch_github_repo_context`
- `fetch_github_owner_context`
- `fetch_package_metadata`
- `fetch_model_card`
- `search_related_sources`
- `collect_reactions`
- `compare_wiki_history`
- `compile_competitive_context`
- `run_research_convergence`

### `knowledge_reports`

Purpose: final human-facing generated report artifacts.

Fields:

- `id TEXT PRIMARY KEY`
- `cluster_id TEXT NOT NULL`
- `wiki_page_id TEXT`
- `digest_candidate_id TEXT`
- `report_kind TEXT NOT NULL`
- `status TEXT NOT NULL`
- `title TEXT NOT NULL`
- `summary TEXT NOT NULL`
- `body_markdown TEXT NOT NULL`
- `source_card_ids_json TEXT NOT NULL`
- `quality_gate_json TEXT NOT NULL`
- `metadata_json TEXT NOT NULL DEFAULT '{}'`
- `created_at TEXT NOT NULL`
- `updated_at TEXT NOT NULL`

Report kinds:

- `instant_alert`
- `daily_digest`
- `wiki_article`
- `competitive_brief`
- `research_brief`

Invariants:

- Body must be human-readable.
- Body must cite source-card ids or wiki/source links.
- Empty/generic pages fail.
- Generated prose never becomes source evidence.

## Shared Source Adapter Contract

Every source family should implement one contract, even if the provider API is
different.

Required functions:

- `validate_source(locator, metadata)`
- `estimate_cost(locator, limits)`
- `fetch_due(cursor, limits)`
- `normalize(records) -> SourceCardInput[]`
- `extract_event_candidates(source_cards)`
- `write_transactionally(source_cards, candidates)`
- `advance_cursor_after_commit(cursor)`
- `record_source_success/failure`
- `classify_provider_error(error)`
- `build_ops_summary`

Required behavior:

- Reject unsafe locators before network.
- Policy and cost checks run before provider calls.
- Secret values never enter output artifacts.
- Cursor advances only after accepted writes are durable.
- Partial writes record drift and repair state.
- Duplicate source cards are deduped by canonical identity.
- Prompt injection text stays evidence.

Source families:

- X bookmarks
- X handles/watch list
- X recent search
- GitHub repo releases/commits/issues
- GitHub org/person repo list
- RSS/Atom feeds
- Blogs and explicit URLs
- Hacker News
- Reddit
- arXiv
- selected email events
- selected Telegram/channel events
- future package registries such as npm, PyPI, crates.io, Hugging Face

## Event Extraction

Extraction should start deterministic and become model-assisted only after
schema validation exists.

Deterministic event cues:

- GitHub release tag, new repo, repo description change, README change.
- Package registry new version.
- RSS/blog title contains release/launch/announcing/benchmark/model/open-source.
- X text contains released, launched, open-sourced, benchmark, model, SDK.
- HN/Reddit post URL matches known repo/docs/blog/model card.
- Multiple source cards mention same repo/model/product within a time window.

Model-assisted extraction can add:

- event type proposal
- concise event title
- relationship candidates
- novelty explanation
- uncertainty labels
- missing evidence questions

Model extraction may not:

- confirm facts without source cards
- create delivery authorization
- invent entities not grounded in evidence
- rewrite source text as instructions

## Clustering Design

Clusters should merge evidence by canonical identity first, then semantic
similarity.

Deterministic keys:

- GitHub repo: `github:<owner>/<repo>`
- GitHub release: `github_release:<owner>/<repo>@<tag>`
- Package: `package:<ecosystem>:<name>@<version>`
- Model: `model:<provider-or-org>:<model-id>`
- Paper: `paper:<doi-or-arxiv-id>`
- X post: `x:<tweet-id>`
- URL canonical host/path

Semantic cluster candidates:

- shared canonical URL
- shared repo/package/model/entity
- title similarity
- same source-card outbound links
- same entity plus time window
- reaction source linking to primary announcement

Cluster scoring:

- novelty: not already covered in wiki
- momentum: source count, source velocity, reaction growth
- importance: entity reputation, domain weight, launch/release markers
- source diversity: first-party plus third-party plus community
- uncertainty: missing primary source, conflicting claims, stale source
- relevance: user interest profile, project links, prior wiki topics

Cluster outputs:

- event ids
- source-card ids
- duplicate groups
- timeline
- source-family coverage
- missing evidence checklist
- recommended editorial action

## Editorial Decision Policy

Decision matrix:

| Condition | Decision |
| --- | --- |
| Low relevance, duplicate, no novelty | `ignore` |
| Interesting but one weak source | `monitor` |
| Interesting, enough for alert, no durable topic | `digest_only` |
| Updates known topic/company/product | `update_existing_page` |
| New durable concept/event | `create_new_page` |
| Important but evidence thin or conflicting | `deepen_research` |
| Prompt injection, unsupported claims, unsafe source | `block_for_review` |

Every decision writes:

- reason in human language
- evidence ids
- uncertainty
- next action
- whether delivery is allowed
- whether wiki write is allowed
- whether research fanout is needed

## Research Fanout

Research fanout should be bounded, source-card backed, and policy/cost gated.

Fanout templates:

### GitHub package or repo launch

- fetch README
- fetch releases/tags
- fetch package metadata if package registry is known
- fetch docs/blog if linked
- collect HN/Reddit/X reactions
- compare to prior wiki pages for same org/repo/domain
- generate competitor context

### Model release

- fetch model card
- fetch repo
- fetch paper/arXiv if present
- extract license and usage constraints
- extract benchmark claims and uncertainty
- compare to prior model families and competitors

### Benchmark release

- fetch benchmark repo/docs
- identify what it replaces or critiques
- identify methodology and task
- compare to prior benchmarks in wiki
- collect expert reaction

### Workflow or practice pattern

- preserve social source as anecdotal unless backed by fuller writeup
- link to prior workflow pages
- identify repeatable method
- collect caveats and privacy/security concerns

### Agent SDK or product launch

- fetch docs
- fetch repo/package if public
- inspect examples
- compare to OpenAI Agents SDK, Anthropic MCP/Claude tooling, LangChain,
  Vercel AI SDK, Mastra, CrewAI, and existing wiki competitors where relevant
- collect notable reactions

Fanout gates:

- Stop when enough evidence exists for the chosen report kind.
- Do not expand unsafe URLs.
- Do not fetch private/logged-in pages from the daemon.
- Do not spend model/provider budget without policy and cost approval.
- Record unresolved questions instead of inventing answers.

## Wiki Writer Design

Wiki pages must be useful to a human and useful to future Arcwell runs.

Required sections for event pages:

- `Summary`
- `What Happened`
- `Why It Matters`
- `Evidence`
- `Timeline`
- `Technical Details`
- `Reactions`
- `Related Arcwell Knowledge`
- `Competitive Context`
- `Uncertainty And Open Questions`
- `What To Watch Next`
- `Sources`

Required sections for durable topic pages:

- `Overview`
- `Current State`
- `Key Players`
- `Concepts And Terms`
- `Notable Events`
- `Comparisons`
- `Open Questions`
- `Source Notes`
- `Related Pages`

Quality gates:

- Must cite source-card ids.
- Must link back to cluster id.
- Must distinguish first-party evidence from reactions.
- Must name uncertainty.
- Must not cite generated summaries as evidence.
- Must not contain raw prompt-injection instructions as instructions.
- Must not duplicate an existing page unless decision says fork.
- Must include enough plain-English explanation to be useful without clicking
  every source.

Reject output if:

- body is empty or mostly metadata
- body is mostly links
- no source-card ids appear
- unsupported claims are detected
- source text instructions leak into author voice
- page repeats an existing page without new information
- model output has schema failure

## Digest And Alert Design

Digest candidates should be created from reports, not raw source lists.

Candidate payload:

- headline
- short verdict
- why now
- what happened
- evidence summary
- source diversity
- uncertainty
- wiki link
- cluster id
- source-card ids
- delivery policy context

Delivery modes:

- immediate alert for high-confidence, high-importance clusters
- daily digest for medium-confidence clusters
- weekly rollup for slow-moving watch topics
- wiki-only for archival or low-priority information

Routing policy:

- recipient must be authorized
- channel must be authorized
- quiet hours enforced
- dedupe window enforced by cluster/report/recipient
- idempotency key required
- delivery attempt ledger required
- provider failure records retry/dead-letter state
- no model-score-only sends

Example alert should read like:

```text
OpenAI appears to have launched a new GitHub package for agent workflows.

What happened: OpenAI published <repo/package>, then amplified it on X. The
GitHub release and docs describe <functionality>. Developers including <names>
are discussing <reaction themes>.

Why it matters: This overlaps with <prior wiki topic> and competes with
<tools>. It may change how we think about <workflow>.

Confidence: high for the release, medium for the community interpretation.

Open questions: package maturity, license implications, whether this replaces
or complements <existing tool>.

Wiki: <page>
Sources: <source-card ids>
```

## Automation And Scheduling

Use declarative schedules stored in SQLite and executed by the resident worker.

Suggested default cadences:

| Source family | Default cadence | Notes |
| --- | ---: | --- |
| X bookmarks | 1 to 6 hours | provider quota and credentials permitting |
| X watch handles | 15 to 60 minutes | high-value handles only |
| GitHub watched repos | 30 to 120 minutes | releases/tags first, issues optional |
| GitHub org/person repo list | daily | detect new repos |
| RSS/Atom/blogs | 30 minutes to 6 hours | depends on feed velocity |
| Hacker News | 30 to 60 minutes | frontpage/newest/topstories profiles |
| Reddit | 1 to 6 hours | OAuth or sanctioned capture path required |
| arXiv | daily | category/query windows |
| email/Telegram selected channels | poll/drain interval | only authorized sources |
| cluster consolidation | after ingest and hourly | merge cross-source evidence |
| editorial decision | after cluster update | idempotent per cluster revision |
| research fanout | bounded async | cost/policy gated |
| wiki writer | after editorial/research | quality gated |
| digest router | immediate plus scheduled | quiet hours and dedupe |

Worker queues:

- `source_poll`
- `source_normalize`
- `event_extract`
- `cluster_update`
- `editorial_decide`
- `investigate_cluster`
- `wiki_write`
- `digest_route`
- `delivery_send`
- `knowledge_maintenance`

Each job needs:

- input schema
- idempotency key
- max attempts
- retry/backoff
- dead-letter reason
- policy/cost gate where relevant
- ops-visible result

## Ops And Controls

Add a `Knowledge` section to `/ops/ui` and `/ops`:

- latest successful source poll by family
- stale source count
- failed source count
- source cards created in last 24h
- events created/updated
- clusters by status
- editorial decisions by action/status
- investigation jobs pending/failed/completed
- wiki pages created/updated
- digest candidates pending/approved/sent
- last successful external alert
- provider credential health
- live-provider blockers

Safe controls:

- pause/resume watch source
- retry failed source poll
- re-run cluster consolidation
- approve/reject editorial decision
- enqueue investigation
- re-run wiki quality gate
- create digest candidate from report
- dead-letter stuck job

Control requirements:

- explicit auth
- local-origin protection
- CSRF token
- policy check
- idempotency key
- audit row
- no broad arbitrary SQL or command execution

## Implementation Milestones

### Milestone 0: Plan And Claim Alignment

- [x] Add this plan.
- [x] Link it from Horizon/radar docs.
- [x] Add TODO entries for unified knowledge pipeline work.
- [x] Ensure `STATUS.md` says current capability is partial, not operational.
- [x] Run docs/plugin verifier if any command/skill/docs references change.

Done when:

- The repo has one canonical cross-source knowledge-system plan.
- Status language distinguishes X local proof from full unified operation.

### Milestone 1: Shared Knowledge Schema

- [x] Add migrations for `knowledge_entities`.
- [x] Add migrations for `knowledge_events`.
- [x] Add migrations for `knowledge_event_sources`.
- [x] Add migrations for `knowledge_clusters`.
- [x] Add migrations for `knowledge_relations`.
- [x] Add migrations for `knowledge_editorial_decisions`.
- [ ] Add migrations for `knowledge_investigation_jobs`.
- [x] Add migrations for initial `knowledge_reports` local-proof subset.
- [x] Add typed Rust structs and row mappers for the local-proof event, source,
      cluster, editorial decision, and report substrate.
- [x] Add typed Rust structs and row mappers for first-pass entities and
      relations.
- [x] Add CRUD/list/read APIs for the local-proof substrate.
- [ ] Add search APIs.
- [x] Add ops snapshot fields.

Refuting tests:

- [x] Duplicate canonical event upserts update instead of duplicate.
- [x] Event cannot be confirmed without source-card evidence.
- [x] Malicious source text is stored as data.
- [x] Entity alias collision requires review.
- [x] Cluster preserves all source-card ids and duplicate groups.
- [x] Missing source cards block cluster/report/decision writes.
- [x] Link-dump and missing-source-card-citation reports are rejected.

Proof level after this milestone: `Local Proof`.

### Milestone 2: Shared Adapter Contract

- [x] Define the first durable adapter-run contract equivalent via
      `knowledge_adapter_runs`.
- [x] Add first bridge from existing adapter-written source cards/radar runs
      into the shared knowledge substrate.
- [x] Wrap RSS adapter jobs into the shared contract.
- [x] Wrap GitHub repo adapter jobs into the shared contract.
- [x] Wrap GitHub owner/org/person adapter jobs into the shared contract.
- [x] Wrap arXiv adapter jobs into the shared contract.
- [x] Wrap Hacker News adapter jobs into the shared contract.
- [x] Wrap Reddit adapter jobs where sanctioned access exists into the shared
      contract.
- [x] Wrap X bookmark/watch/recent-search jobs into the shared contract.
- [x] Add common provider error taxonomy for contract rows.
- [x] Add common cursor/source-health visibility at the adapter-run boundary.
- [ ] Add common source-card write transaction helper.

Refuting tests:

- [x] Cursor does not advance if source-card write fails.
- [x] Policy denial happens before network.
- [x] Cost denial happens before credentials.
- [x] 401/403/429/5xx classify correctly at the contract/error taxonomy level.
- [x] Partial malformed provider page does not corrupt cursor.
- [x] Duplicate provider records do not flood source cards.
- [x] Source-health row distinguishes healthy, stale, blocked, failed, partial.
- [x] Live public RSS, GitHub owner, arXiv, and Hacker News adapter evidence can
      be projected from a scored radar run into confirmed shared knowledge
      events without manual row surgery.
- [x] Adapter success/failure rows include provider/source identity, counts,
      cursor before/after, source-card ids, and classified errors.

Proof level after this milestone: the foreground projection bridge is
`Production Data Proof` for public RSS, GitHub owner, arXiv, and Hacker News
through `scripts/knowledge-live-e2e-proof`. The shared adapter contract now has
local severe proof and is written by the adapter job boundary; broad scheduled
live recurrence and per-source-family live/copy proof packets still remain.

### Milestone 3: Event Extraction

- [x] Implement first deterministic event extraction from source cards.
- [ ] Add canonical keys for GitHub repos/releases, packages, models, papers,
      URLs, X posts, and topic events.
- [x] Add first deterministic entity extraction and linking for providers,
      source items, GitHub owners, and GitHub repos.
- [x] Add first source-role assignment.
- [x] Add schema-gated model-origin entity-resolution proposal recording.
- [x] Add live optional model invocation for entity-resolution suggestions
      behind policy/cost, with pending-review-only durable output.
- [ ] Add live optional model invocation for entity/event extraction behind
      policy/cost.
- [x] Write event-source rows.

Refuting tests:

- [ ] Reaction post cannot become first-party announcement.
- [ ] GitHub release and X announcement for same repo coalesce.
- [ ] Same org launching two different repos creates two events.
- [x] Same bare repo name under two GitHub owners creates distinct entities and
      a non-merge resolution.
- [ ] Simon-style benchmark post links to benchmark entity and prior wiki page.
- [ ] Karpathy workflow social post is labeled anecdotal if no primary writeup.
- [ ] NVIDIA model release requires model-card/repo/paper/license evidence
      before high-confidence model claims.

### Milestone 4: Cross-Source Clustering

- [x] Implement first deterministic cluster creation from projected source
      cards/radar runs.
- [x] Implement first source-diversity and momentum scoring.
- [ ] Implement wiki novelty lookup.
- [x] Implement first deterministic relation extraction for provider reporting,
      GitHub owner/repo ownership, and cluster co-occurrence.
- [x] Add semantic/model cluster proposal behind schema validation.
- [x] Add scheduled model-cluster proposal jobs that write candidate-only
      clusters and preserve the promotion boundary.
- [x] Add copied-home live-provider scheduled model-cluster proof over a bounded
      resident worker loop.
- [x] Add policy-gated promotion before model-origin clusters can drive
      wiki/report/digest expansion.
- [x] Add explicit model-backed cluster writer behind promotion, policy/cost,
      source-card citation, uncertainty, and wiki/report quality gates.
- [x] Add explicit cluster-scoped scheduled model writer watch source that
      reuses the same writer job and suppresses duplicate deterministic
      expansion while active and after terminal model-writer decisions.
- [x] Add live OpenAI scheduled model-writer proof through a bounded resident
      worker loop over a proof-scoped promoted cluster.
- [ ] Add cluster revisioning or metadata to avoid stale report reuse.

Refuting tests:

- [ ] OpenAI GitHub release plus X post plus HN discussion forms one promoted
      production-corpus cluster after policy review.
- [ ] Unrelated OpenAI posts do not merge just because the company matches.
- [ ] Vercel SDK launch compares to prior agent SDK pages without claiming
      equivalence unsupported by evidence.
- [ ] Duplicate URLs are grouped, not dropped.
- [ ] Model cluster output with missing members fails closed.
- [x] Model-origin candidate clusters cannot be expanded or queued before
      `knowledge_cluster.promote` policy approval.
- [x] Scheduled model-cluster proposal jobs skip empty evidence without a
      provider call, fail closed on provider-policy denial, and do not create
      wiki/report/digest side effects before promotion.
- [x] Live OpenAI scheduled model-cluster proof over copied source-card corpus
      creates candidate-only clusters and no expansion side effects.
- [x] Model writer output missing exact source-card citations, missing
      uncertainty/cluster-link structure, or trying to authorize delivery fails
      closed with no wiki/report/digest writes.
- [x] Scheduled model writing cannot be configured for unpromoted model-origin
      clusters and does not create duplicate active writer jobs.
- [x] Live OpenAI scheduled model-writer proof completes from due watch source
      through resident worker, source-health advancement, local investigation
      follow-through, ops/browser visibility, and no external delivery.

### Milestone 5: Editorial Decision Worker

- [x] Add first deterministic `editorial_decide` worker job for shared
      knowledge clusters.
- [x] Route due shared-cluster recurrence and backlog-completion follow-ups
      through `editorial_decide` before expansion.
- [x] Add first foreground deterministic decision rule for source-card/radar
      projection reports.
- [ ] Add model-assisted decision explanation behind policy/cost.
- [x] Add first local duplicate-page avoidance by matching an existing wiki
      page before creating a new cluster-authored page.
- [x] Add first update-vs-new-page selection.
- [x] Add digest-only selection for high-momentum clusters that already have a
      matching wiki page.
- [x] Add block-for-review path for review-only model-origin clusters.

Refuting tests:

- [x] Empty cluster cannot create page.
- [x] Weak single-source rumor becomes monitor or research, not alert.
- [x] Known page update is chosen over duplicate page.
- [x] High-confidence launch creates report candidate through the worker
      follow-up path.
- [x] Unsupported model-origin cluster cannot authorize delivery or write
      wiki/report/digest rows before promotion.
- [x] Due recurrence suppresses active/completed/blocked editorial decisions
      and does not create duplicate editor jobs.

Current local proof boundary:

- `knowledge_cluster_editorial_decide` records a durable
  `editorial_decide` decision with source-card evidence and chooses
  `expand_wiki_and_digest`, `digest_only`, `update_existing_wiki`,
  `monitor_only`, or `block_for_review`.
- A completed `editorial_decide` decision can enqueue exactly one local
  `knowledge_cluster_expand` follow-up for eligible clusters. The resident
  worker now queues due shared clusters and backlog-completion follow-ups into
  `knowledge_cluster_editorial_decide` first; direct due expansion remains an
  explicit operator/repair API, not the autonomous default path.
- The worker never authorizes external delivery. Digest delivery remains behind
  recipient, quiet-hours, dedupe, idempotency, retry/dead-letter, and provider
  proof gates.
- Remaining gaps: model-assisted editorial explanation, robust semantic
  duplicate-page detection beyond first-pass wiki search, broad production
  corpus quality review, live recurring service proof, and external delivery
  proof.

### Milestone 6: Research Fanout

- [ ] Add investigation job planner.
- [ ] Add GitHub context fetcher.
- [ ] Add package registry fetcher where supported.
- [ ] Add model-card fetcher where supported.
- [ ] Add link expansion planner.
- [ ] Add related-source search planner.
- [ ] Add wiki-history comparison.
- [ ] Add competitor-context compiler.
- [ ] Integrate with research convergence for deeper topics.

Refuting tests:

- [ ] Unsafe links are skipped and recorded.
- [ ] Fanout respects cost cap.
- [ ] Provider search selected results are recorded as proof.
- [ ] Worker URL ingestion must complete before report cites page text.
- [ ] Failed fanout leaves cluster `needs_research` or `blocked`, not sent.

### Milestone 7: Wiki Writer And Quality Gate

- [x] Add first report template renderer for foreground projection reports.
- [x] Add source-card citation checker for `knowledge_reports`.
- [x] Add first unsupported-output checker through report length/prose/source-id
      gates.
- [ ] Add stale-evidence checker.
- [ ] Add duplicate-page checker.
- [x] Add prompt-injection-as-data regression coverage for source-card-backed
      knowledge projection and ops rendering.
- [x] Add uncertainty/confidence section validator for `knowledge_reports`.
- [x] Add proof-scoped model-backed wiki writer path that cannot write without
      exact source-card ids and the existing report quality gate.
- [ ] Add wiki page versioning and rollback.

Refuting tests:

- [ ] Link dump fails.
- [ ] Metadata-only report fails.
- [ ] Page with no source-card ids fails.
- [ ] Unsupported claim fails.
- [ ] Prompt injection in source remains quoted evidence.
- [ ] Existing page gets update with version history.
- [ ] Report includes human-readable explanation without requiring source clicks.
- [x] Model writer output that is uncited, malformed, or delivery-authorizing
      fails closed without creating wiki/report/digest rows.

### Milestone 8: Digest Routing

- [ ] Create digest candidate from `knowledge_reports`.
- [ ] Enforce review/policy/channel authorization.
- [ ] Enforce quiet hours.
- [ ] Enforce dedupe windows by cluster/report/recipient.
- [ ] Add immediate vs batch routing.
- [ ] Add retry/dead-letter reconciliation.
- [ ] Link deliveries back to cluster/report/wiki/source cards.

Refuting tests:

- [ ] No model-score-only sends.
- [ ] Rejected report does not send.
- [ ] Duplicate cluster does not send twice in dedupe window.
- [ ] Quiet hours defer and later resume.
- [ ] Provider failure records retry and no false sent status.
- [ ] Delivery body is human-readable and not a source-link dump.

### Milestone 9: Unified Ops

- [x] Add knowledge dashboard rows to ops snapshot.
- [x] Add `/ops/ui` knowledge tables.
- [ ] Add safe controls.
- [ ] Add doctor checks for stale sources, failed clusters, stuck decisions,
      empty reports, and failed delivery retries.
- [ ] Add proof artifact links in ops where available.

Refuting tests:

- [x] Malicious source text is escaped in ops.
- [ ] Stale cursor is visible.
- [ ] Failed fanout is visible.
- [ ] Empty report quality failure is visible.
- [ ] Controls require auth, CSRF, policy, idempotency.

### Milestone 10: Production Proof Suite

Add preserved proof scripts:

- [ ] `scripts/knowledge-local-fixture-proof`
- [x] `scripts/knowledge-live-e2e-proof`
- [ ] `scripts/knowledge-cross-source-production-proof`
- [ ] `scripts/knowledge-github-launch-proof`
- [ ] `scripts/knowledge-x-github-correlation-proof`
- [ ] `scripts/knowledge-model-release-proof`
- [ ] `scripts/knowledge-research-fanout-proof`
- [ ] `scripts/knowledge-wiki-quality-proof`
- [x] `scripts/knowledge-cluster-model-writer-proof`
      passed at
      `.arcwell-dev/proofs/knowledge-cluster-model-writer-proof-20260626T092354Z-43488/artifacts/proof-packet.json`:
      proof source cards -> review-only model-origin cluster -> pre-promotion
      writer denial -> explicit policy promotion -> live OpenAI model-backed
      wiki/report/digest-candidate creation -> no external delivery ->
      authenticated desktop/mobile ops visibility. This is not broad
      production-corpus writer quality or broad automatic model-writing sweeps.
- [x] `scripts/knowledge-model-writer-scheduled-proof`
      passed at
      `.arcwell-dev/proofs/knowledge-model-writer-scheduled-proof-20260626T095734Z-12775/artifacts/proof-packet.json`:
      proof source cards -> review-only model-origin cluster -> pre-promotion
      schedule denial -> explicit policy promotion -> due `knowledge_model_write`
      watch source -> bounded resident worker -> live OpenAI model-backed
      wiki/report/digest-candidate creation -> source-health advancement ->
      local investigation-execution follow-through -> no deterministic
      expansion job -> no external delivery -> authenticated desktop/mobile ops
      visibility. This is not broad production-corpus writer quality,
      multi-day service recurrence, external delivery, or broad automatic
      model-writing sweeps.
- [x] Local resident-worker due model-writer recurrence severe test:
      `severe_resident_worker_enqueues_due_promoted_model_writers_without_manual_job`
      proves `run_worker_once` itself invokes the due promoted model-origin
      writer sweep before shared editorial/expansion recurrence, completes one
      promoted model-writer job without a pre-created operator job or watch
      source, suppresses duplicate terminal reruns, and creates no external
      delivery. This is local proof, not live provider or multi-day proof.
- [x] Local cluster evidence revision stale-decision severe tests:
      `severe_cluster_evidence_revision_reopens_shared_editorial_recurrence`
      and
      `severe_cluster_evidence_revision_reopens_promoted_model_writer_recurrence`
      prove terminal decisions are current only for the source-card set they
      evaluated. Adding a fresh source card to an existing cluster updates the
      cluster fingerprint and reopens shared editorial/model-writer recurrence
      without authorizing external delivery. These tests now also approve an
      initial stale digest candidate, prove refreshed wiki/report/digest
      artifacts cite the new source card, supersede the old undelivered
      candidate, and verify that stale candidate fails the delivery gate.
- [x] `scripts/knowledge-digest-recurrence-proof`
      passed at
      `.arcwell-dev/proofs/knowledge-digest-recurrence-proof-20260626T075355Z-75160/proof-packet.json`:
      copied real source cards -> backlog cluster -> wiki/report expansion ->
      editorial-linked digest candidate -> reviewed scheduled controlled
      provider delivery -> duplicate suppression -> quiet-hours deferral -> ops
      visibility. The current proof uses the largest selected backlog cluster
      with 295 source cards and verifies large-cluster prose bounding plus a
      complete source-card id audit index. It is not wall-clock external
      recurrence.
- [x] `scripts/knowledge-wall-clock-recurrence-proof`
      passed at
      `.arcwell-dev/proofs/knowledge-wall-clock-recurrence-proof-20260626T081914Z-31980/proof-packet.json`:
      copied real source cards -> scheduled backlog watch source -> one
      bounded resident worker loop over 80 wall-clock ticks -> backlog cluster
      -> wiki/report expansion -> editorial-linked digest candidate ->
      policy-gated auto-approval while the same worker was alive -> scheduled
      controlled-provider delivery -> heartbeat and ops visibility. It is not
      multi-day service operation or live external inbox recurrence.
- [ ] `scripts/knowledge-ops-browser-smoke`

Cross-source proof must show:

- real or copied production source cards from at least three source families
- at least one GitHub/X/blog or GitHub/X/HN cluster
- event rows
- cluster rows
- editorial decision rows
- investigation rows
- human-readable wiki report
- digest candidate
- delivery ledger with controlled provider or authorized live channel
- ops visibility
- artifact secret scan

Operational proof must show:

- resident worker detects a later eligible source update
- source-health/cursor update after durable writes
- cluster changes
- wiki update
- digest candidate routing
- allowed-hours delivery
- no manual command between source update and delivery

## End-To-End Example Plans

### OpenAI GitHub Package Launch

Expected sources:

- GitHub repo or release
- package registry if applicable
- OpenAI docs/blog
- OpenAI X announcement
- HN/Reddit/X reactions
- prior Arcwell wiki pages about OpenAI tools, agents, SDKs

Pipeline:

- GitHub adapter detects repo/release.
- X adapter detects announcement.
- HN/Reddit/RSS detect discussion.
- Event extraction creates `github_package_release`.
- Cluster links repo, package, tweet, reactions.
- Investigation fetches README, docs, package metadata, reactions.
- Wiki writer creates or updates a page.
- Digest sends if importance and confidence exceed policy threshold.

Report must include:

- what the package is
- installation/use surface
- maturity signal
- why OpenAI released it now if evidence supports this, otherwise uncertainty
- reactions and critiques
- competitors and related tools
- related Arcwell pages

### Karpathy Claude-In-Slack Workflow

Expected sources:

- X post, blog, transcript, screenshot, or linked writeup
- reactions from developers
- prior wiki pages about Claude workflows, Slack, team-agent usage

Pipeline:

- Event type `workflow_pattern`.
- Confidence stays lower if only social anecdote exists.
- Editorial decision likely `digest_only`, `update_existing_page`, or
  `deepen_research`.
- Wiki page should frame it as a practice pattern, not an official product.

Report must include:

- method described
- why it is useful
- assumptions and missing evidence
- privacy/security caveats
- relation to prior agent/team workflow notes

### Simon Willison Benchmark

Expected sources:

- Simon blog post
- GitHub benchmark repo or gist
- related prior Stork/SVG benchmark page in wiki
- HN/reaction posts

Pipeline:

- Event type `benchmark_release`.
- Relation `replaces` or `extends` only if directly supported.
- Investigation compares methodology and intended task.

Report must include:

- what benchmark measures
- what it replaces or improves
- methodology caveats
- implications for model/tool evaluation

### NVIDIA Open Source Model

Expected sources:

- NVIDIA announcement
- model card
- repo
- paper/arXiv if available
- license
- benchmark claims
- community reaction

Pipeline:

- Event type `model_release`.
- Investigation extracts license, model size, benchmark, use constraints.
- Relation to competing models and prior NVIDIA releases.

Report must include:

- model capabilities
- availability and license
- benchmark caveats
- competitive context
- deployment implications

### Vercel Eve Or Agent SDK Launch

Expected sources:

- Vercel announcement
- docs
- GitHub/package
- examples
- X/HN/reactions
- prior wiki pages about Vercel AI SDK, OpenAI Agents SDK, MCP, workflows

Pipeline:

- Event type `agent_sdk_launch`.
- Investigation compares developer experience and workflow abstraction.
- Editorial decision likely `create_new_page` or update Vercel/agent SDK page.

Report must include:

- what is new
- developer workflow
- relation to Vercel AI SDK and competitors
- adoption/maturity signals
- what to watch next

## Severe Test Matrix

Input safety:

- [ ] malicious X text
- [ ] malicious Reddit/HTML text
- [ ] prompt injection in GitHub README
- [ ] huge RSS entry
- [ ] malformed JSON provider page
- [ ] invalid Unicode/control characters
- [ ] unsafe URLs and redirects
- [ ] duplicate source records

Pipeline consistency:

- [ ] cursor no-advance on partial write
- [ ] event extraction idempotency
- [ ] cluster merge idempotency
- [ ] duplicate page prevention
- [ ] stale cluster does not send
- [ ] failed investigation blocks report promotion
- [ ] generated-only report rejected

Provider and scheduling:

- [ ] 401/403/429/5xx classification
- [ ] expired credentials surfaced without token leak
- [ ] cost denial before network
- [ ] policy denial before credentials
- [ ] retry storm bounded
- [ ] dead-letter after max attempts
- [ ] quiet-hours deferral and resume
- [x] bounded copied-home wall-clock recurrence proof
- [ ] live external or multi-day service recurrence proof

Report quality:

- [ ] no metadata-only body
- [ ] no link dump
- [ ] source-card ids present
- [ ] unsupported claims detected
- [ ] uncertainty section required
- [ ] first-party vs reaction distinction
- [ ] competitive context cites evidence or says unknown
- [ ] historical relation uses wiki/source evidence

Ops/security:

- [ ] ops escapes hostile text
- [ ] controls require auth/CSRF/policy/idempotency
- [ ] secrets redacted from errors/artifacts
- [ ] stuck jobs visible
- [ ] stale sources visible
- [ ] failed deliveries visible

## Proof Packet Template

Every production-data or operational claim needs a proof packet containing:

- feature name and status
- exact claim
- source families used
- live/copy/mock classification
- source counts
- source-health and cursor before/after
- source-card ids
- event ids
- cluster ids
- investigation jobs and results
- wiki page id and body path
- quality gate result
- digest candidate id
- delivery ledger ids
- ops snapshot excerpts
- policy/cost decisions
- secret scan result
- commands run
- tests run
- adversarial judgment: promote, hold, or block
- remaining risks

## Promotion Rules

Promote source family to `Production Data Proof` only when:

- real or copied production data passes source-card, cursor, source-health,
  event extraction, and cluster gates
- proof packet is preserved
- ops can show state
- docs say exactly what was proven

Promote unified pipeline to `Operational` only when:

- at least three source families run on schedule
- cross-source cluster is detected after initial setup
- research fanout completes or blocks honestly
- wiki page is written or updated
- digest candidate is routed
- delivery occurs during allowed hours without manual intervention
- retries/dead letters and stale source states are visible
- proof packet shows all of the above

## Implementation Order

Recommended order:

1. Shared schema and ops visibility.
2. Adapter contract and source-health/cursor helpers.
3. Event extraction from existing source cards.
4. Cross-source clustering.
5. Editorial decisions.
6. Wiki writer quality gate.
7. Investigation fanout.
8. Digest routing from reports.
9. Unified ops controls.
10. Production proof scripts and status updates.

This order prevents the most common mirage: sources and summaries appear before
there is durable evidence, source health, cluster lineage, report quality, or
delivery accountability.
