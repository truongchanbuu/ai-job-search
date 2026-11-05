#[path = "../../../fixtures/config.rs"]
mod config_fixture;
use chrono::{TimeZone, Utc};
use radar_application::{config::Config, learning::*};
use radar_domain::{FeedbackAction as Action, ReasonCode, Verdict};
use std::collections::BTreeSet;
use uuid::Uuid;
fn settings() -> LearningSettings {
    Config::load(config_fixture::directory().path())
        .unwrap()
        .scoring
        .personalization
        .runtime
        .unwrap()
}
fn time(day: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, day, 12, 0, 0).unwrap()
}
fn event(n: u128, action: Action, reason: Option<ReasonCode>) -> LearningEvent {
    LearningEvent {
        feedback_id: Uuid::from_u128(n),
        job_id: Uuid::from_u128(n + 100),
        at: time(n as u32),
        available_at: time(n as u32),
        revision: 1,
        action,
        reason_code: reason,
        excluded: false,
        verified_outcome: matches!(
            action,
            Action::Hired
                | Action::Completed
                | Action::Applied
                | Action::Interview
                | Action::Rejected
        ),
        actual_effort_hours: None,
        snapshot: FeatureSnapshot {
            version: FEATURE_VERSION.into(),
            observed_at: time(1),
            features: BTreeSet::from([
                "technology:rust".into(),
                "budget:limited".into(),
                "availability:limited".into(),
                "category:backend".into(),
            ]),
            base_score: Some(75.0),
            verdict: Verdict::ApplyCandidate,
        },
    }
}
#[test]
fn cold_start_single_feedback_and_sparse_regularization_remain_neutral() {
    let settings = settings();
    for events in [
        vec![],
        vec![event(1, Action::Interested, None)],
        vec![event(1, Action::Skip, Some(ReasonCode::BudgetTooLow))],
    ] {
        let model = fit(Uuid::nil(), &events, time(20), &settings, 5, 0).unwrap();
        let snapshot = event(2, Action::Interested, None).snapshot;
        assert_eq!(
            explain(&snapshot, Some(&model), &settings, &BTreeSet::new()).adjustment,
            0.0
        );
    }
}
#[test]
fn positive_negative_reason_scoping_caps_and_hard_veto_are_explainable() {
    let mut settings = settings();
    settings.min_feature_samples = 1;
    settings.max_adjustment = 2.0;
    let positive: Vec<_> = (1..=8).map(|n| event(n, Action::Hired, None)).collect();
    let model = fit(Uuid::nil(), &positive, time(20), &settings, 1, 0).unwrap();
    let snapshot = event(9, Action::Interested, None).snapshot;
    let ranking = explain(&snapshot, Some(&model), &settings, &BTreeSet::new());
    assert!(ranking.raw_adjustment > ranking.adjustment);
    assert_eq!(ranking.adjustment, 2.0);
    assert_eq!(ranking.evidence_count, 8);
    let negative: Vec<_> = (1..=5)
        .map(|n| event(n, Action::Skip, Some(ReasonCode::ScheduleConflict)))
        .collect();
    let negative_model = fit(Uuid::nil(), &negative, time(20), &settings, 1, 0).unwrap();
    assert!(!negative_model.weights.contains_key("technology:rust"));
    assert!(negative_model.weights["availability:limited"].points < 0.0);
    let mut veto = snapshot;
    veto.verdict = Verdict::Skip;
    veto.base_score = None;
    assert_eq!(
        explain(&veto, Some(&model), &settings, &BTreeSet::new()).adjustment,
        0.0
    );
    assert!(
        explain(&veto, Some(&model), &settings, &BTreeSet::new())
            .final_score
            .is_none()
    );
}
#[test]
fn duplicate_clicks_conflicts_outcomes_and_unreplied_jobs_are_not_equivalent() {
    let mut settings = settings();
    settings.min_feature_samples = 1;
    let mut interested = event(1, Action::Interested, None);
    let mut later = event(2, Action::Save, None);
    later.job_id = interested.job_id;
    let model = fit(
        Uuid::nil(),
        &[interested.clone(), later],
        time(20),
        &settings,
        1,
        0,
    )
    .unwrap();
    assert_eq!(model.samples, 1);
    let hired = fit(
        Uuid::nil(),
        &[event(1, Action::Hired, None)],
        time(20),
        &settings,
        1,
        0,
    )
    .unwrap();
    let preference = fit(
        Uuid::nil(),
        &[interested.clone()],
        time(20),
        &settings,
        1,
        0,
    )
    .unwrap();
    assert!(hired.weights["technology:rust"].points > preference.weights["technology:rust"].points);
    let mut rejected = event(2, Action::Rejected, None);
    rejected.job_id = interested.job_id;
    let model = fit(
        Uuid::nil(),
        &[interested.clone(), rejected],
        time(20),
        &settings,
        1,
        0,
    )
    .unwrap();
    assert!(model.weights["technology:rust"].points > 0.0);
    interested.verified_outcome = false;
    interested.action = Action::Hired;
    assert_eq!(
        fit(Uuid::nil(), &[interested], time(20), &settings, 1, 0)
            .unwrap()
            .samples,
        0
    );
    let conflicting = fit(
        Uuid::nil(),
        &[
            event(1, Action::Interested, None),
            event(2, Action::Skip, Some(ReasonCode::TechnologyMismatch)),
        ],
        time(20),
        &settings,
        1,
        0,
    )
    .unwrap();
    assert_eq!(conflicting.weights["technology:rust"].evidence_count, 2);
    assert!(conflicting.weights["technology:rust"].points < 0.0);
}
#[test]
fn corrections_and_chronological_holdout_do_not_leak_future_labels() {
    let mut settings = settings();
    settings.min_feature_samples = 1;
    let original = event(1, Action::Skip, Some(ReasonCode::BudgetTooLow));
    let mut correction = original.clone();
    correction.available_at = time(15);
    correction.revision = 2;
    correction.reason_code = Some(ReasonCode::TechnologyMismatch);
    let early = fit(
        Uuid::nil(),
        &[original.clone(), correction.clone()],
        time(10),
        &settings,
        1,
        0,
    )
    .unwrap();
    assert!(early.weights.contains_key("budget:limited"));
    assert!(!early.weights.contains_key("technology:rust"));
    let late = fit(
        Uuid::nil(),
        &[original.clone(), correction.clone()],
        time(20),
        &settings,
        1,
        1,
    )
    .unwrap();
    assert!(late.weights.contains_key("technology:rust"));
    correction.excluded = true;
    assert_eq!(
        fit(
            Uuid::nil(),
            &[original, correction],
            time(20),
            &settings,
            1,
            2
        )
        .unwrap()
        .samples,
        0
    );
    let events: Vec<_> = (1..=10)
        .map(|n| event(n, Action::Interested, None))
        .collect();
    let report = backtest(&events, time(20), &settings, 1).unwrap();
    assert!(report.cutoff.unwrap() < events[9].at);
    assert!(report.held_out_jobs > 0);
    let mut future = events.clone();
    future[9].action = Action::Hired;
    let a = fit(
        Uuid::nil(),
        &events,
        report.cutoff.unwrap(),
        &settings,
        1,
        0,
    )
    .unwrap();
    let b = fit(
        Uuid::nil(),
        &future,
        report.cutoff.unwrap(),
        &settings,
        1,
        0,
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(a.weights).unwrap(),
        serde_json::to_value(b.weights).unwrap()
    );
}
#[test]
fn missing_metadata_sensitive_attributes_and_stale_versions_are_excluded() {
    let settings = settings();
    let mut row = event(1, Action::Interested, None);
    row.snapshot.version = "obsolete".into();
    assert_eq!(
        fit(Uuid::nil(), &[row], time(20), &settings, 1, 0)
            .unwrap()
            .samples,
        0
    );
    let job: radar_domain::Job =
        serde_json::from_str::<Vec<radar_domain::Job>>(include_str!("../../../fixtures/jobs.json"))
            .unwrap()
            .remove(0);
    let profile = radar_adapters::load_profile(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../rust/fixtures/profiles"),
    )
    .unwrap();
    let config = Config::load(config_fixture::directory().path()).unwrap();
    let evaluation = radar_domain::evaluate(
        &job,
        &profile,
        &config.policy(),
        config.settings().mode,
        config.settings().weekly_hours,
        time(20),
        "test",
    );
    let mut job = job;
    job.budget = None;
    job.skills.clear();
    job.source_details = Some(radar_domain::SourceDetails {
        experience_level: None,
        payment_verified: None,
        client_country: Some("SENSITIVE-COUNTRY".into()),
    });
    let mut evaluation = evaluation;
    evaluation
        .breakdown
        .insert(radar_domain::Signal::BudgetEffortFit, None);
    let features = extract(&job, &evaluation, time(20), &settings);
    assert!(!features.features.iter().any(|f| f.contains("SENSITIVE")
        || f.starts_with("technology:")
        || f.starts_with("budget:")));
}

#[test]
fn time_decay_window_and_feedback_order_are_deterministic() {
    let mut settings = settings();
    settings.min_feature_samples = 1;
    let mut rows: Vec<_> = (1..=10)
        .map(|n| event(n, Action::Interested, None))
        .collect();
    let original = fit(Uuid::nil(), &rows, time(20), &settings, 1, 0).unwrap();
    rows.reverse();
    let reordered = fit(Uuid::nil(), &rows, time(20), &settings, 1, 0).unwrap();
    assert_eq!(
        serde_json::to_value(&original.weights).unwrap(),
        serde_json::to_value(reordered.weights).unwrap()
    );
    settings.half_life_days = Some(3.0);
    let decayed = fit(Uuid::nil(), &rows, time(20), &settings, 1, 0).unwrap();
    assert!(decayed.weights["technology:rust"].points < original.weights["technology:rust"].points);
    settings.training_window_days = Some(12);
    let window = fit(Uuid::nil(), &rows, time(20), &settings, 1, 0).unwrap();
    assert_eq!(window.samples, 3);
    assert_eq!(window.window_start, Some(time(8)));
    assert!(settings.validate().is_ok());
    settings.training_window_days = Some(0);
    assert!(settings.validate().is_err());
}

#[test]
fn backtest_preserves_production_verdict_priority_before_numeric_scores() {
    let mut settings = settings();
    settings.backtest_top_k = 1;
    let mut events: Vec<_> = (1..=10)
        .map(|n| event(n, Action::Interested, None))
        .collect();
    events[8].action = Action::Skip;
    events[8].reason_code = Some(ReasonCode::BudgetTooLow);
    events[8].snapshot.verdict = Verdict::Review;
    events[8].snapshot.base_score = Some(100.0);
    let report = backtest(&events, time(20), &settings, 1).unwrap();
    assert_eq!(report.held_out_jobs, 2);
    assert_eq!(report.baseline_precision_at_k, Some(1.0));
    assert_eq!(report.candidate_precision_at_k, Some(1.0));
}
