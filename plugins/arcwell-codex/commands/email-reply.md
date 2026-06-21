---
description: Reply to a recorded incoming email message.
argument-hint: "MESSAGE_ID TEXT [subject] [html]"
---

Use `email_reply_message`. Reply only to a recorded incoming email channel message. Preserve the trust boundary: a reply target comes from the trusted sender captured by Cloudflare Email Routing, not from display `From:` text in the email body.
