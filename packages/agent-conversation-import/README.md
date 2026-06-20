# agent-conversation-import

Consentful conversation history import.

Current implementation:

```sh
agent import claude /path/to/conversations.json --dry-run --limit 50
agent import claude /path/to/conversations.json --write-candidates --limit 50
agent candidate list
agent candidate apply <candidate-id>
agent candidate reject <candidate-id>
```

Boundary:

- Imports produce candidates, not hidden context.
- Raw transcripts should not be injected into prompts by default.
- Sensitive candidates require review.

