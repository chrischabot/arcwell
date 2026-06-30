use super::*;

impl Store {
    pub fn create_x_knowledge_clusters_from_radar_run(
        &self,
        run_id: &str,
        max_source_cards: usize,
    ) -> Result<Vec<XKnowledgeCluster>> {
        validate_id(run_id)?;
        let run = self
            .read_radar_run(run_id)?
            .with_context(|| format!("radar run not found: {run_id}"))?;
        let items_by_id = self
            .list_radar_items(run_id)?
            .into_iter()
            .map(|item| (item.id.clone(), item))
            .collect::<BTreeMap<_, _>>();
        let mut selected = Vec::new();
        for score in self.list_radar_scores(run_id)? {
            if score.status != "selected" {
                continue;
            }
            let Some(item) = items_by_id.get(&score.item_id).cloned() else {
                continue;
            };
            if item.source_card_id.is_none() {
                continue;
            }
            selected.push((score, item));
        }
        selected.sort_by(|left, right| {
            right
                .0
                .score
                .partial_cmp(&left.0.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.1.id.cmp(&right.1.id))
        });
        let selected = selected
            .into_iter()
            .take(max_source_cards.clamp(1, 50))
            .collect::<Vec<_>>();
        if selected.is_empty() {
            bail!("x knowledge clustering requires selected radar items with source-card ids");
        }
        let mut grouped = BTreeMap::<String, Vec<(RadarScore, RadarItem)>>::new();
        for entry in selected {
            let key = x_knowledge_cluster_key(&entry.1);
            grouped.entry(key).or_default().push(entry);
        }
        let mut clusters = Vec::new();
        for (cluster_key, selected) in grouped {
            let mut source_card_ids = selected
                .iter()
                .filter_map(|(_, item)| item.source_card_id.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            source_card_ids.sort();
            if source_card_ids.is_empty() {
                continue;
            }
            let radar_item_ids = selected
                .iter()
                .map(|(_, item)| item.id.clone())
                .collect::<Vec<_>>();
            let first_seen_at = selected
                .iter()
                .filter_map(|(_, item)| {
                    item.published_at
                        .as_deref()
                        .or(Some(item.created_at.as_str()))
                })
                .min()
                .unwrap_or(run.started_at.as_str())
                .to_string();
            let last_seen_at = selected
                .iter()
                .filter_map(|(_, item)| {
                    item.published_at
                        .as_deref()
                        .or(Some(item.created_at.as_str()))
                })
                .max()
                .unwrap_or(run.updated_at.as_str())
                .to_string();
            let topic = x_knowledge_cluster_topic(&cluster_key, &selected);
            let novelty_score = selected
                .iter()
                .map(|(score, _)| score.score)
                .fold(0.0_f64, f64::max)
                .clamp(0.0, 10.0)
                / 10.0;
            let momentum_score = (source_card_ids.len() as f64 / 10.0).clamp(0.0, 1.0);
            let stale_score = x_knowledge_stale_score(&last_seen_at);
            let reason = format!(
                "{} selected radar items in `{cluster_key}` with source-card evidence from run {run_id}; top score {:.2}",
                selected.len(),
                selected
                    .first()
                    .map(|(score, _)| score.score)
                    .unwrap_or_default()
            );
            let timestamp = now();
            let source_card_ids_json = serde_json::to_string(&source_card_ids)?;
            let radar_item_ids_json = serde_json::to_string(&radar_item_ids)?;
            let id = format!(
                "xkc-{}",
                &sha256(format!("{topic}:{source_card_ids_json}").as_bytes())[..16]
            );
            let metadata = json!({
                "proof_level": "Local Proof: deterministic source-card-backed radar cluster",
                "source": "radar_selected_items",
                "cluster_key": cluster_key,
                "clusterer": "deterministic_keyword_bucket_v1",
                "duplicate_groups": x_knowledge_duplicate_groups(&selected),
                "radar_status": run.status,
                "radar_stage": run.stage,
            });
            self.conn.execute(
                r#"
                INSERT INTO x_knowledge_clusters
                  (id, topic, status, source_card_ids_json, radar_run_id, radar_item_ids_json,
                   first_seen_at, last_seen_at, novelty_score, momentum_score, stale_score,
                   reason, metadata_json, created_at, updated_at)
                VALUES (?1, ?2, 'candidate', ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?13)
                ON CONFLICT(topic, source_card_ids_json) DO UPDATE SET
                  radar_run_id = excluded.radar_run_id,
                  radar_item_ids_json = excluded.radar_item_ids_json,
                  last_seen_at = excluded.last_seen_at,
                  novelty_score = excluded.novelty_score,
                  momentum_score = excluded.momentum_score,
                  stale_score = excluded.stale_score,
                  reason = excluded.reason,
                  metadata_json = excluded.metadata_json,
                  updated_at = excluded.updated_at
                "#,
                params![
                    id,
                    topic,
                    source_card_ids_json,
                    run_id,
                    radar_item_ids_json,
                    first_seen_at,
                    last_seen_at,
                    novelty_score,
                    momentum_score,
                    stale_score,
                    reason,
                    metadata.to_string(),
                    timestamp,
                ],
            )?;
            clusters.push(
                self.get_x_knowledge_cluster(&id)?
                    .with_context(|| format!("inserted x knowledge cluster not found: {id}"))?,
            );
        }
        if clusters.is_empty() {
            bail!("x knowledge clustering found no source-card-backed clusters");
        }
        clusters.sort_by(|left, right| {
            right
                .novelty_score
                .partial_cmp(&left.novelty_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| right.momentum_score.total_cmp(&left.momentum_score))
                .then_with(|| left.topic.cmp(&right.topic))
        });
        Ok(clusters)
    }

    pub fn get_x_knowledge_cluster(&self, id: &str) -> Result<Option<XKnowledgeCluster>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, topic, status, source_card_ids_json, radar_run_id,
                       radar_item_ids_json, first_seen_at, last_seen_at,
                       novelty_score, momentum_score, stale_score, reason,
                       metadata_json, created_at, updated_at
                FROM x_knowledge_clusters
                WHERE id = ?1
                "#,
                params![id],
                x_knowledge_cluster_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_x_knowledge_clusters(&self, limit: usize) -> Result<Vec<XKnowledgeCluster>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, topic, status, source_card_ids_json, radar_run_id,
                   radar_item_ids_json, first_seen_at, last_seen_at,
                   novelty_score, momentum_score, stale_score, reason,
                   metadata_json, created_at, updated_at
            FROM x_knowledge_clusters
            ORDER BY updated_at DESC
            LIMIT ?1
            "#,
        )?;
        rows(stmt.query_map(params![limit.clamp(1, 500)], x_knowledge_cluster_from_row)?)
    }

    pub fn run_x_editorial_decision_for_cluster(
        &self,
        cluster_id: &str,
    ) -> Result<XEditorialDecision> {
        let cluster = self
            .get_x_knowledge_cluster(cluster_id)?
            .with_context(|| format!("x knowledge cluster not found: {cluster_id}"))?;
        if cluster.source_card_ids.is_empty() {
            bail!("x editorial decision requires source-card evidence");
        }
        let mut cards = Vec::new();
        for source_card_id in &cluster.source_card_ids {
            cards.push(
                self.read_source_card(source_card_id)?
                    .with_context(|| format!("cluster source card not found: {source_card_id}"))?,
            );
        }
        let markdown = render_x_cluster_wiki_page(&cluster, &cards)?;
        let quality_findings = audit_x_cluster_wiki_page(&cluster, &markdown);
        if !quality_findings.is_empty() {
            bail!(
                "x editorial decision quality gate failed: {}",
                quality_findings.join("; ")
            );
        }
        let wiki_page_id = self.add_wiki_page(
            &format!("X Knowledge: {}", cluster.topic),
            &markdown,
            "x-knowledge-editor",
        )?;
        let digest = self.create_digest_candidate(&cluster.topic, &cluster.source_card_ids)?;
        let id = format!(
            "xed-{}",
            &sha256(format!("{}:expand_and_digest", cluster.id).as_bytes())[..16]
        );
        let timestamp = now();
        let reason = format!(
            "Expanded cluster {} into wiki page {} and created digest candidate {} from {} source cards.",
            cluster.id,
            wiki_page_id,
            digest.id,
            cluster.source_card_ids.len()
        );
        self.conn.execute(
            r#"
            INSERT INTO x_editorial_decisions
              (id, cluster_id, decision, status, wiki_page_id, digest_candidate_id,
               source_card_ids_json, reason, quality_findings_json, metadata_json,
               created_at, updated_at)
            VALUES (?1, ?2, 'expand_and_digest_candidate', 'completed', ?3, ?4, ?5, ?6, '[]', ?7, ?8, ?8)
            ON CONFLICT(cluster_id, decision) DO UPDATE SET
              status = excluded.status,
              wiki_page_id = excluded.wiki_page_id,
              digest_candidate_id = excluded.digest_candidate_id,
              source_card_ids_json = excluded.source_card_ids_json,
              reason = excluded.reason,
              quality_findings_json = excluded.quality_findings_json,
              metadata_json = excluded.metadata_json,
              updated_at = excluded.updated_at
            "#,
            params![
                id,
                cluster.id,
                wiki_page_id,
                digest.id,
                serde_json::to_string(&cluster.source_card_ids)?,
                reason,
                json!({
                    "proof_level": "Local Proof: deterministic source-card-backed editorial decision",
                    "cluster_topic": cluster.topic,
                })
                .to_string(),
                timestamp,
            ],
        )?;
        self.get_x_editorial_decision(&id)?
            .with_context(|| format!("inserted x editorial decision not found: {id}"))
    }

    pub fn get_x_editorial_decision(&self, id: &str) -> Result<Option<XEditorialDecision>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, cluster_id, decision, status, wiki_page_id, digest_candidate_id,
                       source_card_ids_json, reason, quality_findings_json, metadata_json,
                       created_at, updated_at
                FROM x_editorial_decisions
                WHERE id = ?1
                "#,
                params![id],
                x_editorial_decision_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_x_editorial_decisions(&self, limit: usize) -> Result<Vec<XEditorialDecision>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, cluster_id, decision, status, wiki_page_id, digest_candidate_id,
                   source_card_ids_json, reason, quality_findings_json, metadata_json,
                   created_at, updated_at
            FROM x_editorial_decisions
            ORDER BY updated_at DESC
            LIMIT ?1
            "#,
        )?;
        rows(stmt.query_map(params![limit.clamp(1, 500)], x_editorial_decision_from_row)?)
    }

    pub fn upsert_knowledge_entity(&self, input: KnowledgeEntityInput) -> Result<KnowledgeEntity> {
        let input = self.normalize_knowledge_entity_input(input)?;
        let id = format!("kent-{}", &sha256(input.canonical_key.as_bytes())[..16]);
        let timestamp = now();
        let existing = self.get_knowledge_entity_by_canonical_key(&input.canonical_key)?;
        let aliases = existing
            .as_ref()
            .map(|entity| merge_string_sets(&entity.aliases, &input.aliases))
            .unwrap_or_else(|| input.aliases.clone());
        let source_card_ids = existing
            .as_ref()
            .map(|entity| merge_string_sets(&entity.source_card_ids, &input.source_card_ids))
            .unwrap_or_else(|| input.source_card_ids.clone());
        let confidence = existing
            .as_ref()
            .map(|entity| entity.confidence.max(input.confidence))
            .unwrap_or(input.confidence);
        self.ensure_knowledge_entity_aliases_available(&input.canonical_key, &aliases)?;
        self.conn.execute(
            r#"
            INSERT INTO knowledge_entities
              (id, entity_type, name, canonical_key, aliases_json, homepage_url,
               source_card_ids_json, wiki_page_id, confidence, metadata_json, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)
            ON CONFLICT(canonical_key) DO UPDATE SET
              entity_type = excluded.entity_type,
              name = excluded.name,
              aliases_json = excluded.aliases_json,
              homepage_url = COALESCE(excluded.homepage_url, knowledge_entities.homepage_url),
              source_card_ids_json = excluded.source_card_ids_json,
              wiki_page_id = COALESCE(excluded.wiki_page_id, knowledge_entities.wiki_page_id),
              confidence = MAX(knowledge_entities.confidence, excluded.confidence),
              metadata_json = excluded.metadata_json,
              updated_at = excluded.updated_at
            "#,
            params![
                id,
                input.entity_type,
                input.name,
                input.canonical_key,
                serde_json::to_string(&aliases)?,
                input.homepage_url,
                serde_json::to_string(&source_card_ids)?,
                input.wiki_page_id,
                confidence,
                input.metadata.to_string(),
                timestamp,
            ],
        )?;
        self.get_knowledge_entity(&id)?
            .with_context(|| format!("inserted knowledge entity not found: {id}"))
    }

    pub fn get_knowledge_entity(&self, id: &str) -> Result<Option<KnowledgeEntity>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, entity_type, name, canonical_key, aliases_json, homepage_url,
                       source_card_ids_json, wiki_page_id, confidence, metadata_json,
                       created_at, updated_at
                FROM knowledge_entities
                WHERE id = ?1
                "#,
                params![id],
                knowledge_entity_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn get_knowledge_entity_by_canonical_key(
        &self,
        canonical_key: &str,
    ) -> Result<Option<KnowledgeEntity>> {
        validate_knowledge_text("knowledge entity canonical key", canonical_key, 500)?;
        self.conn
            .query_row(
                r#"
                SELECT id, entity_type, name, canonical_key, aliases_json, homepage_url,
                       source_card_ids_json, wiki_page_id, confidence, metadata_json,
                       created_at, updated_at
                FROM knowledge_entities
                WHERE canonical_key = ?1
                "#,
                params![canonical_key],
                knowledge_entity_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_knowledge_entities(&self, limit: usize) -> Result<Vec<KnowledgeEntity>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, entity_type, name, canonical_key, aliases_json, homepage_url,
                   source_card_ids_json, wiki_page_id, confidence, metadata_json,
                   created_at, updated_at
            FROM knowledge_entities
            ORDER BY updated_at DESC
            LIMIT ?1
            "#,
        )?;
        rows(stmt.query_map(params![limit.clamp(1, 500)], knowledge_entity_from_row)?)
    }

    pub fn upsert_knowledge_relation(
        &self,
        input: KnowledgeRelationInput,
    ) -> Result<KnowledgeRelation> {
        let input = self.normalize_knowledge_relation_input(input)?;
        let relation_key = knowledge_relation_key(&input);
        let id = format!("krel-{}", &sha256(relation_key.as_bytes())[..16]);
        let existing = self.get_knowledge_relation_by_key(&relation_key)?;
        let source_card_ids = existing
            .as_ref()
            .map(|relation| merge_string_sets(&relation.source_card_ids, &input.source_card_ids))
            .unwrap_or_else(|| input.source_card_ids.clone());
        let confidence = existing
            .as_ref()
            .map(|relation| relation.confidence.max(input.confidence))
            .unwrap_or(input.confidence);
        let timestamp = now();
        self.conn.execute(
            r#"
            INSERT INTO knowledge_relations
              (id, relation_key, relation_type, subject_entity_id, object_entity_id,
               event_id, cluster_id, source_card_ids_json, confidence, reason,
               metadata_json, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?12)
            ON CONFLICT(relation_key) DO UPDATE SET
              event_id = COALESCE(excluded.event_id, knowledge_relations.event_id),
              cluster_id = COALESCE(excluded.cluster_id, knowledge_relations.cluster_id),
              source_card_ids_json = excluded.source_card_ids_json,
              confidence = MAX(knowledge_relations.confidence, excluded.confidence),
              reason = excluded.reason,
              metadata_json = excluded.metadata_json,
              updated_at = excluded.updated_at
            "#,
            params![
                id,
                relation_key,
                input.relation_type,
                input.subject_entity_id,
                input.object_entity_id,
                input.event_id,
                input.cluster_id,
                serde_json::to_string(&source_card_ids)?,
                confidence,
                input.reason,
                input.metadata.to_string(),
                timestamp,
            ],
        )?;
        self.get_knowledge_relation(&id)?
            .with_context(|| format!("inserted knowledge relation not found: {id}"))
    }

    pub fn get_knowledge_relation(&self, id: &str) -> Result<Option<KnowledgeRelation>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, relation_key, relation_type, subject_entity_id, object_entity_id,
                       event_id, cluster_id, source_card_ids_json, confidence, reason,
                       metadata_json, created_at, updated_at
                FROM knowledge_relations
                WHERE id = ?1
                "#,
                params![id],
                knowledge_relation_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn get_knowledge_relation_by_key(
        &self,
        relation_key: &str,
    ) -> Result<Option<KnowledgeRelation>> {
        validate_knowledge_text("knowledge relation key", relation_key, 1_000)?;
        self.conn
            .query_row(
                r#"
                SELECT id, relation_key, relation_type, subject_entity_id, object_entity_id,
                       event_id, cluster_id, source_card_ids_json, confidence, reason,
                       metadata_json, created_at, updated_at
                FROM knowledge_relations
                WHERE relation_key = ?1
                "#,
                params![relation_key],
                knowledge_relation_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_knowledge_relations(&self, limit: usize) -> Result<Vec<KnowledgeRelation>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, relation_key, relation_type, subject_entity_id, object_entity_id,
                   event_id, cluster_id, source_card_ids_json, confidence, reason,
                   metadata_json, created_at, updated_at
            FROM knowledge_relations
            ORDER BY updated_at DESC
            LIMIT ?1
            "#,
        )?;
        rows(stmt.query_map(params![limit.clamp(1, 500)], knowledge_relation_from_row)?)
    }

    pub fn list_knowledge_adapter_runs(&self, limit: usize) -> Result<Vec<KnowledgeAdapterRun>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, job_id, adapter_kind, provider, source_kind, locator, status,
                   error_kind, error, cursor_key, cursor_before, cursor_after,
                   source_card_ids_json, raw_count, accepted_count, rejected_count,
                   duplicate_count, metadata_json, created_at, updated_at
            FROM knowledge_adapter_runs
            ORDER BY updated_at DESC
            LIMIT ?1
            "#,
        )?;
        rows(stmt.query_map(params![limit.clamp(1, 500)], knowledge_adapter_run_from_row)?)
    }

    pub fn list_knowledge_entity_resolutions(
        &self,
        limit: usize,
    ) -> Result<Vec<KnowledgeEntityResolution>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, left_entity_id, right_entity_id, status, decision, confidence,
                   resolver, reason, evidence_json, source_card_ids_json, created_at, updated_at
            FROM knowledge_entity_resolutions
            ORDER BY updated_at DESC
            LIMIT ?1
            "#,
        )?;
        rows(stmt.query_map(
            params![limit.clamp(1, 500)],
            knowledge_entity_resolution_from_row,
        )?)
    }

    pub fn propose_knowledge_entity_resolutions(
        &self,
        limit: usize,
    ) -> Result<Vec<KnowledgeEntityResolution>> {
        let entities = self.list_knowledge_entities(500)?;
        let mut proposals = Vec::new();
        'outer: for left_index in 0..entities.len() {
            for right in entities.iter().skip(left_index + 1) {
                let left = &entities[left_index];
                if let Some(input) = knowledge_entity_resolution_proposal(left, right) {
                    proposals.push(self.upsert_knowledge_entity_resolution(input)?);
                    if proposals.len() >= limit.clamp(1, 500) {
                        break 'outer;
                    }
                }
            }
        }
        Ok(proposals)
    }

    pub fn record_model_knowledge_entity_resolution(
        &self,
        left_entity_id: &str,
        right_entity_id: &str,
        decision: &str,
        confidence: f64,
        reason: &str,
        evidence_json: Value,
        source_card_ids: Vec<String>,
        resolver: Option<&str>,
    ) -> Result<KnowledgeEntityResolution> {
        validate_id(left_entity_id)?;
        validate_id(right_entity_id)?;
        if left_entity_id == right_entity_id {
            bail!("knowledge entity resolution requires two different entities");
        }
        self.get_knowledge_entity(left_entity_id)?
            .with_context(|| format!("left knowledge entity not found: {left_entity_id}"))?;
        self.get_knowledge_entity(right_entity_id)?
            .with_context(|| format!("right knowledge entity not found: {right_entity_id}"))?;
        let source_card_ids = self.normalize_knowledge_source_card_ids(&source_card_ids)?;
        let decision = validate_knowledge_entity_resolution_decision(decision)?;
        if matches!(decision.as_str(), "same_as_candidate" | "merge_candidate")
            && source_card_ids.is_empty()
        {
            bail!("model entity resolution cannot propose sameness without source-card evidence");
        }
        self.upsert_knowledge_entity_resolution(KnowledgeEntityResolutionInput {
            left_entity_id: left_entity_id.to_string(),
            right_entity_id: right_entity_id.to_string(),
            status: "pending_review".to_string(),
            decision,
            confidence,
            resolver: resolver
                .unwrap_or("model-schema-gated-v1")
                .trim()
                .to_string(),
            reason: reason.trim().to_string(),
            evidence_json,
            source_card_ids,
        })
    }

    pub fn invoke_knowledge_entity_resolution_model(
        &self,
        input: KnowledgeEntityResolutionModelInput,
    ) -> Result<KnowledgeEntityResolutionModelInvocation> {
        let input = normalize_knowledge_entity_resolution_model_input(input)?;
        let left = self
            .get_knowledge_entity(&input.left_entity_id)?
            .with_context(|| {
                format!("left knowledge entity not found: {}", input.left_entity_id)
            })?;
        let right = self
            .get_knowledge_entity(&input.right_entity_id)?
            .with_context(|| {
                format!(
                    "right knowledge entity not found: {}",
                    input.right_entity_id
                )
            })?;
        if left.id == right.id {
            bail!("knowledge entity model resolution requires two different entities");
        }
        let source_cards = self.knowledge_entity_resolution_source_cards(&left, &right)?;
        if source_cards.is_empty() {
            bail!("knowledge entity model resolution requires source-card evidence");
        }
        let model = input.model_name.clone().unwrap_or_else(|| {
            if input.model_provider == "mock" {
                "mock-knowledge-entity-resolution".to_string()
            } else {
                std::env::var("ARCWELL_KNOWLEDGE_ENTITY_RESOLUTION_MODEL")
                    .unwrap_or_else(|_| "gpt-5.5-mini".to_string())
            }
        });
        let prompt_version = "knowledge-entity-resolution-v1".to_string();
        let prompt = build_knowledge_entity_resolution_prompt(
            &left,
            &right,
            &source_cards,
            &prompt_version,
        )?;
        let projected_cost = estimated_editorial_cost(&model, prompt.len());
        let invocation_job_id = format!("knowledge-entity-resolution-{}", Uuid::new_v4().simple());
        let (provider_response, cost_decision_id) = if input.model_provider == "mock" {
            (
                mock_knowledge_entity_resolution_response(&left, &right, &source_cards),
                None,
            )
        } else {
            let endpoint = validated_endpoint(
                input.endpoint.as_deref(),
                "https://api.openai.com/v1/responses",
            )?;
            self.policy_guard(PolicyRequest {
                action: "provider.network".to_string(),
                package: Some("arcwell-knowledge".to_string()),
                provider: Some("openai".to_string()),
                source: Some("knowledge_entity_resolution".to_string()),
                channel: None,
                subject: None,
                target: Some(endpoint.as_str().to_string()),
                projected_usd: Some(projected_cost),
                metadata: json!({
                    "left_entity_id": left.id,
                    "right_entity_id": right.id,
                    "model": model,
                    "prompt_version": prompt_version,
                    "source_card_count": source_cards.len()
                }),
                untrusted_excerpt: Some(excerpt(&prompt, 1_000)),
            })?;
            let decision = self.require_cost_budget(
                "arcwell-knowledge",
                &invocation_job_id,
                "openai",
                &model,
                Some("knowledge_entity_resolution"),
                projected_cost,
                "knowledge entity resolution",
            )?;
            (
                openai_knowledge_entity_resolution_response(
                    &prompt,
                    &model,
                    endpoint,
                    self.configured_openai_api_key()?.as_deref(),
                    Duration::from_secs(input.timeout_seconds.unwrap_or(45).clamp(1, 120)),
                )?,
                decision.decision_id,
            )
        };
        let mut resolution_input = parse_knowledge_entity_resolution_model_response(
            &provider_response,
            &left,
            &right,
            &source_cards,
        )?;
        resolution_input.resolver = format!("{}-model-v1", input.model_provider);
        resolution_input.evidence_json = sanitize_work_json(json!({
            "model_provider": input.model_provider,
            "model_name": model,
            "prompt_version": prompt_version,
            "cost_decision_id": cost_decision_id,
            "provider_evidence": resolution_input.evidence_json,
            "left_entity": {
                "id": left.id,
                "entity_type": left.entity_type,
                "name": left.name,
                "canonical_key": left.canonical_key
            },
            "right_entity": {
                "id": right.id,
                "entity_type": right.entity_type,
                "name": right.name,
                "canonical_key": right.canonical_key
            },
            "boundary": "Model output is a reviewable proposal only; it cannot merge or rewrite knowledge graph identity."
        }))?;
        let resolution = self.upsert_knowledge_entity_resolution(resolution_input)?;
        let proof_level = if cost_decision_id.is_some() {
            "Provider Attempt: configured OpenAI credential".to_string()
        } else {
            "Local Proof: deterministic mock entity-resolution model".to_string()
        };
        Ok(KnowledgeEntityResolutionModelInvocation {
            resolution,
            provider_response,
            model_provider: input.model_provider,
            model_name: model,
            cost_decision_id,
            prompt_version,
            proof_level,
        })
    }

    pub(crate) fn knowledge_entity_resolution_source_cards(
        &self,
        left: &KnowledgeEntity,
        right: &KnowledgeEntity,
    ) -> Result<Vec<SourceCard>> {
        let source_card_ids = merge_string_sets(&left.source_card_ids, &right.source_card_ids);
        let mut cards = Vec::new();
        for source_card_id in source_card_ids.into_iter().take(40) {
            if let Some(card) = self.read_source_card(&source_card_id)? {
                cards.push(card);
            }
        }
        Ok(cards)
    }

    pub(crate) fn upsert_knowledge_entity_resolution(
        &self,
        input: KnowledgeEntityResolutionInput,
    ) -> Result<KnowledgeEntityResolution> {
        let input = normalize_knowledge_entity_resolution_input(input)?;
        self.get_knowledge_entity(&input.left_entity_id)?
            .with_context(|| {
                format!("left knowledge entity not found: {}", input.left_entity_id)
            })?;
        self.get_knowledge_entity(&input.right_entity_id)?
            .with_context(|| {
                format!(
                    "right knowledge entity not found: {}",
                    input.right_entity_id
                )
            })?;
        for source_card_id in &input.source_card_ids {
            self.read_source_card(source_card_id)?
                .with_context(|| format!("source card not found: {source_card_id}"))?;
        }
        let pair_key = knowledge_entity_resolution_pair_key(
            &input.left_entity_id,
            &input.right_entity_id,
            &input.resolver,
        );
        let id = format!("keres-{}", &sha256(pair_key.as_bytes())[..16]);
        let timestamp = now();
        self.conn.execute(
            r#"
            INSERT INTO knowledge_entity_resolutions
              (id, left_entity_id, right_entity_id, status, decision, confidence,
               resolver, reason, evidence_json, source_card_ids_json, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)
            ON CONFLICT(left_entity_id, right_entity_id, resolver) DO UPDATE SET
              status = excluded.status,
              decision = excluded.decision,
              confidence = excluded.confidence,
              reason = excluded.reason,
              evidence_json = excluded.evidence_json,
              source_card_ids_json = excluded.source_card_ids_json,
              updated_at = excluded.updated_at
            "#,
            params![
                id,
                input.left_entity_id,
                input.right_entity_id,
                input.status,
                input.decision,
                input.confidence,
                input.resolver,
                input.reason,
                input.evidence_json.to_string(),
                serde_json::to_string(&input.source_card_ids)?,
                timestamp,
            ],
        )?;
        self.get_knowledge_entity_resolution(&id)?
            .with_context(|| format!("inserted knowledge entity resolution not found: {id}"))
    }

    pub(crate) fn get_knowledge_entity_resolution(
        &self,
        id: &str,
    ) -> Result<Option<KnowledgeEntityResolution>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, left_entity_id, right_entity_id, status, decision, confidence,
                       resolver, reason, evidence_json, source_card_ids_json, created_at, updated_at
                FROM knowledge_entity_resolutions
                WHERE id = ?1
                "#,
                params![id],
                knowledge_entity_resolution_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn upsert_knowledge_event(&self, input: KnowledgeEventInput) -> Result<KnowledgeEvent> {
        let input = normalize_knowledge_event_input(input)?;
        let id = format!(
            "kevt-{}",
            &sha256(format!("{}\n{}", input.event_type, input.canonical_key).as_bytes())[..16]
        );
        let timestamp = now();
        let metadata_json = serde_json::to_string(&input.metadata)?;
        self.conn.execute(
            r#"
            INSERT INTO knowledge_events
              (id, event_type, status, title, canonical_key, primary_entity_key, event_time,
               summary, first_seen_at, last_seen_at, confidence, metadata_json, created_at, updated_at)
            VALUES (?1, ?2, 'candidate', ?3, ?4, ?5, ?6, ?7, ?8, ?8, ?9, ?10, ?8, ?8)
            ON CONFLICT(event_type, canonical_key) DO UPDATE SET
              title = excluded.title,
              primary_entity_key = excluded.primary_entity_key,
              event_time = excluded.event_time,
              summary = excluded.summary,
              last_seen_at = excluded.last_seen_at,
              confidence = excluded.confidence,
              metadata_json = excluded.metadata_json,
              updated_at = excluded.updated_at
            "#,
            params![
                id,
                input.event_type,
                input.title,
                input.canonical_key,
                input.primary_entity_key,
                input.event_time,
                input.summary,
                timestamp,
                input.confidence,
                metadata_json,
            ],
        )?;
        self.get_knowledge_event(&id)?
            .with_context(|| format!("inserted knowledge event not found: {id}"))
    }

    pub fn get_knowledge_event(&self, id: &str) -> Result<Option<KnowledgeEvent>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, event_type, status, title, canonical_key, primary_entity_key,
                       event_time, summary, first_seen_at, last_seen_at, confidence,
                       metadata_json, created_at, updated_at
                FROM knowledge_events
                WHERE id = ?1
                "#,
                params![id],
                knowledge_event_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_knowledge_events(&self, limit: usize) -> Result<Vec<KnowledgeEvent>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, event_type, status, title, canonical_key, primary_entity_key,
                   event_time, summary, first_seen_at, last_seen_at, confidence,
                   metadata_json, created_at, updated_at
            FROM knowledge_events
            ORDER BY updated_at DESC
            LIMIT ?1
            "#,
        )?;
        rows(stmt.query_map(params![limit.clamp(1, 500)], knowledge_event_from_row)?)
    }

    pub fn add_knowledge_event_source(
        &self,
        input: KnowledgeEventSourceInput,
    ) -> Result<KnowledgeEventSource> {
        validate_knowledge_event_source_input(&input)?;
        self.get_knowledge_event(&input.event_id)?
            .with_context(|| format!("knowledge event not found: {}", input.event_id))?;
        self.read_source_card(&input.source_card_id)?
            .with_context(|| format!("source card not found: {}", input.source_card_id))?;
        let id = format!(
            "kevsrc-{}",
            &sha256(
                format!(
                    "{}\n{}\n{}",
                    input.event_id, input.source_card_id, input.role
                )
                .as_bytes(),
            )[..16]
        );
        let timestamp = now();
        self.conn.execute(
            r#"
            INSERT INTO knowledge_event_sources
              (id, event_id, source_card_id, role, confidence, claim_summary,
               metadata_json, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)
            ON CONFLICT(event_id, source_card_id, role) DO UPDATE SET
              confidence = excluded.confidence,
              claim_summary = excluded.claim_summary,
              metadata_json = excluded.metadata_json,
              updated_at = excluded.updated_at
            "#,
            params![
                id,
                input.event_id,
                input.source_card_id,
                input.role,
                input.confidence,
                input.claim_summary,
                input.metadata.to_string(),
                timestamp,
            ],
        )?;
        self.get_knowledge_event_source(&id)?
            .with_context(|| format!("inserted knowledge event source not found: {id}"))
    }

    pub fn get_knowledge_event_source(&self, id: &str) -> Result<Option<KnowledgeEventSource>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, event_id, source_card_id, role, confidence, claim_summary,
                       metadata_json, created_at, updated_at
                FROM knowledge_event_sources
                WHERE id = ?1
                "#,
                params![id],
                knowledge_event_source_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_knowledge_event_sources(
        &self,
        event_id: &str,
    ) -> Result<Vec<KnowledgeEventSource>> {
        validate_id(event_id)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, event_id, source_card_id, role, confidence, claim_summary,
                   metadata_json, created_at, updated_at
            FROM knowledge_event_sources
            WHERE event_id = ?1
            ORDER BY updated_at DESC
            "#,
        )?;
        rows(stmt.query_map(params![event_id], knowledge_event_source_from_row)?)
    }

    pub fn confirm_knowledge_event(&self, event_id: &str) -> Result<KnowledgeEvent> {
        validate_id(event_id)?;
        let evidence_count: i64 = self.conn.query_row(
            r#"
            SELECT COUNT(*)
            FROM knowledge_event_sources event_source
            JOIN source_cards source_card ON source_card.id = event_source.source_card_id
            WHERE event_source.event_id = ?1
            "#,
            params![event_id],
            |row| row.get(0),
        )?;
        if evidence_count == 0 {
            bail!("knowledge event confirmation requires source-card evidence");
        }
        let timestamp = now();
        self.conn.execute(
            "UPDATE knowledge_events SET status = 'confirmed', updated_at = ?2 WHERE id = ?1",
            params![event_id, timestamp],
        )?;
        self.get_knowledge_event(event_id)?
            .with_context(|| format!("knowledge event not found: {event_id}"))
    }

    pub fn create_knowledge_cluster(
        &self,
        input: KnowledgeClusterInput,
    ) -> Result<KnowledgeCluster> {
        validate_knowledge_cluster_input(&input)?;
        let source_card_ids = self.normalize_knowledge_source_card_ids(&input.source_card_ids)?;
        let event_ids = self.normalize_knowledge_event_ids(&input.event_ids)?;
        self.ensure_knowledge_cluster_event_evidence(&event_ids, &source_card_ids)?;
        let timestamp = now();
        let first_seen_at = input.first_seen_at.unwrap_or_else(|| timestamp.clone());
        let last_seen_at = input.last_seen_at.unwrap_or_else(|| timestamp.clone());
        let source_card_ids_json = serde_json::to_string(&source_card_ids)?;
        let event_ids_json = serde_json::to_string(&event_ids)?;
        let id = format!(
            "kcl-{}",
            &sha256(format!("{}\n{}", input.topic, source_card_ids_json).as_bytes())[..16]
        );
        self.conn.execute(
            r#"
            INSERT INTO knowledge_clusters
              (id, topic, status, source_card_ids_json, event_ids_json, first_seen_at,
               last_seen_at, novelty_score, momentum_score, stale_score, reason,
               duplicate_groups_json, metadata_json, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?14)
            ON CONFLICT(topic, source_card_ids_json) DO UPDATE SET
              status = excluded.status,
              event_ids_json = excluded.event_ids_json,
              last_seen_at = excluded.last_seen_at,
              novelty_score = excluded.novelty_score,
              momentum_score = excluded.momentum_score,
              stale_score = excluded.stale_score,
              reason = excluded.reason,
              duplicate_groups_json = excluded.duplicate_groups_json,
              metadata_json = excluded.metadata_json,
              updated_at = excluded.updated_at
            "#,
            params![
                id,
                input.topic,
                input.status,
                source_card_ids_json,
                event_ids_json,
                first_seen_at,
                last_seen_at,
                input.novelty_score,
                input.momentum_score,
                input.stale_score,
                input.reason,
                input.duplicate_groups.to_string(),
                input.metadata.to_string(),
                timestamp,
            ],
        )?;
        self.get_knowledge_cluster(&id)?
            .with_context(|| format!("inserted knowledge cluster not found: {id}"))
    }

    pub fn add_source_cards_to_knowledge_cluster(
        &self,
        cluster_id: &str,
        source_card_ids: &[String],
        reason: Option<&str>,
    ) -> Result<KnowledgeCluster> {
        validate_id(cluster_id)?;
        let mut cluster = self
            .get_knowledge_cluster(cluster_id)?
            .with_context(|| format!("knowledge cluster not found: {cluster_id}"))?;
        let mut merged_source_card_ids = cluster.source_card_ids.clone();
        merged_source_card_ids.extend(source_card_ids.iter().cloned());
        let merged_source_card_ids =
            self.normalize_knowledge_source_card_ids(&merged_source_card_ids)?;
        if merged_source_card_ids == cluster.source_card_ids {
            return Ok(cluster);
        }
        let merged_source_cards = self.read_knowledge_source_cards(&merged_source_card_ids)?;
        let new_source_cards = source_card_ids
            .iter()
            .filter_map(|id| {
                merged_source_cards
                    .iter()
                    .find(|card| card.id == *id)
                    .cloned()
            })
            .collect::<Vec<_>>();
        let new_events = self.ensure_knowledge_events_for_source_cards(
            &new_source_cards,
            "knowledge_cluster_evidence_update",
        )?;
        let mut event_ids = cluster.event_ids.clone();
        event_ids.extend(new_events.into_iter().map(|event| event.id));
        let event_ids = self.normalize_knowledge_event_ids(&event_ids)?;
        self.ensure_knowledge_cluster_event_evidence(&event_ids, &merged_source_card_ids)?;
        let last_seen_at = merged_source_cards
            .iter()
            .map(|card| card.retrieved_at.clone())
            .max()
            .unwrap_or_else(now);
        let provider_count = merged_source_cards
            .iter()
            .map(|card| card.provider.clone())
            .collect::<BTreeSet<_>>()
            .len();
        let novelty_score = cluster.novelty_score.max(
            ((provider_count as f64 + merged_source_cards.len() as f64) / 12.0).clamp(0.1, 1.0),
        );
        let momentum_score = cluster
            .momentum_score
            .max((merged_source_cards.len() as f64 / 10.0).clamp(0.1, 1.0));
        let duplicate_groups = knowledge_duplicate_groups_for_cards(&merged_source_cards);
        let timestamp = now();
        let previous_revision = knowledge_source_card_revision(&cluster.source_card_ids);
        let current_revision = knowledge_source_card_revision(&merged_source_card_ids);
        let mut metadata = cluster.metadata.clone();
        if !metadata.is_object() {
            metadata = json!({ "previous_metadata": metadata });
        }
        if let Some(object) = metadata.as_object_mut() {
            object.insert(
                "evidence_revision".to_string(),
                json!({
                    "revision": current_revision,
                    "previous_revision": previous_revision,
                    "source_card_count": merged_source_card_ids.len(),
                    "updated_at": timestamp,
                    "reason": reason.unwrap_or("Merged new source-card evidence into existing knowledge cluster."),
                    "origin": "cluster-evidence-update"
                }),
            );
        }
        let metadata = sanitize_work_json(metadata)?;
        self.conn.execute(
            r#"
            UPDATE knowledge_clusters
            SET source_card_ids_json = ?2,
                event_ids_json = ?3,
                last_seen_at = ?4,
                novelty_score = ?5,
                momentum_score = ?6,
                duplicate_groups_json = ?7,
                metadata_json = ?8,
                updated_at = ?9
            WHERE id = ?1
            "#,
            params![
                cluster.id,
                serde_json::to_string(&merged_source_card_ids)?,
                serde_json::to_string(&event_ids)?,
                last_seen_at,
                novelty_score,
                momentum_score,
                duplicate_groups.to_string(),
                metadata.to_string(),
                timestamp,
            ],
        )?;
        cluster = self
            .get_knowledge_cluster(cluster_id)?
            .with_context(|| format!("updated knowledge cluster not found: {cluster_id}"))?;
        Ok(cluster)
    }

    pub fn get_knowledge_cluster(&self, id: &str) -> Result<Option<KnowledgeCluster>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, topic, status, source_card_ids_json, event_ids_json,
                       first_seen_at, last_seen_at, novelty_score, momentum_score,
                       stale_score, reason, duplicate_groups_json, metadata_json,
                       created_at, updated_at
                FROM knowledge_clusters
                WHERE id = ?1
                "#,
                params![id],
                knowledge_cluster_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_knowledge_clusters(&self, limit: usize) -> Result<Vec<KnowledgeCluster>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, topic, status, source_card_ids_json, event_ids_json,
                   first_seen_at, last_seen_at, novelty_score, momentum_score,
                   stale_score, reason, duplicate_groups_json, metadata_json,
                   created_at, updated_at
            FROM knowledge_clusters
            ORDER BY updated_at DESC
            LIMIT ?1
            "#,
        )?;
        rows(stmt.query_map(params![limit.clamp(1, 500)], knowledge_cluster_from_row)?)
    }

    pub fn promote_knowledge_cluster(
        &self,
        cluster_id: &str,
        reviewer: Option<&str>,
        reason: Option<&str>,
    ) -> Result<KnowledgeClusterPromotionReport> {
        validate_id(cluster_id)?;
        let reviewer =
            sanitize_work_text(reviewer.unwrap_or("arcwell-knowledge-cluster-review"), 200)?;
        let reason = sanitize_work_text(
            reason.unwrap_or(
                "Promoted source-card-backed knowledge cluster after explicit review and policy gate.",
            ),
            2_000,
        )?;
        validate_knowledge_text("knowledge cluster promotion reviewer", &reviewer, 200)?;
        validate_knowledge_text("knowledge cluster promotion reason", &reason, 2_000)?;
        let cluster = self
            .get_knowledge_cluster(cluster_id)?
            .with_context(|| format!("knowledge cluster not found: {cluster_id}"))?;
        if cluster.source_card_ids.is_empty() {
            bail!("knowledge cluster promotion requires source-card evidence");
        }
        if cluster.status == "active" {
            let editorial_decision = self
                .get_knowledge_editorial_decision_for_cluster(&cluster.id, "promote_model_cluster")?
                .with_context(|| {
                    format!(
                        "active knowledge cluster {} has no durable promote_model_cluster decision",
                        cluster.id
                    )
                })?;
            return Ok(KnowledgeClusterPromotionReport {
                cluster,
                editorial_decision,
                policy_decision_id: None,
                proof_level: "Local Proof: already-promoted source-card-backed knowledge cluster"
                    .to_string(),
            });
        }
        if cluster.status != "candidate" {
            bail!(
                "knowledge cluster promotion requires candidate status; cluster {} is {}",
                cluster.id,
                cluster.status
            );
        }

        let decision = self.policy_check(PolicyRequest {
            action: "knowledge_cluster.promote".to_string(),
            package: Some("arcwell-librarian".to_string()),
            provider: None,
            source: Some("knowledge_cluster_model_review".to_string()),
            channel: None,
            subject: Some(cluster.id.clone()),
            target: Some(cluster.id.clone()),
            projected_usd: None,
            metadata: json!({
                "cluster_id": cluster.id,
                "cluster_topic": cluster.topic,
                "cluster_origin": cluster.metadata.get("origin").and_then(Value::as_str),
                "cluster_status": cluster.status,
                "reviewer": reviewer,
                "source_card_count": cluster.source_card_ids.len(),
                "boundary": "Promotion marks a reviewable cluster as eligible for shared wiki/report/digest expansion; it does not approve digest delivery.",
            }),
            untrusted_excerpt: Some(format!("{}\n{}", cluster.topic, cluster.reason)),
        })?;
        if !decision.allowed {
            let blocked =
                self.record_knowledge_editorial_decision(KnowledgeEditorialDecisionInput {
                    cluster_id: cluster.id.clone(),
                    decision: "promote_model_cluster".to_string(),
                    status: "blocked".to_string(),
                    wiki_page_id: None,
                    digest_candidate_id: None,
                    source_card_ids: cluster.source_card_ids.clone(),
                    reason: format!(
                        "Knowledge cluster promotion blocked by policy: {}",
                        redact_secret_like_text(&decision.reason)
                    ),
                    quality_findings: vec!["promotion_policy_not_allowed".to_string()],
                    metadata: json!({
                        "origin": "knowledge_cluster_model_review_v1",
                        "reviewer": reviewer,
                        "policy_decision_id": decision.id,
                        "policy_effect": decision.effect,
                        "matched_rule_id": decision.matched_rule_id,
                    }),
                })?;
            bail!(
                "knowledge cluster promotion blocked by policy: {} (decision_id: {}, editorial_decision_id: {})",
                redact_secret_like_text(&decision.reason),
                decision.id,
                blocked.id
            );
        }

        let timestamp = now();
        let mut metadata = match cluster.metadata.as_object() {
            Some(object) => Value::Object(object.clone()),
            None => json!({}),
        };
        if let Some(object) = metadata.as_object_mut() {
            object.insert(
                "promotion".to_string(),
                json!({
                    "status": "active",
                    "origin": "knowledge_cluster_model_review_v1",
                    "reviewed_by": reviewer,
                    "reason": reason,
                    "policy_decision_id": decision.id,
                    "matched_rule_id": decision.matched_rule_id,
                    "promoted_at": timestamp,
                    "boundary": "This promotion makes the cluster eligible for deterministic source-card-backed expansion; digest delivery still requires separate review/policy/channel gates.",
                }),
            );
        }
        let metadata = sanitize_work_json(metadata)?;
        self.conn.execute(
            r#"
            UPDATE knowledge_clusters
            SET status = 'active', metadata_json = ?2, updated_at = ?3
            WHERE id = ?1
            "#,
            params![cluster.id, metadata.to_string(), timestamp],
        )?;
        let promoted = self
            .get_knowledge_cluster(&cluster.id)?
            .with_context(|| format!("promoted knowledge cluster not found: {}", cluster.id))?;
        let editorial_decision =
            self.record_knowledge_editorial_decision(KnowledgeEditorialDecisionInput {
                cluster_id: promoted.id.clone(),
                decision: "promote_model_cluster".to_string(),
                status: "completed".to_string(),
                wiki_page_id: None,
                digest_candidate_id: None,
                source_card_ids: promoted.source_card_ids.clone(),
                reason: format!(
                    "Promoted knowledge cluster {} to active after explicit policy-gated review by {}. {}",
                    promoted.id, reviewer, reason
                ),
                quality_findings: Vec::new(),
                metadata: json!({
                    "origin": "knowledge_cluster_model_review_v1",
                    "reviewer": reviewer,
                    "policy_decision_id": decision.id,
                    "matched_rule_id": decision.matched_rule_id,
                    "proof_level": "Local Proof: policy-gated promotion from model proposal to active shared cluster",
                }),
            })?;
        Ok(KnowledgeClusterPromotionReport {
            cluster: promoted,
            editorial_decision,
            policy_decision_id: Some(decision.id),
            proof_level:
                "Local Proof: policy-gated promotion from model proposal to active shared cluster"
                    .to_string(),
        })
    }

    pub fn decide_knowledge_cluster_editorial(
        &self,
        cluster_id: &str,
        auto_enqueue: bool,
    ) -> Result<KnowledgeClusterEditorialDecisionReport> {
        validate_id(cluster_id)?;
        let cluster = self
            .get_knowledge_cluster(cluster_id)?
            .with_context(|| format!("knowledge cluster not found: {cluster_id}"))?;
        let replay_source_card_count = cluster.source_card_ids.len();
        if let Some(existing) =
            self.get_knowledge_editorial_decision_for_cluster(&cluster.id, "editorial_decide")?
            && matches!(existing.status.as_str(), "completed" | "blocked")
            && knowledge_editorial_decision_matches_cluster_revision(&existing, &cluster)
        {
            let action = existing
                .metadata
                .get("recommended_action")
                .and_then(Value::as_str)
                .unwrap_or(existing.decision.as_str())
                .to_string();
            let enqueued_job =
                self.maybe_enqueue_editorial_decision_followup(&cluster, &action, auto_enqueue)?;
            return Ok(KnowledgeClusterEditorialDecisionReport {
                cluster,
                editorial_decision: existing,
                recommended_action: action,
                matched_wiki_page: None,
                enqueued_job,
                source_card_count: replay_source_card_count,
                proof_level: "Local Proof: replayed durable deterministic editorial decision"
                    .to_string(),
                metadata: json!({
                    "origin": "knowledge_cluster_editorial_decider_v1",
                    "replayed_existing_decision": true,
                    "auto_enqueue": auto_enqueue,
                }),
            });
        }

        let source_card_count = cluster.source_card_ids.len();
        let mut quality_findings = Vec::new();
        let (recommended_action, status, reason, matched_wiki_page, digest_candidate_id) =
            if source_card_count == 0 {
                (
                    "block_for_review".to_string(),
                    "blocked".to_string(),
                    "Blocked shared knowledge editorial decision because the cluster has no source-card evidence.".to_string(),
                    None,
                    None,
                )
            } else if knowledge_cluster_requires_model_promotion(&cluster) {
                quality_findings.push("model_cluster_requires_promotion".to_string());
                (
                    "block_for_review".to_string(),
                    "blocked".to_string(),
                    format!(
                        "Blocked editorial action for review-only model-origin cluster {}; explicit knowledge_cluster.promote policy is required before wiki/report/digest work.",
                        cluster.id
                    ),
                    None,
                    None,
                )
            } else if let Some(page) = self.find_existing_wiki_page_for_cluster(&cluster)? {
                if source_card_count >= 2
                    && cluster.momentum_score >= 0.55
                    && cluster.stale_score < 0.8
                {
                    let candidate =
                        self.create_digest_candidate(&cluster.topic, &cluster.source_card_ids)?;
                    (
                        "digest_only".to_string(),
                        "completed".to_string(),
                        format!(
                            "Matched existing wiki page {} for cluster {}; created digest candidate {} instead of a duplicate wiki page.",
                            page.id, cluster.id, candidate.id
                        ),
                        Some(page),
                        Some(candidate.id),
                    )
                } else {
                    (
                        "update_existing_wiki".to_string(),
                        "completed".to_string(),
                        format!(
                            "Matched existing wiki page {} for cluster {}; selected update_existing_wiki and did not create a duplicate page.",
                            page.id, cluster.id
                        ),
                        Some(page),
                        None,
                    )
                }
            } else if source_card_count < 2
                || cluster.stale_score >= 0.85
                || (cluster.novelty_score < 0.35 && cluster.momentum_score < 0.20)
            {
                quality_findings.push("insufficient_editorial_signal".to_string());
                (
                    "monitor_only".to_string(),
                    "completed".to_string(),
                    format!(
                        "Cluster {} remains monitor-only: source_card_count={}, novelty={:.2}, momentum={:.2}, stale={:.2}.",
                        cluster.id,
                        source_card_count,
                        cluster.novelty_score,
                        cluster.momentum_score,
                        cluster.stale_score
                    ),
                    None,
                    None,
                )
            } else {
                (
                    "expand_wiki_and_digest".to_string(),
                    "completed".to_string(),
                    format!(
                        "Cluster {} has enough source-card-backed novelty and momentum for wiki expansion plus digest-candidate creation; delivery remains separately gated.",
                        cluster.id
                    ),
                    None,
                    None,
                )
            };

        let matched_wiki_page_id = matched_wiki_page.as_ref().map(|page| page.id.clone());
        let decision = self.record_knowledge_editorial_decision(KnowledgeEditorialDecisionInput {
            cluster_id: cluster.id.clone(),
            decision: "editorial_decide".to_string(),
            status,
            wiki_page_id: matched_wiki_page_id.clone(),
            digest_candidate_id: digest_candidate_id.clone(),
            source_card_ids: cluster.source_card_ids.clone(),
            reason: reason.clone(),
            quality_findings: quality_findings.clone(),
            metadata: json!({
                "origin": "knowledge_cluster_editorial_decider_v1",
                "recommended_action": recommended_action,
                "auto_enqueue": auto_enqueue,
                "source_card_count": source_card_count,
                "novelty_score": cluster.novelty_score,
                "momentum_score": cluster.momentum_score,
                "stale_score": cluster.stale_score,
                "matched_wiki_page_id": matched_wiki_page_id,
                "digest_candidate_id": digest_candidate_id,
                "delivery_authorized": false,
                "boundary": "This decision may create or enqueue local wiki/report/digest-candidate work, but it never authorizes external delivery.",
            }),
        })?;
        let enqueued_job = self.maybe_enqueue_editorial_decision_followup(
            &cluster,
            &recommended_action,
            auto_enqueue,
        )?;
        Ok(KnowledgeClusterEditorialDecisionReport {
            cluster,
            editorial_decision: decision,
            recommended_action,
            matched_wiki_page,
            enqueued_job,
            source_card_count,
            proof_level: "Local Proof: deterministic source-card-backed editorial decision"
                .to_string(),
            metadata: json!({
                "origin": "knowledge_cluster_editorial_decider_v1",
                "auto_enqueue": auto_enqueue,
                "quality_findings": quality_findings,
            }),
        })
    }

    pub(crate) fn maybe_enqueue_editorial_decision_followup(
        &self,
        cluster: &KnowledgeCluster,
        recommended_action: &str,
        auto_enqueue: bool,
    ) -> Result<Option<WikiJob>> {
        if !auto_enqueue || recommended_action != "expand_wiki_and_digest" {
            return Ok(None);
        }
        if let Some(status) = self.knowledge_cluster_expansion_decision_status(&cluster.id)?
            && matches!(status.as_str(), "completed" | "blocked")
        {
            return Ok(None);
        }
        if self.knowledge_cluster_expansion_has_active_job(&cluster.id)? {
            return Ok(None);
        }
        self.enqueue_knowledge_cluster_expansion_job_with_lineage(
            &cluster.id,
            true,
            Some(json!({
                "trigger": "editorial_decide",
                "cluster_id": cluster.id,
                "topic": cluster.topic,
                "source_card_count": cluster.source_card_ids.len(),
                "source_card_ids": cluster.source_card_ids,
                "boundary": "Expansion is a local wiki/report/digest-candidate follow-up; delivery remains separately reviewed and policy-gated."
            })),
        )
        .map(Some)
    }

    pub(crate) fn find_existing_wiki_page_for_cluster(
        &self,
        cluster: &KnowledgeCluster,
    ) -> Result<Option<WikiPageSummary>> {
        let own_source = format!("knowledge-cluster:{}", cluster.id);
        Ok(self
            .search_wiki_pages_for_research(&cluster.topic)?
            .into_iter()
            .find(|page| page.source != own_source && !page.source.starts_with("source-card:")))
    }

    pub fn expand_knowledge_cluster(
        &self,
        cluster_id: &str,
        create_digest: bool,
    ) -> Result<KnowledgeClusterExpansionReport> {
        let cluster = self
            .get_knowledge_cluster(cluster_id)?
            .with_context(|| format!("knowledge cluster not found: {cluster_id}"))?;
        if let Err(error) = ensure_knowledge_cluster_can_expand(&cluster) {
            let _ = self.record_knowledge_editorial_decision(KnowledgeEditorialDecisionInput {
                cluster_id: cluster.id.clone(),
                decision: "model_cluster_expand_requires_promotion".to_string(),
                status: "blocked".to_string(),
                wiki_page_id: None,
                digest_candidate_id: None,
                source_card_ids: cluster.source_card_ids.clone(),
                reason: format!(
                    "Knowledge cluster expansion blocked before writing wiki/report/digest: {error}"
                ),
                quality_findings: vec!["model_cluster_requires_promotion".to_string()],
                metadata: json!({
                    "origin": "knowledge_cluster_editor_v1",
                    "create_digest": create_digest,
                    "cluster_topic": cluster.topic,
                    "blocked_boundary": "Model-proposed clusters are review-only until promoted through knowledge_cluster.promote policy.",
                }),
            });
            return Err(error);
        }
        if cluster.source_card_ids.is_empty() {
            bail!("knowledge cluster expansion requires source-card evidence");
        }
        let source_cards = self.read_knowledge_source_cards(&cluster.source_card_ids)?;
        let markdown = render_knowledge_cluster_wiki_page(&cluster, &source_cards)?;
        let quality_findings = audit_knowledge_cluster_wiki_page(&cluster, &markdown);
        if !quality_findings.is_empty() {
            let _ = self.record_knowledge_editorial_decision(KnowledgeEditorialDecisionInput {
                cluster_id: cluster.id.clone(),
                decision: "expand_wiki_and_digest".to_string(),
                status: "blocked".to_string(),
                wiki_page_id: None,
                digest_candidate_id: None,
                source_card_ids: cluster.source_card_ids.clone(),
                reason: format!(
                    "Knowledge cluster expansion blocked by quality gate: {}",
                    quality_findings.join("; ")
                ),
                quality_findings: quality_findings.clone(),
                metadata: json!({
                    "origin": "knowledge_cluster_editor_v1",
                    "create_digest": create_digest,
                    "cluster_topic": cluster.topic,
                }),
            });
            bail!(
                "knowledge cluster expansion quality gate failed: {}",
                quality_findings.join("; ")
            );
        }

        let wiki_title = format!("Knowledge: {}", cluster.topic);
        let wiki_page_id = self.add_wiki_page(
            &wiki_title,
            &markdown,
            &format!("knowledge-cluster:{}", cluster.id),
        )?;
        let wiki_page = self
            .read_wiki_page(&wiki_page_id)?
            .with_context(|| format!("knowledge cluster wiki page not found: {wiki_page_id}"))?;
        let report = self.record_knowledge_report(KnowledgeReportInput {
            cluster_id: cluster.id.clone(),
            title: cluster.topic.clone(),
            body_markdown: markdown.clone(),
            status: "draft".to_string(),
            source_card_ids: cluster.source_card_ids.clone(),
            metadata: json!({
                "origin": "knowledge_cluster_editor_v1",
                "wiki_page_id": wiki_page_id,
                "create_digest": create_digest,
            }),
        })?;
        let mut superseded_digest_candidate_ids = Vec::new();
        let (digest_candidate, auto_approval) = if create_digest {
            let candidate =
                self.create_digest_candidate(&cluster.topic, &cluster.source_card_ids)?;
            superseded_digest_candidate_ids =
                self.supersede_stale_cluster_digest_candidates(&cluster, &candidate.id)?;
            let (candidate, approval) =
                self.maybe_auto_approve_knowledge_digest_candidate(&cluster, &report, &candidate)?;
            (Some(candidate), approval)
        } else {
            (
                None,
                json!({
                    "status": "skipped",
                    "reason": "digest creation disabled"
                }),
            )
        };
        let decision = self.record_knowledge_editorial_decision(KnowledgeEditorialDecisionInput {
            cluster_id: cluster.id.clone(),
            decision: if create_digest {
                "expand_wiki_and_digest".to_string()
            } else {
                "expand_wiki".to_string()
            },
            status: "completed".to_string(),
            wiki_page_id: Some(wiki_page_id.clone()),
            digest_candidate_id: digest_candidate.as_ref().map(|candidate| candidate.id.clone()),
            source_card_ids: cluster.source_card_ids.clone(),
            reason: format!(
                "Expanded shared knowledge cluster {} into wiki page {}{} from {} source cards.",
                cluster.id,
                wiki_page_id,
                digest_candidate
                    .as_ref()
                    .map(|candidate| format!(" and digest candidate {}", candidate.id))
                    .unwrap_or_default(),
                cluster.source_card_ids.len()
            ),
            quality_findings: Vec::new(),
            metadata: json!({
                "origin": "knowledge_cluster_editor_v1",
                "proof_level": "Local Proof: deterministic source-card-backed shared cluster expansion",
                "report_id": report.id,
                "wiki_page_title": wiki_title,
                "digest_auto_approval": auto_approval,
                "superseded_digest_candidate_ids": superseded_digest_candidate_ids,
            }),
        })?;
        let investigation = self.create_knowledge_cluster_investigation(&cluster.id)?;

        Ok(KnowledgeClusterExpansionReport {
            cluster,
            source_cards,
            wiki_page,
            editorial_decision: decision,
            report,
            digest_candidate,
            investigation,
            quality_findings,
            metadata: json!({
                "origin": "knowledge_cluster_editor_v1",
                "create_digest": create_digest,
                "digest_auto_approval": auto_approval,
                "superseded_digest_candidate_ids": superseded_digest_candidate_ids,
            }),
        })
    }

    pub fn expand_knowledge_cluster_with_model_writer(
        &self,
        input: KnowledgeClusterWriterModelInput,
    ) -> Result<KnowledgeClusterExpansionReport> {
        let input = self.normalize_knowledge_cluster_writer_model_input(input)?;
        let cluster = self
            .get_knowledge_cluster(&input.cluster_id)?
            .with_context(|| format!("knowledge cluster not found: {}", input.cluster_id))?;
        if let Err(error) = ensure_knowledge_cluster_can_expand(&cluster) {
            let _ = self.record_knowledge_editorial_decision(KnowledgeEditorialDecisionInput {
                cluster_id: cluster.id.clone(),
                decision: "model_write_wiki_and_digest".to_string(),
                status: "blocked".to_string(),
                wiki_page_id: None,
                digest_candidate_id: None,
                source_card_ids: cluster.source_card_ids.clone(),
                reason: format!(
                    "Model-backed knowledge writer blocked before writing wiki/report/digest: {error}"
                ),
                quality_findings: vec!["model_cluster_requires_promotion".to_string()],
                metadata: json!({
                    "origin": "knowledge_cluster_model_writer_v1",
                    "create_digest": input.create_digest,
                    "cluster_topic": cluster.topic,
                }),
            });
            return Err(error);
        }
        if cluster.source_card_ids.is_empty() {
            bail!("knowledge cluster model writer requires source-card evidence");
        }
        let source_cards = self.read_knowledge_source_cards(&cluster.source_card_ids)?;
        let invocation = match self.invoke_knowledge_cluster_writer_model(
            &cluster,
            &source_cards,
            &input,
        ) {
            Ok(invocation) => invocation,
            Err(error) => {
                let _ = self.record_knowledge_editorial_decision(
                        KnowledgeEditorialDecisionInput {
                            cluster_id: cluster.id.clone(),
                            decision: "model_write_wiki_and_digest".to_string(),
                            status: "blocked".to_string(),
                            wiki_page_id: None,
                            digest_candidate_id: None,
                            source_card_ids: cluster.source_card_ids.clone(),
                            reason: format!(
                                "Model-backed knowledge writer invocation failed before writing wiki/report/digest: {}",
                                redact_secret_like_text(&error.to_string())
                            ),
                            quality_findings: vec!["model_writer_invocation_failed".to_string()],
                            metadata: json!({
                                "origin": "knowledge_cluster_model_writer_v1",
                                "create_digest": input.create_digest,
                                "cluster_topic": cluster.topic,
                                "model_provider": input.model_provider,
                            }),
                        },
                    );
                bail!("{}", redact_secret_like_text(&error.to_string()));
            }
        };
        require_knowledge_cluster_source_cards(
            &cluster,
            &invocation.source_card_ids,
            "knowledge cluster model writer",
        )?;
        let markdown = invocation.markdown.clone();
        let quality_findings = audit_knowledge_cluster_wiki_page(&cluster, &markdown);
        if !quality_findings.is_empty() {
            let _ = self.record_knowledge_editorial_decision(KnowledgeEditorialDecisionInput {
                cluster_id: cluster.id.clone(),
                decision: "model_write_wiki_and_digest".to_string(),
                status: "blocked".to_string(),
                wiki_page_id: None,
                digest_candidate_id: None,
                source_card_ids: cluster.source_card_ids.clone(),
                reason: format!(
                    "Model-backed knowledge writer blocked by quality gate: {}",
                    quality_findings.join("; ")
                ),
                quality_findings: quality_findings.clone(),
                metadata: json!({
                    "origin": "knowledge_cluster_model_writer_v1",
                    "create_digest": input.create_digest,
                    "cluster_topic": cluster.topic,
                    "model_provider": invocation.model_provider,
                    "model_name": invocation.model_name,
                    "prompt_version": invocation.prompt_version,
                    "cost_decision_id": invocation.cost_decision_id,
                    "proof_level": invocation.proof_level,
                }),
            });
            bail!(
                "knowledge cluster model writer quality gate failed: {}",
                quality_findings.join("; ")
            );
        }

        let wiki_title = format!("Knowledge: {} (Model Draft)", cluster.topic);
        let wiki_page_id = self.add_wiki_page(
            &wiki_title,
            &markdown,
            &format!("knowledge-cluster-model-writer:{}", cluster.id),
        )?;
        let wiki_page = self.read_wiki_page(&wiki_page_id)?.with_context(|| {
            format!("knowledge cluster model wiki page not found: {wiki_page_id}")
        })?;
        let report = self.record_knowledge_report(KnowledgeReportInput {
            cluster_id: cluster.id.clone(),
            title: format!("{} (model draft)", cluster.topic),
            body_markdown: markdown.clone(),
            status: "draft".to_string(),
            source_card_ids: cluster.source_card_ids.clone(),
            metadata: json!({
                "origin": "knowledge_cluster_model_writer_v1",
                "wiki_page_id": wiki_page_id,
                "create_digest": input.create_digest,
                "model_provider": invocation.model_provider,
                "model_name": invocation.model_name,
                "prompt_version": invocation.prompt_version,
                "cost_decision_id": invocation.cost_decision_id,
                "proof_level": invocation.proof_level,
                "score": invocation.score,
            }),
        })?;
        let mut superseded_digest_candidate_ids = Vec::new();
        let (digest_candidate, auto_approval) = if input.create_digest {
            let candidate =
                self.create_digest_candidate(&cluster.topic, &cluster.source_card_ids)?;
            superseded_digest_candidate_ids =
                self.supersede_stale_cluster_digest_candidates(&cluster, &candidate.id)?;
            let (candidate, approval) =
                self.maybe_auto_approve_knowledge_digest_candidate(&cluster, &report, &candidate)?;
            (Some(candidate), approval)
        } else {
            (
                None,
                json!({
                    "status": "skipped",
                    "reason": "digest creation disabled"
                }),
            )
        };
        let decision = self.record_knowledge_editorial_decision(KnowledgeEditorialDecisionInput {
            cluster_id: cluster.id.clone(),
            decision: if input.create_digest {
                "model_write_wiki_and_digest".to_string()
            } else {
                "model_write_wiki".to_string()
            },
            status: "completed".to_string(),
            wiki_page_id: Some(wiki_page_id.clone()),
            digest_candidate_id: digest_candidate.as_ref().map(|candidate| candidate.id.clone()),
            source_card_ids: cluster.source_card_ids.clone(),
            reason: format!(
                "Model-backed writer expanded shared knowledge cluster {} into wiki page {}{} from {} source cards after quality gates.",
                cluster.id,
                wiki_page_id,
                digest_candidate
                    .as_ref()
                    .map(|candidate| format!(" and digest candidate {}", candidate.id))
                    .unwrap_or_default(),
                cluster.source_card_ids.len()
            ),
            quality_findings: Vec::new(),
            metadata: json!({
                "origin": "knowledge_cluster_model_writer_v1",
                "proof_level": invocation.proof_level,
                "report_id": report.id,
                "wiki_page_title": wiki_title,
                "digest_auto_approval": auto_approval,
                "superseded_digest_candidate_ids": superseded_digest_candidate_ids,
                "model_writer": {
                    "model_provider": invocation.model_provider,
                    "model_name": invocation.model_name,
                    "prompt_version": invocation.prompt_version,
                    "cost_decision_id": invocation.cost_decision_id,
                    "score": invocation.score,
                    "boundary": "Model prose is accepted only after source-card citation and wiki/report quality gates; delivery approval remains separate."
                }
            }),
        })?;
        let investigation = self.create_knowledge_cluster_investigation(&cluster.id)?;

        Ok(KnowledgeClusterExpansionReport {
            cluster,
            source_cards,
            wiki_page,
            editorial_decision: decision,
            report,
            digest_candidate,
            investigation,
            quality_findings,
            metadata: json!({
                "origin": "knowledge_cluster_model_writer_v1",
                "create_digest": input.create_digest,
                "digest_auto_approval": auto_approval,
                "model_writer": {
                    "model_provider": invocation.model_provider,
                    "model_name": invocation.model_name,
                    "prompt_version": invocation.prompt_version,
                    "cost_decision_id": invocation.cost_decision_id,
                    "proof_level": invocation.proof_level,
                    "score": invocation.score,
                }
            }),
        })
    }

    pub(crate) fn maybe_auto_approve_knowledge_digest_candidate(
        &self,
        cluster: &KnowledgeCluster,
        report: &KnowledgeReport,
        candidate: &DigestCandidate,
    ) -> Result<(DigestCandidate, Value)> {
        if candidate.review_status == "approved" && candidate.status == "approved" {
            return Ok((
                candidate.clone(),
                json!({
                    "status": "already_approved",
                    "candidate_id": candidate.id,
                }),
            ));
        }
        if candidate.review_status != "unreviewed" {
            return Ok((
                candidate.clone(),
                json!({
                    "status": "skipped",
                    "reason": "candidate_already_reviewed",
                    "candidate_status": candidate.status,
                    "review_status": candidate.review_status,
                }),
            ));
        }
        if candidate.status != "ready" || candidate.score < 0.75 {
            return Ok((
                candidate.clone(),
                json!({
                    "status": "skipped",
                    "reason": "candidate_below_auto_approval_threshold",
                    "candidate_status": candidate.status,
                    "score": candidate.score,
                    "minimum_score": 0.75,
                }),
            ));
        }
        if candidate.source_card_ids.len() < 2 {
            return Ok((
                candidate.clone(),
                json!({
                    "status": "skipped",
                    "reason": "auto_approval_requires_multiple_source_cards",
                    "source_card_count": candidate.source_card_ids.len(),
                }),
            ));
        }
        if report.status != "draft"
            || report.source_card_ids.len() != candidate.source_card_ids.len()
        {
            return Ok((
                candidate.clone(),
                json!({
                    "status": "skipped",
                    "reason": "report_candidate_evidence_mismatch",
                    "report_status": report.status,
                    "report_source_card_count": report.source_card_ids.len(),
                    "candidate_source_card_count": candidate.source_card_ids.len(),
                }),
            ));
        }

        let decision = self.policy_check(PolicyRequest {
            action: "digest_candidate.auto_approve".to_string(),
            package: Some("arcwell-librarian".to_string()),
            provider: None,
            source: Some("knowledge_cluster_expand".to_string()),
            channel: None,
            subject: Some(candidate.id.clone()),
            target: Some(cluster.id.clone()),
            projected_usd: None,
            metadata: json!({
                "cluster_id": cluster.id,
                "cluster_topic": cluster.topic,
                "candidate_id": candidate.id,
                "candidate_score": candidate.score,
                "candidate_reason": candidate.reason,
                "source_card_count": candidate.source_card_ids.len(),
                "report_id": report.id,
                "report_status": report.status,
                "quality_gate": "passed",
                "boundary": "Auto-approval only marks the digest candidate as reviewed; delivery still requires digest_candidate.deliver policy and channel authorization.",
            }),
            untrusted_excerpt: Some(cluster.topic.clone()),
        })?;
        if !decision.allowed {
            return Ok((
                candidate.clone(),
                json!({
                    "status": "blocked",
                    "reason": redact_secret_like_text(&decision.reason),
                    "policy_decision_id": decision.id,
                    "policy_effect": decision.effect,
                    "matched_rule_id": decision.matched_rule_id,
                }),
            ));
        }

        let reviewed = self.approve_digest_candidate(
            &candidate.id,
            Some("arcwell-knowledge-auto-approval"),
            Some("Auto-approved after shared knowledge wiki/report quality gate and explicit digest_candidate.auto_approve policy."),
        )?;
        Ok((
            reviewed,
            json!({
                "status": "approved",
                "reviewed_by": "arcwell-knowledge-auto-approval",
                "policy_decision_id": decision.id,
                "matched_rule_id": decision.matched_rule_id,
                "candidate_score": candidate.score,
                "source_card_count": candidate.source_card_ids.len(),
            }),
        ))
    }

    pub(crate) fn supersede_stale_cluster_digest_candidates(
        &self,
        cluster: &KnowledgeCluster,
        replacement_candidate_id: &str,
    ) -> Result<Vec<String>> {
        validate_id(replacement_candidate_id)?;
        let mut superseded = Vec::new();
        for decision in ["expand_wiki_and_digest", "model_write_wiki_and_digest"] {
            let Some(existing) =
                self.get_knowledge_editorial_decision_for_cluster(&cluster.id, decision)?
            else {
                continue;
            };
            if knowledge_editorial_decision_matches_cluster_revision(&existing, cluster) {
                continue;
            }
            let Some(candidate_id) = existing.digest_candidate_id.as_deref() else {
                continue;
            };
            if candidate_id == replacement_candidate_id {
                continue;
            }
            if self.supersede_digest_candidate_preserving_delivery_ledger(
                candidate_id,
                replacement_candidate_id,
                &format!(
                    "Superseded by refreshed cluster evidence for {} after source-card set changed from {} to {}.",
                    cluster.id,
                    knowledge_source_card_revision(&existing.source_card_ids),
                    knowledge_source_card_revision(&cluster.source_card_ids)
                ),
            )? {
                superseded.push(candidate_id.to_string());
            }
        }
        superseded.sort();
        superseded.dedup();
        Ok(superseded)
    }

    pub(crate) fn supersede_digest_candidate_preserving_delivery_ledger(
        &self,
        candidate_id: &str,
        replacement_candidate_id: &str,
        note: &str,
    ) -> Result<bool> {
        validate_id(candidate_id)?;
        validate_id(replacement_candidate_id)?;
        validate_notes(note)?;
        let timestamp = now();
        let changed = self.conn.execute(
            r#"
            UPDATE digest_candidates
            SET status = 'superseded',
                review_status = 'rejected',
                reviewed_at = ?1,
                reviewed_by = 'arcwell-digest-supersession',
                review_note = ?2,
                updated_at = ?1
            WHERE id = ?3
              AND id <> ?4
              AND status IN ('ready', 'approved')
            "#,
            params![timestamp, note, candidate_id, replacement_candidate_id],
        )?;
        Ok(changed > 0)
    }

    pub fn create_knowledge_cluster_investigation(
        &self,
        cluster_id: &str,
    ) -> Result<KnowledgeClusterInvestigationReport> {
        let cluster = self
            .get_knowledge_cluster(cluster_id)?
            .with_context(|| format!("knowledge cluster not found: {cluster_id}"))?;
        if cluster.source_card_ids.is_empty() {
            bail!("knowledge cluster investigation requires source-card evidence");
        }
        let source_cards = self.read_knowledge_source_cards(&cluster.source_card_ids)?;
        if let Some(existing_decision) =
            self.get_knowledge_editorial_decision_for_cluster(&cluster.id, "investigate_cluster")?
            && existing_decision.status == "completed"
            && let Some(run_id) = existing_decision
                .metadata
                .get("research_run_id")
                .and_then(Value::as_str)
            && let Some(run) = self.get_research_run(run_id)?
        {
            let tasks = self.list_research_tasks(&run.id)?;
            let source_links = self.list_research_run_sources(&run.id)?;
            return Ok(KnowledgeClusterInvestigationReport {
                cluster,
                research_run: run,
                tasks,
                source_links,
                editorial_decision: existing_decision,
                reused_existing: true,
                metadata: json!({
                    "origin": "knowledge_cluster_investigation_v1",
                    "reused_existing": true,
                    "boundary": "Existing research workflow is pending work, not completed investigation.",
                }),
            });
        }

        let query = format!("Knowledge cluster investigation: {}", cluster.topic);
        let run = self.insert_research_run(&query, "deep_open", None)?;
        let mut source_links = Vec::new();
        let source_family = cluster
            .metadata
            .get("source_family")
            .and_then(Value::as_str)
            .unwrap_or("knowledge_cluster");
        for card in &source_cards {
            source_links.push(self.link_source_card_to_research_run(
                &run.id,
                &card.id,
                source_family,
                "source-card",
                "needs_review",
                Some("Linked from shared knowledge cluster investigation queue."),
            )?);
        }
        let source_card_ids = source_cards
            .iter()
            .map(|card| card.id.clone())
            .collect::<Vec<_>>();
        let task_specs = knowledge_cluster_investigation_tasks(&cluster, &source_card_ids);
        let tasks = task_specs
            .into_iter()
            .map(|(role, instructions)| self.insert_research_task(&run.id, &role, &instructions))
            .collect::<Result<Vec<_>>>()?;
        let task_ids = tasks.iter().map(|task| task.id.clone()).collect::<Vec<_>>();
        let source_link_ids = source_links
            .iter()
            .map(|link| link.link.id.clone())
            .collect::<Vec<_>>();
        let decision = self.record_knowledge_editorial_decision(KnowledgeEditorialDecisionInput {
            cluster_id: cluster.id.clone(),
            decision: "investigate_cluster".to_string(),
            status: "completed".to_string(),
            wiki_page_id: None,
            digest_candidate_id: None,
            source_card_ids: cluster.source_card_ids.clone(),
            reason: format!(
                "Queued research run {} with {} source-linked investigation tasks for shared cluster {}.",
                run.id,
                tasks.len(),
                cluster.id
            ),
            quality_findings: Vec::new(),
            metadata: json!({
                "origin": "knowledge_cluster_investigation_v1",
                "research_run_id": run.id,
                "task_ids": task_ids,
                "source_link_ids": source_link_ids,
                "source_card_count": source_card_ids.len(),
                "boundary": "This records pending investigation work only; it does not prove primary-source reading, semantic synthesis, wiki acceptance, or digest delivery.",
            }),
        })?;

        Ok(KnowledgeClusterInvestigationReport {
            cluster,
            research_run: run,
            tasks,
            source_links,
            editorial_decision: decision,
            reused_existing: false,
            metadata: json!({
                "origin": "knowledge_cluster_investigation_v1",
                "reused_existing": false,
                "source_card_count": source_card_ids.len(),
            }),
        })
    }

    pub fn execute_knowledge_cluster_investigation(
        &self,
        cluster_id: &str,
    ) -> Result<KnowledgeClusterInvestigationExecutionReport> {
        let plan = self.create_knowledge_cluster_investigation(cluster_id)?;
        let run_id = plan.research_run.id.clone();
        let source_cards = self.list_research_run_source_cards(&run_id)?;
        if source_cards.is_empty() {
            bail!("knowledge cluster investigation execution requires linked source cards");
        }
        let mut executed_task_count = 0usize;
        let mut already_completed_task_count = 0usize;
        let mut quality_findings = Vec::new();
        for task in plan.tasks.iter() {
            match task.status.as_str() {
                "completed" => {
                    already_completed_task_count += 1;
                    continue;
                }
                "pending" => {}
                other => {
                    quality_findings.push(format!(
                        "task_not_executable:{}:{}",
                        task.id,
                        escape_markdown_line(other)
                    ));
                    continue;
                }
            }
            let role_run = self.start_research_role_run(ResearchRoleRunStart {
                run_id: run_id.clone(),
                role: task.role.clone(),
                host: "arcwell".to_string(),
                host_thread_id: None,
                host_subagent_id: None,
                tool_surface: Some("knowledge_cluster_investigation_execute".to_string()),
                prompt_version: "knowledge-cluster-investigation-executor-v1".to_string(),
                prompt_hash: Some(sha256(task.instructions.as_bytes())),
                execution_mode: "host_sequential".to_string(),
                input_artifact_ids: Vec::new(),
            })?;
            let body = render_knowledge_cluster_investigation_artifact(
                &plan.cluster,
                task,
                &source_cards,
            )?;
            let task_findings = audit_knowledge_cluster_investigation_artifact(
                &body,
                &plan.cluster,
                task,
                &source_cards,
            );
            if !task_findings.is_empty() {
                let message = task_findings.join(", ");
                let _ = self.finish_research_role_run(
                    &role_run.id,
                    "rejected",
                    None,
                    Some("quality_gate"),
                    Some(&message),
                );
                quality_findings.extend(
                    task_findings
                        .into_iter()
                        .map(|finding| format!("{}:{finding}", task.role)),
                );
                continue;
            }
            let artifact = self.record_research_artifact(ResearchArtifactInput {
                run_id: run_id.clone(),
                role_run_id: Some(role_run.id.clone()),
                artifact_type: "knowledge_cluster_investigation_artifact".to_string(),
                title: format!(
                    "Knowledge Cluster Investigation: {} / {}",
                    plan.cluster.topic, task.role
                ),
                body,
                metadata: json!({
                    "origin": "knowledge_cluster_investigation_executor_v1",
                    "cluster_id": plan.cluster.id,
                    "task_id": task.id,
                    "task_role": task.role,
                    "source_card_ids": source_cards.iter().map(|card| card.id.clone()).collect::<Vec<_>>(),
                    "boundary": "Deterministic source-card triage artifact; it is not proof that fresh external primary sources were fetched or that model-backed synthesis was accepted."
                }),
            })?;
            self.finish_research_role_run(
                &role_run.id,
                "completed",
                Some(&artifact.id),
                None,
                None,
            )?;
            let notes = format!(
                "Executed investigation role `{}` into research artifact `{}` from {} linked source cards. Source text was treated as untrusted evidence, not instructions.",
                task.role,
                artifact.id,
                source_cards.len()
            );
            self.complete_research_task(&task.id, &notes)?;
            executed_task_count += 1;
        }
        quality_findings.sort();
        quality_findings.dedup();
        let refreshed_tasks = self.list_research_tasks(&run_id)?;
        let all_tasks_completed = !refreshed_tasks.is_empty()
            && refreshed_tasks
                .iter()
                .all(|task| task.status == "completed");
        if all_tasks_completed && quality_findings.is_empty() {
            self.update_research_run_status(&run_id, "investigation_evidence_ready")?;
        }
        let refreshed_run = self.require_research_run(&run_id)?;
        let role_runs = self.list_research_role_runs(&run_id)?;
        let artifacts = self.list_research_artifacts(&run_id)?;
        let artifact_ids = artifacts
            .iter()
            .filter(|artifact| {
                artifact.metadata.get("cluster_id").and_then(Value::as_str)
                    == Some(plan.cluster.id.as_str())
                    && artifact.metadata.get("origin").and_then(Value::as_str)
                        == Some("knowledge_cluster_investigation_executor_v1")
            })
            .map(|artifact| artifact.id.clone())
            .collect::<Vec<_>>();
        let decision = self.record_knowledge_editorial_decision(KnowledgeEditorialDecisionInput {
            cluster_id: plan.cluster.id.clone(),
            decision: "execute_investigation_tasks".to_string(),
            status: if quality_findings.is_empty() && all_tasks_completed {
                "completed".to_string()
            } else {
                "blocked".to_string()
            },
            wiki_page_id: None,
            digest_candidate_id: None,
            source_card_ids: plan.cluster.source_card_ids.clone(),
            reason: format!(
                "Executed {} pending investigation tasks and found {} already completed tasks for research run {}.",
                executed_task_count, already_completed_task_count, run_id
            ),
            quality_findings: quality_findings.clone(),
            metadata: json!({
                "origin": "knowledge_cluster_investigation_executor_v1",
                "research_run_id": run_id,
                "artifact_ids": artifact_ids,
                "source_card_count": source_cards.len(),
                "executed_task_count": executed_task_count,
                "already_completed_task_count": already_completed_task_count,
                "proof_level": "Local Proof: deterministic source-card-backed investigation task execution",
                "boundary": "This completes deterministic research-task triage artifacts only; autonomous primary-source fetching, model-backed synthesis, accepted wiki expansion, and external delivery remain separate proof gates."
            }),
        })?;
        Ok(KnowledgeClusterInvestigationExecutionReport {
            cluster: plan.cluster,
            research_run: refreshed_run,
            tasks: refreshed_tasks,
            role_runs,
            artifacts,
            editorial_decision: decision,
            executed_task_count,
            already_completed_task_count,
            quality_findings,
            metadata: json!({
                "origin": "knowledge_cluster_investigation_executor_v1",
                "source_card_count": source_cards.len(),
                "artifact_count": artifact_ids.len(),
            }),
        })
    }

    pub fn invoke_knowledge_cluster_model(
        &self,
        input: KnowledgeClusterProposalModelInput,
    ) -> Result<KnowledgeClusterProposalModelInvocation> {
        let input = self.normalize_knowledge_cluster_model_input(input)?;
        let source_cards = self.read_knowledge_source_cards(&input.source_card_ids)?;
        if source_cards.is_empty() {
            bail!("knowledge cluster model proposal requires source-card evidence");
        }
        let model = input.model_name.clone().unwrap_or_else(|| {
            if input.model_provider == "mock" {
                "mock-knowledge-cluster-proposal".to_string()
            } else {
                std::env::var("ARCWELL_KNOWLEDGE_CLUSTER_MODEL")
                    .unwrap_or_else(|_| "gpt-5.5-mini".to_string())
            }
        });
        let prompt_version = "knowledge-cluster-proposal-v1".to_string();
        let prompt = build_knowledge_cluster_proposal_prompt(
            &source_cards,
            input.max_clusters,
            &prompt_version,
        )?;
        let projected_cost = estimated_editorial_cost(&model, prompt.len());
        let invocation_job_id = format!("knowledge-cluster-proposal-{}", Uuid::new_v4().simple());
        let (provider_response, cost_decision_id) = if input.model_provider == "mock" {
            (
                mock_knowledge_cluster_proposal_response(&source_cards, input.max_clusters),
                None,
            )
        } else {
            let endpoint = validated_endpoint(
                input.endpoint.as_deref(),
                "https://api.openai.com/v1/responses",
            )?;
            self.policy_guard(PolicyRequest {
                action: "provider.network".to_string(),
                package: Some("arcwell-knowledge".to_string()),
                provider: Some("openai".to_string()),
                source: Some("knowledge_cluster_proposal".to_string()),
                channel: None,
                subject: None,
                target: Some(endpoint.as_str().to_string()),
                projected_usd: Some(projected_cost),
                metadata: json!({
                    "model": model,
                    "prompt_version": prompt_version,
                    "source_card_count": source_cards.len(),
                    "max_clusters": input.max_clusters
                }),
                untrusted_excerpt: Some(excerpt(&prompt, 1_000)),
            })?;
            let decision = self.require_cost_budget(
                "arcwell-knowledge",
                &invocation_job_id,
                "openai",
                &model,
                Some("knowledge_cluster_proposal"),
                projected_cost,
                "knowledge cluster proposal",
            )?;
            (
                openai_knowledge_cluster_proposal_response(
                    &prompt,
                    &model,
                    endpoint,
                    self.configured_openai_api_key()?.as_deref(),
                    Duration::from_secs(input.timeout_seconds.unwrap_or(45).clamp(1, 120)),
                )?,
                decision.decision_id,
            )
        };
        let proposals = parse_knowledge_cluster_model_response(
            &provider_response,
            &source_cards,
            input.max_clusters,
        )?;
        let proof_level = if cost_decision_id.is_some() {
            "Provider Attempt: configured OpenAI credential".to_string()
        } else {
            "Local Proof: deterministic mock cluster proposal model".to_string()
        };
        let card_by_id = source_cards
            .iter()
            .map(|card| (card.id.clone(), card.clone()))
            .collect::<BTreeMap<_, _>>();
        let mut clusters = Vec::new();
        for proposal in proposals {
            let proposal_cards = proposal
                .source_card_ids
                .iter()
                .filter_map(|id| card_by_id.get(id).cloned())
                .collect::<Vec<_>>();
            let events = self.ensure_knowledge_events_for_source_cards(
                &proposal_cards,
                "model_cluster_proposal",
            )?;
            let event_ids = events
                .iter()
                .map(|event| event.id.clone())
                .collect::<Vec<_>>();
            let first_seen_at = proposal_cards
                .iter()
                .map(|card| card.retrieved_at.clone())
                .min();
            let last_seen_at = proposal_cards
                .iter()
                .map(|card| card.retrieved_at.clone())
                .max();
            clusters.push(self.create_knowledge_cluster(KnowledgeClusterInput {
                topic: proposal.topic,
                status: "candidate".to_string(),
                event_ids,
                source_card_ids: proposal.source_card_ids,
                first_seen_at,
                last_seen_at,
                novelty_score: proposal.novelty_score,
                momentum_score: proposal.momentum_score,
                stale_score: proposal.stale_score,
                reason: proposal.reason,
                duplicate_groups: proposal.duplicate_groups,
                metadata: sanitize_work_json(json!({
                    "origin": "model_cluster_proposal_v1",
                    "model_provider": input.model_provider,
                    "model_name": model,
                    "prompt_version": prompt_version,
                    "cost_decision_id": cost_decision_id,
                    "proof_level": proof_level,
                    "provider_evidence": proposal.evidence,
                    "source_card_count": proposal_cards.len(),
                    "boundary": "Model output is a reviewable clustering proposal only; it cannot write wiki pages, approve editorial decisions, send digests, or rewrite source evidence."
                }))?,
            })?);
        }
        Ok(KnowledgeClusterProposalModelInvocation {
            clusters,
            provider_response,
            model_provider: input.model_provider,
            model_name: model,
            cost_decision_id,
            prompt_version,
            proof_level,
        })
    }

    pub(crate) fn normalize_knowledge_cluster_writer_model_input(
        &self,
        input: KnowledgeClusterWriterModelInput,
    ) -> Result<KnowledgeClusterWriterModelInput> {
        let cluster_id = input.cluster_id.trim().to_string();
        validate_id(&cluster_id)?;
        let model_provider = input.model_provider.trim().to_ascii_lowercase();
        if !matches!(model_provider.as_str(), "mock" | "openai") {
            bail!("unsupported knowledge cluster writer model provider: {model_provider}");
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
        Ok(KnowledgeClusterWriterModelInput {
            cluster_id,
            model_provider,
            model_name,
            endpoint: input.endpoint,
            timeout_seconds: input.timeout_seconds,
            create_digest: input.create_digest,
        })
    }

    pub(crate) fn invoke_knowledge_cluster_writer_model(
        &self,
        cluster: &KnowledgeCluster,
        source_cards: &[SourceCard],
        input: &KnowledgeClusterWriterModelInput,
    ) -> Result<KnowledgeClusterWriterModelInvocation> {
        if source_cards.is_empty() {
            bail!("knowledge cluster writer model requires source-card evidence");
        }
        for card in source_cards {
            if let Some(reason) = metadata_model_prompt_exclusion_reason(
                &card.metadata,
                "knowledge cluster source card",
            ) {
                bail!(
                    "knowledge cluster writer source card is not eligible for model prompt: {reason}"
                );
            }
        }
        let model = input.model_name.clone().unwrap_or_else(|| {
            if input.model_provider == "mock" {
                "mock-knowledge-cluster-writer".to_string()
            } else {
                std::env::var("ARCWELL_KNOWLEDGE_CLUSTER_WRITER_MODEL")
                    .unwrap_or_else(|_| "gpt-4.1-mini".to_string())
            }
        });
        let prompt_version = "knowledge-cluster-writer-v1".to_string();
        let prompt = build_knowledge_cluster_writer_prompt(cluster, source_cards, &prompt_version)?;
        let projected_cost = estimated_editorial_cost(&model, prompt.len());
        let invocation_job_id = format!("knowledge-cluster-writer-{}", Uuid::new_v4().simple());
        let (provider_response, cost_decision_id) = if input.model_provider == "mock" {
            (
                mock_knowledge_cluster_writer_response(cluster, source_cards)?,
                None,
            )
        } else {
            let endpoint = validated_endpoint(
                input.endpoint.as_deref(),
                "https://api.openai.com/v1/responses",
            )?;
            self.policy_guard(PolicyRequest {
                action: "provider.network".to_string(),
                package: Some("arcwell-knowledge".to_string()),
                provider: Some("openai".to_string()),
                source: Some("knowledge_cluster_writer".to_string()),
                channel: None,
                subject: Some(cluster.id.clone()),
                target: Some(endpoint.as_str().to_string()),
                projected_usd: Some(projected_cost),
                metadata: json!({
                    "model": model,
                    "prompt_version": prompt_version,
                    "cluster_id": cluster.id,
                    "source_card_count": source_cards.len(),
                    "boundary": "Model writer output is accepted only after source-card citation and wiki/report quality gates."
                }),
                untrusted_excerpt: Some(excerpt(&prompt, 1_000)),
            })?;
            let decision = self.require_cost_budget(
                "arcwell-knowledge",
                &invocation_job_id,
                "openai",
                &model,
                Some("knowledge_cluster_writer"),
                projected_cost,
                "knowledge cluster writer",
            )?;
            (
                openai_knowledge_cluster_writer_response(
                    &prompt,
                    &model,
                    endpoint,
                    self.configured_openai_api_key()?.as_deref(),
                    Duration::from_secs(input.timeout_seconds.unwrap_or(45).clamp(1, 120)),
                )?,
                decision.decision_id,
            )
        };
        let (markdown, source_card_ids, score) =
            parse_knowledge_cluster_writer_response(&provider_response, cluster, source_cards)?;
        let proof_level = if cost_decision_id.is_some() {
            "Provider Attempt: configured OpenAI credential with source-card-gated wiki/report writer"
                .to_string()
        } else {
            "Local Proof: deterministic mock knowledge cluster writer".to_string()
        };
        Ok(KnowledgeClusterWriterModelInvocation {
            markdown,
            source_card_ids,
            provider_response,
            model_provider: input.model_provider.clone(),
            model_name: model,
            cost_decision_id,
            prompt_version,
            proof_level,
            score,
        })
    }

    pub fn record_knowledge_editorial_decision(
        &self,
        input: KnowledgeEditorialDecisionInput,
    ) -> Result<KnowledgeEditorialDecision> {
        validate_knowledge_editorial_decision_input(&input)?;
        let cluster = self
            .get_knowledge_cluster(&input.cluster_id)?
            .with_context(|| format!("knowledge cluster not found: {}", input.cluster_id))?;
        let source_card_ids = self.normalize_knowledge_source_card_ids(&input.source_card_ids)?;
        require_knowledge_cluster_source_cards(
            &cluster,
            &source_card_ids,
            "knowledge editorial decision",
        )?;
        if let Some(wiki_page_id) = &input.wiki_page_id {
            validate_id(wiki_page_id)?;
        }
        if let Some(digest_candidate_id) = &input.digest_candidate_id {
            validate_id(digest_candidate_id)?;
        }
        let cluster_revision = knowledge_source_card_revision(&cluster.source_card_ids);
        let decision_revision = knowledge_source_card_revision(&source_card_ids);
        let mut metadata = input.metadata;
        if !metadata.is_object() {
            metadata = json!({ "previous_metadata": metadata });
        }
        if let Some(object) = metadata.as_object_mut() {
            object.insert(
                "cluster_evidence_revision".to_string(),
                json!({
                    "revision": cluster_revision,
                    "source_card_count": cluster.source_card_ids.len(),
                }),
            );
            object.insert(
                "decision_evidence_revision".to_string(),
                json!({
                    "revision": decision_revision,
                    "source_card_count": source_card_ids.len(),
                }),
            );
        }
        let id = format!(
            "ked-{}",
            &sha256(format!("{}\n{}", input.cluster_id, input.decision).as_bytes())[..16]
        );
        let timestamp = now();
        self.conn.execute(
            r#"
            INSERT INTO knowledge_editorial_decisions
              (id, cluster_id, decision, status, wiki_page_id, digest_candidate_id,
               source_card_ids_json, reason, quality_findings_json, metadata_json,
               created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)
            ON CONFLICT(cluster_id, decision) DO UPDATE SET
              status = excluded.status,
              wiki_page_id = excluded.wiki_page_id,
              digest_candidate_id = excluded.digest_candidate_id,
              source_card_ids_json = excluded.source_card_ids_json,
              reason = excluded.reason,
              quality_findings_json = excluded.quality_findings_json,
              metadata_json = excluded.metadata_json,
              updated_at = excluded.updated_at
            "#,
            params![
                id,
                input.cluster_id,
                input.decision,
                input.status,
                input.wiki_page_id,
                input.digest_candidate_id,
                serde_json::to_string(&source_card_ids)?,
                input.reason,
                serde_json::to_string(&input.quality_findings)?,
                metadata.to_string(),
                timestamp,
            ],
        )?;
        self.get_knowledge_editorial_decision(&id)?
            .with_context(|| format!("inserted knowledge editorial decision not found: {id}"))
    }

    pub fn get_knowledge_editorial_decision(
        &self,
        id: &str,
    ) -> Result<Option<KnowledgeEditorialDecision>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, cluster_id, decision, status, wiki_page_id, digest_candidate_id,
                       source_card_ids_json, reason, quality_findings_json, metadata_json,
                       created_at, updated_at
                FROM knowledge_editorial_decisions
                WHERE id = ?1
                "#,
                params![id],
                knowledge_editorial_decision_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub(crate) fn get_knowledge_editorial_decision_for_cluster(
        &self,
        cluster_id: &str,
        decision: &str,
    ) -> Result<Option<KnowledgeEditorialDecision>> {
        validate_id(cluster_id)?;
        validate_key(decision)?;
        self.conn
            .query_row(
                r#"
                SELECT id, cluster_id, decision, status, wiki_page_id, digest_candidate_id,
                       source_card_ids_json, reason, quality_findings_json, metadata_json,
                       created_at, updated_at
                FROM knowledge_editorial_decisions
                WHERE cluster_id = ?1 AND decision = ?2
                "#,
                params![cluster_id, decision],
                knowledge_editorial_decision_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_knowledge_editorial_decisions(
        &self,
        limit: usize,
    ) -> Result<Vec<KnowledgeEditorialDecision>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, cluster_id, decision, status, wiki_page_id, digest_candidate_id,
                   source_card_ids_json, reason, quality_findings_json, metadata_json,
                   created_at, updated_at
            FROM knowledge_editorial_decisions
            ORDER BY updated_at DESC
            LIMIT ?1
            "#,
        )?;
        rows(stmt.query_map(
            params![limit.clamp(1, 500)],
            knowledge_editorial_decision_from_row,
        )?)
    }

    pub fn record_knowledge_report(&self, input: KnowledgeReportInput) -> Result<KnowledgeReport> {
        validate_knowledge_report_input(&input)?;
        let cluster = self
            .get_knowledge_cluster(&input.cluster_id)?
            .with_context(|| format!("knowledge cluster not found: {}", input.cluster_id))?;
        let source_card_ids = self.normalize_knowledge_source_card_ids(&input.source_card_ids)?;
        require_knowledge_cluster_source_cards(&cluster, &source_card_ids, "knowledge report")?;
        let quality_findings = audit_knowledge_report(&input.body_markdown, &source_card_ids);
        if !quality_findings.is_empty() {
            bail!(
                "knowledge report quality gate failed: {}",
                quality_findings.join("; ")
            );
        }
        let id = format!(
            "krpt-{}",
            &sha256(format!("{}\n{}", input.cluster_id, input.title).as_bytes())[..16]
        );
        let timestamp = now();
        self.conn.execute(
            r#"
            INSERT INTO knowledge_reports
              (id, cluster_id, title, body_markdown, status, source_card_ids_json,
               quality_findings_json, metadata_json, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, '[]', ?7, ?8, ?8)
            ON CONFLICT(cluster_id, title) DO UPDATE SET
              body_markdown = excluded.body_markdown,
              status = excluded.status,
              source_card_ids_json = excluded.source_card_ids_json,
              quality_findings_json = excluded.quality_findings_json,
              metadata_json = excluded.metadata_json,
              updated_at = excluded.updated_at
            "#,
            params![
                id,
                input.cluster_id,
                input.title,
                input.body_markdown,
                input.status,
                serde_json::to_string(&source_card_ids)?,
                input.metadata.to_string(),
                timestamp,
            ],
        )?;
        self.get_knowledge_report(&id)?
            .with_context(|| format!("inserted knowledge report not found: {id}"))
    }

    pub fn get_knowledge_report(&self, id: &str) -> Result<Option<KnowledgeReport>> {
        validate_id(id)?;
        self.conn
            .query_row(
                r#"
                SELECT id, cluster_id, title, body_markdown, status, source_card_ids_json,
                       quality_findings_json, metadata_json, created_at, updated_at
                FROM knowledge_reports
                WHERE id = ?1
                "#,
                params![id],
                knowledge_report_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_knowledge_reports(&self, limit: usize) -> Result<Vec<KnowledgeReport>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, cluster_id, title, body_markdown, status, source_card_ids_json,
                   quality_findings_json, metadata_json, created_at, updated_at
            FROM knowledge_reports
            ORDER BY updated_at DESC
            LIMIT ?1
            "#,
        )?;
        rows(stmt.query_map(params![limit.clamp(1, 500)], knowledge_report_from_row)?)
    }

    pub(crate) fn list_knowledge_reports_updated_between(
        &self,
        window_start: &str,
        window_end: &str,
        limit: usize,
    ) -> Result<Vec<KnowledgeReport>> {
        validate_timestamp(window_start)?;
        validate_timestamp(window_end)?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, cluster_id, title, body_markdown, status, source_card_ids_json,
                   quality_findings_json, metadata_json, created_at, updated_at
            FROM knowledge_reports
            WHERE updated_at >= ?1 AND updated_at <= ?2
            ORDER BY updated_at DESC
            LIMIT ?3
            "#,
        )?;
        rows(stmt.query_map(
            params![window_start, window_end, limit.clamp(1, 100) as i64],
            knowledge_report_from_row,
        )?)
    }

    pub(crate) fn read_source_cards_by_ids(
        &self,
        source_card_ids: &[String],
    ) -> Result<Vec<SourceCard>> {
        let mut cards = Vec::new();
        let mut seen = BTreeSet::new();
        for source_card_id in source_card_ids {
            if !seen.insert(source_card_id.clone()) {
                continue;
            }
            validate_id(source_card_id)?;
            cards.push(
                self.read_source_card(source_card_id)?
                    .with_context(|| format!("source card not found: {source_card_id}"))?,
            );
        }
        Ok(cards)
    }

    pub(crate) fn daily_briefing_related_wiki_pages(
        &self,
        reports: &[KnowledgeReport],
        source_cards: &[SourceCard],
    ) -> Result<BTreeMap<String, Vec<WikiPageSummary>>> {
        let source_cards_by_id = source_cards
            .iter()
            .map(|card| (card.id.clone(), card.clone()))
            .collect::<BTreeMap<_, _>>();
        let mut by_report = BTreeMap::new();
        for report in reports {
            let mut pages = Vec::new();
            let mut seen = BTreeSet::new();
            let report_cards = report
                .source_card_ids
                .iter()
                .filter_map(|id| source_cards_by_id.get(id))
                .cloned()
                .collect::<Vec<_>>();
            for query in daily_briefing_wiki_queries(report, &report_cards) {
                for page in self
                    .search_wiki_pages_for_research(&query)?
                    .into_iter()
                    .take(8)
                {
                    if !seen.insert(page.id.clone()) {
                        continue;
                    }
                    if daily_briefing_wiki_page_is_story_context(&page) {
                        pages.push(page);
                    }
                    if pages.len() >= 4 {
                        break;
                    }
                }
                if pages.len() >= 4 {
                    break;
                }
            }
            by_report.insert(report.id.clone(), pages);
        }
        Ok(by_report)
    }
}
