# arcwell-conversation-import

Consentful conversation history import.

Current implementation:

```sh
arcwell import claude /path/to/conversations.json --dry-run --limit 50
arcwell import claude /path/to/conversations.json --write-candidates --limit 50
arcwell candidate list
arcwell candidate apply <candidate-id>
arcwell candidate reject <candidate-id>
```

Boundary:

- Imports produce candidates, not hidden context.
- Raw transcripts should not be injected into prompts by default.
- Sensitive candidates require review.

