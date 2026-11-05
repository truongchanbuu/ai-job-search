mod decisions;
pub use decisions::*;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Signal {
    TechnicalFit,
    VerifiedExperience,
    ScopeClarity,
    BudgetEffortFit,
    AvailabilityFit,
    ClientRisk,
    PersonalInterest,
}

impl Signal {
    pub const ALL: [Self; 7] = [
        Self::TechnicalFit,
        Self::VerifiedExperience,
        Self::ScopeClarity,
        Self::BudgetEffortFit,
        Self::AvailabilityFit,
        Self::ClientRisk,
        Self::PersonalInterest,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Employed,
    Available,
    Busy,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Job {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_details: Option<SourceDetails>,
    pub provider: String,
    pub source_id: Option<String>,
    pub url: String,
    pub title: String,
    pub description: Option<String>,
    pub category: String,
    pub job_type: JobType,
    pub skills: Vec<String>,
    pub budget: Option<Budget>,
    pub weekly_hours: Option<u32>,
    pub estimated_effort_hours: Option<u32>,
    pub full_time_overlap: bool,
    pub prohibited: bool,
    pub already_applied: bool,
    pub excluded: bool,
    pub deadline: Option<DateTime<Utc>>,
    pub scope_defined: Option<bool>,
    pub client_history_good: Option<bool>,
    pub connects_cost: Option<u32>,
    pub posted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    Unknown,
    Hourly,
    Fixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Budget {
    pub amount: f64,
    pub currency: String,
}

/// Optional facts returned by an external source, never inferred from missing data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceDetails {
    pub experience_level: Option<String>,
    pub payment_verified: Option<bool>,
    pub client_country: Option<String>,
}

impl Job {
    pub fn validate(&self) -> Result<(), String> {
        if self.provider.trim().is_empty()
            || self.title.trim().is_empty()
            || self.category.trim().is_empty()
        {
            return Err("provider, title and category must be nonempty".into());
        }
        let url = Url::parse(&self.url).map_err(|_| "invalid job URL")?;
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err("job URL must be HTTP(S) without credentials".into());
        }
        if self
            .budget
            .as_ref()
            .is_some_and(|b| !b.amount.is_finite() || b.amount <= 0.0 || b.currency.len() != 3)
            || self.weekly_hours == Some(0)
            || self.estimated_effort_hours == Some(0)
            || self.skills.iter().any(|s| s.trim().is_empty())
        {
            return Err("invalid budget, hours or skills".into());
        }
        Ok(())
    }

    pub fn identity(&self) -> Result<String, String> {
        self.validate()?;
        if let Some(id) = self.source_id.as_ref().filter(|id| !id.trim().is_empty()) {
            // JSON tuple encoding avoids separator collisions in provider IDs.
            return Ok(format!("id:{:?}:{:?}", self.provider.trim(), id.trim()));
        }
        let mut url = Url::parse(&self.url).map_err(|_| "invalid job URL")?;
        url.set_fragment(None);
        let mut query: Vec<(String, String)> = url
            .query_pairs()
            .filter(|(k, _)| !k.starts_with("utm_") && k != "ref")
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();
        query.sort();
        url.set_query(None);
        if !query.is_empty() {
            url.query_pairs_mut().extend_pairs(query);
        }
        Ok(format!("url:{:?}:{}", self.provider.trim(), url))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub proof: String,
    pub context: EvidenceContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceContext {
    Employment,
    Contract,
    Academic,
    Training,
    Personal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    pub skills: BTreeMap<String, Evidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preferences {
    pub preferred_categories: Vec<String>,
    pub blocked_categories: Vec<String>,
    pub minimum_hourly_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub revision: String,
    pub identity: Identity,
    pub preferences: Preferences,
    // Prose is retained as data; it never changes verified facts or executable rules.
    pub documents: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Vetoes {
    pub prohibited_or_unethical: bool,
    pub requires_full_time_overlap: bool,
    pub cannot_meet_required_deadline: bool,
    pub outside_available_weekly_hours: bool,
    pub already_applied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringPolicy {
    pub weights: BTreeMap<Signal, u32>,
    pub shortlist_min_score: u32,
    pub vetoes: Vetoes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    ApplyCandidate,
    Review,
    Skip,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evaluation {
    pub score: Option<f64>,
    pub breakdown: BTreeMap<Signal, Option<f64>>,
    pub verdict: Verdict,
    pub confidence: Confidence,
    pub risk: String,
    pub vetoes: Vec<String>,
    pub strengths: Vec<String>,
    pub gaps: Vec<String>,
    pub unknowns: Vec<String>,
    pub evidence: Vec<String>,
    pub assumptions: Vec<String>,
    pub config_revision: String,
    pub profile_revision: String,
}

pub fn evaluate(
    job: &Job,
    profile: &Profile,
    policy: &ScoringPolicy,
    mode: Mode,
    capacity: u32,
    now: DateTime<Utc>,
    revision: &str,
) -> Evaluation {
    let mut result = Evaluation {
        score: None,
        breakdown: BTreeMap::new(),
        verdict: Verdict::Review,
        confidence: Confidence::High,
        risk: "unknown".into(),
        vetoes: vec![],
        strengths: vec![],
        gaps: vec![],
        unknowns: vec![],
        evidence: vec![],
        assumptions: vec![],
        config_revision: revision.into(),
        profile_revision: profile.revision.clone(),
    };
    let checks = [
        (
            job.prohibited && policy.vetoes.prohibited_or_unethical,
            "policy_prohibited",
        ),
        (
            job.full_time_overlap
                && matches!(mode, Mode::Employed | Mode::Busy)
                && policy.vetoes.requires_full_time_overlap,
            "unrealistic_required_availability",
        ),
        (
            job.weekly_hours.is_some_and(|h| h > capacity)
                && policy.vetoes.outside_available_weekly_hours,
            "over_capacity",
        ),
        (
            job.deadline.is_some_and(|d| {
                d <= now
                    || job
                        .estimated_effort_hours
                        .is_some_and(|h| (d - now).num_seconds() < i64::from(h) * 3600)
            }) && policy.vetoes.cannot_meet_required_deadline,
            "cannot_meet_required_deadline",
        ),
        (
            job.already_applied && policy.vetoes.already_applied,
            "already_applied",
        ),
        (job.excluded, "excluded"),
        (
            profile
                .preferences
                .blocked_categories
                .contains(&job.category),
            "user_blocked_category",
        ),
    ];
    for (veto, reason) in checks {
        if veto {
            result.vetoes.push(reason.into());
        }
    }
    for skill in &job.skills {
        if let Some(e) = profile.identity.skills.get(&skill.to_lowercase()) {
            result.evidence.push(format!(
                "identity.md: {skill}: {} ({:?})",
                e.proof, e.context
            ));
        } else {
            result
                .gaps
                .push(format!("unsupported required skill: {skill}"));
        }
    }
    if !result.gaps.is_empty() {
        result.vetoes.push("unsupported_skill_requirement".into());
    }
    if !result.vetoes.is_empty() {
        result.verdict = Verdict::Skip;
        result.risk = "high".into();
        return result;
    }
    let mut set = |signal, value: Option<f64>, unknown: &str| {
        if value.is_none() {
            result.unknowns.push(unknown.into());
        }
        result.breakdown.insert(signal, value);
    };
    set(
        Signal::TechnicalFit,
        (!job.skills.is_empty()).then_some(100.0),
        "required_skills",
    );
    set(
        Signal::VerifiedExperience,
        (!job.skills.is_empty()).then_some(100.0),
        "related_evidence",
    );
    set(
        Signal::ScopeClarity,
        job.scope_defined.map(|v| if v { 100.0 } else { 0.0 }),
        "scope_clarity",
    );
    let rate = job
        .budget
        .as_ref()
        .filter(|b| b.currency == "USD")
        .and_then(|b| match job.job_type {
            JobType::Unknown => None,
            JobType::Hourly => Some(b.amount),
            JobType::Fixed => job.estimated_effort_hours.map(|h| b.amount / f64::from(h)),
        });
    set(
        Signal::BudgetEffortFit,
        rate.map(|r| (r / profile.preferences.minimum_hourly_usd * 100.0).min(100.0)),
        if job.budget.is_none() {
            "budget"
        } else {
            "budget_currency_or_estimated_effort"
        },
    );
    set(
        Signal::AvailabilityFit,
        job.weekly_hours
            .map(|h| (f64::from(capacity) / f64::from(h)).min(1.0) * 100.0),
        "weekly_hours",
    );
    set(
        Signal::ClientRisk,
        job.client_history_good.map(|v| if v { 100.0 } else { 0.0 }),
        "client_history",
    );
    set(
        Signal::PersonalInterest,
        Some(
            if profile
                .preferences
                .preferred_categories
                .contains(&job.category)
            {
                100.0
            } else {
                50.0
            },
        ),
        "interest",
    );
    if job.connects_cost.is_none() {
        result.unknowns.push("connects_cost".into());
    }
    let missing_description = job.description.as_ref().is_none_or(|s| s.trim().is_empty());
    if missing_description {
        result
            .unknowns
            .push("full_description: needs_review".into());
    }
    // Unknown inputs contribute a neutral midpoint, with separate confidence penalties.
    let score = result
        .breakdown
        .iter()
        .map(|(signal, value)| value.unwrap_or(50.0) * f64::from(policy.weights[signal]) / 100.0)
        .sum::<f64>();
    result.score = Some((score * 100.0).round() / 100.0);
    result.confidence = match result.unknowns.len() {
        0 => Confidence::High,
        1..=2 => Confidence::Medium,
        _ => Confidence::Low,
    };
    result.risk = match job.client_history_good {
        Some(true) => "low",
        Some(false) => "high",
        None => "unknown",
    }
    .into();
    result.verdict = if !missing_description
        && result.confidence != Confidence::Low
        && score >= f64::from(policy.shortlist_min_score)
    {
        Verdict::ApplyCandidate
    } else if score < f64::from(policy.shortlist_min_score) && !missing_description {
        Verdict::Skip
    } else {
        Verdict::Review
    };
    for (signal, value) in &result.breakdown {
        if value.is_some_and(|v| v >= 75.0) {
            result.strengths.push(format!("{signal:?}"));
        }
        if value.is_some_and(|v| v < 50.0) {
            result.gaps.push(format!("weak signal: {signal:?}"));
        }
    }
    result.assumptions.push(
        "Unknown score inputs use a neutral midpoint; confidence is reported separately.".into(),
    );
    if matches!(job.job_type, JobType::Fixed) {
        result.assumptions.push("Effective USD rate uses estimated effort, before fees; effort is not a confirmed fact.".into());
    }
    result
}
