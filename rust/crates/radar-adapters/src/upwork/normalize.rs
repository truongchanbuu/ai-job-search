use super::*;
use chrono::{DateTime, Utc};
use radar_domain::{Budget, JobType, SourceDetails};

pub fn normalize(row: &Value, fields: &BTreeMap<String, String>) -> Result<Job, UpworkError> {
    let field = |name: &str| fields.get(name).and_then(|p| row.pointer(p));
    let text = |name: &str| {
        field(name)
            .and_then(Value::as_str)
            .filter(|s| !s.trim().is_empty())
            .map(str::to_owned)
    };
    let id = text("id")
        .or_else(|| field("id").and_then(Value::as_u64).map(|v| v.to_string()))
        .ok_or_else(|| UpworkError::new("job_identifier_missing"))?;
    let url = text("url").ok_or_else(|| UpworkError::new("job_url_missing"))?;
    let parsed = url::Url::parse(&url).map_err(|_| UpworkError::new("job_url_invalid"))?;
    if parsed.scheme() != "https"
        || !matches!(parsed.host_str(), Some("www.upwork.com" | "upwork.com"))
    {
        return Err(UpworkError::new("job_url_invalid"));
    }
    let job_type = match text("job_type").as_deref() {
        Some(v) if v.eq_ignore_ascii_case("hourly") => JobType::Hourly,
        Some(v) if v.eq_ignore_ascii_case("fixed") || v.eq_ignore_ascii_case("fixed_price") => {
            JobType::Fixed
        }
        _ => JobType::Unknown,
    };
    let budget = match (
        field("budget_amount").and_then(Value::as_f64),
        text("budget_currency"),
    ) {
        (Some(amount), Some(currency)) => Some(Budget { amount, currency }),
        _ => None,
    };
    let job = Job {
        provider: "upwork".into(),
        source_id: Some(id),
        url,
        title: text("title").ok_or_else(|| UpworkError::new("job_title_missing"))?,
        description: text("description"),
        category: text("category").unwrap_or_else(|| "unknown".into()),
        job_type,
        skills: field("skills")
            .and_then(Value::as_array)
            .map(|v| {
                v.iter()
                    .filter_map(|item| {
                        item.as_str().or_else(|| {
                            fields
                                .get("skill_item_name")
                                .and_then(|p| item.pointer(p))
                                .and_then(Value::as_str)
                        })
                    })
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default(),
        budget,
        weekly_hours: field("weekly_hours")
            .and_then(Value::as_u64)
            .and_then(|v| v.try_into().ok()),
        estimated_effort_hours: None,
        full_time_overlap: false,
        prohibited: false,
        already_applied: false,
        excluded: false,
        deadline: None,
        scope_defined: None,
        client_history_good: field("client_history_good").and_then(Value::as_bool),
        connects_cost: field("connects_cost")
            .and_then(Value::as_u64)
            .and_then(|v| v.try_into().ok()),
        posted_at: text("posted_at")
            .and_then(|v| DateTime::parse_from_rfc3339(&v).ok())
            .map(|v| v.with_timezone(&Utc)),
        source_details: Some(SourceDetails {
            experience_level: text("experience_level"),
            payment_verified: field("payment_verified").and_then(Value::as_bool),
            client_country: text("client_country"),
        }),
    };
    job.validate()
        .map_err(|_| UpworkError::new("invalid_job"))?;
    Ok(job)
}
pub fn matches(job: &Job, criteria: &UpworkSearch) -> bool {
    let text = format!("{} {}", job.title, job.description.as_deref().unwrap_or("")).to_lowercase();
    criteria
        .keywords
        .iter()
        .all(|k| text.contains(&k.to_lowercase()))
        && criteria
            .skills
            .iter()
            .all(|s| job.skills.iter().any(|j| j.eq_ignore_ascii_case(s)))
        && criteria.job_type.as_deref().is_none_or(|t| {
            matches!(
                (t, &job.job_type),
                ("hourly", JobType::Hourly) | ("fixed", JobType::Fixed)
            )
        })
        && criteria.minimum_budget_usd.is_none_or(|min| {
            job.budget
                .as_ref()
                .is_some_and(|b| b.currency == "USD" && b.amount >= min)
        })
        && (criteria.experience_levels.is_empty()
            || job
                .source_details
                .as_ref()
                .and_then(|d| d.experience_level.as_ref())
                .is_some_and(|level| {
                    criteria
                        .experience_levels
                        .iter()
                        .any(|v| v.eq_ignore_ascii_case(level))
                }))
}
