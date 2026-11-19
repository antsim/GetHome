//! Sliding-window counter of requests made to the Toggl API.
//!
//! Toggl allows 30 requests per hour for user-scoped endpoints. The app keeps
//! its own count so that it can slow down before hitting the hard limit.

use chrono::{DateTime, Duration, Utc};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct RateBudget {
    window: Duration,
    requests: VecDeque<DateTime<Utc>>,
}

impl Default for RateBudget {
    fn default() -> Self {
        Self::new(Duration::hours(1))
    }
}

impl RateBudget {
    pub fn new(window: Duration) -> Self {
        Self {
            window,
            requests: VecDeque::new(),
        }
    }

    /// Records a request made at `now`.
    pub fn record(&mut self, now: DateTime<Utc>) {
        self.prune(now);
        self.requests.push_back(now);
    }

    /// Number of requests made within the window ending at `now`.
    pub fn count(&mut self, now: DateTime<Utc>) -> usize {
        self.prune(now);
        self.requests.len()
    }

    fn prune(&mut self, now: DateTime<Utc>) {
        let cutoff = now - self.window;
        while self.requests.front().is_some_and(|t| *t <= cutoff) {
            self.requests.pop_front();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn counts_requests_within_sliding_window() {
        let t0 = Utc.with_ymd_and_hms(2026, 1, 5, 8, 0, 0).unwrap();
        let mut budget = RateBudget::default();
        budget.record(t0);
        budget.record(t0 + Duration::minutes(30));
        assert_eq!(budget.count(t0 + Duration::minutes(59)), 2);
        assert_eq!(budget.count(t0 + Duration::minutes(60)), 1);
        assert_eq!(budget.count(t0 + Duration::minutes(91)), 0);
    }
}
