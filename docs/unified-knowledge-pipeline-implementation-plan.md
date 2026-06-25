# Unified Knowledge Pipeline Implementation Plan

Date: 2026-06-25

Status: design plus implemented foreground bridge slices. This document does
not claim that the unified cross-source pipeline is operational. The shared
knowledge substrate and a live source-card/radar projection bridge are now
implemented and tested; scheduled recurrence, wiki expansion, digest routing,
semantic clustering, and broad provider coverage remain open.

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
- Wall-clock recurrence proof for real external alerts.
- Live X freshness is currently blocked by expired credentials.
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
- `/ops` and `/ops/ui` visibility for knowledge events, clusters, editorial
  decisions, reports, entities, and relations.
- Preserved production-data foreground proof:
  `.arcwell-dev/proofs/knowledge-live-e2e-proof-20260625T165254Z-73353/artifacts/proof-packet.json`.

What the bridge proof showed:

- Live public RSS, GitHub owner, arXiv, and Hacker News adapters completed.
- A scored radar run projected 12 source cards into 12 confirmed knowledge
  events.
- The projection wrote 9 source-backed entities and 19 source-backed relations.
- One source-backed cluster, one completed editorial decision, and one
  human-readable report were written durably.
- First-pass source-backed entities and relations are now part of the
  projection output and live proof harness assertions.
- Cursors and ops state were visible after durable writes.
- Authenticated `/ops/ui` rendered desktop and mobile knowledge tables through
  browser automation without horizontal overflow.

What it still does not prove:

- Resident scheduled recurrence over wall-clock time.
- Live X freshness, because local X credentials still need refresh/reauthorize.
- Entity/relation storage.
- Semantic/model-backed entity resolution beyond deterministic source metadata.
- Model-backed semantic synthesis or semantic multi-cluster splitting.
- Wiki page expansion/update jobs.
- Digest candidate routing and external delivery from shared knowledge reports.
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

- [ ] Define `KnowledgeSourceAdapter` trait or equivalent internal interface.
- [x] Add first bridge from existing adapter-written source cards/radar runs
      into the shared knowledge substrate.
- [ ] Wrap RSS/blog adapter.
- [ ] Wrap GitHub repo adapter.
- [ ] Wrap GitHub owner/org/person adapter.
- [ ] Wrap arXiv adapter.
- [ ] Wrap Hacker News adapter.
- [ ] Wrap Reddit adapter where sanctioned access exists.
- [ ] Wrap X bookmark/watch/recent-search adapters.
- [ ] Add common provider error taxonomy.
- [ ] Add common cursor/source-health update helper.
- [ ] Add common source-card write transaction helper.

Refuting tests:

- [ ] Cursor does not advance if source-card write fails.
- [ ] Policy denial happens before network.
- [ ] Cost denial happens before credentials.
- [ ] 401/403/429/5xx classify correctly.
- [ ] Partial malformed provider page does not corrupt cursor.
- [ ] Duplicate provider records do not flood source cards.
- [ ] Source-health row distinguishes healthy, stale, blocked, failed, partial.
- [x] Live public RSS, GitHub owner, arXiv, and Hacker News adapter evidence can
      be projected from a scored radar run into confirmed shared knowledge
      events without manual row surgery.

Proof level after this milestone: the foreground projection bridge is
`Production Data Proof` for public RSS, GitHub owner, arXiv, and Hacker News
through `scripts/knowledge-live-e2e-proof`. The true shared adapter contract is
still open, and each wrapped source family must keep or earn its own live/copy
proof packet.

### Milestone 3: Event Extraction

- [x] Implement first deterministic event extraction from source cards.
- [ ] Add canonical keys for GitHub repos/releases, packages, models, papers,
      URLs, X posts, and topic events.
- [x] Add first deterministic entity extraction and linking for providers,
      source items, GitHub owners, and GitHub repos.
- [x] Add first source-role assignment.
- [ ] Add optional schema-validated model extraction behind policy/cost.
- [x] Write event-source rows.

Refuting tests:

- [ ] Reaction post cannot become first-party announcement.
- [ ] GitHub release and X announcement for same repo coalesce.
- [ ] Same org launching two different repos creates two events.
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
- [ ] Add semantic/model cluster proposal behind schema validation.
- [ ] Add cluster revisioning or metadata to avoid stale report reuse.

Refuting tests:

- [ ] OpenAI GitHub release plus X post plus HN discussion forms one cluster.
- [ ] Unrelated OpenAI posts do not merge just because the company matches.
- [ ] Vercel SDK launch compares to prior agent SDK pages without claiming
      equivalence unsupported by evidence.
- [ ] Duplicate URLs are grouped, not dropped.
- [ ] Model cluster output with missing members fails closed.

### Milestone 5: Editorial Decision Worker

- [ ] Add `editorial_decide` worker job.
- [x] Add first foreground deterministic decision rule for source-card/radar
      projection reports.
- [ ] Add model-assisted decision explanation behind policy/cost.
- [ ] Add duplicate page detection.
- [ ] Add update-vs-new-page selection.
- [ ] Add digest-only selection.
- [ ] Add block-for-review path.

Refuting tests:

- [ ] Empty cluster cannot create page.
- [ ] Weak single-source rumor becomes monitor or research, not alert.
- [ ] Known page update is chosen over duplicate page.
- [ ] High-confidence launch creates report candidate.
- [ ] Unsupported model decision cannot authorize delivery.

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
- [ ] Add wiki page versioning and rollback.

Refuting tests:

- [ ] Link dump fails.
- [ ] Metadata-only report fails.
- [ ] Page with no source-card ids fails.
- [ ] Unsupported claim fails.
- [ ] Prompt injection in source remains quoted evidence.
- [ ] Existing page gets update with version history.
- [ ] Report includes human-readable explanation without requiring source clicks.

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
- [ ] `scripts/knowledge-digest-recurrence-proof`
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
- [ ] wall-clock recurrence proof

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
