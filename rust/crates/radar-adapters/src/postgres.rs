//! Transactional storage adapter. The Phase 1 scan/scorer remains the evaluation use case.
mod ai;
mod interactions;
mod learning;
mod outbox;
mod profiles;
mod scheduler;
mod upwork;
use crate::{FixedClock, ManualJobSource, MockJobSource};
use chrono::{DateTime, Utc};
use radar_application::{
    DailyUsage, JobSource, MemoryRepository, Repository,
    api::*,
    config::{Config, digest},
    scan,
};
use radar_domain::{ApplicationStatus, FeedbackAction, Job, Profile};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use sqlx::{PgPool, Postgres, Row, Transaction, postgres::PgRow, types::Json};
use uuid::Uuid;

#[derive(Default)]
struct ImportOptions {
    expected_revision: Option<i64>,
    per_run_limit: Option<u32>,
    upwork: Option<crate::upwork::UpworkJobSource>,
}

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../migrations");

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("{0}")]
    Upwork(#[from] crate::upwork::UpworkError),
    #[error("{0}")]
    Invalid(String),
    #[error("stale revision or conflicting idempotency key; refresh before retrying")]
    Conflict,
    #[error("record not found")]
    NotFound,
    #[error("database operation failed")]
    Database(sqlx::Error),
    #[error("stored data could not be decoded")]
    Data(#[from] serde_json::Error),
}
impl From<sqlx::Error> for StoreError {
    fn from(error: sqlx::Error) -> Self {
        if error
            .as_database_error()
            .is_some_and(|e| e.code().as_deref() == Some("23505"))
        {
            Self::Conflict
        } else {
            Self::Database(error)
        }
    }
}
pub type Result<T> = std::result::Result<T, StoreError>;

#[derive(Clone)]
pub struct PgRepository {
    pub pool: PgPool,
}

fn name(value: impl Serialize) -> Result<String> {
    Ok(serde_json::from_value(serde_json::to_value(value)?)?)
}
fn enum_value<T: DeserializeOwned>(value: String) -> Result<T> {
    Ok(serde_json::from_value(Value::String(value))?)
}
fn fingerprint(value: &impl Serialize) -> Result<String> {
    Ok(digest(&serde_json::to_vec(value)?))
}
fn identity(job: &Job) -> Result<String> {
    Ok(digest(
        job.identity().map_err(StoreError::Invalid)?.as_bytes(),
    ))
}
fn config_view(row: &PgRow) -> Result<ConfigView> {
    Ok(ConfigView {
        revision: row.try_get("revision")?,
        effective: row.try_get::<Json<Config>, _>("effective")?.0,
        updated_at: row.try_get("updated_at")?,
        actor: row.try_get("actor")?,
    })
}
fn job_view(row: &PgRow) -> Result<JobView> {
    Ok(JobView {
        personalization: None,
        id: row.try_get("id")?,
        job: row.try_get::<Json<Job>, _>("payload")?.0,
        evaluation: row
            .try_get::<Json<radar_domain::Evaluation>, _>("snapshot")?
            .0,
        fetched_at: row.try_get("fetched_at")?,
        ingestions: row.try_get("ingestions")?,
        decision: row
            .try_get::<Option<String>, _>("decision")?
            .map(enum_value)
            .transpose()?,
        revision: row.try_get("revision")?,
        application_status: enum_value(row.try_get("status")?)?,
        submitted_at: row.try_get("submitted_at")?,
        submission_reported_at: row.try_get("submission_reported_at")?,
        connects_spent: row.try_get("connects_spent")?,
    })
}
fn feedback_view(row: &PgRow) -> Result<FeedbackView> {
    Ok(FeedbackView {
        reason_code: row
            .try_get::<Option<String>, _>("reason_code")?
            .map(enum_value)
            .transpose()?,
        learning_excluded: row.try_get("learning_excluded")?,
        learning_version: row.try_get("learning_version")?,
        actual_effort_hours: row.try_get("actual_effort_hours")?,
        id: row.try_get("id")?,
        job_id: row.try_get("job_id")?,
        job_title: row.try_get("title")?,
        action: enum_value(row.try_get("action")?)?,
        reason: row.try_get("reason")?,
        created_at: row.try_get("created_at")?,
        actor: row.try_get("actor")?,
        revision: row.try_get("revision")?,
        application_status: enum_value(row.try_get("application_status")?)?,
    })
}
fn page(page: Option<u32>, size: Option<u32>) -> Result<(u32, u32, i64)> {
    let page = page.unwrap_or(1);
    let size = size.unwrap_or(25);
    if page == 0 || size == 0 || size > 100 {
        return Err(StoreError::Invalid(
            "page must be positive; page_size must be 1–100".into(),
        ));
    }
    Ok((page, size, i64::from(page - 1) * i64::from(size)))
}
const JOB_FROM: &str = "FROM jobs j JOIN applications a ON a.job_id=j.id
 JOIN LATERAL (SELECT snapshot FROM evaluations WHERE job_id=j.id ORDER BY id DESC LIMIT 1) e ON true";
const JOB_FIELDS: &str = "j.id,j.payload,j.fetched_at,j.decision,j.revision,a.status,a.submitted_at,a.submission_reported_at,a.connects_spent,e.snapshot,
 (SELECT count(*) FROM job_ingestions WHERE job_id=j.id) AS ingestions";

async fn audit(
    tx: &mut Transaction<'_, Postgres>,
    actor: &str,
    action: &str,
    entity: &str,
    details: Value,
) -> Result<()> {
    sqlx::query("INSERT INTO audit_log(actor,action,entity,details) VALUES ($1,$2,$3,$4)")
        .bind(actor)
        .bind(action)
        .bind(entity)
        .bind(Json(details))
        .execute(&mut **tx)
        .await?;
    Ok(())
}
fn sources(config: &Config) -> Vec<SourceView> {
    vec![
        SourceView {
            name: "mock".into(),
            enabled: config.radar.job_sources.mock.enabled,
            diagnostic: "synthetic jobs only".into(),
        },
        SourceView {
            name: "manual".into(),
            enabled: config.radar.job_sources.manual.enabled,
            diagnostic: "explicit authorized JSON import".into(),
        },
        SourceView {
            name: "upwork".into(),
            enabled: config.radar.job_sources.upwork_mcp.enabled,
            diagnostic: "manual mode only; see /v1/upwork for authentication and search status"
                .into(),
        },
    ]
}
async fn store_sources(tx: &mut Transaction<'_, Postgres>, config: &Config) -> Result<()> {
    for source in sources(config) {
        sqlx::query("INSERT INTO job_sources(name,enabled,diagnostic) VALUES ($1,$2,$3) ON CONFLICT(name) DO UPDATE SET enabled=excluded.enabled,diagnostic=excluded.diagnostic")
            .bind(source.name).bind(source.enabled).bind(source.diagnostic).execute(&mut **tx).await?;
    }
    Ok(())
}
fn merge_config(base: &Config, overrides: &Value, revision: i64) -> Result<Config> {
    let mut effective = base.clone();
    for key in ["radar", "scoring", "notifications"] {
        if let Some(value) = overrides.get(key) {
            match key {
                "radar" => effective.radar = serde_json::from_value(value.clone())?,
                "scoring" => effective.scoring = serde_json::from_value(value.clone())?,
                _ => effective.notifications = serde_json::from_value(value.clone())?,
            }
            effective
                .provenance
                .insert(key.into(), format!("db-approved:revision:{revision}"));
        }
    }
    if effective.radar.job_sources.upwork_mcp.runtime.is_none() {
        effective.radar.job_sources.upwork_mcp.runtime =
            base.radar.job_sources.upwork_mcp.runtime.clone();
    }
    if effective.radar.proposals.templates.is_none() {
        effective.radar.proposals.templates = base.radar.proposals.templates.clone();
    }
    if effective.scoring.personalization.runtime.is_none() {
        effective.scoring.personalization.runtime = base.scoring.personalization.runtime.clone();
    }
    if effective.radar.ai.runtime.is_none() {
        effective.radar.ai.runtime = base.radar.ai.runtime.clone();
    }
    effective
        .refresh_revision()
        .map_err(|e| StoreError::Invalid(e.to_string()))?;
    Ok(effective)
}

impl PgRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    /// Seed YAML once; reconcile changed deployment defaults without discarding approved namespaces.
    pub async fn initialize(&self, base: &Config) -> Result<()> {
        base.validate()
            .map_err(|e| StoreError::Invalid(e.to_string()))?;
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT pg_advisory_xact_lock(5732026)")
            .execute(&mut *tx)
            .await?;
        let current = sqlx::query("SELECT * FROM runtime_config WHERE singleton=true FOR UPDATE")
            .fetch_optional(&mut *tx)
            .await?;
        match current {
            None => {
                sqlx::query("INSERT INTO config_revisions(revision,snapshot,actor) VALUES (1,$1,'deployment')").bind(Json(base)).execute(&mut *tx).await?;
                sqlx::query("INSERT INTO runtime_config(singleton,revision,effective,baseline_hash,actor) VALUES (true,1,$1,$2,'deployment')")
                    .bind(Json(base)).bind(&base.revision).execute(&mut *tx).await?;
                store_sources(&mut tx, base).await?;
                audit(
                    &mut tx,
                    "deployment",
                    "config_initialized",
                    "config",
                    json!({"revision":1}),
                )
                .await?;
            }
            Some(row) if row.try_get::<String, _>("baseline_hash")? != base.revision => {
                let revision = row.try_get::<i64, _>("revision")? + 1;
                let overrides: Json<Value> = row.try_get("overrides")?;
                let effective = merge_config(base, &overrides.0, revision)?;
                sqlx::query("INSERT INTO config_revisions(revision,snapshot,actor) VALUES ($1,$2,'deployment')").bind(revision).bind(Json(&effective)).execute(&mut *tx).await?;
                sqlx::query("UPDATE runtime_config SET revision=$1,effective=$2,baseline_hash=$3,actor='deployment',updated_at=now() WHERE singleton=true")
                    .bind(revision).bind(Json(&effective)).bind(&base.revision).execute(&mut *tx).await?;
                store_sources(&mut tx, &effective).await?;
                audit(
                    &mut tx,
                    "deployment",
                    "config_reconciled",
                    "config",
                    json!({"revision":revision}),
                )
                .await?;
            }
            Some(_) => {}
        }
        tx.commit().await?;
        Ok(())
    }
    pub async fn config(&self) -> Result<ConfigView> {
        config_view(
            &sqlx::query("SELECT * FROM runtime_config WHERE singleton=true")
                .fetch_one(&self.pool)
                .await?,
        )
    }
    pub async fn update_config(&self, update: ConfigUpdate, actor: &str) -> Result<ConfigView> {
        let mut tx = self.pool.begin().await?;
        let result = self.update_config_in(&mut tx, update, actor).await?;
        tx.commit().await?;
        Ok(result)
    }
    async fn update_config_in(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        update: ConfigUpdate,
        actor: &str,
    ) -> Result<ConfigView> {
        if update.radar.is_none() && update.scoring.is_none() && update.notifications.is_none() {
            return Err(StoreError::Invalid(
                "at least one config namespace is required".into(),
            ));
        }
        let row = sqlx::query("SELECT * FROM runtime_config WHERE singleton=true FOR UPDATE")
            .fetch_one(&mut **tx)
            .await?;
        let current = config_view(&row)?;
        if update.expected_revision != current.revision {
            return Err(StoreError::Conflict);
        }
        let mut overrides: Value = row.try_get::<Json<Value>, _>("overrides")?.0;
        if let Some(value) = update.radar {
            overrides["radar"] = serde_json::to_value(value)?;
        }
        if let Some(value) = update.scoring {
            overrides["scoring"] = serde_json::to_value(value)?;
        }
        if let Some(value) = update.notifications {
            overrides["notifications"] = serde_json::to_value(value)?;
        }
        let revision = current.revision + 1;
        let effective = merge_config(&current.effective, &overrides, revision)?;
        if current.effective.radar.app.active_mode != radar_domain::Mode::Paused
            && effective.radar.app.active_mode == radar_domain::Mode::Paused
        {
            sqlx::query("UPDATE scheduler_state SET previous_mode=$1 WHERE singleton=true")
                .bind(name(current.effective.radar.app.active_mode)?)
                .execute(&mut **tx)
                .await?;
        }

        sqlx::query("INSERT INTO config_revisions(revision,snapshot,actor) VALUES ($1,$2,$3)")
            .bind(revision)
            .bind(Json(&effective))
            .bind(actor)
            .execute(&mut **tx)
            .await?;
        let row = sqlx::query("UPDATE runtime_config SET revision=$1,effective=$2,overrides=$3,actor=$4,updated_at=now() WHERE singleton=true RETURNING *")
            .bind(revision).bind(Json(&effective)).bind(Json(overrides)).bind(actor).fetch_one(&mut **tx).await?;
        store_sources(tx, &effective).await?;
        // Policy edits may release deferred work, but never bypass an actual provider cooldown/backoff.
        sqlx::query("UPDATE notification_deliveries SET next_attempt_at=least(next_attempt_at,now()) WHERE status IN ('pending','retry') AND error IN ('quiet hours','daily notification cap reached','notifications disabled','work mode paused','route or threshold inactive','provider not configured; inspect provider status','waiting for evaluation under current config')").execute(&mut **tx).await?;

        audit(
            tx,
            actor,
            "config_updated",
            "config",
            json!({"previous_revision":current.revision,"revision":revision}),
        )
        .await?;
        config_view(&row)
    }
    pub async fn config_history(&self) -> Result<Vec<ConfigView>> {
        let rows = sqlx::query("SELECT revision,snapshot AS effective,actor,created_at AS updated_at FROM config_revisions ORDER BY revision DESC LIMIT 100").fetch_all(&self.pool).await?;
        rows.iter().map(config_view).collect()
    }
    pub async fn sources(&self) -> Result<Vec<SourceView>> {
        Ok(
            sqlx::query("SELECT name,enabled,diagnostic FROM job_sources WHERE name<>'upwork_mcp' ORDER BY name")
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(|row| SourceView {
                    name: row.get("name"),
                    enabled: row.get("enabled"),
                    diagnostic: row.get("diagnostic"),
                })
                .collect(),
        )
    }
    pub async fn dashboard(&self) -> Result<Dashboard> {
        let row = sqlx::query("SELECT (SELECT count(*) FROM jobs) AS jobs,
            (SELECT count(*) FROM jobs j JOIN LATERAL (SELECT snapshot FROM evaluations WHERE job_id=j.id ORDER BY id DESC LIMIT 1) e ON true WHERE snapshot->>'verdict'='apply_candidate' AND (j.decision IS NULL OR j.decision NOT IN ('skip','not_available')) AND EXISTS(SELECT 1 FROM applications a WHERE a.job_id=j.id AND a.status IN ('none','drafted'))) AS candidates,
            (SELECT count(*) FROM jobs j JOIN LATERAL (SELECT snapshot FROM evaluations WHERE job_id=j.id ORDER BY id DESC LIMIT 1) e ON true WHERE snapshot->>'verdict'='review') AS reviews,
            (SELECT count(*) FROM feedback) AS feedback,
            (SELECT max(created_at) FROM imports) AS last_import").fetch_one(&self.pool).await?;
        let config = self.config().await?;
        Ok(Dashboard {
            jobs: row.try_get("jobs")?,
            candidates: row.try_get("candidates")?,
            reviews: row.try_get("reviews")?,
            feedback: row.try_get("feedback")?,
            last_import: row.try_get("last_import")?,
            mode: config.effective.radar.app.active_mode,
            config_revision: config.revision,
            sources: self.sources().await?,
        })
    }
    pub async fn jobs(&self, query: JobQuery) -> Result<JobPage> {
        let (page, size, offset) = page(query.page, query.page_size)?;
        let context = self.ranking_context().await?;
        let rerank = context.as_ref().is_some_and(|(_, model, suppressed)| {
            model.as_ref().is_some_and(|model| {
                model.samples >= model.minimum_samples
                    && model.weights.iter().any(|(feature, weight)| {
                        weight.points != 0.0 && !suppressed.contains(feature)
                    })
            })
        }) && query
            .sort
            .as_ref()
            .is_none_or(|sort| matches!(sort, JobSort::Score));
        let search = query.search.unwrap_or_default();
        if search.len() > 200 {
            return Err(StoreError::Invalid("search is limited to 200 bytes".into()));
        }
        let verdict = query.verdict.map(name).transpose()?;
        let decision = query.decision.map(name).transpose()?;
        let filter = "WHERE ($1='' OR strpos(lower(j.title || ' ' || coalesce(j.payload->>'description','')),lower($1))>0)
            AND ($2::text IS NULL OR e.snapshot->>'verdict'=$2) AND ($3::text IS NULL OR j.decision=$3)";
        let total: i64 = sqlx::query_scalar(&format!("SELECT count(*) {JOB_FROM} {filter}"))
            .bind(&search)
            .bind(&verdict)
            .bind(&decision)
            .fetch_one(&self.pool)
            .await?;
        let sort = match query.sort.unwrap_or_default() {
            JobSort::Score => {
                "CASE e.snapshot->>'verdict' WHEN 'apply_candidate' THEN 0 WHEN 'review' THEN 1 ELSE 2 END,(e.snapshot->>'score')::float8 DESC NULLS LAST,j.id"
            }
            JobSort::Newest => "j.fetched_at DESC,j.id",
            JobSort::Title => "j.title,j.id",
        };
        if rerank && total > context.as_ref().unwrap().0.max_ranked_jobs as i64 {
            return Err(StoreError::Invalid(
                "personalized ranking exceeds configured job bound; narrow the filter".into(),
            ));
        }
        let limit = if rerank { total } else { i64::from(size) };
        let query_offset = if rerank { 0 } else { offset };
        let rows = sqlx::query(&format!(
            "SELECT {JOB_FIELDS} {JOB_FROM} {filter} ORDER BY {sort} LIMIT $4 OFFSET $5"
        ))
        .bind(search)
        .bind(verdict)
        .bind(decision)
        .bind(limit)
        .bind(query_offset)
        .fetch_all(&self.pool)
        .await?;
        let mut items = rows.iter().map(job_view).collect::<Result<Vec<_>>>()?;
        if let Some(context) = &context {
            for job in &mut items {
                Self::apply_personalization(job, context);
            }
        }
        if rerank {
            items.sort_by(radar_application::learning::order_jobs);
            items = items
                .into_iter()
                .skip(offset as usize)
                .take(size as usize)
                .collect();
        }
        Ok(JobPage {
            items,
            total,
            page,
            page_size: size,
        })
    }
    pub async fn job(&self, id: Uuid) -> Result<JobView> {
        let row = sqlx::query(&format!("SELECT {JOB_FIELDS} {JOB_FROM} WHERE j.id=$1"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(StoreError::NotFound)?;
        let mut job = job_view(&row)?;
        if let Some(context) = self.ranking_context().await? {
            Self::apply_personalization(&mut job, &context);
        }
        Ok(job)
    }
    pub async fn evaluations(&self, id: Uuid) -> Result<Vec<EvaluationView>> {
        self.job(id).await?;
        let rows =
            sqlx::query("SELECT * FROM evaluations WHERE job_id=$1 ORDER BY id DESC LIMIT 100")
                .bind(id)
                .fetch_all(&self.pool)
                .await?;
        rows.iter()
            .map(|row| {
                Ok(EvaluationView {
                    id: row.try_get("id")?,
                    job_id: id,
                    created_at: row.try_get("created_at")?,
                    config_revision: row.try_get("config_revision")?,
                    profile_revision: row.try_get("profile_revision")?,
                    input: row.try_get::<Json<Job>, _>("input")?.0,
                    evaluation: row
                        .try_get::<Json<radar_domain::Evaluation>, _>("snapshot")?
                        .0,
                })
            })
            .collect()
    }
    pub async fn import(
        &self,
        request: ImportRequest,
        profile: &Profile,
        actor: &str,
        now: DateTime<Utc>,
    ) -> Result<ImportResult> {
        let mut tx = self.pool.begin().await?;
        let result = self
            .import_in(
                &mut tx,
                request,
                profile,
                actor,
                now,
                ImportOptions::default(),
            )
            .await?;
        tx.commit().await?;
        Ok(result)
    }
    async fn import_in(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        request: ImportRequest,
        profile: &Profile,
        actor: &str,
        now: DateTime<Utc>,
        options: ImportOptions,
    ) -> Result<ImportResult> {
        let hash = fingerprint(&request)?;
        // A config row lock gives each bounded import one consistent policy and durable quota budget.
        let config = config_view(
            &sqlx::query("SELECT * FROM runtime_config WHERE singleton=true FOR UPDATE")
                .fetch_one(&mut **tx)
                .await?,
        )?;
        if options
            .expected_revision
            .is_some_and(|revision| revision != config.revision)
        {
            return Err(StoreError::Conflict);
        }
        if let Some(row) = sqlx::query("SELECT request_hash,response FROM imports WHERE id=$1")
            .bind(request.idempotency_key)
            .fetch_optional(&mut **tx)
            .await?
        {
            if row.try_get::<String, _>("request_hash")? != hash {
                return Err(StoreError::Conflict);
            }
            return Ok(row.try_get::<Json<ImportResult>, _>("response")?.0);
        }
        let upwork_stats = options.upwork.as_ref().map(|s| s.stats.clone());
        let mut jobs = match request.source {
            ImportSource::Upwork => {
                options
                    .upwork
                    .ok_or_else(|| {
                        StoreError::Invalid("Upwork requires an authenticated manual scan".into())
                    })?
                    .jobs
            }
            ImportSource::Mock => {
                if request.jobs.is_some() {
                    return Err(StoreError::Invalid(
                        "mock imports do not accept supplied jobs".into(),
                    ));
                }
                MockJobSource::bundled(config.effective.radar.job_sources.mock.enabled)
                    .map_err(|_| StoreError::Invalid("mock fixture unavailable".into()))?
                    .jobs
            }
            ImportSource::Manual => request
                .jobs
                .ok_or_else(|| StoreError::Invalid("manual imports require jobs".into()))?,
        };
        if (jobs.is_empty() && !matches!(request.source, ImportSource::Upwork)) || jobs.len() > 1000
        {
            return Err(StoreError::Invalid("import requires 1–1000 records".into()));
        }
        let provider = match request.source {
            ImportSource::Mock => "mock",
            ImportSource::Manual => "manual",
            ImportSource::Upwork => "upwork",
        };
        for job in &jobs {
            job.validate().map_err(StoreError::Invalid)?;
            if job.provider != provider {
                return Err(StoreError::Invalid(
                    "job provider does not match import source".into(),
                ));
            }
        }
        let identities = jobs.iter().map(identity).collect::<Result<Vec<_>>>()?;
        let existing = sqlx::query("SELECT j.canonical_identity,j.decision,j.payload,a.status FROM jobs j JOIN applications a ON a.job_id=j.id WHERE j.canonical_identity=ANY($1) FOR UPDATE OF j")
            .bind(identities).fetch_all(&mut **tx).await?;
        for job in &mut jobs {
            let key = identity(job)?;
            if let Some(row) = existing
                .iter()
                .find(|row| row.get::<String, _>("canonical_identity") == key)
            {
                let status: ApplicationStatus = enum_value(row.try_get("status")?)?;
                let decision = row
                    .try_get::<Option<String>, _>("decision")?
                    .map(enum_value::<FeedbackAction>)
                    .transpose()?;
                let previous: Json<Job> = row.try_get("payload")?;
                job.already_applied |=
                    !matches!(status, ApplicationStatus::None | ApplicationStatus::Drafted)
                        || previous.0.already_applied;
                job.excluded |= decision.is_some_and(FeedbackAction::excludes_recommendations);
            }
        }
        let settings = config.effective.settings();
        let date = now.with_timezone(&settings.timezone).date_naive();
        let usage = sqlx::query("SELECT requests,jobs FROM daily_usage WHERE local_date=$1")
            .bind(date)
            .fetch_optional(&mut **tx)
            .await?;
        let usage = match usage {
            Some(row) => DailyUsage {
                requests: u32::try_from(row.try_get::<i64, _>("requests")?)
                    .map_err(|_| StoreError::Invalid("stored request quota out of range".into()))?,
                jobs: u32::try_from(row.try_get::<i64, _>("jobs")?)
                    .map_err(|_| StoreError::Invalid("stored job quota out of range".into()))?,
            },
            None => DailyUsage::default(),
        };
        if let Some(limit) = options.per_run_limit {
            jobs.truncate(limit as usize);
        }
        let mut buffer = MemoryRepository::with_daily_usage(date, usage);
        let mut source: Box<dyn JobSource + Send> = match request.source {
            ImportSource::Mock => Box::new(MockJobSource {
                jobs,
                enabled: config.effective.radar.job_sources.mock.enabled,
            }),
            ImportSource::Upwork => Box::new(crate::upwork::UpworkJobSource {
                jobs,
                stats: upwork_stats.unwrap_or_default(),
            }),
            ImportSource::Manual => Box::new(
                ManualJobSource::from_jobs(jobs, config.effective.radar.job_sources.manual.enabled)
                    .map_err(|e| StoreError::Invalid(e.to_string()))?,
            ),
        };
        let report = scan(
            source.as_mut(),
            &mut buffer,
            &FixedClock(now),
            &settings,
            profile,
            &config.effective.policy(),
        )
        .map_err(|e| StoreError::Invalid(e.to_string()))?;
        let usage = buffer.usage(date);
        sqlx::query("INSERT INTO daily_usage(local_date,requests,jobs) VALUES ($1,$2,$3) ON CONFLICT(local_date) DO UPDATE SET requests=excluded.requests,jobs=excluded.jobs")
            .bind(date).bind(i64::from(usage.requests)).bind(i64::from(usage.jobs)).execute(&mut **tx).await?;
        sqlx::query("INSERT INTO profile_revisions(revision,snapshot) VALUES ($1,$2) ON CONFLICT DO NOTHING").bind(&profile.revision).bind(Json(profile)).execute(&mut **tx).await?;
        sqlx::query("INSERT INTO imports(id,request_hash,config_revision,profile_revision) VALUES ($1,$2,$3,$4)")
            .bind(request.idempotency_key).bind(hash).bind(config.revision).bind(&profile.revision).execute(&mut **tx).await?;
        for record in &report.records {
            let job = &record
                .ingestions
                .last()
                .ok_or_else(|| StoreError::Invalid("empty staged ingestion".into()))?
                .job;
            let source_id = job
                .source_id
                .as_ref()
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim());
            let id: Uuid = sqlx::query_scalar("INSERT INTO jobs(id,canonical_identity,provider,source_id,title,payload,fetched_at) VALUES ($1,$2,$3,$4,$5,$6,$7)
                ON CONFLICT(canonical_identity) DO UPDATE SET title=excluded.title,payload=excluded.payload,fetched_at=excluded.fetched_at RETURNING id")
                .bind(Uuid::new_v4()).bind(identity(job)?).bind(&job.provider).bind(source_id).bind(&job.title).bind(Json(job)).bind(now).fetch_one(&mut **tx).await?;
            sqlx::query("INSERT INTO applications(job_id) VALUES ($1) ON CONFLICT DO NOTHING")
                .bind(id)
                .execute(&mut **tx)
                .await?;
            for ingestion in &record.ingestions {
                sqlx::query("INSERT INTO job_ingestions(job_id,import_id,payload_hash,input,fetched_at) VALUES ($1,$2,$3,$4,$5)")
                    .bind(id).bind(request.idempotency_key).bind(fingerprint(&ingestion.job)?).bind(Json(&ingestion.job)).bind(ingestion.fetched_at).execute(&mut **tx).await?;
            }
            sqlx::query("INSERT INTO evaluations(job_id,import_id,config_revision,profile_revision,input,snapshot,created_at) VALUES ($1,$2,$3,$4,$5,$6,$7)")
                .bind(id).bind(request.idempotency_key).bind(config.revision).bind(&profile.revision).bind(Json(job)).bind(Json(&record.evaluation)).bind(now).execute(&mut **tx).await?;
        }
        for record in &report.records {
            if record.evaluation.verdict == radar_domain::Verdict::ApplyCandidate
                && record
                    .ingestions
                    .last()
                    .is_some_and(|i| radar_application::bots::notification_ready(&i.job))
                && radar_application::bots::NotificationRouter::confident(
                    &config.effective,
                    &record.evaluation.confidence,
                )
            {
                let job = &record.ingestions.last().expect("validated ingestion").job;
                let row = sqlx::query("SELECT id,revision FROM jobs WHERE canonical_identity=$1")
                    .bind(identity(job)?)
                    .fetch_one(&mut **tx)
                    .await?;
                let id: Uuid = row.try_get("id")?;
                outbox::enqueue_event(
                    tx,
                    &config.effective,
                    &format!("job-matched:{id}"),
                    radar_application::bots::EventKind::JobMatched,
                    outbox::job_message(id, row.try_get("revision")?, job, &record.evaluation),
                    now,
                )
                .await?;
            }
        }
        let new_jobs = report
            .records
            .iter()
            .filter(|r| {
                r.ingestions.last().is_some_and(|i| {
                    identity(&i.job).is_ok_and(|key| {
                        !existing
                            .iter()
                            .any(|e| e.get::<String, _>("canonical_identity") == key)
                    })
                })
            })
            .count();
        let result = ImportResult {
            new_jobs,
            duplicates: report.processed.saturating_sub(new_jobs),
            id: request.idempotency_key,
            status: report.status,
            processed: report.processed,
            canonical_jobs: report.records.len(),
            config_revision: config.revision,
        };
        sqlx::query("UPDATE imports SET response=$1 WHERE id=$2")
            .bind(Json(&result))
            .bind(result.id)
            .execute(&mut **tx)
            .await?;
        audit(
            tx,
            actor,
            "jobs_imported",
            &result.id.to_string(),
            json!({"source":provider,"processed":result.processed,"status":result.status}),
        )
        .await?;
        Ok(result)
    }
    pub async fn feedback(
        &self,
        id: Uuid,
        request: FeedbackRequest,
        actor: &str,
    ) -> Result<FeedbackView> {
        if request.reason.as_ref().is_some_and(|s| s.len() > 2000) {
            return Err(StoreError::Invalid(
                "feedback reason is limited to 2000 bytes".into(),
            ));
        }
        if request.submitted_at.is_some_and(|at| at > Utc::now())
            || (request.submitted_at.is_some() && request.action != FeedbackAction::Applied)
        {
            return Err(StoreError::Invalid(
                "submission time is allowed only for Applied and cannot be in the future".into(),
            ));
        }
        let hash = fingerprint(&(id, &request))?;
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query("SELECT j.revision,j.decision,a.status FROM jobs j JOIN applications a ON a.job_id=j.id WHERE j.id=$1 FOR UPDATE OF j,a")
            .bind(id).fetch_optional(&mut *tx).await?.ok_or(StoreError::NotFound)?;
        if let Some(previous) = sqlx::query(
            "SELECT f.*,j.title FROM feedback f JOIN jobs j ON j.id=f.job_id WHERE f.id=$1",
        )
        .bind(request.idempotency_key)
        .fetch_optional(&mut *tx)
        .await?
        {
            if previous.try_get::<String, _>("request_hash")? != hash {
                return Err(StoreError::Conflict);
            }
            return feedback_view(&previous);
        }
        let revision: i64 = row.try_get("revision")?;
        if revision != request.expected_revision {
            return Err(StoreError::Conflict);
        }
        let current = enum_value::<ApplicationStatus>(row.try_get("status")?)?;
        let status = request
            .action
            .application_status(current, request.confirmed_submission)
            .map_err(|e| StoreError::Invalid(e.into()))?;
        let revision = revision + 1;
        if request.action == FeedbackAction::Other {
            sqlx::query("UPDATE jobs SET revision=$1 WHERE id=$2")
                .bind(revision)
                .bind(id)
                .execute(&mut *tx)
                .await?;
        } else if request.action.is_preference() {
            sqlx::query("UPDATE jobs SET decision=$1,revision=$2 WHERE id=$3")
                .bind(name(request.action)?)
                .bind(revision)
                .bind(id)
                .execute(&mut *tx)
                .await?;
        } else {
            sqlx::query("UPDATE jobs SET revision=$1 WHERE id=$2")
                .bind(revision)
                .bind(id)
                .execute(&mut *tx)
                .await?;
            sqlx::query("UPDATE applications SET status=$1,submitted_at=CASE WHEN $1='submitted' THEN $3 ELSE submitted_at END,submission_reported_at=CASE WHEN $1='submitted' THEN now() ELSE submission_reported_at END WHERE job_id=$2")
                .bind(name(status)?).bind(id).bind(request.submitted_at).execute(&mut *tx).await?;
        }
        sqlx::query("INSERT INTO feedback(id,job_id,request_hash,action,reason,actor,revision,application_status) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)")
            .bind(request.idempotency_key).bind(id).bind(hash).bind(name(request.action)?).bind(&request.reason).bind(actor).bind(revision).bind(name(status)?).execute(&mut *tx).await?;
        self.record_learning_feedback(
            &mut tx,
            id,
            &request,
            row.try_get::<Option<String>, _>("decision")?
                .map(enum_value)
                .transpose()?,
            current,
        )
        .await?;
        audit(
            &mut tx,
            actor,
            "feedback_recorded",
            &id.to_string(),
            json!({"action":request.action,"revision":revision}),
        )
        .await?;
        let row = sqlx::query(
            "SELECT f.*,j.title FROM feedback f JOIN jobs j ON j.id=f.job_id WHERE f.id=$1",
        )
        .bind(request.idempotency_key)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        feedback_view(&row)
    }
    pub async fn feedback_history(&self, query: FeedbackQuery) -> Result<FeedbackPage> {
        let (page, size, offset) = page(query.page, query.page_size)?;
        let total: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM feedback WHERE ($1::uuid IS NULL OR job_id=$1)",
        )
        .bind(query.job_id)
        .fetch_one(&self.pool)
        .await?;
        let rows = sqlx::query("SELECT f.*,j.title FROM feedback f JOIN jobs j ON j.id=f.job_id WHERE ($1::uuid IS NULL OR f.job_id=$1) ORDER BY f.created_at DESC,f.id LIMIT $2 OFFSET $3")
            .bind(query.job_id).bind(i64::from(size)).bind(offset).fetch_all(&self.pool).await?;
        Ok(FeedbackPage {
            items: rows.iter().map(feedback_view).collect::<Result<_>>()?,
            total,
            page,
            page_size: size,
        })
    }
}
