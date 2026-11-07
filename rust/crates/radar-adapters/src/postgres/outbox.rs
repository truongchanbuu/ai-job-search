use super::*;
use crate::{bots::BotProviderRegistry, discord::snowflake};
use radar_application::{bots::*, config::Severity, scheduling::slots};
use radar_domain::{Mode, Verdict};

pub(super) async fn enqueue_event(
    tx: &mut Transaction<'_, Postgres>,
    config: &Config,
    key: &str,
    kind: EventKind,
    message: NotificationMessage,
    now: DateTime<Utc>,
) -> Result<Uuid> {
    let id = Uuid::new_v5(&Uuid::NAMESPACE_OID, key.as_bytes());
    let inserted=sqlx::query("INSERT INTO outbox_events(id,event_key,kind,payload,created_at) VALUES ($1,$2,$3,$4,$5) ON CONFLICT(event_key) DO NOTHING")
        .bind(id).bind(key).bind(name(kind)?).bind(Json(&message)).bind(now).execute(&mut **tx).await?.rows_affected()>0;
    if inserted {
        for destination in NotificationRouter::destinations(config, kind, message.score) {
            if let Some(target) = config.notifications.destinations.get(&destination) {
                sqlx::query("INSERT INTO notification_deliveries(id,event_id,destination,provider,next_attempt_at) VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING")
                    .bind(Uuid::new_v4()).bind(id).bind(destination).bind(&target.provider).bind(now).execute(&mut **tx).await?;
            }
        }
    }
    Ok(id)
}
pub(super) fn job_message(
    id: Uuid,
    revision: i64,
    job: &Job,
    evaluation: &radar_domain::Evaluation,
) -> NotificationMessage {
    NotificationMessage {
        title: job.title.clone(),
        body: format!(
            "Score: {} | Confidence: {:?} | Risk: {}\n{}",
            evaluation
                .score
                .map(|s| format!("{s:.2}"))
                .unwrap_or_else(|| "unknown".into()),
            evaluation.confidence,
            evaluation.risk,
            job.url
        ),
        job_id: Some(id),
        job_revision: Some(revision),
        score: evaluation.score,
    }
}
async fn notify_lock(tx: &mut Transaction<'_, Postgres>) -> Result<bool> {
    Ok(sqlx::query_scalar("SELECT pg_try_advisory_xact_lock(hashtextextended(current_schema()||':radar:notifications',0))").fetch_one(&mut **tx).await?)
}
async fn cooldown(
    tx: &mut Transaction<'_, Postgres>,
    provider: &str,
    scope: &str,
    until: DateTime<Utc>,
) -> Result<()> {
    sqlx::query("INSERT INTO provider_cooldowns(provider,scope,until_at) VALUES ($1,$2,$3) ON CONFLICT(provider,scope) DO UPDATE SET until_at=greatest(provider_cooldowns.until_at,excluded.until_at)")
        .bind(provider).bind(scope).bind(until).execute(&mut **tx).await?;
    Ok(())
}
struct Claimed {
    id: Uuid,
    owner: Uuid,
    provider: String,
    channel: String,
    message: NotificationMessage,
    attempts: i32,
    config: Config,
}
impl PgRepository {
    pub async fn notification_history(
        &self,
        registry: &BotProviderRegistry,
    ) -> Result<NotificationHistory> {
        let config = self.config().await?;
        let rows=sqlx::query("SELECT d.*,e.kind,e.payload->>'title' AS title FROM notification_deliveries d JOIN outbox_events e ON e.id=d.event_id ORDER BY e.created_at DESC,d.id LIMIT 100").fetch_all(&self.pool).await?;
        Ok(NotificationHistory {
            providers: registry.statuses(&config.effective),
            deliveries: rows
                .iter()
                .map(|r| {
                    Ok(DeliveryView {
                        id: r.try_get("id")?,
                        kind: r.try_get("kind")?,
                        destination: r.try_get("destination")?,
                        provider: r.try_get("provider")?,
                        status: r.try_get("status")?,
                        attempts: r.try_get("attempts")?,
                        next_attempt_at: r.try_get("next_attempt_at")?,
                        delivered_at: r.try_get("delivered_at")?,
                        remote_id: r.try_get("remote_id")?,
                        error: r.try_get("error")?,
                        title: r.try_get("title")?,
                    })
                })
                .collect::<Result<_>>()?,
        })
    }
    pub async fn queue_digest(&self, id: Uuid, now: DateTime<Utc>) -> Result<Uuid> {
        let config = self.config().await?;
        let mut tx = self.begin("queue_digest").await?;
        let message = self.digest_message(&mut tx, &config).await?;
        let event = enqueue_event(
            &mut tx,
            &config.effective,
            &format!("digest:{id}"),
            EventKind::DailyDigest,
            message,
            now,
        )
        .await?;
        crate::database::commit(tx, &self.pool, "queue_digest").await?;
        Ok(event)
    }
    async fn digest_message(
        &self,
        connection: &mut sqlx::PgConnection,
        config: &ConfigView,
    ) -> Result<NotificationMessage> {
        let max = config
            .effective
            .notifications
            .runtime
            .as_ref()
            .map_or(5, |s| s.max_digest_jobs);
        let jobs = self
            .jobs_on(
                connection,
                JobQuery {
                    verdict: Some(Verdict::ApplyCandidate),
                    page_size: Some(100),
                    ..Default::default()
                },
            )
            .await?;
        let body = jobs
            .items
            .iter()
            .filter(|j| {
                notification_ready(&j.job)
                    && !j
                        .decision
                        .is_some_and(FeedbackAction::excludes_recommendations)
                    && matches!(
                        j.application_status,
                        ApplicationStatus::None | ApplicationStatus::Drafted
                    )
            })
            .take(max as usize)
            .map(|j| {
                format!(
                    "{} — score {}\n{}",
                    j.job.title,
                    j.evaluation.score.unwrap_or_default(),
                    j.job.url
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");
        let message = NotificationMessage {
            title: "Radar daily digest".into(),
            body: if body.is_empty() {
                "No current application candidates.".into()
            } else {
                body
            },
            job_id: None,
            job_revision: None,
            score: None,
        };
        Ok(message)
    }
    pub async fn plan_digest(&self, now: DateTime<Utc>) -> Result<()> {
        let config = self.config().await?;
        if !config.effective.notifications.notifications.enabled
            || config.effective.radar.app.active_mode == Mode::Paused
        {
            return Ok(());
        }
        let Some(settings) = &config.effective.radar.scheduler else {
            return Ok(());
        };
        let zone: chrono_tz::Tz = config
            .effective
            .notifications
            .notifications
            .timezone
            .parse()
            .expect("validated timezone");
        let mut tx = self.begin("plan_digest").await?;
        if !notify_lock(&mut tx).await? {
            return Ok(());
        }
        let previous: Option<DateTime<Utc>> = sqlx::query_scalar(
            "SELECT planned_through FROM digest_state WHERE singleton=true FOR UPDATE",
        )
        .fetch_one(&mut *tx)
        .await?;
        let floor = now - chrono::Duration::minutes(i64::from(settings.catch_up_minutes));
        let days = [
            radar_application::config::Weekday::Mon,
            radar_application::config::Weekday::Tue,
            radar_application::config::Weekday::Wed,
            radar_application::config::Weekday::Thu,
            radar_application::config::Weekday::Fri,
            radar_application::config::Weekday::Sat,
            radar_application::config::Weekday::Sun,
        ];
        let due = slots(
            &days,
            std::slice::from_ref(&config.effective.notifications.routes.daily_digest.at),
            zone,
            previous.unwrap_or(floor).max(floor),
            now,
        );
        if let Some(at) = due.last() {
            let message = self.digest_message(&mut tx, &config).await?;
            // Queue and cursor advance share one transaction and connection.
            let key = format!(
                "scheduled-digest:{}:{}",
                zone,
                at.with_timezone(&zone).date_naive()
            );
            let id = Uuid::new_v5(&Uuid::NAMESPACE_OID, key.as_bytes());
            // The event key makes this safe if the process dies before advancing the cursor.
            enqueue_event(
                &mut tx,
                &config.effective,
                &format!("digest:{id}"),
                EventKind::DailyDigest,
                message,
                now,
            )
            .await?;
        }
        sqlx::query("UPDATE digest_state SET planned_through=$1 WHERE singleton=true")
            .bind(now)
            .execute(&mut *tx)
            .await?;
        crate::database::commit(tx, &self.pool, "plan_digest").await?;
        Ok(())
    }
    async fn claim_delivery(
        &self,
        registry: &BotProviderRegistry,
        now: DateTime<Utc>,
    ) -> Result<Option<Claimed>> {
        let mut tx = self.begin("claim_delivery").await?;
        if !notify_lock(&mut tx).await? {
            return Ok(None);
        }
        // A crashed POST may have succeeded externally. Never blindly resend it after a lease expires.
        sqlx::query("UPDATE notification_deliveries SET status='uncertain',error='worker interrupted during external delivery; manual review required',lease_until=NULL WHERE status='sending' AND lease_until<=$1").bind(now).execute(&mut *tx).await?;
        let config = self.config_in(&mut tx).await?.effective;
        let Some(runtime) = &config.notifications.runtime else {
            crate::database::commit(tx, &self.pool, "claim_delivery").await?;
            return Ok(None);
        };
        let rows=sqlx::query("SELECT d.*,e.kind,e.payload FROM notification_deliveries d JOIN outbox_events e ON e.id=d.event_id WHERE d.status IN ('pending','retry') AND d.next_attempt_at<=$1 ORDER BY d.next_attempt_at,d.id LIMIT 100 FOR UPDATE OF d SKIP LOCKED").bind(now).fetch_all(&mut *tx).await?;
        for row in rows {
            let id: Uuid = row.try_get("id")?;
            let kind: EventKind = enum_value(row.try_get("kind")?)?;
            let mut message: Json<NotificationMessage> = row.try_get("payload")?;
            let provider: String = row.try_get("provider")?;
            let destination: String = row.try_get("destination")?;
            let critical = kind == EventKind::ScanFailed
                && matches!(
                    config.notifications.routes.scan_failed.severity,
                    Severity::Critical
                );
            let defer_reason = if !config.notifications.notifications.enabled {
                Some("notifications disabled")
            } else if config.radar.app.active_mode == Mode::Paused && kind != EventKind::ScanFailed
            {
                Some("work mode paused")
            } else if registry.notifier(&config, &provider).is_err() {
                Some("provider not configured; inspect provider status")
            } else {
                None
            };
            if let Some(reason) = defer_reason {
                sqlx::query(
                    "UPDATE notification_deliveries SET error=$2,next_attempt_at=$3 WHERE id=$1",
                )
                .bind(id)
                .bind(reason)
                .bind(
                    now + chrono::Duration::seconds(
                        config
                            .radar
                            .scheduler
                            .as_ref()
                            .map_or(2, |s| s.poll_interval_seconds) as i64,
                    ),
                )
                .execute(&mut *tx)
                .await?;
                continue;
            }
            if let Some(job_id) = message.0.job_id {
                let current = self.job_in(&mut tx, job_id).await?;
                if current.evaluation.config_revision != config.revision {
                    sqlx::query("UPDATE notification_deliveries SET error='waiting for evaluation under current config',next_attempt_at=$2 WHERE id=$1").bind(id).bind(now+chrono::Duration::seconds(2)).execute(&mut *tx).await?;
                    continue;
                }
                if !notification_ready(&current.job)
                    || current.evaluation.verdict != Verdict::ApplyCandidate
                    || current
                        .decision
                        .is_some_and(FeedbackAction::excludes_recommendations)
                    || !matches!(
                        current.application_status,
                        ApplicationStatus::None | ApplicationStatus::Drafted
                    )
                {
                    sqlx::query("UPDATE notification_deliveries SET status='cancelled',error='job is no longer an application candidate' WHERE id=$1").bind(id).execute(&mut *tx).await?;
                    continue;
                }
                message.0 = job_message(
                    current.id,
                    current.revision,
                    &current.job,
                    &current.evaluation,
                );
            }
            if !NotificationRouter::destinations(&config, kind, message.0.score)
                .contains(&destination)
            {
                sqlx::query("UPDATE notification_deliveries SET error='route or threshold inactive',next_attempt_at=$2 WHERE id=$1").bind(id).bind(now+chrono::Duration::minutes(1)).execute(&mut *tx).await?;
                continue;
            }
            if !critical && in_quiet_hours(&config, now) {
                sqlx::query("UPDATE notification_deliveries SET next_attempt_at=$2,error='quiet hours' WHERE id=$1").bind(id).bind(after_quiet_hours(&config,now)).execute(&mut *tx).await?;
                continue;
            }
            let Some(target) = config
                .notifications
                .destinations
                .get(&destination)
                .filter(|d| d.provider == provider)
            else {
                continue;
            };
            let Some(channel) = registry
                .secrets
                .get(&target.channel_id_env)
                .filter(|c| snowflake(c))
            else {
                sqlx::query("UPDATE notification_deliveries SET error='destination channel environment reference missing or invalid',next_attempt_at=$2 WHERE id=$1").bind(id).bind(now+chrono::Duration::minutes(1)).execute(&mut *tx).await?;
                continue;
            };
            let blocked:Option<DateTime<Utc>>=sqlx::query_scalar("SELECT max(until_at) FROM provider_cooldowns WHERE provider=$1 AND scope IN ('*',$2) AND until_at>$3").bind(&provider).bind(&channel).bind(now).fetch_one(&mut *tx).await?;
            if let Some(until) = blocked {
                sqlx::query("UPDATE notification_deliveries SET next_attempt_at=$2,error='provider cooldown' WHERE id=$1").bind(id).bind(until).execute(&mut *tx).await?;
                continue;
            }
            let zone: chrono_tz::Tz = config
                .notifications
                .notifications
                .timezone
                .parse()
                .expect("validated timezone");
            let date = now.with_timezone(&zone).date_naive();
            if !critical
                && row.try_get::<Option<chrono::NaiveDate>, _>("budget_date")? != Some(date)
            {
                sqlx::query("INSERT INTO notification_budget(local_date,reserved) VALUES ($1,0) ON CONFLICT DO NOTHING").bind(date).execute(&mut *tx).await?;
                let reserved:Option<i32>=sqlx::query_scalar("UPDATE notification_budget SET reserved=reserved+1 WHERE local_date=$1 AND reserved<$2 RETURNING reserved").bind(date).bind(i64::from(config.notifications.notifications.max_alerts_per_day)).fetch_optional(&mut *tx).await?;
                if reserved.is_none() {
                    sqlx::query("UPDATE notification_deliveries SET error='daily notification cap reached',next_attempt_at=$2 WHERE id=$1").bind(id).bind(now+chrono::Duration::minutes(1)).execute(&mut *tx).await?;
                    continue;
                }
            }
            let owner = Uuid::new_v4();
            let attempts = row.try_get::<i32, _>("attempts")? + 1;
            sqlx::query("UPDATE notification_deliveries SET status='sending',attempts=$2,owner=$3,lease_until=$4,budget_date=$5,channel_id=$6,message=$7,error=NULL WHERE id=$1")
                .bind(id).bind(attempts).bind(owner).bind(now+chrono::Duration::seconds(runtime.delivery_lease_seconds as i64)).bind(date).bind(&channel).bind(Json(&message.0)).execute(&mut *tx).await?;
            crate::database::commit(tx, &self.pool, "claim_delivery").await?;
            return Ok(Some(Claimed {
                id,
                owner,
                provider,
                channel,
                message: message.0,
                attempts,
                config,
            }));
        }
        crate::database::commit(tx, &self.pool, "claim_delivery").await?;
        Ok(None)
    }
    pub async fn deliver_one(
        &self,
        registry: &BotProviderRegistry,
        now: DateTime<Utc>,
    ) -> Result<bool> {
        let Some(claim) = self.claim_delivery(registry, now).await? else {
            return Ok(false);
        };
        let result = match registry.notifier(&claim.config, &claim.provider) {
            Ok(provider) => {
                provider
                    .deliver(
                        &DeliveryTarget::Channel(claim.channel.clone()),
                        &claim.message,
                        claim.id,
                    )
                    .await
            }
            Err(_) => Err(DeliveryFailure::new(
                FailureKind::Retryable,
                "provider_configuration_changed",
            )),
        };
        let mut tx = self.begin("deliver_one").await?;
        let current=sqlx::query("SELECT id FROM notification_deliveries WHERE id=$1 AND owner=$2 AND status='sending' FOR UPDATE").bind(claim.id).bind(claim.owner).fetch_optional(&mut *tx).await?;
        if current.is_none() {
            return Ok(true);
        }
        match result {
            Ok(receipt) => {
                sqlx::query("UPDATE notification_deliveries SET status='sent',remote_id=$2,delivered_at=$3,lease_until=NULL,error=NULL WHERE id=$1").bind(claim.id).bind(receipt.remote_id).bind(now).execute(&mut *tx).await?;
                if let Some(seconds) = receipt
                    .cooldown_seconds
                    .filter(|s| s.is_finite() && *s > 0.0)
                {
                    cooldown(
                        &mut tx,
                        &claim.provider,
                        &claim.channel,
                        now.checked_add_signed(chrono::Duration::milliseconds(
                            (seconds * 1000.0).ceil() as i64,
                        ))
                        .unwrap_or(DateTime::<Utc>::MAX_UTC),
                    )
                    .await?;
                }
            }
            Err(error) => {
                let retry = error.kind == FailureKind::Retryable
                    && claim.attempts
                        < (claim.config.notifications.notifications.retry.max_attempts as i32);
                let status = if retry {
                    "retry"
                } else if error.kind == FailureKind::Uncertain {
                    "uncertain"
                } else {
                    "failed"
                };
                let next = retry_at(
                    &claim.config,
                    now,
                    claim.attempts,
                    error.retry_after_seconds,
                );
                let limited = error.global
                    || error.retry_after_seconds.is_some()
                    || error.code == "discord_rate_limited";
                sqlx::query("UPDATE notification_deliveries SET status=$2,error=$3,next_attempt_at=$4,lease_until=NULL WHERE id=$1").bind(claim.id).bind(status).bind(error.code).bind(next).execute(&mut *tx).await?;
                if limited {
                    cooldown(
                        &mut tx,
                        &claim.provider,
                        if error.global { "*" } else { &claim.channel },
                        next,
                    )
                    .await?;
                }
            }
        }
        sqlx::query("UPDATE outbox_events e SET processed_at=$1 WHERE processed_at IS NULL AND EXISTS(SELECT 1 FROM notification_deliveries d WHERE d.event_id=e.id) AND NOT EXISTS(SELECT 1 FROM notification_deliveries d WHERE d.event_id=e.id AND d.status IN ('pending','retry','sending'))").bind(now).execute(&mut *tx).await?;
        crate::database::commit(tx, &self.pool, "deliver_one").await?;
        Ok(true)
    }
}
