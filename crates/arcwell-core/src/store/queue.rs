use super::*;

impl Store {
    pub(crate) fn active_blog_watch_source_for_url(
        &self,
        url: &str,
    ) -> Result<Option<(WatchSource, String)>> {
        let target = canonical_source_url(url)?;
        for source in self.list_watch_sources()? {
            if source.source_kind != "blog" || source.status != "active" {
                continue;
            }
            if canonical_source_url(&source.locator)? == target {
                let key = watch_source_health_key(&source)?;
                return Ok(Some((source, key)));
            }
        }
        Ok(None)
    }

    pub(crate) fn record_blog_watch_source_success_for_url_ingest(
        &self,
        url: &str,
        page_id: &str,
    ) -> Result<Option<String>> {
        let Some((source, source_key)) = self.active_blog_watch_source_for_url(url)? else {
            return Ok(None);
        };
        let next_run_at = watch_source_cadence_seconds(&source.cadence).map(now_plus_seconds);
        self.record_source_success(SourceHealthUpdate {
            key: &source_key,
            provider: "blog",
            source_kind: "blog",
            locator: &source.locator,
            last_item_id: Some(page_id),
            last_item_date: None,
            cursor_key: None,
            cursor_value: None,
            next_run_at: next_run_at.as_deref(),
        })?;
        Ok(Some(source_key))
    }

    pub(crate) fn record_blog_watch_source_failure_for_url_ingest(
        &self,
        url: &str,
        error: &str,
    ) -> Result<()> {
        if let Some((source, source_key)) = self.active_blog_watch_source_for_url(url)? {
            self.record_source_failure(&source_key, "blog", "blog", &source.locator, error)?;
        }
        Ok(())
    }

    pub fn run_wiki_ingest_file_job(&self, path: &Path) -> Result<WikiJob> {
        let input = json!({ "path": path });
        let job = self.insert_wiki_job("ingest_file", input)?;
        match self.ingest_wiki_file(path) {
            Ok(page_id) => self.complete_wiki_job(&job.id, json!({ "page_id": page_id })),
            Err(error) => self.fail_wiki_job(&job.id, &error.to_string()),
        }
    }

    pub fn run_wiki_ingest_url_job(&self, url: &str) -> Result<WikiJob> {
        let url = validate_fetch_url(url)?;
        self.guard_provider_network_policy(
            "arcwell-llm-wiki",
            "web",
            "url_ingest",
            url.as_str(),
            estimated_network_fetch_cost(1),
            json!({ "entrypoint": "run_wiki_ingest_url_job" }),
        )?;
        let job = self.insert_wiki_job("ingest_url", json!({ "url": url.as_str() }))?;
        let result = (|| -> Result<Value> {
            self.require_cost_budget(
                "arcwell-llm-wiki",
                &job.id,
                "web",
                "url_ingest",
                Some("ingest_url"),
                estimated_network_fetch_cost(1),
                "URL ingest job",
            )?;
            let doc = fetch_url_ingest_document(url.clone())?;
            let markdown = render_url_ingest_page(&doc);
            let page_id = self.add_wiki_page(&doc.title, &markdown, &doc.canonical_url)?;
            let source_health_key =
                self.record_blog_watch_source_success_for_url_ingest(url.as_str(), &page_id)?;
            Ok(json!({
                "page_id": page_id,
                "bytes": doc.byte_len,
                "canonical_url": doc.canonical_url,
                "final_url": doc.final_url,
                "content_type": doc.content_type,
                "source_health_key": source_health_key
            }))
        })();
        match result {
            Ok(result) => self.complete_wiki_job(&job.id, result),
            Err(error) => {
                let error = error.to_string();
                let _ = self.record_blog_watch_source_failure_for_url_ingest(url.as_str(), &error);
                self.fail_wiki_job(&job.id, &error)
            }
        }
    }

    pub fn run_wiki_ingest_rendered_page_job(
        &self,
        input: RenderedPageSnapshotInput,
    ) -> Result<WikiJob> {
        validate_rendered_page_snapshot_input(&input)?;
        let input_json = serde_json::to_value(&input)?;
        let job = self.insert_wiki_job("ingest_rendered_page", input_json.clone())?;
        match self.execute_ingest_rendered_page(&input_json) {
            Ok(result) => self.complete_wiki_job(&job.id, result),
            Err(error) => self.fail_wiki_job(&job.id, &error.to_string()),
        }
    }

    pub fn run_wiki_compile_job(&self, query: &str) -> Result<WikiJob> {
        validate_query(query)?;
        let job = self.insert_wiki_job("compile", json!({ "query": query }))?;
        let result = (|| -> Result<Value> {
            let brief = self.create_research_brief_from_wiki(query, true)?;
            Ok(json!({
                "run_id": brief.run.id,
                "page_id": brief.result_page_id,
                "source_count": brief.source_count
            }))
        })();
        match result {
            Ok(result) => self.complete_wiki_job(&job.id, result),
            Err(error) => self.fail_wiki_job(&job.id, &error.to_string()),
        }
    }

    pub fn run_wiki_expand_page_job(&self, topic: &str) -> Result<WikiJob> {
        validate_query(topic)?;
        let job = self.insert_wiki_job("expand_page", json!({ "topic": topic }))?;
        let result = (|| -> Result<Value> {
            let sources = self.search_source_cards(topic)?;
            let pages = self.search_wiki_pages_for_research(topic)?;
            let markdown = render_expanded_wiki_page(topic, &sources, &pages)?;
            let page_id =
                self.add_wiki_page(&format!("Expanded: {topic}"), &markdown, "wiki-expand")?;
            Ok(json!({
                "page_id": page_id,
                "source_cards": sources.len(),
                "wiki_pages": pages.len()
            }))
        })();
        match result {
            Ok(result) => self.complete_wiki_job(&job.id, result),
            Err(error) => self.fail_wiki_job(&job.id, &error.to_string()),
        }
    }

    pub fn enqueue_wiki_job(&self, kind: &str, input_json: Value) -> Result<WikiJob> {
        validate_job_kind(kind)?;
        self.guard_wiki_job_enqueue_policy(kind, &input_json)?;
        if let Some(job) = self.find_active_duplicate_wiki_job(kind, &input_json)? {
            return Ok(job);
        }
        self.insert_wiki_job_with_status(kind, "pending", input_json)
    }

    pub(crate) fn find_active_duplicate_wiki_job(
        &self,
        kind: &str,
        input_json: &Value,
    ) -> Result<Option<WikiJob>> {
        validate_job_kind(kind)?;
        let input_json = serde_json::to_string(input_json)?;
        self.conn
            .query_row(
                r#"
                SELECT id, kind, status, input_json, result_json, error,
                       attempts, max_attempts, leased_until, worker_id, next_run_at, dead_lettered_at,
                       created_at, updated_at
                FROM wiki_jobs
                WHERE kind = ?1
                  AND input_json = ?2
                  AND status IN ('pending', 'deferred', 'running')
                ORDER BY
                    CASE status
                        WHEN 'running' THEN 0
                        WHEN 'pending' THEN 1
                        WHEN 'deferred' THEN 2
                        ELSE 10
                    END ASC,
                    created_at DESC
                LIMIT 1
                "#,
                params![kind, input_json],
                wiki_job_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn enqueue_rss_job(&self, url: &str) -> Result<WikiJob> {
        let url = validate_fetch_url(url)?;
        self.enqueue_wiki_job("rss_fetch", json!({ "url": url.as_str() }))
    }

    pub fn enqueue_github_repo_job(
        &self,
        owner: &str,
        repo: &str,
        mode: &str,
        limit: usize,
    ) -> Result<WikiJob> {
        validate_github_segment(owner)?;
        validate_github_segment(repo)?;
        validate_github_mode(mode)?;
        self.enqueue_wiki_job(
            "github_repo",
            json!({ "owner": owner, "repo": repo, "mode": mode, "limit": limit.clamp(1, 30) }),
        )
    }

    pub fn enqueue_github_owner_job(&self, owner: &str, limit: usize) -> Result<WikiJob> {
        validate_github_segment(owner)?;
        self.enqueue_wiki_job(
            "github_owner",
            json!({ "owner": owner, "limit": limit.clamp(1, 30) }),
        )
    }

    pub fn enqueue_arxiv_search_job(&self, query: &str, limit: usize) -> Result<WikiJob> {
        validate_query(query)?;
        self.enqueue_wiki_job(
            "arxiv_search",
            json!({ "query": query, "limit": limit.clamp(1, 30) }),
        )
    }

    pub fn enqueue_hackernews_fetch_job(&self, feed: &str, limit: usize) -> Result<WikiJob> {
        let feed = normalize_hackernews_feed(feed)?;
        self.enqueue_wiki_job(
            "hackernews_fetch",
            json!({ "feed": feed, "limit": limit.clamp(1, 30) }),
        )
    }

    pub fn enqueue_reddit_fetch_job(&self, locator: &str, limit: usize) -> Result<WikiJob> {
        let locator = normalize_reddit_locator(locator)?;
        self.enqueue_wiki_job(
            "reddit_fetch",
            json!({ "locator": locator.source_detail(), "limit": limit.clamp(1, 30) }),
        )
    }

    pub fn enqueue_x_recent_search_job(&self, query: &str, max_results: usize) -> Result<WikiJob> {
        validate_query(query)?;
        self.enqueue_wiki_job(
            "x_recent_search",
            json!({ "query": query, "max_results": max_results.clamp(10, 100) }),
        )
    }

    pub fn enqueue_x_import_bookmarks_job(
        &self,
        bookmark_days: i64,
        max_bookmarks: usize,
    ) -> Result<WikiJob> {
        self.enqueue_wiki_job(
            "x_import_bookmarks",
            json!({
                "bookmark_days": bookmark_days.clamp(1, 36_500),
                "max_bookmarks": max_bookmarks.clamp(1, 100_000)
            }),
        )
    }

    pub fn schedule_x_bookmark_import(
        &self,
        bookmark_days: i64,
        max_bookmarks: usize,
        cadence: &str,
        status: &str,
    ) -> Result<WatchSource> {
        self.upsert_watch_source(WatchSourceInput {
            source_kind: "x_bookmarks".to_string(),
            locator: "bookmarks".to_string(),
            label: "X bookmarks".to_string(),
            cadence: cadence.to_string(),
            status: status.to_string(),
            metadata: json!({
                "bookmark_days": bookmark_days.clamp(1, 36_500),
                "max_bookmarks": max_bookmarks.clamp(1, 100_000),
                "origin": "x_schedule_bookmarks",
            }),
        })
    }

    pub fn enqueue_x_monitor_watch_source_job(
        &self,
        handle: &str,
        max_results: usize,
    ) -> Result<WikiJob> {
        let handle = handle.trim().trim_start_matches('@');
        validate_x_handle(handle)?;
        self.enqueue_wiki_job(
            "x_monitor_watch_source",
            json!({ "handle": handle, "max_results": max_results.clamp(10, 100) }),
        )
    }

    pub fn enqueue_radar_run_job(
        &self,
        profile_id_or_name: &str,
        window_hours: Option<i64>,
        fetch_live: bool,
    ) -> Result<WikiJob> {
        let profile = self
            .read_radar_profile(profile_id_or_name)?
            .with_context(|| format!("radar profile not found: {profile_id_or_name}"))?;
        if let Some(window_hours) = window_hours
            && window_hours <= 0
        {
            bail!("window_hours must be greater than zero");
        }
        self.enqueue_wiki_job(
            "radar_run",
            json!({
                "profile": profile.id,
                "window_hours": window_hours,
                "fetch_live": fetch_live
            }),
        )
    }

    pub fn enqueue_knowledge_cluster_expansion_job(
        &self,
        cluster_id: &str,
        create_digest: bool,
    ) -> Result<WikiJob> {
        self.enqueue_knowledge_cluster_expansion_job_with_lineage(cluster_id, create_digest, None)
    }

    pub fn enqueue_knowledge_cluster_editorial_decision_job(
        &self,
        cluster_id: &str,
        auto_enqueue: bool,
    ) -> Result<WikiJob> {
        self.enqueue_knowledge_cluster_editorial_decision_job_with_lineage(
            cluster_id,
            auto_enqueue,
            None,
        )
    }

    pub(crate) fn enqueue_knowledge_cluster_editorial_decision_job_with_lineage(
        &self,
        cluster_id: &str,
        auto_enqueue: bool,
        lineage: Option<Value>,
    ) -> Result<WikiJob> {
        validate_id(cluster_id)?;
        self.get_knowledge_cluster(cluster_id)?
            .with_context(|| format!("knowledge cluster not found: {cluster_id}"))?;
        if self.knowledge_cluster_editorial_decision_has_active_job(cluster_id)? {
            bail!(
                "knowledge cluster editorial decision job already active for cluster {cluster_id}"
            );
        }
        let mut input = json!({ "cluster_id": cluster_id, "auto_enqueue": auto_enqueue });
        if let Some(lineage) = lineage
            && let Some(object) = input.as_object_mut()
        {
            object.insert("lineage".to_string(), lineage);
        }
        self.enqueue_wiki_job("knowledge_cluster_editorial_decide", input)
    }

    pub(crate) fn enqueue_knowledge_cluster_expansion_job_with_lineage(
        &self,
        cluster_id: &str,
        create_digest: bool,
        lineage: Option<Value>,
    ) -> Result<WikiJob> {
        validate_id(cluster_id)?;
        let cluster = self
            .get_knowledge_cluster(cluster_id)?
            .with_context(|| format!("knowledge cluster not found: {cluster_id}"))?;
        ensure_knowledge_cluster_can_expand(&cluster)?;
        let mut input = json!({ "cluster_id": cluster_id, "create_digest": create_digest });
        if let Some(lineage) = lineage
            && let Some(object) = input.as_object_mut()
        {
            object.insert("lineage".to_string(), lineage);
        }
        self.enqueue_wiki_job("knowledge_cluster_expand", input)
    }

    pub fn enqueue_knowledge_cluster_model_writer_job(
        &self,
        cluster_id: &str,
        model_provider: &str,
        model_name: Option<&str>,
        endpoint: Option<&str>,
        timeout_seconds: Option<u64>,
        create_digest: bool,
    ) -> Result<WikiJob> {
        self.enqueue_knowledge_cluster_model_writer_job_with_lineage(
            cluster_id,
            model_provider,
            model_name,
            endpoint,
            timeout_seconds,
            create_digest,
            None,
        )
    }

    pub(crate) fn enqueue_knowledge_cluster_model_writer_job_with_lineage(
        &self,
        cluster_id: &str,
        model_provider: &str,
        model_name: Option<&str>,
        endpoint: Option<&str>,
        timeout_seconds: Option<u64>,
        create_digest: bool,
        lineage: Option<Value>,
    ) -> Result<WikiJob> {
        validate_id(cluster_id)?;
        let cluster = self
            .get_knowledge_cluster(cluster_id)?
            .with_context(|| format!("knowledge cluster not found: {cluster_id}"))?;
        ensure_knowledge_cluster_can_expand(&cluster)?;
        if self.knowledge_cluster_model_writer_has_active_job(cluster_id)? {
            bail!("knowledge cluster model writer job already active for cluster {cluster_id}");
        }
        let provider = model_provider.trim().to_ascii_lowercase();
        if !matches!(provider.as_str(), "mock" | "openai") {
            bail!("unsupported knowledge cluster writer model provider: {provider}");
        }
        let model_name = model_name
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        if let Some(model_name) = &model_name {
            validate_key(model_name)?;
        }
        if let Some(endpoint) = endpoint {
            validated_endpoint(Some(endpoint), "https://api.openai.com/v1/responses")?;
        }
        let mut input = json!({
            "cluster_id": cluster_id,
            "model_provider": provider,
            "model_name": model_name,
            "endpoint": endpoint,
            "timeout_seconds": timeout_seconds,
            "create_digest": create_digest,
        });
        if let Some(lineage) = lineage
            && let Some(object) = input.as_object_mut()
        {
            object.insert("lineage".to_string(), lineage);
        }
        self.enqueue_wiki_job("knowledge_cluster_model_write", input)
    }

    pub fn schedule_job_radar_refresh(
        &self,
        profile_id: &str,
        scope: &str,
        source_ids: Vec<String>,
        fetch_live: bool,
        source_snapshots: Value,
        cadence: &str,
        status: &str,
    ) -> Result<WatchSource> {
        let profile = self.require_job_profile(profile_id)?;
        let scope = sanitize_required_job_text(scope, "job radar scope", JOB_MAX_TEXT)?;
        let source_ids = self.validate_job_source_ids(source_ids)?;
        let source_snapshots = sanitize_work_json(source_snapshots)?;
        validate_watch_source_cadence(cadence)?;
        validate_watch_source_status(status)?;
        self.upsert_watch_source(WatchSourceInput {
            source_kind: "job_radar".to_string(),
            locator: profile.id.clone(),
            label: format!("Job radar: {scope}"),
            cadence: cadence.to_string(),
            status: status.to_string(),
            metadata: json!({
                "origin": "job_radar_schedule",
                "profile_id": profile.id,
                "scope": scope,
                "source_ids": source_ids,
                "fetch_live": fetch_live,
                "source_snapshots": source_snapshots,
            }),
        })
    }

    pub fn enqueue_job_radar_refresh_job(
        &self,
        profile_id: &str,
        scope: &str,
        source_ids: Vec<String>,
        fetch_live: bool,
        source_snapshots: Value,
    ) -> Result<WikiJob> {
        self.enqueue_job_radar_refresh_job_with_lineage(
            profile_id,
            scope,
            source_ids,
            fetch_live,
            source_snapshots,
            None,
        )
    }

    pub(crate) fn enqueue_job_radar_refresh_job_with_lineage(
        &self,
        profile_id: &str,
        scope: &str,
        source_ids: Vec<String>,
        fetch_live: bool,
        source_snapshots: Value,
        lineage: Option<Value>,
    ) -> Result<WikiJob> {
        let profile = self.require_job_profile(profile_id)?;
        let scope = sanitize_required_job_text(scope, "job radar scope", JOB_MAX_TEXT)?;
        let source_ids = self.validate_job_source_ids(source_ids)?;
        let source_snapshots = sanitize_work_json(source_snapshots)?;
        let proof_level =
            job_radar_refresh_derived_proof_level(fetch_live, &source_ids, &source_snapshots)?;
        let mut input = json!({
            "profile_id": profile.id,
            "scope": scope,
            "source_ids": source_ids,
            "fetch_live": fetch_live,
            "source_snapshots": source_snapshots,
            "proof_level": proof_level,
        });
        if let Some(lineage) = lineage {
            input["lineage"] = sanitize_work_json(lineage)?;
        }
        self.enqueue_wiki_job("job_radar_refresh", input)
    }

    pub fn schedule_knowledge_cluster_model_write(
        &self,
        cluster_id: &str,
        model_provider: &str,
        model_name: Option<&str>,
        endpoint: Option<&str>,
        timeout_seconds: Option<u64>,
        create_digest: bool,
        cadence: &str,
        status: &str,
    ) -> Result<WatchSource> {
        validate_id(cluster_id)?;
        let cluster = self
            .get_knowledge_cluster(cluster_id)?
            .with_context(|| format!("knowledge cluster not found: {cluster_id}"))?;
        ensure_knowledge_cluster_can_expand(&cluster)?;
        let provider = model_provider.trim().to_ascii_lowercase();
        if !matches!(provider.as_str(), "mock" | "openai") {
            bail!("unsupported knowledge cluster writer model provider: {provider}");
        }
        let model_name = model_name
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        if let Some(model_name) = &model_name {
            validate_key(model_name)?;
        }
        if let Some(endpoint) = endpoint {
            validated_endpoint(Some(endpoint), "https://api.openai.com/v1/responses")?;
        }
        self.upsert_watch_source(WatchSourceInput {
            source_kind: "knowledge_model_write".to_string(),
            locator: cluster_id.to_string(),
            label: format!("Knowledge model writer: {}", cluster.topic),
            cadence: cadence.to_string(),
            status: status.to_string(),
            metadata: json!({
                "cluster_id": cluster_id,
                "model_provider": provider,
                "model_name": model_name,
                "endpoint": endpoint,
                "timeout_seconds": timeout_seconds,
                "create_digest": create_digest,
                "origin": "knowledge_model_write_schedule",
                "boundary": "Scheduled model writing is cluster-scoped and still requires promotion, provider policy, cost, source-card citations, wiki/report quality gates, and separate digest delivery policy."
            }),
        })
    }

    pub fn enqueue_knowledge_entity_resolution_model_job(
        &self,
        left_entity_id: &str,
        right_entity_id: &str,
        model_provider: &str,
        model_name: Option<&str>,
        endpoint: Option<&str>,
        timeout_seconds: Option<u64>,
    ) -> Result<WikiJob> {
        self.enqueue_knowledge_entity_resolution_model_job_with_lineage(
            left_entity_id,
            right_entity_id,
            model_provider,
            model_name,
            endpoint,
            timeout_seconds,
            None,
        )
    }

    pub(crate) fn enqueue_knowledge_entity_resolution_model_job_with_lineage(
        &self,
        left_entity_id: &str,
        right_entity_id: &str,
        model_provider: &str,
        model_name: Option<&str>,
        endpoint: Option<&str>,
        timeout_seconds: Option<u64>,
        lineage: Option<Value>,
    ) -> Result<WikiJob> {
        validate_id(left_entity_id)?;
        validate_id(right_entity_id)?;
        if left_entity_id == right_entity_id {
            bail!("knowledge entity resolution model job requires two different entities");
        }
        let (left_entity_id, right_entity_id) = if left_entity_id <= right_entity_id {
            (left_entity_id.to_string(), right_entity_id.to_string())
        } else {
            (right_entity_id.to_string(), left_entity_id.to_string())
        };
        self.get_knowledge_entity(&left_entity_id)?
            .with_context(|| format!("left knowledge entity not found: {left_entity_id}"))?;
        self.get_knowledge_entity(&right_entity_id)?
            .with_context(|| format!("right knowledge entity not found: {right_entity_id}"))?;
        if self
            .knowledge_entity_resolution_model_has_active_job(&left_entity_id, &right_entity_id)?
        {
            bail!(
                "knowledge entity resolution model job already active for pair {left_entity_id}/{right_entity_id}"
            );
        }
        let provider = model_provider.trim().to_ascii_lowercase();
        if !matches!(provider.as_str(), "mock" | "openai") {
            bail!("unsupported knowledge entity resolution model provider: {provider}");
        }
        let model_name = model_name
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        if let Some(model_name) = &model_name {
            validate_key(model_name)?;
        }
        if let Some(endpoint) = endpoint {
            validated_endpoint(Some(endpoint), "https://api.openai.com/v1/responses")?;
        }
        let mut input = json!({
            "left_entity_id": left_entity_id,
            "right_entity_id": right_entity_id,
            "model_provider": provider,
            "model_name": model_name,
            "endpoint": endpoint,
            "timeout_seconds": timeout_seconds,
        });
        if let Some(lineage) = lineage
            && let Some(object) = input.as_object_mut()
        {
            object.insert("lineage".to_string(), lineage);
        }
        self.enqueue_wiki_job("knowledge_entity_resolution_model", input)
    }

    pub fn schedule_knowledge_entity_resolution(
        &self,
        model_provider: &str,
        model_name: Option<&str>,
        endpoint: Option<&str>,
        timeout_seconds: Option<u64>,
        max_pairs: usize,
        cadence: &str,
        status: &str,
    ) -> Result<WatchSource> {
        let provider = model_provider.trim().to_ascii_lowercase();
        if !matches!(provider.as_str(), "mock" | "openai") {
            bail!("unsupported knowledge entity resolution model provider: {provider}");
        }
        let model_name = model_name
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        if let Some(model_name) = &model_name {
            validate_key(model_name)?;
        }
        if let Some(endpoint) = endpoint {
            validated_endpoint(Some(endpoint), "https://api.openai.com/v1/responses")?;
        }
        self.upsert_watch_source(WatchSourceInput {
            source_kind: "knowledge_entity_resolution".to_string(),
            locator: "entities".to_string(),
            label: "Knowledge entity resolution".to_string(),
            cadence: cadence.to_string(),
            status: status.to_string(),
            metadata: json!({
                "model_provider": provider,
                "model_name": model_name,
                "endpoint": endpoint,
                "timeout_seconds": timeout_seconds,
                "max_pairs": max_pairs.clamp(1, 100),
                "origin": "knowledge_entity_resolution_schedule",
                "boundary": "Scheduled entity resolution writes review-only proposals from source-card-backed entity pairs; it cannot merge entities or create relations."
            }),
        })
    }

    pub(crate) fn enqueue_due_knowledge_entity_resolution_job_from_source(
        &self,
        source: &WatchSource,
        source_key: &str,
    ) -> Result<Option<WikiJob>> {
        let model_provider = source
            .metadata
            .get("model_provider")
            .and_then(Value::as_str)
            .unwrap_or("mock");
        let model_name = source.metadata.get("model_name").and_then(Value::as_str);
        let endpoint = source.metadata.get("endpoint").and_then(Value::as_str);
        let timeout_seconds = source
            .metadata
            .get("timeout_seconds")
            .and_then(Value::as_u64);
        let max_pairs = source
            .metadata
            .get("max_pairs")
            .and_then(Value::as_u64)
            .unwrap_or(25) as usize;
        let mut report = self.enqueue_due_knowledge_entity_resolution_jobs(
            max_pairs,
            model_provider,
            model_name,
            endpoint,
            timeout_seconds,
            Some(json!({
                "trigger": "watch_source_due",
                "watch_source_id": source.id,
                "watch_source_key": source_key,
                "source_kind": source.source_kind,
                "locator": source.locator,
                "cadence": source.cadence,
                "metadata": source.metadata,
            })),
        )?;
        if let Some(job_id) = report.jobs.pop() {
            return self.get_wiki_job(&job_id)?.map(Some).with_context(|| {
                format!("enqueued knowledge entity resolution job not found: {job_id}")
            });
        }
        self.record_source_success(SourceHealthUpdate {
            key: source_key,
            provider: "arcwell",
            source_kind: "knowledge_entity_resolution",
            locator: &source.locator,
            last_item_id: None,
            last_item_date: None,
            cursor_key: None,
            cursor_value: None,
            next_run_at: Some(&now_plus_seconds(self.watch_source_next_run_seconds(
                "knowledge_entity_resolution",
                &source.locator,
                6 * 60 * 60,
            ))),
        })?;
        Ok(None)
    }

    pub(crate) fn enqueue_due_knowledge_cluster_model_write_job_from_source(
        &self,
        source: &WatchSource,
        source_key: &str,
    ) -> Result<WikiJob> {
        let cluster_id = source
            .metadata
            .get("cluster_id")
            .and_then(Value::as_str)
            .unwrap_or(source.locator.as_str());
        validate_id(cluster_id)?;
        if let Some(status) = self.knowledge_cluster_model_writer_decision_status(cluster_id)?
            && matches!(status.as_str(), "completed" | "blocked")
        {
            bail!("knowledge cluster model writer already has terminal decision for {cluster_id}");
        }
        let model_provider = source
            .metadata
            .get("model_provider")
            .and_then(Value::as_str)
            .unwrap_or("mock");
        let model_name = source.metadata.get("model_name").and_then(Value::as_str);
        let endpoint = source.metadata.get("endpoint").and_then(Value::as_str);
        let timeout_seconds = source
            .metadata
            .get("timeout_seconds")
            .and_then(Value::as_u64);
        let create_digest = source
            .metadata
            .get("create_digest")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        self.enqueue_knowledge_cluster_model_writer_job_with_lineage(
            cluster_id,
            model_provider,
            model_name,
            endpoint,
            timeout_seconds,
            create_digest,
            Some(json!({
                "trigger": "watch_source_due",
                "watch_source_id": source.id,
                "watch_source_key": source_key,
                "source_kind": source.source_kind,
                "locator": source.locator,
                "cluster_id": cluster_id,
                "cadence": source.cadence,
                "metadata": source.metadata,
            })),
        )
    }

    pub fn enqueue_knowledge_cluster_investigation_job(&self, cluster_id: &str) -> Result<WikiJob> {
        validate_id(cluster_id)?;
        self.get_knowledge_cluster(cluster_id)?
            .with_context(|| format!("knowledge cluster not found: {cluster_id}"))?;
        if self.knowledge_cluster_investigation_has_active_job(cluster_id)? {
            bail!("knowledge cluster investigation job already active for cluster {cluster_id}");
        }
        self.enqueue_wiki_job(
            "knowledge_cluster_investigate",
            json!({ "cluster_id": cluster_id }),
        )
    }

    pub fn enqueue_knowledge_cluster_investigation_execution_job(
        &self,
        cluster_id: &str,
    ) -> Result<WikiJob> {
        self.enqueue_knowledge_cluster_investigation_execution_job_with_lineage(cluster_id, None)
    }

    pub(crate) fn enqueue_knowledge_cluster_investigation_execution_job_with_lineage(
        &self,
        cluster_id: &str,
        lineage: Option<Value>,
    ) -> Result<WikiJob> {
        validate_id(cluster_id)?;
        self.get_knowledge_cluster(cluster_id)?
            .with_context(|| format!("knowledge cluster not found: {cluster_id}"))?;
        if self.knowledge_cluster_investigation_execution_has_active_job(cluster_id)? {
            bail!(
                "knowledge cluster investigation execution job already active for cluster {cluster_id}"
            );
        }
        let mut input = json!({ "cluster_id": cluster_id });
        if let Some(lineage) = lineage
            && let Some(object) = input.as_object_mut()
        {
            object.insert("lineage".to_string(), lineage);
        }
        self.enqueue_wiki_job("knowledge_cluster_investigation_execute", input)
    }

    pub fn enqueue_knowledge_cluster_backlog_job(
        &self,
        max_source_cards: usize,
        min_group_size: usize,
        max_clusters: usize,
    ) -> Result<WikiJob> {
        self.enqueue_knowledge_cluster_backlog_job_with_lineage(
            max_source_cards,
            min_group_size,
            max_clusters,
            None,
        )
    }

    pub(crate) fn enqueue_knowledge_cluster_backlog_job_with_lineage(
        &self,
        max_source_cards: usize,
        min_group_size: usize,
        max_clusters: usize,
        lineage: Option<Value>,
    ) -> Result<WikiJob> {
        let mut input = json!({
            "max_source_cards": max_source_cards.clamp(1, 500),
            "min_group_size": min_group_size.clamp(1, 20),
            "max_clusters": max_clusters.clamp(1, 50),
        });
        if let Some(lineage) = lineage
            && let Some(object) = input.as_object_mut()
        {
            object.insert("lineage".to_string(), lineage);
        }
        self.enqueue_wiki_job("knowledge_cluster_backlog", input)
    }

    pub fn enqueue_knowledge_cluster_model_proposal_job(
        &self,
        query: &str,
        model_provider: &str,
        model_name: Option<&str>,
        endpoint: Option<&str>,
        timeout_seconds: Option<u64>,
        max_source_cards: usize,
        max_clusters: usize,
    ) -> Result<WikiJob> {
        self.enqueue_knowledge_cluster_model_proposal_job_with_lineage(
            query,
            model_provider,
            model_name,
            endpoint,
            timeout_seconds,
            max_source_cards,
            max_clusters,
            None,
        )
    }

    pub(crate) fn enqueue_knowledge_cluster_model_proposal_job_with_lineage(
        &self,
        query: &str,
        model_provider: &str,
        model_name: Option<&str>,
        endpoint: Option<&str>,
        timeout_seconds: Option<u64>,
        max_source_cards: usize,
        max_clusters: usize,
        lineage: Option<Value>,
    ) -> Result<WikiJob> {
        let query = normalize_knowledge_model_cluster_query(query)?;
        let provider = model_provider.trim().to_ascii_lowercase();
        if !matches!(provider.as_str(), "mock" | "openai") {
            bail!("unsupported knowledge cluster proposal model provider: {provider}");
        }
        let model_name = model_name
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        if let Some(model_name) = &model_name {
            validate_key(model_name)?;
        }
        if let Some(endpoint) = endpoint {
            validated_endpoint(Some(endpoint), "https://api.openai.com/v1/responses")?;
        }
        let mut input = json!({
            "query": query,
            "model_provider": provider,
            "model_name": model_name,
            "endpoint": endpoint,
            "timeout_seconds": timeout_seconds,
            "max_source_cards": max_source_cards.clamp(1, 80),
            "max_clusters": max_clusters.clamp(1, 12),
        });
        if let Some(lineage) = lineage
            && let Some(object) = input.as_object_mut()
        {
            object.insert("lineage".to_string(), lineage);
        }
        self.enqueue_wiki_job("knowledge_cluster_model_propose", input)
    }

    pub fn schedule_knowledge_cluster_backlog(
        &self,
        max_source_cards: usize,
        min_group_size: usize,
        max_clusters: usize,
        cadence: &str,
        status: &str,
    ) -> Result<WatchSource> {
        self.upsert_watch_source(WatchSourceInput {
            source_kind: "knowledge_backlog".to_string(),
            locator: "source-cards".to_string(),
            label: "Knowledge source-card backlog".to_string(),
            cadence: cadence.to_string(),
            status: status.to_string(),
            metadata: json!({
                "max_source_cards": max_source_cards.clamp(1, 500),
                "min_group_size": min_group_size.clamp(1, 20),
                "max_clusters": max_clusters.clamp(1, 50),
                "origin": "knowledge_backlog_schedule",
            }),
        })
    }

    pub fn schedule_knowledge_cluster_model_proposals(
        &self,
        query: &str,
        model_provider: &str,
        model_name: Option<&str>,
        endpoint: Option<&str>,
        timeout_seconds: Option<u64>,
        max_source_cards: usize,
        max_clusters: usize,
        cadence: &str,
        status: &str,
    ) -> Result<WatchSource> {
        let query = normalize_knowledge_model_cluster_query(query)?;
        let provider = model_provider.trim().to_ascii_lowercase();
        if !matches!(provider.as_str(), "mock" | "openai") {
            bail!("unsupported knowledge cluster proposal model provider: {provider}");
        }
        let model_name = model_name
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        if let Some(model_name) = &model_name {
            validate_key(model_name)?;
        }
        if let Some(endpoint) = endpoint {
            validated_endpoint(Some(endpoint), "https://api.openai.com/v1/responses")?;
        }
        let broad_source_card_sweep = knowledge_model_cluster_query_is_broad(&query);
        self.upsert_watch_source(WatchSourceInput {
            source_kind: "knowledge_model_clusters".to_string(),
            locator: query.clone(),
            label: format!("Knowledge model clusters: {query}"),
            cadence: cadence.to_string(),
            status: status.to_string(),
            metadata: json!({
                "query": query,
                "model_provider": provider,
                "model_name": model_name,
                "endpoint": endpoint,
                "timeout_seconds": timeout_seconds,
                "max_source_cards": max_source_cards.clamp(1, 80),
                "max_clusters": max_clusters.clamp(1, 12),
                "broad_source_card_sweep": broad_source_card_sweep,
                "origin": "knowledge_model_cluster_schedule",
                "boundary": "Scheduled model clustering writes review-only candidate clusters; wiki/report/digest expansion still requires promotion."
            }),
        })
    }

    pub fn enqueue_due_knowledge_cluster_expansion_jobs(
        &self,
        max_clusters: usize,
    ) -> Result<KnowledgeClusterExpansionEnqueueReport> {
        let mut report = KnowledgeClusterExpansionEnqueueReport {
            inspected: 0,
            enqueued: 0,
            skipped: 0,
            jobs: Vec::new(),
            errors: Vec::new(),
        };
        for cluster in self
            .list_knowledge_clusters(max_clusters.clamp(1, 100))?
            .into_iter()
        {
            report.inspected += 1;
            if !matches!(cluster.status.as_str(), "candidate" | "active") {
                report.skipped += 1;
                continue;
            }
            if knowledge_cluster_requires_model_promotion(&cluster) {
                report.skipped += 1;
                continue;
            }
            if let Some(status) =
                self.knowledge_cluster_model_writer_decision_status(&cluster.id)?
                && matches!(status.as_str(), "completed" | "blocked")
            {
                report.skipped += 1;
                continue;
            }
            if let Some(status) = self.knowledge_cluster_expansion_decision_status(&cluster.id)?
                && matches!(status.as_str(), "completed" | "blocked")
            {
                report.skipped += 1;
                continue;
            }
            if self.knowledge_cluster_expansion_has_active_job(&cluster.id)? {
                report.skipped += 1;
                continue;
            }
            if self.knowledge_cluster_editorial_decision_has_active_job(&cluster.id)? {
                report.skipped += 1;
                continue;
            }
            if self.knowledge_cluster_model_writer_has_active_job(&cluster.id)? {
                report.skipped += 1;
                continue;
            }
            match self.enqueue_knowledge_cluster_expansion_job(&cluster.id, true) {
                Ok(job) => {
                    report.enqueued += 1;
                    report.jobs.push(job.id);
                }
                Err(error) => {
                    report.skipped += 1;
                    report
                        .errors
                        .push(format!("{}:{}: {error}", cluster.id, cluster.topic));
                }
            }
        }
        Ok(report)
    }

    pub fn enqueue_due_knowledge_cluster_editorial_decision_jobs(
        &self,
        max_clusters: usize,
    ) -> Result<KnowledgeClusterEditorialDecisionEnqueueReport> {
        let mut report = KnowledgeClusterEditorialDecisionEnqueueReport {
            inspected: 0,
            enqueued: 0,
            skipped: 0,
            jobs: Vec::new(),
            errors: Vec::new(),
        };
        for cluster in self
            .list_knowledge_clusters(max_clusters.clamp(1, 100))?
            .into_iter()
        {
            report.inspected += 1;
            if !matches!(cluster.status.as_str(), "candidate" | "active") {
                report.skipped += 1;
                continue;
            }
            if knowledge_cluster_requires_model_promotion(&cluster) {
                report.skipped += 1;
                continue;
            }
            if let Some(existing) =
                self.get_knowledge_editorial_decision_for_cluster(&cluster.id, "editorial_decide")?
                && matches!(existing.status.as_str(), "completed" | "blocked")
                && knowledge_editorial_decision_matches_cluster_revision(&existing, &cluster)
            {
                report.skipped += 1;
                continue;
            }
            if let Some(status) =
                self.knowledge_cluster_model_writer_decision_status(&cluster.id)?
                && matches!(status.as_str(), "completed" | "blocked")
            {
                report.skipped += 1;
                continue;
            }
            if let Some(status) = self.knowledge_cluster_expansion_decision_status(&cluster.id)?
                && matches!(status.as_str(), "completed" | "blocked")
            {
                report.skipped += 1;
                continue;
            }
            if self.knowledge_cluster_editorial_decision_has_active_job(&cluster.id)? {
                report.skipped += 1;
                continue;
            }
            if self.knowledge_cluster_expansion_has_active_job(&cluster.id)? {
                report.skipped += 1;
                continue;
            }
            if self.knowledge_cluster_model_writer_has_active_job(&cluster.id)? {
                report.skipped += 1;
                continue;
            }
            match self.enqueue_knowledge_cluster_editorial_decision_job_with_lineage(
                &cluster.id,
                true,
                Some(json!({
                    "trigger": "due_cluster_recurrence",
                    "cluster_id": cluster.id.clone(),
                    "topic": cluster.topic.clone(),
                    "source_card_count": cluster.source_card_ids.len(),
                    "source_card_ids": cluster.source_card_ids.clone(),
                    "boundary": "Due recurrence records a durable editorial decision before any wiki/report/digest follow-up."
                })),
            ) {
                Ok(job) => {
                    report.enqueued += 1;
                    report.jobs.push(job.id);
                }
                Err(error) => {
                    report.skipped += 1;
                    report
                        .errors
                        .push(format!("{}:{}: {error}", cluster.id, cluster.topic));
                }
            }
        }
        Ok(report)
    }

    pub fn enqueue_due_knowledge_cluster_model_writer_jobs(
        &self,
        max_clusters: usize,
        model_provider: &str,
        model_name: Option<&str>,
        endpoint: Option<&str>,
        timeout_seconds: Option<u64>,
        create_digest: bool,
    ) -> Result<KnowledgeClusterModelWriterEnqueueReport> {
        let provider = model_provider.trim().to_ascii_lowercase();
        if !matches!(provider.as_str(), "mock" | "openai") {
            bail!("unsupported knowledge cluster writer model provider: {provider}");
        }
        let model_name = model_name
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        if let Some(model_name) = &model_name {
            validate_key(model_name)?;
        }
        if let Some(endpoint) = endpoint {
            validated_endpoint(Some(endpoint), "https://api.openai.com/v1/responses")?;
        }
        let mut report = KnowledgeClusterModelWriterEnqueueReport {
            inspected: 0,
            enqueued: 0,
            skipped: 0,
            jobs: Vec::new(),
            errors: Vec::new(),
        };
        for cluster in self
            .list_knowledge_clusters(max_clusters.clamp(1, 100))?
            .into_iter()
        {
            report.inspected += 1;
            if cluster.status != "active" {
                report.skipped += 1;
                continue;
            }
            if cluster.metadata.get("origin").and_then(Value::as_str)
                != Some("model_cluster_proposal_v1")
            {
                report.skipped += 1;
                continue;
            }
            if let Some(status) =
                self.knowledge_cluster_model_writer_decision_status(&cluster.id)?
                && matches!(status.as_str(), "completed" | "blocked")
            {
                report.skipped += 1;
                continue;
            }
            if let Some(status) = self.knowledge_cluster_expansion_decision_status(&cluster.id)?
                && matches!(status.as_str(), "completed" | "blocked")
            {
                report.skipped += 1;
                continue;
            }
            if self.knowledge_cluster_model_writer_has_active_job(&cluster.id)? {
                report.skipped += 1;
                continue;
            }
            if self.knowledge_cluster_expansion_has_active_job(&cluster.id)? {
                report.skipped += 1;
                continue;
            }
            if self.knowledge_cluster_editorial_decision_has_active_job(&cluster.id)? {
                report.skipped += 1;
                continue;
            }
            match self.enqueue_knowledge_cluster_model_writer_job_with_lineage(
                &cluster.id,
                &provider,
                model_name.as_deref(),
                endpoint,
                timeout_seconds,
                create_digest,
                Some(json!({
                    "trigger": "due_promoted_model_cluster_recurrence",
                    "cluster_id": cluster.id.clone(),
                    "topic": cluster.topic.clone(),
                    "source_card_count": cluster.source_card_ids.len(),
                    "source_card_ids": cluster.source_card_ids.clone(),
                    "boundary": "Due recurrence enqueues a local model-writer job for a promoted model-origin cluster only; external delivery remains a separate reviewed digest policy gate."
                })),
            ) {
                Ok(job) => {
                    report.enqueued += 1;
                    report.jobs.push(job.id);
                }
                Err(error) => {
                    report.skipped += 1;
                    report
                        .errors
                        .push(format!("{}:{}: {error}", cluster.id, cluster.topic));
                }
            }
        }
        Ok(report)
    }

    pub fn enqueue_due_knowledge_entity_resolution_jobs(
        &self,
        max_pairs: usize,
        model_provider: &str,
        model_name: Option<&str>,
        endpoint: Option<&str>,
        timeout_seconds: Option<u64>,
        lineage: Option<Value>,
    ) -> Result<KnowledgeEntityResolutionEnqueueReport> {
        let provider = model_provider.trim().to_ascii_lowercase();
        if !matches!(provider.as_str(), "mock" | "openai") {
            bail!("unsupported knowledge entity resolution model provider: {provider}");
        }
        let model_name = model_name
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        if let Some(model_name) = &model_name {
            validate_key(model_name)?;
        }
        if let Some(endpoint) = endpoint {
            validated_endpoint(Some(endpoint), "https://api.openai.com/v1/responses")?;
        }
        let mut report = KnowledgeEntityResolutionEnqueueReport {
            inspected: 0,
            enqueued: 0,
            skipped: 0,
            jobs: Vec::new(),
            errors: Vec::new(),
        };
        for (left, right) in self.knowledge_entity_resolution_candidate_pairs(max_pairs)? {
            report.inspected += 1;
            if self.knowledge_entity_resolution_model_has_active_job(&left.id, &right.id)? {
                report.skipped += 1;
                continue;
            }
            if self
                .knowledge_entity_resolution_has_model_proposal(&left.id, &right.id, &provider)?
            {
                report.skipped += 1;
                continue;
            }
            match self.enqueue_knowledge_entity_resolution_model_job_with_lineage(
                &left.id,
                &right.id,
                &provider,
                model_name.as_deref(),
                endpoint,
                timeout_seconds,
                lineage.clone().map(|mut value| {
                    if let Some(object) = value.as_object_mut() {
                        object.insert("left_entity_id".to_string(), json!(left.id.clone()));
                        object.insert("right_entity_id".to_string(), json!(right.id.clone()));
                        object.insert(
                            "boundary".to_string(),
                            json!("Due recurrence enqueues a review-only entity-resolution model job; entity identity remains unchanged until separate human/policy review."),
                        );
                    }
                    value
                }),
            ) {
                Ok(job) => {
                    report.enqueued += 1;
                    report.jobs.push(job.id);
                }
                Err(error) => {
                    report.skipped += 1;
                    report.errors.push(format!(
                        "{}:{} / {}:{}: {error}",
                        left.entity_type, left.name, right.entity_type, right.name
                    ));
                }
            }
        }
        Ok(report)
    }

    pub fn enqueue_due_knowledge_cluster_investigation_execution_jobs(
        &self,
        max_clusters: usize,
    ) -> Result<KnowledgeClusterInvestigationExecutionEnqueueReport> {
        let mut report = KnowledgeClusterInvestigationExecutionEnqueueReport {
            inspected: 0,
            enqueued: 0,
            skipped: 0,
            jobs: Vec::new(),
            errors: Vec::new(),
        };
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, cluster_id, decision, status, wiki_page_id, digest_candidate_id,
                   source_card_ids_json, reason, quality_findings_json, metadata_json,
                   created_at, updated_at
            FROM knowledge_editorial_decisions
            WHERE decision = 'investigate_cluster'
              AND status = 'completed'
            ORDER BY updated_at DESC
            LIMIT ?1
            "#,
        )?;
        let decisions = rows(stmt.query_map(
            params![max_clusters.clamp(1, 100)],
            knowledge_editorial_decision_from_row,
        )?)?;
        drop(stmt);
        for decision in decisions {
            report.inspected += 1;
            let cluster = match self.get_knowledge_cluster(&decision.cluster_id)? {
                Some(cluster) => cluster,
                None => {
                    report.skipped += 1;
                    report.errors.push(format!(
                        "{}: knowledge cluster not found",
                        decision.cluster_id
                    ));
                    continue;
                }
            };
            if let Some(status) =
                self.knowledge_cluster_investigation_execution_decision_status(&cluster.id)?
                && matches!(status.as_str(), "completed" | "blocked")
            {
                report.skipped += 1;
                continue;
            }
            if self.knowledge_cluster_investigation_execution_has_active_job(&cluster.id)? {
                report.skipped += 1;
                continue;
            }
            let Some(run_id) = decision
                .metadata
                .get("research_run_id")
                .and_then(Value::as_str)
            else {
                report.skipped += 1;
                report.errors.push(format!(
                    "{}:{}: investigation decision missing research_run_id",
                    cluster.id, cluster.topic
                ));
                continue;
            };
            let run = match self.get_research_run(run_id)? {
                Some(run) => run,
                None => {
                    report.skipped += 1;
                    report.errors.push(format!(
                        "{}:{}: research run not found: {}",
                        cluster.id, cluster.topic, run_id
                    ));
                    continue;
                }
            };
            if matches!(
                run.status.as_str(),
                "stopped" | "completed" | "completed_no_write"
            ) {
                report.skipped += 1;
                continue;
            }
            let tasks = self.list_research_tasks(&run.id)?;
            if !tasks.iter().any(|task| task.status == "pending") {
                report.skipped += 1;
                continue;
            }
            match self.enqueue_knowledge_cluster_investigation_execution_job(&cluster.id) {
                Ok(job) => {
                    report.enqueued += 1;
                    report.jobs.push(job.id);
                }
                Err(error) => {
                    report.skipped += 1;
                    report
                        .errors
                        .push(format!("{}:{}: {error}", cluster.id, cluster.topic));
                }
            }
        }
        Ok(report)
    }

    pub(crate) fn knowledge_cluster_expansion_decision_status(
        &self,
        cluster_id: &str,
    ) -> Result<Option<String>> {
        validate_id(cluster_id)?;
        let cluster = self
            .get_knowledge_cluster(cluster_id)?
            .with_context(|| format!("knowledge cluster not found: {cluster_id}"))?;
        for decision in ["expand_wiki_and_digest", "expand_wiki"] {
            if let Some(existing) =
                self.get_knowledge_editorial_decision_for_cluster(cluster_id, decision)?
                && knowledge_editorial_decision_matches_cluster_revision(&existing, &cluster)
            {
                return Ok(Some(existing.status));
            }
        }
        Ok(None)
    }

    pub(crate) fn knowledge_cluster_model_writer_decision_status(
        &self,
        cluster_id: &str,
    ) -> Result<Option<String>> {
        validate_id(cluster_id)?;
        let cluster = self
            .get_knowledge_cluster(cluster_id)?
            .with_context(|| format!("knowledge cluster not found: {cluster_id}"))?;
        for decision in ["model_write_wiki_and_digest", "model_write_wiki"] {
            if let Some(existing) =
                self.get_knowledge_editorial_decision_for_cluster(cluster_id, decision)?
                && knowledge_editorial_decision_matches_cluster_revision(&existing, &cluster)
            {
                return Ok(Some(existing.status));
            }
        }
        Ok(None)
    }

    pub(crate) fn knowledge_cluster_investigation_execution_decision_status(
        &self,
        cluster_id: &str,
    ) -> Result<Option<String>> {
        validate_id(cluster_id)?;
        let cluster = self
            .get_knowledge_cluster(cluster_id)?
            .with_context(|| format!("knowledge cluster not found: {cluster_id}"))?;
        Ok(self
            .get_knowledge_editorial_decision_for_cluster(
                cluster_id,
                "execute_investigation_tasks",
            )?
            .filter(|decision| {
                knowledge_editorial_decision_matches_cluster_revision(decision, &cluster)
            })
            .map(|decision| decision.status))
    }

    pub(crate) fn knowledge_cluster_expansion_has_active_job(
        &self,
        cluster_id: &str,
    ) -> Result<bool> {
        validate_id(cluster_id)?;
        self.conn
            .query_row(
                r#"
                SELECT COUNT(*)
                FROM wiki_jobs
                WHERE kind = 'knowledge_cluster_expand'
                  AND status IN ('pending', 'running', 'deferred')
                  AND json_extract(input_json, '$.cluster_id') = ?1
                "#,
                params![cluster_id],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count > 0)
            .map_err(Into::into)
    }

    pub(crate) fn knowledge_cluster_backlog_has_active_job(&self) -> Result<bool> {
        self.conn
            .query_row(
                r#"
                SELECT COUNT(*)
                FROM wiki_jobs
                WHERE kind = 'knowledge_cluster_backlog'
                  AND status IN ('pending', 'running', 'deferred')
                "#,
                [],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count > 0)
            .map_err(Into::into)
    }

    pub(crate) fn knowledge_cluster_model_proposal_has_active_job(
        &self,
        query: &str,
    ) -> Result<bool> {
        validate_query(query)?;
        self.conn
            .query_row(
                r#"
                SELECT COUNT(*)
                FROM wiki_jobs
                WHERE kind = 'knowledge_cluster_model_propose'
                  AND status IN ('pending', 'running', 'deferred')
                  AND json_extract(input_json, '$.query') = ?1
                "#,
                params![query],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count > 0)
            .map_err(Into::into)
    }

    pub(crate) fn knowledge_cluster_model_writer_has_active_job(
        &self,
        cluster_id: &str,
    ) -> Result<bool> {
        validate_id(cluster_id)?;
        self.conn
            .query_row(
                r#"
                SELECT COUNT(*)
                FROM wiki_jobs
                WHERE kind = 'knowledge_cluster_model_write'
                  AND (
                    status IN ('pending', 'running', 'deferred')
                    OR (status = 'failed' AND attempts < max_attempts)
                  )
                  AND json_extract(input_json, '$.cluster_id') = ?1
                "#,
                params![cluster_id],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count > 0)
            .map_err(Into::into)
    }

    pub(crate) fn knowledge_entity_resolution_model_has_active_job(
        &self,
        left_entity_id: &str,
        right_entity_id: &str,
    ) -> Result<bool> {
        validate_id(left_entity_id)?;
        validate_id(right_entity_id)?;
        let (left_entity_id, right_entity_id) = if left_entity_id <= right_entity_id {
            (left_entity_id.to_string(), right_entity_id.to_string())
        } else {
            (right_entity_id.to_string(), left_entity_id.to_string())
        };
        self.conn
            .query_row(
                r#"
                SELECT COUNT(*)
                FROM wiki_jobs
                WHERE kind = 'knowledge_entity_resolution_model'
                  AND (
                    status IN ('pending', 'running', 'deferred')
                    OR (status = 'failed' AND attempts < max_attempts)
                  )
                  AND json_extract(input_json, '$.left_entity_id') = ?1
                  AND json_extract(input_json, '$.right_entity_id') = ?2
                "#,
                params![left_entity_id, right_entity_id],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count > 0)
            .map_err(Into::into)
    }

    pub(crate) fn knowledge_entity_resolution_has_model_proposal(
        &self,
        left_entity_id: &str,
        right_entity_id: &str,
        model_provider: &str,
    ) -> Result<bool> {
        validate_id(left_entity_id)?;
        validate_id(right_entity_id)?;
        let resolver = format!("{}-model-v1", model_provider.trim().to_ascii_lowercase());
        let pair_key =
            knowledge_entity_resolution_pair_key(left_entity_id, right_entity_id, &resolver);
        let id = format!("keres-{}", &sha256(pair_key.as_bytes())[..16]);
        self.conn
            .query_row(
                r#"
                SELECT COUNT(*)
                FROM knowledge_entity_resolutions
                WHERE id = ?1
                  AND status IN ('pending_review', 'resolved', 'rejected', 'blocked')
                "#,
                params![id],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count > 0)
            .map_err(Into::into)
    }

    pub(crate) fn knowledge_entity_resolution_candidate_pairs(
        &self,
        max_pairs: usize,
    ) -> Result<Vec<(KnowledgeEntity, KnowledgeEntity)>> {
        let entities = self.list_knowledge_entities(500)?;
        let mut pairs = Vec::new();
        'outer: for left_index in 0..entities.len() {
            let left = &entities[left_index];
            for right in entities.iter().skip(left_index + 1) {
                if knowledge_entity_resolution_proposal(left, right).is_none() {
                    continue;
                }
                pairs.push((left.clone(), right.clone()));
                if pairs.len() >= max_pairs.clamp(1, 100) {
                    break 'outer;
                }
            }
        }
        Ok(pairs)
    }

    pub(crate) fn knowledge_cluster_editorial_decision_has_active_job(
        &self,
        cluster_id: &str,
    ) -> Result<bool> {
        validate_id(cluster_id)?;
        self.conn
            .query_row(
                r#"
                SELECT COUNT(*)
                FROM wiki_jobs
                WHERE kind = 'knowledge_cluster_editorial_decide'
                  AND (
                    status IN ('pending', 'running', 'deferred')
                    OR (status = 'failed' AND attempts < max_attempts)
                  )
                  AND json_extract(input_json, '$.cluster_id') = ?1
                "#,
                params![cluster_id],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count > 0)
            .map_err(Into::into)
    }

    pub(crate) fn knowledge_cluster_investigation_has_active_job(
        &self,
        cluster_id: &str,
    ) -> Result<bool> {
        validate_id(cluster_id)?;
        self.conn
            .query_row(
                r#"
                SELECT COUNT(*)
                FROM wiki_jobs
                WHERE kind = 'knowledge_cluster_investigate'
                  AND status IN ('pending', 'running', 'deferred')
                  AND json_extract(input_json, '$.cluster_id') = ?1
                "#,
                params![cluster_id],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count > 0)
            .map_err(Into::into)
    }

    pub(crate) fn knowledge_cluster_investigation_execution_has_active_job(
        &self,
        cluster_id: &str,
    ) -> Result<bool> {
        validate_id(cluster_id)?;
        self.conn
            .query_row(
                r#"
                SELECT COUNT(*)
                FROM wiki_jobs
                WHERE kind = 'knowledge_cluster_investigation_execute'
                  AND status IN ('pending', 'running', 'deferred')
                  AND json_extract(input_json, '$.cluster_id') = ?1
                "#,
                params![cluster_id],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count > 0)
            .map_err(Into::into)
    }

    pub(crate) fn job_radar_refresh_has_active_job(&self, profile_id: &str) -> Result<bool> {
        validate_id(profile_id)?;
        self.conn
            .query_row(
                r#"
                SELECT COUNT(*)
                FROM wiki_jobs
                WHERE kind = 'job_radar_refresh'
                  AND (
                    status IN ('pending', 'running', 'deferred')
                    OR (status = 'failed' AND attempts < max_attempts)
                  )
                  AND json_extract(input_json, '$.profile_id') = ?1
                "#,
                params![profile_id],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count > 0)
            .map_err(Into::into)
    }

    pub fn enqueue_due_radar_schedule_jobs(
        &self,
        max_profiles: usize,
    ) -> Result<RadarScheduleEnqueueReport> {
        let mut report = RadarScheduleEnqueueReport {
            inspected: 0,
            enqueued: 0,
            skipped: 0,
            jobs: Vec::new(),
            errors: Vec::new(),
        };
        for profile in self
            .list_radar_profiles()?
            .into_iter()
            .take(max_profiles.clamp(1, 100))
        {
            report.inspected += 1;
            let policy = match scheduled_radar_delivery_policy(&profile) {
                Ok(Some(policy)) => policy,
                Ok(None) => {
                    report.skipped += 1;
                    continue;
                }
                Err(error) => {
                    report.skipped += 1;
                    report.errors.push(format!("{}: {error}", profile.name));
                    continue;
                }
            };
            if self.radar_schedule_has_active_job(&profile.id)? {
                report.skipped += 1;
                continue;
            }
            if let Some(latest_due_at) = self.latest_radar_schedule_due_at(&profile.id)?
                && !radar_schedule_interval_elapsed(&latest_due_at, policy.interval_hours)
            {
                report.skipped += 1;
                continue;
            }
            let due_at = radar_schedule_due_slot(policy.interval_hours);
            let tick_key = radar_schedule_tick_key(&profile.id, &due_at, &policy);
            if self.get_radar_schedule_tick_by_key(&tick_key)?.is_some() {
                report.skipped += 1;
                continue;
            }
            match self.create_radar_schedule_tick(&profile.id, &tick_key, &due_at) {
                Ok(tick) => match self
                    .enqueue_wiki_job("radar_scheduled_delivery", json!({ "tick_id": tick.id }))
                {
                    Ok(job) => {
                        self.attach_radar_schedule_job(&tick.id, &job.id)?;
                        report.enqueued += 1;
                        report.jobs.push(job.id);
                    }
                    Err(error) => {
                        self.update_radar_schedule_tick(
                            &tick.id,
                            "blocked",
                            None,
                            None,
                            None,
                            Some(&error.to_string()),
                        )?;
                        report.skipped += 1;
                        report.errors.push(format!("{}: {error}", profile.name));
                    }
                },
                Err(error) => {
                    report.skipped += 1;
                    report.errors.push(format!("{}: {error}", profile.name));
                }
            }
        }
        Ok(report)
    }

    pub(crate) fn radar_schedule_has_active_job(&self, profile_id: &str) -> Result<bool> {
        validate_id(profile_id)?;
        let count: i64 = self.conn.query_row(
            r#"
            SELECT COUNT(*)
            FROM radar_schedule_ticks t
            JOIN wiki_jobs j ON j.id = t.job_id
            WHERE t.profile_id = ?1
              AND t.status IN ('pending', 'running', 'deferred')
              AND j.status IN ('pending', 'running', 'failed', 'deferred')
            "#,
            params![profile_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub(crate) fn latest_radar_schedule_due_at(&self, profile_id: &str) -> Result<Option<String>> {
        validate_id(profile_id)?;
        self.conn
            .query_row(
                r#"
                SELECT due_at
                FROM radar_schedule_ticks
                WHERE profile_id = ?1
                ORDER BY due_at DESC, created_at DESC
                LIMIT 1
                "#,
                params![profile_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(Into::into)
    }

    pub(crate) fn get_radar_schedule_tick(&self, id: &str) -> Result<Option<RadarScheduleTick>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, profile_id, tick_key, due_at, status, job_id, run_id,
                       summary_id, delivery_id, error, created_at, updated_at
                FROM radar_schedule_ticks
                WHERE id = ?1
                "#,
                params![id],
                radar_schedule_tick_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub(crate) fn get_radar_schedule_tick_by_key(
        &self,
        tick_key: &str,
    ) -> Result<Option<RadarScheduleTick>> {
        validate_notes(tick_key)?;
        self.conn
            .query_row(
                r#"
                SELECT id, profile_id, tick_key, due_at, status, job_id, run_id,
                       summary_id, delivery_id, error, created_at, updated_at
                FROM radar_schedule_ticks
                WHERE tick_key = ?1
                "#,
                params![tick_key],
                radar_schedule_tick_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub(crate) fn create_radar_schedule_tick(
        &self,
        profile_id: &str,
        tick_key: &str,
        due_at: &str,
    ) -> Result<RadarScheduleTick> {
        validate_id(profile_id)?;
        validate_notes(tick_key)?;
        validate_timestamp(due_at)?;
        let id = Uuid::new_v4().to_string();
        let timestamp = now();
        self.conn.execute(
            r#"
            INSERT INTO radar_schedule_ticks
              (id, profile_id, tick_key, due_at, status, job_id, run_id,
               summary_id, delivery_id, error, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, 'pending', NULL, NULL, NULL, NULL, NULL, ?5, ?5)
            "#,
            params![id, profile_id, tick_key, due_at, timestamp],
        )?;
        self.get_radar_schedule_tick(&id)?
            .with_context(|| format!("radar schedule tick not found after insert: {id}"))
    }

    pub(crate) fn attach_radar_schedule_job(
        &self,
        tick_id: &str,
        job_id: &str,
    ) -> Result<RadarScheduleTick> {
        validate_id(tick_id)?;
        validate_id(job_id)?;
        let updated_at = now();
        self.conn.execute(
            r#"
            UPDATE radar_schedule_ticks
            SET job_id = ?2,
                status = 'pending',
                updated_at = ?3
            WHERE id = ?1
            "#,
            params![tick_id, job_id, updated_at],
        )?;
        self.get_radar_schedule_tick(tick_id)?
            .with_context(|| format!("radar schedule tick not found after job attach: {tick_id}"))
    }

    pub(crate) fn update_radar_schedule_tick(
        &self,
        tick_id: &str,
        status: &str,
        run_id: Option<&str>,
        summary_id: Option<&str>,
        delivery_id: Option<&str>,
        error: Option<&str>,
    ) -> Result<RadarScheduleTick> {
        validate_id(tick_id)?;
        validate_radar_schedule_status(status)?;
        if let Some(run_id) = run_id {
            validate_id(run_id)?;
        }
        if let Some(summary_id) = summary_id {
            validate_notes(summary_id)?;
        }
        if let Some(delivery_id) = delivery_id {
            validate_id(delivery_id)?;
        }
        let error = error.map(sanitize_radar_delivery_error).transpose()?;
        let updated_at = now();
        self.conn.execute(
            r#"
            UPDATE radar_schedule_ticks
            SET status = ?2,
                run_id = COALESCE(?3, run_id),
                summary_id = COALESCE(?4, summary_id),
                delivery_id = COALESCE(?5, delivery_id),
                error = ?6,
                updated_at = ?7
            WHERE id = ?1
            "#,
            params![
                tick_id,
                status,
                run_id,
                summary_id,
                delivery_id,
                error.as_deref(),
                updated_at
            ],
        )?;
        self.get_radar_schedule_tick(tick_id)?
            .with_context(|| format!("radar schedule tick not found after update: {tick_id}"))
    }

    pub(crate) fn update_radar_schedule_ticks_for_delivery(
        &self,
        delivery_id: &str,
        status: &str,
        error: Option<&str>,
    ) -> Result<usize> {
        validate_id(delivery_id)?;
        validate_radar_schedule_status(status)?;
        let error = error.map(sanitize_radar_delivery_error).transpose()?;
        let updated_at = now();
        let updated = self.conn.execute(
            r#"
            UPDATE radar_schedule_ticks
            SET status = ?2,
                error = ?3,
                updated_at = ?4
            WHERE delivery_id = ?1
              AND status IN ('pending', 'running', 'failed', 'deferred', 'dead_lettered')
            "#,
            params![delivery_id, status, error.as_deref(), updated_at],
        )?;
        Ok(updated)
    }

    pub fn enqueue_research_convergence_job(
        &self,
        input: ResearchConvergenceStepInput,
    ) -> Result<WikiJob> {
        normalize_research_convergence_config(&input)?;
        self.require_research_run(&input.run_id)?;
        self.enqueue_wiki_job("research_convergence_run", serde_json::to_value(input)?)
    }

    pub(crate) fn guard_wiki_job_enqueue_policy(&self, kind: &str, input: &Value) -> Result<()> {
        let (package, provider, target, projected_usd) = wiki_job_policy_context(kind, input);
        self.policy_guard(PolicyRequest {
            action: "worker.enqueue".to_string(),
            package: Some(package.to_string()),
            provider: provider.map(ToOwned::to_owned),
            source: Some(kind.to_string()),
            channel: None,
            subject: None,
            target,
            projected_usd,
            metadata: json!({ "kind": kind, "input": policy_safe_job_input(input) }),
            untrusted_excerpt: None,
        })?;
        Ok(())
    }

    pub(crate) fn guard_provider_network_policy(
        &self,
        package: &str,
        provider: &str,
        source: &str,
        target: &str,
        projected_usd: f64,
        metadata: Value,
    ) -> Result<()> {
        self.policy_guard(PolicyRequest {
            action: "provider.network".to_string(),
            package: Some(package.to_string()),
            provider: Some(provider.to_string()),
            source: Some(source.to_string()),
            channel: None,
            subject: None,
            target: Some(excerpt(target, 240)),
            projected_usd: Some(projected_usd),
            metadata,
            untrusted_excerpt: None,
        })?;
        Ok(())
    }

    pub fn enqueue_due_watch_source_jobs(
        &self,
        max_sources: usize,
    ) -> Result<WatchSourcePollEnqueueReport> {
        let max_sources = max_sources.clamp(1, 100);
        let mut report = WatchSourcePollEnqueueReport {
            inspected: 0,
            enqueued: 0,
            skipped: 0,
            jobs: Vec::new(),
            errors: Vec::new(),
        };
        for source in self.list_watch_sources()? {
            if source.status != "active" {
                continue;
            }
            let source_key = watch_source_health_key(&source)?;
            if let Some(health) = self.get_source_health(&source_key)?
                && let Some(next_run_at) = health.next_run_at.as_deref()
                && !timestamp_is_due(next_run_at)
            {
                continue;
            }
            if report.inspected >= max_sources {
                break;
            }
            report.inspected += 1;
            let job = match source.source_kind.as_str() {
                "rss" => self.enqueue_rss_job(&source.locator),
                "blog" => self.enqueue_wiki_job("ingest_url", json!({ "url": source.locator })),
                "github_owner" => self.enqueue_github_owner_job(&source.locator, 10),
                "arxiv_query" => self.enqueue_arxiv_search_job(&source.locator, 10),
                "hackernews" => self.enqueue_hackernews_fetch_job(&source.locator, 10),
                "reddit" => self.enqueue_reddit_fetch_job(&source.locator, 10),
                "x_bookmarks" => {
                    let bookmark_days = source
                        .metadata
                        .get("bookmark_days")
                        .and_then(Value::as_i64)
                        .unwrap_or(92);
                    let max_bookmarks = source
                        .metadata
                        .get("max_bookmarks")
                        .and_then(Value::as_u64)
                        .unwrap_or(100) as usize;
                    self.enqueue_x_import_bookmarks_job(bookmark_days, max_bookmarks)
                }
                "x_handle" => self.enqueue_x_monitor_watch_source_job(&source.locator, 20),
                "knowledge_backlog" => {
                    if self.knowledge_cluster_backlog_has_active_job()? {
                        report.skipped += 1;
                        continue;
                    }
                    let max_source_cards = source
                        .metadata
                        .get("max_source_cards")
                        .and_then(Value::as_u64)
                        .unwrap_or(100) as usize;
                    let min_group_size = source
                        .metadata
                        .get("min_group_size")
                        .and_then(Value::as_u64)
                        .unwrap_or(2) as usize;
                    let max_clusters = source
                        .metadata
                        .get("max_clusters")
                        .and_then(Value::as_u64)
                        .unwrap_or(12) as usize;
                    self.enqueue_knowledge_cluster_backlog_job_with_lineage(
                        max_source_cards,
                        min_group_size,
                        max_clusters,
                        Some(json!({
                            "trigger": "watch_source_due",
                            "watch_source_id": source.id,
                            "watch_source_key": source_key,
                            "source_kind": source.source_kind,
                            "locator": source.locator,
                            "cadence": source.cadence,
                            "metadata": source.metadata,
                        })),
                    )
                }
                "knowledge_model_clusters" => {
                    let query = source
                        .metadata
                        .get("query")
                        .and_then(Value::as_str)
                        .unwrap_or(source.locator.as_str());
                    let query = normalize_knowledge_model_cluster_query(query)?;
                    if self.knowledge_cluster_model_proposal_has_active_job(&query)? {
                        report.skipped += 1;
                        continue;
                    }
                    let model_provider = source
                        .metadata
                        .get("model_provider")
                        .and_then(Value::as_str)
                        .unwrap_or("mock");
                    let model_name = source.metadata.get("model_name").and_then(Value::as_str);
                    let endpoint = source.metadata.get("endpoint").and_then(Value::as_str);
                    let timeout_seconds = source
                        .metadata
                        .get("timeout_seconds")
                        .and_then(Value::as_u64);
                    let max_source_cards = source
                        .metadata
                        .get("max_source_cards")
                        .and_then(Value::as_u64)
                        .unwrap_or(24) as usize;
                    let max_clusters = source
                        .metadata
                        .get("max_clusters")
                        .and_then(Value::as_u64)
                        .unwrap_or(6) as usize;
                    self.enqueue_knowledge_cluster_model_proposal_job_with_lineage(
                        &query,
                        model_provider,
                        model_name,
                        endpoint,
                        timeout_seconds,
                        max_source_cards,
                        max_clusters,
                        Some(json!({
                            "trigger": "watch_source_due",
                            "watch_source_id": source.id,
                            "watch_source_key": source_key,
                            "source_kind": source.source_kind,
                            "locator": source.locator,
                            "cadence": source.cadence,
                            "metadata": source.metadata,
                        })),
                    )
                }
                "knowledge_model_write" => {
                    let cluster_id = source
                        .metadata
                        .get("cluster_id")
                        .and_then(Value::as_str)
                        .unwrap_or(source.locator.as_str());
                    if self.knowledge_cluster_model_writer_has_active_job(cluster_id)? {
                        report.skipped += 1;
                        continue;
                    }
                    if let Some(status) =
                        self.knowledge_cluster_model_writer_decision_status(cluster_id)?
                        && matches!(status.as_str(), "completed" | "blocked")
                    {
                        self.record_source_success(SourceHealthUpdate {
                            key: &source_key,
                            provider: "arcwell",
                            source_kind: "knowledge_model_write",
                            locator: &source.locator,
                            last_item_id: Some(cluster_id),
                            last_item_date: None,
                            cursor_key: None,
                            cursor_value: None,
                            next_run_at: Some(&now_plus_seconds(3600)),
                        })?;
                        report.skipped += 1;
                        continue;
                    }
                    self.enqueue_due_knowledge_cluster_model_write_job_from_source(
                        &source,
                        &source_key,
                    )
                }
                "knowledge_entity_resolution" => {
                    let maybe_job = self.enqueue_due_knowledge_entity_resolution_job_from_source(
                        &source,
                        &source_key,
                    )?;
                    let Some(job) = maybe_job else {
                        report.skipped += 1;
                        continue;
                    };
                    Ok(job)
                }
                "job_radar" => {
                    let profile_id = source
                        .metadata
                        .get("profile_id")
                        .and_then(Value::as_str)
                        .unwrap_or(source.locator.as_str());
                    if self.job_radar_refresh_has_active_job(profile_id)? {
                        report.skipped += 1;
                        continue;
                    }
                    let scope = source
                        .metadata
                        .get("scope")
                        .and_then(Value::as_str)
                        .unwrap_or("scheduled job radar refresh");
                    let source_ids = job_source_ids_from_value(source.metadata.get("source_ids"))?;
                    let fetch_live = source
                        .metadata
                        .get("fetch_live")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    let source_snapshots = source
                        .metadata
                        .get("source_snapshots")
                        .cloned()
                        .unwrap_or_else(|| json!({}));
                    self.enqueue_job_radar_refresh_job_with_lineage(
                        profile_id,
                        scope,
                        source_ids,
                        fetch_live,
                        source_snapshots,
                        Some(json!({
                            "trigger": "watch_source_due",
                            "watch_source_id": source.id,
                            "watch_source_key": source_key,
                            "source_kind": source.source_kind,
                            "locator": source.locator,
                            "cadence": source.cadence,
                            "metadata": source.metadata,
                        })),
                    )
                }
                other => Err(anyhow::anyhow!("unsupported watch source kind: {other}")),
            };
            match job {
                Ok(job) => {
                    report.enqueued += 1;
                    report.jobs.push(job.id);
                }
                Err(error) => {
                    report.skipped += 1;
                    report.errors.push(format!(
                        "{}:{}: {error}",
                        source.source_kind, source.locator
                    ));
                    let _ = self.record_source_failure(
                        &source_key,
                        &source.source_kind,
                        &source.source_kind,
                        &source.locator,
                        &error.to_string(),
                    );
                }
            }
        }
        Ok(report)
    }

    pub fn run_worker_once(&self, max_jobs: usize) -> Result<WorkerRunReport> {
        let max_jobs = max_jobs.clamp(1, 100);
        let watch_poll = self.enqueue_due_watch_source_jobs(max_jobs)?;
        let watch_poll = if watch_poll.inspected > 0 {
            Some(watch_poll)
        } else {
            None
        };
        let radar_schedule = self.enqueue_due_radar_schedule_jobs(max_jobs)?;
        let radar_schedule = if radar_schedule.inspected > 0 {
            Some(radar_schedule)
        } else {
            None
        };
        let digest_alert_schedule = self.enqueue_due_digest_alert_schedule_jobs(max_jobs)?;
        let digest_alert_schedule = if digest_alert_schedule.inspected > 0 {
            Some(digest_alert_schedule)
        } else {
            None
        };
        let issue_schedule = self.enqueue_due_issue_schedule_jobs(max_jobs)?;
        let issue_schedule = if issue_schedule.inspected > 0 {
            Some(issue_schedule)
        } else {
            None
        };
        let knowledge_cluster_model_writer = self.enqueue_due_knowledge_cluster_model_writer_jobs(
            max_jobs, "mock", None, None, None, true,
        )?;
        let knowledge_cluster_model_writer = if knowledge_cluster_model_writer.inspected > 0 {
            Some(knowledge_cluster_model_writer)
        } else {
            None
        };
        let knowledge_entity_resolution = self.enqueue_due_knowledge_entity_resolution_jobs(
            max_jobs, "mock", None, None, None, None,
        )?;
        let knowledge_entity_resolution = if knowledge_entity_resolution.inspected > 0 {
            Some(knowledge_entity_resolution)
        } else {
            None
        };
        let knowledge_cluster_editorial_decision =
            self.enqueue_due_knowledge_cluster_editorial_decision_jobs(max_jobs)?;
        let knowledge_cluster_editorial_decision =
            if knowledge_cluster_editorial_decision.inspected > 0 {
                Some(knowledge_cluster_editorial_decision)
            } else {
                None
            };
        let knowledge_cluster_expansion =
            self.enqueue_due_knowledge_cluster_expansion_jobs(max_jobs)?;
        let knowledge_cluster_expansion = if knowledge_cluster_expansion.inspected > 0 {
            Some(knowledge_cluster_expansion)
        } else {
            None
        };
        let knowledge_cluster_investigation_execution =
            self.enqueue_due_knowledge_cluster_investigation_execution_jobs(max_jobs)?;
        let knowledge_cluster_investigation_execution =
            if knowledge_cluster_investigation_execution.inspected > 0 {
                Some(knowledge_cluster_investigation_execution)
            } else {
                None
            };
        let mut jobs = Vec::new();
        for _ in 0..max_jobs {
            let Some(job) = self.claim_next_pending_job()? else {
                break;
            };
            jobs.push(self.execute_wiki_job(job)?);
        }
        let (telegram_retry, mut warnings) = self.retry_due_telegram_deliveries_for_worker(10)?;
        let (email_retry, email_warnings) = self.retry_due_email_deliveries_for_worker(10)?;
        warnings.extend(email_warnings);
        let radar_delivery_reconcile = self.reconcile_radar_delivery_attempts(3)?;
        let radar_delivery_reconcile = if radar_delivery_reconcile.inspected > 0 {
            Some(radar_delivery_reconcile)
        } else {
            None
        };
        let digest_delivery_reconcile = self.reconcile_digest_delivery_attempts(3)?;
        let digest_delivery_reconcile = if digest_delivery_reconcile.inspected > 0 {
            Some(digest_delivery_reconcile)
        } else {
            None
        };
        let completed = jobs.iter().filter(|job| job.status == "completed").count();
        let failed = jobs.iter().filter(|job| job.status == "failed").count();
        let deferred = jobs.iter().filter(|job| job.status == "deferred").count();
        let dead_lettered = jobs
            .iter()
            .filter(|job| job.status == "dead_lettered")
            .count();
        self.record_worker_heartbeat(default_worker_id().as_str(), jobs.len() as i64, None)?;
        Ok(WorkerRunReport {
            processed: jobs.len(),
            completed,
            failed,
            deferred,
            dead_lettered,
            jobs,
            watch_poll,
            radar_schedule,
            digest_alert_schedule,
            issue_schedule,
            knowledge_cluster_model_writer,
            knowledge_entity_resolution,
            knowledge_cluster_editorial_decision,
            knowledge_cluster_expansion,
            knowledge_cluster_investigation_execution,
            telegram_retry,
            email_retry,
            radar_delivery_reconcile,
            digest_delivery_reconcile,
            warnings,
        })
    }

    pub fn run_rss_fetch_job(&self, url: &str) -> Result<WikiJob> {
        let job = self.insert_wiki_job("rss_fetch", json!({ "url": url }))?;
        self.execute_wiki_job(job)
    }

    pub fn run_github_repo_job(
        &self,
        owner: &str,
        repo: &str,
        mode: &str,
        limit: usize,
    ) -> Result<WikiJob> {
        let job = self.insert_wiki_job(
            "github_repo",
            json!({ "owner": owner, "repo": repo, "mode": mode, "limit": limit.clamp(1, 30) }),
        )?;
        self.execute_wiki_job(job)
    }

    pub fn run_github_owner_job(&self, owner: &str, limit: usize) -> Result<WikiJob> {
        validate_github_segment(owner)?;
        let job = self.insert_wiki_job(
            "github_owner",
            json!({ "owner": owner, "limit": limit.clamp(1, 30) }),
        )?;
        self.execute_wiki_job(job)
    }

    pub fn run_arxiv_search_job(&self, query: &str, limit: usize) -> Result<WikiJob> {
        let job = self.insert_wiki_job(
            "arxiv_search",
            json!({ "query": query, "limit": limit.clamp(1, 30) }),
        )?;
        self.execute_wiki_job(job)
    }

    pub fn run_knowledge_cluster_model_proposal_job(
        &self,
        query: &str,
        model_provider: &str,
        model_name: Option<&str>,
        endpoint: Option<&str>,
        timeout_seconds: Option<u64>,
        max_source_cards: usize,
        max_clusters: usize,
    ) -> Result<WikiJob> {
        let job = self.enqueue_knowledge_cluster_model_proposal_job(
            query,
            model_provider,
            model_name,
            endpoint,
            timeout_seconds,
            max_source_cards,
            max_clusters,
        )?;
        self.execute_wiki_job(job)
    }

    pub fn run_hackernews_fetch_job(&self, feed: &str, limit: usize) -> Result<WikiJob> {
        let feed = normalize_hackernews_feed(feed)?;
        let job = self.insert_wiki_job(
            "hackernews_fetch",
            json!({ "feed": feed, "limit": limit.clamp(1, 30) }),
        )?;
        self.execute_wiki_job(job)
    }

    pub fn run_reddit_fetch_job(&self, locator: &str, limit: usize) -> Result<WikiJob> {
        let locator = normalize_reddit_locator(locator)?;
        let job = self.insert_wiki_job(
            "reddit_fetch",
            json!({ "locator": locator.source_detail(), "limit": limit.clamp(1, 30) }),
        )?;
        self.execute_wiki_job(job)
    }

    pub fn run_x_recent_search_job(&self, query: &str, max_results: usize) -> Result<WikiJob> {
        let job = self.insert_wiki_job(
            "x_recent_search",
            json!({ "query": query, "max_results": max_results.clamp(10, 100) }),
        )?;
        self.execute_wiki_job(job)
    }

    pub fn run_radar_run_job(
        &self,
        profile_id_or_name: &str,
        window_hours: Option<i64>,
        fetch_live: bool,
    ) -> Result<WikiJob> {
        let profile = self
            .read_radar_profile(profile_id_or_name)?
            .with_context(|| format!("radar profile not found: {profile_id_or_name}"))?;
        if let Some(window_hours) = window_hours
            && window_hours <= 0
        {
            bail!("window_hours must be greater than zero");
        }
        let job = self.insert_wiki_job(
            "radar_run",
            json!({
                "profile": profile.id,
                "window_hours": window_hours,
                "fetch_live": fetch_live
            }),
        )?;
        self.execute_wiki_job(job)
    }

    pub fn run_research_convergence_job(
        &self,
        input: ResearchConvergenceStepInput,
    ) -> Result<WikiJob> {
        let job = self.insert_wiki_job("research_convergence_run", serde_json::to_value(input)?)?;
        self.execute_wiki_job(job)
    }

    pub fn list_wiki_jobs(&self) -> Result<Vec<WikiJob>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, kind, status, input_json, result_json, error,
                   attempts, max_attempts, leased_until, worker_id, next_run_at, dead_lettered_at,
                   created_at, updated_at
            FROM wiki_jobs
            ORDER BY updated_at DESC
            "#,
        )?;
        rows(stmt.query_map([], wiki_job_from_row)?)
    }

    pub fn get_wiki_job(&self, id: &str) -> Result<Option<WikiJob>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, kind, status, input_json, result_json, error,
                       attempts, max_attempts, leased_until, worker_id, next_run_at, dead_lettered_at,
                       created_at, updated_at
                FROM wiki_jobs
                WHERE id = ?1
                "#,
                params![id],
                wiki_job_from_row,
            )
            .optional()
            .map_err(Into::into)
    }
}
