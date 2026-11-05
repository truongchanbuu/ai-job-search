//! Deterministic, owner-controlled display ranking. Does not change domain eligibility.
use crate::{api::JobView, config::digest};
use anyhow::{Result, ensure};
use chrono::{DateTime, Utc};
use radar_domain::{
    Evaluation, FeedbackAction as Action, Job, JobType, ReasonCode, Signal, Verdict,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

pub const FEATURE_VERSION: &str = "radar-features-1";
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LearningSettings {
    pub enabled: bool,
    pub max_adjustment: f64,
    pub max_feature_points: f64,
    pub prior_strength: f64,
    pub min_feature_samples: usize,
    pub max_feedback_events: usize,
    pub max_ranked_jobs: usize,
    pub backtest_max_events: usize,
    pub backtest_top_k: usize,
    pub half_life_days: Option<f64>,
    #[serde(default)]
    pub training_window_days: Option<u32>,
    pub small_effort_hours: u32,
    pub medium_effort_hours: u32,
    pub small_workload_hours: u32,
    pub allowed_technologies: Vec<String>,
    pub allowed_categories: Vec<String>,
    pub action_weights: BTreeMap<Action, f64>,
    pub high_score_threshold: f64,
    pub low_score_threshold: f64,
}
impl LearningSettings {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.max_adjustment.is_finite()
                && (0.0..=20.0).contains(&self.max_adjustment)
                && self.max_feature_points.is_finite()
                && (0.0..=10.0).contains(&self.max_feature_points)
                && self.prior_strength.is_finite()
                && (1.0..=100.0).contains(&self.prior_strength),
            "invalid personalization bounds/shrinkage"
        );
        ensure!(
            (1..=100).contains(&self.min_feature_samples)
                && (10..=5000).contains(&self.max_feedback_events)
                && (10..=2000).contains(&self.max_ranked_jobs)
                && (2..=500).contains(&self.backtest_max_events)
                && (1..=100).contains(&self.backtest_top_k),
            "invalid personalization sample limits"
        );
        ensure!(
            self.training_window_days
                .is_none_or(|days| (1..=36500).contains(&days))
                && self
                    .half_life_days
                    .is_none_or(|v| v.is_finite() && v >= 1.0)
                && self.small_effort_hours > 0
                && self.medium_effort_hours > self.small_effort_hours
                && self.small_workload_hours > 0,
            "invalid personalization feature settings"
        );
        ensure!(
            self.allowed_technologies.len() <= 128
                && self.allowed_categories.len() <= 64
                && self
                    .allowed_technologies
                    .iter()
                    .chain(&self.allowed_categories)
                    .all(|s| !s.is_empty()
                        && s.len() <= 64
                        && *s == s.trim().to_lowercase()
                        && !s.contains([':', '\n'])),
            "invalid personalization allowlists"
        );
        for action in [
            Action::Interested,
            Action::Save,
            Action::Skip,
            Action::NotAvailable,
            Action::Applied,
            Action::Interview,
            Action::Hired,
            Action::Rejected,
            Action::Completed,
            Action::Withdrawn,
            Action::Other,
        ] {
            let w = self
                .action_weights
                .get(&action)
                .ok_or_else(|| anyhow::anyhow!("missing feedback weight"))?;
            ensure!(
                w.is_finite() && (-5.0..=5.0).contains(w),
                "invalid feedback weight"
            );
            ensure!(
                match action {
                    Action::Rejected | Action::Other => *w == 0.0,
                    Action::Skip | Action::NotAvailable | Action::Withdrawn => *w <= 0.0,
                    _ => *w >= 0.0,
                },
                "feedback weight contradicts feedback meaning"
            );
        }
        ensure!(
            (0.0..=100.0).contains(&self.low_score_threshold)
                && (0.0..=100.0).contains(&self.high_score_threshold)
                && self.low_score_threshold < self.high_score_threshold,
            "invalid calibration thresholds"
        );
        Ok(())
    }
    pub fn fingerprint(&self, minimum_samples: usize) -> Result<String> {
        let mut settings = self.clone();
        settings.enabled = false;
        Ok(digest(&serde_json::to_vec(&(
            FEATURE_VERSION,
            settings,
            minimum_samples,
        ))?))
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSnapshot {
    pub version: String,
    pub observed_at: DateTime<Utc>,
    pub features: BTreeSet<String>,
    pub base_score: Option<f64>,
    pub verdict: Verdict,
}
pub fn extract(
    job: &Job,
    evaluation: &Evaluation,
    observed_at: DateTime<Utc>,
    settings: &LearningSettings,
) -> FeatureSnapshot {
    let mut features = BTreeSet::new();
    for skill in &job.skills {
        let skill = skill.trim().to_lowercase();
        if settings.allowed_technologies.contains(&skill) {
            features.insert(format!("technology:{skill}"));
        }
    }
    let category = job.category.trim().to_lowercase();
    if settings.allowed_categories.contains(&category) {
        features.insert(format!("category:{category}"));
    }
    match job.job_type {
        JobType::Hourly => {
            features.insert("job_type:hourly".into());
        }
        JobType::Fixed => {
            features.insert("job_type:fixed".into());
        }
        JobType::Unknown => {}
    }
    for (signal, prefix) in [
        (Signal::BudgetEffortFit, "budget"),
        (Signal::AvailabilityFit, "availability"),
    ] {
        if let Some(Some(value)) = evaluation.breakdown.get(&signal) {
            features.insert(format!(
                "{prefix}:{}",
                if *value >= 100.0 {
                    "suitable"
                } else {
                    "limited"
                }
            ));
        }
    }
    if let Some(scope) = job.scope_defined {
        features.insert(format!(
            "scope:{}",
            if scope { "defined" } else { "unclear" }
        ));
    }
    if let Some(hours) = job.estimated_effort_hours {
        features.insert(format!(
            "effort:{}",
            if hours <= settings.small_effort_hours {
                "small"
            } else if hours <= settings.medium_effort_hours {
                "medium"
            } else {
                "large"
            }
        ));
    }
    if let Some(hours) = job.weekly_hours {
        features.insert(format!(
            "workload:{}",
            if hours <= settings.small_workload_hours {
                "small"
            } else {
                "large"
            }
        ));
    }
    if let Some(good) = job.client_history_good {
        features.insert(format!(
            "client:history_{}",
            if good { "positive" } else { "warning" }
        ));
    }
    if let Some(details) = &job.source_details {
        if let Some(payment) = details.payment_verified {
            features.insert(format!(
                "client:payment_{}",
                if payment { "verified" } else { "unverified" }
            ));
        }
        if let Some(level) = &details.experience_level {
            let level = level.to_lowercase();
            if ["entry_level", "intermediate", "expert"].contains(&level.as_str()) {
                features.insert(format!("experience:{level}"));
            }
        }
    }
    FeatureSnapshot {
        version: FEATURE_VERSION.into(),
        observed_at,
        features,
        base_score: evaluation.score,
        verdict: evaluation.verdict.clone(),
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEvent {
    pub feedback_id: Uuid,
    pub job_id: Uuid,
    pub at: DateTime<Utc>,
    pub available_at: DateTime<Utc>,
    pub revision: i64,
    pub action: Action,
    pub reason_code: Option<ReasonCode>,
    pub excluded: bool,
    pub verified_outcome: bool,
    pub snapshot: FeatureSnapshot,
    pub actual_effort_hours: Option<f64>,
}
fn reason_matches(reason: ReasonCode, feature: &str) -> bool {
    use ReasonCode::*;
    let prefixes: &[&str] = match reason {
        TechnologyMismatch => &["technology:", "experience:"],
        TooDifficult => &["effort:", "experience:"],
        NotInteresting => &["category:", "job_type:"],
        BudgetTooLow => &["budget:"],
        DeadlineUnrealistic => &["effort:", "workload:"],
        ScheduleConflict => &["availability:", "workload:"],
        ScopeUnclear => &["scope:"],
        ClientRisk => &["client:"],
        TooCompetitive | BetterOpportunityAvailable | Other => &[],
    };
    prefixes.iter().any(|p| feature.starts_with(p))
}
pub fn signal(event: &LearningEvent, feature: &str, settings: &LearningSettings) -> f64 {
    let weight = settings
        .action_weights
        .get(&event.action)
        .copied()
        .unwrap_or(0.0);
    if !event.action.is_preference() && !event.verified_outcome {
        return 0.0;
    }
    if weight > 0.0 {
        if event
            .reason_code
            .is_some_and(|r| reason_matches(r, feature))
        {
            0.0
        } else {
            weight
        }
    } else if event
        .reason_code
        .is_some_and(|r| reason_matches(r, feature))
        || (event.action == Action::NotAvailable
            && event.reason_code.is_none()
            && reason_matches(ReasonCode::ScheduleConflict, feature))
    {
        weight
    } else {
        0.0
    }
}
pub fn valid_events(events: &[LearningEvent], cutoff: DateTime<Utc>) -> Vec<LearningEvent> {
    let mut latest: BTreeMap<Uuid, &LearningEvent> = BTreeMap::new();
    for event in events
        .iter()
        .filter(|e| e.at <= cutoff && e.available_at <= cutoff)
    {
        if latest.get(&event.feedback_id).is_none_or(|old| {
            (old.available_at, old.revision) < (event.available_at, event.revision)
        }) {
            latest.insert(event.feedback_id, event);
        }
    }
    latest
        .into_values()
        .filter(|e| {
            !e.excluded && e.snapshot.version == FEATURE_VERSION && e.snapshot.observed_at <= e.at
        })
        .cloned()
        .collect()
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureWeight {
    pub points: f64,
    pub evidence_count: usize,
    pub positive: usize,
    pub negative: usize,
    pub confidence: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceModel {
    pub id: Uuid,
    pub version: String,
    pub feature_version: String,
    pub config_hash: String,
    pub feedback_revision: i64,
    pub cutoff: DateTime<Utc>,
    #[serde(default)]
    pub window_start: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub samples: usize,
    pub minimum_samples: usize,
    pub weights: BTreeMap<String, FeatureWeight>,
    pub training: Vec<LearningEvent>,
    pub backtest: Backtest,
    pub calibration: Calibration,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Contribution {
    pub feature: String,
    pub points: f64,
    pub evidence_count: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingExplanation {
    pub base_score: Option<f64>,
    pub adjustment: f64,
    pub final_score: Option<f64>,
    pub raw_adjustment: f64,
    pub model_id: Option<Uuid>,
    pub contributions: Vec<Contribution>,
    pub confidence: f64,
    pub evidence_count: usize,
    pub model_updated_at: Option<DateTime<Utc>>,
    pub diagnostic: String,
}
pub fn fit(
    id: Uuid,
    events: &[LearningEvent],
    cutoff: DateTime<Utc>,
    settings: &LearningSettings,
    minimum_samples: usize,
    feedback_revision: i64,
) -> Result<PreferenceModel> {
    let mut jobs: BTreeMap<Uuid, LearningEvent> = BTreeMap::new();
    for event in valid_events(events, cutoff) {
        if settings
            .training_window_days
            .is_some_and(|days| event.at < cutoff - chrono::Duration::days(days as i64))
        {
            continue;
        }
        if event.snapshot.verdict == Verdict::Skip
            || event.snapshot.base_score.is_none()
            || !event
                .snapshot
                .features
                .iter()
                .any(|f| signal(&event, f, settings) != 0.0)
        {
            continue;
        }
        if jobs
            .get(&event.job_id)
            .is_none_or(|old| (old.at, old.feedback_id) < (event.at, event.feedback_id))
        {
            jobs.insert(event.job_id, event);
        }
    }
    let training: Vec<_> = jobs.into_values().collect();
    let mut sums: BTreeMap<String, (f64, f64, usize, usize, usize)> = BTreeMap::new();
    for event in &training {
        let decay = settings.half_life_days.map_or(1.0, |half| {
            2.0_f64.powf(-((cutoff - event.at).num_seconds().max(0) as f64 / 86400.0) / half)
        });
        for feature in &event.snapshot.features {
            let value = signal(event, feature, settings);
            if value == 0.0 {
                continue;
            }
            let sum = sums.entry(feature.clone()).or_default();
            sum.0 += value * decay;
            sum.1 += decay;
            sum.2 += 1;
            if value > 0.0 {
                sum.3 += 1;
            } else {
                sum.4 += 1;
            }
        }
    }
    let weights = sums
        .into_iter()
        .map(|(feature, (sum, effective, count, positive, negative))| {
            let points = if count < settings.min_feature_samples {
                0.0
            } else {
                (settings.max_feature_points * sum / (settings.prior_strength + effective))
                    .clamp(-settings.max_feature_points, settings.max_feature_points)
            };
            (
                feature,
                FeatureWeight {
                    points,
                    evidence_count: count,
                    positive,
                    negative,
                    confidence: effective / (settings.prior_strength + effective),
                },
            )
        })
        .collect();
    Ok(PreferenceModel {
        id,
        version: "radar-preference-1".into(),
        feature_version: FEATURE_VERSION.into(),
        config_hash: settings.fingerprint(minimum_samples)?,
        feedback_revision,
        cutoff,
        window_start: settings
            .training_window_days
            .and_then(|days| cutoff.checked_sub_signed(chrono::Duration::days(days as i64))),
        created_at: Utc::now(),
        samples: training.len(),
        minimum_samples,
        weights,
        training,
        backtest: Backtest::default(),
        calibration: Calibration::default(),
    })
}
pub fn explain(
    snapshot: &FeatureSnapshot,
    model: Option<&PreferenceModel>,
    settings: &LearningSettings,
    suppressed: &BTreeSet<String>,
) -> RankingExplanation {
    let mut result = RankingExplanation {
        base_score: snapshot.base_score,
        adjustment: 0.0,
        final_score: snapshot.base_score,
        raw_adjustment: 0.0,
        model_id: model.map(|m| m.id),
        contributions: vec![],
        confidence: 0.0,
        evidence_count: 0,
        model_updated_at: model.map(|m| m.created_at),
        diagnostic: "baseline; disabled, stale or no active model".into(),
    };
    let Some(model) = model else {
        return result;
    };
    if snapshot.verdict == Verdict::Skip || snapshot.base_score.is_none() {
        result.diagnostic = "hard eligibility / unknown base score preserved".into();
        return result;
    }
    if model.feature_version != FEATURE_VERSION
        || snapshot.version != FEATURE_VERSION
        || model.samples < model.minimum_samples
    {
        result.diagnostic = "insufficient samples or stale feature version".into();
        return result;
    }
    for feature in &snapshot.features {
        if suppressed.contains(feature) {
            continue;
        }
        if let Some(weight) = model.weights.get(feature).filter(|w| w.points != 0.0) {
            result.contributions.push(Contribution {
                feature: feature.clone(),
                points: weight.points,
                evidence_count: weight.evidence_count,
            });
        }
    }
    result.raw_adjustment = result.contributions.iter().map(|c| c.points).sum();
    if let Some(base) = snapshot.base_score {
        let final_score = (base
            + result
                .raw_adjustment
                .clamp(-settings.max_adjustment, settings.max_adjustment))
        .clamp(0.0, 100.0);
        result.final_score = Some(final_score);
        result.adjustment = final_score - base;
    }
    result.evidence_count = model
        .training
        .iter()
        .filter(|e| {
            result.contributions.iter().any(|c| {
                e.snapshot.features.contains(&c.feature) && signal(e, &c.feature, settings) != 0.0
            })
        })
        .count();
    result.confidence =
        result.evidence_count as f64 / (settings.prior_strength + result.evidence_count as f64);
    result.diagnostic="bounded preference adjustment; confidence is evidence strength, not conversion probability".into();
    result
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Backtest {
    pub training_samples: usize,
    pub held_out_jobs: usize,
    pub cutoff: Option<DateTime<Utc>>,
    pub baseline_precision_at_k: Option<f64>,
    pub candidate_precision_at_k: Option<f64>,
    pub ranking_agreement: Option<f64>,
    pub eligible_jobs: usize,
    pub diagnostic: String,
}
pub fn backtest(
    events: &[LearningEvent],
    as_of: DateTime<Utc>,
    settings: &LearningSettings,
    minimum_samples: usize,
) -> Result<Backtest> {
    let mut all = valid_events(events, as_of);
    all.sort_by_key(|e| (e.at, e.feedback_id));
    all.retain(|e| {
        e.snapshot.verdict != Verdict::Skip
            && e.snapshot.base_score.is_some()
            && e.snapshot
                .features
                .iter()
                .any(|f| signal(e, f, settings) != 0.0)
    });
    if all.len() > settings.backtest_max_events {
        all = all.split_off(all.len() - settings.backtest_max_events);
    }
    if all.len() < 4 {
        return Ok(Backtest {
            diagnostic:
                "insufficient explicit feedback for chronological holdout; no accuracy claim".into(),
            ..Default::default()
        });
    }
    let split = all[(all.len() * 7 / 10).min(all.len() - 2)].at;
    let heldout: BTreeMap<Uuid, LearningEvent> = all
        .iter()
        .filter(|e| e.at > split)
        .cloned()
        .map(|e| (e.job_id, e))
        .collect();
    let training: Vec<_> = events
        .iter()
        .filter(|e| e.at <= split && e.available_at <= split && !heldout.contains_key(&e.job_id))
        .cloned()
        .collect();
    let model = fit(Uuid::nil(), &training, split, settings, minimum_samples, 0)?;
    let mut baseline: Vec<_> = heldout.values().cloned().collect();
    let mut candidate = baseline.clone();
    baseline.sort_by(|a, b| {
        verdict_order(&a.snapshot.verdict)
            .cmp(&verdict_order(&b.snapshot.verdict))
            .then_with(|| {
                b.snapshot
                    .base_score
                    .unwrap()
                    .total_cmp(&a.snapshot.base_score.unwrap())
            })
            .then(a.job_id.cmp(&b.job_id))
    });
    candidate.sort_by(|a, b| {
        verdict_order(&a.snapshot.verdict)
            .cmp(&verdict_order(&b.snapshot.verdict))
            .then_with(|| {
                explain(&b.snapshot, Some(&model), settings, &BTreeSet::new())
                    .final_score
                    .unwrap()
                    .total_cmp(
                        &explain(&a.snapshot, Some(&model), settings, &BTreeSet::new())
                            .final_score
                            .unwrap(),
                    )
            })
            .then(a.job_id.cmp(&b.job_id))
    });
    let k = settings.backtest_top_k.min(heldout.len());
    let precision = |list: &[LearningEvent]| -> Option<f64> {
        if k == 0 {
            None
        } else {
            Some(
                list.iter()
                    .take(k)
                    .filter(|e| settings.action_weights[&e.action] > 0.0)
                    .count() as f64
                    / k as f64,
            )
        }
    };
    Ok(Backtest {training_samples:model.samples,held_out_jobs:heldout.len(),cutoff:Some(split),baseline_precision_at_k:precision(&baseline),candidate_precision_at_k:precision(&candidate),ranking_agreement:(!heldout.is_empty()).then(||baseline.iter().zip(&candidate).filter(|(a,b)|a.job_id==b.job_id).count() as f64/heldout.len() as f64),eligible_jobs:heldout.len(),diagnostic:"chronological prefix / disjoint held-out jobs; feedback-only sample, not population accuracy or causal evidence".into()})
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Calibration {
    pub explicit_jobs: usize,
    pub positive_jobs: usize,
    pub negative_jobs: usize,
    pub verified_successes: usize,
    pub high_score_negatives: usize,
    pub low_score_successes: usize,
    pub ai_owner_disagreements: usize,
    pub ai_comparisons: usize,
    pub drafted_jobs: usize,
    pub effort_comparisons: usize,
    pub optimistic_effort_estimates: usize,
    pub ranked_comparisons: usize,
    pub missing_historical_inputs: usize,
    pub suggestions: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RebuildRequest {
    pub idempotency_key: Uuid,
    pub cutoff: Option<DateTime<Utc>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreferenceAction {
    Activate,
    Rollback,
    Disable,
    Enable,
    Reset,
    Suppress,
    Unsuppress,
    Reject,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreferenceControl {
    pub expected_revision: i64,
    pub action: PreferenceAction,
    pub model_id: Option<Uuid>,
    pub feature: Option<String>,
    pub confirm_reset: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeedbackCorrection {
    pub expected_version: i64,
    pub reason_code: Option<ReasonCode>,
    pub excluded: bool,
    pub actual_effort_hours: Option<f64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PreferenceView {
    pub revision: i64,
    pub feedback_revision: i64,
    pub enabled: bool,
    pub active_model: Option<Uuid>,
    pub suppressed: BTreeSet<String>,
    pub models: Vec<PreferenceModel>,
    pub rejected: BTreeSet<Uuid>,
    pub stale: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingRow {
    pub job_id: Uuid,
    pub title: String,
    pub verdict: Verdict,
    pub explanation: RankingExplanation,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelComparison {
    pub model_id: Uuid,
    pub baseline: Vec<RankingRow>,
    pub candidate: Vec<RankingRow>,
    pub backtest: Backtest,
}
fn verdict_order(verdict: &Verdict) -> u8 {
    match verdict {
        Verdict::ApplyCandidate => 0,
        Verdict::Review => 1,
        Verdict::Skip => 2,
    }
}
pub fn order_jobs(a: &JobView, b: &JobView) -> std::cmp::Ordering {
    let score = |j: &JobView| {
        j.personalization
            .as_ref()
            .and_then(|e| e.final_score)
            .or(j.evaluation.score)
    };
    verdict_order(&a.evaluation.verdict)
        .cmp(&verdict_order(&b.evaluation.verdict))
        .then_with(|| {
            score(b)
                .unwrap_or(f64::NEG_INFINITY)
                .total_cmp(&score(a).unwrap_or(f64::NEG_INFINITY))
        })
        .then(a.id.cmp(&b.id))
}
