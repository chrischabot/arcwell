use super::*;

impl Store {
    pub fn record_research_host_search(
        &self,
        input: ResearchHostSearchInput,
    ) -> Result<ResearchHostSearchRecord> {
        let input = normalize_research_host_search_input(input)?;
        self.require_research_run(&input.run_id)?;
        if let Some(role_run_id) = &input.role_run_id {
            let role_run = self
                .get_research_role_run(role_run_id)?
                .with_context(|| format!("research role run not found: {role_run_id}"))?;
            if role_run.run_id != input.run_id {
                bail!("host search role run belongs to a different research run");
            }
        }
        let search_id = research_host_search_id();
        let requested_domains_json = serde_json::to_string(&input.requested_domains)?;
        let timestamp = now();
        self.conn.execute(
            r#"
            INSERT INTO research_host_searches
              (id, run_id, role_run_id, host, tool_surface, query, query_intent, requested_recency, requested_domains_json, executed_at, retrieved_at, cost_decision_id, result_count, status)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10, ?11, ?12, 'recorded')
            "#,
            params![
                search_id,
                input.run_id,
                input.role_run_id,
                input.host,
                input.tool_surface,
                input.query,
                input.query_intent,
                input.requested_recency,
                requested_domains_json,
                timestamp,
                input.cost_decision_id,
                input.results.len() as i64,
            ],
        )?;
        for result in input.results {
            let (research_source_id, source_card_id) = if result.selected_for_ingest {
                let source = self.upsert_research_source(ResearchSourceInput {
                    url: Some(result.canonical_url.clone()),
                    local_ref: None,
                    title: result.title.clone(),
                    source_family: result
                        .source_family_guess
                        .clone()
                        .unwrap_or_else(|| "web".to_string()),
                    source_type: "web".to_string(),
                    provider: "host-native".to_string(),
                    author: None,
                    published_at: result.published_at.clone(),
                    language: None,
                    priority: 50,
                    reason: format!(
                        "Selected from host-native search `{}` at rank {}.",
                        input.query, result.rank
                    ),
                    canonical_key: Some(format!("host-search:{}", result.canonical_url)),
                    fetch_status: "candidate".to_string(),
                    read_depth: "snippet-only".to_string(),
                    metadata: json!({
                        "origin": "host_search_record",
                        "host_search_id": search_id,
                        "host": input.host,
                        "tool_surface": input.tool_surface,
                        "rank": result.rank,
                        "query": input.query,
                    }),
                })?;
                let linked = self.link_research_source_to_run(
                    &input.run_id,
                    &source.id,
                    None,
                    "candidate",
                    "snippet-only",
                    Some("Selected from recorded host-native search; not source-carded yet."),
                )?;
                (Some(linked.source.id), linked.link.source_card_id)
            } else {
                (None, None)
            };
            let result_id =
                research_host_search_result_id(&search_id, result.rank, &result.canonical_url);
            let provider_metadata_json = serde_json::to_string(&result.provider_metadata)?;
            self.conn.execute(
                r#"
                INSERT INTO research_host_search_results
                  (id, host_search_id, rank, title, url, canonical_url, snippet, published_at, source_family_guess, provider_metadata_json, selected_for_ingest, research_source_id, source_card_id)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                ON CONFLICT(host_search_id, rank, canonical_url) DO UPDATE SET
                  title = excluded.title,
                  url = excluded.url,
                  snippet = excluded.snippet,
                  published_at = excluded.published_at,
                  source_family_guess = excluded.source_family_guess,
                  provider_metadata_json = excluded.provider_metadata_json,
                  selected_for_ingest = excluded.selected_for_ingest,
                  research_source_id = excluded.research_source_id,
                  source_card_id = excluded.source_card_id
                "#,
                params![
                    result_id,
                    search_id,
                    result.rank as i64,
                    result.title,
                    result.url,
                    result.canonical_url,
                    result.snippet,
                    result.published_at,
                    result.source_family_guess,
                    provider_metadata_json,
                    if result.selected_for_ingest { 1 } else { 0 },
                    research_source_id,
                    source_card_id,
                ],
            )?;
        }
        self.refresh_research_challenges_from_host_search_proofs(&input.run_id)?;
        self.read_research_host_search(&search_id)?
            .with_context(|| format!("inserted host search not found: {search_id}"))
    }

    pub fn list_research_host_searches(
        &self,
        run_id: &str,
    ) -> Result<Vec<ResearchHostSearchRecord>> {
        self.require_research_run(run_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, run_id, role_run_id, host, tool_surface, query, query_intent, requested_recency, requested_domains_json, executed_at, retrieved_at, cost_decision_id, result_count, status, error_kind, error_message_redacted
            FROM research_host_searches
            WHERE run_id = ?1
            ORDER BY executed_at ASC
            "#,
        )?;
        let searches = rows(stmt.query_map(params![run_id], research_host_search_from_row)?)?;
        searches
            .into_iter()
            .map(|search| {
                let results = self.list_research_host_search_results(&search.id)?;
                Ok(ResearchHostSearchRecord { search, results })
            })
            .collect()
    }

    pub fn read_research_host_search(&self, id: &str) -> Result<Option<ResearchHostSearchRecord>> {
        validate_id(id)?;
        let search = self
            .conn
            .query_row(
                r#"
                SELECT id, run_id, role_run_id, host, tool_surface, query, query_intent, requested_recency, requested_domains_json, executed_at, retrieved_at, cost_decision_id, result_count, status, error_kind, error_message_redacted
                FROM research_host_searches
                WHERE id = ?1
                "#,
                params![id],
                research_host_search_from_row,
            )
            .optional()?;
        search
            .map(|search| {
                let results = self.list_research_host_search_results(&search.id)?;
                Ok(ResearchHostSearchRecord { search, results })
            })
            .transpose()
    }

    pub(crate) fn list_research_host_search_results(
        &self,
        host_search_id: &str,
    ) -> Result<Vec<ResearchHostSearchResult>> {
        validate_id(host_search_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, host_search_id, rank, title, url, canonical_url, snippet, published_at, source_family_guess, provider_metadata_json, selected_for_ingest, research_source_id, source_card_id
            FROM research_host_search_results
            WHERE host_search_id = ?1
            ORDER BY rank ASC
            "#,
        )?;
        rows(stmt.query_map(
            params![host_search_id],
            research_host_search_result_from_row,
        )?)
    }

    pub(crate) fn read_research_host_search_result(
        &self,
        id: &str,
    ) -> Result<Option<ResearchHostSearchResult>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, host_search_id, rank, title, url, canonical_url, snippet, published_at, source_family_guess, provider_metadata_json, selected_for_ingest, research_source_id, source_card_id
                FROM research_host_search_results
                WHERE id = ?1
                "#,
                params![id],
                research_host_search_result_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub(crate) fn research_url_ingest_context(
        &self,
        input: &Value,
    ) -> Result<Option<ResearchUrlIngestContext>> {
        let Some(run_id) = input
            .get("research_run_id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            return Ok(None);
        };
        self.require_research_run(run_id)?;
        let host_search_id = input
            .get("host_search_id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        if let Some(host_search_id) = &host_search_id {
            let host_search = self
                .read_research_host_search(host_search_id)?
                .with_context(|| format!("research host search not found: {host_search_id}"))?;
            if host_search.search.run_id != run_id {
                bail!("research URL ingest host search belongs to a different research run");
            }
        }
        let host_search_result = input
            .get("host_search_result_id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|id| {
                validate_id(id)?;
                let result = self
                    .read_research_host_search_result(id)?
                    .with_context(|| format!("research host search result not found: {id}"))?;
                if let Some(host_search_id) = &host_search_id
                    && result.host_search_id != *host_search_id
                {
                    bail!("research URL ingest host search result does not belong to host search");
                }
                let host_search = self
                    .read_research_host_search(&result.host_search_id)?
                    .with_context(|| {
                        format!(
                            "research host search not found for result: {}",
                            result.host_search_id
                        )
                    })?;
                if host_search.search.run_id != run_id {
                    bail!(
                        "research URL ingest host search result belongs to a different research run"
                    );
                }
                Ok::<_, anyhow::Error>(result)
            })
            .transpose()?;
        let source_family = input
            .get("source_family")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .or_else(|| {
                host_search_result
                    .as_ref()
                    .and_then(|result| result.source_family_guess.clone())
            })
            .unwrap_or_else(|| "web".to_string());
        validate_key(&source_family)?;
        let source_type = input
            .get("source_type")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("web")
            .to_string();
        validate_key(&source_type)?;
        Ok(Some(ResearchUrlIngestContext {
            run_id: run_id.to_string(),
            host_search_id,
            host_search_result,
            source_family,
            source_type,
        }))
    }

    pub fn extract_research_document_file(
        &self,
        input: ResearchDocumentInput,
    ) -> Result<ResearchDocumentRecord> {
        let input = normalize_research_document_input(input)?;
        self.require_research_run(&input.run_id)?;
        let source_url = if let Some(source_id) = &input.research_source_id {
            let source = self
                .read_research_source(source_id)?
                .with_context(|| format!("research source not found: {source_id}"))?;
            source.url
        } else {
            None
        };
        if let Some(card_id) = &input.source_card_id {
            self.read_source_card(card_id)?
                .with_context(|| format!("source card not found: {card_id}"))?;
        }
        let bytes = fs::read(&input.path)
            .with_context(|| format!("reading research document {}", input.path.display()))?;
        if bytes.len() as u64 > RESEARCH_DOCUMENT_MAX_BYTES {
            bail!("research document is too large");
        }
        let byte_sha256 = sha256(&bytes);
        let byte_len = bytes.len() as u64;
        let media_type = input
            .media_type
            .clone()
            .unwrap_or_else(|| infer_research_document_media_type(&input.path));
        let id = research_document_id(&input.run_id, &input.path, &byte_sha256);
        let extraction = extract_research_document_content(&id, &input.path, &media_type, &bytes)?;

        self.conn.execute(
            "DELETE FROM research_document_spans WHERE document_id = ?1",
            params![id],
        )?;
        self.conn.execute(
            "DELETE FROM research_tables WHERE document_id = ?1",
            params![id],
        )?;

        let warning_flags_json = serde_json::to_string(&extraction.warning_flags)?;
        self.conn.execute(
            r#"
            INSERT INTO research_documents
              (id, run_id, research_source_id, source_card_id, url, local_path, media_type,
               byte_sha256, byte_len, retrieved_at, extractor_name, extractor_version,
               extraction_status, page_count, sheet_count, table_count, warning_flags_json,
               error_message_redacted)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
            ON CONFLICT(id) DO UPDATE SET
              research_source_id = excluded.research_source_id,
              source_card_id = excluded.source_card_id,
              url = excluded.url,
              local_path = excluded.local_path,
              media_type = excluded.media_type,
              byte_sha256 = excluded.byte_sha256,
              byte_len = excluded.byte_len,
              retrieved_at = excluded.retrieved_at,
              extractor_name = excluded.extractor_name,
              extractor_version = excluded.extractor_version,
              extraction_status = excluded.extraction_status,
              page_count = excluded.page_count,
              sheet_count = excluded.sheet_count,
              table_count = excluded.table_count,
              warning_flags_json = excluded.warning_flags_json,
              error_message_redacted = excluded.error_message_redacted
            "#,
            params![
                id,
                input.run_id,
                input.research_source_id,
                input.source_card_id,
                source_url,
                input.path.display().to_string(),
                media_type,
                byte_sha256,
                byte_len as i64,
                now(),
                extraction.extractor_name,
                extraction.extractor_version,
                extraction.status,
                extraction.page_count as i64,
                extraction.sheet_count as i64,
                extraction.tables.len() as i64,
                warning_flags_json,
                extraction.error_message_redacted,
            ],
        )?;

        for span in extraction.spans {
            let warning_flags_json = serde_json::to_string(&span.warning_flags)?;
            let bbox_json = span
                .bbox_json
                .map(|value| serde_json::to_string(&value))
                .transpose()?;
            self.conn.execute(
                r#"
                INSERT INTO research_document_spans
                  (id, document_id, span_id, page_number, section_label, char_start, char_end,
                   text_sha256, text_excerpt, bbox_json, confidence, warning_flags_json)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                "#,
                params![
                    span.id,
                    id,
                    span.span_id,
                    span.page_number.map(|value| value as i64),
                    span.section_label,
                    span.char_start as i64,
                    span.char_end as i64,
                    span.text_sha256,
                    span.text_excerpt,
                    bbox_json,
                    span.confidence,
                    warning_flags_json,
                ],
            )?;
        }

        for table in extraction.tables {
            let warning_flags_json = serde_json::to_string(&table.table.warning_flags)?;
            let bbox_json = table
                .table
                .bbox_json
                .clone()
                .map(|value| serde_json::to_string(&value))
                .transpose()?;
            self.conn.execute(
                r#"
                INSERT INTO research_tables
                  (id, document_id, table_id, page_number, sheet_name, caption, bbox_json,
                   row_count, column_count, extraction_method, confidence, warning_flags_json)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                "#,
                params![
                    table.table.id,
                    id,
                    table.table.table_id,
                    table.table.page_number.map(|value| value as i64),
                    table.table.sheet_name,
                    table.table.caption,
                    bbox_json,
                    table.table.row_count as i64,
                    table.table.column_count as i64,
                    table.table.extraction_method,
                    table.table.confidence,
                    warning_flags_json,
                ],
            )?;
            for cell in table.cells {
                let footnote_refs_json = serde_json::to_string(&cell.footnote_refs)?;
                let bbox_json = cell
                    .bbox_json
                    .map(|value| serde_json::to_string(&value))
                    .transpose()?;
                self.conn.execute(
                    r#"
                    INSERT INTO research_table_cells
                      (id, table_id, row_index, column_index, row_header, column_header, raw_text,
                       normalized_text, numeric_value, unit, footnote_refs_json, bbox_json, confidence)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                    "#,
                    params![
                        cell.id,
                        cell.table_id,
                        cell.row_index as i64,
                        cell.column_index as i64,
                        cell.row_header,
                        cell.column_header,
                        cell.raw_text,
                        cell.normalized_text,
                        cell.numeric_value,
                        cell.unit,
                        footnote_refs_json,
                        bbox_json,
                        cell.confidence,
                    ],
                )?;
            }
        }

        self.read_research_document(&id)?
            .with_context(|| format!("inserted research document not found: {id}"))
    }

    pub fn read_research_document(&self, id: &str) -> Result<Option<ResearchDocumentRecord>> {
        validate_id(id)?;
        let document = self
            .conn
            .query_row(
                r#"
                SELECT id, run_id, research_source_id, source_card_id, url, local_path, media_type,
                       byte_sha256, byte_len, retrieved_at, extractor_name, extractor_version,
                       extraction_status, page_count, sheet_count, table_count, warning_flags_json,
                       error_message_redacted
                FROM research_documents
                WHERE id = ?1
                "#,
                params![id],
                research_document_from_row,
            )
            .optional()?;
        document
            .map(|document| {
                let spans = self.list_research_document_spans(&document.id)?;
                let tables = self.list_research_document_tables(&document.id)?;
                Ok(ResearchDocumentRecord {
                    document,
                    spans,
                    tables,
                })
            })
            .transpose()
    }

    pub fn list_research_documents(&self, run_id: &str) -> Result<Vec<ResearchDocumentRecord>> {
        self.require_research_run(run_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, run_id, research_source_id, source_card_id, url, local_path, media_type,
                   byte_sha256, byte_len, retrieved_at, extractor_name, extractor_version,
                   extraction_status, page_count, sheet_count, table_count, warning_flags_json,
                   error_message_redacted
            FROM research_documents
            WHERE run_id = ?1
            ORDER BY retrieved_at ASC, id ASC
            "#,
        )?;
        let documents = rows(stmt.query_map(params![run_id], research_document_from_row)?)?;
        documents
            .into_iter()
            .map(|document| {
                let spans = self.list_research_document_spans(&document.id)?;
                let tables = self.list_research_document_tables(&document.id)?;
                Ok(ResearchDocumentRecord {
                    document,
                    spans,
                    tables,
                })
            })
            .collect()
    }

    pub(crate) fn list_research_document_spans(
        &self,
        document_id: &str,
    ) -> Result<Vec<ResearchDocumentSpan>> {
        validate_id(document_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, document_id, span_id, page_number, section_label, char_start, char_end,
                   text_sha256, text_excerpt, bbox_json, confidence, warning_flags_json
            FROM research_document_spans
            WHERE document_id = ?1
            ORDER BY COALESCE(page_number, 0) ASC, char_start ASC, span_id ASC
            "#,
        )?;
        rows(stmt.query_map(params![document_id], research_document_span_from_row)?)
    }

    pub(crate) fn list_research_document_tables(
        &self,
        document_id: &str,
    ) -> Result<Vec<ResearchTableRecord>> {
        validate_id(document_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, document_id, table_id, page_number, sheet_name, caption, bbox_json,
                   row_count, column_count, extraction_method, confidence, warning_flags_json
            FROM research_tables
            WHERE document_id = ?1
            ORDER BY COALESCE(page_number, 0) ASC, table_id ASC
            "#,
        )?;
        let tables = rows(stmt.query_map(params![document_id], research_table_from_row)?)?;
        tables
            .into_iter()
            .map(|table| {
                let cells = self.list_research_table_cells(&table.id)?;
                Ok(ResearchTableRecord { table, cells })
            })
            .collect()
    }

    pub(crate) fn list_research_table_cells(
        &self,
        table_id: &str,
    ) -> Result<Vec<ResearchTableCell>> {
        validate_id(table_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, table_id, row_index, column_index, row_header, column_header, raw_text,
                   normalized_text, numeric_value, unit, footnote_refs_json, bbox_json, confidence
            FROM research_table_cells
            WHERE table_id = ?1
            ORDER BY row_index ASC, column_index ASC
            "#,
        )?;
        rows(stmt.query_map(params![table_id], research_table_cell_from_row)?)
    }

    pub fn build_research_evidence_pack(&self, run_id: &str) -> Result<ResearchArtifact> {
        let run = self.require_research_run(run_id)?;
        let sources = self.list_research_run_sources(run_id)?;
        let claims = self.list_research_claims(run_id)?;
        let clusters = self.build_research_clusters(run_id)?;
        let host_searches = self.list_research_host_searches(run_id)?;
        let documents = self.list_research_documents(run_id)?;
        let audit = self.audit_research_run(run_id)?;
        let payload = json!({
            "schema_version": 2,
            "boundary": "Evidence pack is generated editorial input, not primary evidence.",
            "run": run,
            "sources": sources,
            "claims": claims,
            "clusters": clusters,
            "host_searches": host_searches,
            "documents": documents,
            "audit": audit.audit,
        });
        self.record_research_artifact(ResearchArtifactInput {
            run_id: run_id.to_string(),
            role_run_id: None,
            artifact_type: "evidence_pack".to_string(),
            title: format!("Evidence pack for {}", run.query),
            body: serde_json::to_string_pretty(&payload)?,
            metadata: json!({
                "artifact_role": "model_editorial_input",
                "source": "deterministic_evidence_pack",
                "schema_version": 2,
            }),
        })
    }

    pub fn record_research_editorial_run(
        &self,
        input: ResearchEditorialRunInput,
    ) -> Result<ResearchEditorialRun> {
        let input = normalize_research_editorial_run_input(input)?;
        self.require_research_run(&input.run_id)?;
        let input_artifact_hash = if let Some(input_artifact_id) = &input.input_artifact_id {
            let artifact = self
                .read_research_artifact(input_artifact_id)?
                .with_context(|| format!("input artifact not found: {input_artifact_id}"))?;
            if artifact.run_id != input.run_id {
                bail!("editorial input artifact belongs to a different research run");
            }
            Some(artifact.body_sha256)
        } else {
            None
        };
        if let Some(output_artifact_id) = &input.output_artifact_id {
            let artifact = self
                .read_research_artifact(output_artifact_id)?
                .with_context(|| format!("output artifact not found: {output_artifact_id}"))?;
            if artifact.run_id != input.run_id {
                bail!("editorial output artifact belongs to a different research run");
            }
        }
        let id = research_editorial_run_id();
        let score_json = serde_json::to_string(&input.score)?;
        let error_message_redacted = input
            .error_message
            .as_deref()
            .map(|value| sanitize_work_text(value, 2_000))
            .transpose()?;
        self.conn.execute(
            r#"
            INSERT INTO research_editorial_runs
              (id, run_id, stage, model_provider, model_name, prompt_version,
               input_artifact_hash, input_artifact_id, output_artifact_id, cost_decision_id,
               status, score_json, error_message_redacted, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            "#,
            params![
                id,
                input.run_id,
                input.stage,
                input.model_provider,
                input.model_name,
                input.prompt_version,
                input_artifact_hash,
                input.input_artifact_id,
                input.output_artifact_id,
                input.cost_decision_id,
                input.status,
                score_json,
                error_message_redacted,
                now(),
            ],
        )?;
        self.get_research_editorial_run(&id)?
            .with_context(|| format!("inserted research editorial run not found: {id}"))
    }

    pub fn list_research_editorial_runs(&self, run_id: &str) -> Result<Vec<ResearchEditorialRun>> {
        self.require_research_run(run_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, run_id, stage, model_provider, model_name, prompt_version,
                   input_artifact_hash, input_artifact_id, output_artifact_id, cost_decision_id,
                   status, score_json, error_message_redacted, created_at
            FROM research_editorial_runs
            WHERE run_id = ?1
            ORDER BY created_at ASC, id ASC
            "#,
        )?;
        rows(stmt.query_map(params![run_id], research_editorial_run_from_row)?)
    }

    pub fn get_research_editorial_run(&self, id: &str) -> Result<Option<ResearchEditorialRun>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, run_id, stage, model_provider, model_name, prompt_version,
                       input_artifact_hash, input_artifact_id, output_artifact_id, cost_decision_id,
                       status, score_json, error_message_redacted, created_at
                FROM research_editorial_runs
                WHERE id = ?1
                "#,
                params![id],
                research_editorial_run_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn invoke_research_editorial(
        &self,
        input: ResearchEditorialInvokeInput,
    ) -> Result<ResearchEditorialInvocation> {
        let input = normalize_research_editorial_invoke_input(input)?;
        self.require_research_run(&input.run_id)?;
        let input_artifact = if let Some(input_artifact_id) = &input.input_artifact_id {
            let artifact = self
                .read_research_artifact(input_artifact_id)?
                .with_context(|| format!("input artifact not found: {input_artifact_id}"))?;
            if artifact.run_id != input.run_id {
                bail!("editorial input artifact belongs to a different research run");
            }
            artifact
        } else {
            self.build_research_evidence_pack(&input.run_id)?
        };
        let provider = input.model_provider.clone();
        let model = input.model_name.clone().unwrap_or_else(|| {
            if provider == "mock" {
                "mock-editorial".to_string()
            } else {
                std::env::var("ARCWELL_RESEARCH_EDITORIAL_MODEL")
                    .unwrap_or_else(|_| "gpt-5.5".to_string())
            }
        });
        let prompt = build_research_editorial_prompt(&input.stage, &input_artifact)?;
        let invocation_job_id = format!("editorial-{}", Uuid::new_v4().simple());
        let (provider_response, cost_decision_id) = if provider == "mock" {
            (
                mock_editorial_provider_response(&input.stage, &input_artifact),
                None,
            )
        } else if provider == "openai" {
            let endpoint = validated_endpoint(
                input.endpoint.as_deref(),
                "https://api.openai.com/v1/responses",
            )?;
            self.policy_guard(PolicyRequest {
                action: "provider.network".to_string(),
                package: Some("arcwell-deep-research".to_string()),
                provider: Some("openai".to_string()),
                source: Some("research_editorial_invoke".to_string()),
                channel: None,
                subject: None,
                target: Some(endpoint.as_str().to_string()),
                projected_usd: Some(estimated_editorial_cost(&model, prompt.len())),
                metadata: json!({
                    "stage": input.stage,
                    "input_artifact_id": input_artifact.id,
                    "prompt_version": input.prompt_version
                }),
                untrusted_excerpt: Some(excerpt(&input_artifact.body, 1_000)),
            })?;
            let decision = self.require_cost_budget(
                "arcwell-deep-research",
                &invocation_job_id,
                "openai",
                &model,
                Some("research_editorial_invoke"),
                estimated_editorial_cost(&model, prompt.len()),
                "research editorial invocation",
            )?;
            (
                openai_editorial_provider_response(
                    &prompt,
                    &model,
                    endpoint,
                    input.api_key.as_deref(),
                    Duration::from_secs(input.timeout_seconds.unwrap_or(30).clamp(1, 120)),
                )?,
                decision.decision_id,
            )
        } else {
            bail!("unsupported research editorial provider: {provider}");
        };
        let parsed = parse_editorial_provider_response(&provider_response);
        let (mut status, score, body, error_message) = match parsed {
            Ok(parsed) => parsed,
            Err(error) => (
                "failed".to_string(),
                json!({}),
                None,
                Some(format!("provider returned invalid editorial JSON: {error}")),
            ),
        };
        let mut body = body
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(ToOwned::to_owned);
        if matches!(status.as_str(), "completed" | "accepted")
            && body.is_none()
            && score.as_object().is_some_and(|object| !object.is_empty())
        {
            body = Some(format!(
                "Provider returned structured editorial score without a prose body.\n\nScore JSON:\n{}",
                canonical_json(&score)?
            ));
        }
        let missing_completed_body =
            matches!(status.as_str(), "completed" | "accepted") && body.is_none();
        let error_message =
            if missing_completed_body {
                Some(error_message.unwrap_or_else(|| {
                    "completed editorial invocation returned no body".to_string()
                }))
            } else {
                error_message
            };
        if missing_completed_body {
            status = "failed".to_string();
        }
        let output_artifact = if let Some(body) = body
            .as_ref()
            .filter(|_| matches!(status.as_str(), "completed" | "accepted"))
        {
            Some(self.record_research_artifact(ResearchArtifactInput {
                    run_id: input.run_id.clone(),
                    role_run_id: None,
                    artifact_type: editorial_output_artifact_type(&input.stage).to_string(),
                    title: format!(
                        "{} output for {}",
                        humanize_research_theme(&input.stage.replace('_', " ")),
                        input_artifact.title
                    ),
                    body: body.to_string(),
                    metadata: json!({
                        "artifact_role": "model_editorial_output",
                        "provider": provider,
                        "model": model,
                        "stage": input.stage,
                        "input_artifact_id": input_artifact.id,
                        "prompt_version": input.prompt_version,
                        "body_present": !body.starts_with("Provider returned structured editorial score without a prose body."),
                        "score_body_synthesized": body.starts_with("Provider returned structured editorial score without a prose body.")
                    }),
                })?)
        } else {
            None
        };
        let editorial_run = self.record_research_editorial_run(ResearchEditorialRunInput {
            run_id: input.run_id,
            stage: input.stage,
            model_provider: provider,
            model_name: model,
            prompt_version: input.prompt_version,
            input_artifact_id: Some(input_artifact.id),
            output_artifact_id: output_artifact.as_ref().map(|artifact| artifact.id.clone()),
            cost_decision_id,
            status,
            score,
            error_message,
        })?;
        Ok(ResearchEditorialInvocation {
            editorial_run,
            output_artifact,
            provider_response: sanitize_work_json(provider_response)?,
        })
    }
}
