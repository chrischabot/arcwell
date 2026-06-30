use super::*;

pub struct CommerceRunConfigInput {
    pub run_id: String,
    pub domain_profile: String,
    pub target_qualified_count: usize,
    pub geography: Option<String>,
    pub freshness_window: String,
    pub allowed_private_context_sources: Vec<String>,
    pub allowed_public_source_families: Vec<String>,
    pub allow_marketplaces: bool,
    pub allow_chrome_profile: bool,
    pub max_provider_calls: Option<usize>,
    pub max_browser_pages: Option<usize>,
    pub max_cost_usd: Option<f64>,
    pub stop_rules: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommerceRunConfig {
    pub run_id: String,
    pub domain_profile: String,
    pub target_qualified_count: usize,
    pub geography: Option<String>,
    pub freshness_window: String,
    pub allowed_private_context_sources: Vec<String>,
    pub allowed_public_source_families: Vec<String>,
    pub allow_marketplaces: bool,
    pub allow_chrome_profile: bool,
    pub max_provider_calls: Option<usize>,
    pub max_browser_pages: Option<usize>,
    pub max_cost_usd: Option<f64>,
    pub stop_rules: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommerceCandidateInput {
    pub run_id: String,
    pub domain: String,
    pub source_url: String,
    pub retailer_or_provider: String,
    pub title: String,
    pub normalized_item_key: String,
    pub variant_key: String,
    pub price: Option<String>,
    pub currency: Option<String>,
    pub geography: Option<String>,
    pub candidate_status: String,
    pub score: Option<f64>,
    pub score_reasons: Value,
    pub disqualification_reasons: Value,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommerceCandidate {
    pub id: String,
    pub run_id: String,
    pub domain: String,
    pub source_url: String,
    pub retailer_or_provider: String,
    pub title: String,
    pub normalized_item_key: String,
    pub variant_key: String,
    pub price: Option<String>,
    pub currency: Option<String>,
    pub geography: Option<String>,
    pub candidate_status: String,
    pub score: Option<f64>,
    pub score_reasons: Value,
    pub disqualification_reasons: Value,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommerceAvailabilityProofInput {
    pub run_id: String,
    pub candidate_id: String,
    pub proof_method: String,
    pub variant_key: String,
    pub variant_label: String,
    pub availability_state: String,
    pub visible_evidence: Option<String>,
    pub selector_or_dom_hint: Option<String>,
    pub screenshot_artifact_id: Option<String>,
    pub page_snapshot_artifact_id: Option<String>,
    pub confidence: f64,
    pub caveats: Value,
    pub checked_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommerceAvailabilityProof {
    pub id: String,
    pub run_id: String,
    pub candidate_id: String,
    pub proof_method: String,
    pub variant_key: String,
    pub variant_label: String,
    pub availability_state: String,
    pub visible_evidence: Option<String>,
    pub selector_or_dom_hint: Option<String>,
    pub screenshot_artifact_id: Option<String>,
    pub page_snapshot_artifact_id: Option<String>,
    pub confidence: f64,
    pub caveats: Value,
    pub checked_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommerceContextFactInput {
    pub run_id: String,
    pub fact_key: String,
    pub fact_kind: String,
    pub redacted_value: String,
    pub source_family: String,
    pub source_ref: Option<String>,
    pub confidence: f64,
    pub user_confirmed: bool,
    pub may_persist_to_memory: bool,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommerceContextFact {
    pub id: String,
    pub run_id: String,
    pub fact_key: String,
    pub fact_kind: String,
    pub redacted_value: String,
    pub source_family: String,
    pub source_ref: Option<String>,
    pub confidence: f64,
    pub user_confirmed: bool,
    pub may_persist_to_memory: bool,
    pub metadata: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommerceVerificationAttemptInput {
    pub run_id: String,
    pub candidate_id: String,
    pub method: String,
    pub result: String,
    pub error_kind: Option<String>,
    pub final_url: Option<String>,
    pub http_status: Option<i64>,
    pub browser_required: bool,
    pub chrome_profile_required: bool,
    pub artifact_ids: Vec<String>,
    pub next_action: Option<String>,
    pub attempted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommerceVerificationAttempt {
    pub id: String,
    pub run_id: String,
    pub candidate_id: String,
    pub attempted_at: String,
    pub method: String,
    pub result: String,
    pub error_kind: Option<String>,
    pub final_url: Option<String>,
    pub http_status: Option<i64>,
    pub browser_required: bool,
    pub chrome_profile_required: bool,
    pub artifact_ids: Vec<String>,
    pub next_action: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommerceReportJudgmentInput {
    pub run_id: String,
    pub decision: String,
    pub blocking_findings: Value,
    pub non_blocking_findings: Value,
    pub claims_checked: Value,
    pub availability_proofs_checked: Value,
    pub privacy_review: Value,
    pub remaining_risks: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommerceReportJudgment {
    pub id: String,
    pub run_id: String,
    pub decision: String,
    pub blocking_findings: Value,
    pub non_blocking_findings: Value,
    pub claims_checked: Value,
    pub availability_proofs_checked: Value,
    pub privacy_review: Value,
    pub remaining_risks: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommerceRenderedPageCheckInput {
    pub run_id: String,
    pub candidate_id: String,
    pub variant_key: String,
    pub variant_label: String,
    pub snapshot: RenderedPageSnapshotInput,
    pub selector_or_dom_hint: Option<String>,
    pub chrome_profile_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommerceRenderedPageCheck {
    pub candidate: CommerceCandidate,
    pub page_snapshot_artifact: ResearchArtifact,
    pub source_card: SourceCard,
    pub research_source_link: ResearchRunSourceRecord,
    pub verification_attempt: CommerceVerificationAttempt,
    pub availability_proof: CommerceAvailabilityProof,
    pub availability_state: String,
    pub visible_evidence: Option<String>,
    pub extracted_price: Option<String>,
    pub extracted_currency: Option<String>,
    pub shipping_caveat: Option<String>,
    pub checked_at: String,
    pub caveats: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommerceContextPacket {
    pub run_id: String,
    pub artifact: ResearchArtifact,
    pub fact_count: usize,
    pub missing_fact_count: usize,
    pub user_confirmed_count: usize,
    pub may_persist_to_memory_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommerceReport {
    pub run_id: String,
    pub artifact: ResearchArtifact,
    pub judgment: CommerceReportJudgment,
    pub recommended_count: usize,
    pub unavailable_count: usize,
    pub blocked_count: usize,
    pub unknown_count: usize,
    pub context_fact_count: usize,
    pub source_card_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobCandidateProfileInput {
    pub label: String,
    pub current_resume_source: Option<String>,
    pub linkedin_source: Option<String>,
    pub github_profile: Option<String>,
    pub blog_url: Option<String>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobCandidateProfile {
    pub id: String,
    pub label: String,
    pub current_resume_source: Option<String>,
    pub linkedin_source: Option<String>,
    pub github_profile: Option<String>,
    pub blog_url: Option<String>,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobEvidenceCardInput {
    pub profile_id: String,
    pub title: String,
    pub evidence_type: String,
    pub visibility: String,
    pub summary: String,
    pub proof_url: Option<String>,
    pub local_path: Option<String>,
    pub source_date: Option<String>,
    pub confidence: String,
    pub tags: Vec<String>,
    pub safe_application_text: String,
    pub unsafe_terms: Vec<String>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobEvidenceCard {
    pub id: String,
    pub profile_id: String,
    pub title: String,
    pub evidence_type: String,
    pub visibility: String,
    pub summary: String,
    pub proof_url: Option<String>,
    pub local_path: Option<String>,
    pub source_date: Option<String>,
    pub confidence: String,
    pub tags: Vec<String>,
    pub safe_application_text: String,
    pub unsafe_terms: Vec<String>,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobEvidenceClaimInput {
    pub evidence_card_id: String,
    pub claim: String,
    pub claim_kind: String,
    pub proof_level: String,
    pub can_use_in_resume: bool,
    pub can_use_in_outreach: bool,
    pub can_use_in_interview: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobEvidenceClaim {
    pub id: String,
    pub evidence_card_id: String,
    pub claim: String,
    pub claim_kind: String,
    pub proof_level: String,
    pub can_use_in_resume: bool,
    pub can_use_in_outreach: bool,
    pub can_use_in_interview: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobEvidenceReviewFinding {
    pub severity: String,
    pub finding_type: String,
    pub evidence_card_id: Option<String>,
    pub claim_id: Option<String>,
    pub message: String,
    pub next_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobEvidenceReviewReport {
    pub profile_id: String,
    pub generated_at: String,
    pub decision: String,
    pub evidence_card_count: usize,
    pub claim_count: usize,
    pub counts_by_visibility: BTreeMap<String, usize>,
    pub counts_by_evidence_type: BTreeMap<String, usize>,
    pub counts_by_confidence: BTreeMap<String, usize>,
    pub counts_by_proof_level: BTreeMap<String, usize>,
    pub claim_use_counts: BTreeMap<String, usize>,
    pub privacy_decision_counts: BTreeMap<String, usize>,
    pub ready_card_ids: Vec<String>,
    pub needs_review_card_ids: Vec<String>,
    pub blocked_card_ids: Vec<String>,
    pub findings: Vec<JobEvidenceReviewFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPrivacyRuleInput {
    pub pattern: String,
    pub rule_type: String,
    pub severity: String,
    pub replacement_guidance: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPrivacyRule {
    pub id: String,
    pub pattern: String,
    pub rule_type: String,
    pub severity: String,
    pub replacement_guidance: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPrivacyFinding {
    pub rule_id: Option<String>,
    pub pattern: String,
    pub rule_type: String,
    pub severity: String,
    pub replacement_guidance: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPrivacyCheck {
    pub id: String,
    pub artifact_type: String,
    pub artifact_id: Option<String>,
    pub decision: String,
    pub findings: Vec<JobPrivacyFinding>,
    pub checked_text_hash: String,
    pub checked_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSourceInput {
    pub source_family: String,
    pub name: String,
    pub url: String,
    pub market_scope: String,
    pub refresh_policy: String,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSource {
    pub id: String,
    pub source_family: String,
    pub name: String,
    pub url: String,
    pub market_scope: String,
    pub refresh_policy: String,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSourceHealthInput {
    pub source_id: String,
    pub status: String,
    pub http_status: Option<i64>,
    pub error_code: Option<String>,
    pub fetched_count: usize,
    pub accepted_count: usize,
    pub rejected_count: usize,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSourceHealth {
    pub id: String,
    pub source_id: String,
    pub checked_at: String,
    pub status: String,
    pub http_status: Option<i64>,
    pub error_code: Option<String>,
    pub fetched_count: usize,
    pub accepted_count: usize,
    pub rejected_count: usize,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRoleCardInput {
    pub company: String,
    pub role_title: String,
    pub canonical_url: Option<String>,
    pub source_family: String,
    pub source_url: String,
    pub source_confidence: String,
    pub date_accessed: Option<String>,
    pub posting_freshness: String,
    pub location: Option<String>,
    pub work_mode: Option<String>,
    pub company_stage_or_size: Option<String>,
    pub role_seniority: Option<String>,
    pub core_requirements: Vec<String>,
    pub implied_business_problem: Option<String>,
    pub why_they_might_need_user: Option<String>,
    pub evidence_card_ids: Vec<String>,
    pub gaps_or_blockers: Vec<String>,
    pub cluster: Option<String>,
    pub current_status: String,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRoleCard {
    pub id: String,
    pub company: String,
    pub role_title: String,
    pub canonical_url: Option<String>,
    pub source_family: String,
    pub source_url: String,
    pub source_confidence: String,
    pub date_accessed: String,
    pub posting_freshness: String,
    pub location: Option<String>,
    pub work_mode: Option<String>,
    pub company_stage_or_size: Option<String>,
    pub role_seniority: Option<String>,
    pub core_requirements: Vec<String>,
    pub implied_business_problem: Option<String>,
    pub why_they_might_need_user: Option<String>,
    pub evidence_card_ids: Vec<String>,
    pub gaps_or_blockers: Vec<String>,
    pub cluster: Option<String>,
    pub current_status: String,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRoleSourceLinkInput {
    pub role_id: String,
    pub source_id: Option<String>,
    pub source_url: String,
    pub confidence: String,
    pub evidence_excerpt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRoleSourceLink {
    pub id: String,
    pub role_id: String,
    pub source_id: Option<String>,
    pub source_url: String,
    pub observed_at: String,
    pub confidence: String,
    pub evidence_excerpt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobFitScoreInput {
    pub role_id: String,
    pub profile_id: String,
    pub scorer: String,
    pub role_fit: f64,
    pub domain_fit: f64,
    pub evidence_fit: f64,
    pub geo_work_fit: f64,
    pub stage_fit: f64,
    pub practical_odds: f64,
    pub interest_energy: f64,
    pub blockers: Vec<String>,
    pub evidence_card_ids: Vec<String>,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobFitScore {
    pub id: String,
    pub role_id: String,
    pub profile_id: String,
    pub scored_at: String,
    pub scorer: String,
    pub role_fit: f64,
    pub domain_fit: f64,
    pub evidence_fit: f64,
    pub geo_work_fit: f64,
    pub stage_fit: f64,
    pub practical_odds: f64,
    pub interest_energy: f64,
    pub weighted_score: f64,
    pub tier: String,
    pub blockers: Vec<String>,
    pub evidence_card_ids: Vec<String>,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSkepticFindingInput {
    pub role_id: String,
    pub severity: String,
    pub finding_type: String,
    pub finding: String,
    pub next_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSkepticFinding {
    pub id: String,
    pub role_id: String,
    pub severity: String,
    pub finding_type: String,
    pub finding: String,
    pub next_action: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobApplicationPacketInput {
    pub role_id: String,
    pub profile_id: String,
    pub evidence_card_ids: Vec<String>,
    pub resume_emphasis: String,
    pub tailored_bullets: Vec<String>,
    pub outreach_note: String,
    #[serde(default)]
    pub proof_links: Value,
    pub likely_objections: Vec<String>,
    pub interview_stories: Vec<String>,
    pub questions_to_ask: Vec<String>,
    pub reviewer_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobApplicationPacket {
    pub id: String,
    pub role_id: String,
    pub profile_id: String,
    pub generated_at: String,
    pub status: String,
    pub evidence_card_ids: Vec<String>,
    pub resume_emphasis: String,
    pub tailored_bullets: Vec<String>,
    pub outreach_note: String,
    pub proof_links: Value,
    pub likely_objections: Vec<String>,
    pub interview_stories: Vec<String>,
    pub questions_to_ask: Vec<String>,
    pub privacy_check_id: String,
    pub reviewer_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobApplicationPacketStatusInput {
    pub packet_id: String,
    pub status: String,
    pub reviewer_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobApplicationPacketExport {
    pub packet_id: String,
    pub role_id: String,
    pub profile_id: String,
    pub path: String,
    pub byte_len: usize,
    pub sha256: String,
    pub privacy_check_id: String,
    pub proof_level: String,
    pub delivery_status: String,
    pub application_status_changed: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobApplicationPacketSetExport {
    pub profile_id: String,
    pub packet_ids: Vec<String>,
    pub out_dir: String,
    pub manifest_path: String,
    pub exported_count: usize,
    pub total_byte_len: usize,
    pub export_set_sha256: String,
    pub exports: Vec<JobApplicationPacketExport>,
    pub proof_level: String,
    pub delivery_status: String,
    pub application_status_changed: bool,
    pub warnings: Vec<String>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobCompanyCardInput {
    pub company_name: String,
    pub website_url: String,
    pub source_family: String,
    pub market: String,
    pub stage: Option<String>,
    pub funding_signal: Option<String>,
    pub product_category: Option<String>,
    pub technical_audience: Option<String>,
    pub developer_facing_score: f64,
    pub london_relevance: String,
    pub remote_maturity: Option<String>,
    pub hiring_page_url: Option<String>,
    pub founder_or_team_signal: Option<String>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobCompanyCard {
    pub id: String,
    pub company_name: String,
    pub website_url: String,
    pub source_family: String,
    pub market: String,
    pub stage: Option<String>,
    pub funding_signal: Option<String>,
    pub product_category: Option<String>,
    pub technical_audience: Option<String>,
    pub developer_facing_score: f64,
    pub london_relevance: String,
    pub remote_maturity: Option<String>,
    pub hiring_page_url: Option<String>,
    pub founder_or_team_signal: Option<String>,
    pub last_checked_at: String,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobContactInput {
    pub name: String,
    pub company_id: Option<String>,
    pub role_title: Option<String>,
    pub public_profile_url: String,
    pub source_url: String,
    pub relationship_status: String,
    pub relevance: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobContact {
    pub id: String,
    pub name: String,
    pub company_id: Option<String>,
    pub role_title: Option<String>,
    pub public_profile_url: String,
    pub source_url: String,
    pub relationship_status: String,
    pub relevance: String,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobIntroPathInput {
    pub role_id: String,
    pub contact_id: String,
    pub path_type: String,
    pub confidence: String,
    pub next_action: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobIntroPath {
    pub id: String,
    pub role_id: String,
    pub contact_id: String,
    pub path_type: String,
    pub confidence: String,
    pub next_action: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSearchRunInput {
    pub profile_id: String,
    pub scope: String,
    pub proof_level: String,
    pub source_count: usize,
    pub role_count: usize,
    pub new_role_count: usize,
    pub stale_role_count: usize,
    pub error_count: usize,
    pub report_artifact_id: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSearchRun {
    pub id: String,
    pub profile_id: String,
    pub scope: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub proof_level: String,
    pub source_count: usize,
    pub role_count: usize,
    pub new_role_count: usize,
    pub stale_role_count: usize,
    pub error_count: usize,
    pub report_artifact_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRoleStatusEventInput {
    pub role_id: String,
    pub run_id: Option<String>,
    pub status: String,
    pub previous_tier: Option<String>,
    pub current_tier: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRoleStatusEvent {
    pub id: String,
    pub role_id: String,
    pub run_id: Option<String>,
    pub status: String,
    pub previous_tier: Option<String>,
    pub current_tier: Option<String>,
    pub note: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobApplicationInput {
    pub role_id: String,
    pub packet_id: Option<String>,
    pub status: String,
    pub applied_at: Option<String>,
    pub follow_up_at: Option<String>,
    pub outcome_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobApplication {
    pub id: String,
    pub role_id: String,
    pub packet_id: Option<String>,
    pub status: String,
    pub applied_at: Option<String>,
    pub follow_up_at: Option<String>,
    pub outcome_note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobWeeklyReport {
    pub id: String,
    pub profile_id: String,
    pub scope: String,
    pub generated_at: String,
    pub proof_level: String,
    pub body: String,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobWeeklyReportDeliveryInput {
    pub report_id: String,
    pub channel: String,
    pub subject: String,
    pub target: String,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobWeeklyReportDelivery {
    pub id: String,
    pub report_id: String,
    pub channel: String,
    pub subject: String,
    pub target: String,
    pub status: String,
    pub privacy_check_id: Option<String>,
    pub channel_message_id: Option<String>,
    pub idempotency_key: String,
    pub error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobWeeklyReportDeliveryReport {
    pub delivery: JobWeeklyReportDelivery,
    pub weekly_report: JobWeeklyReport,
    pub privacy_check: Option<JobPrivacyCheck>,
    pub channel_message: Option<ChannelMessage>,
    pub idempotent_replay: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobWeeklyReportDeliverySendInput {
    pub delivery_id: String,
    pub telegram_bot_token: Option<String>,
    pub email_account_id: Option<String>,
    pub email_api_token: Option<String>,
    pub email_from: Option<String>,
    pub api_base: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobWeeklyReportDeliverySendReport {
    pub delivery: JobWeeklyReportDelivery,
    pub weekly_report: JobWeeklyReport,
    pub privacy_check: Option<JobPrivacyCheck>,
    pub channel_message: Option<ChannelMessage>,
    pub channel_delivery_attempt: Option<ChannelDeliveryAttempt>,
    pub idempotent_replay: bool,
    pub proof_level: String,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobShortlistEntry {
    pub role: JobRoleCard,
    pub score: Option<JobFitScore>,
    pub outcome_warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobShortlist {
    pub profile_id: String,
    pub generated_at: String,
    pub entries: Vec<JobShortlistEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobOutreachReadinessEntry {
    pub role: JobRoleCard,
    pub score: Option<JobFitScore>,
    pub packet_id: Option<String>,
    pub packet_status: Option<String>,
    pub privacy_check_id: Option<String>,
    pub intro_path_ids: Vec<String>,
    pub contact_ids: Vec<String>,
    pub warm_intro_ready_count: usize,
    pub public_only_count: usize,
    pub decision: String,
    pub blockers: Vec<String>,
    pub next_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobOutreachReadinessReport {
    pub profile_id: String,
    pub generated_at: String,
    pub proof_level: String,
    pub ready_count: usize,
    pub blocked_count: usize,
    pub entries: Vec<JobOutreachReadinessEntry>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobOperationalAuditGate {
    pub name: String,
    pub decision: String,
    pub evidence: Value,
    pub missing_requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobOperationalAudit {
    pub profile_id: String,
    pub scope: String,
    pub generated_at: String,
    pub decision: String,
    pub proof_level: String,
    pub ops_summary: JobOpsSummary,
    pub refresh_audit: JobRefreshAudit,
    pub outreach_readiness: JobOutreachReadinessReport,
    pub source_family_counts: BTreeMap<String, usize>,
    pub evidence_visibility_counts: BTreeMap<String, usize>,
    pub packet_status_counts: BTreeMap<String, usize>,
    pub intro_status_counts: BTreeMap<String, usize>,
    pub weekly_report_count: usize,
    pub weekly_delivery_status_counts: BTreeMap<String, usize>,
    pub weekly_report_delivery_attempt_count: usize,
    pub job_radar_job_status_counts: BTreeMap<String, usize>,
    pub job_radar_completed_count: usize,
    pub job_radar_dead_lettered_count: usize,
    pub job_radar_attempt_count: i64,
    pub gates: Vec<JobOperationalAuditGate>,
    pub operational_blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobCompanyTargetEntry {
    pub company: JobCompanyCard,
    pub score: f64,
    pub tier: String,
    pub reasons: Vec<String>,
    pub warnings: Vec<String>,
    pub matched_evidence_tags: Vec<String>,
    pub next_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobCompanyTargetReport {
    pub profile_id: String,
    pub market: Option<String>,
    pub generated_at: String,
    pub proof_level: String,
    pub entries: Vec<JobCompanyTargetEntry>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JobImportBatchInput {
    pub profile: Option<JobCandidateProfileInput>,
    #[serde(default)]
    pub evidence_cards: Vec<JobEvidenceCardInput>,
    #[serde(default)]
    pub evidence_claims: Vec<JobEvidenceClaimInput>,
    #[serde(default)]
    pub privacy_rules: Vec<JobPrivacyRuleInput>,
    #[serde(default)]
    pub sources: Vec<JobSourceInput>,
    #[serde(default)]
    pub source_health: Vec<JobSourceHealthInput>,
    #[serde(default)]
    pub roles: Vec<JobRoleCardInput>,
    #[serde(default)]
    pub role_source_links: Vec<JobRoleSourceLinkInput>,
    #[serde(default)]
    pub fit_scores: Vec<JobFitScoreInput>,
    #[serde(default)]
    pub skeptic_findings: Vec<JobSkepticFindingInput>,
    #[serde(default)]
    pub packets: Vec<JobApplicationPacketInput>,
    #[serde(default)]
    pub companies: Vec<JobCompanyCardInput>,
    #[serde(default)]
    pub contacts: Vec<JobContactInput>,
    #[serde(default)]
    pub intro_paths: Vec<JobIntroPathInput>,
    #[serde(default)]
    pub search_runs: Vec<JobSearchRunInput>,
    #[serde(default)]
    pub role_status_events: Vec<JobRoleStatusEventInput>,
    #[serde(default)]
    pub applications: Vec<JobApplicationInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobImportBatchReport {
    pub imported_at: String,
    pub proof_level: String,
    pub profile_ids: Vec<String>,
    pub evidence_card_ids: Vec<String>,
    pub evidence_claim_ids: Vec<String>,
    pub privacy_rule_ids: Vec<String>,
    pub source_ids: Vec<String>,
    pub source_health_ids: Vec<String>,
    pub role_ids: Vec<String>,
    pub role_source_link_ids: Vec<String>,
    pub fit_score_ids: Vec<String>,
    pub skeptic_finding_ids: Vec<String>,
    pub packet_ids: Vec<String>,
    pub company_ids: Vec<String>,
    pub contact_ids: Vec<String>,
    pub intro_path_ids: Vec<String>,
    pub search_run_ids: Vec<String>,
    pub role_status_event_ids: Vec<String>,
    pub application_ids: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobManualRefreshInput {
    pub profile_id: String,
    pub scope: String,
    #[serde(default)]
    pub observed_role_ids: Vec<String>,
    #[serde(default)]
    pub stale_role_ids: Vec<String>,
    #[serde(default)]
    pub closed_role_ids: Vec<String>,
    #[serde(default)]
    pub source_health_ids: Vec<String>,
    pub proof_level: String,
    pub report_artifact_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobManualRefreshReport {
    pub run: JobSearchRun,
    pub events: Vec<JobRoleStatusEvent>,
    pub source_health: Vec<JobSourceHealth>,
    pub new_role_count: usize,
    pub unchanged_role_count: usize,
    pub promoted_role_count: usize,
    pub demoted_role_count: usize,
    pub stale_role_count: usize,
    pub closed_role_count: usize,
    pub error_count: usize,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRefreshAuditRunEvidence {
    pub run_id: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub proof_level: String,
    pub source_count: usize,
    pub role_count: usize,
    pub new_role_count: usize,
    pub stale_role_count: usize,
    pub error_count: usize,
    pub event_counts: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRefreshAudit {
    pub profile_id: String,
    pub scope: String,
    pub generated_at: String,
    pub decision: String,
    pub proof_level: String,
    pub minimum_elapsed_hours: i64,
    pub elapsed_hours: Option<f64>,
    pub completed_run_count: usize,
    pub first_run_id: Option<String>,
    pub latest_run_id: Option<String>,
    pub first_started_at: Option<String>,
    pub latest_started_at: Option<String>,
    pub total_source_count: usize,
    pub total_role_count: usize,
    pub total_error_count: usize,
    pub transition_counts: BTreeMap<String, usize>,
    pub run_evidence: Vec<JobRefreshAuditRunEvidence>,
    pub missing_requirements: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSourceRefreshInput {
    pub source_id: String,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub fetched_url: Option<String>,
    #[serde(default)]
    pub fetch_live: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSourceRefreshReport {
    pub source: JobSource,
    pub source_health: JobSourceHealth,
    pub roles: Vec<JobRoleCard>,
    pub companies: Vec<JobCompanyCard>,
    pub role_source_links: Vec<JobRoleSourceLink>,
    pub stale_role_events: Vec<JobRoleStatusEvent>,
    pub fetched_count: usize,
    pub accepted_count: usize,
    pub rejected_count: usize,
    pub warnings: Vec<String>,
}
