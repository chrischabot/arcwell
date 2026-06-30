use super::*;

impl Store {
    pub(crate) fn x_import_following_watch_sources_with_base(
        &self,
        max_users: usize,
        endpoint: &str,
    ) -> Result<XFollowingWatchImportReport> {
        self.require_cost_budget(
            "arcwell-x",
            "x_following_watch",
            "x",
            "following",
            Some("x_following_watch"),
            estimated_x_following_cost(max_users),
            "X following watch import",
        )?;
        let token = self.x_bearer_token_for_endpoint(endpoint)?;
        let base = validated_x_api_base(endpoint)?;
        let me_url = base.join("/2/users/me?user.fields=username,name")?;
        let me = fetch_x_json(me_url.as_str(), Some(&token))?;
        let user_id = me
            .pointer("/data/id")
            .and_then(Value::as_str)
            .context("X /2/users/me response missing data.id")?;
        validate_key(user_id)?;

        let max_users = max_users.clamp(1, 5_000);
        let mut seen = 0;
        let mut added = 0;
        let mut updated = 0;
        let mut unchanged = 0;
        let mut rejected = 0;
        let mut pagination_token: Option<String> = None;

        while seen < max_users {
            let page_size = (max_users - seen).clamp(1, 1_000);
            let mut url = base.join(&format!("/2/users/{user_id}/following"))?;
            {
                let mut pairs = url.query_pairs_mut();
                pairs
                    .append_pair("max_results", &page_size.to_string())
                    .append_pair(
                        "user.fields",
                        "username,name,description,verified,verified_type",
                    );
                if let Some(token) = &pagination_token {
                    pairs.append_pair("pagination_token", token);
                }
            }
            let value = fetch_x_json(url.as_str(), Some(&token))?;
            let users = value
                .get("data")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            if users.is_empty() {
                pagination_token = value
                    .pointer("/meta/next_token")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned);
                break;
            }
            for user in users {
                if seen >= max_users {
                    break;
                }
                seen += 1;
                match x_following_user_to_watch_source(&user) {
                    Ok(input) => match self.upsert_watch_source_with_status(input) {
                        Ok((_source, status)) => match status {
                            WatchSourceUpsertStatus::Added => added += 1,
                            WatchSourceUpsertStatus::Updated => updated += 1,
                            WatchSourceUpsertStatus::Unchanged => unchanged += 1,
                        },
                        Err(_) => rejected += 1,
                    },
                    Err(_) => rejected += 1,
                }
            }
            pagination_token = value
                .pointer("/meta/next_token")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            if pagination_token.is_none() {
                break;
            }
        }

        Ok(XFollowingWatchImportReport {
            seen,
            imported: added + updated + unchanged,
            added,
            updated,
            unchanged,
            rejected,
            next_token: pagination_token,
        })
    }

    pub fn x_rebuild_definitive_watch_sources(
        &self,
        bookmark_days: i64,
        max_bookmarks: usize,
        max_recent_follows: usize,
    ) -> Result<XDefinitiveWatchReport> {
        let endpoint =
            std::env::var("ARCWELL_X_API_BASE").unwrap_or_else(|_| "https://api.x.com".to_string());
        self.x_rebuild_definitive_watch_sources_with_base(
            bookmark_days,
            max_bookmarks,
            max_recent_follows,
            &endpoint,
        )
    }

    pub(crate) fn x_rebuild_definitive_watch_sources_with_base(
        &self,
        bookmark_days: i64,
        max_bookmarks: usize,
        max_recent_follows: usize,
        endpoint: &str,
    ) -> Result<XDefinitiveWatchReport> {
        self.require_cost_budget(
            "arcwell-x",
            "x_definitive_watch",
            "x",
            "bookmarks_following",
            Some("x_definitive_watch"),
            estimated_x_definitive_watch_cost(max_bookmarks, max_recent_follows),
            "X definitive watch rebuild",
        )?;
        let token = self.x_bearer_token_for_endpoint(endpoint)?;
        let base = validated_x_api_base(endpoint)?;
        let user_id = self.x_user_id(&base, &token)?;
        let bookmark_days = bookmark_days.clamp(1, 36_500);
        let max_bookmarks = max_bookmarks.clamp(10, 100_000);
        let max_recent_follows = max_recent_follows.clamp(0, 100);
        let cutoff = Utc::now() - chrono::Duration::days(bookmark_days);
        let bookmark_since = cutoff.to_rfc3339();

        let mut bookmark_tweets_seen = 0;
        let mut bookmark_tweets_within_window = 0;
        let mut recent_follows_seen = 0;
        let mut rejected = 0;
        let mut bookmark_handles = BTreeSet::new();
        let mut follow_handles = BTreeSet::new();
        let mut inputs: BTreeMap<String, WatchSourceInput> = BTreeMap::new();

        let mut pagination_token: Option<String> = None;
        while bookmark_tweets_seen < max_bookmarks {
            let page_size = (max_bookmarks - bookmark_tweets_seen).clamp(10, 100);
            let mut url = base.join(&format!("/2/users/{user_id}/bookmarks"))?;
            {
                let mut pairs = url.query_pairs_mut();
                pairs
                    .append_pair("max_results", &page_size.to_string())
                    .append_pair("tweet.fields", "created_at,author_id,public_metrics")
                    .append_pair("expansions", "author_id")
                    .append_pair(
                        "user.fields",
                        "username,name,description,verified,verified_type",
                    );
                if let Some(token) = &pagination_token {
                    pairs.append_pair("pagination_token", token);
                }
            }
            let value = fetch_x_json(url.as_str(), Some(&token))?;
            x_fail_on_response_errors(&value)?;
            let tweets = value
                .get("data")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            if tweets.is_empty() {
                break;
            }
            let users = x_users_by_id(&value);
            for tweet in tweets {
                if bookmark_tweets_seen >= max_bookmarks {
                    break;
                }
                bookmark_tweets_seen += 1;
                match x_bookmark_tweet_author_watch_source(&tweet, &users, cutoff) {
                    Ok(Some(input)) => {
                        bookmark_tweets_within_window += 1;
                        bookmark_handles.insert(input.locator.clone());
                        merge_x_watch_source(&mut inputs, input, "bookmark");
                    }
                    Ok(None) => {}
                    Err(_) => rejected += 1,
                }
            }
            pagination_token = value
                .pointer("/meta/next_token")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            if pagination_token.is_none() {
                break;
            }
        }

        if max_recent_follows > 0 {
            let mut url = base.join(&format!("/2/users/{user_id}/following"))?;
            {
                let mut pairs = url.query_pairs_mut();
                pairs
                    .append_pair("max_results", &max_recent_follows.to_string())
                    .append_pair(
                        "user.fields",
                        "username,name,description,verified,verified_type",
                    );
            }
            let value = fetch_x_json(url.as_str(), Some(&token))?;
            x_fail_on_response_errors(&value)?;
            let users = value
                .get("data")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            for user in users.into_iter().take(max_recent_follows) {
                recent_follows_seen += 1;
                match x_user_to_watch_source(&user, "x-api/following-recent", "recent_follow") {
                    Ok(input) => {
                        follow_handles.insert(input.locator.clone());
                        merge_x_watch_source(&mut inputs, input, "recent_follow");
                    }
                    Err(_) => rejected += 1,
                }
            }
        }

        let final_handles = inputs.len();
        self.conn.execute("BEGIN IMMEDIATE", [])?;
        let replace_result = (|| -> Result<usize> {
            let removed_previous = self.delete_watch_sources_by_kind("x_handle")?;
            for input in inputs.into_values() {
                self.upsert_watch_source(input)?;
            }
            Ok(removed_previous)
        })();
        let removed_previous = match replace_result {
            Ok(removed_previous) => {
                self.conn.execute("COMMIT", [])?;
                removed_previous
            }
            Err(error) => {
                let _ = self.conn.execute("ROLLBACK", []);
                return Err(error);
            }
        };

        Ok(XDefinitiveWatchReport {
            removed_previous,
            bookmark_tweets_seen,
            bookmark_tweets_within_window,
            bookmark_authors: bookmark_handles.len(),
            recent_follows_seen,
            recent_follow_authors: follow_handles.len(),
            final_handles,
            rejected,
            bookmark_since,
        })
    }

    pub fn x_monitor_watch_sources(
        &self,
        max_sources: usize,
        max_results_per_source: usize,
    ) -> Result<XMonitorReport> {
        let endpoint =
            std::env::var("ARCWELL_X_API_BASE").unwrap_or_else(|_| "https://api.x.com".to_string());
        self.x_monitor_watch_sources_with_base(max_sources, max_results_per_source, &endpoint)
    }

    pub fn x_monitor_watch_source(
        &self,
        handle: &str,
        max_results_per_source: usize,
    ) -> Result<XMonitorReport> {
        let endpoint =
            std::env::var("ARCWELL_X_API_BASE").unwrap_or_else(|_| "https://api.x.com".to_string());
        self.x_monitor_watch_source_with_base(handle, max_results_per_source, &endpoint)
    }

    pub(crate) fn x_monitor_watch_sources_with_base(
        &self,
        max_sources: usize,
        max_results_per_source: usize,
        endpoint: &str,
    ) -> Result<XMonitorReport> {
        let max_sources = max_sources.clamp(1, X_MONITOR_MAX_SOURCES);
        let watch_sources: Vec<WatchSource> = self
            .list_watch_sources()?
            .into_iter()
            .filter(|source| source.source_kind == "x_handle" && source.status == "active")
            .take(max_sources)
            .collect();
        self.x_monitor_selected_watch_sources_with_base(
            watch_sources,
            max_results_per_source,
            endpoint,
        )
    }

    pub(crate) fn x_monitor_watch_source_with_base(
        &self,
        handle: &str,
        max_results_per_source: usize,
        endpoint: &str,
    ) -> Result<XMonitorReport> {
        let handle = handle.trim().trim_start_matches('@');
        validate_x_handle(handle)?;
        let source_id = watch_source_id("x_handle", handle);
        let Some(source) = self.read_watch_source(&source_id)? else {
            return Ok(XMonitorReport {
                watched_sources: 0,
                polled_sources: 0,
                attempted_sources: 0,
                deferred_sources: 0,
                imported: 0,
                skipped_duplicates: 0,
                rejected: 0,
                failed_sources: 0,
                rate_limited_sources: 0,
                digest_candidates: 0,
                stopped_reason: None,
                sources: Vec::new(),
            });
        };
        if source.status != "active" {
            return Ok(XMonitorReport {
                watched_sources: 0,
                polled_sources: 0,
                attempted_sources: 0,
                deferred_sources: 0,
                imported: 0,
                skipped_duplicates: 0,
                rejected: 0,
                failed_sources: 0,
                rate_limited_sources: 0,
                digest_candidates: 0,
                stopped_reason: None,
                sources: Vec::new(),
            });
        }
        self.x_monitor_selected_watch_sources_with_base(
            vec![source],
            max_results_per_source,
            endpoint,
        )
    }

    pub(crate) fn x_monitor_selected_watch_sources_with_base(
        &self,
        watch_sources: Vec<WatchSource>,
        max_results_per_source: usize,
        endpoint: &str,
    ) -> Result<XMonitorReport> {
        if watch_sources.is_empty() {
            return Ok(XMonitorReport {
                watched_sources: 0,
                polled_sources: 0,
                attempted_sources: 0,
                deferred_sources: 0,
                imported: 0,
                skipped_duplicates: 0,
                rejected: 0,
                failed_sources: 0,
                rate_limited_sources: 0,
                digest_candidates: 0,
                stopped_reason: None,
                sources: Vec::new(),
            });
        }
        let max_sources = watch_sources.len().clamp(1, X_MONITOR_MAX_SOURCES);
        let max_results_per_source = max_results_per_source.clamp(10, 100);
        let projected = estimated_x_monitor_cost(max_sources, max_results_per_source);
        self.policy_guard(PolicyRequest {
            action: "provider.network".to_string(),
            package: Some("arcwell-x".to_string()),
            provider: Some("x".to_string()),
            source: Some("x_monitor".to_string()),
            channel: None,
            subject: None,
            target: Some(endpoint.to_string()),
            projected_usd: Some(projected),
            metadata: json!({
                "max_sources": max_sources,
                "max_results_per_source": max_results_per_source
            }),
            untrusted_excerpt: None,
        })?;
        self.require_cost_budget(
            "arcwell-x",
            "x_monitor",
            "x",
            "watch_recent_search",
            Some("x_monitor"),
            projected,
            "X production monitor",
        )?;

        let monitor_started_at = now();
        let token = match self.x_bearer_token_for_endpoint(endpoint) {
            Ok(token) => token,
            Err(error) => {
                let completed_at = now();
                let error_text = error.to_string();
                let _ = self.release_cost_reservation(
                    "arcwell-x",
                    "x_monitor",
                    "x",
                    "watch_recent_search",
                    Some("x_monitor"),
                );
                let _ = self.record_source_failure(
                    "x:monitor",
                    "x",
                    "x_monitor",
                    "watch_sources",
                    &error_text,
                );
                let _ = self.record_x_sync_run(XSyncRunInsert {
                    account_id: None,
                    stream: "watch_monitor",
                    transport: "x_api",
                    status: "failed",
                    started_at: &monitor_started_at,
                    completed_at: &completed_at,
                    seen: 0,
                    inserted: 0,
                    updated: 0,
                    skipped_duplicates: 0,
                    rejected: 0,
                    cursor_key: None,
                    previous_cursor: None,
                    new_cursor: None,
                    error: Some(&error_text),
                    metadata: json!({
                        "stage": "token",
                        "max_sources": max_sources,
                        "max_results_per_source": max_results_per_source
                    }),
                });
                return Err(error);
            }
        };
        let base = match validated_x_api_base(endpoint) {
            Ok(base) => base,
            Err(error) => {
                let completed_at = now();
                let error_text = error.to_string();
                let _ = self.record_x_sync_run(XSyncRunInsert {
                    account_id: None,
                    stream: "watch_monitor",
                    transport: "x_api",
                    status: "failed",
                    started_at: &monitor_started_at,
                    completed_at: &completed_at,
                    seen: 0,
                    inserted: 0,
                    updated: 0,
                    skipped_duplicates: 0,
                    rejected: 0,
                    cursor_key: None,
                    previous_cursor: None,
                    new_cursor: None,
                    error: Some(&error_text),
                    metadata: json!({
                        "stage": "base_url",
                        "endpoint": endpoint,
                        "max_sources": max_sources,
                        "max_results_per_source": max_results_per_source
                    }),
                });
                return Err(error);
            }
        };
        let mut source_reports = Vec::new();
        let mut imported = 0;
        let mut skipped_duplicates = 0;
        let mut rejected = 0;
        let mut failed_sources = 0;
        let mut rate_limited_sources = 0;
        let mut digest_candidates = 0;
        let mut stopped_reason = None;

        for source in &watch_sources {
            let handle = source.locator.clone();
            let cursor_key = format!("x:watch:{handle}");
            let previous_cursor = self.get_cursor(&cursor_key)?.map(|cursor| cursor.value);
            let source_started_at = now();
            let result = self.x_poll_watch_source(
                &base,
                &token,
                &handle,
                &cursor_key,
                previous_cursor.as_deref(),
                max_results_per_source,
            );
            match result {
                Ok(report) => {
                    let completed_at = now();
                    self.record_x_sync_run(XSyncRunInsert {
                        account_id: None,
                        stream: "watch_monitor",
                        transport: "x_api",
                        status: "completed",
                        started_at: &source_started_at,
                        completed_at: &completed_at,
                        seen: report.seen,
                        inserted: report.imported,
                        updated: 0,
                        skipped_duplicates: report.skipped_duplicates,
                        rejected: report.rejected,
                        cursor_key: Some(&cursor_key),
                        previous_cursor: previous_cursor.as_deref(),
                        new_cursor: report.effective_cursor.as_deref(),
                        error: None,
                        metadata: json!({
                            "handle": handle,
                            "max_results": max_results_per_source
                        }),
                    })?;
                    imported += report.imported;
                    skipped_duplicates += report.skipped_duplicates;
                    rejected += report.rejected;
                    if report.digest_candidate_id.is_some() {
                        digest_candidates += 1;
                    }
                    source_reports.push(report);
                }
                Err(error) => {
                    if x_failure_should_release_budget(&error) {
                        let _ = self.release_cost_reservation(
                            "arcwell-x",
                            "x_monitor",
                            "x",
                            "watch_recent_search",
                            Some("x_monitor"),
                        );
                    }
                    failed_sources += 1;
                    let error_text = redact_secret_like_text(&error.to_string());
                    let failure_classification = classify_provider_failure(&error_text);
                    if failure_classification.status == "rate_limited" {
                        rate_limited_sources += 1;
                    }
                    let completed_at = now();
                    let _ = self.record_source_failure(
                        &cursor_key,
                        "x",
                        "x_monitor",
                        &handle,
                        &error_text,
                    );
                    let _ = self.record_x_sync_run(XSyncRunInsert {
                        account_id: None,
                        stream: "watch_monitor",
                        transport: "x_api",
                        status: "failed",
                        started_at: &source_started_at,
                        completed_at: &completed_at,
                        seen: 0,
                        inserted: 0,
                        updated: 0,
                        skipped_duplicates: 0,
                        rejected: 0,
                        cursor_key: Some(&cursor_key),
                        previous_cursor: previous_cursor.as_deref(),
                        new_cursor: None,
                        error: Some(&error_text),
                        metadata: json!({
                            "handle": handle,
                            "max_results": max_results_per_source,
                            "failure_kind": failure_classification.status,
                            "rate_limit_failures_so_far": rate_limited_sources,
                            "rate_limit_abort_threshold": X_MONITOR_MAX_RATE_LIMIT_FAILURES_PER_RUN
                        }),
                    });
                    source_reports.push(XMonitorSourceReport {
                        handle,
                        cursor_key,
                        previous_cursor,
                        newest_id: None,
                        effective_cursor: None,
                        seen: 0,
                        imported: 0,
                        skipped_duplicates: 0,
                        rejected: 0,
                        digest_candidate_id: None,
                        status: "failed".to_string(),
                        error: Some(excerpt(&error_text, 2000)),
                    });
                    if rate_limited_sources >= X_MONITOR_MAX_RATE_LIMIT_FAILURES_PER_RUN
                        && source_reports.len() < watch_sources.len()
                    {
                        stopped_reason = Some(format!(
                            "rate_limit_abort_after_{}_failures",
                            X_MONITOR_MAX_RATE_LIMIT_FAILURES_PER_RUN
                        ));
                        break;
                    }
                }
            }
        }
        let deferred_sources = watch_sources.len().saturating_sub(source_reports.len());

        Ok(XMonitorReport {
            watched_sources: watch_sources.len(),
            polled_sources: source_reports.len(),
            attempted_sources: source_reports.len(),
            deferred_sources,
            imported,
            skipped_duplicates,
            rejected,
            failed_sources,
            rate_limited_sources,
            digest_candidates,
            stopped_reason,
            sources: source_reports,
        })
    }

    pub(crate) fn x_poll_watch_source(
        &self,
        base: &Url,
        token: &str,
        handle: &str,
        cursor_key: &str,
        previous_cursor: Option<&str>,
        max_results: usize,
    ) -> Result<XMonitorSourceReport> {
        validate_x_handle(handle)?;
        let url = x_watch_recent_search_url(base, handle, previous_cursor, max_results)?;
        let mut stale_since_id_retried = false;
        let value = match fetch_x_json(url.as_str(), Some(token)) {
            Ok(value) => match x_fail_on_response_errors(&value) {
                Ok(()) => value,
                Err(error) if previous_cursor.is_some() && x_is_stale_since_id_error(&error) => {
                    stale_since_id_retried = true;
                    let url = x_watch_recent_search_url(base, handle, None, max_results)?;
                    let value = fetch_x_json(url.as_str(), Some(token))?;
                    x_fail_on_response_errors(&value)?;
                    value
                }
                Err(error) => return Err(error),
            },
            Err(error) if previous_cursor.is_some() && x_is_stale_since_id_error(&error) => {
                stale_since_id_retried = true;
                let url = x_watch_recent_search_url(base, handle, None, max_results)?;
                let value = fetch_x_json(url.as_str(), Some(token))?;
                x_fail_on_response_errors(&value)?;
                value
            }
            Err(error) => return Err(error),
        };
        let import_value =
            x_search_response_to_import_items(&value, "watch_monitor", Some(handle))?;
        let report = self.import_x_json_value_without_sync_run(&import_value)?;
        if report.rejected > 0 {
            let first_error = report
                .rejected_errors
                .first()
                .map(|error| format!("; first rejection: {error}"))
                .unwrap_or_default();
            bail!(
                "X monitor source @{handle} returned {rejected} malformed item(s){first_error}; cursor was not advanced",
                rejected = report.rejected
            );
        }

        let newest_id = value
            .pointer("/meta/newest_id")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        let cursor_baseline = if stale_since_id_retried {
            None
        } else {
            previous_cursor
        };
        let effective_cursor = x_effective_cursor(cursor_baseline, newest_id.as_deref());
        if effective_cursor.as_deref() != previous_cursor
            && let Some(cursor) = &effective_cursor
        {
            self.set_cursor(cursor_key, cursor)?;
        } else if stale_since_id_retried && effective_cursor.is_none() {
            self.delete_cursor(cursor_key)?;
        }

        let source_card_ids: Vec<String> = report
            .items
            .iter()
            .filter_map(|item| item.source_card_id.clone())
            .collect();
        let digest_candidate_id = if source_card_ids.is_empty() {
            None
        } else {
            let candidate =
                self.create_digest_candidate(&format!("X watch @{handle}"), &source_card_ids)?;
            for item in &report.items {
                if let Some(source_card_id) = item.source_card_id.as_deref() {
                    self.upsert_x_digest_projection(
                        &item.x_id,
                        source_card_id,
                        item.wiki_page_id.as_deref(),
                        &candidate.id,
                    )?;
                }
            }
            Some(candidate.id)
        };

        self.record_source_success(SourceHealthUpdate {
            key: cursor_key,
            provider: "x",
            source_kind: "x_monitor",
            locator: handle,
            last_item_id: report.items.first().map(|item| item.x_id.as_str()),
            last_item_date: report
                .items
                .first()
                .and_then(|item| item.created_at.as_deref()),
            cursor_key: Some(cursor_key),
            cursor_value: effective_cursor.as_deref(),
            next_run_at: Some(&now_plus_seconds(
                self.watch_source_next_run_seconds("x_handle", handle, 900),
            )),
        })?;

        Ok(XMonitorSourceReport {
            handle: handle.to_string(),
            cursor_key: cursor_key.to_string(),
            previous_cursor: previous_cursor.map(ToOwned::to_owned),
            newest_id,
            effective_cursor,
            seen: report.seen,
            imported: report.imported,
            skipped_duplicates: report.skipped_duplicates,
            rejected: report.rejected,
            digest_candidate_id,
            status: "healthy".to_string(),
            error: None,
        })
    }

    pub(crate) fn x_bearer_token_for_endpoint(&self, endpoint: &str) -> Result<String> {
        if let Ok(token) = std::env::var("X_BEARER_TOKEN")
            && !token.trim().is_empty()
        {
            return Ok(token);
        }
        match self.get_usable_secret_value("X_BEARER_TOKEN") {
            Ok(Some(token)) => Ok(token),
            Ok(None) => {
                if self.get_usable_secret_value("X_REFRESH_TOKEN")?.is_some() {
                    self.refresh_x_bearer_token_for_endpoint(endpoint)
                } else {
                    bail!("X_BEARER_TOKEN is required")
                }
            }
            Err(error) => {
                let error_text = error.to_string();
                if error_text.contains("X_BEARER_TOKEN") && error_text.contains("expired") {
                    if self.get_usable_secret_value("X_REFRESH_TOKEN")?.is_some() {
                        self.refresh_x_bearer_token_for_endpoint(endpoint).map_err(
                            |refresh_error| {
                                anyhow::anyhow!(
                                    "refreshing expired X_BEARER_TOKEN failed: {}",
                                    redact_secret_like_text(&refresh_error.to_string())
                                )
                            },
                        )
                    } else {
                        Err(error)
                    }
                } else {
                    Err(error)
                }
            }
        }
    }

    pub(crate) fn refresh_x_bearer_token_for_endpoint(&self, endpoint: &str) -> Result<String> {
        let client_id = self
            .resolve_x_client_id()?
            .context("X_CLIENT_ID is required to refresh X_BEARER_TOKEN")?;
        self.x_oauth_refresh_with_base(&client_id, None, endpoint)?;
        self.get_usable_secret_value("X_BEARER_TOKEN")?
            .context("X OAuth refresh did not store a usable X_BEARER_TOKEN")
    }

    pub(crate) fn resolve_x_client_id(&self) -> Result<Option<String>> {
        let client_id = if let Ok(value) = std::env::var("X_CLIENT_ID")
            && !value.trim().is_empty()
        {
            Some(value.trim().to_string())
        } else if let Ok(value) = std::env::var("TWITTER_OAUTH2_CLIENT_ID")
            && !value.trim().is_empty()
        {
            Some(value.trim().to_string())
        } else {
            self.get_usable_secret_value("X_CLIENT_ID")?
                .or_else(|| {
                    self.get_usable_secret_value("TWITTER_OAUTH2_CLIENT_ID")
                        .ok()
                        .flatten()
                })
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        };
        if let Some(client_id) = &client_id {
            validate_key(client_id)?;
        }
        Ok(client_id)
    }

    pub(crate) fn x_user_id(&self, base: &Url, token: &str) -> Result<String> {
        let me_url = base.join("/2/users/me?user.fields=username,name")?;
        let me = fetch_x_json(me_url.as_str(), Some(token))?;
        let user_id = me
            .pointer("/data/id")
            .and_then(Value::as_str)
            .context("X /2/users/me response missing data.id")?;
        validate_key(user_id)?;
        Ok(user_id.to_string())
    }
}

fn x_watch_recent_search_url(
    base: &Url,
    handle: &str,
    previous_cursor: Option<&str>,
    max_results: usize,
) -> Result<Url> {
    let mut url = base.join("/2/tweets/search/recent")?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs
            .append_pair("query", &format!("from:{handle} -is:retweet"))
            .append_pair("max_results", &max_results.clamp(10, 100).to_string())
            .append_pair("tweet.fields", "created_at,author_id")
            .append_pair("expansions", "author_id")
            .append_pair("user.fields", "username,name");
        if let Some(previous_cursor) = previous_cursor {
            pairs.append_pair("since_id", previous_cursor);
        }
    }
    Ok(url)
}

fn x_is_stale_since_id_error(error: &anyhow::Error) -> bool {
    let error = error.to_string();
    error.contains("'since_id' must be a tweet id created after")
        || error.contains("\"since_id\" must be a tweet id created after")
}
