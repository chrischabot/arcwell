use super::*;

pub(crate) fn watch_source_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<WatchSource> {
    let metadata_json: String = row.get(6)?;
    let metadata = parse_json_column(&metadata_json, 6)?;
    Ok(WatchSource {
        id: row.get(0)?,
        source_kind: row.get(1)?,
        locator: row.get(2)?,
        label: row.get(3)?,
        cadence: row.get(4)?,
        status: row.get(5)?,
        metadata,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

pub(crate) fn wiki_job_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<WikiJob> {
    let input_json: String = row.get(3)?;
    let result_json: Option<String> = row.get(4)?;
    Ok(WikiJob {
        id: row.get(0)?,
        kind: row.get(1)?,
        status: row.get(2)?,
        input_json: parse_json_column(&input_json, 3)?,
        result_json: result_json
            .as_deref()
            .map(|raw| parse_json_column(raw, 4))
            .transpose()?,
        error: row.get(5)?,
        attempts: row.get(6)?,
        max_attempts: row.get(7)?,
        leased_until: row.get(8)?,
        worker_id: row.get(9)?,
        next_run_at: row.get(10)?,
        dead_lettered_at: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

pub(crate) fn research_run_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ResearchRun> {
    Ok(ResearchRun {
        id: row.get(0)?,
        query: row.get(1)?,
        status: row.get(2)?,
        result_page_id: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

pub(crate) fn x_item_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<XItem> {
    let metrics_json: String = row.get(8)?;
    let raw_json: String = row.get(9)?;
    Ok(XItem {
        id: row.get(0)?,
        x_id: row.get(1)?,
        author: row.get(2)?,
        text: row.get(3)?,
        url: row.get(4)?,
        created_at: row.get(5)?,
        imported_at: row.get(6)?,
        retrieved_at: row.get(7)?,
        metrics: parse_json_column(&metrics_json, 8)?,
        raw: parse_json_column(&raw_json, 9)?,
        source_card_id: row.get(10)?,
        wiki_page_id: row.get(11)?,
        sources: Vec::new(),
    })
}

pub(crate) fn x_item_source_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<XItemSource> {
    let metadata_json: String = row.get(5)?;
    Ok(XItemSource {
        id: row.get(0)?,
        x_id: row.get(1)?,
        source_kind: row.get(2)?,
        source_detail: row.get(3)?,
        seen_at: row.get(4)?,
        metadata: parse_json_column(&metadata_json, 5)?,
    })
}

pub(crate) fn local_x_thread_tweet_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<LocalXThreadTweet> {
    Ok(LocalXThreadTweet {
        x_id: row.get(0)?,
        author: row.get(1)?,
        text: row.get(2)?,
        url: row.get(3)?,
        created_at: row.get(4)?,
        first_seen_at: row.get(5)?,
        conversation_id: row.get(6)?,
        reply_to_x_id: row.get(7)?,
        quote_x_id: row.get(8)?,
        retweet_x_id: row.get(9)?,
        source_card_id: row.get(10)?,
        wiki_page_id: row.get(11)?,
    })
}

pub(crate) fn x_link_occurrence_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<XLinkOccurrence> {
    Ok(XLinkOccurrence {
        tweet_x_id: row.get(0)?,
        url: row.get(1)?,
        expanded_url: row.get(2)?,
        display_url: row.get(3)?,
        source: row.get(4)?,
        first_seen_at: row.get(5)?,
        last_seen_at: row.get(6)?,
    })
}

pub(crate) fn x_thread_refs(
    tweets: &BTreeMap<String, LocalXThreadTweet>,
) -> Vec<(String, String, String)> {
    let mut refs = Vec::new();
    for tweet in tweets.values() {
        if let Some(ref_x_id) = &tweet.conversation_id {
            refs.push((
                tweet.x_id.clone(),
                "conversation".to_string(),
                ref_x_id.clone(),
            ));
        }
        if let Some(ref_x_id) = &tweet.reply_to_x_id {
            refs.push((tweet.x_id.clone(), "reply_to".to_string(), ref_x_id.clone()));
        }
        if let Some(ref_x_id) = &tweet.quote_x_id {
            refs.push((tweet.x_id.clone(), "quote".to_string(), ref_x_id.clone()));
        }
        if let Some(ref_x_id) = &tweet.retweet_x_id {
            refs.push((tweet.x_id.clone(), "retweet".to_string(), ref_x_id.clone()));
        }
    }
    refs
}

pub(crate) fn x_thread_relation(
    tweet: &LocalXThreadTweet,
    root_x_id: &str,
    conversation_id: &str,
) -> String {
    if tweet.x_id == root_x_id {
        "root"
    } else if tweet.reply_to_x_id.as_deref() == Some(root_x_id) {
        "reply"
    } else if tweet.quote_x_id.as_deref() == Some(root_x_id) {
        "quote"
    } else if tweet.retweet_x_id.as_deref() == Some(root_x_id) {
        "retweet"
    } else if tweet.conversation_id.as_deref() == Some(conversation_id) {
        "conversation"
    } else {
        "referenced"
    }
    .to_string()
}

pub(crate) fn x_thread_reply_depth(
    tweet: &LocalXThreadTweet,
    root_x_id: &str,
    tweets: &BTreeMap<String, LocalXThreadTweet>,
    max_depth: usize,
) -> (usize, bool, bool) {
    if tweet.x_id == root_x_id {
        return (0, false, false);
    }
    let mut seen = BTreeSet::new();
    seen.insert(tweet.x_id.clone());
    let mut depth = 0;
    let mut next = tweet.reply_to_x_id.as_deref();
    while let Some(parent_x_id) = next {
        depth += 1;
        if parent_x_id == root_x_id {
            return (depth, false, false);
        }
        if !seen.insert(parent_x_id.to_string()) {
            return (depth, true, false);
        }
        if depth >= max_depth {
            return (depth, false, tweets.contains_key(parent_x_id));
        }
        next = tweets
            .get(parent_x_id)
            .and_then(|parent| parent.reply_to_x_id.as_deref());
    }
    if depth == 0 {
        (1, false, false)
    } else {
        (depth, false, false)
    }
}

pub(crate) fn x_sync_run_summary_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<XSyncRunSummary> {
    let error: Option<String> = row.get(15)?;
    Ok(XSyncRunSummary {
        id: row.get(0)?,
        account_id: row.get(1)?,
        stream: row.get(2)?,
        transport: row.get(3)?,
        status: row.get(4)?,
        started_at: row.get(5)?,
        completed_at: row.get(6)?,
        seen: nonnegative_usize(row.get(7)?),
        inserted: nonnegative_usize(row.get(8)?),
        updated: nonnegative_usize(row.get(9)?),
        skipped_duplicates: nonnegative_usize(row.get(10)?),
        rejected: nonnegative_usize(row.get(11)?),
        cursor_key: row.get(12)?,
        previous_cursor: row.get(13)?,
        new_cursor: row.get(14)?,
        error: error.map(|value| redact_secret_like_text(&value)),
    })
}

pub(crate) fn ensure_x_canonical_schema_on(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS x_accounts (
          id TEXT PRIMARY KEY,
          x_user_id TEXT UNIQUE,
          handle TEXT NOT NULL,
          display_name TEXT NOT NULL DEFAULT '',
          profile_id TEXT,
          is_default INTEGER NOT NULL DEFAULT 0,
          preferred_transport TEXT NOT NULL DEFAULT 'x_api',
          metadata_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS x_profiles (
          id TEXT PRIMARY KEY,
          x_user_id TEXT UNIQUE,
          handle TEXT NOT NULL,
          display_name TEXT NOT NULL DEFAULT '',
          description TEXT NOT NULL DEFAULT '',
          raw_json TEXT NOT NULL DEFAULT '{}',
          first_seen_at TEXT NOT NULL,
          last_seen_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS x_profile_snapshots (
          profile_id TEXT NOT NULL,
          snapshot_hash TEXT NOT NULL,
          observed_at TEXT NOT NULL,
          last_seen_at TEXT NOT NULL,
          source TEXT NOT NULL,
          handle TEXT NOT NULL,
          display_name TEXT NOT NULL DEFAULT '',
          description TEXT NOT NULL DEFAULT '',
          raw_json TEXT NOT NULL DEFAULT '{}',
          PRIMARY KEY(profile_id, snapshot_hash)
        );

        CREATE TABLE IF NOT EXISTS x_profile_entities (
          profile_id TEXT NOT NULL,
          kind TEXT NOT NULL,
          value TEXT NOT NULL,
          normalized_value TEXT NOT NULL,
          source TEXT NOT NULL,
          weight INTEGER NOT NULL DEFAULT 1,
          is_active INTEGER NOT NULL DEFAULT 1,
          first_seen_at TEXT NOT NULL,
          last_seen_at TEXT NOT NULL,
          PRIMARY KEY(profile_id, kind, value, source)
        );

        CREATE TABLE IF NOT EXISTS x_profile_aliases (
          profile_id TEXT NOT NULL,
          handle TEXT NOT NULL,
          normalized_handle TEXT NOT NULL,
          x_user_id TEXT,
          source TEXT NOT NULL,
          first_seen_at TEXT NOT NULL,
          last_seen_at TEXT NOT NULL,
          is_current INTEGER NOT NULL DEFAULT 0,
          raw_json TEXT NOT NULL DEFAULT '{}',
          PRIMARY KEY(profile_id, normalized_handle)
        );

        CREATE TABLE IF NOT EXISTS x_profile_identity_conflicts (
          id TEXT PRIMARY KEY,
          conflict_kind TEXT NOT NULL,
          handle TEXT NOT NULL,
          normalized_handle TEXT NOT NULL,
          existing_profile_id TEXT,
          incoming_profile_id TEXT,
          existing_x_user_id TEXT,
          incoming_x_user_id TEXT,
          source TEXT NOT NULL,
          raw_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS x_tweets (
          id TEXT PRIMARY KEY,
          x_id TEXT NOT NULL UNIQUE,
          author_profile_id TEXT,
          text TEXT NOT NULL,
          url TEXT NOT NULL,
          created_at TEXT,
          lang TEXT,
          conversation_id TEXT,
          reply_to_x_id TEXT,
          quote_x_id TEXT,
          retweet_x_id TEXT,
          metrics_json TEXT NOT NULL DEFAULT '{}',
          entities_json TEXT NOT NULL DEFAULT '{}',
          raw_json TEXT NOT NULL DEFAULT '{}',
          first_seen_at TEXT NOT NULL,
          last_seen_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS x_tweet_refs (
          tweet_x_id TEXT NOT NULL,
          ref_kind TEXT NOT NULL,
          ref_x_id TEXT NOT NULL,
          source TEXT NOT NULL,
          created_at TEXT NOT NULL,
          PRIMARY KEY(tweet_x_id, ref_kind, ref_x_id, source)
        );

        CREATE TABLE IF NOT EXISTS x_tweet_links (
          tweet_x_id TEXT NOT NULL,
          url TEXT NOT NULL,
          expanded_url TEXT,
          display_url TEXT,
          source TEXT NOT NULL,
          first_seen_at TEXT NOT NULL,
          last_seen_at TEXT NOT NULL,
          raw_json TEXT NOT NULL DEFAULT '{}',
          PRIMARY KEY(tweet_x_id, url, source)
        );

        CREATE TABLE IF NOT EXISTS x_link_expansions (
          url TEXT PRIMARY KEY,
          status TEXT NOT NULL,
          wiki_page_id TEXT,
          final_url TEXT,
          canonical_url TEXT,
          content_type TEXT,
          bytes INTEGER,
          last_error TEXT,
          first_attempted_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS x_tweet_edges (
          account_id TEXT NOT NULL,
          tweet_x_id TEXT NOT NULL,
          edge_kind TEXT NOT NULL,
          source_kind TEXT NOT NULL,
          source_detail TEXT NOT NULL DEFAULT '',
          transport TEXT NOT NULL,
          first_seen_at TEXT NOT NULL,
          last_seen_at TEXT NOT NULL,
          seen_count INTEGER NOT NULL DEFAULT 1,
          cursor_key TEXT,
          raw_json TEXT NOT NULL DEFAULT '{}',
          PRIMARY KEY(account_id, tweet_x_id, edge_kind, source_kind, source_detail)
        );

        CREATE TABLE IF NOT EXISTS x_collections (
          account_id TEXT NOT NULL,
          tweet_x_id TEXT NOT NULL,
          collection_kind TEXT NOT NULL,
          collected_at TEXT,
          source TEXT NOT NULL,
          first_seen_at TEXT NOT NULL,
          last_seen_at TEXT NOT NULL,
          raw_json TEXT NOT NULL DEFAULT '{}',
          PRIMARY KEY(account_id, tweet_x_id, collection_kind)
        );

        CREATE TABLE IF NOT EXISTS x_sync_runs (
          id TEXT PRIMARY KEY,
          account_id TEXT,
          stream TEXT NOT NULL,
          transport TEXT NOT NULL,
          status TEXT NOT NULL,
          started_at TEXT NOT NULL,
          completed_at TEXT,
          seen INTEGER NOT NULL DEFAULT 0,
          inserted INTEGER NOT NULL DEFAULT 0,
          updated INTEGER NOT NULL DEFAULT 0,
          skipped_duplicates INTEGER NOT NULL DEFAULT 0,
          rejected INTEGER NOT NULL DEFAULT 0,
          cursor_key TEXT,
          previous_cursor TEXT,
          new_cursor TEXT,
          error TEXT,
          metadata_json TEXT NOT NULL DEFAULT '{}'
        );

        CREATE TABLE IF NOT EXISTS x_projections (
          id TEXT PRIMARY KEY,
          entity_kind TEXT NOT NULL,
          entity_id TEXT NOT NULL,
          projection_kind TEXT NOT NULL,
          status TEXT NOT NULL,
          source_card_id TEXT,
          wiki_page_id TEXT,
          digest_candidate_id TEXT,
          last_error TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(entity_kind, entity_id, projection_kind)
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS x_tweets_fts
        USING fts5(x_id UNINDEXED, author_handle, text, url_text);

        CREATE INDEX IF NOT EXISTS idx_x_profiles_handle ON x_profiles(handle);
        CREATE INDEX IF NOT EXISTS idx_x_profile_aliases_handle ON x_profile_aliases(normalized_handle);
        CREATE INDEX IF NOT EXISTS idx_x_profile_identity_conflicts_created ON x_profile_identity_conflicts(created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_x_tweets_author_created ON x_tweets(author_profile_id, created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_x_tweets_created ON x_tweets(created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_x_tweets_conversation ON x_tweets(conversation_id, created_at ASC);
        CREATE INDEX IF NOT EXISTS idx_x_tweet_links_url ON x_tweet_links(url);
        CREATE INDEX IF NOT EXISTS idx_x_tweet_links_tweet ON x_tweet_links(tweet_x_id);
        CREATE INDEX IF NOT EXISTS idx_x_link_expansions_status ON x_link_expansions(status, updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_x_tweet_edges_tweet ON x_tweet_edges(tweet_x_id);
        CREATE INDEX IF NOT EXISTS idx_x_tweet_edges_kind ON x_tweet_edges(account_id, edge_kind, last_seen_at DESC);
        CREATE INDEX IF NOT EXISTS idx_x_collections_kind ON x_collections(account_id, collection_kind, last_seen_at DESC);
        CREATE INDEX IF NOT EXISTS idx_x_sync_runs_started ON x_sync_runs(started_at DESC);
        CREATE INDEX IF NOT EXISTS idx_x_projections_status ON x_projections(status, updated_at DESC);
        "#,
    )?;
    ensure_x_knowledge_schema_on(conn)?;
    Ok(())
}

pub(crate) fn ensure_x_knowledge_schema_on(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS x_knowledge_clusters (
          id TEXT PRIMARY KEY,
          topic TEXT NOT NULL,
          status TEXT NOT NULL,
          source_card_ids_json TEXT NOT NULL,
          radar_run_id TEXT,
          radar_item_ids_json TEXT NOT NULL DEFAULT '[]',
          first_seen_at TEXT NOT NULL,
          last_seen_at TEXT NOT NULL,
          novelty_score REAL NOT NULL DEFAULT 0,
          momentum_score REAL NOT NULL DEFAULT 0,
          stale_score REAL NOT NULL DEFAULT 0,
          reason TEXT NOT NULL,
          metadata_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(topic, source_card_ids_json)
        );

        CREATE INDEX IF NOT EXISTS idx_x_knowledge_clusters_status_updated
        ON x_knowledge_clusters(status, updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_x_knowledge_clusters_radar_run
        ON x_knowledge_clusters(radar_run_id);

        CREATE TABLE IF NOT EXISTS x_editorial_decisions (
          id TEXT PRIMARY KEY,
          cluster_id TEXT NOT NULL,
          decision TEXT NOT NULL,
          status TEXT NOT NULL,
          wiki_page_id TEXT,
          digest_candidate_id TEXT,
          source_card_ids_json TEXT NOT NULL,
          reason TEXT NOT NULL,
          quality_findings_json TEXT NOT NULL DEFAULT '[]',
          metadata_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(cluster_id, decision)
        );

        CREATE INDEX IF NOT EXISTS idx_x_editorial_decisions_cluster
        ON x_editorial_decisions(cluster_id, updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_x_editorial_decisions_status
        ON x_editorial_decisions(status, updated_at DESC);
        "#,
    )?;
    Ok(())
}

pub(crate) fn ensure_issue_schedule_schema_on(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS issue_schedules (
          id TEXT PRIMARY KEY,
          name TEXT NOT NULL,
          status TEXT NOT NULL,
          kind TEXT NOT NULL,
          channel TEXT NOT NULL,
          recipient_ref TEXT NOT NULL,
          time_zone TEXT NOT NULL,
          hour INTEGER NOT NULL,
          minute INTEGER NOT NULL,
          catch_up_hours INTEGER NOT NULL,
          metadata_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(kind, name)
        );

        CREATE TABLE IF NOT EXISTS issue_schedule_ticks (
          id TEXT PRIMARY KEY,
          schedule_id TEXT NOT NULL,
          tick_key TEXT NOT NULL UNIQUE,
          due_at TEXT NOT NULL,
          status TEXT NOT NULL,
          job_id TEXT,
          candidate_id TEXT,
          delivery_id TEXT,
          error TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          FOREIGN KEY(schedule_id) REFERENCES issue_schedules(id) ON DELETE CASCADE,
          FOREIGN KEY(job_id) REFERENCES wiki_jobs(id) ON DELETE SET NULL,
          FOREIGN KEY(candidate_id) REFERENCES digest_candidates(id) ON DELETE SET NULL,
          FOREIGN KEY(delivery_id) REFERENCES digest_deliveries(id) ON DELETE SET NULL
        );

        CREATE INDEX IF NOT EXISTS idx_issue_schedule_ticks_schedule_due
        ON issue_schedule_ticks(schedule_id, due_at);

        CREATE INDEX IF NOT EXISTS idx_issue_schedule_ticks_status
        ON issue_schedule_ticks(status);
        "#,
    )?;
    Ok(())
}

pub(crate) fn ensure_knowledge_schema_on(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS knowledge_events (
          id TEXT PRIMARY KEY,
          event_type TEXT NOT NULL,
          status TEXT NOT NULL,
          title TEXT NOT NULL,
          canonical_key TEXT NOT NULL,
          primary_entity_key TEXT,
          event_time TEXT,
          summary TEXT NOT NULL,
          first_seen_at TEXT NOT NULL,
          last_seen_at TEXT NOT NULL,
          confidence REAL NOT NULL DEFAULT 0,
          metadata_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(event_type, canonical_key)
        );

        CREATE INDEX IF NOT EXISTS idx_knowledge_events_status_updated
        ON knowledge_events(status, updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_knowledge_events_type_time
        ON knowledge_events(event_type, event_time DESC, last_seen_at DESC);

        CREATE TABLE IF NOT EXISTS knowledge_event_sources (
          id TEXT PRIMARY KEY,
          event_id TEXT NOT NULL,
          source_card_id TEXT NOT NULL,
          role TEXT NOT NULL,
          confidence REAL NOT NULL DEFAULT 0,
          claim_summary TEXT NOT NULL,
          metadata_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(event_id, source_card_id, role),
          FOREIGN KEY(event_id) REFERENCES knowledge_events(id) ON DELETE CASCADE,
          FOREIGN KEY(source_card_id) REFERENCES source_cards(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_knowledge_event_sources_event
        ON knowledge_event_sources(event_id, updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_knowledge_event_sources_source
        ON knowledge_event_sources(source_card_id, updated_at DESC);

        CREATE TABLE IF NOT EXISTS knowledge_clusters (
          id TEXT PRIMARY KEY,
          topic TEXT NOT NULL,
          status TEXT NOT NULL,
          source_card_ids_json TEXT NOT NULL,
          event_ids_json TEXT NOT NULL DEFAULT '[]',
          first_seen_at TEXT NOT NULL,
          last_seen_at TEXT NOT NULL,
          novelty_score REAL NOT NULL DEFAULT 0,
          momentum_score REAL NOT NULL DEFAULT 0,
          stale_score REAL NOT NULL DEFAULT 0,
          reason TEXT NOT NULL,
          duplicate_groups_json TEXT NOT NULL DEFAULT '{}',
          metadata_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(topic, source_card_ids_json)
        );

        CREATE INDEX IF NOT EXISTS idx_knowledge_clusters_status_updated
        ON knowledge_clusters(status, updated_at DESC);

        CREATE TABLE IF NOT EXISTS knowledge_editorial_decisions (
          id TEXT PRIMARY KEY,
          cluster_id TEXT NOT NULL,
          decision TEXT NOT NULL,
          status TEXT NOT NULL,
          wiki_page_id TEXT,
          digest_candidate_id TEXT,
          source_card_ids_json TEXT NOT NULL,
          reason TEXT NOT NULL,
          quality_findings_json TEXT NOT NULL DEFAULT '[]',
          metadata_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(cluster_id, decision),
          FOREIGN KEY(cluster_id) REFERENCES knowledge_clusters(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_knowledge_editorial_cluster
        ON knowledge_editorial_decisions(cluster_id, updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_knowledge_editorial_status
        ON knowledge_editorial_decisions(status, updated_at DESC);

        CREATE TABLE IF NOT EXISTS knowledge_reports (
          id TEXT PRIMARY KEY,
          cluster_id TEXT NOT NULL,
          title TEXT NOT NULL,
          body_markdown TEXT NOT NULL,
          status TEXT NOT NULL,
          source_card_ids_json TEXT NOT NULL,
          quality_findings_json TEXT NOT NULL DEFAULT '[]',
          metadata_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(cluster_id, title),
          FOREIGN KEY(cluster_id) REFERENCES knowledge_clusters(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_knowledge_reports_cluster
        ON knowledge_reports(cluster_id, updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_knowledge_reports_status
        ON knowledge_reports(status, updated_at DESC);

        CREATE TABLE IF NOT EXISTS knowledge_entities (
          id TEXT PRIMARY KEY,
          entity_type TEXT NOT NULL,
          name TEXT NOT NULL,
          canonical_key TEXT NOT NULL UNIQUE,
          aliases_json TEXT NOT NULL DEFAULT '[]',
          homepage_url TEXT,
          source_card_ids_json TEXT NOT NULL DEFAULT '[]',
          wiki_page_id TEXT,
          confidence REAL NOT NULL DEFAULT 0,
          metadata_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_knowledge_entities_type_updated
        ON knowledge_entities(entity_type, updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_knowledge_entities_name
        ON knowledge_entities(name);

        CREATE TABLE IF NOT EXISTS knowledge_relations (
          id TEXT PRIMARY KEY,
          relation_key TEXT NOT NULL UNIQUE,
          relation_type TEXT NOT NULL,
          subject_entity_id TEXT NOT NULL,
          object_entity_id TEXT NOT NULL,
          event_id TEXT,
          cluster_id TEXT,
          source_card_ids_json TEXT NOT NULL DEFAULT '[]',
          confidence REAL NOT NULL DEFAULT 0,
          reason TEXT NOT NULL,
          metadata_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          FOREIGN KEY(subject_entity_id) REFERENCES knowledge_entities(id) ON DELETE CASCADE,
          FOREIGN KEY(object_entity_id) REFERENCES knowledge_entities(id) ON DELETE CASCADE,
          FOREIGN KEY(event_id) REFERENCES knowledge_events(id) ON DELETE SET NULL,
          FOREIGN KEY(cluster_id) REFERENCES knowledge_clusters(id) ON DELETE SET NULL
        );

        CREATE INDEX IF NOT EXISTS idx_knowledge_relations_subject
        ON knowledge_relations(subject_entity_id, updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_knowledge_relations_object
        ON knowledge_relations(object_entity_id, updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_knowledge_relations_type
        ON knowledge_relations(relation_type, updated_at DESC);

        CREATE TABLE IF NOT EXISTS knowledge_adapter_runs (
          id TEXT PRIMARY KEY,
          job_id TEXT NOT NULL UNIQUE,
          adapter_kind TEXT NOT NULL,
          provider TEXT NOT NULL,
          source_kind TEXT NOT NULL,
          locator TEXT NOT NULL,
          status TEXT NOT NULL,
          error_kind TEXT,
          error TEXT,
          cursor_key TEXT,
          cursor_before TEXT,
          cursor_after TEXT,
          source_card_ids_json TEXT NOT NULL DEFAULT '[]',
          raw_count INTEGER NOT NULL DEFAULT 0,
          accepted_count INTEGER NOT NULL DEFAULT 0,
          rejected_count INTEGER NOT NULL DEFAULT 0,
          duplicate_count INTEGER NOT NULL DEFAULT 0,
          metadata_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          FOREIGN KEY(job_id) REFERENCES wiki_jobs(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_knowledge_adapter_runs_status
        ON knowledge_adapter_runs(status, updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_knowledge_adapter_runs_provider
        ON knowledge_adapter_runs(provider, source_kind, updated_at DESC);

        CREATE TABLE IF NOT EXISTS knowledge_entity_resolutions (
          id TEXT PRIMARY KEY,
          left_entity_id TEXT NOT NULL,
          right_entity_id TEXT NOT NULL,
          status TEXT NOT NULL,
          decision TEXT NOT NULL,
          confidence REAL NOT NULL DEFAULT 0,
          resolver TEXT NOT NULL,
          reason TEXT NOT NULL,
          evidence_json TEXT NOT NULL DEFAULT '{}',
          source_card_ids_json TEXT NOT NULL DEFAULT '[]',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          UNIQUE(left_entity_id, right_entity_id, resolver),
          FOREIGN KEY(left_entity_id) REFERENCES knowledge_entities(id) ON DELETE CASCADE,
          FOREIGN KEY(right_entity_id) REFERENCES knowledge_entities(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_knowledge_entity_resolutions_status
        ON knowledge_entity_resolutions(status, updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_knowledge_entity_resolutions_decision
        ON knowledge_entity_resolutions(decision, confidence DESC);
        "#,
    )?;
    Ok(())
}

pub(crate) fn ensure_x_watch_curation_schema_on(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS x_watch_manual_rules (
          handle TEXT PRIMARY KEY,
          decision TEXT NOT NULL,
          category TEXT NOT NULL,
          reason TEXT NOT NULL,
          metadata_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS x_watch_curation_runs (
          id TEXT PRIMARY KEY,
          classifier_version TEXT NOT NULL,
          mode TEXT NOT NULL,
          status TEXT NOT NULL,
          input_count INTEGER NOT NULL DEFAULT 0,
          keep_count INTEGER NOT NULL DEFAULT 0,
          review_keep_leaning_count INTEGER NOT NULL DEFAULT 0,
          review_drop_leaning_count INTEGER NOT NULL DEFAULT 0,
          needs_profile_enrichment_count INTEGER NOT NULL DEFAULT 0,
          pause_candidate_count INTEGER NOT NULL DEFAULT 0,
          paused_count INTEGER NOT NULL DEFAULT 0,
          restored_count INTEGER NOT NULL DEFAULT 0,
          error TEXT,
          metadata_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          completed_at TEXT
        );

        CREATE TABLE IF NOT EXISTS x_watch_curation_decisions (
          id TEXT PRIMARY KEY,
          run_id TEXT NOT NULL,
          watch_source_id TEXT NOT NULL,
          handle TEXT NOT NULL,
          previous_status TEXT NOT NULL,
          proposed_status TEXT NOT NULL,
          recommendation TEXT NOT NULL,
          category TEXT NOT NULL,
          score INTEGER NOT NULL DEFAULT 0,
          confidence REAL NOT NULL DEFAULT 0,
          reason TEXT NOT NULL,
          evidence_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          applied_at TEXT,
          FOREIGN KEY(run_id) REFERENCES x_watch_curation_runs(id) ON DELETE CASCADE,
          FOREIGN KEY(watch_source_id) REFERENCES watch_sources(id) ON DELETE CASCADE,
          UNIQUE(run_id, watch_source_id)
        );

        CREATE INDEX IF NOT EXISTS idx_x_watch_curation_decisions_run
        ON x_watch_curation_decisions(run_id, recommendation, score DESC);
        CREATE INDEX IF NOT EXISTS idx_x_watch_curation_decisions_handle
        ON x_watch_curation_decisions(handle, created_at DESC);

        CREATE TABLE IF NOT EXISTS x_watch_curation_evidence (
          id TEXT PRIMARY KEY,
          decision_id TEXT NOT NULL,
          evidence_kind TEXT NOT NULL,
          evidence_value TEXT NOT NULL,
          weight INTEGER NOT NULL DEFAULT 0,
          created_at TEXT NOT NULL,
          FOREIGN KEY(decision_id) REFERENCES x_watch_curation_decisions(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_x_watch_curation_evidence_decision
        ON x_watch_curation_evidence(decision_id);

        CREATE TABLE IF NOT EXISTS x_watch_restore_snapshots (
          run_id TEXT NOT NULL,
          watch_source_id TEXT NOT NULL,
          previous_status TEXT NOT NULL,
          previous_label TEXT NOT NULL,
          previous_cadence TEXT NOT NULL,
          previous_metadata_json TEXT NOT NULL DEFAULT '{}',
          restored_at TEXT,
          created_at TEXT NOT NULL,
          PRIMARY KEY(run_id, watch_source_id),
          FOREIGN KEY(run_id) REFERENCES x_watch_curation_runs(id) ON DELETE CASCADE,
          FOREIGN KEY(watch_source_id) REFERENCES watch_sources(id) ON DELETE CASCADE
        );
        "#,
    )?;
    Ok(())
}

pub(crate) fn ensure_column_on(
    conn: &Connection,
    table: &str,
    column: &str,
    alter_sql: &str,
) -> Result<()> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let columns = rows(stmt.query_map([], |row| row.get::<_, String>(1))?)?;
    if !columns.iter().any(|existing| existing == column) {
        conn.execute(alter_sql, [])?;
    }
    Ok(())
}

pub(crate) fn table_columns_on(conn: &Connection, table: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    rows(stmt.query_map([], |row| row.get::<_, String>(1))?)
}
