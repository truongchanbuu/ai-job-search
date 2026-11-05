//! Shared wire contracts. No HTTP framework, database, or terminal dependency.
use crate::config::{Config, NotificationsConfig, RadarConfig, ScoringConfig};
use chrono::{DateTime, Utc};
use radar_domain::{ApplicationStatus, Evaluation, FeedbackAction, Job, Mode, Verdict};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorBody {
    pub error: ErrorDetail,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    pub status: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigView {
    pub revision: i64,
    pub effective: Config,
    pub updated_at: DateTime<Utc>,
    pub actor: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigUpdate {
    pub expected_revision: i64,
    pub radar: Option<RadarConfig>,
    pub scoring: Option<ScoringConfig>,
    pub notifications: Option<NotificationsConfig>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceView {
    pub name: String,
    pub enabled: bool,
    pub diagnostic: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub jobs: i64,
    pub candidates: i64,
    pub reviews: i64,
    pub feedback: i64,
    pub mode: Mode,
    pub config_revision: i64,
    pub last_import: Option<DateTime<Utc>>,
    pub sources: Vec<SourceView>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub personalization: Option<crate::learning::RankingExplanation>,
    pub id: Uuid,
    pub job: Job,
    pub evaluation: Evaluation,
    pub fetched_at: DateTime<Utc>,
    pub ingestions: i64,
    pub decision: Option<FeedbackAction>,
    pub revision: i64,
    pub application_status: ApplicationStatus,
    pub submitted_at: Option<DateTime<Utc>>,
    pub submission_reported_at: Option<DateTime<Utc>>,
    pub connects_spent: Option<i32>,
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobSort {
    #[default]
    Score,
    Newest,
    Title,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobQuery {
    pub search: Option<String>,
    pub verdict: Option<Verdict>,
    pub decision: Option<FeedbackAction>,
    pub sort: Option<JobSort>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPage {
    pub items: Vec<JobView>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationView {
    pub id: i64,
    pub job_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub config_revision: i64,
    pub profile_revision: String,
    pub input: Job,
    pub evaluation: Evaluation,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportSource {
    Mock,
    Manual,
    Upwork,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportRequest {
    pub idempotency_key: Uuid,
    pub source: ImportSource,
    pub jobs: Option<Vec<Job>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    #[serde(default)]
    pub new_jobs: usize,
    #[serde(default)]
    pub duplicates: usize,
    pub id: Uuid,
    pub status: String,
    pub processed: usize,
    pub canonical_jobs: usize,
    pub config_revision: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeedbackRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<radar_domain::ReasonCode>,
    pub idempotency_key: Uuid,
    pub expected_revision: i64,
    pub action: FeedbackAction,
    pub reason: Option<String>,
    #[serde(default)]
    pub confirmed_submission: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submitted_at: Option<DateTime<Utc>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackView {
    pub reason_code: Option<radar_domain::ReasonCode>,
    pub learning_excluded: bool,
    pub learning_version: i64,
    pub actual_effort_hours: Option<f64>,
    pub id: Uuid,
    pub job_id: Uuid,
    pub job_title: String,
    pub action: FeedbackAction,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub actor: String,
    pub revision: i64,
    pub application_status: ApplicationStatus,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeedbackQuery {
    pub job_id: Option<Uuid>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackPage {
    pub items: Vec<FeedbackView>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Pending,
    Running,
    Completed,
    Failed,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanRun {
    pub source: String,
    pub search_profile: Option<String>,
    pub id: Uuid,
    pub status: RunStatus,
    pub origin: String,
    pub scheduled_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub attempts: i32,
    pub result: Option<ImportResult>,
    pub error: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerView {
    pub active_workers: i64,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub enabled: bool,
    pub mode: Mode,
    pub next_scan: Option<DateTime<Utc>>,
    pub last_run: Option<ScanRun>,
    pub runs: Vec<ScanRun>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScanRequest {
    pub idempotency_key: Uuid,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModeRequest {
    pub idempotency_key: Uuid,
    pub expected_revision: i64,
    pub paused: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotStatus {
    pub instance: String,
    pub configured: bool,
    pub diagnostic: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryView {
    pub id: Uuid,
    pub kind: String,
    pub destination: String,
    pub provider: String,
    pub status: String,
    pub attempts: i32,
    pub next_attempt_at: DateTime<Utc>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub remote_id: Option<String>,
    pub error: Option<String>,
    pub title: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationHistory {
    pub deliveries: Vec<DeliveryView>,
    pub providers: Vec<BotStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpworkScanRequest {
    pub idempotency_key: Uuid,
    pub profile: Option<String>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpworkStatus {
    pub enabled: bool,
    pub execution_mode: String,
    pub connection: String,
    pub authentication: String,
    pub account_selected: bool,
    pub account_label: Option<String>,
    pub last_successful_search: Option<DateTime<Utc>>,
    pub last_error_category: Option<String>,
    pub fetched: usize,
    pub new_jobs: usize,
    pub duplicates: usize,
    pub accepted: usize,
    pub invalid: usize,
    pub filtered: usize,
    pub partial: bool,
    pub empty_reason: Option<String>,
    pub filters_ignored: Vec<String>,
    pub skills_filtered_client_side: bool,
    pub retry_after_seconds: Option<u64>,
}
