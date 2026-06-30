---
name: technical-blog-content
description: Use when writing, rewriting, auditing, selecting, or publishing practitioner-facing technical blog posts, especially for chabot.dev, public project write-ups, agent systems, developer tools, infrastructure, research pipelines, DevRel, and credibility-sensitive engineering writing.
---

# Technical Blog Content

Write technical posts that a serious practitioner would keep reading.

This skill exists because a previous approach failed. It turned projects into
thin explanations of their own parts: commands, test names, object inventories,
and vague claims about why the work mattered. That is not a technical article.
It is workshop exhaust.

The article starts only after the real subject is found.

## Non-Negotiable Standard

Do not draft the post until you can answer this in one sentence:

> A reader starts believing or not seeing X; by the end, evidence makes Y
> harder to deny.

If that sentence is weak, the post is not ready.

Bad:

- "Granite is an Obsidian-compatible Markdown app."
- "Arcwell clusters sources before writing reports."
- "valkey-bun implements Redis locally."
- "code-search combines BM25 and grep."

Better:

- "Obsidian compatibility is not whether a note is plain text; it is whether a
  mixed vault with links, Canvas files, Bases, metadata, plugins, and derived
  state can stay portable while another app works on it."
- "One model launch looked different depending on whether the reader saw only
  the official announcement, the press framing, the launch thread, or the
  developer reaction. The cluster is what made that difference visible."
- "A local Redis replacement earns trust only when ordinary clients can use it
  without changing their assumptions about sessions, pub/sub, transactions, and
  binary data."
- "BM25 is useful to agents before they know the repo's vocabulary; exact
  search becomes useful after the first ranked pass teaches them the names."

Those are possible claims, not automatic claims. Use only what the source
material proves.

## Required Pre-Draft Brief

Before writing public prose, create a brief. It can be internal for small edits,
but it must exist. If the brief does not pass, keep the post in draft or ask for
more source material.

The brief must include:

1. **Subject**
   The actual technical problem or practice. Not the project name.

2. **Reader**
   The practitioner who would care, and what they already understand. Do not
   write remedial intros for senior readers, but do give enough context that
   insider names are not doing hidden work.

3. **Starting Belief**
   The plausible but incomplete belief the reader or builder might begin with.

4. **Pressure**
   The friction, failure, constraint, ambiguity, cost, or competing evidence
   that makes the subject worth writing about.

5. **Changed Belief**
   What the evidence makes clearer by the end.

6. **Evidence Chain**
   A sequence, not a pile:
   signal -> check -> mechanism -> output -> decision or limit.

7. **Receipt**
   The artifact that proves the claim for this article. A receipt is the thing
   the reader can inspect, not a sign that the writer did work.

8. **Mechanism**
   The code, architecture, command, schema, or workflow that explains how the
   receipt happened. Mechanism enters after the problem is alive.

9. **Boundary**
   What the post does not prove, and why that limit matters to the claim.

10. **Do Not Write**
    The tempting but wrong angles, phrases, or details that would dilute the
    piece.

If any field is vague, stop. Do not compensate with better prose.

## Editorial Research Gate

Use a lightweight version of the Arcwell deep-research pipeline before drafting
or publishing. The aim is not to turn every post into a formal research report.
The aim is to stop unsupported, boring, or self-referential claims before they
become prose.

Run these phases:

1. **Source Map**
   List the source families that can prove or weaken the post: public repo,
   README, issue, fixture, generated output, benchmark, source set, external
   source, user history, Codex history, Claude history, screenshot, live page,
   state row, or report. Mark each as public, private, publishable,
   non-publishable, primary, or secondary.

2. **Claim Ledger**
   Extract the important factual and interpretive claims the post wants to
   make. Keep them short. Each claim needs an evidence pointer or it is not
   allowed in the draft.

3. **Receipt Match**
   Identify the one receipt that would make the article worth reading. If there
   are many small receipts and no central one, the likely article is still a
   project note, not a public essay.

4. **Skeptic Pass**
   Attack the claims before drafting. Look for:
   - a simpler explanation that makes the claim uninteresting
   - a competing tool that already does the same thing
   - a private fact that cannot be published
   - a test that proves less than the prose wants it to prove
   - a benchmark or fixture that is too narrow
   - missing primary-source evidence
   - a claim that sounds impressive only because it hides context
   - stale dates, invented causality, or unsupported market reads

5. **Refutation Search**
   For external, market, product, standards, or news claims, search for
   contradiction and criticism. Prefer primary sources first, then credible
   secondary analysis. If the topic may have changed, verify it live.

6. **Revision Decision**
   Mark each claim as `survived`, `weakened`, `contradicted`, or `unresolved`.
   Revise the brief around survived claims. Weakened claims may stay only with
   the caveat attached. Contradicted or unresolved high-impact claims must be
   removed or send the post back to source gathering.

7. **Adversarial Editorial Review**
   Before publication, reread the draft as the smartest skeptical reader likely
   to see it. Ask what would make them close the tab:
   - the opening explains something obvious
   - the post claims novelty where there is none
   - the receipt is a test run instead of the thing under discussion
   - the prose withholds context needed to care
   - the project is presented as inherently important
   - private source notes leak into public claims
   - the piece is accurate but has no status change

If the adversarial review fails, do not polish. Rewrite the brief or move the
post to draft.

For serious research, market, launch, standards, or fast-moving posts, use the
full `arc:deep-research` and `arc:research-audit` workflows instead of this
lightweight gate. The blog post can be written from the accepted report, but the
report itself remains the evidence base.

## What A Post Is About

A project is usually not the subject.

The subject is the tension the project exposed:

- a local substitute crossing a real client boundary
- a noisy source set becoming one report
- a file format remaining portable under app pressure
- a dashboard changing a budget or priority
- an agent boundary refusing a dangerous shortcut
- a benchmark changing a product decision
- a runtime handoff proving that state survived the move
- a UI click invalidating a claimed implementation

The project can be the setting, tool, or proof. It is rarely the story by
itself.

## What Counts As A Receipt

A receipt is claim-aligned evidence. It is not any artifact near the work.

Strong receipts include:

- a generated report that is better because multiple source families were read
  together
- a real client, provider, user, fixture, benchmark, trace, state row,
  screenshot, source bundle, or output crossing the boundary under discussion
- a before/after behavior a reader can understand without private context
- a source set where each source family contributes a different part of the
  interpretation
- a failed assumption that changed the implementation or product decision
- a limitation that changes how confidently the reader should use the result

Weak receipts include:

- "I cloned the repo"
- "I ran tests"
- "tests passed"
- "the project compiled"
- "this command exists"
- "there are objects called X, Y, and Z"
- "the code lives in these files"
- "the implementation has a policy gate"

Tests can be receipts only when the article is about test design, rejection
behavior, compatibility boundaries, or a failure a test made visible. Otherwise
they belong in a footnote, README, release note, or local verification log.

## Opening Rules

The opening must give the reader a reason to care before it names internals.

Do not open with:

- a project pitch
- a command
- a test run
- a timestamp unless the time itself changes the story
- "the hard part is" without naming a real hard thing
- "built around that claim" when the claim is still abstract
- "not just X, but Y" before X has failed on the page
- a generic maxim about software, AI, agents, or local-first work

Good openings usually do one of these:

- name the real operating situation
- show a signal that one view failed to explain
- show a caller crossing a boundary
- show an object that changed status
- show a constraint that removed an easy answer
- show the artifact that made the author's belief change

The first 200 words must answer "so what?" in concrete terms. A senior reader
may already know the domain; they still need to know why this article exists.

## Evidence Movement

A good post moves. It does not merely describe.

The reader should be able to follow a chain:

1. Here is the situation.
2. Here is the incomplete or misleading read.
3. Here is the check or source that changed the read.
4. Here is the mechanism that made the check possible.
5. Here is the output or behavior.
6. Here is what we can now decide, trust, reject, or watch.

If paragraphs can be rearranged without harming the argument, the draft is
probably a catalogue.

If the object has the same status at the end as it had at the beginning, the
post has described it instead of developing it.

## Mechanism Belongs After Need

Code, schemas, commands, tests, and architecture matter when they answer a
question the reader now has.

Use mechanism to explain:

- how source material became a cluster
- how a local service satisfied a real client expectation
- how derived state stayed disposable
- how a compatibility check found a format edge
- how an agent boundary changed behavior
- how a metric changed a decision

Do not use mechanism to prove diligence.

Before adding a command, ask:

- What reader question does this answer?
- What claim would weaken if this were removed?
- Is this the receipt, or only the workshop trail?

## Article Shapes

Use a shape that fits the material. Do not publish the outline as visible
scaffolding.

### Event To System

For source ingestion, monitoring, clustering, reporting, market reads, and
research pipelines.

Start with a real event or source bundle. Show why a single source gave an
incomplete read. Explain ingestion, normalization, dedupe, clustering, review,
generation, and delivery only as needed to understand how the event became a
better report. End with the report, the decision it supports, and what remains
unverified.

The receipt is usually the source bundle and the resulting report.

### Caller To Contract

For local substitutes, protocols, SDKs, APIs, and compatibility work.

Start with the caller or artifact that expects a contract: a Redis client, an
Obsidian vault, a Worker binding, a browser, a CLI, a webhook. Show what the
caller assumes. Then show where the implementation satisfies, bends, or refuses
that contract.

The receipt is usually the caller interaction, fixture, trace, state row,
round-trip, or before/after behavior.

### Incident To Rule

For bugs, security, reliability, UX, and agent behavior.

Start with the symptom. Show the check that changed the interpretation. Show
the repair or boundary. End with the rule earned by the incident.

The receipt is usually the failure, the root-cause evidence, and the repaired
behavior.

### Comparison With Consequence

For benchmarks, product strategy, DevRel measurement, and technical choices.

Do not list dimensions. Show the moment the comparison changes a decision:
budget, adoption path, API design, documentation, operational watch list, or
engineering priority.

The receipt is usually the metric, benchmark, or case that changed the decision.

## Worked Examples

### News Clustering

Weak angle:

> Arcwell has a command that writes model-generated knowledge reports.

Better angle:

> Interesting technical news fragments immediately. The official source,
> launch thread, press story, and developer reaction each carry different
> information. A useful report has to preserve those differences before it
> summarizes them.

Possible structure:

1. Start with the category problem: when a story breaks, one source is not
   enough.
2. Name the source families Arcwell ingests, only if verified.
3. Explain normalization and clustering through the event.
4. Show the GPT-5.6 source set as the example.
5. Show what each source family contributed.
6. Include the report as the receipt.
7. State what remained unverified, such as benchmark replication or access.

The article is about clustering and reporting. The report is the proof. Tests
are not the proof unless the article is about the clustering test itself.

### Granite And Obsidian Compatibility

Weak angle:

> Markdown is readable text, so a local-first Markdown app should preserve the
> vault.

That is too shallow. A reader can run `cat note.md`. No one needs an article for
that.

Better angle, if the source supports it:

> Obsidian compatibility is a mixed-format contract. Notes are Markdown, but a
> real vault also contains wikilinks, embeds, aliases, tags, Canvas files, Bases,
> plugin settings, metadata caches, and derived search state. The interesting
> problem is not reading one file. It is letting another app add value without
> making the vault depend on that app to remain understandable.

Possible structure:

1. Start with the contract a real vault expects, not with "Markdown is files."
2. Show the fixture vault or example set: links, frontmatter, Canvas, Bases,
   callouts, block IDs, plugin config, or whatever the repo actually proves.
3. Explain which state is source of truth and which state is derived.
4. Show a round-trip or compatibility receipt: what is read, what is preserved,
   what is indexed, what is not rewritten.
5. Explain why `.granite/` exists only after the reader understands the source
   vs derived-state split.
6. State limits honestly: which Obsidian behaviors are supported, partial, or
   unproved.

The article is not "we also use files." The article is the contract between a
portable vault and app-specific value.

### Local Redis Substitute

Weak angle:

> valkey-bun implements Redis commands and tests pass.

Better angle:

> A local Redis substitute is useful only if ordinary Redis clients can keep
> their habits. The question is whether sessions, counters, pub/sub,
> transactions, streams, and binary payloads survive without a separate daemon.

The receipt is a real client conversation or workflow, not the count of tests.

### Search For Agents

Weak angle:

> code-search has BM25 and exact search.

Better angle:

> Agents often search badly before they know a repository's nouns. A ranked
> pass can teach the vocabulary; exact search becomes powerful after that.

The receipt is a query where the ranked pass changes the next search, not the
existence of the index.

## Source Mining

Before rewriting or drafting, inspect the material. Do not infer the story from
the existing article.

Use the relevant sources:

- current post and frontmatter
- README and architecture docs
- source files that implement the claimed behavior
- tests only when they reveal a boundary, fixture, failure, or compatibility
  condition
- examples, fixtures, screenshots, traces, reports, generated outputs, state
  rows, benchmark data, issue threads, commits, and public repo history
- Claude and Codex histories when they reveal the original problem, failure,
  decision, or learning

Treat histories as private source notes. Do not publish private names, secret
project names, credentials, internal paths, or anything that is not meant to be
public. Use them to find the real story, then verify publishable claims in
public repos or safe artifacts.

## Claim Audit Rules

Borrow the discipline of deep research:

- Generated briefs, existing posts, summaries, and model answers are not
  evidence. They can point to evidence, but they cannot prove the claim.
- Important factual claims need primary evidence or a named local artifact.
- Interpretations must be separated from confirmed facts.
- Dates must be explicit for launches, versions, announcements, prices, laws,
  APIs, benchmarks, and fast-moving products.
- Contradictions and caveats stay visible. Do not smooth them into confidence.
- Source coverage matters. Say what families were inspected and what was not.
- If a claim depends on private history, either find public evidence or remove
  the claim.
- If the receipt cannot be shown or described safely, the post probably does not
  belong on the public shelf yet.

The final draft should be traceable back to the claim ledger. A sentence that
cannot be traced to evidence, interpretation, or clearly marked opinion should
be cut.

## Rewrite Workflow

1. Read the existing post.
2. Read the source material instead of trusting the post.
3. Create the required pre-draft brief.
4. Build the source map and claim ledger.
5. Run the skeptic pass and refutation search.
6. Revise, weaken, or reject claims before drafting.
7. Choose the article shape from the survived evidence.
8. Draft the opening from the situation, pressure, or receipt.
9. Bring in mechanism only after the reader needs it.
10. Include the actual receipt or a precise description of it.
11. State limits that affect the claim.
12. Remove sections that prove only diligence.
13. Run the Humanizer pass.
14. Run the Impeccable slop guard.
15. Run the adversarial editorial review.
16. Compare the result against the brief. If the status change disappeared,
    rewrite before publishing.

## Humanizer Pass

Use the installed `humanizer` skill after the draft has a real spine.
Humanizer does not create the story. It removes model habits from a story that
already works.

Check for:

- significance inflation: "pivotal", "crucial", "transformative", "at its core"
- fake contrast: "not just X, but Y" before evidence earns the contrast
- repeated paragraph starts such as "The", "This", "That", "There"
- generic signposting: "let's dive in", "here's what you need to know"
- aphorism stacks and cinematic one-liners
- marketing verbs: "unlock", "leverage", "showcase", "seamless", "robust"
- vague nouns: "system", "workflow", "boundary", "artifact", "value" without
  the concrete thing attached
- em dashes and decorative punctuation habits

Preserve real voice, uncertainty, dry humor, specific numbers, and first-person
judgment when they are true and useful.

## Impeccable Slop Guard

Use Impeccable's slop-prevention standard for public surfaces:

- If a reader could say "a model wrote this" without hesitation, restart the
  section.
- Do not use generic content scaffolding: repeated tiny labels, stock "why it
  matters" blocks, numbered public templates, placeholder headings, or marketing
  language.
- Every paragraph must change the reader's understanding or advance the chain.
- No paragraph should restate its heading before doing work.
- Avoid polished vagueness. Specific and slightly rough is better than smooth
  and empty.

## Public Shelf Gate

Keep a post public only when a practitioner can retell its status change.

Examples:

- "A scattered launch story became a source-backed report."
- "A vault fixture proved which state stayed portable and which state was
  derived."
- "A Redis client workflow showed which local behaviors were real and which
  production claims stayed outside scope."
- "A ranked search pass changed the next exact search."
- "A browser click invalidated a claimed implementation."

Draft or hide the post when the best defense is:

- "it is accurate"
- "it is technical"
- "it has tests"
- "it supports the resume"
- "the project is interesting"
- "the architecture is important"
- "the code exists"

Those are source material, not publication standards.

## Final Quality Questions

Before returning or publishing, answer:

1. What is the subject, excluding the project name?
2. What belief changes on the page?
3. Where is the first real pressure or friction?
4. What is the receipt?
5. What did the receipt change?
6. Which mechanism is necessary, and which mechanism is workshop noise?
7. What did we leave out because it would dilute the claim?
8. What remains unproved?
9. Can a senior practitioner retell the article in one sentence?

If these answers are weak, do not polish. Rebuild the story.
