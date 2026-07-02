use super::*;

pub(crate) fn backfill_x_canonical_from_compatibility_on(conn: &Connection) -> Result<()> {
    ensure_x_default_account_on(conn)?;
    let mut stmt = conn.prepare(
        r#"
        SELECT id, x_id, author, text, url, created_at, imported_at, retrieved_at,
               metrics_json, raw_json, source_card_id, wiki_page_id
        FROM x_items
        ORDER BY imported_at ASC
        "#,
    )?;
    let items = rows(stmt.query_map([], x_item_from_row)?)?;
    for item in items {
        let mut input = XItemInput {
            x_id: item.x_id.clone(),
            author: item.author.clone(),
            text: item.text.clone(),
            url: item.url.clone(),
            created_at: item.created_at.clone(),
            conversation_id: None,
            reply_to_x_id: None,
            quote_x_id: None,
            retweet_x_id: None,
            retrieved_at: item.retrieved_at.clone().or(Some(item.imported_at.clone())),
            metrics: item.metrics.clone(),
            raw: item.raw.clone(),
            source_kind: "json_import".to_string(),
            source_detail: None,
            source_metadata: json!({ "backfilled_from": "x_items" }),
        };
        upsert_x_canonical_on(
            conn,
            &input,
            item.source_card_id.as_deref(),
            item.wiki_page_id.as_deref(),
        )?;
        let mut source_stmt = conn.prepare(
            r#"
            SELECT id, x_id, source_kind, source_detail, seen_at, metadata_json
            FROM x_item_sources
            WHERE x_id = ?1
            ORDER BY seen_at ASC
            "#,
        )?;
        let sources = rows(source_stmt.query_map(params![item.x_id], x_item_source_from_row)?)?;
        for source in sources {
            input.source_kind = source.source_kind;
            input.source_detail = source.source_detail;
            input.retrieved_at = Some(source.seen_at);
            input.source_metadata = source.metadata;
            upsert_x_canonical_edge_on(conn, &input)?;
        }
    }
    Ok(())
}

pub(crate) fn ensure_x_default_account_on(conn: &Connection) -> Result<()> {
    let now = now();
    conn.execute(
        r#"
        INSERT INTO x_accounts
          (id, x_user_id, handle, display_name, is_default, preferred_transport, metadata_json, created_at, updated_at)
        VALUES ('acct_default', NULL, 'default', 'Default X Account', 1, 'x_api', '{"synthetic":true}', ?1, ?1)
        ON CONFLICT(id) DO UPDATE SET
          updated_at = excluded.updated_at,
          is_default = 1
        "#,
        params![now],
    )?;
    Ok(())
}

pub(crate) fn upsert_x_canonical_on(
    conn: &Connection,
    input: &XItemInput,
    source_card_id: Option<&str>,
    wiki_page_id: Option<&str>,
) -> Result<()> {
    ensure_x_default_account_on(conn)?;
    let seen_at = input.retrieved_at.clone().unwrap_or_else(now);
    let now_value = now();
    let x_author_id = input
        .source_metadata
        .get("x_author_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty());
    let profile_id = resolve_x_profile_id_on(conn, &input.author, x_author_id, input)?;
    let metrics_json = canonical_json(&input.metrics)?;
    let raw_json = canonical_json(&input.raw)?;
    let entities_json = canonical_json(&x_item_entities(input))?;
    let profile_raw = input
        .source_metadata
        .as_object()
        .map(|metadata| {
            json!({
                "name": metadata.get("author_name").cloned(),
                "description": metadata.get("author_description").cloned(),
                "verified": metadata.get("verified").cloned(),
                "verified_type": metadata.get("verified_type").cloned(),
                "x_author_id": metadata.get("x_author_id").cloned()
            })
        })
        .unwrap_or_else(|| json!({}));
    let profile_raw_json = canonical_json(&profile_raw)?;
    let display_name = input
        .source_metadata
        .get("author_name")
        .and_then(Value::as_str)
        .unwrap_or(&input.author);
    let description = input
        .source_metadata
        .get("author_description")
        .and_then(Value::as_str)
        .unwrap_or("");
    conn.execute(
        r#"
        INSERT INTO x_profiles
          (id, x_user_id, handle, display_name, description, raw_json, first_seen_at, last_seen_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, ?8)
        ON CONFLICT(id) DO UPDATE SET
          handle = excluded.handle,
          x_user_id = COALESCE(x_profiles.x_user_id, excluded.x_user_id),
          display_name = CASE WHEN excluded.display_name != '' THEN excluded.display_name ELSE x_profiles.display_name END,
          description = CASE WHEN excluded.description != '' THEN excluded.description ELSE x_profiles.description END,
          raw_json = CASE WHEN excluded.raw_json != '{}' THEN excluded.raw_json ELSE x_profiles.raw_json END,
          last_seen_at = excluded.last_seen_at,
          updated_at = excluded.updated_at
        "#,
        params![
            profile_id,
            x_author_id,
            input.author,
            display_name,
            description,
            profile_raw_json,
            seen_at,
            now_value
        ],
    )?;
    upsert_x_profile_alias_on(
        conn,
        &profile_id,
        &input.author,
        x_author_id,
        &input.source_kind,
        &seen_at,
        &profile_raw_json,
    )?;
    let snapshot_hash = sha256(
        format!(
            "{}\n{}\n{}\n{}",
            input.author, display_name, description, profile_raw
        )
        .as_bytes(),
    );
    conn.execute(
        r#"
        INSERT INTO x_profile_snapshots
          (profile_id, snapshot_hash, observed_at, last_seen_at, source, handle, display_name, description, raw_json)
        VALUES (?1, ?2, ?3, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT(profile_id, snapshot_hash) DO UPDATE SET
          last_seen_at = excluded.last_seen_at
        "#,
        params![
            profile_id,
            snapshot_hash,
            seen_at,
            input.source_kind,
            input.author,
            display_name,
            description,
            profile_raw_json
        ],
    )?;
    conn.execute(
        r#"
        INSERT INTO x_tweets
          (id, x_id, author_profile_id, text, url, created_at, conversation_id, reply_to_x_id,
           quote_x_id, retweet_x_id, metrics_json, entities_json, raw_json, first_seen_at, last_seen_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?14, ?15)
        ON CONFLICT(x_id) DO UPDATE SET
          author_profile_id = COALESCE(excluded.author_profile_id, x_tweets.author_profile_id),
          text = CASE WHEN x_tweets.text = '' THEN excluded.text ELSE x_tweets.text END,
          url = CASE WHEN x_tweets.url = '' THEN excluded.url ELSE x_tweets.url END,
          created_at = COALESCE(x_tweets.created_at, excluded.created_at),
          conversation_id = COALESCE(x_tweets.conversation_id, excluded.conversation_id),
          reply_to_x_id = COALESCE(x_tweets.reply_to_x_id, excluded.reply_to_x_id),
          quote_x_id = COALESCE(x_tweets.quote_x_id, excluded.quote_x_id),
          retweet_x_id = COALESCE(x_tweets.retweet_x_id, excluded.retweet_x_id),
          metrics_json = CASE WHEN excluded.metrics_json != '{}' THEN excluded.metrics_json ELSE x_tweets.metrics_json END,
          entities_json = CASE WHEN excluded.entities_json != '{}' THEN excluded.entities_json ELSE x_tweets.entities_json END,
          raw_json = CASE WHEN excluded.raw_json != '{}' THEN excluded.raw_json ELSE x_tweets.raw_json END,
          last_seen_at = excluded.last_seen_at,
          updated_at = excluded.updated_at
        "#,
        params![
            format!("xtweet-{}", &sha256(input.x_id.as_bytes())[..32]),
            input.x_id,
            profile_id,
            input.text,
            input.url,
            input.created_at,
            input.conversation_id,
            input.reply_to_x_id,
            input.quote_x_id,
            input.retweet_x_id,
            metrics_json,
            entities_json,
            raw_json,
            seen_at,
            now_value
        ],
    )?;
    upsert_x_canonical_edge_on(conn, input)?;
    if input.source_kind == "bookmark" {
        conn.execute(
            r#"
            INSERT INTO x_collections
              (account_id, tweet_x_id, collection_kind, collected_at, source, first_seen_at, last_seen_at, raw_json)
            VALUES ('acct_default', ?1, 'bookmark', ?2, ?3, ?2, ?2, ?4)
            ON CONFLICT(account_id, tweet_x_id, collection_kind) DO UPDATE SET
              last_seen_at = excluded.last_seen_at,
              raw_json = excluded.raw_json
            "#,
            params![input.x_id, seen_at, input.source_kind, raw_json],
        )?;
    }
    if source_card_id.is_some() || wiki_page_id.is_some() {
        upsert_x_projection_on(conn, &input.x_id, source_card_id, wiki_page_id)?;
    }
    upsert_x_tweets_fts_on(conn, &input.x_id)?;
    Ok(())
}

pub(crate) fn upsert_x_canonical_edge_on(conn: &Connection, input: &XItemInput) -> Result<()> {
    let seen_at = input.retrieved_at.clone().unwrap_or_else(now);
    let metadata_json = canonical_json(&input.source_metadata)?;
    let transport = x_edge_transport(input);
    conn.execute(
        r#"
        INSERT INTO x_tweet_edges
          (account_id, tweet_x_id, edge_kind, source_kind, source_detail, transport, first_seen_at, last_seen_at, seen_count, raw_json)
        VALUES ('acct_default', ?1, ?2, ?2, ?3, ?4, ?5, ?5, 1, ?6)
        ON CONFLICT(account_id, tweet_x_id, edge_kind, source_kind, source_detail) DO UPDATE SET
          last_seen_at = excluded.last_seen_at,
          transport = excluded.transport,
          seen_count = x_tweet_edges.seen_count + 1,
          raw_json = excluded.raw_json
        "#,
        params![
            input.x_id,
            input.source_kind,
            input.source_detail.clone().unwrap_or_default(),
            transport,
            seen_at,
            metadata_json
        ],
    )?;
    upsert_x_tweet_refs_on(conn, input)?;
    Ok(())
}

fn x_edge_transport(input: &XItemInput) -> &'static str {
    match input
        .source_metadata
        .get("imported_from")
        .and_then(Value::as_str)
    {
        Some("x_api_mcp") => "x_api_mcp",
        Some("xurl_token_api") => "xurl_token_api",
        _ => "x_api",
    }
}

pub(crate) fn upsert_x_tweet_refs_on(conn: &Connection, input: &XItemInput) -> Result<()> {
    let seen_at = input.retrieved_at.clone().unwrap_or_else(now);
    let mut refs: Vec<(&str, &String)> = Vec::new();
    if let Some(conversation_id) = &input.conversation_id {
        refs.push(("conversation", conversation_id));
    }
    if let Some(reply_to_x_id) = &input.reply_to_x_id {
        refs.push(("reply_to", reply_to_x_id));
    }
    if let Some(quote_x_id) = &input.quote_x_id {
        refs.push(("quote", quote_x_id));
    }
    if let Some(retweet_x_id) = &input.retweet_x_id {
        refs.push(("retweet", retweet_x_id));
    }
    for (ref_kind, ref_x_id) in refs {
        conn.execute(
            r#"
            INSERT INTO x_tweet_refs (tweet_x_id, ref_kind, ref_x_id, source, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(tweet_x_id, ref_kind, ref_x_id, source) DO NOTHING
            "#,
            params![input.x_id, ref_kind, ref_x_id, input.source_kind, seen_at],
        )?;
    }
    Ok(())
}

pub(crate) fn upsert_x_projection_on(
    conn: &Connection,
    x_id: &str,
    source_card_id: Option<&str>,
    wiki_page_id: Option<&str>,
) -> Result<()> {
    let id = format!(
        "xproj-{}",
        &sha256(format!("tweet\n{x_id}\nsource_card").as_bytes())[..32]
    );
    let now_value = now();
    conn.execute(
        r#"
        INSERT INTO x_projections
          (id, entity_kind, entity_id, projection_kind, status, source_card_id, wiki_page_id, last_error, created_at, updated_at)
        VALUES (?1, 'tweet', ?2, 'source_card', ?3, ?4, ?5, NULL, ?6, ?6)
        ON CONFLICT(entity_kind, entity_id, projection_kind) DO UPDATE SET
          status = excluded.status,
          source_card_id = COALESCE(excluded.source_card_id, x_projections.source_card_id),
          wiki_page_id = COALESCE(excluded.wiki_page_id, x_projections.wiki_page_id),
          last_error = NULL,
          updated_at = excluded.updated_at
        "#,
        params![
            id,
            x_id,
            if source_card_id.is_some() { "completed" } else { "pending" },
            source_card_id,
            wiki_page_id,
            now_value
        ],
    )?;
    Ok(())
}

pub(crate) fn upsert_x_tweets_fts_on(conn: &Connection, x_id: &str) -> Result<()> {
    conn.execute("DELETE FROM x_tweets_fts WHERE x_id = ?1", params![x_id])?;
    conn.execute(
        r#"
        INSERT INTO x_tweets_fts (x_id, author_handle, text, url_text)
        SELECT t.x_id, COALESCE(p.handle, ''), t.text, t.url
        FROM x_tweets t
        LEFT JOIN x_profiles p ON p.id = t.author_profile_id
        WHERE t.x_id = ?1
        "#,
        params![x_id],
    )?;
    Ok(())
}

pub(crate) fn rebuild_x_tweets_fts_on(conn: &Connection) -> Result<usize> {
    conn.execute("DELETE FROM x_tweets_fts", [])?;
    let inserted = conn.execute(
        r#"
        INSERT INTO x_tweets_fts (x_id, author_handle, text, url_text)
        SELECT t.x_id, COALESCE(p.handle, ''), t.text, t.url
        FROM x_tweets t
        LEFT JOIN x_profiles p ON p.id = t.author_profile_id
        "#,
        [],
    )?;
    Ok(inserted)
}

pub(crate) fn resolve_x_profile_id_on(
    conn: &Connection,
    handle: &str,
    x_author_id: Option<&str>,
    input: &XItemInput,
) -> Result<String> {
    let normalized_handle = normalize_x_handle_for_identity(handle);
    if let Some(x_user_id) = x_author_id {
        if let Some(existing_profile_id) = conn
            .query_row(
                "SELECT id FROM x_profiles WHERE x_user_id = ?1",
                params![x_user_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
        {
            return Ok(existing_profile_id);
        }

        let handle_profile_id = x_profile_id(handle);
        if let Some((existing_profile_id, existing_x_user_id)) = conn
            .query_row(
                "SELECT id, x_user_id FROM x_profiles WHERE id = ?1",
                params![handle_profile_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)),
            )
            .optional()?
        {
            if let Some(existing_x_user_id) = existing_x_user_id
                && existing_x_user_id != x_user_id
            {
                record_x_profile_identity_conflict_on(
                    conn,
                    "handle_reuse",
                    handle,
                    &normalized_handle,
                    Some(&existing_profile_id),
                    Some(&x_profile_user_id(x_user_id)),
                    Some(&existing_x_user_id),
                    Some(x_user_id),
                    input,
                )?;
                bail!(
                    "X profile identity conflict for handle @{normalized_handle}: existing user id differs from incoming user id"
                );
            }
            return Ok(existing_profile_id);
        }
        if let Some((existing_profile_id, existing_x_user_id)) = conn
            .query_row(
                r#"
                SELECT id, x_user_id FROM x_profiles
                WHERE lower(handle) = ?1 AND x_user_id IS NOT NULL
                UNION
                SELECT profile_id, x_user_id FROM x_profile_aliases
                WHERE normalized_handle = ?1 AND x_user_id IS NOT NULL
                LIMIT 1
                "#,
                params![normalized_handle],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()?
        {
            if existing_x_user_id != x_user_id {
                record_x_profile_identity_conflict_on(
                    conn,
                    "handle_reuse",
                    handle,
                    &normalized_handle,
                    Some(&existing_profile_id),
                    Some(&x_profile_user_id(x_user_id)),
                    Some(&existing_x_user_id),
                    Some(x_user_id),
                    input,
                )?;
                bail!(
                    "X profile identity conflict for handle @{normalized_handle}: existing user id differs from incoming user id"
                );
            }
            return Ok(existing_profile_id);
        }

        return Ok(x_profile_user_id(x_user_id));
    }
    Ok(x_profile_id(handle))
}

pub(crate) fn upsert_x_profile_record_on(
    conn: &Connection,
    user: &Value,
    source: &str,
    observed_at: &str,
) -> Result<String> {
    let x_user_id = user
        .get("id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .context("X profile response missing id")?;
    let handle = user
        .get("username")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .context("X profile response missing username")?;
    validate_x_handle(handle)?;
    let display_name = user.get("name").and_then(Value::as_str).unwrap_or(handle);
    let description = user
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("");
    let raw_json = canonical_json(user)?;
    let conflict_input = XItemInput {
        x_id: format!("profile:{x_user_id}:{handle}"),
        author: handle.to_string(),
        text: String::new(),
        url: format!("https://x.com/{}", handle.trim_start_matches('@')),
        created_at: None,
        conversation_id: None,
        reply_to_x_id: None,
        quote_x_id: None,
        retweet_x_id: None,
        retrieved_at: Some(observed_at.to_string()),
        metrics: json!({}),
        raw: user.clone(),
        source_kind: source.to_string(),
        source_detail: Some("profile_lookup".to_string()),
        source_metadata: json!({
            "x_author_id": x_user_id,
            "author_name": display_name,
            "author_description": description,
            "verified": user.get("verified").cloned(),
            "verified_type": user.get("verified_type").cloned()
        }),
    };
    let profile_id = resolve_x_profile_id_on(conn, handle, Some(x_user_id), &conflict_input)?;
    conn.execute(
        r#"
        INSERT INTO x_profiles
          (id, x_user_id, handle, display_name, description, raw_json, first_seen_at, last_seen_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, ?7)
        ON CONFLICT(id) DO UPDATE SET
          handle = excluded.handle,
          x_user_id = COALESCE(x_profiles.x_user_id, excluded.x_user_id),
          display_name = CASE WHEN excluded.display_name != '' THEN excluded.display_name ELSE x_profiles.display_name END,
          description = CASE WHEN excluded.description != '' THEN excluded.description ELSE x_profiles.description END,
          raw_json = CASE WHEN excluded.raw_json != '{}' THEN excluded.raw_json ELSE x_profiles.raw_json END,
          last_seen_at = excluded.last_seen_at,
          updated_at = excluded.updated_at
        "#,
        params![
            profile_id,
            x_user_id,
            handle.trim_start_matches('@'),
            display_name,
            description,
            raw_json,
            observed_at,
        ],
    )?;
    upsert_x_profile_alias_on(
        conn,
        &profile_id,
        handle,
        Some(x_user_id),
        source,
        observed_at,
        &raw_json,
    )?;
    let snapshot_hash = sha256(
        format!(
            "{}\n{}\n{}\n{}",
            handle.trim_start_matches('@'),
            display_name,
            description,
            raw_json
        )
        .as_bytes(),
    );
    conn.execute(
        r#"
        INSERT INTO x_profile_snapshots
          (profile_id, snapshot_hash, observed_at, last_seen_at, source, handle, display_name, description, raw_json)
        VALUES (?1, ?2, ?3, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT(profile_id, snapshot_hash) DO UPDATE SET
          last_seen_at = excluded.last_seen_at
        "#,
        params![
            profile_id,
            snapshot_hash,
            observed_at,
            source,
            handle.trim_start_matches('@'),
            display_name,
            description,
            raw_json,
        ],
    )?;
    Ok(profile_id)
}

pub(crate) fn upsert_x_profile_alias_on(
    conn: &Connection,
    profile_id: &str,
    handle: &str,
    x_user_id: Option<&str>,
    source: &str,
    seen_at: &str,
    raw_json: &str,
) -> Result<()> {
    let normalized_handle = normalize_x_handle_for_identity(handle);
    conn.execute(
        "UPDATE x_profile_aliases SET is_current = 0 WHERE profile_id = ?1 AND normalized_handle != ?2",
        params![profile_id, normalized_handle],
    )?;
    conn.execute(
        r#"
        INSERT INTO x_profile_aliases
          (profile_id, handle, normalized_handle, x_user_id, source, first_seen_at, last_seen_at, is_current, raw_json)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, 1, ?7)
        ON CONFLICT(profile_id, normalized_handle) DO UPDATE SET
          handle = excluded.handle,
          x_user_id = COALESCE(x_profile_aliases.x_user_id, excluded.x_user_id),
          source = excluded.source,
          last_seen_at = excluded.last_seen_at,
          is_current = 1,
          raw_json = excluded.raw_json
        "#,
        params![
            profile_id,
            handle.trim_start_matches('@'),
            normalized_handle,
            x_user_id,
            source,
            seen_at,
            raw_json
        ],
    )?;
    Ok(())
}

// allow: refactoring this N-arg signature is out of scope for the lint-cleanup pass.
#[allow(clippy::too_many_arguments)]
pub(crate) fn record_x_profile_identity_conflict_on(
    conn: &Connection,
    conflict_kind: &str,
    handle: &str,
    normalized_handle: &str,
    existing_profile_id: Option<&str>,
    incoming_profile_id: Option<&str>,
    existing_x_user_id: Option<&str>,
    incoming_x_user_id: Option<&str>,
    input: &XItemInput,
) -> Result<()> {
    let created_at = now();
    let raw_json = canonical_json(&json!({
        "tweet_x_id": input.x_id,
        "source_kind": input.source_kind,
        "source_detail": input.source_detail,
        "source_metadata": input.source_metadata,
    }))?;
    let stable = format!(
        "{conflict_kind}\n{normalized_handle}\n{}\n{}\n{}",
        existing_x_user_id.unwrap_or(""),
        incoming_x_user_id.unwrap_or(""),
        input.x_id
    );
    conn.execute(
        r#"
        INSERT OR IGNORE INTO x_profile_identity_conflicts
          (id, conflict_kind, handle, normalized_handle, existing_profile_id,
           incoming_profile_id, existing_x_user_id, incoming_x_user_id, source,
           raw_json, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        "#,
        params![
            format!("xconf-{}", &sha256(stable.as_bytes())[..32]),
            conflict_kind,
            handle.trim_start_matches('@'),
            normalized_handle,
            existing_profile_id,
            incoming_profile_id,
            existing_x_user_id,
            incoming_x_user_id,
            input.source_kind,
            raw_json,
            created_at
        ],
    )?;
    Ok(())
}

pub(crate) fn x_profile_id(handle: &str) -> String {
    let normalized = normalize_x_handle_for_identity(handle);
    let hash = sha256(normalized.as_bytes());
    format!("xprof-{}", &hash[..32])
}

pub(crate) fn x_profile_user_id(x_user_id: &str) -> String {
    let hash = sha256(format!("user:{x_user_id}").as_bytes());
    format!("xprof-{}", &hash[..32])
}

pub(crate) fn normalize_x_handle_for_identity(handle: &str) -> String {
    handle.trim().trim_start_matches('@').to_ascii_lowercase()
}
