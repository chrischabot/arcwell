use super::*;

#[test]
fn severe_model_entity_resolution_cost_denial_precedes_provider_call() {
    // CLAIM: Cost kill switches stop OpenAI entity-resolution proposals
    // before credentials, provider calls, or durable resolution writes.
    // ORACLE: provider kill switch error is returned, no resolution row is
    // inserted, and the denied cost decision is recorded for ops.
    // SEVERITY: Severe because unattended research systems need hard spend
    // brakes that cannot be bypassed by model-backed enrichment.
    let store = test_store("knowledge-model-resolution-cost-deny");
    store
        .set_cost_policy("provider", "openai", None, true, None)
        .unwrap();
    let left_card = seed_knowledge_source_card(&store, "cost-left", "Cost left evidence.");
    let right_card = seed_knowledge_source_card(&store, "cost-right", "Cost right evidence.");
    let left = store
        .upsert_knowledge_entity(KnowledgeEntityInput {
            entity_type: "company".to_string(),
            name: "Cost Left".to_string(),
            canonical_key: "company:cost-left".to_string(),
            aliases: vec!["Cost Left".to_string()],
            homepage_url: Some("https://left.example.com".to_string()),
            source_card_ids: vec![left_card.id],
            wiki_page_id: None,
            confidence: 0.8,
            metadata: json!({}),
        })
        .unwrap();
    let right = store
        .upsert_knowledge_entity(KnowledgeEntityInput {
            entity_type: "company".to_string(),
            name: "Cost Right".to_string(),
            canonical_key: "company:cost-right".to_string(),
            aliases: vec!["Cost Right".to_string()],
            homepage_url: Some("https://right.example.com".to_string()),
            source_card_ids: vec![right_card.id],
            wiki_page_id: None,
            confidence: 0.8,
            metadata: json!({}),
        })
        .unwrap();

    let error = store
        .invoke_knowledge_entity_resolution_model(KnowledgeEntityResolutionModelInput {
            left_entity_id: left.id,
            right_entity_id: right.id,
            model_provider: "openai".to_string(),
            model_name: Some("gpt-5.5-mini".to_string()),
            endpoint: Some("https://api.openai.com/v1/responses".to_string()),
            timeout_seconds: Some(5),
        })
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("budget blocked knowledge entity resolution"),
        "{error}"
    );
    assert!(!error.contains("OPENAI_API_KEY"), "{error}");
    assert!(
        store
            .list_knowledge_entity_resolutions(10)
            .unwrap()
            .is_empty()
    );
    let decisions = store.list_cost_decisions(10).unwrap();
    assert_eq!(decisions.len(), 1);
    assert!(!decisions[0].allowed);
    assert_eq!(
        decisions[0].source.as_deref(),
        Some("knowledge_entity_resolution")
    );
}

#[test]
fn severe_knowledge_projection_from_radar_run_uses_selected_source_cards() {
    // CLAIM: A scored radar run can become a unified knowledge projection
    // without bypassing selected source-card provenance.
    // ORACLE: selected radar evidence creates confirmed events, cluster
    // lineage points back to the radar run, and unscored/empty runs fail.
    // SEVERITY: Severe because live API/browser E2E will use radar as the
    // source acquisition layer before knowledge projection.
    let store = test_store("knowledge-radar-projection");
    store
            .add_source_card(SourceCardInput {
                title: "Knowledge radar package release".to_string(),
                url: "https://example.com/knowledge-radar-package-release".to_string(),
                source_type: "github_release".to_string(),
                provider: "github".to_string(),
                summary: "Knowledge radar projection agent package release with enough launch detail to score strongly.".to_string(),
                claims: vec![SourceClaim {
                    claim: "A package release was published.".to_string(),
                    kind: "fact".to_string(),
                    confidence: 0.9,
                }],
                retrieved_at: Some("2026-06-25T00:00:00Z".to_string()),
                metadata: json!({ "owner": "openai", "repo": "agents", "tag": "v1" }),
            })
            .unwrap();
    store
            .add_source_card(SourceCardInput {
                title: "Knowledge radar developer reaction".to_string(),
                url: "https://example.com/knowledge-radar-developer-reaction".to_string(),
                source_type: "hackernews_story".to_string(),
                provider: "hackernews".to_string(),
                summary: "Knowledge radar projection developer reaction connects the package to agent infrastructure and MCP adoption.".to_string(),
                claims: Vec::new(),
                retrieved_at: Some("2026-06-25T00:30:00Z".to_string()),
                metadata: json!({}),
            })
            .unwrap();
    let empty_profile = store
            .create_radar_profile(RadarProfileInput {
                name: "knowledge-empty-radar".to_string(),
                description: "Empty knowledge radar".to_string(),
                window_hours: 24,
                min_score: 1.0,
                max_items: Some(5),
                languages: vec!["en".to_string()],
                source_selectors: json!([{ "kind": "source_card_query", "query": "missing radar evidence" }]),
                delivery_policy: json!({ "delivery": "manual_only" }),
                model_policy: json!({ "model_scoring": "disabled" }),
                metadata: json!({}),
            })
            .unwrap();
    let empty_run = store.run_radar_profile(&empty_profile.id, None).unwrap();
    let empty_projection = store
        .project_knowledge_from_radar_run(&empty_run.run.id, None, 5)
        .unwrap_err();
    assert!(
        empty_projection
            .to_string()
            .contains("requires selected source-card evidence")
    );

    let profile = store
            .create_radar_profile(RadarProfileInput {
                name: "knowledge-radar-projection".to_string(),
                description: "Knowledge radar projection".to_string(),
                window_hours: 24,
                min_score: 1.0,
                max_items: Some(5),
                languages: vec!["en".to_string()],
                source_selectors: json!([{ "kind": "source_card_query", "query": "Knowledge radar projection" }]),
                delivery_policy: json!({ "delivery": "manual_only" }),
                model_policy: json!({ "model_scoring": "disabled" }),
                metadata: json!({}),
            })
            .unwrap();
    let run = store.run_radar_profile(&profile.id, None).unwrap();
    assert_eq!(run.run.status, "scored");
    let projection = store
        .project_knowledge_from_radar_run(&run.run.id, Some("Knowledge radar projection trend"), 10)
        .unwrap();
    assert_eq!(
        projection
            .cluster
            .metadata
            .pointer("/source_metadata/radar_run_id")
            .and_then(Value::as_str),
        Some(run.run.id.as_str())
    );
    assert!(
        projection
            .report
            .body_markdown
            .contains("Knowledge radar projection")
    );
    assert!(
        projection
            .events
            .iter()
            .all(|event| event.status == "confirmed")
    );
}

#[test]
fn severe_narrative_claim_filter_excludes_titles_and_page_dumps() {
    // CLAIM: the analyst narrative layer promotes analytical findings, not
    // source titles or scraped page dumps that merely contain interesting
    // numbers.
    // ORACLE: source-title and page-dump claims are filtered while a direct
    // measurement claim remains eligible for statement compilation.
    // SEVERITY: Severe because promoting raw corpus inventory as current
    // position was the failure mode in the saturated live proof.
    fn record(text: &str, confidence: f64, caveats: Vec<&str>) -> ResearchClaimRecord {
        ResearchClaimRecord {
            claim: ResearchClaim {
                id: format!("rclaim-{}", &sha256(text.as_bytes())[..16]),
                run_id: "rrun-narrative-filter".to_string(),
                text: text.to_string(),
                kind: "measurement".to_string(),
                subject: None,
                predicate: None,
                object_value: None,
                temporal_scope: None,
                confidence,
                caveats: caveats.into_iter().map(ToOwned::to_owned).collect(),
                extraction_provider: "test".to_string(),
                extraction_model: "fixture".to_string(),
                extracted_at: now(),
                metadata: json!({}),
            },
            sources: Vec::new(),
            document_anchors: Vec::new(),
        }
    }

    let title = record(
        "Top AI Startups in the UK 2026 (Funding & Valuation)",
        0.55,
        vec![
            "Imported by the production proof harness from provider search snippet/source-card evidence; verify against full source text before publication.",
        ],
    );
    let page_dump = record(
        "LDN/ai 74 companies and 127 connections map people insights events news bits. Table of Contents Toggle What is the state of London artificial intelligence in 2026? Published 16 March 2026 and updated weekly. London is Europe's largest artificial intelligence hub. More than $8 billion in venture capital flowed into London-based AI companies between 2021 and 2024. Image: Unsplash. Last updated April 2026. Read more and subscribe to the newsletter for the full database of companies, investors, and people.",
        0.72,
        vec![
            "Bounded URL-ingest extraction; verify quoted/numeric claims against the original page before external publication.",
        ],
    );
    let analytical = record(
        "UK AI startups raised over GBP 6 billion in 2025, accounting for more than one third of UK venture capital.",
        0.72,
        vec![
            "Bounded URL-ingest extraction; verify quoted/numeric claims against the original page before external publication.",
        ],
    );
    let records = vec![title, page_dump, analytical];

    let narrative = narrative_research_claims(&records);
    assert_eq!(narrative.len(), 1);
    assert!(
        narrative[0]
            .claim
            .text
            .contains("raised over GBP 6 billion")
    );
}
