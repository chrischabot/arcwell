---
name: job-hunting
description: Use when finding, qualifying, prioritizing, tailoring, or tracking roles the user is a strong fit for across global companies, London startups, European startups, and remote developer-facing AI/devtools/platform roles.
---

# Job Hunting

Use this skill for role discovery, role-fit scoring, application strategy,
tailored application packets, outreach notes, interview prep, and pipeline
tracking.

This is a fit-finding skill, not a mass-application skill. The goal is to find
roles where the user's actual experience, public evidence, writing, projects,
and seniority make a credible application.

Current status:

- `Local Proof`: Arcwell now has a durable local job-hunting ledger for
  profiles, evidence cards, privacy checks, role cards, source confidence, fit
  scores, application packets, local packet export, intro paths, application
  status, reviewed-packet batch import, manual refresh reconciliation, and
  weekly reports, outreach-readiness, weekly-report delivery prepare/send,
  and operational-audit blocker reports. Core APIs, CLI
  `arcwell job ...`, and MCP tools are locally tested.
- `Controlled Local Proof`: the imported seven-role P1 packet set can be
  copied into a proof home, approved for local export review only, exported to
  seven Markdown files plus one manifest through `packet-export-set`, and
  rechecked with seven export-time privacy checks. This is not user approval
  for sending, operational-home export, Google Docs draft creation, delivery,
  or submission proof.
- `Bounded Production Data Proof`:
  `scripts/job-source-family-boundary-proof` creates a disposable proof home,
  refreshes source-family fixtures, and live-refreshes the YC London software
  engineering board. It proves company/ATS pages create canonical-confirmed
  roles, VC/startup boards create secondary-confirmed leads, broad job boards
  create aggregator-only leads, and generic `/jobs/role/...` category
  navigation is rejected instead of entering the role set. The latest live YC
  proof wrote 26 secondary-confirmed roles, 33 company cards, and 0 generic
  category role imports. This is source-family boundary proof, not exhaustive
  market coverage or an apply-now shortlist.
- `Bounded Production Data Proof`: `scripts/job-company-target-role-proof`
  copies the London company-map proof home, live-refreshes five selected
  company/ATS pages, writes ten same-day canonical-confirmed live roles, rejects
  parser-noise role cards, and scores seven confirmed roles against reviewed
  evidence. This is bounded role confirmation, not exhaustive market coverage,
  future freshness, application-packet creation, sending, warm intros, or
  outcomes.
- `Bounded Production Data Proof`:
  `scripts/job-company-target-expanded-role-proof` copies the job-hunting proof
  home, live-refreshes four selected direct company/ATS role pages from expanded
  global and European AI/devtools/platform targets, writes four same-day
  canonical-confirmed live roles with zero live parser-noise role titles, and
  scores all four as Tier 1 against reviewed evidence. This is selected
  direct-role expansion proof, not exhaustive market coverage, future
  freshness, packet export, sending, warm intros, or outcomes.
- `Controlled Local Proof over production-data input`:
  `scripts/job-company-target-expanded-packet-proof` copies the expanded
  direct-role proof home, creates four draft packets for the confirmed Tier 1
  Anthropic/Sierra/Tailscale/Langfuse roles, records four passing packet
  privacy checks, leaves the packets in `draft`, and writes no new application
  rows. This is not current-refresh proof beyond the source role proof, user
  approval, export, Google Docs draft creation, sending, submission, warm
  intros, or outcomes.
- `Controlled Local Proof over production-data input`:
  `scripts/job-company-target-expanded-packet-export-proof` copies the expanded
  packet proof home, approves those four packets for local export review only,
  exports four Markdown files, records four export-time privacy checks, and
  writes no new application rows. This is not real user approval,
  operational-home export, Google Docs draft creation, sending, submission,
  warm intros, or outcomes.
- `Controlled Local Proof over production-data input`:
  `scripts/job-company-target-packet-proof` copies the bounded company-target
  role proof home, creates five draft packets for the confirmed Tier 1
  Ably/Northflank roles, records five passing packet privacy checks, leaves the
  packets in `draft`, and records no applications. This is not current-refresh
  proof beyond the source role proof, user approval, export, Google Docs draft
  creation, sending, submission, warm intros, or outcomes.
- `Controlled Local Proof over production-data input`:
  `scripts/job-company-target-packet-export-proof` copies the company-target
  packet proof home, approves those five packets for local export review only,
  exports five Markdown files, records five export-time privacy checks, and
  records no applications. This is not real user approval, operational-home
  export, Google Docs draft creation, sending, submission, warm intros, or
  outcomes.
- `Controlled Local Proof`:
  `scripts/job-radar-scheduled-local-proof` reruns the severe replay-backed
  scheduled-radar tests for the `job_radar` watch-source path, missing-snapshot
  failure health, and MCP schedule/enqueue coverage. This is replay proof, not
  live freshness, recurrence, or operational monitoring.
- `Controlled Local Proof`:
  `scripts/job-radar-failure-health-proof` and
  `scripts/job-radar-retry-recovery-local-proof` rerun the severe tests for
  provider-policy denial health, same queued-job retry after policy repair, and
  dead-letter after retry exhaustion. These prove local failure/recovery
  semantics only; they do not prove live provider recovery, broad source
  coverage, or wall-clock recurrence.
- `Production Data Proof (bounded scheduled live fetch slice)`:
  `scripts/job-radar-live-fetch-proof` copies the expanded direct-role proof
  home, schedules and immediately enqueues `job_radar_refresh` with
  `fetch_live=true`, drains one worker job, live-refreshes the four selected
  direct role sources with four healthy source-health rows, writes one
  production-data-proof search run, and writes one local weekly report. This is
  one scheduled-worker live-fetch slice, not real recurrence, not the one-day
  refresh proof, not exhaustive market coverage, and not operational radar.
- `Production Data Proof (bounded live-radar recurrence slice)`:
  `scripts/job-radar-live-recurrence-proof` preserves a controlled
  live-radar recurrence proof root. The current packet records `status=passed`
  and `gate_state=passed` after two completed `fetch_live=true`
  `job_radar_refresh` jobs enqueued by the `job_radar` watch-source poller over
  the same four selected direct role sources. The first worker job was
  `605ea016-26cf-4568-ab68-f90bc15b1a52`; the second was
  `97407161-652f-4fdb-83bd-ecf4d1c583b3`; `hours_between_runs=1.004091`.
  This proves one selected hot-cadence recurrence interval in a copied proof
  home, not the one-day refresh proof, broad market coverage, operational-home
  monitoring, live external delivery, application sending, warm intros, or
  outcomes.
- `Controlled Local Proof over production-data input`:
  `scripts/job-application-pipeline-proof` copies the expanded direct-role
  packet-export proof home, proves a draft packet cannot back `applied`,
  records controlled `planned`, `applied`, `intro_requested`, `interview`,
  `rejected`, and `withdrawn` rows, records one source-evidence-noted
  public-only contact plus intro path, and shows application status in shortlist
  warnings plus application status, intro status, next actions, and role-status
  change visibility for scored roles in weekly-report output.
  Contact relevance labels require source-evidence or user-confirmation notes.
  This is not real sending, operational-home tracking, Google Docs draft
  creation, warm-intro proof, live freshness, recurrence, or outcome-learning
  proof.
- `Controlled Local Proof`: `scripts/job-outreach-readiness-proof` creates a
  disposable scored role and proves outreach readiness blocks no-packet,
  draft-packet, public-only-route, and later privacy-regression states. It
  passes only after an approved packet, fresh outreach-note privacy pass, and a
  known warm route exist, and it writes zero channel messages or provider
  delivery attempts. This is readiness classification only, not outreach send,
  Google Docs draft creation, real user-network agreement, application
  submission, operational tracking, live freshness, or recurrence.
- `Controlled Local Proof`: `scripts/job-weekly-report-delivery-proof` seeds a
  disposable weekly report, proves unauthorized and privacy-blocked delivery
  preparation writes blocked rows with no channel message/provider attempt,
  proves an authorized privacy-passing report writes one prepared channel
  message, proves replay does not duplicate the message, then sends that
  prepared email through a loopback Cloudflare Email-compatible provider and
  records exactly one successful `channel_delivery_attempt`. Severe tests also
  prove a Cloudflare Email provider cost kill switch blocks before any provider
  attempt. This is controlled provider-path proof only, not live external
  email/Telegram delivery, application submission, Google Docs draft creation,
  operational tracking, live freshness, or recurrence.
- `Controlled Local Proof`: `scripts/job-operational-audit-proof` seeds a
  disposable home where evidence/source/scoring/approved-packet/warm-route/
  application-history/weekly-report/delivery-preparation slices exist, proves
  provider-delivery is blocked before a successful delivery attempt, sends a
  prepared weekly report through a loopback Cloudflare Email-compatible
  provider, enqueues and drains two replay-backed `job_radar_refresh` worker
  jobs, then proves `job_operational_audit` recognizes the provider-delivery
  and scheduled-radar gates as passing while still blocking operational
  promotion on the real one-day refresh gate. This is a blocker report with
  controlled provider-path and replay-worker evidence, not live fetch, live
  external provider send, application submission, wall-clock recurrence, or
  operational status.
- `Incomplete Gate`: `scripts/job-refresh-one-day-proof` preserves the
  controlled one-day refresh proof root. The current first checkpoint has one
  completed run with source evidence and `new`, `unchanged`, `stale`, and
  `closed` transitions. It is not a pass until the same proof root has a
  second completed run after the real 24-hour elapsed-time window. The current
  packet records `gate_state=waiting_wall_clock`,
  `ready_at_utc=2026-06-30T04:06:11.893Z`, `second_started_at=null`,
  `hours_between_runs=null`, no second refresh checkpoint yet, and the exact
  `next_action` rerun command.
- It is not yet an operational tracker, crawler, scheduler, broad live source
  refresher, or application CRM.
- Do not claim exhaustive market coverage, automatic freshness, or live
  monitoring unless a production-data run has just verified those claims.

Target capability:

Arcwell should be able to run a job-intelligence pass that discovers current
roles, normalizes them into evidence cards, clusters them by market thesis,
scores fit against the user's evidence, runs a skeptic pass, and returns an
apply-now shortlist plus tailored next actions.

The output is the recommendation, not the ledger. The ledger exists so the
recommendation can be inspected.

## Default Positioning

Assume this working profile unless the user updates it:

- Senior/staff-level engineer and developer-facing product builder.
- Strong fit for developer relations, developer experience, developer advocacy,
  AI agents, developer tools, platform engineering, technical product, open
  source, API/platform ecosystems, and technical content with real engineering
  ownership.
- Evidence base includes current resume, public GitHub, public blog/portfolio,
  LinkedIn when available, and local Codex/Claude histories only when the user
  asks for that context or it is already in the active task.
- Stronger applications should emphasize systems taste, agent/tooling work,
  technical writing, public practitioner communication, product judgment,
  standards/API/platform work, Rust/Swift/TypeScript experience, local-first
  tools, search/memory/research systems, and ability to turn fuzzy technical
  domains into usable developer workflows.
- Avoid positioning the user as a generic content marketer, community manager,
  or junior implementation engineer unless the role has real technical
  ownership and senior scope.

Do not invent credentials, employment dates, public availability, metrics,
customer names, or private project details. If a claim comes from memory or an
older artifact and has not been verified in the current turn, label it as
possibly stale before using it externally.

## Modes

When the user asks for job-hunting help, identify the mode:

- `discover`: find live roles and companies.
- `qualify`: score roles the user provides or roles already found.
- `prioritize`: rank a role list into apply-now, warm-intro, monitor, and pass.
- `tailor`: adapt resume emphasis, cover note, outreach, and proof links.
- `track`: maintain or update an application pipeline.
- `prep`: prepare for recruiter, hiring-manager, technical, or founder calls.

If the mode is unclear, default to `discover` plus `qualify` for senior
developer-facing AI/devtools/platform roles.

## Freshness Rule

For live openings, always verify current postings with web search or direct
career pages unless the user explicitly gives the role list. Job openings,
locations, salary bands, and visa requirements are time-sensitive.

Use primary sources whenever possible:

- Company career pages.
- ATS pages such as Ashby, Greenhouse, Lever, Workable, SmartRecruiters, and
  Teamtailor.
- LinkedIn, Wellfound, Otta, Welcome to the Jungle, Cord, Y Combinator jobs,
  and VC portfolio job boards.
- Company blogs, founder posts, GitHub orgs, launch posts, funding news, and
  product docs to understand the company and role.

For broad or high-stakes searches, apply the deep-research pattern: source map,
candidate list, claim ledger, skeptic pass, final shortlist. If the user
explicitly asks for deep research or the scope is broad enough to require
coverage, use the `deep-research` skill and live web search.

Record for each role:

- Company.
- Role title.
- Source URL.
- Location and work mode.
- Company stage or size, if available.
- Posting date or freshness signal, if available.
- Salary or compensation, if available.
- Date accessed.
- Fit tier and next action.

Do not recommend a role as apply-now until the posting page is still live.

## Job-Intelligence Pipeline

Use this pipeline for serious searches:

1. **Profile and Evidence Ledger**
   - Confirm the current resume, public GitHub, blog/portfolio, LinkedIn, and
     private-safe experience summaries that can be used.
   - Separate public evidence from private evidence.
   - Mark stale or unverified facts before using them in application material.
   - When a reviewed packet exists, import it with `arcwell job import --path`
     or MCP `job_import_batch`. This records supplied facts only; it does not
     prove live source discovery.
   - After importing or adding evidence, run the local evidence-readiness
     report with `arcwell job evidence-review <profile_id>` or MCP
     `job_evidence_review_report`. Treat block findings as stop conditions for
     application material, and treat warnings as review work rather than proof
     of readiness.

2. **Source Map**
   - Search across source families, not a single board.
   - Include company pages, ATS pages, job boards, VC portfolio boards,
     founder posts, funding/news signals, GitHub/product activity, and
     relevant community signals.
   - Record coverage gaps instead of hiding them.

3. **Role Source Cards**
   - Normalize each live candidate into a structured card.
   - Deduplicate by company, role title, ATS id, canonical URL, and location.
   - Treat aggregator-only listings as leads until the company/ATS source is
     confirmed.

4. **Company and Market Clustering**
   - Group role cards into themes that explain the market.
   - Useful clusters include AI agents, developer tools, DevRel/DevEx,
     eval/observability, London early-stage startups, Berlin/European scaleups,
     global platform companies, and founder-led technical products.
   - Prefer clusters that create an application strategy, not just taxonomy.

5. **Fit Ledger**
   - Score roles against the user's actual evidence.
   - Map the role's most important requirements to resume bullets, public repos,
     blog posts, projects, standards/API/platform work, or private-safe
     experience.
   - Missing evidence is a decision input, not something to smooth over.

6. **Skeptic Pass**
   - Attack every Tier 1 or Tier 2 role before recommending it.
   - Look for stale postings, junior scope, content-only marketing, hidden sales
     quota, location/visa mismatch, weak company signal, missing public proof,
     and private-name leakage.

7. **Shortlist and Application Packet**
   - Produce a reader-facing shortlist with next actions.
   - For each Tier 1 role, include application angle, evidence to emphasize,
     likely objection, outreach/warm-intro path, and proof links.
   - Use MCP `job_packet_approve`, or local `arcwell job packet-approve`,
     before recording `applied`, `intro_requested`, `replied`, `interview`, or
     `offer` with a packet id. Approval records review intent; it does not
     mean anything was sent.
   - Use MCP `job_packet_export`, or local
     `arcwell job packet-export <packet-id> --out <dir>`, only after approval
     when the user wants an inspectable local Markdown packet. Export rechecks
     privacy and writes a local file; it does not send the application, create
     a Google Doc, or record the role as applied.
   - Use MCP `job_packet_export_set`, or local
     `arcwell job packet-export-set <profile-id> --packet-id <id> --out <dir>`,
     when a reviewed set of approved packets should be exported together.
     Packet-set export writes local Markdown files plus a manifest; it does
     not create Google Docs, send, submit, or record applications.
   - Use `scripts/job-p1-packet-export-proof` only as a controlled copied-home
     proof for the current seven-packet P1 set. It proves local packet-set
     Markdown export, manifest creation, and export-time privacy checks, not
     user approval or sending.
   - Use `scripts/job-application-pipeline-proof` only as a controlled copied-home
     proof that application pipeline rows and reporting are wired. It proves
     approved-packet gating for applied-like statuses, application status
     visibility, source-evidence-noted public-only intro visibility, contact
     relevance note gating, next-action rendering, and role-status change
     visibility for scored roles in the requested profile; it does not prove
     any application was sent, user-approved for real use, backed by a real warm
     intro, live-fresh, or recurring.
   - Use MCP `job_outreach_readiness`, or local
     `arcwell job outreach-readiness <profile-id>`, before treating an
     approved packet and contact path as actionable outreach. It rechecks the
     packet outreach note for privacy and requires a known or possible-mutual
     route; public-only contacts remain identify/monitor work. This does not
     send outreach or prove a real introduction.
   - Use MCP `job_operational_audit`, or local
     `arcwell job operational-audit <profile-id> --scope <scope>`, before any
     claim that the job-hunting radar is operational. The audit reads durable
     state and reports blockers; it does not fetch sources, send messages,
     submit applications, or satisfy the one-day/provider/recurrence gates by
     itself.
   - Use MCP `job_company_targets`, or local `arcwell job company-targets`, for
     company-card scouting. Treat the output as monitoring/outreach research,
     not as current openings or application-ready role evidence.
   - Keep source-card ids, scrape details, duplicate rows, and failed fetches in
     the internal ledger unless the user asks for them.

8. **Watch and Refresh**
   - For ongoing work, track role status, application status, response outcome,
     source freshness, and next refresh date.
   - Do not call a watchlist current unless it has been refreshed.
   - Use MCP `job_source_refresh`, or the matching local job source-refresh CLI
     subcommand, for one configured source at a time. Caller-supplied page
     text/html is a manual snapshot; `fetch_live` is explicit live network
     access and must pass provider-network policy.
   - Use `scripts/job-source-family-boundary-proof` when changing source
     parsing or source-family scoring. It must keep company/ATS roles
     canonical-confirmed, VC/startup-board roles secondary-confirmed, broad
     job-board roles aggregator-only, and generic role-category links out of
     role cards.
   - Use MCP `job_refresh_manual`, or the matching local job refresh CLI
     subcommand, only for caller-supplied observed/stale/closed role ids. It
     reconciles durable state across a search pass; it does not fetch sources.
   - Use MCP `job_refresh_audit`, or local
     `arcwell job refresh-audit <profile_id> --scope <scope>`, before treating
     refresh history as the two-refresh/one-day proof gate. The operational
     gate requires the default 24-hour elapsed check; lowering the threshold is
     only useful for local audit-logic tests.
   - Use `scripts/job-radar-scheduled-local-proof`,
     `scripts/job-radar-failure-health-proof`, and
     `scripts/job-radar-retry-recovery-local-proof` when changing scheduled
     radar replay, failure-health, retry, or dead-letter semantics. These are
     Local Proof scripts; they do not satisfy live freshness or recurrence.
   - Use `scripts/job-refresh-one-day-proof` to create or resume the preserved
     controlled one-day proof root. `--allow-incomplete` is only for recording
     the current blocked packet; it must not be treated as passing the gate.
     The current preserved root should be rerun after
     `2026-06-30T04:06:11.893Z` UTC.
   - Use `scripts/job-radar-live-recurrence-proof` to create or resume the
     preserved controlled live-radar recurrence proof root. The current root
     has passed one selected hot-cadence recurrence interval over four direct
     role sources. This still does not prove one-day refresh, broad coverage,
     operational-home monitoring, or live external delivery.
   - Use MCP `job_radar_schedule` / `job_radar_enqueue`, or the matching local
     `arcwell job radar-schedule` / `arcwell job radar-enqueue` CLI
     subcommands, only when configured source ids exist. Replay snapshots are
     Local Proof for scheduled worker behavior; `fetch_live=true` is explicit
     policy/cost-gated live access and still needs recurrence proof before the
     radar is called operational.

## Role Source Card

Every real candidate should be representable as:

- `company`
- `role_title`
- `canonical_url`
- `source_family`: company, ATS, job board, VC board, founder post, funding
  signal, GitHub/product signal, referral, other
- `source_url`
- `date_accessed`
- `posting_freshness`: live, stale, duplicate, aggregator-only, unknown
- `location`
- `work_mode`: onsite, hybrid, remote-UK, remote-Europe, global-remote,
  unclear
- `company_stage_or_size`
- `role_seniority`
- `core_requirements`
- `implied_business_problem`
- `why_they_might_need_the_user`
- `evidence_matches`
- `gaps_or_blockers`
- `cluster`
- `fit_scores`
- `tier`
- `next_action`

If a role cannot be reduced to this shape, do not treat it as a strong
recommendation.

## P1 Production-Data Pass

A P1 pass is the first real, live-data version of the workflow. It should be
bounded, current, and auditable.

Default scope:

- London and UK: include tiny startups, seed/Series A, midsize startups,
  scaleups, and global companies.
- Berlin and European hubs: prioritize small-to-midsize startups and scaleups;
  include tiny companies only when remote maturity and domain fit are unusually
  strong.
- Global/remote: include global AI/devtools/platform companies where UK/EU
  remote is plausible or explicit.

Minimum evidence target:

- Search at least three source families.
- Inspect at least 20 plausible live role leads when time allows.
- Produce at least 8 role cards unless the market is genuinely quiet or source
  access blocks it.
- Produce a Tier 1/Tier 2 shortlist, not just a raw list.
- Mark rejected roles with the concrete reason when they were plausible enough
  to inspect.

P1 output must include:

- Source-family coverage.
- Clusters found.
- Tiered shortlist.
- Evidence mapping for each Tier 1 and Tier 2 role.
- Skeptic-pass concerns.
- Next action for every recommended role.
- Stop reason and coverage limits.

P1 does not prove:

- exhaustive coverage of all roles
- future freshness
- scheduled monitoring
- application conversion rate
- that every company will sponsor/accept UK candidates unless the source says
  so

## Concentric Market Logic

Use distance, stage, and work mode together. The farther the role is from the
user's strongest practical market, the more evidence the company and role need
to justify the application.

Default rings:

1. London and UK
   - Include tiny startups, seed/Series A companies, small-to-midsize startups,
     scaleups, and global companies.
   - Tiny London startups can be good matches when the product is technical,
     founder access matters, and the role can use the user's mix of engineering,
     developer storytelling, and product judgment.

2. UK remote, Europe remote, and global remote
   - Prefer stronger role-market fit, explicit remote support, and companies
     with evidence of hiring senior distributed people.
   - Tiny remote startups need a stronger product/domain match than tiny local
     startups.

3. European hubs: Berlin, Amsterdam, Paris, Dublin, Stockholm, Copenhagen,
   Zurich, Barcelona, Lisbon
   - Small-to-midsize startups and scaleups are usually better matches than very
     early companies unless the role is remote-first and highly aligned.
   - Berlin is a strong match for mid-sized AI/devtools/platform startups,
     especially if the role is senior, developer-facing, and remote/hybrid
     expectations are realistic.

4. Big global companies
   - Include global developer-platform, AI, cloud, data, security, and tooling
     companies when the role is senior enough and clearly aligned.
   - Location is less of a penalty when the role explicitly supports UK/EU
     remote or global remote hiring.

5. Farther-away early-stage startups
   - Usually pass unless there is exceptional fit: agent/devtools focus,
     founder-led technical audience, clear compensation/work-mode fit, and a
     role where the user's public evidence directly answers the job.

## Company Stage Heuristic

Use this as a guide, not a rigid rule:

- Tiny startup: good in London; possible elsewhere only with exceptional
  technical fit, remote maturity, or founder relationship.
- Seed/Series A: good in London/UK; selective for Europe/remote.
- Small-to-midsize startup: strong target in London, Berlin, Amsterdam, Paris,
  Dublin, Stockholm, and remote Europe.
- Scaleup: strong target across UK/EU/global remote when the role has senior
  DevRel/DevEx/platform scope.
- Big global company: worth applying when the role is sharply aligned and the
  application can be tailored with public proof.

## Fit Scoring

Score every real candidate before recommending it. Use 0-5 for each category:

- Role fit: seniority, scope, function, day-to-day work.
- Domain fit: AI agents, developer tools, platform, APIs, cloud, data,
  security, open source, standards, or ecosystem work.
- Evidence fit: resume, GitHub, blog, projects, talks, standards work, public
  writing, or shipped systems that directly support the application.
- Geography/work-mode fit: London, UK, Europe, remote, hybrid, relocation,
  timezone, and visa practicality.
- Company-stage fit: tiny, seed, Series A, midsize, scaleup, enterprise.
- Practical odds: hiring signal, network path, role specificity, seniority
  match, compensation band, competition, and freshness.
- Interest/energy: whether the user is likely to do compelling work there.

Then classify:

- `Tier 1`: Apply now. Strong fit, current posting, no hard blockers, and clear
  evidence to use.
- `Tier 2`: Worth warm intro, recruiter message, or monitoring. Good fit but
  has one or two uncertainties.
- `Tier 3`: Only use for volume or opportunistic outreach.
- `Pass`: Do not apply. Give the reason.

Name blockers explicitly:

- Location, visa, or timezone mismatch.
- Junior/mid-level scope.
- Pure sales quota without technical ownership.
- Content-only marketing without credible engineering involvement.
- Required domain or language experience the user does not have.
- Compensation, contract, relocation, or travel mismatch, if known.
- Stale, duplicate, or no-longer-live posting.

## Discovery Sources

Use several source families so the list is not just big-company career pages.

For London tiny and early-stage startups:

- Y Combinator jobs.
- Seedcamp, LocalGlobe, Entrepreneur First, Accel, Index, Balderton, Notion,
  Hoxton, Creandum, Northzone, and other UK/EU portfolio jobs.
- London AI, developer-tools, data, security, and infrastructure communities.
- Founder posts and company hiring pages from recent launches or funding news.

For Berlin and European midsize startups:

- Point Nine, Project A, Cherry Ventures, Earlybird, HV Capital, La Famiglia /
  General Catalyst, Creandum, Northzone, and Index portfolio jobs.
- Berlin, Amsterdam, Paris, Dublin, Stockholm, Copenhagen, Zurich, Barcelona,
  and Lisbon company career pages.
- Workable, Greenhouse, Lever, Ashby, Teamtailor, and Welcome to the Jungle.

For global and remote roles:

- OpenAI, Anthropic, GitHub, Cloudflare, Vercel, Stripe, Datadog, Docker, Snyk,
  MongoDB, Elastic, Supabase, Neon, Temporal, Sourcegraph, Hugging Face,
  Mistral, LangChain, LlamaIndex, Modal, Replicate, Chroma, Pinecone, Weaviate,
  and similar AI/devtools/platform companies.
- Only include live roles with plausible UK/EU remote support or London/Europe
  location fit.

Do not assume these companies have open roles. Search current openings.

## Evidence Mapping

For each strong role, map the job requirements to concrete evidence:

- Resume bullet or work history.
- Public blog post.
- Public GitHub repo.
- Portfolio project.
- Standards/API/platform contribution.
- Open-source or community artifact.
- Internal/private experience that can be described without naming secrets.

If evidence is missing, say whether to:

- apply anyway and handle it in the note,
- create a short proof artifact first,
- write a small targeted blog/project note,
- find a warmer path,
- or pass.

Never expose private project names, unreleased systems, confidential employer
details, credentials, customer data, or secret internal names. Translate private
experience into generic, truthful descriptions unless the user explicitly
authorizes a public name.

## Output Formats

For discovery, return a shortlist with:

| Tier | Company | Role | Location | Stage | Why it fits | Concerns | Next action | Source |

For a role memo, use:

- Company and role.
- Source URL and date accessed.
- Fit thesis.
- What the job asks for.
- Evidence to use.
- Gaps or risks.
- Application angle.
- Outreach target or warm-intro path.
- Tailored resume emphasis.
- Cover/outreach note, if requested.
- Decision: apply now, warm intro, monitor, or pass.

For application packets:

- Do not write generic cover letters.
- Write concise, role-specific notes grounded in the company's actual product,
  the job description, and the user's public/private-safe evidence.
- Include proof links only when they strengthen the fit.
- Prefer direct practitioner language over marketing language.

For a P1 production-data report, include:

- Bottom line.
- Source coverage and date accessed.
- Market clusters.
- Tier 1 apply-now roles.
- Tier 2 warm-intro or monitor roles.
- Pass/reject notes for inspected but weak roles.
- Evidence gaps to fix in resume, blog, GitHub, or portfolio.
- Next 24-hour application plan.

## Proof Levels

Use precise status language:

- `Scaffold`: skill text, templates, or prompts exist.
- `Local Proof`: fixture roles can be normalized, deduped, clustered, scored,
  imported from reviewed packets, reconciled through manual refresh, replayed
  through scheduled job radar, and rejected when stale or low-fit.
- `Production Data Proof`: a live pass over real sources produces source
  cards, clusters, fit scores, and a verified shortlist where apply-now roles
  are still live. Source-family boundary proof can also be production-data
  proof for parser/scoring boundaries, but it is not a shortlist by itself.
- `Operational`: production-data proof plus scheduled refresh, retries,
  source-health visibility, stale detection, application tracking, and outcome
  learning.
- `Done`: operational and no known core proof gate remains.

Do not collapse these levels. A good manually researched shortlist is
production-data proof for that run only; a replay-backed scheduled worker pass
is Local Proof for orchestration only. Neither is an operational job radar.

## Mirage Checks

Reject or weaken the output when:

- The list comes from one job board only.
- A posting is recommended from an aggregator without company/ATS confirmation.
- A VC/startup-board or broad-board lead is treated as canonical without direct
  company/ATS confirmation.
- Generic role-category links, such as `/jobs/role/...`, appear as role cards.
- The role is current only because a snippet appeared in search results.
- Fit score is based mainly on title matching.
- London/Berlin/global-stage logic is ignored.
- No requirement is mapped to the user's evidence.
- The evidence uses private project names or secret internal details.
- A polished application note hides a serious blocker.
- The output is a pretty table with no next action.
- The report says "latest" or "current" without live verification date.
- The recommendations cannot explain why a rejected role was rejected.

## Quality Gate

Before giving final recommendations, check:

- Are all apply-now roles live?
- Did the list include both big global companies and smaller startups where
  relevant?
- Did the list respect the concentric market logic?
- Did every recommendation name the evidence that makes the user a good fit?
- Did every weak or risky role get a concern, not a flattering gloss?
- Are private names and private project details removed or generalized?
- Is the next action concrete enough to do today?
- Is the proof level named honestly?
- Is the stop reason or coverage limit clear?

If the answer to any of these is no, fix the list before presenting it.
