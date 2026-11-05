use super::*;
use radar_application::learning::*;
use radar_domain::ReasonCode;
use std::collections::{BTreeMap, BTreeSet};

fn invalid(message: &str) -> StoreError {
    StoreError::Invalid(message.into())
}
fn legacy_code(reason: Option<&str>) -> Option<ReasonCode> {
    reason.and_then(|r| serde_json::from_value(json!(r)).ok())
}
type RankingContext = (LearningSettings, Option<PreferenceModel>, BTreeSet<String>);
impl PgRepository {
    pub(super) async fn ranking_context(&self) -> Result<Option<RankingContext>> {
        let config = self.config().await?;
        let Some(settings) = config
            .effective
            .scoring
            .personalization
            .runtime
            .clone()
            .filter(|s| s.enabled)
        else {
            return Ok(None);
        };
        let row = sqlx::query("SELECT * FROM preference_state WHERE singleton=true")
            .fetch_one(&self.pool)
            .await?;
        let suppressed: Json<BTreeSet<String>> = row.try_get("suppressed")?;
        let active: Option<Uuid> = row.try_get("active_model")?;
        let model = if let Some(id) = active {
            let row = sqlx::query("SELECT snapshot,rejected FROM preference_models WHERE id=$1")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;
            let Json(model): Json<PreferenceModel> = row.try_get("snapshot")?;
            if !row.try_get::<bool, _>("rejected")?
                && model.feature_version == FEATURE_VERSION
                && model.feedback_revision
                    == sqlx::query_scalar::<_, i64>(
                        "SELECT feedback_revision FROM preference_state WHERE singleton=true",
                    )
                    .fetch_one(&self.pool)
                    .await?
                && model.config_hash
                    == settings
                        .fingerprint(
                            config
                                .effective
                                .scoring
                                .personalization
                                .min_feedback_samples_for_suggestion
                                as usize,
                        )
                        .map_err(|_| invalid("invalid learning configuration"))?
            {
                Some(model)
            } else {
                None
            }
        } else {
            None
        };
        Ok(Some((settings, model, suppressed.0)))
    }
    pub(super) fn apply_personalization(job: &mut JobView, context: &RankingContext) {
        let snapshot = extract(&job.job, &job.evaluation, job.fetched_at, &context.0);
        let excluded = job
            .decision
            .is_some_and(FeedbackAction::excludes_recommendations)
            || !matches!(
                job.application_status,
                ApplicationStatus::None | ApplicationStatus::Drafted
            );
        let model = if excluded { None } else { context.1.as_ref() };
        let mut explanation = explain(&snapshot, model, &context.0, &context.2);
        if excluded {
            explanation.diagnostic =
                "owner-excluded or application already tracked; no promotion".into();
        }
        job.personalization = Some(explanation);
    }
    pub(super) async fn record_learning_feedback(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        job_id: Uuid,
        request: &FeedbackRequest,
        previous_decision: Option<FeedbackAction>,
        previous_status: ApplicationStatus,
    ) -> Result<()> {
        let code = request
            .reason_code
            .or_else(|| legacy_code(request.reason.as_deref()));
        let config = self.config().await?;
        let snapshot = if let Some(settings) = &config.effective.scoring.personalization.runtime {
            let row=sqlx::query("SELECT input,snapshot,created_at FROM evaluations WHERE job_id=$1 ORDER BY created_at DESC,id DESC LIMIT 1").bind(job_id).fetch_one(&mut **tx).await?;
            let job: Json<Job> = row.try_get("input")?;
            let evaluation: Json<radar_domain::Evaluation> = row.try_get("snapshot")?;
            let features = extract(&job.0, &evaluation.0, row.try_get("created_at")?, settings);
            let ranking = if let Some(context) = self.ranking_context().await? {
                Some(explain(
                    &features,
                    if previous_decision.is_some_and(FeedbackAction::excludes_recommendations)
                        || !matches!(
                            previous_status,
                            ApplicationStatus::None | ApplicationStatus::Drafted
                        )
                    {
                        None
                    } else {
                        context.1.as_ref()
                    },
                    &context.0,
                    &context.2,
                ))
            } else {
                None
            };
            Some(json!({"features":features,"ranking":ranking}))
        } else {
            None
        };
        sqlx::query("UPDATE feedback SET reason_code=$2,learning_snapshot=$3 WHERE id=$1")
            .bind(request.idempotency_key)
            .bind(code.map(name).transpose()?)
            .bind(snapshot.map(Json))
            .execute(&mut **tx)
            .await?;
        sqlx::query("INSERT INTO feedback_learning_revisions(feedback_id,version,reason_code,excluded) VALUES($1,1,$2,false)").bind(request.idempotency_key).bind(code.map(name).transpose()?).execute(&mut **tx).await?;
        Ok(())
    }
    async fn learning_events(
        &self,
        settings: &LearningSettings,
        cutoff: DateTime<Utc>,
    ) -> Result<(Vec<LearningEvent>, usize)> {
        // Historical inputs come from the last evaluation that existed when feedback was recorded.
        let window_start = settings
            .training_window_days
            .map(|days| {
                cutoff
                    .checked_sub_signed(chrono::Duration::days(days as i64))
                    .ok_or_else(|| invalid("invalid training window"))
            })
            .transpose()?;
        let rows=sqlx::query("SELECT f.id,f.job_id,f.action,f.actor,f.reason,f.application_status,f.created_at,f.learning_snapshot,r.version,r.reason_code,r.excluded,r.actual_effort_hours,r.created_at AS available_at,e.input-'description' AS input,e.snapshot,e.created_at AS observed_at FROM feedback f JOIN feedback_learning_revisions r ON r.feedback_id=f.id LEFT JOIN LATERAL(SELECT input,snapshot,created_at FROM evaluations WHERE job_id=f.job_id AND created_at<=f.created_at ORDER BY created_at DESC,id DESC LIMIT 1)e ON true WHERE f.created_at<=$1 AND r.created_at<=$1 AND ($3::timestamptz IS NULL OR f.created_at>=$3) ORDER BY f.created_at,f.id,r.version LIMIT $2")
            .bind(cutoff).bind(settings.max_feedback_events as i64+1).bind(window_start).fetch_all(&self.pool).await?;
        if rows.len() > settings.max_feedback_events {
            return Err(invalid(
                "feedback history exceeds configured rebuild bound; choose an explicit smaller data window/configuration",
            ));
        }
        let mut events = vec![];
        let mut missing = 0;
        for row in rows {
            let actor: String = row.try_get("actor")?;
            if actor != "owner" && !actor.starts_with("discord:") {
                continue;
            }
            let recorded: Option<Json<Value>> = row.try_get("learning_snapshot")?;
            if recorded
                .as_ref()
                .and_then(|r| r.0.pointer("/features/version"))
                .and_then(Value::as_str)
                .is_some_and(|version| version != FEATURE_VERSION)
            {
                missing += 1;
                continue;
            }
            let Some(input) = row.try_get::<Option<Json<Job>>, _>("input")? else {
                missing += 1;
                continue;
            };
            let evaluation: Json<radar_domain::Evaluation> = row.try_get("snapshot")?;
            let snapshot = extract(
                &input.0,
                &evaluation.0,
                row.try_get("observed_at")?,
                settings,
            );
            let action: FeedbackAction = enum_value(row.try_get("action")?)?;
            let status: String = row.try_get("application_status")?;
            let verified_outcome = match action {
                FeedbackAction::Applied => status == "submitted",
                FeedbackAction::Interview => status == "interviewing",
                FeedbackAction::Hired => status == "hired",
                FeedbackAction::Completed => status == "completed",
                FeedbackAction::Rejected => status == "rejected",
                FeedbackAction::Withdrawn => status == "withdrawn",
                _ => false,
            };
            let version: i64 = row.try_get("version")?;
            let code: Option<String> = row.try_get("reason_code")?;
            let code = code.map(enum_value).transpose()?.or_else(|| {
                if version == 1 {
                    legacy_code(
                        row.try_get::<Option<String>, _>("reason")
                            .ok()
                            .flatten()
                            .as_deref(),
                    )
                } else {
                    None
                }
            });
            events.push(LearningEvent {
                feedback_id: row.try_get("id")?,
                job_id: row.try_get("job_id")?,
                at: row.try_get("created_at")?,
                available_at: row.try_get("available_at")?,
                revision: version,
                action,
                reason_code: code,
                excluded: row.try_get("excluded")?,
                verified_outcome,
                snapshot,
                actual_effort_hours: row.try_get("actual_effort_hours")?,
            });
        }
        Ok((events, missing))
    }
    async fn calibration_from(
        &self,
        events: &[LearningEvent],
        missing: usize,
        settings: &LearningSettings,
        cutoff: DateTime<Utc>,
    ) -> Result<Calibration> {
        let mut latest = BTreeMap::new();
        for event in valid_events(events, cutoff) {
            if event.action == FeedbackAction::Other {
                continue;
            }
            if latest.get(&event.job_id).is_none_or(|old: &LearningEvent| {
                (old.at, old.feedback_id) < (event.at, event.feedback_id)
            }) {
                latest.insert(event.job_id, event);
            }
        }
        let mut report = Calibration {
            explicit_jobs: latest.len(),
            missing_historical_inputs: missing,
            ..Default::default()
        };
        for event in latest.values() {
            let positive = matches!(
                event.action,
                FeedbackAction::Interested | FeedbackAction::Save
            );
            let negative = matches!(
                event.action,
                FeedbackAction::Skip | FeedbackAction::NotAvailable
            );
            let success = event.verified_outcome
                && matches!(
                    event.action,
                    FeedbackAction::Hired | FeedbackAction::Completed
                );
            report.positive_jobs += usize::from(positive);
            report.negative_jobs += usize::from(negative);
            report.verified_successes += usize::from(success);
            report.high_score_negatives += usize::from(
                negative
                    && event
                        .snapshot
                        .base_score
                        .is_some_and(|s| s >= settings.high_score_threshold),
            );
            report.low_score_successes += usize::from(
                success
                    && event
                        .snapshot
                        .base_score
                        .is_some_and(|s| s <= settings.low_score_threshold),
            );
            let row=sqlx::query("SELECT (SELECT output FROM ai_operations WHERE job_id=$1 AND kind='evaluate' AND status='completed' AND completed_at<=$2 ORDER BY completed_at DESC,id DESC LIMIT 1) AS output,EXISTS(SELECT 1 FROM proposal_revisions WHERE job_id=$1 AND created_at<=$2) AS drafted,(SELECT learning_snapshot->'ranking' FROM feedback WHERE id=$3) AS ranking")
                .bind(event.job_id).bind(event.at).bind(event.feedback_id).fetch_one(&self.pool).await?;
            let output: Option<Json<Value>> = row.try_get("output")?;
            if let Some(output) = output {
                report.ai_comparisons += 1;
                report.ai_owner_disagreements += usize::from(
                    negative
                        && matches!(
                            output.0.get("recommendation").and_then(Value::as_str),
                            Some("strong_candidate" | "worth_investigating")
                        ),
                );
            }
            report.drafted_jobs += usize::from(row.try_get::<bool, _>("drafted")?);
            report.ranked_comparisons += usize::from(
                row.try_get::<Option<Json<Value>>, _>("ranking")?
                    .is_some_and(|v| !v.0.is_null()),
            );
            if let Some(actual) = event.actual_effort_hours {
                // Compare estimates made before the recorded application, not after delivery.
                let row=sqlx::query("SELECT output FROM ai_operations WHERE job_id=$1 AND kind='evaluate' AND status='completed' AND completed_at<coalesce((SELECT min(created_at) FROM feedback WHERE job_id=$1 AND action='applied'),$2) ORDER BY completed_at DESC,id DESC LIMIT 1")
                    .bind(event.job_id).bind(event.at).fetch_optional(&self.pool).await?;
                if let Some(row) = row {
                    let output: Json<Value> = row.try_get("output")?;
                    if let Some(estimate) = output
                        .0
                        .pointer("/estimated_effort/conservative_hours")
                        .and_then(Value::as_f64)
                    {
                        report.effort_comparisons += 1;
                        report.optimistic_effort_estimates += usize::from(actual > estimate);
                    }
                }
            }
        }
        if report.ai_owner_disagreements > 0 {
            report.suggestions.push(format!("Review {} AI/owner disagreements among {} comparable jobs; inspect explicit reasons before changing the rubric.",report.ai_owner_disagreements,report.ai_comparisons));
        }
        if report.high_score_negatives > 0 {
            report.suggestions.push(format!("{} high-base-score jobs received explicit negative preference feedback; review reasons, not inferred causes.",report.high_score_negatives));
        }
        if report.low_score_successes > 0 {
            report.suggestions.push(format!("{} low-base-score jobs have manually reported successful outcomes; review constraints and evidence without assuming causation.",report.low_score_successes));
        }
        if report.optimistic_effort_estimates > 0 {
            report.suggestions.push(format!("{} of {} pre-application effort estimates were below reported actual effort; review scope and assumptions.",report.optimistic_effort_estimates,report.effort_comparisons));
        }
        Ok(report)
    }
    pub async fn preference_view(&self) -> Result<PreferenceView> {
        let config = self.config().await?;
        let row = sqlx::query("SELECT * FROM preference_state WHERE singleton=true")
            .fetch_one(&self.pool)
            .await?;
        let mut view = PreferenceView {
            revision: row.try_get("revision")?,
            feedback_revision: row.try_get("feedback_revision")?,
            enabled: config
                .effective
                .scoring
                .personalization
                .runtime
                .as_ref()
                .is_some_and(|s| s.enabled),
            active_model: row.try_get("active_model")?,
            suppressed: row.try_get::<Json<BTreeSet<String>>, _>("suppressed")?.0,
            ..Default::default()
        };
        let rows=sqlx::query("SELECT snapshot,rejected FROM preference_models ORDER BY created_at DESC,id DESC LIMIT 30").fetch_all(&self.pool).await?;
        for row in rows {
            let Json(mut model): Json<PreferenceModel> = row.try_get("snapshot")?;
            model.training.clear();
            if row.try_get::<bool, _>("rejected")? {
                view.rejected.insert(model.id);
            }
            view.models.push(model);
        }
        if let Some(id) = view.active_model {
            let model = self.preference_model(id).await?;
            view.stale = model.feedback_revision != view.feedback_revision
                || model.feature_version != FEATURE_VERSION
                || config
                    .effective
                    .scoring
                    .personalization
                    .runtime
                    .as_ref()
                    .is_none_or(|settings| {
                        settings
                            .fingerprint(
                                config
                                    .effective
                                    .scoring
                                    .personalization
                                    .min_feedback_samples_for_suggestion
                                    as usize,
                            )
                            .ok()
                            .as_ref()
                            != Some(&model.config_hash)
                    });
            if !view.models.iter().any(|m| m.id == id) {
                let mut model = model;
                model.training.clear();
                view.models.push(model);
            }
        }
        Ok(view)
    }
    pub async fn preference_model(&self, id: Uuid) -> Result<PreferenceModel> {
        let row = sqlx::query("SELECT snapshot FROM preference_models WHERE id=$1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(StoreError::NotFound)?;
        Ok(row.try_get::<Json<PreferenceModel>, _>("snapshot")?.0)
    }
    pub async fn rebuild_preferences(&self, request: RebuildRequest) -> Result<PreferenceModel> {
        let hash = fingerprint(&request)?;
        let database_now: DateTime<Utc> = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&self.pool)
            .await?;
        let cutoff = request.cutoff.unwrap_or(database_now);
        if cutoff > database_now {
            return Err(invalid("training cutoff cannot be in the future"));
        }
        let mut tx = self.pool.begin().await?;
        let locked: bool = sqlx::query_scalar("SELECT pg_try_advisory_xact_lock(hashtextextended(current_schema() || ':radar-preferences', 0))")
            .fetch_one(&mut *tx)
            .await?;
        if !locked {
            return Err(StoreError::Conflict);
        }
        if let Some(row) =
            sqlx::query("SELECT request_hash,snapshot FROM preference_models WHERE id=$1")
                .bind(request.idempotency_key)
                .fetch_optional(&mut *tx)
                .await?
        {
            if row.try_get::<String, _>("request_hash")? != hash {
                return Err(StoreError::Conflict);
            }
            return Ok(row.try_get::<Json<PreferenceModel>, _>("snapshot")?.0);
        }
        let state = sqlx::query(
            "SELECT feedback_revision FROM preference_state WHERE singleton=true FOR UPDATE",
        )
        .fetch_one(&mut *tx)
        .await?;
        let current_feedback_revision: i64 = state.try_get("feedback_revision")?;
        let later_corrections: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM feedback_learning_revisions WHERE version>1 AND created_at>$1",
        )
        .bind(cutoff)
        .fetch_one(&mut *tx)
        .await?;
        let feedback_revision = current_feedback_revision - later_corrections;
        let config = self.config().await?;
        if !config
            .effective
            .scoring
            .personalization
            .propose_weight_adjustments
        {
            return Err(invalid("learning suggestions disabled"));
        }
        let settings = config
            .effective
            .scoring
            .personalization
            .runtime
            .as_ref()
            .ok_or_else(|| invalid("personalization runtime not configured"))?;
        let minimum = config
            .effective
            .scoring
            .personalization
            .min_feedback_samples_for_suggestion as usize;
        let (events, missing) = self.learning_events(settings, cutoff).await?;
        let mut model = fit(
            request.idempotency_key,
            &events,
            cutoff,
            settings,
            minimum,
            feedback_revision,
        )
        .map_err(|_| invalid("model fitting failed"))?;
        model.backtest =
            backtest(&events, cutoff, settings, minimum).map_err(|_| invalid("backtest failed"))?;
        model.calibration = self
            .calibration_from(&events, missing, settings, cutoff)
            .await?;
        sqlx::query("INSERT INTO preference_models(id,request_hash,snapshot) VALUES($1,$2,$3)")
            .bind(model.id)
            .bind(hash)
            .bind(Json(&model))
            .execute(&mut *tx)
            .await?;
        audit(
            &mut tx,
            "owner",
            "preference_candidate_built",
            "model",
            json!({"model_id":model.id,"samples":model.samples,"cutoff":model.cutoff}),
        )
        .await?;
        tx.commit().await?;
        Ok(model)
    }
    pub async fn control_preferences(&self, request: PreferenceControl) -> Result<PreferenceView> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended(current_schema() || ':radar-preferences', 0))")
            .execute(&mut *tx)
            .await?;
        let row = sqlx::query("SELECT * FROM preference_state WHERE singleton=true FOR UPDATE")
            .fetch_one(&mut *tx)
            .await?;
        let revision: i64 = row.try_get("revision")?;
        if revision != request.expected_revision {
            return Err(StoreError::Conflict);
        }
        let feedback_revision: i64 = row.try_get("feedback_revision")?;
        let mut active: Option<Uuid> = row.try_get("active_model")?;
        let mut suppressed = row.try_get::<Json<BTreeSet<String>>, _>("suppressed")?.0;
        let config = self.config().await?;
        match request.action {
            PreferenceAction::Activate | PreferenceAction::Rollback => {
                let id = request
                    .model_id
                    .ok_or_else(|| invalid("model ID required"))?;
                let model = self.preference_model(id).await?;
                let rejected: bool =
                    sqlx::query_scalar("SELECT rejected FROM preference_models WHERE id=$1")
                        .bind(id)
                        .fetch_one(&mut *tx)
                        .await?;
                let settings = config
                    .effective
                    .scoring
                    .personalization
                    .runtime
                    .as_ref()
                    .ok_or_else(|| invalid("personalization settings missing"))?;
                if rejected
                    || model.feedback_revision != feedback_revision
                    || model.feature_version != FEATURE_VERSION
                    || model.config_hash
                        != settings
                            .fingerprint(
                                config
                                    .effective
                                    .scoring
                                    .personalization
                                    .min_feedback_samples_for_suggestion
                                    as usize,
                            )
                            .map_err(|_| invalid("configuration fingerprint failed"))?
                {
                    return Err(invalid(
                        "stale or rejected model; rebuild and review before activation",
                    ));
                }
                active = Some(id);
            }
            PreferenceAction::Reset => {
                if !request.confirm_reset {
                    return Err(invalid("reset requires explicit confirmation"));
                }
                active = None;
            }
            PreferenceAction::Disable | PreferenceAction::Enable => {
                let mut scoring = config.effective.scoring;
                scoring
                    .personalization
                    .runtime
                    .as_mut()
                    .ok_or_else(|| invalid("personalization runtime missing"))?
                    .enabled = matches!(request.action, PreferenceAction::Enable);
                self.update_config_in(
                    &mut tx,
                    ConfigUpdate {
                        expected_revision: config.revision,
                        radar: None,
                        scoring: Some(scoring),
                        notifications: None,
                    },
                    "owner",
                )
                .await?;
            }
            PreferenceAction::Suppress | PreferenceAction::Unsuppress => {
                let feature = request
                    .feature
                    .as_ref()
                    .ok_or_else(|| invalid("feature required"))?;
                let id = request
                    .model_id
                    .or(active)
                    .ok_or_else(|| invalid("model required"))?;
                if !self
                    .preference_model(id)
                    .await?
                    .weights
                    .contains_key(feature)
                {
                    return Err(invalid("feature not present in model"));
                }
                if matches!(request.action, PreferenceAction::Suppress) {
                    suppressed.insert(feature.clone());
                } else {
                    suppressed.remove(feature);
                }
            }
            PreferenceAction::Reject => {
                let id = request
                    .model_id
                    .ok_or_else(|| invalid("model ID required"))?;
                let affected =
                    sqlx::query("UPDATE preference_models SET rejected=true WHERE id=$1")
                        .bind(id)
                        .execute(&mut *tx)
                        .await?
                        .rows_affected();
                if affected == 0 {
                    return Err(StoreError::NotFound);
                }
                if active == Some(id) {
                    active = None;
                }
            }
        }
        sqlx::query("UPDATE preference_state SET revision=revision+1,active_model=$1,suppressed=$2 WHERE singleton=true").bind(active).bind(Json(&suppressed)).execute(&mut *tx).await?;
        audit(&mut tx,"owner","preference_control","model",json!({"action":request.action,"active":active,"suppressed":suppressed,"revision":revision+1})).await?;
        tx.commit().await?;
        self.preference_view().await
    }
    pub async fn correct_learning_feedback(
        &self,
        id: Uuid,
        request: FeedbackCorrection,
    ) -> Result<FeedbackView> {
        if request
            .actual_effort_hours
            .is_some_and(|h| !h.is_finite() || h <= 0.0 || h > 10000.0)
        {
            return Err(invalid("actual effort must be positive and bounded"));
        }
        let mut tx = self.pool.begin().await?;
        // Match rebuild lock order so correction and model admission cannot race.
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended(current_schema() || ':radar-preferences', 0))")
            .execute(&mut *tx)
            .await?;
        let row=sqlx::query("SELECT f.*,j.title FROM feedback f JOIN jobs j ON j.id=f.job_id WHERE f.id=$1 FOR UPDATE OF f").bind(id).fetch_optional(&mut *tx).await?.ok_or(StoreError::NotFound)?;
        let version: i64 = row.try_get("learning_version")?;
        if version != request.expected_version {
            return Err(StoreError::Conflict);
        }
        if request.actual_effort_hours.is_some()
            && row.try_get::<String, _>("action")? != "completed"
        {
            return Err(invalid(
                "actual effort requires an explicitly completed application outcome",
            ));
        }
        let code = request.reason_code.map(name).transpose()?;
        sqlx::query("INSERT INTO feedback_learning_revisions(feedback_id,version,reason_code,excluded,actual_effort_hours) VALUES($1,$2,$3,$4,$5)").bind(id).bind(version+1).bind(&code).bind(request.excluded).bind(request.actual_effort_hours).execute(&mut *tx).await?;
        sqlx::query("UPDATE feedback SET learning_version=learning_version+1,reason_code=$2,learning_excluded=$3,actual_effort_hours=$4 WHERE id=$1").bind(id).bind(code).bind(request.excluded).bind(request.actual_effort_hours).execute(&mut *tx).await?;
        sqlx::query("UPDATE preference_state SET feedback_revision=feedback_revision+1,revision=revision+1 WHERE singleton=true").execute(&mut *tx).await?;
        audit(
            &mut tx,
            "owner",
            "learning_feedback_corrected",
            "feedback",
            json!({"feedback_id":id,"version":version+1,"excluded":request.excluded}),
        )
        .await?;
        let updated = sqlx::query(
            "SELECT f.*,j.title FROM feedback f JOIN jobs j ON j.id=f.job_id WHERE f.id=$1",
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        feedback_view(&updated)
    }
    pub async fn preference_calibration(&self) -> Result<Calibration> {
        let config = self.config().await?;
        let settings = config
            .effective
            .scoring
            .personalization
            .runtime
            .as_ref()
            .ok_or_else(|| invalid("personalization runtime missing"))?;
        let cutoff = Utc::now();
        let (events, missing) = self.learning_events(settings, cutoff).await?;
        self.calibration_from(&events, missing, settings, cutoff)
            .await
    }
    pub async fn compare_preferences(&self, id: Uuid) -> Result<ModelComparison> {
        let model = self.preference_model(id).await?;
        let config = self.config().await?;
        let settings = config
            .effective
            .scoring
            .personalization
            .runtime
            .as_ref()
            .ok_or_else(|| invalid("personalization runtime missing"))?;
        if model.feature_version != FEATURE_VERSION
            || model.config_hash
                != settings
                    .fingerprint(
                        config
                            .effective
                            .scoring
                            .personalization
                            .min_feedback_samples_for_suggestion as usize,
                    )
                    .map_err(|_| invalid("invalid configuration fingerprint"))?
        {
            return Err(invalid(
                "stale feature/configuration version; rebuild candidate",
            ));
        }
        let rows = sqlx::query(&format!(
            "SELECT {JOB_FIELDS} {JOB_FROM} ORDER BY j.id LIMIT $1"
        ))
        .bind(settings.max_ranked_jobs as i64 + 1)
        .fetch_all(&self.pool)
        .await?;
        if rows.len() > settings.max_ranked_jobs {
            return Err(invalid("comparison job bound exceeded"));
        }
        let mut jobs = rows.iter().map(job_view).collect::<Result<Vec<_>>>()?;
        jobs.sort_by(radar_application::learning::order_jobs);
        let view = self.preference_view().await?;
        if model.feedback_revision != view.feedback_revision {
            return Err(invalid(
                "feedback corrections made this model stale; rebuild before comparing current rankings",
            ));
        }
        let context = (settings.clone(), Some(model.clone()), view.suppressed);
        let baseline = jobs
            .iter()
            .map(|job| RankingRow {
                job_id: job.id,
                title: job.job.title.clone(),
                verdict: job.evaluation.verdict.clone(),
                explanation: explain(
                    &extract(&job.job, &job.evaluation, job.fetched_at, settings),
                    None,
                    settings,
                    &BTreeSet::new(),
                ),
            })
            .collect();
        for job in &mut jobs {
            Self::apply_personalization(job, &context);
        }
        jobs.sort_by(radar_application::learning::order_jobs);
        let candidate = jobs
            .into_iter()
            .map(|job| RankingRow {
                job_id: job.id,
                title: job.job.title,
                verdict: job.evaluation.verdict,
                explanation: job.personalization.unwrap(),
            })
            .collect();
        Ok(ModelComparison {
            model_id: id,
            baseline,
            candidate,
            backtest: model.backtest,
        })
    }
}
