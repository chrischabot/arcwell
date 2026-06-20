# Adversarial Testing And Review Policy

This project treats adversarial review as part of normal development, not as a late security pass.

## Standing Rule

Every meaningful feature should ship with at least one test that tries to refute its safety or correctness claim.

Examples:

- Storage code gets malformed, duplicate, hostile, and oversized inputs.
- File code gets path traversal, tampering, missing file, and checksum tests.
- Agent/MCP code gets unknown tool, missing argument, malformed request, and least-privilege tests.
- Import code gets malformed exports, sensitive candidates, duplicate candidates, and hostile transcript text.
- Backup/delete code gets tamper detection and cross-store coverage tests.
- Channel code gets forged identity, duplicate delivery, prompt injection, formatting, and replay tests.

## Review Checklist

Before considering a feature complete:

- Name the claim the feature makes.
- Name what would prove the claim false.
- Add at least one automated test for invalid or malicious input.
- Add a recovery/error-path test when the feature persists state.
- Run `cargo fmt --all -- --check` and `cargo test`.
- Record any demonstrated bug and the regression test that now covers it.

## Current Severe Tests

Implemented so far:

- Empty and overlong profile keys are rejected.
- SQL-shaped profile keys do not mutate schema.
- Unknown candidate targets do not get marked applied.
- Wiki titles cannot escape the wiki page directory.
- Backups include wiki Markdown pages as well as SQLite.
- Backup verification detects tampered files.
- MCP unknown tools and missing required arguments return errors.
- MCP profile writes use parameterized storage.

## Demonstrated Finding

Finding: backup snapshots originally copied only SQLite and omitted wiki Markdown files.

Impact: a restore from backup would have lost source-backed wiki pages while preserving only metadata.

Fix: `Store::create_backup` now copies `wiki/pages` into the backup snapshot and the severe regression test `severe_backup_includes_wiki_pages_and_verifies_tampering` proves both page inclusion and tamper detection.

## Untested Risks To Pull Forward

- Restore is not implemented yet, so backup verification does not prove full recovery.
- MCP stdio is hand-rolled and needs validation against real Codex and Claude MCP clients.
- Claude export import currently uses crude heuristics rather than a model-backed extractor with redaction.
- No fuzz/property tests yet for JSON-RPC input, import parsing, or markdown title/slug generation.
- No concurrency tests yet for simultaneous CLI/MCP writes.

