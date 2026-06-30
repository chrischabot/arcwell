use super::*;

impl Store {
    pub(crate) fn configured_github_api_token(&self) -> Result<Option<String>> {
        Ok(self
            .get_usable_secret_value("GITHUB_TOKEN")?
            .or_else(|| std::env::var("GITHUB_TOKEN").ok()))
    }

    pub(crate) fn github_provider_credential_deferred_result(
        &self,
        source_key: &str,
        source_kind: &str,
        locator: &str,
    ) -> Result<Option<Value>> {
        let provider_health_key = "provider:github:credential-probe";
        let Some(health) = self.get_source_health(provider_health_key)? else {
            return Ok(None);
        };
        if health.status == "healthy" {
            return Ok(None);
        }
        let deferred_until = now_plus_seconds(3600);
        Ok(Some(json!({
            "status": "deferred",
            "deferred_until": deferred_until,
            "reason": "github credential probe is not healthy; provider network skipped",
            "provider": "github",
            "provider_health_key": provider_health_key,
            "provider_health_status": health.status,
            "provider_health_error": health.last_error.map(|error| excerpt(&error, 500)),
            "source_health_key": source_key,
            "source_kind": source_kind,
            "locator": locator,
        })))
    }

    pub(crate) fn execute_rss_fetch(&self, input: &Value) -> Result<Value> {
        let url_raw = input
            .get("url")
            .and_then(Value::as_str)
            .context("rss_fetch missing url")?;
        let source_key = format!(
            "rss:{}",
            canonical_source_url(url_raw).unwrap_or_else(|_| url_raw.to_string())
        );
        let result = (|| -> Result<Value> {
            let url = validate_fetch_url(url_raw)?;
            self.guard_provider_network_policy(
                "arcwell-llm-wiki",
                "rss",
                "rss_fetch",
                url.as_str(),
                estimated_network_fetch_cost(1),
                json!({ "source_key": source_key }),
            )?;
            let body = fetch_text(url.as_str(), None)?;
            let feed_items = parse_feed_items(&body, 25)?;
            self.write_rss_feed_items(&source_key, url.as_str(), feed_items)
        })();
        if let Err(error) = &result {
            let _ =
                self.record_source_failure(&source_key, "rss", "rss", url_raw, &error.to_string());
        }
        result
    }

    pub(crate) fn write_rss_feed_items(
        &self,
        source_key: &str,
        feed_url: &str,
        feed_items: Vec<FeedItem>,
    ) -> Result<Value> {
        let mut card_ids = BTreeSet::new();
        let mut last_item_id = None;
        let mut last_item_date = None;
        for item in feed_items {
            let item_id = item.id.clone();
            let item_date = item.published.clone();
            let card = self.add_source_card(SourceCardInput {
                title: item.title,
                url: item.url,
                source_type: "rss".to_string(),
                provider: "rss".to_string(),
                summary: item.summary,
                claims: Vec::new(),
                retrieved_at: item.published.or_else(|| Some(now())),
                metadata: json!({
                    "source_kind": "rss",
                    "source_detail": feed_url,
                    "feed_url": feed_url,
                    "id": item_id
                }),
            })?;
            card_ids.insert(card.id);
            last_item_id = Some(item_id);
            if item_date.is_some() {
                last_item_date = item_date;
            }
        }
        let card_ids: Vec<String> = card_ids.into_iter().collect();
        let cursor_key = source_key.to_string();
        let cursor_before = self.get_cursor(&cursor_key)?.map(|cursor| cursor.value);
        let cursor_value = last_item_date
            .clone()
            .or_else(|| last_item_id.clone())
            .unwrap_or_else(now);
        self.set_cursor(&cursor_key, &cursor_value)?;
        self.record_source_success(SourceHealthUpdate {
            key: source_key,
            provider: "rss",
            source_kind: "rss",
            locator: feed_url,
            last_item_id: last_item_id.as_deref(),
            last_item_date: last_item_date.as_deref(),
            cursor_key: Some(&cursor_key),
            cursor_value: Some(&cursor_value),
            next_run_at: Some(&now_plus_seconds(
                self.watch_source_next_run_seconds("rss", feed_url, 3600),
            )),
        })?;
        Ok(json!({
            "source_cards": card_ids,
            "count": card_ids.len(),
            "cursor": cursor_key,
            "cursor_before": cursor_before,
            "cursor_value": cursor_value
        }))
    }

    pub(crate) fn execute_github_repo(&self, input: &Value) -> Result<Value> {
        let owner = input
            .get("owner")
            .and_then(Value::as_str)
            .context("github_repo missing owner")?;
        let repo = input
            .get("repo")
            .and_then(Value::as_str)
            .context("github_repo missing repo")?;
        let mode = input
            .get("mode")
            .and_then(Value::as_str)
            .unwrap_or("releases");
        let limit = input.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize;
        validate_github_segment(owner)?;
        validate_github_segment(repo)?;
        validate_github_mode(mode)?;
        let endpoint = match mode {
            "commits" => format!(
                "https://api.github.com/repos/{owner}/{repo}/commits?per_page={}",
                limit.clamp(1, 30)
            ),
            _ => format!(
                "https://api.github.com/repos/{owner}/{repo}/releases?per_page={}",
                limit.clamp(1, 30)
            ),
        };
        let cursor_key = format!("github:{owner}/{repo}:{mode}");
        let locator = format!("{owner}/{repo}:{mode}");
        let result = (|| -> Result<Value> {
            if let Some(deferred) = self.github_provider_credential_deferred_result(
                &cursor_key,
                "github_repo",
                &locator,
            )? {
                return Ok(deferred);
            }
            self.guard_provider_network_policy(
                "arcwell-llm-wiki",
                "github",
                "github_repo",
                &endpoint,
                estimated_network_fetch_cost(1),
                json!({ "owner": owner, "repo": repo, "mode": mode, "limit": limit.clamp(1, 30) }),
            )?;
            let token = self.configured_github_api_token()?;
            let value = fetch_json(&endpoint, token.as_deref(), "github")?;
            let items = value
                .as_array()
                .context("github response must be an array")?;
            let mut card_ids = BTreeSet::new();
            let mut last_item_id = None;
            let mut last_item_date = None;
            for item in items.iter().take(limit.clamp(1, 30)) {
                let card_input = if mode == "commits" {
                    github_commit_to_source_card(owner, repo, item)?
                } else {
                    github_release_to_source_card(owner, repo, item)?
                };
                last_item_id = github_item_id(item);
                last_item_date = card_input.retrieved_at.clone().or(last_item_date);
                let card = self.add_source_card(card_input)?;
                card_ids.insert(card.id);
            }
            let cursor_before = self.get_cursor(&cursor_key)?.map(|cursor| cursor.value);
            let cursor_value = last_item_date
                .clone()
                .or_else(|| last_item_id.clone())
                .unwrap_or_else(now);
            self.set_cursor(&cursor_key, &cursor_value)?;
            self.record_source_success(SourceHealthUpdate {
                key: &cursor_key,
                provider: "github",
                source_kind: "github_repo",
                locator: &locator,
                last_item_id: last_item_id.as_deref(),
                last_item_date: last_item_date.as_deref(),
                cursor_key: Some(&cursor_key),
                cursor_value: Some(&cursor_value),
                next_run_at: Some(&now_plus_seconds(self.watch_source_next_run_seconds(
                    "github_repo",
                    &locator,
                    3600,
                ))),
            })?;
            let card_ids: Vec<String> = card_ids.into_iter().collect();
            Ok(
                json!({ "source_cards": card_ids, "count": card_ids.len(), "cursor": cursor_key, "cursor_before": cursor_before, "cursor_value": cursor_value }),
            )
        })();
        if let Err(error) = &result {
            let _ = self.record_source_failure(
                &cursor_key,
                "github",
                "github_repo",
                &locator,
                &error.to_string(),
            );
        }
        result
    }

    pub(crate) fn execute_github_owner(&self, input: &Value) -> Result<Value> {
        let owner = input
            .get("owner")
            .and_then(Value::as_str)
            .context("github_owner missing owner")?;
        let limit = input.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize;
        validate_github_segment(owner)?;
        let endpoint = format!(
            "https://api.github.com/users/{owner}/repos?sort=updated&direction=desc&per_page={}",
            limit.clamp(1, 30)
        );
        let cursor_key = format!("github-owner:{owner}");
        let result = (|| -> Result<Value> {
            if let Some(deferred) =
                self.github_provider_credential_deferred_result(&cursor_key, "github_owner", owner)?
            {
                return Ok(deferred);
            }
            self.guard_provider_network_policy(
                "arcwell-llm-wiki",
                "github",
                "github_owner",
                &endpoint,
                estimated_network_fetch_cost(1),
                json!({ "owner": owner, "limit": limit.clamp(1, 30) }),
            )?;
            let token = self.configured_github_api_token()?;
            let value = fetch_json(&endpoint, token.as_deref(), "github")?;
            let repos = value
                .as_array()
                .context("github owner response must be an array")?;
            let mut card_ids = BTreeSet::new();
            let mut last_item_id = None;
            let mut last_item_date = None;
            for item in repos.iter().take(limit.clamp(1, 30)) {
                let card_input = github_repo_summary_to_source_card(owner, item)?;
                last_item_id = item.get("id").map(|id| id.to_string()).or(last_item_id);
                last_item_date = card_input.retrieved_at.clone().or(last_item_date);
                let card = self.add_source_card(card_input)?;
                card_ids.insert(card.id);
            }
            let cursor_before = self.get_cursor(&cursor_key)?.map(|cursor| cursor.value);
            let cursor_value = last_item_date
                .clone()
                .or_else(|| last_item_id.clone())
                .unwrap_or_else(now);
            self.set_cursor(&cursor_key, &cursor_value)?;
            self.record_source_success(SourceHealthUpdate {
                key: &cursor_key,
                provider: "github",
                source_kind: "github_owner",
                locator: owner,
                last_item_id: last_item_id.as_deref(),
                last_item_date: last_item_date.as_deref(),
                cursor_key: Some(&cursor_key),
                cursor_value: Some(&cursor_value),
                next_run_at: Some(&now_plus_seconds(self.watch_source_next_run_seconds(
                    "github_owner",
                    owner,
                    3600,
                ))),
            })?;
            let card_ids: Vec<String> = card_ids.into_iter().collect();
            Ok(
                json!({ "source_cards": card_ids, "count": card_ids.len(), "cursor": cursor_key, "cursor_before": cursor_before, "cursor_value": cursor_value }),
            )
        })();
        if let Err(error) = &result {
            let _ = self.record_source_failure(
                &cursor_key,
                "github",
                "github_owner",
                owner,
                &error.to_string(),
            );
        }
        result
    }

    pub(crate) fn execute_arxiv_search(&self, input: &Value) -> Result<Value> {
        let query = input
            .get("query")
            .and_then(Value::as_str)
            .context("arxiv_search missing query")?;
        let limit = input.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize;
        validate_query(query)?;
        let mut url = Url::parse("https://export.arxiv.org/api/query")?;
        url.query_pairs_mut()
            .append_pair("search_query", query)
            .append_pair("start", "0")
            .append_pair("max_results", &limit.clamp(1, 30).to_string())
            .append_pair("sortBy", "submittedDate")
            .append_pair("sortOrder", "descending");
        let cursor_key = format!("arxiv:{query}");
        let result = (|| -> Result<Value> {
            self.guard_provider_network_policy(
                "arcwell-llm-wiki",
                "arxiv",
                "arxiv_search",
                url.as_str(),
                estimated_network_fetch_cost(1),
                json!({ "query": query, "limit": limit.clamp(1, 30) }),
            )?;
            let body = fetch_text(url.as_str(), None)?;
            let items = parse_arxiv_entries(&body, limit.clamp(1, 30))?;
            let mut card_ids = BTreeSet::new();
            let mut last_item_id = None;
            let mut last_item_date = None;
            for item in items {
                let item_id = item.id.clone();
                let item_date = item.published.clone();
                let card = self.add_source_card(SourceCardInput {
                    title: item.title,
                    url: item.url,
                    source_type: "arxiv".to_string(),
                    provider: "arxiv".to_string(),
                    summary: item.summary,
                    claims: Vec::new(),
                    retrieved_at: item.published.or_else(|| Some(now())),
                    metadata: json!({
                        "source_kind": "arxiv",
                        "source_detail": query,
                        "id": item_id,
                        "authors": item.authors
                    }),
                })?;
                card_ids.insert(card.id);
                last_item_id = Some(item_id);
                if item_date.is_some() {
                    last_item_date = item_date;
                }
            }
            let cursor_before = self.get_cursor(&cursor_key)?.map(|cursor| cursor.value);
            let cursor_value = last_item_date
                .clone()
                .or_else(|| last_item_id.clone())
                .unwrap_or_else(now);
            self.set_cursor(&cursor_key, &cursor_value)?;
            self.record_source_success(SourceHealthUpdate {
                key: &cursor_key,
                provider: "arxiv",
                source_kind: "arxiv_query",
                locator: query,
                last_item_id: last_item_id.as_deref(),
                last_item_date: last_item_date.as_deref(),
                cursor_key: Some(&cursor_key),
                cursor_value: Some(&cursor_value),
                next_run_at: Some(&now_plus_seconds(self.watch_source_next_run_seconds(
                    "arxiv_query",
                    query,
                    3600,
                ))),
            })?;
            let card_ids: Vec<String> = card_ids.into_iter().collect();
            Ok(
                json!({ "source_cards": card_ids, "count": card_ids.len(), "cursor": cursor_key, "cursor_before": cursor_before, "cursor_value": cursor_value }),
            )
        })();
        if let Err(error) = &result {
            let _ = self.record_source_failure(
                &cursor_key,
                "arxiv",
                "arxiv_query",
                query,
                &error.to_string(),
            );
        }
        result
    }

    pub(crate) fn execute_hackernews_fetch(&self, input: &Value) -> Result<Value> {
        self.execute_hackernews_fetch_with_base(input, "https://hacker-news.firebaseio.com/v0")
    }

    pub(crate) fn execute_hackernews_fetch_with_base(
        &self,
        input: &Value,
        base: &str,
    ) -> Result<Value> {
        let feed_raw = input
            .get("feed")
            .or_else(|| input.get("locator"))
            .and_then(Value::as_str)
            .unwrap_or("topstories");
        let feed = normalize_hackernews_feed(feed_raw)?;
        let limit = input.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize;
        let limit = limit.clamp(1, 30);
        let source_key = format!("hackernews:{feed}");
        let feed_url = hackernews_api_url(base, &format!("{feed}.json"))?;
        let result = (|| -> Result<Value> {
            self.guard_provider_network_policy(
                "arcwell-llm-wiki",
                "hackernews",
                "hackernews_fetch",
                feed_url.as_str(),
                estimated_network_fetch_cost(1 + (limit * 4)),
                json!({ "feed": feed, "limit": limit, "source_key": source_key }),
            )?;
            let ids_value = fetch_json(feed_url.as_str(), None, "hackernews")?;
            let ids = ids_value
                .as_array()
                .context("hackernews feed response must be an array")?;
            let mut card_ids = BTreeSet::new();
            let mut last_item_id = None;
            let mut last_item_date = None;
            let mut skipped_items = Vec::new();
            for id_value in ids.iter().take(limit) {
                let Some(item_id) = id_value.as_u64() else {
                    skipped_items.push(json!({ "reason": "non_integer_id", "value": id_value }));
                    continue;
                };
                let item_url = hackernews_api_url(base, &format!("item/{item_id}.json"))?;
                let item = match fetch_json(item_url.as_str(), None, "hackernews") {
                    Ok(item) => item,
                    Err(error) => {
                        skipped_items.push(json!({
                            "id": item_id,
                            "reason": "item_fetch_failed",
                            "error": excerpt(&error.to_string(), 240)
                        }));
                        continue;
                    }
                };
                let comments = self.fetch_hackernews_top_comments(base, &item)?;
                match hackernews_item_to_source_card(&feed, &item, &comments)? {
                    Some(card_input) => {
                        last_item_id = Some(item_id.to_string());
                        last_item_date = card_input.retrieved_at.clone().or(last_item_date);
                        let card = self.add_source_card(card_input)?;
                        card_ids.insert(card.id);
                    }
                    None => skipped_items.push(json!({
                        "id": item_id,
                        "reason": "not_usable_story"
                    })),
                }
            }
            if card_ids.is_empty() && !ids.is_empty() {
                bail!(
                    "hackernews fetch produced no usable stories; skipped={}",
                    skipped_items.len()
                );
            }
            let cursor_before = self.get_cursor(&source_key)?.map(|cursor| cursor.value);
            let cursor_value = last_item_date
                .clone()
                .or_else(|| last_item_id.clone())
                .unwrap_or_else(now);
            self.set_cursor(&source_key, &cursor_value)?;
            self.record_source_success(SourceHealthUpdate {
                key: &source_key,
                provider: "hackernews",
                source_kind: "hackernews",
                locator: &feed,
                last_item_id: last_item_id.as_deref(),
                last_item_date: last_item_date.as_deref(),
                cursor_key: Some(&source_key),
                cursor_value: Some(&cursor_value),
                next_run_at: Some(&now_plus_seconds(self.watch_source_next_run_seconds(
                    "hackernews",
                    &feed,
                    1800,
                ))),
            })?;
            let card_ids: Vec<String> = card_ids.into_iter().collect();
            Ok(json!({
                "source_cards": card_ids,
                "count": card_ids.len(),
                "cursor": source_key,
                "cursor_before": cursor_before,
                "cursor_value": cursor_value,
                "skipped_items": skipped_items
            }))
        })();
        if let Err(error) = &result {
            let _ = self.record_source_failure(
                &source_key,
                "hackernews",
                "hackernews",
                &feed,
                &error.to_string(),
            );
        }
        result
    }

    pub(crate) fn fetch_hackernews_top_comments(
        &self,
        base: &str,
        story: &Value,
    ) -> Result<Vec<HackerNewsCommentExcerpt>> {
        let mut comments = Vec::new();
        let Some(kids) = story.get("kids").and_then(Value::as_array) else {
            return Ok(comments);
        };
        for kid in kids.iter().filter_map(Value::as_u64).take(3) {
            let url = hackernews_api_url(base, &format!("item/{kid}.json"))?;
            let Ok(comment) = fetch_json(url.as_str(), None, "hackernews") else {
                continue;
            };
            if comment.get("deleted").and_then(Value::as_bool) == Some(true)
                || comment.get("dead").and_then(Value::as_bool) == Some(true)
                || comment.get("type").and_then(Value::as_str) != Some("comment")
            {
                continue;
            }
            let text = comment
                .get("text")
                .and_then(Value::as_str)
                .map(html_fragment_to_text)
                .map(|text| excerpt(&text, 500))
                .unwrap_or_default();
            if text.trim().is_empty() {
                continue;
            }
            comments.push(HackerNewsCommentExcerpt {
                id: kid,
                by: comment
                    .get("by")
                    .and_then(Value::as_str)
                    .map(excerpt_hn_user),
                text,
            });
        }
        Ok(comments)
    }

    pub(crate) fn execute_reddit_fetch(&self, input: &Value) -> Result<Value> {
        self.execute_reddit_fetch_with_base(input, "https://www.reddit.com")
    }

    pub fn ingest_reddit_browser_listing(
        &self,
        locator_raw: &str,
        listing: &Value,
        limit: usize,
    ) -> Result<Value> {
        let locator = normalize_reddit_locator(locator_raw)?;
        let limit = limit.clamp(1, 30);
        let source_detail = locator.source_detail();
        let source_key = format!("reddit:{source_detail}");
        self.write_reddit_json_listing(
            "https://www.reddit.com",
            &source_key,
            &locator,
            listing,
            limit,
            None,
            "host_browser_json",
            false,
            None,
        )
    }

    pub(crate) fn execute_reddit_fetch_with_base(
        &self,
        input: &Value,
        base: &str,
    ) -> Result<Value> {
        let locator_raw = input
            .get("locator")
            .or_else(|| input.get("subreddit"))
            .and_then(Value::as_str)
            .context("reddit_fetch missing locator")?;
        let locator = normalize_reddit_locator(locator_raw)?;
        let limit = input.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize;
        let limit = limit.clamp(1, 30);
        let source_detail = locator.source_detail();
        let source_key = format!("reddit:{source_detail}");
        let listing_url = reddit_listing_url(base, &locator, limit)?;
        let result = (|| -> Result<Value> {
            self.guard_provider_network_policy(
                "arcwell-llm-wiki",
                "reddit",
                "reddit_fetch",
                listing_url.as_str(),
                estimated_network_fetch_cost(1 + (limit * 2)),
                json!({ "locator": source_detail, "limit": limit, "source_key": source_key }),
            )?;
            let reddit_bearer_token = std::env::var("REDDIT_BEARER_TOKEN")
                .ok()
                .map(|token| token.trim().to_string())
                .filter(|token| !token.is_empty());
            let use_json = input.get("transport").and_then(Value::as_str) == Some("json")
                || reddit_bearer_token.is_some();
            if use_json {
                match fetch_json(
                    listing_url.as_str(),
                    reddit_bearer_token.as_deref(),
                    "reddit",
                ) {
                    Ok(listing) => self.write_reddit_json_listing(
                        base,
                        &source_key,
                        &locator,
                        &listing,
                        limit,
                        None,
                        "json",
                        true,
                        reddit_bearer_token.as_deref(),
                    ),
                    Err(json_error) => self.fetch_and_write_reddit_rss_fallback(
                        base,
                        &source_key,
                        &locator,
                        limit,
                        &json_error.to_string(),
                    ),
                }
            } else {
                self.fetch_and_write_reddit_rss_fallback(
                    base,
                    &source_key,
                    &locator,
                    limit,
                    "unauthenticated Reddit JSON skipped; Reddit Data API guidance requires OAuth",
                )
            }
        })();
        if let Err(error) = &result {
            let _ = self.record_source_failure(
                &source_key,
                "reddit",
                "reddit",
                &source_detail,
                &error.to_string(),
            );
        }
        result
    }

    pub(crate) fn fetch_and_write_reddit_rss_fallback(
        &self,
        base: &str,
        source_key: &str,
        locator: &RedditLocator,
        limit: usize,
        json_error: &str,
    ) -> Result<Value> {
        let rss_url = reddit_rss_url(base, locator, limit)?;
        let user_agent = provider_user_agent("reddit");
        let body =
            fetch_text_with_user_agent(rss_url.as_str(), None, &user_agent).map_err(|error| {
                anyhow::anyhow!(
                    "reddit RSS fallback failed after JSON path was unavailable: {}; rss_error={}",
                    excerpt(json_error, 240),
                    excerpt(&error.to_string(), 500)
                )
            })?;
        let feed_items = parse_feed_items(&body, limit)?;
        self.write_reddit_rss_fallback_items(source_key, locator, feed_items, json_error)
    }

    pub(crate) fn write_reddit_json_listing(
        &self,
        base: &str,
        source_key: &str,
        locator: &RedditLocator,
        listing: &Value,
        limit: usize,
        fallback_error: Option<&str>,
        transport: &str,
        fetch_comments: bool,
        bearer_token: Option<&str>,
    ) -> Result<Value> {
        let children = listing
            .get("data")
            .and_then(|data| data.get("children"))
            .and_then(Value::as_array)
            .context("reddit listing response must contain data.children")?;
        let mut card_ids = BTreeSet::new();
        let mut last_item_id = None;
        let mut last_item_date = None;
        let mut skipped_items = Vec::new();
        for child in children.iter().take(limit) {
            let Some(data) = child.get("data") else {
                skipped_items.push(json!({ "reason": "missing_data" }));
                continue;
            };
            let post_id = data
                .get("id")
                .and_then(Value::as_str)
                .map(|value| value.to_string());
            let comments = match (fetch_comments, post_id.as_deref()) {
                (true, Some(post_id)) => self
                    .fetch_reddit_top_comments(base, &locator.subreddit, post_id, bearer_token)
                    .unwrap_or_default(),
                _ => Vec::new(),
            };
            let comment_capture = if fetch_comments {
                "best_effort_json_comments"
            } else {
                "not_captured_browser_listing"
            };
            match reddit_post_to_source_card(
                locator,
                data,
                &comments,
                fallback_error,
                transport,
                comment_capture,
            )? {
                Some(card_input) => {
                    last_item_id = post_id.or(last_item_id);
                    last_item_date = card_input.retrieved_at.clone().or(last_item_date);
                    let card = self.add_source_card(card_input)?;
                    card_ids.insert(card.id);
                }
                None => skipped_items.push(json!({
                    "id": post_id,
                    "reason": "not_usable_post"
                })),
            }
        }
        if card_ids.is_empty() {
            bail!(
                "reddit fetch produced no usable posts; children={} skipped={}",
                children.len(),
                skipped_items.len()
            );
        }
        self.finish_reddit_fetch(
            source_key,
            locator,
            card_ids,
            last_item_id,
            last_item_date,
            transport,
            skipped_items,
        )
    }

    pub(crate) fn write_reddit_rss_fallback_items(
        &self,
        source_key: &str,
        locator: &RedditLocator,
        feed_items: Vec<FeedItem>,
        json_error: &str,
    ) -> Result<Value> {
        let mut card_ids = BTreeSet::new();
        let mut last_item_id = None;
        let mut last_item_date = None;
        for item in feed_items {
            let item_id = item.id.clone();
            let item_date = item.published.clone();
            let card = self.add_source_card(SourceCardInput {
                title: format!("Reddit: {}", item.title),
                url: item.url.clone(),
                source_type: "reddit_post".to_string(),
                provider: "reddit".to_string(),
                summary: excerpt(&item.summary, 2_000),
                claims: vec![SourceClaim {
                    claim: format!(
                        "Reddit item {} appeared in {} via RSS fallback.",
                        item_id,
                        locator.source_detail()
                    ),
                    kind: "fact".to_string(),
                    confidence: 0.75,
                }],
                retrieved_at: item.published.or_else(|| Some(now())),
                metadata: json!({
                    "source_kind": "reddit",
                    "source_detail": locator.source_detail(),
                    "subreddit": locator.subreddit,
                    "sort": locator.sort,
                    "id": item_id,
                    "transport": "rss_fallback",
                    "json_error": excerpt(json_error, 500),
                    "top_comments": [],
                    "top_comment_count": 0,
                    "comment_capture": "unavailable_rss_fallback"
                }),
            })?;
            card_ids.insert(card.id);
            last_item_id = Some(item_id);
            if item_date.is_some() {
                last_item_date = item_date;
            }
        }
        self.finish_reddit_fetch(
            source_key,
            locator,
            card_ids,
            last_item_id,
            last_item_date,
            "rss_fallback",
            Vec::new(),
        )
    }

    pub(crate) fn finish_reddit_fetch(
        &self,
        source_key: &str,
        locator: &RedditLocator,
        card_ids: BTreeSet<String>,
        last_item_id: Option<String>,
        last_item_date: Option<String>,
        transport: &str,
        skipped_items: Vec<Value>,
    ) -> Result<Value> {
        let cursor_before = self.get_cursor(source_key)?.map(|cursor| cursor.value);
        let cursor_value = last_item_date
            .clone()
            .or_else(|| last_item_id.clone())
            .unwrap_or_else(now);
        self.set_cursor(source_key, &cursor_value)?;
        self.record_source_success(SourceHealthUpdate {
            key: source_key,
            provider: "reddit",
            source_kind: "reddit",
            locator: &locator.source_detail(),
            last_item_id: last_item_id.as_deref(),
            last_item_date: last_item_date.as_deref(),
            cursor_key: Some(source_key),
            cursor_value: Some(&cursor_value),
            next_run_at: Some(&now_plus_seconds(self.watch_source_next_run_seconds(
                "reddit",
                &locator.source_detail(),
                1800,
            ))),
        })?;
        let card_ids: Vec<String> = card_ids.into_iter().collect();
        Ok(json!({
            "source_cards": card_ids,
            "count": card_ids.len(),
            "cursor": source_key,
            "cursor_before": cursor_before,
            "cursor_value": cursor_value,
            "transport": transport,
            "skipped_items": skipped_items
        }))
    }

    pub(crate) fn fetch_reddit_top_comments(
        &self,
        base: &str,
        subreddit: &str,
        post_id: &str,
        bearer_token: Option<&str>,
    ) -> Result<Vec<RedditCommentExcerpt>> {
        let url = reddit_comments_url(base, subreddit, post_id)?;
        let value = fetch_json(url.as_str(), bearer_token, "reddit")?;
        let comments = value
            .as_array()
            .and_then(|items| items.get(1))
            .and_then(|listing| listing.get("data"))
            .and_then(|data| data.get("children"))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let mut out = Vec::new();
        for child in comments.into_iter().take(8) {
            if child.get("kind").and_then(Value::as_str) == Some("more") {
                continue;
            }
            let Some(data) = child.get("data") else {
                continue;
            };
            let text = data
                .get("body_html")
                .or_else(|| data.get("body"))
                .and_then(Value::as_str)
                .map(html_fragment_to_text)
                .map(|text| excerpt(&text, 500))
                .unwrap_or_default();
            if text.trim().is_empty()
                || data.get("removed").and_then(Value::as_bool) == Some(true)
                || data.get("distinguished").and_then(Value::as_str) == Some("moderator")
                    && text == "[removed]"
            {
                continue;
            }
            out.push(RedditCommentExcerpt {
                id: data
                    .get("id")
                    .and_then(Value::as_str)
                    .map(|value| excerpt(value, 80))
                    .unwrap_or_else(|| "unknown".to_string()),
                by: data
                    .get("author")
                    .and_then(Value::as_str)
                    .map(excerpt_reddit_author),
                score: data.get("score").and_then(Value::as_i64),
                text,
            });
            if out.len() >= 3 {
                break;
            }
        }
        Ok(out)
    }
}
