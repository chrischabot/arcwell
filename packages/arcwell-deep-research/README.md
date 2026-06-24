# arcwell-deep-research

**Status:** Partial.

Repository tracking: [STATUS.md](../../STATUS.md) and [TODO.md](../../TODO.md).

Deep research coordination for MCP-capable agents.

Target product contract:

- Arcwell has one user-facing research mode: deep.
- Invoking research means broad source discovery, deep source mining, structured
  source cards and claims, skeptic/refutation passes, cited synthesis, audit,
  and durable wiki writeback.
- There is no quick/surface research mode. A short executive summary or brief is
  an output artifact inside a deep run, not a separate product path.
- The normal assistant should not auto-trigger Deep Research for every question;
  it should run only when research is explicitly invoked or unmistakably
  requested.

Design reference: `docs/deep-research-system-design.md`.

Current implementation:

```sh
arcwell research run "topic or question"
arcwell research status <run-id>
arcwell research read <run-id>
arcwell research audit-run <run-id>
arcwell research stop <run-id>
arcwell research sources <run-id>
arcwell research add-source <run-id> --title "Source" --url https://example.com/source
arcwell research link-source-card <run-id> <source-card-id>
arcwell research extraction-prompt <run-id> <source-card-id>
arcwell research ingest-claims <run-id> <source-card-id> --output-json '{"claims":[]}'
arcwell research claims <run-id>
arcwell research clusters <run-id>
arcwell research skeptic <run-id>
arcwell research report <run-id> "source coverage satisfied"
arcwell research plan "topic or question" --max-sources 5
arcwell research search "topic or question" --provider brave --write-wiki
arcwell research workflow "topic or question" # compatibility alias for run
arcwell research tasks <run-id>
arcwell research role-start <run-id> research-scout --execution-mode host_sequential
arcwell research role-finish <role-run-id> completed --output-artifact-id <artifact-id>
arcwell research role-runs <run-id>
arcwell research artifact-add <run-id> source_map "Source map" --body "..."
arcwell research artifacts <run-id>
arcwell research artifact-read <artifact-id>
arcwell research host-search-record <run-id> "query" --results-json '[...]'
arcwell research host-searches <run-id>
arcwell research host-search-read <host-search-id>
arcwell research document-extract <run-id> ./source.csv
arcwell research documents <run-id>
arcwell research document-read <document-id>
arcwell research evidence-pack <run-id>
arcwell research editorial-invoke <run-id> editorial_drafter --model-provider mock
arcwell research editorial-record <run-id> editorial_drafter --model-name gpt-5 --input-artifact-id <evidence-pack-id> --output-artifact-id <draft-id>
arcwell research editorial-runs <run-id>
arcwell research editorial-read <editorial-run-id>
arcwell research complete-task <task-id> "notes"
arcwell research brief "topic or question"
arcwell research brief "topic or question" --no-write
arcwell research audit "topic or question" # legacy query audit
arcwell research runs
```

MCP tools:

- `research_plan`
- `research_run`
- `research_status`
- `research_read`
- `research_audit_run`
- `research_stop`
- `research_web_search`
- `research_workflow_create`
- `research_sources`
- `research_source_add`
- `research_source_card_link`
- `research_extraction_prompt`
- `research_claims_ingest`
- `research_claims`
- `research_clusters`
- `research_skeptic_pass`
- `research_report_compile`
- `research_tasks`
- `research_role_start`
- `research_role_finish`
- `research_role_runs`
- `research_artifact_add`
- `research_artifacts`
- `research_artifact_read`
- `research_host_search_record`
- `research_host_searches`
- `research_host_search_read`
- `research_document_extract`
- `research_documents`
- `research_document_read`
- `research_evidence_pack`
- `research_editorial_invoke`
- `research_editorial_record`
- `research_editorial_runs`
- `research_editorial_read`
- `research_task_complete`
- `research_brief_from_wiki`
- `research_audit`
- `research_runs`

MCP resources:

- `arcwell://research`

Current boundary:

- The local service records deep runs, role tasks, role-run traces, research artifacts, source ledgers, run-source links, extracted claims, clusters, contradictions, skeptic reports, evidence packs, editorial/eval records, and final report artifacts.
- The host agent performs current web search with its native tools when available.
- Host-native search proof can be recorded with query, host/tool surface, selected results, result metadata, linked run sources, and audit gates for missing or weak proof. A fresh in-app Codex `web.run` smoke has been recorded in a disposable proof run; full-report host-search orchestration is still pending.
- Optional daemon providers are `brave`, `openai`, and `perplexity`.
- Provider endpoints are guarded to official HTTPS origins or loopback tests unless explicitly overridden.
- Local CSV/TSV/XLSX extraction creates document/table/cell artifacts with byte hashes, extractor version, formula-injection protection, formula preservation as untrusted text, and stable row/column anchors. PDF text extraction uses a bounded local `pdftotext` helper when available; PDF layout table candidates are emitted with explicit heuristic warnings and confidence scores. Wrapped-header, irregular-column, and footnoted numeric PDF table fixtures lower confidence and preserve footnote refs instead of looking like clean spreadsheet evidence. Malformed, scanned, encrypted, unsupported, or failed documents record blocked/warning states instead of evidence.
- Claim ingestion can link model-produced claims to same-run document spans, tables, and table cells after validating that the referenced artifacts exist. Reports and evidence packs surface those anchors, and run audit warns on warned or low-confidence table/document extractions.
- `research_evidence_pack` creates deterministic bounded editorial input artifacts from durable run state. `research_editorial_invoke` can run mock or OpenAI-backed editorial/eval stages through policy/cost guards, create inspectable output artifacts, and record failed malformed-provider results. `research_editorial_record` remains available for externally run editorial stages; `research_audit_run` fails completed drafts that lack citation-verifier or adversarial-evaluator proof or have unsupported/invalid citation/eval scores. A live OpenAI smoke has proven provider invocation, cost recording, nested Responses API output parsing, and fail-closed rejection on insufficient evidence; production model quality over a saturated corpus is still pending.
- Generated research briefs and source-card wiki artifacts are outputs, not source material. Research source selection excludes generated `Research Brief: ...`, `Expanded: ...`, and `Source Card: ...` wiki pages from local-source evidence.
- `research_audit` checks local source cards for schema/version metadata, generated-page recursion, uncited model answers, stale retrieval dates, prompt-injection/SEO-spam indicators, low reliability, robots `noindex`, low-confidence claims, and conflicting launch dates. `research_audit_run` also includes run-linked source cards even when query text search misses them.
- `research_claims_ingest` validates model-produced JSON and rejects malformed output, prompt-injection text, and uncertainty loss before storing structured claims.
- `research_skeptic_pass` builds deterministic clusters, records structured claim contradictions, and fails runs with missing primary evidence or generated/model-answer evidence loops.
- `research_report_compile` compiles a durable report from linked sources, extracted claims, clusters, skeptic findings, and audit results. It marks reports incomplete when audit or skeptic checks fail.
- Brief/report source selection excludes generated/model-answer cards plus explicitly untrusted or low-reliability source cards. Those cards remain stored and auditable as evidence, but they do not ground synthesized outputs.
- Brave, OpenAI, and Perplexity are adapters, not hard dependencies.

Target host-agent loop:

1. Start a deep run with `research_run`.
2. Call `research_plan` when local context and suggested searches help.
3. Build a source map across primary, secondary, dissenting, technical, historical, and adjacent source families.
4. Search current sources using the host's native web-search capability, or `research_web_search` when a provider key is configured.
5. Record native search provenance with `research_host_search_record` before relying on discovered URLs.
6. Ingest and link source cards with `source_card_add` plus `run_id` or `research_source_card_link`.
7. Extract local CSV/TSV/XLSX/PDF source material with `research_document_extract` when tables or document spans matter.
8. Extract structured claims through `research_extraction_prompt` and `research_claims_ingest`.
9. Run `research_clusters`, `research_skeptic_pass`, and `research_audit_run`.
10. Build `research_evidence_pack`, run any explicit model-backed editorial/eval loop with `research_editorial_invoke` or an external provider, and record/import each stage.
11. Compile the final report with `research_report_compile`.

Future work:

- Full fresh end-to-end Codex subagent workflow proof for scout, corpus builder, extractor, skeptic, synthesizer, and auditor roles inside a completed report. A disposable proof run has recorded two real in-app Codex subagents plus host-native search, but not the full role suite.
- Live deep-run proof against the three design-reference topics, including source-count/saturation reporting, cost records, and fresh-provider evidence.
- Live provider editorial/eval quality smoke over a saturated corpus. A real OpenAI invocation with cost records now exists for an insufficient-evidence proof run, and it correctly rejected unsupported drafting.
- Broader difficult-document fixtures: hidden/merged XLSX sheets, dates, encrypted/scanned PDFs, rotated PDF tables, and external footnote-heavy statistical tables beyond the current local wrapped-header/footnote fixture.
- End-to-end scheduled monitor loops that expand wiki pages when important sources change; current hooks enqueue due watch-source jobs only.
