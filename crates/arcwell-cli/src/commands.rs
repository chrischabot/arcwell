use crate::*;

#[derive(Parser)]
#[command(name = "arcwell")]
#[command(about = "Local Arcwell CLI")]
pub(crate) struct Cli {
    #[arg(long, env = "ARCWELL_HOME")]
    pub(crate) home: Option<PathBuf>,

    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Subcommand)]
pub(crate) enum Command {
    Health,
    Ops,
    Provider(ProviderCommand),
    Doctor(DoctorArgs),
    Service(ServiceCommand),
    Serve(ServeArgs),
    Mcp,
    Worker(WorkerCommand),
    Profile(ProfileCommand),
    Memory(MemoryCommand),
    Wiki(WikiCommand),
    SourceCard(SourceCardCommand),
    Knowledge(KnowledgeCommand),
    Research(ResearchCommand),
    Commerce(CommerceCommand),
    Job(JobCommand),
    Radar(RadarCommand),
    X(XCommand),
    Telegram(TelegramCommand),
    Email(EmailCommand),
    Edge(EdgeCommand),
    Project(ProjectCommand),
    Controller(ControllerCommand),
    Work(WorkCommand),
    Procedure(ProcedureCommand),
    Import(ImportCommand),
    Candidate(CandidateCommand),
    Backup(BackupCommand),
    Cost(CostCommand),
    Policy(PolicyCommand),
    Secrets(SecretsCommand),
    Cursors(CursorCommand),
    Proof(ProofCommand),
}

#[derive(Args)]
pub(crate) struct DoctorArgs {
    #[arg(long)]
    pub(crate) strict: bool,
    #[arg(long, default_value_t = 300)]
    pub(crate) max_worker_heartbeat_age_seconds: i64,
    #[arg(long, default_value_t = 0)]
    pub(crate) max_dead_lettered_jobs: i64,
    #[arg(long, default_value_t = 7 * 24 * 60 * 60)]
    pub(crate) max_backup_age_seconds: i64,
}

#[derive(Args)]
pub(crate) struct ProviderCommand {
    #[command(subcommand)]
    pub(crate) command: ProviderSubcommand,
}

#[derive(Args)]
pub(crate) struct ProofCommand {
    #[command(subcommand)]
    pub(crate) command: ProofSubcommand,
}

#[derive(Subcommand)]
pub(crate) enum ProofSubcommand {
    Record {
        #[arg(long)]
        scope: String,
        #[arg(long)]
        title: String,
        #[arg(long, default_value = "local_proof")]
        proof_level: String,
        #[arg(long, default_value = "partial")]
        status: String,
        #[arg(long)]
        summary: String,
        #[arg(long)]
        artifact_root: Option<String>,
        #[arg(long)]
        reviewer: Option<String>,
        #[arg(long, default_value = "[]")]
        claims_json: String,
        #[arg(long, default_value = "[]")]
        artifacts_json: String,
        #[arg(long, default_value = "[]")]
        checks_json: String,
        #[arg(long, default_value = "{}")]
        metadata_json: String,
    },
    Read {
        packet_id: String,
    },
    List {
        #[arg(long)]
        scope: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    Latest {
        #[arg(long, alias = "scope")]
        capability: String,
    },
    VerifyPacket {
        path: PathBuf,
    },
    Promote {
        packet_id: String,
        #[arg(long)]
        reviewer: String,
    },
}

#[derive(Subcommand)]
pub(crate) enum ProviderSubcommand {
    Probe {
        #[arg(long, value_delimiter = ',')]
        providers: Vec<String>,
    },
}

#[derive(Args)]
pub(crate) struct ServiceCommand {
    #[command(subcommand)]
    pub(crate) command: ServiceSubcommand,
}

#[derive(Subcommand)]
pub(crate) enum ServiceSubcommand {
    Install {
        #[arg(long, default_value_t = 10)]
        max_jobs_per_tick: usize,
        #[arg(long, default_value_t = 5000)]
        idle_sleep_ms: u64,
        #[arg(long)]
        no_load: bool,
    },
    Status,
    RecurrenceAudit {
        #[arg(long, default_value_t = 48)]
        min_span_hours: i64,
        #[arg(long, default_value_t = 15 * 60)]
        max_gap_seconds: i64,
    },
    Restart,
    Logs,
    Uninstall {
        #[arg(long)]
        no_unload: bool,
    },
}

#[derive(Args)]
pub(crate) struct ServeArgs {
    #[arg(long, default_value = "127.0.0.1:8787")]
    pub(crate) addr: SocketAddr,
    #[arg(long, env = "ARCWELL_HTTP_AUTH_TOKEN")]
    pub(crate) auth_token: Option<String>,
    #[arg(long, default_value_t = 8192)]
    pub(crate) max_uri_bytes: usize,
    #[arg(long, default_value_t = 65536)]
    pub(crate) max_body_bytes: u64,
}

#[derive(Args)]
pub(crate) struct WorkerCommand {
    #[command(subcommand)]
    pub(crate) command: WorkerSubcommand,
}

#[derive(Subcommand)]
pub(crate) enum WorkerSubcommand {
    RunOnce {
        #[arg(long, default_value_t = 10)]
        max_jobs: usize,
    },
    Run {
        #[arg(long, default_value_t = 10)]
        max_jobs_per_tick: usize,
        #[arg(long, default_value_t = 5000)]
        idle_sleep_ms: u64,
        #[arg(long)]
        max_ticks: Option<usize>,
    },
}

#[derive(Args)]
pub(crate) struct ProfileCommand {
    #[command(subcommand)]
    pub(crate) command: ProfileSubcommand,
}

#[derive(Subcommand)]
pub(crate) enum ProfileSubcommand {
    Set {
        key: String,
        value: String,
        #[arg(long, default_value = "normal")]
        sensitivity: String,
        #[arg(long, default_value = "manual")]
        source: String,
    },
    Get {
        key: String,
    },
    Search {
        query: String,
    },
    List,
    Delete {
        key: String,
    },
}

#[derive(Args)]
pub(crate) struct MemoryCommand {
    #[command(subcommand)]
    pub(crate) command: MemorySubcommand,
}

#[derive(Subcommand)]
pub(crate) enum MemorySubcommand {
    Add {
        text: String,
        #[arg(long, default_value = "fact")]
        kind: String,
        #[arg(long, default_value = "normal")]
        sensitivity: String,
        #[arg(long, default_value = "manual")]
        source: String,
        #[arg(long, default_value_t = 0.8)]
        confidence: f64,
    },
    Search {
        query: String,
    },
    List {
        #[arg(long, default_value_t = 50)]
        limit: u32,
    },
    Delete {
        id: String,
    },
    Mem0Add {
        text: String,
        #[arg(long)]
        user_id: Option<String>,
        #[arg(long, default_value = "manual")]
        source: String,
        #[arg(long, default_value = "normal")]
        sensitivity: String,
        #[arg(long, default_value_t = false)]
        infer: bool,
    },
    Mem0Search {
        query: String,
        #[arg(long)]
        user_id: Option<String>,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    Mem0Update {
        id: String,
        text: String,
        #[arg(long)]
        user_id: Option<String>,
    },
    Mem0Delete {
        id: String,
        #[arg(long)]
        user_id: Option<String>,
    },
    Mem0History {
        id: String,
    },
    Mem0ForgetUser {
        #[arg(long)]
        user_id: Option<String>,
    },
    Recall {
        query: String,
        #[arg(long)]
        user_id: Option<String>,
        #[arg(long, default_value_t = 8)]
        limit: usize,
    },
    Capture {
        text: String,
        #[arg(long, default_value = "manual")]
        source: String,
        #[arg(long)]
        user_id: Option<String>,
        #[arg(long)]
        auto_apply: bool,
        #[arg(long)]
        infer: bool,
    },
    Events {
        #[arg(long, default_value_t = 50)]
        limit: u32,
    },
    Decisions {
        #[arg(long, default_value_t = 50)]
        limit: u32,
    },
    Tombstones {
        #[arg(long, default_value_t = 50)]
        limit: u32,
    },
    EvalCorpus,
    Dream,
    HookRecall {
        #[arg(long, default_value = "user-prompt-submit")]
        event: String,
        #[arg(long)]
        query: Option<String>,
        #[arg(long)]
        user_id: Option<String>,
        #[arg(long, default_value_t = 8)]
        limit: usize,
    },
    HookCapture {
        #[arg(long, default_value = "stop")]
        event: String,
        #[arg(long)]
        text: Option<String>,
        #[arg(long)]
        user_id: Option<String>,
        #[arg(long)]
        auto_apply: bool,
        #[arg(long)]
        infer: bool,
    },
}

#[derive(Args)]
pub(crate) struct ImportCommand {
    #[command(subcommand)]
    pub(crate) command: ImportSubcommand,
}

#[derive(Subcommand)]
pub(crate) enum ImportSubcommand {
    Claude {
        path: PathBuf,
        #[arg(long)]
        dry_run: bool,
        #[arg(long, default_value_t = 25)]
        limit: usize,
        #[arg(long)]
        user_id: Option<String>,
        #[arg(long)]
        write_candidates: bool,
    },
    Runs {
        #[arg(long, default_value_t = 25)]
        limit: usize,
    },
}

#[derive(Args)]
pub(crate) struct CandidateCommand {
    #[command(subcommand)]
    pub(crate) command: CandidateSubcommand,
}

#[derive(Subcommand)]
pub(crate) enum CandidateSubcommand {
    List {
        #[arg(long, default_value = "pending")]
        status: String,
    },
    Apply {
        id: String,
    },
    Reject {
        id: String,
    },
}

#[derive(Args)]
pub(crate) struct ProcedureCommand {
    #[command(subcommand)]
    pub(crate) command: ProcedureSubcommand,
}

#[derive(Subcommand)]
pub(crate) enum ProcedureSubcommand {
    Propose {
        run_id: String,
        #[arg(long)]
        auto_approve: bool,
    },
    Candidate {
        operation: String,
        title: String,
        method: String,
        #[arg(long)]
        procedure_id: Option<String>,
        #[arg(long)]
        base_version: Option<i64>,
        #[arg(long, default_value = "manual")]
        source_run_id: Vec<String>,
        #[arg(long, default_value = "normal")]
        sensitivity: String,
        #[arg(long, default_value = "manual procedure candidate")]
        reason: String,
        #[arg(long, default_value = "")]
        trigger_context: String,
        #[arg(long, default_value = "")]
        problem: String,
        #[arg(long)]
        precondition: Vec<String>,
        #[arg(long)]
        tool: Vec<String>,
        #[arg(long)]
        validation_command: Vec<String>,
        #[arg(long)]
        known_risk: Vec<String>,
    },
    Candidates {
        #[arg(long, default_value = "pending")]
        status: String,
    },
    Apply {
        id: String,
    },
    Reject {
        id: String,
        #[arg(long)]
        reason: Option<String>,
    },
    Search {
        #[arg(long)]
        query: Option<String>,
        #[arg(long, default_value = "active")]
        status: String,
        #[arg(long, default_value_t = 25)]
        limit: usize,
    },
    Read {
        id: String,
    },
    RetrievalContext {
        query: String,
        #[arg(long, default_value_t = 5)]
        limit: usize,
    },
    ExportSkill {
        id: String,
        skill_name: String,
    },
    Curate,
}

#[derive(Args)]
pub(crate) struct BackupCommand {
    #[command(subcommand)]
    pub(crate) command: BackupSubcommand,
}

#[derive(Subcommand)]
pub(crate) enum BackupSubcommand {
    Create,
    Status,
    Verify,
    Restore {
        #[arg(long)]
        from: PathBuf,
        #[arg(long)]
        target_home: Option<PathBuf>,
        #[arg(long)]
        replace: bool,
    },
}

#[derive(Args)]
pub(crate) struct CostCommand {
    #[command(subcommand)]
    pub(crate) command: CostSubcommand,
}

#[derive(Subcommand)]
pub(crate) enum CostSubcommand {
    Add {
        package: String,
        job_id: String,
        provider: String,
        model: String,
        #[arg(long, default_value_t = 0.0)]
        estimated_usd: f64,
        #[arg(long, default_value_t = 0.0)]
        actual_usd: f64,
    },
    SetPolicy {
        scope: String,
        key: String,
        #[arg(long)]
        limit_usd: Option<f64>,
        #[arg(long)]
        kill_switch: bool,
        #[arg(long)]
        override_until: Option<String>,
    },
    Policies,
    Check {
        package: String,
        provider: String,
        #[arg(long)]
        source: Option<String>,
        #[arg(long, default_value_t = 0.0)]
        projected_usd: f64,
    },
    Summary,
}

#[derive(Args)]
pub(crate) struct PolicyCommand {
    #[command(subcommand)]
    pub(crate) command: PolicySubcommand,
}

#[derive(Subcommand)]
pub(crate) enum PolicySubcommand {
    Check(PolicyRequestArgs),
    Explain(PolicyRequestArgs),
    List {
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    Rules,
    Override {
        #[command(flatten)]
        request: PolicyRequestArgs,
        #[arg(long)]
        reason: String,
        #[arg(long)]
        expires_at: String,
    },
    Approvals {
        #[arg(long)]
        status: Option<String>,
    },
    Approve {
        id: String,
        #[arg(long)]
        reason: Option<String>,
    },
    Reject {
        id: String,
        #[arg(long)]
        reason: Option<String>,
    },
}

#[derive(Args)]
pub(crate) struct PolicyRequestArgs {
    #[arg(long)]
    pub(crate) action: String,
    #[arg(long)]
    pub(crate) package: Option<String>,
    #[arg(long)]
    pub(crate) provider: Option<String>,
    #[arg(long)]
    pub(crate) source: Option<String>,
    #[arg(long)]
    pub(crate) channel: Option<String>,
    #[arg(long)]
    pub(crate) subject: Option<String>,
    #[arg(long)]
    pub(crate) target: Option<String>,
    #[arg(long)]
    pub(crate) projected_usd: Option<f64>,
    #[arg(long)]
    pub(crate) metadata_json: Option<String>,
    #[arg(long)]
    pub(crate) untrusted_excerpt: Option<String>,
}

#[derive(Args)]
pub(crate) struct SecretsCommand {
    #[command(subcommand)]
    pub(crate) command: SecretsSubcommand,
}

#[derive(Subcommand)]
pub(crate) enum SecretsSubcommand {
    SetRef {
        name: String,
        location: String,
        scope: String,
        #[arg(long)]
        expires_at: Option<String>,
    },
    List,
    SetValue {
        name: String,
        value: String,
        #[arg(long, default_value = "local")]
        scope: String,
        #[arg(long)]
        provider: Option<String>,
        #[arg(long)]
        expires_at: Option<String>,
    },
    GetValue {
        name: String,
    },
    ListValues,
    Health,
    DeleteValue {
        name: String,
    },
}

#[derive(Args)]
pub(crate) struct CursorCommand {
    #[command(subcommand)]
    pub(crate) command: CursorSubcommand,
}

#[derive(Subcommand)]
pub(crate) enum CursorSubcommand {
    List,
    Get { key: String },
}

#[derive(Args)]
pub(crate) struct WikiCommand {
    #[command(subcommand)]
    pub(crate) command: WikiSubcommand,
}

#[derive(Args)]
pub(crate) struct SourceCardCommand {
    #[command(subcommand)]
    pub(crate) command: SourceCardSubcommand,
}

#[derive(Args)]
pub(crate) struct KnowledgeCommand {
    #[command(subcommand)]
    pub(crate) command: KnowledgeSubcommand,
}

#[derive(Subcommand)]
pub(crate) enum KnowledgeSubcommand {
    ProjectRadarRun {
        run_id: String,
        #[arg(long)]
        topic: Option<String>,
        #[arg(long, default_value_t = 12)]
        max_source_cards: usize,
    },
    ProjectSourceCardQuery {
        query: String,
        #[arg(long)]
        topic: Option<String>,
        #[arg(long, default_value_t = 12)]
        max_source_cards: usize,
    },
    ClusterBacklog {
        #[arg(long, default_value_t = 100)]
        max_source_cards: usize,
        #[arg(long, default_value_t = 2)]
        min_group_size: usize,
        #[arg(long, default_value_t = 12)]
        max_clusters: usize,
    },
    Events {
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    Clusters {
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    ExpandCluster {
        cluster_id: String,
        #[arg(long)]
        skip_digest: bool,
    },
    WriteClusterModel {
        cluster_id: String,
        #[arg(long, default_value = "mock")]
        provider: String,
        #[arg(long)]
        model_name: Option<String>,
        #[arg(long)]
        endpoint: Option<String>,
        #[arg(long)]
        timeout_seconds: Option<u64>,
        #[arg(long)]
        skip_digest: bool,
    },
    PromoteCluster {
        cluster_id: String,
        #[arg(long)]
        reviewer: Option<String>,
        #[arg(long)]
        reason: Option<String>,
    },
    DecideClusterEditorial {
        cluster_id: String,
        #[arg(long)]
        no_enqueue: bool,
    },
    InvestigateCluster {
        cluster_id: String,
    },
    ExecuteClusterInvestigation {
        cluster_id: String,
    },
    EnqueueClusterExpansion {
        cluster_id: String,
        #[arg(long)]
        skip_digest: bool,
    },
    EnqueueClusterEditorialDecision {
        cluster_id: String,
        #[arg(long)]
        no_enqueue: bool,
    },
    EnqueueClusterModelWrite {
        cluster_id: String,
        #[arg(long, default_value = "mock")]
        provider: String,
        #[arg(long)]
        model_name: Option<String>,
        #[arg(long)]
        endpoint: Option<String>,
        #[arg(long)]
        timeout_seconds: Option<u64>,
        #[arg(long)]
        skip_digest: bool,
    },
    ScheduleClusterModelWrite {
        cluster_id: String,
        #[arg(long, default_value = "mock")]
        provider: String,
        #[arg(long)]
        model_name: Option<String>,
        #[arg(long)]
        endpoint: Option<String>,
        #[arg(long)]
        timeout_seconds: Option<u64>,
        #[arg(long)]
        skip_digest: bool,
        #[arg(long, default_value = "warm")]
        cadence: String,
        #[arg(long, default_value = "active")]
        status: String,
    },
    EnqueueDueModelWrites {
        #[arg(long, default_value_t = 25)]
        max_clusters: usize,
        #[arg(long, default_value = "mock")]
        provider: String,
        #[arg(long)]
        model_name: Option<String>,
        #[arg(long)]
        endpoint: Option<String>,
        #[arg(long)]
        timeout_seconds: Option<u64>,
        #[arg(long)]
        skip_digest: bool,
    },
    EnqueueClusterInvestigation {
        cluster_id: String,
    },
    EnqueueClusterInvestigationExecution {
        cluster_id: String,
    },
    EnqueueBacklogClustering {
        #[arg(long, default_value_t = 100)]
        max_source_cards: usize,
        #[arg(long, default_value_t = 2)]
        min_group_size: usize,
        #[arg(long, default_value_t = 12)]
        max_clusters: usize,
    },
    ScheduleBacklogClustering {
        #[arg(long, default_value_t = 100)]
        max_source_cards: usize,
        #[arg(long, default_value_t = 2)]
        min_group_size: usize,
        #[arg(long, default_value_t = 12)]
        max_clusters: usize,
        #[arg(long, default_value = "warm")]
        cadence: String,
        #[arg(long, default_value = "active")]
        status: String,
    },
    EnqueueModelClusters {
        query: String,
        #[arg(long, default_value = "mock")]
        provider: String,
        #[arg(long)]
        model_name: Option<String>,
        #[arg(long)]
        endpoint: Option<String>,
        #[arg(long)]
        timeout_seconds: Option<u64>,
        #[arg(long, default_value_t = 24)]
        max_source_cards: usize,
        #[arg(long, default_value_t = 6)]
        max_clusters: usize,
    },
    RunModelClusters {
        query: String,
        #[arg(long, default_value = "mock")]
        provider: String,
        #[arg(long)]
        model_name: Option<String>,
        #[arg(long)]
        endpoint: Option<String>,
        #[arg(long)]
        timeout_seconds: Option<u64>,
        #[arg(long, default_value_t = 24)]
        max_source_cards: usize,
        #[arg(long, default_value_t = 6)]
        max_clusters: usize,
    },
    ScheduleModelClusters {
        query: String,
        #[arg(long, default_value = "mock")]
        provider: String,
        #[arg(long)]
        model_name: Option<String>,
        #[arg(long)]
        endpoint: Option<String>,
        #[arg(long)]
        timeout_seconds: Option<u64>,
        #[arg(long, default_value_t = 24)]
        max_source_cards: usize,
        #[arg(long, default_value_t = 6)]
        max_clusters: usize,
        #[arg(long, default_value = "warm")]
        cadence: String,
        #[arg(long, default_value = "active")]
        status: String,
    },
    ProposeClusters {
        query: String,
        #[arg(long, default_value = "mock")]
        provider: String,
        #[arg(long)]
        model_name: Option<String>,
        #[arg(long)]
        endpoint: Option<String>,
        #[arg(long)]
        timeout_seconds: Option<u64>,
        #[arg(long, default_value_t = 24)]
        max_source_cards: usize,
        #[arg(long, default_value_t = 6)]
        max_clusters: usize,
    },
    Reports {
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    Entities {
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    ResolveEntities {
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    UpsertEntity {
        #[arg(long)]
        entity_type: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        canonical_key: String,
        #[arg(long, default_value = "[]")]
        aliases_json: String,
        #[arg(long)]
        homepage_url: Option<String>,
        #[arg(long, default_value = "[]")]
        source_card_ids_json: String,
        #[arg(long)]
        wiki_page_id: Option<String>,
        #[arg(long, default_value_t = 0.8)]
        confidence: f64,
        #[arg(long, default_value = "{}")]
        metadata_json: String,
    },
    ResolveEntityModel {
        left_entity_id: String,
        right_entity_id: String,
        #[arg(long, default_value = "mock")]
        provider: String,
        #[arg(long)]
        model_name: Option<String>,
        #[arg(long)]
        endpoint: Option<String>,
        #[arg(long)]
        timeout_seconds: Option<u64>,
    },
    EnqueueEntityResolutionModel {
        left_entity_id: String,
        right_entity_id: String,
        #[arg(long, default_value = "mock")]
        provider: String,
        #[arg(long)]
        model_name: Option<String>,
        #[arg(long)]
        endpoint: Option<String>,
        #[arg(long)]
        timeout_seconds: Option<u64>,
    },
    EnqueueDueEntityResolution {
        #[arg(long, default_value_t = 25)]
        max_pairs: usize,
        #[arg(long, default_value = "mock")]
        provider: String,
        #[arg(long)]
        model_name: Option<String>,
        #[arg(long)]
        endpoint: Option<String>,
        #[arg(long)]
        timeout_seconds: Option<u64>,
    },
    ScheduleEntityResolution {
        #[arg(long, default_value_t = 25)]
        max_pairs: usize,
        #[arg(long, default_value = "mock")]
        provider: String,
        #[arg(long)]
        model_name: Option<String>,
        #[arg(long)]
        endpoint: Option<String>,
        #[arg(long)]
        timeout_seconds: Option<u64>,
        #[arg(long, default_value = "warm")]
        cadence: String,
        #[arg(long, default_value = "active")]
        status: String,
    },
    ScheduleDailyBriefing {
        #[arg(long, default_value = "Arcwell AI daily briefing")]
        name: String,
        #[arg(long, default_value = "email")]
        channel: String,
        #[arg(long)]
        recipient_ref: String,
        #[arg(long, default_value = "local")]
        time_zone: String,
        #[arg(long, default_value_t = 7)]
        hour: i64,
        #[arg(long, default_value_t = 0)]
        minute: i64,
        #[arg(long, default_value_t = 72)]
        catch_up_hours: i64,
        #[arg(long, default_value_t = 12)]
        max_reports: usize,
        #[arg(long, default_value_t = 80)]
        max_source_cards: usize,
        #[arg(long, default_value = "active")]
        status: String,
    },
    IssueSchedules,
    IssueScheduleTicks {
        #[arg(long)]
        schedule_id: Option<String>,
    },
    EntityResolutions {
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    Relations {
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    AdapterRuns {
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
}

#[derive(Args)]
pub(crate) struct ResearchCommand {
    #[command(subcommand)]
    pub(crate) command: ResearchSubcommand,
}

#[derive(Subcommand)]
pub(crate) enum ResearchSubcommand {
    Capabilities,
    Run {
        query: String,
    },
    Status {
        run_id: String,
    },
    Read {
        run_id: String,
    },
    AuditRun {
        run_id: String,
    },
    Stop {
        run_id: String,
    },
    Sources {
        run_id: String,
    },
    AddSource {
        run_id: String,
        #[arg(long)]
        title: String,
        #[arg(long)]
        url: Option<String>,
        #[arg(long)]
        local_ref: Option<String>,
        #[arg(long, default_value = "uncategorized")]
        source_family: String,
        #[arg(long, default_value = "web")]
        source_type: String,
        #[arg(long, default_value = "manual")]
        provider: String,
        #[arg(long)]
        reason: Option<String>,
        #[arg(long, default_value_t = 50)]
        priority: i64,
        #[arg(long, default_value = "candidate")]
        fetch_status: String,
        #[arg(long, default_value = "snippet-only")]
        read_depth: String,
        #[arg(long, default_value = "candidate")]
        triage_status: String,
        #[arg(long)]
        canonical_key: Option<String>,
        #[arg(long)]
        notes: Option<String>,
    },
    LinkSourceCard {
        run_id: String,
        source_card_id: String,
        #[arg(long, default_value = "uncategorized")]
        source_family: String,
        #[arg(long, default_value = "full-text")]
        read_depth: String,
        #[arg(long, default_value = "must-read-primary")]
        triage_status: String,
        #[arg(long)]
        notes: Option<String>,
    },
    ExtractionPrompt {
        run_id: String,
        source_card_id: String,
    },
    IngestClaims {
        run_id: String,
        source_card_id: String,
        #[arg(long, default_value = "manual")]
        provider: String,
        #[arg(long, default_value = "manual")]
        model: String,
        #[arg(long)]
        output_json: String,
    },
    Claims {
        run_id: String,
    },
    Clusters {
        run_id: String,
    },
    Skeptic {
        run_id: String,
    },
    Report {
        run_id: String,
        saturation_reason: String,
        #[arg(long)]
        no_write: bool,
    },
    Converge(ResearchConvergenceArgs),
    ConvergeStep(ResearchConvergenceArgs),
    ConvergeEnqueue(ResearchConvergenceArgs),
    ConvergenceStatus {
        run_id: String,
    },
    Iterations {
        run_id: String,
    },
    IterationRead {
        id: String,
    },
    Statements {
        run_id: String,
    },
    Challenges {
        run_id: String,
    },
    ConvergenceHostSearchTasks {
        run_id: String,
    },
    ConvergenceProviderSearch {
        run_id: String,
        #[arg(long, default_value = "brave")]
        provider: String,
        #[arg(long)]
        max_tasks: Option<usize>,
        #[arg(long)]
        max_results: Option<usize>,
        #[arg(long)]
        max_provider_calls: Option<usize>,
        #[arg(long)]
        enqueue_selected_url_ingest: bool,
        #[arg(long)]
        max_ingest_jobs: Option<usize>,
        #[arg(long)]
        cost_cap_usd: Option<f64>,
        #[arg(long)]
        endpoint: Option<String>,
        #[arg(long)]
        api_key: Option<String>,
        #[arg(long)]
        model: Option<String>,
        #[arg(long)]
        timeout_seconds: Option<u64>,
    },
    Disproofs {
        run_id: String,
    },
    Revisions {
        run_id: String,
    },
    FactChecks {
        run_id: String,
    },
    ActiveFactCheck {
        run_id: String,
        #[arg(long)]
        artifact_id: Option<String>,
        #[arg(long)]
        max_sentences: Option<usize>,
        #[arg(long)]
        no_challenges: bool,
    },
    ConvergenceCloseLoop(ResearchConvergenceCloseLoopArgs),
    ConvergenceSnapshots {
        run_id: String,
    },
    ConvergenceReport {
        run_id: String,
    },
    ReportJudgments {
        run_id: String,
    },
    Plan {
        query: String,
        #[arg(long, default_value_t = 5)]
        max_sources: usize,
    },
    Search {
        query: String,
        #[arg(long, default_value = "host")]
        provider: String,
        #[arg(long, default_value_t = 5)]
        max_results: usize,
        #[arg(long)]
        endpoint: Option<String>,
        #[arg(long)]
        api_key: Option<String>,
        #[arg(long)]
        model: Option<String>,
        #[arg(long, default_value_t = 15)]
        timeout_seconds: u64,
        #[arg(long)]
        write_wiki: bool,
    },
    Workflow {
        query: String,
    },
    Tasks {
        run_id: String,
    },
    RoleStart {
        run_id: String,
        role: String,
        #[arg(long, default_value = "codex")]
        host: String,
        #[arg(long, default_value = "host_sequential")]
        execution_mode: String,
        #[arg(long)]
        host_thread_id: Option<String>,
        #[arg(long)]
        host_subagent_id: Option<String>,
        #[arg(long)]
        tool_surface: Option<String>,
        #[arg(long, default_value = "v1")]
        prompt_version: String,
        #[arg(long)]
        prompt_hash: Option<String>,
        #[arg(long = "input-artifact-id")]
        input_artifact_ids: Vec<String>,
    },
    RoleFinish {
        role_run_id: String,
        status: String,
        #[arg(long)]
        output_artifact_id: Option<String>,
        #[arg(long)]
        error_kind: Option<String>,
        #[arg(long)]
        error_message: Option<String>,
    },
    RoleRuns {
        run_id: String,
    },
    ArtifactAdd {
        run_id: String,
        artifact_type: String,
        title: String,
        #[arg(long)]
        body: String,
        #[arg(long)]
        role_run_id: Option<String>,
        #[arg(long, default_value = "{}")]
        metadata_json: String,
    },
    Artifacts {
        run_id: String,
    },
    ArtifactRead {
        id: String,
    },
    HostSearchRecord {
        run_id: String,
        query: String,
        #[arg(long, default_value = "codex")]
        host: String,
        #[arg(long, default_value = "host-native")]
        tool_surface: String,
        #[arg(long)]
        role_run_id: Option<String>,
        #[arg(long)]
        query_intent: Option<String>,
        #[arg(long)]
        requested_recency: Option<i64>,
        #[arg(long = "requested-domain")]
        requested_domains: Vec<String>,
        #[arg(long)]
        cost_decision_id: Option<String>,
        #[arg(long)]
        results_json: String,
    },
    HostSearches {
        run_id: String,
    },
    HostSearchRead {
        id: String,
    },
    DocumentExtract {
        run_id: String,
        path: PathBuf,
        #[arg(long)]
        media_type: Option<String>,
        #[arg(long)]
        research_source_id: Option<String>,
        #[arg(long)]
        source_card_id: Option<String>,
    },
    Documents {
        run_id: String,
    },
    DocumentRead {
        id: String,
    },
    EvidencePack {
        run_id: String,
    },
    EditorialInvoke {
        run_id: String,
        stage: String,
        #[arg(long, default_value = "openai")]
        model_provider: String,
        #[arg(long)]
        model_name: Option<String>,
        #[arg(long, default_value = "v1")]
        prompt_version: String,
        #[arg(long)]
        input_artifact_id: Option<String>,
        #[arg(long)]
        endpoint: Option<String>,
        #[arg(long)]
        api_key: Option<String>,
        #[arg(long)]
        timeout_seconds: Option<u64>,
    },
    EditorialRecord {
        run_id: String,
        stage: String,
        #[arg(long, default_value = "openai")]
        model_provider: String,
        #[arg(long)]
        model_name: String,
        #[arg(long, default_value = "v1")]
        prompt_version: String,
        #[arg(long)]
        input_artifact_id: Option<String>,
        #[arg(long)]
        output_artifact_id: Option<String>,
        #[arg(long)]
        cost_decision_id: Option<String>,
        #[arg(long, default_value = "completed")]
        status: String,
        #[arg(long, default_value = "{}")]
        score_json: String,
        #[arg(long)]
        error_message: Option<String>,
    },
    EditorialRuns {
        run_id: String,
    },
    EditorialRead {
        id: String,
    },
    CompleteTask {
        task_id: String,
        notes: String,
    },
    Brief {
        query: String,
        #[arg(long)]
        no_write: bool,
    },
    Audit {
        query: String,
    },
    Runs,
}

#[derive(Args)]
pub(crate) struct CommerceCommand {
    #[command(subcommand)]
    pub(crate) command: CommerceSubcommand,
}

#[derive(Subcommand)]
pub(crate) enum CommerceSubcommand {
    Capabilities,
    ConfigSet {
        run_id: String,
        #[arg(long)]
        domain_profile: String,
        #[arg(long, default_value_t = 20)]
        target_qualified_count: usize,
        #[arg(long)]
        geography: Option<String>,
        #[arg(long, default_value = "24h")]
        freshness_window: String,
        #[arg(long = "private-source")]
        allowed_private_context_sources: Vec<String>,
        #[arg(long = "public-source-family")]
        allowed_public_source_families: Vec<String>,
        #[arg(long)]
        allow_marketplaces: bool,
        #[arg(long)]
        allow_chrome_profile: bool,
        #[arg(long)]
        max_provider_calls: Option<usize>,
        #[arg(long)]
        max_browser_pages: Option<usize>,
        #[arg(long)]
        max_cost_usd: Option<f64>,
        #[arg(long, default_value = "{}")]
        stop_rules_json: String,
    },
    Config {
        run_id: String,
    },
    CandidateAdd {
        run_id: String,
        #[arg(long)]
        domain: String,
        #[arg(long)]
        source_url: String,
        #[arg(long)]
        retailer_or_provider: String,
        #[arg(long)]
        title: String,
        #[arg(long)]
        normalized_item_key: String,
        #[arg(long)]
        variant_key: String,
        #[arg(long)]
        price: Option<String>,
        #[arg(long)]
        currency: Option<String>,
        #[arg(long)]
        geography: Option<String>,
        #[arg(long, default_value = "maybe")]
        candidate_status: String,
        #[arg(long)]
        score: Option<f64>,
        #[arg(long, default_value = "{}")]
        score_reasons_json: String,
        #[arg(long, default_value = "[]")]
        disqualification_reasons_json: String,
        #[arg(long, default_value = "{}")]
        metadata_json: String,
    },
    Candidates {
        run_id: String,
    },
    AvailabilityProofAdd {
        run_id: String,
        #[arg(long)]
        candidate_id: String,
        #[arg(long)]
        proof_method: String,
        #[arg(long)]
        variant_key: String,
        #[arg(long)]
        variant_label: String,
        #[arg(long)]
        availability_state: String,
        #[arg(long)]
        visible_evidence: Option<String>,
        #[arg(long)]
        selector_or_dom_hint: Option<String>,
        #[arg(long)]
        screenshot_artifact_id: Option<String>,
        #[arg(long)]
        page_snapshot_artifact_id: Option<String>,
        #[arg(long, default_value_t = 0.7)]
        confidence: f64,
        #[arg(long, default_value = "[]")]
        caveats_json: String,
        #[arg(long)]
        checked_at: Option<String>,
    },
    AvailabilityProofs {
        run_id: String,
    },
    RenderedPageCheck {
        run_id: String,
        #[arg(long)]
        candidate_id: String,
        #[arg(long)]
        variant_key: String,
        #[arg(long)]
        variant_label: String,
        #[arg(long)]
        requested_url: String,
        #[arg(long)]
        final_url: Option<String>,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        rendered_html: Option<String>,
        #[arg(long)]
        rendered_html_file: Option<PathBuf>,
        #[arg(long)]
        rendered_text: Option<String>,
        #[arg(long)]
        rendered_text_file: Option<PathBuf>,
        #[arg(long)]
        captured_at: Option<String>,
        #[arg(long)]
        browser: Option<String>,
        #[arg(long)]
        screenshot_path: Option<String>,
        #[arg(long)]
        selector_or_dom_hint: Option<String>,
        #[arg(long)]
        chrome_profile_required: bool,
    },
    ContextFactAdd {
        run_id: String,
        #[arg(long)]
        fact_key: String,
        #[arg(long)]
        fact_kind: String,
        #[arg(long)]
        redacted_value: String,
        #[arg(long)]
        source_family: String,
        #[arg(long)]
        source_ref: Option<String>,
        #[arg(long, default_value_t = 0.7)]
        confidence: f64,
        #[arg(long)]
        user_confirmed: bool,
        #[arg(long)]
        may_persist_to_memory: bool,
        #[arg(long, default_value = "{}")]
        metadata_json: String,
    },
    ContextFacts {
        run_id: String,
    },
    ContextPacket {
        run_id: String,
    },
    VerificationAttemptAdd {
        run_id: String,
        #[arg(long)]
        candidate_id: String,
        #[arg(long)]
        method: String,
        #[arg(long)]
        result: String,
        #[arg(long)]
        error_kind: Option<String>,
        #[arg(long)]
        final_url: Option<String>,
        #[arg(long)]
        http_status: Option<i64>,
        #[arg(long)]
        browser_required: bool,
        #[arg(long)]
        chrome_profile_required: bool,
        #[arg(long = "artifact-id")]
        artifact_ids: Vec<String>,
        #[arg(long)]
        next_action: Option<String>,
        #[arg(long)]
        attempted_at: Option<String>,
    },
    VerificationAttempts {
        run_id: String,
    },
    ReportJudgmentAdd {
        run_id: String,
        #[arg(long)]
        decision: String,
        #[arg(long, default_value = "[]")]
        blocking_findings_json: String,
        #[arg(long, default_value = "[]")]
        non_blocking_findings_json: String,
        #[arg(long, default_value = "[]")]
        claims_checked_json: String,
        #[arg(long, default_value = "[]")]
        availability_proofs_checked_json: String,
        #[arg(long, default_value = "{}")]
        privacy_review_json: String,
        #[arg(long, default_value = "[]")]
        remaining_risks_json: String,
    },
    ReportJudgments {
        run_id: String,
    },
    Report {
        run_id: String,
    },
}

#[derive(Args)]
pub(crate) struct JobCommand {
    #[command(subcommand)]
    pub(crate) command: JobSubcommand,
}

#[derive(Subcommand)]
pub(crate) enum JobSubcommand {
    ProfileAdd {
        #[arg(long)]
        label: String,
        #[arg(long)]
        current_resume_source: Option<String>,
        #[arg(long)]
        linkedin_source: Option<String>,
        #[arg(long)]
        github_profile: Option<String>,
        #[arg(long)]
        blog_url: Option<String>,
        #[arg(long, default_value = "{}")]
        metadata_json: String,
    },
    Profiles,
    Profile {
        profile_id: String,
    },
    Import {
        #[arg(long)]
        path: PathBuf,
    },
    EvidenceAdd {
        profile_id: String,
        #[arg(long)]
        title: String,
        #[arg(long)]
        evidence_type: String,
        #[arg(long, default_value = "needs_review")]
        visibility: String,
        #[arg(long)]
        summary: String,
        #[arg(long)]
        proof_url: Option<String>,
        #[arg(long)]
        local_path: Option<String>,
        #[arg(long)]
        source_date: Option<String>,
        #[arg(long, default_value = "user_claimed")]
        confidence: String,
        #[arg(long = "tag")]
        tags: Vec<String>,
        #[arg(long)]
        safe_application_text: String,
        #[arg(long = "unsafe-term")]
        unsafe_terms: Vec<String>,
        #[arg(long, default_value = "{}")]
        metadata_json: String,
    },
    Evidence {
        evidence_id: String,
    },
    EvidenceList {
        profile_id: String,
    },
    EvidenceReview {
        profile_id: String,
    },
    EvidenceClaimAdd {
        evidence_card_id: String,
        #[arg(long)]
        claim: String,
        #[arg(long)]
        claim_kind: String,
        #[arg(long, default_value = "unverified")]
        proof_level: String,
        #[arg(long)]
        can_use_in_resume: bool,
        #[arg(long)]
        can_use_in_outreach: bool,
        #[arg(long)]
        can_use_in_interview: bool,
    },
    PrivacyRuleAdd {
        #[arg(long)]
        pattern: String,
        #[arg(long, default_value = "blocked_term")]
        rule_type: String,
        #[arg(long, default_value = "block")]
        severity: String,
        #[arg(long)]
        replacement_guidance: Option<String>,
    },
    PrivacyCheck {
        #[arg(long)]
        artifact_type: String,
        #[arg(long)]
        artifact_id: Option<String>,
        #[arg(long)]
        text: String,
        #[arg(long = "blocked-term")]
        blocked_terms: Vec<String>,
    },
    SourceAdd {
        #[arg(long)]
        source_family: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        url: String,
        #[arg(long)]
        market_scope: String,
        #[arg(long, default_value = "manual")]
        refresh_policy: String,
        #[arg(long, default_value = "{}")]
        metadata_json: String,
    },
    SourceHealthAdd {
        source_id: String,
        #[arg(long)]
        status: String,
        #[arg(long)]
        http_status: Option<i64>,
        #[arg(long)]
        error_code: Option<String>,
        #[arg(long, default_value_t = 0)]
        fetched_count: usize,
        #[arg(long, default_value_t = 0)]
        accepted_count: usize,
        #[arg(long, default_value_t = 0)]
        rejected_count: usize,
        #[arg(long)]
        note: Option<String>,
    },
    SourceRefresh {
        source_id: String,
        #[arg(long)]
        body: Option<String>,
        #[arg(long)]
        body_path: Option<PathBuf>,
        #[arg(long)]
        fetched_url: Option<String>,
        #[arg(long)]
        fetch_live: bool,
    },
    RadarSchedule {
        profile_id: String,
        #[arg(long)]
        scope: String,
        #[arg(long = "source-id")]
        source_ids: Vec<String>,
        #[arg(long)]
        fetch_live: bool,
        #[arg(long, default_value = "{}")]
        source_snapshots_json: String,
        #[arg(long)]
        source_snapshots_path: Option<PathBuf>,
        #[arg(long, default_value = "warm")]
        cadence: String,
        #[arg(long, default_value = "active")]
        status: String,
        #[arg(long)]
        email_to: Option<String>,
        #[arg(long)]
        delivery_idempotency_key: Option<String>,
    },
    RadarEnqueue {
        profile_id: String,
        #[arg(long)]
        scope: String,
        #[arg(long = "source-id")]
        source_ids: Vec<String>,
        #[arg(long)]
        fetch_live: bool,
        #[arg(long, default_value = "{}")]
        source_snapshots_json: String,
        #[arg(long)]
        source_snapshots_path: Option<PathBuf>,
        #[arg(long)]
        email_to: Option<String>,
        #[arg(long)]
        delivery_idempotency_key: Option<String>,
    },
    RoleAdd {
        #[arg(long)]
        company: String,
        #[arg(long)]
        role_title: String,
        #[arg(long)]
        canonical_url: Option<String>,
        #[arg(long)]
        source_family: String,
        #[arg(long)]
        source_url: String,
        #[arg(long, default_value = "unknown")]
        source_confidence: String,
        #[arg(long)]
        date_accessed: Option<String>,
        #[arg(long, default_value = "unknown")]
        posting_freshness: String,
        #[arg(long)]
        location: Option<String>,
        #[arg(long)]
        work_mode: Option<String>,
        #[arg(long)]
        company_stage_or_size: Option<String>,
        #[arg(long)]
        role_seniority: Option<String>,
        #[arg(long = "requirement")]
        core_requirements: Vec<String>,
        #[arg(long)]
        implied_business_problem: Option<String>,
        #[arg(long)]
        why_they_might_need_user: Option<String>,
        #[arg(long = "evidence-card-id")]
        evidence_card_ids: Vec<String>,
        #[arg(long = "gap-or-blocker")]
        gaps_or_blockers: Vec<String>,
        #[arg(long)]
        cluster: Option<String>,
        #[arg(long, default_value = "unknown")]
        current_status: String,
        #[arg(long, default_value = "{}")]
        metadata_json: String,
    },
    Role {
        role_id: String,
    },
    Roles,
    RoleSourceLinkAdd {
        role_id: String,
        #[arg(long)]
        source_id: Option<String>,
        #[arg(long)]
        source_url: String,
        #[arg(long, default_value = "unknown")]
        confidence: String,
        #[arg(long)]
        evidence_excerpt: Option<String>,
    },
    ScoreAdd {
        role_id: String,
        #[arg(long)]
        profile_id: String,
        #[arg(long, default_value = "human")]
        scorer: String,
        #[arg(long)]
        role_fit: f64,
        #[arg(long)]
        domain_fit: f64,
        #[arg(long)]
        evidence_fit: f64,
        #[arg(long)]
        geo_work_fit: f64,
        #[arg(long)]
        stage_fit: f64,
        #[arg(long)]
        practical_odds: f64,
        #[arg(long)]
        interest_energy: f64,
        #[arg(long = "blocker")]
        blockers: Vec<String>,
        #[arg(long = "evidence-card-id")]
        evidence_card_ids: Vec<String>,
        #[arg(long)]
        explanation: String,
    },
    Shortlist {
        profile_id: String,
    },
    OutreachReadiness {
        profile_id: String,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    CompanyTargets {
        profile_id: String,
        #[arg(long)]
        market: Option<String>,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    SkepticFindingAdd {
        role_id: String,
        #[arg(long)]
        severity: String,
        #[arg(long)]
        finding_type: String,
        #[arg(long)]
        finding: String,
        #[arg(long)]
        next_action: Option<String>,
    },
    PacketCreate {
        role_id: String,
        #[arg(long)]
        profile_id: String,
        #[arg(long = "evidence-card-id")]
        evidence_card_ids: Vec<String>,
        #[arg(long)]
        resume_emphasis: String,
        #[arg(long = "tailored-bullet")]
        tailored_bullets: Vec<String>,
        #[arg(long)]
        outreach_note: String,
        #[arg(long, default_value = "{}")]
        proof_links_json: String,
        #[arg(long = "likely-objection")]
        likely_objections: Vec<String>,
        #[arg(long = "interview-story")]
        interview_stories: Vec<String>,
        #[arg(long = "question-to-ask")]
        questions_to_ask: Vec<String>,
        #[arg(long)]
        reviewer_note: Option<String>,
    },
    Packet {
        packet_id: String,
    },
    PacketApprove {
        packet_id: String,
        #[arg(long)]
        reviewer_note: String,
    },
    PacketExport {
        packet_id: String,
        #[arg(long)]
        out: PathBuf,
    },
    PacketExportSet {
        profile_id: String,
        #[arg(long = "packet-id")]
        packet_ids: Vec<String>,
        #[arg(long)]
        out: PathBuf,
    },
    CompanyAdd {
        #[arg(long)]
        company_name: String,
        #[arg(long)]
        website_url: String,
        #[arg(long)]
        source_family: String,
        #[arg(long)]
        market: String,
        #[arg(long)]
        stage: Option<String>,
        #[arg(long)]
        funding_signal: Option<String>,
        #[arg(long)]
        product_category: Option<String>,
        #[arg(long)]
        technical_audience: Option<String>,
        #[arg(long, default_value_t = 0.0)]
        developer_facing_score: f64,
        #[arg(long)]
        london_relevance: String,
        #[arg(long)]
        remote_maturity: Option<String>,
        #[arg(long)]
        hiring_page_url: Option<String>,
        #[arg(long)]
        founder_or_team_signal: Option<String>,
        #[arg(long, default_value = "{}")]
        metadata_json: String,
    },
    ContactAdd {
        #[arg(long)]
        name: String,
        #[arg(long)]
        company_id: Option<String>,
        #[arg(long)]
        role_title: Option<String>,
        #[arg(long)]
        public_profile_url: String,
        #[arg(long)]
        source_url: String,
        #[arg(long, default_value = "unknown")]
        relationship_status: String,
        #[arg(long)]
        relevance: String,
        #[arg(long)]
        note: Option<String>,
    },
    IntroAdd {
        role_id: String,
        #[arg(long)]
        contact_id: String,
        #[arg(long, default_value = "unknown")]
        path_type: String,
        #[arg(long, default_value = "weak")]
        confidence: String,
        #[arg(long)]
        next_action: Option<String>,
        #[arg(long, default_value = "identify")]
        status: String,
    },
    SearchRunAdd {
        profile_id: String,
        #[arg(long)]
        scope: String,
        #[arg(long, default_value = "local_proof")]
        proof_level: String,
        #[arg(long, default_value_t = 0)]
        source_count: usize,
        #[arg(long, default_value_t = 0)]
        role_count: usize,
        #[arg(long, default_value_t = 0)]
        new_role_count: usize,
        #[arg(long, default_value_t = 0)]
        stale_role_count: usize,
        #[arg(long, default_value_t = 0)]
        error_count: usize,
        #[arg(long)]
        report_artifact_id: Option<String>,
        #[arg(long)]
        completed_at: Option<String>,
    },
    RoleStatusAdd {
        role_id: String,
        #[arg(long)]
        run_id: Option<String>,
        #[arg(long)]
        status: String,
        #[arg(long)]
        previous_tier: Option<String>,
        #[arg(long)]
        current_tier: Option<String>,
        #[arg(long)]
        note: Option<String>,
    },
    ApplicationRecord {
        role_id: String,
        #[arg(long)]
        packet_id: Option<String>,
        #[arg(long)]
        status: String,
        #[arg(long)]
        applied_at: Option<String>,
        #[arg(long)]
        follow_up_at: Option<String>,
        #[arg(long)]
        outcome_note: Option<String>,
    },
    Refresh {
        profile_id: String,
        #[arg(long)]
        scope: String,
        #[arg(long = "observed-role-id")]
        observed_role_ids: Vec<String>,
        #[arg(long = "stale-role-id")]
        stale_role_ids: Vec<String>,
        #[arg(long = "closed-role-id")]
        closed_role_ids: Vec<String>,
        #[arg(long = "source-health-id")]
        source_health_ids: Vec<String>,
        #[arg(long, default_value = "local_proof")]
        proof_level: String,
        #[arg(long)]
        report_artifact_id: Option<String>,
    },
    RefreshAudit {
        profile_id: String,
        #[arg(long)]
        scope: String,
        #[arg(long, default_value_t = 24)]
        min_elapsed_hours: i64,
    },
    OperationalAudit {
        profile_id: String,
        #[arg(long)]
        scope: String,
        #[arg(long, default_value_t = 24)]
        min_elapsed_hours: i64,
    },
    WeeklyReport {
        profile_id: String,
        #[arg(long)]
        scope: String,
    },
    WeeklyReportDeliveryPrepare {
        report_id: String,
        #[arg(long)]
        channel: String,
        #[arg(long)]
        subject: String,
        #[arg(long)]
        target: String,
        #[arg(long)]
        idempotency_key: Option<String>,
    },
    WeeklyReportDeliverySend {
        delivery_id: String,
        #[arg(long)]
        telegram_bot_token: Option<String>,
        #[arg(long)]
        email_account_id: Option<String>,
        #[arg(long)]
        email_api_token: Option<String>,
        #[arg(long)]
        email_from: Option<String>,
        #[arg(long)]
        api_base: Option<String>,
    },
    WeeklyReportDeliveries {
        #[arg(long)]
        report_id: Option<String>,
    },
}

#[derive(Args, Clone)]
pub(crate) struct ResearchConvergenceArgs {
    pub(crate) run_id: String,
    #[arg(long)]
    pub(crate) max_iterations: Option<usize>,
    #[arg(long)]
    pub(crate) max_seconds: Option<i64>,
    #[arg(long)]
    pub(crate) max_sources: Option<usize>,
    #[arg(long)]
    pub(crate) max_provider_calls: Option<usize>,
    #[arg(long)]
    pub(crate) cost_cap_usd: Option<f64>,
    #[arg(long)]
    pub(crate) source_novelty_threshold: Option<f64>,
    #[arg(long)]
    pub(crate) confidence_delta_threshold: Option<f64>,
    #[arg(long)]
    pub(crate) no_progress_iteration_limit: Option<usize>,
    #[arg(long)]
    pub(crate) require_active_fact_check: Option<bool>,
    #[arg(long)]
    pub(crate) allow_long_run: Option<bool>,
    #[arg(long)]
    pub(crate) no_write: Option<bool>,
    #[arg(long)]
    pub(crate) editorial_provider: Option<String>,
    #[arg(long)]
    pub(crate) editorial_model_name: Option<String>,
    #[arg(long)]
    pub(crate) editorial_endpoint: Option<String>,
    #[arg(long)]
    pub(crate) editorial_timeout_seconds: Option<u64>,
}

pub(crate) fn research_convergence_step_input(
    args: ResearchConvergenceArgs,
) -> ResearchConvergenceStepInput {
    ResearchConvergenceStepInput {
        run_id: args.run_id,
        max_iterations: args.max_iterations,
        max_seconds: args.max_seconds,
        max_sources: args.max_sources,
        max_provider_calls: args.max_provider_calls,
        cost_cap_usd: args.cost_cap_usd,
        source_novelty_threshold: args.source_novelty_threshold,
        confidence_delta_threshold: args.confidence_delta_threshold,
        no_progress_iteration_limit: args.no_progress_iteration_limit,
        require_active_fact_check: args.require_active_fact_check,
        allow_long_run: args.allow_long_run,
        no_write: args.no_write,
        editorial_provider: args.editorial_provider,
        editorial_model_name: args.editorial_model_name,
        editorial_endpoint: args.editorial_endpoint,
        editorial_timeout_seconds: args.editorial_timeout_seconds,
    }
}

pub(crate) fn research_convergence_start_input(
    args: ResearchConvergenceArgs,
) -> ResearchConvergenceStartInput {
    ResearchConvergenceStartInput {
        run_id: args.run_id,
        max_iterations: args.max_iterations,
        max_seconds: args.max_seconds,
        max_sources: args.max_sources,
        max_provider_calls: args.max_provider_calls,
        cost_cap_usd: args.cost_cap_usd,
        source_novelty_threshold: args.source_novelty_threshold,
        confidence_delta_threshold: args.confidence_delta_threshold,
        no_progress_iteration_limit: args.no_progress_iteration_limit,
        require_active_fact_check: args.require_active_fact_check,
        allow_long_run: args.allow_long_run,
        no_write: args.no_write,
        editorial_provider: args.editorial_provider,
        editorial_model_name: args.editorial_model_name,
        editorial_endpoint: args.editorial_endpoint,
        editorial_timeout_seconds: args.editorial_timeout_seconds,
    }
}

#[derive(Args, Clone)]
pub(crate) struct ResearchConvergenceCloseLoopArgs {
    pub(crate) run_id: String,
    #[arg(long)]
    pub(crate) artifact_id: Option<String>,
    #[arg(long)]
    pub(crate) max_sentences: Option<usize>,
    #[arg(long)]
    pub(crate) no_challenges: bool,
    #[arg(long)]
    pub(crate) no_compile_report_before_check: bool,
    #[arg(long)]
    pub(crate) no_rerun_after_check: bool,
    #[arg(long)]
    pub(crate) no_compile_final_report: bool,
    #[arg(long)]
    pub(crate) provider: Option<String>,
    #[arg(long)]
    pub(crate) provider_max_tasks: Option<usize>,
    #[arg(long)]
    pub(crate) provider_max_results: Option<usize>,
    #[arg(long)]
    pub(crate) provider_max_provider_calls: Option<usize>,
    #[arg(long)]
    pub(crate) enqueue_selected_url_ingest: bool,
    #[arg(long)]
    pub(crate) max_ingest_jobs: Option<usize>,
    #[arg(long)]
    pub(crate) provider_cost_cap_usd: Option<f64>,
    #[arg(long)]
    pub(crate) provider_endpoint: Option<String>,
    #[arg(long)]
    pub(crate) provider_api_key: Option<String>,
    #[arg(long)]
    pub(crate) provider_model: Option<String>,
    #[arg(long)]
    pub(crate) provider_timeout_seconds: Option<u64>,
    #[arg(long)]
    pub(crate) max_iterations: Option<usize>,
    #[arg(long)]
    pub(crate) max_seconds: Option<i64>,
    #[arg(long)]
    pub(crate) max_sources: Option<usize>,
    #[arg(long)]
    pub(crate) max_provider_calls: Option<usize>,
    #[arg(long)]
    pub(crate) cost_cap_usd: Option<f64>,
    #[arg(long)]
    pub(crate) source_novelty_threshold: Option<f64>,
    #[arg(long)]
    pub(crate) confidence_delta_threshold: Option<f64>,
    #[arg(long)]
    pub(crate) no_progress_iteration_limit: Option<usize>,
    #[arg(long)]
    pub(crate) require_active_fact_check: Option<bool>,
    #[arg(long)]
    pub(crate) allow_long_run: Option<bool>,
    #[arg(long)]
    pub(crate) no_write: Option<bool>,
    #[arg(long)]
    pub(crate) editorial_provider: Option<String>,
    #[arg(long)]
    pub(crate) editorial_model_name: Option<String>,
    #[arg(long)]
    pub(crate) editorial_endpoint: Option<String>,
    #[arg(long)]
    pub(crate) editorial_timeout_seconds: Option<u64>,
}

pub(crate) fn research_convergence_close_loop_input(
    args: ResearchConvergenceCloseLoopArgs,
) -> ResearchConvergenceCloseLoopInput {
    ResearchConvergenceCloseLoopInput {
        run_id: args.run_id,
        artifact_id: args.artifact_id,
        max_sentences: args.max_sentences,
        create_challenges: Some(!args.no_challenges),
        compile_report_before_check: Some(!args.no_compile_report_before_check),
        rerun_after_check: Some(!args.no_rerun_after_check),
        compile_final_report: Some(!args.no_compile_final_report),
        provider: args.provider,
        provider_max_tasks: args.provider_max_tasks,
        provider_max_results: args.provider_max_results,
        provider_max_provider_calls: args.provider_max_provider_calls,
        enqueue_selected_url_ingest: Some(args.enqueue_selected_url_ingest),
        max_ingest_jobs: args.max_ingest_jobs,
        provider_cost_cap_usd: args.provider_cost_cap_usd,
        provider_endpoint: args.provider_endpoint,
        provider_api_key: args.provider_api_key,
        provider_model: args.provider_model,
        provider_timeout_seconds: args.provider_timeout_seconds,
        max_iterations: args.max_iterations,
        max_seconds: args.max_seconds,
        max_sources: args.max_sources,
        max_provider_calls: args.max_provider_calls,
        cost_cap_usd: args.cost_cap_usd,
        source_novelty_threshold: args.source_novelty_threshold,
        confidence_delta_threshold: args.confidence_delta_threshold,
        no_progress_iteration_limit: args.no_progress_iteration_limit,
        require_active_fact_check: args.require_active_fact_check,
        allow_long_run: args.allow_long_run,
        no_write: args.no_write,
        editorial_provider: args.editorial_provider,
        editorial_model_name: args.editorial_model_name,
        editorial_endpoint: args.editorial_endpoint,
        editorial_timeout_seconds: args.editorial_timeout_seconds,
    }
}

#[derive(Args)]
pub(crate) struct RadarCommand {
    #[command(subcommand)]
    pub(crate) command: RadarSubcommand,
}

#[derive(Subcommand)]
pub(crate) enum RadarSubcommand {
    Profile {
        #[command(subcommand)]
        command: RadarProfileSubcommand,
    },
    Run {
        profile: String,
        #[arg(long)]
        window_hours: Option<i64>,
        #[arg(long)]
        fetch_live: bool,
    },
    Enqueue {
        profile: String,
        #[arg(long)]
        window_hours: Option<i64>,
        #[arg(long)]
        fetch_live: bool,
    },
    Runs,
    Stage {
        run_id: String,
    },
    Summarize {
        run_id: String,
        #[arg(long, default_value = "en")]
        language: String,
        #[arg(long, default_value = "markdown")]
        format: String,
    },
    Summary {
        run_id: String,
        #[arg(long, default_value = "en")]
        language: String,
        #[arg(long, default_value = "markdown")]
        format: String,
    },
    Deliver {
        run_id: String,
        #[arg(long, default_value = "telegram")]
        channel: String,
        #[arg(long)]
        recipient: String,
        #[arg(long, default_value = "en")]
        language: String,
        #[arg(long, default_value = "markdown")]
        format: String,
        #[arg(long)]
        idempotency_key: Option<String>,
        #[arg(long)]
        bot_token: Option<String>,
        #[arg(long)]
        account_id: Option<String>,
        #[arg(long)]
        api_token: Option<String>,
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        api_base: Option<String>,
    },
    Deliveries {
        #[arg(long)]
        run_id: Option<String>,
    },
    Audit {
        run_id: String,
    },
    SourceQuality {
        run_id: String,
    },
    SourceQualityTrends {
        #[arg(long, default_value_t = 2)]
        min_windows: usize,
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    ModelScore {
        run_id: String,
        #[arg(long, default_value = "mock")]
        provider: String,
        #[arg(long)]
        model: Option<String>,
        #[arg(long, default_value_t = 10)]
        max_items: usize,
        #[arg(long)]
        endpoint: Option<String>,
        #[arg(long)]
        api_key: Option<String>,
    },
    RepairFts {
        #[arg(long)]
        run_id: Option<String>,
    },
}

#[derive(Subcommand)]
pub(crate) enum RadarProfileSubcommand {
    Create {
        name: String,
        #[arg(long, default_value = "")]
        description: String,
        #[arg(long, default_value_t = 24)]
        window_hours: i64,
        #[arg(long, default_value_t = 5.0)]
        min_score: f64,
        #[arg(long)]
        max_items: Option<i64>,
        #[arg(long, value_delimiter = ',', default_value = "en")]
        language: Vec<String>,
        #[arg(long = "source-card-query")]
        source_card_query: Vec<String>,
        #[arg(long = "selector-json")]
        selector_json: Vec<String>,
        #[arg(long = "delivery-policy-json")]
        delivery_policy_json: Option<String>,
        #[arg(long = "model-policy-json")]
        model_policy_json: Option<String>,
        #[arg(long = "metadata-json")]
        metadata_json: Option<String>,
    },
    List,
    Read {
        profile: String,
    },
}

#[derive(Args)]
pub(crate) struct XCommand {
    #[command(subcommand)]
    pub(crate) command: XSubcommand,
}

#[derive(Args)]
pub(crate) struct TelegramCommand {
    #[command(subcommand)]
    pub(crate) command: TelegramSubcommand,
}

#[derive(Args)]
pub(crate) struct EmailCommand {
    #[command(subcommand)]
    pub(crate) command: EmailSubcommand,
}

#[derive(Args)]
pub(crate) struct EdgeCommand {
    #[command(subcommand)]
    pub(crate) command: EdgeSubcommand,
}

#[derive(Args)]
pub(crate) struct ProjectCommand {
    #[command(subcommand)]
    pub(crate) command: ProjectSubcommand,
}

#[derive(Args)]
pub(crate) struct ControllerCommand {
    #[command(subcommand)]
    pub(crate) command: ControllerSubcommand,
}

#[derive(Args)]
pub(crate) struct WorkCommand {
    #[command(subcommand)]
    pub(crate) command: WorkSubcommand,
}

#[derive(Subcommand)]
pub(crate) enum ProjectSubcommand {
    Create {
        name: String,
        summary: String,
        #[arg(long = "alias")]
        aliases: Vec<String>,
    },
    List,
    Resolve {
        query: String,
        #[arg(long)]
        context_project_id: Option<String>,
    },
    StatusRecord {
        project_id: String,
        status: String,
        summary: String,
        #[arg(long, default_value = "manual")]
        source: String,
        #[arg(long)]
        thread_ref: Option<String>,
        #[arg(long, default_value_t = 0.5)]
        confidence: f64,
    },
    StatusSyncRecord {
        project_id: String,
        status: String,
        summary: String,
        #[arg(long)]
        host: String,
        #[arg(long)]
        thread_id: String,
        #[arg(long, default_value_t = 0.8)]
        confidence: f64,
        #[arg(long)]
        stale_after_seconds: Option<i64>,
    },
    StatusGet {
        project_id: String,
        #[arg(long)]
        channel: Option<String>,
        #[arg(long)]
        subject: Option<String>,
    },
}

#[derive(Subcommand)]
pub(crate) enum ControllerSubcommand {
    Route {
        #[arg(long, default_value = "telegram")]
        channel: String,
        #[arg(long)]
        account_id: Option<String>,
        #[arg(long)]
        conversation_id: String,
        #[arg(long)]
        sender: String,
        text: String,
    },
    ThreadUpsert {
        host: String,
        host_thread_id: String,
        #[arg(long)]
        project_id: Option<String>,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        cwd: Option<String>,
        #[arg(long)]
        branch: Option<String>,
        #[arg(long)]
        worktree: Option<String>,
        #[arg(long, default_value = "active")]
        status: String,
        #[arg(long, default_value_t = true)]
        active: bool,
        #[arg(long)]
        archived: bool,
        #[arg(long)]
        current_goal: Option<String>,
        #[arg(long)]
        latest_summary: Option<String>,
        #[arg(long)]
        latest_summary_source: Option<String>,
        #[arg(long)]
        last_activity_at: Option<String>,
    },
    Threads {
        #[arg(long)]
        project_id: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long, default_value_t = 25)]
        limit: usize,
    },
    ThreadGet {
        id: String,
    },
    RunCreate {
        #[arg(long)]
        thread_id: Option<String>,
        #[arg(long)]
        project_id: Option<String>,
        #[arg(long)]
        origin_channel_message_id: Option<String>,
        #[arg(long, default_value = "codex")]
        host: String,
        #[arg(long)]
        host_run_id: Option<String>,
        #[arg(long, default_value = "work")]
        kind: String,
        #[arg(long, default_value = "running")]
        status: String,
        requested_action: String,
    },
    Runs {
        #[arg(long)]
        project_id: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long, default_value_t = 25)]
        limit: usize,
    },
    RunGet {
        id: String,
    },
    RunUpdate {
        run_id: String,
        status: String,
        #[arg(long)]
        host_run_id: Option<String>,
    },
    Stop {
        run_id: String,
        reason: String,
    },
    Event {
        #[arg(long)]
        run_id: Option<String>,
        #[arg(long)]
        thread_id: Option<String>,
        #[arg(long)]
        project_id: Option<String>,
        event_type: String,
        summary: String,
        #[arg(long, default_value = "{}")]
        data: String,
        #[arg(long, default_value = "manual-cli")]
        source: String,
    },
    Events {
        #[arg(long)]
        run_id: Option<String>,
        #[arg(long)]
        project_id: Option<String>,
        #[arg(long, default_value_t = 25)]
        limit: usize,
    },
    Pending {
        #[arg(long)]
        status: Option<String>,
        #[arg(long, default_value_t = 25)]
        limit: usize,
    },
    PendingResolve {
        id: String,
        status: String,
        #[arg(long)]
        thread_id: Option<String>,
        #[arg(long)]
        run_id: Option<String>,
    },
}

#[derive(Subcommand)]
pub(crate) enum WorkSubcommand {
    Start {
        goal: String,
        #[arg(long)]
        project_id: Option<String>,
        #[arg(long)]
        host_id: Option<String>,
        #[arg(long)]
        thread_id: Option<String>,
        #[arg(long, default_value = "manual-cli")]
        agent_surface: String,
    },
    Event {
        run_id: String,
        event_type: String,
        summary: String,
        #[arg(long, default_value = "{}")]
        data: String,
    },
    ArtifactAdd {
        run_id: String,
        artifact_type: String,
        locator: String,
        #[arg(long, default_value = "evidence")]
        role: String,
        #[arg(long, default_value = "{}")]
        metadata: String,
    },
    LinkAdd {
        run_id: String,
        target_type: String,
        target_id: String,
        #[arg(long, default_value = "evidence")]
        role: String,
        #[arg(long)]
        generated_summary: bool,
    },
    Finish {
        run_id: String,
        status: String,
        outcome: String,
        #[arg(long)]
        validation_summary: Option<String>,
        #[arg(long = "follow-up")]
        follow_ups: Vec<String>,
        #[arg(long = "lesson")]
        reusable_lessons: Vec<String>,
    },
    Search {
        #[arg(long)]
        query: Option<String>,
        #[arg(long)]
        project_id: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long, default_value_t = 25)]
        limit: usize,
    },
    Read {
        run_id: String,
    },
    Stale {
        #[arg(long, default_value_t = 7)]
        max_age_days: i64,
        #[arg(long, default_value_t = 25)]
        limit: usize,
    },
    FollowUps {
        #[arg(long, default_value_t = 25)]
        limit: usize,
    },
    ConsolidationCandidates {
        #[arg(long, default_value_t = 25)]
        limit: usize,
    },
    RetrievalContext {
        query: String,
        #[arg(long, default_value_t = 7)]
        stale_after_days: i64,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    Consolidate {
        run_id: String,
        #[arg(long)]
        write_project_status: bool,
    },
}

#[derive(Subcommand)]
pub(crate) enum TelegramSubcommand {
    Drain {
        #[arg(long, default_value_t = 25)]
        max_events: usize,
    },
    Authorize {
        subject: String,
        #[arg(long)]
        read_projects: bool,
        #[arg(long)]
        write_projects: bool,
        #[arg(long)]
        send: bool,
    },
    Authorizations,
    Deliveries {
        #[arg(long)]
        message_id: Option<String>,
    },
    Send {
        chat_id: String,
        text: String,
        #[arg(long)]
        bot_token: Option<String>,
        #[arg(long)]
        api_base: Option<String>,
    },
    RetryDue {
        #[arg(long)]
        bot_token: Option<String>,
        #[arg(long)]
        api_base: Option<String>,
        #[arg(long, default_value_t = 25)]
        max_attempts: usize,
    },
}

#[derive(Subcommand)]
pub(crate) enum EmailSubcommand {
    Drain {
        #[arg(long, default_value_t = 25)]
        max_events: usize,
    },
    Poll {
        #[arg(long)]
        url: Option<String>,
        #[arg(long)]
        secret: Option<String>,
        #[arg(long, default_value_t = 25)]
        max_events: usize,
    },
    Authorize {
        address: String,
        #[arg(long)]
        read_projects: bool,
        #[arg(long)]
        write_projects: bool,
        #[arg(long)]
        send: bool,
    },
    Send {
        to: String,
        subject: String,
        text: String,
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        html: Option<String>,
        #[arg(long)]
        account_id: Option<String>,
        #[arg(long)]
        api_token: Option<String>,
        #[arg(long)]
        api_base: Option<String>,
    },
    Reply {
        message_id: String,
        text: String,
        #[arg(long)]
        subject: Option<String>,
        #[arg(long)]
        html: Option<String>,
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        account_id: Option<String>,
        #[arg(long)]
        api_token: Option<String>,
        #[arg(long)]
        api_base: Option<String>,
    },
}

#[derive(Subcommand)]
pub(crate) enum EdgeSubcommand {
    DrainRemote {
        #[arg(long)]
        url: Option<String>,
        #[arg(long)]
        secret: Option<String>,
        #[arg(long, default_value_t = 25)]
        max_events: usize,
    },
}

#[derive(Subcommand)]
pub(crate) enum XSubcommand {
    ImportJson {
        path: PathBuf,
    },
    DiscoverArchives {
        #[arg(long = "dir")]
        dirs: Vec<PathBuf>,
        #[arg(long, default_value_t = 25)]
        limit: usize,
    },
    ImportArchive {
        path: PathBuf,
        #[arg(long, value_delimiter = ',')]
        select: Vec<String>,
        #[arg(long, default_value_t = 10000)]
        limit: usize,
    },
    ExportPortable {
        #[arg(long)]
        out: PathBuf,
    },
    ValidatePortable {
        dir: PathBuf,
    },
    ImportPortable {
        dir: PathBuf,
    },
    RecentSearch {
        query: String,
        #[arg(long, default_value_t = 10)]
        max_results: usize,
    },
    EnqueueRecentSearch {
        query: String,
        #[arg(long, default_value_t = 10)]
        max_results: usize,
    },
    ImportBookmarks {
        #[arg(long, default_value_t = 92)]
        bookmark_days: i64,
        #[arg(long, default_value_t = 100)]
        max_bookmarks: usize,
    },
    ScheduleBookmarks {
        #[arg(long, default_value_t = 92)]
        bookmark_days: i64,
        #[arg(long, default_value_t = 1000)]
        max_bookmarks: usize,
        #[arg(long, default_value = "warm")]
        cadence: String,
        #[arg(long, default_value = "active")]
        status: String,
    },
    ClusterRadarRun {
        run_id: String,
        #[arg(long, default_value_t = 20)]
        max_source_cards: usize,
    },
    EditorialDecide {
        cluster_id: String,
    },
    ImportFollowingWatchSources {
        #[arg(long, default_value_t = 1000)]
        max_users: usize,
    },
    RebuildDefinitiveWatchSources {
        #[arg(long, default_value_t = 92)]
        bookmark_days: i64,
        #[arg(long, default_value_t = 1000)]
        max_bookmarks: usize,
        #[arg(long, default_value_t = 100)]
        max_recent_follows: usize,
    },
    CurateWatchSources {
        #[arg(long, conflicts_with = "apply")]
        dry_run: bool,
        #[arg(long, conflicts_with = "dry_run")]
        apply: bool,
        #[arg(long, default_value = "pause-only")]
        mode: String,
    },
    RestoreWatchCuration {
        run_id: String,
    },
    WatchCurationReport {
        #[arg(long)]
        run_id: Option<String>,
    },
    ImportWatchManualRules {
        #[arg(long)]
        path: Option<PathBuf>,
        #[arg(long, default_value = "[]")]
        rules_json: String,
        #[arg(long)]
        reviewed_by: String,
        #[arg(long, conflicts_with = "apply")]
        dry_run: bool,
        #[arg(long, conflicts_with = "dry_run")]
        apply: bool,
    },
    EnrichWatchProfiles {
        #[arg(long)]
        run_id: Option<String>,
        #[arg(long = "handle")]
        handles: Vec<String>,
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    MonitorWatchSources {
        #[arg(long, default_value_t = 25)]
        max_sources: usize,
        #[arg(long, default_value_t = 10)]
        max_results_per_source: usize,
    },
    MonitorWatchSource {
        handle: String,
        #[arg(long, default_value_t = 10)]
        max_results_per_source: usize,
    },
    RepairHealth {
        #[arg(long, default_value_t = 24)]
        defer_rate_limited_hours: i64,
        #[arg(long, default_value_t = 10000)]
        limit: usize,
    },
    OauthProbe {
        #[arg(long)]
        search_query: Option<String>,
    },
    OauthUrl {
        #[arg(long)]
        client_id: Option<String>,
        #[arg(long)]
        redirect_uri: Option<String>,
        #[arg(long, value_delimiter = ',')]
        scopes: Vec<String>,
    },
    OauthExchange {
        #[arg(long)]
        client_id: Option<String>,
        #[arg(long)]
        redirect_uri: Option<String>,
        #[arg(long)]
        code: String,
        #[arg(long)]
        code_verifier: String,
        #[arg(long)]
        client_secret: Option<String>,
    },
    OauthReauthorize {
        #[arg(long)]
        client_id: Option<String>,
        #[arg(long)]
        redirect_uri: Option<String>,
        #[arg(long)]
        client_secret: Option<String>,
        #[arg(long, value_delimiter = ',')]
        scopes: Vec<String>,
        #[arg(long, default_value_t = 180)]
        timeout_seconds: u64,
        #[arg(long, default_value = "from:openai")]
        probe_search_query: String,
        #[arg(long)]
        no_open_browser: bool,
    },
    OauthRefresh {
        #[arg(long)]
        client_id: Option<String>,
        #[arg(long)]
        client_secret: Option<String>,
    },
    OauthRevoke {
        #[arg(long, default_value = "X_BEARER_TOKEN")]
        name: String,
        #[arg(long)]
        client_id: Option<String>,
        #[arg(long)]
        client_secret: Option<String>,
        #[arg(long)]
        token_type_hint: Option<String>,
        #[arg(long)]
        delete_local: bool,
    },
    List {
        #[arg(long)]
        query: Option<String>,
        #[arg(long)]
        source: Option<String>,
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    Bookmarks {
        #[arg(long)]
        query: Option<String>,
        #[arg(long, default_value_t = 25)]
        limit: usize,
    },
    SearchTweets {
        query: String,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    Research {
        query: String,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    Thread {
        x_id: String,
        #[arg(long, default_value_t = 50)]
        max_depth: usize,
    },
    ExtractLinks {
        #[arg(long, default_value_t = 1000)]
        limit: usize,
    },
    ExpandLinks {
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    Links {
        #[arg(long)]
        query: Option<String>,
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    RebuildFts,
    RepairProjections {
        #[arg(long, default_value_t = 1000)]
        limit: usize,
    },
    Stats,
    Report {
        #[arg(long)]
        query: Option<String>,
    },
}

#[derive(Subcommand)]
pub(crate) enum WikiSubcommand {
    Add {
        title: String,
        content: String,
        #[arg(long, default_value = "manual")]
        source: String,
    },
    IngestFile {
        path: PathBuf,
    },
    IngestDir {
        path: PathBuf,
    },
    ImportCodexSwiftSources {
        path: PathBuf,
    },
    Sources,
    Search {
        query: String,
    },
    IngestJob {
        path: PathBuf,
    },
    IngestUrl {
        url: String,
    },
    IngestRendered {
        requested_url: String,
        #[arg(long)]
        final_url: Option<String>,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        rendered_html: Option<String>,
        #[arg(long)]
        rendered_html_file: Option<PathBuf>,
        #[arg(long)]
        rendered_text: Option<String>,
        #[arg(long)]
        rendered_text_file: Option<PathBuf>,
        #[arg(long)]
        captured_at: Option<String>,
        #[arg(long)]
        browser: Option<String>,
        #[arg(long)]
        screenshot_path: Option<String>,
    },
    EnqueueRss {
        url: String,
    },
    EnqueueGithub {
        owner: String,
        repo: String,
        #[arg(long, default_value = "releases")]
        mode: String,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    EnqueueGithubOwner {
        owner: String,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    EnqueueArxiv {
        query: String,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    RunRss {
        url: String,
    },
    RunGithub {
        owner: String,
        repo: String,
        #[arg(long, default_value = "releases")]
        mode: String,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    RunGithubOwner {
        owner: String,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    RunArxiv {
        query: String,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    Compile {
        query: String,
    },
    Expand {
        topic: String,
    },
    Jobs,
    Job {
        id: String,
    },
    DecisionLedger {
        #[command(subcommand)]
        command: WikiDecisionLedgerSubcommand,
    },
    List,
    Read {
        id: String,
    },
}

#[derive(Subcommand)]
pub(crate) enum WikiDecisionLedgerSubcommand {
    Summary,
    List {
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
}

#[derive(Subcommand)]
pub(crate) enum SourceCardSubcommand {
    Add {
        #[arg(long)]
        title: String,
        #[arg(long)]
        url: String,
        #[arg(long, default_value = "web")]
        source_type: String,
        #[arg(long, default_value = "manual")]
        provider: String,
        #[arg(long)]
        summary: String,
        #[arg(long, default_value = "[]")]
        claims_json: String,
    },
    IngestRedditBrowserListing {
        #[arg(long)]
        locator: String,
        #[arg(long)]
        listing_json: PathBuf,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    Search {
        query: String,
    },
    Read {
        id: String,
    },
}
