use super::*;
use radar_application::{AiProvider, ai::*};
use std::sync::Arc;

fn kind_name(kind: TaskKind) -> &'static str {
    match kind {
        TaskKind::Evaluate => "evaluate",
        TaskKind::Proposal => "proposal",
    }
}
fn invalid(message: &str) -> StoreError {
    StoreError::Invalid(message.into())
}
impl PgRepository {
    pub async fn recover_ai(&self) -> Result<()> {
        sqlx::query("UPDATE ai_operations SET status='failed',error='interrupted_or_expired; explicit retry with a new key required',completed_at=now() WHERE status='running' AND lease_until<now()")
            .execute(&self.pool).await?;
        Ok(())
    }
    pub async fn ai_history(&self, job_id: Uuid, profile: &Profile) -> Result<AiHistory> {
        let job = self.job(job_id).await?;
        let config = self.config().await?;
        let hash = job_hash(&job).map_err(|_| invalid("job fingerprint failed"))?;
        let rows = sqlx::query(
            "SELECT * FROM ai_operations WHERE job_id=$1 ORDER BY created_at DESC,id LIMIT 50",
        )
        .bind(job_id)
        .fetch_all(&self.pool)
        .await?;
        let mut operations = vec![];
        for row in rows {
            let Json(mut operation): Json<AiOperation> = row.try_get("snapshot")?;
            operation.status = row.try_get("status")?;
            operation.error = row.try_get("error")?;
            if operation.status == "running"
                && row.try_get::<DateTime<Utc>, _>("lease_until")? < Utc::now()
            {
                operation.status = "failed".into();
                operation.error =
                    Some("interrupted_or_expired; explicit retry with a new key required".into());
            }
            operation.approved = row.try_get("approved")?;
            operation.output = row
                .try_get::<Option<Json<Value>>, _>("output")?
                .map(|v| v.0);
            operation.usage = serde_json::from_value(row.try_get::<Json<Value>, _>("usage")?.0)?;
            operation.stale = operation.job_hash != hash
                || operation.evidence.profile_revision != profile.revision
                || operation.config_revision != config.revision;
            operations.push(operation);
        }
        let rows = sqlx::query(
            "SELECT * FROM proposal_revisions WHERE job_id=$1 ORDER BY revision DESC LIMIT 100",
        )
        .bind(job_id)
        .fetch_all(&self.pool)
        .await?;
        let drafts = rows
            .into_iter()
            .map(|r| {
                Ok(ProposalRevision {
                    id: r.try_get("id")?,
                    job_id,
                    operation_id: r.try_get("operation_id")?,
                    revision: r.try_get("revision")?,
                    ai_text: r.try_get("ai_text")?,
                    user_text: r.try_get("user_text")?,
                    approved_text: r.try_get("approved_text")?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(AiHistory { operations, drafts })
    }
    pub async fn begin_ai(
        &self,
        job_id: Uuid,
        kind: TaskKind,
        request: AiRequest,
        profile: &Profile,
    ) -> Result<(AiOperation, Option<(AiInput, Config)>)> {
        self.recover_ai().await?;
        let config = self.config().await?;
        let ai = &config.effective.radar.ai;
        let runtime = ai
            .runtime
            .as_ref()
            .filter(|r| r.enabled)
            .ok_or_else(|| invalid("AI disabled; explicitly configure a provider"))?;
        let request_hash = fingerprint(&(job_id, kind, &request))?;
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT pg_advisory_xact_lock(5732035)")
            .execute(&mut *tx)
            .await?;
        if let Some(row) = sqlx::query("SELECT job_id,request_hash FROM ai_operations WHERE id=$1")
            .bind(request.idempotency_key)
            .fetch_optional(&mut *tx)
            .await?
        {
            if row.try_get::<String, _>("request_hash")? != request_hash {
                return Err(StoreError::Conflict);
            }
            tx.commit().await?;
            let operation = self
                .ai_history(job_id, profile)
                .await?
                .operations
                .into_iter()
                .find(|o| o.id == request.idempotency_key)
                .ok_or(StoreError::NotFound)?;
            return Ok((operation, None));
        }
        if kind == TaskKind::Evaluate && request.template_id.is_some() {
            return Err(invalid(
                "proposal template applies only to proposal generation",
            ));
        }
        let proposal_template = if kind == TaskKind::Proposal {
            match &config.effective.radar.proposals.templates {
                Some(catalog) => catalog
                    .select(request.template_id.as_deref())
                    .map_err(|_| invalid("proposal template unavailable"))?,
                None if request.template_id.is_some() => {
                    return Err(invalid("proposal templates not configured"));
                }
                None => None,
            }
        } else {
            None
        };
        let job = self.job(job_id).await?;
        if request.expected_revision != job.revision {
            return Err(StoreError::Conflict);
        }
        let snapshot = evidence(profile, config.effective.settings().weekly_hours)
            .map_err(|_| invalid("invalid verified evidence snapshot"))?;
        if snapshot.facts.is_empty() {
            return Err(invalid("verified evidence is required"));
        }
        let mut approved_analysis = None;
        if kind == TaskKind::Proposal {
            if !snapshot
                .facts
                .iter()
                .any(|fact| fact.verified && fact.shareable)
            {
                return Err(invalid(
                    "shareable verified evidence required before proposal generation",
                ));
            }
            if job.evaluation.verdict == radar_domain::Verdict::Skip {
                return Err(invalid(
                    "deterministic veto/skip remains authoritative; proposal blocked",
                ));
            }
            let id = request
                .evaluation_id
                .ok_or_else(|| invalid("approved evaluation ID required"))?;
            let analysis = self
                .ai_history(job_id, profile)
                .await?
                .operations
                .into_iter()
                .find(|o| {
                    o.id == id
                        && o.approved
                        && !o.stale
                        && o.kind == TaskKind::Evaluate
                        && o.status == "completed"
                })
                .ok_or_else(|| invalid("current approved evaluation required"))?;
            approved_analysis = Some(serde_json::from_value(
                analysis
                    .output
                    .ok_or_else(|| invalid("evaluation output missing"))?,
            )?);
        }
        let feedback = self
            .feedback_history(FeedbackQuery {
                job_id: Some(job_id),
                ..Default::default()
            })
            .await?
            .items
            .into_iter()
            .take(10)
            .map(|f| format!("{:?}: {}", f.action, f.reason.unwrap_or_default()))
            .collect();
        let mut input = AiInput {
            proposal_template: proposal_template.clone(),
            rubric: runtime.evaluation_rubric.clone(),
            kind,
            job: job.clone(),
            evidence: snapshot.clone(),
            approved_analysis,
            feedback,
            max_proposal_words: proposal_template
                .as_ref()
                .map_or(runtime.max_proposal_words, |t| {
                    t.definition.max_words.min(runtime.max_proposal_words)
                }),
        };
        if let Some(template) = &input.proposal_template
            && radar_application::proposal_templates::relevant_evidence(
                &input.job.job,
                &input.evidence.facts,
                template.definition.max_evidence,
            )
            .is_empty()
        {
            return Err(invalid(
                "no relevant shareable evidence for this template; review profile evidence first",
            ));
        }
        input.job.evaluation.evidence.clear(); // Raw identity proof may contain confidential employer details.
        if serde_json::to_vec(&input)?.len() > runtime.max_input_bytes {
            return Err(invalid("AI input exceeds configured bound"));
        }
        let active: i64 =
            sqlx::query_scalar("SELECT count(*) FROM ai_operations WHERE status='running'")
                .fetch_one(&mut *tx)
                .await?;
        if active >= ai.max_concurrent_tasks as i64 {
            return Err(invalid("AI concurrency limit reached"));
        }
        let today=sqlx::query("SELECT count(*) AS total,count(*) FILTER(WHERE kind=$1) AS tasks,coalesce(sum(reserved_cost),0)::float8 AS cost FROM ai_operations WHERE (created_at AT TIME ZONE $2)::date=(now() AT TIME ZONE $2)::date")
            .bind(kind_name(kind)).bind(&config.effective.radar.app.timezone).fetch_one(&mut *tx).await?;
        let task_limit = match kind {
            TaskKind::Evaluate => runtime
                .max_evaluations_per_day
                .min(config.effective.work_mode().max_deep_evaluations_per_day),
            TaskKind::Proposal => runtime.max_proposals_per_day,
        };
        let reservation = if ai.provider == "mock" {
            0.0
        } else {
            runtime.reserved_cost_usd_per_call
        };
        if today.try_get::<i64, _>("total")? >= ai.max_tasks_per_day as i64
            || today.try_get::<i64, _>("tasks")? >= task_limit as i64
            || today.try_get::<f64, _>("cost")? + reservation > ai.max_estimated_cost_usd_per_day
        {
            return Err(invalid("AI daily task/cost reservation limit reached"));
        }
        let operation = AiOperation {
            proposal_template,
            id: request.idempotency_key,
            job_id,
            kind,
            status: "running".into(),
            error: None,
            provider: ai.provider.clone(),
            model: if ai.provider == "mock" {
                "mock-deterministic-1".into()
            } else {
                runtime.model.clone()
            },
            prompt_version: PROMPT_VERSION.into(),
            evidence: snapshot,
            job_hash: job_hash(&job).map_err(|_| invalid("job fingerprint failed"))?,
            job_revision: job.revision,
            config_revision: config.revision,
            stale: false,
            output: None,
            usage: Usage::default(),
            approved: false,
        };
        sqlx::query("INSERT INTO ai_operations(id,job_id,kind,request_hash,status,snapshot,reserved_cost,lease_until) VALUES($1,$2,$3,$4,'running',$5,$6,now()+make_interval(secs=>$7))")
            .bind(operation.id).bind(job_id).bind(kind_name(kind)).bind(request_hash).bind(Json(&operation)).bind(reservation).bind(ai.timeout_seconds as f64+60.0).execute(&mut *tx).await?;
        audit(
            &mut tx,
            "owner",
            "ai_requested",
            "job",
            json!({"job_id":job_id,"operation_id":operation.id,"kind":kind}),
        )
        .await?;
        tx.commit().await?;
        Ok((operation, Some((input, config.effective))))
    }
    pub async fn execute_ai(
        &self,
        operation: AiOperation,
        input: AiInput,
        config: Config,
        provider: Arc<dyn AiProvider>,
    ) -> Result<()> {
        let runtime = config
            .radar
            .ai
            .runtime
            .as_ref()
            .ok_or_else(|| invalid("AI runtime missing"))?;
        let cancelled = async {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                let status: Option<String> =
                    sqlx::query_scalar("SELECT status FROM ai_operations WHERE id=$1")
                        .bind(operation.id)
                        .fetch_optional(&self.pool)
                        .await?;
                if status.as_deref() != Some("running") {
                    return Ok::<(), StoreError>(());
                }
            }
        };
        let mut failure_usage = Usage::default();
        let result = tokio::select! {
            result=tokio::time::timeout(std::time::Duration::from_secs(config.radar.ai.timeout_seconds as u64),provider.infer(runtime,&input))=>match result {Ok(Ok(reply))=>Ok(reply),Ok(Err(error))=>{
                if let Some(error)=error.downcast_ref::<crate::ai::ProviderFailure>() {failure_usage=error.usage.clone();Err(error.category)} else {Err("provider_failure")}
            },Err(_)=>Err("provider_timeout")},
            _=cancelled=>return Ok(()),
        };
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query("SELECT status FROM ai_operations WHERE id=$1 FOR UPDATE")
            .bind(operation.id)
            .fetch_one(&mut *tx)
            .await?;
        if row.try_get::<String, _>("status")? != "running" {
            return Ok(());
        }
        let usage = result
            .as_ref()
            .ok()
            .map(|r| r.usage.clone())
            .unwrap_or(failure_usage);
        let reply = result.and_then(|mut reply| {
            let encoded = serde_json::to_vec(&reply.output).map_err(|_| "invalid_model_output")?;
            if encoded.len() > runtime.max_output_bytes {
                return Err("model_output_too_large");
            }
            if !jsonschema::validator_for(&output_schema(operation.kind))
                .map_err(|_| "invalid_output_schema")?
                .is_valid(&reply.output)
            {
                return Err("invalid_model_output");
            }
            reply.output = validate_output(&input, reply.output.clone())
                .map_err(|_| "unsupported_claim_or_invalid_evidence")?;
            Ok(reply)
        });
        match reply {
            Ok(reply) => {
                let current = self.job(operation.job_id).await?;
                let current_config = self.config().await?;
                if job_hash(&current).map_err(|_| invalid("job fingerprint failed"))?
                    != operation.job_hash
                    || current_config.revision != operation.config_revision
                {
                    sqlx::query("UPDATE ai_operations SET status='failed',error='stale_inputs',completed_at=now(),usage=$2 WHERE id=$1").bind(operation.id).bind(Json(reply.usage)).execute(&mut *tx).await?;
                } else {
                    if operation.kind == TaskKind::Proposal {
                        let proposal: Proposal = serde_json::from_value(reply.output.clone())?;
                        sqlx::query("SELECT pg_advisory_xact_lock(5732035)")
                            .execute(&mut *tx)
                            .await?;
                        sqlx::query("INSERT INTO proposal_revisions(job_id,operation_id,revision,ai_text,user_text) SELECT $1,$2,coalesce(max(revision),0)+1,$3,(SELECT user_text FROM proposal_revisions WHERE job_id=$1 ORDER BY revision DESC LIMIT 1) FROM proposal_revisions WHERE job_id=$1")
                            .bind(operation.job_id).bind(operation.id).bind(proposal.proposal_text).execute(&mut *tx).await?;
                    }
                    sqlx::query("UPDATE ai_operations SET status='completed',output=$2,usage=$3,completed_at=now() WHERE id=$1").bind(operation.id).bind(Json(reply.output)).bind(Json(reply.usage)).execute(&mut *tx).await?;
                }
            }
            Err(category) => {
                sqlx::query("UPDATE ai_operations SET status='failed',error=$2,completed_at=now(),usage=$3 WHERE id=$1").bind(operation.id).bind(category).bind(Json(usage)).execute(&mut *tx).await?;
            }
        }
        audit(
            &mut tx,
            "owner",
            "ai_finished",
            "job",
            json!({"job_id":operation.job_id,"operation_id":operation.id}),
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }
    pub async fn cancel_ai(&self, job_id: Uuid, id: Uuid) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("UPDATE ai_operations SET status='cancelled',error='user_cancelled',completed_at=now() WHERE id=$1 AND job_id=$2 AND status='running'").bind(id).bind(job_id).execute(&mut *tx).await?;
        audit(
            &mut tx,
            "owner",
            "ai_cancel_requested",
            "job",
            json!({"job_id":job_id,"operation_id":id}),
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }
    pub async fn approve_ai(
        &self,
        job_id: Uuid,
        id: Uuid,
        revision: i64,
        profile: &Profile,
    ) -> Result<()> {
        let current = self.job(job_id).await?;
        if current.revision != revision {
            return Err(StoreError::Conflict);
        }
        let op = self
            .ai_history(job_id, profile)
            .await?
            .operations
            .into_iter()
            .find(|o| {
                o.id == id && o.kind == TaskKind::Evaluate && o.status == "completed" && !o.stale
            })
            .ok_or_else(|| invalid("current completed evaluation required"))?;
        let mut tx = self.pool.begin().await?;
        sqlx::query("UPDATE ai_operations SET approved=true WHERE id=$1")
            .bind(op.id)
            .execute(&mut *tx)
            .await?;
        audit(
            &mut tx,
            "owner",
            "ai_evaluation_approved",
            "job",
            json!({"job_id":job_id,"operation_id":id}),
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }
    pub async fn edit_proposal(
        &self,
        job_id: Uuid,
        request: DraftEdit,
        profile: &Profile,
    ) -> Result<ProposalRevision> {
        let config = self.config().await?;
        let runtime = config
            .effective
            .radar
            .ai
            .runtime
            .as_ref()
            .ok_or_else(|| invalid("AI settings missing"))?;
        if request.text.trim().is_empty() || request.text.len() > runtime.max_output_bytes {
            return Err(invalid("invalid draft text size"));
        }
        let history = self.ai_history(job_id, profile).await?;
        let draft = history.drafts.first().ok_or(StoreError::NotFound)?;
        if request.expected_revision != draft.revision {
            return Err(StoreError::Conflict);
        }
        if request.approve
            && !history
                .operations
                .iter()
                .any(|o| o.id == draft.operation_id && !o.stale && o.status == "completed")
        {
            return Err(invalid("stale proposal cannot be approved"));
        }
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT pg_advisory_xact_lock(5732035)")
            .execute(&mut *tx)
            .await?;
        let latest: i64 =
            sqlx::query_scalar("SELECT max(revision) FROM proposal_revisions WHERE job_id=$1")
                .bind(job_id)
                .fetch_one(&mut *tx)
                .await?;
        if latest != request.expected_revision {
            return Err(StoreError::Conflict);
        }
        let row=sqlx::query("INSERT INTO proposal_revisions(job_id,operation_id,revision,ai_text,user_text,approved_text) VALUES($1,$2,$3,$4,$5,$6) RETURNING id")
            .bind(job_id).bind(draft.operation_id).bind(latest+1).bind(&draft.ai_text).bind(&request.text).bind(request.approve.then_some(&request.text)).fetch_one(&mut *tx).await?;
        let result = ProposalRevision {
            id: row.try_get("id")?,
            job_id,
            operation_id: draft.operation_id,
            revision: latest + 1,
            ai_text: draft.ai_text.clone(),
            user_text: Some(request.text.clone()),
            approved_text: request.approve.then_some(request.text),
        };
        audit(
            &mut tx,
            "owner",
            "proposal_saved",
            "job",
            json!({"job_id":job_id,"revision":result.revision,"approved":request.approve}),
        )
        .await?;
        tx.commit().await?;
        Ok(result)
    }
}
