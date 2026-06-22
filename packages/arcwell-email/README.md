# arcwell-email

**Status:** Partial/Risk. The package defines the email channel boundary and
ships a tested local mapper for normalized inbound email metadata. The edge
inbox Worker now has a bounded Cloudflare Email Routing handler that normalizes
raw MIME into durable `email` edge events under configured route/sender policy.
The Rust core can poll the edge inbox, drain those events into local email
channel messages/source cards, and send/reply through Cloudflare Email Service
with rich HTML after recipient authorization. Controlled live smoke has proven
one author-originated Cloudflare Email Routing ingress and one Cloudflare Email
Service outbound send with real addresses kept only in ignored local config.

Repository tracking: [STATUS.md](../../STATUS.md) and [TODO.md](../../TODO.md).

Email channel and ingestion package.

## Decision

Inbound capture should use Cloudflare Email Routing into `arcwell-edge-inbox`
for Arcwell-owned proactive addresses such as `launches@...` or `alerts@...`.
That matches the existing always-on edge buffer and avoids broad mailbox OAuth
scopes for passive ingestion.

Gmail remains host-native first:

- Use connected Gmail tools for interactive user-selected reading, drafting,
  and sending.
- Only archive a selected thread into Arcwell when the user or local policy
  explicitly asks for project/source-card/work-run provenance.
- Do not add Gmail API polling until a narrow label/scope/storage need is
  proven. Gmail body scopes are too broad for the default proactive loop.

## Boundary

The package-owned adapter contract is normalized metadata, not raw authority:

- `messageId`: required idempotency key seed.
- `envelopeFrom` / `signedSender`: trusted sender identity.
- `headerFrom`: display evidence only; never authorization.
- `envelopeTo`: route key.
- `auth`: SPF/DKIM/DMARC verdicts from the capture layer.
- `subject`, `bodyText`, `bodyHtml`: untrusted evidence.
- `attachments`: metadata only unless a future policy explicitly stages content.

The mapper emits:

- an `email` edge event with a stable idempotency key,
- an inbound `channel_message` draft with `UNTRUSTED_CHANNEL_EVIDENCE`,
- an optional `source_card` draft with `untrusted_email_evidence`.

Configured author mail is the only exception to the "body is evidence" rule.
If the trusted envelope/authenticated sender matches local
`ARCWELL_AUTHOR_EMAILS` or `ARCWELL_AUTHOR_EMAIL`, Arcwell records the message
as `TRUSTED_AUTHOR_EMAIL_INSTRUCTIONS`. A spoofed display `From:` header never
grants this. Open-source defaults are placeholders such as `agent@example.com`
and `user@example.com`; real addresses belong in ignored local env files or the
Arcwell secret store.

The Rust/local service remains the durable owner of SQLite, wiki source cards,
channel messages, policy decisions, and project bindings.

## Routing And Authorization

Routing is fail-closed:

- recipient address must match a configured route,
- trusted sender must match an allowed address or domain,
- sender allow rules must include the route id,
- DMARC must pass by default,
- auto-replies and bulk/list mail do not enter the proactive loop,
- duplicate `Message-ID` values map to one idempotency key.

Sender authorization uses envelope/signed metadata. A spoofed `From:` header is
preserved as evidence and may add a warning, but it cannot grant access to a
route or project.

## Content Safety

Email body/content is untrusted evidence, never instructions unless the trusted
sender is a configured author.

Current local mapper behavior:

- strips active HTML/script/style before preview mapping,
- records prompt-injection text as inert evidence,
- flags tracking links but does not fetch them,
- rejects oversized bodies,
- ignores attachment content by default and records bounded metadata,
- rejects attachment-count and attachment-size bombs,
- rejects auto-responder/list-loop hazards.

The current Cloudflare Email Routing handler keeps raw MIME parsing bounded
before enqueueing sanitized metadata into `arcwell-edge-inbox`. It rejects
oversized raw messages, missing routes, unauthorized envelope senders, missing
`Message-ID`, and failing DMARC by default.

Worker configuration:

- `EMAIL_ROUTES_JSON`: JSON array of `{ "id", "recipient", "projectId", "allowedSenders" }`.
- `EMAIL_ALLOWED_SENDERS_JSON`: optional global sender/domain allowlist used when a route does not define `allowedSenders`.
- `EMAIL_MAX_RAW_BYTES`: raw MIME byte cap, defaulting to the Worker payload cap.
- `EMAIL_MAX_PREVIEW_CHARS`: sanitized text preview cap.
- `EMAIL_REQUIRE_DMARC_PASS`: defaults to true; set to `false` only for controlled tests.

## Outbound Delivery

Arcwell can send or reply through Cloudflare Email Service from a locally
configured sender (`ARCWELL_AGENT_EMAIL_FROM` or `ARCWELL_AGENT_EMAIL`) after:

- explicit recipient authorization,
- delivery attempt records,
- policy/cost checks before provider egress,
- rich HTML active-content rejection,
- token-safe provider error recording.

Cloudflare account IDs, API tokens, and real agent/author email addresses must
not be committed. Use ignored `.env` values or `arcwell secrets`.

One-shot local polling is:

```sh
arcwell email poll
```

That command uses `ARCWELL_EDGE_URL`/`ARCWELL_EDGE_SECRET` or matching Arcwell
secrets, leases remote edge events, and then runs the local email drain. Use
tracked examples only with `agent@example.com` and `user@example.com`; real
addresses belong in ignored local config.

The 2026-06-21 manual live smoke used local-only real addresses to prove the
configured author-to-agent route, then scrubbed tracked documentation back to
`agent@example.com` and `user@example.com`.

Repeatable local and guarded live smoke is:

```sh
scripts/email-live-smoke --no-live
ARCWELL_EMAIL_LIVE_CONFIRM=route scripts/email-live-smoke --live
```

The script runs local mapper/Worker/Rust severe checks by default. Live mode
drains the configured edge inbox into the configured local Arcwell home only
after explicit confirmation, waits for a unique controlled author-originated
subject, and can send a rich outbound smoke only when
`ARCWELL_EMAIL_OUTBOUND_CONFIRM=send` and `ARCWELL_EMAIL_SMOKE_TO` are set. The
remaining operations gap is scheduler/digest delivery and production
monitoring/alerting.

## Local Severe Tests

```sh
cd packages/arcwell-email && npm test
cd ../arcwell-edge-inbox/worker && npm test
cd ../../..
cargo test -p arcwell-core email -- --nocapture
scripts/email-live-smoke --no-live
```

The fixture-backed tests try to refute the mapper with spoofed `From:` headers,
malicious HTML/script/CSS, markdown prompt injection, attachment bombs, tracking
links, duplicate `Message-ID`, oversized bodies, auto-responders, and
unauthorized routing. Worker tests additionally prove Email Routing MIME
normalization, duplicate idempotency, route/sender rejection, raw-size rejection,
and durable edge enqueue behavior. Rust tests prove local drain persistence,
configured-author trust, spoofed display-sender rejection, outbound
authorization, active HTML rejection, and provider-token redaction.
