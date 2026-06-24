# Qualified Commerce Research Design

Date: 2026-06-23

Anti-mirage expansion: 2026-06-24

## Product Contract

Qualified commerce research is a specialized Arcwell research workflow for
finding real-world purchasable, rentable, or bookable options that match the
user's private context and current constraints.

It is not a search-result summarizer. A candidate is recommendable only when the
run has checked the actual page or host surface that proves the relevant
variant is currently available.

Initial proof domain: UK fashion retail, especially shoes and garments where
size, material, comfort, style, price, and availability all matter.

Target expansion domains:

- retail products and marketplaces;
- flats and rooms to rent;
- flights, hotels, restaurants, and other booking inventory;
- local services or appointments where availability changes quickly.

The workflow should remain generic, but the first implementation should use
fashion because it stresses the hardest parts: private style context, exact
size variants, subjective quality, JS-heavy retailer pages, marketplaces,
comfort claims, returns, and broad option sets.

Current status: `Partial/Production Data Proof (bounded)`.

The durable local ledger exists for run config, exact-variant candidates,
selector-backed availability proofs, redacted context facts, verification attempts, report
judgments, host-supplied rendered-page checks, commerce source-card linkage,
compiled context packets, structured price/currency and delivery-caveat
extraction, gated commerce report artifacts, and a preserved proof harness.

Bounded production-data proof: on 2026-06-24, a disposable `ARCWELL_HOME`
recorded two Chrome DevTools rendered captures from live M&S UK pages:
Autograph White Sole Suede Loafers in visible M&S size `8½` and M&S Denim
Shirt in visible size `2XL`. Both were classified as selector-backed
exact-variant available, linked to source cards, extracted prices (`£55` and
`£35`), compiled a redacted
context packet, and produced an accepted report. Proof artifacts are under
`/tmp/arcwell-commerce-live-proof-20260624/*-final.json`.

This does not prove broad autonomous market discovery, 20+ option shopping,
marketplaces, rentals, flights, logged-in Chrome-profile paths, scheduled
workers, recovery/ops, or cross-retailer quality. Arcwell must not claim
qualified commerce research works end to end until those surfaces have their
own proof packets and status agreement.

Proof harness status: `scripts/commerce-research-production-proof --sample
--target-qualified 2 --min-recommended 2` locally proves the preserved harness,
report gate, source-card linkage, context packet, and fail-closed proof packet
shape. It is not production-data proof unless run with a real browser-capture
manifest.

## User-Visible Claim Ledger

### Claim 1: Qualified Option Discovery

User-visible claim:

Arcwell can search across a relevant market and return a broad list of options
that match the user's private context and hard constraints.

Exact inputs:

- user request, such as "soft-soled loafers in the UK";
- domain profile, such as fashion, rental, travel, or service;
- geography and freshness needs;
- target output count;
- privacy level and allowed context sources;
- provider/search/cost limits.

Exact outputs:

- candidate ledger with source URL, provider, title, price, geography,
  category/domain, search path, and status;
- source-family coverage summary;
- near-miss and blocked candidate list.

Durable state:

- research run;
- raw candidates;
- public source cards for public product/listing/review pages;
- private/redacted context packet artifacts;
- search/provider proof records.

Runtime surfaces:

- CLI/MCP read surfaces;
- Codex skill;
- research report;
- audit/proof packet;
- TODO/STATUS only after proof exists.

Refuting observations:

- the final report contains only search snippets or model guesses;
- candidates cannot be traced to durable source URLs;
- source-family coverage is absent;
- private context was ignored or invented;
- no durable rows/artifacts can be inspected after the run.

### Claim 2: Exact Availability Proof

User-visible claim:

Every main recommendation has page-level proof that the exact relevant variant
is available at the time checked.

Exact inputs:

- candidate URL;
- variant key, such as UK shoe size 8.5, shirt size XXL, date/occupancy, or
  rental move-in window;
- verification method: static fetch, rendered browser, Chrome profile, or
  manual user confirmation.

Exact outputs:

- availability proof row/artifact;
- checked timestamp;
- URL after redirects;
- visible evidence snippet or DOM/screenshot reference;
- availability state;
- caveats and confidence.

Durable state:

- same-run availability proof linked to candidate;
- screenshot/page snapshot when available;
- blocked or unknown proof if verification fails.

Refuting observations:

- a candidate appears in the main list with `unknown`, stale, wrong-size, or
  missing availability;
- the proof says a product is available because the product page exists, not
  because the variant is selectable/in stock;
- availability is inferred from a search snippet;
- a static fetch misses JS state and the run does not escalate or mark blocked.

### Claim 3: Context-Aware Fit

User-visible claim:

Arcwell uses the user's private context to rank and explain options without
leaking raw private data into public outputs.

Exact inputs:

- Arcwell memory/profile;
- Garderobe read context;
- approved browser history, screenshots, spreadsheets, emails, or other
  personal inputs;
- per-run answers to missing preference questions.

Exact outputs:

- bounded context packet;
- explicit/inferred/uncertain/missing context labels;
- scoring explanations that reference redacted context categories rather than
  raw private records.

Refuting observations:

- the run asks for facts already present in approved context;
- the report exposes raw wardrobe rows, emails, browser history, screenshots,
  addresses, account details, or unrelated personal data;
- ranking cannot explain which context mattered;
- inferred preferences are treated as certain facts.

### Claim 4: Evidence-Aware Quality And Comfort

User-visible claim:

When relevant evidence exists, Arcwell includes product/listing reviews,
specialist sources, and public discussion in scoring rather than relying only
on merchant copy.

Refuting observations:

- comfort, quality, commute, reliability, or trust claims are asserted without
  evidence labels;
- review evidence contradicts the recommendation and no skeptic note appears;
- merchant copy is treated as neutral proof.

### Claim 5: Honest Stop And Status

User-visible claim:

The report says what was checked, what failed, what remains uncertain, and why
the run stopped.

Refuting observations:

- "best available" is claimed without checked coverage;
- blocked pages disappear from the report;
- the run says "current/latest" without checked timestamps;
- docs/status imply operational capability before production-data proof.

## False-Done Traps

These are the easiest ways for this feature to become a mirage:

- a skill prompt exists but no candidate/availability schema exists;
- a browser screenshot exists but is not linked to the exact variant proof;
- a page title and price are extracted but size/date availability is missing;
- recommendations cite search snippets instead of inspected pages;
- a model ranks products before availability has been proven;
- private context is mentioned in prose but not compiled into an auditable
  context packet;
- Garderobe is queried directly, but raw wardrobe inventory leaks into wiki or
  source-card outputs;
- the system asks the user for size/style facts already present in approved
  memory/context;
- Chrome-profile escalation is possible in theory but not represented in
  blocked-state reporting;
- local fixtures pass, but no real UK retail run proves JS-heavy pages,
  marketplaces, and stock states;
- a live proof succeeds once but leaves no preserved proof packet;
- the final report has attractive prose but no disqualified near misses,
  blocked candidates, timestamps, or stop reason;
- STATUS/README says the feature works before severe tests and live proofs pass.

## Proof Levels

### Missing

No design, data model, or agent surface exists.

### Scaffold

Design/TODO/skill text exists. It may describe the intended workflow, but it
cannot support a user-visible capability claim.

Current state: `Partial/Local Proof`.

### Partial

Some surfaces exist, such as candidate storage, browser extraction, or a skill,
but at least one core claim is unproven: exact availability, private-context
boundedness, CLI/MCP parity, proof artifacts, or report audit.

Allowed language:

```text
Qualified commerce research is partially implemented. It can do X, but it
cannot yet be trusted for Y.
```

### Local Proof

Deterministic tests and fixtures prove:

- candidate and availability-proof storage;
- same-run validation;
- exact-variant availability checks;
- static-versus-rendered extraction differences;
- prompt-injection handling;
- private-context redaction;
- report exclusion of unverified candidates;
- CLI/MCP/skill parity in a disposable home.

Mocks and fixtures satisfy this level only.

### Production Data Proof

Authorized real market runs pass in a disposable or controlled Arcwell home.

For the first fashion gate, proof requires:

- real UK retailer and marketplace sources;
- configured Brave, Perplexity, and/or OpenAI search provider proof under cost
  policy;
- at least 80 raw candidates or an honest market-size stop reason;
- 20+ checked candidates when the market supports it;
- rendered-browser proof for JS-heavy pages;
- exact UK 8.5 loafer availability proof;
- exact clothing-size denim shirt availability proof;
- review/quality evidence where available;
- disqualified near misses and blocked pages preserved;
- no raw private-context leakage;
- final report audit accepts the output or blocks promotion with exact reasons.

### Operational

Production-data proof plus:

- resumable worker execution, if runs become long-running;
- retry/dead-letter handling for fetches and browser verification;
- policy/cost enforcement before provider or browser-heavy actions;
- ops/doctor visibility for healthy, stale, blocked, failed, partial, retrying,
  and unknown states;
- proof packet preservation;
- docs, TODO, STATUS, CLI, MCP, skill, and plugin cache freshness agree.

### Done

Operational, no known core proof gates remain, and regression/proof scripts are
part of the normal verification loop.

## Non-Negotiable Behavior

1. Availability is a claim that requires proof.

   A product, listing, flight, room, or appointment cannot be recommended unless
   the run records a checked availability state for the exact relevant variant.
   For shoes this means the selected UK size, such as UK 8.5. For clothing this
   means the user's relevant size. For flats, flights, and bookings it means the
   selected date/location/occupancy or equivalent inventory constraint.

2. Page-level proof is sufficient for v1.

   The workflow does not need to add items to basket or proceed into checkout.
   It should inspect the page, rendered DOM, visible controls, and page text
   for signals such as selectable sizes, crossed-out sizes, disabled controls,
   sold-out labels, unavailable dates, or booking calendars.

3. Private context is a feature, not an afterthought.

   The workflow should use Arcwell memory/profile, Garderobe, browser history,
   screenshots, spreadsheets, emails, and other approved personal context
   sources when available. It should infer preferences from evidence rather
   than forcing a heavy static configuration.

4. Private context remains permissioned and auditable.

   The run may use private context to guide search and scoring, but it must not
   leak raw wardrobe inventory, emails, browser history, measurements, or
   private notes into public wiki/source-card outputs by default. Private
   evidence should be referenced through redacted local artifacts or scoped
   context summaries.

5. Ambiguity becomes a memory opportunity.

   If a run cannot infer a material preference, size, geography, marketplace
   tolerance, comfort priority, or budget range, it should ask a concise
   question at the start or at the decision point. When the user answers, the
   system should offer to store that preference in the appropriate memory or
   profile surface for future runs.

6. Broad output is intentional.

   Commerce matching is hit-and-miss. The default final output should target
   20+ qualified candidates when the market has enough options, not a tiny
   top-three list. Ranking should help the user scan, but variety and near
   misses are part of the value.

## Relationship To Existing Arcwell Systems

Qualified commerce research should reuse the existing deep-research and
source-card substrate rather than becoming a parallel shopping bot.

- Deep research provides durable run state, search proof, source cards, claims,
  skeptic passes, audits, and report artifacts.
- Source cards represent public product/listing/review/evidence pages.
- Research artifacts represent private context summaries, browser snapshots,
  screenshots, availability proofs, disqualification notes, and final reports.
- Garderobe remains the private wardrobe source of truth. The workflow may read
  it for style and fit context, but Arcwell memory/wiki should not ingest raw
  wardrobe inventory by default.
- Browser tooling is a first-class fetch path for JS-heavy pages, not a fallback
  after the answer has already been drafted.
- Existing Brave, Perplexity, and OpenAI search providers should be reused
  behind the same cost and policy gates as deep research.

## Domain Profile Contract

Qualified commerce research should be generic, but every domain needs an
explicit profile. A profile defines:

- variant key shape;
- hard filters;
- soft scoring dimensions;
- source families;
- browser verification strategy;
- freshness window;
- blocked/private surfaces;
- proof packet requirements;
- domain-specific severe fixtures.

No domain should be promoted from `Scaffold` simply because the generic schema
exists. The first domain profile to reach proof should be UK fashion retail.

### UK Fashion Retail Profile

Variant keys:

- shoes: category, size system, size, width when known, color/material when
  relevant;
- garments: category, size, fit, color, material, gender/department when
  relevant;
- marketplaces: listing id, seller, item condition, size, color/material,
  location/shipping state.

Hard filters:

- exact relevant size is available;
- ships to the user's geography or is locally collectible when allowed;
- not outside explicit max budget unless kept as a disqualified near miss;
- not checkout/account/payment-only proof;
- not unsupported by page evidence.

Soft scoring:

- style fit to memory/profile/Garderobe context;
- comfort, stability, shock absorption, and mobility fit when requested;
- material and construction quality;
- review evidence strength;
- wardrobe compatibility;
- return policy and retailer trust;
- price/value fit;
- uniqueness versus duplicate options.

Freshness:

- retail availability proof should be treated as same-day evidence by default;
- marketplace listings should be considered short-lived and should be labeled
  stale sooner than ordinary retailer pages;
- final reports must display `checked_at` for every main recommendation.

### Rental Profile

Variant keys:

- listing id, location, rent, move-in window, bedroom count, furnished state,
  accessibility constraints, commute target, and viewing availability.

Hard filters:

- listing is active at page check time;
- rent/deposit/council-tax/service-fee assumptions are explicit;
- location and commute constraints are represented;
- scams, stale listings, and agent-only placeholders are disqualified or marked
  blocked.

Scoring:

- commute and transport reliability;
- accessibility and mobility fit;
- light/noise/layout signals;
- cost realism;
- agent/listing trust;
- viewing availability;
- neighbourhood fit and safety/context evidence where available.

### Travel Booking Profile

Variant keys:

- origin, destination, date or date window, passenger count, cabin, bags,
  flexibility/refund needs, and arrival/departure time constraints.

Hard filters:

- fare or booking option is visible at page check time;
- price includes required baggage/fees when possible or caveats are explicit;
- connection and airport-change risks are visible;
- no account/payment step is required for proof.

Scoring:

- schedule fit;
- price and total-cost caveats;
- connection risk;
- baggage/refund flexibility;
- airline or operator reliability evidence;
- disruption risk and airport logistics.

Rental and travel profiles should stay `Scaffold` until fashion proves the
generic candidate/availability/context/report shape.

## Execution State Machine

Every run should move through explicit states. A report cannot skip states even
if an agent can produce plausible prose.

```text
created
  -> context_compiled
  -> source_map_created
  -> candidates_collected
  -> candidates_triaged
  -> browser_verification_running
  -> availability_proven
  -> scored
  -> skeptic_checked
  -> report_compiled
  -> report_audited
  -> accepted | blocked | stopped_incomplete
```

Blocked states should be durable:

- `context_blocked`: private context source unavailable or consent missing;
- `search_blocked`: provider missing, policy denied, cost denied, or query
  failed;
- `verification_blocked`: page inaccessible, bot challenge, JS failure, or
  Chrome-profile escalation needed;
- `privacy_blocked`: raw private data would leak into output;
- `quality_blocked`: insufficient proof for main recommendations;
- `audit_blocked`: report contains unverified claims or generated-evidence
  recursion.

State transitions should be idempotent. Rerunning a phase should update or
append proof history without deleting prior contradictory evidence.

## Evidence Model

The core new artifact is a checked option.

```text
commerce_candidate
  run_id
  domain: fashion | rental | travel | service | other
  source_url
  retailer_or_provider
  title
  normalized_item_key
  price
  currency
  geography
  candidate_status: qualified | disqualified | maybe | blocked
  score
  score_reasons
  disqualification_reasons
```

```text
availability_proof
  run_id
  candidate_id
  checked_at
  proof_method: static_fetch | rendered_browser | chrome_profile | manual_user
  variant_key
  variant_label
  availability_state: available | unavailable | unknown | blocked
  visible_evidence
  selector_or_dom_hint
  screenshot_artifact_id
  page_snapshot_artifact_id
  confidence
  caveats
```

Additional implementation records:

```text
commerce_run_config
  run_id
  domain_profile
  target_qualified_count
  geography
  freshness_window
  allowed_private_context_sources
  allowed_public_source_families
  allow_marketplaces
  allow_chrome_profile
  max_provider_calls
  max_browser_pages
  max_cost_usd
  stop_rules_json
```

```text
commerce_context_fact
  run_id
  fact_key
  fact_kind: explicit | inferred | uncertain | missing
  redacted_value
  source_family
  source_ref
  confidence
  user_confirmed
  may_persist_to_memory
```

```text
commerce_verification_attempt
  run_id
  candidate_id
  attempted_at
  method
  result: available | unavailable | unknown | blocked | error
  error_kind
  final_url
  http_status
  browser_required
  chrome_profile_required
  artifact_ids
  next_action
```

```text
commerce_report_judgment
  run_id
  decision: accept | hold | block
  blocking_findings
  non_blocking_findings
  claims_checked
  availability_proofs_checked
  privacy_review
  remaining_risks
```

Use research artifacts for early implementations if adding dedicated tables is
too expensive, but do not promote beyond `Partial` until the shape is typed
enough for same-run validation, CLI/MCP reads, and severe tests.

For the loafer example, the variant key might be:

```text
category=shoe; size_system=UK; size=8.5; color=brown; width=unknown
```

For a denim shirt:

```text
category=shirt; size=XXL; color=mid-blue; fit=regular
```

For a flat:

```text
city=London; neighbourhood=...; move_in_window=...; bedrooms=...; budget=...
```

For a flight:

```text
origin=...; destination=...; depart_date=...; return_date=...; cabin=...
```

## Context Model

Each run should compile a bounded private context packet before searching. The
packet is an artifact, not a permanent public source card.

Context families:

- body and sizing facts: shoe size, shirt size, trouser size, fit caveats;
- style preferences: colors, silhouettes, brands, logo tolerance, materials;
- budget class and price sensitivity;
- accessibility and mobility constraints;
- geography, shipping, travel, and local-market constraints;
- known dislikes, blocked retailers, blocked brands, and ethical constraints;
- wardrobe gaps and compatibility notes from Garderobe;
- prior purchases, returns, screenshots, spreadsheets, and relevant emails when
  available and authorized.

The packet should distinguish:

- explicit user facts;
- inferred preferences with evidence;
- uncertain guesses;
- missing facts that would materially improve the search.

## Workflow

### 1. Scope And Context

Start a durable research run with domain, geography, target quantity, hard
filters, soft preferences, privacy level, and cost/search limits.

Compile private context. For the first UK fashion implementation, this should
include:

- size facts, such as UK 8.5 shoes or clothing sizes;
- mobility and comfort priorities, such as stability and shock absorption;
- style direction, such as Ivy-compatible and low-logo;
- budget class and avoided quality bands;
- existing wardrobe compatibility.

Ask only when a missing fact changes the search materially.

### 2. Source Map

Build a market/source map before deep inspection.

Fashion source families:

- brand-owned shops;
- UK department stores and menswear stockists;
- specialist shoe or clothing retailers;
- resale and marketplace sources such as eBay and Vinted when allowed;
- review sites, forum discussions, Reddit, blogs, and comfort/fit reviews;
- previous purchase/order/email evidence where relevant.

Other domains should define their own source families. Rentals may include
letting agents, portals, local listings, transport maps, crime/noise/flood
context, and availability calendars. Flights may include airline sites,
aggregators, fare calendars, baggage rules, and disruption/reliability signals.

### 3. Broad Search

Use configured Arcwell search providers and host search. Generate enough
candidates to support a broad final list. For fashion, a useful target is 80 to
150 raw candidates before dedupe for a 20+ qualified output.

Store raw candidates with retrieval path, query, source family, geography, and
reason selected.

### 4. Triage And Dedupe

Normalize product/listing keys and collapse exact duplicates while preserving
source-specific availability. Disqualify obviously wrong categories, geography,
price bands, fake/outdated pages, and irrelevant sizes, but preserve the reason.

Do not discard near misses too early. Many useful final reports need a section
for "almost right, failed because...".

### 5. Browser Verification

For each promising candidate, inspect the live page. Prefer the Codex in-app
browser for speed and isolation. Escalate to the user's real Chrome profile
only when cookies, login state, region personalization, or bot friction makes
that materially better.

The browser verifier must capture:

- URL after redirects;
- timestamp;
- visible product/listing title;
- selected or inspected variant;
- visible availability signal;
- price and currency;
- geography/shipping or booking caveat;
- screenshot or page snapshot when possible;
- blocked/ambiguous state when verification fails.

For v1, "page says UK 8.5 is selectable/in stock" is enough. No basket or
checkout action is required.

### 6. Qualification And Scoring

Score only after availability proof exists or has honestly failed.

Fashion scoring dimensions:

- exact size availability;
- style fit to private context;
- comfort/material evidence;
- quality and durability expectation;
- price fit;
- wardrobe compatibility;
- returns and shipping risk;
- review/supporting evidence strength;
- brand/retailer trust;
- novelty or distinctiveness within the option set.

Domains should define equivalent scoring dimensions. Rentals might score
commute, accessibility, cost, availability date, natural light, noise risk, and
agent trust. Flights might score schedule, price, baggage, refundability,
connection risk, and airline reliability.

### 7. Skeptic Pass

Before final output, the workflow should try to invalidate each recommended
candidate:

- Is the relevant variant actually available?
- Is the page stale, marketplace-expired, or region-blocked?
- Did search find a cheaper duplicate from the same reputable source?
- Does a review contradict the comfort or quality assumption?
- Is the size system ambiguous?
- Is the candidate outside the inferred style or budget class?
- Did private context get over-applied from weak evidence?

Candidates that fail should move to disqualified or maybe, not remain in the
main list.

### 8. Final Output

The report should be broad and scan-friendly.

Recommended sections:

- top candidates, grouped by best match / good alternatives / riskier but
  interesting;
- table with retailer, item/listing, price, relevant variant, availability,
  proof timestamp, why it fits, and caveats;
- disqualified near misses with exact reason;
- blocked/unverified candidates that looked promising;
- inferred preferences used;
- questions worth storing for future searches;
- stop reason and coverage summary.

Every available recommendation should carry a checked timestamp and proof
method. Unknown availability is not acceptable in the main recommendation list.

## Privacy And Storage

Public pages can become source cards. Private context should become scoped,
redacted artifacts unless the user explicitly asks to preserve the raw input.

Allowed by default:

- source cards for public product/listing/review pages;
- redacted context packet;
- availability proof with public URL, variant, and timestamp;
- screenshots of public product pages when useful;
- final report.

Not allowed by default:

- raw wardrobe inventory rows in Arcwell wiki;
- raw emails, browser history, spreadsheets, or screenshots that contain private
  unrelated details;
- checkout, account, address, or payment pages;
- automated purchase, booking, contact, bid, message, or add-to-basket actions.

## Skill Surface

Possible initial skill:

```text
$qualified-commerce-research
```

The skill should be a guided wrapper over deep research, not a new isolated
runtime. It should know the domain profile, context sources, proof requirements,
and output shape.

Example invocations:

```text
Find me soft-soled loafers in the UK, available in UK 8.5, that fit my style.
```

```text
Find me a denim shirt that fits my wardrobe, size, and quality preferences.
```

```text
Find available flats to rent that match my commute and accessibility needs.
```

```text
Find flights for this trip, but verify actual available fare options.
```

## CLI, MCP, And Skill Surface

Do not expose a confident skill before the underlying read/write surfaces can be
inspected.

### CLI

Current local ledger commands are intentionally explicit:

```sh
arcwell commerce capabilities
arcwell commerce config-set <run-id> --domain-profile uk-fashion-retail --target-qualified 20
arcwell commerce candidate-add <run-id> --url <url> --title <title> --variant-key <key>
arcwell commerce rendered-page-check <run-id> <candidate-id> --variant-key <key> --variant-label "UK 8.5"
arcwell commerce context-fact-add <run-id> --fact-key shirt_size --redacted-value XXL
arcwell commerce context-packet <run-id>
arcwell commerce report <run-id>
```

Higher-level wrappers such as `arcwell commerce run/status/proofs/audit/stop`
remain future work. The current surface should claim only durable ledger writes,
host-supplied rendered-page checks, source-card linkage, context packets, and
gated reports.

### MCP

Current MCP tools mirror the ledger surface:

- `commerce_research_capabilities`
- `commerce_run_config_set`
- `commerce_candidate_add`
- `commerce_availability_proof_add`
- `commerce_rendered_page_check`
- `commerce_context_fact_add`
- `commerce_context_packet_compile`
- `commerce_research_report`

Higher-level tools such as `commerce_research_run/status/proofs/audit/stop`
remain future wrappers over the same ledger.

Capabilities should report proof level per domain profile:

```json
{
  "qualified_commerce": {
    "status": "partial_bounded_production_data_proof",
    "profiles": {
      "uk-fashion-retail": "bounded_two_item_mands_proof",
      "rental": "missing",
      "travel": "missing"
    },
    "browser_rendered_extraction": "host_supplied_local_check_proven",
    "exact_variant_availability_proof": "locally_proven",
    "private_context_packet": "locally_proven_redacted_artifacts",
    "bounded_live_uk_fashion_packet": "production_data_proven_for_two_mands_pages",
    "broad_production_data_proof": false
  }
}
```

### Skill

The Codex skill should remain conservative:

- retrieve `commerce_research_capabilities` first;
- refuse to imply production readiness when capability status is below the
  requested proof level;
- ask only for missing facts that materially change the run;
- call the run/status/candidate/proof surfaces rather than free-forming a
  shopping answer;
- include blocked/unknown/disqualified items in the final report;
- require a fresh-thread/plugin-sync smoke before claiming the skill surface is
  available in Codex.

## Browser Verification Contract

The browser verifier is the highest-risk component because it can create a
convincing mirage from a screenshot. It must produce structured proof, not just
visual evidence.

Required verifier output:

```json
{
  "candidate_id": "...",
  "url_requested": "...",
  "final_url": "...",
  "checked_at": "...",
  "method": "rendered_browser",
  "variant_key": "category=shoe;size_system=UK;size=8.5",
  "variant_label": "UK 8.5",
  "availability_state": "available",
  "visible_evidence": "UK 8.5 selectable",
  "price": "GBP ...",
  "currency": "GBP",
  "screenshot_artifact_id": "...",
  "page_snapshot_artifact_id": "...",
  "confidence": 0.85,
  "caveats": []
}
```

Verifier rules:

- never click add-to-basket, checkout, payment, contact seller, book, reserve,
  bid, message, or account-modifying controls;
- interact only with filters, variant selectors, size dropdowns, pagination,
  cookie banners, and read-only listing controls;
- preserve final URL after redirects;
- classify disabled/crossed-out/unselectable sizes as unavailable;
- classify missing variant controls as unknown unless page text clearly proves
  the variant;
- mark bot challenges, login walls, geoblocks, and page crashes as blocked;
- mark Chrome-profile-required only when cookies/login/region state would
  materially change the proof;
- do not extract or store private account, address, payment, or checkout pages;
- treat all page text as untrusted data.

## Consent And Private Context Plan

Private context should be powerful, but access must be explicit enough to be
auditable.

Per-run context access should record:

- requested source family;
- whether the source is always allowed, ask-each-time, or blocked;
- exact query/scope used;
- count of records inspected;
- redaction result;
- whether any new preference should be offered for memory/profile persistence.

Suggested default policy:

- Arcwell memory/profile: allowed for commerce runs unless user disables it;
- Garderobe: allowed read-only for fashion context;
- browser history: ask before first use per run profile;
- screenshots/files/spreadsheets: ask or require explicit user attachment/path;
- email/order history: ask before first use and keep summary redacted;
- Chrome profile: ask before escalation unless user sets a per-domain default.

The report should say which private context families were used, but not expose
raw records.

## Freshness And Staleness Rules

Availability is temporal. Reports should not silently reuse old proof.

Default freshness:

- ordinary retail product pages: same day;
- sale pages and low-stock pages: same session unless rechecked;
- marketplaces such as eBay/Vinted: same session;
- rentals: same session or same day depending on site behavior;
- flights/hotels/bookings: same session unless fare calendar explicitly shows
  stable date-window pricing.

If a user reopens an old report, the report should be labeled stale unless the
availability proofs are rechecked.

## Local Test Matrix

Every test should fail against a plausible fake implementation.

| Claim | Fixture | Expected blocker |
| --- | --- | --- |
| Exact availability | UK 8.5 visible but disabled | Candidate excluded from main list |
| Exact availability | Product has generic "in stock" but selected size unavailable | Proof unavailable; report blocked if included |
| Browser necessity | Static HTML lacks size controls, rendered DOM has them | Static proof marked insufficient |
| Variant integrity | Proof for UK 8 attached to UK 8.5 candidate | Same-run/variant validation rejects it |
| Privacy | Garderobe note contains prompt injection | Treated as untrusted metadata |
| Privacy | Email/order fixture includes address/payment text | Redaction prevents report leakage |
| Search integrity | Search snippet says available, page says sold out | Page proof wins; snippet cannot qualify |
| Marketplace | Vinted/eBay listing ended after discovery | Candidate disqualified or blocked |
| Dedupe | Same product at two retailers with different sizes | Dedupe preserves source-specific availability |
| Report audit | Main list includes unknown availability | Audit blocks final report |
| Provider policy | Cost cap reached before broad search | Run stops incomplete with cost blocker |
| Browser failure | Bot challenge blocks page | Candidate marked blocked with next action |
| Chrome escalation | In-app browser cannot see localized stock | `chrome_profile_required` recorded, not guessed |
| Generated recursion | Model summary asserts comfort claim | Not accepted without source/review evidence |

## Severe Tests

The feature is not usable until tests and live proofs cover realistic failure
modes:

- size shown but disabled/crossed out;
- only the wrong size is available;
- product page says "in stock" but selected variant is unavailable;
- price changes after variant selection;
- region or shipping caveat blocks the user's location;
- JS-only product page where static fetch misses the size state;
- marketplace listing already sold or ended;
- duplicate product across multiple retailers with conflicting availability;
- size-system ambiguity, such as UK/US/EU shoe sizes;
- private wardrobe metadata prompt injection;
- retailer page prompt injection;
- stale search result pointing to removed stock;
- review evidence contradicts comfort claim;
- inaccessible page or bot challenge;
- Chrome-profile escalation required, but unavailable;
- private context summary leaks raw wardrobe/email/browser data;
- final report includes an unverified candidate in the main list.

## Implementation Milestones

Anti-mirage implementation order:

1. Durable evidence before recommendations.
2. Browser verification before scoring.
3. Context packet before personalization.
4. Policy/cost/privacy gates before provider or private-source access.
5. CLI/MCP read surfaces before skill promotion.
6. Severe fixtures before live proof.
7. Live proof packet before STATUS/README promotion.

Milestones:

1. Design and TODO only.

   Capture the workflow, evidence model, privacy boundary, and proof gates.

2. Browser-rendered extraction.

   Add the general deep-research capability for browser-rendered JavaScript
   extraction, with proof artifacts and blocked-state reporting. The extractor
   must preserve source text as untrusted data and must not promote a static
   page title, generic "in stock" label, or selector-less flattened text into
   recommendation-grade exact-variant availability.

3. Commerce data model.

   Add candidate and availability-proof artifacts or tables, with CLI/MCP read
   surfaces and same-run validation. Invalid cross-run proof attachment,
   missing variant keys, malformed availability states, and duplicate proof
   rows must fail closed or preserve a clear conflict state.

4. Context packet compiler.

   Compile bounded private context from memory/profile/Garderobe and later
   approved browser history, screenshots, spreadsheets, and emails. Start with
   fashion context. The compiler must label explicit, inferred, uncertain, and
   missing facts, and must produce redacted artifacts suitable for reports.

5. Fashion v1 skill.

   Implement a Codex skill for UK fashion searches using the generic commerce
   model, broad output, browser verification, and skeptic disqualification.
   The skill must reject final output when the main list contains unverified
   candidates.

6. Severe local fixtures.

   Build static and rendered-page fixtures for the failures above.

7. Preserved proof harness.

   Drive the public CLI through `scripts/commerce-research-production-proof`,
   producing an inspectable proof packet and failing non-zero when release
   gates are not met.

8. Live proof.

   Run a preserved proof on UK loafers in UK 8.5 and a denim shirt search,
   with 20+ checked candidates where the market supports it.

9. Generalize domains.

   Add rental and travel domain profiles only after the fashion proof exposes
   the stable abstraction.

## Proof Packet

Every promotion beyond `Scaffold` needs a preserved proof packet.

Required fields:

- feature name and proof level;
- user-visible claims covered;
- exact request, domain, geography, and target count;
- private context sources allowed and actually used;
- redaction/privacy review result;
- search providers and host/browser tools used;
- cost/policy decisions;
- raw candidate count and source-family counts;
- dedupe count and conflict handling;
- checked candidate count;
- availability proof count by method;
- blocked/unknown/unavailable/disqualified counts;
- screenshots/page snapshots reviewed;
- review/quality/comfort evidence count;
- final report artifact id/path;
- audit result;
- CLI/MCP/skill surfaces exercised;
- tests run;
- live commands run;
- artifacts reviewed manually;
- promotion judgment: promote, hold, or block;
- remaining risks and exact next action.

Implemented local replay harness:

```sh
scripts/commerce-research-production-proof \
  --sample \
  --target-qualified 2 \
  --min-recommended 2
```

Production-data fashion release gate:

```sh
scripts/commerce-research-production-proof \
  --manifest .arcwell-dev/proofs/commerce-uk-fashion-20/manifest.json \
  --profile uk-fashion-20 \
  --target-qualified 20 \
  --min-recommended 20 \
  --require-production-data
```

Marketplace gate:

```sh
scripts/commerce-research-production-proof \
  --manifest .arcwell-dev/proofs/commerce-marketplaces/manifest.json \
  --require-production-data \
  --require-marketplace
```

Logged-in Chrome-profile gate:

```sh
scripts/commerce-research-production-proof \
  --manifest .arcwell-dev/proofs/commerce-chrome-profile/manifest.json \
  --require-production-data \
  --require-chrome-profile
```

Rental and flight gates:

```sh
scripts/commerce-research-production-proof \
  --manifest .arcwell-dev/proofs/commerce-rental/manifest.json \
  --require-production-data \
  --require-domain rental

scripts/commerce-research-production-proof \
  --manifest .arcwell-dev/proofs/commerce-flight/manifest.json \
  --require-production-data \
  --require-domain travel
```

Operational worker gate:

```sh
scripts/commerce-research-production-proof \
  --manifest .arcwell-dev/proofs/commerce-worker/manifest.json \
  --require-production-data \
  --require-worker-proof
```

The proof script exits non-zero when blockers remain. A preserved proof with a
blocking result is still valuable evidence, but it does not promote the feature.

## Report Acceptance Gate

A final report is acceptable only when all of these are true:

- every main recommendation has exact-variant availability proof with visible
  evidence and artifact provenance;
- unknown, blocked, wrong-size, stale, and unverified candidates are outside
  the main recommendation list;
- each candidate has a checked timestamp and proof method;
- private context used by ranking is summarized without raw private leakage;
- scoring separates hard filters, soft preferences, and evidence quality;
- review/comfort/quality claims are labeled by evidence strength;
- near misses and disqualified candidates include exact reasons;
- source-family coverage and stop reason are explicit;
- generated summaries and model prose are not treated as evidence;
- audit can trace claims to source cards, availability proofs, or redacted
  context artifacts.

## Ops And Recovery Requirements

Commerce research should stay manual until the proof path is reliable. If it
becomes long-running or scheduled, promotion to `Operational` additionally
requires:

- worker job state with leases and resumable browser-verification steps;
- retry and dead-letter behavior for blocked pages, provider errors, browser
  crashes, and partial writes;
- cost reservations or caps before broad provider search and browser-heavy
  verification;
- source-health or equivalent provider-health state for retailers/providers
  where currentness is claimed;
- ops snapshot visibility for running, blocked, partial, stale, failed,
  retrying, and completed runs;
- idempotent reruns that do not duplicate candidates or overwrite proof history;
- user-stop behavior before the next expensive or privacy-sensitive action.

## Release Road

Qualified commerce can move toward production in independent lanes, but public
claims must name the lanes that have passed:

1. `Partial/Production Data Proof (bounded)`: current two-item live browser
   packet plus local replay proof harness.
2. `UK fashion broad production proof`: at least 20 exact-variant main
   recommendations from a real manifest, or an explicit market-scarcity blocker
   with broad checked evidence.
3. `Marketplace proof`: eBay/Vinted-style listing manifests prove sold/ended
   rejection, condition/seller capture, and freshness caveats.
4. `Chrome-profile proof`: supervised logged-in browser captures prove
   `chrome_profile` verification without leaking private page data into public
   artifacts.
5. `Rental/travel proof`: domain profiles pass exact availability semantics for
   rentals and flights before those domains are advertised.
6. `Operational`: queued discovery/check/report work passes worker, retry,
   dead-letter, idempotency, stop, cost, and ops visibility gates.

## Open Design Questions

- How should Arcwell request and record consent before using browser history,
  emails, screenshots, and spreadsheets for a specific run?
- Should Chrome-profile escalation be explicit every time, or controlled by a
  per-domain preference?
- How long should availability proofs be considered fresh?
- Should public product screenshots be stored by default, or only when the DOM
  signal is ambiguous?
- What is the right data shape for marketplace listings where the item itself
  is unique and can disappear quickly?
- Which private context facts should be memory/profile facts versus Garderobe
  facts versus per-run preferences?
