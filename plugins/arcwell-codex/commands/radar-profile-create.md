---
description: Create a local source-card-backed radar profile
argument-hint: <name> --source-card-query <query>
---

# Radar Profile Create

The user invoked this command with: $ARGUMENTS

Use `radar_profile_create`. Require at least one selector. The locally proven
selector is `source_card_query`; other Horizon-style selectors must remain
visible as unsupported or partial instead of being described as live.
When the user asks for source/category balance, pass structured profile
`metadata.balance` such as `max_per_source` or `category_quotas`; do not imply
production category-balance proof unless a real-data proof packet exists.
