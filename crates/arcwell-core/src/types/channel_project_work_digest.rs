use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeEvent {
    pub id: String,
    pub source: String,
    pub idempotency_key: String,
    pub status: String,
    pub payload_json: Value,
    pub attempts: i64,
    pub max_attempts: i64,
    pub leased_until: Option<String>,
    pub next_run_at: Option<String>,
    pub error: Option<String>,
    pub received_at: String,
    pub expires_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeRemoteDrainReport {
    pub attempted: usize,
    pub imported: usize,
    pub acked: usize,
    pub nacked: usize,
    pub empty: bool,
    pub events: Vec<EdgeEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramDrainReport {
    pub processed: usize,
    pub acked: usize,
    pub nacked: usize,
    pub messages: Vec<ChannelMessage>,
    pub controller_routes: Vec<ControllerRouteReport>,
    pub controller_route_errors: Vec<String>,
}

pub(crate) struct RecordedTelegramEvent {
    pub(crate) message: ChannelMessage,
    pub(crate) conversation_id: String,
    pub(crate) controller_sender: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramSendReport {
    pub ok: bool,
    pub status: u16,
    pub response: Value,
    pub message: ChannelMessage,
    pub delivery: ChannelDeliveryAttempt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramRetryReport {
    pub attempted: usize,
    pub sent: usize,
    pub failed: usize,
    pub reports: Vec<TelegramSendReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailRetryReport {
    pub attempted: usize,
    pub sent: usize,
    pub failed: usize,
    pub reports: Vec<EmailSendReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailDrainReport {
    pub processed: usize,
    pub acked: usize,
    pub nacked: usize,
    pub messages: Vec<ChannelMessage>,
    pub source_cards: Vec<SourceCard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSendReport {
    pub ok: bool,
    pub status: u16,
    pub response: Value,
    pub message: ChannelMessage,
    pub delivery: ChannelDeliveryAttempt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMessage {
    pub id: String,
    pub channel: String,
    pub direction: String,
    pub project_id: Option<String>,
    pub sender: String,
    pub body: String,
    pub status: String,
    pub source_event_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelAuthorization {
    pub channel: String,
    pub subject: String,
    pub can_read_projects: bool,
    pub can_write_projects: bool,
    pub can_send: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelDeliveryAttempt {
    pub id: String,
    pub message_id: String,
    pub channel: String,
    pub destination: String,
    pub attempt: i64,
    pub ok: bool,
    pub provider_status: i64,
    pub response: Value,
    pub error: Option<String>,
    pub retry_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRecord {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub status: String,
    pub summary: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStatusSnapshot {
    pub id: String,
    pub project_id: String,
    pub status: String,
    pub summary: String,
    pub source: String,
    pub thread_ref: Option<String>,
    pub confidence: f64,
    pub created_at: String,
    pub live_verified: bool,
    pub verified_host: Option<String>,
    pub verified_thread_id: Option<String>,
    pub verified_at: Option<String>,
    pub stale_after_seconds: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStatusProvenance {
    pub source: String,
    pub thread_ref: Option<String>,
    pub timestamp: String,
    pub confidence: f64,
    pub live_verified: bool,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectLiveHostCapability {
    pub host: String,
    pub live_inventory_available: bool,
    pub live_thread_read_available: bool,
    pub manual_snapshot_supported: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectLiveState {
    pub available: bool,
    pub source: String,
    pub checked_at: String,
    pub confidence: f64,
    pub reason: String,
    pub hosts: Vec<ProjectLiveHostCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStatusReport {
    pub project: ProjectRecord,
    pub latest_status: Option<ProjectStatusSnapshot>,
    pub live_state: ProjectLiveState,
    pub provenance: Vec<ProjectStatusProvenance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectResolution {
    pub project: ProjectRecord,
    pub confidence: f64,
    pub matched_alias: Option<String>,
    pub latest_status: Option<ProjectStatusSnapshot>,
    pub live_state: ProjectLiveState,
    pub live_state_available: bool,
    pub live_state_source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerChannelContext {
    pub id: String,
    pub channel: String,
    pub account_id: String,
    pub conversation_id: String,
    pub sender: String,
    pub trust_tier: String,
    pub last_project_id: Option<String>,
    pub last_thread_id: Option<String>,
    pub last_run_id: Option<String>,
    pub last_intent: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerThread {
    pub id: String,
    pub host: String,
    pub host_thread_id: String,
    pub project_id: Option<String>,
    pub title: Option<String>,
    pub cwd: Option<String>,
    pub branch: Option<String>,
    pub worktree: Option<String>,
    pub status: String,
    pub active: bool,
    pub archived: bool,
    pub current_goal: Option<String>,
    pub latest_summary: Option<String>,
    pub latest_summary_source: Option<String>,
    pub last_activity_at: Option<String>,
    pub last_synced_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerRun {
    pub id: String,
    pub thread_id: Option<String>,
    pub project_id: Option<String>,
    pub origin_channel_message_id: Option<String>,
    pub host: String,
    pub host_run_id: Option<String>,
    pub kind: String,
    pub status: String,
    pub requested_action: String,
    pub cancel_requested: bool,
    pub cancel_reason: Option<String>,
    pub started_at: String,
    pub updated_at: String,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerEvent {
    pub id: String,
    pub run_id: Option<String>,
    pub thread_id: Option<String>,
    pub project_id: Option<String>,
    pub event_type: String,
    pub summary: String,
    pub data: Value,
    pub source: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerPendingAction {
    pub id: String,
    pub channel: String,
    pub conversation_id: String,
    pub sender: String,
    pub action_type: String,
    pub project_id: Option<String>,
    pub thread_id: Option<String>,
    pub run_id: Option<String>,
    pub payload: Value,
    pub reason: String,
    pub status: String,
    pub expires_at: String,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerOutboxMessage {
    pub id: String,
    pub channel: String,
    pub target: String,
    pub related_message_id: Option<String>,
    pub run_id: Option<String>,
    pub body: String,
    pub status: String,
    pub idempotency_key: String,
    pub created_at: String,
    pub delivered_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerRouteReport {
    pub intent: String,
    pub confidence: f64,
    pub summary: String,
    pub project: Option<ProjectRecord>,
    pub thread: Option<ControllerThread>,
    pub run: Option<ControllerRun>,
    pub pending_action: Option<ControllerPendingAction>,
    pub context: ControllerChannelContext,
    pub active_runs: Vec<ControllerRun>,
    pub recent_events: Vec<ControllerEvent>,
    pub host_adapter_required: bool,
    pub host_adapter_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkRun {
    pub id: String,
    pub goal: String,
    pub project_id: Option<String>,
    pub host_id: Option<String>,
    pub thread_id: Option<String>,
    pub agent_surface: String,
    pub status: String,
    pub outcome: Option<String>,
    pub validation_summary: Option<String>,
    pub follow_ups: Vec<String>,
    pub reusable_lessons: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkEvent {
    pub id: String,
    pub run_id: String,
    pub event_type: String,
    pub summary: String,
    pub data: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkArtifact {
    pub id: String,
    pub run_id: String,
    pub artifact_type: String,
    pub locator: String,
    pub role: String,
    pub metadata: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkLink {
    pub id: String,
    pub run_id: String,
    pub target_type: String,
    pub target_id: String,
    pub role: String,
    pub generated_summary: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkRunRead {
    pub run: WorkRun,
    pub events: Vec<WorkEvent>,
    pub artifacts: Vec<WorkArtifact>,
    pub links: Vec<WorkLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkConsolidation {
    pub run_id: String,
    pub project_id: Option<String>,
    pub status: String,
    pub summary: String,
    pub evidence: Vec<String>,
    pub warnings: Vec<String>,
    pub project_status: Option<ProjectStatusSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Procedure {
    pub id: String,
    pub title: String,
    pub trigger_context: String,
    pub problem: String,
    pub preconditions: Vec<String>,
    pub tools: Vec<String>,
    pub validation_commands: Vec<String>,
    pub known_risks: Vec<String>,
    pub confidence: f64,
    pub freshness_days: i64,
    pub last_reviewed_at: String,
    pub status: String,
    pub current_version: i64,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureVersion {
    pub id: String,
    pub procedure_id: String,
    pub version: i64,
    pub method: String,
    pub source_run_ids: Vec<String>,
    pub provenance: Value,
    pub artifact_path: PathBuf,
    pub content_sha256: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureRead {
    pub procedure: Procedure,
    pub current: ProcedureVersion,
    pub versions: Vec<ProcedureVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureCandidate {
    pub id: String,
    pub operation: String,
    pub procedure_id: Option<String>,
    pub base_version: Option<i64>,
    pub title: String,
    pub trigger_context: String,
    pub problem: String,
    pub preconditions: Vec<String>,
    pub method: String,
    pub tools: Vec<String>,
    pub validation_commands: Vec<String>,
    pub known_risks: Vec<String>,
    pub source_run_ids: Vec<String>,
    pub provenance: Value,
    pub sensitivity: String,
    pub status: String,
    pub reason: String,
    pub content_sha256: String,
    pub created_at: String,
    pub updated_at: String,
    pub applied_at: Option<String>,
    pub rejected_reason: Option<String>,
    pub applied_result: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureCandidateInput {
    pub operation: String,
    pub procedure_id: Option<String>,
    pub base_version: Option<i64>,
    pub title: String,
    pub trigger_context: String,
    pub problem: String,
    pub preconditions: Vec<String>,
    pub method: String,
    pub tools: Vec<String>,
    pub validation_commands: Vec<String>,
    pub known_risks: Vec<String>,
    pub source_run_ids: Vec<String>,
    pub provenance: Value,
    pub sensitivity: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureCandidateApplyReport {
    pub ok: bool,
    pub candidate_id: String,
    pub operation: String,
    pub procedure_id: Option<String>,
    pub version: Option<i64>,
    pub artifact_path: Option<PathBuf>,
    pub result: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureProposalReport {
    pub run_id: String,
    pub candidates: Vec<ProcedureCandidate>,
    pub auto_approval_blocked: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureCurateReport {
    pub candidates_created: usize,
    pub duplicate_groups: usize,
    pub stale_candidates: usize,
    pub candidates: Vec<ProcedureCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkFollowUp {
    pub run_id: String,
    pub project_id: Option<String>,
    pub host_id: Option<String>,
    pub thread_id: Option<String>,
    pub goal: String,
    pub follow_up: String,
    pub completed_at: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkRetrievalContext {
    pub query: String,
    pub generated_at: String,
    pub stale_runs: Vec<WorkRun>,
    pub consolidation_candidates: Vec<WorkRun>,
    pub follow_ups: Vec<WorkFollowUp>,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobOpsSummary {
    pub profile_count: usize,
    pub evidence_card_count: usize,
    pub source_count: usize,
    pub role_count: usize,
    pub role_status_counts: BTreeMap<String, usize>,
    pub score_tier_counts: BTreeMap<String, usize>,
    pub source_health_counts: BTreeMap<String, usize>,
    pub privacy_decision_counts: BTreeMap<String, usize>,
    pub application_status_counts: BTreeMap<String, usize>,
    pub follow_up_count: usize,
    pub stale_or_closed_roles: Vec<JobRoleCard>,
    pub source_health_failures: Vec<JobSourceHealth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureRetrievalContext {
    pub query: String,
    pub generated_at: String,
    pub procedures: Vec<ProcedureRead>,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureSkillExport {
    pub procedure_id: String,
    pub version: i64,
    pub skill_name: String,
    pub skill_dir: PathBuf,
    pub skill_path: PathBuf,
    pub content_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestCandidate {
    pub id: String,
    pub topic: String,
    pub score: f64,
    pub reason: String,
    pub status: String,
    pub source_card_ids: Vec<String>,
    pub review_status: String,
    pub reviewed_at: Option<String>,
    pub reviewed_by: Option<String>,
    pub review_note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestCandidateDeliveryGate {
    pub candidate: DigestCandidate,
    pub allowed: bool,
    pub reason: String,
    pub policy_decision: PolicyDecisionRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestCandidateTelegramDeliveryReport {
    pub gate: DigestCandidateDeliveryGate,
    pub digest_delivery: DigestDelivery,
    pub telegram: Option<TelegramSendReport>,
    pub replayed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestCandidateEmailDeliveryReport {
    pub gate: DigestCandidateDeliveryGate,
    pub digest_delivery: DigestDelivery,
    pub email: Option<EmailSendReport>,
    pub replayed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestDelivery {
    pub id: String,
    pub candidate_id: String,
    pub channel: String,
    pub subject: String,
    pub target: String,
    pub idempotency_key: String,
    pub status: String,
    pub policy_decision_id: Option<String>,
    pub channel_message_id: Option<String>,
    pub channel_delivery_attempt_id: Option<String>,
    pub error: Option<String>,
    pub retry_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPipelineReport {
    pub candidates_created: usize,
    pub duplicates_suppressed: usize,
    pub candidates: Vec<Candidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRunRecord {
    pub id: String,
    pub source_kind: String,
    pub source_path: String,
    pub mode: String,
    pub status: String,
    pub conversations_seen: usize,
    pub conversations_sampled: usize,
    pub candidates_seen: usize,
    pub candidates_sampled: usize,
    pub candidates_written: usize,
    pub duplicates_suppressed: usize,
    pub error: Option<String>,
    pub metadata: Value,
    pub started_at: String,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRunFinish {
    pub status: String,
    pub conversations_seen: usize,
    pub conversations_sampled: usize,
    pub candidates_seen: usize,
    pub candidates_sampled: usize,
    pub candidates_written: usize,
    pub duplicates_suppressed: usize,
    pub error: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, Serialize)]
pub struct OpsSnapshot {
    pub health: HealthReport,
    pub worker: Option<WorkerHeartbeat>,
    pub x_stats: XStatsReport,
    pub radar_runs: Vec<RadarRun>,
    pub radar_source_quality: Vec<RadarSourceQuality>,
    pub radar_deliveries: Vec<RadarDelivery>,
    pub knowledge_adapter_runs: Vec<KnowledgeAdapterRun>,
    pub knowledge_entities: Vec<KnowledgeEntity>,
    pub knowledge_entity_resolutions: Vec<KnowledgeEntityResolution>,
    pub knowledge_relations: Vec<KnowledgeRelation>,
    pub knowledge_events: Vec<KnowledgeEvent>,
    pub knowledge_clusters: Vec<KnowledgeCluster>,
    pub knowledge_editorial_decisions: Vec<KnowledgeEditorialDecision>,
    pub knowledge_reports: Vec<KnowledgeReport>,
    pub x_knowledge_clusters: Vec<XKnowledgeCluster>,
    pub x_editorial_decisions: Vec<XEditorialDecision>,
    pub jobs: Vec<WikiJob>,
    pub edge_events: Vec<EdgeEvent>,
    pub cursors: Vec<CursorState>,
    pub source_health: Vec<SourceHealth>,
    pub projects: Vec<ProjectRecord>,
    pub project_status_snapshots: Vec<ProjectStatusSnapshot>,
    pub source_cards: Vec<SourceCard>,
    pub watch_sources: Vec<WatchSource>,
    pub channel_messages: Vec<ChannelMessage>,
    pub channel_delivery_attempts: Vec<ChannelDeliveryAttempt>,
    pub digest_candidates: Vec<DigestCandidate>,
    pub digest_deliveries: Vec<DigestDelivery>,
    pub issue_schedules: Vec<IssueSchedule>,
    pub issue_schedule_ticks: Vec<IssueScheduleTick>,
    pub work_runs: Vec<WorkRun>,
    pub procedures: Vec<Procedure>,
    pub procedure_candidates: Vec<ProcedureCandidate>,
    pub job_hunting: JobOpsSummary,
    pub memory_candidates: Vec<Candidate>,
    pub memory_lifecycle_events: Vec<MemoryLifecycleEvent>,
    pub memory_decisions: Vec<MemoryDecisionLedgerEntry>,
    pub memory_forget_tombstones: Vec<MemoryForgetTombstone>,
    pub import_runs: Vec<ImportRunRecord>,
    pub controller_threads: Vec<ControllerThread>,
    pub controller_runs: Vec<ControllerRun>,
    pub controller_events: Vec<ControllerEvent>,
    pub controller_pending_actions: Vec<ControllerPendingAction>,
    pub cost_policies: Vec<CostPolicy>,
    pub cost_decisions: Vec<CostDecisionRecord>,
    pub policy_decisions: Vec<PolicyDecisionRecord>,
    pub policy_approvals: Vec<PolicyApprovalRecord>,
    pub secrets: Vec<SecretRef>,
    pub secret_health: Vec<SecretHealth>,
}
