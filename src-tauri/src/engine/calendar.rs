//! Local-day boundaries in a given time zone.

use chrono::{DateTime, Duration, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;

/// The UTC instants at which a local calendar day starts and ends.
///
/// Days are not always 24 hours long: on DST transition days they are 23 or
/// 25 hours, which is why bounds are computed from local midnights.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayBounds {
    pub date: NaiveDate,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl DayBounds {
    pub fn for_date(date: NaiveDate, tz: Tz) -> Self {
        let next = date.succ_opt().unwrap_or(date);
        Self {
            date,
            start: local_midnight(date, tz),
            end: local_midnight(next, tz),
        }
    }

    pub fn containing(now: DateTime<Utc>, tz: Tz) -> Self {
        Self::for_date(local_date(now, tz), tz)
    }

    /// Seconds of the interval `[start, end)` that fall within this day.
    pub fn overlap_secs(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> i64 {
        let from = start.max(self.start);
        let to = end.min(self.end);
        (to - from).num_seconds().max(0)
    }
}

/// The local calendar date of `now` in `tz`.
pub fn local_date(now: DateTime<Utc>, tz: Tz) -> NaiveDate {
    now.with_timezone(&tz).date_naive()
}

/// The first existing local instant of `date`. Some zones skip midnight on
/// DST days, in which case the day starts at the first valid local time.
fn local_midnight(date: NaiveDate, tz: Tz) -> DateTime<Utc> {
    let midnight = date.and_time(NaiveTime::MIN);
    (0..=3)
        .find_map(|hours| {
            tz.from_local_datetime(&(midnight + Duration::hours(hours)))
                .earliest()
        })
        .map(|local| local.with_timezone(&Utc))
        // Unreachable for real time zones; fall back to UTC midnight.
        .unwrap_or_else(|| Utc.from_utc_datetime(&midnight))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_tz::Europe::Helsinki;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
    }

    #[test]
    fn normal_day_is_24_hours() {
        let day = DayBounds::for_date(date(2026, 1, 5), Helsinki);
        assert_eq!(
            day.start,
            Utc.with_ymd_and_hms(2026, 1, 4, 22, 0, 0).unwrap()
        );
        assert_eq!(day.end - day.start, Duration::hours(24));
    }

    #[test]
    fn dst_days_are_23_and_25_hours() {
        let spring = DayBounds::for_date(date(2026, 3, 29), Helsinki);
        assert_eq!(spring.end - spring.start, Duration::hours(23));
        let autumn = DayBounds::for_date(date(2026, 10, 25), Helsinki);
        assert_eq!(autumn.end - autumn.start, Duration::hours(25));
    }

    #[test]
    fn handles_zone_that_skips_midnight() {
        // America/Santiago moves clocks from 00:00 to 01:00 in September.
        let day = DayBounds::for_date(date(2026, 9, 6), chrono_tz::America::Santiago);
        assert!(day.end > day.start);
    }

    #[test]
    fn overlap_is_clipped_to_day() {
        let day = DayBounds::for_date(date(2026, 1, 5), Helsinki);
        let start = day.start - Duration::hours(1);
        let end = day.start + Duration::hours(2);
        assert_eq!(day.overlap_secs(start, end), 7200);
        assert_eq!(day.overlap_secs(day.end, day.end + Duration::hours(1)), 0);
    }
}
