use crate::config::{Config, Weekday};
use chrono::{DateTime, Datelike, Duration, LocalResult, NaiveTime, TimeZone, Utc};
use radar_domain::Mode;

/// DST gaps are skipped; an ambiguous wall-clock slot runs once at its earlier instant.
pub fn scan_slots(
    config: &Config,
    after: DateTime<Utc>,
    until: DateTime<Utc>,
) -> Vec<DateTime<Utc>> {
    if config.radar.app.active_mode == Mode::Paused {
        return vec![];
    }
    let zone: chrono_tz::Tz = config
        .radar
        .app
        .timezone
        .parse()
        .expect("validated timezone");
    slots(
        &config.work_mode().schedule.weekdays,
        &config.work_mode().schedule.at,
        zone,
        after,
        until,
    )
}
pub fn slots(
    days: &[Weekday],
    times: &[String],
    zone: chrono_tz::Tz,
    after: DateTime<Utc>,
    until: DateTime<Utc>,
) -> Vec<DateTime<Utc>> {
    let mut date = after.with_timezone(&zone).date_naive();
    let last = until.with_timezone(&zone).date_naive();
    let mut found = vec![];
    while date <= last {
        let day = match date.weekday() {
            chrono::Weekday::Mon => Weekday::Mon,
            chrono::Weekday::Tue => Weekday::Tue,
            chrono::Weekday::Wed => Weekday::Wed,
            chrono::Weekday::Thu => Weekday::Thu,
            chrono::Weekday::Fri => Weekday::Fri,
            chrono::Weekday::Sat => Weekday::Sat,
            chrono::Weekday::Sun => Weekday::Sun,
        };
        if days.contains(&day) {
            for time in times {
                let Ok(time) = NaiveTime::parse_from_str(time, "%H:%M") else {
                    continue;
                };
                let at = match zone.from_local_datetime(&date.and_time(time)) {
                    LocalResult::Single(t) => Some(t),
                    LocalResult::Ambiguous(a, b) => Some(a.min(b)),
                    LocalResult::None => None,
                };
                if let Some(at) = at {
                    let at = at.with_timezone(&Utc);
                    if at > after && at <= until {
                        found.push(at);
                    }
                }
            }
        }
        let Some(next) = date.succ_opt() else { break };
        date = next;
    }
    found.sort();
    found.dedup();
    found
}
pub fn next_scan(config: &Config, after: DateTime<Utc>) -> Option<DateTime<Utc>> {
    if config.radar.scheduler.as_ref().is_none_or(|s| !s.enabled) {
        return None;
    }
    scan_slots(config, after, after + Duration::days(8))
        .into_iter()
        .next()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn local_days_and_dst_are_explicit() {
        let at = |s: &str| s.parse::<DateTime<Utc>>().unwrap();
        let found = slots(
            &[Weekday::Mon],
            &["00:30".into()],
            chrono_tz::Asia::Ho_Chi_Minh,
            at("2026-11-01T17:00:00Z"),
            at("2026-11-01T18:00:00Z"),
        );
        assert_eq!(found, vec![at("2026-11-01T17:30:00Z")]);
        assert!(
            slots(
                &[Weekday::Sun],
                &["02:30".into()],
                chrono_tz::America::New_York,
                at("2026-03-08T05:00:00Z"),
                at("2026-03-08T10:00:00Z")
            )
            .is_empty()
        );
        assert_eq!(
            slots(
                &[Weekday::Sun],
                &["01:30".into()],
                chrono_tz::America::New_York,
                at("2026-11-01T04:00:00Z"),
                at("2026-11-01T09:00:00Z")
            )
            .len(),
            1
        );
    }
}
