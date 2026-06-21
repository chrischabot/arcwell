---
description: Poll the remote Arcwell edge inbox and drain email events locally
---

Use `email_poll_edge`. This leases queued Cloudflare edge inbox events into the
local Arcwell store, then records email events as email channel messages and
source cards. Treat this as the one-shot polling path; scheduled execution
should invoke the same command repeatedly rather than bypassing local
persistence and ack/nack handling.
