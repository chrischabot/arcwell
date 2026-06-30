use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPageSummary {
    pub id: String,
    pub title: String,
    pub path: String,
    pub content_sha256: String,
    pub source: String,
    pub status: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPage {
    pub id: String,
    pub title: String,
    pub path: String,
    pub content_sha256: String,
    pub source: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiIngestReport {
    pub root: PathBuf,
    pub seen: usize,
    pub imported: usize,
    pub skipped: usize,
    pub page_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedPageSnapshotInput {
    pub requested_url: String,
    pub final_url: Option<String>,
    pub title: Option<String>,
    pub rendered_html: Option<String>,
    pub rendered_text: Option<String>,
    pub captured_at: Option<String>,
    pub browser: Option<String>,
    pub screenshot_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiSyncReport {
    pub root: PathBuf,
    pub seen: usize,
    pub imported: usize,
    pub skipped: usize,
    pub deleted: usize,
    pub page_ids: Vec<String>,
    pub deleted_page_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiDecisionLedgerEntry {
    pub page_id: String,
    pub page_title: String,
    pub decision: String,
    pub reviewed_source_card_ids: Vec<String>,
    pub source_count: usize,
    pub rationale: String,
    pub follow_up: String,
    pub reviewed_at: String,
    pub first_seen_at: String,
    pub updated_at: String,
    pub source_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiDecisionLedgerSummary {
    pub rows: usize,
    pub pages: usize,
    pub decision_counts: BTreeMap<String, usize>,
    pub newest_reviewed_at: Option<String>,
    pub oldest_reviewed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceClaim {
    pub claim: String,
    pub kind: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCardInput {
    pub title: String,
    pub url: String,
    pub source_type: String,
    pub provider: String,
    pub summary: String,
    pub claims: Vec<SourceClaim>,
    pub retrieved_at: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCard {
    pub id: String,
    pub title: String,
    pub url: String,
    pub source_type: String,
    pub provider: String,
    pub summary: String,
    pub claims: Vec<SourceClaim>,
    pub retrieved_at: String,
    pub wiki_page_id: String,
    pub content_sha256: String,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEventInput {
    pub event_type: String,
    pub title: String,
    pub canonical_key: String,
    pub primary_entity_key: Option<String>,
    pub event_time: Option<String>,
    pub summary: String,
    pub confidence: f64,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEvent {
    pub id: String,
    pub event_type: String,
    pub status: String,
    pub title: String,
    pub canonical_key: String,
    pub primary_entity_key: Option<String>,
    pub event_time: Option<String>,
    pub summary: String,
    pub first_seen_at: String,
    pub last_seen_at: String,
    pub confidence: f64,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEventSourceInput {
    pub event_id: String,
    pub source_card_id: String,
    pub role: String,
    pub confidence: f64,
    pub claim_summary: String,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEventSource {
    pub id: String,
    pub event_id: String,
    pub source_card_id: String,
    pub role: String,
    pub confidence: f64,
    pub claim_summary: String,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeClusterInput {
    pub topic: String,
    pub status: String,
    pub event_ids: Vec<String>,
    pub source_card_ids: Vec<String>,
    pub first_seen_at: Option<String>,
    pub last_seen_at: Option<String>,
    pub novelty_score: f64,
    pub momentum_score: f64,
    pub stale_score: f64,
    pub reason: String,
    pub duplicate_groups: Value,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeCluster {
    pub id: String,
    pub topic: String,
    pub status: String,
    pub source_card_ids: Vec<String>,
    pub event_ids: Vec<String>,
    pub first_seen_at: String,
    pub last_seen_at: String,
    pub novelty_score: f64,
    pub momentum_score: f64,
    pub stale_score: f64,
    pub reason: String,
    pub duplicate_groups: Value,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeClusterProposalModelInput {
    pub source_card_ids: Vec<String>,
    pub model_provider: String,
    pub model_name: Option<String>,
    pub endpoint: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub max_clusters: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeClusterProposalModelInvocation {
    pub clusters: Vec<KnowledgeCluster>,
    pub provider_response: Value,
    pub model_provider: String,
    pub model_name: String,
    pub cost_decision_id: Option<String>,
    pub prompt_version: String,
    pub proof_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeClusterWriterModelInput {
    pub cluster_id: String,
    pub model_provider: String,
    pub model_name: Option<String>,
    pub endpoint: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub create_digest: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeClusterWriterModelInvocation {
    pub markdown: String,
    pub source_card_ids: Vec<String>,
    pub provider_response: Value,
    pub model_provider: String,
    pub model_name: String,
    pub cost_decision_id: Option<String>,
    pub prompt_version: String,
    pub proof_level: String,
    pub score: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeClusterPromotionReport {
    pub cluster: KnowledgeCluster,
    pub editorial_decision: KnowledgeEditorialDecision,
    pub policy_decision_id: Option<String>,
    pub proof_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEditorialDecisionInput {
    pub cluster_id: String,
    pub decision: String,
    pub status: String,
    pub wiki_page_id: Option<String>,
    pub digest_candidate_id: Option<String>,
    pub source_card_ids: Vec<String>,
    pub reason: String,
    pub quality_findings: Vec<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEditorialDecision {
    pub id: String,
    pub cluster_id: String,
    pub decision: String,
    pub status: String,
    pub wiki_page_id: Option<String>,
    pub digest_candidate_id: Option<String>,
    pub source_card_ids: Vec<String>,
    pub reason: String,
    pub quality_findings: Vec<String>,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeReportInput {
    pub cluster_id: String,
    pub title: String,
    pub body_markdown: String,
    pub status: String,
    pub source_card_ids: Vec<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeReport {
    pub id: String,
    pub cluster_id: String,
    pub title: String,
    pub body_markdown: String,
    pub status: String,
    pub source_card_ids: Vec<String>,
    pub quality_findings: Vec<String>,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntityInput {
    pub entity_type: String,
    pub name: String,
    pub canonical_key: String,
    pub aliases: Vec<String>,
    pub homepage_url: Option<String>,
    pub source_card_ids: Vec<String>,
    pub wiki_page_id: Option<String>,
    pub confidence: f64,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntity {
    pub id: String,
    pub entity_type: String,
    pub name: String,
    pub canonical_key: String,
    pub aliases: Vec<String>,
    pub homepage_url: Option<String>,
    pub source_card_ids: Vec<String>,
    pub wiki_page_id: Option<String>,
    pub confidence: f64,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeRelationInput {
    pub relation_type: String,
    pub subject_entity_id: String,
    pub object_entity_id: String,
    pub event_id: Option<String>,
    pub cluster_id: Option<String>,
    pub source_card_ids: Vec<String>,
    pub confidence: f64,
    pub reason: String,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeRelation {
    pub id: String,
    pub relation_key: String,
    pub relation_type: String,
    pub subject_entity_id: String,
    pub object_entity_id: String,
    pub event_id: Option<String>,
    pub cluster_id: Option<String>,
    pub source_card_ids: Vec<String>,
    pub confidence: f64,
    pub reason: String,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeAdapterRun {
    pub id: String,
    pub job_id: String,
    pub adapter_kind: String,
    pub provider: String,
    pub source_kind: String,
    pub locator: String,
    pub status: String,
    pub error_kind: Option<String>,
    pub error: Option<String>,
    pub cursor_key: Option<String>,
    pub cursor_before: Option<String>,
    pub cursor_after: Option<String>,
    pub source_card_ids: Vec<String>,
    pub raw_count: i64,
    pub accepted_count: i64,
    pub rejected_count: i64,
    pub duplicate_count: i64,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntityResolution {
    pub id: String,
    pub left_entity_id: String,
    pub right_entity_id: String,
    pub status: String,
    pub decision: String,
    pub confidence: f64,
    pub resolver: String,
    pub reason: String,
    pub evidence_json: Value,
    pub source_card_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntityResolutionModelInput {
    pub left_entity_id: String,
    pub right_entity_id: String,
    pub model_provider: String,
    pub model_name: Option<String>,
    pub endpoint: Option<String>,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntityResolutionModelInvocation {
    pub resolution: KnowledgeEntityResolution,
    pub provider_response: Value,
    pub model_provider: String,
    pub model_name: String,
    pub cost_decision_id: Option<String>,
    pub prompt_version: String,
    pub proof_level: String,
}

#[derive(Debug, Clone)]
pub(crate) struct KnowledgeEntityResolutionInput {
    pub(crate) left_entity_id: String,
    pub(crate) right_entity_id: String,
    pub(crate) status: String,
    pub(crate) decision: String,
    pub(crate) confidence: f64,
    pub(crate) resolver: String,
    pub(crate) reason: String,
    pub(crate) evidence_json: Value,
    pub(crate) source_card_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeProjectionReport {
    pub topic: String,
    pub proof_level: String,
    pub source_family: String,
    pub source_cards: Vec<SourceCard>,
    pub events: Vec<KnowledgeEvent>,
    pub event_sources: Vec<KnowledgeEventSource>,
    pub entities: Vec<KnowledgeEntity>,
    pub relations: Vec<KnowledgeRelation>,
    pub cluster: KnowledgeCluster,
    pub editorial_decision: KnowledgeEditorialDecision,
    pub report: KnowledgeReport,
    pub warnings: Vec<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeClusterBacklogReport {
    pub inspected: usize,
    pub accepted: usize,
    pub skipped: usize,
    pub groups_considered: usize,
    pub projections: Vec<KnowledgeProjectionReport>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeClusterEditorialDecisionReport {
    pub cluster: KnowledgeCluster,
    pub editorial_decision: KnowledgeEditorialDecision,
    pub recommended_action: String,
    pub matched_wiki_page: Option<WikiPageSummary>,
    pub enqueued_job: Option<WikiJob>,
    pub source_card_count: usize,
    pub proof_level: String,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeClusterExpansionReport {
    pub cluster: KnowledgeCluster,
    pub source_cards: Vec<SourceCard>,
    pub wiki_page: WikiPage,
    pub editorial_decision: KnowledgeEditorialDecision,
    pub report: KnowledgeReport,
    pub digest_candidate: Option<DigestCandidate>,
    pub investigation: KnowledgeClusterInvestigationReport,
    pub quality_findings: Vec<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeClusterInvestigationReport {
    pub cluster: KnowledgeCluster,
    pub research_run: ResearchRun,
    pub tasks: Vec<ResearchTask>,
    pub source_links: Vec<ResearchRunSourceRecord>,
    pub editorial_decision: KnowledgeEditorialDecision,
    pub reused_existing: bool,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeClusterInvestigationExecutionReport {
    pub cluster: KnowledgeCluster,
    pub research_run: ResearchRun,
    pub tasks: Vec<ResearchTask>,
    pub role_runs: Vec<ResearchRoleRun>,
    pub artifacts: Vec<ResearchArtifact>,
    pub editorial_decision: KnowledgeEditorialDecision,
    pub executed_task_count: usize,
    pub already_completed_task_count: usize,
    pub quality_findings: Vec<String>,
    pub metadata: Value,
}
