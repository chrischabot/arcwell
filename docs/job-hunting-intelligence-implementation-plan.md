# Job Hunting Intelligence Implementation Plan

Date: 2026-06-28

Status: partially implemented. Schema v20, core APIs, CLI `arcwell job ...`,
and MCP tools now provide Local Proof for the durable/manual job-hunting
ledger: profiles, evidence cards, privacy checks, role cards, source
confidence, fit scores, packets, intro paths, applications, batch import,
configured source refresh, manual refresh reconciliation, weekly reports,
weekly-report delivery preparation, local ops visibility, and scheduled replay
job radar. A controlled-home
production-data proof now imports the
current resume, public GitHub metadata, and current blog/project evidence into
26 reviewed evidence cards and 26 claims, then passes the evidence-readiness
report. A second controlled-home production-data proof replays the bounded P1
Tier 1 shortlist into 7 durable role cards, 7 source-linked role-source links,
7 source-health rows, 7 fit scores, 7 skeptic findings, 7 draft packets, and a
7-entry Tier 1 shortlist with 7 passing packet privacy checks. A local-proof
scheduled job-radar worker path now enqueues `job_radar_refresh` from a
`job_radar` watch source, refreshes replay snapshots, writes source-health,
search-run, and weekly-report state, and records failed health when snapshots
are missing. No operational scheduled recurrence, exhaustive market coverage,
broad live freshness, user-approved/sent applications, or operational
application CRM is claimed. A browser-backed proof now renders a controlled
copy of the imported P1 data through `/ops/ui`,
including live-refresh source-health states, one controlled closed-role
transition, one blocked privacy check, one application follow-up, readable
mobile job tables, and desktop/mobile no-body-overflow checks. A London
startup/company map proof imports 30 current-source company cards with 20
high-fit developer-platform or AI-agent companies, 3 public founder/team
targets, 30 explicit public-only contact paths, and no warm-intro claim.
Local Proof now also compiles a company-target scouting report from durable
company cards and public evidence tags, while preserving the boundary that
company-target scouting reports are not current openings or application-ready
role cards by themselves. A
bounded company-target role-confirmation proof now live-refreshes five selected
London company/ATS pages from that map, writes ten same-day
canonical-confirmed role cards, rejects parser-noise role cards, and scores
seven confirmed roles against reviewed evidence. This proves only the bounded
role-confirmation slice, not exhaustive coverage, future freshness, packets,
sending, or outcomes. A
bounded source-family boundary proof now creates a disposable proof home,
refreshes representative fixtures, and live-refreshes the YC London software
engineering board. It proves the parser/scoring boundary that company/ATS pages
produce canonical-confirmed roles, VC/startup boards produce
secondary-confirmed leads, broad job boards produce aggregator-only leads, and
generic `/jobs/role/...` category navigation is rejected instead of entering
the role set. The latest packet wrote 26 live YC roles as
secondary-confirmed, 33 company cards, and 0 generic category role imports.
This is not exhaustive market coverage or an apply-now shortlist. A
second bounded direct-role expansion now live-refreshes four selected global
and European AI/devtools/platform role pages, writes four same-day
canonical-confirmed role cards with zero live parser-noise titles, and scores
Anthropic, Sierra, Tailscale, and Langfuse as Tier 1 against reviewed evidence.
This proves only those selected direct pages, not broad market coverage,
future freshness, packet export, sending, or outcomes. Rerunnable local proof
scripts now verify that scheduled radar can execute replay snapshots, that a
queued live `job_radar_refresh` denied before source work writes failed health
without source writes, that a policy-blocked job can recover on the same queued
job after policy is repaired, and that repeated policy denial dead-letters
instead of retrying forever. This remains Local Proof, not live provider
recovery or wall-clock recurrence. A bounded scheduled live-fetch proof now
copies the expanded direct-role proof home, schedules and immediately enqueues
`job_radar_refresh` with `fetch_live=true`, drains one worker job, live-refreshes
the four selected direct role sources with healthy source-health rows, writes a
production-data-proof search run, and writes a local weekly report. This is one
scheduled-worker live-fetch slice, not wall-clock recurrence or the one-day
audit gate. A preserved live-radar recurrence proof has started at
`.arcwell-dev/proofs/job-radar-live-recurrence-controlled-proof/artifacts/proof-packet.json`:
the first checkpoint was enqueued by the `job_radar` watch-source poller and
completed a `fetch_live=true` worker refresh over the same four direct sources,
but the packet is intentionally incomplete until a second watch-poll worker run
completes after `2026-06-29T12:43:19.550521+00:00` UTC. Outcome history is now
Local Proof at the note layer: recorded
application outcomes appear in shortlist and weekly report notes for related
roles without silently changing fit scores or tiers.
Controlled application pipeline state is now proven over copied production-data
input: a draft packet cannot back `applied`, approved packets can support
applied-like statuses, weekly reports count `planned`, `applied`,
`intro_requested`, `interview`, `rejected`, and `withdrawn`, shortlist entries
surface the corresponding warnings, public-only intro paths remain explicitly
not-warm, and weekly reports render intro status plus next actions. This is not
real sending, operational-home tracking, Google Docs draft creation, warm-intro
proof, or outcome learning.
Outreach readiness is now Controlled Local Proof: scored roles are blocked
until they have an approved packet, a fresh passing privacy recheck over the
outreach note, and a known or possible-mutual route. Public-only contact paths
stay blocked as identify/monitor work, and no channel messages or delivery
attempts are written. This is readiness classification only, not outreach
sending, Google Docs draft creation, user-network proof, or replies.
Weekly-report delivery is now Controlled Local Proof through the provider path:
authorized privacy-passing reports can write one prepared channel message,
send that prepared message through a loopback Cloudflare Email-compatible
provider, record exactly one successful `channel_delivery_attempt`, and replay
without a duplicate provider call. Unauthorized and privacy-blocked attempts
still write blocked rows without provider attempts. This is controlled
provider-path proof only, not live external email/Telegram delivery or
operational recurrence. A v19-to-v20 migration fixture also proves existing
weekly reports are preserved and the delivery ledger works after upgrade.
Approved packet export is now Local Proof as well: an approved, privacy-passing
packet can be written to local Markdown with an export-time privacy recheck and
explicit `not_sent`/not-applied boundaries.
Controlled P1 packet export is now proven over the seven imported Tier 1
packets: a copied proof home approves the packets for local export review only,
exports the approved set through `packet-export-set`, writes seven Markdown
files plus one JSON manifest, records seven export-time privacy checks, and
keeps application rows at zero before and after export.
Controlled company-target packet creation is now proven over the bounded live
role-confirmation input: a copied proof home creates five new draft packets for
the confirmed Tier 1 Ably/Northflank roles, records five passing packet privacy
checks, leaves those packets in `draft`, and keeps application rows at zero.
This is draft creation only, not approval, export, sending, submission, or
freshness beyond the source role proof.
Controlled expanded direct-role packet creation is now proven over the selected
global and European role-confirmation input: a copied proof home creates four
new draft packets for the confirmed Tier 1 Anthropic, Sierra, Tailscale, and
Langfuse roles, records four passing packet privacy checks, leaves those
packets in `draft`, and writes no new application rows. This is draft creation
only, not approval, export, sending, submission, or freshness beyond the source
role proof.
Controlled expanded direct-role packet export is now proven as the next
copied-home step: the four draft packets can be approved for local export
review only, written to four Markdown files, rechecked with four export-time
privacy checks, and still write no new application rows. This is local Markdown
export only, not user approval for submission, Google Docs draft creation,
delivery, or application status.
Controlled company-target packet export is now proven as the next copied-home
step: the five draft packets can be approved for local export review only,
written to five Markdown files, rechecked with five export-time privacy checks,
and still leave application rows at zero. This is local Markdown export only,
not user approval for submission, Google Docs draft creation, delivery, or
application status.
Refresh audit is now Local Proof for the gate logic: Arcwell can read durable
search-run and role-status rows, require two completed runs, require the
configured elapsed-time window, and name missing transition evidence without
claiming the one-day proof has happened. A preserved one-day proof harness now
exists at
`.arcwell-dev/proofs/job-refresh-one-day-controlled-proof/artifacts/proof-packet.json`;
its first checkpoint records source-count evidence plus `new`, `unchanged`,
`stale`, and `closed` transitions, but the packet is intentionally incomplete
until a second run happens after the real 24-hour window. It now records
`gate_state=waiting_wall_clock`,
`ready_at_utc=2026-06-30T04:06:11.893Z`, `hours_remaining`,
`second_started_at=null`, `hours_between_runs=null`, and the exact
`next_action` rerun command.

Skill gate: `arc:anti-mirage`.

Implemented scaffold and local-proof surfaces:

- `hosts/codex/skills/job-hunting/SKILL.md`
- `plugins/arcwell-codex/skills/job-hunting/SKILL.md`
- `docs/reports/2026-06-28-job-hunting-p1.md`
- `scripts/job-source-family-boundary-proof`
- `scripts/job-implementation-plan-audit-proof`
- `crates/arcwell-core/src/lib.rs` schema/API layer
- `crates/arcwell-cli/src/main.rs` CLI/MCP surface

The P1 report proved one bounded manual production-data pass. Later controlled
proofs covered partial live source refresh and browser-visible ops over a copy
of the imported P1 state. The London startup map proof covered a 30-company
current-source watch map, not exhaustive market coverage. The source-family
boundary proof covers parser confidence boundaries over fixtures plus one live
YC board, not current apply-now recommendations. The scheduled
job-radar proofs cover deterministic replay, missing-snapshot failure health,
pre-executor policy-denial health, retry recovery after policy repair,
dead-letter after retry exhaustion, and one bounded scheduled live-fetch worker
drain over four selected direct sources. They do not prove live wall-clock
recurrence. These proofs did not prove future freshness, operational scheduled
monitoring, a second wall-clock refresh, warm intros, or application outcome
conversion learning.

## Product Claim

Arcwell should help the user run a high-signal job search as an intelligence
workflow:

> Arcwell can maintain a private, source-backed job-hunting workspace that
> maps the user's evidence to live roles, ranks opportunities by fit, produces
> application packets for the best roles, tracks outcomes, and refreshes the
> market without leaking private project names or confusing stale postings for
> current opportunities.

This is not a mass-application bot. It is a decision-support and execution
system for targeted senior roles.

## Non-Claims

Arcwell must not claim:

- every relevant job has been found
- a role is current without a live company/ATS/source confirmation
- a role is Tier 1 without evidence mapping and skeptic review
- a resume/outreach packet is safe without privacy checks
- operational scheduled monitoring exists before worker jobs, source health,
  refresh reports, recurrence, and recovery are proven
- a role is likely to convert without outcome data
- warm-intro paths exist when only public names have been found

## User-Facing Result

For the user, the system should produce:

- a current apply-now shortlist
- role memos with source links and fit scores
- application packets for Tier 1 roles
- a warm-intro and outreach map
- a weekly job market report
- an application pipeline with outcomes and follow-up dates
- evidence gaps to fix in resume, GitHub, blog, or portfolio

The user-facing output should be concise and decision-oriented. The ledger
should remain available for inspection, but it should not be the main reading
surface.

## Anti-Mirage Boundary

This capability is not done because any one of these exists:

- a skill prompt says "job hunting"
- a role list was produced once
- a CSV or Markdown report has job links
- an LLM wrote fit notes
- a company name appears in a search result
- a source adapter scraped one page
- a scheduled worker ran without source health
- a weekly report exists but does not verify stale postings
- an outreach draft exists without privacy checks

The capability becomes operational only when current configured sources are
refreshed, source health can explain success and failure, role cards are
deduped and indexed, fit scores map to evidence cards, privacy checks run
before application material, user-approved application events are tracked, and
weekly reports distinguish new, stale, promoted, demoted, applied, replied,
rejected, and blocked roles.

## Proof Levels

- `Missing`: no implementation or only notes.
- `Scaffold`: skill text, docs, schemas, templates, or report examples exist.
- `Local Proof`: deterministic fixtures pass for normalization, dedupe,
  scoring, privacy checks, application-packet rendering, and weekly report
  generation.
- `Production Data Proof`: live roles from real company/ATS/startup sources
  are ingested or manually captured into durable role cards, scored against
  real evidence cards, and compiled into an auditable shortlist.
- `Operational`: production-data proof plus recurring refresh, source health,
  stale detection, application tracking, warm-intro status, weekly reporting,
  and outcome learning.
- `Done`: operational and no known core proof gate remains.

Do not collapse these levels. The current state is durable Local Proof, one
bounded manual `Production Data Proof` report, one controlled-home
production-data proof for the candidate evidence ledger, and one controlled P1
shortlist replay with scored roles and privacy-checked draft packets. The code
can now import reviewed packets, reconcile caller-supplied refresh observations,
and run policy-gated live refresh over configured sources with durable
partial/failure source-health rows. A browser-backed controlled proof renders
the imported state through `/ops/ui` with non-healthy source-health rows,
stale/closed role visibility, privacy-block visibility, follow-up visibility,
and readable desktop/mobile job tables. A controlled London startup/company map
proof imports 30 source-labeled companies and 20 high-fit targets. A local
scheduled job-radar proof script covers replay snapshots through the worker and
missing-snapshot failure health; separate local proof scripts cover
failure-health, retry recovery, and dead-letter behavior for policy-blocked
queued radar jobs. Local outcome
notes surface recorded application outcomes without fabricating predictive
causality. Controlled application pipeline proof now records user-supplied
application statuses in copied state and surfaces them in reports while keeping
delivery and scoring claims separate. Local packet export writes inspectable
Markdown only; it does not prove Google Docs draft creation, external sending,
delivery, or submission.
A controlled seven-packet P1 export proof writes all imported Tier 1 packets to
local Markdown in a copied proof home, but it is not user approval for sending
and not proof over the user's operational home. Local refresh audit proves the
gate logic, not the real one-day refresh. It does not yet prove complete
current live role coverage, operational scheduled recurrence, a one-day refresh
loop, true warm intros, autonomous market discovery, or statistical conversion
learning.
Source-family boundary proof now shows broad boards and startup/VC boards stay
demoted while generic role-category navigation is rejected, but that only
protects the role-ingestion boundary; it does not make those secondary leads
application-ready without direct company/ATS confirmation.

## Architecture Shape

Keep this native to Arcwell:

- SQLite for durable state.
- Source cards or source-card-like records for external evidence.
- Wiki/docs for readable summaries and reports.
- Existing policy/cost/secret boundaries before provider/model calls.
- Worker jobs only after manual and local proof exist.
- Codex skill as the operating discipline.
- MCP/CLI surfaces for durable actions once the data model exists.

Do not build a separate SaaS-style CRM first. The first useful system is a
private, inspectable local ledger with strong source and privacy discipline.

## Improvement 1: Candidate Evidence Ledger

### Claim

Arcwell can maintain a private, reusable evidence ledger that maps the user's
actual resume, public work, private-safe experience, blog posts, GitHub
projects, and skill stories to role requirements.

### Why It Matters

Without this, every role-fit explanation drifts into generic language. The
system must know the difference between:

- public proof link
- resume bullet
- private but usable experience
- private and not safe to mention
- stale or unverified claim
- evidence gap

### Data Model

```text
job_candidate_profiles
  id
  label
  current_resume_source
  linkedin_source
  github_profile
  blog_url
  created_at
  updated_at

job_evidence_cards
  id
  profile_id
  title
  evidence_type        # resume, github, blog, project, work, standard, talk, private_safe
  visibility           # public, private_safe, private_blocked, needs_review
  summary
  proof_url
  local_path
  source_date
  confidence           # verified, user_claimed, inferred, stale
  tags                 # agent-systems, devrel, rust, swift, mcp, etc.
  safe_application_text
  unsafe_terms
  created_at
  updated_at

job_evidence_claims
  id
  evidence_card_id
  claim
  claim_kind           # technical, leadership, writing, product, community, domain
  proof_level          # public, resume, private_safe, unverified
  can_use_in_resume
  can_use_in_outreach
  can_use_in_interview
```

### Workflow

1. Import current resume, GitHub profile/repo list, public blog/project pages,
   and user-approved private-safe notes.
2. Extract candidate evidence cards.
3. Mark each card with visibility and confidence.
4. Produce an evidence index by theme:
   - AI agent systems
   - MCP/tools/hooks/subagents
   - developer-facing systems
   - technical writing for senior engineers
   - local-first assistant infrastructure
   - Rust/Swift/TypeScript
   - public projects
   - product/platform judgment
5. Review and approve before use in applications.

### Mirage Checks

- Evidence card exists but has no source or proof URL.
- Private project name appears in safe application text.
- A stale resume claim is treated as verified.
- One generic "AI agents" card is reused for every role.
- The ledger has public links but no actual application-ready phrasing.

### Tests

- Reject evidence cards with `visibility=private_blocked` when rendering
  resume/outreach material.
- Reject application packets that contain unsafe terms.
- Mark unverified evidence as unusable for claims that require public proof.
- Preserve uncertainty when source dates are missing.

### Production Proof

Use the current resume, GitHub, blog/project pages, and user-approved private
summaries to produce at least 20 evidence cards. Manually review them and prove
that no blocked private names appear in generated application text.

Current proof: `.arcwell-dev/proofs/job-evidence-production-import-20260628T213435Z/artifacts/proof-packet.md`
imports 26 reviewed evidence cards and 26 claims from the current Markdown
resume, public GitHub metadata, and current public blog/project evidence into a
disposable Arcwell home. The evidence-readiness report passed with 26 ready
cards, 26 privacy passes, no blocked cards, no needs-review cards, and no
findings. This proves the evidence-ledger import/review gate only; it does not
prove P1 role import, role scoring, application packets, live source refresh,
or operational job radar.

## Improvement 2: Source Confidence

### Claim

Arcwell can distinguish confirmed live roles from weak leads by source family,
canonical URL, freshness, and confirmation status.

### Why It Matters

A job-search report can look useful while half the roles are expired, US-only,
or visible only through stale aggregators. Tiering must depend on source
confidence.

### Data Model

```text
job_sources
  id
  source_family        # company, ats, job_board, vc_board, founder_post, funding_signal
  name
  url
  market_scope         # london, uk, berlin, europe, global_remote
  refresh_policy
  created_at

job_source_health
  id
  source_id
  checked_at
  status               # healthy, stale, blocked, failed, partial, unknown
  http_status
  error_code
  fetched_count
  accepted_count
  rejected_count
  note

job_role_source_links
  id
  role_id
  source_id
  source_url
  observed_at
  confidence           # canonical_confirmed, secondary_confirmed, aggregator_only, stale, unknown
  evidence_excerpt
```

### Source Confidence Rules

- `canonical_confirmed`: company career page or official ATS role page.
- `secondary_confirmed`: reputable secondary page links to a plausible role,
  but company/ATS page was not confirmed.
- `aggregator_only`: found on a job board, but canonical role source missing.
- `stale`: role page closed, archived, expired, or search snippet disagrees
  with official source.
- `unknown`: source blocked, dynamically hidden, or not enough data.

Tiering rule:

- Tier 1 requires `canonical_confirmed`.
- Tier 2 may use `secondary_confirmed` with an explicit verification action.
- `aggregator_only`, `stale`, and `unknown` cannot be apply-now.

### Mirage Checks

- Search snippet says "new" but role page is missing.
- LinkedIn shows a role but official careers page does not.
- The same role appears under multiple URLs and is counted multiple times.
- A remote role is treated as UK/EU-compatible without confirmation.

### Tests

- Deduplicate same role by ATS id, canonical URL, company, title, and location.
- Demote aggregator-only roles below Tier 1.
- Mark a role stale when official source contradicts secondary source.
- Keep source-health failures visible in the final report.

### Production Proof

Refresh at least 20 plausible role leads across at least three source families.
Produce a report where every Tier 1 role has a canonical source and every
weaker source has a verification note.

## Improvement 3: Numeric Fit Scoring

### Claim

Arcwell can score each role using explicit, auditable dimensions instead of
title matching or prose-only impressions.

### Score Dimensions

Use 0-5:

- `role_fit`: seniority, function, scope, day-to-day work.
- `domain_fit`: AI agents, developer tools, platform, APIs, cloud, data,
  security, open source, standards, ecosystem work.
- `evidence_fit`: concrete evidence cards mapped to requirements.
- `geo_work_fit`: London, UK, Europe, remote, hybrid, timezone, relocation.
- `stage_fit`: tiny startup, seed, Series A, midsize, scaleup, enterprise.
- `practical_odds`: source confidence, hiring signal, network path,
  compensation band, competition, freshness.
- `interest_energy`: whether the user is likely to do good work there.

Derived fields:

- `total_score`
- `weighted_score`
- `tier`
- `score_explanation`
- `score_blockers`

### Weighting

Default weights:

```text
role_fit:        1.4
domain_fit:      1.3
evidence_fit:    1.5
geo_work_fit:    1.2
stage_fit:       1.0
practical_odds:  1.2
interest_energy: 1.0
```

Hard blockers override score:

- source not live
- location impossible
- junior/mid-only role
- private evidence would be required to make the case
- pure sales/content role with no technical ownership

### Data Model

```text
job_fit_scores
  id
  role_id
  profile_id
  scored_at
  scorer             # human, model, hybrid
  role_fit
  domain_fit
  evidence_fit
  geo_work_fit
  stage_fit
  practical_odds
  interest_energy
  weighted_score
  tier
  blockers_json
  explanation
```

### Mirage Checks

- Score is high because title contains "Developer Advocate".
- Evidence score is high without evidence-card links.
- Geography score ignores source work-mode text.
- A role with a hard blocker still becomes Tier 1.

### Tests

- Hard blockers demote role regardless of weighted score.
- Roles without evidence-card links cannot get evidence score above 2.
- Canonical-confirmed live roles outrank equivalent secondary-only roles.
- Same inputs produce deterministic scores.

### Production Proof

Score all roles in a live P1 pass. Every Tier 1 and Tier 2 role must show the
numeric score, the evidence links behind `evidence_fit`, and skeptic notes for
low dimensions.

## Improvement 4: Application Packets

### Claim

Arcwell can turn a Tier 1 role into a safe, tailored application packet:
resume emphasis, proof links, outreach note, likely objections, and interview
stories. After user approval, Arcwell can export that packet as local Markdown
for review without recording the application as sent.

### Packet Shape

```text
job_application_packets
  id
  role_id
  profile_id
  generated_at
  status              # draft, reviewed, approved, used, rejected
  resume_emphasis
  tailored_bullets
  outreach_note
  proof_links_json
  likely_objections
  interview_stories
  privacy_check_id
  reviewer_note
```

Packet sections:

- Role thesis.
- Resume top-third emphasis.
- 3-5 tailored bullets.
- Proof links to include.
- Outreach note.
- Warm-intro note, if available.
- Likely objections.
- Interview stories.
- Questions to ask.
- Privacy and claim check.

### Rendering Rules

- Do not write generic cover letters.
- Do not repeat the job description back to the company.
- Do not make unsupported claims.
- Do not include private names.
- Keep outreach short and specific.
- Use practitioner language, not marketing gloss.

### Mirage Checks

- Packet sounds polished but could be sent to any company.
- Packet cites public links that do not support the claim.
- Packet depends on private evidence.
- Packet hides the main weakness instead of naming how to handle it.
- Export writes a file but silently records the role as applied.
- Export relies on an old packet privacy check after policy has changed.

### Tests

- Fail packet if no role source is attached.
- Fail packet if no evidence cards are linked.
- Fail packet if privacy check flags blocked terms.
- Fail packet if outreach note has no company/product-specific sentence.
- Fail packet export unless the packet is approved and privacy-passing.
- Recheck privacy over the exact Markdown export before writing the file.

### Production Proof

Generate reviewed packets for the seven P1 Tier 1 roles:

- Sierra
- Anthropic Claude Code
- Tailscale
- Ably
- Mistral
- Langfuse
- Temporal

The proof is not that packets exist. The proof is that each packet maps role
requirements to approved evidence and passes privacy review. Local export proof
adds a smaller claim: approved packets can become inspectable Markdown without
recording an application event. A controlled seven-packet proof now exports the
imported P1 packet set through `packet-export-set` in a copied proof home with
seven Markdown files, one JSON manifest, seven export-time privacy checks, and
zero application rows:
`.arcwell-dev/proofs/job-p1-packet-export-controlled-proof-20260629T085332Z-55708/artifacts/proof-packet.json`.
It is not proof of user approval, delivery, or submission.

For the company-target path, controlled packet creation is now proven over the
bounded live role-confirmation input. Proof at
`.arcwell-dev/proofs/job-company-target-packet-proof-20260629T043746Z-98813/artifacts/proof-packet.json`
copies the company-target role proof home, creates five new draft packets for
the confirmed Tier 1 Ably/Northflank roles, records five passing packet
privacy checks, keeps all five new packets in `draft`, and keeps application
rows at zero. This is not approval, export, sending, submission, or proof that
the roles are still current beyond the source role proof.

For the expanded direct-role path, controlled packet creation is now proven over
the selected global and European role-confirmation input. Proof at
`.arcwell-dev/proofs/job-company-target-expanded-packet-proof-20260629T052029Z-33758/artifacts/proof-packet.json`
copies the expanded direct-role proof home, creates four new draft packets for
the confirmed Tier 1 Anthropic, Sierra, Tailscale, and Langfuse roles, records
four passing packet privacy checks, keeps all four new packets in `draft`, and
writes no new application rows. This is not approval, export, sending,
submission, or proof that the roles are still current beyond the source role
proof.

Controlled export is also proven for those four expanded direct-role packets.
Proof at
`.arcwell-dev/proofs/job-company-target-expanded-packet-export-proof-20260629T052450Z-51935/artifacts/proof-packet.json`
copies the expanded packet proof home, approves the four packets for local
export review only, writes four Markdown files, records four export-time
privacy checks, and writes no new application rows. This is not user approval
for submission, Google Docs draft creation, delivery, sent/applied status, or
operational-home export.

Controlled export is also proven for those five company-target packets. Proof
at
`.arcwell-dev/proofs/job-company-target-packet-export-proof-20260629T044317Z-35795/artifacts/proof-packet.json`
copies the packet proof home, approves the five packets for local export review
only, writes five Markdown files, records five export-time privacy checks, and
keeps application rows at zero. This is not user approval for submission,
Google Docs draft creation, delivery, sent/applied status, or operational-home
export.

## Improvement 5: London Startup Sourcing Layer

### Claim

Arcwell can search tiny and early London startups more deliberately than broad
job boards do.

### Why It Matters

The user's concentric strategy treats tiny London startups as plausible because
proximity, founder access, and technical trust can offset stage risk. Standard
job boards miss or flatten this segment.

### Source Families

Initial London startup sources:

- Y Combinator Work at a Startup.
- Seedcamp portfolio and job pages.
- LocalGlobe portfolio and job pages.
- Entrepreneur First companies and hiring posts.
- Hoxton, Balderton, Index, Accel, Notion, Northzone, Creandum, Atomico,
  Point Nine, and other relevant UK/EU portfolios.
- Recent funding announcements.
- Founder posts and company blogs.
- London AI/devtools/security/data community signals.

### Company Card Shape

```text
job_company_cards
  id
  company_name
  website_url
  source_family
  market
  stage
  funding_signal
  product_category
  technical_audience
  developer_facing_score
  london_relevance
  remote_maturity
  hiring_page_url
  founder_or_team_signal
  last_checked_at
```

### Workflow

1. Build a London startup source map.
2. Normalize companies before roles.
3. Score company relevance even when no role is posted.
4. Capture founder/team signal.
5. Watch companies that are high-fit but not currently hiring.

### Mirage Checks

- A startup is included only because it is AI-branded.
- No evidence that the company sells to or depends on developers.
- No hiring page or contact path exists.
- A tiny non-London company is treated like a London local-fit opportunity.

### Tests

- Company card requires at least one source URL.
- Tiny-startup fit requires London relevance or exceptional remote evidence.
- Companies without current roles become monitor targets, not apply-now roles.

### Production Proof

Produce a London startup map with at least 30 company cards, 10 high-fit
companies, current hiring status, and at least 3 founder/warm-intro targets.

## Improvement 6: Warm-Intro Map

### Claim

Arcwell can identify and track plausible people-based routes into Tier 1 and
Tier 2 opportunities without pretending public name discovery is a real intro.

### Data Model

```text
job_contacts
  id
  name
  company_id
  role_title
  public_profile_url
  source_url
  relationship_status  # unknown, public_only, possible_mutual, known, contacted
  relevance            # hiring_manager, recruiter, founder, devrel_lead, engineer, investor
  note

job_intro_paths
  id
  role_id
  contact_id
  path_type            # direct, mutual, recruiter, investor, community, unknown
  confidence           # confirmed, plausible, weak
  next_action
  status               # identify, ask, sent, replied, declined, stale
```

### Workflow

1. For each Tier 1 role, find likely hiring manager, recruiter, DevRel lead,
   founder, or technical team lead.
2. Check public sources for relevance.
3. Mark whether there is a known, possible, or unknown relationship path.
4. Draft a short outreach note only after privacy checks pass.
5. Track status and follow-up date.

### Mirage Checks

- Public profile found, but no relationship path exists.
- "Warm intro" means only a LinkedIn search result.
- Outreach note includes private claims.
- Contact relevance is guessed from title alone.

### Tests

- Cannot mark `warm_intro` without `relationship_status=known` or
  `possible_mutual`.
- Cannot mark a contact as hiring manager without source evidence or user
  confirmation.
- Outreach rendering requires approved application packet or evidence cards.

### Production Proof

For seven Tier 1 roles, produce an intro map with at least one plausible contact
path each, and explicitly mark which are public-only versus warm-intro-ready.

## Improvement 7: Weekly Refresh And Report

### Claim

Arcwell can refresh a configured job search, detect changes, and produce a
weekly report with new roles, stale roles, promoted/demoted roles, application
status, and next actions.

### Report Sections

- Bottom line.
- Newly opened roles.
- Closed or stale roles.
- Promoted roles.
- Demoted roles.
- Applications sent.
- Replies/interviews/rejections.
- Warm-intro status.
- Evidence gaps.
- Next five actions.
- Source health.
- Coverage limits.

### Data Model

```text
job_search_runs
  id
  profile_id
  scope
  started_at
  completed_at
  proof_level
  source_count
  role_count
  new_role_count
  stale_role_count
  error_count
  report_artifact_id

job_role_status_events
  id
  role_id
  run_id
  status              # new, unchanged, promoted, demoted, stale, closed, applied
  previous_tier
  current_tier
  note
  created_at

job_applications
  id
  role_id
  packet_id
  status              # planned, applied, intro_requested, replied, interview, rejected, offer, withdrawn
  applied_at
  follow_up_at
  outcome_note
```

### Worker Shape

Worker jobs should come last:

1. Manual report generation.
2. Local fixture report generation.
3. Live manual refresh.
4. Controlled scheduled refresh.
5. Operational weekly refresh with source health.

### Mirage Checks

- Weekly report exists but sources were not refreshed.
- Report says "new" because previous state was missing.
- Closed roles remain apply-now.
- Source failures are hidden.
- Application status is inferred from memory instead of user-confirmed events.

### Tests

- Closed canonical role becomes stale/closed and cannot remain Tier 1.
- Repeated run with same roles marks unchanged, not new.
- Source failure appears in source-health section.
- Applied roles retain application status across refreshes.

### Production Proof

Run two weekly refreshes at least one day apart in a controlled home. Prove that
new/stale/unchanged role state is computed from durable prior state, not from a
single report snapshot.

## Improvement 8: Privacy And Public-Shelf Checks

### Claim

Arcwell can prevent private project names, confidential employer details,
unreleased systems, customer data, and unsupported claims from leaking into
resume, outreach, or interview-prep material.

### Safety Model

```text
job_privacy_rules
  id
  pattern
  rule_type            # blocked_term, sensitive_claim, needs_review, public_ok
  severity             # block, warn, note
  replacement_guidance
  created_at

job_privacy_checks
  id
  artifact_type        # evidence_card, packet, outreach, resume, report
  artifact_id
  checked_at
  decision             # pass, warn, block
  findings_json
```

### Check Categories

- Private project names.
- Secret internal product names.
- Employer/customer details.
- Credentials or token-like strings.
- Unreleased systems.
- Unsupported metrics.
- Inflated seniority or scope.
- Claims that require public evidence but only have private evidence.
- Links that point to private/local files.

### Workflow

1. User-approved blocked term list.
2. Public-shelf classification for every evidence card.
3. Privacy check before packet generation.
4. Privacy check after packet generation.
5. Human review for `warn`.
6. Block application export for `block`.

### Mirage Checks

- A packet passes because it avoids exact blocked strings but still reveals the
  secret by description.
- A private artifact path is included as a proof link.
- A claim is public-safe but unsupported.
- A "review needed" warning is treated as pass.

### Tests

- Block exact private terms.
- Block local file paths in public proof links.
- Warn on claims whose only evidence is private.
- Reject packets with unsupported metrics.
- Require user approval to override warnings.

### Production Proof

Run privacy checks over all Tier 1 application packets and prove no blocked
terms, local-only proof links, or private-only claims remain in approved
outreach/resume material.

## Implementation Order

### Phase 0: Keep The Current Scaffold Honest

Deliverables:

- Keep `job-hunting` skill as operational guidance.
- Keep this plan as design only.
- Keep P1 reports clearly labeled as bounded manual production-data passes.

Proof:

- Plugin/docs verifier passes after skill changes.
- Reports include source coverage, stop reason, and proof level.

### Phase 1: Evidence Ledger And Privacy Checks

Build first because every downstream feature depends on safe evidence.

Deliverables:

- Evidence-card schema. **Implemented: Local Proof.**
- Privacy-rule schema. **Implemented: Local Proof.**
- Reviewed JSON packet import path for already-extracted resume/GitHub/blog/user
  notes. **Implemented: Local Proof.**
- Evidence review report. **Implemented: Local Proof.**
- Privacy check renderer. **Implemented for text, packet gates, and the local
  evidence-readiness report.**

Proof:

- Controlled-home production-data proof imported 26 reviewed evidence cards and
  26 claims from the current resume, public GitHub metadata, and public
  blog/project evidence.
- No blocked terms in safe application text; the evidence-readiness report
  passed with 26 privacy passes and no blocked or needs-review cards.
- Fixture tests for blocked private terms and unsupported claims.

### Phase 2: Role Source Cards And Source Confidence

Deliverables:

- Role-card schema. **Implemented: Local Proof.**
- Source and source-health schema. **Implemented: Local Proof.**
- Manual `role add` and reviewed batch `job import` path. **Implemented: Local
  Proof.**
- Configured single-source refresh from caller-supplied page text/html, plus
  explicit policy-gated `fetch_live` for the stored source URL. **Implemented:
  Local Proof.**
- Canonical/secondary/aggregator/stale confidence logic. **Implemented for
  local scoring gates and configured source refresh; exhaustive live page
  verification still missing.**

Proof:

- Controlled-home proof entered the 7 P1 Tier 1 roles as durable role cards,
  with 7 source records, 7 source-linked role-source links, 7 healthy
  source-health rows, and 7 company cards.
- Partial live proof ran configured refresh over those 7 imported sources with
  explicit provider-network policy; 6 sources were partial, 1 failed, all 7 P1
  roles remained live, and source-health rows preserved the partial/failure
  state.
- Tier 1 requires canonical confirmation; this proof uses the bounded P1
  report's canonical/company-source classification rather than rediscovering
  the jobs live.
- Secondary-only roles are automatically held for verification.

### Phase 3: Scoring And Skeptic Pass

Deliverables:

- Fit score schema.
- Deterministic scoring function.
- Skeptic-finding schema.
- Shortlist compiler.

Proof:

- Seven P1 Tier 1 roles score deterministically in
  `.arcwell-dev/proofs/job-hunting-p1-tier1-import-20260628T214548Z/artifacts/proof-packet.md`;
  the compiled shortlist has 7 Tier 1 entries.
- Hard blockers demote roles.
- Evidence score cannot exceed 2 without evidence-card links.

### Phase 4: Application Packets

Deliverables:

- Packet schema.
- Packet renderer.
- Role-specific resume emphasis.
- Outreach note and warm-intro note templates.
- Privacy check before approval.
- Local Markdown export after approval.

Proof:

- Draft packets for the seven P1 Tier 1 roles.
- Every packet links role requirements to approved evidence cards.
- Privacy checks pass before export; the proof records 7 packet privacy checks
  with decision `pass`. User approval and sending are still out of scope.
- Local packet export rejects drafts, rechecks privacy at export time, writes
  Markdown only after approval, and leaves application status unchanged.
- Controlled P1 packet export copies the imported P1 proof home, approves the
  seven draft packets for local export review only, exports seven Markdown
  files, records seven `packet_export` privacy checks, and keeps application
  rows at zero. This does not prove user-approved sending, Google Docs export,
  ATS submission, or operational-home packet export.
- Controlled company-target packet proof copies the bounded role-confirmation
  proof home, creates five new draft packets for the confirmed Tier 1
  Ably/Northflank roles, records five passing packet privacy checks, leaves the
  created packets in `draft`, and records no applications:
  `.arcwell-dev/proofs/job-company-target-packet-proof-20260629T043746Z-98813/artifacts/proof-packet.json`.
- Controlled expanded direct-role packet proof copies the selected direct-role
  proof home, creates four new draft packets for the confirmed Tier 1
  Anthropic/Sierra/Tailscale/Langfuse roles, records four passing packet
  privacy checks, leaves the created packets in `draft`, and records no new
  applications:
  `.arcwell-dev/proofs/job-company-target-expanded-packet-proof-20260629T052029Z-33758/artifacts/proof-packet.json`.
- Controlled expanded direct-role packet export copies that packet proof home,
  approves the four packets for local export review only, exports four Markdown
  files, records four `packet_export` privacy checks, and records no new
  applications:
  `.arcwell-dev/proofs/job-company-target-expanded-packet-export-proof-20260629T052450Z-51935/artifacts/proof-packet.json`.
- Controlled company-target packet export copies that packet proof home,
  approves the five packets for local export review only, exports five Markdown
  files, records five `packet_export` privacy checks, and records no
  applications:
  `.arcwell-dev/proofs/job-company-target-packet-export-proof-20260629T044317Z-35795/artifacts/proof-packet.json`.

### Phase 5: Startup Sourcing And Company Cards

Deliverables:

- Company-card schema.
- London startup source map.
- Manual company capture path.
- Local company-target scouting report.

Proof:

- Controlled proof at
  `.arcwell-dev/proofs/job-hunting-london-startup-map-20260628T222016Z-20346/artifacts/proof-packet.md`
  imported 30 company cards from current public source families.
- The summary records 20 high-fit companies, 30 active-or-recently-listed
  hiring signals, 3 public founder/team targets, 30 public-only contact paths,
  and 0 warm-intro-ready paths.
- Local Proof at
  `.arcwell-dev/proofs/job-company-targets-local-proof-20260628T235317Z/artifacts/proof-packet.json`
  compiles a deterministic target report from durable company cards and public
  evidence tags while writing no role cards.
- Bounded production-data proof at
  `.arcwell-dev/proofs/job-company-target-role-proof-20260629T042335Z-31884/artifacts/proof-packet.json`
  copies the London company-map proof home, live-refreshes five selected
  company/ATS pages, writes ten same-day canonical-confirmed live roles, rejects
  parser-noise roles, and scores seven confirmed roles against reviewed
  evidence.
- Bounded direct-role expansion proof at
  `.arcwell-dev/proofs/job-company-target-expanded-role-proof-20260629T050451Z-33630/artifacts/proof-packet.json`
  copies the job-hunting proof home, live-refreshes four selected direct
  company/ATS role pages from expanded global and European AI/devtools/platform
  targets, writes four same-day canonical-confirmed live roles with zero live
  parser-noise role titles, and scores all four as Tier 1 against reviewed
  evidence.
- This is a company watch map plus bounded role proof. It does not prove
  official current hiring on every company page, user-network warm intros,
  broad role-level scoring for all companies, or scheduled refresh.
- Current hiring status and contact path marked.

### Phase 6: Warm-Intro Map

Deliverables:

- Contact schema.
- Intro-path schema.
- Contact relevance classification. **Implemented: Local Proof for bounded
  labels and source-evidence/user-confirmation note gating.**
- Outreach/warm-intro status tracking. **Implemented: Local Proof for
  public-only versus warm-ready status, plus Controlled Local Proof for
  non-sending outreach-readiness classification.**

Proof:

- Seven Tier 1 roles have at least one contact path.
- Public-only contacts are not mislabeled as warm intros.
- Strategic contact relevance labels cannot be recorded from title inference
  alone; notes must name source evidence or user confirmation.
- Outreach readiness blocks no-packet, draft-packet, public-only-route,
  newer-draft-over-stale-approval, and privacy-regression cases; it only passes
  with an approved packet, fresh privacy pass, and known or possible-mutual
  route.

### Phase 7: Weekly Refresh

Deliverables:

- Search-run schema. **Implemented: Local Proof.**
- Role-status event schema. **Implemented: Local Proof.**
- Weekly report compiler. **Implemented: Local Proof.**
- Manual refresh command for caller-supplied observed/stale/closed role ids.
  **Implemented: Local Proof.**
- Refresh audit command for the two-refresh/elapsed-time transition gate.
  **Implemented: Local Proof for audit logic only.**
- Preserved one-day refresh proof harness. **Implemented: incomplete first
  checkpoint only; the real 24-hour gate is still open.** The packet now
  exposes explicit gate state and second-run timing fields for the eventual
  rerun.
- Worker job after manual proof. **Implemented: Local Proof for replay
  snapshots.**

Proof:

- Two refreshes produce correct new/unchanged/stale transitions. **Audit gate
  locally proven; preserved proof root has first checkpoint with
  new/unchanged/stale/closed transitions; second wall-clock run still open.**
- Source-health failures appear in report.
- Applied roles keep application status.

### Phase 8: Operational Job Radar

Deliverables:

- Scheduled worker job. **Implemented: Local Proof for replay snapshots.**
- Retry, dead-letter, recovery, and source-health visibility. **Implemented:
  Local Proof for missing-source health, pre-executor policy-denial health,
  same-job recovery after policy repair, and dead-letter after retry
  exhaustion. Live policy/cost recovery over recurring external refresh remains
  open.**
- Ops view for local ledger state. **Implemented: Local Proof.**
- Weekly digest delivery, if explicitly authorized. **Implemented as
  Controlled Local Proof for prepared weekly-report provider delivery through a
  loopback Cloudflare Email-compatible provider, including policy, cost,
  privacy, authorization, provider-failure, and idempotent replay gates. Live
  external provider delivery remains open.**
- Outcome learning for future scoring. **Implemented: Local Proof for explicit
  shortlist/report notes only; no predictive scoring or conversion model.**
- Application pipeline state. **Implemented: Controlled Local Proof over copied
  production-data input for planned/applied/intro/interview/rejected/withdrawn
  rows, approved-packet gating for applied-like statuses, public-only intro
  visibility, and next-action rendering. This is not real sending, delivery,
  warm-intro proof, or operational-home CRM proof.**
- Operational readiness audit. **Implemented: Controlled Local Proof for a
  read-only blocker report. The audit can show local slices passing while still
  blocking operational promotion when a required gate is missing. The proof now
  shows provider-delivery blocking before a successful attempt, provider
  delivery passing after a loopback Cloudflare Email-compatible delivery
  attempt, and scheduled-radar passing after two replay-backed
  `job_radar_refresh` worker jobs, while the real one-day refresh gate still
  blocks. This does not prove wall-clock recurrence or live external delivery.**

Proof:

- Scheduled replay run completes in controlled home. **Proven locally.**
- Source health shows healthy/stale/failed/partial states. **Locally proven for
  refreshed replay success and missing-snapshot failure; broad live recurrence
  remains open.**
- Pre-executor policy/cost denial leaves operator-visible health. **Locally
  proven for provider-network policy denial.**
- Policy-blocked scheduled radar jobs retry and then either recover after
  policy repair or dead-letter after retry exhaustion. **Locally proven for
  queued `job_radar_refresh`; not live recurrence proof.**
- Report is delivered only to authorized target. **Controlled Local Proof now
  prepares a report only for an authorized channel subject after privacy checks,
  records blocked rows for unauthorized/privacy-blocked attempts, writes one
  prepared channel message on success, sends that prepared message through a
  loopback provider, records one successful `channel_delivery_attempt`, and
  replays idempotently without a duplicate provider call. A provider cost kill
  switch blocks before any provider attempt. Live external provider send
  remains unproven.**
- Outcome data changes future scoring notes without fabricating causality.
  **Locally proven for same-company and same-role application-history notes;
  tiers and weighted scores remain evidence/source-based.**
- Application statuses appear in shortlist and weekly reports without claiming
  delivery. **Controlled copied-home proof records six statuses and proves a
  draft packet cannot back `applied`. The same proof now records one
  public-only intro path and renders `warm_intro_ready: 0`, `identify: 1`, and
  next actions for scored roles in the requested profile. Contact relevance
  labels now require a note naming source evidence or user confirmation.**

## Suggested CLI And MCP Surfaces

Do not add all surfaces at once. Add only as the durable layer exists.

Current CLI examples:

```text
arcwell job import --path reviewed-job-packet.json
arcwell job evidence-list <profile-id>
arcwell job evidence-review <profile-id>
arcwell job privacy-check --artifact-type outreach --artifact-id <id> --text <text>

arcwell job role-add --company <name> --role-title <title> --source-family company --source-url <url> --source-confidence canonical_confirmed --posting-freshness same_day --current-status live
arcwell job role <role-id>
arcwell job source-refresh <source-id> --body-path source.html
arcwell job source-refresh <source-id> --fetch-live
arcwell job radar-schedule <profile-id> --scope <scope> --source-id <id>
arcwell job radar-enqueue <profile-id> --scope <scope> --source-id <id>
arcwell job radar-schedule <profile-id> --scope <scope> --source-id <id> --fetch-live --cadence cold --email-to <email>
arcwell job radar-enqueue <profile-id> --scope <scope> --source-id <id> --fetch-live --email-to <email>

arcwell job score-add <role-id> --profile-id <profile-id>
arcwell job shortlist <profile-id>
arcwell job outreach-readiness <profile-id>
arcwell job packet-create <role-id> --profile-id <profile-id>
arcwell job packet-approve <packet-id> --reviewer-note "Reviewed by user"
arcwell job packet-export <packet-id> --out packet-exports/
arcwell job packet-export-set <profile-id> --packet-id <packet-id> --out packet-exports/
arcwell job company-targets <profile-id> --market london

arcwell job company-add --company-name <name> --website-url <url> --source-family company --market london --london-relevance high
arcwell job intro-add <role-id> --contact-id <contact-id>
arcwell job application-record <role-id> --packet-id <packet-id> --status applied
arcwell job refresh <profile-id> --scope <scope> --observed-role-id <id>
arcwell job refresh-audit <profile-id> --scope <scope>
arcwell job weekly-report <profile-id> --scope <scope>
arcwell job weekly-report-delivery-prepare <report-id> --channel email --subject <subject> --target <target>
arcwell job weekly-report-deliveries --report-id <report-id>
```

Current MCP tools:

- `job_profile_add`
- `job_profiles`
- `job_import_batch`
- `job_evidence_add`
- `job_evidence_list`
- `job_evidence_review_report`
- `job_privacy_check`
- `job_role_add`
- `job_roles`
- `job_score_add`
- `job_shortlist`
- `job_outreach_readiness`
- `job_company_targets`
- `job_packet_create`
- `job_packet_approve`
- `job_packet_export`
- `job_packet_export_set`
- `job_application_record`
- `job_source_refresh`
- `job_radar_schedule`
- `job_radar_enqueue`
- `job_refresh_manual`
- `job_refresh_audit`
- `job_operational_audit`
- `job_weekly_report`
- `job_weekly_report_delivery_prepare`
- `job_weekly_report_deliveries`

## Refuting Tests

Minimum severe tests before promotion beyond scaffold:

- `severe_job_privacy_blocks_private_terms_in_packet`
- `severe_job_privacy_blocks_local_file_proof_links`
- `severe_job_role_demotes_aggregator_only_listing`
- `severe_job_role_marks_closed_canonical_source_stale`
- `severe_job_score_hard_blocker_overrides_high_fit`
- `severe_job_score_requires_evidence_links_for_high_evidence_fit`
- `severe_job_shortlist_dedupes_same_ats_role`
- `severe_job_import_batch_records_reviewed_packet_without_claiming_live_discovery`
- `severe_job_application_requires_approved_packet_for_applied_status`
- `severe_job_outreach_readiness_requires_approved_packet_privacy_and_warm_route`
- `severe_job_packet_export_requires_approved_packet_without_recording_application`
- `severe_job_packet_export_rechecks_privacy_before_writing_file`
- `severe_job_packet_export_set_is_local_only_and_preflighted`
- `severe_job_evidence_review_report_passes_only_reviewed_claim_mapped_safe_evidence`
- `severe_job_evidence_review_report_blocks_public_local_and_private_term_mirages`
- `severe_job_source_refresh_writes_roles_health_and_stales_missing_roles`
- `severe_job_source_refresh_keeps_vc_board_roles_secondary_and_company_cards_monitored`
- `severe_job_source_refresh_policy_denial_records_failed_health_without_writes`
- `severe_job_manual_refresh_does_not_reannounce_unchanged_or_closed_roles`
- `severe_job_radar_refresh_policy_recovery_retries_same_failed_job`
- `severe_job_radar_refresh_policy_denial_dead_letters_after_retry_exhaustion`
- `severe_job_refresh_audit_blocks_immediate_repeats_for_one_day_gate`
- `severe_job_refresh_audit_passes_transition_logic_only_with_lowered_elapsed_gate`
- `severe_job_operational_audit_blocks_fake_promotion_despite_local_slices`
- `severe_job_outcome_history_adds_notes_without_tier_fabrication`
- `severe_ops_ui_surfaces_job_hunting_state_without_raw_html`
- `severe_job_weekly_refresh_preserves_application_status`
- `severe_job_intro_public_profile_is_not_warm_intro`
- `severe_job_contact_relevance_requires_source_evidence_or_user_confirmation`

## P1-to-Operational Proof Packet

Before claiming operational status, produce:

- Feature name and current status.
- Exact sources configured.
- Data volume and time window.
- Role cards created, updated, stale, rejected, and deduped.
- Source-health states.
- Evidence-card count and visibility breakdown.
- Privacy check findings.
- Fit-score distribution.
- Tier 1/Tier 2/Tier 3/pass counts.
- Application packets created and approved.
- Warm-intro paths found and confirmed.
- Applications recorded and outcomes.
- Weekly refresh transitions.
- Worker jobs run, retries, and failures.
- Remaining risks and next actions.

## Open Decisions

- Whether to store job records in existing Arcwell core tables or introduce a
  distinct `arcwell-job-hunting` package.
- Whether role-source full text should become general source cards or
  job-specific source records linked to source cards.
- Whether external source adapters should be custom or mostly browser/manual
  for the first operational version.
- Whether Google Drive resume docs should be imported into the evidence ledger
  through connectors or manual export first.
- Whether application packets should write Google Docs drafts or remain local
  Markdown until privacy checks mature.

## Next Concrete Step

Use the implemented reviewed-packet import path to load real candidate evidence
and the existing P1 shortlist:

1. Completed for the evidence ledger: import the current resume, public GitHub
   metadata, and current public blog/project evidence into at least 20 reviewed
   evidence cards, then run the evidence-readiness report and treat any block
   findings as stop conditions before using those cards in packets. Proof:
   `.arcwell-dev/proofs/job-evidence-production-import-20260628T213435Z/artifacts/proof-packet.md`.
2. Completed for controlled P1 replay: import the P1 roles as durable role
   cards with source confidence, evidence mappings, scores, skeptic findings,
   and draft packet material. Proof:
   `.arcwell-dev/proofs/job-hunting-p1-tier1-import-20260628T214548Z/artifacts/proof-packet.md`.
3. Completed for controlled P1 replay: run privacy checks over the seven P1
   Tier 1 application-packet drafts. The proof records 7 passing packet privacy
   checks; user approval/export of that seven-packet set remains open.
4. Completed as local packet-approval proof: draft packets cannot back an
   `applied` application record, approval requires a reviewer note and passing
   packet privacy check, and the MCP workflow moves draft to approved before
   recording applied. Proof:
   `.arcwell-dev/proofs/job-packet-approval-local-proof-20260629T000346Z/artifacts/proof-packet.json`.
   This is approval-state proof, not export/send proof.
5. Completed as controlled manual refresh reconciliation: the browser ops proof
   copied the imported/live-refresh P1 home, used existing non-healthy
   source-health rows, closed one imported Tier 1 role, recorded one blocked
   privacy check, and recorded one follow-up application row. This is not the
   one-day wall-clock refresh proof.
6. Completed as partial live proof: use configured source refresh over the
   imported source list with policy-allowed `fetch_live`. Proof:
   `.arcwell-dev/proofs/job-source-refresh-live-policy-allow-linked-fixed-20260628T215717Z/artifacts/proof-packet.md`.
   The run preserved accepted, rejected, partial, and failed source-health
   evidence, but does not prove complete current live coverage.
7. Completed as browser-backed controlled production-data proof: inspect
   `/ops/ui` over the imported data and keep stale/closed roles,
   source-health failures, privacy blocks, application follow-ups, and job
   summary metrics visible on desktop and mobile without body overflow. Proof:
   `.arcwell-dev/proofs/job-hunting-ops-browser-proof-20260628T221108Z-9768/artifacts/proof-packet.json`.
8. Completed as controlled company-map proof: import the London startup/company
   source map with 30 company cards, 20 high-fit companies, 3 public
   founder/team targets, and no warm-intro claim. Proof:
   `.arcwell-dev/proofs/job-hunting-london-startup-map-20260628T222016Z-20346/artifacts/proof-packet.md`.
9. Completed as public-only Tier 1 contact-path proof: import contact and
   intro-path scaffolding for the seven imported Tier 1 roles with 0
   warm-intro-ready claims. Proof:
   `.arcwell-dev/proofs/job-hunting-tier1-intro-map-20260628T231128Z-33821/artifacts/proof-packet.md`.
10. Completed as local scheduled replay proof: create and execute the
   `job_radar` watch-source path through `job_radar_refresh`, proving replay
   source refresh, weekly report creation, honest source-health advancement,
   and failed health for missing snapshots. Proof:
   `.arcwell-dev/proofs/job-radar-scheduled-local-proof-20260629T121212Z-42806/artifacts/proof-packet.json`.
   Rerunnable proof script: `scripts/job-radar-scheduled-local-proof`.
   Follow-up severe coverage proves optional CLI/MCP delivery metadata can be
   carried from a scheduled/enqueued `job_radar_refresh` into report
   preparation and a controlled Cloudflare Email-compatible provider send. This
   remains controlled provider-path proof, not live external email proof or
   operational recurrence.
11. Completed as local failure-health proof: a queued live
    `job_radar_refresh` blocked by provider-network policy writes failed
    generic source health for `job:radar:<profile_id>` and no job-source or
    role rows. Proof:
    `.arcwell-dev/proofs/job-radar-failure-health-proof-20260629T121156Z-40419/artifacts/proof-packet.json`.
    Rerunnable proof script: `scripts/job-radar-failure-health-proof`.
12. Completed as local company-target proof: compile a scouting report from
    durable company cards and public evidence tags, rank London company targets,
    expose not-current-role warnings, and write no role cards. Proof:
    `.arcwell-dev/proofs/job-company-targets-local-proof-20260628T235317Z/artifacts/proof-packet.json`.
13. Completed as bounded production-data role-confirmation proof: copy the
    London company-map proof home, live-refresh five selected company/ATS pages,
    write ten same-day canonical-confirmed live roles, reject parser-noise role
    cards, and score seven confirmed roles against reviewed evidence. Proof:
    `.arcwell-dev/proofs/job-company-target-role-proof-20260629T042335Z-31884/artifacts/proof-packet.json`.
14. Completed as bounded direct-role expansion proof: copy the job-hunting
    proof home, live-refresh four selected direct company/ATS role pages from
    expanded global and European AI/devtools/platform targets, write four
    same-day canonical-confirmed live roles with zero live parser-noise titles,
    and score Anthropic, Sierra, Tailscale, and Langfuse as Tier 1 against
    reviewed evidence. Proof:
    `.arcwell-dev/proofs/job-company-target-expanded-role-proof-20260629T050451Z-33630/artifacts/proof-packet.json`.
15. Completed as controlled expanded direct-role draft-packet proof: the
    selected direct-role proof home can be copied, and the four confirmed
    Tier 1 Anthropic/Sierra/Tailscale/Langfuse roles can get new
    privacy-passing draft packets using the scored evidence ledger while
    application rows do not increase. Proof:
    `.arcwell-dev/proofs/job-company-target-expanded-packet-proof-20260629T052029Z-33758/artifacts/proof-packet.json`.
    This is not user review, approval, Markdown/Google Docs export, delivery,
    submission, or operational-home packet proof.
16. Completed as controlled expanded direct-role packet-export proof: the four
    expanded direct-role packets can be approved inside a copied proof home for
    local export review only, exported to four local Markdown files, rechecked
    with four export-time privacy checks, and left with no new application
    rows. Proof:
    `.arcwell-dev/proofs/job-company-target-expanded-packet-export-proof-20260629T052450Z-51935/artifacts/proof-packet.json`.
    This is not user approval for sending, Google Docs draft creation, ATS
    submission, delivery, or proof over the user's operational home.
17. Completed as bounded scheduled live-fetch proof: the expanded direct-role
    proof home can be copied, a `job_radar` watch source can be scheduled, an
    immediate `job_radar_refresh` can be enqueued with `fetch_live=true`, and
    one worker pass can live-refresh the four selected direct role sources with
    healthy source-health rows and a production-data-proof search run. Proof:
    `.arcwell-dev/proofs/job-radar-live-fetch-proof-20260629T053332Z-16818/artifacts/proof-packet.json`.
    This is not real wall-clock recurrence, one-day refresh proof, exhaustive
    market coverage, or operational radar.
18. Completed as local retry/recovery/dead-letter proof: a policy-blocked
    `job_radar_refresh` retries the same queued job, recovers after policy is
    allowed when replay snapshots cover all configured sources, and
    dead-letters after max-attempts policy denial without source or role writes.
    Proof:
    `.arcwell-dev/proofs/job-radar-retry-recovery-local-proof-20260629T121206Z-41836/artifacts/proof-packet.json`.
    Rerunnable proof script: `scripts/job-radar-retry-recovery-local-proof`.
19. Completed as local outcome-history note proof: durable application
    outcomes add explicit shortlist and weekly-report notes for related roles
    without changing weighted scores or tiers. Proof:
    `.arcwell-dev/proofs/job-outcome-history-local-proof-20260629T031421Z/artifacts/proof-packet.json`.
20. Completed as controlled application-pipeline proof: the expanded
    direct-role packet-export proof home can be copied, a draft packet is
    blocked from backing `applied`, six controlled application rows can be
    recorded across `planned`, `applied`, `intro_requested`, `interview`,
    `rejected`, and `withdrawn`, one source-evidence-noted public-only contact
    and intro path can be recorded without becoming warm-intro-ready, and
    shortlist plus weekly-report output surfaces application statuses, intro
    status, next actions, and a controlled role-status change for scored roles
    in the requested profile. Proof:
    `.arcwell-dev/proofs/job-application-pipeline-proof-20260629T064631Z-15803/artifacts/proof-packet.json`.
    This is not real user approval, operational-home application tracking,
    sent/submitted applications, Google Docs draft creation, warm-intro proof,
    live freshness, recurrence, or outcome-learning proof.
21. Completed as controlled outreach-readiness proof: a fresh disposable proof
    home walks one scored role through no packet, draft packet plus public-only
    contact path, approved packet plus public-only path, approved packet plus
    known warm route, a newer draft that blocks stale approval, approval of
    that revision, and a later privacy-rule regression. The report blocks every
    unsafe or route-thin state, passes only approved/privacy-passing known-route
    states, and writes zero channel messages or provider delivery attempts.
    Proof:
    `.arcwell-dev/proofs/job-outreach-readiness-proof-20260629T074806Z-87604/artifacts/proof-packet.json`.
    This is readiness classification only, not outreach send, real user-network
    proof, Google Docs draft creation, application submission, replies,
    operational-home tracking, live freshness, or recurrence proof.
22. Completed as controlled weekly-report delivery provider-path proof: a fresh
    disposable proof home seeds profile/evidence/role/score/report state,
    blocks an unauthorized email subject with no privacy check, channel message,
    or provider attempt, authorizes the subject, prepares one outbound channel
    message after a passing privacy check, sends that message through a
    loopback Cloudflare Email-compatible provider, records one successful
    provider attempt, proves provider-send replay reuses the same attempt
    without a duplicate provider call, then blocks a privacy-denied report with
    no new channel message or provider attempt. Follow-up severe tests prove a
    Cloudflare Email provider cost kill switch blocks before any provider
    attempt and records a denied cost decision. Proof:
    `.arcwell-dev/proofs/job-weekly-report-delivery-proof-20260629T093944Z-4028/artifacts/proof-packet.json`.
    This is not live external provider delivery, email/Telegram send to a real
    recipient, application submission, Google Docs draft creation,
    operational-home tracking, live freshness, or recurrence proof.
23. Completed as controlled operational-audit proof: a disposable proof home
    satisfies evidence, source, Tier 1 score, approved packet, warm-route,
    application-history, weekly-report, and delivery-preparation slices, proves
    the provider-delivery gate blocks before a successful attempt, sends the
    prepared weekly report through a loopback Cloudflare Email-compatible
    provider, enqueues and drains two replay-backed `job_radar_refresh` worker
    jobs, then proves `job_operational_audit` recognizes both provider-delivery
    and scheduled-radar gates as passing while still blocking operational
    promotion on the real one-day refresh gate. Proof:
    `.arcwell-dev/proofs/job-operational-audit-proof-20260629T095526Z-2463/artifacts/proof-packet.json`.
    This is a blocker report over durable state, not live fetch, live provider send,
    application submission, wall-clock recurrence, or operational status.
24. Completed as local packet-export proof: approved privacy-passing packets
    can be exported to local Markdown, export rejects draft packets, privacy is
    rechecked against the exact Markdown before writing, and no application is
    recorded or marked sent. Proof:
    `.arcwell-dev/proofs/job-packet-export-local-proof-20260629T032842Z/artifacts/proof-packet.json`.
25. Completed as controlled P1 packet-export proof: the seven imported Tier 1
    packets can be approved inside a copied proof home for local export review
    only, exported through `packet-export-set` to seven local Markdown files
    plus one JSON manifest, rechecked with seven export-time privacy checks,
    and left with zero application rows before and after export. Proof:
    `.arcwell-dev/proofs/job-p1-packet-export-controlled-proof-20260629T085332Z-55708/artifacts/proof-packet.json`.
    This is not user approval for sending, Google Docs draft creation, ATS
    submission, or proof over the user's operational home.
26. Completed as controlled company-target draft-packet proof: the bounded
    company-target role proof home can be copied, and the five confirmed
    Tier 1 Ably/Northflank roles can get new privacy-passing draft packets
    using the scored evidence ledger while application rows remain zero. Proof:
    `.arcwell-dev/proofs/job-company-target-packet-proof-20260629T043746Z-98813/artifacts/proof-packet.json`.
    This is not user review, approval, Markdown/Google Docs export, delivery,
    submission, or operational-home packet proof.
27. Completed as controlled company-target packet-export proof: the five
    company-target packets can be approved inside a copied proof home for local
    export review only, exported to five local Markdown files, rechecked with
    five export-time privacy checks, and left with zero application rows before
    and after export. Proof:
    `.arcwell-dev/proofs/job-company-target-packet-export-proof-20260629T044317Z-35795/artifacts/proof-packet.json`.
    This is not user approval for sending, Google Docs draft creation, ATS
    submission, delivery, or proof over the user's operational home.
28. Completed as local refresh-audit proof: the audit reads durable completed
    refresh runs and linked role-status events, blocks immediate repeats under
    the default 24-hour gate, and can separately prove transition/source
    evidence when the elapsed threshold is deliberately lowered for local
    audit-logic tests. Proof:
    `.arcwell-dev/proofs/job-refresh-audit-local-proof-20260629T034434Z/artifacts/proof-packet.json`.
29. Started the preserved controlled one-day refresh proof. First checkpoint at
    `.arcwell-dev/proofs/job-refresh-one-day-controlled-proof/artifacts/proof-packet.json`
    records one completed refresh run with seven source-health rows and
    `new`, `unchanged`, `stale`, and `closed` transitions. The packet is
    intentionally `incomplete`: it still lacks a second completed run and the
    real 24-hour elapsed window. It now records
    `gate_state=waiting_wall_clock`, `second_started_at=null`, and
    `hours_between_runs=null` so the final rerun cannot hide whether the second
    checkpoint actually happened. Rerun
    `scripts/job-refresh-one-day-proof --proof-root /Users/chabotc/Projects/arcwell/.arcwell-dev/proofs/job-refresh-one-day-controlled-proof`
    after `2026-06-30T04:06:11.893Z` UTC.
30. Completed as bounded controlled live-radar recurrence proof:
    `.arcwell-dev/proofs/job-radar-live-recurrence-controlled-proof/artifacts/proof-packet.json`
    copied the expanded direct-role proof home, scheduled a `job_radar` watch
    source, let `worker run-once` enqueue and complete the first
    `fetch_live=true` `job_radar_refresh` from `watch_poll`, waited for the
    real hot-cadence due time, then completed a second `watch_poll`-enqueued
    `fetch_live=true` worker refresh over the same four selected direct role
    sources. The packet records `status=passed`, `gate_state=passed`,
    `first_worker_job_id=605ea016-26cf-4568-ab68-f90bc15b1a52`,
    `second_worker_job_id=97407161-652f-4fdb-83bd-ecf4d1c583b3`,
    `first_started_at=2026-06-29T11:43:19.543890+00:00`,
    `second_started_at=2026-06-29T12:43:34.270688+00:00`, and
    `hours_between_runs=1.004091`. This proves one selected hot-cadence
    recurrence interval in a copied proof home; it is not the one-day refresh
    proof, broad coverage, operational-home monitoring, live external delivery,
    application sending, warm-intro proof, or outcome proof.
31. Completed as aggregate implementation-plan audit proof:
    `scripts/job-implementation-plan-audit-proof` resolves the implementation
    plan's proof-packet references plus the current source-family boundary
    packet and the preserved live-radar recurrence packet, verifies all
    referenced packets exist and JSON packets parse,
    checks referenced job proof scripts plus the core radar replay/failure/
    retry proof scripts are executable, confirms the latest
    operational-audit packet has the expected controlled `weekly_refresh`
    block while evidence/source/scoring/privacy/packet/outreach/application/
    weekly-report/delivery/scheduled-radar gates pass, and checks both Codex
    job-hunting skill copies still label the one-day gate as incomplete and
    keep the live-radar recurrence slice bounded and non-operational. Latest
    proof:
    `.arcwell-dev/proofs/job-implementation-plan-audit-proof-20260629T1257-one-day-packet-hygiene/artifacts/proof-packet.json`.
    It records `status=hold_wall_clock` and `audit_decision=hold` because the
    controlled live-radar recurrence proof passed and the preserved
    one-day proof is still waiting for the real 24-hour rerun. The aggregate
    gates preserve each proof packet's second-run fields and
    `hours_between_runs`, while re-evaluating `ready_at_utc` so a stale
    `waiting_wall_clock` packet becomes `ready_for_rerun` after the wall-clock
    gate passes. This is an evidence-integrity and claim-boundary audit; it
    does not fetch sources, send applications, satisfy the one-day wall-clock
    gate, or promote the job-hunting system to Operational.

Do not call scheduled job radar operational until the one-day wall-clock
refresh, broader operational-home recurrence, live policy/cost recovery, live
external provider-send proof, and ops proof gates pass. The bounded live
`fetch_live` worker slice proves one scheduled drain, the live-radar recurrence
proof covers one selected hot-cadence interval only, and scheduled-report email
coverage is controlled provider-path proof only. Do not call manual refresh live
coverage unless the caller has supplied current source-health evidence for the
run. Do not treat
company-target reports as apply-ready role evidence outside the bounded sources
where canonical current role cards exist and role-level scoring has been run.
