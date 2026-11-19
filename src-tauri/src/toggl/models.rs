//! Product-independent domain models. The wire formats of Toggl Track and
//! Toggl 2.0 are mapped to these in [`super::track`] and [`super::focus`].

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::client::Flavor;

/// A tracked time entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeEntry {
    pub id: i64,
    pub workspace_id: i64,
    pub project_id: Option<i64>,
    pub task_id: Option<i64>,
    pub description: Option<String>,
    pub start: DateTime<Utc>,
    /// `None` while the timer is running.
    pub stop: Option<DateTime<Utc>>,
    pub tags: EntryTags,
    pub billable: bool,
}

impl TimeEntry {
    pub fn is_running(&self) -> bool {
        self.stop.is_none()
    }
}

/// Tags of an entry. Toggl Track identifies tags by name, Toggl 2.0 by id.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum EntryTags {
    #[default]
    None,
    Names(Vec<String>),
    Ids(Vec<i64>),
}

/// A request to start a running time entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartEntry {
    pub workspace_id: i64,
    pub project_id: Option<i64>,
    pub task_id: Option<i64>,
    pub description: Option<String>,
    pub tags: EntryTags,
    pub billable: bool,
    pub start: DateTime<Utc>,
}

/// The account and workspace a token gives access to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub flavor: Flavor,
    /// User's name. Toggl 2.0 does not expose it.
    pub name: Option<String>,
    pub workspace_id: i64,
    pub workspace_name: Option<String>,
    /// Toggl 2.0 only.
    pub organization_id: Option<i64>,
}
