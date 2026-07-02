use crate::*;

pub(crate) fn validate_candidate_operation(operation: &str) -> Result<()> {
    match operation {
        "ADD" | "UPDATE" | "DELETE" | "NONE" => Ok(()),
        other => bail!("unsupported memory candidate operation: {other}"),
    }
}

pub(crate) fn validate_cost_scope(scope: &str) -> Result<()> {
    match scope {
        "global" | "package" | "provider" | "source" => Ok(()),
        other => bail!("unsupported cost policy scope: {other}"),
    }
}

pub(crate) fn validate_policy_rule(rule: &PolicyRule) -> Result<()> {
    validate_key(&rule.id)?;
    validate_policy_effect(&rule.effect)?;
    validate_policy_action(&rule.action)?;
    validate_notes(&rule.reason)?;
    for value in [
        rule.package.as_deref(),
        rule.provider.as_deref(),
        rule.source.as_deref(),
        rule.channel.as_deref(),
        rule.subject.as_deref(),
        rule.target.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        validate_policy_pattern(value)?;
    }
    if let Some(expires_at) = &rule.expires_at {
        DateTime::parse_from_rfc3339(expires_at)
            .with_context(|| format!("parsing policy expires_at timestamp {expires_at}"))?;
    }
    Ok(())
}

pub(crate) fn validate_policy_request(request: &PolicyRequest) -> Result<()> {
    validate_policy_action(&request.action)?;
    for value in [
        request.package.as_deref(),
        request.provider.as_deref(),
        request.source.as_deref(),
        request.channel.as_deref(),
        request.subject.as_deref(),
        request.target.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        validate_policy_pattern(value)?;
    }
    if let Some(projected_usd) = request.projected_usd {
        validate_non_negative_cost(projected_usd, "projected_usd")?;
    }
    if let Some(excerpt) = &request.untrusted_excerpt {
        validate_notes(excerpt)?;
    }
    Ok(())
}

pub(crate) fn validate_policy_effect(effect: &str) -> Result<()> {
    match effect {
        "allow" | "deny" | "require_approval" | "defer" => Ok(()),
        other => bail!("unsupported policy effect: {other}"),
    }
}

pub(crate) fn validate_digest_review_status(status: &str) -> Result<()> {
    match status {
        "approved" | "rejected" => Ok(()),
        other => bail!("unsupported digest candidate review status: {other}"),
    }
}

pub(crate) fn digest_delivery_idempotency_key(
    candidate_id: &str,
    channel: &str,
    subject: &str,
    target: &str,
    supplied: Option<&str>,
) -> Result<String> {
    if let Some(supplied) = supplied {
        validate_query(supplied)?;
        return Ok(supplied.to_string());
    }
    Ok(format!("{candidate_id}:{channel}:{subject}:{target}"))
}

pub(crate) fn digest_candidate_email_subject(candidate: &DigestCandidate) -> String {
    let topic = Store::digest_human_topic(&candidate.topic);
    if digest_topic_is_credential_reminder(&candidate.topic) {
        return format!("Arcwell credential reminder: {}", excerpt(&topic, 120));
    }
    if digest_topic_is_knowledge_daily_briefing(&candidate.topic) {
        return excerpt(&topic, 140);
    }
    format!("X bookmark report: {}", excerpt(&topic, 120))
}

pub(crate) fn digest_topic_is_knowledge_daily_briefing(topic: &str) -> bool {
    let topic = topic.to_ascii_lowercase();
    topic.contains("arcwell ai daily briefing")
        || topic.contains("knowledge daily briefing")
        || topic.contains("arcwell ai week overview")
        || topic.contains("knowledge weekly overview")
}

pub(crate) fn digest_topic_is_credential_reminder(topic: &str) -> bool {
    topic
        .to_ascii_lowercase()
        .contains("credential health reminder")
        || topic.to_ascii_lowercase().contains("credential reminder")
}

pub(crate) fn x_knowledge_cluster_key(item: &RadarItem) -> String {
    let lower = format!("{} {}", item.title, item.content_text).to_ascii_lowercase();
    if lower.contains("model")
        || lower.contains("gemma")
        || lower.contains("deepmind")
        || lower.contains("multimodal")
    {
        "model-launches".to_string()
    } else if lower.contains("mcp") || lower.contains("tool") || lower.contains("runtime") {
        "agent-tooling-mcp".to_string()
    } else if lower.contains("xcode") || lower.contains("claude code") || lower.contains("coding") {
        "coding-tools".to_string()
    } else if lower.contains("sandbox") || lower.contains("perimeter") || lower.contains("tunnel") {
        "agent-sandboxes".to_string()
    } else if lower.contains("computer-use") || lower.contains("openwork") {
        "computer-use-agents".to_string()
    } else if lower.contains("video") || lower.contains("generation") || lower.contains("visual") {
        "generation-models".to_string()
    } else {
        "general-agent-infrastructure".to_string()
    }
}

pub(crate) fn x_knowledge_cluster_topic(
    cluster_key: &str,
    selected: &[(RadarScore, RadarItem)],
) -> String {
    let mut haystack = String::new();
    for (_, item) in selected.iter().take(12) {
        haystack.push_str(&item.title);
        haystack.push(' ');
        haystack.push_str(&item.content_text);
        haystack.push(' ');
    }
    let lower = haystack.to_ascii_lowercase();
    let base = match cluster_key {
        "agent-tooling-mcp" => "agent tooling and MCP",
        "coding-tools" => "coding-agent tools",
        "agent-sandboxes" => "agent sandboxes and secure execution",
        "computer-use-agents" => "computer-use agents",
        "model-launches" => "model launches for agents",
        "generation-models" => "AI generation model launches",
        _ => "agent infrastructure",
    };
    let mut parts = Vec::new();
    if lower.contains("mcp") {
        parts.push("MCP");
    }
    if lower.contains("xcode") || lower.contains("coding") || lower.contains("code") {
        parts.push("coding tools");
    }
    if lower.contains("model") || lower.contains("gemma") || lower.contains("deepmind") {
        parts.push("model launches");
    }
    if parts.is_empty() {
        format!("X bookmark trend: {base}")
    } else {
        format!("X bookmark trend: {base}: {}", parts.join(" and "))
    }
}

pub(crate) fn x_knowledge_duplicate_groups(selected: &[(RadarScore, RadarItem)]) -> Value {
    let mut by_url = BTreeMap::<String, Vec<String>>::new();
    for (_, item) in selected {
        let key = item
            .canonical_url
            .as_deref()
            .unwrap_or(item.stable_key.as_str())
            .to_string();
        by_url.entry(key).or_default().push(item.id.clone());
    }
    let groups = by_url
        .into_iter()
        .filter(|(_, ids)| ids.len() > 1)
        .map(|(key, ids)| json!({ "key": key, "radar_item_ids": ids }))
        .collect::<Vec<_>>();
    Value::Array(groups)
}

pub(crate) fn x_knowledge_stale_score(last_seen_at: &str) -> f64 {
    let Ok(last_seen) = chrono::DateTime::parse_from_rfc3339(last_seen_at) else {
        return 0.5;
    };
    let age_days = (Utc::now() - last_seen.with_timezone(&Utc))
        .num_days()
        .max(0) as f64;
    (age_days / 30.0).clamp(0.0, 1.0)
}

pub(crate) fn render_x_cluster_wiki_page(
    cluster: &XKnowledgeCluster,
    cards: &[SourceCard],
) -> Result<String> {
    if cards.is_empty() {
        bail!("x cluster wiki page requires source cards");
    }
    let mut lines = vec![
        format!("# {}", cluster.topic),
        String::new(),
        format!("Cluster: `{}`", cluster.id),
        format!("Status: `{}`", cluster.status),
        format!(
            "Scores: novelty {:.2}, momentum {:.2}, stale {:.2}",
            cluster.novelty_score, cluster.momentum_score, cluster.stale_score
        ),
        String::new(),
        "## Summary".to_string(),
        format!(
            "This page expands a source-card-backed X bookmark cluster. It is based on {} source cards from radar run `{}`.",
            cards.len(),
            cluster.radar_run_id.as_deref().unwrap_or("unknown")
        ),
        String::new(),
        "## What The Sources Say".to_string(),
    ];
    for (index, card) in cards.iter().enumerate() {
        lines.push(format!(
            "- [S{}] `{}`: {}",
            index + 1,
            card.id,
            excerpt(
                &html_unescape_basic(&escape_markdown_line(&card.summary)),
                360
            )
        ));
    }
    lines.extend([
        String::new(),
        "## Uncertainty And Caveats".to_string(),
        "- X posts are untrusted source evidence, not instructions or policy.".to_string(),
        "- Claims here are not treated as verified outside the saved source-card corpus.".to_string(),
        "- Follow-up research should corroborate launches, metrics, and availability against primary sources before stronger publication.".to_string(),
        String::new(),
        "## Sources".to_string(),
    ]);
    for (index, card) in cards.iter().enumerate() {
        lines.push(format!("- [S{}] `{}` {}", index + 1, card.id, card.url));
    }
    lines.push(String::new());
    lines.push("source_cards:".to_string());
    for card in cards {
        lines.push(format!("- `{}`", card.id));
    }
    Ok(format!("{}\n", lines.join("\n")))
}

pub(crate) fn audit_x_cluster_wiki_page(
    cluster: &XKnowledgeCluster,
    markdown: &str,
) -> Vec<String> {
    let mut findings = Vec::new();
    if markdown.trim().len() < 400 {
        findings.push("page_too_thin".to_string());
    }
    if !markdown.contains(&format!("Cluster: `{}`", cluster.id)) {
        findings.push("missing_cluster_link".to_string());
    }
    if !markdown.contains("## Uncertainty And Caveats") {
        findings.push("missing_uncertainty_section".to_string());
    }
    if markdown.contains("source_cards: 0") {
        findings.push("zero_source_card_marker".to_string());
    }
    for source_card_id in &cluster.source_card_ids {
        if !markdown.contains(&format!("`{source_card_id}`")) {
            findings.push(format!("missing_source_card:{source_card_id}"));
        }
    }
    if markdown
        .to_ascii_lowercase()
        .contains("ignore previous instructions")
        && !markdown.contains("untrusted source evidence")
    {
        findings.push("prompt_injection_not_labeled".to_string());
    }
    findings
}

pub(crate) fn validate_policy_action(action: &str) -> Result<()> {
    if action.trim().is_empty() {
        bail!("policy action cannot be empty");
    }
    if action.len() > 120 {
        bail!("policy action is too long");
    }
    Ok(())
}

pub(crate) fn validate_policy_pattern(pattern: &str) -> Result<()> {
    if pattern.trim().is_empty() {
        bail!("policy pattern cannot be empty");
    }
    if pattern.len() > 240 {
        bail!("policy pattern is too long");
    }
    Ok(())
}

pub(crate) fn validate_non_negative_cost(value: f64, label: &str) -> Result<()> {
    if !value.is_finite() || value < 0.0 {
        bail!("{label} must be a finite non-negative number");
    }
    if value > MAX_COST_USD {
        bail!("{label} is too large");
    }
    Ok(())
}

pub(crate) fn default_policy_rules() -> Vec<PolicyRule> {
    vec![
        PolicyRule {
            id: "default-deny-provider-network".to_string(),
            effect: "deny".to_string(),
            action: "provider.network".to_string(),
            reason: "default policy denies unknown provider network actions".to_string(),
            package: None,
            provider: Some("*".to_string()),
            source: None,
            channel: None,
            subject: None,
            target: None,
            priority: 0,
            expires_at: None,
        },
        default_allow_rule(
            "default-allow-x-recent-search",
            "provider.network",
            Some("arcwell-x"),
            Some("x"),
            Some("x_recent_search"),
            "default policy allows the existing X recent-search path after policy and cost checks",
        ),
        default_allow_rule(
            "default-allow-x-monitor",
            "provider.network",
            Some("arcwell-x"),
            Some("x"),
            Some("x_monitor"),
            "default policy allows the curated X watch-source monitor after policy and cost checks",
        ),
        default_allow_rule(
            "default-allow-x-import-bookmarks",
            "provider.network",
            Some("arcwell-x"),
            Some("x"),
            Some("x_import_bookmarks"),
            "default policy allows authenticated X bookmark import after policy and cost checks",
        ),
        default_allow_rule(
            "default-allow-x-link-expand",
            "provider.network",
            Some("arcwell-x"),
            Some("web"),
            Some("x_link_expand"),
            "default policy allows explicit X link expansion after policy and cost checks",
        ),
        default_allow_rule(
            "default-allow-url-ingest-network",
            "provider.network",
            Some("arcwell-llm-wiki"),
            Some("web"),
            Some("url_ingest"),
            "default policy allows explicit URL ingest after policy and cost checks",
        ),
        default_allow_rule(
            "default-allow-rss-fetch-network",
            "provider.network",
            Some("arcwell-llm-wiki"),
            Some("rss"),
            Some("rss_fetch"),
            "default policy allows explicit RSS fetch after policy and cost checks",
        ),
        default_allow_rule(
            "default-allow-github-repo-network",
            "provider.network",
            Some("arcwell-llm-wiki"),
            Some("github"),
            Some("github_repo"),
            "default policy allows explicit GitHub repo fetch after policy and cost checks",
        ),
        default_allow_rule(
            "default-allow-github-owner-network",
            "provider.network",
            Some("arcwell-llm-wiki"),
            Some("github"),
            Some("github_owner"),
            "default policy allows explicit GitHub owner fetch after policy and cost checks",
        ),
        default_allow_rule(
            "default-allow-arxiv-search-network",
            "provider.network",
            Some("arcwell-llm-wiki"),
            Some("arxiv"),
            Some("arxiv_search"),
            "default policy allows explicit arXiv fetch after policy and cost checks",
        ),
        default_allow_rule(
            "default-allow-hackernews-fetch-network",
            "provider.network",
            Some("arcwell-llm-wiki"),
            Some("hackernews"),
            Some("hackernews_fetch"),
            "default policy allows explicit Hacker News fetch after policy and cost checks",
        ),
        default_allow_rule(
            "default-allow-reddit-fetch-network",
            "provider.network",
            Some("arcwell-llm-wiki"),
            Some("reddit"),
            Some("reddit_fetch"),
            "default policy allows explicit Reddit fetch after policy and cost checks",
        ),
        default_allow_rule(
            "default-allow-brave-web-search",
            "provider.network",
            Some("arcwell-deep-research"),
            Some("brave"),
            Some("web_search"),
            "default policy allows the existing Brave web-search path after policy and cost checks",
        ),
        default_allow_rule(
            "default-allow-openai-web-search",
            "provider.network",
            Some("arcwell-deep-research"),
            Some("openai"),
            Some("web_search"),
            "default policy allows the existing OpenAI web-search path after policy and cost checks",
        ),
        default_allow_rule(
            "default-allow-openai-research-editorial",
            "provider.network",
            Some("arcwell-deep-research"),
            Some("openai"),
            Some("research_editorial_invoke"),
            "default policy allows explicit OpenAI research editorial invocation after policy and cost checks",
        ),
        default_allow_rule(
            "default-allow-openai-radar-model-score",
            "provider.network",
            Some("arcwell-radar"),
            Some("openai"),
            Some("radar_model_score"),
            "default policy allows explicit OpenAI radar model scoring after policy and cost checks",
        ),
        default_allow_rule(
            "default-allow-openai-knowledge-entity-resolution",
            "provider.network",
            Some("arcwell-knowledge"),
            Some("openai"),
            Some("knowledge_entity_resolution"),
            "default policy allows explicit OpenAI knowledge entity resolution after policy and cost checks",
        ),
        default_allow_rule(
            "default-allow-openai-knowledge-cluster-proposal",
            "provider.network",
            Some("arcwell-knowledge"),
            Some("openai"),
            Some("knowledge_cluster_proposal"),
            "default policy allows explicit OpenAI knowledge cluster proposals after policy and cost checks",
        ),
        default_allow_rule(
            "default-allow-openai-knowledge-cluster-writer",
            "provider.network",
            Some("arcwell-knowledge"),
            Some("openai"),
            Some("knowledge_cluster_writer"),
            "default policy allows explicit OpenAI knowledge cluster writer drafts after policy, cost, and quality checks",
        ),
        default_allow_rule(
            "default-allow-perplexity-web-search",
            "provider.network",
            Some("arcwell-deep-research"),
            Some("perplexity"),
            Some("web_search"),
            "default policy allows the existing Perplexity web-search path after policy and cost checks",
        ),
        default_allow_rule(
            "default-allow-x-oauth",
            "provider.oauth",
            Some("arcwell-x"),
            Some("x"),
            Some("x_oauth"),
            "default policy allows explicit X OAuth token exchange and refresh after policy and cost checks",
        ),
        default_allow_rule(
            "default-allow-worker-enqueue",
            "worker.enqueue",
            None,
            None,
            None,
            "default policy allows explicit local worker job enqueue for supported job kinds",
        ),
        default_allow_rule(
            "default-allow-memory-capture",
            "memory.capture",
            Some("arcwell-memory"),
            None,
            None,
            "default policy allows explicit local memory capture into reviewable candidates",
        ),
        default_allow_rule(
            "default-allow-source-card-write",
            "source.write",
            Some("arcwell-llm-wiki"),
            Some("*"),
            Some("source_card_add"),
            "default policy allows explicit source-card writes into the local wiki",
        ),
        default_allow_rule(
            "default-allow-reviewed-memory-apply",
            "memory.apply",
            None,
            None,
            None,
            "default policy allows explicit review-candidate application",
        ),
        default_allow_rule(
            "default-allow-reviewed-profile-write",
            "profile.write",
            None,
            None,
            None,
            "default policy allows explicit review-candidate profile writes",
        ),
        default_allow_rule(
            "default-allow-local-project-write",
            "project.write",
            None,
            None,
            None,
            "default policy allows local manual project writes",
        ),
        default_allow_rule(
            "default-allow-controller-write",
            "controller.write",
            Some("arcwell-controller"),
            Some("*"),
            Some("*"),
            "default policy allows explicit local controller registry writes after channel authorization checks",
        ),
        default_allow_rule(
            "default-allow-controller-stop",
            "controller.stop",
            Some("arcwell-controller"),
            Some("*"),
            Some("*"),
            "default policy allows explicit local controller stop requests after channel authorization checks",
        ),
        default_allow_rule(
            "default-allow-local-secret-read",
            "secret.read",
            None,
            None,
            Some("*"),
            "default policy allows explicit local secret value reads through admin surfaces",
        ),
        default_allow_rule(
            "default-allow-local-secret-write",
            "secret.write",
            None,
            None,
            Some("*"),
            "default policy allows explicit local secret value/ref writes through admin surfaces",
        ),
        default_allow_rule(
            "default-allow-reviewed-procedure-apply",
            "procedure.apply",
            Some("arcwell-procedures"),
            None,
            None,
            "default policy allows explicit reviewed procedure candidate application",
        ),
        PolicyRule {
            id: "default-allow-telegram-send".to_string(),
            effect: "allow".to_string(),
            action: "channel.send".to_string(),
            reason: "default policy allows explicit Telegram sends after channel authorization policy remains available".to_string(),
            package: None,
            provider: Some("telegram".to_string()),
            source: None,
            channel: Some("telegram".to_string()),
            subject: Some("*".to_string()),
            target: None,
            priority: 0,
            expires_at: None,
        },
        PolicyRule {
            id: "default-allow-email-send".to_string(),
            effect: "allow".to_string(),
            action: "channel.send".to_string(),
            reason: "default policy allows explicit email sends after channel authorization policy remains available".to_string(),
            package: Some("arcwell-email".to_string()),
            provider: Some("cloudflare_email".to_string()),
            source: Some("email_send".to_string()),
            channel: Some("email".to_string()),
            subject: Some("*".to_string()),
            target: None,
            priority: 0,
            expires_at: None,
        },
        PolicyRule {
            id: "default-allow-email-retry".to_string(),
            effect: "allow".to_string(),
            action: "channel.send".to_string(),
            reason: "default policy allows due email delivery retries after channel authorization and cost policy remain available".to_string(),
            package: Some("arcwell-email".to_string()),
            provider: Some("cloudflare_email".to_string()),
            source: Some("email_retry".to_string()),
            channel: Some("email".to_string()),
            subject: Some("*".to_string()),
            target: None,
            priority: 0,
            expires_at: None,
        },
    ]
}

pub(crate) fn local_ops_ui_policy_rules() -> Vec<PolicyRule> {
    vec![
        PolicyRule {
            id: "allow-local-ops-ui-actions".to_string(),
            effect: "allow".to_string(),
            action: "ops.*".to_string(),
            reason: "allow authenticated local Arcwell cockpit controls after service-owned HTTP auth, CSRF, idempotency, and local-origin checks".to_string(),
            package: Some("arcwell-cli".to_string()),
            provider: None,
            source: Some("ops-ui".to_string()),
            channel: Some("http".to_string()),
            subject: Some("local-operator".to_string()),
            target: None,
            priority: 40,
            expires_at: None,
        },
        PolicyRule {
            id: "allow-local-ops-ui-x-bookmark-worker-enqueue".to_string(),
            effect: "allow".to_string(),
            action: "worker.enqueue".to_string(),
            reason: "allow authenticated local cockpit X bookmark enqueue requests; provider and source-write gates still run when the worker executes the job".to_string(),
            package: Some("arcwell-x".to_string()),
            provider: Some("x".to_string()),
            source: Some("x_import_bookmarks".to_string()),
            channel: None,
            subject: None,
            target: None,
            priority: 40,
            expires_at: None,
        },
        PolicyRule {
            id: "allow-local-ops-ui-knowledge-worker-enqueue".to_string(),
            effect: "allow".to_string(),
            action: "worker.enqueue".to_string(),
            reason: "allow authenticated local cockpit knowledge job enqueue requests; provider, cost, promotion, and source-write gates still run at execution".to_string(),
            package: Some("arcwell-knowledge".to_string()),
            provider: None,
            source: Some("*".to_string()),
            channel: None,
            subject: None,
            target: None,
            priority: 40,
            expires_at: None,
        },
    ]
}

pub(crate) fn default_allow_rule(
    id: &str,
    action: &str,
    package: Option<&str>,
    provider: Option<&str>,
    source: Option<&str>,
    reason: &str,
) -> PolicyRule {
    PolicyRule {
        id: id.to_string(),
        effect: "allow".to_string(),
        action: action.to_string(),
        reason: reason.to_string(),
        package: package.map(ToOwned::to_owned),
        provider: provider.map(ToOwned::to_owned),
        source: source.map(ToOwned::to_owned),
        channel: None,
        subject: None,
        target: None,
        priority: 0,
        expires_at: None,
    }
}

pub(crate) fn best_policy_rule<'a>(
    rules: &'a [PolicyRule],
    request: &PolicyRequest,
) -> Result<Option<&'a PolicyRule>> {
    Ok(matching_policy_rule_refs(rules, request)?
        .into_iter()
        .next())
}

pub(crate) fn matching_policy_rules(
    rules: &[PolicyRule],
    request: &PolicyRequest,
) -> Result<Vec<PolicyRule>> {
    Ok(matching_policy_rule_refs(rules, request)?
        .into_iter()
        .cloned()
        .collect())
}

pub(crate) fn matching_policy_rule_refs<'a>(
    rules: &'a [PolicyRule],
    request: &PolicyRequest,
) -> Result<Vec<&'a PolicyRule>> {
    let mut matches = Vec::new();
    for rule in rules {
        if policy_rule_expired(rule)? || !policy_rule_matches(rule, request) {
            continue;
        }
        matches.push((
            policy_rule_specificity(rule, request),
            effect_rank(&rule.effect),
            rule,
        ));
    }
    matches.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| right.1.cmp(&left.1))
            .then_with(|| right.2.priority.cmp(&left.2.priority))
            .then_with(|| left.2.id.cmp(&right.2.id))
    });
    Ok(matches.into_iter().map(|(_, _, rule)| rule).collect())
}

pub(crate) fn policy_rule_expired(rule: &PolicyRule) -> Result<bool> {
    let Some(expires_at) = &rule.expires_at else {
        return Ok(false);
    };
    let expires_at = DateTime::parse_from_rfc3339(expires_at)
        .with_context(|| format!("parsing policy rule {} expires_at", rule.id))?
        .with_timezone(&Utc);
    Ok(expires_at <= Utc::now())
}

pub(crate) fn policy_rule_matches(rule: &PolicyRule, request: &PolicyRequest) -> bool {
    pattern_matches(Some(&rule.action), Some(&request.action))
        && pattern_matches(rule.package.as_deref(), request.package.as_deref())
        && pattern_matches(rule.provider.as_deref(), request.provider.as_deref())
        && pattern_matches(rule.source.as_deref(), request.source.as_deref())
        && pattern_matches(rule.channel.as_deref(), request.channel.as_deref())
        && pattern_matches(rule.subject.as_deref(), request.subject.as_deref())
        && pattern_matches(rule.target.as_deref(), request.target.as_deref())
}

pub(crate) fn policy_rule_specificity(rule: &PolicyRule, request: &PolicyRequest) -> i64 {
    pattern_specificity(Some(&rule.action), Some(&request.action))
        + pattern_specificity(rule.package.as_deref(), request.package.as_deref())
        + pattern_specificity(rule.provider.as_deref(), request.provider.as_deref())
        + pattern_specificity(rule.source.as_deref(), request.source.as_deref())
        + pattern_specificity(rule.channel.as_deref(), request.channel.as_deref())
        + pattern_specificity(rule.subject.as_deref(), request.subject.as_deref())
        + pattern_specificity(rule.target.as_deref(), request.target.as_deref())
}

pub(crate) fn pattern_matches(pattern: Option<&str>, value: Option<&str>) -> bool {
    let Some(pattern) = pattern else {
        return true;
    };
    let Some(value) = value else {
        return false;
    };
    if pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return value.starts_with(prefix);
    }
    pattern == value
}

pub(crate) fn pattern_specificity(pattern: Option<&str>, value: Option<&str>) -> i64 {
    let Some(pattern) = pattern else {
        return 0;
    };
    if !pattern_matches(Some(pattern), value) || pattern == "*" {
        return 0;
    }
    if pattern.ends_with('*') { 1 } else { 3 }
}

pub(crate) fn effect_rank(effect: &str) -> i64 {
    match effect {
        "deny" => 4,
        "require_approval" => 3,
        "defer" => 2,
        "allow" => 1,
        _ => 0,
    }
}

pub(crate) fn policy_decision_metadata(request: &PolicyRequest) -> Value {
    let mut metadata = request.metadata.clone();
    if !metadata.is_object() {
        metadata = json!({ "value": metadata });
    }
    if let Some(projected_usd) = request.projected_usd {
        metadata["projected_usd"] = json!(projected_usd);
    }
    if let Some(excerpt) = &request.untrusted_excerpt {
        metadata["untrusted_excerpt"] = json!(sanitize_policy_excerpt(excerpt));
    }
    metadata
}

pub(crate) fn sanitize_policy_excerpt(excerpt: &str) -> String {
    excerpt
        .chars()
        .filter(|ch| *ch == '\n' || *ch == '\t' || !ch.is_control())
        .take(2000)
        .collect()
}

pub(crate) fn secret_ref_location_kind(location: &str) -> &'static str {
    if location.starts_with("env:") {
        "env"
    } else if location.starts_with("file:") {
        "file"
    } else if location.starts_with("keychain:") {
        "keychain"
    } else {
        "other"
    }
}

pub(crate) fn validate_confidence(value: f64) -> Result<()> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        bail!("confidence must be a finite number between 0 and 1");
    }
    Ok(())
}

pub(crate) fn cost_override_active(override_until: Option<&str>) -> Result<bool> {
    let Some(override_until) = override_until else {
        return Ok(false);
    };
    let until = DateTime::parse_from_rfc3339(override_until)
        .with_context(|| format!("parsing override_until timestamp {override_until}"))?
        .with_timezone(&Utc);
    Ok(until > Utc::now())
}

pub(crate) fn estimated_web_search_cost(max_results: usize) -> f64 {
    0.005 + (max_results.clamp(1, 20) as f64 * 0.001)
}

pub(crate) fn estimated_editorial_cost(model: &str, prompt_len: usize) -> f64 {
    let multiplier = if model.contains("mini") || model.contains("small") {
        0.00002
    } else {
        0.00008
    };
    0.01 + ((prompt_len.clamp(1, 500_000) as f64 / 1_000.0) * multiplier)
}

pub(crate) fn estimated_radar_model_score_cost(
    model: &str,
    prompt_len: usize,
    item_count: usize,
) -> f64 {
    let multiplier = if model.contains("mini") || model.contains("small") {
        0.00002
    } else {
        0.00008
    };
    0.005
        + ((prompt_len.clamp(1, 250_000) as f64 / 1_000.0) * multiplier)
        + (item_count.clamp(1, 25) as f64 * 0.0005)
}

pub(crate) fn estimated_x_recent_search_cost(max_results: usize) -> f64 {
    0.002 + (max_results.clamp(10, 100) as f64 * 0.0002)
}

pub(crate) fn estimated_network_fetch_cost(units: usize) -> f64 {
    0.001 + (units.clamp(1, 1000) as f64 * 0.0001)
}

pub(crate) fn estimated_x_following_cost(max_users: usize) -> f64 {
    0.002 + (max_users.clamp(1, 5_000).div_ceil(1_000) as f64 * 0.001)
}

pub(crate) fn estimated_x_definitive_watch_cost(
    max_bookmarks: usize,
    max_recent_follows: usize,
) -> f64 {
    0.002
        + (max_bookmarks.clamp(10, 100_000).div_ceil(100) as f64 * 0.001)
        + if max_recent_follows > 0 { 0.001 } else { 0.0 }
}

pub(crate) fn estimated_x_monitor_cost(max_sources: usize, max_results_per_source: usize) -> f64 {
    0.002
        + (max_sources.clamp(1, X_MONITOR_MAX_SOURCES) as f64
            * (0.0005 + max_results_per_source.clamp(10, 100) as f64 * 0.00005))
}

pub(crate) fn estimated_memory_provider_cost() -> f64 {
    0.002
}

pub(crate) fn estimated_channel_send_cost() -> f64 {
    0.0001
}

pub(crate) fn wiki_job_policy_context(
    kind: &str,
    input: &Value,
) -> (
    &'static str,
    Option<&'static str>,
    Option<String>,
    Option<f64>,
) {
    match kind {
        "ingest_url" => (
            "arcwell-llm-wiki",
            Some("web"),
            input
                .get("url")
                .and_then(Value::as_str)
                .map(|value| excerpt(value, 240)),
            Some(estimated_network_fetch_cost(1)),
        ),
        "ingest_rendered_page" => (
            "arcwell-llm-wiki",
            None,
            input
                .get("requested_url")
                .and_then(Value::as_str)
                .map(|value| excerpt(value, 240)),
            None,
        ),
        "rss_fetch" => (
            "arcwell-llm-wiki",
            Some("rss"),
            input
                .get("url")
                .and_then(Value::as_str)
                .map(|value| excerpt(value, 240)),
            Some(estimated_network_fetch_cost(1)),
        ),
        "github_repo" => (
            "arcwell-llm-wiki",
            Some("github"),
            Some(format!(
                "{}/{}",
                input.get("owner").and_then(Value::as_str).unwrap_or(""),
                input.get("repo").and_then(Value::as_str).unwrap_or("")
            )),
            Some(estimated_network_fetch_cost(
                input.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize,
            )),
        ),
        "github_owner" => (
            "arcwell-llm-wiki",
            Some("github"),
            input
                .get("owner")
                .and_then(Value::as_str)
                .map(|value| excerpt(value, 240)),
            Some(estimated_network_fetch_cost(
                input.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize,
            )),
        ),
        "arxiv_search" => (
            "arcwell-llm-wiki",
            Some("arxiv"),
            input
                .get("query")
                .and_then(Value::as_str)
                .map(|value| excerpt(value, 240)),
            Some(estimated_network_fetch_cost(
                input.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize,
            )),
        ),
        "hackernews_fetch" => (
            "arcwell-llm-wiki",
            Some("hackernews"),
            input
                .get("feed")
                .and_then(Value::as_str)
                .map(|value| excerpt(value, 240)),
            Some(estimated_network_fetch_cost(
                1 + ((input.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize)
                    .clamp(1, 30)
                    * 4),
            )),
        ),
        "reddit_fetch" => (
            "arcwell-llm-wiki",
            Some("reddit"),
            input
                .get("locator")
                .and_then(Value::as_str)
                .map(|value| excerpt(value, 240)),
            Some(estimated_network_fetch_cost(
                1 + ((input.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize)
                    .clamp(1, 30)
                    * 2),
            )),
        ),
        "x_recent_search" => (
            "arcwell-x",
            Some("x"),
            input
                .get("query")
                .and_then(Value::as_str)
                .map(|value| excerpt(value, 240)),
            Some(estimated_x_recent_search_cost(
                input
                    .get("max_results")
                    .and_then(Value::as_u64)
                    .unwrap_or(10) as usize,
            )),
        ),
        "x_import_bookmarks" => (
            "arcwell-x",
            Some("x"),
            Some("bookmarks".to_string()),
            Some(estimated_x_definitive_watch_cost(
                input
                    .get("max_bookmarks")
                    .and_then(Value::as_u64)
                    .unwrap_or(100) as usize,
                0,
            )),
        ),
        "x_monitor_watch_source" => (
            "arcwell-x",
            Some("x"),
            input
                .get("handle")
                .and_then(Value::as_str)
                .map(|value| excerpt(value, 240)),
            Some(estimated_x_monitor_cost(
                1,
                input
                    .get("max_results")
                    .and_then(Value::as_u64)
                    .unwrap_or(10) as usize,
            )),
        ),
        "x_profile_enrichment" => (
            "arcwell-x",
            Some("x"),
            Some("https://api.x.com".to_string()),
            Some(estimated_network_fetch_cost(
                (input.get("limit").and_then(Value::as_u64).unwrap_or(100) as usize)
                    .clamp(1, 1_000)
                    .div_ceil(100),
            )),
        ),
        "research_convergence_run" => (
            "arcwell-deep-research",
            None,
            input
                .get("run_id")
                .and_then(Value::as_str)
                .map(|value| excerpt(value, 240)),
            None,
        ),
        "knowledge_cluster_editorial_decide" => (
            "arcwell-knowledge",
            None,
            input
                .get("cluster_id")
                .and_then(Value::as_str)
                .map(|value| excerpt(value, 240)),
            None,
        ),
        "knowledge_cluster_expand" => (
            "arcwell-knowledge",
            None,
            input
                .get("cluster_id")
                .and_then(Value::as_str)
                .map(|value| excerpt(value, 240)),
            None,
        ),
        "knowledge_cluster_model_write" => (
            "arcwell-knowledge",
            input
                .get("model_provider")
                .and_then(Value::as_str)
                .and_then(|value| match value {
                    "openai" => Some("openai"),
                    _ => None,
                }),
            input
                .get("cluster_id")
                .and_then(Value::as_str)
                .map(|value| excerpt(value, 240)),
            None,
        ),
        "knowledge_entity_resolution_model" => (
            "arcwell-knowledge",
            input
                .get("model_provider")
                .and_then(Value::as_str)
                .and_then(|value| match value {
                    "openai" => Some("openai"),
                    "mock" => Some("mock"),
                    _ => None,
                }),
            Some(format!(
                "{} / {}",
                input
                    .get("left_entity_id")
                    .and_then(Value::as_str)
                    .unwrap_or(""),
                input
                    .get("right_entity_id")
                    .and_then(Value::as_str)
                    .unwrap_or("")
            )),
            None,
        ),
        "knowledge_cluster_backlog" => (
            "arcwell-knowledge",
            None,
            Some("source-cards".to_string()),
            None,
        ),
        "knowledge_cluster_model_propose" => (
            "arcwell-knowledge",
            None,
            input
                .get("query")
                .and_then(Value::as_str)
                .map(|value| excerpt(value, 240)),
            None,
        ),
        "knowledge_cluster_investigate" => (
            "arcwell-knowledge",
            None,
            input
                .get("cluster_id")
                .and_then(Value::as_str)
                .map(|value| excerpt(value, 240)),
            None,
        ),
        "knowledge_cluster_investigation_execute" => (
            "arcwell-knowledge",
            None,
            input
                .get("cluster_id")
                .and_then(Value::as_str)
                .map(|value| excerpt(value, 240)),
            None,
        ),
        "radar_scheduled_delivery" => (
            "arcwell-radar",
            None,
            input
                .get("tick_id")
                .and_then(Value::as_str)
                .map(|value| excerpt(value, 120)),
            None,
        ),
        "knowledge_daily_briefing" => (
            "arcwell-knowledge",
            None,
            input
                .get("tick_id")
                .and_then(Value::as_str)
                .map(|value| excerpt(value, 120)),
            None,
        ),
        "email_delivery_verification_request" => (
            "arcwell-knowledge",
            None,
            input
                .get("verification_state")
                .and_then(Value::as_str)
                .map(|value| excerpt(value, 120)),
            None,
        ),
        "email_delivery_mailbox_repair" => (
            "arcwell-knowledge",
            None,
            input
                .get("verification_state")
                .and_then(Value::as_str)
                .map(|value| excerpt(value, 120)),
            None,
        ),
        "radar_run" => (
            "arcwell-radar",
            None,
            input
                .get("profile")
                .and_then(Value::as_str)
                .map(|value| excerpt(value, 240)),
            if input
                .get("fetch_live")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                Some(estimated_network_fetch_cost(10))
            } else {
                None
            },
        ),
        "job_radar_refresh" => {
            let fetch_live = input
                .get("fetch_live")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            (
                "arcwell-job-hunting",
                fetch_live.then_some("web"),
                input
                    .get("scope")
                    .and_then(Value::as_str)
                    .map(|value| excerpt(value, 240)),
                fetch_live.then(|| {
                    estimated_network_fetch_cost(
                        input
                            .get("source_ids")
                            .and_then(Value::as_array)
                            .map(Vec::len)
                            .unwrap_or(1)
                            .clamp(1, 50),
                    )
                }),
            )
        }
        _ => ("arcwell-llm-wiki", None, Some(kind.to_string()), None),
    }
}

pub(crate) fn policy_safe_job_input(input: &Value) -> Value {
    match input {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (key, value) in map {
                let safe = match value {
                    Value::String(value) => json!(excerpt(value, 240)),
                    Value::Number(_) | Value::Bool(_) | Value::Null => value.clone(),
                    _ => json!(excerpt(&value.to_string(), 240)),
                };
                out.insert(key.clone(), safe);
            }
            Value::Object(out)
        }
        other => json!(excerpt(&other.to_string(), 240)),
    }
}

#[derive(Debug, Clone)]
pub(crate) struct KnowledgeAdapterContext {
    pub(crate) provider: String,
    pub(crate) source_kind: String,
    pub(crate) locator: String,
    pub(crate) cursor_key: Option<String>,
}

pub(crate) fn knowledge_adapter_context_for_job(
    job: &WikiJob,
) -> Result<Option<KnowledgeAdapterContext>> {
    let input = &job.input_json;
    let context = match job.kind.as_str() {
        "rss_fetch" => {
            let Some(url) = input.get("url").and_then(Value::as_str) else {
                return Ok(None);
            };
            let canonical = canonical_source_url(url).unwrap_or_else(|_| url.to_string());
            KnowledgeAdapterContext {
                provider: "rss".to_string(),
                source_kind: "rss".to_string(),
                locator: url.to_string(),
                cursor_key: Some(format!("rss:{canonical}")),
            }
        }
        "github_repo" => {
            let owner = input.get("owner").and_then(Value::as_str).unwrap_or("");
            let repo = input.get("repo").and_then(Value::as_str).unwrap_or("");
            let mode = input
                .get("mode")
                .and_then(Value::as_str)
                .unwrap_or("releases");
            if owner.is_empty() || repo.is_empty() {
                return Ok(None);
            }
            KnowledgeAdapterContext {
                provider: "github".to_string(),
                source_kind: "github_repo".to_string(),
                locator: format!("{owner}/{repo}:{mode}"),
                cursor_key: Some(format!("github:{owner}/{repo}:{mode}")),
            }
        }
        "github_owner" => {
            let Some(owner) = input.get("owner").and_then(Value::as_str) else {
                return Ok(None);
            };
            KnowledgeAdapterContext {
                provider: "github".to_string(),
                source_kind: "github_owner".to_string(),
                locator: owner.to_string(),
                cursor_key: Some(format!("github-owner:{owner}")),
            }
        }
        "arxiv_search" => {
            let Some(query) = input.get("query").and_then(Value::as_str) else {
                return Ok(None);
            };
            KnowledgeAdapterContext {
                provider: "arxiv".to_string(),
                source_kind: "arxiv_query".to_string(),
                locator: query.to_string(),
                cursor_key: Some(format!("arxiv:{query}")),
            }
        }
        "hackernews_fetch" => {
            let feed = input
                .get("feed")
                .or_else(|| input.get("locator"))
                .and_then(Value::as_str)
                .unwrap_or("topstories");
            let feed = normalize_hackernews_feed(feed)?;
            KnowledgeAdapterContext {
                provider: "hackernews".to_string(),
                source_kind: "hackernews".to_string(),
                locator: feed.clone(),
                cursor_key: Some(format!("hackernews:{feed}")),
            }
        }
        "reddit_fetch" => {
            let Some(locator_raw) = input.get("locator").and_then(Value::as_str) else {
                return Ok(None);
            };
            let locator = normalize_reddit_locator(locator_raw)?;
            KnowledgeAdapterContext {
                provider: "reddit".to_string(),
                source_kind: "reddit".to_string(),
                locator: locator.source_detail(),
                cursor_key: Some(format!("reddit:{}", locator.source_detail())),
            }
        }
        "x_recent_search" => {
            let Some(query) = input.get("query").and_then(Value::as_str) else {
                return Ok(None);
            };
            KnowledgeAdapterContext {
                provider: "x".to_string(),
                source_kind: "x_recent_search".to_string(),
                locator: query.to_string(),
                cursor_key: Some(format!("x:recent-search:{query}")),
            }
        }
        "x_import_bookmarks" => KnowledgeAdapterContext {
            provider: "x".to_string(),
            source_kind: "x_bookmarks".to_string(),
            locator: "bookmarks".to_string(),
            cursor_key: Some("x:bookmarks".to_string()),
        },
        "x_monitor_watch_source" => {
            let Some(handle) = input.get("handle").and_then(Value::as_str) else {
                return Ok(None);
            };
            let handle = handle.trim().trim_start_matches('@');
            KnowledgeAdapterContext {
                provider: "x".to_string(),
                source_kind: "x_watch".to_string(),
                locator: handle.to_string(),
                cursor_key: Some(format!("x:watch:{handle}")),
            }
        }
        _ => return Ok(None),
    };
    Ok(Some(context))
}

pub(crate) fn adapter_result_shape(result: &Value) -> Value {
    json!({
        "has_source_cards": result.get("source_cards").and_then(Value::as_array).is_some(),
        "source_card_count": result.get("source_cards").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
        "has_cursor": result.get("cursor").is_some() || result.get("cursor_key").is_some(),
        "has_cursor_value": result.get("cursor_value").is_some() || result.get("new_cursor").is_some(),
        "keys": result.as_object().map(|object| object.keys().cloned().collect::<Vec<_>>()).unwrap_or_default()
    })
}

pub(crate) fn adapter_source_card_ids_from_result(result: &Value) -> Vec<String> {
    result
        .get("source_cards")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub(crate) fn knowledge_cluster_ids_from_result(result: &Value) -> Vec<String> {
    result
        .get("cluster_ids")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .filter(|value| validate_id(value).is_ok())
                .map(ToOwned::to_owned)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub(crate) fn classify_source_adapter_error_kind(error: &str) -> String {
    let lower = error.to_ascii_lowercase();
    if lower.contains("401") || lower.contains("unauthorized") || lower.contains("credential") {
        "auth".to_string()
    } else if lower.contains("403") || lower.contains("forbidden") || lower.contains("policy") {
        "policy_or_permission".to_string()
    } else if lower.contains("429") || lower.contains("rate limit") || lower.contains("quota") {
        "rate_limited".to_string()
    } else if lower.contains("500")
        || lower.contains("502")
        || lower.contains("503")
        || lower.contains("504")
        || lower.contains("timeout")
        || lower.contains("temporarily")
    {
        "transient_provider".to_string()
    } else if lower.contains("json")
        || lower.contains("malformed")
        || lower.contains("parse")
        || lower.contains("schema")
    {
        "malformed_provider_payload".to_string()
    } else {
        "provider_or_adapter_error".to_string()
    }
}

pub(crate) fn provider_network_source_for_job(kind: &str) -> &str {
    match kind {
        "ingest_url" => "url_ingest",
        "ingest_rendered_page" => "rendered_page_snapshot",
        "x_import_bookmarks" => "x_import_bookmarks",
        "x_monitor_watch_source" => "x_monitor",
        "x_profile_enrichment" => "x_profile_enrichment",
        "knowledge_cluster_model_write" => "knowledge_cluster_writer",
        "knowledge_entity_resolution_model" => "knowledge_entity_resolution",
        "job_radar_refresh" => "job_source_refresh",
        other => other,
    }
}

pub(crate) fn deferred_job_until(result: &Value) -> Result<Option<String>> {
    if result.get("status").and_then(Value::as_str) != Some("deferred") {
        return Ok(None);
    }
    let deferred_until = result
        .get("deferred_until")
        .and_then(Value::as_str)
        .context("deferred job result requires deferred_until")?;
    validate_timestamp(deferred_until)?;
    Ok(Some(deferred_until.to_string()))
}

pub(crate) fn scheduled_job_cost_projection(
    job: &WikiJob,
) -> Result<Option<(&'static str, &'static str, &'static str, f64)>> {
    let projection = match job.kind.as_str() {
        "ingest_url" => Some((
            "web",
            "url_ingest",
            "ingest_url",
            estimated_network_fetch_cost(1),
        )),
        "rss_fetch" => Some((
            "rss",
            "rss_fetch",
            "rss_fetch",
            estimated_network_fetch_cost(1),
        )),
        "github_repo" => {
            let limit = job
                .input_json
                .get("limit")
                .and_then(Value::as_u64)
                .unwrap_or(10) as usize;
            Some((
                "github",
                "github_repo",
                "github_repo",
                estimated_network_fetch_cost(limit.clamp(1, 30)),
            ))
        }
        "github_owner" => {
            let limit = job
                .input_json
                .get("limit")
                .and_then(Value::as_u64)
                .unwrap_or(10) as usize;
            Some((
                "github",
                "github_owner",
                "github_owner",
                estimated_network_fetch_cost(limit.clamp(1, 30)),
            ))
        }
        "arxiv_search" => {
            let limit = job
                .input_json
                .get("limit")
                .and_then(Value::as_u64)
                .unwrap_or(10) as usize;
            Some((
                "arxiv",
                "arxiv_search",
                "arxiv_search",
                estimated_network_fetch_cost(limit.clamp(1, 30)),
            ))
        }
        "hackernews_fetch" => {
            let limit = job
                .input_json
                .get("limit")
                .and_then(Value::as_u64)
                .unwrap_or(10) as usize;
            Some((
                "hackernews",
                "hackernews_fetch",
                "hackernews_fetch",
                estimated_network_fetch_cost(1 + (limit.clamp(1, 30) * 4)),
            ))
        }
        "reddit_fetch" => {
            let limit = job
                .input_json
                .get("limit")
                .and_then(Value::as_u64)
                .unwrap_or(10) as usize;
            Some((
                "reddit",
                "reddit_fetch",
                "reddit_fetch",
                estimated_network_fetch_cost(1 + (limit.clamp(1, 30) * 2)),
            ))
        }
        "x_recent_search"
        | "x_import_bookmarks"
        | "x_monitor_watch_source"
        | "x_profile_enrichment" => None,
        "job_radar_refresh" => {
            if job
                .input_json
                .get("fetch_live")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                let source_count = job
                    .input_json
                    .get("source_ids")
                    .and_then(Value::as_array)
                    .map(Vec::len)
                    .unwrap_or(1)
                    .clamp(1, 50);
                Some((
                    "web",
                    "job_source_refresh",
                    "job_radar_refresh",
                    estimated_network_fetch_cost(source_count),
                ))
            } else {
                None
            }
        }
        "research_convergence_run" => None,
        _ => None,
    };
    Ok(projection)
}
