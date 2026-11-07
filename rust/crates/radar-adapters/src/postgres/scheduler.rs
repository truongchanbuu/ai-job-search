use super::*;
use radar_application::scheduling::{next_scan, scan_slots};
use radar_domain::Mode;

async fn coordination_lock(tx: &mut Transaction<'_, Postgres>) -> Result<bool> {
    Ok(sqlx::query_scalar("SELECT pg_try_advisory_xact_lock(hashtextextended(current_schema()||':radar:scan-worker',0))").fetch_one(&mut **tx).await?)
}
pub(super) fn run_view(row: &PgRow) -> Result<ScanRun> {
    Ok(ScanRun {
        source: row.try_get("source")?,
        search_profile: row.try_get("search_profile")?,
        id: row.try_get("id")?,
        status: enum_value(row.try_get("status")?)?,
        origin: row.try_get("origin")?,
        scheduled_at: row.try_get("scheduled_at")?,
        started_at: row.try_get("started_at")?,
        completed_at: row.try_get("completed_at")?,
        attempts: row.try_get("attempts")?,
        result: row
            .try_get::<Option<Json<ImportResult>>, _>("result")?
            .map(|j| j.0),
        error: row.try_get("error")?,
    })
}
impl PgRepository {
    pub async fn queue_scan(&self, id: Uuid, origin: &str, now: DateTime<Utc>) -> Result<ScanRun> {
        if !["manual", "discord"].contains(&origin) {
            return Err(StoreError::Invalid("invalid scan origin".into()));
        }
        let mut tx = self.begin("queue_scan").await?;
        let config = config_view(
            &sqlx::query("SELECT * FROM runtime_config WHERE singleton=true FOR UPDATE")
                .fetch_one(&mut *tx)
                .await?,
        )?;
        if let Some(row) = sqlx::query("SELECT * FROM scan_runs WHERE id=$1")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?
        {
            if row.try_get::<String, _>("source")? != "mock" {
                return Err(StoreError::Conflict);
            }
            return run_view(&row);
        }
        let limit = config
            .effective
            .radar
            .scheduler
            .as_ref()
            .ok_or_else(|| StoreError::Invalid("scheduler settings missing".into()))?
            .max_pending_runs;
        let count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM scan_runs WHERE status IN ('pending','running')",
        )
        .fetch_one(&mut *tx)
        .await?;
        if count >= i64::from(limit) {
            return Err(StoreError::Invalid(
                "scan queue is full; wait for existing runs".into(),
            ));
        }
        let row=sqlx::query("INSERT INTO scan_runs(id,run_key,origin,scheduled_at) VALUES ($1,$2,$3,$4) RETURNING *").bind(id).bind(format!("request:{id}")).bind(origin).bind(now).fetch_one(&mut *tx).await?;
        crate::database::commit(tx, &self.pool, "queue_scan").await?;
        run_view(&row)
    }
    pub async fn scheduler_status(&self, now: DateTime<Utc>) -> Result<SchedulerView> {
        let config = self.config().await?;
        let rows = sqlx::query("SELECT * FROM scan_runs ORDER BY scheduled_at DESC,id LIMIT 50")
            .fetch_all(&self.pool)
            .await?;
        let runs = rows.iter().map(run_view).collect::<Result<Vec<_>>>()?;
        let window = config
            .effective
            .radar
            .scheduler
            .as_ref()
            .map_or(0, |s| s.recovery_after_seconds);
        let row=sqlx::query("SELECT count(*) AS active,max(last_seen) AS last_seen FROM worker_heartbeats WHERE last_seen>$1").bind(now-chrono::Duration::seconds(window as i64)).fetch_one(&self.pool).await?;
        Ok(SchedulerView {
            active_workers: row.try_get("active")?,
            last_heartbeat: row.try_get("last_seen")?,
            enabled: config
                .effective
                .radar
                .scheduler
                .as_ref()
                .is_some_and(|s| s.enabled),
            mode: config.effective.radar.app.active_mode,
            next_scan: next_scan(&config.effective, now),
            last_run: runs.iter().find(|r| r.completed_at.is_some()).cloned(),
            runs,
        })
    }
    pub async fn plan_scans(&self, now: DateTime<Utc>) -> Result<()> {
        let mut tx = self.begin("plan_scans").await?;
        if !coordination_lock(&mut tx).await? {
            return Ok(());
        }
        let config = config_view(
            &sqlx::query("SELECT * FROM runtime_config WHERE singleton=true FOR UPDATE")
                .fetch_one(&mut *tx)
                .await?,
        )?;
        let state = sqlx::query("SELECT * FROM scheduler_state WHERE singleton=true FOR UPDATE")
            .fetch_one(&mut *tx)
            .await?;
        let Some(settings) = &config.effective.radar.scheduler else {
            return Ok(());
        };
        sqlx::query("UPDATE scan_runs SET status='failed',owner=NULL,lease_until=NULL,completed_at=$1,error='interrupted manual Upwork run; trigger a new manual scan' WHERE source='upwork' AND status='running' AND lease_until<=$1").bind(now).execute(&mut *tx).await?;
        sqlx::query("UPDATE scan_runs SET status='pending',owner=NULL,lease_until=NULL,error='interrupted run recovered' WHERE source='mock' AND status='running' AND lease_until<=$1").bind(now).execute(&mut *tx).await?;
        let previous = state.try_get::<Option<DateTime<Utc>>, _>("planned_through")?;
        let revision = state.try_get::<Option<i64>, _>("config_revision")?;
        let floor = now - chrono::Duration::minutes(i64::from(settings.catch_up_minutes));
        let after = if revision.is_some_and(|r| r != config.revision) {
            now - chrono::Duration::seconds(settings.poll_interval_seconds as i64)
        } else {
            previous.unwrap_or(floor).max(floor)
        };
        if settings.enabled {
            let slots = scan_slots(&config.effective, after, now);
            if let Some(slot) = slots.last() {
                let key = format!("mock:scheduled:{slot}");
                sqlx::query("INSERT INTO scan_runs(id,run_key,origin,scheduled_at) VALUES ($1,$2,'scheduled',$3) ON CONFLICT(run_key) DO NOTHING")
                    .bind(Uuid::new_v4()).bind(key).bind(slot).execute(&mut *tx).await?;
                audit(
                    &mut tx,
                    "scheduler",
                    "schedule_planned",
                    "scheduler",
                    json!({"slot":slot,"coalesced_slots":slots.len().saturating_sub(1)}),
                )
                .await?;
            }
        }
        sqlx::query(
            "UPDATE scheduler_state SET planned_through=$1,config_revision=$2 WHERE singleton=true",
        )
        .bind(now)
        .bind(config.revision)
        .execute(&mut *tx)
        .await?;
        crate::database::commit(tx, &self.pool, "plan_scans").await?;
        Ok(())
    }
    /// Claim is committed independently so another process can recover an interrupted execution.
    pub async fn claim_scan(&self, now: DateTime<Utc>) -> Result<Option<(Uuid, Uuid)>> {
        let mut tx = self.begin("claim_scan").await?;
        if !coordination_lock(&mut tx).await? {
            return Ok(None);
        }
        let config = self.config_in(&mut tx).await?;
        let Some(settings) = config.effective.radar.scheduler else {
            return Ok(None);
        };
        let row=sqlx::query("SELECT * FROM scan_runs WHERE status='pending' AND scheduled_at<=$1 AND NOT EXISTS(SELECT 1 FROM scan_runs WHERE status='running') ORDER BY scheduled_at,id LIMIT 1 FOR UPDATE SKIP LOCKED")
            .bind(now).fetch_optional(&mut *tx).await?;
        let Some(row) = row else { return Ok(None) };
        let id: Uuid = row.try_get("id")?;
        let owner = Uuid::new_v4();
        sqlx::query("UPDATE scan_runs SET status='running',owner=$2,lease_until=$3,started_at=$4,attempts=attempts+1,error=NULL WHERE id=$1")
            .bind(id).bind(owner).bind(now+chrono::Duration::seconds(settings.recovery_after_seconds as i64)).bind(now).execute(&mut *tx).await?;
        crate::database::commit(tx, &self.pool, "claim_scan").await?;
        Ok(Some((id, owner)))
    }
    pub async fn execute_scan(
        &self,
        id: Uuid,
        owner: Uuid,
        profile: &Profile,
        now: DateTime<Utc>,
    ) -> Result<()> {
        self.execute_scan_with_gateway(id, owner, profile, now, &crate::upwork::OfficialUpwork)
            .await
    }
    pub async fn execute_scan_with_gateway(
        &self,
        id: Uuid,
        owner: Uuid,
        profile: &Profile,
        now: DateTime<Utc>,
        gateway: &dyn crate::upwork::UpworkGateway,
    ) -> Result<()> {
        let mut tx = self.begin("execute_scan_with_gateway").await?;
        if !coordination_lock(&mut tx).await? {
            return Ok(());
        }
        let row = sqlx::query(
            "SELECT * FROM scan_runs WHERE id=$1 AND status='running' AND owner=$2 FOR UPDATE",
        )
        .bind(id)
        .bind(owner)
        .fetch_optional(&mut *tx)
        .await?;
        let Some(row) = row else { return Ok(()) };
        let config = self.config_in(&mut tx).await?;
        let is_upwork = row.try_get::<String, _>("source")? == "upwork";
        let limit = config
            .effective
            .radar
            .scheduler
            .as_ref()
            .map(|s| s.max_jobs_per_run);
        // Savepoint retains the claim on a source error while rolling back partial job changes.
        sqlx::query("SAVEPOINT scan_work").execute(&mut *tx).await?;
        let prepared = if is_upwork {
            self.prepare_upwork(
                &config,
                row.try_get::<Option<String>, _>("search_profile")?
                    .as_deref(),
                limit,
                gateway,
                now,
            )
            .await
        } else {
            Ok(None)
        };
        let mut stats = prepared
            .as_ref()
            .ok()
            .and_then(|s| s.as_ref())
            .map(|s| s.stats.clone());
        let result = match prepared {
            Ok(source) => {
                self.import_in(
                    &mut tx,
                    ImportRequest {
                        idempotency_key: id,
                        source: if is_upwork {
                            ImportSource::Upwork
                        } else {
                            ImportSource::Mock
                        },
                        jobs: None,
                    },
                    profile,
                    "scheduler",
                    now,
                    ImportOptions {
                        expected_revision: is_upwork.then_some(config.revision),
                        per_run_limit: limit,
                        upwork: source,
                    },
                )
                .await
            }
            Err(e) => Err(e),
        };
        match result {
            Ok(mut result) => {
                if is_upwork {
                    let stats = stats.take().unwrap_or_default();
                    if stats.partial {
                        result.status = format!("{}; partial results", result.status)
                    }
                    self.store_upwork_success(&mut tx, &config, &stats, &result, now)
                        .await?;
                }
                sqlx::query("UPDATE scan_runs SET status='completed',result=$2,completed_at=$3,lease_until=NULL WHERE id=$1").bind(id).bind(Json(result)).bind(now).execute(&mut *tx).await?;
            }
            Err(error) => {
                sqlx::query("ROLLBACK TO SAVEPOINT scan_work")
                    .execute(&mut *tx)
                    .await?;
                let diagnostic = if is_upwork {
                    format!("Upwork manual search failed: {}", error)
                } else {
                    "mock scan failed; check source configuration and database status".into()
                };
                if is_upwork {
                    self.store_upwork_error(&mut tx, &config, &error).await?;
                }
                super::outbox::enqueue_event(
                    &mut tx,
                    &config.effective,
                    &format!("scan-failed:{id}"),
                    radar_application::bots::EventKind::ScanFailed,
                    radar_application::bots::NotificationMessage {
                        title: "Radar scan failed".into(),
                        body: format!("Run {id}: {diagnostic}"),
                        job_id: None,
                        job_revision: None,
                        score: None,
                    },
                    now,
                )
                .await?;
                sqlx::query("UPDATE scan_runs SET status='failed',error=$3,completed_at=$2,lease_until=NULL WHERE id=$1").bind(id).bind(now).bind(diagnostic).execute(&mut *tx).await?;
            }
        }
        crate::database::commit(tx, &self.pool, "execute_scan_with_gateway").await?;
        Ok(())
    }
    pub async fn run_scan_tick(&self, profile: &Profile, now: DateTime<Utc>) -> Result<()> {
        self.plan_scans(now).await?;
        if let Some((id, owner)) = self.claim_scan(now).await? {
            self.execute_scan(id, owner, profile, now).await?;
        }
        Ok(())
    }
    pub async fn set_paused(&self, request: ModeRequest, actor: &str) -> Result<ConfigView> {
        let mut tx = self.begin("set_paused").await?;
        let current = config_view(
            &sqlx::query("SELECT * FROM runtime_config WHERE singleton=true FOR UPDATE")
                .fetch_one(&mut *tx)
                .await?,
        )?;
        if let Some(row) = sqlx::query("SELECT * FROM mode_requests WHERE id=$1")
            .bind(request.idempotency_key)
            .fetch_optional(&mut *tx)
            .await?
        {
            if row.try_get::<bool, _>("paused")? != request.paused {
                return Err(StoreError::Conflict);
            }
            return Ok(row.try_get::<Json<ConfigView>, _>("response")?.0);
        }
        if current.revision != request.expected_revision {
            return Err(StoreError::Conflict);
        }
        let state = sqlx::query(
            "SELECT previous_mode FROM scheduler_state WHERE singleton=true FOR UPDATE",
        )
        .fetch_one(&mut *tx)
        .await?;
        let mut radar = current.effective.radar.clone();
        if request.paused && radar.app.active_mode != Mode::Paused {
            sqlx::query("UPDATE scheduler_state SET previous_mode=$1 WHERE singleton=true")
                .bind(name(radar.app.active_mode)?)
                .execute(&mut *tx)
                .await?;
            radar.app.active_mode = Mode::Paused;
        } else if !request.paused && radar.app.active_mode == Mode::Paused {
            radar.app.active_mode = state
                .try_get::<Option<String>, _>("previous_mode")?
                .map(enum_value)
                .transpose()?
                .unwrap_or(
                    radar
                        .scheduler
                        .as_ref()
                        .ok_or_else(|| {
                            StoreError::Invalid("scheduler settings required to resume".into())
                        })?
                        .resume_mode,
                );
        }
        let response = if radar.app.active_mode == current.effective.radar.app.active_mode {
            current
        } else {
            self.update_config_in(
                &mut tx,
                ConfigUpdate {
                    expected_revision: current.revision,
                    radar: Some(radar),
                    scoring: None,
                    notifications: None,
                },
                actor,
            )
            .await?
        };
        sqlx::query("INSERT INTO mode_requests(id,paused,response) VALUES ($1,$2,$3)")
            .bind(request.idempotency_key)
            .bind(request.paused)
            .bind(Json(&response))
            .execute(&mut *tx)
            .await?;
        crate::database::commit(tx, &self.pool, "set_paused").await?;
        Ok(response)
    }
}
