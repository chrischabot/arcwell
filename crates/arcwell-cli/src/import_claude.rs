use crate::*;
pub(crate) fn read_stdin_lossy() -> Result<String> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    Ok(input)
}

pub(crate) fn hook_text_from_input(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
        return Some(trimmed.to_string());
    };
    for pointer in [
        "/prompt",
        "/user_prompt",
        "/userPrompt",
        "/message",
        "/text",
        "/input",
        "/transcript",
        "/conversation",
        "/last_message",
        "/lastMessage",
    ] {
        if let Some(text) = value.pointer(pointer).and_then(Value::as_str)
            && !text.trim().is_empty()
        {
            return Some(text.to_string());
        }
    }
    if let Some(messages) = value.get("messages").and_then(Value::as_array) {
        let joined = messages
            .iter()
            .filter_map(|message| {
                message
                    .get("content")
                    .or_else(|| message.get("text"))
                    .and_then(Value::as_str)
            })
            .collect::<Vec<_>>()
            .join("\n");
        if !joined.trim().is_empty() {
            return Some(joined);
        }
    }
    Some(trimmed.to_string())
}

#[derive(Debug, Serialize)]
pub(crate) struct ClaudeImportReport {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) import_run_id: Option<String>,
    pub(crate) source_kind: String,
    pub(crate) source_path: String,
    pub(crate) conversations_seen: usize,
    pub(crate) conversations_sampled: usize,
    pub(crate) candidates_seen: usize,
    pub(crate) candidates_sampled: usize,
    pub(crate) candidates_written: usize,
    pub(crate) duplicates_suppressed: usize,
    pub(crate) candidates: Vec<ImportCandidate>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ImportCandidate {
    pub(crate) target: String,
    pub(crate) kind: String,
    pub(crate) content: String,
    pub(crate) sensitivity: String,
    pub(crate) source_ref: String,
    pub(crate) operation: String,
    pub(crate) memory_id: Option<String>,
    pub(crate) user_id: Option<String>,
    pub(crate) metadata: Value,
}

pub(crate) fn analyze_claude_export(
    path: &PathBuf,
    limit: usize,
    user_id: Option<&str>,
) -> Result<ClaudeImportReport> {
    if let Some(canonical_path) = resolve_claude_canonical_export(path) {
        return analyze_claude_canonical_export(&canonical_path, limit, user_id);
    }
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    let value: Value = serde_json::from_slice(&bytes).context("parsing Claude export JSON")?;
    let conversations = value
        .as_array()
        .context("expected Claude export root to be an array")?;

    let mut candidates = Vec::new();
    for (idx, conversation) in conversations.iter().enumerate().take(limit) {
        let title = conversation
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim();
        let summary = conversation
            .get("summary")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim();
        let uuid = conversation
            .get("uuid")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let source_ref = format!("claude:{uuid}");
        let haystack = format!("{title}\n{summary}").to_lowercase();

        if haystack.contains("adhd")
            || haystack.contains("bpd")
            || haystack.contains("rejection sensitivity")
        {
            candidates.push(ImportCandidate {
                target: "profile".to_string(),
                kind: "support.competence_respect".to_string(),
                content: "For emotionally sensitive or personalized tasks, consult relevant durable context, choose sufficient reasoning effort, use available tools, and disclose unavailable context rather than guessing.".to_string(),
                sensitivity: "sensitive".to_string(),
                source_ref: source_ref.clone(),
                operation: "ADD".to_string(),
                memory_id: None,
                user_id: user_id.map(ToOwned::to_owned),
                metadata: redact_secret_like_json(json!({
                    "source": "claude_raw_conversation_import",
                    "conversation_uuid": uuid,
                    "title": title,
                    "summary": summary
                })),
            });
        }

        if haystack.contains("style") || haystack.contains("writing") || haystack.contains("blog") {
            candidates.push(ImportCandidate {
                target: "profile".to_string(),
                kind: "writing.style_source".to_string(),
                content: "Writing and style preferences should be maintained as inspectable profile/style documents, not hidden memory.".to_string(),
                sensitivity: "normal".to_string(),
                source_ref: source_ref.clone(),
                operation: "ADD".to_string(),
                memory_id: None,
                user_id: user_id.map(ToOwned::to_owned),
                metadata: redact_secret_like_json(json!({
                    "source": "claude_raw_conversation_import",
                    "conversation_uuid": uuid,
                    "title": title,
                    "summary": summary
                })),
            });
        }

        if haystack.contains("wardrobe")
            || haystack.contains("outfit")
            || haystack.contains("sprezzatura")
        {
            candidates.push(ImportCandidate {
                target: "memory".to_string(),
                kind: "preference".to_string(),
                content: "Wardrobe and outfit advice should account for inventory, fit, weather, comfort, formality, rotation, and prior decisions.".to_string(),
                sensitivity: "normal".to_string(),
                source_ref: source_ref.clone(),
                operation: "ADD".to_string(),
                memory_id: None,
                user_id: user_id.map(ToOwned::to_owned),
                metadata: redact_secret_like_json(json!({
                    "source": "claude_raw_conversation_import",
                    "conversation_uuid": uuid,
                    "title": title,
                    "summary": summary
                })),
            });
        }

        if title.is_empty() && summary.is_empty() && idx + 1 >= limit {
            break;
        }
    }

    Ok(ClaudeImportReport {
        import_run_id: None,
        source_kind: "raw_conversations".to_string(),
        source_path: path.display().to_string(),
        conversations_seen: conversations.len(),
        conversations_sampled: conversations.len().min(limit),
        candidates_seen: candidates.len(),
        candidates_sampled: candidates.len(),
        candidates_written: 0,
        duplicates_suppressed: 0,
        candidates,
    })
}

pub(crate) fn resolve_claude_canonical_export(path: &Path) -> Option<PathBuf> {
    if path.is_dir() {
        for candidate in [
            path.join("out").join("canonical_memories.jsonl"),
            path.join("canonical_memories.jsonl"),
            path.join("out").join("mem0").join("mem0_ingest.jsonl"),
            path.join("mem0_ingest.jsonl"),
        ] {
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        return None;
    }

    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    if matches!(name, "canonical_memories.jsonl" | "mem0_ingest.jsonl") {
        Some(path.to_path_buf())
    } else {
        None
    }
}

pub(crate) fn analyze_claude_canonical_export(
    path: &Path,
    limit: usize,
    user_id: Option<&str>,
) -> Result<ClaudeImportReport> {
    let (candidates_seen, rows) = read_jsonl_values(path, Some(limit))?;
    let mut candidates = Vec::new();
    for value in rows {
        candidates.push(import_candidate_from_claude_memory(value, user_id)?);
    }
    Ok(ClaudeImportReport {
        import_run_id: None,
        source_kind: "canonical_memories".to_string(),
        source_path: path.display().to_string(),
        conversations_seen: 0,
        conversations_sampled: 0,
        candidates_seen,
        candidates_sampled: candidates.len(),
        candidates_written: 0,
        duplicates_suppressed: 0,
        candidates,
    })
}

pub(crate) fn read_jsonl_values(
    path: &Path,
    sample_limit: Option<usize>,
) -> Result<(usize, Vec<Value>)> {
    let file = fs::File::open(path).with_context(|| format!("reading {}", path.display()))?;
    let reader = std::io::BufReader::new(file);
    let mut seen = 0;
    let mut rows = Vec::new();
    for (idx, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("reading {} line {}", path.display(), idx + 1))?;
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(&line)
            .with_context(|| format!("parsing {} line {}", path.display(), idx + 1))?;
        seen += 1;
        if sample_limit.is_none_or(|limit| rows.len() < limit) {
            rows.push(value);
        }
    }
    Ok((seen, rows))
}

pub(crate) fn import_candidate_from_claude_memory(
    value: Value,
    user_id: Option<&str>,
) -> Result<ImportCandidate> {
    if value.get("metadata").is_some() && value.get("memory_id").is_some() {
        import_candidate_from_mem0_row(value, user_id)
    } else {
        import_candidate_from_canonical_row(value, user_id)
    }
}

pub(crate) fn import_candidate_from_mem0_row(
    value: Value,
    user_id: Option<&str>,
) -> Result<ImportCandidate> {
    let memory = redact_secret_like_text(&required_value_string(&value, "memory")?);
    let memory_id = optional_value_string(&value, "memory_id");
    let metadata = value.get("metadata").cloned().unwrap_or_else(|| json!({}));
    let category = metadata
        .get("category")
        .and_then(Value::as_str)
        .unwrap_or("fact");
    let sensitivity = metadata
        .get("sensitivity")
        .and_then(Value::as_str)
        .unwrap_or("normal");
    let source_ref = claude_source_ref(memory_id.as_deref(), &metadata);
    let operation = claude_memory_operation(&value, &metadata);
    let candidate_memory_id = if matches!(operation.as_str(), "UPDATE" | "DELETE") {
        memory_id.clone()
    } else {
        None
    };
    Ok(ImportCandidate {
        target: "memory".to_string(),
        kind: claude_memory_kind(category),
        content: memory,
        sensitivity: sensitivity.to_string(),
        source_ref,
        operation: operation.clone(),
        memory_id: candidate_memory_id,
        user_id: user_id
            .map(ToOwned::to_owned)
            .or_else(|| import_user_id(&value, &metadata, None)),
        metadata: add_claude_import_metadata(metadata, memory_id, &operation),
    })
}

pub(crate) fn import_candidate_from_canonical_row(
    value: Value,
    user_id: Option<&str>,
) -> Result<ImportCandidate> {
    let memory = redact_secret_like_text(&required_value_string(&value, "memory")?);
    let details = optional_value_string(&value, "details");
    let content = match details.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(details) => format!("{memory}\n\n{}", redact_secret_like_text(details)),
        None => memory,
    };
    let memory_id = optional_value_string(&value, "memory_id");
    let category = optional_value_string(&value, "category").unwrap_or_else(|| "fact".to_string());
    let sensitivity =
        optional_value_string(&value, "sensitivity").unwrap_or_else(|| "normal".to_string());
    let source_ref = claude_source_ref(memory_id.as_deref(), &value);
    let operation = claude_memory_operation(&value, &value);
    let candidate_memory_id = if matches!(operation.as_str(), "UPDATE" | "DELETE") {
        memory_id.clone()
    } else {
        None
    };
    Ok(ImportCandidate {
        target: "memory".to_string(),
        kind: claude_memory_kind(&category),
        content,
        sensitivity,
        source_ref,
        operation: operation.clone(),
        memory_id: candidate_memory_id,
        user_id: user_id
            .map(ToOwned::to_owned)
            .or_else(|| import_user_id(&value, &value, None))
            .or_else(|| Some("chris".to_string())),
        metadata: add_claude_import_metadata(value, memory_id, &operation),
    })
}

pub(crate) fn add_claude_import_metadata(
    mut metadata: Value,
    memory_id: Option<String>,
    operation: &str,
) -> Value {
    if !metadata.is_object() {
        metadata = json!({ "source_value": metadata });
    }
    let object = metadata.as_object_mut();
    if let Some(object) = object {
        object.insert("imported_from".to_string(), json!("claude_history_export"));
        object.insert("operation".to_string(), json!(operation));
        if let Some(memory_id) = memory_id {
            object.insert("claude_memory_id".to_string(), json!(memory_id));
        }
    }
    redact_secret_like_json(metadata)
}

pub(crate) fn required_value_string(value: &Value, key: &str) -> Result<String> {
    optional_value_string(value, key)
        .filter(|value| !value.trim().is_empty())
        .with_context(|| format!("Claude memory row missing string field {key:?}"))
}

pub(crate) fn optional_value_string(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub(crate) fn claude_memory_kind(category: &str) -> String {
    let cleaned = category
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    format!(
        "claude_export.{}",
        cleaned.trim_matches('_').trim().if_empty("fact")
    )
}

pub(crate) fn claude_source_ref(memory_id: Option<&str>, value: &Value) -> String {
    if let Some(memory_id) = memory_id {
        return format!("claude_export:{memory_id}");
    }
    value
        .get("evidence")
        .and_then(Value::as_array)
        .and_then(|rows| rows.first())
        .and_then(|row| row.get("source_uri"))
        .and_then(Value::as_str)
        .unwrap_or("claude_export:unknown")
        .to_string()
}

pub(crate) fn candidate_dedupe_key(
    target: &str,
    kind: &str,
    content: &str,
    source_ref: &str,
    user_id: Option<&str>,
) -> String {
    if source_ref.starts_with("claude_export:") {
        return format!(
            "{}\0{}\0{}\0{}",
            target,
            kind,
            user_id.unwrap_or(""),
            source_ref
        );
    }
    format!(
        "{}\0{}\0{}\0{}\0{}",
        target,
        kind,
        user_id.unwrap_or(""),
        source_ref,
        content
    )
}

pub(crate) fn import_user_id(
    value: &Value,
    metadata: &Value,
    default_user_id: Option<&str>,
) -> Option<String> {
    default_user_id
        .map(ToOwned::to_owned)
        .or_else(|| optional_value_string(value, "user_id"))
        .or_else(|| optional_value_string(metadata, "user_id"))
        .or_else(|| optional_value_string(value, "user"))
        .or_else(|| optional_value_string(metadata, "user"))
}

pub(crate) fn claude_memory_operation(value: &Value, metadata: &Value) -> String {
    optional_value_string(value, "operation")
        .or_else(|| optional_value_string(metadata, "operation"))
        .or_else(|| optional_value_string(value, "op"))
        .or_else(|| optional_value_string(metadata, "op"))
        .map(|value| match value.to_ascii_uppercase().as_str() {
            "UPDATE" | "UPDATED" => "UPDATE".to_string(),
            "DELETE" | "DELETED" | "REMOVE" | "REMOVED" => "DELETE".to_string(),
            "NONE" | "NOOP" | "SKIP" => "NONE".to_string(),
            _ => "ADD".to_string(),
        })
        .unwrap_or_else(|| "ADD".to_string())
}

pub(crate) fn redact_secret_like_json(value: Value) -> Value {
    match value {
        Value::String(text) => Value::String(redact_secret_like_text(&text)),
        Value::Array(items) => {
            Value::Array(items.into_iter().map(redact_secret_like_json).collect())
        }
        Value::Object(object) => Value::Object(
            object
                .into_iter()
                .map(|(key, value)| {
                    let value = if is_sensitive_json_key(&key) {
                        Value::String("[REDACTED]".to_string())
                    } else {
                        redact_secret_like_json(value)
                    };
                    (key, value)
                })
                .collect(),
        ),
        other => other,
    }
}

pub(crate) fn is_sensitive_json_key(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase();
    normalized.contains("token")
        || normalized.contains("secret")
        || normalized.contains("password")
        || normalized == "authorization"
        || normalized == "api_key"
        || normalized == "apikey"
}

pub(crate) trait IfEmpty {
    fn if_empty<'a>(&'a self, fallback: &'a str) -> &'a str;
}

impl IfEmpty for str {
    fn if_empty<'a>(&'a self, fallback: &'a str) -> &'a str {
        if self.is_empty() { fallback } else { self }
    }
}
