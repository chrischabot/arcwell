use crate::*;

pub(crate) fn normalize_radar_profile_input(
    mut input: RadarProfileInput,
) -> Result<RadarProfileInput> {
    validate_key(&input.name)?;
    input.name = input.name.trim().to_string();
    input.description = excerpt(input.description.trim(), 2_000);
    if input.window_hours <= 0 || input.window_hours > 24 * 365 {
        bail!("window_hours must be between 1 and 8760");
    }
    if !input.min_score.is_finite() || !(0.0..=10.0).contains(&input.min_score) {
        bail!("min_score must be a finite number between 0 and 10");
    }
    if let Some(max_items) = input.max_items
        && !(1..=500).contains(&max_items)
    {
        bail!("max_items must be between 1 and 500 when set");
    }
    input.languages = input
        .languages
        .into_iter()
        .map(|language| language.trim().to_ascii_lowercase())
        .filter(|language| !language.is_empty())
        .collect();
    input.languages.sort();
    input.languages.dedup();
    if input.languages.is_empty() {
        input.languages.push("en".to_string());
    }
    if !input.source_selectors.is_array() {
        bail!("source_selectors must be an array");
    }
    for selector in input.source_selectors.as_array().unwrap_or(&Vec::new()) {
        let kind = radar_selector_kind(selector).context("radar selector requires kind")?;
        validate_key(&kind)?;
        if radar_selector_is_source_card_backed(selector) {
            let query = selector
                .get("query")
                .or_else(|| selector.get("locator"))
                .or_else(|| selector.get("handle"))
                .and_then(Value::as_str)
                .context("source-card-backed radar selector requires query, locator, or handle")?;
            validate_query(query)?;
        }
    }
    if !input.delivery_policy.is_object() {
        bail!("delivery_policy must be an object");
    }
    if !input.model_policy.is_object() {
        bail!("model_policy must be an object");
    }
    if !input.metadata.is_object() {
        bail!("metadata must be an object");
    }
    radar_balance_config_from_metadata(&input.metadata)?;
    Ok(input)
}

pub(crate) fn radar_profile_status(source_selectors: &Value) -> String {
    let unsupported = unsupported_radar_selectors(source_selectors);
    let supported = source_selectors
        .as_array()
        .map(|selectors| {
            selectors
                .iter()
                .any(|selector| radar_selector_is_source_card_backed(selector))
        })
        .unwrap_or(false);
    match (supported, unsupported.is_empty()) {
        (true, true) => "local_proof_ready".to_string(),
        (true, false) => "partial".to_string(),
        (false, false) => "unsupported".to_string(),
        (false, true) => "empty".to_string(),
    }
}

pub(crate) fn radar_selector_kind(selector: &Value) -> Option<String> {
    selector
        .get("kind")
        .or_else(|| selector.get("source_kind"))
        .and_then(Value::as_str)
        .map(|kind| kind.trim().to_ascii_lowercase())
        .filter(|kind| !kind.is_empty())
}

pub(crate) fn radar_selector_locator(selector: &Value) -> Option<String> {
    selector
        .get("query")
        .or_else(|| selector.get("locator"))
        .or_else(|| selector.get("handle"))
        .and_then(Value::as_str)
        .map(|locator| locator.trim().to_string())
        .filter(|locator| !locator.is_empty())
}

pub(crate) fn parse_github_repo_locator(locator: &str) -> Option<(String, String)> {
    let trimmed = locator.trim().trim_matches('/');
    let mut parts = trimmed.split('/');
    let owner = parts.next()?.trim();
    let repo = parts.next()?.trim();
    if owner.is_empty() || repo.is_empty() || parts.next().is_some() {
        return None;
    }
    Some((owner.to_string(), repo.to_string()))
}

pub(crate) fn unsupported_radar_selectors(source_selectors: &Value) -> Vec<Value> {
    source_selectors
        .as_array()
        .map(|selectors| {
            selectors
                .iter()
                .filter(|selector| !radar_selector_is_source_card_backed(selector))
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn radar_selector_is_source_card_backed(selector: &Value) -> bool {
    matches!(
        radar_selector_kind(selector).as_deref(),
        Some("source_card_query")
            | Some("rss")
            | Some("github")
            | Some("github_release")
            | Some("github_owner")
            | Some("arxiv")
            | Some("hackernews")
            | Some("hn")
            | Some("reddit")
            | Some("x")
            | Some("x_handle")
    )
}

pub(crate) fn radar_source_card_matches_selector(
    card: &SourceCard,
    kind: &str,
    locator: &str,
) -> bool {
    let locator = locator.trim().trim_start_matches('@').to_ascii_lowercase();
    let url = card.url.to_ascii_lowercase();
    let provider = card.provider.to_ascii_lowercase();
    let source_type = card.source_type.to_ascii_lowercase();
    let title = card.title.to_ascii_lowercase();
    let source_kind = card
        .metadata
        .get("source_kind")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_ascii_lowercase();
    let metadata_text = serde_json::to_string(&card.metadata)
        .unwrap_or_default()
        .to_ascii_lowercase();
    let source_detail = card
        .metadata
        .get("source_detail")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim_start_matches('@')
        .to_ascii_lowercase();
    let author = card
        .metadata
        .get("author")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim_start_matches('@')
        .to_ascii_lowercase();
    let locator_matches = locator == "*"
        || url.contains(&locator)
        || title.contains(&locator)
        || metadata_text.contains(&locator)
        || source_detail == locator
        || author == locator;
    match kind {
        "rss" => {
            locator_matches
                && (source_kind.contains("rss")
                    || source_kind.contains("atom")
                    || source_type.contains("rss")
                    || provider.contains("rss"))
        }
        "github" | "github_release" | "github_owner" => {
            locator_matches
                && (url.contains("github.com")
                    || source_kind.contains("github")
                    || source_type.contains("github")
                    || provider.contains("github"))
        }
        "arxiv" => {
            locator_matches
                && (url.contains("arxiv.org")
                    || source_kind.contains("arxiv")
                    || source_type.contains("arxiv")
                    || provider.contains("arxiv"))
        }
        "hackernews" | "hn" => {
            let hn_locator =
                normalize_hackernews_feed(&locator).unwrap_or_else(|_| locator.clone());
            (locator_matches || source_detail == hn_locator || metadata_text.contains(&hn_locator))
                && (url.contains("news.ycombinator.com")
                    || source_kind.contains("hackernews")
                    || source_type.contains("hackernews")
                    || provider.contains("hackernews"))
        }
        "reddit" => {
            let reddit_locator = normalize_reddit_locator(&locator)
                .map(|locator| locator.source_detail())
                .unwrap_or_else(|_| locator.clone());
            (locator_matches
                || source_detail == reddit_locator
                || metadata_text.contains(&reddit_locator))
                && (url.contains("reddit.com/")
                    || source_kind.contains("reddit")
                    || source_type.contains("reddit")
                    || provider.contains("reddit"))
        }
        "x" | "x_handle" => {
            locator_matches
                && (url.contains("x.com/")
                    || url.contains("twitter.com/")
                    || source_kind == "x"
                    || source_kind.contains("watch_monitor")
                    || source_type == "x"
                    || provider.contains("x-import"))
        }
        _ => false,
    }
}

pub(crate) fn radar_exact_url_key(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(mut url) = Url::parse(trimmed) {
        url.set_fragment(None);
        if url.path() == "/" {
            url.set_path("");
        }
        return Some(url.to_string().trim_end_matches('/').to_ascii_lowercase());
    }
    Some(trimmed.trim_end_matches('/').to_ascii_lowercase())
}

pub(crate) fn radar_exact_dedup_group(
    run_id: &str,
    dedup_kind: &str,
    reason: &str,
    members: Vec<RadarItem>,
) -> Result<RadarDedupGroup> {
    radar_dedup_group(run_id, dedup_kind, reason, 1.0, None, members)
}

pub(crate) fn radar_dedup_group(
    run_id: &str,
    dedup_kind: &str,
    reason: &str,
    confidence: f64,
    model_provider: Option<String>,
    mut members: Vec<RadarItem>,
) -> Result<RadarDedupGroup> {
    members.sort_by(|left, right| {
        let left_score = score_radar_item_heuristic(left).0;
        let right_score = score_radar_item_heuristic(right).0;
        right_score
            .partial_cmp(&left_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.title.cmp(&right.title))
            .then_with(|| left.id.cmp(&right.id))
    });
    let primary_item_id = members
        .first()
        .map(|item| item.id.clone())
        .context("radar dedupe group requires at least one member")?;
    let member_item_ids = members
        .iter()
        .map(|item| item.id.clone())
        .collect::<Vec<_>>();
    let stable = format!("{run_id}\n{dedup_kind}\n{}", member_item_ids.join("\n"));
    Ok(RadarDedupGroup {
        id: format!("radar-dedup-{}", &sha256(stable.as_bytes())[..32]),
        run_id: run_id.to_string(),
        dedup_kind: dedup_kind.to_string(),
        primary_item_id,
        member_item_ids,
        reason: reason.to_string(),
        confidence,
        model_provider,
        cost_decision_id: None,
        created_at: now(),
    })
}

pub(crate) fn radar_topic_dedup_group(
    run_id: &str,
    mut members: Vec<(RadarItem, RadarScore)>,
) -> Result<RadarDedupGroup> {
    members.sort_by(|left, right| {
        right
            .1
            .score
            .partial_cmp(&left.1.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.0.title.cmp(&right.0.title))
            .then_with(|| left.0.id.cmp(&right.0.id))
    });
    let items = members
        .into_iter()
        .map(|(item, _)| item)
        .collect::<Vec<_>>();
    let reason = radar_topic_dedup_reason(&items);
    let primary_item_id = items
        .first()
        .map(|item| item.id.clone())
        .context("radar topic dedupe group requires at least one member")?;
    let member_item_ids = items.iter().map(|item| item.id.clone()).collect::<Vec<_>>();
    let stable = format!("{run_id}\nsemantic_topic\n{}", member_item_ids.join("\n"));
    Ok(RadarDedupGroup {
        id: format!("radar-dedup-{}", &sha256(stable.as_bytes())[..32]),
        run_id: run_id.to_string(),
        dedup_kind: "semantic_topic".to_string(),
        primary_item_id,
        member_item_ids,
        reason,
        confidence: 0.82,
        model_provider: None,
        cost_decision_id: None,
        created_at: now(),
    })
}

pub(crate) fn radar_topic_dedupe_signature(item: &RadarItem) -> Option<BTreeSet<String>> {
    let mut tokens = BTreeSet::new();
    radar_topic_dedupe_collect_tokens(&item.title, &mut tokens);
    for key in ["topic", "category", "category_group"] {
        if let Some(value) = item.metadata.get(key).and_then(Value::as_str) {
            radar_topic_dedupe_collect_tokens(value, &mut tokens);
        }
    }
    let values = item
        .metadata
        .get("topics")
        .or_else(|| item.metadata.get("categories"));
    if let Some(values) = values.and_then(Value::as_array) {
        for value in values {
            if let Some(value) = value.as_str() {
                radar_topic_dedupe_collect_tokens(value, &mut tokens);
            }
        }
    }
    if tokens.len() >= 3 {
        Some(tokens)
    } else {
        None
    }
}

pub(crate) fn radar_topic_dedupe_collect_tokens(text: &str, tokens: &mut BTreeSet<String>) {
    for raw in text
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        let mut token = raw.to_ascii_lowercase();
        if token.len() > 4 && token.ends_with('s') {
            token.pop();
        }
        if token.len() < 3 || token.len() > 48 || radar_topic_dedupe_stopword(&token) {
            continue;
        }
        tokens.insert(token);
    }
}

pub(crate) fn radar_topic_dedupe_stopword(token: &str) -> bool {
    matches!(
        token,
        "about"
            | "after"
            | "again"
            | "against"
            | "all"
            | "and"
            | "are"
            | "around"
            | "from"
            | "has"
            | "into"
            | "new"
            | "not"
            | "now"
            | "over"
            | "per"
            | "the"
            | "this"
            | "through"
            | "via"
            | "with"
            | "without"
    )
}

pub(crate) fn radar_topic_signatures_match(
    left: &BTreeSet<String>,
    right: &BTreeSet<String>,
) -> bool {
    let intersection = left.intersection(right).count();
    if intersection < 3 {
        return false;
    }
    let left_overlap = intersection as f64 / left.len() as f64;
    let right_overlap = intersection as f64 / right.len() as f64;
    let union = left.union(right).count();
    let jaccard = intersection as f64 / union as f64;
    (left_overlap >= 0.60 && right_overlap >= 0.60) || (intersection >= 4 && jaccard >= 0.45)
}

pub(crate) fn radar_topic_dedup_reason(members: &[RadarItem]) -> String {
    let shared = members
        .iter()
        .filter_map(radar_topic_dedupe_signature)
        .reduce(|left, right| left.intersection(&right).cloned().collect())
        .unwrap_or_default()
        .into_iter()
        .take(8)
        .collect::<Vec<_>>();
    if shared.is_empty() {
        "deterministic topic similarity".to_string()
    } else {
        format!(
            "deterministic topic similarity over shared tokens: {}",
            shared.join(", ")
        )
    }
}

pub(crate) fn normalize_radar_summary_language(language: &str) -> Result<String> {
    let language = language.trim().to_ascii_lowercase();
    if language.is_empty() || language.len() > 16 {
        bail!("radar summary language must be 1-16 characters");
    }
    if !language
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        bail!("radar summary language contains unsupported characters");
    }
    Ok(language)
}

pub(crate) fn normalize_radar_summary_format(format: &str) -> Result<String> {
    let format = format.trim().to_ascii_lowercase();
    match format.as_str() {
        "markdown" => Ok(format),
        _ => bail!("radar summary format must be markdown"),
    }
}

pub(crate) fn normalize_radar_delivery_channel(channel: &str) -> Result<String> {
    let channel = channel.trim().to_ascii_lowercase();
    match channel.as_str() {
        "telegram" | "email" => Ok(channel),
        _ => bail!("radar delivery channel must be telegram or email"),
    }
}

pub(crate) fn normalize_radar_delivery_recipient(
    channel: &str,
    recipient_ref: &str,
) -> Result<String> {
    match channel {
        "telegram" => {
            let chat_id = recipient_ref
                .trim()
                .strip_prefix("telegram:chat:")
                .unwrap_or_else(|| recipient_ref.trim());
            validate_query(chat_id)?;
            Ok(format!("telegram:chat:{chat_id}"))
        }
        "email" => {
            let email = recipient_ref
                .trim()
                .strip_prefix("email:")
                .unwrap_or_else(|| recipient_ref.trim());
            let email =
                normalize_email_address(email).context("invalid radar delivery email recipient")?;
            Ok(format!("email:{email}"))
        }
        other => bail!("unsupported radar delivery channel: {other}"),
    }
}

pub(crate) fn normalize_radar_delivery_idempotency_key(
    explicit: Option<&str>,
    summary: &RadarSummary,
    channel: &str,
    recipient_ref: &str,
) -> Result<String> {
    if let Some(explicit) = explicit {
        validate_query(explicit)?;
        return Ok(explicit.trim().to_string());
    }
    Ok(format!(
        "radar-delivery-{}",
        &sha256(
            format!(
                "{}\n{}\n{}\n{}\n{}",
                summary.run_id, summary.id, summary.language, channel, recipient_ref
            )
            .as_bytes()
        )[..32]
    ))
}

pub(crate) fn digest_alert_schedule_policy(
    schedule: &DigestAlertSchedule,
) -> Result<DigestAlertSchedulePolicy> {
    let channel = normalize_radar_delivery_channel(&schedule.channel)?;
    let recipient_ref = normalize_radar_delivery_recipient(&channel, &schedule.recipient_ref)?;
    let quiet_hours = schedule
        .quiet_hours
        .as_ref()
        .map(parse_scheduled_radar_quiet_hours)
        .transpose()?;
    let policy = DigestAlertSchedulePolicy {
        channel,
        recipient_ref,
        min_score: schedule.min_score,
        max_candidates: schedule.max_candidates,
        interval_hours: schedule.interval_hours,
        quiet_hours,
    };
    validate_digest_alert_schedule_policy(&policy)?;
    Ok(policy)
}

pub(crate) fn validate_digest_alert_schedule_policy(
    policy: &DigestAlertSchedulePolicy,
) -> Result<()> {
    if !policy.min_score.is_finite() || !(0.0..=10.0).contains(&policy.min_score) {
        bail!("digest alert min_score must be finite and between 0 and 10");
    }
    if !(1..=50).contains(&policy.max_candidates) {
        bail!("digest alert max_candidates must be between 1 and 50");
    }
    if !(1..=24 * 365).contains(&policy.interval_hours) {
        bail!("digest alert interval_hours must be between 1 and 8760");
    }
    Ok(())
}

pub(crate) fn digest_alert_quiet_hours_deferred_until(
    policy: &DigestAlertSchedulePolicy,
    now: DateTime<Utc>,
) -> Result<Option<DateTime<Utc>>> {
    let radar_policy = ScheduledRadarDeliveryPolicy {
        interval_hours: policy.interval_hours,
        channel: policy.channel.clone(),
        recipient_ref: policy.recipient_ref.clone(),
        language: "en".to_string(),
        format: "markdown".to_string(),
        fetch_live: false,
        quiet_hours: policy.quiet_hours.clone(),
    };
    radar_quiet_hours_deferred_until(&radar_policy, now)
}

pub(crate) fn digest_alert_schedule_tick_key(
    schedule_id: &str,
    due_at: &str,
    policy: &DigestAlertSchedulePolicy,
) -> String {
    format!(
        "digest-alert-{}",
        &sha256(
            format!(
                "{}\n{}\n{}\n{}\n{}\n{}",
                schedule_id,
                due_at,
                policy.channel,
                policy.recipient_ref,
                policy.min_score,
                policy.max_candidates
            )
            .as_bytes()
        )[..32]
    )
}

pub(crate) fn digest_alert_delivery_subject(policy: &DigestAlertSchedulePolicy) -> Result<String> {
    normalize_radar_delivery_recipient(&policy.channel, &policy.recipient_ref)
}

pub(crate) fn digest_alert_schedule_is_credential_reminder(schedule: &DigestAlertSchedule) -> bool {
    matches!(
        schedule.name.trim().to_ascii_lowercase().as_str(),
        "credential reminder" | "credential reminders" | "credential health reminders"
    )
}

pub(crate) fn validate_issue_schedule_kind(kind: &str) -> Result<()> {
    match kind {
        "knowledge_daily_briefing" => Ok(()),
        other => bail!("unsupported issue schedule kind: {other}"),
    }
}

pub(crate) fn validate_issue_schedule_metadata(metadata: &Value) -> Result<()> {
    let cadence = issue_schedule_cadence(metadata)?;
    if cadence == "weekly" || metadata.get("weekday").is_some() {
        issue_schedule_weekday_number(metadata)?;
    }
    Ok(())
}

pub(crate) fn issue_schedule_cadence(metadata: &Value) -> Result<String> {
    let cadence = metadata
        .get("cadence")
        .and_then(Value::as_str)
        .unwrap_or("daily")
        .trim()
        .to_ascii_lowercase();
    match cadence.as_str() {
        "daily" | "weekly" => Ok(cadence),
        other => bail!("issue schedule cadence must be daily or weekly, got {other}"),
    }
}

fn issue_schedule_weekday_number(metadata: &Value) -> Result<u32> {
    let weekday = metadata
        .get("weekday")
        .and_then(Value::as_str)
        .unwrap_or("friday")
        .trim()
        .to_ascii_lowercase();
    let day = match weekday.as_str() {
        "monday" | "mon" => 0,
        "tuesday" | "tue" | "tues" => 1,
        "wednesday" | "wed" => 2,
        "thursday" | "thu" | "thur" | "thurs" => 3,
        "friday" | "fri" => 4,
        "saturday" | "sat" => 5,
        "sunday" | "sun" => 6,
        other => bail!("issue schedule weekday must be a weekday name, got {other}"),
    };
    Ok(day)
}

pub(crate) fn validate_issue_schedule_status(status: &str) -> Result<()> {
    match status {
        "active" | "paused" => Ok(()),
        other => bail!("issue schedule status must be active or paused, got {other}"),
    }
}

pub(crate) fn validate_issue_schedule_tick_status(status: &str) -> Result<()> {
    match status {
        "pending" | "running" | "sent" | "failed" | "blocked" | "deferred" | "empty"
        | "partial" | "dead_lettered" => Ok(()),
        other => bail!("unsupported issue schedule tick status: {other}"),
    }
}

pub(crate) fn normalize_issue_schedule_time_zone(time_zone: &str) -> Result<String> {
    let time_zone = time_zone.trim().to_ascii_lowercase();
    match time_zone.as_str() {
        "utc" | "local" => Ok(time_zone),
        other => bail!("issue schedule time_zone must be utc or local, got {other}"),
    }
}

pub(crate) fn issue_schedule_id(kind: &str, name: &str) -> String {
    format!(
        "isch-{}",
        &sha256(format!("{kind}\n{name}").as_bytes())[..24]
    )
}

pub(crate) fn issue_schedule_tick_key(
    schedule_id: &str,
    due_at: &str,
    schedule: &IssueSchedule,
) -> String {
    format!(
        "issue-{}",
        &sha256(
            format!(
                "{}\n{}\n{}\n{}\n{}",
                schedule_id, due_at, schedule.kind, schedule.channel, schedule.recipient_ref
            )
            .as_bytes()
        )[..32]
    )
}

#[cfg(test)]
pub(crate) fn issue_schedule_due_slots(
    latest_due_at: Option<&str>,
    created_at: &str,
    hour: i64,
    minute: i64,
    catch_up_hours: i64,
    time_zone: &str,
    now_utc: DateTime<Utc>,
    max_ticks: usize,
) -> Result<Vec<String>> {
    issue_schedule_due_slots_with_metadata(
        latest_due_at,
        created_at,
        hour,
        minute,
        catch_up_hours,
        time_zone,
        now_utc,
        max_ticks,
        &Value::Null,
    )
}

pub(crate) fn issue_schedule_due_slots_with_metadata(
    latest_due_at: Option<&str>,
    created_at: &str,
    hour: i64,
    minute: i64,
    catch_up_hours: i64,
    time_zone: &str,
    now_utc: DateTime<Utc>,
    max_ticks: usize,
    metadata: &Value,
) -> Result<Vec<String>> {
    validate_timestamp(created_at)?;
    let created_at = DateTime::parse_from_rfc3339(created_at)?.with_timezone(&Utc);
    let lower_bound = now_utc - ChronoDuration::hours(catch_up_hours.clamp(1, 24 * 14));
    let latest_bound = if let Some(value) = latest_due_at {
        validate_timestamp(value)?;
        Some(DateTime::parse_from_rfc3339(value)?.with_timezone(&Utc) + ChronoDuration::seconds(1))
    } else {
        None
    };
    let mut start = created_at.max(lower_bound);
    if let Some(latest_bound) = latest_bound {
        start = start.max(latest_bound);
    }
    let hour = hour.clamp(0, 23) as u32;
    let minute = minute.clamp(0, 59) as u32;
    let cadence = issue_schedule_cadence(metadata)?;
    let mut slots = match (
        normalize_issue_schedule_time_zone(time_zone)?.as_str(),
        cadence.as_str(),
    ) {
        ("utc", "daily") => issue_schedule_due_slots_utc(start, now_utc, hour, minute)?,
        ("local", "daily") => issue_schedule_due_slots_local(start, now_utc, hour, minute)?,
        ("utc", "weekly") => issue_schedule_weekly_due_slots_utc(
            start,
            now_utc,
            hour,
            minute,
            issue_schedule_weekday_number(metadata)?,
        )?,
        ("local", "weekly") => issue_schedule_weekly_due_slots_local(
            start,
            now_utc,
            hour,
            minute,
            issue_schedule_weekday_number(metadata)?,
        )?,
        _ => unreachable!("time zone normalized above"),
    };
    slots.sort();
    slots.truncate(max_ticks.clamp(1, 30));
    Ok(slots.into_iter().map(|slot| slot.to_rfc3339()).collect())
}

#[cfg(test)]
pub(crate) fn issue_schedule_next_scheduled_slot(
    created_at: &str,
    hour: i64,
    minute: i64,
    time_zone: &str,
    now_utc: DateTime<Utc>,
) -> Result<String> {
    issue_schedule_next_scheduled_slot_with_metadata(
        created_at,
        hour,
        minute,
        time_zone,
        now_utc,
        &Value::Null,
    )
}

pub(crate) fn issue_schedule_next_scheduled_slot_with_metadata(
    created_at: &str,
    hour: i64,
    minute: i64,
    time_zone: &str,
    now_utc: DateTime<Utc>,
    metadata: &Value,
) -> Result<String> {
    validate_timestamp(created_at)?;
    let created_at = DateTime::parse_from_rfc3339(created_at)?.with_timezone(&Utc);
    let hour = hour.clamp(0, 23) as u32;
    let minute = minute.clamp(0, 59) as u32;
    let cadence = issue_schedule_cadence(metadata)?;
    match (
        normalize_issue_schedule_time_zone(time_zone)?.as_str(),
        cadence.as_str(),
    ) {
        ("utc", "daily") => {
            issue_schedule_next_scheduled_slot_utc(created_at, now_utc, hour, minute)
        }
        ("local", "daily") => {
            issue_schedule_next_scheduled_slot_local(created_at, now_utc, hour, minute)
        }
        ("utc", "weekly") => issue_schedule_next_weekly_slot_utc(
            created_at,
            now_utc,
            hour,
            minute,
            issue_schedule_weekday_number(metadata)?,
        ),
        ("local", "weekly") => issue_schedule_next_weekly_slot_local(
            created_at,
            now_utc,
            hour,
            minute,
            issue_schedule_weekday_number(metadata)?,
        ),
        _ => unreachable!("time zone normalized above"),
    }
}

fn issue_schedule_next_scheduled_slot_utc(
    created_at: DateTime<Utc>,
    now_utc: DateTime<Utc>,
    hour: u32,
    minute: u32,
) -> Result<String> {
    let start = created_at.max(now_utc);
    let mut date = start.date_naive();
    for _ in 0..=370 {
        let Some(candidate) = Utc
            .with_ymd_and_hms(date.year(), date.month(), date.day(), hour, minute, 0)
            .single()
        else {
            bail!("constructing UTC issue schedule slot failed");
        };
        if candidate > now_utc && candidate >= created_at {
            return Ok(candidate.to_rfc3339());
        }
        let Some(next) = date.succ_opt() else {
            break;
        };
        date = next;
    }
    bail!("no future UTC issue schedule slot found within 370 days")
}

fn issue_schedule_next_scheduled_slot_local(
    created_at: DateTime<Utc>,
    now_utc: DateTime<Utc>,
    hour: u32,
    minute: u32,
) -> Result<String> {
    let start_local = created_at.max(now_utc).with_timezone(&Local);
    let mut date = start_local.date_naive();
    for _ in 0..=370 {
        let local_slot =
            match Local.with_ymd_and_hms(date.year(), date.month(), date.day(), hour, minute, 0) {
                chrono::LocalResult::Single(value) => Some(value),
                chrono::LocalResult::Ambiguous(earliest, _) => Some(earliest),
                chrono::LocalResult::None => None,
            };
        if let Some(local_slot) = local_slot {
            let candidate = local_slot.with_timezone(&Utc);
            if candidate > now_utc && candidate >= created_at {
                return Ok(candidate.to_rfc3339());
            }
        }
        let Some(next) = date.succ_opt() else {
            break;
        };
        date = next;
    }
    bail!("no future local issue schedule slot found within 370 days")
}

pub(crate) fn issue_schedule_due_slots_utc(
    start: DateTime<Utc>,
    now_utc: DateTime<Utc>,
    hour: u32,
    minute: u32,
) -> Result<Vec<DateTime<Utc>>> {
    let mut date = start.date_naive();
    let end_date = now_utc.date_naive();
    let mut slots = Vec::new();
    while date <= end_date {
        let Some(candidate) = Utc
            .with_ymd_and_hms(date.year(), date.month(), date.day(), hour, minute, 0)
            .single()
        else {
            bail!("constructing UTC issue schedule slot failed");
        };
        if candidate >= start && candidate <= now_utc {
            slots.push(candidate);
        }
        let Some(next) = date.succ_opt() else {
            break;
        };
        date = next;
    }
    Ok(slots)
}

pub(crate) fn issue_schedule_weekly_due_slots_utc(
    start: DateTime<Utc>,
    now_utc: DateTime<Utc>,
    hour: u32,
    minute: u32,
    weekday: u32,
) -> Result<Vec<DateTime<Utc>>> {
    let mut date = start.date_naive();
    let end_date = now_utc.date_naive();
    let mut slots = Vec::new();
    while date <= end_date {
        if date.weekday().num_days_from_monday() == weekday {
            let Some(candidate) = Utc
                .with_ymd_and_hms(date.year(), date.month(), date.day(), hour, minute, 0)
                .single()
            else {
                bail!("constructing UTC issue schedule slot failed");
            };
            if candidate >= start && candidate <= now_utc {
                slots.push(candidate);
            }
        }
        let Some(next) = date.succ_opt() else {
            break;
        };
        date = next;
    }
    Ok(slots)
}

pub(crate) fn issue_schedule_due_slots_local(
    start: DateTime<Utc>,
    now_utc: DateTime<Utc>,
    hour: u32,
    minute: u32,
) -> Result<Vec<DateTime<Utc>>> {
    let start_local = start.with_timezone(&Local);
    let now_local = now_utc.with_timezone(&Local);
    let mut date = start_local.date_naive();
    let end_date = now_local.date_naive();
    let mut slots = Vec::new();
    while date <= end_date {
        let local_slot =
            match Local.with_ymd_and_hms(date.year(), date.month(), date.day(), hour, minute, 0) {
                chrono::LocalResult::Single(value) => Some(value),
                chrono::LocalResult::Ambiguous(earliest, _) => Some(earliest),
                chrono::LocalResult::None => None,
            };
        if let Some(local_slot) = local_slot {
            let candidate = local_slot.with_timezone(&Utc);
            if candidate >= start && candidate <= now_utc {
                slots.push(candidate);
            }
        }
        let Some(next) = date.succ_opt() else {
            break;
        };
        date = next;
    }
    Ok(slots)
}

pub(crate) fn issue_schedule_weekly_due_slots_local(
    start: DateTime<Utc>,
    now_utc: DateTime<Utc>,
    hour: u32,
    minute: u32,
    weekday: u32,
) -> Result<Vec<DateTime<Utc>>> {
    let start_local = start.with_timezone(&Local);
    let now_local = now_utc.with_timezone(&Local);
    let mut date = start_local.date_naive();
    let end_date = now_local.date_naive();
    let mut slots = Vec::new();
    while date <= end_date {
        if date.weekday().num_days_from_monday() == weekday {
            let local_slot = match Local.with_ymd_and_hms(
                date.year(),
                date.month(),
                date.day(),
                hour,
                minute,
                0,
            ) {
                chrono::LocalResult::Single(value) => Some(value),
                chrono::LocalResult::Ambiguous(earliest, _) => Some(earliest),
                chrono::LocalResult::None => None,
            };
            if let Some(local_slot) = local_slot {
                let candidate = local_slot.with_timezone(&Utc);
                if candidate >= start && candidate <= now_utc {
                    slots.push(candidate);
                }
            }
        }
        let Some(next) = date.succ_opt() else {
            break;
        };
        date = next;
    }
    Ok(slots)
}

fn issue_schedule_next_weekly_slot_utc(
    created_at: DateTime<Utc>,
    now_utc: DateTime<Utc>,
    hour: u32,
    minute: u32,
    weekday: u32,
) -> Result<String> {
    let start = created_at.max(now_utc);
    let mut date = start.date_naive();
    for _ in 0..=370 {
        if date.weekday().num_days_from_monday() == weekday {
            let Some(candidate) = Utc
                .with_ymd_and_hms(date.year(), date.month(), date.day(), hour, minute, 0)
                .single()
            else {
                bail!("constructing UTC issue schedule slot failed");
            };
            if candidate > now_utc && candidate >= created_at {
                return Ok(candidate.to_rfc3339());
            }
        }
        let Some(next) = date.succ_opt() else {
            break;
        };
        date = next;
    }
    bail!("no future UTC weekly issue schedule slot found within 370 days")
}

fn issue_schedule_next_weekly_slot_local(
    created_at: DateTime<Utc>,
    now_utc: DateTime<Utc>,
    hour: u32,
    minute: u32,
    weekday: u32,
) -> Result<String> {
    let start_local = created_at.max(now_utc).with_timezone(&Local);
    let mut date = start_local.date_naive();
    for _ in 0..=370 {
        if date.weekday().num_days_from_monday() == weekday {
            let local_slot = match Local.with_ymd_and_hms(
                date.year(),
                date.month(),
                date.day(),
                hour,
                minute,
                0,
            ) {
                chrono::LocalResult::Single(value) => Some(value),
                chrono::LocalResult::Ambiguous(earliest, _) => Some(earliest),
                chrono::LocalResult::None => None,
            };
            if let Some(local_slot) = local_slot {
                let candidate = local_slot.with_timezone(&Utc);
                if candidate > now_utc && candidate >= created_at {
                    return Ok(candidate.to_rfc3339());
                }
            }
        }
        let Some(next) = date.succ_opt() else {
            break;
        };
        date = next;
    }
    bail!("no future local weekly issue schedule slot found within 370 days")
}

pub(crate) fn issue_schedule_day_label(due_at: &str) -> String {
    DateTime::parse_from_rfc3339(due_at)
        .map(|value| value.date_naive().to_string())
        .unwrap_or_else(|_| Utc::now().date_naive().to_string())
}

pub(crate) fn daily_briefing_wiki_queries(
    report: &KnowledgeReport,
    source_cards: &[SourceCard],
) -> Vec<String> {
    let mut queries = Vec::new();
    for candidate in [
        report
            .title
            .trim()
            .strip_prefix("Daily Knowledge Report:")
            .unwrap_or(report.title.trim())
            .trim()
            .to_string(),
        report.title.trim().to_string(),
    ] {
        if !candidate.is_empty() {
            queries.push(excerpt(&candidate, 180));
        }
    }
    for card in source_cards.iter().take(3) {
        queries.push(excerpt(&card.title, 180));
        let summary_terms = card
            .summary
            .split_whitespace()
            .filter(|token| token.len() > 3)
            .take(12)
            .collect::<Vec<_>>()
            .join(" ");
        if !summary_terms.trim().is_empty() {
            queries.push(excerpt(&summary_terms, 180));
        }
    }
    let mut seen = BTreeSet::new();
    queries
        .into_iter()
        .filter_map(|query| {
            let query = query.trim().to_string();
            if query.is_empty() || !seen.insert(query.clone()) {
                None
            } else {
                Some(query)
            }
        })
        .collect()
}

pub(crate) fn daily_briefing_wiki_page_is_story_context(page: &WikiPageSummary) -> bool {
    let title = page.title.to_ascii_lowercase();
    let source = page.source.to_ascii_lowercase();
    !title.starts_with("source card:")
        && !source.starts_with("source-card:")
        && !is_generated_wiki_page(&page.title)
}

pub(crate) fn render_knowledge_daily_briefing(
    schedule: &IssueSchedule,
    tick: &IssueScheduleTick,
    reports: &[KnowledgeReport],
    source_cards: &[SourceCard],
    window_start: &str,
    window_end: &str,
    related_wiki_pages: &BTreeMap<String, Vec<WikiPageSummary>>,
) -> String {
    let day = issue_schedule_day_label(&tick.due_at);
    let window = daily_briefing_window(window_start, window_end);
    let window_label = daily_briefing_window_label(window.as_ref());
    let weekly_overview = issue_schedule_is_weekly_overview(schedule);
    let max_stories = schedule
        .metadata
        .get("max_stories")
        .and_then(Value::as_u64)
        .map(|value| value as usize)
        .unwrap_or(if weekly_overview { 8 } else { 5 })
        .clamp(1, 12);
    let stories = daily_briefing_reader_stories(reports, source_cards, window, max_stories);
    let mut lines = vec![
        issue_schedule_reader_heading(schedule, &day),
        String::new(),
        "## Bottom Line".to_string(),
        daily_briefing_lede_for_stories(&stories, &window_label),
        String::new(),
    ];
    if stories.is_empty() {
        lines.push("## Quiet Day".to_string());
        lines.push(format!("The {window_label} scan was not empty; it just did not produce a clean story. The apparent activity was old feed items, reply-level social chatter, and routine code-hosting updates. None of that changes the picture for model access, agents, evaluation, or developer workflow on its own."));
        lines.push(String::new());
    } else {
        lines.push(if weekly_overview {
            "## Big Stories".to_string()
        } else {
            "## Today's Stories".to_string()
        });
    }
    for (index, story) in stories.iter().enumerate() {
        let report = story.report;
        let wiki_pages = related_wiki_pages
            .get(&report.id)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        lines.push(format!("### {}. {}", index + 1, story.title));
        lines.push(story.body.clone());
        lines.push(String::new());
        if weekly_overview {
            lines.push("#### Development This Week".to_string());
            lines.push(weekly_overview_story_development(story, &window_label));
            lines.push(String::new());
        }
        if let Some(prior_context_insight) =
            daily_briefing_prior_context_insight(report, &story.source_cards, wiki_pages)
        {
            lines.push("#### Context".to_string());
            lines.push(prior_context_insight);
            lines.push(String::new());
        }
        lines.push("#### Further Reading".to_string());
        lines.extend(daily_briefing_key_source_lines(&story.source_cards, 2));
        lines.push(String::new());
    }
    lines.push(if weekly_overview {
        "## End-of-Week Read".to_string()
    } else {
        "## Editor's Read".to_string()
    });
    lines.push(if weekly_overview {
        weekly_overview_issue_read(&stories, &window_label)
    } else {
        daily_briefing_issue_read(&stories, &window_label)
    });
    lines.push(String::new());
    lines.push(if weekly_overview {
        "## What Carries Into Next Week".to_string()
    } else {
        "## Watch Next".to_string()
    });
    lines.push(if weekly_overview {
        weekly_overview_watch_next(&stories, &window_label)
    } else {
        daily_briefing_watch_next(&stories, &window_label)
    });
    lines.join("\n")
}

struct DailyBriefingReaderStory<'a> {
    report: &'a KnowledgeReport,
    title: String,
    body: String,
    source_cards: Vec<&'a SourceCard>,
}

pub(crate) fn issue_schedule_is_weekly_overview(schedule: &IssueSchedule) -> bool {
    issue_schedule_cadence(&schedule.metadata)
        .map(|cadence| cadence == "weekly")
        .unwrap_or(false)
        || schedule
            .metadata
            .get("issue_format")
            .and_then(Value::as_str)
            .is_some_and(|value| {
                matches!(
                    value.trim().to_ascii_lowercase().as_str(),
                    "weekly_overview" | "week_overview"
                )
            })
}

pub(crate) fn issue_schedule_reader_label(schedule: &IssueSchedule) -> &'static str {
    if issue_schedule_is_weekly_overview(schedule) {
        "weekly overview"
    } else {
        "daily briefing"
    }
}

pub(crate) fn issue_schedule_reader_heading(schedule: &IssueSchedule, day: &str) -> String {
    if issue_schedule_is_weekly_overview(schedule) {
        let title = schedule
            .metadata
            .get("issue_title")
            .and_then(Value::as_str)
            .unwrap_or("AI Week Overview")
            .trim();
        return format!("# {} - Week ending {}", excerpt(title, 80), day);
    }
    format!("# AI Daily Briefing - {day}")
}

pub(crate) fn daily_briefing_window(
    window_start: &str,
    window_end: &str,
) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    let start = DateTime::parse_from_rfc3339(window_start)
        .ok()?
        .with_timezone(&Utc);
    let end = DateTime::parse_from_rfc3339(window_end)
        .ok()?
        .with_timezone(&Utc);
    Some((start, end))
}

pub(crate) fn daily_briefing_window_label(
    window: Option<&(DateTime<Utc>, DateTime<Utc>)>,
) -> String {
    let Some((start, end)) = window else {
        return "current window".to_string();
    };
    let seconds = (*end - *start).num_seconds().max(1);
    let hours = ((seconds + 3_599) / 3_600).clamp(1, 24 * 14);
    if hours == 24 {
        "last 24 hours".to_string()
    } else {
        format!("last {hours} hours")
    }
}

fn daily_briefing_reader_stories<'a>(
    reports: &'a [KnowledgeReport],
    source_cards: &'a [SourceCard],
    window: Option<(DateTime<Utc>, DateTime<Utc>)>,
    max_stories: usize,
) -> Vec<DailyBriefingReaderStory<'a>> {
    let mut stories = Vec::new();
    let mut seen_titles = BTreeSet::new();
    for report in reports {
        let story_cards =
            daily_briefing_report_fresh_source_cards(report, source_cards, window.as_ref());
        if story_cards.is_empty() {
            continue;
        }
        let story_cards = daily_briefing_ranked_source_cards(&story_cards)
            .into_iter()
            .filter(|card| daily_briefing_source_card_is_reader_worthy(card))
            .collect::<Vec<_>>();
        if story_cards.is_empty() {
            continue;
        }
        if !daily_briefing_report_has_newsletter_story(report, &story_cards) {
            continue;
        }
        let title = daily_briefing_story_title(report, &story_cards);
        if !seen_titles.insert(title.to_ascii_lowercase()) {
            continue;
        }
        let body = daily_briefing_story_body(report, &story_cards);
        if daily_briefing_output_has_forbidden_reader_language(&title)
            || daily_briefing_output_has_forbidden_reader_language(&body)
        {
            continue;
        }
        stories.push(DailyBriefingReaderStory {
            report,
            title,
            body,
            source_cards: story_cards,
        });
        if stories.len() >= max_stories {
            break;
        }
    }
    stories
}

fn daily_briefing_lede_for_stories(
    stories: &[DailyBriefingReaderStory<'_>],
    window_label: &str,
) -> String {
    if stories.is_empty() {
        return daily_briefing_lede_for_titles_in_window(&[], window_label);
    }
    let titles = stories
        .iter()
        .map(|story| story.title.as_str())
        .collect::<Vec<_>>();
    daily_briefing_lede_for_titles_in_window(&titles, window_label)
}

pub(crate) fn daily_briefing_report_has_newsletter_story(
    report: &KnowledgeReport,
    source_cards: &[&SourceCard],
) -> bool {
    if source_cards.is_empty() {
        return false;
    }
    if daily_briefing_report_is_generated_storying_artifact(report) {
        if daily_briefing_source_cards_are_github_repo_only(source_cards) {
            return false;
        }
        let topic = daily_briefing_report_display_title(report);
        if daily_briefing_generated_bucket_title_is_not_reader_story(&topic) {
            return false;
        }
        if daily_briefing_title_entity(&topic).is_none() {
            return false;
        }
        if source_cards.len() > 25 {
            return false;
        }
        if !source_cards
            .iter()
            .all(|card| card.provider.eq_ignore_ascii_case("github"))
        {
            let readable_sources = daily_briefing_ranked_source_cards(source_cards)
                .into_iter()
                .filter(|card| daily_briefing_source_reader_score(card) >= 60)
                .count();
            let readable_non_reply_sources = daily_briefing_ranked_source_cards(source_cards)
                .into_iter()
                .filter(|card| {
                    daily_briefing_source_reader_score(card) >= 60
                        && !daily_briefing_source_takeaway_text(card, 500)
                            .to_ascii_lowercase()
                            .starts_with('@')
                })
                .count();
            if readable_sources < 2 || readable_non_reply_sources == 0 {
                return false;
            }
        }
    }
    true
}

pub(crate) fn daily_briefing_generated_bucket_title_is_not_reader_story(title: &str) -> bool {
    let lower = title.trim().to_ascii_lowercase();
    lower == "community reaction"
        || lower.ends_with(": community reaction")
        || lower == "ai usage practices"
        || lower.ends_with(": ai usage practices")
        || lower == "repository and package activity"
        || lower.ends_with(": repository and package activity")
        || lower == "source-backed updates"
        || lower.ends_with(": source-backed updates")
}

pub(crate) fn daily_briefing_source_cards_are_github_repo_only(
    source_cards: &[&SourceCard],
) -> bool {
    source_cards.iter().all(|card| {
        card.provider.eq_ignore_ascii_case("github")
            && card.source_type.eq_ignore_ascii_case("github_repo")
    })
}

fn daily_briefing_issue_read(
    stories: &[DailyBriefingReaderStory<'_>],
    window_label: &str,
) -> String {
    if stories.is_empty() {
        return format!(
            "The useful conclusion is negative: nothing fresh and well-sourced from the {window_label} changed the picture."
        );
    }
    "The useful read is whether the primary links are backed by docs, releases, benchmarks, or credible developer use.".to_string()
}

fn weekly_overview_story_development(
    story: &DailyBriefingReaderStory<'_>,
    window_label: &str,
) -> String {
    let dates = story
        .source_cards
        .iter()
        .filter_map(|card| source_card_issue_date(card))
        .collect::<BTreeSet<_>>();
    match (dates.iter().next(), dates.iter().next_back(), dates.len()) {
        (Some(first), Some(last), count) if count > 1 => format!(
            "The saved evidence spans {first} to {last} across {count} dated source points, so this reads as a developing thread across the {window_label}."
        ),
        (Some(date), _, _) => format!(
            "The saved evidence is concentrated on {date}; treat it as this week's clearest update on the thread, not proof of a multi-day arc by itself."
        ),
        _ => format!(
            "The saved evidence sits inside the {window_label}, but its source timestamps are too weak to claim a detailed week-long progression."
        ),
    }
}

fn source_card_issue_date(card: &SourceCard) -> Option<String> {
    DateTime::parse_from_rfc3339(&card.retrieved_at)
        .or_else(|_| DateTime::parse_from_rfc3339(&card.created_at))
        .ok()
        .map(|value| value.with_timezone(&Utc).date_naive().to_string())
}

fn weekly_overview_issue_read(
    stories: &[DailyBriefingReaderStory<'_>],
    window_label: &str,
) -> String {
    if stories.is_empty() {
        return format!(
            "The end-of-week read is negative: nothing fresh and well-sourced from the {window_label} rose to a big-story threshold."
        );
    }
    let titles = stories
        .iter()
        .take(4)
        .map(|story| story.title.as_str())
        .collect::<Vec<_>>()
        .join("; ");
    format!(
        "The big read for the {window_label} is the shape across these threads: {titles}. The useful question is which ones turn into shipped docs, pricing, benchmarks, or durable developer adoption next."
    )
}

fn daily_briefing_watch_next(
    stories: &[DailyBriefingReaderStory<'_>],
    window_label: &str,
) -> String {
    if stories.is_empty() {
        return format!(
            "The next real issue should start from a primary announcement, release note, benchmark, shipping detail, or credible developer reaction dated inside the {window_label}."
        );
    }
    "Check official release notes or docs first, then look for independent developer use before promoting any item into a trend.".to_string()
}

fn weekly_overview_watch_next(
    stories: &[DailyBriefingReaderStory<'_>],
    window_label: &str,
) -> String {
    if stories.is_empty() {
        return format!(
            "Next week's read should start from primary announcements, release notes, benchmark movement, or credible developer use dated after this quiet {window_label}."
        );
    }
    "Carry forward only the threads that gain primary-source follow-through, shipped developer surface area, or credible adoption evidence.".to_string()
}

pub(crate) fn daily_briefing_lede_for_titles_in_window(
    titles: &[&str],
    window_label: &str,
) -> String {
    match titles {
        [] => format!(
            "Quiet day. The {window_label} had activity, but not a clean AI story worth elevating; old feed items, reply-level social chatter, and routine code-hosting updates did not clear the bar."
        ),
        [lead] => format!(
            "The cleanest item today is {lead}. I would treat it as a story to watch, not a settled claim, and read it for what the evidence actually changes."
        ),
        [lead, second] => format!(
            "Today is a watchlist issue: {lead} and {second} are the strongest items from the {window_label}. The interesting part is what gets backed by docs, releases, benchmarks, or real developer use."
        ),
        [lead, second, third, ..] => format!(
            "Today is a watchlist issue: {lead}, {second}, and {third} are the strongest items from the {window_label}. The useful tension is what gets backed by docs, releases, benchmarks, or real developer use."
        ),
    }
}

pub(crate) fn daily_briefing_report_display_title(report: &KnowledgeReport) -> String {
    let mut title = report.title.trim();
    for prefix in [
        "Daily Knowledge Report:",
        "Knowledge Report:",
        "Knowledge Cluster Expansion:",
        "Model-Written Knowledge Cluster Expansion:",
    ] {
        if let Some(stripped) = title.strip_prefix(prefix) {
            title = stripped.trim();
            break;
        }
    }
    daily_briefing_humanize_title(title)
}

pub(crate) fn daily_briefing_humanize_title(title: &str) -> String {
    let mut out = title.trim().to_string();
    for (from, to) in [
        (" Ai", " AI"),
        ("Openai", "OpenAI"),
        ("Qwenlm", "QwenLM"),
        ("Moonshotai", "MoonshotAI"),
    ] {
        out = out.replace(from, to);
    }
    out
}

pub(crate) fn daily_briefing_story_title(
    report: &KnowledgeReport,
    source_cards: &[&SourceCard],
) -> String {
    let topic = daily_briefing_report_display_title(report);
    if source_cards
        .iter()
        .all(|card| card.provider.eq_ignore_ascii_case("github"))
    {
        let repo_names = source_cards
            .iter()
            .map(|card| daily_briefing_source_label(card))
            .filter(|label| label.contains('/'))
            .map(|label| {
                label
                    .split('/')
                    .next_back()
                    .unwrap_or(label.as_str())
                    .trim()
                    .to_string()
            })
            .filter(|label| !label.is_empty())
            .take(3)
            .collect::<Vec<_>>();
        if !repo_names.is_empty() {
            if let Some(entity) = daily_briefing_title_entity(&topic) {
                return entity;
            }
        }
    }
    daily_briefing_title_entity(&topic).unwrap_or(topic)
}

fn daily_briefing_title_entity(title: &str) -> Option<String> {
    let mut topic = title.trim();
    for suffix in [
        ": release and launch activity",
        ": model release activity",
        ": benchmarks and evaluation",
        ": MCP and agent infrastructure",
        ": repository and package activity",
        ": community reaction",
        ": AI usage practices",
    ] {
        if let Some(stripped) = topic.strip_suffix(suffix) {
            topic = stripped.trim();
            break;
        }
    }
    if topic.is_empty()
        || topic.eq_ignore_ascii_case("release and launch activity")
        || topic.eq_ignore_ascii_case("model release activity")
        || topic.eq_ignore_ascii_case("community reaction")
        || topic.eq_ignore_ascii_case("repository and package activity")
        || topic.eq_ignore_ascii_case("ai usage practices")
        || topic.eq_ignore_ascii_case("benchmarks and evaluation")
        || topic.eq_ignore_ascii_case("mcp and agent infrastructure")
    {
        None
    } else {
        Some(topic.to_string())
    }
}

pub(crate) fn daily_briefing_report_source_cards<'a>(
    report: &KnowledgeReport,
    source_cards: &'a [SourceCard],
) -> Vec<&'a SourceCard> {
    let ids = report.source_card_ids.iter().collect::<BTreeSet<_>>();
    source_cards
        .iter()
        .filter(|card| ids.contains(&card.id) && !is_generated_source_card(card))
        .collect()
}

pub(crate) fn daily_briefing_report_fresh_source_cards<'a>(
    report: &KnowledgeReport,
    source_cards: &'a [SourceCard],
    window: Option<&(DateTime<Utc>, DateTime<Utc>)>,
) -> Vec<&'a SourceCard> {
    daily_briefing_report_source_cards(report, source_cards)
        .into_iter()
        .filter(|card| daily_briefing_source_card_is_in_window(card, window))
        .collect()
}

pub(crate) fn daily_briefing_source_card_is_in_window(
    card: &SourceCard,
    window: Option<&(DateTime<Utc>, DateTime<Utc>)>,
) -> bool {
    let Some((start, end)) = window else {
        return true;
    };
    let Some(timestamp) = daily_briefing_source_card_evidence_time(card) else {
        return true;
    };
    timestamp >= *start && timestamp <= *end
}

pub(crate) fn daily_briefing_source_card_evidence_time(card: &SourceCard) -> Option<DateTime<Utc>> {
    for value in [
        daily_briefing_source_metadata_string(card, "published_at"),
        daily_briefing_source_metadata_string(card, "pushed_at"),
        daily_briefing_source_metadata_string(card, "created_at"),
        daily_briefing_source_metadata_string(card, "updated_at"),
        Some(card.retrieved_at.clone()),
    ]
    .into_iter()
    .flatten()
    {
        if let Some(parsed) = daily_briefing_parse_source_time(value.trim()) {
            return Some(parsed);
        }
    }
    None
}

pub(crate) fn daily_briefing_parse_source_time(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .or_else(|_| DateTime::parse_from_rfc2822(value))
        .ok()
        .map(|parsed| parsed.with_timezone(&Utc))
}

pub(crate) fn daily_briefing_story_body(
    report: &KnowledgeReport,
    source_cards: &[&SourceCard],
) -> String {
    if daily_briefing_report_is_generated_storying_artifact(report)
        || daily_briefing_report_body_is_projection_boilerplate(&report.body_markdown)
    {
        return daily_briefing_source_card_story(report, source_cards);
    }
    let source_ids = report.source_card_ids.iter().collect::<Vec<_>>();
    let mut lines = Vec::new();
    let mut skip_internal_list = false;
    let mut skip_internal_section = false;
    for line in report.body_markdown.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_ascii_lowercase();
        if lower.starts_with("## ") || lower.starts_with("# ") {
            skip_internal_section = false;
        }
        if daily_briefing_is_internal_reader_section(&lower) {
            skip_internal_section = true;
            continue;
        }
        if lower == "source_cards:"
            || lower == "cluster_links:"
            || lower == "filed evidence:"
            || lower == "sources:"
            || lower == "evidence appendix:"
            || lower.starts_with("filed evidence:")
        {
            skip_internal_list = true;
            continue;
        }
        if lower.starts_with('#') {
            continue;
        }
        if skip_internal_section {
            continue;
        }
        if skip_internal_list
            && (trimmed.starts_with("- `")
                || trimmed.starts_with("- src-")
                || trimmed.starts_with("- [")
                || trimmed.chars().next().is_some_and(|c| c.is_ascii_digit()))
        {
            continue;
        }
        skip_internal_list = false;
        if lower.starts_with("the last 24 hours did not produce") {
            continue;
        }
        if lower.starts_with("relationship to earlier wiki context:") {
            continue;
        }
        let Some(clean) = daily_briefing_reader_line(line, &source_ids) else {
            continue;
        };
        if !clean.trim().is_empty() {
            lines.push(clean);
        }
        if lines.len() >= 24 {
            break;
        }
    }
    let body = lines.join("\n");
    if body.trim().len() < 160 {
        daily_briefing_source_card_story(report, source_cards)
    } else {
        excerpt_preserving_whitespace(&body, 1_600)
    }
}

pub(crate) fn daily_briefing_report_is_generated_storying_artifact(
    report: &KnowledgeReport,
) -> bool {
    let title = report.title.to_ascii_lowercase();
    if title.starts_with("knowledge cluster expansion:")
        || title.starts_with("model-written knowledge cluster expansion:")
    {
        return true;
    }
    let origin = report
        .metadata
        .get("origin")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    matches!(
        origin.as_str(),
        "knowledge_cluster_editor_v1"
            | "knowledge_cluster_model_writer_v1"
            | "deterministic_source_card_projection_v1"
            | "source_card_backlog"
            | "source_card_backlog_clustering"
    )
}

pub(crate) fn daily_briefing_report_body_is_projection_boilerplate(markdown: &str) -> bool {
    let lower = markdown.to_ascii_lowercase();
    [
        "knowledge cluster expansion",
        "cluster:",
        "source family:",
        "proof level:",
        "scores: novelty",
        "first seen:",
        "last seen:",
        "source_card_backlog_storying",
        "the system expanded this shared knowledge story",
        "arcwell expanded this shared knowledge cluster",
        "durable sources across",
        "provider buckets",
        "unified evidence-backed knowledge story",
        "unified knowledge system",
        "durable source-card rows into the unified knowledge pipeline",
        "durable source rows into the unified knowledge pipeline",
        "provider family buckets",
        "stored as source-card ids",
        "stored as source references",
        "worth keeping on the working map because",
        "official or primary-style sources give the topic a factual starting point",
        "official or primary-style source give the topic a factual starting point",
        "independent reaction still needs to be checked",
        "this page is a working note",
        "first bridge between the existing live/captured ingestion machinery",
        "source-agnostic knowledge substrate",
        "primary-source-style rows",
        "github repositories detected",
        "external domains detected",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

pub(crate) fn daily_briefing_source_card_story(
    report: &KnowledgeReport,
    source_cards: &[&SourceCard],
) -> String {
    let topic = daily_briefing_story_title(report, source_cards);
    if source_cards.is_empty() {
        return format!(
            "There is not enough readable evidence attached to {topic} to make this a story. Hold stronger claims until the next scan has a source people can inspect."
        );
    }
    let source_summary = daily_briefing_source_summary_clause(source_cards);
    let angle = daily_briefing_interpretive_angle(report, source_cards);
    if source_cards
        .iter()
        .all(|card| card.provider.eq_ignore_ascii_case("github"))
    {
        return format!(
            "The GitHub links point to {source_summary}. Read that narrowly: they show code movement, not a launch, benchmark result, or adoption trend. The next test is whether docs, release notes, or developers connect it to a shipped change. My read for now: {angle}."
        );
    }
    format!(
        "{topic}: {source_summary}. Read it as {angle}. This is still early evidence; I want primary docs, release notes, benchmarks, or credible developer use before treating it as settled."
    )
}

pub(crate) fn daily_briefing_source_summary_clause(source_cards: &[&SourceCard]) -> String {
    let ranked = daily_briefing_ranked_source_cards(source_cards);
    let names = ranked
        .iter()
        .take(3)
        .map(|card| {
            let label = daily_briefing_source_label(card).replace(" on X", "");
            let summary = daily_briefing_source_takeaway_text(card, 170);
            if summary.is_empty() {
                label
            } else {
                format!("{label}: {summary}")
            }
        })
        .collect::<Vec<_>>();
    if names.is_empty() {
        "too thin to interpret".to_string()
    } else if source_cards
        .iter()
        .all(|card| card.provider.eq_ignore_ascii_case("github"))
    {
        format!("GitHub links for {}", human_join_strings(&names))
    } else {
        names.join("; ")
    }
}

pub(crate) fn daily_briefing_ranked_source_cards<'a>(
    source_cards: &[&'a SourceCard],
) -> Vec<&'a SourceCard> {
    let mut ranked = source_cards.to_vec();
    ranked.sort_by(|left, right| {
        daily_briefing_source_reader_score(right)
            .cmp(&daily_briefing_source_reader_score(left))
            .then_with(|| right.retrieved_at.cmp(&left.retrieved_at))
            .then_with(|| left.id.cmp(&right.id))
    });
    ranked
}

pub(crate) fn daily_briefing_source_reader_score(card: &SourceCard) -> i32 {
    let text = daily_briefing_source_takeaway_text(card, 500);
    let lower = text.to_ascii_lowercase();
    if text.trim().is_empty() {
        return -100;
    }
    let mut score = text.chars().count().min(220) as i32;
    if lower.starts_with('@') {
        score -= 45;
    }
    if lower.starts_with("watch the full interview here")
        || lower.contains("thank you for joining")
        || lower.contains("hell yeah")
        || lower.contains("you lied")
        || lower.trim().chars().count() <= 24
    {
        score -= 80;
    }
    for needle in [
        "released",
        "launch",
        "docs",
        "benchmark",
        "ships",
        "deploy",
        "available",
        "report",
        "verification",
        "governance",
        "revenue",
        "model",
        "agent",
    ] {
        if lower.contains(needle) {
            score += 18;
        }
    }
    score
}

pub(crate) fn daily_briefing_source_card_is_reader_worthy(card: &SourceCard) -> bool {
    let text = daily_briefing_source_takeaway_text(card, 500);
    let lower = text.to_ascii_lowercase();
    if text.trim().is_empty() {
        return false;
    }
    if daily_briefing_source_text_is_low_signal(&lower) {
        return false;
    }
    if daily_briefing_source_card_is_social(card) {
        if daily_briefing_source_text_is_reply(&lower) {
            return false;
        }
        let char_count = text.chars().count();
        if char_count < 110 {
            return false;
        }
        return daily_briefing_source_text_has_substantive_signal(&lower)
            && daily_briefing_source_reader_score(card) >= 90;
    }
    daily_briefing_source_reader_score(card) >= 50
}

pub(crate) fn daily_briefing_source_card_is_social(card: &SourceCard) -> bool {
    card.provider.eq_ignore_ascii_case("x")
        || card.provider.eq_ignore_ascii_case("twitter")
        || card.source_type.eq_ignore_ascii_case("x")
        || card.source_type.eq_ignore_ascii_case("x_tweet")
        || daily_briefing_source_metadata_string(card, "source_owner")
            .is_some_and(|owner| owner.eq_ignore_ascii_case("x.com"))
}

pub(crate) fn daily_briefing_source_text_is_reply(lower_text: &str) -> bool {
    let trimmed = lower_text.trim_start();
    trimmed.starts_with('@') || trimmed.starts_with("rt @")
}

pub(crate) fn daily_briefing_source_text_is_low_signal(lower_text: &str) -> bool {
    let trimmed = lower_text.trim();
    trimmed.chars().count() <= 40
        || trimmed.starts_with("watch the full interview here")
        || trimmed.contains("thank you for joining")
        || trimmed.contains("thanks for joining")
        || trimmed.contains("hell yeah")
        || trimmed.contains("you lied")
        || trimmed.contains("congrats on your marriage")
        || trimmed == "more tomorrow"
}

pub(crate) fn daily_briefing_source_text_has_substantive_signal(lower_text: &str) -> bool {
    [
        "announced",
        "released",
        "launched",
        "available",
        "benchmark",
        "evaluation",
        "report",
        "revenue",
        "pricing",
        "deploy",
        "deployment",
        "docs",
        "api",
        "sdk",
        "model",
        "agent",
        "integration",
        "workflow",
        "verification",
        "governance",
        "inference",
        "compute",
    ]
    .iter()
    .any(|needle| lower_text.contains(needle))
}

pub(crate) fn daily_briefing_is_internal_reader_section(lower: &str) -> bool {
    [
        "## bookmark completeness proof",
        "## failures and incompleteness",
        "## recommended follow-up",
        "## what was ingested",
        "## coverage and uncertainty",
        "## evidence notes",
        "## evidence pattern",
        "## additional source index",
        "## next investigation",
        "## editorial next steps",
        "## follow-up research",
        "## follow up research",
        "## confidence and uncertainty",
        "## warnings",
        "## sources",
        "# sources",
        "coverage and uncertainty",
        "evidence notes",
        "evidence pattern",
        "additional source index",
        "next investigation",
        "editorial next steps",
        "follow-up research",
        "follow up research",
        "confidence and uncertainty",
        "warnings",
        "sources",
    ]
    .iter()
    .any(|prefix| lower.starts_with(prefix))
}

pub(crate) fn daily_briefing_reader_line(line: &str, source_ids: &[&String]) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("source text is untrusted evidence")
        || lower.starts_with("operationally,")
        || lower.starts_with("the system projected")
        || lower.starts_with("arcwell projected")
        || lower.contains("local audit ledger")
        || lower.contains("digest delivery gate")
        || lower.contains("approved candidate")
        || lower.contains("candidate id")
        || lower.contains("digest candidate")
        || lower.contains("new wiki page is knowledge:")
        || lower.contains("durable source rows")
        || lower.contains("durable source-card rows")
        || lower.contains("unified knowledge pipeline")
        || lower.contains("provider family buckets")
        || lower.contains("primary-source-style rows")
        || lower.contains("github repositories detected")
        || lower.contains("external domains detected")
    {
        return None;
    }
    if lower.starts_with("uncertainty:") {
        return daily_briefing_reader_caveat(trimmed);
    }
    let without_ids = strip_source_card_ids_for_reader(line, source_ids);
    let clean = daily_briefing_rewrite_reader_language(&without_ids);
    if daily_briefing_contains_forbidden_reader_language(&clean) {
        return None;
    }
    Some(clean)
}

pub(crate) fn daily_briefing_reader_caveat(line: &str) -> Option<String> {
    let lower = line.to_ascii_lowercase();
    let caveat = if lower.contains("not a new-launch") || lower.contains("not a new launch") {
        "Caveat: this is not a formal launch signal; treat it as directional product or repository momentum until a primary announcement, availability details, or independent adoption evidence appears."
    } else if lower.contains("github") || lower.contains("repo") || lower.contains("repository") {
        "Caveat: this mostly comes from repository activity, not a formal announcement or proof of adoption."
    } else if lower.contains("older") || lower.contains("backlog") {
        "Caveat: some of this material predates the briefing window; it is context for a developing theme, not fresh overnight news."
    } else if daily_briefing_contains_forbidden_reader_language(line) {
        "Caveat: the public evidence is incomplete, so treat this as a developing signal rather than a settled claim."
    } else {
        let text = line
            .split_once(':')
            .map(|(_, rest)| rest.trim())
            .unwrap_or(line)
            .trim();
        if text.is_empty() {
            return None;
        }
        return Some(format!(
            "Caveat: {}",
            daily_briefing_rewrite_reader_language(text)
        ));
    };
    Some(caveat.to_string())
}

pub(crate) fn daily_briefing_key_source_lines(
    source_cards: &[&SourceCard],
    max_sources: usize,
) -> Vec<String> {
    let mut lines = Vec::new();
    let ranked = daily_briefing_ranked_source_cards(source_cards);
    for card in ranked.iter().take(max_sources) {
        lines.push(format!(
            "- [{}]({}) - {}",
            escape_markdown_link_text(&daily_briefing_source_label(card)),
            card.url,
            daily_briefing_source_takeaway(card)
        ));
    }
    if lines.is_empty() {
        lines.push("- No public source link is available for this story; hold stronger claims until fresh evidence is attached.".to_string());
    }
    lines
}

pub(crate) fn daily_briefing_source_label(card: &SourceCard) -> String {
    daily_briefing_rewrite_reader_language(&knowledge_projection_source_label(card))
}

pub(crate) fn daily_briefing_source_takeaway(card: &SourceCard) -> String {
    let mut parts = Vec::new();
    let text = daily_briefing_source_takeaway_text(card, 180);
    if !text.is_empty() {
        parts.push(text);
    }
    if let Some(language) = daily_briefing_source_metadata_string(card, "language")
        && !language.eq_ignore_ascii_case("unknown")
    {
        parts.push(format!("{language}."));
    }
    if let Some(pushed_at) = daily_briefing_source_metadata_string(card, "pushed_at") {
        parts.push(format!(
            "Last pushed {}.",
            daily_briefing_date_only(&pushed_at)
        ));
    }
    if let Some(stars) = daily_briefing_source_metadata_u64(card, "stargazers_count") {
        parts.push(format!("{stars} stars."));
    }
    if parts.is_empty() {
        "Source available for inspection.".to_string()
    } else {
        parts.join(" ")
    }
}

pub(crate) fn daily_briefing_source_takeaway_text(card: &SourceCard, max_chars: usize) -> String {
    let candidates = vec![
        card.summary.trim().to_string(),
        source_card_metadata_string(&card.metadata, "description")
            .unwrap_or_default()
            .trim()
            .to_string(),
        Store::digest_card_evidence_text(card).trim().to_string(),
    ];
    for candidate in candidates {
        if candidate.is_empty()
            || candidate
                .to_ascii_lowercase()
                .ends_with("is a public github repository.")
            || candidate
                .to_ascii_lowercase()
                .starts_with("no repository description")
        {
            continue;
        }
        return excerpt(
            &daily_briefing_rewrite_reader_language(&strip_bare_urls(&html_unescape_basic(
                &candidate,
            ))),
            max_chars,
        );
    }
    String::new()
}

pub(crate) fn daily_briefing_source_metadata_string(
    card: &SourceCard,
    key: &str,
) -> Option<String> {
    source_card_metadata_string(&card.metadata, key).or_else(|| {
        card.metadata
            .get("raw")
            .and_then(|raw| raw.get(key))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    })
}

pub(crate) fn daily_briefing_source_metadata_u64(card: &SourceCard, key: &str) -> Option<u64> {
    card.metadata
        .get("raw")
        .and_then(|raw| raw.get(key))
        .and_then(Value::as_u64)
        .or_else(|| card.metadata.get(key).and_then(Value::as_u64))
}

pub(crate) fn daily_briefing_date_only(value: &str) -> String {
    value.split('T').next().unwrap_or(value).to_string()
}

pub(crate) fn daily_briefing_prior_context_insight(
    report: &KnowledgeReport,
    source_cards: &[&SourceCard],
    wiki_pages: &[WikiPageSummary],
) -> Option<String> {
    let explicit_context = extract_relationship_context_line(&report.body_markdown);
    let angle = daily_briefing_interpretive_angle(report, source_cards);
    if let Some(context) = explicit_context {
        return Some(format!(
            "This changes the earlier read rather than merely adding another matching link: {} The practical question is whether later sources confirm, narrow, or contradict that shift around {}.",
            daily_briefing_rewrite_reader_language(&strip_source_card_ids_for_reader(
                &context,
                &report.source_card_ids.iter().collect::<Vec<_>>()
            )),
            angle
        ));
    }
    if daily_briefing_report_is_generated_storying_artifact(report) {
        return None;
    }
    if !daily_briefing_has_prior_context_signal(report) {
        return None;
    }
    let titles = wiki_pages
        .iter()
        .take(3)
        .map(|page| daily_briefing_rewrite_reader_language(page.title.trim()))
        .collect::<Vec<_>>();
    let titles = titles
        .iter()
        .map(|title| title.trim())
        .filter(|title| !title.is_empty())
        .collect::<Vec<_>>();
    if titles.is_empty() {
        return None;
    }
    Some(format!(
        "Compared with {}, this update shifts the emphasis toward {}. The next check is whether the older framing was too broad, whether a previously secondary layer is becoming central, or whether later sources contradict the shift.",
        human_join(&titles),
        angle
    ))
}

pub(crate) fn daily_briefing_has_prior_context_signal(report: &KnowledgeReport) -> bool {
    let haystack = format!("{}\n{}", report.title, report.body_markdown).to_ascii_lowercase();
    [
        "prior notes",
        "earlier",
        "previously",
        "previous ",
        "standing assumption",
        "assumption",
        "contradict",
        "tension",
        "reframes",
        "reframe",
        "shifted",
        "shifts",
        "different than",
        "different from",
        "we learned",
        "turns out",
    ]
    .iter()
    .any(|needle| haystack.contains(needle))
}

pub(crate) fn extract_relationship_context_line(markdown: &str) -> Option<String> {
    markdown.lines().find_map(|line| {
        let trimmed = line.trim();
        if trimmed
            .to_ascii_lowercase()
            .starts_with("relationship to earlier wiki context:")
        {
            trimmed
                .split_once(':')
                .map(|(_, rest)| rest.trim().to_string())
                .filter(|rest| !rest.is_empty())
        } else {
            None
        }
    })
}

pub(crate) fn daily_briefing_interpretive_angle(
    report: &KnowledgeReport,
    source_cards: &[&SourceCard],
) -> String {
    let report_context = if daily_briefing_report_is_generated_storying_artifact(report) {
        daily_briefing_report_display_title(report)
    } else {
        format!("{}\n{}", report.title, report.body_markdown)
    };
    let haystack = format!(
        "{}\n{}",
        report_context,
        source_cards
            .iter()
            .map(|card| format!("{} {}", card.title, card.summary))
            .collect::<Vec<_>>()
            .join("\n")
    )
    .to_ascii_lowercase();
    if haystack.contains("gemini")
        || haystack.contains("live api")
        || haystack.contains("multimodal")
    {
        "realtime multimodal developer experience becoming part of the agent stack".to_string()
    } else if haystack.contains("azure") || haystack.contains("microsoft foundry") {
        "Claude being packaged as managed enterprise infrastructure, not just a model endpoint"
            .to_string()
    } else if haystack.contains("spotify") || haystack.contains("verification") {
        "agent adoption being argued through verification, supervision, and production workflow controls"
            .to_string()
    } else if haystack.contains("trainium") || haystack.contains("tpu") || haystack.contains("cuda")
    {
        "frontier-agent demand spilling into cloud silicon and inference-supply-chain questions"
            .to_string()
    } else if haystack.contains("nvidia dc compute")
        || haystack.contains("accelerator model")
        || haystack.contains("hbm")
    {
        "AI infrastructure expectations being revised around compute, memory, and second-half demand"
            .to_string()
    } else if (haystack.contains("grok") && haystack.contains("ai gateway"))
        || haystack.contains("voice agent")
        || haystack.contains("realtime voice")
    {
        "voice and realtime models moving into deployable developer tooling".to_string()
    } else if haystack.contains("copilotkit")
        || haystack.contains("ag-ui")
        || haystack.contains("generative ui")
        || haystack.contains("mcp")
    {
        "the interface layer making agent behavior visible, steerable, and testable".to_string()
    } else if haystack.contains("simon")
        || haystack.contains("codex cli")
        || haystack.contains("shell")
        || haystack.contains("local")
    {
        "local, scriptable agent workflows becoming more concrete than launch-page demos"
            .to_string()
    } else if haystack.contains("eval")
        || haystack.contains("benchmark")
        || haystack.contains("observability")
        || haystack.contains("swe-bench")
        || haystack.contains("helicone")
    {
        "evaluation, tracing, pricing, and reproducibility becoming purchase criteria".to_string()
    } else if haystack.contains("preview")
        || haystack.contains("restricted")
        || haystack.contains("access")
    {
        "availability, verification, and access control becoming as important as the model claim"
            .to_string()
    } else if source_cards
        .iter()
        .all(|card| card.provider.eq_ignore_ascii_case("github"))
    {
        "repository activity that only matters if release notes, docs, or developers connect it to a shipped change".to_string()
    } else {
        "early evidence that needs a stronger primary source before it deserves a bigger claim"
            .to_string()
    }
}

pub(crate) fn strip_source_card_ids_for_reader(text: &str, source_card_ids: &[&String]) -> String {
    let mut out = text.to_string();
    for source_card_id in source_card_ids {
        out = out.replace(&format!("`{source_card_id}`"), "linked source");
        out = out.replace(source_card_id.as_str(), "linked source");
    }
    out
}

pub(crate) fn daily_briefing_rewrite_reader_language(text: &str) -> String {
    let mut out = text.to_string();
    for (from, to) in [
        ("AI/devrel", "AI developer ecosystem"),
        ("DevRel", "developer adoption"),
        ("devrel", "developer adoption"),
        (
            "Arcwell's important update is negative evidence:",
            "The important update is what still has not appeared:",
        ),
        ("Arcwell has not captured", "No credible source showed"),
        ("Arcwell's evidence", "The available evidence"),
        ("Arcwell's source", "The available source"),
        ("Arcwell", "the system"),
        (
            "local GitHub/source-card evidence",
            "GitHub and source evidence",
        ),
        ("local GitHub cards", "GitHub repository activity"),
        ("local GitHub/source evidence", "GitHub and source evidence"),
        ("local corpus", "tracked sources"),
        ("local record", "available reporting"),
        ("source-card-backed", "evidence-backed"),
        ("source backed", "evidence-backed"),
        ("source-backed", "evidence-backed"),
        ("source-card evidence", "available evidence"),
        ("source card evidence", "available evidence"),
        ("source-card ids", "source references"),
        ("source card ids", "source references"),
        ("source-card ID", "source reference"),
        ("source-card id", "source reference"),
        ("source-card", "source"),
        ("source card", "source"),
        ("cluster includes", "story includes"),
        ("cluster ties together", "story ties together"),
        ("cluster is based on", "story is based on"),
        ("cluster", "story"),
        ("backlog projection", "older material"),
        ("wiki page", "background note"),
        ("wiki context", "earlier context"),
        ("wiki", "background knowledge"),
        ("Knowledge:", ""),
        ("project metadata", "project details"),
        ("metadata", "details"),
        ("source evidence", "linked source"),
        ("Knowledge Cluster Expansion:", ""),
        ("Knowledge Report:", ""),
    ] {
        out = out.replace(from, to);
    }
    out
}

pub(crate) fn daily_briefing_output_has_forbidden_reader_language(text: &str) -> bool {
    !daily_briefing_forbidden_reader_terms(text).is_empty()
}

pub(crate) fn daily_briefing_contains_forbidden_reader_language(text: &str) -> bool {
    !daily_briefing_forbidden_reader_terms(text).is_empty()
}

pub(crate) fn daily_briefing_forbidden_reader_terms(text: &str) -> Vec<&'static str> {
    let lower = text.to_ascii_lowercase();
    daily_briefing_forbidden_reader_language_terms()
        .iter()
        .copied()
        .filter(|term| lower.contains(term))
        .collect()
}

pub(crate) fn daily_briefing_forbidden_reader_language_terms() -> &'static [&'static str] {
    &[
        "arcwell",
        "local corpus",
        "local record",
        "source-card",
        "source evidence",
        "source references",
        "source card id",
        "source-card id",
        "source-backed",
        "knowledge report",
        "knowledge cluster expansion",
        "cluster",
        "proof level",
        "source family",
        "backlog projection",
        "source_card_backlog",
        "knowledge:",
        "durable source rows",
        "durable source-card rows",
        "durable sources",
        "unified knowledge pipeline",
        "unified knowledge system",
        "unified evidence-backed knowledge",
        "provider family bucket",
        "provider buckets",
        "primary-source-style",
        "github repositories detected",
        "external domains detected",
        "wiki",
        "local audit ledger",
        "digest candidate",
        "approved candidate",
        "digest delivery gate",
        "candidate id",
        "cluster id",
    ]
}

pub(crate) fn human_join_strings(items: &[String]) -> String {
    let refs = items.iter().map(String::as_str).collect::<Vec<_>>();
    human_join(&refs)
}

pub(crate) fn human_join(items: &[&str]) -> String {
    match items {
        [] => "the existing wiki".to_string(),
        [one] => format!("\"{one}\""),
        [first, second] => format!("\"{first}\" and \"{second}\""),
        [first, rest @ ..] => {
            let mut parts = vec![format!("\"{first}\"")];
            for item in &rest[..rest.len().saturating_sub(1)] {
                parts.push(format!("\"{item}\""));
            }
            let last = rest.last().copied().unwrap_or(first);
            format!("{}, and \"{}\"", parts.join(", "), last)
        }
    }
}

pub(crate) fn digest_alert_telegram_chat_id(recipient_ref: &str) -> Result<&str> {
    let chat_id = recipient_ref
        .strip_prefix("telegram:chat:")
        .context("scheduled digest Telegram recipient must be telegram:chat:<id>")?;
    validate_key(chat_id)?;
    Ok(chat_id)
}

pub(crate) fn digest_alert_email_recipient(recipient_ref: &str) -> Result<&str> {
    let email = recipient_ref
        .strip_prefix("email:")
        .context("scheduled digest email recipient must be email:<address>")?;
    normalize_email_address(email).context("invalid scheduled digest email recipient")?;
    Ok(email)
}

pub(crate) fn digest_source_card_is_x_origin(source_card: &SourceCard) -> bool {
    if source_card.provider.eq_ignore_ascii_case("x")
        || source_card.provider.eq_ignore_ascii_case("x-import")
        || source_card.source_type.eq_ignore_ascii_case("x")
        || source_card.source_type.starts_with("x_")
    {
        return true;
    }
    source_card
        .metadata
        .get("x_id")
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty())
        || source_card
            .metadata
            .get("x_author_id")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.trim().is_empty())
}

pub(crate) fn digest_source_card_is_knowledge_daily_briefing(source_card: &SourceCard) -> bool {
    source_card.provider.eq_ignore_ascii_case("arcwell")
        && source_card
            .source_type
            .eq_ignore_ascii_case("knowledge_daily_briefing")
        && source_card_metadata_string(&source_card.metadata, "source_kind").as_deref()
            == Some("knowledge_daily_briefing")
}

pub(crate) fn digest_source_card_is_credential_reminder(source_card: &SourceCard) -> bool {
    source_card.provider.eq_ignore_ascii_case("arcwell")
        && source_card
            .source_type
            .eq_ignore_ascii_case("credential_health")
        && source_card_metadata_string(&source_card.metadata, "source_kind").as_deref()
            == Some("credential_reminder")
}

pub(crate) fn scheduled_radar_delivery_policy(
    profile: &RadarProfile,
) -> Result<Option<ScheduledRadarDeliveryPolicy>> {
    let policy = profile
        .delivery_policy
        .as_object()
        .context("delivery_policy must be an object")?;
    let delivery = policy
        .get("delivery")
        .and_then(Value::as_str)
        .unwrap_or("manual_only");
    let enabled = policy
        .get("enabled")
        .and_then(Value::as_bool)
        .unwrap_or(delivery == "scheduled");
    if !enabled || delivery == "manual_only" {
        return Ok(None);
    }
    if delivery != "scheduled" {
        bail!("scheduled radar delivery requires delivery='scheduled'");
    }
    let channel = normalize_radar_delivery_channel(
        policy
            .get("channel")
            .and_then(Value::as_str)
            .unwrap_or("telegram"),
    )?;
    let recipient_raw = policy
        .get("recipient_ref")
        .or_else(|| policy.get("recipient"))
        .and_then(Value::as_str)
        .context("scheduled radar delivery requires recipient_ref")?;
    let recipient_ref = normalize_radar_delivery_recipient(&channel, recipient_raw)?;
    let interval_hours = policy
        .get("interval_hours")
        .or_else(|| policy.get("cadence_hours"))
        .and_then(Value::as_i64)
        .unwrap_or(24);
    if !(1..=24 * 365).contains(&interval_hours) {
        bail!("scheduled radar delivery interval_hours must be between 1 and 8760");
    }
    let language = normalize_radar_summary_language(
        policy
            .get("language")
            .and_then(Value::as_str)
            .unwrap_or_else(|| {
                profile
                    .languages
                    .first()
                    .map(String::as_str)
                    .unwrap_or("en")
            }),
    )?;
    let format = normalize_radar_summary_format(
        policy
            .get("format")
            .and_then(Value::as_str)
            .unwrap_or("markdown"),
    )?;
    if policy.get("bot_token").is_some()
        || policy.get("telegram_bot_token").is_some()
        || policy.get("api_token").is_some()
        || policy.get("email_api_token").is_some()
        || policy.get("cloudflare_api_token").is_some()
        || policy.get("cloudflare_email_api_token").is_some()
        || policy.get("email_account_id").is_some()
        || policy.get("cloudflare_account_id").is_some()
    {
        bail!("scheduled radar delivery policy must not contain raw secrets");
    }
    let quiet_hours = policy
        .get("quiet_hours")
        .map(parse_scheduled_radar_quiet_hours)
        .transpose()?;
    Ok(Some(ScheduledRadarDeliveryPolicy {
        interval_hours,
        channel,
        recipient_ref,
        language,
        format,
        fetch_live: policy
            .get("fetch_live")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        quiet_hours,
    }))
}

pub(crate) fn parse_scheduled_radar_quiet_hours(value: &Value) -> Result<ScheduledRadarQuietHours> {
    let object = value
        .as_object()
        .context("scheduled radar quiet_hours must be an object")?;
    let timezone = object
        .get("timezone")
        .and_then(Value::as_str)
        .unwrap_or("UTC");
    if timezone != "UTC" {
        bail!("scheduled radar quiet_hours currently supports timezone='UTC' only");
    }
    let start = object
        .get("start")
        .and_then(Value::as_str)
        .context("scheduled radar quiet_hours requires start")?;
    let end = object
        .get("end")
        .and_then(Value::as_str)
        .context("scheduled radar quiet_hours requires end")?;
    let start_minutes = parse_hh_mm_minutes(start, "quiet_hours.start")?;
    let end_minutes = parse_hh_mm_minutes(end, "quiet_hours.end")?;
    if start_minutes == end_minutes {
        bail!("scheduled radar quiet_hours start and end must differ");
    }
    Ok(ScheduledRadarQuietHours {
        start_minutes,
        end_minutes,
    })
}

pub(crate) fn parse_hh_mm_minutes(value: &str, label: &str) -> Result<u32> {
    let (hour, minute) = value
        .split_once(':')
        .with_context(|| format!("{label} must use HH:MM"))?;
    if hour.len() != 2 || minute.len() != 2 {
        bail!("{label} must use zero-padded HH:MM");
    }
    let hour = hour
        .parse::<u32>()
        .with_context(|| format!("{label} hour must be numeric"))?;
    let minute = minute
        .parse::<u32>()
        .with_context(|| format!("{label} minute must be numeric"))?;
    if hour > 23 || minute > 59 {
        bail!("{label} must be within 00:00 and 23:59");
    }
    Ok(hour * 60 + minute)
}

pub(crate) fn radar_quiet_hours_deferred_until(
    policy: &ScheduledRadarDeliveryPolicy,
    now: DateTime<Utc>,
) -> Result<Option<DateTime<Utc>>> {
    let Some(quiet_hours) = &policy.quiet_hours else {
        return Ok(None);
    };
    let now_minutes = now.hour() * 60 + now.minute();
    let start = quiet_hours.start_minutes;
    let end = quiet_hours.end_minutes;
    let active = if start < end {
        now_minutes >= start && now_minutes < end
    } else {
        now_minutes >= start || now_minutes < end
    };
    if !active {
        return Ok(None);
    }
    let end_hour = end / 60;
    let end_minute = end % 60;
    let today_end = now
        .date_naive()
        .and_hms_opt(end_hour, end_minute, 0)
        .context("constructing scheduled radar quiet-hours end")?
        .and_utc();
    let deferred_until = if today_end <= now {
        today_end + ChronoDuration::days(1)
    } else {
        today_end
    };
    Ok(Some(deferred_until))
}

pub(crate) fn radar_schedule_due_slot(interval_hours: i64) -> String {
    let now = Utc::now();
    let timestamp = now.timestamp();
    let interval_seconds = interval_hours.max(1) * 3600;
    let slot = timestamp - timestamp.rem_euclid(interval_seconds);
    DateTime::<Utc>::from_timestamp(slot, 0)
        .unwrap_or(now)
        .to_rfc3339()
}

pub(crate) fn radar_schedule_interval_elapsed(latest_due_at: &str, interval_hours: i64) -> bool {
    DateTime::parse_from_rfc3339(latest_due_at)
        .map(|latest| {
            latest.with_timezone(&Utc) + ChronoDuration::hours(interval_hours.max(1)) <= Utc::now()
        })
        .unwrap_or(true)
}

pub(crate) fn radar_schedule_tick_key(
    profile_id: &str,
    due_at: &str,
    policy: &ScheduledRadarDeliveryPolicy,
) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        profile_id, due_at, policy.language, policy.format, policy.recipient_ref
    )
}

pub(crate) fn validate_radar_delivery_status(status: &str) -> Result<()> {
    match status {
        "pending" | "sent" | "failed" | "blocked" | "deferred" | "dead_lettered" => Ok(()),
        other => bail!("unsupported radar delivery status: {other}"),
    }
}

pub(crate) fn validate_radar_schedule_status(status: &str) -> Result<()> {
    match status {
        "pending" | "running" | "sent" | "failed" | "blocked" | "deferred" | "dead_lettered" => {
            Ok(())
        }
        other => bail!("unsupported radar schedule status: {other}"),
    }
}

pub(crate) fn sanitize_radar_delivery_error(error: &str) -> Result<String> {
    let mut sanitized = redact_secret_like_text(error);
    sanitized = sanitized.replace('\0', "");
    let sanitized = excerpt(sanitized.trim(), 1_000);
    validate_notes(&sanitized)?;
    Ok(sanitized)
}

pub(crate) fn radar_score_status_counts(scores: &[RadarScore]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for score in scores {
        *counts.entry(score.status.clone()).or_insert(0) += 1;
    }
    counts
}

pub(crate) fn render_radar_summary_markdown(
    run: &RadarRun,
    profile: &RadarProfile,
    selected: &[(RadarScore, RadarItem)],
    status_counts: &BTreeMap<String, usize>,
    dedup_group_count: usize,
    audit: &RadarAuditReport,
) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!(
        "# {}\n\n",
        escape_research_report_text(&format!("Radar Summary: {}", profile.name))
    ));
    markdown.push_str("> Trust label: GENERATED_RADAR_SUMMARY. This report is generated from local radar item and score records. It is not source evidence, not a live fetch, not model-backed synthesis, and not delivery; any semantic dedupe is deterministic local grouping only.\n\n");
    markdown.push_str("## Run\n\n");
    markdown.push_str(&format!(
        "- Run: `{}`\n- Profile: `{}`\n- Window: `{}` to `{}`\n- Status: `{}` / stage `{}`\n- Counts: raw {}, normalized {}, indexed {}, scored {}, selected {}, dedupe groups {}, summaries {}, deliveries {}\n- Audit: `{}` with {} finding(s)\n\n",
        run.id,
        profile.name,
        run.window_start,
        run.window_end,
        run.status,
        run.stage,
        run.raw_count,
        run.normalized_count,
        run.indexed_count,
        run.scored_count,
        run.filtered_count,
        dedup_group_count,
        run.summary_count,
        run.delivery_count,
        if audit.ok { "ok" } else { "findings" },
        audit.findings.len()
    ));
    if !status_counts.is_empty() {
        markdown.push_str("Score statuses: ");
        let statuses = status_counts
            .iter()
            .map(|(status, count)| format!("`{}` {}", escape_markdown_line(status), count))
            .collect::<Vec<_>>();
        markdown.push_str(&statuses.join(", "));
        markdown.push_str(".\n\n");
    }
    markdown.push_str("## Selected Items\n\n");
    for (index, (score, item)) in selected.iter().enumerate() {
        let label = escape_markdown_link_text(&item.title);
        if let Some(url) = &item.canonical_url {
            markdown.push_str(&format!("{}. [{}]({})\n", index + 1, label, url));
        } else {
            markdown.push_str(&format!("{}. {}\n", index + 1, label));
        }
        markdown.push_str(&format!(
            "   - Score: {:.2} via `{}`; status `{}`.\n",
            score.score,
            escape_markdown_line(&score.score_kind),
            escape_markdown_line(&score.status)
        ));
        markdown.push_str(&format!(
            "   - Reason: {}.\n",
            escape_untrusted_markdown_text(&score.reason)
        ));
        markdown.push_str(&format!(
            "   - Source: provider `{}`, kind `{}`, item `{}`{}.\n",
            escape_markdown_line(&item.provider),
            escape_markdown_line(&item.source_kind),
            item.id,
            item.source_card_id
                .as_ref()
                .map(|id| format!(", source card `{id}`"))
                .unwrap_or_default()
        ));
        markdown.push_str(&format!(
            "   - Evidence excerpt: {}.\n\n",
            escape_untrusted_markdown_text(&excerpt(&item.content_text, 360))
        ));
    }
    if !audit.findings.is_empty() {
        markdown.push_str("## Audit Findings\n\n");
        for finding in &audit.findings {
            markdown.push_str(&format!(
                "- `{}` `{}`: {} Evidence: {}.\n",
                escape_markdown_line(&finding.severity),
                escape_markdown_line(&finding.code),
                escape_untrusted_markdown_text(&finding.message),
                escape_untrusted_markdown_text(&finding.evidence)
            ));
        }
        markdown.push('\n');
    }
    markdown.push_str("## Boundaries\n\n");
    markdown.push_str("- This artifact did not fetch new sources.\n");
    markdown.push_str("- This artifact did not run model scoring or model summarization.\n");
    markdown.push_str(
        "- This artifact did not deliver a message to email, Telegram, or any channel.\n",
    );
    markdown.push_str(
        "- Generated summary text must not be cited as primary evidence in later research.\n",
    );
    markdown
}

pub(crate) fn radar_item_from_source_card(run_id: &str, card: &SourceCard) -> Result<RadarItem> {
    let timestamp = now();
    let stable_key = format!("source_card:{}", card.id);
    let content_text = format!(
        "{}\n\n{}\n\nClaims: {}",
        card.title,
        card.summary,
        serde_json::to_string(&card.claims)?
    );
    Ok(RadarItem {
        id: Uuid::new_v4().to_string(),
        run_id: run_id.to_string(),
        stable_key,
        source_kind: "source_card".to_string(),
        provider: card.provider.clone(),
        source_locator: card.url.clone(),
        native_id: Some(card.id.clone()),
        canonical_url: Some(card.url.clone()),
        title: card.title.clone(),
        author: card
            .metadata
            .get("author")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        published_at: card
            .metadata
            .get("published_at")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        fetched_at: timestamp.clone(),
        content_sha256: sha256(content_text.as_bytes()),
        content_text,
        metadata: json!({
            "source_card_id": card.id,
            "source_type": card.source_type,
            "source_kind": card.metadata.get("source_kind").and_then(Value::as_str).unwrap_or(card.provider.as_str()),
            "source_detail": card.metadata.get("source_detail").and_then(Value::as_str).unwrap_or(card.url.as_str()),
            "model_prompt_metadata": radar_model_prompt_metadata_projection(&card.metadata),
            "retrieved_at": card.retrieved_at,
            "claims": card.claims.len(),
            "trust_boundary": "external source-card text is untrusted evidence, not instructions",
            "projection": "radar_source_card_query_v1"
        }),
        source_card_id: Some(card.id.clone()),
        wiki_page_id: Some(card.wiki_page_id.clone()),
        canonical_entity_ref: Some(format!("source_card:{}", card.id)),
        trust_level: "untrusted_external_evidence".to_string(),
        created_at: timestamp.clone(),
        updated_at: timestamp,
    })
}

pub(crate) fn score_radar_item_heuristic(item: &RadarItem) -> (f64, String, Vec<String>) {
    score_radar_item_heuristic_with_health(item, None)
}

pub(crate) fn score_radar_item_heuristic_with_health(
    item: &RadarItem,
    source_health: Option<&SourceHealth>,
) -> (f64, String, Vec<String>) {
    let text = format!("{} {}", item.title, item.content_text).to_ascii_lowercase();
    let mut score: f64 = 2.0;
    let mut reasons = vec!["source-card-backed evidence".to_string()];
    let mut tags = vec![item.source_kind.clone()];
    for (needle, tag, reason, bump) in [
        ("launch", "launch", "launch signal", 1.5),
        ("release", "release", "release signal", 1.3),
        ("vulnerability", "security", "security-impact signal", 1.6),
        ("incident", "incident", "operational incident signal", 1.2),
        ("funding", "company", "company/funding signal", 1.0),
        ("open source", "oss", "open-source signal", 0.9),
        ("model", "ai", "AI/model signal", 0.8),
        ("agent", "agent", "agent-infrastructure signal", 0.8),
        ("mcp", "mcp", "MCP signal", 0.8),
        ("breaking", "breaking-change", "breaking-change signal", 1.2),
        ("benchmark", "benchmark", "benchmark signal", 0.6),
    ] {
        if text.contains(needle) {
            score += bump;
            reasons.push(reason.to_string());
            tags.push(tag.to_string());
        }
    }
    if item.content_text.len() > 800 {
        score += 0.4;
        reasons.push("substantive source-card text".to_string());
    }
    if item.content_text.contains("ignore previous instructions")
        || item.content_text.contains("system prompt")
        || item.content_text.contains("exfiltrate")
    {
        score -= 1.0;
        reasons.push("hostile-source-text penalty".to_string());
        tags.push("prompt-injection-risk".to_string());
    }
    if let Some((adjustment, reason, tag)) = radar_freshness_adjustment(item) {
        score += adjustment;
        reasons.push(reason);
        tags.push(tag);
    }
    if let Some(health) = source_health {
        if health.status == "healthy" {
            score += 0.2;
            reasons.push("healthy source-health signal".to_string());
            tags.push("source-health-healthy".to_string());
            if let Some(last_success_at) = health.last_success_at.as_deref()
                && timestamp_age_hours(last_success_at)
                    .map(|age| age > 24 * 7)
                    .unwrap_or(false)
            {
                score -= 0.4;
                reasons.push("stale source-health success timestamp penalty".to_string());
                tags.push("source-health-stale".to_string());
            }
        } else {
            score -= 1.0;
            reasons.push(format!("source-health {} penalty", health.status));
            tags.push("source-health-nonhealthy".to_string());
            tags.push(format!("source-health-{}", health.status));
        }
    }
    tags.sort();
    tags.dedup();
    (score.clamp(0.0, 10.0), reasons.join(", "), tags)
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RadarBalanceConfig {
    pub(crate) max_per_source: Option<usize>,
    pub(crate) category_quotas: BTreeMap<String, usize>,
}

impl RadarBalanceConfig {
    pub(crate) fn enabled(&self) -> bool {
        self.max_per_source.is_some() || !self.category_quotas.is_empty()
    }

    pub(crate) fn to_json(&self) -> Value {
        json!({
            "max_per_source": self.max_per_source,
            "category_quotas": self.category_quotas,
        })
    }
}

pub(crate) fn radar_balance_config_from_metadata(metadata: &Value) -> Result<RadarBalanceConfig> {
    let Some(balance) = metadata
        .get("balance")
        .or_else(|| metadata.get("radar_balance"))
    else {
        return Ok(RadarBalanceConfig::default());
    };
    if balance.is_null() {
        return Ok(RadarBalanceConfig::default());
    }
    let Some(balance) = balance.as_object() else {
        bail!("radar metadata balance must be an object");
    };
    let max_per_source = match balance
        .get("max_per_source")
        .or_else(|| balance.get("source_cap"))
    {
        Some(value) => Some(parse_radar_balance_cap(value, "max_per_source")?),
        None => None,
    };
    let mut category_quotas = BTreeMap::new();
    if let Some(value) = balance
        .get("category_quotas")
        .or_else(|| balance.get("categories"))
    {
        let Some(object) = value.as_object() else {
            bail!("radar metadata balance.category_quotas must be an object");
        };
        for (raw_category, raw_cap) in object {
            let category = normalize_radar_balance_key(raw_category, "category")?;
            let cap = parse_radar_balance_cap(raw_cap, &format!("category_quotas.{category}"))?;
            category_quotas.insert(category, cap);
        }
    }
    Ok(RadarBalanceConfig {
        max_per_source,
        category_quotas,
    })
}

pub(crate) fn parse_radar_balance_cap(value: &Value, field: &str) -> Result<usize> {
    let Some(cap) = value.as_u64() else {
        bail!("radar metadata balance.{field} must be a positive integer");
    };
    if !(1..=500).contains(&cap) {
        bail!("radar metadata balance.{field} must be between 1 and 500");
    }
    Ok(cap as usize)
}

pub(crate) fn normalize_radar_balance_key(raw: &str, label: &str) -> Result<String> {
    let key = raw.trim().to_ascii_lowercase();
    if key.is_empty() {
        bail!("radar balance {label} cannot be empty");
    }
    if key.len() > 80 {
        bail!("radar balance {label} is too long");
    }
    if !key
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
    {
        bail!("radar balance {label} may only contain letters, numbers, hyphen, or underscore");
    }
    Ok(key)
}

pub(crate) fn radar_balance_source_key(item: &RadarItem) -> String {
    let (source_kind, locator) = radar_source_quality_key_for_item(item);
    format!("{source_kind}:{locator}")
}

pub(crate) fn radar_balance_category_for_item(
    item: &RadarItem,
    tags: &[String],
    balance_config: &RadarBalanceConfig,
) -> Option<String> {
    if balance_config.category_quotas.is_empty() {
        return None;
    }
    let candidates = radar_balance_category_candidates(item, tags);
    balance_config
        .category_quotas
        .keys()
        .find(|category| candidates.contains(*category))
        .cloned()
}

pub(crate) fn radar_balance_category_candidates(
    item: &RadarItem,
    tags: &[String],
) -> BTreeSet<String> {
    let mut candidates = BTreeSet::new();
    for value in [item.source_kind.as_str(), item.provider.as_str()] {
        if let Ok(category) = normalize_radar_balance_key(value, "category") {
            candidates.insert(category);
        }
    }
    for key in ["category", "category_group", "topic", "source_type"] {
        if let Some(value) = item.metadata.get(key).and_then(Value::as_str)
            && let Ok(category) = normalize_radar_balance_key(value, "category")
        {
            candidates.insert(category);
        }
    }
    if let Some(values) = item.metadata.get("categories").and_then(Value::as_array) {
        for value in values {
            if let Some(value) = value.as_str()
                && let Ok(category) = normalize_radar_balance_key(value, "category")
            {
                candidates.insert(category);
            }
        }
    }
    for tag in tags {
        if let Ok(category) = normalize_radar_balance_key(tag, "category") {
            candidates.insert(category);
        }
    }
    candidates
}

pub(crate) fn radar_source_quality_key_for_item(item: &RadarItem) -> (String, String) {
    let source_kind = item
        .metadata
        .get("source_kind")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            if item.provider.trim().is_empty() {
                item.source_kind.as_str()
            } else {
                item.provider.as_str()
            }
        })
        .to_string();
    let locator = item
        .metadata
        .get("source_detail")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(item.source_locator.as_str())
        .to_string();
    (source_kind, locator)
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ExpectedRadarSourceQuality {
    pub(crate) raw_count: i64,
    pub(crate) accepted_count: i64,
    pub(crate) duplicate_count: i64,
}

pub(crate) fn expected_radar_source_quality_counts(
    items: &[RadarItem],
    scores: &[RadarScore],
) -> BTreeMap<(String, String), ExpectedRadarSourceQuality> {
    let scores_by_item = scores
        .iter()
        .filter(|score| score.score_kind == "heuristic_v1")
        .map(|score| (score.item_id.as_str(), score))
        .collect::<BTreeMap<_, _>>();
    let mut expected = BTreeMap::new();
    for item in items {
        let Some(score) = scores_by_item.get(item.id.as_str()) else {
            continue;
        };
        let entry = expected
            .entry(radar_source_quality_key_for_item(item))
            .or_insert(ExpectedRadarSourceQuality {
                raw_count: 0,
                accepted_count: 0,
                duplicate_count: 0,
            });
        entry.raw_count += 1;
        if score.status == "selected" {
            entry.accepted_count += 1;
        }
        if score.status.starts_with("duplicate_") {
            entry.duplicate_count += 1;
        }
    }
    expected
}

pub(crate) fn radar_score_distribution_json(scores: &[RadarScore]) -> Value {
    let heuristic_scores = scores
        .iter()
        .filter(|score| score.score_kind == "heuristic_v1")
        .collect::<Vec<_>>();
    let mut values = heuristic_scores
        .iter()
        .map(|score| score.score)
        .filter(|score| score.is_finite())
        .collect::<Vec<_>>();
    values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let mut status_counts = BTreeMap::<String, usize>::new();
    for score in &heuristic_scores {
        *status_counts.entry(score.status.clone()).or_insert(0) += 1;
    }
    let average = if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    };
    json!({
        "score_kind": "heuristic_v1",
        "schema_version": 1,
        "score_count": heuristic_scores.len(),
        "finite_score_count": values.len(),
        "selected_count": status_counts.get("selected").copied().unwrap_or(0),
        "below_threshold_count": status_counts.get("below_threshold").copied().unwrap_or(0),
        "over_profile_limit_count": status_counts.get("over_profile_limit").copied().unwrap_or(0),
        "duplicate_count": heuristic_scores
            .iter()
            .filter(|score| score.status.starts_with("duplicate_"))
            .count(),
        "status_counts": status_counts,
        "min": values.first().copied(),
        "max": values.last().copied(),
        "average": average,
        "p10": percentile_sorted(&values, 0.10),
        "p50": percentile_sorted(&values, 0.50),
        "p90": percentile_sorted(&values, 0.90),
    })
}

pub(crate) fn radar_source_health_for_item<'a>(
    source_health: &'a [SourceHealth],
    item: &RadarItem,
) -> Option<&'a SourceHealth> {
    let (source_kind, locator) = radar_source_quality_key_for_item(item);
    radar_source_health_for_quality_key(source_health, &source_kind, &locator, &item.provider)
}

pub(crate) fn radar_source_health_for_quality_key<'a>(
    source_health: &'a [SourceHealth],
    source_kind: &str,
    locator: &str,
    provider: &str,
) -> Option<&'a SourceHealth> {
    let locators = radar_source_health_locator_candidates(source_kind, locator);
    source_health
        .iter()
        .find(|health| {
            locators
                .iter()
                .any(|candidate| health.source_kind == source_kind && health.locator == *candidate)
        })
        .or_else(|| {
            source_health.iter().find(|health| {
                locators.iter().any(|candidate| {
                    !provider.is_empty()
                        && health.provider == provider
                        && health.locator == *candidate
                })
            })
        })
}

pub(crate) fn radar_source_health_locator_candidates(
    source_kind: &str,
    locator: &str,
) -> Vec<String> {
    let mut locators = vec![locator.to_string()];
    if matches!(source_kind, "github_release" | "github_commit") {
        locators.push(format!("{locator}:releases"));
        locators.push(format!("{locator}:commits"));
    }
    locators
}

pub(crate) fn radar_freshness_adjustment(item: &RadarItem) -> Option<(f64, String, String)> {
    let timestamp = item
        .published_at
        .as_deref()
        .or_else(|| item.metadata.get("retrieved_at").and_then(Value::as_str))
        .or(Some(item.fetched_at.as_str()))?;
    let age_hours = timestamp_age_hours(timestamp)?;
    if age_hours <= 48 {
        Some((
            0.4,
            "fresh source timestamp signal".to_string(),
            "fresh-source".to_string(),
        ))
    } else if age_hours > 24 * 90 {
        Some((
            -1.0,
            "very stale source timestamp penalty".to_string(),
            "very-stale-source".to_string(),
        ))
    } else if age_hours > 24 * 30 {
        Some((
            -0.5,
            "stale source timestamp penalty".to_string(),
            "stale-source".to_string(),
        ))
    } else {
        None
    }
}

pub(crate) fn timestamp_age_hours(timestamp: &str) -> Option<i64> {
    let parsed = DateTime::parse_from_rfc3339(timestamp)
        .ok()?
        .with_timezone(&Utc);
    Some((Utc::now() - parsed).num_hours())
}

pub(crate) fn percentile_sorted(values: &[f64], percentile: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let percentile = percentile.clamp(0.0, 1.0);
    let index = ((values.len() - 1) as f64 * percentile).round() as usize;
    values.get(index).copied()
}
