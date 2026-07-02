use super::*;

pub(crate) fn x_author_from_tweet_url(raw: &str) -> Option<String> {
    let url = Url::parse(raw).ok()?;
    let host = url.host_str()?.to_ascii_lowercase();
    if host != "x.com" && host != "twitter.com" && host != "mobile.twitter.com" {
        return None;
    }
    let first = url.path_segments()?.next()?;
    if first.is_empty() || first == "i" {
        return None;
    }
    Some(first.trim_start_matches('@').to_string())
}

pub(crate) fn parse_x_item_input(value: &Value) -> Result<XItemInput> {
    let object = value.as_object().context("x item must be an object")?;
    let x_id = first_string(object, &["x_id", "id", "tweet_id"])
        .context("x item missing id")?
        .to_string();
    let author = first_string(object, &["author", "username", "handle"])
        .unwrap_or("unknown")
        .trim_start_matches('@')
        .to_string();
    let text = first_string(object, &["text", "body", "content"])
        .context("x item missing text")?
        .to_string();
    let url = first_string(object, &["url", "link"])
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("https://x.com/{author}/status/{x_id}"));
    let created_at = first_string(object, &["created_at", "date"]).map(ToOwned::to_owned);
    let conversation_id = first_string(object, &["conversation_id"]).map(ToOwned::to_owned);
    let reply_to_x_id = first_string(
        object,
        &["reply_to_x_id", "in_reply_to_x_id", "in_reply_to_status_id"],
    )
    .map(ToOwned::to_owned)
    .or_else(|| referenced_tweet_id(value, "replied_to"));
    let quote_x_id = first_string(object, &["quote_x_id", "quoted_tweet_id"])
        .map(ToOwned::to_owned)
        .or_else(|| referenced_tweet_id(value, "quoted"));
    let retweet_x_id = first_string(object, &["retweet_x_id", "retweeted_tweet_id"])
        .map(ToOwned::to_owned)
        .or_else(|| referenced_tweet_id(value, "retweeted"));
    let metrics = object
        .get("metrics")
        .or_else(|| object.get("public_metrics"))
        .or_else(|| object.get("metrics_json"))
        .cloned()
        .unwrap_or_else(|| json!({}));
    let raw = object
        .get("raw")
        .or_else(|| object.get("raw_json"))
        .cloned()
        .unwrap_or_else(|| value.clone());
    let source_kind = first_string(object, &["source_kind", "source"])
        .unwrap_or("json_import")
        .to_string();
    let source_detail =
        first_string(object, &["source_detail", "source_label"]).map(ToOwned::to_owned);
    let source_metadata = object
        .get("source_metadata")
        .or_else(|| object.get("provenance"))
        .cloned()
        .unwrap_or_else(|| json!({}));
    let retrieved_at = first_string(object, &["retrieved_at", "seen_at"]).map(ToOwned::to_owned);
    Ok(XItemInput {
        x_id,
        author,
        text,
        url,
        created_at,
        conversation_id,
        reply_to_x_id,
        quote_x_id,
        retweet_x_id,
        retrieved_at,
        metrics,
        raw,
        source_kind,
        source_detail,
        source_metadata,
    })
}

pub(crate) fn validate_x_item_input(input: &XItemInput) -> Result<()> {
    validate_key(&input.x_id)?;
    validate_key(&input.author)?;
    validate_optional_x_ref("conversation_id", input.conversation_id.as_deref())?;
    validate_optional_x_ref("reply_to_x_id", input.reply_to_x_id.as_deref())?;
    validate_optional_x_ref("quote_x_id", input.quote_x_id.as_deref())?;
    validate_optional_x_ref("retweet_x_id", input.retweet_x_id.as_deref())?;
    validate_notes(&input.text)?;
    validate_public_http_url(&input.url)?;
    validate_x_item_source_kind(&input.source_kind)?;
    Ok(())
}

pub(crate) fn validate_optional_x_ref(label: &str, value: Option<&str>) -> Result<()> {
    if let Some(value) = value {
        validate_key(value).with_context(|| format!("invalid X {label}"))?;
    }
    Ok(())
}

pub(crate) fn validate_x_item_source_kind(source_kind: &str) -> Result<()> {
    match source_kind {
        "archive" | "archive_like" | "bookmark" | "json_import" | "portable_import"
        | "recent_search" | "watch_monitor" => Ok(()),
        other => bail!("unsupported X item source kind: {other}"),
    }
}

pub(crate) fn x_item_source_id(
    x_id: &str,
    source_kind: &str,
    source_detail: Option<&str>,
) -> String {
    let hash = sha256(format!("{x_id}\n{source_kind}\n{}", source_detail.unwrap_or("")).as_bytes());
    format!("xsrc-{}", &hash[..32])
}

pub(crate) fn x_following_user_to_watch_source(user: &Value) -> Result<WatchSourceInput> {
    x_user_to_watch_source(user, "x-api/following", "following")
}

pub(crate) fn x_user_to_watch_source(
    user: &Value,
    origin: &str,
    reason: &str,
) -> Result<WatchSourceInput> {
    let object = user
        .as_object()
        .context("X following user must be an object")?;
    let username = first_string(object, &["username", "handle"])
        .context("X following user missing username")?
        .trim_start_matches('@')
        .to_string();
    validate_x_handle(&username)?;
    let name = first_string(object, &["name"]).unwrap_or(&username);
    let description = first_string(object, &["description"]).unwrap_or("");
    Ok(WatchSourceInput {
        source_kind: "x_handle".to_string(),
        locator: username.clone(),
        label: format!("@{username} - {name}"),
        cadence: "warm".to_string(),
        status: "active".to_string(),
        metadata: json!({
            "origin": origin,
            "reasons": [reason],
            "x_user_id": first_string(object, &["id"]),
            "name": name,
            "description": description.chars().take(500).collect::<String>(),
            "verified": object.get("verified").and_then(Value::as_bool),
            "verified_type": first_string(object, &["verified_type"]),
        }),
    })
}

pub(crate) fn x_users_by_id(value: &Value) -> BTreeMap<String, Value> {
    value
        .pointer("/includes/users")
        .and_then(Value::as_array)
        .map(|users| {
            users
                .iter()
                .filter_map(|user| {
                    let id = user.get("id")?.as_str()?;
                    Some((id.to_string(), user.clone()))
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn x_bookmark_tweet_author_watch_source(
    tweet: &Value,
    users: &BTreeMap<String, Value>,
    cutoff: DateTime<Utc>,
) -> Result<Option<WatchSourceInput>> {
    let created_at = tweet
        .get("created_at")
        .and_then(Value::as_str)
        .context("bookmarked tweet missing created_at")?;
    let created_at = DateTime::parse_from_rfc3339(created_at)
        .context("bookmarked tweet has invalid created_at")?
        .with_timezone(&Utc);
    if created_at < cutoff {
        return Ok(None);
    }
    let author_id = tweet
        .get("author_id")
        .and_then(Value::as_str)
        .context("bookmarked tweet missing author_id")?;
    let user = users
        .get(author_id)
        .with_context(|| format!("bookmarked tweet author not expanded: {author_id}"))?;
    let mut input = x_user_to_watch_source(user, "x-api/bookmarks", "bookmark")?;
    input.metadata["bookmark_tweet_id"] = tweet.get("id").cloned().unwrap_or(Value::Null);
    input.metadata["bookmark_tweet_created_at"] =
        Value::String(created_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
    Ok(Some(input))
}

pub(crate) fn x_bookmark_tweet_to_item_input(
    tweet: &Value,
    users: &BTreeMap<String, Value>,
    cutoff: DateTime<Utc>,
) -> Result<Option<XItemInput>> {
    let object = tweet
        .as_object()
        .context("bookmarked tweet must be an object")?;
    let x_id = first_string(object, &["id"]).context("bookmarked tweet missing id")?;
    let author_id =
        first_string(object, &["author_id"]).context("bookmarked tweet missing author_id")?;
    let text = first_string(object, &["text"]).context("bookmarked tweet missing text")?;
    let created_at_raw =
        first_string(object, &["created_at"]).context("bookmarked tweet missing created_at")?;
    let created_at = DateTime::parse_from_rfc3339(created_at_raw)
        .context("bookmarked tweet has invalid created_at")?
        .with_timezone(&Utc);
    if created_at < cutoff {
        return Ok(None);
    }
    let user = users
        .get(author_id)
        .with_context(|| format!("bookmarked tweet author not expanded: {author_id}"))?;
    let user_object = user
        .as_object()
        .context("bookmarked tweet author expansion must be an object")?;
    let author = first_string(user_object, &["username", "handle"])
        .unwrap_or(author_id)
        .trim_start_matches('@')
        .to_string();
    validate_x_handle(&author)?;
    let retrieved_at = now();
    Ok(Some(XItemInput {
        x_id: x_id.to_string(),
        author: author.clone(),
        text: text.to_string(),
        url: format!("https://x.com/{author}/status/{x_id}"),
        created_at: Some(created_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)),
        conversation_id: first_string(object, &["conversation_id"]).map(ToOwned::to_owned),
        reply_to_x_id: referenced_tweet_id(tweet, "replied_to"),
        quote_x_id: referenced_tweet_id(tweet, "quoted"),
        retweet_x_id: referenced_tweet_id(tweet, "retweeted"),
        retrieved_at: Some(retrieved_at.clone()),
        metrics: tweet
            .get("public_metrics")
            .cloned()
            .unwrap_or_else(|| json!({})),
        raw: tweet.clone(),
        source_kind: "bookmark".to_string(),
        source_detail: Some("bookmarks".to_string()),
        source_metadata: json!({
            "imported_from": "x_api/bookmarks",
            "bookmark_imported_at": retrieved_at,
            "tweet_created_at": created_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "x_author_id": author_id,
            "author_name": first_string(user_object, &["name"]),
            "author_description": first_string(user_object, &["description"]),
            "verified": user.get("verified").and_then(Value::as_bool),
            "verified_type": first_string(user_object, &["verified_type"])
        }),
    }))
}

pub(crate) fn merge_x_watch_source(
    inputs: &mut BTreeMap<String, WatchSourceInput>,
    mut input: WatchSourceInput,
    reason: &str,
) {
    if let Some(existing) = inputs.get_mut(&input.locator) {
        let mut reasons: BTreeSet<String> = existing
            .metadata
            .get("reasons")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(ToOwned::to_owned)
            .collect();
        reasons.insert(reason.to_string());
        existing.metadata["reasons"] = json!(reasons.into_iter().collect::<Vec<_>>());
        existing.metadata["origin"] = json!("x-api/definitive");
    } else {
        input.metadata["origin"] = json!("x-api/definitive");
        inputs.insert(input.locator.clone(), input);
    }
}

pub(crate) fn first_string<'a>(
    object: &'a serde_json::Map<String, Value>,
    keys: &[&str],
) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| object.get(*key).and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

pub(crate) fn referenced_tweet_id(value: &Value, reference_type: &str) -> Option<String> {
    value
        .get("referenced_tweets")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find_map(|reference| {
            let object = reference.as_object()?;
            let kind = first_string(object, &["type"])?;
            if kind != reference_type {
                return None;
            }
            first_string(object, &["id", "x_id", "tweet_id"]).map(ToOwned::to_owned)
        })
}

pub(crate) fn x_item_entities(input: &XItemInput) -> Value {
    input
        .raw
        .get("entities")
        .cloned()
        .or_else(|| input.source_metadata.get("entities").cloned())
        .unwrap_or_else(|| json!({}))
}

pub(crate) fn x_link_candidates(
    text: &str,
    tweet_url: &str,
    entities: &Value,
    raw: &Value,
) -> Vec<XLinkCandidate> {
    let mut candidates = Vec::new();
    for entity in entities
        .get("urls")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(object) = entity.as_object()
            && let Some(url) = first_string(object, &["expanded_url", "unwound_url", "url"])
        {
            candidates.push(XLinkCandidate {
                url: url.to_string(),
                expanded_url: first_string(object, &["expanded_url", "unwound_url"])
                    .map(ToOwned::to_owned),
                display_url: first_string(object, &["display_url"]).map(ToOwned::to_owned),
                source: "entity".to_string(),
                raw: entity.clone(),
            });
        }
    }
    for entity in raw
        .pointer("/entities/urls")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(object) = entity.as_object()
            && let Some(url) = first_string(object, &["expanded_url", "unwound_url", "url"])
        {
            candidates.push(XLinkCandidate {
                url: url.to_string(),
                expanded_url: first_string(object, &["expanded_url", "unwound_url"])
                    .map(ToOwned::to_owned),
                display_url: first_string(object, &["display_url"]).map(ToOwned::to_owned),
                source: "raw_entity".to_string(),
                raw: entity.clone(),
            });
        }
    }
    for token in text.split_whitespace().chain([tweet_url]) {
        if let Some(url) = plain_http_url_token(token) {
            candidates.push(XLinkCandidate {
                url,
                expanded_url: None,
                display_url: None,
                source: "text".to_string(),
                raw: json!({ "token": token }),
            });
        }
    }
    candidates
}

pub(crate) fn plain_http_url_token(token: &str) -> Option<String> {
    let trimmed = token.trim_matches(|ch: char| {
        matches!(
            ch,
            '"' | '\'' | '<' | '>' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | '.' | ';' | ':'
        )
    });
    if trimmed.starts_with("https://") || trimmed.starts_with("http://") {
        Some(trimmed.to_string())
    } else {
        None
    }
}

pub(crate) fn suggested_searches(query: &str) -> Vec<String> {
    vec![
        query.to_string(),
        format!("{query} official docs OR blog"),
        format!("{query} GitHub"),
        format!("{query} analysis criticism"),
    ]
}
