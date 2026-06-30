use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchHostSearchInput {
    pub run_id: String,
    pub role_run_id: Option<String>,
    pub host: String,
    pub tool_surface: String,
    pub query: String,
    pub query_intent: Option<String>,
    pub requested_recency: Option<i64>,
    pub requested_domains: Vec<String>,
    pub cost_decision_id: Option<String>,
    pub results: Vec<ResearchHostSearchResultInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchHostSearchResultInput {
    pub rank: usize,
    pub title: String,
    pub url: String,
    pub snippet: Option<String>,
    pub published_at: Option<String>,
    pub source_family_guess: Option<String>,
    #[serde(default)]
    pub provider_metadata: Value,
    #[serde(default)]
    pub selected_for_ingest: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchHostSearch {
    pub id: String,
    pub run_id: String,
    pub role_run_id: Option<String>,
    pub host: String,
    pub tool_surface: String,
    pub query: String,
    pub query_intent: Option<String>,
    pub requested_recency: Option<i64>,
    pub requested_domains: Vec<String>,
    pub executed_at: String,
    pub retrieved_at: String,
    pub cost_decision_id: Option<String>,
    pub result_count: usize,
    pub status: String,
    pub error_kind: Option<String>,
    pub error_message_redacted: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchHostSearchResult {
    pub id: String,
    pub host_search_id: String,
    pub rank: usize,
    pub title: String,
    pub url: String,
    pub canonical_url: String,
    pub snippet: Option<String>,
    pub published_at: Option<String>,
    pub source_family_guess: Option<String>,
    pub provider_metadata: Value,
    pub selected_for_ingest: bool,
    pub research_source_id: Option<String>,
    pub source_card_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchHostSearchRecord {
    pub search: ResearchHostSearch,
    pub results: Vec<ResearchHostSearchResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchDocumentInput {
    pub run_id: String,
    pub research_source_id: Option<String>,
    pub source_card_id: Option<String>,
    pub path: PathBuf,
    pub media_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchDocument {
    pub id: String,
    pub run_id: String,
    pub research_source_id: Option<String>,
    pub source_card_id: Option<String>,
    pub url: Option<String>,
    pub local_path: Option<String>,
    pub media_type: String,
    pub byte_sha256: String,
    pub byte_len: u64,
    pub retrieved_at: String,
    pub extractor_name: String,
    pub extractor_version: String,
    pub extraction_status: String,
    pub page_count: usize,
    pub sheet_count: usize,
    pub table_count: usize,
    pub warning_flags: Vec<String>,
    pub error_message_redacted: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchDocumentSpan {
    pub id: String,
    pub document_id: String,
    pub span_id: String,
    pub page_number: Option<usize>,
    pub section_label: Option<String>,
    pub char_start: usize,
    pub char_end: usize,
    pub text_sha256: String,
    pub text_excerpt: String,
    pub bbox_json: Option<Value>,
    pub confidence: f64,
    pub warning_flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchTable {
    pub id: String,
    pub document_id: String,
    pub table_id: String,
    pub page_number: Option<usize>,
    pub sheet_name: Option<String>,
    pub caption: Option<String>,
    pub bbox_json: Option<Value>,
    pub row_count: usize,
    pub column_count: usize,
    pub extraction_method: String,
    pub confidence: f64,
    pub warning_flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchTableCell {
    pub id: String,
    pub table_id: String,
    pub row_index: usize,
    pub column_index: usize,
    pub row_header: Option<String>,
    pub column_header: Option<String>,
    pub raw_text: String,
    pub normalized_text: String,
    pub numeric_value: Option<f64>,
    pub unit: Option<String>,
    pub footnote_refs: Vec<String>,
    pub bbox_json: Option<Value>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchTableRecord {
    pub table: ResearchTable,
    pub cells: Vec<ResearchTableCell>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchDocumentRecord {
    pub document: ResearchDocument,
    pub spans: Vec<ResearchDocumentSpan>,
    pub tables: Vec<ResearchTableRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchEditorialRunInput {
    pub run_id: String,
    pub stage: String,
    pub model_provider: String,
    pub model_name: String,
    pub prompt_version: String,
    pub input_artifact_id: Option<String>,
    pub output_artifact_id: Option<String>,
    pub cost_decision_id: Option<String>,
    pub status: String,
    pub score: Value,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchEditorialInvokeInput {
    pub run_id: String,
    pub stage: String,
    pub model_provider: String,
    pub model_name: Option<String>,
    pub prompt_version: String,
    pub input_artifact_id: Option<String>,
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchEditorialInvocation {
    pub editorial_run: ResearchEditorialRun,
    pub output_artifact: Option<ResearchArtifact>,
    pub provider_response: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchEditorialRun {
    pub id: String,
    pub run_id: String,
    pub stage: String,
    pub model_provider: String,
    pub model_name: String,
    pub prompt_version: String,
    pub input_artifact_hash: Option<String>,
    pub input_artifact_id: Option<String>,
    pub output_artifact_id: Option<String>,
    pub cost_decision_id: Option<String>,
    pub status: String,
    pub score: Value,
    pub error_message_redacted: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchConfig {
    pub provider: String,
    pub max_results: usize,
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub provider: String,
    pub rank: usize,
    pub retrieved_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResponse {
    pub query: String,
    pub provider: String,
    pub results: Vec<WebSearchResult>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XItem {
    pub id: String,
    pub x_id: String,
    pub author: String,
    pub text: String,
    pub url: String,
    pub created_at: Option<String>,
    pub imported_at: String,
    pub retrieved_at: Option<String>,
    pub metrics: Value,
    pub raw: Value,
    pub source_card_id: Option<String>,
    pub wiki_page_id: Option<String>,
    pub sources: Vec<XItemSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XThreadReport {
    pub generated_at: String,
    pub mode: String,
    pub root_x_id: String,
    pub conversation_id: Option<String>,
    pub max_depth: usize,
    pub tweets: Vec<XThreadTweet>,
    pub missing_context: Vec<XThreadMissingContext>,
    pub cycle_detected: bool,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XThreadTweet {
    pub x_id: String,
    pub author: String,
    pub text: String,
    pub url: String,
    pub created_at: Option<String>,
    pub first_seen_at: String,
    pub conversation_id: Option<String>,
    pub reply_to_x_id: Option<String>,
    pub quote_x_id: Option<String>,
    pub retweet_x_id: Option<String>,
    pub relation_to_root: String,
    pub depth: usize,
    pub source_card_id: Option<String>,
    pub wiki_page_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct XThreadMissingContext {
    pub tweet_x_id: String,
    pub ref_kind: String,
    pub ref_x_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XLinkIndexReport {
    pub generated_at: String,
    pub limit: usize,
    pub tweets_scanned: usize,
    pub links_indexed: usize,
    pub skipped_unsafe: usize,
    pub links: Vec<XLinkOccurrence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XLinkOccurrence {
    pub tweet_x_id: String,
    pub url: String,
    pub expanded_url: Option<String>,
    pub display_url: Option<String>,
    pub source: String,
    pub first_seen_at: String,
    pub last_seen_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XLinkExpansionReport {
    pub generated_at: String,
    pub limit: usize,
    pub candidates: usize,
    pub expanded: usize,
    pub already_completed: usize,
    pub failed: usize,
    pub items: Vec<XLinkExpansionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XLinkExpansionItem {
    pub url: String,
    pub status: String,
    pub wiki_page_id: Option<String>,
    pub final_url: Option<String>,
    pub canonical_url: Option<String>,
    pub content_type: Option<String>,
    pub bytes: Option<usize>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XItemSource {
    pub id: String,
    pub x_id: String,
    pub source_kind: String,
    pub source_detail: Option<String>,
    pub seen_at: String,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XImportReport {
    pub seen: usize,
    pub imported: usize,
    pub skipped_duplicates: usize,
    pub rejected: usize,
    pub rejected_errors: Vec<String>,
    pub pages_fetched: Option<usize>,
    pub requested_limit: Option<usize>,
    pub exhausted: Option<bool>,
    pub stop_reason: Option<String>,
    pub next_token: Option<String>,
    pub source_card_projections: Option<usize>,
    pub drift_warnings: Vec<String>,
    pub items: Vec<XItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XKnowledgeCluster {
    pub id: String,
    pub topic: String,
    pub status: String,
    pub source_card_ids: Vec<String>,
    pub radar_run_id: Option<String>,
    pub radar_item_ids: Vec<String>,
    pub first_seen_at: String,
    pub last_seen_at: String,
    pub novelty_score: f64,
    pub momentum_score: f64,
    pub stale_score: f64,
    pub reason: String,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XEditorialDecision {
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
pub struct XArchiveImportReport {
    pub path: String,
    pub selected: Vec<String>,
    pub files_seen: usize,
    pub files_imported: usize,
    pub bytes_read: usize,
    pub skipped_files: usize,
    pub unsupported_slices: BTreeMap<String, usize>,
    pub unsupported_files: Vec<String>,
    pub warnings: Vec<String>,
    pub import: XImportReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XArchiveDiscoveryReport {
    pub generated_at: String,
    pub roots: Vec<String>,
    pub inspected_paths: usize,
    pub candidates: Vec<XArchiveDiscoveryCandidate>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XArchiveDiscoveryCandidate {
    pub path: String,
    pub kind: String,
    pub score: f64,
    pub size_bytes: Option<u64>,
    pub modified_at: Option<String>,
    pub supported_slices: Vec<String>,
    pub evidence: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XPortableShardReport {
    pub path: String,
    pub rows: usize,
    pub bytes: usize,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XPortableExportReport {
    pub out_dir: String,
    pub manifest_path: String,
    pub generated_at: String,
    pub rows_exported: usize,
    pub shards: Vec<XPortableShardReport>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XPortableValidateReport {
    pub dir: String,
    pub manifest_path: String,
    pub valid: bool,
    pub rows: usize,
    pub shards: Vec<XPortableShardReport>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XPortableImportReport {
    pub dir: String,
    pub validation: XPortableValidateReport,
    pub import: XImportReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XMonitorSourceReport {
    pub handle: String,
    pub cursor_key: String,
    pub previous_cursor: Option<String>,
    pub newest_id: Option<String>,
    pub effective_cursor: Option<String>,
    pub seen: usize,
    pub imported: usize,
    pub skipped_duplicates: usize,
    pub rejected: usize,
    pub digest_candidate_id: Option<String>,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XMonitorReport {
    pub watched_sources: usize,
    pub polled_sources: usize,
    pub attempted_sources: usize,
    pub deferred_sources: usize,
    pub imported: usize,
    pub skipped_duplicates: usize,
    pub rejected: usize,
    pub failed_sources: usize,
    pub rate_limited_sources: usize,
    pub digest_candidates: usize,
    pub stopped_reason: Option<String>,
    pub sources: Vec<XMonitorSourceReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XFollowingWatchImportReport {
    pub seen: usize,
    pub imported: usize,
    pub added: usize,
    pub updated: usize,
    pub unchanged: usize,
    pub rejected: usize,
    pub next_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XDefinitiveWatchReport {
    pub removed_previous: usize,
    pub bookmark_tweets_seen: usize,
    pub bookmark_tweets_within_window: usize,
    pub bookmark_authors: usize,
    pub recent_follows_seen: usize,
    pub recent_follow_authors: usize,
    pub final_handles: usize,
    pub rejected: usize,
    pub bookmark_since: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XWatchCurationRun {
    pub id: String,
    pub classifier_version: String,
    pub mode: String,
    pub status: String,
    pub input_count: usize,
    pub keep_count: usize,
    pub review_keep_leaning_count: usize,
    pub review_drop_leaning_count: usize,
    pub needs_profile_enrichment_count: usize,
    pub pause_candidate_count: usize,
    pub paused_count: usize,
    pub restored_count: usize,
    pub error: Option<String>,
    pub metadata: Value,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XWatchCurationDecision {
    pub id: String,
    pub run_id: String,
    pub watch_source_id: String,
    pub handle: String,
    pub previous_status: String,
    pub proposed_status: String,
    pub recommendation: String,
    pub category: String,
    pub score: i64,
    pub confidence: f64,
    pub reason: String,
    pub evidence: Value,
    pub created_at: String,
    pub applied_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XWatchCurationReport {
    pub proof_level: String,
    pub run: XWatchCurationRun,
    pub counts: BTreeMap<String, usize>,
    pub decisions: Vec<XWatchCurationDecision>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XWatchCurationRestoreReport {
    pub proof_level: String,
    pub run_id: String,
    pub restored_count: usize,
    pub restored_watch_source_ids: Vec<String>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XWatchManualRuleInput {
    pub handle: String,
    pub decision: String,
    pub category: String,
    pub reason: String,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XWatchManualRuleImportReport {
    pub proof_level: String,
    pub reviewed_by: String,
    pub dry_run: bool,
    pub seen: usize,
    pub imported: usize,
    pub updated: usize,
    pub rejected: usize,
    pub items: Vec<XWatchManualRuleImportItem>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XWatchManualRuleImportItem {
    pub handle: String,
    pub decision: String,
    pub category: String,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XProfileEnrichmentReport {
    pub proof_level: String,
    pub requested: usize,
    pub fetched: usize,
    pub updated: usize,
    pub not_found: usize,
    pub failed_batches: usize,
    pub source_health_keys: Vec<String>,
    pub sync_run_ids: Vec<String>,
    pub items: Vec<XProfileEnrichmentItem>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XProfileEnrichmentItem {
    pub handle: String,
    pub profile_id: Option<String>,
    pub x_user_id: Option<String>,
    pub status: String,
    pub source_health_key: String,
    pub display_name_present: bool,
    pub description_present: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XReport {
    pub query: Option<String>,
    pub items: Vec<XItem>,
    pub links: Vec<XReportLink>,
    pub markdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XReportLink {
    pub tweet_x_id: String,
    pub url: String,
    pub display_url: Option<String>,
    pub source: String,
    pub expansion_status: String,
    pub wiki_page_id: Option<String>,
    pub final_url: Option<String>,
    pub canonical_url: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XResearchBrief {
    pub query: String,
    pub generated_at: String,
    pub no_write: bool,
    pub items: Vec<XResearchBriefItem>,
    pub markdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XResearchBriefItem {
    pub x_id: String,
    pub author: String,
    pub url: String,
    pub created_at: Option<String>,
    pub source_card_id: String,
    pub wiki_page_id: Option<String>,
    pub quote: String,
    pub thread_context: Vec<XResearchBriefThreadTweet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XResearchBriefThreadTweet {
    pub x_id: String,
    pub author: String,
    pub url: String,
    pub relation_to_root: String,
    pub depth: usize,
    pub source_card_id: String,
    pub quote: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XFtsRebuildReport {
    pub tweets_indexed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XProjectionRepairReport {
    pub generated_at: String,
    pub limit: usize,
    pub candidates: usize,
    pub repaired: usize,
    pub already_completed: usize,
    pub failed: usize,
    pub items: Vec<XProjectionRepairItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XProjectionRepairItem {
    pub x_id: String,
    pub status: String,
    pub source_card_id: Option<String>,
    pub wiki_page_id: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XStatsReport {
    pub generated_at: String,
    pub compatibility: XCompatibilityStats,
    pub canonical: XCanonicalStats,
    pub drift: XStatsDrift,
    pub portable_export: XPortableExportFreshness,
    pub projections_by_status: BTreeMap<String, i64>,
    pub digest_projections_by_status: BTreeMap<String, i64>,
    pub digest_candidates_linked_to_x: i64,
    pub sync_runs_by_status: BTreeMap<String, i64>,
    pub unresolved_failed_sync_runs: i64,
    pub source_health_by_status: BTreeMap<String, i64>,
    pub watch_sources_by_status: BTreeMap<String, i64>,
    pub latest_sync_runs: Vec<XSyncRunSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XCompatibilityStats {
    pub x_items: i64,
    pub x_item_sources: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XCanonicalStats {
    pub accounts: i64,
    pub profiles: i64,
    pub profile_snapshots: i64,
    pub profile_entities: i64,
    pub tweets: i64,
    pub tweet_refs: i64,
    pub tweet_edges: i64,
    pub collections: i64,
    pub projections: i64,
    pub sync_runs: i64,
    pub fts_rows: i64,
    pub x_cursors: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XStatsDrift {
    pub compatibility_without_canonical: i64,
    pub canonical_without_compatibility: i64,
    pub tweets_without_fts: i64,
    pub fts_without_tweets: i64,
    pub projection_failures: i64,
    pub non_healthy_sources: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XPortableExportFreshness {
    pub status: String,
    pub missing: bool,
    pub stale: bool,
    pub latest_completed_at: Option<String>,
    pub latest_out_dir: Option<String>,
    pub latest_manifest_path: Option<String>,
    pub latest_manifest_sha256: Option<String>,
    pub latest_rows_exported: Option<usize>,
    pub latest_failed_at: Option<String>,
    pub latest_error: Option<String>,
    pub current_tweet_count: i64,
    pub latest_tweet_updated_at: Option<String>,
    pub tweets_updated_after_export: i64,
    pub row_count_mismatch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XSyncRunSummary {
    pub id: String,
    pub account_id: Option<String>,
    pub stream: String,
    pub transport: String,
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub seen: usize,
    pub inserted: usize,
    pub updated: usize,
    pub skipped_duplicates: usize,
    pub rejected: usize,
    pub cursor_key: Option<String>,
    pub previous_cursor: Option<String>,
    pub new_cursor: Option<String>,
    pub error: Option<String>,
}

pub(crate) struct XSyncRunInsert<'a> {
    pub(crate) account_id: Option<&'a str>,
    pub(crate) stream: &'a str,
    pub(crate) transport: &'a str,
    pub(crate) status: &'a str,
    pub(crate) started_at: &'a str,
    pub(crate) completed_at: &'a str,
    pub(crate) seen: usize,
    pub(crate) inserted: usize,
    pub(crate) updated: usize,
    pub(crate) skipped_duplicates: usize,
    pub(crate) rejected: usize,
    pub(crate) cursor_key: Option<&'a str>,
    pub(crate) previous_cursor: Option<&'a str>,
    pub(crate) new_cursor: Option<&'a str>,
    pub(crate) error: Option<&'a str>,
    pub(crate) metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XOAuthStart {
    pub authorization_url: String,
    pub state: String,
    pub code_verifier: String,
    pub code_challenge: String,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XOAuthTokenStoreReport {
    pub stored: Vec<String>,
    pub token_type: Option<String>,
    pub expires_in: Option<i64>,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XOAuthRevocationReport {
    pub secret_name: String,
    pub token_type_hint: Option<String>,
    pub provider_status: u16,
    pub revoked_provider_side: bool,
    pub deleted_local_secret: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XOAuthScopeProbeEndpoint {
    pub name: String,
    pub required_scope: String,
    pub path: String,
    pub status: String,
    pub classification: String,
    pub evidence: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XOAuthScopeProbeReport {
    pub status: String,
    pub account_id: Option<String>,
    pub username: Option<String>,
    pub endpoints: Vec<XOAuthScopeProbeEndpoint>,
    pub required_scopes: Vec<String>,
    pub missing_or_unproven_scopes: Vec<String>,
    pub source_health_key: String,
    pub sync_run_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XOAuthReauthorizePreflightReport {
    pub status: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCredentialProbeEndpoint {
    pub provider: String,
    pub secret_name: Option<String>,
    pub endpoint: String,
    pub status: String,
    pub classification: String,
    pub evidence: String,
    pub error: Option<String>,
    pub source_health_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCredentialProbeReport {
    pub status: String,
    pub checked_at: String,
    pub providers_requested: Vec<String>,
    pub endpoints: Vec<ProviderCredentialProbeEndpoint>,
    pub source_health_keys: Vec<String>,
    pub missing_or_failed_providers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRateLimitDeferReport {
    pub scanned: usize,
    pub deferred: usize,
    pub defer_until: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XHealthRepairReport {
    pub repaired_bookmark_health: usize,
    pub repaired_watch_health: usize,
    pub rate_limited_scanned: usize,
    pub rate_limited_deferred: usize,
    pub defer_until: String,
}
