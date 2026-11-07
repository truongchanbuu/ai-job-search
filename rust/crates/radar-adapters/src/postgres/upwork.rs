use super::*;
use crate::upwork::{SearchStats, UpworkGateway, UpworkJobSource, state::PrivateState};
impl PgRepository {
    pub async fn queue_upwork_scan(
        &self,
        request: UpworkScanRequest,
        origin: &str,
        now: DateTime<Utc>,
    ) -> Result<ScanRun> {
        if !matches!(origin, "manual" | "discord") {
            return Err(StoreError::Invalid("Upwork manual mode only".into()));
        }
        let config = self.config().await?;
        let upwork = &config.effective.radar.job_sources.upwork_mcp;
        if !upwork.enabled {
            return Err(StoreError::Invalid(
                "Upwork source disabled; manual mode only".into(),
            ));
        }
        let settings = upwork
            .runtime
            .as_ref()
            .ok_or_else(|| StoreError::Invalid("Upwork configuration missing".into()))?;
        let profile = request
            .profile
            .unwrap_or_else(|| settings.default_profile.clone());
        if !settings.profiles.get(&profile).is_some_and(|p| p.enabled) {
            return Err(StoreError::Invalid(
                "Upwork search profile disabled or missing".into(),
            ));
        }
        // Same admission limit and idempotency ledger as mock scans, with source bound atomically.
        let mut tx = self.begin("queue_upwork_scan").await?;
        sqlx::query("SELECT revision FROM runtime_config WHERE singleton=true FOR UPDATE")
            .fetch_one(&mut *tx)
            .await?;
        if let Some(row) = sqlx::query("SELECT source,search_profile FROM scan_runs WHERE id=$1")
            .bind(request.idempotency_key)
            .fetch_optional(&mut *tx)
            .await?
        {
            if row.try_get::<String, _>("source")? != "upwork"
                || row
                    .try_get::<Option<String>, _>("search_profile")?
                    .as_deref()
                    != Some(&profile)
            {
                return Err(StoreError::Conflict);
            }
            crate::database::commit(tx, &self.pool, "queue_upwork_scan").await?;
            return self.scan_run(request.idempotency_key).await;
        }
        let count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM scan_runs WHERE status IN ('pending','running')",
        )
        .fetch_one(&mut *tx)
        .await?;
        let limit = config
            .effective
            .radar
            .scheduler
            .as_ref()
            .map_or(0, |s| s.max_pending_runs);
        if count >= i64::from(limit) {
            return Err(StoreError::Invalid("scan queue is full".into()));
        }
        sqlx::query("INSERT INTO scan_runs(id,run_key,origin,scheduled_at,source,search_profile) VALUES ($1,$2,$3,$4,'upwork',$5)").bind(request.idempotency_key).bind(format!("request:{}",request.idempotency_key)).bind(origin).bind(now).bind(profile).execute(&mut *tx).await?;
        crate::database::commit(tx, &self.pool, "queue_upwork_scan").await?;
        self.scan_run(request.idempotency_key).await
    }
    pub async fn scan_run(&self, id: Uuid) -> Result<ScanRun> {
        let row = sqlx::query("SELECT * FROM scan_runs WHERE id=$1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(StoreError::NotFound)?;
        super::scheduler::run_view(&row)
    }
    pub(super) async fn prepare_upwork(
        &self,
        config: &ConfigView,
        profile: Option<&str>,
        limit: Option<u32>,
        gateway: &dyn UpworkGateway,
        now: DateTime<Utc>,
    ) -> Result<Option<UpworkJobSource>> {
        let source = &config.effective.radar.job_sources.upwork_mcp;
        if !source.enabled {
            return Err(StoreError::Invalid(
                "Upwork source disabled; manual mode only".into(),
            ));
        }
        if config.effective.radar.app.active_mode == radar_domain::Mode::Paused {
            return Err(StoreError::Invalid("work mode paused".into()));
        }
        let settings = source
            .runtime
            .as_ref()
            .ok_or_else(|| StoreError::Invalid("Upwork runtime missing".into()))?;
        let mode = config.effective.settings();
        let date = now.with_timezone(&mode.timezone).date_naive();
        let usage = sqlx::query("SELECT requests,jobs FROM daily_usage WHERE local_date=$1")
            .bind(date)
            .fetch_optional(&self.pool)
            .await?;
        let (requests, jobs) = usage
            .map(|r| {
                (
                    r.get::<i64, _>("requests") as u32,
                    r.get::<i64, _>("jobs") as u32,
                )
            })
            .unwrap_or_default();
        if requests >= mode.max_requests || jobs >= mode.max_jobs {
            return Err(StoreError::Invalid(
                "daily quota reached before Upwork search".into(),
            ));
        }
        let limit = (mode.max_jobs - jobs).min(limit.unwrap_or(settings.max_results_per_scan));
        gateway
            .search(
                settings,
                profile.unwrap_or(&settings.default_profile),
                limit as usize,
            )
            .await
            .map(Some)
            .map_err(StoreError::Upwork)
    }
    pub(super) async fn store_upwork_success(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        config: &ConfigView,
        stats: &SearchStats,
        result: &ImportResult,
        now: DateTime<Utc>,
    ) -> Result<()> {
        let view = UpworkStatus {
            enabled: config.effective.radar.job_sources.upwork_mcp.enabled,
            execution_mode: config
                .effective
                .radar
                .job_sources
                .upwork_mcp
                .runtime
                .as_ref()
                .map(|s| s.execution_mode.clone())
                .unwrap_or_else(|| "manual".into()),
            connection: "last_search_succeeded; no persistent connection".into(),
            authentication: "authenticated_at_last_search".into(),
            account_selected: true,
            account_label: None,
            last_successful_search: Some(now),
            last_error_category: stats.error_category.clone(),
            fetched: stats.fetched,
            new_jobs: result.new_jobs,
            duplicates: result.duplicates,
            accepted: stats.accepted,
            invalid: stats.invalid,
            filtered: stats.filtered,
            partial: stats.partial,
            empty_reason: stats.empty_reason.clone(),
            filters_ignored: stats.filters_ignored.clone(),
            skills_filtered_client_side: stats.skills_filtered_client_side,
            retry_after_seconds: stats.retry_after_seconds,
            quota: None,
        };
        sqlx::query("UPDATE upwork_source_state SET snapshot=$1 WHERE singleton=true")
            .bind(Json(view))
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
    pub(super) async fn store_upwork_error(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        config: &ConfigView,
        error: &StoreError,
    ) -> Result<()> {
        let mut view = self.upwork_status().await?;
        view.enabled = config.effective.radar.job_sources.upwork_mcp.enabled;
        view.execution_mode = config
            .effective
            .radar
            .job_sources
            .upwork_mcp
            .runtime
            .as_ref()
            .map(|s| s.execution_mode.clone())
            .unwrap_or_else(|| "manual".into());
        view.connection = "disconnected".into();
        // Only application-owned structured categories reach this path; never upstream error prose.
        let category = match error {
            StoreError::Upwork(e) => {
                view.retry_after_seconds = e.retry_after_seconds;
                e.category.clone()
            }
            StoreError::Invalid(_) => "source_configuration_or_quota".into(),
            _ => "database_unavailable".into(),
        };
        view.last_error_category = Some(category.clone());
        if category.contains("reauthorization") {
            view.authentication = "reauthorization_required".into();
        }
        sqlx::query("UPDATE upwork_source_state SET snapshot=$1 WHERE singleton=true")
            .bind(Json(view))
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
    pub async fn upwork_status(&self) -> Result<UpworkStatus> {
        let config = self.config().await?;
        let saved: Json<Value> =
            sqlx::query_scalar("SELECT snapshot FROM upwork_source_state WHERE singleton=true")
                .fetch_one(&self.pool)
                .await?;
        let mut view: UpworkStatus = if saved.0 == json!({}) {
            UpworkStatus {
                execution_mode: "manual_only".into(),
                connection: "disconnected".into(),
                authentication: "reauthorization_required".into(),
                ..Default::default()
            }
        } else {
            serde_json::from_value(saved.0)?
        };
        view.enabled = config.effective.radar.job_sources.upwork_mcp.enabled;
        if let Some(s) = &config.effective.radar.job_sources.upwork_mcp.runtime {
            match PrivateState::from_settings(s) {
                Ok(state) => {
                    if let Ok(quota) = state.quota_snapshot(chrono::Utc::now().timestamp()) {
                        view.quota = Some(UpworkQuota {
                            minute_used: quota.minute_used,
                            minute_limit: s.max_requests_per_minute,
                            minute_reset_at: chrono::DateTime::from_timestamp(
                                quota.minute_reset_at,
                                0,
                            )
                            .unwrap_or_else(chrono::Utc::now),
                            hour_used: quota.hour_used,
                            hour_limit: s.max_requests_per_hour,
                            hour_reset_at: chrono::DateTime::from_timestamp(quota.hour_reset_at, 0)
                                .unwrap_or_else(chrono::Utc::now),
                            day_used: quota.day_used,
                            day_limit: s.max_requests_per_day,
                            day_reset_at: chrono::DateTime::from_timestamp(quota.day_reset_at, 0)
                                .unwrap_or_else(chrono::Utc::now),
                            blocked_until: quota
                                .blocked_until
                                .and_then(|at| chrono::DateTime::from_timestamp(at, 0)),
                        });
                    }
                    let account = state
                        .read::<crate::upwork::Account>("account.enc")
                        .ok()
                        .flatten();
                    view.account_selected = account.is_some();
                    view.account_label = account.map(|a| {
                        format!("{} ({})", a.name, a.role_label)
                            .chars()
                            .filter(|c| !c.is_control())
                            .take(200)
                            .collect()
                    });
                    match state.read::<rmcp::transport::auth::StoredCredentials>("oauth.enc") {
                        Ok(Some(credentials))
                            if credentials.token_response.is_some()
                                && view.last_error_category.as_deref()
                                    != Some("reauthorization_required") =>
                        {
                            view.authentication =
                                "credentials_stored; validity checked on manual search".into()
                        }
                        Ok(_) => view.authentication = "reauthorization_required".into(),
                        Err(e) => {
                            view.authentication = "state_unavailable".into();
                            view.last_error_category.get_or_insert(e.category);
                        }
                    }
                }
                Err(e) if view.enabled => {
                    view.authentication = "state_unavailable".into();
                    view.last_error_category.get_or_insert(e.category);
                }
                Err(_) => {}
            }
        }
        Ok(view)
    }
}
