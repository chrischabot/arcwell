use crate::*;

pub(crate) fn wiki(store: Store, args: WikiCommand) -> Result<()> {
    match args.command {
        WikiSubcommand::Add {
            title,
            content,
            source,
        } => {
            let id = store.add_wiki_page(&title, &content, &source)?;
            print_json(&json!({ "ok": true, "id": id }))
        }
        WikiSubcommand::IngestFile { path } => {
            let id = store.ingest_wiki_file(&path)?;
            print_json(&json!({ "ok": true, "id": id }))
        }
        WikiSubcommand::IngestDir { path } => print_json(&store.ingest_wiki_dir(&path)?),
        WikiSubcommand::ImportCodexSwiftSources { path } => {
            print_json(&store.import_codex_swift_sources(&path)?)
        }
        WikiSubcommand::Sources => print_json(&store.list_watch_sources()?),
        WikiSubcommand::Search { query } => print_json(&store.search_wiki_pages(&query)?),
        WikiSubcommand::IngestJob { path } => print_json(&store.run_wiki_ingest_file_job(&path)?),
        WikiSubcommand::IngestUrl { url } => print_json(&store.run_wiki_ingest_url_job(&url)?),
        WikiSubcommand::IngestRendered {
            requested_url,
            final_url,
            title,
            rendered_html,
            rendered_html_file,
            rendered_text,
            rendered_text_file,
            captured_at,
            browser,
            screenshot_path,
        } => {
            let rendered_html = optional_inline_or_file(rendered_html, rendered_html_file)?;
            let rendered_text = optional_inline_or_file(rendered_text, rendered_text_file)?;
            print_json(
                &store.run_wiki_ingest_rendered_page_job(RenderedPageSnapshotInput {
                    requested_url,
                    final_url,
                    title,
                    rendered_html,
                    rendered_text,
                    captured_at,
                    browser,
                    screenshot_path,
                })?,
            )
        }
        WikiSubcommand::EnqueueRss { url } => print_json(&store.enqueue_rss_job(&url)?),
        WikiSubcommand::EnqueueGithub {
            owner,
            repo,
            mode,
            limit,
        } => print_json(&store.enqueue_github_repo_job(&owner, &repo, &mode, limit)?),
        WikiSubcommand::EnqueueGithubOwner { owner, limit } => {
            print_json(&store.enqueue_github_owner_job(&owner, limit)?)
        }
        WikiSubcommand::EnqueueArxiv { query, limit } => {
            print_json(&store.enqueue_arxiv_search_job(&query, limit)?)
        }
        WikiSubcommand::RunRss { url } => print_json(&store.run_rss_fetch_job(&url)?),
        WikiSubcommand::RunGithub {
            owner,
            repo,
            mode,
            limit,
        } => print_json(&store.run_github_repo_job(&owner, &repo, &mode, limit)?),
        WikiSubcommand::RunGithubOwner { owner, limit } => {
            print_json(&store.run_github_owner_job(&owner, limit)?)
        }
        WikiSubcommand::RunArxiv { query, limit } => {
            print_json(&store.run_arxiv_search_job(&query, limit)?)
        }
        WikiSubcommand::Compile { query } => print_json(&store.run_wiki_compile_job(&query)?),
        WikiSubcommand::Expand { topic } => print_json(&store.run_wiki_expand_page_job(&topic)?),
        WikiSubcommand::Jobs => print_json(&store.list_wiki_jobs()?),
        WikiSubcommand::Job { id } => print_json(&store.get_wiki_job(&id)?),
        WikiSubcommand::DecisionLedger { command } => wiki_decision_ledger(store, command),
        WikiSubcommand::List => print_json(&store.list_wiki_pages()?),
        WikiSubcommand::Read { id } => print_json(&store.read_wiki_page(&id)?),
    }
}

pub(crate) fn wiki_decision_ledger(
    store: Store,
    command: WikiDecisionLedgerSubcommand,
) -> Result<()> {
    match command {
        WikiDecisionLedgerSubcommand::Summary => print_json(&store.wiki_decision_ledger_summary()?),
        WikiDecisionLedgerSubcommand::List { limit } => {
            print_json(&store.list_wiki_decision_ledger(limit)?)
        }
    }
}

pub(crate) fn source_card(store: Store, args: SourceCardCommand) -> Result<()> {
    match args.command {
        SourceCardSubcommand::Add {
            title,
            url,
            source_type,
            provider,
            summary,
            claims_json,
        } => {
            let claims = serde_json::from_str(&claims_json).context("parsing --claims-json")?;
            print_json(&store.add_source_card(SourceCardInput {
                title,
                url,
                source_type,
                provider,
                summary,
                claims,
                retrieved_at: None,
                metadata: json!({ "created_by": "arcwell-cli" }),
            })?)
        }
        SourceCardSubcommand::IngestRedditBrowserListing {
            locator,
            listing_json,
            limit,
        } => {
            let size = fs::metadata(&listing_json)
                .with_context(|| format!("reading metadata for {}", listing_json.display()))?
                .len();
            if size > 2_000_000 {
                bail!("Reddit browser listing JSON is too large");
            }
            let body = fs::read_to_string(&listing_json)
                .with_context(|| format!("reading {}", listing_json.display()))?;
            let listing: Value = serde_json::from_str(&body)
                .with_context(|| format!("parsing {}", listing_json.display()))?;
            print_json(&store.ingest_reddit_browser_listing(&locator, &listing, limit)?)
        }
        SourceCardSubcommand::Search { query } => print_json(&store.search_source_cards(&query)?),
        SourceCardSubcommand::Read { id } => print_json(&store.read_source_card(&id)?),
    }
}

pub(crate) fn knowledge(store: Store, args: KnowledgeCommand) -> Result<()> {
    match args.command {
        KnowledgeSubcommand::ProjectRadarRun {
            run_id,
            topic,
            max_source_cards,
        } => print_json(&store.project_knowledge_from_radar_run(
            &run_id,
            topic.as_deref(),
            max_source_cards,
        )?),
        KnowledgeSubcommand::ProjectSourceCardQuery {
            query,
            topic,
            max_source_cards,
        } => print_json(&store.project_knowledge_from_source_card_query(
            &query,
            topic.as_deref(),
            max_source_cards,
        )?),
        KnowledgeSubcommand::ClusterBacklog {
            max_source_cards,
            min_group_size,
            max_clusters,
        } => print_json(&store.cluster_source_card_backlog(
            max_source_cards,
            min_group_size,
            max_clusters,
        )?),
        KnowledgeSubcommand::Events { limit } => print_json(&store.list_knowledge_events(limit)?),
        KnowledgeSubcommand::Clusters { limit } => {
            print_json(&store.list_knowledge_clusters(limit)?)
        }
        KnowledgeSubcommand::ExpandCluster {
            cluster_id,
            skip_digest,
        } => print_json(&store.expand_knowledge_cluster(&cluster_id, !skip_digest)?),
        KnowledgeSubcommand::WriteClusterModel {
            cluster_id,
            provider,
            model_name,
            endpoint,
            timeout_seconds,
            skip_digest,
        } => print_json(&store.expand_knowledge_cluster_with_model_writer(
            KnowledgeClusterWriterModelInput {
                cluster_id,
                model_provider: provider,
                model_name,
                endpoint,
                timeout_seconds,
                create_digest: !skip_digest,
            },
        )?),
        KnowledgeSubcommand::PromoteCluster {
            cluster_id,
            reviewer,
            reason,
        } => print_json(&store.promote_knowledge_cluster(
            &cluster_id,
            reviewer.as_deref(),
            reason.as_deref(),
        )?),
        KnowledgeSubcommand::DecideClusterEditorial {
            cluster_id,
            no_enqueue,
        } => print_json(&store.decide_knowledge_cluster_editorial(&cluster_id, !no_enqueue)?),
        KnowledgeSubcommand::InvestigateCluster { cluster_id } => {
            print_json(&store.create_knowledge_cluster_investigation(&cluster_id)?)
        }
        KnowledgeSubcommand::ExecuteClusterInvestigation { cluster_id } => {
            print_json(&store.execute_knowledge_cluster_investigation(&cluster_id)?)
        }
        KnowledgeSubcommand::EnqueueClusterExpansion {
            cluster_id,
            skip_digest,
        } => print_json(&store.enqueue_knowledge_cluster_expansion_job(&cluster_id, !skip_digest)?),
        KnowledgeSubcommand::EnqueueClusterEditorialDecision {
            cluster_id,
            no_enqueue,
        } => print_json(
            &store.enqueue_knowledge_cluster_editorial_decision_job(&cluster_id, !no_enqueue)?,
        ),
        KnowledgeSubcommand::EnqueueClusterModelWrite {
            cluster_id,
            provider,
            model_name,
            endpoint,
            timeout_seconds,
            skip_digest,
        } => print_json(&store.enqueue_knowledge_cluster_model_writer_job(
            &cluster_id,
            &provider,
            model_name.as_deref(),
            endpoint.as_deref(),
            timeout_seconds,
            !skip_digest,
        )?),
        KnowledgeSubcommand::ScheduleClusterModelWrite {
            cluster_id,
            provider,
            model_name,
            endpoint,
            timeout_seconds,
            skip_digest,
            cadence,
            status,
        } => print_json(&store.schedule_knowledge_cluster_model_write(
            &cluster_id,
            &provider,
            model_name.as_deref(),
            endpoint.as_deref(),
            timeout_seconds,
            !skip_digest,
            &cadence,
            &status,
        )?),
        KnowledgeSubcommand::EnqueueDueModelWrites {
            max_clusters,
            provider,
            model_name,
            endpoint,
            timeout_seconds,
            skip_digest,
        } => print_json(&store.enqueue_due_knowledge_cluster_model_writer_jobs(
            max_clusters,
            &provider,
            model_name.as_deref(),
            endpoint.as_deref(),
            timeout_seconds,
            !skip_digest,
        )?),
        KnowledgeSubcommand::EnqueueClusterInvestigation { cluster_id } => {
            print_json(&store.enqueue_knowledge_cluster_investigation_job(&cluster_id)?)
        }
        KnowledgeSubcommand::EnqueueClusterInvestigationExecution { cluster_id } => {
            print_json(&store.enqueue_knowledge_cluster_investigation_execution_job(&cluster_id)?)
        }
        KnowledgeSubcommand::EnqueueBacklogClustering {
            max_source_cards,
            min_group_size,
            max_clusters,
        } => print_json(&store.enqueue_knowledge_cluster_backlog_job(
            max_source_cards,
            min_group_size,
            max_clusters,
        )?),
        KnowledgeSubcommand::ScheduleBacklogClustering {
            max_source_cards,
            min_group_size,
            max_clusters,
            cadence,
            status,
        } => print_json(&store.schedule_knowledge_cluster_backlog(
            max_source_cards,
            min_group_size,
            max_clusters,
            &cadence,
            &status,
        )?),
        KnowledgeSubcommand::EnqueueModelClusters {
            query,
            provider,
            model_name,
            endpoint,
            timeout_seconds,
            max_source_cards,
            max_clusters,
        } => print_json(&store.enqueue_knowledge_cluster_model_proposal_job(
            &query,
            &provider,
            model_name.as_deref(),
            endpoint.as_deref(),
            timeout_seconds,
            max_source_cards,
            max_clusters,
        )?),
        KnowledgeSubcommand::RunModelClusters {
            query,
            provider,
            model_name,
            endpoint,
            timeout_seconds,
            max_source_cards,
            max_clusters,
        } => print_json(&store.run_knowledge_cluster_model_proposal_job(
            &query,
            &provider,
            model_name.as_deref(),
            endpoint.as_deref(),
            timeout_seconds,
            max_source_cards,
            max_clusters,
        )?),
        KnowledgeSubcommand::ScheduleModelClusters {
            query,
            provider,
            model_name,
            endpoint,
            timeout_seconds,
            max_source_cards,
            max_clusters,
            cadence,
            status,
        } => print_json(&store.schedule_knowledge_cluster_model_proposals(
            &query,
            &provider,
            model_name.as_deref(),
            endpoint.as_deref(),
            timeout_seconds,
            max_source_cards,
            max_clusters,
            &cadence,
            &status,
        )?),
        KnowledgeSubcommand::ProposeClusters {
            query,
            provider,
            model_name,
            endpoint,
            timeout_seconds,
            max_source_cards,
            max_clusters,
        } => {
            let source_card_ids = store
                .search_source_cards(&query)?
                .into_iter()
                .take(max_source_cards.clamp(1, 80))
                .map(|card| card.id)
                .collect::<Vec<_>>();
            print_json(&store.invoke_knowledge_cluster_model(
                KnowledgeClusterProposalModelInput {
                    source_card_ids,
                    model_provider: provider,
                    model_name,
                    endpoint,
                    timeout_seconds,
                    max_clusters,
                },
            )?)
        }
        KnowledgeSubcommand::Reports { limit } => print_json(&store.list_knowledge_reports(limit)?),
        KnowledgeSubcommand::Entities { limit } => {
            print_json(&store.list_knowledge_entities(limit)?)
        }
        KnowledgeSubcommand::ResolveEntities { limit } => {
            print_json(&store.propose_knowledge_entity_resolutions(limit)?)
        }
        KnowledgeSubcommand::UpsertEntity {
            entity_type,
            name,
            canonical_key,
            aliases_json,
            homepage_url,
            source_card_ids_json,
            wiki_page_id,
            confidence,
            metadata_json,
        } => {
            let aliases = serde_json::from_str(&aliases_json).context("parsing --aliases-json")?;
            let source_card_ids = serde_json::from_str(&source_card_ids_json)
                .context("parsing --source-card-ids-json")?;
            let metadata = parse_json_arg(&metadata_json, "--metadata-json")?;
            print_json(&store.upsert_knowledge_entity(KnowledgeEntityInput {
                entity_type,
                name,
                canonical_key,
                aliases,
                homepage_url,
                source_card_ids,
                wiki_page_id,
                confidence,
                metadata,
            })?)
        }
        KnowledgeSubcommand::ResolveEntityModel {
            left_entity_id,
            right_entity_id,
            provider,
            model_name,
            endpoint,
            timeout_seconds,
        } => print_json(&store.invoke_knowledge_entity_resolution_model(
            KnowledgeEntityResolutionModelInput {
                left_entity_id,
                right_entity_id,
                model_provider: provider,
                model_name,
                endpoint,
                timeout_seconds,
            },
        )?),
        KnowledgeSubcommand::EnqueueEntityResolutionModel {
            left_entity_id,
            right_entity_id,
            provider,
            model_name,
            endpoint,
            timeout_seconds,
        } => print_json(&store.enqueue_knowledge_entity_resolution_model_job(
            &left_entity_id,
            &right_entity_id,
            &provider,
            model_name.as_deref(),
            endpoint.as_deref(),
            timeout_seconds,
        )?),
        KnowledgeSubcommand::EnqueueDueEntityResolution {
            max_pairs,
            provider,
            model_name,
            endpoint,
            timeout_seconds,
        } => print_json(&store.enqueue_due_knowledge_entity_resolution_jobs(
            max_pairs,
            &provider,
            model_name.as_deref(),
            endpoint.as_deref(),
            timeout_seconds,
            None,
        )?),
        KnowledgeSubcommand::ScheduleEntityResolution {
            max_pairs,
            provider,
            model_name,
            endpoint,
            timeout_seconds,
            cadence,
            status,
        } => print_json(&store.schedule_knowledge_entity_resolution(
            &provider,
            model_name.as_deref(),
            endpoint.as_deref(),
            timeout_seconds,
            max_pairs,
            &cadence,
            &status,
        )?),
        KnowledgeSubcommand::ScheduleDailyBriefing {
            name,
            channel,
            recipient_ref,
            time_zone,
            hour,
            minute,
            catch_up_hours,
            max_reports,
            max_source_cards,
            status,
        } => print_json(&store.upsert_issue_schedule(IssueScheduleInput {
            name,
            kind: "knowledge_daily_briefing".to_string(),
            channel,
            recipient_ref,
            time_zone,
            hour,
            minute,
            catch_up_hours,
            metadata: serde_json::json!({
                "window_hours": 24,
                "max_reports": max_reports,
                "max_source_cards": max_source_cards,
                "max_catch_up_ticks": 3,
                "created_from": "arcwell-cli knowledge schedule-daily-briefing"
            }),
            status: Some(status),
        })?),
        KnowledgeSubcommand::IssueSchedules => print_json(&store.list_issue_schedules()?),
        KnowledgeSubcommand::IssueScheduleTicks { schedule_id } => {
            print_json(&store.list_issue_schedule_ticks(schedule_id.as_deref())?)
        }
        KnowledgeSubcommand::EntityResolutions { limit } => {
            print_json(&store.list_knowledge_entity_resolutions(limit)?)
        }
        KnowledgeSubcommand::Relations { limit } => {
            print_json(&store.list_knowledge_relations(limit)?)
        }
        KnowledgeSubcommand::AdapterRuns { limit } => {
            print_json(&store.list_knowledge_adapter_runs(limit)?)
        }
    }
}

pub(crate) fn research(store: Store, args: ResearchCommand) -> Result<()> {
    match args.command {
        ResearchSubcommand::Capabilities => print_json(&research_capabilities(store.paths())),
        ResearchSubcommand::Run { query } => print_json(&store.create_deep_research_run(&query)?),
        ResearchSubcommand::Status { run_id } => print_json(&store.research_run_status(&run_id)?),
        ResearchSubcommand::Read { run_id } => print_json(&store.read_research_run(&run_id)?),
        ResearchSubcommand::AuditRun { run_id } => print_json(&store.audit_research_run(&run_id)?),
        ResearchSubcommand::Stop { run_id } => print_json(&store.stop_research_run(&run_id)?),
        ResearchSubcommand::Sources { run_id } => {
            print_json(&store.list_research_run_sources(&run_id)?)
        }
        ResearchSubcommand::AddSource {
            run_id,
            title,
            url,
            local_ref,
            source_family,
            source_type,
            provider,
            reason,
            priority,
            fetch_status,
            read_depth,
            triage_status,
            canonical_key,
            notes,
        } => {
            let source = store.upsert_research_source(ResearchSourceInput {
                url,
                local_ref,
                title: title.clone(),
                source_family,
                source_type,
                provider,
                author: None,
                published_at: None,
                language: None,
                priority,
                reason: reason.unwrap_or_else(|| format!("Candidate source for {title}")),
                canonical_key,
                fetch_status,
                read_depth: read_depth.clone(),
                metadata: json!({ "created_by": "arcwell-cli" }),
            })?;
            print_json(&store.link_research_source_to_run(
                &run_id,
                &source.id,
                None,
                &triage_status,
                &read_depth,
                notes.as_deref(),
            )?)
        }
        ResearchSubcommand::LinkSourceCard {
            run_id,
            source_card_id,
            source_family,
            read_depth,
            triage_status,
            notes,
        } => print_json(&store.link_source_card_to_research_run(
            &run_id,
            &source_card_id,
            &source_family,
            &read_depth,
            &triage_status,
            notes.as_deref(),
        )?),
        ResearchSubcommand::ExtractionPrompt {
            run_id,
            source_card_id,
        } => print_json(&store.build_research_extraction_prompt(&run_id, &source_card_id)?),
        ResearchSubcommand::IngestClaims {
            run_id,
            source_card_id,
            provider,
            model,
            output_json,
        } => print_json(&store.ingest_research_claims_from_model_output(
            &run_id,
            &source_card_id,
            &provider,
            &model,
            &output_json,
        )?),
        ResearchSubcommand::Claims { run_id } => print_json(&store.list_research_claims(&run_id)?),
        ResearchSubcommand::Clusters { run_id } => {
            print_json(&store.build_research_clusters(&run_id)?)
        }
        ResearchSubcommand::Skeptic { run_id } => {
            print_json(&store.run_research_skeptic_pass(&run_id)?)
        }
        ResearchSubcommand::Report {
            run_id,
            saturation_reason,
            no_write,
        } => print_json(&store.compile_research_report(&run_id, &saturation_reason, !no_write)?),
        ResearchSubcommand::Converge(args) => print_json(
            &store.run_research_convergence_to_stop(research_convergence_step_input(args))?,
        ),
        ResearchSubcommand::ConvergeStep(args) => {
            print_json(&store.start_research_convergence(research_convergence_start_input(args))?)
        }
        ResearchSubcommand::ConvergeEnqueue(args) => print_json(
            &store.enqueue_research_convergence_job(research_convergence_step_input(args))?,
        ),
        ResearchSubcommand::ConvergenceStatus { run_id } => {
            print_json(&store.research_convergence_status(&run_id)?)
        }
        ResearchSubcommand::Iterations { run_id } => {
            print_json(&store.list_research_iterations(&run_id)?)
        }
        ResearchSubcommand::IterationRead { id } => {
            print_json(&store.read_research_iteration(&id)?)
        }
        ResearchSubcommand::Statements { run_id } => {
            print_json(&store.list_research_statements(&run_id)?)
        }
        ResearchSubcommand::Challenges { run_id } => {
            print_json(&store.list_research_challenges(&run_id)?)
        }
        ResearchSubcommand::ConvergenceHostSearchTasks { run_id } => {
            print_json(&store.list_research_convergence_host_search_tasks(&run_id)?)
        }
        ResearchSubcommand::ConvergenceProviderSearch {
            run_id,
            provider,
            max_tasks,
            max_results,
            max_provider_calls,
            enqueue_selected_url_ingest,
            max_ingest_jobs,
            cost_cap_usd,
            endpoint,
            api_key,
            model,
            timeout_seconds,
        } => print_json(&store.run_research_convergence_provider_search(
            ResearchConvergenceProviderSearchInput {
                run_id,
                provider,
                max_tasks,
                max_results,
                max_provider_calls,
                enqueue_selected_url_ingest: Some(enqueue_selected_url_ingest),
                max_ingest_jobs,
                cost_cap_usd,
                endpoint,
                api_key,
                model,
                timeout_seconds,
            },
        )?),
        ResearchSubcommand::Disproofs { run_id } => {
            print_json(&store.list_research_disproofs(&run_id)?)
        }
        ResearchSubcommand::Revisions { run_id } => {
            print_json(&store.list_research_revisions(&run_id)?)
        }
        ResearchSubcommand::FactChecks { run_id } => {
            print_json(&store.list_research_fact_checks(&run_id)?)
        }
        ResearchSubcommand::ActiveFactCheck {
            run_id,
            artifact_id,
            max_sentences,
            no_challenges,
        } => print_json(
            &store.run_research_active_fact_check(ResearchActiveFactCheckInput {
                run_id,
                artifact_id,
                max_sentences,
                create_challenges: Some(!no_challenges),
            })?,
        ),
        ResearchSubcommand::ConvergenceCloseLoop(args) => print_json(
            &store
                .run_research_convergence_close_loop(research_convergence_close_loop_input(args))?,
        ),
        ResearchSubcommand::ConvergenceSnapshots { run_id } => {
            print_json(&store.list_research_convergence_snapshots(&run_id)?)
        }
        ResearchSubcommand::ConvergenceReport { run_id } => {
            print_json(&store.compile_research_convergence_report(&run_id)?)
        }
        ResearchSubcommand::ReportJudgments { run_id } => {
            print_json(&store.list_research_report_judgments(&run_id)?)
        }
        ResearchSubcommand::Plan { query, max_sources } => {
            print_json(&store.create_research_plan(&query, max_sources)?)
        }
        ResearchSubcommand::Search {
            query,
            provider,
            max_results,
            endpoint,
            api_key,
            model,
            timeout_seconds,
            write_wiki,
        } => {
            let config = WebSearchConfig {
                provider,
                max_results,
                endpoint,
                api_key,
                model,
                timeout_seconds,
            };
            if write_wiki {
                let (response, page_id) = store.web_search_to_wiki(&query, config)?;
                print_json(&json!({ "response": response, "page_id": page_id }))
            } else {
                print_json(&store.web_search(&query, config)?)
            }
        }
        ResearchSubcommand::Workflow { query } => {
            print_json(&store.create_research_workflow(&query)?)
        }
        ResearchSubcommand::Tasks { run_id } => print_json(&store.list_research_tasks(&run_id)?),
        ResearchSubcommand::RoleStart {
            run_id,
            role,
            host,
            execution_mode,
            host_thread_id,
            host_subagent_id,
            tool_surface,
            prompt_version,
            prompt_hash,
            input_artifact_ids,
        } => print_json(&store.start_research_role_run(ResearchRoleRunStart {
            run_id,
            role,
            host,
            host_thread_id,
            host_subagent_id,
            tool_surface,
            prompt_version,
            prompt_hash,
            execution_mode,
            input_artifact_ids,
        })?),
        ResearchSubcommand::RoleFinish {
            role_run_id,
            status,
            output_artifact_id,
            error_kind,
            error_message,
        } => print_json(&store.finish_research_role_run(
            &role_run_id,
            &status,
            output_artifact_id.as_deref(),
            error_kind.as_deref(),
            error_message.as_deref(),
        )?),
        ResearchSubcommand::RoleRuns { run_id } => {
            print_json(&store.list_research_role_runs(&run_id)?)
        }
        ResearchSubcommand::ArtifactAdd {
            run_id,
            artifact_type,
            title,
            body,
            role_run_id,
            metadata_json,
        } => {
            let metadata =
                serde_json::from_str(&metadata_json).context("parsing --metadata-json")?;
            print_json(&store.record_research_artifact(ResearchArtifactInput {
                run_id,
                role_run_id,
                artifact_type,
                title,
                body,
                metadata,
            })?)
        }
        ResearchSubcommand::Artifacts { run_id } => {
            print_json(&store.list_research_artifacts(&run_id)?)
        }
        ResearchSubcommand::ArtifactRead { id } => print_json(&store.read_research_artifact(&id)?),
        ResearchSubcommand::HostSearchRecord {
            run_id,
            query,
            host,
            tool_surface,
            role_run_id,
            query_intent,
            requested_recency,
            requested_domains,
            cost_decision_id,
            results_json,
        } => {
            let results: Vec<ResearchHostSearchResultInput> =
                serde_json::from_str(&results_json).context("parsing --results-json")?;
            print_json(&store.record_research_host_search(ResearchHostSearchInput {
                run_id,
                role_run_id,
                host,
                tool_surface,
                query,
                query_intent,
                requested_recency,
                requested_domains,
                cost_decision_id,
                results,
            })?)
        }
        ResearchSubcommand::HostSearches { run_id } => {
            print_json(&store.list_research_host_searches(&run_id)?)
        }
        ResearchSubcommand::HostSearchRead { id } => {
            print_json(&store.read_research_host_search(&id)?)
        }
        ResearchSubcommand::DocumentExtract {
            run_id,
            path,
            media_type,
            research_source_id,
            source_card_id,
        } => print_json(
            &store.extract_research_document_file(ResearchDocumentInput {
                run_id,
                research_source_id,
                source_card_id,
                path,
                media_type,
            })?,
        ),
        ResearchSubcommand::Documents { run_id } => {
            print_json(&store.list_research_documents(&run_id)?)
        }
        ResearchSubcommand::DocumentRead { id } => print_json(&store.read_research_document(&id)?),
        ResearchSubcommand::EvidencePack { run_id } => {
            print_json(&store.build_research_evidence_pack(&run_id)?)
        }
        ResearchSubcommand::EditorialInvoke {
            run_id,
            stage,
            model_provider,
            model_name,
            prompt_version,
            input_artifact_id,
            endpoint,
            api_key,
            timeout_seconds,
        } => print_json(
            &store.invoke_research_editorial(ResearchEditorialInvokeInput {
                run_id,
                stage,
                model_provider,
                model_name,
                prompt_version,
                input_artifact_id,
                endpoint,
                api_key,
                timeout_seconds,
            })?,
        ),
        ResearchSubcommand::EditorialRecord {
            run_id,
            stage,
            model_provider,
            model_name,
            prompt_version,
            input_artifact_id,
            output_artifact_id,
            cost_decision_id,
            status,
            score_json,
            error_message,
        } => {
            let score = serde_json::from_str(&score_json).context("parsing --score-json")?;
            print_json(
                &store.record_research_editorial_run(ResearchEditorialRunInput {
                    run_id,
                    stage,
                    model_provider,
                    model_name,
                    prompt_version,
                    input_artifact_id,
                    output_artifact_id,
                    cost_decision_id,
                    status,
                    score,
                    error_message,
                })?,
            )
        }
        ResearchSubcommand::EditorialRuns { run_id } => {
            print_json(&store.list_research_editorial_runs(&run_id)?)
        }
        ResearchSubcommand::EditorialRead { id } => {
            print_json(&store.get_research_editorial_run(&id)?)
        }
        ResearchSubcommand::CompleteTask { task_id, notes } => {
            print_json(&store.complete_research_task(&task_id, &notes)?)
        }
        ResearchSubcommand::Brief { query, no_write } => {
            print_json(&store.create_research_brief_from_wiki(&query, !no_write)?)
        }
        ResearchSubcommand::Audit { query } => print_json(&store.audit_research_output(&query)?),
        ResearchSubcommand::Runs => print_json(&store.list_research_runs()?),
    }
}
