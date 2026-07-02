use anyhow::{Context, Result, bail};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use calamine::{Data, Range, Reader, SheetType, SheetVisible, open_workbook_auto};
use chrono::{DateTime, Datelike, Duration as ChronoDuration, Local, TimeZone, Timelike, Utc};
use reqwest::StatusCode;
use reqwest::blocking::Client;
use reqwest::header::{
    ACCEPT, AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE, HeaderMap, HeaderValue, LOCATION,
    RETRY_AFTER,
};
use reqwest::redirect::Policy;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use url::Url;
use uuid::Uuid;
use walkdir::WalkDir;

pub const APP_NAME: &str = "arcwell";
pub const SCHEMA_VERSION: i64 = 25;
pub const SOURCE_CARD_SCHEMA_VERSION: u64 = 1;
const MAX_COST_USD: f64 = 1_000_000.0;
const SOURCE_CARD_STALE_DAYS: i64 = 180;
const PROJECT_SYNC_DEFAULT_STALE_AFTER_SECONDS: i64 = 6 * 60 * 60;
const PROJECT_SYNC_MAX_STALE_AFTER_SECONDS: i64 = 7 * 24 * 60 * 60;
const SECRET_EXPIRY_WARNING_WINDOW_SECONDS: i64 = 72 * 60 * 60;
const WORKER_HEARTBEAT_EVENT_RETENTION_DAYS: i64 = 14;
const X_MONITOR_MAX_SOURCES: usize = 1_000;
const X_MONITOR_MAX_RATE_LIMIT_FAILURES_PER_RUN: usize = 3;
const X_ARCHIVE_MAX_FILE_BYTES: u64 = 25_000_000;
const X_ARCHIVE_MAX_TOTAL_BYTES: u64 = 100_000_000;
const X_ARCHIVE_MAX_ENTRIES: usize = 5_000;
const X_ARCHIVE_DISCOVERY_MAX_PATHS: usize = 10_000;
const X_ARCHIVE_DISCOVERY_MAX_ZIP_ENTRIES: usize = 50;
const FETCH_TEXT_MAX_BYTES: u64 = 8_000_000;

mod types;
pub use types::*;

pub struct Store {
    paths: AppPaths,
    conn: Connection,
}

mod store;

mod backup;
pub use backup::*;
mod common;
pub use common::*;
mod knowledge;
pub(crate) use knowledge::*;
mod work_procedure;
pub(crate) use work_procedure::*;
mod policy_digest;
pub(crate) use policy_digest::*;
mod channel_project_memory;
pub use channel_project_memory::*;
mod db_schema;
pub(crate) use db_schema::*;
mod radar_digest;
pub(crate) use radar_digest::*;
mod health_validation;
pub(crate) use health_validation::*;
mod input_validation;
pub(crate) use input_validation::*;
mod health_checks;
pub(crate) use health_checks::*;
mod job_normalization;
pub(crate) use job_normalization::*;
mod commerce_research;
pub(crate) use commerce_research::*;
mod x_sources;
pub(crate) use x_sources::*;
mod research_reports_search;
pub(crate) use research_reports_search::*;

#[cfg(test)]
mod tests;
