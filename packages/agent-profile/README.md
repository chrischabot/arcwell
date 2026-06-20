# agent-profile

Inspectable personal operating profile.

Current implementation lives in the shared Rust CLI:

```sh
agent profile set <key> <value>
agent profile get <key>
agent profile search <query>
agent profile list
agent profile delete <key>
```

Boundary:

- Profile stores durable operating preferences and explicit rules.
- It is not a hidden prompt blob.
- It is separate from `agent-memory`, which stores compact personal facts and learned preferences.

