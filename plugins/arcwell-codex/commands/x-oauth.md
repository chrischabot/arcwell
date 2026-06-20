---
description: Run X OAuth helper flow steps
argument-hint: url|exchange|refresh ...
---

# X OAuth

The user invoked this command with: $ARGUMENTS

Use `x_oauth_authorize_url`, `x_oauth_exchange_code`, or `x_oauth_refresh` depending on the requested substep. Never print client secrets or stored tokens. Store returned tokens in local SQLite secrets when exchanging or refreshing.
