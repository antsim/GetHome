//! Synchronisation with Toggl: polling, backoff, wake/day-change detection
//! and timer control.
//!
//! [`SyncEngine`] owns the authoritative [`DayState`] and publishes it as an
//! [`EngineSnapshot`] through a `watch` channel. It never sleeps itself: the
//! caller drives it by calling [`SyncEngine::tick`] about once per second,
//! which keeps it deterministic and testable with a manual clock.

use std::sync::Arc;

use chrono::{DateTime, Duration, SubsecRound, Utc};
use chrono_tz::Tz;
use serde::Serialize;
use tokio::sync::watch;

use super::calendar::{local_date, DayBounds};
use super::state::{DayState, EntryTemplate, SyncStatus, SyncWarning};
use crate::clock::Clock;
use crate::toggl::{TogglClient, TogglError};

/// Polling and backoff timings. See PLAN.md, chapter 3.
#[derive(Debug, Clone)]
pub struct PollConfig {
    /// Interval for `GET /me/time_entries/current`.
    pub current_interval: Duration,
    /// Slower interval used when the request budget is running low.
    pub throttled_current_interval: Duration,
    /// Interval for a full fetch of today's entries.
    pub full_sync_interval: Duration,
    /// Requests per hour after which polling slows down.
    pub budget_soft_limit: usize,
    /// A gap between ticks longer than this means the machine was asleep.
    pub wake_gap: Duration,
    /// Minimum time between manual refreshes.
    pub manual_refresh_throttle: Duration,
    /// Successive delays after failures; the last one repeats.
    pub backoff_steps: Vec<Duration>,
}

impl Default for PollConfig {
    fn default() -> Self {
        Self {
            current_interval: Duration::minutes(4),
            throttled_current_interval: Duration::minutes(10),
            full_sync_interval: Duration::minutes(30),
            budget_soft_limit: 25,
            wake_gap: Duration::seconds(60),
            manual_refresh_throttle: Duration::seconds(30),
            backoff_steps: vec![
                Duration::minutes(1),
                Duration::minutes(2),
                Duration::minutes(5),
                Duration::minutes(10),
            ],
        }
    }
}

/// What the rest of the app sees of the engine.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EngineSnapshot {
    pub day: Option<DayState>,
    pub sync: SyncStatus,
}

/// Result of a manual refresh request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RefreshOutcome {
    Refreshed,
    /// Ignored because the previous manual refresh was too recent.
    Throttled,
    /// Ignored because Toggl asked us to back off.
    RateLimited,
    /// The refresh was attempted but failed; see the sync warning.
    Failed,
}

#[derive(Debug, thiserror::Error)]
pub enum ControlError {
    #[error("no timer is running")]
    NotRunning,
    #[error("a timer is already running")]
    AlreadyRunning,
    #[error(transparent)]
    Toggl(#[from] TogglError),
}

#[derive(Debug, Default)]
struct Backoff {
    failures: usize,
    until: Option<DateTime<Utc>>,
}

impl Backoff {
    fn is_active(&self, now: DateTime<Utc>) -> bool {
        self.until.is_some_and(|until| now < until)
    }

    fn fail(&mut self, now: DateTime<Utc>, steps: &[Duration]) {
        let delay = steps
            .get(self.failures)
            .or_else(|| steps.last())
            .copied()
            .unwrap_or_else(|| Duration::minutes(1));
        self.until = Some(now + delay);
        self.failures += 1;
    }

    fn reset(&mut self) {
        *self = Self::default();
    }
}

pub struct SyncEngine {
    client: Arc<TogglClient>,
    clock: Arc<dyn Clock>,
    config: PollConfig,
    tz: Tz,
    default_workspace_id: Option<i64>,
    snapshot: watch::Sender<EngineSnapshot>,

    day: Option<DayState>,
    sync: SyncStatus,
    full_sync_due: bool,
    next_full_sync: DateTime<Utc>,
    next_current_poll: DateTime<Utc>,
    backoff: Backoff,
    auth_blocked: bool,
    last_tick: Option<DateTime<Utc>>,
    last_manual_refresh: Option<DateTime<Utc>>,
}

impl SyncEngine {
    pub fn new(
        client: Arc<TogglClient>,
        clock: Arc<dyn Clock>,
        config: PollConfig,
        tz: Tz,
        default_workspace_id: Option<i64>,
    ) -> Self {
        let (snapshot, _) = watch::channel(EngineSnapshot::default());
        Self {
            client,
            clock,
            config,
            tz,
            default_workspace_id,
            snapshot,
            day: None,
            sync: SyncStatus::default(),
            full_sync_due: true,
            next_full_sync: DateTime::<Utc>::MIN_UTC,
            next_current_poll: DateTime::<Utc>::MIN_UTC,
            backoff: Backoff::default(),
            auth_blocked: false,
            last_tick: None,
            last_manual_refresh: None,
        }
    }

    /// Receiver that is notified whenever the day or sync status changes.
    pub fn subscribe(&self) -> watch::Receiver<EngineSnapshot> {
        self.snapshot.subscribe()
    }

    /// Runs whatever synchronisation is due. Call about once per second.
    pub async fn tick(&mut self) {
        let now = self.clock.now();
        if self
            .last_tick
            .is_some_and(|prev| now - prev > self.config.wake_gap)
        {
            log::info!("clock jumped since last tick (sleep?); scheduling a full sync");
            self.full_sync_due = true;
        }
        self.last_tick = Some(now);

        if self.day.as_ref().map(|d| d.date) != Some(local_date(now, self.tz)) {
            self.full_sync_due = true;
        }

        if !self.can_sync(now) {
            return;
        }
        if self.full_sync_due || now >= self.next_full_sync {
            self.full_sync(now).await;
        } else if now >= self.next_current_poll {
            self.poll_current(now).await;
        }
    }

    /// Manual "refresh now". Throttled to protect the request budget.
    pub async fn refresh_now(&mut self) -> Result<RefreshOutcome, TogglError> {
        let now = self.clock.now();
        if !self.client.has_token() {
            return Err(TogglError::MissingToken);
        }
        if self.auth_blocked {
            return Err(TogglError::Unauthorized);
        }
        if self
            .last_manual_refresh
            .is_some_and(|t| now - t < self.config.manual_refresh_throttle)
        {
            return Ok(RefreshOutcome::Throttled);
        }
        if self.backoff.is_active(now) && self.sync.warning == SyncWarning::RateLimited {
            return Ok(RefreshOutcome::RateLimited);
        }
        self.last_manual_refresh = Some(now);
        self.backoff.reset();
        Ok(if self.full_sync(now).await {
            RefreshOutcome::Refreshed
        } else {
            RefreshOutcome::Failed
        })
    }

    /// Stops the running timer. The UI is updated optimistically before the
    /// request; a full sync verifies the result on the next tick.
    pub async fn stop_timer(&mut self) -> Result<(), ControlError> {
        let now = self.clock.now();
        let today = DayBounds::containing(now, self.tz);
        let previous = self.day.clone();
        let running = self
            .day
            .as_mut()
            .and_then(|day| day.stop_running(now, &today))
            .ok_or(ControlError::NotRunning)?;
        let Some(entry_id) = running.id else {
            self.day = previous;
            return Err(ControlError::NotRunning);
        };
        self.publish();

        let result = self
            .client
            .stop_time_entry(running.workspace_id, entry_id)
            .await;
        self.full_sync_due = true;
        self.track(result, now).map(drop).map_err(|err| {
            self.day = previous;
            self.publish();
            err.into()
        })
    }

    /// Starts a new timer that continues the most recent entry (description,
    /// project, tags). Optimistic like [`Self::stop_timer`].
    pub async fn resume_timer(&mut self) -> Result<(), ControlError> {
        let now = self.clock.now().trunc_subsecs(0);
        if self.day.as_ref().is_some_and(|d| d.running.is_some()) {
            return Err(ControlError::AlreadyRunning);
        }
        if !self.client.has_token() {
            return Err(TogglError::MissingToken.into());
        }
        let template = match self.day.as_ref().and_then(|d| d.latest_entry.clone()) {
            Some(template) => template,
            None => self.fetch_resume_template(now).await?,
        };

        let previous = self.day.clone();
        let today = DayBounds::containing(now, self.tz);
        self.day
            .get_or_insert_with(|| DayState::empty(today.date))
            .start_running(now, template.clone());
        self.publish();

        let result = self
            .client
            .start_time_entry(&template.to_new_entry(now))
            .await;
        match self.track(result, now) {
            Ok(entry) => {
                if let Some(running) = self.day.as_mut().and_then(|d| d.running.as_mut()) {
                    running.id = Some(entry.id);
                    running.start = entry.start;
                }
                self.full_sync_due = true;
                self.publish();
                Ok(())
            }
            Err(err) => {
                self.day = previous;
                self.publish();
                Err(err.into())
            }
        }
    }

    /// Call after the API token has changed (or been removed).
    pub fn token_changed(&mut self, default_workspace_id: Option<i64>) {
        self.default_workspace_id = default_workspace_id;
        self.auth_blocked = false;
        self.backoff.reset();
        self.day = None;
        self.sync = SyncStatus::default();
        self.full_sync_due = true;
        self.publish();
    }

    pub fn set_time_zone(&mut self, tz: Tz) {
        if tz != self.tz {
            self.tz = tz;
            self.full_sync_due = true;
        }
    }

    fn can_sync(&mut self, now: DateTime<Utc>) -> bool {
        if !self.client.has_token() {
            self.set_warning(SyncWarning::NoToken);
            return false;
        }
        !self.auth_blocked && !self.backoff.is_active(now)
    }

    /// Fetches today's entries and rebuilds the day. Returns `true` on success.
    async fn full_sync(&mut self, now: DateTime<Utc>) -> bool {
        let today = DayBounds::containing(now, self.tz);
        // Start one day early so that entries crossing midnight are included.
        let from = today.start - Duration::days(1);
        let result = self.client.time_entries(from, today.end).await;
        match self.track(result, now) {
            Ok(entries) => {
                self.day = Some(DayState::from_entries(&today, &entries));
                self.full_sync_due = false;
                self.next_full_sync = now + self.config.full_sync_interval;
                self.schedule_current_poll(now);
                self.publish();
                true
            }
            Err(_) => false,
        }
    }

    /// Checks the running entry; a change triggers a full sync.
    async fn poll_current(&mut self, now: DateTime<Utc>) {
        let result = self.client.current_time_entry().await;
        let Ok(current) = self.track(result, now) else {
            return;
        };
        self.schedule_current_poll(now);
        let known_id = self
            .day
            .as_ref()
            .and_then(|d| d.running.as_ref())
            .and_then(|r| r.id);
        if current.map(|e| e.id) != known_id {
            log::info!("running entry changed in Toggl; fetching today's entries");
            self.full_sync(now).await;
        }
    }

    async fn fetch_resume_template(
        &mut self,
        now: DateTime<Utc>,
    ) -> Result<EntryTemplate, TogglError> {
        let recent = self.client.recent_time_entries().await;
        if let Some(template) = EntryTemplate::latest_of(&self.track(recent, now)?) {
            return Ok(template);
        }
        let workspace_id = match self.default_workspace_id {
            Some(id) => id,
            None => {
                let me = self.client.me().await;
                let id = self.track(me, now)?.default_workspace_id;
                self.default_workspace_id = Some(id);
                id
            }
        };
        Ok(EntryTemplate::blank(workspace_id))
    }

    fn schedule_current_poll(&mut self, now: DateTime<Utc>) {
        let interval = if self.client.requests_in_last_hour() > self.config.budget_soft_limit {
            self.config.throttled_current_interval
        } else {
            self.config.current_interval
        };
        self.next_current_poll = now + interval;
    }

    /// Updates the sync status from the result of a request.
    fn track<T>(
        &mut self,
        result: Result<T, TogglError>,
        now: DateTime<Utc>,
    ) -> Result<T, TogglError> {
        match &result {
            Ok(_) => {
                self.backoff.reset();
                self.sync = SyncStatus {
                    last_synced: Some(now),
                    warning: SyncWarning::None,
                };
            }
            Err(err) => {
                log::warn!("Toggl request failed: {err}");
                self.sync.warning = match err {
                    TogglError::MissingToken => SyncWarning::NoToken,
                    TogglError::Unauthorized => {
                        self.auth_blocked = true;
                        SyncWarning::AuthError
                    }
                    err if err.is_rate_limit() => {
                        self.backoff.fail(now, &self.config.backoff_steps);
                        SyncWarning::RateLimited
                    }
                    _ => {
                        self.backoff.fail(now, &self.config.backoff_steps);
                        SyncWarning::Offline
                    }
                };
            }
        }
        self.publish();
        result
    }

    fn set_warning(&mut self, warning: SyncWarning) {
        if self.sync.warning != warning {
            self.sync.warning = warning;
            self.publish();
        }
    }

    fn publish(&self) {
        let next = EngineSnapshot {
            day: self.day.clone(),
            sync: self.sync,
        };
        self.snapshot.send_if_modified(|current| {
            let changed = *current != next;
            if changed {
                *current = next;
            }
            changed
        });
    }
}

#[cfg(test)]
mod tests;
