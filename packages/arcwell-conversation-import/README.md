# arcwell-conversation-import

**Status:** Partial.

Repository tracking: [STATUS.md](../../STATUS.md) and [TODO.md](../../TODO.md).

Consentful conversation history import.

Current implementation:

```sh
arcwell import claude /path/to/conversations.json --dry-run --limit 50
arcwell import claude /path/to/conversations.json --write-candidates --limit 50
arcwell import claude /path/to/claude-history-export --dry-run --limit 50
arcwell import claude /path/to/claude-history-export --write-candidates --limit 2286
arcwell import claude /path/to/claude-history-export --write-candidates --user-id local-user
arcwell import runs --limit 25
arcwell candidate list
arcwell candidate apply <candidate-id>
arcwell candidate reject <candidate-id>
```

When a Claude export directory contains `out/canonical_memories.jsonl` or
`out/mem0/mem0_ingest.jsonl`, `arcwell import claude` imports those coalesced
memory rows as pending Arcwell memory candidates with operation, memory id, user
scope, and provenance metadata. Secret-like strings are redacted from candidate
content and metadata. JSONL imports count all rows while storing only the
requested sample limit, and repeated `--write-candidates` runs suppress duplicate
pending/applied/rejected candidates. Each import attempt writes an aggregate
ledger record with status, counts, duplicate suppression totals, and redacted
errors; inspect it with `arcwell import runs`.

Raw `conversations.json` remains a fallback heuristic importer. It does not yet
have the richer canonical-row extraction quality.

Boundary:

- Imports produce candidates, not hidden context.
- Raw transcripts should not be injected into prompts by default.
- Sensitive candidates require review.
- Applying imported candidates to Arcwell Memory is still a separate explicit
  review step.
- Codex export parsing, raw full-transcript streaming, and a richer review UI
  are still missing.
