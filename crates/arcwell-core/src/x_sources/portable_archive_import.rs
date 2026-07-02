use super::*;

pub(crate) fn export_x_portable(
    conn: &Connection,
    out_dir: &Path,
) -> Result<XPortableExportReport> {
    fs::create_dir_all(out_dir).with_context(|| format!("creating {}", out_dir.display()))?;
    let generated_at = now();
    let (tweet_rows, redacted_values) = x_portable_tweet_rows(conn)?;
    let shard = write_x_portable_jsonl_shard(out_dir, "data/x/tweets.jsonl", &tweet_rows)?;
    let manifest = json!({
        "format": "arcwell-x-portable",
        "version": 1,
        "generated_at": generated_at,
        "shards": [
            {
                "path": shard.path,
                "rows": shard.rows,
                "bytes": shard.bytes,
                "sha256": shard.sha256
            }
        ],
        "counts": {
            "tweets": shard.rows
        },
        "excludes": [
            "oauth_tokens",
            "sqlite_secret_values",
            "fts_shadow_tables",
            "raw_dms"
        ],
        "redactions": {
            "secret_like_fields_or_values": redacted_values
        }
    });
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    if x_portable_text_has_secret_like_value(&manifest_json) {
        bail!("portable X manifest contains token-like text");
    }
    let manifest_path = out_dir.join("manifest.json");
    fs::write(&manifest_path, manifest_json)
        .with_context(|| format!("writing {}", manifest_path.display()))?;
    let warnings = if redacted_values > 0 {
        vec![format!(
            "redacted {redacted_values} secret-like X portable field/value occurrence(s)"
        )]
    } else {
        Vec::new()
    };
    Ok(XPortableExportReport {
        out_dir: out_dir.display().to_string(),
        manifest_path: manifest_path.display().to_string(),
        generated_at,
        rows_exported: shard.rows,
        shards: vec![shard],
        warnings,
    })
}

pub(crate) fn x_portable_tweet_rows(conn: &Connection) -> Result<(Vec<Value>, usize)> {
    let mut stmt = conn.prepare(
        r#"
        SELECT t.x_id, COALESCE(p.handle, 'archive') AS author, t.text, t.url,
               t.created_at, t.conversation_id, t.reply_to_x_id, t.quote_x_id,
               t.retweet_x_id, t.metrics_json, t.entities_json, t.raw_json,
               t.first_seen_at
        FROM x_tweets t
        LEFT JOIN x_profiles p ON p.id = t.author_profile_id
        ORDER BY t.x_id ASC
        "#,
    )?;
    let rows = stmt.query_map([], |row| {
        let metrics_json: String = row.get(9)?;
        let entities_json: String = row.get(10)?;
        let raw_json: String = row.get(11)?;
        let metrics = serde_json::from_str::<Value>(&metrics_json).unwrap_or_else(|_| json!({}));
        let entities = serde_json::from_str::<Value>(&entities_json).unwrap_or_else(|_| json!({}));
        let raw = serde_json::from_str::<Value>(&raw_json).unwrap_or_else(|_| json!({}));
        Ok(json!({
            "id": row.get::<_, String>(0)?,
            "author": row.get::<_, String>(1)?,
            "text": row.get::<_, String>(2)?,
            "url": row.get::<_, String>(3)?,
            "created_at": row.get::<_, Option<String>>(4)?,
            "conversation_id": row.get::<_, Option<String>>(5)?,
            "reply_to_x_id": row.get::<_, Option<String>>(6)?,
            "quote_x_id": row.get::<_, Option<String>>(7)?,
            "retweet_x_id": row.get::<_, Option<String>>(8)?,
            "metrics": metrics,
            "entities": entities,
            "raw": raw,
            "retrieved_at": row.get::<_, Option<String>>(12)?,
            "source_kind": "portable_import",
            "source_detail": "arcwell-x-portable",
            "source_metadata": {
                "origin": "arcwell_x_portable",
                "portable_format_version": 1
            }
        }))
    })?;
    let mut values = Vec::new();
    let mut redactions = 0usize;
    for row in rows {
        let value = row?;
        let id = value
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let (value, row_redactions) = sanitize_x_portable_export_value(value);
        redactions += row_redactions;
        if x_portable_value_has_secret_like_value(&value) {
            bail!("portable X tweet row contains token-like text after sanitization: {id}");
        }
        values.push(value);
    }
    Ok((values, redactions))
}

pub(crate) fn write_x_portable_jsonl_shard(
    out_dir: &Path,
    relative_path: &str,
    rows: &[Value],
) -> Result<XPortableShardReport> {
    let relative = safe_x_portable_relative_path(relative_path)?;
    let path = out_dir.join(&relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    let mut body = String::new();
    for row in rows {
        let line = serde_json::to_string(row)?;
        if x_portable_text_has_secret_like_value(&line) {
            bail!("portable X shard contains token-like text");
        }
        body.push_str(&line);
        body.push('\n');
    }
    fs::write(&path, body.as_bytes()).with_context(|| format!("writing {}", path.display()))?;
    Ok(XPortableShardReport {
        path: relative.to_string_lossy().replace('\\', "/"),
        rows: rows.len(),
        bytes: body.len(),
        sha256: sha256(body.as_bytes()),
    })
}

pub(crate) fn validate_x_portable(dir: &Path) -> Result<XPortableValidateReport> {
    let manifest_path = dir.join("manifest.json");
    let manifest_bytes =
        fs::read(&manifest_path).with_context(|| format!("reading {}", manifest_path.display()))?;
    let manifest: Value =
        serde_json::from_slice(&manifest_bytes).context("parsing portable X manifest")?;
    if manifest.get("format").and_then(Value::as_str) != Some("arcwell-x-portable") {
        bail!("portable X manifest has unsupported format");
    }
    if manifest.get("version").and_then(Value::as_i64) != Some(1) {
        bail!("portable X manifest has unsupported version");
    }
    let shards = manifest
        .get("shards")
        .and_then(Value::as_array)
        .context("portable X manifest missing shards")?;
    let mut shard_reports = Vec::new();
    let mut total_rows = 0usize;
    for shard in shards {
        let relative = shard
            .get("path")
            .and_then(Value::as_str)
            .context("portable X shard missing path")?;
        let expected_sha = shard
            .get("sha256")
            .and_then(Value::as_str)
            .context("portable X shard missing sha256")?;
        let expected_rows = shard
            .get("rows")
            .and_then(Value::as_u64)
            .context("portable X shard missing rows")? as usize;
        let relative_path = safe_x_portable_relative_path(relative)?;
        let path = dir.join(&relative_path);
        let bytes = fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
        let actual_sha = sha256(&bytes);
        if actual_sha != expected_sha {
            bail!("portable X shard hash mismatch: {relative}");
        }
        let body = String::from_utf8(bytes).context("portable X shard is not UTF-8")?;
        if x_portable_text_has_secret_like_value(&body) {
            bail!("portable X shard contains token-like text: {relative}");
        }
        let rows = parse_x_portable_jsonl_rows(&body)
            .with_context(|| format!("parsing portable X shard {relative}"))?;
        if rows.len() != expected_rows {
            bail!(
                "portable X shard row count mismatch: {relative} expected {expected_rows} got {}",
                rows.len()
            );
        }
        total_rows += rows.len();
        shard_reports.push(XPortableShardReport {
            path: relative.to_string(),
            rows: rows.len(),
            bytes: body.len(),
            sha256: actual_sha,
        });
    }
    Ok(XPortableValidateReport {
        dir: dir.display().to_string(),
        manifest_path: manifest_path.display().to_string(),
        valid: true,
        rows: total_rows,
        shards: shard_reports,
        warnings: Vec::new(),
    })
}

pub(crate) fn read_x_portable_import_rows(dir: &Path) -> Result<Vec<Value>> {
    let manifest_path = dir.join("manifest.json");
    let manifest: Value = serde_json::from_slice(&fs::read(&manifest_path)?)
        .context("parsing portable X manifest")?;
    let shards = manifest
        .get("shards")
        .and_then(Value::as_array)
        .context("portable X manifest missing shards")?;
    let mut rows = Vec::new();
    for shard in shards {
        let relative = shard
            .get("path")
            .and_then(Value::as_str)
            .context("portable X shard missing path")?;
        let path = dir.join(safe_x_portable_relative_path(relative)?);
        let body =
            fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        rows.extend(parse_x_portable_jsonl_rows(&body)?);
    }
    Ok(rows)
}

pub(crate) fn parse_x_portable_jsonl_rows(body: &str) -> Result<Vec<Value>> {
    let mut rows = Vec::new();
    for (index, line) in body.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(line)
            .with_context(|| format!("parsing JSONL line {}", index + 1))?;
        if !value.is_object() {
            bail!("portable X JSONL line {} is not an object", index + 1);
        }
        if x_portable_value_has_secret_like_value(&value) {
            bail!(
                "portable X JSONL line {} contains token-like text",
                index + 1
            );
        }
        rows.push(value);
    }
    Ok(rows)
}

pub(crate) fn safe_x_portable_relative_path(path: &str) -> Result<PathBuf> {
    if path.is_empty() || path.len() > 1_000 || path.contains('\0') || path.contains('\\') {
        bail!("unsafe portable X relative path");
    }
    let relative = PathBuf::from(path);
    if relative.is_absolute() {
        bail!("unsafe portable X relative path");
    }
    for component in relative.components() {
        match component {
            std::path::Component::Normal(_) => {}
            _ => bail!("unsafe portable X relative path"),
        }
    }
    Ok(relative)
}

pub(crate) fn x_portable_value_has_secret_like_value(value: &Value) -> bool {
    match value {
        Value::String(value) => x_portable_text_has_secret_like_value(value),
        Value::Array(values) => values.iter().any(x_portable_value_has_secret_like_value),
        Value::Object(object) => object.iter().any(|(key, value)| {
            x_portable_key_is_secret_like(key) || x_portable_value_has_secret_like_value(value)
        }),
        _ => false,
    }
}

pub(crate) fn sanitize_x_portable_export_value(value: Value) -> (Value, usize) {
    match value {
        Value::String(value) => {
            let redacted = redact_secret_like_text_preserving_whitespace(&value);
            if redacted != value {
                return (Value::String(redacted), 1);
            }
            if x_portable_text_has_secret_like_value(&redacted) {
                return (Value::String("[REDACTED]".to_string()), 1);
            }
            (Value::String(value), 0)
        }
        Value::Array(values) => {
            let mut redactions = 0usize;
            let values = values
                .into_iter()
                .map(|value| {
                    let (value, value_redactions) = sanitize_x_portable_export_value(value);
                    redactions += value_redactions;
                    value
                })
                .collect::<Vec<_>>();
            (Value::Array(values), redactions)
        }
        Value::Object(object) => {
            let mut redactions = 0usize;
            let mut sanitized = serde_json::Map::new();
            for (key, value) in object {
                if x_portable_key_is_secret_like(&key) {
                    redactions += 1;
                    continue;
                }
                let (value, value_redactions) = sanitize_x_portable_export_value(value);
                redactions += value_redactions;
                sanitized.insert(key, value);
            }
            if redactions > 0 {
                sanitized.insert(
                    "_arcwell_redacted_field_or_value_count".to_string(),
                    json!(redactions),
                );
            }
            (Value::Object(sanitized), redactions)
        }
        other => (other, 0),
    }
}

pub(crate) fn x_portable_key_is_secret_like(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    lower.contains("access_token")
        || lower.contains("refresh_token")
        || lower == "token"
        || lower.contains("client_secret")
        || lower.contains("api_key")
}

pub(crate) fn x_portable_text_has_secret_like_value(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("bearer ")
        || lower.contains("access_token")
        || lower.contains("refresh_token")
        || lower.contains("client_secret")
        || lower.contains("api_key")
        || lower.contains("sk-")
        || lower.contains("xoxb-")
        || lower.contains("xoxp-")
}

pub(crate) fn normalize_x_archive_select(select: &[String]) -> Result<BTreeSet<String>> {
    let mut normalized = BTreeSet::new();
    if select.is_empty() {
        normalized.extend([
            "bookmarks".to_string(),
            "likes".to_string(),
            "tweets".to_string(),
        ]);
        return Ok(normalized);
    }
    for raw in select {
        for part in raw.split(',') {
            let value = part.trim().to_ascii_lowercase();
            if value.is_empty() {
                continue;
            }
            match value.as_str() {
                "all" => {
                    normalized.extend([
                        "bookmarks".to_string(),
                        "likes".to_string(),
                        "tweets".to_string(),
                    ]);
                }
                "tweet" | "tweets" => {
                    normalized.insert("tweets".to_string());
                }
                "bookmark" | "bookmarks" => {
                    normalized.insert("bookmarks".to_string());
                }
                "like" | "likes" => {
                    normalized.insert("likes".to_string());
                }
                "profile" | "profiles" | "follower" | "followers" | "following" | "follow"
                | "media" | "dm" | "dms" | "direct_messages" | "direct-messages" => {
                    bail!("X archive selector '{value}' is not implemented yet");
                }
                other => bail!("unsupported X archive selector: {other}"),
            }
        }
    }
    if normalized.is_empty() {
        bail!("X archive selection cannot be empty");
    }
    Ok(normalized)
}

pub(crate) fn collect_x_archive_items(
    path: &Path,
    selected: &BTreeSet<String>,
    limit: usize,
) -> Result<XArchiveCollectedItems> {
    let mut collected = XArchiveCollectedItems {
        files_seen: 0,
        files_imported: 0,
        bytes_read: 0,
        skipped_files: 0,
        unsupported_slices: BTreeMap::new(),
        unsupported_files: Vec::new(),
        warnings: Vec::new(),
        items: Vec::new(),
    };
    if path.is_dir() {
        collect_x_archive_dir(path, selected, limit, &mut collected)?;
    } else {
        collect_x_archive_file(path, selected, limit, &mut collected)?;
    }
    Ok(collected)
}

pub(crate) fn collect_x_archive_file(
    path: &Path,
    selected: &BTreeSet<String>,
    limit: usize,
    collected: &mut XArchiveCollectedItems,
) -> Result<()> {
    if !path.exists() {
        bail!("X archive path does not exist: {}", path.display());
    }
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if extension == "zip" {
        collect_x_archive_zip(path, selected, limit, collected)?;
        return Ok(());
    }
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("archive.json");
    collect_x_archive_named_reader(path, name, selected, limit, collected)
}

pub(crate) fn collect_x_archive_dir(
    root: &Path,
    selected: &BTreeSet<String>,
    limit: usize,
    collected: &mut XArchiveCollectedItems,
) -> Result<()> {
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry?;
        if collected.files_seen >= X_ARCHIVE_MAX_ENTRIES || collected.items.len() >= limit {
            break;
        }
        if entry.file_type().is_dir() {
            continue;
        }
        if entry.file_type().is_symlink() {
            collected.skipped_files += 1;
            collected
                .warnings
                .push(format!("skipped symlink {}", entry.path().display()));
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(root)
            .unwrap_or_else(|_| entry.path())
            .to_string_lossy()
            .replace('\\', "/");
        let safe_relative = safe_x_archive_member_name(&relative)?;
        collect_x_archive_named_reader(entry.path(), &safe_relative, selected, limit, collected)?;
    }
    Ok(())
}

pub(crate) fn collect_x_archive_zip(
    path: &Path,
    selected: &BTreeSet<String>,
    limit: usize,
    collected: &mut XArchiveCollectedItems,
) -> Result<()> {
    let file = fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let mut archive = zip::ZipArchive::new(file).context("opening X archive zip")?;
    for index in 0..archive.len() {
        if collected.files_seen >= X_ARCHIVE_MAX_ENTRIES || collected.items.len() >= limit {
            break;
        }
        let mut member = archive.by_index(index)?;
        let safe_name = safe_x_archive_member_name(member.name())?;
        if member.is_dir() {
            continue;
        }
        reject_nested_x_archive_member(&safe_name)?;
        let Some(kind) = x_archive_file_kind(&safe_name, selected) else {
            collected.skipped_files += 1;
            record_unsupported_x_archive_file(collected, &safe_name);
            continue;
        };
        collected.files_seen += 1;
        validate_x_archive_file_budget(member.size(), collected.bytes_read as u64)?;
        let text = read_utf8_limited(&mut member, X_ARCHIVE_MAX_FILE_BYTES)
            .with_context(|| format!("reading archive member {safe_name}"))?;
        collected.bytes_read += text.len();
        let mut items =
            parse_x_archive_payload(&safe_name, kind, &text, limit - collected.items.len())?;
        if items.is_empty() {
            collected.skipped_files += 1;
        } else {
            collected.files_imported += 1;
            collected.items.append(&mut items);
        }
    }
    Ok(())
}

pub(crate) fn collect_x_archive_named_reader(
    path: &Path,
    relative_name: &str,
    selected: &BTreeSet<String>,
    limit: usize,
    collected: &mut XArchiveCollectedItems,
) -> Result<()> {
    let safe_name = safe_x_archive_member_name(relative_name)?;
    reject_nested_x_archive_member(&safe_name)?;
    let Some(kind) = x_archive_file_kind(&safe_name, selected) else {
        collected.skipped_files += 1;
        record_unsupported_x_archive_file(collected, &safe_name);
        return Ok(());
    };
    collected.files_seen += 1;
    let metadata = fs::metadata(path).with_context(|| format!("reading {}", path.display()))?;
    validate_x_archive_file_budget(metadata.len(), collected.bytes_read as u64)?;
    let mut file = fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let text = read_utf8_limited(&mut file, X_ARCHIVE_MAX_FILE_BYTES)
        .with_context(|| format!("reading {safe_name}"))?;
    collected.bytes_read += text.len();
    let mut items =
        parse_x_archive_payload(&safe_name, kind, &text, limit - collected.items.len())?;
    if items.is_empty() {
        collected.skipped_files += 1;
    } else {
        collected.files_imported += 1;
        collected.items.append(&mut items);
    }
    Ok(())
}

pub(crate) fn safe_x_archive_member_name(name: &str) -> Result<String> {
    if name.is_empty() || name.len() > 1_000 || name.contains('\0') || name.contains('\\') {
        bail!("unsafe X archive member path");
    }
    let path = Path::new(name);
    if path.is_absolute() {
        bail!("unsafe X archive member path");
    }
    for component in path.components() {
        match component {
            std::path::Component::Normal(_) => {}
            _ => bail!("unsafe X archive member path"),
        }
    }
    Ok(path.to_string_lossy().replace('\\', "/"))
}

pub(crate) fn reject_nested_x_archive_member(name: &str) -> Result<()> {
    let normalized = name.to_ascii_lowercase();
    if normalized.ends_with(".zip")
        || normalized.ends_with(".tar")
        || normalized.ends_with(".tgz")
        || normalized.ends_with(".tar.gz")
        || normalized.ends_with(".gz")
    {
        bail!("nested X archive members are not supported");
    }
    Ok(())
}

pub(crate) fn validate_x_archive_file_budget(file_size: u64, current_total: u64) -> Result<()> {
    if file_size > X_ARCHIVE_MAX_FILE_BYTES {
        bail!("X archive member is too large");
    }
    if current_total.saturating_add(file_size) > X_ARCHIVE_MAX_TOTAL_BYTES {
        bail!("X archive import total bytes exceed limit");
    }
    Ok(())
}

pub(crate) fn read_utf8_limited<R: Read>(reader: &mut R, max_bytes: u64) -> Result<String> {
    let mut limited = reader.take(max_bytes + 1);
    let mut bytes = Vec::new();
    limited.read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max_bytes {
        bail!("X archive member is too large");
    }
    String::from_utf8(bytes).context("X archive member is not valid UTF-8")
}

pub(crate) fn x_archive_file_kind(path: &str, selected: &BTreeSet<String>) -> Option<&'static str> {
    let normalized = path.to_ascii_lowercase();
    if normalized.contains("direct-message") || normalized.contains("direct_messages") {
        return None;
    }
    if selected.contains("bookmarks") && normalized.contains("bookmark") {
        return Some("bookmark");
    }
    if selected.contains("likes") && normalized.contains("like") {
        return Some("like");
    }
    if selected.contains("tweets") && normalized.contains("tweet") {
        return Some("tweet");
    }
    None
}

pub(crate) fn record_unsupported_x_archive_file(
    collected: &mut XArchiveCollectedItems,
    path: &str,
) {
    let Some(kind) = x_archive_unsupported_slice_kind(path) else {
        return;
    };
    *collected
        .unsupported_slices
        .entry(kind.to_string())
        .or_insert(0) += 1;
    if collected.unsupported_files.len() < 50 {
        collected.unsupported_files.push(path.to_string());
    }
    let warning = format!("unsupported X archive slice {kind}: {path}");
    if !collected.warnings.contains(&warning) {
        collected.warnings.push(warning);
    }
}

pub(crate) fn x_archive_unsupported_slice_kind(path: &str) -> Option<&'static str> {
    let normalized = path.to_ascii_lowercase().replace(['_', ' '], "-");
    let file_name = normalized.rsplit('/').next().unwrap_or(normalized.as_str());
    if normalized.contains("direct-message")
        || normalized.contains("/dm")
        || file_name.starts_with("dm")
    {
        return Some("direct_messages");
    }
    if file_name.contains("profile") {
        return Some("profiles");
    }
    if file_name.contains("following") {
        return Some("following");
    }
    if file_name.contains("follower") {
        return Some("followers");
    }
    if file_name.contains("media") || normalized.contains("/media/") {
        return Some("media");
    }
    if file_name.contains("account") {
        return Some("account");
    }
    None
}

pub(crate) fn parse_x_archive_payload(
    source_path: &str,
    record_kind: &str,
    text: &str,
    limit: usize,
) -> Result<Vec<Value>> {
    if limit == 0 {
        return Ok(Vec::new());
    }
    let value = parse_x_archive_json_payload(text)
        .with_context(|| format!("parsing X archive payload {source_path}"))?;
    let records = x_archive_records(value)?;
    let mut items = Vec::new();
    for record in records.iter().take(limit) {
        if let Some(item) = x_archive_record_to_import_item(source_path, record_kind, record) {
            items.push(item);
        }
    }
    Ok(items)
}

pub(crate) fn parse_x_archive_json_payload(text: &str) -> Result<Value> {
    if let Ok(value) = serde_json::from_str::<Value>(text) {
        return Ok(value);
    }
    let start_array = text.find('[');
    let start_object = text.find('{');
    let (start, end_char) = match (start_array, start_object) {
        (Some(array), Some(object)) if array < object => (array, ']'),
        (Some(array), None) => (array, ']'),
        (_, Some(object)) => (object, '}'),
        _ => bail!("X archive payload does not contain JSON"),
    };
    let end = text
        .rfind(end_char)
        .context("X archive payload wrapper is incomplete")?;
    if end < start {
        bail!("X archive payload wrapper is malformed");
    }
    serde_json::from_str(&text[start..=end]).context("parsing wrapped X archive JSON")
}

pub(crate) fn x_archive_records(value: Value) -> Result<Vec<Value>> {
    match value {
        Value::Array(records) => Ok(records),
        Value::Object(mut object) => {
            for key in ["tweets", "tweet", "bookmarks", "bookmark", "likes", "like"] {
                if let Some(Value::Array(records)) = object.remove(key) {
                    return Ok(records);
                }
            }
            Ok(vec![Value::Object(object)])
        }
        _ => bail!("X archive payload must be an array or object"),
    }
}

pub(crate) fn x_archive_record_to_import_item(
    source_path: &str,
    record_kind: &str,
    record: &Value,
) -> Option<Value> {
    let object = record.as_object()?;
    let payload = object
        .get(record_kind)
        .or_else(|| object.get("tweet"))
        .or_else(|| object.get("like"))
        .or_else(|| object.get("bookmark"))
        .and_then(Value::as_object)
        .unwrap_or(object);
    let mut url = first_string(payload, &["url", "tweetUrl", "tweet_url", "expandedUrl"])
        .map(ToOwned::to_owned);
    let mut x_id = first_string(
        payload,
        &["id_str", "id", "tweetId", "tweet_id", "status_id"],
    )
    .map(ToOwned::to_owned)
    .or_else(|| url.as_deref().and_then(x_tweet_id_from_url));
    let mut author = first_string(
        payload,
        &["author", "username", "screen_name", "handle", "user"],
    )
    .map(|value| value.trim_start_matches('@').to_string())
    .or_else(|| url.as_deref().and_then(x_author_from_tweet_url))
    .unwrap_or_else(|| "archive".to_string());
    if author.is_empty() {
        author = "archive".to_string();
    }
    let text = first_string(
        payload,
        &[
            "full_text",
            "fullText",
            "text",
            "body",
            "content",
            "noteTweetText",
        ],
    )?
    .to_string();
    if x_id.is_none()
        && let Some(id) = first_string(payload, &["tweet_id"])
    {
        x_id = Some(id.to_string());
    }
    let x_id = x_id?;
    if url.is_none() {
        url = Some(format!("https://x.com/{author}/status/{x_id}"));
    }
    let source_kind = match record_kind {
        "bookmark" => "bookmark",
        "like" => "archive_like",
        _ => "archive",
    };
    Some(json!({
        "id": x_id,
        "author": author,
        "text": text,
        "url": url,
        "created_at": first_string(payload, &["created_at", "createdAt", "date"]),
        "conversation_id": first_string(payload, &["conversation_id", "conversationId"]),
        "reply_to_x_id": first_string(payload, &["in_reply_to_status_id_str", "in_reply_to_status_id", "reply_to_x_id"]),
        "quote_x_id": first_string(payload, &["quoted_status_id_str", "quoted_tweet_id", "quote_x_id"]),
        "retweet_x_id": first_string(payload, &["retweeted_status_id_str", "retweeted_tweet_id", "retweet_x_id"]),
        "metrics": {
            "favorite_count": first_string(payload, &["favorite_count", "favoriteCount", "like_count"]),
            "retweet_count": first_string(payload, &["retweet_count", "retweetCount"]),
            "reply_count": first_string(payload, &["reply_count", "replyCount"]),
        },
        "raw": record,
        "source_kind": source_kind,
        "source_detail": source_path,
        "source_metadata": {
            "origin": "x_archive",
            "archive_path": source_path,
            "record_kind": record_kind,
            "x_author_id": first_string(payload, &["x_author_id", "author_id", "user_id", "userId", "userIdStr"]),
            "network_fetch": false
        }
    }))
}

pub(crate) fn x_tweet_id_from_url(raw: &str) -> Option<String> {
    let url = Url::parse(raw).ok()?;
    let host = url.host_str()?.to_ascii_lowercase();
    if host != "x.com" && host != "twitter.com" && host != "mobile.twitter.com" {
        return None;
    }
    let mut segments = url.path_segments()?;
    while let Some(segment) = segments.next() {
        if segment == "status" || segment == "statuses" {
            return segments
                .next()
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned);
        }
    }
    None
}
