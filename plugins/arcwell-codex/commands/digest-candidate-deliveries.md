---
description: List digest candidate delivery ledger rows
argument-hint: [candidate-id]
---

# /digest-candidate-deliveries

List durable digest delivery ledger rows.

Use `digest_candidate_deliveries` to inspect delivery status for digest
candidates, optionally filtered by `candidate_id`. Treat this as the source of
truth for whether a digest candidate was blocked, sent, failed, or replayed.
Do not infer digest delivery state only from generic channel delivery attempts.
