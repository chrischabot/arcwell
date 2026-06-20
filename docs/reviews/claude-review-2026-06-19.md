# Review: Gap Analysis for the `arcwell-*` Family Plan

This is a strong, unusually thorough document. Below are the findings I think actually matter, ordered by severity. I've skipped re-summarizing the parts that are already solid (see the "already strong" list at the end so you don't overbuild them).

---

## P0 — Critical gaps / risky assumptions

### 1. Data durability (backup + sync) is deferred to "later" but is actually core
**Why it matters:** Personal memory, profile, decision ledgers, and a ~9,000-page wiki are local-first SQLite with "optional encrypted edge backup/sync later" repeated throughout. For a daily-driver personal brain, a dead SSD or corrupted SQLite file means total loss of irreplaceable personal context. This is not a phase-11 nicety; it's a precondition for trusting the system with health facts, decision history, and years of imported conversation.
**Recommendation:** Promote backup to P0/P1. Define a minimum: automated, encrypted, versioned snapshots of every local store (memory/profile/wiki/decision/controller) to R2 or another object store, with restore + integrity-check tooling and a `doctor` check that fails loudly if the last successful backup is stale. Make it a shared concern in `arcwell-local`, not per-app.
**Where:** New "Durability / Backup / Restore" subsection under *Security And Reliability Invariants*; add to `arcwell-local`/`arcwell-runtime`.

### 2. Availability when the laptop is offline undermines the mobile/Telegram promise
**Why it matters:** The "brain" (controller + Codex bridge) is local. Cloudflare buffers *inbound* Telegram/email, but when the laptop is asleep/closed/away, the user cannot actually get a Codex thread resumed, an answer produced, or a proactive digest generated. The product narrative is "daily work shell + proactive monitoring + Telegram reachability from mobile," which implicitly requires always-on. Right now you get an always-on *inbox* but a frequently-offline *processor*.
**Recommendation:** Make the offline behavior an explicit product decision, not an accident. Options to spell out: (a) a "laptop wake/keep-alive" expectation documented as a constraint; (b) a small edge-resident responder for read-only/status/memory-search queries that don't need local Codex; (c) queued-and-deferred semantics with clear user-facing "I'll answer when your laptop is back online" receipts and SLAs. Pick one per capability and document it.
**Where:** *Deployment Toolbox* + *Edge Inbox / Local Drain Pattern* + `arcwell-controller` (offline routing/notification behavior).

### 3. No system-wide cost ceiling for always-on model usage
**Why it matters:** Only `arcwell-x` has a spend cap. But twice-daily monitoring runs a two-stage interestingness classifier across many sources, page-expansion pulls multiple sources per launch, deep research fans out 3–5 subagents, and the "effort router" *auto-escalates* to higher reasoning on high-stakes tasks. That's a recurring, unbounded, mostly-invisible spend that grows with the number of watched sources. A misconfigured watch list or a runaway dream/reconcile loop could quietly cost a lot.
**Recommendation:** Add a cross-service budget/cost ledger as a first-class concern: per-job token/cost recording, per-source and per-package daily/monthly ceilings, a global kill-switch, and cost surfaced in `arcwell-ops-ui` and `ops_health()`. Tie the effort router to a budget guardrail so auto-escalation can't bypass caps. Make "estimated cost before run" available for ingestion/expansion/research jobs.
**Where:** New "Cost / Budget Policy" section near *Model Policy*; add `ops_costs_*` tools to the *MCP status surface*; reference in `arcwell-quality-kit`'s effort router.

### 4. The entire controller/channel layer rests on Codex app-server/SDK stability — unverified
**Why it matters:** `arcwell-controller`, `arcwell-telegram`, and the whole "Codex as the turn-runner" model depend on app-server thread APIs (start/resume/fork/list) being documented, stable, and reachable from a non-JS client. The doc itself hedges ("if official SDK support is JS-only, implement the narrow app-server HTTP/SSE client in Rust or shell out as a temporary adapter"). If that protocol is unstable or undocumented, Phase 6 and the flagship "how's the de-porting going?" demo are at risk.
**Recommendation:** Move a thin app-server/SDK integration spike to Phase 0/1 as a go/no-go. Prove: start thread, stream `turn/start` events, resume, list — from Rust (or via a documented adapter). Define a fallback contract if only JS is viable (e.g., a tiny Node sidecar exposing a stable local HTTP/MCP shim) so the rest of the stack doesn't depend on Rust-native support.
**Where:** *Suggested Build Order* Phase 0; add to *Assumptions* as an explicit risk; `arcwell-controller` implementation notes.

### 5. Prompt-injection path through wiki → content → auto-publish is not closed
**Why it matters:** Channel and search-snippet injection are well fenced. But the wiki ingests untrusted web/X/newsletter content, `arcwell-content` drafts from wiki pages, and `arcwell-content` allows "auto-publish for preauthorized low-risk channels." That is a clean exfiltration/abuse chain: malicious page → wiki claim → content draft → auto-posted publicly, or instructions in a source steering a draft. The injection controls are scoped to inbound channels and search, not the ingestion→synthesis→publish pipeline.
**Recommendation:** Treat wiki content as untrusted *all the way to publish*. Require human review for any public post derived from externally-ingested sources (no auto-publish of source-derived content, even "low-risk"). Add provenance gating in `arcwell-content` (which sources fed a draft) and an injection/exfil scan before publish. Keep auto-publish only for content with no untrusted-source lineage.
**Where:** *Conversation Import Privacy Model* sibling section on *Ingestion→Synthesis→Publish trust*; `arcwell-content` publish policy; *Security And Reliability Invariants*.

---

## P1 — High importance

### 6. MCP tool sprawl will degrade agent tool selection
**Why it matters:** The plan defines well over 100 MCP tools across ~15 servers (controller, ops, memory lifecycle, wiki watch/librarian/expand, import, profile, search, channels, etc.). Codex and Claude both degrade in tool-selection quality and burn context as tool counts climb, and have practical limits on connected servers. Naming collisions (`*_status`, `*_list`) across servers also confuse routing.
**Recommendation:** Add a "tool surface budget" design rule: expose a *small* high-level tool set to the agent by default and keep the long tail behind progressive disclosure (skills that call detailed tools, or a single dispatcher tool per domain). Reserve namespaces, define which servers are loaded in which host profile, and measure context cost. Don't enable all 15 servers in one Codex/Claude session.
**Where:** *Current Regular Codex Extension Surfaces* + a new "Tool Surface / Context Budget" note in *UI Strategy* or *Open-Source Package Shape*.

### 7. Claude Desktop parity is overpromised
**Why it matters:** "Claude Desktop as first-class" recurs, but Claude Desktop's extension model is materially weaker than Codex's: no skills/automations/hooks equivalent, and remote MCP needs custom connectors with OAuth/DCR (the reason garderobe is on Cloudflare). Pre-turn recall hooks, post-turn capture, `channel_capture`, and dream scheduling cannot run inside Claude Desktop the way they do in Codex. So "auto-capture personal memory from Claude conversations" is largely not achievable without Claude exposing those surfaces.
**Recommendation:** Publish an explicit Claude-vs-Codex capability matrix per package: what works (MCP tools/resources, remote connectors), what degrades (no auto recall/capture, manual tool calls only), and what's unavailable (hooks, skills, automations). Set expectations that Claude Desktop is a *read/query and manual-capture* client, not a lifecycle host. This avoids designing flows that silently no-op there.
**Where:** `hosts/claude/` docs convention; *Assumptions And Success Criteria* (qualify the portability claim); each package's docs.

### 8. Credential/OAuth lifecycle across many providers has no owner
**Why it matters:** Secrets are mentioned (keychain/0600), but the system accumulates OAuth grants and tokens for X, Google (multiple scopes), Telegram bot token, OpenAI, Brave, Perplexity, plus Cloudflare-held edge secrets — each with refresh, expiry, rotation, and revocation. There's no central credential lifecycle, no expiry alerting, and tokens are split across local + edge. Silent token expiry is the most common way always-on monitors die.
**Recommendation:** Define a shared credential service/contract (`arcwell-secrets` or part of `arcwell-local`): unified storage abstraction (keychain/file/edge), refresh scheduling, expiry/health surfaced in `doctor` and `ops_health`, and channel notification on grant failure. Document where each token lives and why.
**Where:** New `arcwell-secrets` (or `arcwell-local` responsibility); *Security And Reliability Invariants*; *doctor* conventions.

### 9. Sensitive/health data handling is labeled but not protected
**Why it matters:** The system intentionally stores medications, ADHD context, health-adjacent constraints, sizes, and rejection-sensitivity signals. There are sensitivity *labels* and review queues, but no stated encryption-at-rest, no policy on which model providers sensitive data may be sent to, no edge-sync restriction for health data, and no data-residency stance. For real personal use (and for open-sourcing), this is a meaningful privacy/liability gap.
**Recommendation:** Add a sensitive-data tier: encrypted at rest, never synced to edge unless separately opted-in and client-side encrypted, never included in source-card/wiki cross-links, and explicit rules on which extraction/model calls may see it. Default health/medical to local-only + encrypted + review-required.
**Where:** *Conversation Import Privacy Model* + *Security And Reliability Invariants* + `arcwell-profile`/`arcwell-memory` storage notes.

---

## P2 — Medium importance

### 10. Silent daemon death has no detection/alerting
**Why it matters:** `doctor` is on-demand. If `xmonitord`, `telegramd`, or the wiki watcher crashes, nothing tells the user; proactive monitoring just stops. For an "agent that respects your time," silent failure of the proactive layer is exactly the under-effort failure the doc warns about.
**Recommendation:** Add heartbeat/liveness with self-alerting: each daemon writes liveness, a lightweight watchdog (local + optional edge mirror) notices missed heartbeats and notifies via the channel layer. Surface in `ops_health`. This is the "the monitor itself must be monitored" invariant.
**Where:** `arcwell-ops-ui` / *MCP status surface*; *Security And Reliability Invariants*.

### 11. No migration tooling for forced embedding/model deprecation
**Why it matters:** Hosted embeddings/models get sunset. "Don't mix dimensions in a corpus" is stated, but a 9k-page corpus *will* eventually need re-embedding when a provider deprecates a model. Without a planned, resumable, cost-aware re-embed/migration path, a deprecation becomes an emergency.
**Recommendation:** Add a corpus migration tool (`wiki_reembed`/migration job): records provider+dimensions per corpus, supports background re-embedding with cost estimate and resumability, and a compatibility check at startup. Note this as expected maintenance, not an edge case.
**Where:** *Model Policy* + `arcwell-llm-wiki` Phase 4.

### 12. Memory growth, decay, and reinforcement feedback loop
**Why it matters:** Dream/reconcile auto-applies in trusted scopes and capture is default-on. Over months this can bloat, drift, and — because recall feeds conversations that then feed capture — *reinforce* earlier wrong facts into seeming consensus. There's no retention/decay/confidence-aging policy and no guard against self-reinforcement.
**Recommendation:** Add memory retention/decay and confidence aging; mark facts whose only provenance is prior recall (not fresh user statements) so the loop doesn't self-confirm; periodic "stale preference" review in the dream digest.
**Where:** `arcwell-memory` lifecycle model; *Decisions*.

### 13. Schema/envelope versioning across edge↔local is unaddressed
**Why it matters:** Edge Workers enqueue envelopes/events; local drains consume them. Independent deploy cadence (TS edge vs Rust local) guarantees version skew. Memory/wiki/profile schemas also evolve. There's no stated compatibility/migration discipline beyond "generate schemas from Rust types."
**Recommendation:** Version the envelope and queue payloads explicitly; require backward-compatible drains; define a migration policy and a schema-compat test in CI (edge producer vs local consumer). Add to `arcwell-envelope`/`arcwell-edge-protocol`.
**Where:** *Open-Source Package Shape* shared crates; *Edge Inbox / Local Drain Pattern*.

### 14. Deletion doesn't clearly cascade across stores + edge
**Why it matters:** "Memory is deletable/portable" is a success criterion, but a deleted memory may persist in: decision ledgers, import ledgers, wiki cross-links, edge backups, and queue history. Without cascade, "forget that" is partially false — a trust-breaking gap given the privacy framing.
**Recommendation:** Define delete semantics: tombstone propagation, cascade to cross-links/ledgers/backups, and a `*_delete` that reports everywhere the item existed. Test "right to be forgotten" end-to-end.
**Where:** *Conversation Import Privacy Model*; `arcwell-memory`/`arcwell-profile` delete tools; *Security And Reliability Invariants*.

### 15. Legal/ToS/copyright exposure for ingested third-party content
**Why it matters:** The wiki stores tweets, blogs, papers, newsletters, and search results — and this is meant to be open-sourced for others to run. X API terms, Perplexity/Brave result-storage restrictions, and newsletter/article copyright create compliance risk that the doc flags only as a one-line "provider terms may restrict storing raw results."
**Recommendation:** Add a short compliance posture: store metadata + cache validators + minimal quotes by default, separate raw-content storage behind explicit per-source policy/TTL, document the ToS constraints per provider, and ship conservative defaults for redistributed code.
**Where:** *Search Provider Policy* + *LLM Wiki Freshness* change-detection rules; a `docs/threat-model` + `docs/legal` convention.

---

## P3 — Lower, but worth noting

### 16. Install/onboarding complexity is itself an adoption risk
~15 packages, Rust builds, launchd/systemd, Cloudflare deploys, and OAuth setup for many providers is a steep cold start — for you and for open-source adopters. **Recommendation:** a top-level installer/doctor and a "minimal happy path" (memory + profile only) that works in one command before the full fleet. **Where:** *Suggested Build Order* Phase 0/11; `arcwell-runtime`.

### 17. Wardrobe inventory population (images/vision) is unspecified
`arcwell-garderobe` is a flagship, but how clothing inventory gets captured (photos → vision cataloging → R2) isn't covered. Without ingestion, the MCP is an empty database. **Recommendation:** specify a photo-ingestion + vision-cataloging flow and blob storage. **Where:** `arcwell-garderobe` Phase 9.

### 18. Public edge endpoints need explicit abuse/rate-limiting
Email Workers, webhooks, OAuth callbacks, and `/mcp` are public attack surface (spam, DoS, queue flooding, injection). OAuth is covered; rate-limiting/abuse controls are not. **Recommendation:** rate limits, payload caps (partly there), and abuse handling as standard edge helpers. **Where:** `arcwell-edge-protocol`; *Edge Inbox* rules.

### 19. Two memory systems (Codex built-in Memories + `arcwell-memory`) can conflict
Both can be enabled; the agent then has two overlapping memory surfaces, risking duplication/inconsistency. **Recommendation:** state a clear default (disable built-in Memories when `arcwell-memory` is active, or define non-overlapping roles) rather than "separate layers." **Where:** *Decisions* / `arcwell-memory` notes.

---

## Things that are already strong (don't overbuild these)

- **Memory-vs-wiki separation** (compact personal facts vs source-backed corpus, separate SQLite stores, distinct provenance) — clean and correct.
- **`arcwell-channel-kit` instead of a generic `arcwell-push`** — the inbound/outbound/identity/formatting/receipt contract is the right abstraction.
- **Edge-inbox / local-drain pattern** — bounded inbox, idempotency keys, cursor/lease/ack/dead-letter, TTL — well specified.
- **Approval tiers + friction policy** — the read_only/confirm_writes/trusted_owner_fast/dangerous_confirm model with durable, resumable approvals is thoughtful and matches the stated UX goal.
- **Conversation-import privacy model** — dry-run/redaction/candidate-review/import-ledger and "no raw transcripts into prompts" is exactly right.
- **Search provider policy** (host-native first, optional Brave/Perplexity adapters, normalized source cards) — portable and pragmatic.
- **`arcwell-controller`** design — registry + active-run monitor + NL resolver + follow-up context is the correct seam; don't expand it further until the app-server spike (Finding 4) lands.
- **Not porting the Swift runtime/supervisor/workflow engine** — correct call; resist the temptation to revisit.
- **Rust-first + SQLite-first + model-names-in-config** — good defaults; the explicit "avoid Postgres in first install path" and "drop MLX" decisions are well-reasoned.
- **UI strategy** (MCP-first, Obsidian for wiki, standalone ops UI, no dependence on undocumented Codex in-message rendering) — appropriately humble about host limits.

---

**Net:** The architecture and boundaries are sound. The biggest unaddressed risks are not in the design of any one package but in the **cross-cutting operational layer**: durability/backup, offline availability, cost control, credential lifecycle, and the unverified Codex app-server dependency. I'd pull backup (1), the app-server spike (4), cost ceilings (3), and the wiki→publish injection guard (5) forward into Phase 0/1, since they're cheaper to design in now than to retrofit once real personal data and always-on spend are flowing.
