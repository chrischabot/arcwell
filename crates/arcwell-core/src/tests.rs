use super::*;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::{Arc, Mutex, OnceLock};

static LOOPBACK_URL_INGEST_ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static REDDIT_BEARER_TOKEN_ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static X_API_BASE_ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn with_loopback_url_ingest_allowed<T>(f: impl FnOnce() -> T) -> T {
    let _guard = LOOPBACK_URL_INGEST_ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("loopback URL ingest env lock poisoned");
    unsafe {
        std::env::set_var("ARCWELL_ALLOW_LOOPBACK_URL_INGEST", "1");
    }
    let result = catch_unwind(AssertUnwindSafe(f));
    unsafe {
        std::env::remove_var("ARCWELL_ALLOW_LOOPBACK_URL_INGEST");
    }
    match result {
        Ok(value) => value,
        Err(payload) => resume_unwind(payload),
    }
}
fn with_reddit_bearer_token<T>(token: &str, f: impl FnOnce() -> T) -> T {
    let _guard = REDDIT_BEARER_TOKEN_ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("Reddit bearer token env lock poisoned");
    unsafe {
        std::env::set_var("REDDIT_BEARER_TOKEN", token);
    }
    let result = catch_unwind(AssertUnwindSafe(f));
    unsafe {
        std::env::remove_var("REDDIT_BEARER_TOKEN");
    }
    match result {
        Ok(value) => value,
        Err(payload) => resume_unwind(payload),
    }
}
fn without_reddit_bearer_token<T>(f: impl FnOnce() -> T) -> T {
    let _guard = REDDIT_BEARER_TOKEN_ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("Reddit bearer token env lock poisoned");
    unsafe {
        std::env::remove_var("REDDIT_BEARER_TOKEN");
    }
    let result = catch_unwind(AssertUnwindSafe(f));
    unsafe {
        std::env::remove_var("REDDIT_BEARER_TOKEN");
    }
    match result {
        Ok(value) => value,
        Err(payload) => resume_unwind(payload),
    }
}

fn with_x_api_base<T>(base: &str, f: impl FnOnce() -> T) -> T {
    let _guard = X_API_BASE_ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("X API base env lock poisoned");
    let previous = std::env::var_os("ARCWELL_X_API_BASE");
    unsafe {
        std::env::set_var("ARCWELL_X_API_BASE", base);
    }
    let result = catch_unwind(AssertUnwindSafe(f));
    unsafe {
        if let Some(previous) = previous {
            std::env::set_var("ARCWELL_X_API_BASE", previous);
        } else {
            std::env::remove_var("ARCWELL_X_API_BASE");
        }
    }
    match result {
        Ok(value) => value,
        Err(payload) => resume_unwind(payload),
    }
}
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

fn test_store(name: &str) -> Store {
    let root = std::env::temp_dir().join(format!("arcwell-test-{name}-{}", Uuid::new_v4()));
    Store::open(AppPaths::new(root)).unwrap()
}

fn test_paths(name: &str) -> AppPaths {
    let root = std::env::temp_dir().join(format!("arcwell-test-{name}-{}", Uuid::new_v4()));
    AppPaths::new(root)
}

fn job_fixture_profile(store: &Store) -> JobCandidateProfile {
    store
        .record_job_candidate_profile(JobCandidateProfileInput {
            label: "Chris Chabot".to_string(),
            current_resume_source: Some("resume:current".to_string()),
            linkedin_source: Some("https://www.linkedin.com/in/chrischabot/".to_string()),
            github_profile: Some("https://github.com/chrischabot".to_string()),
            blog_url: Some("https://chabot.dev".to_string()),
            metadata: json!({}),
        })
        .unwrap()
}

fn job_fixture_evidence(store: &Store, profile_id: &str) -> JobEvidenceCard {
    store
        .record_job_evidence_card(JobEvidenceCardInput {
            profile_id: profile_id.to_string(),
            title: "Open Cloud".to_string(),
            evidence_type: "github".to_string(),
            visibility: "public".to_string(),
            summary: "Public project showing cloud and developer-tooling work.".to_string(),
            proof_url: Some("https://github.com/chrischabot/opencloud".to_string()),
            local_path: None,
            source_date: Some("2026-06-28".to_string()),
            confidence: "verified".to_string(),
            tags: vec!["developer-tools".to_string(), "cloud".to_string()],
            safe_application_text: "Built public developer tooling around cloud workflows."
                .to_string(),
            unsafe_terms: vec![],
            metadata: json!({}),
        })
        .unwrap()
}

fn job_fixture_reviewed_evidence_with_claim(
    store: &Store,
    profile_id: &str,
    index: usize,
) -> JobEvidenceCard {
    let card = store
        .record_job_evidence_card(JobEvidenceCardInput {
            profile_id: profile_id.to_string(),
            title: format!("Public evidence {index}"),
            evidence_type: if index % 2 == 0 {
                "github".to_string()
            } else {
                "blog".to_string()
            },
            visibility: "public".to_string(),
            summary: format!("Reviewed public evidence card {index}."),
            proof_url: Some(format!("https://example.com/evidence/{index}")),
            local_path: None,
            source_date: Some("2026-06-28".to_string()),
            confidence: "verified".to_string(),
            tags: vec!["agents".to_string(), "developer-tools".to_string()],
            safe_application_text: format!(
                "Built and explained public developer-tooling system {index}."
            ),
            unsafe_terms: vec![],
            metadata: json!({}),
        })
        .unwrap();
    store
        .record_job_evidence_claim(JobEvidenceClaimInput {
            evidence_card_id: card.id.clone(),
            claim: format!("Can support senior developer-tooling claim {index}."),
            claim_kind: "skill".to_string(),
            proof_level: "public".to_string(),
            can_use_in_resume: true,
            can_use_in_outreach: true,
            can_use_in_interview: true,
        })
        .unwrap();
    card
}

fn job_fixture_role(store: &Store, evidence_id: &str, source_confidence: &str) -> JobRoleCard {
    let canonical_url = (source_confidence == "canonical_confirmed")
        .then(|| "https://example.com/careers/staff-agent-platform-engineer".to_string());
    store
        .record_job_role_card(JobRoleCardInput {
            company: "Example AI".to_string(),
            role_title: "Staff Agent Platform Engineer".to_string(),
            canonical_url,
            source_family: if source_confidence == "aggregator_only" {
                "job_board".to_string()
            } else {
                "company".to_string()
            },
            source_url: if source_confidence == "aggregator_only" {
                "https://jobs.example.net/example-ai-agent-platform".to_string()
            } else {
                "https://example.com/careers/staff-agent-platform-engineer".to_string()
            },
            source_confidence: source_confidence.to_string(),
            date_accessed: Some("2026-06-28T10:00:00Z".to_string()),
            posting_freshness: "same_day".to_string(),
            location: Some("London or remote Europe".to_string()),
            work_mode: Some("hybrid_or_remote".to_string()),
            company_stage_or_size: Some("startup".to_string()),
            role_seniority: Some("staff".to_string()),
            core_requirements: vec!["agent systems".to_string(), "developer tooling".to_string()],
            implied_business_problem: Some(
                "The company needs reliable agent infrastructure for developers.".to_string(),
            ),
            why_they_might_need_user: Some(
                "Public work maps to agent and developer-tooling problems.".to_string(),
            ),
            evidence_card_ids: vec![evidence_id.to_string()],
            gaps_or_blockers: vec![],
            cluster: Some("agent-platform".to_string()),
            current_status: "live".to_string(),
            metadata: json!({}),
        })
        .unwrap()
}

fn job_fixture_score_input(role_id: &str, profile_id: &str, evidence_id: &str) -> JobFitScoreInput {
    JobFitScoreInput {
        role_id: role_id.to_string(),
        profile_id: profile_id.to_string(),
        scorer: "human".to_string(),
        role_fit: 5.0,
        domain_fit: 5.0,
        evidence_fit: 5.0,
        geo_work_fit: 5.0,
        stage_fit: 4.5,
        practical_odds: 4.5,
        interest_energy: 5.0,
        blockers: vec![],
        evidence_card_ids: vec![evidence_id.to_string()],
        explanation: "Strong match across agent systems and developer tooling.".to_string(),
    }
}

fn seed_job_refresh_audit_fixture(store: &Store) -> (JobCandidateProfile, String) {
    let profile = job_fixture_profile(store);
    let evidence = job_fixture_evidence(store, &profile.id);
    let scope = "London agent platform roles".to_string();
    let source = store
        .record_job_source(JobSourceInput {
            source_family: "company".to_string(),
            name: "Example AI careers".to_string(),
            url: "https://example.com/careers".to_string(),
            market_scope: "london".to_string(),
            refresh_policy: "manual".to_string(),
            metadata: json!({}),
        })
        .unwrap();
    let observed = job_fixture_role(store, &evidence.id, "canonical_confirmed");
    let stale = store
        .record_job_role_card(JobRoleCardInput {
            company: "Stale AI".to_string(),
            role_title: "Staff Platform Engineer".to_string(),
            canonical_url: Some("https://stale.example/jobs/staff-platform".to_string()),
            source_family: "company".to_string(),
            source_url: "https://stale.example/jobs/staff-platform".to_string(),
            source_confidence: "canonical_confirmed".to_string(),
            date_accessed: Some("2026-06-28T10:00:00Z".to_string()),
            posting_freshness: "same_day".to_string(),
            location: Some("London".to_string()),
            work_mode: Some("hybrid".to_string()),
            company_stage_or_size: Some("startup".to_string()),
            role_seniority: Some("staff".to_string()),
            core_requirements: vec!["platform engineering".to_string()],
            implied_business_problem: Some("Build developer-facing systems.".to_string()),
            why_they_might_need_user: Some("Agent tooling evidence maps to the role.".to_string()),
            evidence_card_ids: vec![evidence.id.clone()],
            gaps_or_blockers: vec![],
            cluster: Some("agent-platform".to_string()),
            current_status: "live".to_string(),
            metadata: json!({}),
        })
        .unwrap();
    let closed = store
        .record_job_role_card(JobRoleCardInput {
            company: "Closed AI".to_string(),
            role_title: "Developer Tools Lead".to_string(),
            canonical_url: Some("https://closed.example/jobs/devtools-lead".to_string()),
            source_family: "company".to_string(),
            source_url: "https://closed.example/jobs/devtools-lead".to_string(),
            source_confidence: "canonical_confirmed".to_string(),
            date_accessed: Some("2026-06-28T10:00:00Z".to_string()),
            posting_freshness: "same_day".to_string(),
            location: Some("London".to_string()),
            work_mode: Some("remote".to_string()),
            company_stage_or_size: Some("startup".to_string()),
            role_seniority: Some("lead".to_string()),
            core_requirements: vec!["developer tools".to_string()],
            implied_business_problem: Some("Improve developer workflows.".to_string()),
            why_they_might_need_user: Some(
                "Public cloud tooling evidence maps to the role.".to_string(),
            ),
            evidence_card_ids: vec![evidence.id.clone()],
            gaps_or_blockers: vec![],
            cluster: Some("developer-tools".to_string()),
            current_status: "live".to_string(),
            metadata: json!({}),
        })
        .unwrap();
    store
        .record_job_fit_score(job_fixture_score_input(
            &observed.id,
            &profile.id,
            &evidence.id,
        ))
        .unwrap();

    let first_health = store
        .record_job_source_health(JobSourceHealthInput {
            source_id: source.id.clone(),
            status: "healthy".to_string(),
            http_status: Some(200),
            error_code: None,
            fetched_count: 1,
            accepted_count: 1,
            rejected_count: 0,
            note: Some("Initial controlled refresh.".to_string()),
        })
        .unwrap();
    store
        .run_job_manual_refresh(JobManualRefreshInput {
            profile_id: profile.id.clone(),
            scope: scope.clone(),
            observed_role_ids: vec![observed.id.clone()],
            stale_role_ids: vec![],
            closed_role_ids: vec![],
            source_health_ids: vec![first_health.id],
            proof_level: "local_proof".to_string(),
            report_artifact_id: None,
        })
        .unwrap();

    let second_health = store
        .record_job_source_health(JobSourceHealthInput {
            source_id: source.id,
            status: "partial".to_string(),
            http_status: Some(200),
            error_code: Some("controlled_partial".to_string()),
            fetched_count: 3,
            accepted_count: 1,
            rejected_count: 2,
            note: Some("Second controlled refresh with stale and closed roles.".to_string()),
        })
        .unwrap();
    store
        .run_job_manual_refresh(JobManualRefreshInput {
            profile_id: profile.id.clone(),
            scope: scope.clone(),
            observed_role_ids: vec![observed.id],
            stale_role_ids: vec![stale.id],
            closed_role_ids: vec![closed.id],
            source_health_ids: vec![second_health.id],
            proof_level: "local_proof".to_string(),
            report_artifact_id: None,
        })
        .unwrap();

    (profile, scope)
}

fn research_convergence_test_input(run_id: &str) -> ResearchConvergenceStepInput {
    ResearchConvergenceStepInput {
        run_id: run_id.to_string(),
        max_iterations: None,
        max_seconds: None,
        max_sources: None,
        max_provider_calls: None,
        cost_cap_usd: None,
        source_novelty_threshold: None,
        confidence_delta_threshold: None,
        no_progress_iteration_limit: None,
        require_active_fact_check: None,
        allow_long_run: None,
        no_write: None,
        editorial_provider: None,
        editorial_model_name: None,
        editorial_endpoint: None,
        editorial_timeout_seconds: None,
    }
}

fn seed_research_convergence_claim(store: &Store, run_id: &str, text: &str) -> String {
    let card = store
        .add_source_card(SourceCardInput {
            title: "Primary convergence fixture".to_string(),
            url: format!(
                "https://example.com/convergence-fixture-{}",
                sha256(format!("{run_id}\n{text}").as_bytes())[..12].to_string()
            ),
            source_type: "paper".to_string(),
            provider: "test".to_string(),
            summary: text.to_string(),
            claims: vec![SourceClaim {
                claim: text.to_string(),
                kind: "fact".to_string(),
                confidence: 0.86,
            }],
            retrieved_at: None,
            metadata: json!({ "source_role": "primary", "trust_level": "high" }),
        })
        .unwrap();
    store
        .link_source_card_to_research_run(
            run_id,
            &card.id,
            "papers",
            "full-text",
            "must-read-primary",
            None,
        )
        .unwrap();
    store
        .ingest_research_claims_from_model_output(
            run_id,
            &card.id,
            "test",
            "fixture",
            &json!({
                "claims": [{
                    "text": text,
                    "kind": "fact",
                    "subject": "the system",
                    "predicate": "uses",
                    "object": "deterministic verification",
                    "confidence": 0.86,
                    "caveats": ["Fixture source only."],
                    "quote": "deterministic verification"
                }]
            })
            .to_string(),
        )
        .unwrap();
    card.id
}

fn seed_knowledge_source_card(store: &Store, slug: &str, summary: &str) -> SourceCard {
    store
        .add_source_card(SourceCardInput {
            title: format!("Knowledge source {slug}"),
            url: format!("https://example.com/knowledge/{slug}"),
            source_type: "web".to_string(),
            provider: "test".to_string(),
            summary: summary.to_string(),
            claims: vec![SourceClaim {
                claim: summary.to_string(),
                kind: "fact".to_string(),
                confidence: 0.8,
            }],
            retrieved_at: None,
            metadata: json!({
                "source_role": "primary",
                "trust_level": "medium",
                "test_fixture": true
            }),
        })
        .unwrap()
}

fn seed_daily_knowledge_report(
    store: &Store,
    slug: &str,
    topic: &str,
    summary: &str,
    generated_only: bool,
) -> (SourceCard, KnowledgeCluster, KnowledgeReport) {
    let mut metadata = json!({
        "source_role": "primary",
        "trust_level": "medium",
        "test_fixture": true
    });
    let source_type = if generated_only {
        metadata = json!({
            "source_role": "generated_synthesis",
            "source_kind": "knowledge_daily_briefing",
            "generated": true,
            "test_fixture": true
        });
        "knowledge_daily_briefing"
    } else {
        "web"
    };
    let card = store
        .add_source_card(SourceCardInput {
            title: format!("Daily knowledge source {slug}"),
            url: format!("https://example.com/daily-knowledge/{slug}"),
            source_type: source_type.to_string(),
            provider: "test".to_string(),
            summary: summary.to_string(),
            claims: vec![SourceClaim {
                claim: summary.to_string(),
                kind: "fact".to_string(),
                confidence: 0.84,
            }],
            retrieved_at: None,
            metadata,
        })
        .unwrap();
    let cluster = store
            .create_knowledge_cluster(KnowledgeClusterInput {
                topic: topic.to_string(),
                status: "active".to_string(),
                event_ids: Vec::new(),
                source_card_ids: vec![card.id.clone()],
                first_seen_at: None,
                last_seen_at: None,
                novelty_score: 0.88,
                momentum_score: 0.72,
                stale_score: 0.0,
                reason: "Daily briefing fixture has source-card-backed evidence for a timely AI knowledge update.".to_string(),
                duplicate_groups: json!({}),
                metadata: json!({ "fixture": "daily_knowledge_report" }),
            })
            .unwrap();
    let body = format!(
        "# {topic}\n\nCluster: `{}`\n\n## Executive Read\nThe update matters because {summary} This paragraph is intentionally written as reader-facing analysis rather than source bookkeeping, and it names confidence so a daily briefing can summarize the story without pretending the evidence is stronger than it is. The current confidence is moderate and the remaining uncertainty is whether official primary sources and wider developer reaction will corroborate the initial signal. Evidence: `{}`.\n\n## What Happened\nThe source-backed claim is that {summary} Arcwell should treat the source as untrusted evidence, compare it against official announcements, and keep the developing story attached to this cluster instead of creating duplicate pages. Evidence: `{}`.\n\n## Why It Matters\nThis is relevant to the AI and developer-relations knowledge graph because it can connect product launches, package activity, benchmark changes, community reaction, and company strategy in a single evolving page. The report should compare against existing wiki page history, watch for follow-up coverage, and distinguish confirmed facts from early reception. Evidence: `{}`.\n\n## Editorial Next Steps\n- Verify official primary source coverage and compare against existing wiki pages before duplicate-page creation.\n- Corroborate community reception, benchmark claims, pricing or access details, and follow-up posts before raising confidence.\n\n## Confidence And Uncertainty\nConfidence is moderate. Uncertainty remains around access, timing, independent corroboration, and whether later source cards change the interpretation.\n\nsource_cards:\n- `{}`\n",
        cluster.id, card.id, card.id, card.id, card.id
    );
    let report = store
        .record_knowledge_report(KnowledgeReportInput {
            cluster_id: cluster.id.clone(),
            title: format!("Daily Knowledge Report: {topic}"),
            body_markdown: body,
            status: "draft".to_string(),
            source_card_ids: vec![card.id.clone()],
            metadata: json!({ "fixture": "daily_knowledge_report" }),
        })
        .unwrap();
    (card, cluster, report)
}

fn due_utc_schedule_input(
    name: &str,
    recipient_ref: &str,
    metadata: Value,
) -> (IssueScheduleInput, String, String) {
    let now = Utc::now();
    let due = now - ChronoDuration::minutes(1);
    let created_at = due - ChronoDuration::hours(2);
    (
        IssueScheduleInput {
            name: name.to_string(),
            kind: "knowledge_daily_briefing".to_string(),
            channel: "email".to_string(),
            recipient_ref: recipient_ref.to_string(),
            time_zone: "utc".to_string(),
            hour: due.hour() as i64,
            minute: due.minute() as i64,
            catch_up_hours: 72,
            status: Some("active".to_string()),
            metadata,
        },
        created_at.to_rfc3339(),
        due.to_rfc3339(),
    )
}

fn force_issue_schedule_created_at(store: &Store, schedule_id: &str, created_at: &str) {
    store
        .conn
        .execute(
            "UPDATE issue_schedules SET created_at = ?2 WHERE id = ?1",
            params![schedule_id, created_at],
        )
        .unwrap();
}

fn force_knowledge_report_updated_at(store: &Store, report_id: &str, updated_at: &str) {
    store
        .conn
        .execute(
            "UPDATE knowledge_reports SET updated_at = ?2 WHERE id = ?1",
            params![report_id, updated_at],
        )
        .unwrap();
}

fn force_knowledge_report_body(store: &Store, report_id: &str, body_markdown: &str) {
    store
        .conn
        .execute(
            "UPDATE knowledge_reports SET body_markdown = ?2 WHERE id = ?1",
            params![report_id, body_markdown],
        )
        .unwrap();
}

fn write_daily_briefing_email_policy(store: &Store, recipient_ref: &str, target_email: &str) {
    write_policy(
        store,
        &format!(
            r#"
[[rules]]
id = "allow-daily-briefing-worker-enqueue"
effect = "allow"
action = "worker.enqueue"
package = "arcwell-knowledge"
source = "knowledge_daily_briefing"
reason = "allow native daily briefing issue schedule worker enqueue"
priority = 20

[[rules]]
id = "allow-daily-briefing-source-write"
effect = "allow"
action = "source.write"
package = "arcwell-llm-wiki"
provider = "arcwell"
source = "source_card_add"
reason = "allow generated daily briefing audit source card"
priority = 15

[[rules]]
id = "allow-daily-briefing-auto-approval"
effect = "allow"
action = "digest_candidate.auto_approve"
package = "arcwell-knowledge"
source = "knowledge_daily_briefing"
channel = "email"
subject = "{recipient_ref}"
target = "{recipient_ref}"
reason = "allow daily briefing auto approval for authorized recipient"
priority = 10

[[rules]]
id = "allow-daily-briefing-digest-delivery"
effect = "allow"
action = "digest_candidate.deliver"
package = "arcwell-knowledge"
source = "knowledge_daily_briefing_delivery"
channel = "email"
subject = "{recipient_ref}"
target = "{recipient_ref}"
reason = "allow daily briefing email delivery"
priority = 10

[[rules]]
id = "allow-daily-briefing-email-send"
effect = "allow"
action = "channel.send"
package = "arcwell-email"
provider = "cloudflare_email"
source = "email_send"
channel = "email"
subject = "{recipient_ref}"
target = "{target_email}"
reason = "allow daily briefing Cloudflare Email send"
priority = 10
"#
        ),
    );
}

fn seed_knowledge_event(store: &Store, canonical_key: &str) -> KnowledgeEvent {
    store
        .upsert_knowledge_event(KnowledgeEventInput {
            event_type: "package_release".to_string(),
            title: "OpenAI published a new package".to_string(),
            canonical_key: canonical_key.to_string(),
            primary_entity_key: Some("github:openai/example-package".to_string()),
            event_time: None,
            summary: "A package-release event candidate discovered from cross-source evidence."
                .to_string(),
            confidence: 0.72,
            metadata: json!({ "source_family": "github" }),
        })
        .unwrap()
}

mod knowledge;
use knowledge::seed_saturated_convergence_fixture;
mod proof;
mod schema_memory_policy;
use schema_memory_policy::{
    clear_x_bearer_env, mock_base_server, mock_header_server, mock_json_server,
    mock_oauth_request_assertion_server, mock_recording_sequence_server, mock_sequence_server,
    mock_status_server, mock_x_definitive_server, mock_x_following_server,
    test_provider_probe_spec, write_policy,
};
mod commerce;
mod job;
mod radar_fetch;
mod research;
mod wiki_worker_project_channel_digest;
mod x_pipeline;
