# arcwell-profile

Inspectable personal operating profile.

Current implementation lives in the shared Rust CLI:

```sh
arcwell profile set <key> <value>
arcwell profile get <key>
arcwell profile search <query>
arcwell profile list
arcwell profile delete <key>
```

Boundary:

- Profile stores durable operating preferences and explicit rules.
- It is not a hidden prompt blob.
- It is separate from `arcwell-memory`, which stores compact personal facts and learned preferences.

