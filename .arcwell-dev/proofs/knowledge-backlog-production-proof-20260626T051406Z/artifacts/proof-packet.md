# Knowledge Backlog Production-Corpus Proof

## Feature Name And Status

Feature: deterministic source-card backlog clustering and human-readable knowledge report projection.

Status: Production Data Proof for copied-home foreground backlog clustering over existing source cards. Not Operational.

## User-Visible Claim

Arcwell can take unclustered durable source cards from the real local corpus, split them into deterministic entity/theme clusters, and produce source-card-backed human-readable knowledge reports without turning generated/model output into source evidence.

This proof does not claim autonomous semantic/model clustering, wall-clock recurrence, live X credential freshness, live external delivery, or production service operation.

## Inputs And Outputs

Input:

- Copied real Arcwell home from `/Users/chabotc/.arcwell` into `.arcwell-dev/proofs/knowledge-backlog-production-proof-20260626T051406Z/home`.
- Existing production corpus inventory observed during health check: 6,547 source cards, 11,566 wiki pages, 1,781 watch sources, and 5,164 X tweet/item rows.
- Command cap: `--max-source-cards 500 --min-group-size 2 --max-clusters 12`.

Output:

- `.arcwell-dev/proofs/knowledge-backlog-production-proof-20260626T051406Z/cluster-backlog.json`
- 12 projected knowledge reports in the copied home.
- 157 accepted source cards, 343 skipped source cards, 22 groups considered, and 0 projection-time warnings.

## Durable State Written

The foreground clustering command ran against the copied SQLite home and wrote knowledge clusters, editorial decisions, reports, and source-card-backed projections in that disposable home.

The real home was not mutated by this proof.

## Source Families Used

The inspected corpus came from existing durable source cards. The largest produced cluster used `x-import` source-card evidence. Generated-only/model-output evidence is skipped by the backlog clustering path and is covered by severe tests in the main suite.

## Projection Summary

| Topic | Source Cards | Decision | Proof Level In Output |
| --- | ---: | --- | --- |
| Anthropic: source-backed updates | 65 | create_human_report | Local Proof |
| agent SDK and workflow tooling | 16 | create_human_report | Local Proof |
| OpenAI: source-backed updates | 13 | create_human_report | Local Proof |
| Vercel: source-backed updates | 11 | create_human_report | Local Proof |
| release and launch activity | 9 | create_human_report | Local Proof |
| MCP and agent infrastructure | 8 | create_human_report | Local Proof |
| benchmarks and evaluation | 6 | create_human_report | Local Proof |
| model release activity | 5 | create_human_report | Local Proof |
| Anthropic: MCP and agent infrastructure | 4 | create_human_report | Local Proof |
| Anthropic: agent SDK and workflow tooling | 3 | create_human_report | Local Proof |
| Anthropic: release and launch activity | 3 | create_human_report | Local Proof |
| Andrej Karpathy: source-backed updates | 2 | create_human_report | Local Proof |

The command output still labels report projections as `Local Proof`; the anti-mirage interpretation is that this run is production-corpus evidence for the deterministic bridge, not an operational promotion of the report writer.

## Human-Readable Report Gate

Every projected report body contained:

- `## What happened`
- `## Why it matters`
- `## Evidence`

The sampled report was narrative prose plus source-card snippets, not a bare digest metadata dump. It still remains deterministic source-card prose, not a model-backed analyst synthesis that investigates primary documents, compares history, or writes competitive analysis.

## Policy, Cost, Secrets, Authorization, And Trust Boundaries

- No provider/model calls were made in this proof.
- No external delivery was attempted.
- No credentials were printed into the proof packet.
- Source text remains untrusted evidence and must not be interpreted as an instruction.
- The copied home contains local private data and should stay under `.arcwell-dev/proofs`, not be committed wholesale.

## Idempotency And Duplicate Behavior

The committed severe tests cover replay suppression for source cards already clustered and duplicate active backlog job suppression. This production-corpus proof did not perform a second mutation replay over the copied home.

## Cursor, Source-Health, Retry, And Partial Failure Behavior

The committed resident-worker tests prove source-health advancement after durable backlog clustering. This foreground copied-home command does not prove wall-clock recurrence, long-running worker operation, retry recovery, or live cursor freshness.

## Indexing, Search, And Reporting Behavior

This proof demonstrates source-card-backed knowledge report generation from copied production rows. It does not prove model-quality synthesis, richer investigation jobs, wiki page expansion quality beyond the deterministic gate, or broad semantic coalescing across source families.

## Delivery Behavior

No digest/email/Telegram delivery was attempted. External recurrence remains open.

## Tests Added For This Slice

- `severe_source_card_backlog_clustering_splits_topics_and_skips_replay`
- `severe_resident_worker_runs_scheduled_backlog_then_expands_cluster`
- `severe_ops_ui_knowledge_backlog_controls_require_auth_csrf_policy_and_idempotency`
- Existing ops X controls regression rerun.

## Commands Run

```sh
cargo fmt -- --check
cargo test -p arcwell-core severe_source_card_backlog_clustering_splits_topics_and_skips_replay -- --nocapture
cargo test -p arcwell-core severe_resident_worker_runs_scheduled_backlog_then_expands_cluster -- --nocapture
cargo test -p arcwell-core knowledge_cluster -- --nocapture
cargo test -p arcwell worker -- --nocapture
cargo test -p arcwell ops_ui -- --nocapture
cargo test -p arcwell severe_ops_ui_knowledge_backlog_controls_require_auth_csrf_policy_and_idempotency -- --nocapture
cargo test -p arcwell severe_ops_ui_x_controls_require_auth_csrf_policy_and_idempotency -- --nocapture
cargo test --all --all-features
scripts/verify-codex-plugin-docs
scripts/arcwell-dev sync
ARCWELL_HOME=.arcwell-dev/proofs/knowledge-backlog-production-proof-20260626T051406Z/home target/debug/arcwell health
ARCWELL_HOME=.arcwell-dev/proofs/knowledge-backlog-production-proof-20260626T051406Z/home target/debug/arcwell knowledge cluster-backlog --max-source-cards 500 --min-group-size 2 --max-clusters 12
```

## Ops Visibility

`/ops/ui` now exposes authenticated Knowledge Controls for scheduling a backlog watch source and enqueueing a one-shot backlog clustering job. The ops controls require local origin, auth token, CSRF token, idempotency key, and explicit policy permission.

This proof did not run a browser screenshot against the copied-home controls.

## Docs And Status Updates

`README.md`, `STATUS.md`, and `TODO.md` should describe this as copied production-corpus evidence for deterministic backlog clustering/report projection only, with operational gaps still open.

## Adversarial Review Judgment

Judgment: promote narrowly, hold broadly.

Promote:

- Deterministic source-card backlog clustering can operate over a copied production corpus.
- It splits multiple topic clusters, skips nonaccepted rows, and creates human-readable report bodies rather than raw link dumps.
- Ops controls for scheduling/enqueueing the job are locally severe-tested for authorization, CSRF, policy, and idempotency.

Hold:

- Wall-clock autonomous recurrence.
- Live X credential freshness and bookmark/provider breadth.
- Semantic/model multi-source clustering over production data.
- Model-backed writer/editor investigation and synthesis.
- Live external digest recurrence.
- Broad ops repair controls and dashboard-level recovery workflows.

## Remaining Risks And Next Actions

- The output proof level is still `Local Proof`; add explicit production-corpus proof metadata if future commands should distinguish copied-production evidence from fixture evidence.
- Reports are readable but still deterministic and shallow; next slice should connect model-backed writer/editor jobs to clusters with strict citation gates.
- Backlog clustering is entity/theme heuristic based; next slice should production-proof semantic/model clustering over multi-provider source cards.
- Worker recurrence is covered by local tests, not a wall-clock resident-service proof; next slice should run a supervised long-lived worker over an eligible later item and capture source-health, wiki/report, digest, and delivery ledger transitions.
- X live provider is blocked by expired credentials; credential refresh must be solved before current X breadth can be promoted.
