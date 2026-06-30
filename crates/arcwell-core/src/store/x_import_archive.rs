use super::*;

impl Store {
    pub fn import_x_json_file(&self, path: &Path) -> Result<XImportReport> {
        let bytes = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
        if bytes.len() > 5_000_000 {
            bail!("x import file is too large");
        }
        let value: Value = serde_json::from_slice(&bytes).context("parsing X import JSON")?;
        self.import_x_json_value_with_sync(
            &value,
            json!({ "source": "file", "path": path.display().to_string(), "bytes": bytes.len() }),
        )
    }

    pub fn discover_x_archives(
        &self,
        roots: &[PathBuf],
        limit: usize,
    ) -> Result<XArchiveDiscoveryReport> {
        discover_x_archives(roots, limit)
    }

    pub fn export_x_portable(&self, out_dir: &Path) -> Result<XPortableExportReport> {
        let started_at = now();
        let result = export_x_portable(&self.conn, out_dir);
        let completed_at = now();
        match &result {
            Ok(report) => {
                let latest_tweet_updated_at: Option<String> =
                    self.conn
                        .query_row("SELECT MAX(updated_at) FROM x_tweets", [], |row| row.get(0))?;
                let manifest_sha256 = fs::read(&report.manifest_path)
                    .ok()
                    .map(|bytes| sha256(&bytes));
                self.record_x_sync_run(XSyncRunInsert {
                    account_id: None,
                    stream: "export_portable",
                    transport: "local_portable",
                    status: "completed",
                    started_at: &started_at,
                    completed_at: &completed_at,
                    seen: report.rows_exported,
                    inserted: 0,
                    updated: 0,
                    skipped_duplicates: 0,
                    rejected: 0,
                    cursor_key: None,
                    previous_cursor: None,
                    new_cursor: None,
                    error: None,
                    metadata: json!({
                        "out_dir": report.out_dir,
                        "manifest_path": report.manifest_path,
                        "manifest_sha256": manifest_sha256,
                        "generated_at": report.generated_at,
                        "rows_exported": report.rows_exported,
                        "current_tweet_count": self.count("x_tweets")?,
                        "latest_tweet_updated_at": latest_tweet_updated_at,
                        "shards": report.shards,
                        "warnings": report.warnings,
                    }),
                })?;
            }
            Err(error) => {
                self.record_x_sync_run(XSyncRunInsert {
                    account_id: None,
                    stream: "export_portable",
                    transport: "local_portable",
                    status: "failed",
                    started_at: &started_at,
                    completed_at: &completed_at,
                    seen: 0,
                    inserted: 0,
                    updated: 0,
                    skipped_duplicates: 0,
                    rejected: 0,
                    cursor_key: None,
                    previous_cursor: None,
                    new_cursor: None,
                    error: Some(&redact_secret_like_text(&error.to_string())),
                    metadata: json!({ "out_dir": out_dir.display().to_string() }),
                })?;
            }
        }
        result
    }

    pub fn validate_x_portable(&self, dir: &Path) -> Result<XPortableValidateReport> {
        validate_x_portable(dir)
    }

    pub fn import_x_portable(&self, dir: &Path) -> Result<XPortableImportReport> {
        let started_at = now();
        let validation = validate_x_portable(dir)?;
        let rows = read_x_portable_import_rows(dir)?;
        let result = self.import_x_json_value_without_sync_run(&Value::Array(rows));
        let completed_at = now();
        match &result {
            Ok(report) => {
                self.record_x_sync_run(XSyncRunInsert {
                    account_id: None,
                    stream: "import_portable",
                    transport: "local_portable",
                    status: "completed",
                    started_at: &started_at,
                    completed_at: &completed_at,
                    seen: report.seen,
                    inserted: report.imported,
                    updated: 0,
                    skipped_duplicates: report.skipped_duplicates,
                    rejected: report.rejected,
                    cursor_key: None,
                    previous_cursor: None,
                    new_cursor: None,
                    error: None,
                    metadata: json!({ "dir": dir.display().to_string(), "validated_rows": validation.rows }),
                })?;
            }
            Err(error) => {
                self.record_x_sync_run(XSyncRunInsert {
                    account_id: None,
                    stream: "import_portable",
                    transport: "local_portable",
                    status: "failed",
                    started_at: &started_at,
                    completed_at: &completed_at,
                    seen: validation.rows,
                    inserted: 0,
                    updated: 0,
                    skipped_duplicates: 0,
                    rejected: 0,
                    cursor_key: None,
                    previous_cursor: None,
                    new_cursor: None,
                    error: Some(&redact_secret_like_text(&error.to_string())),
                    metadata: json!({ "dir": dir.display().to_string() }),
                })?;
            }
        }
        Ok(XPortableImportReport {
            dir: dir.display().to_string(),
            validation,
            import: result?,
        })
    }

    pub fn import_x_archive(
        &self,
        path: &Path,
        select: &[String],
        limit: usize,
    ) -> Result<XArchiveImportReport> {
        let started_at = now();
        let limit = limit.clamp(1, 100_000);
        let selected = normalize_x_archive_select(select)?;
        let metadata = json!({
            "source": "archive",
            "path": path.display().to_string(),
            "selected": selected.iter().cloned().collect::<Vec<_>>(),
            "limit": limit
        });
        let result = (|| -> Result<XArchiveImportReport> {
            let collected = collect_x_archive_items(path, &selected, limit)?;
            let import =
                self.import_x_json_value_without_sync_run(&Value::Array(collected.items))?;
            Ok(XArchiveImportReport {
                path: path.display().to_string(),
                selected: selected.iter().cloned().collect(),
                files_seen: collected.files_seen,
                files_imported: collected.files_imported,
                bytes_read: collected.bytes_read,
                skipped_files: collected.skipped_files,
                unsupported_slices: collected.unsupported_slices,
                unsupported_files: collected.unsupported_files,
                warnings: collected.warnings,
                import,
            })
        })();
        let completed_at = now();
        match &result {
            Ok(report) => {
                self.record_x_sync_run(XSyncRunInsert {
                    account_id: None,
                    stream: "import_archive",
                    transport: "local_archive",
                    status: "completed",
                    started_at: &started_at,
                    completed_at: &completed_at,
                    seen: report.import.seen,
                    inserted: report.import.imported,
                    updated: 0,
                    skipped_duplicates: report.import.skipped_duplicates,
                    rejected: report.import.rejected,
                    cursor_key: None,
                    previous_cursor: None,
                    new_cursor: None,
                    error: None,
                    metadata,
                })?;
            }
            Err(error) => {
                self.record_x_sync_run(XSyncRunInsert {
                    account_id: None,
                    stream: "import_archive",
                    transport: "local_archive",
                    status: "failed",
                    started_at: &started_at,
                    completed_at: &completed_at,
                    seen: 0,
                    inserted: 0,
                    updated: 0,
                    skipped_duplicates: 0,
                    rejected: 0,
                    cursor_key: None,
                    previous_cursor: None,
                    new_cursor: None,
                    error: Some(&redact_secret_like_text(&error.to_string())),
                    metadata,
                })?;
            }
        }
        result
    }

    pub fn import_x_json_value(&self, value: &Value) -> Result<XImportReport> {
        self.import_x_json_value_with_sync(value, json!({ "source": "value" }))
    }

    pub(crate) fn import_x_json_value_without_sync_run(
        &self,
        value: &Value,
    ) -> Result<XImportReport> {
        self.import_x_json_value_inner(value, None)
    }

    pub(crate) fn import_x_json_value_with_sync(
        &self,
        value: &Value,
        metadata: Value,
    ) -> Result<XImportReport> {
        self.import_x_json_value_inner(value, Some(metadata))
    }

    pub(crate) fn import_x_json_value_inner(
        &self,
        value: &Value,
        sync_metadata: Option<Value>,
    ) -> Result<XImportReport> {
        let started_at = now();
        let items = match value.as_array() {
            Some(items) => items,
            None => {
                if let Some(metadata) = sync_metadata {
                    let completed_at = now();
                    self.record_x_sync_run(XSyncRunInsert {
                        account_id: None,
                        stream: "import_json",
                        transport: "local_json",
                        status: "failed",
                        started_at: &started_at,
                        completed_at: &completed_at,
                        seen: 0,
                        inserted: 0,
                        updated: 0,
                        skipped_duplicates: 0,
                        rejected: 0,
                        cursor_key: None,
                        previous_cursor: None,
                        new_cursor: None,
                        error: Some("expected X import root to be an array"),
                        metadata,
                    })?;
                }
                bail!("expected X import root to be an array");
            }
        };
        let mut imported_items = Vec::new();
        let mut skipped_duplicates = 0;
        let mut rejected = 0;
        let mut rejected_errors = Vec::new();
        for item in items {
            match parse_x_item_input(item).and_then(|input| self.insert_x_item(input)) {
                Ok(Some(item)) => imported_items.push(item),
                Ok(None) => skipped_duplicates += 1,
                Err(error) => {
                    rejected += 1;
                    if rejected_errors.len() < 10 {
                        rejected_errors
                            .push(excerpt(&redact_secret_like_text(&error.to_string()), 500));
                    }
                }
            }
        }
        let report = XImportReport {
            seen: items.len(),
            imported: imported_items.len(),
            skipped_duplicates,
            rejected,
            rejected_errors,
            pages_fetched: None,
            requested_limit: None,
            exhausted: None,
            stop_reason: None,
            next_token: None,
            source_card_projections: Some(
                imported_items
                    .iter()
                    .filter(|item| item.source_card_id.is_some())
                    .count(),
            ),
            drift_warnings: Vec::new(),
            items: imported_items,
        };
        if let Some(metadata) = sync_metadata {
            let completed_at = now();
            self.record_x_sync_run(XSyncRunInsert {
                account_id: None,
                stream: "import_json",
                transport: "local_json",
                status: "completed",
                started_at: &started_at,
                completed_at: &completed_at,
                seen: report.seen,
                inserted: report.imported,
                updated: 0,
                skipped_duplicates: report.skipped_duplicates,
                rejected: report.rejected,
                cursor_key: None,
                previous_cursor: None,
                new_cursor: None,
                error: None,
                metadata,
            })?;
        }
        Ok(report)
    }

    pub fn list_x_items(&self, query: Option<&str>) -> Result<Vec<XItem>> {
        self.list_x_items_filtered(query, None, None)
    }

    pub fn list_x_items_filtered(
        &self,
        query: Option<&str>,
        source_kind: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<XItem>> {
        if let Some(query) = query {
            validate_query(query)?;
        }
        if let Some(source_kind) = source_kind {
            validate_x_item_source_kind(source_kind)?;
        }
        let limit = limit.unwrap_or(100).clamp(1, 1_000) as i64;
        let mut params_vec: Vec<String> = Vec::new();
        let mut where_clauses = Vec::new();
        if let Some(query) = query {
            params_vec.push(format!("%{}%", query));
            where_clauses
                .push("(x.x_id LIKE ? OR x.author LIKE ? OR x.text LIKE ? OR x.url LIKE ?)");
        }
        if let Some(source_kind) = source_kind {
            params_vec.push(source_kind.to_string());
            where_clauses.push(
                "EXISTS (SELECT 1 FROM x_item_sources s WHERE s.x_id = x.x_id AND s.source_kind = ?)",
            );
        }
        let mut sql = String::from(
            r#"
            SELECT x.id, x.x_id, x.author, x.text, x.url, x.created_at, x.imported_at,
                   x.retrieved_at, x.metrics_json, x.raw_json, x.source_card_id, x.wiki_page_id
            FROM x_items x
            "#,
        );
        if !where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_clauses.join(" AND "));
        }
        sql.push_str(
            " ORDER BY COALESCE(x.created_at, x.imported_at) DESC, x.imported_at DESC LIMIT ?",
        );
        let mut params_dyn: Vec<&dyn rusqlite::ToSql> = Vec::new();
        match (query, source_kind) {
            (Some(_), Some(_)) => {
                params_dyn.push(&params_vec[0]);
                params_dyn.push(&params_vec[0]);
                params_dyn.push(&params_vec[0]);
                params_dyn.push(&params_vec[0]);
                params_dyn.push(&params_vec[1]);
            }
            (Some(_), None) => {
                params_dyn.push(&params_vec[0]);
                params_dyn.push(&params_vec[0]);
                params_dyn.push(&params_vec[0]);
                params_dyn.push(&params_vec[0]);
            }
            (None, Some(_)) => {
                params_dyn.push(&params_vec[0]);
            }
            (None, None) => {}
        }
        params_dyn.push(&limit);
        let mut stmt = self.conn.prepare(&sql)?;
        let mut items = rows(stmt.query_map(params_dyn.as_slice(), x_item_from_row)?)?;
        for item in &mut items {
            item.sources = self.list_x_item_sources(&item.x_id)?;
        }
        Ok(items)
    }

    pub fn x_report(&self, query: Option<&str>) -> Result<XReport> {
        let items = self.list_x_items(query)?;
        let links = self.x_report_links_for_items(&items)?;
        let markdown = render_x_report(query, &items, &links);
        Ok(XReport {
            query: query.map(ToOwned::to_owned),
            items,
            links,
            markdown,
        })
    }

    pub fn x_research_brief(&self, query: &str, limit: usize) -> Result<XResearchBrief> {
        validate_query(query)?;
        let limit = limit.clamp(1, 50);
        let items = self.search_x_tweets(query, limit)?;
        if items.is_empty() {
            bail!("x research brief requires at least one local X tweet matching query");
        }
        let mut brief_items = Vec::new();
        for item in items {
            let source_card_id = item.source_card_id.clone().ok_or_else(|| {
                anyhow::anyhow!(
                    "x research brief requires source-card links for every tweet; missing: {}",
                    item.x_id
                )
            })?;
            self.validate_x_research_source_card(&item.x_id, &source_card_id)?;
            let thread = self.x_thread(&item.x_id, 25)?;
            let mut thread_context = Vec::new();
            for tweet in thread
                .tweets
                .into_iter()
                .filter(|tweet| tweet.x_id != item.x_id)
            {
                let thread_source_card_id = tweet.source_card_id.clone().ok_or_else(|| {
                    anyhow::anyhow!(
                        "x research brief requires source-card links for every local thread-context tweet; missing: {}",
                        tweet.x_id
                    )
                })?;
                self.validate_x_research_source_card(&tweet.x_id, &thread_source_card_id)?;
                if thread_context.len() < 8 {
                    thread_context.push(XResearchBriefThreadTweet {
                        x_id: tweet.x_id,
                        author: tweet.author,
                        url: tweet.url,
                        relation_to_root: tweet.relation_to_root,
                        depth: tweet.depth,
                        source_card_id: thread_source_card_id,
                        quote: tweet.text,
                    });
                }
            }
            brief_items.push(XResearchBriefItem {
                x_id: item.x_id,
                author: item.author,
                url: item.url,
                created_at: item.created_at,
                source_card_id,
                wiki_page_id: item.wiki_page_id,
                quote: item.text,
                thread_context,
            });
        }
        let generated_at = now();
        let markdown = render_x_research_brief(query, &generated_at, &brief_items);
        Ok(XResearchBrief {
            query: query.to_string(),
            generated_at,
            no_write: true,
            items: brief_items,
            markdown,
        })
    }

    pub(crate) fn validate_x_research_source_card(
        &self,
        x_id: &str,
        source_card_id: &str,
    ) -> Result<()> {
        let status = self
            .conn
            .query_row(
                r#"
                SELECT xp.status
                FROM x_projections xp
                JOIN source_cards sc ON sc.id = xp.source_card_id
                WHERE xp.entity_kind = 'tweet'
                  AND xp.entity_id = ?1
                  AND xp.projection_kind = 'source_card'
                  AND xp.source_card_id = ?2
                LIMIT 1
                "#,
                params![x_id, source_card_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        match status.as_deref() {
            Some("completed") => Ok(()),
            Some(other) => bail!(
                "x research brief requires completed source-card projection for tweet {}; found status {}",
                x_id,
                other
            ),
            None => bail!(
                "x research brief requires existing source-card projection for tweet {}; missing or dangling source_card_id {}",
                x_id,
                source_card_id
            ),
        }
    }

    pub(crate) fn x_report_links_for_items(&self, items: &[XItem]) -> Result<Vec<XReportLink>> {
        let mut links = Vec::new();
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
              l.tweet_x_id,
              l.url,
              l.display_url,
              l.source,
              COALESCE(e.status, 'unexpanded') AS expansion_status,
              e.wiki_page_id,
              e.final_url,
              e.canonical_url,
              e.last_error
            FROM x_tweet_links l
            LEFT JOIN x_link_expansions e ON e.url = l.url
            WHERE l.tweet_x_id = ?1
            ORDER BY l.last_seen_at DESC, l.url ASC
            "#,
        )?;
        for item in items {
            let item_links = rows(stmt.query_map(params![item.x_id.as_str()], |row| {
                Ok(XReportLink {
                    tweet_x_id: row.get(0)?,
                    url: row.get(1)?,
                    display_url: row.get(2)?,
                    source: row.get(3)?,
                    expansion_status: row.get(4)?,
                    wiki_page_id: row.get(5)?,
                    final_url: row.get(6)?,
                    canonical_url: row.get(7)?,
                    last_error: row.get(8)?,
                })
            })?)?;
            links.extend(item_links);
        }
        Ok(links)
    }
}
