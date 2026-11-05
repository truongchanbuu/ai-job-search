use crate::config::{ConfidenceLevel, Config};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Timelike, Utc};
use radar_domain::{Confidence, FeedbackAction};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    JobMatched,
    DailyDigest,
    ScanFailed,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessage {
    pub title: String,
    pub body: String,
    pub job_id: Option<Uuid>,
    pub job_revision: Option<i64>,
    pub score: Option<f64>,
}
// Interaction tokens intentionally have neither Debug nor Serialize implementations here.
pub enum DeliveryTarget {
    Channel(String),
    Interaction {
        application_id: String,
        token: String,
    },
}
#[derive(Debug, Clone)]
pub struct DeliveryReceipt {
    pub remote_id: String,
    pub cooldown_seconds: Option<f64>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureKind {
    Retryable,
    Permanent,
    Uncertain,
}
#[derive(Debug, Clone)]
pub struct DeliveryFailure {
    pub kind: FailureKind,
    pub code: String,
    pub retry_after_seconds: Option<f64>,
    pub global: bool,
}
impl DeliveryFailure {
    pub fn new(kind: FailureKind, code: &str) -> Self {
        Self {
            kind,
            code: code.into(),
            retry_after_seconds: None,
            global: false,
        }
    }
}
#[async_trait]
pub trait NotificationProvider: Send + Sync {
    async fn deliver(
        &self,
        target: &DeliveryTarget,
        message: &NotificationMessage,
        id: Uuid,
    ) -> Result<DeliveryReceipt, DeliveryFailure>;
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum BotAction {
    Status,
    Jobs,
    Scan,
    Pause,
    Resume,
    Digest,
    Feedback {
        delivery_id: Uuid,
        action: FeedbackAction,
        reason: Option<String>,
    },
    FeedbackModal {
        delivery_id: Uuid,
        action: FeedbackAction,
    },
}
pub struct VerifiedInteraction {
    pub id: String,
    pub application_id: String,
    pub user_id: String,
    pub guild_id: String,
    pub message_id: Option<String>,
    pub channel_id: String,
    pub token: String,
    pub action: Option<BotAction>,
    pub ping: bool,
}
pub trait InteractionProvider: Send + Sync {
    fn authorize(&self, user: &str, guild: &str, channel: &str) -> bool;
    fn verify(
        &self,
        body: &[u8],
        signature: &str,
        timestamp: &str,
        now: DateTime<Utc>,
    ) -> Result<VerifiedInteraction, String>;
}

pub struct NotificationRouter;
impl NotificationRouter {
    pub fn destinations(config: &Config, kind: EventKind, score: Option<f64>) -> Vec<String> {
        let routes = &config.notifications.routes;
        match kind {
            EventKind::JobMatched
                if score.is_some_and(|s| {
                    s >= f64::from(
                        routes
                            .job_matched
                            .minimum_score
                            .max(config.scoring.thresholds.instant_alert_min_score),
                    )
                }) =>
            {
                routes.job_matched.destinations.clone()
            }
            EventKind::DailyDigest => routes.daily_digest.destinations.clone(),
            EventKind::ScanFailed => routes.scan_failed.destinations.clone(),
            _ => vec![],
        }
    }
    pub fn confident(config: &Config, confidence: &Confidence) -> bool {
        let value = match confidence {
            Confidence::Low => 0,
            Confidence::Medium => 1,
            Confidence::High => 2,
        };
        value
            >= match config.scoring.thresholds.minimum_confidence_to_alert {
                ConfidenceLevel::Low => 0,
                ConfidenceLevel::Medium => 1,
                ConfidenceLevel::High => 2,
            }
    }
}
pub fn in_quiet_hours(config: &Config, now: DateTime<Utc>) -> bool {
    let zone: chrono_tz::Tz = config
        .notifications
        .notifications
        .timezone
        .parse()
        .expect("validated timezone");
    let quiet = &config.notifications.notifications.quiet_hours;
    let start = chrono::NaiveTime::parse_from_str(&quiet.start, "%H:%M").expect("validated time");
    let end = chrono::NaiveTime::parse_from_str(&quiet.end, "%H:%M").expect("validated time");
    let time = now.with_timezone(&zone).time();
    if start == end {
        false
    } else if start < end {
        time >= start && time < end
    } else {
        time >= start || time < end
    }
}
pub fn after_quiet_hours(config: &Config, now: DateTime<Utc>) -> DateTime<Utc> {
    if !in_quiet_hours(config, now) {
        return now;
    }
    let mut next = now.with_second(0).unwrap().with_nanosecond(0).unwrap() + Duration::minutes(1);
    // A UTC walk handles both DST gaps and repeated local hours without guessing an offset.
    for _ in 0..=1560 {
        if !in_quiet_hours(config, next) {
            return next;
        }
        next += Duration::minutes(1);
    }
    next
}
pub fn retry_delay(config: &Config, attempts: i32, retry_after: Option<f64>) -> Duration {
    let retry = &config.notifications.notifications.retry;
    let max = config
        .notifications
        .runtime
        .as_ref()
        .map_or(u64::MAX, |s| s.max_backoff_seconds);
    let exponential = u64::from(retry.base_delay_seconds)
        .saturating_mul(1u64 << ((attempts - 1).clamp(0, 30) as u32))
        .min(max);
    let seconds = (exponential as f64).max(
        retry_after
            .filter(|s| s.is_finite() && *s >= 0.0)
            .unwrap_or(0.0),
    );
    Duration::milliseconds((seconds * 1000.0).ceil().min(i64::MAX as f64) as i64)
}

pub fn retry_at(
    config: &Config,
    now: DateTime<Utc>,
    attempts: i32,
    retry_after: Option<f64>,
) -> DateTime<Utc> {
    // A corrupt/hostile retry-after must block delivery rather than overflow and crash the worker.
    now.checked_add_signed(retry_delay(config, attempts, retry_after))
        .unwrap_or(DateTime::<Utc>::MAX_UTC)
}

/// Missing optional budget/client facts affect confidence; missing essential card facts block delivery.
pub fn notification_ready(job: &radar_domain::Job) -> bool {
    job.validate().is_ok()
        && job
            .description
            .as_ref()
            .is_some_and(|s| !s.trim().is_empty())
        && !matches!(job.job_type, radar_domain::JobType::Unknown)
}
