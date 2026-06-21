---
description: Send an email through Arcwell's configured email provider.
argument-hint: "TO SUBJECT TEXT [html]"
---

Use `email_send_message`. Require an explicit user request before sending. Confirm the recipient is intended, keep HTML free of active content, and report delivery state. The default sender comes from local Arcwell config, not tracked plugin text.
