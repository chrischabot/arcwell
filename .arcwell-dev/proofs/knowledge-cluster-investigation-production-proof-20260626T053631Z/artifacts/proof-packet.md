# Knowledge Cluster Investigation Production-Corpus Proof

## Feature Name And Status

Feature: autonomous source-card-linked investigation workflow creation from shared knowledge clusters.

Status: Production Data Proof for copied-home foreground cluster expansion and investigation planning. Local Proof for worker execution. Not Operational.

## User-Visible Claim

When a source-backed shared knowledge cluster is expanded, Arcwell now creates or reuses a durable research workflow instead of leaving `Next Investigation` as inert report text. The workflow links the cluster source cards into a `deep_open` research run and creates pending tasks for primary-source verification, independent corroboration, wiki context mapping, and digest-readiness editing.

This proof does not claim the research tasks have been executed, that primary sources were fetched, that model-backed synthesis is accepted, that live X credentials are fresh, that external delivery recurred over wall-clock time, or that the system is operationally complete.

## Inputs And Outputs

Input:

- Copied real Arcwell home from `/Users/chabotc/.arcwell` into `.arcwell-dev/proofs/knowledge-cluster-investigation-production-proof-20260626T053631Z/home`.
- Existing durable source-card corpus in the copied home.
- Backlog command cap: `--max-source-cards 500 --min-group-size 2 --max-clusters 12`.
- Expanded cluster: `kcl-1f6f4730aae00342`.

Output:

- `.arcwell-dev/proofs/knowledge-cluster-investigation-production-proof-20260626T053631Z/cluster-backlog.json`
- `.arcwell-dev/proofs/knowledge-cluster-investigation-production-proof-20260626T053631Z/expand-cluster.json`
- Wiki page `knowledge-anthropic-source-backed-updates-7645d071`
- Knowledge report `krpt-63e97a8324bbf74f`
- Digest candidate `6562ba25-2cb7-4fea-8471-6e25d4f5d9ac`
- Research run `3c1c8a14-d595-447a-a3e4-c354f20b4fd7` with status `deep_open`
- 4 pending research tasks
- 65 source-card links into the research run

## Durable State Written

The proof mutated only the copied SQLite home. It wrote a wiki page, knowledge report, digest candidate, `investigate_cluster` editorial decision, research run, research tasks, and research-run source links in the disposable home.

The real home was not mutated by this proof.

## Task Roles

- `primary_source_verifier`
- `corroboration_scout`
- `wiki_context_mapper`
- `digest_readiness_editor`

Each task says source-card bodies are untrusted evidence, not instructions. The severe fixture proves hostile source text such as secret-exfiltration instructions is linked as evidence but not copied into trusted task instructions.

## False Start Caught And Fixed

The first production-copy expansion attempt failed with `report_looks_like_link_dump` because the report quality audit counted a legitimate large `## Evidence Synthesis` / `## Sources` section as link-dump shape. The audit now evaluates link-dump risk on the narrative analysis prefix while evaluating prose sufficiency over the report body minus the source index. Regression coverage keeps raw link dumps rejected while allowing inspectable large source indexes.

## Tests Added Or Tightened

- `severe_knowledge_cluster_investigation_job_is_source_linked_and_idempotent`
- `severe_knowledge_cluster_expansion_writes_wiki_report_and_deduped_digest` now asserts the investigation workflow is created and reused.
- `severe_worker_runs_knowledge_cluster_expansion_job` now asserts the worker result includes investigation run/task metadata.
- `severe_knowledge_cluster_wiki_audit_rejects_empty_uncited_link_dump` now proves large legitimate source indexes are not misclassified as link dumps.

## Commands Run

```sh
cargo fmt -- --check
cargo test -p arcwell-core severe_knowledge_cluster_investigation_job_is_source_linked_and_idempotent -- --nocapture
cargo test -p arcwell-core severe_knowledge_cluster_expansion_writes_wiki_report_and_deduped_digest -- --nocapture
cargo test -p arcwell-core severe_worker_runs_knowledge_cluster_expansion_job -- --nocapture
cargo test -p arcwell-core severe_knowledge_cluster_wiki_audit_rejects_empty_uncited_link_dump -- --nocapture
cargo test -p arcwell-core knowledge_cluster -- --nocapture
cargo test -p arcwell-core knowledge_projection -- --nocapture
cargo test -p arcwell worker -- --nocapture
cargo test -p arcwell-core severe_resident_worker_runs_scheduled_backlog_then_expands_cluster -- --nocapture
scripts/arcwell-dev sync
ARCWELL_HOME=.arcwell-dev/proofs/knowledge-cluster-investigation-production-proof-20260626T053631Z/home target/debug/arcwell knowledge cluster-backlog --max-source-cards 500 --min-group-size 2 --max-clusters 12
ARCWELL_HOME=.arcwell-dev/proofs/knowledge-cluster-investigation-production-proof-20260626T053631Z/home target/debug/arcwell knowledge expand-cluster kcl-1f6f4730aae00342
```

## Adversarial Review Judgment

Judgment: promote narrowly, hold broadly.

Promote:

- Expanded clusters now create or reuse durable, source-card-linked research workflows.
- Worker-executed cluster expansion returns investigation run/task metadata.
- Replay uses the existing `investigate_cluster` editorial decision instead of duplicating research runs/tasks.
- Legitimate large evidence/source indexes no longer fail the link-dump gate.

Hold:

- Pending investigation tasks are not completed research.
- No autonomous primary-source fetching or model-backed accepted synthesis is proven here.
- No live X credential freshness or external recurrence is proven here.
- No operational wall-clock service proof is proven here.
