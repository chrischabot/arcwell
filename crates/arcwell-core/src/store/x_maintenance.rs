use super::*;

impl Store {
    pub(crate) fn ensure_x_canonical_schema(&self) -> Result<()> {
        ensure_x_canonical_schema_on(&self.conn)
    }

    pub fn x_rebuild_fts(&self) -> Result<XFtsRebuildReport> {
        let tweets_indexed = rebuild_x_tweets_fts_on(&self.conn)?;
        Ok(XFtsRebuildReport { tweets_indexed })
    }

    pub fn x_repair_projections(&self, limit: usize) -> Result<XProjectionRepairReport> {
        let limit = limit.clamp(1, 10_000);
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
              t.x_id,
              COALESCE(p.handle, 'unknown') AS author,
              t.text,
              t.url,
              t.created_at,
              t.last_seen_at,
              t.metrics_json,
              COALESCE(xp.status, 'missing') AS projection_status,
              xp.source_card_id,
              xp.wiki_page_id,
              sc.id AS existing_source_card_id,
              wp.id AS existing_wiki_page_id
            FROM x_tweets t
            LEFT JOIN x_profiles p ON p.id = t.author_profile_id
            LEFT JOIN x_projections xp
              ON xp.entity_kind = 'tweet'
             AND xp.entity_id = t.x_id
             AND xp.projection_kind = 'source_card'
            LEFT JOIN source_cards sc ON sc.id = xp.source_card_id
            LEFT JOIN wiki_pages wp ON wp.id = xp.wiki_page_id
            WHERE xp.id IS NULL
               OR xp.status != 'completed'
               OR xp.source_card_id IS NULL
               OR xp.wiki_page_id IS NULL
               OR sc.id IS NULL
               OR wp.id IS NULL
            ORDER BY COALESCE(t.created_at, t.first_seen_at) DESC, t.first_seen_at DESC
            LIMIT ?1
            "#,
        )?;
        let candidates = rows(stmt.query_map(params![limit as i64], |row| {
            let metrics_json: String = row.get(6)?;
            Ok(RepairableXTweetProjection {
                x_id: row.get(0)?,
                author: row.get(1)?,
                text: row.get(2)?,
                url: row.get(3)?,
                created_at: row.get(4)?,
                retrieved_at: row.get(5)?,
                metrics: parse_json_column(&metrics_json, 6)?,
                projection_status: row.get(7)?,
                source_card_id: row.get(8)?,
                wiki_page_id: row.get(9)?,
                existing_source_card_id: row.get(10)?,
                existing_wiki_page_id: row.get(11)?,
            })
        })?)?;
        let mut report = XProjectionRepairReport {
            generated_at: now(),
            limit,
            candidates: candidates.len(),
            repaired: 0,
            already_completed: 0,
            failed: 0,
            items: Vec::new(),
        };
        for candidate in candidates {
            if candidate.projection_status == "completed"
                && candidate.existing_source_card_id.is_some()
                && candidate.existing_wiki_page_id.is_some()
            {
                report.already_completed += 1;
                report.items.push(XProjectionRepairItem {
                    x_id: candidate.x_id,
                    status: "already_completed".to_string(),
                    source_card_id: candidate.source_card_id,
                    wiki_page_id: candidate.wiki_page_id,
                    error: None,
                });
                continue;
            }
            match self.repair_x_source_card_projection(&candidate) {
                Ok(card) => {
                    report.repaired += 1;
                    report.items.push(XProjectionRepairItem {
                        x_id: candidate.x_id,
                        status: "repaired".to_string(),
                        source_card_id: Some(card.id),
                        wiki_page_id: Some(card.wiki_page_id),
                        error: None,
                    });
                }
                Err(error) => {
                    let message = redact_secret_like_text(&error.to_string());
                    self.mark_x_projection_failed(&candidate.x_id, &message)?;
                    report.failed += 1;
                    report.items.push(XProjectionRepairItem {
                        x_id: candidate.x_id,
                        status: "failed".to_string(),
                        source_card_id: candidate.source_card_id,
                        wiki_page_id: candidate.wiki_page_id,
                        error: Some(message),
                    });
                }
            }
        }
        Ok(report)
    }

    pub fn x_thread(&self, x_id: &str, max_depth: usize) -> Result<XThreadReport> {
        validate_key(x_id)?;
        let max_depth = max_depth.clamp(1, 200);
        let root = self
            .load_x_thread_tweet(x_id)?
            .with_context(|| format!("local X tweet not found: {x_id}"))?;
        let conversation_id = root
            .conversation_id
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(&root.x_id)
            .to_string();
        let mut tweets = BTreeMap::new();
        tweets.insert(root.x_id.clone(), root);

        let conversation_tweets = self.load_x_thread_conversation(&conversation_id)?;
        for tweet in conversation_tweets {
            tweets.insert(tweet.x_id.clone(), tweet);
        }

        let mut missing = BTreeSet::new();
        let mut truncated = false;
        for _ in 0..max_depth {
            let refs = x_thread_refs(&tweets);
            let mut changed = false;
            for (tweet_x_id, ref_kind, ref_x_id) in refs {
                if ref_x_id == tweet_x_id || tweets.contains_key(&ref_x_id) {
                    continue;
                }
                if let Some(tweet) = self.load_x_thread_tweet(&ref_x_id)? {
                    tweets.insert(tweet.x_id.clone(), tweet);
                    changed = true;
                } else {
                    missing.insert(XThreadMissingContext {
                        tweet_x_id,
                        ref_kind,
                        ref_x_id,
                        reason: "missing_local_tweet".to_string(),
                    });
                }
            }
            let loaded_ids: Vec<String> = tweets.keys().cloned().collect();
            for loaded_id in loaded_ids {
                for tweet in self.load_x_thread_referencing(&loaded_id)? {
                    if tweets.contains_key(&tweet.x_id) {
                        continue;
                    }
                    tweets.insert(tweet.x_id.clone(), tweet);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        for (tweet_x_id, ref_kind, ref_x_id) in x_thread_refs(&tweets) {
            if ref_x_id == tweet_x_id || tweets.contains_key(&ref_x_id) {
                continue;
            }
            if self.load_x_thread_tweet(&ref_x_id)?.is_some() {
                truncated = true;
                missing.insert(XThreadMissingContext {
                    tweet_x_id,
                    ref_kind,
                    ref_x_id,
                    reason: "max_depth_exceeded".to_string(),
                });
            }
        }

        let mut cycle_detected = false;
        let mut output = Vec::new();
        for tweet in tweets.values() {
            let (depth, cycle, depth_truncated) =
                x_thread_reply_depth(tweet, x_id, &tweets, max_depth);
            cycle_detected |= cycle;
            truncated |= depth_truncated;
            output.push(XThreadTweet {
                x_id: tweet.x_id.clone(),
                author: tweet.author.clone(),
                text: tweet.text.clone(),
                url: tweet.url.clone(),
                created_at: tweet.created_at.clone(),
                first_seen_at: tweet.first_seen_at.clone(),
                conversation_id: tweet.conversation_id.clone(),
                reply_to_x_id: tweet.reply_to_x_id.clone(),
                quote_x_id: tweet.quote_x_id.clone(),
                retweet_x_id: tweet.retweet_x_id.clone(),
                relation_to_root: x_thread_relation(tweet, x_id, &conversation_id),
                depth,
                source_card_id: tweet.source_card_id.clone(),
                wiki_page_id: tweet.wiki_page_id.clone(),
            });
        }
        output.sort_by(|left, right| {
            left.created_at
                .cmp(&right.created_at)
                .then_with(|| left.first_seen_at.cmp(&right.first_seen_at))
                .then_with(|| left.x_id.cmp(&right.x_id))
        });

        Ok(XThreadReport {
            generated_at: now(),
            mode: "local".to_string(),
            root_x_id: x_id.to_string(),
            conversation_id: Some(conversation_id),
            max_depth,
            tweets: output,
            missing_context: missing.into_iter().collect(),
            cycle_detected,
            truncated,
        })
    }

    pub(crate) fn load_x_thread_tweet(&self, x_id: &str) -> Result<Option<LocalXThreadTweet>> {
        self.conn
            .query_row(
                r#"
                SELECT
                  t.x_id,
                  COALESCE(p.handle, 'unknown') AS author,
                  t.text,
                  t.url,
                  t.created_at,
                  t.first_seen_at,
                  t.conversation_id,
                  t.reply_to_x_id,
                  t.quote_x_id,
                  t.retweet_x_id,
                  xp.source_card_id,
                  xp.wiki_page_id
                FROM x_tweets t
                LEFT JOIN x_profiles p ON p.id = t.author_profile_id
                LEFT JOIN x_projections xp
                  ON xp.entity_kind = 'tweet'
                 AND xp.entity_id = t.x_id
                 AND xp.projection_kind = 'source_card'
                WHERE t.x_id = ?1
                "#,
                params![x_id],
                local_x_thread_tweet_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub(crate) fn load_x_thread_conversation(
        &self,
        conversation_id: &str,
    ) -> Result<Vec<LocalXThreadTweet>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
              t.x_id,
              COALESCE(p.handle, 'unknown') AS author,
              t.text,
              t.url,
              t.created_at,
              t.first_seen_at,
              t.conversation_id,
              t.reply_to_x_id,
              t.quote_x_id,
              t.retweet_x_id,
              xp.source_card_id,
              xp.wiki_page_id
            FROM x_tweets t
            LEFT JOIN x_profiles p ON p.id = t.author_profile_id
            LEFT JOIN x_projections xp
              ON xp.entity_kind = 'tweet'
             AND xp.entity_id = t.x_id
             AND xp.projection_kind = 'source_card'
            WHERE t.conversation_id = ?1
               OR t.x_id = ?1
            ORDER BY COALESCE(t.created_at, t.first_seen_at) ASC, t.first_seen_at ASC, t.x_id ASC
            "#,
        )?;
        rows(stmt.query_map(params![conversation_id], local_x_thread_tweet_from_row)?)
    }

    pub(crate) fn load_x_thread_referencing(
        &self,
        ref_x_id: &str,
    ) -> Result<Vec<LocalXThreadTweet>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
              t.x_id,
              COALESCE(p.handle, 'unknown') AS author,
              t.text,
              t.url,
              t.created_at,
              t.first_seen_at,
              t.conversation_id,
              t.reply_to_x_id,
              t.quote_x_id,
              t.retweet_x_id,
              xp.source_card_id,
              xp.wiki_page_id
            FROM x_tweets t
            LEFT JOIN x_profiles p ON p.id = t.author_profile_id
            LEFT JOIN x_projections xp
              ON xp.entity_kind = 'tweet'
             AND xp.entity_id = t.x_id
             AND xp.projection_kind = 'source_card'
            WHERE t.reply_to_x_id = ?1
               OR t.quote_x_id = ?1
               OR t.retweet_x_id = ?1
            ORDER BY COALESCE(t.created_at, t.first_seen_at) ASC, t.first_seen_at ASC, t.x_id ASC
            "#,
        )?;
        rows(stmt.query_map(params![ref_x_id], local_x_thread_tweet_from_row)?)
    }

    pub fn x_extract_links(&self, limit: usize) -> Result<XLinkIndexReport> {
        let limit = limit.clamp(1, 100_000);
        let mut stmt = self.conn.prepare(
            r#"
            SELECT x_id, text, url, entities_json, raw_json, last_seen_at
            FROM x_tweets
            ORDER BY COALESCE(created_at, first_seen_at) DESC, first_seen_at DESC, x_id ASC
            LIMIT ?1
            "#,
        )?;
        let tweets = rows(stmt.query_map(params![limit as i64], |row| {
            let entities_json: String = row.get(3)?;
            let raw_json: String = row.get(4)?;
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                parse_json_column(&entities_json, 3)?,
                parse_json_column(&raw_json, 4)?,
                row.get::<_, String>(5)?,
            ))
        })?)?;

        let mut links = Vec::new();
        let mut skipped_unsafe = 0;
        let mut seen = BTreeSet::new();
        for (tweet_x_id, text, tweet_url, entities, raw, seen_at) in &tweets {
            for candidate in x_link_candidates(&text, &tweet_url, &entities, &raw) {
                let selected_url = candidate
                    .expanded_url
                    .as_deref()
                    .unwrap_or(candidate.url.as_str())
                    .to_string();
                if validate_indexable_x_link_url(&selected_url).is_err() {
                    skipped_unsafe += 1;
                    continue;
                }
                let key = format!("{tweet_x_id}\n{selected_url}");
                if !seen.insert(key) {
                    continue;
                }
                let raw_json = canonical_json(&candidate.raw)?;
                self.conn.execute(
                    r#"
                    INSERT INTO x_tweet_links
                      (tweet_x_id, url, expanded_url, display_url, source, first_seen_at, last_seen_at, raw_json)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, ?7)
                    ON CONFLICT(tweet_x_id, url, source) DO UPDATE SET
                      expanded_url = COALESCE(excluded.expanded_url, x_tweet_links.expanded_url),
                      display_url = COALESCE(excluded.display_url, x_tweet_links.display_url),
                      last_seen_at = excluded.last_seen_at,
                      raw_json = excluded.raw_json
                    "#,
                    params![
                        tweet_x_id,
                        selected_url,
                        candidate.expanded_url,
                        candidate.display_url,
                        candidate.source,
                        seen_at,
                        raw_json
                    ],
                )?;
                links.push(self.load_x_link(tweet_x_id, &selected_url, &candidate.source)?);
            }
        }
        Ok(XLinkIndexReport {
            generated_at: now(),
            limit,
            tweets_scanned: tweets.len(),
            links_indexed: links.len(),
            skipped_unsafe,
            links,
        })
    }

    pub fn x_links(&self, query: Option<&str>, limit: usize) -> Result<Vec<XLinkOccurrence>> {
        let limit = limit.clamp(1, 10_000);
        if let Some(query) = query {
            validate_query(query)?;
            let pattern = format!("%{}%", query.replace('%', "\\%").replace('_', "\\_"));
            let mut stmt = self.conn.prepare(
                r#"
                SELECT tweet_x_id, url, expanded_url, display_url, source, first_seen_at, last_seen_at
                FROM x_tweet_links
                WHERE url LIKE ?1 ESCAPE '\'
                   OR COALESCE(expanded_url, '') LIKE ?1 ESCAPE '\'
                   OR COALESCE(display_url, '') LIKE ?1 ESCAPE '\'
                   OR tweet_x_id LIKE ?1 ESCAPE '\'
                ORDER BY last_seen_at DESC, tweet_x_id ASC, url ASC
                LIMIT ?2
                "#,
            )?;
            rows(stmt.query_map(params![pattern, limit as i64], x_link_occurrence_from_row)?)
        } else {
            let mut stmt = self.conn.prepare(
                r#"
                SELECT tweet_x_id, url, expanded_url, display_url, source, first_seen_at, last_seen_at
                FROM x_tweet_links
                ORDER BY last_seen_at DESC, tweet_x_id ASC, url ASC
                LIMIT ?1
                "#,
            )?;
            rows(stmt.query_map(params![limit as i64], x_link_occurrence_from_row)?)
        }
    }

    pub(crate) fn load_x_link(
        &self,
        tweet_x_id: &str,
        url: &str,
        source: &str,
    ) -> Result<XLinkOccurrence> {
        self.conn
            .query_row(
                r#"
                SELECT tweet_x_id, url, expanded_url, display_url, source, first_seen_at, last_seen_at
                FROM x_tweet_links
                WHERE tweet_x_id = ?1 AND url = ?2 AND source = ?3
                "#,
                params![tweet_x_id, url, source],
                x_link_occurrence_from_row,
            )
            .with_context(|| format!("indexed X link not found: {tweet_x_id} {url} {source}"))
    }

    pub fn x_expand_links(&self, limit: usize) -> Result<XLinkExpansionReport> {
        let limit = limit.clamp(1, 1_000);
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
              l.url,
              COALESCE(e.status, 'missing') AS status,
              e.wiki_page_id,
              e.final_url,
              e.canonical_url,
              e.content_type,
              e.bytes,
              e.last_error,
              wp.id AS existing_wiki_page_id
            FROM (SELECT DISTINCT url FROM x_tweet_links) l
            LEFT JOIN x_link_expansions e ON e.url = l.url
            LEFT JOIN wiki_pages wp ON wp.id = e.wiki_page_id
            WHERE e.url IS NULL
               OR e.status != 'completed'
               OR e.wiki_page_id IS NULL
               OR wp.id IS NULL
            ORDER BY l.url ASC
            LIMIT ?1
            "#,
        )?;
        let candidates = rows(stmt.query_map(params![limit as i64], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<i64>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
            ))
        })?)?;
        let mut report = XLinkExpansionReport {
            generated_at: now(),
            limit,
            candidates: candidates.len(),
            expanded: 0,
            already_completed: 0,
            failed: 0,
            items: Vec::new(),
        };
        for (
            url,
            status,
            wiki_page_id,
            final_url,
            canonical_url,
            content_type,
            bytes,
            last_error,
            existing_wiki_page_id,
        ) in candidates
        {
            if status == "completed" && wiki_page_id.is_some() && existing_wiki_page_id.is_some() {
                report.already_completed += 1;
                report.items.push(XLinkExpansionItem {
                    url,
                    status: "already_completed".to_string(),
                    wiki_page_id,
                    final_url,
                    canonical_url,
                    content_type,
                    bytes: bytes.map(nonnegative_usize),
                    error: last_error,
                });
                continue;
            }
            match self.expand_one_x_link(&url) {
                Ok(item) => {
                    report.expanded += 1;
                    report.items.push(item);
                }
                Err(error) => {
                    let message = redact_secret_like_text(&error.to_string());
                    self.record_x_link_expansion_failure(&url, &message)?;
                    report.failed += 1;
                    report.items.push(XLinkExpansionItem {
                        url,
                        status: "failed".to_string(),
                        wiki_page_id: None,
                        final_url: None,
                        canonical_url: None,
                        content_type: None,
                        bytes: None,
                        error: Some(message),
                    });
                }
            }
        }
        Ok(report)
    }

    pub(crate) fn expand_one_x_link(&self, raw_url: &str) -> Result<XLinkExpansionItem> {
        let url = validate_fetch_url(raw_url)?;
        self.guard_provider_network_policy(
            "arcwell-x",
            "web",
            "x_link_expand",
            url.as_str(),
            estimated_network_fetch_cost(1),
            json!({ "entrypoint": "x_expand_links" }),
        )?;
        self.require_cost_budget(
            "arcwell-x",
            &format!("x-link-expand-{}", &sha256(url.as_str().as_bytes())[..32]),
            "web",
            "x_link_expand",
            Some("x_link_expand"),
            estimated_network_fetch_cost(1),
            "X link expansion",
        )?;
        let doc = fetch_url_ingest_document(url)?;
        let markdown = render_url_ingest_page(&doc);
        let page_id = self.add_wiki_page(
            &format!("X Link Expansion: {}", doc.title),
            &markdown,
            &format!("x-link-expand:{}", doc.canonical_url),
        )?;
        let now_value = now();
        self.conn.execute(
            r#"
            INSERT INTO x_link_expansions
              (url, status, wiki_page_id, final_url, canonical_url, content_type, bytes, last_error, first_attempted_at, updated_at)
            VALUES (?1, 'completed', ?2, ?3, ?4, ?5, ?6, NULL, ?7, ?7)
            ON CONFLICT(url) DO UPDATE SET
              status = 'completed',
              wiki_page_id = excluded.wiki_page_id,
              final_url = excluded.final_url,
              canonical_url = excluded.canonical_url,
              content_type = excluded.content_type,
              bytes = excluded.bytes,
              last_error = NULL,
              updated_at = excluded.updated_at
            "#,
            params![
                raw_url,
                page_id,
                doc.final_url,
                doc.canonical_url,
                doc.content_type,
                count_to_i64(doc.byte_len),
                now_value
            ],
        )?;
        Ok(XLinkExpansionItem {
            url: raw_url.to_string(),
            status: "expanded".to_string(),
            wiki_page_id: Some(page_id),
            final_url: Some(doc.final_url),
            canonical_url: Some(doc.canonical_url),
            content_type: Some(doc.content_type),
            bytes: Some(doc.byte_len),
            error: None,
        })
    }

    pub(crate) fn record_x_link_expansion_failure(&self, url: &str, error: &str) -> Result<()> {
        let now_value = now();
        self.conn.execute(
            r#"
            INSERT INTO x_link_expansions
              (url, status, last_error, first_attempted_at, updated_at)
            VALUES (?1, 'failed', ?2, ?3, ?3)
            ON CONFLICT(url) DO UPDATE SET
              status = 'failed',
              wiki_page_id = NULL,
              final_url = NULL,
              canonical_url = NULL,
              content_type = NULL,
              bytes = NULL,
              last_error = excluded.last_error,
              updated_at = excluded.updated_at
            "#,
            params![url, excerpt(error, 2000), now_value],
        )?;
        Ok(())
    }

    pub(crate) fn repair_x_source_card_projection(
        &self,
        candidate: &RepairableXTweetProjection,
    ) -> Result<SourceCard> {
        let card = self.add_source_card(x_source_card_input_from_repair(candidate))?;
        upsert_x_projection_on(
            &self.conn,
            &candidate.x_id,
            Some(&card.id),
            Some(&card.wiki_page_id),
        )?;
        self.conn.execute(
            r#"
            UPDATE x_items
            SET source_card_id = COALESCE(source_card_id, ?2),
                wiki_page_id = COALESCE(wiki_page_id, ?3)
            WHERE x_id = ?1
            "#,
            params![candidate.x_id, card.id, card.wiki_page_id],
        )?;
        Ok(card)
    }

    pub(crate) fn mark_x_projection_failed(&self, x_id: &str, error: &str) -> Result<()> {
        let id = format!(
            "xproj-{}",
            &sha256(format!("tweet\n{x_id}\nsource_card").as_bytes())[..32]
        );
        let now_value = now();
        self.conn.execute(
            r#"
            INSERT INTO x_projections
              (id, entity_kind, entity_id, projection_kind, status, last_error, created_at, updated_at)
            VALUES (?1, 'tweet', ?2, 'source_card', 'failed', ?3, ?4, ?4)
            ON CONFLICT(entity_kind, entity_id, projection_kind) DO UPDATE SET
              status = 'failed',
              last_error = excluded.last_error,
              updated_at = excluded.updated_at
            "#,
            params![id, x_id, error, now_value],
        )?;
        Ok(())
    }

    pub(crate) fn upsert_x_digest_projection(
        &self,
        x_id: &str,
        source_card_id: &str,
        wiki_page_id: Option<&str>,
        digest_candidate_id: &str,
    ) -> Result<()> {
        validate_key(x_id)?;
        validate_id(source_card_id)?;
        if let Some(wiki_page_id) = wiki_page_id {
            validate_id(wiki_page_id)?;
        }
        validate_id(digest_candidate_id)?;
        let id = format!(
            "xproj-{}",
            &sha256(format!("tweet\n{x_id}\ndigest_candidate").as_bytes())[..32]
        );
        let now_value = now();
        self.conn.execute(
            r#"
            INSERT INTO x_projections
              (id, entity_kind, entity_id, projection_kind, status, source_card_id, wiki_page_id, digest_candidate_id, last_error, created_at, updated_at)
            VALUES (?1, 'tweet', ?2, 'digest_candidate', 'completed', ?3, ?4, ?5, NULL, ?6, ?6)
            ON CONFLICT(entity_kind, entity_id, projection_kind) DO UPDATE SET
              status = 'completed',
              source_card_id = excluded.source_card_id,
              wiki_page_id = COALESCE(excluded.wiki_page_id, x_projections.wiki_page_id),
              digest_candidate_id = excluded.digest_candidate_id,
              last_error = NULL,
              updated_at = excluded.updated_at
            "#,
            params![
                id,
                x_id,
                source_card_id,
                wiki_page_id,
                digest_candidate_id,
                now_value
            ],
        )?;
        Ok(())
    }

    pub fn x_stats(&self) -> Result<XStatsReport> {
        let mut latest_stmt = self.conn.prepare(
            r#"
            SELECT id, account_id, stream, transport, status, started_at, completed_at, seen, inserted,
                   updated, skipped_duplicates, rejected, cursor_key, previous_cursor,
                   new_cursor, error
            FROM x_sync_runs
            ORDER BY started_at DESC
            LIMIT 10
            "#,
        )?;
        let latest_sync_runs = rows(latest_stmt.query_map([], x_sync_run_summary_from_row)?)?;
        Ok(XStatsReport {
            generated_at: now(),
            compatibility: XCompatibilityStats {
                x_items: self.count("x_items")?,
                x_item_sources: self.count("x_item_sources")?,
            },
            canonical: XCanonicalStats {
                accounts: self.count("x_accounts")?,
                profiles: self.count("x_profiles")?,
                profile_snapshots: self.count("x_profile_snapshots")?,
                profile_entities: self.count("x_profile_entities")?,
                tweets: self.count("x_tweets")?,
                tweet_refs: self.count("x_tweet_refs")?,
                tweet_edges: self.count("x_tweet_edges")?,
                collections: self.count("x_collections")?,
                projections: self.count("x_projections")?,
                sync_runs: self.count("x_sync_runs")?,
                fts_rows: self.count("x_tweets_fts")?,
                x_cursors: self.count_query("SELECT COUNT(*) FROM cursors WHERE key LIKE 'x:%'")?,
            },
            drift: XStatsDrift {
                compatibility_without_canonical: self.count_query(
                    "SELECT COUNT(*) FROM x_items i LEFT JOIN x_tweets t ON t.x_id = i.x_id WHERE t.x_id IS NULL",
                )?,
                canonical_without_compatibility: self.count_query(
                    "SELECT COUNT(*) FROM x_tweets t LEFT JOIN x_items i ON i.x_id = t.x_id WHERE i.x_id IS NULL",
                )?,
                tweets_without_fts: self.count_query(
                    "SELECT COUNT(*) FROM x_tweets t LEFT JOIN x_tweets_fts f ON f.x_id = t.x_id WHERE f.x_id IS NULL",
                )?,
                fts_without_tweets: self.count_query(
                    "SELECT COUNT(*) FROM x_tweets_fts f LEFT JOIN x_tweets t ON t.x_id = f.x_id WHERE t.x_id IS NULL",
                )?,
                projection_failures: self.count_query(
                    "SELECT COUNT(*) FROM x_projections WHERE status = 'failed'",
                )?,
                non_healthy_sources: self.count_query(
                    r#"
                    SELECT COUNT(*)
                    FROM source_health
                    WHERE (provider = 'x' OR key LIKE 'x:%')
                      AND status != 'healthy'
                      AND NOT (
                        status = 'rate_limited'
                        AND next_run_at IS NOT NULL
                        AND next_run_at > strftime('%Y-%m-%dT%H:%M:%f+00:00', 'now')
                      )
                    "#,
                )?,
            },
            portable_export: self.x_portable_export_freshness()?,
            projections_by_status: self.grouped_counts(
                "SELECT status, COUNT(*) FROM x_projections GROUP BY status ORDER BY status",
            )?,
            digest_projections_by_status: self.grouped_counts(
                "SELECT status, COUNT(*) FROM x_projections WHERE projection_kind = 'digest_candidate' GROUP BY status ORDER BY status",
            )?,
            digest_candidates_linked_to_x: self.count_query(
                "SELECT COUNT(DISTINCT digest_candidate_id) FROM x_projections WHERE projection_kind = 'digest_candidate' AND digest_candidate_id IS NOT NULL",
            )?,
            sync_runs_by_status: self.grouped_counts(
                "SELECT status, COUNT(*) FROM x_sync_runs GROUP BY status ORDER BY status",
            )?,
            unresolved_failed_sync_runs: self.count_query(
                r#"
                SELECT COUNT(*)
                FROM x_sync_runs failed
                WHERE failed.status = 'failed'
                  AND NOT EXISTS (
                    SELECT 1
                    FROM x_sync_runs later
                    WHERE later.status = 'completed'
                      AND later.started_at > failed.started_at
                      AND later.stream = failed.stream
                      AND COALESCE(later.cursor_key, '') = COALESCE(failed.cursor_key, '')
                  )
                  AND NOT EXISTS (
                    SELECT 1
                    FROM source_health health
                    WHERE health.status = 'rate_limited'
                      AND health.next_run_at IS NOT NULL
                      AND health.next_run_at > strftime('%Y-%m-%dT%H:%M:%f+00:00', 'now')
                      AND COALESCE(health.cursor_key, health.key, '') = COALESCE(failed.cursor_key, '')
                  )
                  AND NOT (
                    failed.stream = 'watch_monitor'
                    AND failed.cursor_key LIKE 'x:watch:%'
                    AND NOT EXISTS (
                      SELECT 1
                      FROM watch_sources ws
                      WHERE ws.source_kind = 'x_handle'
                        AND ws.status = 'active'
                        AND ws.locator = substr(failed.cursor_key, length('x:watch:') + 1)
                    )
                  )
                "#,
            )?,
            source_health_by_status: self.grouped_counts(
                "SELECT status, COUNT(*) FROM source_health WHERE provider = 'x' OR key LIKE 'x:%' GROUP BY status ORDER BY status",
            )?,
            watch_sources_by_status: self.grouped_counts(
                "SELECT status, COUNT(*) FROM watch_sources WHERE source_kind = 'x_handle' GROUP BY status ORDER BY status",
            )?,
            latest_sync_runs,
        })
    }

    pub fn x_repair_health(
        &self,
        defer_rate_limited_hours: i64,
        limit: usize,
    ) -> Result<XHealthRepairReport> {
        let deferred = self.x_defer_rate_limited_sources(defer_rate_limited_hours, limit)?;
        let next_bookmark_run_at = now_plus_seconds(6 * 60 * 60);
        let repaired_bookmark_health = self.conn.execute(
            r#"
            UPDATE source_health
            SET status = 'healthy',
                last_success_at = (
                  SELECT MAX(completed_at)
                  FROM x_sync_runs
                  WHERE stream = 'bookmarks' AND status = 'completed'
                ),
                last_error = NULL,
                next_run_at = ?1,
                updated_at = ?2
            WHERE key = 'x:bookmarks'
              AND status != 'healthy'
              AND EXISTS (
                SELECT 1
                FROM x_sync_runs later
                WHERE later.stream = 'bookmarks'
                  AND later.status = 'completed'
                  AND later.completed_at > COALESCE(source_health.last_failure_at, '')
              )
            "#,
            params![next_bookmark_run_at, now()],
        )?;
        let next_watch_run_at = now_plus_seconds(defer_rate_limited_hours.clamp(1, 24 * 30) * 3600);
        let repaired_watch_health = self.conn.execute(
            r#"
            UPDATE source_health
            SET status = 'healthy',
                last_success_at = (
                  SELECT MAX(later.completed_at)
                  FROM x_sync_runs later
                  WHERE later.stream = 'watch_monitor'
                    AND later.status = 'completed'
                    AND COALESCE(later.cursor_key, '') = source_health.key
                ),
                last_error = NULL,
                next_run_at = ?1,
                updated_at = ?2
            WHERE (provider = 'x' OR key LIKE 'x:%')
              AND source_kind = 'x_monitor'
              AND status != 'healthy'
              AND EXISTS (
                SELECT 1
                FROM x_sync_runs later
                WHERE later.stream = 'watch_monitor'
                  AND later.status = 'completed'
                  AND COALESCE(later.cursor_key, '') = source_health.key
                  AND later.completed_at > COALESCE(source_health.last_failure_at, '')
              )
            "#,
            params![next_watch_run_at, now()],
        )?;
        let retired_legacy_x_handle_health = self.conn.execute(
            r#"
            DELETE FROM source_health
            WHERE source_kind = 'x_handle'
              AND (provider = 'x' OR key LIKE 'x:%')
              AND NOT EXISTS (
                SELECT 1
                FROM watch_sources ws
                WHERE ws.source_kind = 'x_handle'
                  AND ws.locator = source_health.locator
                  AND ws.status = 'active'
              )
            "#,
            [],
        )?;
        let retired_orphan_x_monitor_health = self.conn.execute(
            r#"
            DELETE FROM source_health
            WHERE source_kind = 'x_monitor'
              AND (provider = 'x' OR key LIKE 'x:%')
              AND key != 'x:monitor'
              AND NOT EXISTS (
                SELECT 1
                FROM watch_sources ws
                WHERE ws.source_kind = 'x_handle'
                  AND ws.locator = source_health.locator
                  AND ws.status = 'active'
              )
            "#,
            [],
        )?;
        Ok(XHealthRepairReport {
            repaired_bookmark_health,
            repaired_watch_health,
            retired_legacy_x_handle_health,
            retired_orphan_x_monitor_health,
            rate_limited_scanned: deferred.scanned,
            rate_limited_deferred: deferred.deferred,
            defer_until: deferred.defer_until,
        })
    }

    pub(crate) fn x_defer_rate_limited_sources(
        &self,
        defer_rate_limited_hours: i64,
        limit: usize,
    ) -> Result<XRateLimitDeferReport> {
        let limit = limit.clamp(1, 100_000);
        let defer_until = now_plus_seconds(defer_rate_limited_hours.clamp(1, 24 * 30) * 3600);
        let scanned = self.count_query(
            r#"
            SELECT COUNT(*)
            FROM source_health
            WHERE (provider = 'x' OR key LIKE 'x:%')
              AND status = 'rate_limited'
              AND (
                next_run_at IS NULL
                OR next_run_at <= strftime('%Y-%m-%dT%H:%M:%f+00:00', 'now')
              )
            "#,
        )? as usize;
        let updated_at = now();
        let deferred = self.conn.execute(
            r#"
            UPDATE source_health
            SET next_run_at = ?1,
                updated_at = ?2
            WHERE rowid IN (
              SELECT rowid
              FROM source_health
              WHERE (provider = 'x' OR key LIKE 'x:%')
                AND status = 'rate_limited'
                AND (
                  next_run_at IS NULL
                  OR next_run_at <= strftime('%Y-%m-%dT%H:%M:%f+00:00', 'now')
                )
              ORDER BY updated_at ASC, key ASC
              LIMIT ?3
            )
            "#,
            params![defer_until, updated_at, limit as i64],
        )?;
        Ok(XRateLimitDeferReport {
            scanned,
            deferred,
            defer_until,
        })
    }

    pub(crate) fn x_portable_export_freshness(&self) -> Result<XPortableExportFreshness> {
        let current_tweet_count = self.count("x_tweets")?;
        let latest_completed: Option<(String, usize, Value)> = self
            .conn
            .query_row(
                r#"
                SELECT completed_at, seen, metadata_json
                FROM x_sync_runs
                WHERE stream = 'export_portable' AND status = 'completed'
                ORDER BY completed_at DESC
                LIMIT 1
                "#,
                [],
                |row| {
                    let metadata_json: String = row.get(2)?;
                    let metadata =
                        serde_json::from_str(&metadata_json).unwrap_or_else(|_| json!({}));
                    Ok((
                        row.get::<_, Option<String>>(0)?.unwrap_or_default(),
                        nonnegative_usize(row.get(1)?),
                        metadata,
                    ))
                },
            )
            .optional()?;
        let latest_failed: Option<(String, String)> = self
            .conn
            .query_row(
                r#"
                SELECT completed_at, COALESCE(error, '')
                FROM x_sync_runs
                WHERE stream = 'export_portable' AND status = 'failed'
                ORDER BY completed_at DESC
                LIMIT 1
                "#,
                [],
                |row| {
                    Ok((
                        row.get::<_, Option<String>>(0)?.unwrap_or_default(),
                        row.get::<_, String>(1)?,
                    ))
                },
            )
            .optional()?;
        let latest_tweet_updated_at: Option<String> =
            self.conn
                .query_row("SELECT MAX(updated_at) FROM x_tweets", [], |row| row.get(0))?;
        let tweets_updated_after_export = if let Some((completed_at, _, _)) = &latest_completed {
            self.conn.query_row(
                "SELECT COUNT(*) FROM x_tweets WHERE updated_at > ?1",
                params![completed_at],
                |row| row.get(0),
            )?
        } else {
            0
        };
        let (
            latest_completed_at,
            latest_rows_exported,
            latest_out_dir,
            latest_manifest_path,
            latest_manifest_sha256,
        ) = latest_completed
            .map(|(completed_at, rows, metadata)| {
                let out_dir = metadata
                    .get("out_dir")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned);
                let manifest_path = metadata
                    .get("manifest_path")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned);
                let manifest_sha256 = metadata
                    .get("manifest_sha256")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
                    .or_else(|| {
                        manifest_path.as_ref().and_then(|path| {
                            fs::read(path).ok().map(|bytes| sha256(&bytes)).or_else(|| {
                                out_dir.as_ref().and_then(|dir| {
                                    fs::read(Path::new(dir).join("manifest.json"))
                                        .ok()
                                        .map(|bytes| sha256(&bytes))
                                })
                            })
                        })
                    });
                (
                    Some(completed_at),
                    Some(rows),
                    out_dir,
                    manifest_path,
                    manifest_sha256,
                )
            })
            .unwrap_or((None, None, None, None, None));
        let (latest_failed_at, latest_error) = latest_failed
            .map(|(failed_at, error)| (Some(failed_at), Some(redact_secret_like_text(&error))))
            .unwrap_or((None, None));
        let missing = current_tweet_count > 0 && latest_completed_at.is_none();
        let row_count_mismatch = latest_rows_exported
            .map(|rows| rows as i64 != current_tweet_count)
            .unwrap_or(false);
        let stale = tweets_updated_after_export > 0 || row_count_mismatch;
        let latest_failed_after_completed = match (&latest_failed_at, &latest_completed_at) {
            (Some(failed_at), Some(completed_at)) => failed_at > completed_at,
            (Some(_), None) => true,
            _ => false,
        };
        let status = if missing {
            "missing"
        } else if stale {
            "stale"
        } else if latest_failed_after_completed {
            "failed"
        } else if latest_completed_at.is_some() {
            "fresh"
        } else {
            "empty"
        }
        .to_string();
        Ok(XPortableExportFreshness {
            status,
            missing,
            stale,
            latest_completed_at,
            latest_out_dir,
            latest_manifest_path,
            latest_manifest_sha256,
            latest_rows_exported,
            latest_failed_at,
            latest_error,
            current_tweet_count,
            latest_tweet_updated_at,
            tweets_updated_after_export,
            row_count_mismatch,
        })
    }

    pub(crate) fn x_health_warnings(stats: &XStatsReport) -> Vec<String> {
        let mut warnings = Vec::new();
        if stats.drift.compatibility_without_canonical > 0 {
            warnings.push(format!(
                "X compatibility drift: {} x_items row(s) have no canonical tweet",
                stats.drift.compatibility_without_canonical
            ));
        }
        if stats.drift.canonical_without_compatibility > 0 {
            warnings.push(format!(
                "X compatibility drift: {} canonical tweet(s) have no x_items projection",
                stats.drift.canonical_without_compatibility
            ));
        }
        if stats.drift.tweets_without_fts > 0 {
            warnings.push(format!(
                "X FTS drift: {} canonical tweet(s) are missing FTS rows",
                stats.drift.tweets_without_fts
            ));
        }
        if stats.drift.fts_without_tweets > 0 {
            warnings.push(format!(
                "X FTS drift: {} FTS row(s) have no canonical tweet",
                stats.drift.fts_without_tweets
            ));
        }
        if stats.drift.projection_failures > 0 {
            warnings.push(format!(
                "X projection failures: {} failed projection row(s)",
                stats.drift.projection_failures
            ));
        }
        if stats.drift.non_healthy_sources > 0 {
            warnings.push(format!(
                "X source health: {} non-healthy source row(s)",
                stats.drift.non_healthy_sources
            ));
        }
        if stats.unresolved_failed_sync_runs > 0 {
            warnings.push(format!(
                "X sync failures: {} unresolved failed sync run(s)",
                stats.unresolved_failed_sync_runs
            ));
        }
        if stats.portable_export.missing {
            warnings.push(format!(
                "X portable export missing: {} canonical tweet row(s) have no completed portable export",
                stats.portable_export.current_tweet_count
            ));
        } else if stats.portable_export.stale {
            warnings.push(format!(
                "X portable export is stale: {} tweet row(s) changed since latest export; row count mismatch: {}",
                stats.portable_export.tweets_updated_after_export,
                stats.portable_export.row_count_mismatch
            ));
        } else if stats.portable_export.status == "failed" {
            warnings.push("X portable export failed after the latest completed export".to_string());
        }
        warnings
    }

    pub fn search_x_tweets(&self, query: &str, limit: usize) -> Result<Vec<XItem>> {
        validate_query(query)?;
        let fts_query = x_fts_query(query).context("query has no searchable X terms")?;
        let limit = limit.clamp(1, 1_000) as i64;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
              COALESCE(i.id, t.id) AS id,
              t.x_id,
              COALESCE(i.author, p.handle, 'unknown') AS author,
              t.text,
              COALESCE(i.url, 'https://x.com/' || COALESCE(p.handle, 'unknown') || '/status/' || t.x_id) AS url,
              t.created_at,
              COALESCE(i.imported_at, t.first_seen_at) AS imported_at,
              COALESCE(i.retrieved_at, t.last_seen_at) AS retrieved_at,
              COALESCE(i.metrics_json, t.metrics_json) AS metrics_json,
              COALESCE(i.raw_json, t.raw_json) AS raw_json,
              COALESCE(i.source_card_id, (
                SELECT source_card_id
                FROM x_projections xp
                WHERE xp.entity_kind = 'tweet'
                  AND xp.entity_id = t.x_id
                  AND xp.projection_kind = 'source_card'
                  AND xp.source_card_id IS NOT NULL
                ORDER BY xp.updated_at DESC
                LIMIT 1
              )) AS source_card_id,
              COALESCE(i.wiki_page_id, (
                SELECT wiki_page_id
                FROM x_projections xp
                WHERE xp.entity_kind = 'tweet'
                  AND xp.entity_id = t.x_id
                  AND xp.projection_kind = 'source_card'
                  AND xp.wiki_page_id IS NOT NULL
                ORDER BY xp.updated_at DESC
                LIMIT 1
              )) AS wiki_page_id
            FROM x_tweets_fts f
            JOIN x_tweets t ON t.x_id = f.x_id
              LEFT JOIN x_profiles p ON p.id = t.author_profile_id
              LEFT JOIN x_items i ON i.x_id = t.x_id
              WHERE x_tweets_fts MATCH ?1
              ORDER BY COALESCE(t.created_at, t.first_seen_at) DESC, t.first_seen_at DESC
              LIMIT ?2
              "#,
        )?;
        let mut items = rows(stmt.query_map(params![fts_query, limit], x_item_from_row)?)?;
        for item in &mut items {
            item.sources = self.list_x_item_sources(&item.x_id)?;
        }
        Ok(items)
    }
}
