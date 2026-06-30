use super::*;

#[derive(Debug, Default)]
pub(crate) struct OpsUiOptions {
    pub(crate) q: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) sort: String,
    pub(crate) detail: Option<String>,
    pub(crate) notice: Option<String>,
}

impl OpsUiOptions {
    pub(crate) fn from_query(query: OpsUiQuery) -> Self {
        Self {
            q: trimmed_non_empty(query.q),
            status: trimmed_non_empty(query.status),
            sort: trimmed_non_empty(query.sort).unwrap_or_else(|| "updated_desc".to_string()),
            detail: trimmed_non_empty(query.detail),
            notice: trimmed_non_empty(query.notice),
        }
    }
}

#[cfg(test)]
pub(crate) fn render_ops_ui(snapshot: &OpsSnapshot) -> String {
    render_ops_ui_with_options(snapshot, &OpsUiOptions::default(), None, false)
}

pub(crate) fn render_ops_ui_with_options(
    snapshot: &OpsSnapshot,
    options: &OpsUiOptions,
    csrf_token: Option<&str>,
    controls_enabled: bool,
) -> String {
    let health_class = if snapshot.health.ok { "ok" } else { "bad" };
    let failed_deliveries = snapshot
        .channel_delivery_attempts
        .iter()
        .filter(|attempt| !attempt.ok)
        .count();
    let failed_radar_deliveries = snapshot
        .radar_deliveries
        .iter()
        .filter(|delivery| matches!(delivery.status.as_str(), "failed" | "blocked"))
        .count();
    let failed_job_sources = snapshot.job_hunting.source_health_failures.len();
    let job_privacy_blocks = snapshot
        .job_hunting
        .privacy_decision_counts
        .get("block")
        .copied()
        .unwrap_or(0);
    let health_score = ops_health_score(snapshot);
    let mut html = String::new();
    html.push_str(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Arcwell Ops</title>
<style>
:root{color-scheme:light dark;font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif}
*{box-sizing:border-box}
body{margin:0;background:#f6f7f9;color:#1f2328}
main{max-width:1440px;margin:0 auto;padding:24px}
h1{font-size:28px;margin:0 0 6px}
h2{font-size:18px;margin:0 0 10px}
p{margin:4px 0 14px}.muted{color:#57606a}.notice{border-left:4px solid #1f6feb;padding:8px 10px;background:white}
.section{margin-top:24px}
.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(160px,1fr));gap:8px}
.metric{border:1px solid #d8dee4;background:white;padding:10px;border-radius:6px;min-width:0}
.metric span{display:block;color:#57606a;font-size:12px}.metric b{display:block;font-size:22px;line-height:1.15;margin-top:4px;overflow-wrap:anywhere}
.summary-grid .metric b{font-size:16px;line-height:1.25}
.ops-form{display:grid;grid-template-columns:2fr 1fr 1fr auto;gap:8px;align-items:end;margin-top:18px}
.ops-form label{display:grid;gap:4px;font-size:12px;color:#57606a}
.control-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:8px;margin-top:14px}
.control-grid form{border:1px solid #d8dee4;background:white;border-radius:6px;padding:10px;display:grid;gap:8px;min-width:0}
.control-grid .fields{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:6px}
.control-grid label{display:grid;gap:4px;font-size:12px;color:#57606a}
input,select,button{font:inherit;border:1px solid #d8dee4;border-radius:6px;background:white;color:inherit;padding:7px;max-width:100%;min-width:0}
button{font-weight:600;cursor:pointer}.danger{color:#b42318}.actions form{display:flex;gap:6px;flex-wrap:wrap}.actions input[name=reason]{min-width:220px}
.detail{border:1px solid #d8dee4;background:white;padding:12px;border-radius:6px}
.ok{color:#116329}.bad{color:#b42318}.warn{color:#9a6700}.pill{font-size:13px;font-weight:600}
table{width:100%;border-collapse:collapse;background:white;border:1px solid #d8dee4}
th,td{text-align:left;border-bottom:1px solid #d8dee4;padding:8px;vertical-align:top;font-size:13px;overflow-wrap:anywhere;word-break:break-word;max-width:520px}
th{white-space:nowrap;overflow-wrap:normal;word-break:normal}
th{background:#eef2f6}
a{color:#0969da;text-decoration:none}a:hover{text-decoration:underline}
code,pre{white-space:pre-wrap;word-break:break-word}
.bar{display:flex;gap:2px;align-items:stretch;min-width:120px;height:12px}
.bar span{display:block;min-width:1px;border-radius:2px}
.bar .selected{background:#1f883d}.bar .over{background:#9a6700}.bar .below{background:#6e7781}.bar .duplicate{background:#8250df}.bar .quota{background:#bf8700}.bar .other{background:#57606a}
.scroll{overflow:auto}
@media (max-width:720px){main{padding:14px}h1{font-size:24px}.grid{grid-template-columns:repeat(2,minmax(0,1fr))}.ops-form,.control-grid .fields{grid-template-columns:1fr}th,td{font-size:12px;padding:7px;max-width:none}.scroll{overflow:visible}.ops-table{border:0;background:transparent}.ops-table thead{position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0)}.ops-table tr{display:block;border:1px solid #d8dee4;border-radius:6px;background:white;margin:0 0 8px;overflow:hidden}.ops-table td{display:grid;grid-template-columns:minmax(94px,34%) minmax(0,1fr);gap:8px;border-bottom:1px solid #d8dee4}.ops-table td::before{content:attr(data-label);font-weight:600;color:#57606a;overflow-wrap:normal;word-break:normal}.ops-table td[colspan]{display:block}.ops-table td[colspan]::before{content:""}}
@media (prefers-color-scheme:dark){body{background:#0d1117;color:#e6edf3}.muted,.metric span,.ops-form label,.control-grid label{color:#8b949e}.metric,table,.detail,.notice,.control-grid form,input,select,button{background:#161b22;border-color:#30363d}th,td{border-color:#30363d}th{background:#21262d}.ops-table tr{background:#161b22;border-color:#30363d}.ops-table td::before{color:#8b949e}a{color:#58a6ff}}
</style>
</head>
<body><main>"#,
    );
    html.push_str(&format!(
        "<h1>Arcwell Ops <span class=\"pill {}\">{}</span></h1>",
        health_class,
        if snapshot.health.ok {
            "healthy"
        } else {
            "needs attention"
        }
    ));
    html.push_str("<p class=\"muted\">Local operations snapshot with filtered queues, source health, credential summaries, and narrow authenticated remediation controls where supported.</p>");
    if let Some(notice) = &options.notice {
        html.push_str(&format!(
            "<p class=\"notice\">{}</p>",
            html_escape(&ops_notice_text(notice))
        ));
    }
    html.push_str(&render_ops_filter_form(options));
    html.push_str(&render_x_ops_control_panel(csrf_token, controls_enabled));
    html.push_str(&render_knowledge_ops_control_panel(
        csrf_token,
        controls_enabled,
    ));
    html.push_str("<section class=\"grid\">");
    for (label, value) in [
        ("Health score", health_score.score as usize),
        ("Jobs", snapshot.jobs.len()),
        ("Dead letters", snapshot.health.dead_lettered_jobs as usize),
        ("Edge events", snapshot.edge_events.len()),
        ("Cursors", snapshot.cursors.len()),
        ("Sources", snapshot.watch_sources.len()),
        ("Source health", snapshot.source_health.len()),
        ("Radar runs", snapshot.radar_runs.len()),
        ("Radar source quality", snapshot.radar_source_quality.len()),
        ("Radar deliveries", snapshot.radar_deliveries.len()),
        (
            "Knowledge adapter runs",
            snapshot.knowledge_adapter_runs.len(),
        ),
        ("Knowledge entities", snapshot.knowledge_entities.len()),
        (
            "Knowledge resolutions",
            snapshot.knowledge_entity_resolutions.len(),
        ),
        ("Knowledge relations", snapshot.knowledge_relations.len()),
        ("Knowledge events", snapshot.knowledge_events.len()),
        ("Knowledge clusters", snapshot.knowledge_clusters.len()),
        (
            "Knowledge editorial",
            snapshot.knowledge_editorial_decisions.len(),
        ),
        ("Knowledge reports", snapshot.knowledge_reports.len()),
        ("X clusters", snapshot.x_knowledge_clusters.len()),
        (
            "X editorial decisions",
            snapshot.x_editorial_decisions.len(),
        ),
        ("Source cards", snapshot.source_cards.len()),
        ("Projects", snapshot.projects.len()),
        ("Project statuses", snapshot.project_status_snapshots.len()),
        ("Channels", snapshot.channel_messages.len()),
        ("Telegram failures", failed_deliveries),
        ("Radar delivery failures", failed_radar_deliveries),
        ("Job roles", snapshot.job_hunting.role_count),
        ("Job source failures", failed_job_sources),
        ("Job privacy blocks", job_privacy_blocks),
        ("Job follow-ups", snapshot.job_hunting.follow_up_count),
        ("Import runs", snapshot.import_runs.len()),
        ("Memory candidates", snapshot.memory_candidates.len()),
        ("Procedure candidates", snapshot.procedure_candidates.len()),
        ("Work runs", snapshot.work_runs.len()),
        ("Policy approvals", snapshot.policy_approvals.len()),
        ("Secrets", snapshot.secret_health.len()),
        ("Cost policies", snapshot.cost_policies.len()),
    ] {
        html.push_str(&format!(
            "<div class=\"metric\"><span>{}</span><b>{}</b></div>",
            html_escape(label),
            value
        ));
    }
    html.push_str("</section>");
    html.push_str(&render_ops_summary(snapshot, &health_score));
    if let Some(detail) = &options.detail {
        html.push_str(&render_ops_detail(snapshot, detail));
    }
    if !snapshot.health.warnings.is_empty() {
        html.push_str("<section class=\"section\"><h2>Warnings</h2><ul>");
        for warning in &snapshot.health.warnings {
            html.push_str(&format!("<li class=\"warn\">{}</li>", html_escape(warning)));
        }
        html.push_str("</ul></section>");
    }
    html.push_str("<section class=\"section\"><h2>Worker Heartbeat</h2>");
    if let Some(heartbeat) = &snapshot.health.latest_worker_heartbeat {
        html.push_str(&format!(
            "<pre>{}</pre>",
            html_escape(&serde_json::to_string_pretty(heartbeat).unwrap_or_default())
        ));
    } else {
        html.push_str("<p class=\"bad\">No worker heartbeat recorded.</p>");
    }
    if !snapshot.health.latest_worker_heartbeat_events.is_empty() {
        html.push_str("<h3>Recent heartbeat events</h3><pre>");
        html.push_str(&html_escape(
            &serde_json::to_string_pretty(&snapshot.health.latest_worker_heartbeat_events)
                .unwrap_or_default(),
        ));
        html.push_str("</pre>");
    }
    html.push_str("</section>");
    html.push_str(&ops_table(
        "Health And Backups",
        &["home", "db", "schema", "latest backup", "warnings"],
        [vec![
            snapshot.health.home.display().to_string(),
            snapshot.health.db.display().to_string(),
            snapshot.health.schema_version.to_string(),
            snapshot.health.latest_backup.clone().unwrap_or_default(),
            snapshot.health.warnings.join("\n"),
        ]],
    ));
    html.push_str(&ops_table_with_raw_columns(
        "Jobs",
        &[
            "id", "kind", "status", "attempts", "lineage", "worker", "next run", "updated", "error",
        ],
        filtered_jobs(snapshot, options)
            .into_iter()
            .take(75)
            .map(|job| {
                vec![
                    detail_link("job", &job.id, &short_id(&job.id)),
                    job.kind.clone(),
                    job.status.clone(),
                    format!("{}/{}", job.attempts, job.max_attempts),
                    job_lineage_summary(job),
                    job.worker_id.clone().unwrap_or_default(),
                    job.next_run_at.clone().unwrap_or_default(),
                    job.updated_at.clone(),
                    job.error.clone().unwrap_or_default(),
                ]
            }),
        &[0],
    ));
    html.push_str(&ops_table_with_raw_columns(
        "Edge Events",
        &[
            "id", "source", "status", "attempts", "updated", "error", "action",
        ],
        filtered_edge_events(snapshot, options)
            .into_iter()
            .take(75)
            .map(|event| {
                vec![
                    detail_link("edge", &event.id, &short_id(&event.id)),
                    event.source.clone(),
                    event.status.clone(),
                    format!("{}/{}", event.attempts, event.max_attempts),
                    event.updated_at.clone(),
                    event.error.clone().unwrap_or_default(),
                    render_edge_event_action(event, csrf_token, controls_enabled),
                ]
            }),
        &[0, 6],
    ));
    html.push_str(&ops_table(
        "Cursors",
        &["key", "value", "updated"],
        snapshot.cursors.iter().take(100).map(|cursor| {
            vec![
                cursor.key.clone(),
                cursor.value.clone(),
                cursor.updated_at.clone(),
            ]
        }),
    ));
    html.push_str(&ops_table(
        "Watch Sources",
        &["kind", "label", "locator", "cadence", "status", "updated"],
        filtered_watch_sources(snapshot, options)
            .into_iter()
            .take(100)
            .map(|source| {
                vec![
                    source.source_kind.clone(),
                    source.label.clone(),
                    source.locator.clone(),
                    source.cadence.clone(),
                    source.status.clone(),
                    source.updated_at.clone(),
                ]
            }),
    ));
    html.push_str(&ops_table(
        "Source Health",
        &[
            "provider",
            "kind",
            "locator",
            "status",
            "last success",
            "last failure",
            "error",
        ],
        filtered_source_health(snapshot, options)
            .into_iter()
            .take(100)
            .map(|health| {
                vec![
                    health.provider.clone(),
                    health.source_kind.clone(),
                    health.locator.clone(),
                    health.status.clone(),
                    health.last_success_at.clone().unwrap_or_default(),
                    health.last_failure_at.clone().unwrap_or_default(),
                    health.last_error.clone().unwrap_or_default(),
                ]
            }),
    ));
    html.push_str(&ops_table(
        "Job Hunting Stale Or Closed Roles",
        &[
            "company",
            "role",
            "status",
            "source",
            "confidence",
            "updated",
        ],
        snapshot
            .job_hunting
            .stale_or_closed_roles
            .iter()
            .take(50)
            .map(|role| {
                vec![
                    role.company.clone(),
                    role.role_title.clone(),
                    role.current_status.clone(),
                    role.source_url.clone(),
                    role.source_confidence.clone(),
                    role.updated_at.clone(),
                ]
            }),
    ));
    html.push_str(&ops_table(
        "Job Hunting Source Health Failures",
        &[
            "source", "status", "http", "error", "fetched", "accepted", "rejected", "note",
        ],
        snapshot
            .job_hunting
            .source_health_failures
            .iter()
            .take(50)
            .map(|health| {
                vec![
                    short_id(&health.source_id),
                    health.status.clone(),
                    health
                        .http_status
                        .map(|status| status.to_string())
                        .unwrap_or_default(),
                    health.error_code.clone().unwrap_or_default(),
                    health.fetched_count.to_string(),
                    health.accepted_count.to_string(),
                    health.rejected_count.to_string(),
                    health.note.clone().unwrap_or_default(),
                ]
            }),
    ));
    html.push_str(&ops_table(
        "Knowledge Entities",
        &[
            "entity",
            "type",
            "name",
            "canonical",
            "sources",
            "confidence",
            "updated",
        ],
        snapshot.knowledge_entities.iter().take(100).map(|entity| {
            vec![
                short_id(&entity.id),
                entity.entity_type.clone(),
                entity.name.clone(),
                entity.canonical_key.clone(),
                entity.source_card_ids.len().to_string(),
                format!("{:.2}", entity.confidence),
                entity.updated_at.clone(),
            ]
        }),
    ));
    html.push_str(&ops_table(
        "Knowledge Relations",
        &[
            "relation",
            "type",
            "subject",
            "object",
            "sources",
            "confidence",
            "updated",
        ],
        snapshot
            .knowledge_relations
            .iter()
            .take(100)
            .map(|relation| {
                vec![
                    short_id(&relation.id),
                    relation.relation_type.clone(),
                    short_id(&relation.subject_entity_id),
                    short_id(&relation.object_entity_id),
                    relation.source_card_ids.len().to_string(),
                    format!("{:.2}", relation.confidence),
                    relation.updated_at.clone(),
                ]
            }),
    ));
    html.push_str(&ops_table(
        "Knowledge Adapter Runs",
        &[
            "adapter", "provider", "kind", "locator", "status", "accepted", "rejected", "cursor",
            "updated",
        ],
        snapshot.knowledge_adapter_runs.iter().take(100).map(|run| {
            vec![
                short_id(&run.id),
                run.provider.clone(),
                run.source_kind.clone(),
                run.locator.clone(),
                run.status.clone(),
                run.accepted_count.to_string(),
                run.rejected_count.to_string(),
                run.cursor_key.clone().unwrap_or_default(),
                run.updated_at.clone(),
            ]
        }),
    ));
    html.push_str(&ops_table(
        "Knowledge Entity Resolutions",
        &[
            "resolution",
            "decision",
            "status",
            "confidence",
            "resolver",
            "sources",
            "reason",
            "updated",
        ],
        snapshot
            .knowledge_entity_resolutions
            .iter()
            .take(100)
            .map(|resolution| {
                vec![
                    short_id(&resolution.id),
                    resolution.decision.clone(),
                    resolution.status.clone(),
                    format!("{:.2}", resolution.confidence),
                    resolution.resolver.clone(),
                    resolution.source_card_ids.len().to_string(),
                    resolution.reason.clone(),
                    resolution.updated_at.clone(),
                ]
            }),
    ));
    html.push_str(&ops_table(
        "Knowledge Events",
        &["event", "type", "status", "title", "confidence", "updated"],
        snapshot.knowledge_events.iter().take(100).map(|event| {
            vec![
                short_id(&event.id),
                event.event_type.clone(),
                event.status.clone(),
                event.title.clone(),
                format!("{:.2}", event.confidence),
                event.updated_at.clone(),
            ]
        }),
    ));
    html.push_str(&ops_table(
        "Knowledge Clusters",
        &[
            "cluster", "topic", "status", "sources", "events", "novelty", "momentum", "updated",
        ],
        snapshot.knowledge_clusters.iter().take(100).map(|cluster| {
            vec![
                short_id(&cluster.id),
                cluster.topic.clone(),
                cluster.status.clone(),
                cluster.source_card_ids.len().to_string(),
                cluster.event_ids.len().to_string(),
                format!("{:.2}", cluster.novelty_score),
                format!("{:.2}", cluster.momentum_score),
                cluster.updated_at.clone(),
            ]
        }),
    ));
    html.push_str(&ops_table(
        "Knowledge Reports",
        &["report", "cluster", "status", "title", "sources", "updated"],
        snapshot.knowledge_reports.iter().take(100).map(|report| {
            vec![
                short_id(&report.id),
                short_id(&report.cluster_id),
                report.status.clone(),
                report.title.clone(),
                report.source_card_ids.len().to_string(),
                report.updated_at.clone(),
            ]
        }),
    ));
    html.push_str(&ops_table_with_raw_columns(
        "X Knowledge Clusters",
        &[
            "cluster", "topic", "status", "sources", "novelty", "momentum", "stale", "reason",
            "updated",
        ],
        filtered_x_knowledge_clusters(snapshot, options)
            .into_iter()
            .take(100)
            .map(|cluster| {
                vec![
                    detail_link("x-cluster", &cluster.id, &short_id(&cluster.id)),
                    cluster.topic.clone(),
                    cluster.status.clone(),
                    cluster.source_card_ids.len().to_string(),
                    format!("{:.2}", cluster.novelty_score),
                    format!("{:.2}", cluster.momentum_score),
                    format!("{:.2}", cluster.stale_score),
                    cluster.reason.clone(),
                    cluster.updated_at.clone(),
                ]
            }),
        &[0],
    ));
    html.push_str(&ops_table_with_raw_columns(
        "X Editorial Decisions",
        &[
            "decision",
            "cluster",
            "action",
            "status",
            "wiki page",
            "digest candidate",
            "sources",
            "reason",
            "updated",
        ],
        filtered_x_editorial_decisions(snapshot, options)
            .into_iter()
            .take(100)
            .map(|decision| {
                vec![
                    detail_link("x-editorial", &decision.id, &short_id(&decision.id)),
                    detail_link(
                        "x-cluster",
                        &decision.cluster_id,
                        &short_id(&decision.cluster_id),
                    ),
                    decision.decision.clone(),
                    decision.status.clone(),
                    decision.wiki_page_id.clone().unwrap_or_default(),
                    decision.digest_candidate_id.clone().unwrap_or_default(),
                    decision.source_card_ids.len().to_string(),
                    decision.reason.clone(),
                    decision.updated_at.clone(),
                ]
            }),
        &[0, 1],
    ));
    html.push_str(&ops_table_with_raw_columns(
        "Radar Runs",
        &[
            "run",
            "status",
            "raw",
            "scored",
            "selected",
            "distribution",
            "avg score",
            "p50",
            "p90",
            "window",
        ],
        filtered_radar_runs(snapshot, options)
            .into_iter()
            .take(100)
            .map(|run| {
                let distribution = run
                    .metadata
                    .get("score_distribution")
                    .unwrap_or(&Value::Null);
                vec![
                    detail_link("radar-run", &run.id, &short_id(&run.id)),
                    format!("{} / {}", run.status, run.stage),
                    run.raw_count.to_string(),
                    radar_distribution_u64(distribution, "score_count")
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| run.scored_count.to_string()),
                    radar_distribution_u64(distribution, "selected_count")
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| run.filtered_count.to_string()),
                    render_radar_score_bar(distribution),
                    radar_distribution_f64(distribution, "average")
                        .map(|value| format!("{value:.2}"))
                        .unwrap_or_default(),
                    radar_distribution_f64(distribution, "p50")
                        .map(|value| format!("{value:.2}"))
                        .unwrap_or_default(),
                    radar_distribution_f64(distribution, "p90")
                        .map(|value| format!("{value:.2}"))
                        .unwrap_or_default(),
                    format!("{} -> {}", run.window_start, run.window_end),
                ]
            }),
        &[0, 5],
    ));
    html.push_str(&ops_table(
        "Radar Source Quality",
        &[
            "run",
            "kind",
            "locator",
            "status",
            "raw",
            "accepted",
            "avg score",
            "signal/noise",
            "duplicate rate",
            "failures",
            "window",
        ],
        filtered_radar_source_quality(snapshot, options)
            .into_iter()
            .take(100)
            .map(|quality| {
                vec![
                    short_id(&quality.run_id),
                    quality.source_kind.clone(),
                    quality.locator.clone(),
                    quality.status.clone(),
                    quality.raw_count.to_string(),
                    quality.accepted_count.to_string(),
                    quality
                        .average_score
                        .map(|value| format!("{value:.2}"))
                        .unwrap_or_default(),
                    quality
                        .signal_to_noise
                        .map(|value| format!("{value:.2}"))
                        .unwrap_or_default(),
                    quality
                        .duplicate_rate
                        .map(|value| format!("{value:.2}"))
                        .unwrap_or_default(),
                    quality.failure_count.to_string(),
                    format!("{} -> {}", quality.window_start, quality.window_end),
                ]
            }),
    ));
    html.push_str(&ops_table(
        "Radar Deliveries",
        &[
            "run",
            "summary",
            "channel",
            "recipient",
            "status",
            "channel attempt",
            "error",
            "updated",
        ],
        filtered_radar_deliveries(snapshot, options)
            .into_iter()
            .take(100)
            .map(|delivery| {
                vec![
                    short_id(&delivery.run_id),
                    short_id(&delivery.summary_id),
                    delivery.channel.clone(),
                    delivery.recipient_ref.clone(),
                    delivery.status.clone(),
                    delivery
                        .delivery_attempt_id
                        .as_deref()
                        .map(short_id)
                        .unwrap_or_default(),
                    delivery.error.clone().unwrap_or_default(),
                    delivery.updated_at.clone(),
                ]
            }),
    ));
    html.push_str(&ops_table(
        "Source Cards",
        &["provider", "type", "title", "url", "summary", "updated"],
        snapshot.source_cards.iter().take(100).map(|card| {
            vec![
                card.provider.clone(),
                card.source_type.clone(),
                card.title.clone(),
                card.url.clone(),
                card.summary.clone(),
                card.updated_at.clone(),
            ]
        }),
    ));
    html.push_str(&ops_table(
        "Projects",
        &["name", "status", "summary", "aliases", "updated"],
        snapshot.projects.iter().take(100).map(|project| {
            vec![
                project.name.clone(),
                project.status.clone(),
                project.summary.clone(),
                project.aliases.join(", "),
                project.updated_at.clone(),
            ]
        }),
    ));
    html.push_str(&ops_table(
        "Project Status Proposals",
        &[
            "project",
            "status",
            "source",
            "thread",
            "confidence",
            "summary",
            "created",
        ],
        snapshot
            .project_status_snapshots
            .iter()
            .take(50)
            .map(|status| {
                vec![
                    status.project_id.clone(),
                    status.status.clone(),
                    status.source.clone(),
                    status.thread_ref.clone().unwrap_or_default(),
                    format!("{:.2}", status.confidence),
                    status.summary.clone(),
                    status.created_at.clone(),
                ]
            }),
    ));
    html.push_str(&ops_table(
        "Channels",
        &[
            "channel",
            "direction",
            "project",
            "sender",
            "status",
            "body",
        ],
        snapshot.channel_messages.iter().take(50).map(|message| {
            vec![
                message.channel.clone(),
                message.direction.clone(),
                message.project_id.clone().unwrap_or_default(),
                message.sender.clone(),
                message.status.clone(),
                message.body.clone(),
            ]
        }),
    ));
    html.push_str(&ops_table(
        "Telegram Delivery Failures",
        &[
            "channel",
            "destination",
            "attempt",
            "status",
            "retry",
            "error",
            "response",
        ],
        snapshot
            .channel_delivery_attempts
            .iter()
            .filter(|attempt| !attempt.ok)
            .take(50)
            .map(|attempt| {
                vec![
                    attempt.channel.clone(),
                    attempt.destination.clone(),
                    attempt.attempt.to_string(),
                    attempt.provider_status.to_string(),
                    attempt.retry_at.clone().unwrap_or_default(),
                    attempt.error.clone().unwrap_or_default(),
                    json_cell(&attempt.response),
                ]
            }),
    ));
    html.push_str(&ops_table(
        "Import Ledger",
        &[
            "source",
            "mode",
            "status",
            "seen",
            "sampled",
            "written",
            "duplicates",
            "error",
            "started",
        ],
        snapshot.import_runs.iter().take(50).map(|run| {
            vec![
                format!("{} {}", run.source_kind, run.source_path),
                run.mode.clone(),
                run.status.clone(),
                run.candidates_seen.to_string(),
                run.candidates_sampled.to_string(),
                run.candidates_written.to_string(),
                run.duplicates_suppressed.to_string(),
                run.error.clone().unwrap_or_default(),
                run.started_at.clone(),
            ]
        }),
    ));
    html.push_str(&ops_table(
        "Memory Review",
        &[
            "operation",
            "status",
            "sensitivity",
            "user",
            "source",
            "content",
        ],
        snapshot.memory_candidates.iter().take(50).map(|candidate| {
            vec![
                candidate.operation.clone(),
                candidate.status.clone(),
                candidate.sensitivity.clone(),
                candidate.user_id.clone().unwrap_or_default(),
                candidate.source_ref.clone(),
                candidate.content.clone(),
            ]
        }),
    ));
    html.push_str(&ops_table(
        "Memory Decisions",
        &[
            "operation",
            "user",
            "source",
            "confidence",
            "reason",
            "created",
        ],
        snapshot.memory_decisions.iter().take(50).map(|decision| {
            vec![
                decision.operation.clone(),
                decision.user_id.clone().unwrap_or_default(),
                decision.source_ref.clone(),
                format!("{:.2}", decision.confidence),
                decision.reason.clone(),
                decision.created_at.clone(),
            ]
        }),
    ));
    html.push_str(&ops_table(
        "Procedures",
        &[
            "title", "status", "version", "trigger", "problem", "updated",
        ],
        snapshot.procedures.iter().take(50).map(|procedure| {
            vec![
                procedure.title.clone(),
                procedure.status.clone(),
                procedure.current_version.to_string(),
                procedure.trigger_context.clone(),
                procedure.problem.clone(),
                procedure.updated_at.clone(),
            ]
        }),
    ));
    html.push_str(&ops_table(
        "Procedure Candidates",
        &[
            "operation",
            "status",
            "title",
            "sensitivity",
            "reason",
            "created",
        ],
        snapshot
            .procedure_candidates
            .iter()
            .take(50)
            .map(|candidate| {
                vec![
                    candidate.operation.clone(),
                    candidate.status.clone(),
                    candidate.title.clone(),
                    candidate.sensitivity.clone(),
                    candidate.reason.clone(),
                    candidate.created_at.clone(),
                ]
            }),
    ));
    html.push_str(&ops_table(
        "Work Runs",
        &[
            "goal",
            "status",
            "project",
            "host",
            "thread",
            "outcome",
            "validation",
        ],
        snapshot.work_runs.iter().take(50).map(|run| {
            vec![
                run.goal.clone(),
                run.status.clone(),
                run.project_id.clone().unwrap_or_default(),
                run.host_id.clone().unwrap_or_default(),
                run.thread_id.clone().unwrap_or_default(),
                run.outcome.clone().unwrap_or_default(),
                run.validation_summary.clone().unwrap_or_default(),
            ]
        }),
    ));
    html.push_str(&ops_table(
        "Costs",
        &["scope", "key", "limit", "kill switch", "updated"],
        snapshot.cost_policies.iter().map(|policy| {
            vec![
                policy.scope.clone(),
                policy.key.clone(),
                policy
                    .limit_usd
                    .map(|value| format!("{value:.4}"))
                    .unwrap_or_else(|| "none".to_string()),
                policy.kill_switch.to_string(),
                policy.updated_at.clone(),
            ]
        }),
    ));
    html.push_str(&ops_table(
        "Cost Decisions",
        &[
            "allowed",
            "package",
            "provider",
            "source",
            "projected",
            "reason",
        ],
        snapshot.cost_decisions.iter().take(50).map(|decision| {
            vec![
                decision.allowed.to_string(),
                decision.package.clone(),
                decision.provider.clone(),
                decision.source.clone().unwrap_or_default(),
                format!("{:.4}", decision.projected_usd),
                decision.reason.clone(),
            ]
        }),
    ));
    html.push_str(&ops_table(
        "Policy Decisions",
        &[
            "effect", "allowed", "action", "rule", "reason", "target", "created",
        ],
        snapshot.policy_decisions.iter().take(50).map(|decision| {
            vec![
                decision.effect.clone(),
                decision.allowed.to_string(),
                decision.action.clone(),
                decision.matched_rule_id.clone().unwrap_or_default(),
                decision.reason.clone(),
                decision.target.clone().unwrap_or_default(),
                decision.created_at.clone(),
            ]
        }),
    ));
    html.push_str(&ops_table(
        "Policy Approvals",
        &["status", "action", "reason", "decision", "created"],
        snapshot.policy_approvals.iter().take(50).map(|approval| {
            vec![
                approval.status.clone(),
                approval.action.clone(),
                approval.reason.clone(),
                approval.decision_id.clone(),
                approval.created_at.clone(),
            ]
        }),
    ));
    html.push_str(&ops_table(
        "Provider And Secret Health",
        &[
            "name", "scope", "provider", "source", "present", "status", "warnings",
        ],
        filtered_secret_health(snapshot, options)
            .into_iter()
            .take(100)
            .map(|secret| {
                vec![
                    secret.name.clone(),
                    secret.scope.clone(),
                    secret.provider.clone().unwrap_or_default(),
                    secret.source.clone(),
                    secret.present.to_string(),
                    secret.status.clone(),
                    secret.warnings.join("\n"),
                ]
            }),
    ));
    html.push_str(&ops_table(
        "Secret References",
        &["name", "scope", "location", "expires", "updated"],
        snapshot.secrets.iter().take(100).map(|secret| {
            vec![
                secret.name.clone(),
                secret.scope.clone(),
                secret.location.clone(),
                secret.expires_at.clone().unwrap_or_default(),
                secret.updated_at.clone(),
            ]
        }),
    ));
    html.push_str("</main></body></html>");
    html
}

pub(crate) fn json_cell(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}

pub(crate) fn ops_table<I>(title: &str, headers: &[&str], rows: I) -> String
where
    I: IntoIterator<Item = Vec<String>>,
{
    ops_table_with_raw_columns(title, headers, rows, &[])
}

pub(crate) fn ops_table_with_raw_columns<I>(
    title: &str,
    headers: &[&str],
    rows: I,
    raw_columns: &[usize],
) -> String
where
    I: IntoIterator<Item = Vec<String>>,
{
    let mut html = format!(
        "<section class=\"section\"><h2>{}</h2><div class=\"scroll\"><table class=\"ops-table\"><thead><tr>",
        html_escape(title)
    );
    for header in headers {
        html.push_str(&format!("<th>{}</th>", html_escape(header)));
    }
    html.push_str("</tr></thead><tbody>");
    let mut any = false;
    for row in rows {
        any = true;
        html.push_str("<tr>");
        for (index, cell) in row.into_iter().enumerate() {
            let label = headers.get(index).copied().unwrap_or_default();
            if raw_columns.contains(&index) {
                html.push_str(&format!(
                    "<td data-label=\"{}\">{cell}</td>",
                    html_escape(label)
                ));
            } else {
                html.push_str(&format!(
                    "<td data-label=\"{}\">{}</td>",
                    html_escape(label),
                    html_escape(&cell)
                ));
            }
        }
        html.push_str("</tr>");
    }
    if !any {
        html.push_str(&format!(
            "<tr><td colspan=\"{}\">No rows.</td></tr>",
            headers.len()
        ));
    }
    html.push_str("</tbody></table></div></section>");
    html
}

#[derive(Debug)]
pub(crate) struct OpsHealthScore {
    pub(crate) score: i64,
    pub(crate) label: &'static str,
    pub(crate) issues: Vec<String>,
}

pub(crate) fn ops_health_score(snapshot: &OpsSnapshot) -> OpsHealthScore {
    let failed_jobs = snapshot
        .jobs
        .iter()
        .filter(|job| matches!(job.status.as_str(), "failed" | "dead_lettered"))
        .count() as i64;
    let dead_edge = snapshot
        .edge_events
        .iter()
        .filter(|event| event.status == "dead_lettered")
        .count() as i64;
    let failed_sources = snapshot
        .source_health
        .iter()
        .filter(|source| source.status != "healthy")
        .count() as i64;
    let failing_radar_source_quality = snapshot
        .radar_source_quality
        .iter()
        .filter(|quality| matches!(quality.status.as_str(), "failed" | "partial"))
        .count() as i64;
    let low_signal_radar_source_quality = snapshot
        .radar_source_quality
        .iter()
        .filter(|quality| quality.status == "low_signal")
        .count() as i64;
    let failed_radar_deliveries = snapshot
        .radar_deliveries
        .iter()
        .filter(|delivery| matches!(delivery.status.as_str(), "failed" | "blocked"))
        .count() as i64;
    let job_source_failures = snapshot.job_hunting.source_health_failures.len() as i64;
    let job_privacy_blocks = snapshot
        .job_hunting
        .privacy_decision_counts
        .get("block")
        .copied()
        .unwrap_or(0) as i64;
    let bad_secrets = snapshot
        .secret_health
        .iter()
        .filter(|secret| !secret.present || secret.status != "ok")
        .count() as i64;
    let failed_deliveries = snapshot
        .channel_delivery_attempts
        .iter()
        .filter(|attempt| !attempt.ok)
        .count() as i64;
    let x_drift = snapshot.x_stats.drift.compatibility_without_canonical
        + snapshot.x_stats.drift.canonical_without_compatibility
        + snapshot.x_stats.drift.tweets_without_fts
        + snapshot.x_stats.drift.fts_without_tweets
        + snapshot.x_stats.drift.projection_failures
        + snapshot.x_stats.drift.non_healthy_sources;
    let x_failed_sync_runs = snapshot.x_stats.unresolved_failed_sync_runs;
    let mut issues = Vec::new();
    if !snapshot.health.ok {
        issues.push("base health report is failing".to_string());
    }
    if failed_jobs > 0 {
        issues.push(format!("{failed_jobs} failed or dead-lettered wiki jobs"));
    }
    if dead_edge > 0 {
        issues.push(format!("{dead_edge} dead-lettered edge events"));
    }
    if failed_sources > 0 {
        issues.push(format!("{failed_sources} non-healthy sources"));
    }
    if failing_radar_source_quality > 0 {
        issues.push(format!(
            "{failing_radar_source_quality} failed or partial radar source-quality window(s)"
        ));
    }
    if low_signal_radar_source_quality > 0 {
        issues.push(format!(
            "{low_signal_radar_source_quality} low-signal radar source-quality window(s)"
        ));
    }
    if bad_secrets > 0 {
        issues.push(format!("{bad_secrets} missing or unhealthy credentials"));
    }
    if failed_deliveries > 0 {
        issues.push(format!("{failed_deliveries} failed channel deliveries"));
    }
    if failed_radar_deliveries > 0 {
        issues.push(format!(
            "{failed_radar_deliveries} failed or blocked radar delivery attempt(s)"
        ));
    }
    if job_source_failures > 0 {
        issues.push(format!(
            "{job_source_failures} non-healthy job source check(s)"
        ));
    }
    if job_privacy_blocks > 0 {
        issues.push(format!("{job_privacy_blocks} blocked job privacy check(s)"));
    }
    if x_drift > 0 {
        issues.push(format!("{x_drift} X drift/source-health issue(s)"));
    }
    if x_failed_sync_runs > 0 {
        issues.push(format!(
            "{x_failed_sync_runs} unresolved failed X sync run(s)"
        ));
    }
    for warning in &snapshot.health.warnings {
        issues.push(warning.clone());
    }
    let penalty = (snapshot.health.warnings.len() as i64 * 8)
        + (failed_jobs * 8)
        + (dead_edge * 8)
        + (failed_sources * 5)
        + (failing_radar_source_quality * 4)
        + (low_signal_radar_source_quality * 2)
        + (failed_radar_deliveries * 4)
        + (job_source_failures * 4)
        + (job_privacy_blocks * 5)
        + (bad_secrets * 6)
        + (failed_deliveries * 4)
        + (x_drift * 6)
        + (x_failed_sync_runs * 5)
        + if snapshot.health.ok { 0 } else { 12 };
    let score = (100 - penalty).clamp(0, 100);
    let label = if score >= 90 {
        "good"
    } else if score >= 70 {
        "watch"
    } else {
        "needs attention"
    };
    OpsHealthScore {
        score,
        label,
        issues,
    }
}

pub(crate) fn render_ops_filter_form(options: &OpsUiOptions) -> String {
    let q = options.q.clone().unwrap_or_default();
    let status = options.status.clone().unwrap_or_default();
    let sort = if options.sort.is_empty() {
        "updated_desc"
    } else {
        options.sort.as_str()
    };
    let sort_options = [
        ("updated_desc", "Updated newest"),
        ("updated_asc", "Updated oldest"),
        ("status", "Status"),
        ("kind", "Kind/source"),
        ("attempts_desc", "Attempts"),
    ];
    let mut html = format!(
        "<form class=\"ops-form\" method=\"get\" action=\"/ops/ui\"><label>Search<input name=\"q\" value=\"{}\" placeholder=\"queue, source, credential, error\"></label><label>Status<input name=\"status\" value=\"{}\" placeholder=\"failed, pending, ok\"></label><label>Sort<select name=\"sort\">",
        html_escape(&q),
        html_escape(&status)
    );
    for (value, label) in sort_options {
        let selected = if value == sort { " selected" } else { "" };
        html.push_str(&format!(
            "<option value=\"{}\"{}>{}</option>",
            html_escape(value),
            selected,
            html_escape(label)
        ));
    }
    html.push_str("</select></label><button type=\"submit\">Apply</button></form>");
    html
}

pub(crate) fn render_x_ops_control_panel(
    csrf_token: Option<&str>,
    controls_enabled: bool,
) -> String {
    let mut html = String::new();
    html.push_str("<section class=\"section\"><h2>X Controls</h2>");
    let Some(csrf_token) = csrf_token else {
        html.push_str("<p class=\"muted\">Open /ops/ui from the authenticated HTTP server to use X controls.</p></section>");
        return html;
    };
    if !controls_enabled {
        html.push_str("<p class=\"muted\">Disabled: start server with ARCWELL_HTTP_AUTH_TOKEN to enable mutations.</p></section>");
        return html;
    }
    html.push_str("<div class=\"control-grid\">");
    html.push_str(&format!(
        r#"<form method="post" action="/ops/actions/x/bookmarks/schedule">
<input type="hidden" name="csrf_token" value="{}">
<input type="hidden" name="idempotency_key" value="{}">
<div><b>Schedule bookmark ingestion</b><p class="muted">Create or update the resident X bookmark watch source.</p></div>
<div class="fields">
<label>Days<input name="bookmark_days" type="number" min="1" max="36500" value="92"></label>
<label>Max<input name="max_bookmarks" type="number" min="1" max="100000" value="1000"></label>
<label>Cadence<input name="cadence" maxlength="40" value="warm"></label>
<label>Status<select name="status"><option value="active">active</option><option value="paused">paused</option></select></label>
</div>
<button type="submit">Schedule</button>
</form>"#,
        html_escape(csrf_token),
        html_escape(&ops_control_idempotency_key("x-bookmarks-schedule")),
    ));
    html.push_str(&format!(
        r#"<form method="post" action="/ops/actions/x/bookmarks/enqueue">
<input type="hidden" name="csrf_token" value="{}">
<input type="hidden" name="idempotency_key" value="{}">
<div><b>Queue bookmark import</b><p class="muted">Enqueue one bookmark import job without claiming provider health.</p></div>
<div class="fields">
<label>Days<input name="bookmark_days" type="number" min="1" max="36500" value="92"></label>
<label>Max<input name="max_bookmarks" type="number" min="1" max="100000" value="1000"></label>
</div>
<button type="submit">Queue import</button>
</form>"#,
        html_escape(csrf_token),
        html_escape(&ops_control_idempotency_key("x-bookmarks-enqueue")),
    ));
    html.push_str(&format!(
        r#"<form method="post" action="/ops/actions/worker/run-once">
<input type="hidden" name="csrf_token" value="{}">
<input type="hidden" name="idempotency_key" value="{}">
<div><b>Run worker once</b><p class="muted">Poll due schedules and drain a bounded number of local jobs.</p></div>
<div class="fields">
<label>Max jobs<input name="max_jobs" type="number" min="1" max="25" value="5"></label>
</div>
<button type="submit">Run once</button>
</form>"#,
        html_escape(csrf_token),
        html_escape(&ops_control_idempotency_key("worker-run-once")),
    ));
    html.push_str("</div></section>");
    html
}

pub(crate) fn render_knowledge_ops_control_panel(
    csrf_token: Option<&str>,
    controls_enabled: bool,
) -> String {
    let mut html = String::new();
    html.push_str("<section class=\"section\"><h2>Knowledge Controls</h2>");
    let Some(csrf_token) = csrf_token else {
        html.push_str("<p class=\"muted\">Open /ops/ui from the authenticated HTTP server to use knowledge controls.</p></section>");
        return html;
    };
    if !controls_enabled {
        html.push_str("<p class=\"muted\">Disabled: start server with ARCWELL_HTTP_AUTH_TOKEN to enable mutations.</p></section>");
        return html;
    }
    html.push_str("<div class=\"control-grid\">");
    html.push_str(&format!(
        r#"<form method="post" action="/ops/actions/knowledge/backlog/schedule">
<input type="hidden" name="csrf_token" value="{}">
<input type="hidden" name="idempotency_key" value="{}">
<div><b>Schedule backlog clustering</b><p class="muted">Create or update the local source-card backlog watch source.</p></div>
<div class="fields">
<label>Max cards<input name="max_source_cards" type="number" min="1" max="500" value="100"></label>
<label>Min group<input name="min_group_size" type="number" min="1" max="20" value="2"></label>
<label>Max clusters<input name="max_clusters" type="number" min="1" max="50" value="12"></label>
<label>Status<select name="status"><option value="active">active</option><option value="paused">paused</option></select></label>
<label>Cadence<input name="cadence" maxlength="40" value="warm"></label>
</div>
<button type="submit">Schedule</button>
</form>"#,
        html_escape(csrf_token),
        html_escape(&ops_control_idempotency_key("knowledge-backlog-schedule")),
    ));
    html.push_str(&format!(
        r#"<form method="post" action="/ops/actions/knowledge/backlog/enqueue">
<input type="hidden" name="csrf_token" value="{}">
<input type="hidden" name="idempotency_key" value="{}">
<div><b>Queue backlog clustering</b><p class="muted">Enqueue one source-card backlog clustering job without claiming source health.</p></div>
<div class="fields">
<label>Max cards<input name="max_source_cards" type="number" min="1" max="500" value="100"></label>
<label>Min group<input name="min_group_size" type="number" min="1" max="20" value="2"></label>
<label>Max clusters<input name="max_clusters" type="number" min="1" max="50" value="12"></label>
</div>
<button type="submit">Queue clustering</button>
</form>"#,
        html_escape(csrf_token),
        html_escape(&ops_control_idempotency_key("knowledge-backlog-enqueue")),
    ));
    html.push_str(&format!(
        r#"<form method="post" action="/ops/actions/knowledge/model-clusters/schedule">
<input type="hidden" name="csrf_token" value="{}">
<input type="hidden" name="idempotency_key" value="{}">
<div><b>Schedule model clustering</b><p class="muted">Create or update a review-only model-cluster proposal watch source; source-cards runs a broad unclustered corpus sweep.</p></div>
<div class="fields">
<label>Query<input name="query" maxlength="200" value="source-cards"></label>
<label>Provider<select name="model_provider"><option value="mock">mock</option><option value="openai">openai</option></select></label>
<label>Model<input name="model_name" maxlength="80" placeholder="gpt-4.1-mini"></label>
<label>Endpoint<input name="endpoint" maxlength="300" placeholder="optional"></label>
<label>Timeout<input name="timeout_seconds" type="number" min="1" max="600" placeholder="optional"></label>
<label>Max cards<input name="max_source_cards" type="number" min="1" max="80" value="24"></label>
<label>Max clusters<input name="max_clusters" type="number" min="1" max="12" value="6"></label>
<label>Status<select name="status"><option value="active">active</option><option value="paused">paused</option></select></label>
<label>Cadence<input name="cadence" maxlength="40" value="warm"></label>
</div>
<button type="submit">Schedule models</button>
</form>"#,
        html_escape(csrf_token),
        html_escape(&ops_control_idempotency_key(
            "knowledge-model-clusters-schedule"
        )),
    ));
    html.push_str(&format!(
        r#"<form method="post" action="/ops/actions/knowledge/model-clusters/enqueue">
<input type="hidden" name="csrf_token" value="{}">
<input type="hidden" name="idempotency_key" value="{}">
<div><b>Queue model clustering</b><p class="muted">Enqueue one review-only model-cluster proposal job; source-cards uses fresh unclustered evidence.</p></div>
<div class="fields">
<label>Query<input name="query" maxlength="200" value="source-cards"></label>
<label>Provider<select name="model_provider"><option value="mock">mock</option><option value="openai">openai</option></select></label>
<label>Model<input name="model_name" maxlength="80" placeholder="gpt-4.1-mini"></label>
<label>Endpoint<input name="endpoint" maxlength="300" placeholder="optional"></label>
<label>Timeout<input name="timeout_seconds" type="number" min="1" max="600" placeholder="optional"></label>
<label>Max cards<input name="max_source_cards" type="number" min="1" max="80" value="24"></label>
<label>Max clusters<input name="max_clusters" type="number" min="1" max="12" value="6"></label>
</div>
<button type="submit">Queue models</button>
</form>"#,
        html_escape(csrf_token),
        html_escape(&ops_control_idempotency_key(
            "knowledge-model-clusters-enqueue"
        )),
    ));
    html.push_str(&format!(
        r#"<form method="post" action="/ops/actions/knowledge/clusters/enqueue-editorial-decisions">
<input type="hidden" name="csrf_token" value="{}">
<input type="hidden" name="idempotency_key" value="{}">
<div><b>Queue cluster editorial review</b><p class="muted">Find eligible shared clusters and enqueue editorial decisions before wiki/report/digest expansion.</p></div>
<div class="fields">
<label>Max clusters<input name="max_clusters" type="number" min="1" max="100" value="25"></label>
</div>
<button type="submit">Queue review</button>
</form>"#,
        html_escape(csrf_token),
        html_escape(&ops_control_idempotency_key(
            "knowledge-cluster-editorial-decisions"
        )),
    ));
    html.push_str(&format!(
        r#"<form method="post" action="/ops/actions/knowledge/clusters/promote">
<input type="hidden" name="csrf_token" value="{}">
<input type="hidden" name="idempotency_key" value="{}">
<div><b>Promote model cluster</b><p class="muted">Mark one reviewed model-origin candidate cluster active before expansion can write wiki/report/digest artifacts.</p></div>
<div class="fields">
<label>Cluster id<input name="cluster_id" maxlength="120" placeholder="kcl-..."></label>
<label>Reviewer<input name="reviewer" maxlength="200" value="ops-ui"></label>
<label>Reason<input name="reason" maxlength="2000" value="Reviewed source-card evidence and approved promotion."></label>
</div>
<button type="submit">Promote cluster</button>
</form>"#,
        html_escape(csrf_token),
        html_escape(&ops_control_idempotency_key("knowledge-cluster-promote")),
    ));
    html.push_str(&format!(
        r#"<form method="post" action="/ops/actions/knowledge/model-writes/schedule">
<input type="hidden" name="csrf_token" value="{}">
<input type="hidden" name="idempotency_key" value="{}">
<div><b>Schedule model writer</b><p class="muted">Create or update a cluster-scoped model writer watch source for a promoted cluster.</p></div>
<div class="fields">
<label>Cluster id<input name="cluster_id" maxlength="120" placeholder="kcl-..."></label>
<label>Provider<select name="model_provider"><option value="mock">mock</option><option value="openai">openai</option></select></label>
<label>Model<input name="model_name" maxlength="80" placeholder="gpt-4.1-mini"></label>
<label>Endpoint<input name="endpoint" maxlength="300" placeholder="optional"></label>
<label>Timeout<input name="timeout_seconds" type="number" min="1" max="600" placeholder="optional"></label>
<label>Digest<select name="create_digest"><option value="true">create</option><option value="false">skip</option></select></label>
<label>Status<select name="status"><option value="active">active</option><option value="paused">paused</option></select></label>
<label>Cadence<input name="cadence" maxlength="40" value="warm"></label>
</div>
<button type="submit">Schedule writer</button>
</form>"#,
        html_escape(csrf_token),
        html_escape(&ops_control_idempotency_key(
            "knowledge-model-write-schedule"
        )),
    ));
    html.push_str(&format!(
        r#"<form method="post" action="/ops/actions/knowledge/model-writes/enqueue">
<input type="hidden" name="csrf_token" value="{}">
<input type="hidden" name="idempotency_key" value="{}">
<div><b>Queue model writer</b><p class="muted">Enqueue one cluster-scoped model writer job for a promoted cluster.</p></div>
<div class="fields">
<label>Cluster id<input name="cluster_id" maxlength="120" placeholder="kcl-..."></label>
<label>Provider<select name="model_provider"><option value="mock">mock</option><option value="openai">openai</option></select></label>
<label>Model<input name="model_name" maxlength="80" placeholder="gpt-4.1-mini"></label>
<label>Endpoint<input name="endpoint" maxlength="300" placeholder="optional"></label>
<label>Timeout<input name="timeout_seconds" type="number" min="1" max="600" placeholder="optional"></label>
<label>Digest<select name="create_digest"><option value="true">create</option><option value="false">skip</option></select></label>
</div>
<button type="submit">Queue writer</button>
</form>"#,
        html_escape(csrf_token),
        html_escape(&ops_control_idempotency_key(
            "knowledge-model-write-enqueue"
        )),
    ));
    html.push_str(&format!(
        r#"<form method="post" action="/ops/actions/knowledge/model-writes/enqueue-due">
<input type="hidden" name="csrf_token" value="{}">
<input type="hidden" name="idempotency_key" value="{}">
<div><b>Queue due model writers</b><p class="muted">Find promoted model-origin clusters without terminal writer output and enqueue source-card-gated model writer jobs.</p></div>
<div class="fields">
<label>Max clusters<input name="max_clusters" type="number" min="1" max="100" value="25"></label>
<label>Provider<select name="model_provider"><option value="mock">mock</option><option value="openai">openai</option></select></label>
<label>Model<input name="model_name" maxlength="80" placeholder="gpt-4.1-mini"></label>
<label>Endpoint<input name="endpoint" maxlength="300" placeholder="optional"></label>
<label>Timeout<input name="timeout_seconds" type="number" min="1" max="600" placeholder="optional"></label>
<label>Digest<select name="create_digest"><option value="true">create</option><option value="false">skip</option></select></label>
</div>
<button type="submit">Queue due writers</button>
</form>"#,
        html_escape(csrf_token),
        html_escape(&ops_control_idempotency_key(
            "knowledge-model-writes-due"
        )),
    ));
    html.push_str(&format!(
        r#"<form method="post" action="/ops/actions/knowledge/entity-resolution/schedule">
<input type="hidden" name="csrf_token" value="{}">
<input type="hidden" name="idempotency_key" value="{}">
<div><b>Schedule entity resolution</b><p class="muted">Create or update the review-only entity-resolution watch source for source-card-backed entity pairs.</p></div>
<div class="fields">
<label>Max pairs<input name="max_pairs" type="number" min="1" max="100" value="25"></label>
<label>Provider<select name="model_provider"><option value="mock">mock</option><option value="openai">openai</option></select></label>
<label>Model<input name="model_name" maxlength="80" placeholder="gpt-4.1-mini"></label>
<label>Endpoint<input name="endpoint" maxlength="300" placeholder="optional"></label>
<label>Timeout<input name="timeout_seconds" type="number" min="1" max="600" placeholder="optional"></label>
<label>Status<select name="status"><option value="active">active</option><option value="paused">paused</option></select></label>
<label>Cadence<input name="cadence" maxlength="40" value="warm"></label>
</div>
<button type="submit">Schedule resolution</button>
</form>"#,
        html_escape(csrf_token),
        html_escape(&ops_control_idempotency_key(
            "knowledge-entity-resolution-schedule"
        )),
    ));
    html.push_str(&format!(
        r#"<form method="post" action="/ops/actions/knowledge/entity-resolution/enqueue-due">
<input type="hidden" name="csrf_token" value="{}">
<input type="hidden" name="idempotency_key" value="{}">
<div><b>Queue due entity resolution</b><p class="muted">Find eligible entity pairs and enqueue review-only model resolution jobs.</p></div>
<div class="fields">
<label>Max pairs<input name="max_pairs" type="number" min="1" max="100" value="25"></label>
<label>Provider<select name="model_provider"><option value="mock">mock</option><option value="openai">openai</option></select></label>
<label>Model<input name="model_name" maxlength="80" placeholder="gpt-4.1-mini"></label>
<label>Endpoint<input name="endpoint" maxlength="300" placeholder="optional"></label>
<label>Timeout<input name="timeout_seconds" type="number" min="1" max="600" placeholder="optional"></label>
</div>
<button type="submit">Queue resolution</button>
</form>"#,
        html_escape(csrf_token),
        html_escape(&ops_control_idempotency_key(
            "knowledge-entity-resolution-enqueue-due"
        )),
    ));
    html.push_str(&format!(
        r#"<form method="post" action="/ops/actions/knowledge/investigations/enqueue-execution">
<input type="hidden" name="csrf_token" value="{}">
<input type="hidden" name="idempotency_key" value="{}">
<div><b>Queue investigation execution</b><p class="muted">Find source-linked investigation tasks and enqueue deterministic execution jobs.</p></div>
<div class="fields">
<label>Max clusters<input name="max_clusters" type="number" min="1" max="100" value="25"></label>
</div>
<button type="submit">Queue investigations</button>
</form>"#,
        html_escape(csrf_token),
        html_escape(&ops_control_idempotency_key("knowledge-investigation-execution")),
    ));
    html.push_str("</div></section>");
    html
}

pub(crate) fn ops_control_idempotency_key(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::new_v4())
}
