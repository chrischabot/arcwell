---
description: Deliver an approved digest candidate to Telegram
argument-hint: [candidate-id]
---

# /digest-candidate-deliver-telegram

Deliver an approved digest candidate to a Telegram chat.

Use `digest_candidate_deliver_telegram` only after the user has explicitly
approved the candidate and named the destination chat. This command must pass
the digest candidate review/policy gate and the normal Telegram send
authorization/policy/cost/provider path. Report the resulting channel message
and delivery attempt ids. Do not treat this as email delivery, quiet-hours
scheduling, or recurring delivery.
