# Iterated Epistemic Convergence Design

Date: 2026-06-23

## Purpose

Arcwell Deep Research currently has durable source cards, structured claims,
clusters, skeptic passes, evidence packs, editorial/eval records, audit gates,
report compilation, and an initial durable convergence loop with iterations,
statements, challenges, disproofs, revisions, fact checks, snapshots, report
judgments, worker resume, exact host-search task listing, host-search proof
consumption, policy/cost-gated provider fallback for pending challenge search
tasks, and an opt-in model-backed convergence citation/evaluator gate. That is
now a real local substrate, but it is not yet the complete first-class system
described here.

This document designs the next layer: an iterated epistemic convergence loop
that repeatedly turns evidence into explicit statements, attacks those
statements, revises them, searches for what would change the answer, and stops
only when the position has stabilized under bounded time, cost, and evidence
novelty constraints.

The target product behavior is:

```text
compile findings
-> make explicit statements and conclusions
-> pressure-test and try to disprove them
-> search/read/experiment based on what would matter
-> revise the position
-> repeat until no strong disproof remains or a clear stop limit is hit
```

"Settled" does not mean true. It means no strong disproof was found within the
run's declared scope, search depth, source coverage, time, cost, and
methodological limits.

## Research Basis

The design is grounded in current research-agent systems, evidence-synthesis
practice, and structured analytic methods.

### Deep Research Agents

The 2025 survey ["Deep Research Agents: A Systematic Examination and
Roadmap"](https://arxiv.org/html/2506.18096v2) defines deep research agents as
systems combining dynamic reasoning, adaptive planning, multi-hop retrieval,
iterative tool use, and structured analytical reports. It also calls out open
challenges directly relevant to Arcwell: broader information sources, fact
checking, asynchronous parallel execution, tool-integrated reasoning, benchmark
misalignment, multi-agent optimization, and self-evolving agents.

OpenAI describes Deep Research as multi-step internet research over hundreds of
online sources, with search, interpretation, analysis of text/images/PDFs, and
pivoting as information is encountered:
https://openai.com/index/introducing-deep-research/

Anthropic's multi-agent research architecture uses an orchestrator-worker
pattern: a lead agent plans, persists memory, delegates to specialized
subagents, synthesizes results, and decides whether more research is needed:
https://www.anthropic.com/engineering/multi-agent-research-system

Implication for Arcwell: keep Codex as the lead/orchestrator for live search
and subagent work, but make the loop state durable in Arcwell so long-running
research can resume, audit, and explain why it stopped.

### Iterative Refinement And Reflection Papers

[Self-Refine](https://arxiv.org/abs/2303.17651) shows that a
feedback/refine loop can improve output quality without extra training, but the
Arcwell version must not rely on undifferentiated self-critique. It should make
feedback structured, evidence-bound, and auditable.

[Reflexion](https://arxiv.org/abs/2303.11366) uses linguistic feedback and
episodic memory to improve later trials. Arcwell should store iteration
lessons, failed assumptions, and disproofs as first-class artifacts, not just as
hidden prompt context.

[ReAct](https://arxiv.org/abs/2210.03629) interleaves reasoning and actions, so
the model can update plans as tools return evidence. Arcwell should expose one
bounded loop step at a time: reason about the current position, choose the next
high-value action, perform it, record the result, and re-score.

[Self-RAG](https://arxiv.org/abs/2310.11511) argues for adaptive retrieval and
self-reflection rather than retrieving a fixed number of passages regardless of
need. Arcwell should retrieve based on concrete challenge gaps, not just source
count.

[STORM](https://arxiv.org/abs/2402.14207) improves long-form grounded writing
through multi-perspective question asking, simulated expert conversations, and
outline curation. Arcwell should use perspective generation early and again
after each revision to find missing frames.

[Prometheus](https://arxiv.org/abs/2310.08491) demonstrates fine-grained rubric
evaluation for long-form outputs. Arcwell should use task-specific rubrics for
evaluator stages, but treat LLM judges as evidence of quality, not final truth.

### Evaluation And Fact Checking

Anthropic's agent-eval guidance recommends converting manual checks and real
failure cases into eval tasks, then making behavioral changes visible before
production: https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents

[DeepResearchEval](https://arxiv.org/html/2601.09688v1) proposes task-specific
report-quality evaluation plus active fact checking that extracts report
statements and verifies them through external retrieval, including uncited
claims. Arcwell should copy the pattern: segment reports, extract verifiable
statements, independently retrieve evidence, label claims right/wrong/unknown,
and feed wrong/unknown high-impact claims back into the next iteration.

### Evidence Synthesis And Analytic Tradecraft

Cochrane's update guidance treats updates as a decision: whether the question
still matters, whether methods are still adequate, and whether new evidence or
methods should change the review:
https://www.cochrane.org/authors/handbooks-and-manuals/handbook/current/chapter-iv

GRADE provides a disciplined language for certainty, including high, moderate,
low, and very low confidence, and downgrades for risk of bias, inconsistency,
indirectness, imprecision, and publication bias:
https://www.cdc.gov/acip-grade-handbook/hcp/chapter-7-grade-criteria-determining-certainty-of-evidence/index.html

The US Government "Structured Analytic Techniques" primer emphasizes methods
that challenge judgments and expose cognitive bias:
https://www.stat.berkeley.edu/~aldous/157/Papers/Tradecraft%20Primer-apr09.pdf

Analysis of Competing Hypotheses is the core pattern to borrow: do not only ask
"what supports the current view?" Ask which evidence is diagnostic against each
alternative hypothesis.

### Research Tools Worth Integrating Or Emulating

Arcwell should not delegate truth to external research tools, but it should use
them as source discovery, corpus expansion, or eval references where configured.

- Codex host search remains the default current-search path because it is
  native to the operating environment and can be recorded through
  `research_host_search_record`.
- OpenAlex and Semantic Scholar are strong scholarly source-discovery layers.
  OpenAlex is a fully open catalog of the global research system, while
  Semantic Scholar is an AI-powered scholarly search surface.
- Elicit is the best product reference for systematic-review ergonomics:
  screening, data extraction, supporting quotes, and auditable/reproducible
  steps.
- Perplexity Sonar Deep Research is useful as an optional comparison/eval
  provider because it advertises exhaustive research across hundreds of sources,
  but its output should be treated as generated synthesis unless source links
  are inspected.
- LangGraph is a useful reference architecture for durable long-running loops,
  persistence, human-in-the-loop, and streaming. Arcwell should copy the
  durability idea into its SQLite/Rust/Codex environment rather than adopting a
  second orchestration runtime by default.

## Product Contract

Iterated convergence is not a separate shallow/deep mode. It is the serious
research behavior inside the one deep mode when the subject merits it.

The user-visible promise:

- Arcwell will state its current position explicitly.
- Arcwell will state the strongest arguments and evidence against that position.
- Arcwell will search for disconfirming and missing evidence, not just support.
- Arcwell will revise or drop conclusions when disproof is strong.
- Arcwell will stop for declared reasons: convergence, budget, time, source
  exhaustion, user stop, policy stop, tool failure, or unresolved conflict.
- Arcwell will show what changed between iterations.
- Arcwell will never call a conclusion settled while high-severity unresolved
  disproofs remain.

## Architecture

The design preserves the existing boundary:

- Codex owns live reasoning, host-native search, subagent orchestration,
  document inspection, and model calls exposed in the current thread.
- Arcwell owns durable state, schemas, source cards, claims, statements,
  challenges, disproofs, revisions, convergence metrics, costs, audits, and
  reports.
- The worker can resume local durable jobs and run provider-backed adapters
  where credentials/policy allow, but it should not pretend to have host-native
  Codex capabilities when it does not.

### High-Level Flow

```text
research_run
  -> source map and baseline retrieval
  -> claim extraction
  -> statement compiler
  -> convergence iteration 1
       -> statement set
       -> challenge set
       -> disproof search plan
       -> targeted retrieval/extraction
       -> verification and scoring
       -> revision set
       -> convergence snapshot
  -> convergence iteration N
  -> final report compile
  -> active fact-check
  -> audit
```

Each iteration must be restartable. If Codex, the MCP server, or the machine
dies mid-run, Arcwell should resume from the last completed step without
forgetting which conclusions were under attack.

### Loop Step Algorithm

Pseudo-code:

```text
while true:
  snapshot = read_run_state(run_id)
  statements = compile_or_read_current_statements(snapshot)
  challenges = generate_challenges(statements, source_coverage, prior_disproofs)
  high_value_questions = rank_challenges_by_expected_information_gain(challenges)
  evidence_delta = retrieve_and_extract(high_value_questions)
  verdicts = test_statements_against(evidence_delta, old_evidence, alternatives)
  revisions = revise_statement_set(statements, verdicts)
  metrics = score_convergence(statements, revisions, verdicts, evidence_delta)
  write_iteration_artifacts(...)

  if stop_rule_passes(metrics):
    mark_converged_or_stopped(...)
    break
```

The loop is not "reflect until happy." It is "identify what would change the
answer, go look for it, and revise if it does."

## Core Data Model

### `research_iterations`

One row per convergence pass.

Fields:

- `id`
- `run_id`
- `iteration_index`
- `parent_iteration_id`
- `status`: `planned`, `running`, `challenged`, `retrieving`, `revising`,
  `settled`, `stopped`, `failed`
- `objective`
- `position_artifact_id`
- `statement_set_artifact_id`
- `challenge_pack_artifact_id`
- `disproof_pack_artifact_id`
- `revision_artifact_id`
- `convergence_snapshot_id`
- `cost_decision_id`
- `started_at`
- `completed_at`
- `stop_reason`
- `error_message_redacted`

### `research_statements`

Statements are the unit under test. Claims are extracted evidence; statements
are the current analytical position.

Fields:

- `id`
- `run_id`
- `iteration_id`
- `parent_statement_id`
- `stable_key`
- `statement_type`: `fact`, `measurement`, `interpretation`, `conclusion`,
  `recommendation`, `hypothesis`, `design_proposal`, `forecast`,
  `open_question`
- `text`
- `scope`
- `temporal_scope`
- `confidence`: numeric 0.0 to 1.0
- `certainty_label`: `high`, `moderate`, `low`, `very_low`
- `status`: `proposed`, `survived`, `weakened`, `refuted`, `replaced`,
  `split`, `merged`, `unresolved`
- `importance`: `critical`, `high`, `medium`, `low`
- `evidence_json`: claim/source-card/document-anchor ids
- `counterevidence_json`
- `assumptions_json`
- `caveats_json`
- `created_by_role`
- `created_at`

### `research_challenges`

A challenge is a structured attempt to break a statement.

Fields:

- `id`
- `run_id`
- `iteration_id`
- `statement_id`
- `challenge_type`: `contradiction`, `alternative_hypothesis`,
  `missing_primary_source`, `stale_evidence`, `selection_bias`,
  `methodological_flaw`, `benchmark_flaw`, `numeric_error`, `table_anchor_gap`,
  `security_risk`, `privacy_risk`, `feasibility_risk`, `regulatory_risk`,
  `prior_art`, `economic_viability`, `implementation_complexity`,
  `citation_gap`
- `severity`: `critical`, `error`, `warning`, `info`
- `rationale`
- `would_change_answer_if_true`: boolean
- `search_plan_json`
- `required_source_families_json`
- `status`: `open`, `searching`, `answered`, `unresolved`, `waived`
- `created_by_role`
- `created_at`

### `research_disproofs`

Disproofs are the verdicts from testing challenges.

Fields:

- `id`
- `run_id`
- `iteration_id`
- `challenge_id`
- `statement_id`
- `verdict`: `refutes`, `weakens`, `supports`, `irrelevant`, `inconclusive`,
  `unknown`
- `strength`: `strong`, `moderate`, `weak`
- `evidence_json`
- `reasoning_summary`
- `confidence_delta`
- `requires_revision`: boolean
- `created_by_role`
- `created_at`

### `research_revisions`

Revisions preserve lineage.

Fields:

- `id`
- `run_id`
- `iteration_id`
- `from_statement_id`
- `to_statement_id`
- `revision_type`: `dropped`, `narrowed`, `confidence_downgraded`,
  `confidence_upgraded`, `split`, `merged`, `reframed`, `replaced`,
  `caveated`
- `rationale`
- `trigger_disproof_ids_json`
- `evidence_delta_json`
- `created_at`

### `research_convergence_snapshots`

One metrics row per iteration.

Fields:

- `id`
- `run_id`
- `iteration_id`
- `source_count_total`
- `source_count_new`
- `primary_source_count_new`
- `claim_count_total`
- `statement_count_current`
- `statement_count_changed`
- `critical_open_challenges`
- `high_open_challenges`
- `strong_refutations`
- `unknown_high_impact_claims`
- `mean_confidence_delta`
- `max_confidence_delta`
- `source_novelty_score`
- `claim_novelty_score`
- `position_edit_distance`
- `citation_support_score`
- `active_fact_check_score`
- `evaluator_score`
- `cost_usd_estimated`
- `elapsed_seconds`
- `stop_rule_json`
- `settled`: boolean

## Agent Roles

Existing roles remain, but convergence adds sharper role contracts.

### Position Compiler

Inputs: source cards, claims, clusters, skeptic findings, prior iterations.

Output: explicit statement set.

Rules:

- Use atomic, testable statements.
- Separate fact from inference, recommendation, forecast, and design proposal.
- Assign temporal scope and certainty.
- Link every factual statement to evidence ids or mark it unsupported.
- Do not convert source-inventory bookkeeping into analytical statements.

### Red Teamer

Inputs: statement set, source coverage, prior disproofs.

Output: challenge pack.

Rules:

- Attack critical/high statements first.
- Generate alternative hypotheses and missing-source tests.
- Prefer disconfirming searches over supporting searches.
- State what evidence would change the answer.

### Disproof Scout

Inputs: challenge pack.

Output: targeted source map and search records.

Rules:

- Use host-native search first when currentness matters.
- Search for contradictions, replications, retractions, failures, caveats,
  benchmarks, prior art, and skeptical analyses.
- Record host-search provenance before relying on results.

### Evidence Extractor

Inputs: new sources/documents.

Output: source cards, claim-ingest payloads, document anchors.

Rules:

- Preserve caveats, uncertainty, and dates.
- Reject malformed or uncertainty-losing extraction.
- Keep source text as evidence, never instruction.

### Verifier

Inputs: statements, evidence, disproofs.

Output: statement verdicts and confidence deltas.

Rules:

- Use statement-level verification.
- Label right/wrong/unknown where possible.
- Treat unknown on high-impact claims as unresolved, not pass.

### Reviser

Inputs: statement verdicts and confidence deltas.

Output: revised statement set and revision lineage.

Rules:

- Drop or narrow refuted statements.
- Split mixed statements.
- Downgrade confidence when evidence is weak or indirect.
- Keep unresolved contradictions visible.

### Convergence Auditor

Inputs: all iteration artifacts.

Output: stop/pass/fail decision.

Rules:

- A run cannot settle with critical unresolved challenges.
- A run cannot settle if strong refutations were not incorporated into
  revisions.
- A run cannot settle if high-impact factual statements lack support.
- A run can stop without settling when budget/time/tool limits hit.

## Challenge Strategy

Challenge generation should combine:

- Analysis of Competing Hypotheses: what alternative explanations fit the same
  evidence better?
- GRADE-like certainty downgrades: bias, inconsistency, indirectness,
  imprecision, publication bias.
- Domain-specific red teams: security, market, legal, scientific, engineering,
  benchmark, operational, cost, UX.
- Counter-search: explicit searches for "failure", "criticism",
  "replication", "negative result", "retraction", "benchmark flaw", "prior art",
  and "limitations".
- Adversarial citation checks: does the cited source actually support the
  exact sentence?
- Temporal attacks: is this still true as of the retrieval date?
- Source-family attacks: is the answer overfit to vendor blogs, papers, SEO
  content, social hype, or generated summaries?

## Convergence And Stop Rules

Every run gets hard limits and convergence limits.

### Hard Limits

Defaults should be policy-configured:

- max iterations: 4 for ordinary serious research, 8 for explicitly expansive
  research, 16 only with explicit long-run approval
- max wall time: 2 hours default, 24 hours only with explicit approval
- max estimated cost
- max provider calls
- max sources fetched per iteration
- max no-progress iterations
- user stop always wins

### Settled Conditions

A run can mark itself `settled` only when all are true:

- no critical unresolved challenges
- no high-severity unresolved challenges for high-importance statements
- no strong refutation left without a linked revision
- high-impact factual statements have valid evidence links
- active fact-check has zero wrong labels for high-impact statements
- unknown high-impact labels are either resolved or surfaced as caveats
- source novelty is below threshold for at least two consecutive iterations
- mean confidence delta is below threshold for at least two consecutive
  iterations
- current position edit distance is below threshold for at least two consecutive
  iterations
- citation verifier passes
- adversarial evaluator score meets gate
- deterministic `research_audit_run` passes

Suggested initial thresholds:

- source novelty below 0.05
- claim novelty below 0.05
- mean confidence delta below 0.03
- max confidence delta below 0.10 for non-critical statements
- zero critical open challenges
- zero high-impact wrong active fact-check labels
- at most 5 percent unknown high-impact labels, all caveated

These thresholds are starting points, not truth. They should be tuned by evals.

### Stop Without Settling

The loop should stop incomplete when:

- cost/time/source limits hit
- search/tool/provider access fails
- the topic remains unstable or current evidence keeps changing
- severe contradiction cannot be resolved
- the user stops the run
- policy blocks needed sources or tools

The report must then say `stopped_incomplete`, not `settled`.

## Invention And New-Technology Research

The same loop should support speculative invention, but with stricter labels.

New technology proposals are `design_proposal` or `hypothesis` statements, not
facts. They can graduate only after passing:

- prior-art search
- feasibility analysis
- threat model
- implementation complexity review
- benchmark or experiment plan
- cost and operational review
- failure-mode red team
- safety/security/privacy review where relevant

The loop for invention:

```text
source landscape
-> unmet constraints and failure modes
-> candidate design proposals
-> prior-art and impossibility search
-> technical feasibility attack
-> prototype/experiment plan if useful and safe
-> revised design
-> report: proposal, evidence, novelty, risks, tests needed
```

Arcwell must never present invented designs as proven. It can present them as
promising if they survive prior-art and feasibility attacks, and as proven only
after experiments or implementation evidence exists.

## CLI And MCP Surface

### CLI

```sh
arcwell research converge <run-id>
arcwell research converge-step <run-id>
arcwell research iterations <run-id>
arcwell research iteration-read <iteration-id>
arcwell research statements <run-id>
arcwell research challenges <run-id>
arcwell research disproofs <run-id>
arcwell research convergence-status <run-id>
arcwell research convergence-report <run-id>
```

New run flags:

```sh
arcwell research run "query" \
  --converge \
  --max-iterations 8 \
  --max-hours 6 \
  --budget-usd 25 \
  --source-novelty-threshold 0.05 \
  --confidence-delta-threshold 0.03
```

### MCP Tools

- `research_convergence_start`
- `research_convergence_step`
- `research_convergence_status`
- `research_iterations`
- `research_iteration_read`
- `research_statements`
- `research_statement_add`
- `research_statement_update`
- `research_challenges`
- `research_challenge_add`
- `research_disproof_record`
- `research_revision_record`
- `research_convergence_report_compile`

All tools should return stable JSON envelopes with `run_id`, ids created,
status, warnings, and next recommended actions.

### Capability Reporting

`research_capabilities` should add:

- `iterated_convergence`
- `statement_lineage`
- `challenge_disproof_records`
- `convergence_stop_rules`
- `active_fact_checking`
- `long_running_resume`

Each field should report `available`, `partial`, or `unavailable`, with the
current thread/tool-schema caveat where relevant.

## Report Wiring

The final report needs new sections:

- Executive judgment
- Final position
- What changed through iteration
- Statements that survived
- Statements weakened or refuted
- Strongest remaining caveats
- Unresolved disproofs
- Source coverage and saturation
- Convergence stop reason
- Active fact-check summary
- Iteration appendix
- Statement/disproof/revision ledger

Narrative rule: the report leads with the settled/revised judgment, not the
process log. The iteration ledger is inspectable but belongs in appendices.

## Implementation Plan

### Phase 0: Design And Schema Review

Behavioral claim:

> Arcwell can represent an iterative convergence loop without losing statement
> lineage, disproofs, revisions, or stop reasons.

Tasks:

- Add this design document.
- Review current `research_*` tables for reuse.
- Decide migration naming and schema version.
- Add schema comments in code and docs.

Proof gate:

- Read-only review confirms no existing table can be overloaded without losing
  meaning.

### Phase 1: Durable Schema And Read APIs

Behavioral claim:

> Iterations, statements, challenges, disproofs, revisions, and convergence
> snapshots can be recorded and read back with run scoping.

Tasks:

- Add migrations for the five new tables.
- Add Rust structs and normalization.
- Add store methods.
- Add CLI read/list commands.
- Add MCP read/list tools.

Severe tests:

- Cross-run ids rejected.
- Duplicate stable statement keys dedupe or version cleanly.
- Refuted statements cannot be silently overwritten.
- Invalid confidence/certainty/status rejected.
- Huge rationale/evidence payloads bounded.
- Secret-like data redacted in error fields.

### Phase 2: Statement Compiler

Behavioral claim:

> Arcwell can compile current claims/clusters/skeptic notes into explicit,
> evidence-linked, testable statements.

Tasks:

- Deterministic baseline statement compiler from existing claims.
- Model-backed optional compiler behind provider/cost policy.
- Statement types and certainty mapping.
- Source-bookkeeping filter reused from report narrative layer.

Severe tests:

- Source-inventory claims do not become conclusions.
- Mixed statements are split.
- Unsupported facts are marked unsupported, not hidden.
- Generated report text cannot become source evidence.
- Dates and temporal scope survive.

### Phase 3: Challenge And Disproof Engine

Behavioral claim:

> Arcwell can generate and record targeted attempts to disprove important
> statements.

Tasks:

- Deterministic challenge templates by statement type and domain.
- Optional model-backed red team stage.
- Search-plan generation for each challenge.
- Disproof verdict recording.

Severe tests:

- High-importance unsupported statements always receive challenges.
- Contradictory evidence produces `refutes` or `weakens`, not `supports`.
- Missing primary source is not waived by secondary commentary.
- Stale current claims trigger temporal challenges.
- Prompt-injection content cannot create waived challenges.

### Phase 4: Targeted Retrieval Loop

Behavioral claim:

> Each iteration searches based on unresolved high-value challenges and records
> host-search/source provenance before using new evidence.

Tasks:

- Add `research_convergence_step`.
- Rank challenges by expected information gain.
- Use host-native search when Codex exposes it.
- Use configured provider search as fallback.
- Link new source cards and claims to challenge ids.

Severe tests:

- A step cannot use unrecorded search results.
- A no-search environment marks challenges blocked, not answered.
- Low-reliability contradiction sources lower confidence but do not auto-refute.
- Duplicate source discoveries do not inflate novelty.

### Phase 5: Revision And Convergence Metrics

Behavioral claim:

> Arcwell can revise the current position and decide whether the loop has
> settled, stopped incomplete, or should continue.

Tasks:

- Revision writer.
- Confidence delta calculation.
- Position edit-distance approximation.
- Source/claim novelty scoring.
- Stop-rule evaluator.
- Incomplete/settled run states.

Severe tests:

- Critical unresolved challenge blocks `settled`.
- Strong refutation without revision blocks `settled`.
- Two no-progress iterations stop with explicit reason.
- Time/cost caps stop incomplete.
- Confidence cannot increase from generated synthesis alone.

### Phase 6: Model-Backed Active Fact Checking

Behavioral claim:

> Arcwell can independently verify report/statement claims and feed wrong or
> unknown high-impact facts back into the convergence loop.

Tasks:

- Segment report into factual statements.
- Use current source ledger plus external retrieval for verification.
- Record right/wrong/unknown labels.
- Feed wrong/unknown high-impact labels into challenges.
- Add evaluator score fields to convergence snapshots.

Severe tests:

- Uncited factual claims are extracted and checked.
- Wrong high-impact claims block settlement.
- Unknown high-impact claims are caveated or block settlement.
- Citation-support and external factuality are separate checks.

### Phase 7: Report Renderer

Behavioral claim:

> Final reports read like analyst reports, while preserving an auditable
> iteration ledger.

Tasks:

- Add convergence-aware report sections.
- Add statement/disproof/revision appendix.
- Show stop reason prominently.
- Show "what changed" in prose, not raw logs.

Severe tests:

- Refuted statements cannot appear as final conclusions.
- Unresolved severe disproofs appear in the executive caveats.
- Metadata-only iterations do not produce fake progress.
- Appendix preserves traceability.

### Phase 8: Long-Running Execution

Behavioral claim:

> A convergence run can operate for hours with resumable state, bounded cost,
> and observable progress.

Tasks:

- Worker job kind for convergence step scheduling.
- Heartbeat and resume behavior.
- Cost reservations per iteration and provider call.
- Progress resource/MCP status.
- Human checkpoint for high-cost/long-run continuation.

Severe tests:

- Crash after challenge generation resumes without duplicate writes.
- Crash after provider call but before revision does not lose cost/evidence.
- Stale lease can be reclaimed.
- Runaway loops hit max iteration/no-progress caps.
- User stop interrupts before next expensive action.

### Phase 9: Production Proofs

Behavioral claim:

> Iterated convergence materially improves report correctness and judgment
> quality over the current single-pass deep research flow.

Proof runs:

- Current technical/literature topic: image compression algorithms.
- Market/ecosystem topic: AI startup scene in London.
- Security/architecture topic: safe cloud code execution platform with
  compile-time security constraint verification.
- Invention topic: propose a new architecture for verified cloud code execution
  and attack it with prior art, feasibility, and threat-model checks.

Required artifacts:

- preserved run home
- host-search proof
- source ledger
- source cards
- document anchors where relevant
- statement ledger
- challenge ledger
- disproof ledger
- revision ledger
- convergence snapshots
- editorial/eval records
- final report
- audit output
- cost records

## Eval Plan

Compare four variants:

1. Current single-pass deep report.
2. Single-pass plus model editorial/eval.
3. Iterated convergence without active external fact-check.
4. Iterated convergence with active fact-check.

Metrics:

- unsupported factual sentence rate
- wrong high-impact claim count
- unknown high-impact claim count
- citation precision
- citation recall
- contradiction preservation
- caveat preservation
- final confidence calibration
- source-family coverage
- primary-source coverage
- novelty per iteration
- report usefulness score
- time/cost per accepted report
- human reviewer correction count

Use a mixed eval set:

- deterministic fixtures with seeded contradictions
- stale/currentness fixtures
- table/PDF numeric fixtures
- source-instruction injection fixtures
- live topics with preserved source snapshots
- human-reviewed analyst reports

## False-Done Traps

- A loop that only asks the same model to criticize itself is not convergence.
- A loop that searches only for support is not convergence.
- A loop that stops because the report sounds polished is not convergence.
- A loop that hides unresolved disproofs is worse than single-pass research.
- A loop that treats generated reports as evidence will self-reinforce.
- A loop that cannot resume is not suitable for hours/day research.
- A loop that lacks cost/time caps will eventually become an ops problem.
- A "settled" label without statement/disproof lineage is a trust bug.

## Recommended Build Order

Build the durable substrate first, then the loop controller:

1. Schema/read APIs.
2. Statement compiler.
3. Challenge/disproof/revision records.
4. Stop-rule evaluator.
5. One-step convergence MCP/CLI.
6. Codex skill prompt wiring.
7. Report renderer.
8. Worker/resume.
9. Active fact-check.
10. Live saturated proofs.

This order gives useful inspectable artifacts early without pretending the
autonomous loop is already solved.

## Open Questions

- Should statement confidence be continuous only, GRADE-like labels only, or
  both?
- Should high-risk domains require human approval before marking `settled`?
- How much of active fact-checking should use host-native search versus
  provider search APIs?
- Should long-running convergence be a worker job, a Codex-hosted workflow, or
  a hybrid where Codex performs each high-value retrieval phase and Arcwell
  schedules only bookkeeping/resume steps?
- Should Arcwell support parallel challenge branches in phase one, or start with
  sequential convergence and add parallel branches after correctness is proven?

## Anti-Shell Completion Contract

This section exists because the largest product risk is a feature that looks
complete from its API names, schemas, or prompts while the actual behavior is
thin. Iterated convergence cannot be called complete because a table, command,
or prompt exists. It is complete only when the full behavioral loop is proven
against adversarial fixtures and live runs.

### Universal Completion Rule

For every feature slice below, completion requires all of these:

- [ ] The behavioral claim is written in the PR/change note.
- [ ] The valid input space is named.
- [ ] The invalid input space is named.
- [ ] The security boundary is named.
- [ ] The persistence invariant is named.
- [ ] The cost/performance budget is named.
- [ ] At least one test tries to refute the happy path.
- [ ] At least one malicious input test tries to abuse the feature.
- [ ] At least one invalid input test proves honest failure.
- [ ] At least one restart/retry/idempotency test exists if durable state is
      written.
- [ ] At least one performance or scale test exists if the feature iterates over
      sources, claims, statements, challenges, or documents.
- [ ] CLI, MCP, skill docs, `STATUS.md`, and `TODO.md` describe the same
      maturity level.
- [ ] A severe/adversarial review has a written verdict.
- [ ] The exact validation commands and preserved artifacts are recorded.
- [ ] No known critical/high false-done trap remains unaddressed.

### Completion Labels

Use these labels in code comments, docs, status rows, and reports:

- `scaffold`: names or schemas exist, but behavior is not end-to-end.
- `local_proven`: deterministic local tests prove behavior with mock/provider
  fixtures.
- `live_proven`: the exact claimed external/provider/host behavior was tested
  live with preserved artifacts.
- `saturated_proven`: a full hundred-source class run completed with source
  ledger, statements, disproofs, revisions, convergence snapshots, editorial
  gates, audit, report, and report judgment.
- `production_ready`: saturated proof plus restart, cost, performance, malicious
  input, invalid input, docs, and operator observability gates pass.

No feature may jump labels. A live provider smoke does not imply saturated
proof. A saturated preserved-corpus rerender does not imply current live proof.
A prompt contract does not imply behavior.

## Feature Claim Ledger

Each feature below must be implemented and proven independently. Cross-feature
integration is a separate proof layer, not assumed from unit tests.

### F1: Run Configuration And Scope

Behavioral claim:

> A convergence run starts with explicit question, scope, limits, budget,
> freshness needs, privacy, and stop rules, and no hidden default can silently
> turn a small run into an expensive day-long job.

Implementation checklist:

- [ ] Add convergence fields to run creation inputs.
- [ ] Add default limit policy for max iterations, wall time, source count,
      provider calls, and cost.
- [ ] Add explicit opt-in for runs longer than the default limit.
- [ ] Add privacy/no-write policy propagation to every convergence artifact.
- [ ] Add freshness requirement and retrieval date baseline.
- [ ] Add run-level stop-rule config serialization.
- [ ] Add CLI flags for convergence limits.
- [ ] Add MCP schema fields for convergence limits.
- [ ] Add skill prompt language that forbids silent escalation.
- [ ] Add status/read output that shows current limits and remaining budget.

Severe tests:

- [ ] Missing limit config falls back to safe defaults.
- [ ] Negative, NaN, infinite, and huge limits are rejected.
- [ ] A user stop prevents the next expensive action.
- [ ] A no-write run writes no wiki/source artifacts outside allowed run state.
- [ ] Privacy flags propagate into role prompts and artifacts.
- [ ] A low-budget run cannot make provider calls after the cap.
- [ ] A long-run request without explicit approval stops at default limits.

Completeness measurement:

- [ ] CLI and MCP create identical persisted limit state for the same request.
- [ ] `research_read` shows limits, elapsed time, and stop-rule config.
- [ ] Adversarial review confirms there is no hidden escalation path.

### F2: Durable Iteration Schema

Behavioral claim:

> Every convergence pass is represented as a durable iteration with links to
> position, statement set, challenge pack, disproof pack, revision artifact,
> convergence snapshot, cost, timestamps, and stop/error state.

Implementation checklist:

- [ ] Add `research_iterations` migration.
- [ ] Add indexes by `run_id`, `iteration_index`, `status`, and timestamps.
- [ ] Add foreign-key or application-level validation for artifact ids.
- [ ] Add Rust structs and row mappers.
- [ ] Add create/read/list store methods.
- [ ] Add CLI list/read.
- [ ] Add MCP list/read.
- [ ] Add run-read embedding of latest iteration summary.
- [ ] Add redacted error field.
- [ ] Add schema migration ledger entry.

Severe tests:

- [ ] Cross-run artifact ids are rejected.
- [ ] Duplicate iteration indexes cannot corrupt lineage.
- [ ] A failed iteration preserves error and partial artifacts.
- [ ] Long error messages are bounded and redacted.
- [ ] Missing parent iteration is rejected when parent is supplied.
- [ ] A run can list 1000 iterations without pathological slowdown.

Completeness measurement:

- [ ] A disposable run can create, fail, resume, and list iterations.
- [ ] All rows survive process restart and database reopen.
- [ ] SQLite query plan uses indexes for common list/read paths.

### F3: Statement Ledger

Behavioral claim:

> Arcwell can represent the current analytical position as explicit,
> evidence-linked, testable statements with type, scope, confidence, certainty,
> status, importance, assumptions, caveats, and lineage.

Implementation checklist:

- [ ] Add `research_statements` migration.
- [ ] Add stable statement key normalization.
- [ ] Add statement type enum normalization.
- [ ] Add status enum normalization.
- [ ] Add certainty label normalization.
- [ ] Add importance normalization.
- [ ] Add confidence validation.
- [ ] Add evidence/counterevidence JSON validation.
- [ ] Add parent/current lineage fields.
- [ ] Add CLI/MCP list/read/add/update tools.

Invalid input tests:

- [ ] Empty text rejected.
- [ ] Overlong text rejected or bounded.
- [ ] Invalid statement type rejected.
- [ ] Invalid certainty label rejected.
- [ ] Confidence below 0 or above 1 rejected.
- [ ] Evidence ids from another run rejected.
- [ ] Duplicate stable keys versioned rather than silently overwritten.

Malicious input tests:

- [ ] Prompt-injection text is stored as data, not executed.
- [ ] Markdown/HTML/script payloads are escaped in renderers.
- [ ] SQL metacharacters do not affect queries.
- [ ] Unicode spoofing does not bypass stable-key dedupe.
- [ ] Cross-run statement id cannot be used to attach evidence.

Completeness measurement:

- [ ] A report can show current statements, dropped statements, and statement
      lineage.
- [ ] Every high-impact final conclusion has a statement row.
- [ ] No report conclusion exists only as prose.

### F4: Statement Compiler

Behavioral claim:

> Arcwell can compile source cards, claims, clusters, skeptic notes, and prior
> iterations into a statement set without turning source inventory, generated
> output, or weak evidence into false conclusions.

Implementation checklist:

- [ ] Deterministic compiler from current claims and clusters.
- [ ] Optional model-backed compiler behind provider/cost policy.
- [ ] Filter source/corpus bookkeeping before statement creation.
- [ ] Preserve dates and temporal scope.
- [ ] Split compound statements.
- [ ] Mark unsupported inference explicitly.
- [ ] Link each statement to evidence or unresolved assumption.
- [ ] Store compiler artifact and prompt version.
- [ ] Add compiler score/warnings.
- [ ] Add compiler output to iteration.

Quality tests:

- [ ] Compound statement fixture splits into atomic statements.
- [ ] Metadata-only source corpus creates no analytical conclusion.
- [ ] Conflicting claims produce separate unresolved statements or caveats.
- [ ] High-confidence model prose without source evidence is rejected.
- [ ] Currentness-sensitive statements carry retrieval/date scope.

Adversarial tests:

- [ ] Generated report artifact cannot be used as evidence for new statements.
- [ ] SEO spam claims are not promoted to high confidence.
- [ ] Vendor-only support downgrades certainty for broad claims.
- [ ] Contradictory benchmark claims are not smoothed into one average claim.

Completeness measurement:

- [ ] Statement compiler catches all final factual report claims in a seeded
      fixture.
- [ ] Independent auditor cannot find unsupported final conclusions that lack
      statement rows.

### F5: Challenge Generator

Behavioral claim:

> Important statements receive targeted challenges that specify what evidence
> would change the answer.

Implementation checklist:

- [ ] Add deterministic challenge templates by statement type.
- [ ] Add domain lenses: scientific, market, security, legal, engineering,
      product, operational, cost, policy.
- [ ] Add optional model-backed red-team generator.
- [ ] Add expected-information-gain ranking.
- [ ] Add required source-family output.
- [ ] Add challenge status lifecycle.
- [ ] Add challenge search-plan artifact.
- [ ] Add CLI/MCP add/list/read.
- [ ] Link challenges to statement ids and iteration ids.

Invalid input tests:

- [ ] Challenge without statement id rejected.
- [ ] Challenge against cross-run statement rejected.
- [ ] Unknown challenge type rejected.
- [ ] Missing severity rejected.
- [ ] Empty search plan rejected for high-severity challenges.

Malicious input tests:

- [ ] Prompt injection cannot mark a challenge waived.
- [ ] Source text cannot create fake `answered` status.
- [ ] Hostile challenge rationale cannot inject report markdown/script.
- [ ] Duplicate challenge ids cannot overwrite earlier challenges.

Completeness measurement:

- [ ] Every critical/high statement has at least one challenge.
- [ ] Every unsupported high-impact statement has a missing-evidence challenge.
- [ ] Adversarial review agrees challenge set would catch plausible false
      conclusions.

### F6: Disproof Retrieval

Behavioral claim:

> The loop searches for evidence that would weaken or refute statements, not
> just evidence that supports the current narrative.

Implementation checklist:

- [ ] Add challenge-ranked retrieval planner.
- [ ] Use host-native search first when available.
- [ ] Record all host search queries/results before reliance.
- [ ] Fall back to configured provider search with policy/cost checks.
- [ ] Link search results to challenges.
- [ ] Link new source cards to challenges.
- [ ] Record blocked/unavailable searches.
- [ ] Add "supporting-only search" audit warning.
- [ ] Add retrieval novelty scoring.

Quality tests:

- [ ] Known contradiction source is discovered from a challenge query.
- [ ] Duplicate sources do not inflate novelty.
- [ ] Blocked search produces unresolved challenge, not pass.
- [ ] Low-reliability contradiction weakens but does not auto-refute.

Malicious tests:

- [ ] SSRF-style URLs rejected during ingestion/fetch.
- [ ] Redirect to private/metadata IP rejected.
- [ ] Search snippets with tool instructions stored as data only.
- [ ] Malicious URLs cannot become local file reads.

Completeness measurement:

- [ ] Each high-severity challenge has search provenance or a blocked reason.
- [ ] Final report can show disconfirming search coverage.

### F7: Evidence Extraction And Claim Ingest For Iterations

Behavioral claim:

> New evidence discovered by a convergence step is converted into source cards,
> claims, and document anchors with uncertainty preserved.

Implementation checklist:

- [ ] Attach new source cards to challenge ids.
- [ ] Attach claims to challenge ids.
- [ ] Add document anchor validation for challenge evidence.
- [ ] Preserve extractor warnings in disproof scoring.
- [ ] Link source family/read-depth to convergence metrics.
- [ ] Add same-run validation for every evidence link.
- [ ] Add generated-output exclusion.
- [ ] Add source-card artifact recursion prevention.

Invalid input tests:

- [ ] Malformed claim JSON rejected.
- [ ] Missing uncertainty/caveat on uncertain claim rejected or downgraded.
- [ ] Cross-run document anchor rejected.
- [ ] Nonexistent span/table/cell anchor rejected.
- [ ] Unsupported document format fails honestly.

Malicious tests:

- [ ] PDF prompt injection treated as source text.
- [ ] XLSX formula text preserved as untrusted text.
- [ ] CSV formula injection does not execute.
- [ ] Oversized PDF/XLSX/CSV rejected or bounded.
- [ ] Source-card title/URL cannot inject report HTML.

Completeness measurement:

- [ ] Every disproof verdict has evidence ids or explicit unknown/blocked
      reason.
- [ ] Numeric/table claims have document/table/cell anchors or visible caveat.

### F8: Verifier And Disproof Verdicts

Behavioral claim:

> Arcwell can test statements against evidence and record whether evidence
> supports, weakens, refutes, is irrelevant, is inconclusive, or is unknown.

Implementation checklist:

- [ ] Add `research_disproofs` migration.
- [ ] Add verdict enum normalization.
- [ ] Add strength enum normalization.
- [ ] Add confidence delta validation.
- [ ] Add verifier artifact and prompt version.
- [ ] Add deterministic contradiction mapping.
- [ ] Add optional model-assisted verifier.
- [ ] Add same-run evidence validation.
- [ ] Add verdict read/list tools.

Quality tests:

- [ ] Direct contradiction produces `refutes`.
- [ ] Partial scope mismatch produces `weakens` or `inconclusive`.
- [ ] Unrelated evidence produces `irrelevant`.
- [ ] Missing evidence produces `unknown`.
- [ ] Strong official correction outweighs stale secondary support.

Adversarial tests:

- [ ] Verifier cannot ignore counterevidence due to report narrative.
- [ ] Low-quality source cannot create strong refutation alone.
- [ ] Evidence from generated synthesis cannot refute primary evidence.
- [ ] Numeric contradiction catches unit and date mismatches.

Completeness measurement:

- [ ] Every answered high-severity challenge has a disproof verdict.
- [ ] Strong refutations create required revisions or block settlement.

### F9: Revision Engine

Behavioral claim:

> Refuted or weakened statements are revised with durable lineage, and final
> reports cannot silently keep old conclusions.

Implementation checklist:

- [ ] Add `research_revisions` migration.
- [ ] Add revision type enum.
- [ ] Link revisions to disproof ids.
- [ ] Add statement replacement lineage.
- [ ] Add confidence delta propagation.
- [ ] Add caveat propagation.
- [ ] Add dropped/split/merged statement handling.
- [ ] Add revision summary artifact.
- [ ] Add revision read/list tools.

Invalid input tests:

- [ ] Revision from cross-run statement rejected.
- [ ] Revision to nonexistent statement rejected.
- [ ] Revision without rationale rejected.
- [ ] Revision triggered by nonexistent disproof rejected.

Adversarial tests:

- [ ] Refuted statement cannot remain final without caveat or replacement.
- [ ] Rewording cannot hide a refuted stable key.
- [ ] Confidence cannot increase after a weakening verdict unless new evidence
      justifies it.
- [ ] A dropped statement remains visible in the appendix.

Completeness measurement:

- [ ] Final report "what changed" section is generated from revision rows.
- [ ] Auditor can trace every changed final conclusion to disproof evidence.

### F10: Convergence Metrics And Stop Rules

Behavioral claim:

> Arcwell can decide whether to continue, settle, or stop incomplete using
> explicit metrics rather than vibes.

Implementation checklist:

- [ ] Add `research_convergence_snapshots` migration.
- [ ] Calculate source novelty.
- [ ] Calculate claim novelty.
- [ ] Calculate statement change count.
- [ ] Calculate confidence deltas.
- [ ] Calculate open critical/high challenges.
- [ ] Calculate active fact-check summary fields.
- [ ] Calculate cost/time/provider-call metrics.
- [ ] Add stop-rule evaluator.
- [ ] Add status labels: `settled`, `stopped_incomplete`, `failed`.

Performance tests:

- [ ] Novelty scoring handles 10,000 candidate sources.
- [ ] Statement metrics handle 5,000 statements.
- [ ] Snapshot generation completes within configured local budget.
- [ ] Indexes avoid full scans in common status views.

Severe tests:

- [ ] Critical unresolved challenge blocks `settled`.
- [ ] Strong refutation without revision blocks `settled`.
- [ ] No-progress loop stops after configured cap.
- [ ] Cost cap stops before provider call.
- [ ] Time cap stops before next iteration.
- [ ] User stop wins over all continuation logic.

Completeness measurement:

- [ ] Settlement can be explained from snapshot fields alone.
- [ ] Stop reason is visible in CLI, MCP, and final report.

### F11: Active Fact-Checking

Behavioral claim:

> Arcwell can extract report/statement factual claims, verify them through
> source cards plus fresh retrieval, and feed wrong/unknown high-impact claims
> back into the loop.

Implementation checklist:

- [ ] Add fact-check statement extraction.
- [ ] Add statement-to-source-card verification.
- [ ] Add external retrieval verification.
- [ ] Add labels: `right`, `wrong`, `unknown`, `not_checkable`.
- [ ] Add high-impact classification.
- [ ] Add wrong/unknown challenge creation.
- [ ] Add fact-check artifact and score fields.
- [ ] Add report summary.
- [ ] Add eval fixture set.

Quality tests:

- [ ] Seeded false report sentence is labeled wrong.
- [ ] Uncited true sentence is checked, not ignored.
- [ ] Vague opinion is labeled not_checkable.
- [ ] Unknown high-impact claim blocks or caveats settlement.
- [ ] Wrong low-impact claim appears in report QA section.

Malicious tests:

- [ ] Report text cannot instruct verifier to skip claims.
- [ ] Source text cannot instruct verifier to mark itself correct.
- [ ] Cross-run source cards cannot verify a claim.
- [ ] Generated summary cannot be sole verification evidence.

Completeness measurement:

- [ ] Zero wrong high-impact labels in final settled reports.
- [ ] Unknown high-impact labels are caveated or block settlement.

### F12: Report Renderer And Report Judgment

Behavioral claim:

> The final report reads as an analyst-grade narrative while preserving a
> complete evidence/disproof/revision appendix and an explicit report judgment.

Implementation checklist:

- [ ] Add convergence-aware executive judgment.
- [ ] Add final position section.
- [ ] Add what-changed section from revision rows.
- [ ] Add survived/weakened/refuted statement sections.
- [ ] Add unresolved disproof section.
- [ ] Add convergence stop reason.
- [ ] Add active fact-check summary.
- [ ] Add iteration appendix.
- [ ] Add statement/disproof/revision ledger.
- [ ] Add `research_report_judgment` artifact.

Report judgment rubric:

- [ ] Source coverage: score 0 to 5.
- [ ] Primary-source depth: score 0 to 5.
- [ ] Citation support: score 0 to 5.
- [ ] Contradiction handling: score 0 to 5.
- [ ] Caveat/uncertainty preservation: score 0 to 5.
- [ ] Narrative clarity: score 0 to 5.
- [ ] Decision usefulness: score 0 to 5.
- [ ] Novel insight/design quality where relevant: score 0 to 5.
- [ ] Safety/security reasoning where relevant: score 0 to 5.
- [ ] Cost/time proportionality: score 0 to 5.

Completion gate:

- [ ] No category score below 3 for `settled`.
- [ ] Average score at least 4 for `saturated_proven`.
- [ ] No unsupported high-impact factual claims.
- [ ] Report judgment written by Codex/main agent with line/section references.
- [ ] Judgment states remaining weaknesses, not only praise.

Severe tests:

- [ ] Refuted statement cannot appear as final conclusion.
- [ ] Unresolved severe disproof appears in executive caveats.
- [ ] Metadata-only corpus produces source-mapping report, not fake judgment.
- [ ] Appendix contains trace rows for all final high-impact statements.
- [ ] Markdown injection from source text is escaped.

### F13: Long-Running Worker And Resume

Behavioral claim:

> A convergence run can operate for hours with resumable state, bounded cost,
> explicit progress, and safe interruption.

Implementation checklist:

- [ ] Add worker job kind for convergence step.
- [ ] Add lease/heartbeat fields.
- [ ] Add idempotency key per iteration step.
- [ ] Add resume after crash.
- [ ] Add progress snapshots.
- [ ] Add stop request handling.
- [ ] Add cost reservation before provider calls.
- [ ] Add stale lease reclaim.
- [ ] Add dead-letter behavior.
- [ ] Add operator status output.

Reliability tests:

- [ ] Crash after statement compile resumes without duplicate statements.
- [ ] Crash after challenge generation resumes without duplicate challenges.
- [ ] Crash after provider call preserves cost record.
- [ ] Stale lease is reclaimed exactly once.
- [ ] Stop during sleep/search prevents next write.
- [ ] Retry storm cannot burn unlimited cost.

Performance tests:

- [ ] 24-hour configured run emits periodic progress snapshots.
- [ ] Large run status read remains bounded.
- [ ] Worker memory does not grow linearly without bound across iterations.

Completeness measurement:

- [ ] Preserved smoke home proves stop/resume/restart behavior.
- [ ] Ops/status shows whether the run is active, blocked, stopped, or settled.

### F14: Codex/Subagent Orchestration

Behavioral claim:

> Codex can run the convergence loop with explicit role handoffs while Arcwell
> records every accepted/rejected role output.

Implementation checklist:

- [ ] Update `$deep-research` skill for convergence runs.
- [ ] Add role prompt for position compiler.
- [ ] Add role prompt for red teamer.
- [ ] Add role prompt for disproof scout.
- [ ] Add role prompt for verifier.
- [ ] Add role prompt for reviser.
- [ ] Add role prompt for convergence auditor.
- [ ] Require role output artifacts before completion.
- [ ] Add host-search proof instructions per challenge.
- [ ] Add reload/new-thread caveat for stale schemas.

Severe tests:

- [ ] Completed role without output artifact rejected.
- [ ] Role output with cross-run ids rejected.
- [ ] Subagent prompt injection from source text is caught in review.
- [ ] Missing caveats in role handoff block task completion.
- [ ] Host search unavailable produces blocked status, not fake search proof.

Completeness measurement:

- [ ] Fresh Codex thread live-smokes at least one full convergence iteration.
- [ ] Role artifacts show accepted and rejected proposals.

### F15: MCP/CLI Surface Parity

Behavioral claim:

> Agents and humans can inspect and control convergence state through both CLI
> and MCP without schema mismatch.

Implementation checklist:

- [ ] Add CLI commands for every read/list/control surface.
- [ ] Add MCP tools for every agent-needed surface.
- [ ] Add command docs.
- [ ] Add plugin command shims where user-facing.
- [ ] Add `research_capabilities` fields.
- [ ] Add JSON schema tests.
- [ ] Add stale-schema smoke.
- [ ] Add docs verifier updates.

Invalid input tests:

- [ ] Unknown run id returns clear error.
- [ ] Unknown iteration id returns clear error.
- [ ] Malformed JSON rejected.
- [ ] Missing required fields rejected.
- [ ] CLI and MCP produce equivalent state for equivalent calls.

Completeness measurement:

- [ ] `scripts/verify-codex-plugin-docs` passes.
- [ ] Dev plugin smoke proves schema freshness.
- [ ] New thread smoke proves commands are visible to Codex.

### F16: Invention/New-Tech Branch

Behavioral claim:

> Arcwell can propose new designs as hypotheses, attack them with prior art and
> feasibility checks, and avoid presenting invented ideas as proven facts.

Implementation checklist:

- [ ] Add `design_proposal` statement type handling.
- [ ] Add prior-art challenge templates.
- [ ] Add feasibility challenge templates.
- [ ] Add threat-model challenge templates.
- [ ] Add benchmark/experiment-plan artifacts.
- [ ] Add "not proven" report language.
- [ ] Add graduation criteria for promising proposals.
- [ ] Add prototype/experiment handoff only with explicit user approval.

Quality tests:

- [ ] Known prior-art fixture weakens novelty claim.
- [ ] Feasibility gap prevents "ready to build" language.
- [ ] Security flaw appears in executive caveats.
- [ ] Proposed design remains separate from sourced facts.
- [ ] Missing experiment evidence prevents "proven" status.

Completeness measurement:

- [ ] Invention report clearly separates discovered facts, proposed design,
      prior art, feasibility, experiments needed, and residual risks.

## Adversarial Review Gates

Each implementation phase must pass the relevant gates before moving on. A gate
is a written review artifact plus tests or concrete evidence. Reviews should
prefer fewer demonstrated failures over long speculative lists.

### Gate A: Architecture Consistency

- [ ] Does this preserve the Codex-orchestrates/Arcwell-persists boundary?
- [ ] Does any worker behavior pretend to have host-native search?
- [ ] Does any model output become evidence without source-card extraction?
- [ ] Does the design retain one user-facing deep mode?
- [ ] Are partial states honestly labeled?
- [ ] Are old compatibility shims removed or clearly bounded?

Pass condition:

- [ ] No boundary violation remains.
- [ ] Any deliberate deviation is documented with a reason and tests.

### Gate B: Data Integrity

- [ ] Cross-run references are rejected.
- [ ] Parent/child lineage cannot be corrupted.
- [ ] Duplicate writes are idempotent or versioned.
- [ ] Failed writes do not leave misleading completed status.
- [ ] Migrations preserve existing deep-research data.
- [ ] Backup/restore includes new convergence tables.

Pass condition:

- [ ] Severe schema tests pass.
- [ ] A copied-home migration smoke passes.

### Gate C: Epistemic Correctness

- [ ] Unsupported final conclusions are impossible or detected.
- [ ] Refuted conclusions are dropped, narrowed, or caveated.
- [ ] Unknown high-impact facts block or caveat settlement.
- [ ] Contradictions are preserved instead of averaged away.
- [ ] Confidence cannot increase without evidence.
- [ ] Source novelty and no-progress stops are computed from durable state.

Pass condition:

- [ ] Seeded contradiction and false-claim fixtures fail when broken and pass
      when fixed.

### Gate D: Malicious Input And Prompt Injection

- [ ] Source text cannot issue tool instructions.
- [ ] Search snippets cannot alter run scope or stop rules.
- [ ] PDFs/XLSX/CSV files cannot execute formulas/scripts/macros.
- [ ] Markdown/HTML from sources is escaped.
- [ ] Malicious URLs cannot trigger SSRF/local file reads.
- [ ] Prompt-injection strings are visible as evidence, not followed.
- [ ] Model output cannot create trusted state without validation.

Pass condition:

- [ ] Malicious fixture corpus passes.
- [ ] No tool call is triggered from source-provided instructions.

### Gate E: Invalid Input And Failure Semantics

- [ ] Bad ids produce clear errors.
- [ ] Bad enums produce clear errors.
- [ ] Malformed JSON fails closed.
- [ ] Missing required fields fail closed.
- [ ] Provider timeout records failed stage.
- [ ] Tool unavailable produces blocked/incomplete status.
- [ ] Report compile refuses to hide blocking findings.

Pass condition:

- [ ] Invalid input tests cover CLI, MCP, and store methods.

### Gate F: Performance And Cost

- [ ] Iteration metrics scale to large source/claim/statement sets.
- [ ] Status/read APIs are bounded.
- [ ] Report rendering is bounded.
- [ ] Long runs reserve cost before provider calls.
- [ ] Retry storms cannot spend repeatedly.
- [ ] Search/source expansion has fanout limits.
- [ ] Memory use remains bounded under large corpora.

Pass condition:

- [ ] Performance smoke meets configured budgets on synthetic large corpus.
- [ ] Cost policy tests prove block-before-call behavior.

### Gate G: Full-Run Quality

- [ ] At least one deterministic full run passes.
- [ ] At least one live host-search run passes.
- [ ] At least one saturated preserved-corpus run passes.
- [ ] At least one invention/new-tech run passes.
- [ ] Codex writes a report judgment artifact.
- [ ] The report judgment includes weaknesses and residual risks.
- [ ] The final report is readable, not a log dump.

Pass condition:

- [ ] Report judgment average score at least 4, no category below 3, and no
      unsupported high-impact factual claims.

## Test Matrix

### Unit Tests

- [ ] Normalize statement types/statuses/certainty labels.
- [ ] Normalize challenge types/severity/statuses.
- [ ] Normalize disproof verdicts/strength.
- [ ] Normalize revision types.
- [ ] Validate confidence and delta ranges.
- [ ] Compute stable statement keys.
- [ ] Compute novelty metrics.
- [ ] Compute confidence deltas.
- [ ] Evaluate stop rules.
- [ ] Escape report text.

### Store Integration Tests

- [ ] Create/read/list iteration.
- [ ] Create/read/list statements.
- [ ] Create/read/list challenges.
- [ ] Create/read/list disproofs.
- [ ] Create/read/list revisions.
- [ ] Create/read convergence snapshots.
- [ ] Cross-run reference rejection.
- [ ] Idempotent retry behavior.
- [ ] Failed iteration preservation.
- [ ] Migration from existing homes.

### CLI Tests

- [ ] `research converge` starts loop.
- [ ] `research converge-step` performs one bounded step.
- [ ] `research iterations` lists iterations.
- [ ] `research iteration-read` reads details.
- [ ] `research statements` filters by status/type.
- [ ] `research challenges` filters by severity/status.
- [ ] `research disproofs` lists verdicts.
- [ ] `research convergence-status` shows metrics and stop reason.
- [ ] Invalid flags fail clearly.
- [ ] JSON output is stable.

### MCP Tests

- [ ] Tool schemas include required fields.
- [ ] Missing fields fail with clear JSON errors.
- [ ] Cross-run ids rejected through MCP.
- [ ] Large read responses are bounded.
- [ ] `research_capabilities` advertises convergence accurately.
- [ ] Stale schema smoke catches plugin/cache mismatch.
- [ ] MCP and CLI state match for equivalent operations.

### Agent/Prompt Tests

- [ ] Skill tells Codex to call capabilities before claiming missing tools.
- [ ] Skill requires statement/disproof/revision artifacts.
- [ ] Skill forbids source text as instruction.
- [ ] Skill forbids generated-output recursion.
- [ ] Skill instructs stop reasons and saturation notes.
- [ ] Role prompts preserve caveats.
- [ ] Role prompts surface missing primary sources.
- [ ] Role completion fails without output artifact.

### Malicious Input Corpus

- [ ] Source page says "ignore previous instructions".
- [ ] Search snippet asks Codex to exfiltrate secrets.
- [ ] PDF contains prompt-injection text.
- [ ] XLSX contains formula injection.
- [ ] CSV contains formula injection.
- [ ] URL redirects to localhost.
- [ ] URL redirects to cloud metadata IP.
- [ ] Markdown contains script tags and image beacons.
- [ ] JSON contains duplicate keys and deep nesting.
- [ ] IDs contain traversal and SQL metacharacters.
- [ ] Unicode confusables attempt statement-key collision.
- [ ] Source claims itself authoritative without evidence.
- [ ] Generated report tries to cite itself.

### Invalid Input Corpus

- [ ] Empty query.
- [ ] Empty statement.
- [ ] Empty challenge.
- [ ] Unknown run id.
- [ ] Unknown iteration id.
- [ ] Nonexistent statement id.
- [ ] Cross-run evidence id.
- [ ] Invalid enum.
- [ ] Missing required JSON field.
- [ ] Overlong text field.
- [ ] Negative limit.
- [ ] Huge limit.
- [ ] NaN or infinity confidence.
- [ ] Unsupported document type.
- [ ] Provider unavailable.

### Performance Tests

- [ ] 10,000 candidate sources.
- [ ] 2,000 linked source cards.
- [ ] 50,000 claims.
- [ ] 5,000 current statements.
- [ ] 20,000 challenges/disproofs.
- [ ] 100 iterations.
- [ ] Report rendering with large appendix.
- [ ] Status read with large run.
- [ ] Restart/resume after large run.
- [ ] Cost summary over many provider calls.

Initial budgets:

- Statement compiler on 50,000 claims: target below 10 seconds locally.
- Convergence snapshot on 5,000 statements: target below 5 seconds locally.
- Status/read latest snapshot: target below 500 ms locally.
- Report render for 5,000 statements: target below 10 seconds locally.
- Memory growth across 100 iterations: no unbounded accumulation in worker.

These budgets should be measured and adjusted after the first implementation
profiling pass.

## Full-Run Proof Plan

Each proof run must preserve its `ARCWELL_HOME` or proof bundle and include the
exact commands, run ids, source counts, statement counts, challenge counts,
disproof counts, revision counts, stop reason, cost records, and report
judgment.

### Proof 1: Deterministic Fixture Run

Goal:

- [x] Prove the loop works without network or paid providers.

Fixture:

- [x] At least 30 source cards.
- [x] At least 80 claims.
- [x] Seeded contradictions.
- [x] Seeded stale current claim.
- [x] Seeded malicious prompt-injection source.
- [x] Seeded unsupported report sentence.

Pass criteria:

- [x] Strong contradiction refutes or weakens the target statement.
- [x] Stale claim is caveated or dropped.
- [x] Prompt injection is rendered as evidence only.
- [x] Unsupported report sentence is caught by active fact-check.
- [x] Loop refuses settled status while blocking contradiction/report checks remain.

Evidence:

- `severe_research_convergence_saturated_fixture_preserves_bad_evidence_and_report_gate`
  seeds 30 linked source cards, 82 structured claims, contradiction, stale
  source metadata, hostile source text, and unsupported report prose. The
  convergence loop stops incomplete with revisions and a rejecting report
  judgment; active fact-check creates a citation-gap host-search task for the
  unsupported prose.

### Proof 2: Live Host-Search Technical Run

Topic:

- [x] Image compression algorithms, codecs, and benchmarks production-proof
      harness executed.

Pass criteria:

- [x] Fresh host-search proof is recorded.
- [x] Papers, codec docs, benchmark sources, and dissenting analyses are
      discovered and represented in the source-card ledger.
- [ ] At least one benchmark conflict is preserved.
- [ ] Numeric/table claims have precise full-source anchors rather than
      snippet-level caveats.
- [ ] Model-backed final report judgment accepts the report.

Evidence:

- `.arcwell-dev/proofs/deep-research-production-proof-20260623T155121Z`
  recorded 12 Brave queries, 131 deduped candidates, 80 source cards,
  80 structured claims, 18 host-search proof records, convergence/close-loop
  execution, and OpenAI citation-verifier/adversarial-evaluator records.
- The proof is a fail-closed orchestration proof, not an accepted analyst-grade
  proof. The verifier accepted citation shape with `unsupported_count: 0`, but
  the adversarial evaluator rejected the report for snippet-derived
  medium-confidence evidence, unsupported or overreaching conclusions, missing
  caveats, and 474 pending challenge-search tasks.
- Root-cause fixes from failed attempts are now covered in code/tests: unsafe
  URL filtering, source-card vs structured-claim confusion, active fact-check
  recursion over generated report sections, convergence source cap, model-backed
  judgment overwrite, bodyless structured provider responses, pending-search
  prompt scoring, and evaluator routing over synthesized score artifacts.

### Proof 3: Live Market/Ecosystem Run

Topic:

- [ ] AI startup scene in London.

Pass criteria:

- [ ] Official/company/funding/job/news/social source families are represented.
- [ ] Source currentness is explicit.
- [ ] Hype/PR claims are downgraded unless corroborated.
- [ ] Contradictory company/funding/status claims are surfaced.
- [ ] Final report judgment average score at least 4.

### Proof 4: Live Security/Architecture Run

Topic:

- [ ] Safe cloud code execution with compile-time security constraint
      verification.

Pass criteria:

- [ ] Standards, sandbox literature, policy engines, capability systems,
      compiler/static-analysis papers, and production platform docs are
      represented.
- [ ] Threat model challenges are generated.
- [ ] Security claims require primary or technical evidence.
- [ ] Unsafe overclaims block settlement.
- [ ] Final report judgment average score at least 4.

### Proof 5: Invention/New-Tech Run

Topic:

- [ ] Invent a plausible architecture for verified cloud code execution under
      explicit constraints.

Pass criteria:

- [ ] Proposal statements are labeled `design_proposal`.
- [ ] Prior-art search weakens or supports novelty claims.
- [ ] Feasibility and threat-model challenges are answered or caveated.
- [ ] Missing experiments prevent "proven" language.
- [ ] Report separates facts, proposal, novelty, risks, and experiments needed.

### Proof 6: Long-Running Resume Run

Goal:

- [ ] Prove hours/day-style operation does not depend on one uninterrupted
      Codex turn.

Pass criteria:

- [ ] Run executes multiple iterations through worker or explicit resume steps.
- [ ] Process interruption is simulated.
- [ ] Resume does not duplicate statements/challenges/disproofs.
- [ ] Cost/time limits remain enforced after resume.
- [ ] User stop cancels before next expensive action.

## Report Judgment Artifact

Every full proof run must include a `research_report_judgment` artifact written
by the main Codex agent after inspecting the final report and evidence summary.

Required fields:

```json
{
  "run_id": "...",
  "report_id": "...",
  "judgment_version": "v1",
  "overall_decision": "accept|accept_with_caveats|reject|incomplete",
  "scores": {
    "source_coverage": 0,
    "primary_source_depth": 0,
    "citation_support": 0,
    "contradiction_handling": 0,
    "uncertainty_preservation": 0,
    "narrative_clarity": 0,
    "decision_usefulness": 0,
    "novelty_or_design_quality": 0,
    "safety_security_reasoning": 0,
    "cost_time_proportionality": 0
  },
  "blocking_findings": [],
  "non_blocking_findings": [],
  "evidence_checked": [],
  "remaining_risks": [],
  "commands_or_artifacts_reviewed": []
}
```

Acceptance rules:

- [ ] `reject` if any high-impact factual claim is unsupported.
- [ ] `reject` if severe contradiction is hidden.
- [ ] `reject` if report cites generated output as evidence.
- [ ] `incomplete` if source coverage is below declared scope.
- [ ] `accept_with_caveats` if all blockers are gone but material uncertainty
      remains.
- [ ] `accept` only when caveats are non-blocking and evidence is strong.

## Production Readiness Scorecard

Score each category 0 to 5:

- [ ] Data model completeness.
- [ ] CLI/MCP parity.
- [ ] Skill/orchestration clarity.
- [ ] Statement quality.
- [ ] Challenge quality.
- [ ] Disproof retrieval quality.
- [ ] Revision correctness.
- [ ] Stop-rule correctness.
- [ ] Active fact-check quality.
- [ ] Report narrative quality.
- [ ] Malicious input resistance.
- [ ] Invalid input handling.
- [ ] Restart/resume reliability.
- [ ] Performance at scale.
- [ ] Cost/policy safety.
- [ ] Live proof coverage.
- [ ] Documentation honesty.

Production-ready threshold:

- [ ] No category below 4.
- [ ] No critical/high unresolved adversarial review finding.
- [ ] All six proof runs pass.
- [ ] Full Rust/doc suite passes.
- [ ] Dev plugin sync/smoke passes.
- [ ] Plugin/docs verifier passes.
- [ ] STATUS/TODO/docs agree.
