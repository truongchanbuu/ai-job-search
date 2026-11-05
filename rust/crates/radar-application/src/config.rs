use crate::ScanSettings;
use anyhow::{Context, Result, ensure};
use chrono::NaiveTime;
use radar_domain::{Mode, ScoringPolicy, Signal, Vetoes};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BotSettings {
    pub request_timeout_seconds: u64,
    pub delivery_lease_seconds: u64,
    pub max_backoff_seconds: u64,
    pub max_digest_jobs: u32,
    pub signature_max_age_seconds: i64,
    pub signature_future_skew_seconds: i64,
    pub acknowledgment_timeout_ms: u64,
    pub interaction_ttl_seconds: i64,
    pub interaction_encryption_key_env: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchedulerSettings {
    pub enabled: bool,
    pub poll_interval_seconds: u64,
    pub max_jobs_per_run: u32,
    pub max_pending_runs: u32,
    pub ui_refresh_seconds: u64,
    pub recovery_after_seconds: u64,
    pub catch_up_minutes: u32,
    pub resume_mode: Mode,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RadarConfig {
    pub schema_version: u32,
    pub app: App,
    pub modes: Modes,
    pub job_sources: Sources,
    pub ai: Ai,
    pub proposals: Proposals,
    pub storage: Storage,
    #[serde(default)]
    pub scheduler: Option<SchedulerSettings>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct App {
    pub timezone: String,
    pub active_mode: Mode,
    pub cloud_enabled: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Modes {
    pub employed: WorkMode,
    pub available: WorkMode,
    pub busy: WorkMode,
    pub paused: WorkMode,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkMode {
    pub weekly_freelance_hours: u32,
    pub schedule: Schedule,
    pub max_source_requests_per_day: u32,
    pub max_jobs_processed_per_day: u32,
    pub max_deep_evaluations_per_day: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Schedule {
    pub weekdays: Vec<Weekday>,
    pub at: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Weekday {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sources {
    pub mock: Enabled,
    pub manual: Enabled,
    pub upwork_mcp: Upwork,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Enabled {
    pub enabled: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Upwork {
    #[serde(default)]
    pub runtime: Option<UpworkSettings>,
    pub enabled: bool,
    pub require_capability_check: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpworkSettings {
    pub state_dir_env: String,
    pub encryption_key_env: String,
    pub client_id_env: String,
    pub client_metadata_url_env: String,
    pub request_timeout_seconds: u64,
    pub login_timeout_seconds: u64,
    pub callback_port: u16,
    pub max_requests_per_minute: u32,
    pub max_requests_per_hour: u32,
    pub max_requests_per_day: u32,
    pub max_pages_per_scan: u32,
    pub max_results_per_scan: u32,
    pub retry_backoff_seconds: u64,
    pub default_profile: String,
    pub profiles: BTreeMap<String, UpworkSearch>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpworkSearch {
    pub enabled: bool,
    pub keywords: Vec<String>,
    pub skills: Vec<String>,
    pub job_type: Option<String>,
    pub minimum_budget_usd: Option<f64>,
    pub experience_levels: Vec<String>,
    pub result_limit: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ai {
    #[serde(default)]
    pub runtime: Option<AiRuntime>,
    pub provider: String,
    pub max_tasks_per_day: u32,
    pub timeout_seconds: u32,
    pub max_concurrent_tasks: u32,
    pub max_estimated_cost_usd_per_day: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiRuntime {
    pub evaluation_rubric: String,
    pub enabled: bool,
    pub model: String,
    pub api_key_env: String,
    pub max_evaluations_per_day: u32,
    pub max_proposals_per_day: u32,
    pub max_input_bytes: usize,
    pub max_output_bytes: usize,
    pub max_output_tokens: u32,
    pub max_proposal_words: usize,
    pub reserved_cost_usd_per_call: f64,
    pub input_usd_per_million_tokens: Option<f64>,
    pub output_usd_per_million_tokens: Option<f64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Proposals {
    #[serde(default)]
    pub templates: Option<crate::proposal_templates::ProposalTemplates>,
    pub allow_auto_draft: bool,
    pub allow_marketplace_writes: bool,
    pub require_human_approval: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Storage {
    pub database_url_env: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScoringConfig {
    pub schema_version: u32,
    pub weights: BTreeMap<Signal, u32>,
    pub thresholds: Thresholds,
    pub hard_vetoes: Vetoes,
    pub uncertainty: Uncertainty,
    pub personalization: Personalization,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Thresholds {
    pub shortlist_min_score: u32,
    pub instant_alert_min_score: u32,
    pub minimum_confidence_to_alert: ConfidenceLevel,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfidenceLevel {
    Low,
    Medium,
    High,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Uncertainty {
    pub unknown_budget: Flag,
    pub unknown_connects_cost: Flag,
    pub unknown_client_history: Flag,
    pub missing_full_description: NeedsReview,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Flag {
    Flag,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NeedsReview {
    NeedsReview,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Personalization {
    #[serde(default)]
    pub runtime: Option<crate::learning::LearningSettings>,
    pub propose_weight_adjustments: bool,
    pub require_approval_before_activation: bool,
    pub min_feedback_samples_for_suggestion: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NotificationsConfig {
    #[serde(default)]
    pub runtime: Option<BotSettings>,
    pub schema_version: u32,
    pub notifications: NotificationSettings,
    pub providers: BTreeMap<String, Provider>,
    pub destinations: BTreeMap<String, Destination>,
    pub routes: Routes,
    pub interactions: Interactions,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NotificationSettings {
    pub enabled: bool,
    pub timezone: String,
    pub quiet_hours: QuietHours,
    pub max_alerts_per_day: u32,
    pub retry: Retry,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuietHours {
    pub start: String,
    pub end: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Retry {
    pub max_attempts: u32,
    pub base_delay_seconds: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub enum Provider {
    Discord {
        enabled: bool,
        token_env: String,
        application_id_env: String,
        public_key_env: String,
        allowed_user_ids_env: String,
        allowed_guild_ids_env: String,
    },
    Telegram {
        enabled: bool,
        token_env: String,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Destination {
    pub provider: String,
    pub channel_id_env: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Routes {
    pub job_matched: JobRoute,
    pub daily_digest: DigestRoute,
    pub scan_failed: FailureRoute,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobRoute {
    pub destinations: Vec<String>,
    pub minimum_score: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DigestRoute {
    pub destinations: Vec<String>,
    pub at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailureRoute {
    pub destinations: Vec<String>,
    pub severity: Severity,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Critical,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Interactions {
    pub discord_enabled: bool,
    pub commands: Vec<String>,
    pub feedback_actions: Vec<String>,
    pub never_allow_auto_submit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub radar: RadarConfig,
    pub scoring: ScoringConfig,
    pub notifications: NotificationsConfig,
    pub revision: String,
    pub provenance: BTreeMap<String, String>,
}

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn parse<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    // Do not echo source text: deployment YAML might accidentally contain secrets.
    serde_yaml::from_str(&text).map_err(|e| {
        anyhow::anyhow!(
            "invalid schema in {} at {:?}: check unknown/missing keys and typed values",
            path.display(),
            e.location(),
        )
    })
}
fn time(value: &str) -> Result<()> {
    ensure!(
        value.len() == 5 && NaiveTime::parse_from_str(value, "%H:%M").is_ok(),
        "invalid HH:MM time"
    );
    Ok(())
}
fn env_ref(value: &str) -> Result<()> {
    let name = value
        .strip_prefix("${")
        .and_then(|v| v.strip_suffix('}'))
        .unwrap_or(value);
    ensure!(
        !name.is_empty()
            && name
                .chars()
                .enumerate()
                .all(|(i, c)| c == '_' || c.is_ascii_uppercase() || (i > 0 && c.is_ascii_digit())),
        "invalid environment reference (expected ENV_NAME or ${{ENV_NAME}})"
    );
    Ok(())
}
impl Config {
    /// Each namespace is loaded once from the selected YAML directory. No duplicate compiled numeric defaults.
    pub fn load(directory: &Path) -> Result<Self> {
        let paths = ["radar", "scoring", "notifications"];
        let mut config = Self {
            radar: parse(&directory.join("radar.yaml"))?,
            scoring: parse(&directory.join("scoring.yaml"))?,
            notifications: parse(&directory.join("notifications.yaml"))?,
            revision: String::new(),
            provenance: paths
                .into_iter()
                .map(|s| {
                    (
                        s.into(),
                        format!("yaml:{}", directory.join(format!("{s}.yaml")).display()),
                    )
                })
                .collect(),
        };
        config.refresh_revision()?;
        Ok(config)
    }
    pub fn refresh_revision(&mut self) -> Result<()> {
        self.validate()?;
        self.revision = digest(&serde_json::to_vec(&(
            &self.radar,
            &self.scoring,
            &self.notifications,
        ))?);
        Ok(())
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.radar.schema_version == 1
                && self.scoring.schema_version == 1
                && self.notifications.schema_version == 1,
            "unsupported schema_version"
        );
        self.radar
            .app
            .timezone
            .parse::<chrono_tz::Tz>()
            .context("invalid app timezone")?;
        self.notifications
            .notifications
            .timezone
            .parse::<chrono_tz::Tz>()
            .context("invalid notification timezone")?;
        if let Some(scheduler) = &self.radar.scheduler {
            ensure!(
                scheduler.poll_interval_seconds > 0
                    && scheduler.poll_interval_seconds <= 60
                    && scheduler.max_jobs_per_run > 0
                    && scheduler.max_pending_runs > 0
                    && scheduler.max_pending_runs <= 1000
                    && scheduler.ui_refresh_seconds > 0
                    && scheduler.ui_refresh_seconds <= 60
                    && scheduler.recovery_after_seconds >= 10
                    && scheduler.recovery_after_seconds <= 3600
                    && scheduler.catch_up_minutes > 0
                    && scheduler.catch_up_minutes <= 1440
                    && scheduler.resume_mode != Mode::Paused,
                "invalid scheduler limits or resume mode"
            );
        }
        let weights = &self.scoring.weights;
        ensure!(
            weights.len() == Signal::ALL.len()
                && Signal::ALL.iter().all(|s| weights.contains_key(s))
                && weights.values().map(|v| u64::from(*v)).sum::<u64>() == 100,
            "all seven weights must be present and sum to 100"
        );
        ensure!(
            self.scoring.thresholds.shortlist_min_score
                <= self.scoring.thresholds.instant_alert_min_score
                && self.scoring.thresholds.instant_alert_min_score <= 100
                && self.notifications.routes.job_matched.minimum_score <= 100,
            "invalid score thresholds"
        );
        for mode in [
            &self.radar.modes.employed,
            &self.radar.modes.available,
            &self.radar.modes.busy,
        ] {
            ensure!(
                mode.weekly_freelance_hours > 0
                    && mode.weekly_freelance_hours <= 168
                    && !mode.schedule.weekdays.is_empty()
                    && !mode.schedule.at.is_empty(),
                "active mode needs capacity and a schedule"
            );
            ensure!(
                mode.max_source_requests_per_day > 0
                    && mode.max_jobs_processed_per_day > 0
                    && mode.max_deep_evaluations_per_day <= mode.max_jobs_processed_per_day,
                "invalid mode quotas"
            );
            for at in &mode.schedule.at {
                time(at)?;
            }
            for (i, day) in mode.schedule.weekdays.iter().enumerate() {
                ensure!(
                    !mode.schedule.weekdays[..i].contains(day),
                    "duplicate schedule weekday"
                );
            }
            for (i, at) in mode.schedule.at.iter().enumerate() {
                ensure!(
                    !mode.schedule.at[..i].contains(at),
                    "duplicate schedule time"
                );
            }
        }
        let paused = &self.radar.modes.paused;
        ensure!(
            paused.weekly_freelance_hours == 0
                && paused.schedule.weekdays.is_empty()
                && paused.schedule.at.is_empty()
                && paused.max_source_requests_per_day == 0
                && paused.max_jobs_processed_per_day == 0
                && paused.max_deep_evaluations_per_day == 0,
            "paused mode must have no schedule or quotas"
        );
        ensure!(
            self.radar.job_sources.upwork_mcp.require_capability_check,
            "Upwork capability checks cannot be bypassed"
        );
        if let Some(u) = &self.radar.job_sources.upwork_mcp.runtime {
            ensure!(
                (1..=60).contains(&u.request_timeout_seconds)
                    && (30..=900).contains(&u.login_timeout_seconds)
                    && u.callback_port > 0
                    && u.max_requests_per_minute > 0
                    && u.max_requests_per_minute <= 10
                    && u.max_requests_per_hour > 0
                    && u.max_requests_per_hour <= 100
                    && u.max_requests_per_day > 0
                    && u.max_requests_per_day <= 500
                    && (1..=10).contains(&u.max_pages_per_scan)
                    && (1..=100).contains(&u.max_results_per_scan)
                    && (1..=3600).contains(&u.retry_backoff_seconds),
                "invalid conservative Upwork request limits"
            );
            ensure!(
                u.profiles.contains_key(&u.default_profile),
                "Upwork default profile missing"
            );
            for (name, p) in &u.profiles {
                ensure!(
                    !name.is_empty()
                        && name.len() <= 64
                        && name
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
                    "invalid Upwork profile name"
                );
                ensure!(
                    p.result_limit > 0
                        && p.result_limit <= u.max_results_per_scan
                        && p.job_type
                            .as_deref()
                            .is_none_or(|v| matches!(v, "hourly" | "fixed"))
                        && p.minimum_budget_usd
                            .is_none_or(|b| b.is_finite() && b > 0.0)
                        && p.keywords
                            .iter()
                            .chain(&p.skills)
                            .chain(&p.experience_levels)
                            .all(|s| !s.trim().is_empty() && s.len() <= 200),
                    "invalid Upwork search profile"
                );
            }
        }
        ensure!(
            !self.radar.job_sources.upwork_mcp.enabled
                || self.radar.job_sources.upwork_mcp.runtime.is_some(),
            "Upwork runtime settings required when enabled"
        );
        ensure!(
            matches!(self.radar.ai.provider.as_str(), "mock" | "openai_responses"),
            "supported AI providers: mock, openai_responses"
        );
        if let Some(settings) = &self.scoring.personalization.runtime {
            settings.validate()?;
        }
        if let Some(templates) = &self.radar.proposals.templates {
            templates.validate()?;
        }
        let ai = &self.radar.ai;
        ensure!(
            (1..=600).contains(&ai.timeout_seconds)
                && (1..=8).contains(&ai.max_concurrent_tasks)
                && ai.max_estimated_cost_usd_per_day.is_finite()
                && ai.max_estimated_cost_usd_per_day >= 0.0,
            "invalid AI limits"
        );
        if let Some(runtime) = &ai.runtime {
            env_ref(&runtime.api_key_env)?;
            ensure!(
                !runtime.evaluation_rubric.trim().is_empty()
                    && runtime.evaluation_rubric.len() <= 8000
                    && (1024..=262144).contains(&runtime.max_input_bytes)
                    && (1024..=131072).contains(&runtime.max_output_bytes)
                    && (256..=8192).contains(&runtime.max_output_tokens)
                    && (80..=1000).contains(&runtime.max_proposal_words)
                    && runtime.max_evaluations_per_day > 0
                    && runtime.max_proposals_per_day > 0
                    && runtime.reserved_cost_usd_per_call.is_finite()
                    && runtime.reserved_cost_usd_per_call >= 0.0
                    && (ai.provider == "mock"
                        || !runtime.enabled
                        || (!runtime.model.trim().is_empty()
                            && runtime.reserved_cost_usd_per_call > 0.0))
                    && [
                        runtime.input_usd_per_million_tokens,
                        runtime.output_usd_per_million_tokens
                    ]
                    .iter()
                    .all(|v| v.is_none_or(|n| n.is_finite() && n >= 0.0)),
                "invalid AI runtime settings"
            );
        }
        let proposals = &self.radar.proposals;
        ensure!(
            !proposals.allow_marketplace_writes
                && proposals.require_human_approval
                && !proposals.allow_auto_draft,
            "Marketplace writes and automatic drafts are disabled; human approval required"
        );
        ensure!(
            self.scoring
                .personalization
                .require_approval_before_activation
                && self
                    .scoring
                    .personalization
                    .min_feedback_samples_for_suggestion
                    > 0,
            "personalization changes require approval and evidence"
        );
        ensure!(
            self.scoring.hard_vetoes.prohibited_or_unethical
                && self.scoring.hard_vetoes.already_applied,
            "policy and already-applied vetoes cannot be disabled"
        );
        env_ref(&self.radar.storage.database_url_env)?;
        let n = &self.notifications;
        time(&n.notifications.quiet_hours.start)?;
        time(&n.notifications.quiet_hours.end)?;
        time(&n.routes.daily_digest.at)?;
        ensure!(
            n.notifications.retry.max_attempts > 0 && n.notifications.retry.base_delay_seconds > 0,
            "invalid notification retry limits"
        );
        ensure!(
            n.interactions.never_allow_auto_submit,
            "automatic submission must remain disabled"
        );
        if let Some(runtime) = &n.runtime {
            ensure!(
                runtime.request_timeout_seconds > 0
                    && runtime.request_timeout_seconds <= 30
                    && runtime.delivery_lease_seconds > runtime.request_timeout_seconds * 2
                    && runtime.max_backoff_seconds > 0
                    && runtime.max_backoff_seconds <= 86400
                    && runtime.max_digest_jobs > 0
                    && runtime.max_digest_jobs <= 10
                    && runtime.signature_max_age_seconds > 0
                    && runtime.signature_max_age_seconds <= 300
                    && runtime.signature_future_skew_seconds >= 0
                    && runtime.signature_future_skew_seconds <= 30
                    && runtime.acknowledgment_timeout_ms > 0
                    && runtime.acknowledgment_timeout_ms <= 2000
                    && runtime.interaction_ttl_seconds > 0
                    && runtime.interaction_ttl_seconds <= 900,
                "invalid bot runtime limits"
            );
            env_ref(&runtime.interaction_encryption_key_env)?;
        }
        ensure!(
            n.interactions
                .commands
                .iter()
                .all(|c| ["status", "jobs", "scan", "pause", "resume", "digest"]
                    .contains(&c.as_str()))
                && n.interactions
                    .feedback_actions
                    .iter()
                    .all(|a| ["interested", "save", "skip"].contains(&a.as_str())),
            "unsupported bot command or feedback action"
        );
        for p in n.providers.values() {
            match p {
                Provider::Discord {
                    token_env,
                    application_id_env,
                    public_key_env,
                    allowed_user_ids_env,
                    allowed_guild_ids_env,
                    ..
                } => {
                    for value in [
                        token_env,
                        application_id_env,
                        public_key_env,
                        allowed_user_ids_env,
                        allowed_guild_ids_env,
                    ] {
                        env_ref(value)?;
                    }
                }
                Provider::Telegram { token_env, .. } => env_ref(token_env)?,
            }
        }
        for destination in n.destinations.values() {
            ensure!(
                n.providers.contains_key(&destination.provider),
                "destination references unknown provider"
            );
            env_ref(&destination.channel_id_env)?;
        }
        for destinations in [
            &n.routes.job_matched.destinations,
            &n.routes.daily_digest.destinations,
            &n.routes.scan_failed.destinations,
        ] {
            for destination in destinations {
                ensure!(
                    n.destinations.contains_key(destination),
                    "route references unknown destination"
                );
            }
        }
        Ok(())
    }
    pub fn policy(&self) -> ScoringPolicy {
        ScoringPolicy {
            weights: self.scoring.weights.clone(),
            shortlist_min_score: self.scoring.thresholds.shortlist_min_score,
            vetoes: self.scoring.hard_vetoes.clone(),
        }
    }
    pub fn work_mode(&self) -> &WorkMode {
        match self.radar.app.active_mode {
            Mode::Employed => &self.radar.modes.employed,
            Mode::Available => &self.radar.modes.available,
            Mode::Busy => &self.radar.modes.busy,
            Mode::Paused => &self.radar.modes.paused,
        }
    }
    pub fn settings(&self) -> ScanSettings {
        let mode = self.work_mode();
        let zone: chrono_tz::Tz = self.radar.app.timezone.parse().expect("validated timezone");
        ScanSettings {
            mode: self.radar.app.active_mode,
            weekly_hours: mode.weekly_freelance_hours,
            max_requests: mode.max_source_requests_per_day,
            max_jobs: mode.max_jobs_processed_per_day,
            timezone: zone,
            config_revision: self.revision.clone(),
        }
    }
}
