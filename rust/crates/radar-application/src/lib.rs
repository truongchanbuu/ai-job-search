pub mod ai;
pub mod bots;
pub mod learning;
pub use bots::{InteractionProvider, NotificationProvider};
pub mod api;
pub mod config;
pub mod scheduling;

use anyhow::{Result, bail};
use chrono::{DateTime, NaiveDate, Utc};
use radar_domain::{Evaluation, Job, Mode, Profile, ScoringPolicy, Verdict, evaluate};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub trait Clock {
    fn now(&self) -> DateTime<Utc>;
}

#[derive(Debug, Clone)]
pub struct Capability {
    pub available: bool,
    pub diagnostic: String,
}

pub trait JobSource {
    fn name(&self) -> &str;
    fn capability(&self) -> Capability;
    /// Bounded import; transport errors can include retry_after, never retried here.
    fn fetch(&mut self, limit: usize) -> Result<Vec<Job>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiDraft {
    pub text: String,
    pub evidence: Vec<String>,
    pub estimated_cost_usd: f64,
}
#[async_trait::async_trait]
pub trait AiProvider: Send + Sync {
    async fn infer(
        &self,
        settings: &config::AiRuntime,
        input: &ai::AiInput,
    ) -> Result<ai::ProviderReply>;
    fn draft(&self, job: &Job, profile: &Profile) -> Result<AiDraft>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub event_id: String,
    pub job_id: String,
    pub score: f64,
}

#[derive(Debug, Clone)]
pub enum Command {
    Scan,
    Pause,
    Resume,
    Save(String),
    Skip(String),
}

pub trait Scheduler {
    fn next_run(&self, after: DateTime<Utc>) -> Result<Option<DateTime<Utc>>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ingestion {
    pub fetched_at: DateTime<Utc>,
    pub job: Job,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub id: String,
    pub ingestions: Vec<Ingestion>,
    pub evaluation: Evaluation,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DailyUsage {
    pub requests: u32,
    pub jobs: u32,
}

pub trait Repository {
    fn usage(&self, date: NaiveDate) -> DailyUsage;
    fn record_request(&mut self, date: NaiveDate);
    fn upsert(
        &mut self,
        date: NaiveDate,
        job: Job,
        at: DateTime<Utc>,
        evaluation: Evaluation,
    ) -> Result<()>;
    fn ranked(&self) -> Vec<Record>;
}

#[derive(Debug, Clone)]
pub struct ScanSettings {
    pub mode: Mode,
    pub weekly_hours: u32,
    pub max_requests: u32,
    pub max_jobs: u32,
    pub timezone: chrono_tz::Tz,
    pub config_revision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub status: String,
    pub processed: usize,
    pub records: Vec<Record>,
}

pub fn scan(
    source: &mut dyn JobSource,
    repository: &mut dyn Repository,
    clock: &dyn Clock,
    settings: &ScanSettings,
    profile: &Profile,
    policy: &ScoringPolicy,
) -> Result<ScanReport> {
    let report = |status: &str, processed, repo: &dyn Repository| ScanReport {
        status: status.into(),
        processed,
        records: repo.ranked(),
    };
    if settings.mode == Mode::Paused {
        return Ok(report("paused: no discovery", 0, repository));
    }
    let capability = source.capability();
    if !capability.available {
        bail!(
            "source {} disabled: {}",
            source.name(),
            capability.diagnostic
        );
    }
    let now = clock.now();
    let date = now.with_timezone(&settings.timezone).date_naive();
    let usage = repository.usage(date);
    if usage.requests >= settings.max_requests || usage.jobs >= settings.max_jobs {
        return Ok(report("daily quota reached", 0, repository));
    }
    // Count failed requests too, preventing unlimited retries against a broken source.
    repository.record_request(date);
    let remaining = (settings.max_jobs - usage.jobs) as usize;
    let jobs = source.fetch(remaining)?;
    if jobs.len() > remaining {
        bail!("source exceeded requested import limit");
    }
    // Validate the entire batch before changing job state.
    for job in &jobs {
        job.validate().map_err(anyhow::Error::msg)?;
        if job.provider != source.name() {
            bail!("job provider does not match source provenance");
        }
    }
    let processed = jobs.len();
    for job in jobs {
        let evaluation = evaluate(
            &job,
            profile,
            policy,
            settings.mode,
            settings.weekly_hours,
            now,
            &settings.config_revision,
        );
        repository.upsert(date, job, now, evaluation)?;
    }
    Ok(report("completed", processed, repository))
}

/// Process-local repository: intentionally no restart durability or cloud-state claims.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MemoryRepository {
    records: BTreeMap<String, Record>,
    usage: BTreeMap<NaiveDate, DailyUsage>,
}
impl MemoryRepository {
    /// Stage the existing scan use case inside an external storage transaction.
    pub fn with_daily_usage(date: NaiveDate, usage: DailyUsage) -> Self {
        Self {
            records: BTreeMap::new(),
            usage: BTreeMap::from([(date, usage)]),
        }
    }
}
impl Repository for MemoryRepository {
    fn usage(&self, date: NaiveDate) -> DailyUsage {
        self.usage.get(&date).cloned().unwrap_or_default()
    }
    fn record_request(&mut self, date: NaiveDate) {
        self.usage.entry(date).or_default().requests += 1;
    }
    fn upsert(
        &mut self,
        date: NaiveDate,
        job: Job,
        at: DateTime<Utc>,
        evaluation: Evaluation,
    ) -> Result<()> {
        let id = job.identity().map_err(anyhow::Error::msg)?;
        let record = self.records.entry(id.clone()).or_insert_with(|| Record {
            id,
            ingestions: vec![],
            evaluation: evaluation.clone(),
        });
        record.ingestions.push(Ingestion {
            fetched_at: at,
            job,
        });
        record.evaluation = evaluation;
        self.usage.entry(date).or_default().jobs += 1;
        Ok(())
    }
    fn ranked(&self) -> Vec<Record> {
        let mut records: Vec<_> = self.records.values().cloned().collect();
        records.sort_by(|a, b| {
            let order = |v: &Verdict| match v {
                Verdict::ApplyCandidate => 0,
                Verdict::Review => 1,
                Verdict::Skip => 2,
            };
            order(&a.evaluation.verdict)
                .cmp(&order(&b.evaluation.verdict))
                .then_with(|| {
                    b.evaluation
                        .score
                        .unwrap_or(-1.0)
                        .total_cmp(&a.evaluation.score.unwrap_or(-1.0))
                })
                .then_with(|| a.id.cmp(&b.id))
        });
        records
    }
}

pub mod profiles;

pub mod proposal_templates;
