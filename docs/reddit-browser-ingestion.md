# Reddit Browser Ingestion

Status: supervised browser-capture path, production-data proven for listing
ingestion and radar projection. This is not unattended Reddit monitoring.

## Claim

Arcwell can ingest a Reddit listing JSON document supplied by an authorized host
browser session, after sanitization, and project it into:

- source cards
- wiki source-card pages
- Reddit cursor state
- source-health state
- radar items, FTS rows, scores, summaries, and audit output

The repeatable proof script is:

```sh
scripts/reddit-browser-production-proof path/to/reddit-listing.json
```

The script uses a disposable `ARCWELL_HOME`, persists only a sanitized listing
artifact, runs `arcwell source-card ingest-reddit-browser-listing`, runs a Reddit
radar profile, writes a summary, runs audit, inspects ops, and emits a
`proof-packet.json`.

Current release-candidate packet:

```text
.arcwell-dev/proofs/reddit-browser-production-proof-20260625T064407Z-87932/artifacts/proof-packet.json
```

## Boundary

This path is supervised. A user or agent supplies the browser-rendered Reddit
JSON file. Arcwell does not read browser cookies, local storage, passwords,
Chrome profile databases, extension state, or raw browser session stores.

This path does not prove:

- daemon-side Reddit OAuth
- scheduled unattended Reddit fetch
- recursive comment capture
- digest delivery
- model-written synthesis
- long-running service behavior

## Accepted Input

Input must be Reddit listing JSON shaped like:

```json
{
  "kind": "Listing",
  "data": {
    "children": [
      {
        "kind": "t3",
        "data": {
          "id": "post_id",
          "subreddit": "rust",
          "title": "Post title",
          "permalink": "/r/rust/comments/post_id/post_slug/",
          "url": "https://example.com/optional-external-url",
          "selftext": "optional post text",
          "author": "optional_author",
          "score": 42,
          "upvote_ratio": 0.97,
          "num_comments": 5,
          "over_18": false,
          "created_utc": 1782144000
        }
      }
    ]
  }
}
```

The CLI command is:

```sh
arcwell source-card ingest-reddit-browser-listing \
  --locator r/rust/hot \
  --listing-json path/to/reddit-listing-sanitized.json \
  --limit 10
```

The listing file must be 2 MB or smaller.

The repeatable proof script also treats the supplied listing artifact as stale
when its file modification time is older than 72 hours. Override this only for
an explicitly labeled replay proof:

```sh
ARCWELL_REDDIT_BROWSER_MAX_CAPTURE_AGE_HOURS=168 \
  scripts/reddit-browser-production-proof path/to/reddit-listing.json
```

## Persisted Fields

The proof script sanitizes every post to this allow-list:

- `id`
- `subreddit`
- `title`
- `permalink`
- `url`
- `selftext`
- `selftext_html`
- `author`
- `score`
- `upvote_ratio`
- `num_comments`
- `over_18`
- `created_utc`
- `removed_by_category`
- `hidden`

All other Reddit response fields are dropped before Arcwell writes proof
artifacts.

## Redaction Gate

`scripts/reddit-browser-production-proof` fails if persisted artifacts contain
browser storage references, authorization material, Reddit session material,
response hash fields, account interaction booleans, user report payloads, or
embedded media payloads.

The proof packet records:

- input hash and size
- input modification time and age
- sanitized child count
- Arcwell binary path and hash
- source-card/wiki/radar counts
- cursor key and value
- source-health status
- radar run id and proof level
- summary id
- remaining boundaries

## Inspection

After a proof run, inspect:

```sh
cat .arcwell-dev/proofs/reddit-browser-production-proof-*/artifacts/proof-packet.json
cat .arcwell-dev/proofs/reddit-browser-production-proof-*/artifacts/ops.json
cat .arcwell-dev/proofs/reddit-browser-production-proof-*/artifacts/radar-audit.json
cat .arcwell-dev/proofs/reddit-browser-production-proof-*/artifacts/radar-summary-read.json
```

The source-health row should include `reddit:<locator>` with `status=healthy`.
The cursor should use the same key. The radar run metadata should include:

- `proof_level=Production Data Proof`
- `source_family=host_browser_then_source_card_projection`

## Promotion Rule

This path can be called release-ready only as a supervised browser-capture
feature. It must not be described as live, current, scheduled, unattended, or
OAuth-backed Reddit monitoring until the daemon-side source path has its own
production proof.
