use anyhow::{Context, Result, bail};
#[cfg(test)]
use arcwell_core::WatchSourceInput;
use arcwell_core::{
    AdversarialReviewFindingInput, AdversarialReviewRunInput, AppPaths,
    CommerceAvailabilityProofInput, CommerceCandidateInput, CommerceContextFactInput,
    CommerceRenderedPageCheckInput, CommerceReportJudgmentInput, CommerceRunConfigInput,
    CommerceVerificationAttemptInput, DigestAlertScheduleInput, DoctorOptions, ImportRunFinish,
    IssueScheduleInput, JobApplicationInput, JobApplicationPacketInput,
    JobApplicationPacketStatusInput, JobCandidateProfileInput, JobCompanyCardInput,
    JobContactInput, JobEvidenceCardInput, JobEvidenceClaimInput, JobFitScoreInput,
    JobImportBatchInput, JobIntroPathInput, JobManualRefreshInput, JobPrivacyRuleInput,
    JobRoleCardInput, JobRoleSourceLinkInput, JobRoleStatusEventInput, JobSearchRunInput,
    JobSkepticFindingInput, JobSourceHealthInput, JobSourceInput, JobSourceRefreshInput,
    JobWeeklyReportDeliveryInput, JobWeeklyReportDeliverySendInput,
    KnowledgeClusterProposalModelInput, KnowledgeClusterWriterModelInput, KnowledgeEntityInput,
    KnowledgeEntityResolutionModelInput, OpsSnapshot, PolicyRequest, ProcedureCandidateInput,
    ProofArtifactInput, ProofCheckInput, ProofClaimInput, ProofPacketInput, RadarDeliveryInput,
    RadarProfileInput, RadarRun, RenderedPageSnapshotInput, ResearchActiveFactCheckInput,
    ResearchArtifactInput, ResearchConvergenceCloseLoopInput,
    ResearchConvergenceProviderSearchInput, ResearchConvergenceStartInput,
    ResearchConvergenceStepInput, ResearchDocumentInput, ResearchEditorialInvokeInput,
    ResearchEditorialRunInput, ResearchHostSearchInput, ResearchHostSearchResultInput,
    ResearchRoleRunStart, ResearchSourceInput, SourceCardInput, Store, WebSearchConfig,
    XStatsReport, XWatchManualRuleInput, personal_memory_eval_corpus,
};
use axum::{
    Json, Router,
    body::Bytes,
    extract::{Query, State, rejection::QueryRejection},
    http::{HeaderMap, HeaderValue, StatusCode, Uri, header},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::ffi::OsString;
use std::fs;
use std::io::{BufRead, Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

mod commands;
pub(crate) use commands::*;

mod server;
pub(crate) use server::*;
mod mcp;
pub(crate) use mcp::*;
mod import_claude;
pub(crate) use import_claude::*;

mod slash_alias;
pub(crate) use slash_alias::*;
mod cli_dispatch;
pub(crate) use cli_dispatch::*;
mod service_memory_cli;
pub(crate) use service_memory_cli::*;
mod knowledge_research_cli;
pub(crate) use knowledge_research_cli::*;
mod commerce_job_radar_cli;
pub(crate) use commerce_job_radar_cli::*;
mod x_cli;
pub(crate) use x_cli::*;
mod channels_work_cli;
pub(crate) use channels_work_cli::*;
mod import_policy_cli;
pub(crate) use import_policy_cli::*;
mod proof_cli;
pub(crate) use proof_cli::*;
mod guard_cli;
pub(crate) use guard_cli::*;

fn main() -> Result<()> {
    let args = std::env::args_os().collect::<Vec<_>>();
    let alias_resolution = resolve_slash_alias(args)?;
    if let SlashAliasResolution::Mcp {
        home,
        tool,
        arguments,
    } = alias_resolution
    {
        let paths = home
            .map(AppPaths::new)
            .map(Ok)
            .unwrap_or_else(AppPaths::from_env_or_default)?;
        print_json(&call_mcp_tool(&paths, tool, arguments)?)?;
        return Ok(());
    }
    if let SlashAliasResolution::HostOnly { alias, reason } = alias_resolution {
        bail!("/{alias} is a Codex-host slash command, not a standalone CLI command: {reason}");
    }
    let SlashAliasResolution::Cli(args) = alias_resolution else {
        unreachable!();
    };
    let cli = Cli::parse_from(args);
    let paths = cli
        .home
        .map(AppPaths::new)
        .map(Ok)
        .unwrap_or_else(AppPaths::from_env_or_default)?;

    match cli.command {
        Command::Serve(args) => tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?
            .block_on(serve(paths, args)),
        Command::Mcp => mcp(paths),
        Command::Backup(BackupCommand {
            command:
                BackupSubcommand::Restore {
                    from,
                    target_home,
                    replace,
                },
        }) => {
            let target_paths = target_home.map(AppPaths::new).unwrap_or(paths);
            print_json(&Store::restore_backup_path(&from, &target_paths, replace)?)
        }
        command => {
            let store = Store::open(paths)?;
            run(store, command)
        }
    }
}

#[cfg(test)]
mod tests;
