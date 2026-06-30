# Source ingestion audit

Snapshot: 2026-06-29 12:51 UTC

Database: `/Users/chabotc/.arcwell/arcwell.sqlite3`

Proof packet: `.arcwell-dev/audits/source-ingestion-20260629T1252Z-current/artifacts/proof-packet.json`

Full source inventory: `.arcwell-dev/audits/source-ingestion-20260629T1252Z-current/artifacts/source_inventory.csv`

Source-list appendix: `.arcwell-dev/audits/source-ingestion-20260629T1252Z-current/artifacts/source_lists/README.md`

## Verdict

Arcwell has a real source-to-wiki-to-knowledge pipeline. It is not yet a robust regular indexer across every configured source family.

The strong part is provenance and downstream projection:

- `8,022 / 8,022` source cards point at existing wiki pages.
- `5,372 / 5,372` X items have both source-card and wiki-page projections.
- The knowledge layer has `266` clusters with `1,581` source-card references.
- It has `480` draft reports with `2,798` source references.
- It has `1,583` knowledge event-source links.
- It has `517` digest candidates carrying `5,795` source references.
- Current database counts also show `13,263` wiki pages, `1,545` knowledge entities, `7,096` relations, and `108` pending entity-resolution proposals.

The weak part is operational freshness:

- `660` source-health rows still record previous policy-deferred worker-enqueue failures and are due or overdue.
- X account monitoring is not healthy: `639` handle rows are failed, `914` configured handles have no health row, and `167` `x_monitor_watch_source` jobs are dead-lettered.
- GitHub owner ingestion is partly healthy, but `16` owner rows are rate-limited and `693` GitHub owner jobs are dead-lettered.
- RSS/feed ingestion has durable cards, but most configured feed health rows are currently failed.
- Blog URL jobs are completing, but only half of the configured blog URL watches have source-health rows.

So the correct status is: source evidence is being stored and linked into the wiki; downstream clustering/report/digest systems are active; regular freshness and recovery are only partial.

## Cadence

Configured cadence means target scheduling, not guaranteed successful indexing:

- `hot`: hourly
- `warm`: every six hours
- `cold`: daily

All configured source families in this audit are `warm` except `54` GitHub owners marked `hot`.

## Source families

| Source family | Configured | Active | Health now | Last useful success | Worker/check history | Durable evidence |
| --- | ---: | ---: | --- | --- | ---: | ---: |
| X/Twitter accounts | 1,553 | 1,553 | 639 failed, 914 no health row | 2026-06-25 05:34 UTC source-health success; latest durable item 2026-06-27 15:31 UTC | 167 monitor jobs, all dead-lettered | 4,867 durable items attributed to handles |
| X bookmarks | 1 | 1 | failed | 2026-06-27 15:07 UTC | 0 jobs in this family summary | 0 in this watch row |
| GitHub owners/orgs/users | 188 | 187 | 167 healthy, 5 failed, 16 rate-limited | 2026-06-29 11:55 UTC | 1,783 jobs, 1,089 completed, 693 dead-lettered | 2,066 durable owner-linked items; 7,491 accepted adapter items |
| GitHub repos observed from source cards | 2,055 observed repos | derived | not all first-class watch rows | latest GitHub card 2026-06-29 11:53 UTC | depends on owner/repo jobs | listed from durable source cards |
| RSS/blog feeds | 23 | 17 | 1 healthy, 22 failed | 2026-06-29 10:43 UTC | 38 jobs, 28 completed, 2 dead-lettered | 382 durable feed items |
| Direct blog/company/personal URLs | 14 | 14 | 7 healthy, 7 no health row | 2026-06-27 23:48 UTC source health; jobs through 2026-06-29 12:51 UTC | 18,778 jobs, 18,542 completed | source-health linkage is incomplete for this family |
| Research papers / arXiv queries | 3 | 3 | 3 healthy | 2026-06-29 10:43 UTC | 113 jobs, 113 completed | 168 durable paper items |
| Internal knowledge watches | 3 | 3 | 2 healthy, 1 no health row | 2026-06-29 12:27 UTC | internal scheduled knowledge jobs active | backlog/entity-resolution state exists |

## Full source lists

The full list is split into CSVs under `.arcwell-dev/audits/source-ingestion-20260629T1252Z-current/artifacts/source_lists/`:

- `twitter_accounts.csv`: 1,553 rows
- `twitter_bookmarks.csv`: 1 row
- `github_orgs_and_users.csv`: 188 rows
- `github_repos_observed_from_source_cards.csv`: 2,055 rows
- `rss_and_blog_feeds.csv`: 23 rows
- `blog_urls.csv`: 14 rows
- `research_paper_queries.csv`: 3 rows
- `internal_analysis_sources.csv`: 3 rows
- `blogs_and_feeds_classified.csv`: 37 rows
- `source_card_provider_summary.csv`: 16 rows

Each configured-source CSV includes locator, label, cadence, watch status, health status, last success, last failure, last error, next run, worker job counts, adapter counts, durable item count, and latest durable item timestamp.

## Blog and feed sources

Direct blog/company/personal URL watches:

- `https://ai.meta.com/blog/`
- `https://blog.google/technology/ai/`
- `https://deepmind.google/discover/blog/`
- `https://developer.nvidia.com/blog/category/generative-ai/`
- `https://eugeneyan.com/start-here/`
- `https://hamel.dev/`
- `https://huggingface.co/blog`
- `https://jalammar.github.io`
- `https://karpathy.github.io`
- `https://mistral.ai/news/`
- `https://openai.com/news/`
- `https://research.google/blog/`
- `https://www.anthropic.com/news`
- `https://www.microsoft.com/en-us/research/blog/`

RSS/feed watches:

- `https://alphasignalai.substack.com/feed`
- `https://bullrich.dev/tldr-rss/ai.rss`
- `https://cameronrwolfe.substack.com/feed` paused
- `https://cobusgreyling.substack.com/feed`
- `https://importai.substack.com/feed`
- `https://lilianweng.github.io/index.xml`
- `https://magazine.sebastianraschka.com/feed` paused
- `https://news.smol.ai/rss.xml`
- `https://simonwillison.net/atom/everything/`
- `https://sub.thursdai.news/feed`
- `https://thecreatorsai.com/feed`
- `https://theneurondaily.com` paused
- `https://thesequence.substack.com/feed`
- `https://til.simonwillison.net/atom/all` paused
- `https://turingpost.substack.com/feed`
- `https://www.bensbites.com/feed`
- `https://www.exponentialview.co/feed`
- `https://www.interconnects.ai/feed`
- `https://www.latent.space/feed`
- `https://www.normaltech.ai/feed`
- `https://www.superhuman.ai` paused
- `https://www.therundown.ai` paused
- `https://www.whatshotit.vc/feed`

Research-paper watches:

- `cat:cs.AI`
- `cat:cs.CL`
- `cat:cs.LG`

## Policy and scheduling

Representative policy checks in the proof packet show scheduled enqueue is allowed for:

- direct blog URL ingest
- RSS fetch
- GitHub owner fetch
- arXiv search
- knowledge backlog
- knowledge entity resolution
- X handle monitor enqueue

The same policy check still defers X bookmark enqueue. X is also operationally unhealthy due to failed/rate-limited/dead-lettered monitor state, so broad X freshness should not be claimed.

## Downstream proof

The proof packet validates two hard invariants:

- every `source_cards` row points at an existing `wiki_pages` row
- every `x_items` row has both `source_card_id` and `wiki_page_id`

Downstream tables show the wiki evidence is being consumed by the knowledge system:

- `knowledge_clusters`: `266` clusters, `1,581` source references, latest update 2026-06-29 12:03 UTC
- `knowledge_reports`: `480` draft reports, `2,798` source references, latest update 2026-06-29 12:51 UTC
- `knowledge_event_sources`: `1,416` primary and `167` secondary source links
- `digest_candidates`: `5` approved, `126` pending, `386` ready
- `knowledge_entities`: `1,545`
- `knowledge_relations`: `7,096`
- `knowledge_entity_resolutions`: `108` pending-review resolution rows

This proves linkage and active downstream processing. It does not prove every source is fresh, every downstream job is caught up, or model-written analysis quality.

## Robustness judgment

Current status: `partial_operational_audit`.

Do not call the ingestion system robust until these gates pass:

1. A bounded worker pass refreshes the stale non-X policy-deferred source-health rows.
2. A fresh audit shows repaired RSS/GitHub/arXiv rows no longer carrying stale policy-deferred due state.
3. X watch polling is capped or sharded enough to avoid dead-letter/rate-limit collapse.
4. X bookmark policy is repaired only after that capping/sharding proof exists.
5. Dead-lettered GitHub, RSS, X, and knowledge jobs are triaged by kind and replayed only after root cause repair.
6. Blog URL scheduling writes clean source-health/cadence state for all configured blog URL watches.
