use super::*;
use crate::{bots::BotProviderRegistry, discord::TokenCipher};
use radar_application::bots::*;
use radar_domain::Verdict;

impl PgRepository {
    /// Only verified requests reach this method. It stores a short acknowledgment, never runs a scan.
    pub async fn accept_interaction(
        &self,
        provider: &str,
        interaction: VerifiedInteraction,
        body_hash: &str,
        config: &Config,
        registry: &BotProviderRegistry,
        now: DateTime<Utc>,
    ) -> Result<Value> {
        let mut tx = self.begin("accept_interaction").await?;
        if let Some(row)=sqlx::query("SELECT body_hash,acknowledgment FROM discord_interactions WHERE provider=$1 AND interaction_id=$2").bind(provider).bind(&interaction.id).fetch_optional(&mut *tx).await? {
            if row.try_get::<String,_>("body_hash")?!=body_hash{return Err(StoreError::Conflict);}
            return Ok(row.try_get::<Json<Value>,_>("acknowledgment")?.0);
        }
        let action = interaction
            .action
            .ok_or_else(|| StoreError::Invalid("interaction action missing".into()))?;
        match &action {
            BotAction::Feedback { delivery_id, .. }
            | BotAction::FeedbackModal { delivery_id, .. } => {
                let row=sqlx::query("SELECT d.remote_id FROM notification_deliveries d WHERE d.id=$1 AND d.provider=$2 AND d.channel_id=$3 AND d.status='sent'")
                    .bind(delivery_id).bind(provider).bind(&interaction.channel_id).fetch_optional(&mut *tx).await?.ok_or_else(||StoreError::Invalid("unknown or unconfirmed notification delivery".into()))?;
                if let Some(message) = &interaction.message_id {
                    if row.try_get::<Option<String>, _>("remote_id")?.as_ref() != Some(message) {
                        return Err(StoreError::Invalid("notification message mismatch".into()));
                    }
                } else {
                    let exists:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM discord_interactions WHERE provider=$1 AND user_id=$2 AND channel_id=$3 AND command->>'command'='feedback_modal' AND command->>'delivery_id'=$4 AND expires_at>$5)")
                        .bind(provider).bind(&interaction.user_id).bind(&interaction.channel_id).bind(delivery_id.to_string()).bind(now).fetch_one(&mut *tx).await?;
                    if !exists {
                        return Err(StoreError::Invalid(
                            "feedback modal has no authenticated origin".into(),
                        ));
                    }
                }
            }
            _ => {}
        }
        let modal = matches!(action, BotAction::FeedbackModal { .. });
        let acknowledgment = if let BotAction::FeedbackModal { delivery_id, .. } = &action {
            json!({"type":9,"data":{"title":"Why skip this job?","custom_id":format!("radar-feedback:{delivery_id}:skip"),"components":[{"type":1,"components":[{"type":4,"custom_id":"reason","label":"Reason (optional)","style":2,"required":false,"max_length":500}]}]}})
        } else {
            json!({"type":5,"data":{"flags":64}})
        };
        let encrypted = if modal {
            None
        } else {
            Some(
                TokenCipher::from_config(config, registry.secrets.as_ref())
                    .map_err(StoreError::Invalid)?
                    .seal(provider, &interaction.id, &interaction.token)
                    .map_err(StoreError::Invalid)?,
            )
        };
        let runtime = config
            .notifications
            .runtime
            .as_ref()
            .ok_or_else(|| StoreError::Invalid("bot settings missing".into()))?;
        sqlx::query("INSERT INTO discord_interactions(provider,interaction_id,body_hash,application_id,user_id,guild_id,channel_id,message_id,command,token_encrypted,acknowledgment,status,reply_status,reply_next_at,received_at,expires_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$14,$15) ON CONFLICT DO NOTHING")
            .bind(provider).bind(&interaction.id).bind(body_hash).bind(interaction.application_id).bind(interaction.user_id).bind(interaction.guild_id).bind(interaction.channel_id).bind(interaction.message_id).bind(Json(action)).bind(encrypted).bind(Json(&acknowledgment))
            .bind(if modal{"completed"}else{"pending"}).bind(if modal{"none"}else{"pending"}).bind(now).bind(now+chrono::Duration::seconds(runtime.interaction_ttl_seconds)).execute(&mut *tx).await?;
        // Resolve concurrent retries against the same receipt after the uniqueness wait.
        let row=sqlx::query("SELECT body_hash,acknowledgment FROM discord_interactions WHERE provider=$1 AND interaction_id=$2").bind(provider).bind(&interaction.id).fetch_one(&mut *tx).await?;
        if row.try_get::<String, _>("body_hash")? != body_hash {
            return Err(StoreError::Conflict);
        }
        let response = row.try_get::<Json<Value>, _>("acknowledgment")?.0;
        crate::database::commit(tx, &self.pool, "accept_interaction").await?;
        Ok(response)
    }
    pub async fn process_interaction(
        &self,
        registry: &BotProviderRegistry,
        now: DateTime<Utc>,
    ) -> Result<bool> {
        let config = self.config().await?.effective;
        let Some(runtime) = &config.notifications.runtime else {
            return Ok(false);
        };
        let mut tx = self.begin("process_interaction").await?;
        sqlx::query("UPDATE discord_interactions SET status=CASE WHEN status IN ('pending','running') THEN 'failed' ELSE status END,reply_status=CASE WHEN reply_status IN ('pending','sending') THEN 'failed' ELSE reply_status END,token_encrypted=NULL,error='interaction token expired' WHERE expires_at<=$1 AND token_encrypted IS NOT NULL")
            .bind(now).execute(&mut *tx).await?;
        let row=sqlx::query("SELECT * FROM discord_interactions WHERE (status='pending' OR (status='running' AND lease_until<=$1)) AND expires_at>$1 ORDER BY received_at,interaction_id LIMIT 1 FOR UPDATE SKIP LOCKED")
            .bind(now).fetch_optional(&mut *tx).await?;
        let Some(row) = row else {
            crate::database::commit(tx, &self.pool, "process_interaction").await?;
            return Ok(false);
        };
        let provider: String = row.try_get("provider")?;
        let id: String = row.try_get("interaction_id")?;
        let owner = Uuid::new_v4();
        sqlx::query("UPDATE discord_interactions SET status='running',owner=$3,lease_until=$4 WHERE provider=$1 AND interaction_id=$2").bind(&provider).bind(&id).bind(owner).bind(now+chrono::Duration::seconds(runtime.delivery_lease_seconds as i64)).execute(&mut *tx).await?;
        crate::database::commit(tx, &self.pool, "process_interaction").await?;
        let actor = format!(
            "discord:{provider}:{}",
            row.try_get::<String, _>("user_id")?
        );
        let action: Json<BotAction> = row.try_get("command")?;
        let result = if registry.verifier(&config, &provider).is_ok_and(|v| {
            v.authorize(
                &row.get::<String, _>("user_id"),
                &row.get::<String, _>("guild_id"),
                &row.get::<String, _>("channel_id"),
            )
        }) {
            self.perform_bot_action(
                &provider,
                &id,
                &actor,
                &row.try_get::<String, _>("user_id")?,
                action.0,
                now,
            )
            .await
        } else {
            Err(StoreError::Invalid(
                "Discord authorization changed; no action executed".into(),
            ))
        };
        let (status,message)=match result {Ok(message)=>("completed",message),Err(StoreError::Conflict)=>("failed","State changed since this notification. Refresh in the terminal before making another decision.".into()),Err(StoreError::Invalid(reason))=>("failed",reason),Err(_)=>("failed","Command failed. Check the terminal/server state before retrying.".into())};
        let response = NotificationMessage {
            title: "Radar command result".into(),
            body: message,
            job_id: None,
            job_revision: None,
            score: None,
        };
        sqlx::query("UPDATE discord_interactions SET status=$4,result=$5,completed_at=$6,lease_until=NULL WHERE provider=$1 AND interaction_id=$2 AND owner=$3 AND status='running'")
            .bind(provider).bind(id).bind(owner).bind(status).bind(Json(response)).bind(now).execute(&self.pool).await?;
        Ok(true)
    }
    async fn perform_bot_action(
        &self,
        provider: &str,
        interaction_id: &str,
        actor: &str,
        user: &str,
        action: BotAction,
        now: DateTime<Utc>,
    ) -> Result<String> {
        let operation = Uuid::new_v5(
            &Uuid::NAMESPACE_OID,
            format!("discord:{provider}:{interaction_id}").as_bytes(),
        );
        match action {
            BotAction::Status => {
                let dashboard = self.dashboard().await?;
                let scheduler = self.scheduler_status(now).await?;
                Ok(format!(
                    "Mode: {:?}\nJobs: {} | Candidates: {} | Feedback: {}\nNext scan: {}\nUpwork: disabled",
                    dashboard.mode,
                    dashboard.jobs,
                    dashboard.candidates,
                    dashboard.feedback,
                    scheduler
                        .next_scan
                        .map(|n| n.to_rfc3339())
                        .unwrap_or_else(|| "not scheduled".into())
                ))
            }
            BotAction::Jobs => {
                let config = self.config().await?;
                let size = config
                    .effective
                    .notifications
                    .runtime
                    .as_ref()
                    .map_or(5, |s| s.max_digest_jobs);
                let jobs = self
                    .jobs(JobQuery {
                        verdict: Some(Verdict::ApplyCandidate),
                        page_size: Some(size),
                        ..Default::default()
                    })
                    .await?;
                let lines = jobs
                    .items
                    .iter()
                    .filter(|j| {
                        !j.decision
                            .is_some_and(FeedbackAction::excludes_recommendations)
                            && matches!(
                                j.application_status,
                                ApplicationStatus::None | ApplicationStatus::Drafted
                            )
                    })
                    .map(|j| {
                        format!(
                            "{} | score {} | {}\n{}",
                            j.id,
                            j.evaluation.score.unwrap_or_default(),
                            j.job.title,
                            j.job.url
                        )
                    })
                    .collect::<Vec<_>>();
                Ok(if lines.is_empty() {
                    "No current candidates. Queue /scan or review the terminal.".into()
                } else {
                    lines.join("\n\n")
                })
            }
            BotAction::Scan => {
                let run = self.queue_scan(operation, "discord", now).await?;
                Ok(format!(
                    "Mock scan queued: {} ({:?}). No marketplace search or writes.",
                    run.id, run.status
                ))
            }
            BotAction::Pause | BotAction::Resume => {
                let paused = matches!(action, BotAction::Pause);
                let current = self.config().await?;
                let config = self
                    .set_paused(
                        ModeRequest {
                            idempotency_key: operation,
                            expected_revision: current.revision,
                            paused,
                        },
                        actor,
                    )
                    .await?;
                Ok(format!(
                    "Work mode: {:?} (config revision {})",
                    config.effective.radar.app.active_mode, config.revision
                ))
            }
            BotAction::Digest => {
                self.queue_digest(operation, now).await?;
                Ok("Digest queued for configured destinations, subject to quiet hours and notification caps.".into())
            }
            BotAction::Feedback {
                delivery_id,
                action,
                reason,
            } => {
                let row=sqlx::query("SELECT e.payload,d.message FROM notification_deliveries d JOIN outbox_events e ON e.id=d.event_id WHERE d.id=$1 AND d.provider=$2 AND d.status='sent'").bind(delivery_id).bind(provider).fetch_optional(&self.pool).await?.ok_or(StoreError::NotFound)?;
                let message = row
                    .try_get::<Option<Json<NotificationMessage>>, _>("message")?
                    .or(Some(row.try_get("payload")?))
                    .ok_or(StoreError::NotFound)?
                    .0;
                let job = message.job_id.ok_or(StoreError::NotFound)?;
                let revision = message.job_revision.ok_or(StoreError::NotFound)?;
                let key = Uuid::new_v5(
                    &Uuid::NAMESPACE_OID,
                    format!(
                        "discord-feedback:{provider}:{delivery_id}:{user}:{}",
                        name(action)?
                    )
                    .as_bytes(),
                );
                let result = self
                    .feedback(
                        job,
                        FeedbackRequest {
                            reason_code: None,
                            idempotency_key: key,
                            expected_revision: revision,
                            action,
                            reason,
                            confirmed_submission: false,
                            submitted_at: None,
                        },
                        actor,
                    )
                    .await?;
                Ok(format!(
                    "Saved {:?} for job {} (revision {}).",
                    result.action, result.job_id, result.revision
                ))
            }
            BotAction::FeedbackModal { .. } => Ok("Feedback form opened.".into()),
        }
    }
    pub async fn reply_interaction(
        &self,
        registry: &BotProviderRegistry,
        now: DateTime<Utc>,
    ) -> Result<bool> {
        let config = self.config().await?.effective;
        let Some(runtime) = &config.notifications.runtime else {
            return Ok(false);
        };
        let mut tx = self.begin("reply_interaction").await?;
        let row=sqlx::query("SELECT * FROM discord_interactions WHERE status IN ('completed','failed') AND (reply_status='pending' OR (reply_status='sending' AND reply_lease_until<=$1)) AND reply_next_at<=$1 AND expires_at>$1 AND token_encrypted IS NOT NULL ORDER BY received_at,interaction_id LIMIT 1 FOR UPDATE SKIP LOCKED").bind(now).fetch_optional(&mut *tx).await?;
        let Some(row) = row else { return Ok(false) };
        let provider: String = row.try_get("provider")?;
        let id: String = row.try_get("interaction_id")?;
        let owner = Uuid::new_v4();
        let attempts = row.try_get::<i32, _>("reply_attempts")? + 1;
        sqlx::query("UPDATE discord_interactions SET reply_status='sending',reply_attempts=$3,reply_owner=$4,reply_lease_until=$5 WHERE provider=$1 AND interaction_id=$2")
            .bind(&provider).bind(&id).bind(attempts).bind(owner).bind(now+chrono::Duration::seconds(runtime.delivery_lease_seconds as i64)).execute(&mut *tx).await?;
        crate::database::commit(tx, &self.pool, "reply_interaction").await?;
        let result = async {
            let cipher =
                TokenCipher::from_config(&config, registry.secrets.as_ref()).map_err(|_| {
                    DeliveryFailure::new(FailureKind::Permanent, "interaction_key_unavailable")
                })?;
            let token = cipher
                .open(
                    &provider,
                    &id,
                    &row.try_get::<Vec<u8>, _>("token_encrypted").map_err(|_| {
                        DeliveryFailure::new(FailureKind::Permanent, "token_unavailable")
                    })?,
                )
                .map_err(|_| DeliveryFailure::new(FailureKind::Permanent, "token_unavailable"))?;
            let target = DeliveryTarget::Interaction {
                application_id: row.get("application_id"),
                token,
            };
            let message = row.get::<Json<NotificationMessage>, _>("result").0;
            registry
                .notifier(&config, &provider)
                .map_err(|_| DeliveryFailure::new(FailureKind::Retryable, "provider_unavailable"))?
                .deliver(
                    &target,
                    &message,
                    Uuid::new_v5(
                        &Uuid::NAMESPACE_OID,
                        format!("reply:{provider}:{id}").as_bytes(),
                    ),
                )
                .await
        }
        .await;
        let (status, error, next) = match result {
            Ok(_) => ("sent", None, now),
            Err(error) => {
                let retry = error.kind == FailureKind::Retryable
                    && attempts < (config.notifications.notifications.retry.max_attempts as i32);
                (
                    if retry { "pending" } else { "failed" },
                    Some(error.code),
                    retry_at(&config, now, attempts, error.retry_after_seconds),
                )
            }
        };
        sqlx::query("UPDATE discord_interactions SET reply_status=$4,error=$5,reply_next_at=$6,reply_lease_until=NULL,token_encrypted=CASE WHEN $4 IN ('sent','failed') THEN NULL ELSE token_encrypted END WHERE provider=$1 AND interaction_id=$2 AND reply_owner=$3")
            .bind(provider).bind(id).bind(owner).bind(status).bind(error).bind(next).execute(&self.pool).await?;
        Ok(true)
    }
}
