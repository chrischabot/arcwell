use super::*;

impl Store {
    pub fn project_knowledge_from_source_card_query(
        &self,
        query: &str,
        topic: Option<&str>,
        max_source_cards: usize,
    ) -> Result<KnowledgeProjectionReport> {
        validate_query(query)?;
        let cards = self
            .search_source_cards(query)?
            .into_iter()
            .take(max_source_cards.clamp(1, 50))
            .collect::<Vec<_>>();
        let topic = topic
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("Knowledge trend: {query}"));
        self.project_knowledge_from_source_cards(
            &topic,
            cards,
            "source_card_query",
            "Local Proof",
            "existing_source_card_projection",
            Vec::new(),
            json!({ "query": query, "max_source_cards": max_source_cards.clamp(1, 50) }),
        )
    }

    pub fn cluster_source_card_backlog(
        &self,
        max_source_cards: usize,
        min_group_size: usize,
        max_clusters: usize,
    ) -> Result<KnowledgeClusterBacklogReport> {
        let max_source_cards = max_source_cards.clamp(1, 500);
        let min_group_size = min_group_size.clamp(1, 20);
        let max_clusters = max_clusters.clamp(1, 50);
        let clustered_source_card_ids = self.knowledge_clustered_source_card_ids()?;
        let mut groups = BTreeMap::<String, (String, Vec<SourceCard>)>::new();
        let mut report = KnowledgeClusterBacklogReport {
            inspected: 0,
            accepted: 0,
            skipped: 0,
            groups_considered: 0,
            projections: Vec::new(),
            warnings: Vec::new(),
        };

        for card in self.list_source_cards()?.into_iter().take(max_source_cards) {
            report.inspected += 1;
            if clustered_source_card_ids.contains(&card.id) {
                report.skipped += 1;
                continue;
            }
            if source_card_is_generated_only_evidence(&card) {
                report.skipped += 1;
                report
                    .warnings
                    .push(format!("skipped generated-only source card {}", card.id));
                continue;
            }
            let Some(group) = knowledge_backlog_group_for_source_card(&card) else {
                report.skipped += 1;
                continue;
            };
            groups
                .entry(group.key)
                .or_insert_with(|| (group.topic, Vec::new()))
                .1
                .push(card);
            report.accepted += 1;
        }

        report.groups_considered = groups.len();
        let mut groups = groups.into_values().collect::<Vec<_>>();
        groups.sort_by(|left, right| {
            right
                .1
                .len()
                .cmp(&left.1.len())
                .then_with(|| left.0.cmp(&right.0))
        });
        for (topic, cards) in groups.into_iter().take(max_clusters) {
            if cards.len() < min_group_size {
                report.skipped += cards.len();
                continue;
            }
            let source_card_ids = cards.iter().map(|card| card.id.clone()).collect::<Vec<_>>();
            let group_metadata = knowledge_backlog_group_projection_metadata(
                &topic,
                &source_card_ids,
                &cards,
                max_source_cards,
                min_group_size,
                max_clusters,
            );
            let projection = self.project_knowledge_from_source_cards(
                &topic,
                cards,
                "source_card_backlog",
                "Local Proof",
                "source_card_backlog_clustering",
                Vec::new(),
                group_metadata,
            )?;
            report.projections.push(projection);
        }

        Ok(report)
    }

    pub fn project_knowledge_from_radar_run(
        &self,
        run_id: &str,
        topic: Option<&str>,
        max_source_cards: usize,
    ) -> Result<KnowledgeProjectionReport> {
        validate_id(run_id)?;
        let run = self
            .read_radar_run(run_id)?
            .with_context(|| format!("radar run not found: {run_id}"))?;
        let profile = self
            .read_radar_profile(&run.profile_id)?
            .with_context(|| format!("radar profile not found: {}", run.profile_id))?;
        let item_by_id = self
            .list_radar_items(run_id)?
            .into_iter()
            .map(|item| (item.id.clone(), item))
            .collect::<BTreeMap<_, _>>();
        let mut selected = self
            .list_radar_scores(run_id)?
            .into_iter()
            .filter(|score| score.status == "selected")
            .filter_map(|score| {
                item_by_id
                    .get(&score.item_id)
                    .cloned()
                    .map(|item| (score, item))
            })
            .collect::<Vec<_>>();
        selected.sort_by(|left, right| {
            right
                .0
                .score
                .partial_cmp(&left.0.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.1.id.cmp(&right.1.id))
        });
        let mut source_card_ids = selected
            .iter()
            .filter_map(|(_, item)| item.source_card_id.clone())
            .collect::<Vec<_>>();
        if source_card_ids.is_empty() {
            bail!("knowledge projection from radar run requires selected source-card evidence");
        }
        source_card_ids.sort();
        source_card_ids.dedup();
        let mut cards = Vec::new();
        for source_card_id in source_card_ids
            .into_iter()
            .take(max_source_cards.clamp(1, 50))
        {
            cards.push(
                self.read_source_card(&source_card_id)?
                    .with_context(|| format!("radar source card not found: {source_card_id}"))?,
            );
        }
        let topic = topic
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("Knowledge radar: {}", profile.name));
        let proof_level = run
            .metadata
            .get("proof_level")
            .and_then(Value::as_str)
            .unwrap_or("Local Proof");
        let source_family = run
            .metadata
            .get("source_family")
            .and_then(Value::as_str)
            .unwrap_or("radar_source_card_projection");
        let mut warnings = Vec::new();
        if run.status != "scored" {
            warnings.push(format!(
                "radar run status is {}; projection uses available selected source-card evidence only",
                run.status
            ));
        }
        if run
            .metadata
            .get("live_fetch_failed")
            .and_then(Value::as_bool)
            == Some(true)
        {
            warnings.push("radar run recorded live adapter failures".to_string());
        }
        self.project_knowledge_from_source_cards(
            &topic,
            cards,
            "radar_run",
            proof_level,
            source_family,
            warnings,
            json!({
                "radar_run_id": run.id,
                "radar_profile_id": profile.id,
                "radar_profile_name": profile.name,
                "radar_status": run.status,
                "selected_items": selected.len(),
                "max_source_cards": max_source_cards.clamp(1, 50),
            }),
        )
    }

    // allow: refactoring this N-arg signature is out of scope for the lint-cleanup pass.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn project_knowledge_from_source_cards(
        &self,
        topic: &str,
        source_cards: Vec<SourceCard>,
        origin: &str,
        proof_level: &str,
        source_family: &str,
        warnings: Vec<String>,
        metadata: Value,
    ) -> Result<KnowledgeProjectionReport> {
        validate_knowledge_text("knowledge projection topic", topic, 500)?;
        let mut source_cards = source_cards
            .into_iter()
            .map(|card| (card.id.clone(), card))
            .collect::<BTreeMap<_, _>>()
            .into_values()
            .collect::<Vec<_>>();
        source_cards.sort_by(|left, right| {
            right
                .retrieved_at
                .cmp(&left.retrieved_at)
                .then_with(|| left.id.cmp(&right.id))
        });
        if source_cards.is_empty() {
            bail!("knowledge projection requires at least one source card");
        }

        let mut events = Vec::new();
        let mut event_sources = Vec::new();
        let mut entities_by_id = BTreeMap::<String, KnowledgeEntity>::new();
        let mut relations_by_id = BTreeMap::<String, KnowledgeRelation>::new();
        for card in &source_cards {
            let event_input = knowledge_event_input_from_source_card(card).with_context(|| {
                format!(
                    "building knowledge event input from source card {} ({})",
                    card.id, card.title
                )
            })?;
            let event = self.upsert_knowledge_event(event_input).with_context(|| {
                format!(
                    "upserting knowledge event from source card {} title_len={} summary_len={}",
                    card.id,
                    card.title.len(),
                    card.summary.len()
                )
            })?;
            let event_source = self.add_knowledge_event_source(KnowledgeEventSourceInput {
                event_id: event.id.clone(),
                source_card_id: card.id.clone(),
                role: knowledge_source_role_for_card(card),
                confidence: knowledge_source_confidence_for_card(card),
                claim_summary: knowledge_claim_summary_for_card(card),
                metadata: json!({
                    "origin": origin,
                    "provider": card.provider,
                    "source_type": card.source_type,
                    "source_card_url": card.url,
                }),
            })?;
            let confirmed = self.confirm_knowledge_event(&event.id)?;
            let (card_entities, card_relations) =
                self.project_knowledge_entities_for_source_card(card, &confirmed)?;
            for entity in card_entities {
                entities_by_id.insert(entity.id.clone(), entity);
            }
            for relation in card_relations {
                relations_by_id.insert(relation.id.clone(), relation);
            }
            events.push(confirmed);
            event_sources.push(event_source);
        }

        events.sort_by(|left, right| left.id.cmp(&right.id));
        events.dedup_by(|left, right| left.id == right.id);
        let event_ids = events
            .iter()
            .map(|event| event.id.clone())
            .collect::<Vec<_>>();
        let source_card_ids = source_cards
            .iter()
            .map(|card| card.id.clone())
            .collect::<Vec<_>>();
        let first_seen_at = source_cards
            .iter()
            .map(|card| card.retrieved_at.clone())
            .min();
        let last_seen_at = source_cards
            .iter()
            .map(|card| card.retrieved_at.clone())
            .max();
        let provider_count = source_cards
            .iter()
            .map(|card| card.provider.clone())
            .collect::<BTreeSet<_>>()
            .len();
        let novelty_score =
            ((provider_count as f64 + source_cards.len() as f64) / 12.0).clamp(0.1, 1.0);
        let momentum_score = (source_cards.len() as f64 / 10.0).clamp(0.1, 1.0);
        let stale_score = source_cards
            .iter()
            .filter_map(|card| timestamp_age_hours(&card.retrieved_at))
            .min()
            .map(|hours| {
                if hours > 24 * 90 {
                    1.0
                } else if hours > 24 * 30 {
                    0.65
                } else if hours > 24 * 7 {
                    0.35
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0);
        let duplicate_groups = knowledge_duplicate_groups_for_cards(&source_cards);
        let cluster = self.create_knowledge_cluster(KnowledgeClusterInput {
            topic: topic.to_string(),
            status: "candidate".to_string(),
            event_ids,
            source_card_ids: source_card_ids.clone(),
            first_seen_at,
            last_seen_at,
            novelty_score,
            momentum_score,
            stale_score,
            reason: format!(
                "Projected {} source cards from {origin} into a unified source-backed knowledge cluster.",
                source_cards.len()
            ),
            duplicate_groups,
            metadata: json!({
                "origin": origin,
                "proof_level": proof_level,
                "source_family": source_family,
                "provider_count": provider_count,
                "projection": "knowledge_source_card_projection_v1",
                "source_metadata": metadata,
            }),
        })?;
        let cluster_relations =
            self.project_knowledge_cluster_relations(&cluster, &source_cards, &entities_by_id)?;
        for relation in cluster_relations {
            relations_by_id.insert(relation.id.clone(), relation);
        }
        let editorial_decision =
            self.record_knowledge_editorial_decision(KnowledgeEditorialDecisionInput {
                cluster_id: cluster.id.clone(),
                decision: "create_human_report".to_string(),
                status: "completed".to_string(),
                wiki_page_id: None,
                digest_candidate_id: None,
                source_card_ids: cluster.source_card_ids.clone(),
                reason: format!(
                    "Created a working knowledge note for `{}` from {} linked sources.",
                    cluster.topic,
                    cluster.source_card_ids.len()
                ),
                quality_findings: Vec::new(),
                metadata: json!({
                    "origin": origin,
                    "proof_level": proof_level,
                    "source_family": source_family,
                }),
            })?;
        let report_body = render_knowledge_projection_report(
            &cluster,
            &source_cards,
            proof_level,
            source_family,
            &warnings,
        );
        let report = self.record_knowledge_report(KnowledgeReportInput {
            cluster_id: cluster.id.clone(),
            title: cluster.topic.clone(),
            body_markdown: report_body,
            status: "draft".to_string(),
            source_card_ids: cluster.source_card_ids.clone(),
            metadata: json!({
                "origin": origin,
                "proof_level": proof_level,
                "source_family": source_family,
                "reporter": "deterministic_source_card_projection_v1",
            }),
        })?;

        Ok(KnowledgeProjectionReport {
            topic: topic.to_string(),
            proof_level: proof_level.to_string(),
            source_family: source_family.to_string(),
            source_cards,
            events,
            event_sources,
            entities: entities_by_id.into_values().collect(),
            relations: relations_by_id.into_values().collect(),
            cluster,
            editorial_decision,
            report,
            warnings,
            metadata,
        })
    }

    pub(crate) fn project_knowledge_entities_for_source_card(
        &self,
        card: &SourceCard,
        event: &KnowledgeEvent,
    ) -> Result<(Vec<KnowledgeEntity>, Vec<KnowledgeRelation>)> {
        let inputs = knowledge_entity_inputs_for_card(card);
        let mut entities = Vec::new();
        let mut by_key = BTreeMap::new();
        for input in inputs {
            let input_key = input.canonical_key.clone();
            let entity = self.upsert_knowledge_entity(input)?;
            by_key.insert(input_key, entity.clone());
            by_key.insert(entity.canonical_key.clone(), entity.clone());
            entities.push(entity);
        }

        let mut relations = Vec::new();
        if let Some(primary_key) = knowledge_projected_primary_entity_key_for_card(card)
            && let (Some(primary), Some(provider)) = (
                by_key.get(&primary_key),
                by_key.get(&knowledge_provider_entity_key(card)),
            )
            && primary.id != provider.id
        {
            relations.push(self.upsert_knowledge_relation(KnowledgeRelationInput {
                relation_type: "reported_by_provider".to_string(),
                subject_entity_id: primary.id.clone(),
                object_entity_id: provider.id.clone(),
                event_id: Some(event.id.clone()),
                cluster_id: None,
                source_card_ids: vec![card.id.clone()],
                confidence: knowledge_source_confidence_for_card(card),
                reason: format!(
                    "Source card `{}` says `{}` is evidence from provider `{}`.",
                    card.id, primary.name, card.provider
                ),
                metadata: json!({
                    "source_card_url": card.url,
                    "provider": card.provider,
                    "source_type": card.source_type,
                }),
            })?);
        }
        if let (Some(owner), Some(repo)) = (
            knowledge_github_owner_key(card).and_then(|key| by_key.get(&key)),
            knowledge_github_repo_key(card).and_then(|key| by_key.get(&key)),
        ) && owner.id != repo.id
        {
            relations.push(self.upsert_knowledge_relation(KnowledgeRelationInput {
                relation_type: "owns_repo".to_string(),
                subject_entity_id: owner.id.clone(),
                object_entity_id: repo.id.clone(),
                event_id: Some(event.id.clone()),
                cluster_id: None,
                source_card_ids: vec![card.id.clone()],
                confidence: 0.9_f64.min(knowledge_source_confidence_for_card(card)),
                reason: format!(
                    "GitHub source-card metadata links owner `{}` to repo `{}`.",
                    owner.name, repo.name
                ),
                metadata: json!({
                    "source_card_url": card.url,
                    "provider": card.provider,
                    "source_type": card.source_type,
                }),
            })?);
        }
        Ok((entities, relations))
    }

    pub(crate) fn project_knowledge_cluster_relations(
        &self,
        cluster: &KnowledgeCluster,
        source_cards: &[SourceCard],
        entities_by_id: &BTreeMap<String, KnowledgeEntity>,
    ) -> Result<Vec<KnowledgeRelation>> {
        let mut entities_by_key = entities_by_id
            .values()
            .map(|entity| (entity.canonical_key.clone(), entity.clone()))
            .collect::<BTreeMap<_, _>>();
        for card in source_cards {
            if let Some(primary_key) = knowledge_projected_primary_entity_key_for_card(card)
                && !entities_by_key.contains_key(&primary_key)
                && let Some(entity) = self.get_knowledge_entity_by_canonical_key(&primary_key)?
            {
                entities_by_key.insert(primary_key, entity);
            }
        }
        let mut by_primary = BTreeMap::<String, (KnowledgeEntity, BTreeSet<String>)>::new();
        for card in source_cards {
            if let Some(primary_key) = knowledge_projected_primary_entity_key_for_card(card)
                && let Some(entity) = entities_by_key.get(&primary_key)
            {
                by_primary
                    .entry(entity.id.clone())
                    .or_insert_with(|| (entity.clone(), BTreeSet::new()))
                    .1
                    .insert(card.id.clone());
            }
        }
        let primary = by_primary.into_values().collect::<Vec<_>>();
        let mut relations = Vec::new();
        for left_index in 0..primary.len() {
            for right_index in (left_index + 1)..primary.len() {
                let (left, left_sources) = &primary[left_index];
                let (right, right_sources) = &primary[right_index];
                let (subject, object) = if left.id <= right.id {
                    (left, right)
                } else {
                    (right, left)
                };
                let source_card_ids = left_sources
                    .union(right_sources)
                    .cloned()
                    .collect::<Vec<_>>();
                relations.push(self.upsert_knowledge_relation(KnowledgeRelationInput {
                    relation_type: "co_clustered_with".to_string(),
                    subject_entity_id: subject.id.clone(),
                    object_entity_id: object.id.clone(),
                    event_id: None,
                    cluster_id: Some(cluster.id.clone()),
                    source_card_ids,
                    confidence: 0.6,
                    reason: format!(
                        "`{}` and `{}` appear in the same source-card-backed knowledge cluster `{}`.",
                        subject.name, object.name, cluster.topic
                    ),
                    metadata: json!({
                        "cluster_topic": cluster.topic,
                        "relation_scope": "deterministic_cluster_cooccurrence",
                    }),
                })?);
            }
        }
        Ok(relations)
    }

    pub(crate) fn normalize_knowledge_source_card_ids(
        &self,
        ids: &[String],
    ) -> Result<Vec<String>> {
        let ids = ids
            .iter()
            .map(|id| id.trim().to_string())
            .filter(|id| !id.is_empty())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        if ids.is_empty() {
            bail!("knowledge item requires source-card evidence");
        }
        for id in &ids {
            validate_id(id)?;
            self.read_source_card(id)?
                .with_context(|| format!("source card not found: {id}"))?;
        }
        Ok(ids)
    }

    pub(crate) fn read_knowledge_source_cards(&self, ids: &[String]) -> Result<Vec<SourceCard>> {
        let source_card_ids = self.normalize_knowledge_source_card_ids(ids)?;
        let mut cards = Vec::new();
        for source_card_id in source_card_ids {
            cards.push(
                self.read_source_card(&source_card_id)?
                    .with_context(|| format!("source card not found: {source_card_id}"))?,
            );
        }
        Ok(cards)
    }

    pub(crate) fn knowledge_clustered_source_card_ids(&self) -> Result<BTreeSet<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT source_card_ids_json FROM knowledge_clusters")?;
        let mut rows = stmt.query([])?;
        let mut ids = BTreeSet::new();
        while let Some(row) = rows.next()? {
            let raw: String = row.get(0)?;
            for id in serde_json::from_str::<Vec<String>>(&raw).unwrap_or_default() {
                ids.insert(id);
            }
        }
        Ok(ids)
    }

    pub(crate) fn normalize_knowledge_cluster_model_input(
        &self,
        input: KnowledgeClusterProposalModelInput,
    ) -> Result<KnowledgeClusterProposalModelInput> {
        let source_card_ids = self.normalize_knowledge_source_card_ids(&input.source_card_ids)?;
        let model_provider = input.model_provider.trim().to_ascii_lowercase();
        if !matches!(model_provider.as_str(), "mock" | "openai") {
            bail!("unsupported knowledge cluster proposal model provider: {model_provider}");
        }
        let model_name = input
            .model_name
            .map(|model| model.trim().to_string())
            .filter(|model| !model.is_empty());
        if let Some(model_name) = &model_name {
            validate_key(model_name)?;
        }
        if let Some(endpoint) = &input.endpoint {
            validated_endpoint(Some(endpoint), "https://api.openai.com/v1/responses")?;
        }
        Ok(KnowledgeClusterProposalModelInput {
            source_card_ids,
            model_provider,
            model_name,
            endpoint: input.endpoint,
            timeout_seconds: input.timeout_seconds,
            max_clusters: input.max_clusters.clamp(1, 12),
        })
    }

    pub(crate) fn ensure_knowledge_events_for_source_cards(
        &self,
        source_cards: &[SourceCard],
        origin: &str,
    ) -> Result<Vec<KnowledgeEvent>> {
        let mut events = Vec::new();
        for card in source_cards {
            let event =
                self.upsert_knowledge_event(knowledge_event_input_from_source_card(card)?)?;
            self.add_knowledge_event_source(KnowledgeEventSourceInput {
                event_id: event.id.clone(),
                source_card_id: card.id.clone(),
                role: knowledge_source_role_for_card(card),
                confidence: knowledge_source_confidence_for_card(card),
                claim_summary: knowledge_claim_summary_for_card(card),
                metadata: json!({
                    "origin": origin,
                    "provider": card.provider,
                    "source_type": card.source_type,
                    "source_card_url": card.url,
                }),
            })?;
            events.push(self.confirm_knowledge_event(&event.id)?);
        }
        events.sort_by(|left, right| left.id.cmp(&right.id));
        events.dedup_by(|left, right| left.id == right.id);
        Ok(events)
    }

    pub(crate) fn normalize_knowledge_entity_input(
        &self,
        input: KnowledgeEntityInput,
    ) -> Result<KnowledgeEntityInput> {
        let aliases = normalize_knowledge_aliases(&input.aliases, Some(&input.name));
        let source_card_ids = self.normalize_knowledge_source_card_ids(&input.source_card_ids)?;
        let normalized = KnowledgeEntityInput {
            entity_type: input.entity_type.trim().to_string(),
            name: input.name.trim().to_string(),
            canonical_key: input.canonical_key.trim().to_string(),
            aliases,
            homepage_url: input
                .homepage_url
                .map(|url| url.trim().to_string())
                .filter(|url| !url.is_empty()),
            source_card_ids,
            wiki_page_id: input
                .wiki_page_id
                .map(|id| id.trim().to_string())
                .filter(|id| !id.is_empty()),
            confidence: input.confidence,
            metadata: input.metadata,
        };
        validate_knowledge_entity_input(&normalized)?;
        Ok(normalized)
    }

    pub(crate) fn ensure_knowledge_entity_aliases_available(
        &self,
        canonical_key: &str,
        aliases: &[String],
        ignore_entity_id: Option<&str>,
    ) -> Result<()> {
        let aliases = normalize_knowledge_aliases(aliases, None);
        if aliases.is_empty() {
            return Ok(());
        }
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, entity_type, name, canonical_key, aliases_json, homepage_url,
                   source_card_ids_json, wiki_page_id, confidence, metadata_json,
                   created_at, updated_at
            FROM knowledge_entities
            "#,
        )?;
        let existing = rows(stmt.query_map([], knowledge_entity_from_row)?)?;
        let wanted = aliases
            .iter()
            .map(|alias| normalize_knowledge_alias_key(alias))
            .collect::<BTreeSet<_>>();
        for entity in existing {
            if entity.canonical_key == canonical_key {
                continue;
            }
            if ignore_entity_id == Some(entity.id.as_str()) {
                continue;
            }
            let mut entity_aliases =
                normalize_knowledge_aliases(&entity.aliases, Some(&entity.name));
            entity_aliases.push(entity.canonical_key.clone());
            let entity_keys = entity_aliases
                .iter()
                .map(|alias| normalize_knowledge_alias_key(alias))
                .collect::<BTreeSet<_>>();
            if let Some(conflict) = wanted.intersection(&entity_keys).next() {
                bail!(
                    "knowledge entity alias collision requires review: `{}` already belongs to `{}`",
                    conflict,
                    entity.canonical_key
                );
            }
        }
        Ok(())
    }

    pub(crate) fn normalize_knowledge_relation_input(
        &self,
        input: KnowledgeRelationInput,
    ) -> Result<KnowledgeRelationInput> {
        let source_card_ids = self.normalize_knowledge_source_card_ids(&input.source_card_ids)?;
        let normalized = KnowledgeRelationInput {
            relation_type: input.relation_type.trim().to_string(),
            subject_entity_id: input.subject_entity_id.trim().to_string(),
            object_entity_id: input.object_entity_id.trim().to_string(),
            event_id: input
                .event_id
                .map(|id| id.trim().to_string())
                .filter(|id| !id.is_empty()),
            cluster_id: input
                .cluster_id
                .map(|id| id.trim().to_string())
                .filter(|id| !id.is_empty()),
            source_card_ids,
            confidence: input.confidence,
            reason: input.reason.trim().to_string(),
            metadata: input.metadata,
        };
        validate_knowledge_relation_input(&normalized)?;
        self.get_knowledge_entity(&normalized.subject_entity_id)?
            .with_context(|| {
                format!(
                    "knowledge relation subject entity not found: {}",
                    normalized.subject_entity_id
                )
            })?;
        self.get_knowledge_entity(&normalized.object_entity_id)?
            .with_context(|| {
                format!(
                    "knowledge relation object entity not found: {}",
                    normalized.object_entity_id
                )
            })?;
        if let Some(event_id) = &normalized.event_id {
            self.get_knowledge_event(event_id)?
                .with_context(|| format!("knowledge relation event not found: {event_id}"))?;
        }
        if let Some(cluster_id) = &normalized.cluster_id {
            self.get_knowledge_cluster(cluster_id)?
                .with_context(|| format!("knowledge relation cluster not found: {cluster_id}"))?;
        }
        Ok(normalized)
    }

    pub(crate) fn normalize_knowledge_event_ids(&self, ids: &[String]) -> Result<Vec<String>> {
        let ids = ids
            .iter()
            .map(|id| id.trim().to_string())
            .filter(|id| !id.is_empty())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        for id in &ids {
            validate_id(id)?;
            self.get_knowledge_event(id)?
                .with_context(|| format!("knowledge event not found: {id}"))?;
        }
        Ok(ids)
    }

    pub(crate) fn ensure_knowledge_cluster_event_evidence(
        &self,
        event_ids: &[String],
        source_card_ids: &[String],
    ) -> Result<()> {
        if event_ids.is_empty() {
            return Ok(());
        }
        let cluster_source_ids = source_card_ids.iter().collect::<BTreeSet<_>>();
        for event_id in event_ids {
            let mut stmt = self.conn.prepare(
                r#"
                SELECT event_source.source_card_id
                FROM knowledge_event_sources event_source
                JOIN source_cards source_card ON source_card.id = event_source.source_card_id
                WHERE event_source.event_id = ?1
                "#,
            )?;
            let linked_source_ids =
                rows(stmt.query_map(params![event_id], |row| row.get::<_, String>(0))?)?;
            if !linked_source_ids
                .iter()
                .any(|source_card_id| cluster_source_ids.contains(source_card_id))
            {
                bail!(
                    "knowledge cluster event {event_id} has no live source-card evidence in the cluster"
                );
            }
        }
        Ok(())
    }
}
