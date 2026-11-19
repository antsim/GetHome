//! Serde models for the subset of the Toggl Track API v9 used by the app.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Value sent as `created_with` when the app creates time entries.
pub const CREATED_WITH: &str = "tyoaikalaskuri";

/// A Toggl time entry (`/me/time_entries`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeEntry {
    pub id: i64,
    pub workspace_id: i64,
    #[serde(default)]
    pub project_id: Option<i64>,
    #[serde(default)]
    pub task_id: Option<i64>,
    #[serde(default)]
    pub description: Option<String>,
    pub start: DateTime<Utc>,
    #[serde(default)]
    pub stop: Option<DateTime<Utc>>,
    /// Duration in seconds. Negative for a running entry, so it must not be
    /// used for running entries; compute `now - start` instead.
    pub duration: i64,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub billable: bool,
}

impl TimeEntry {
    /// A running entry has no stop time and a negative duration.
    pub fn is_running(&self) -> bool {
        self.stop.is_none() || self.duration < 0
    }
}

/// Payload for creating (starting) a time entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NewTimeEntry {
    pub created_with: &'static str,
    pub workspace_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    pub billable: bool,
    pub start: DateTime<Utc>,
    /// `-1` starts a running entry.
    pub duration: i64,
}

/// The authenticated user (`/me`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Me {
    pub id: i64,
    #[serde(default)]
    pub fullname: String,
    pub default_workspace_id: i64,
    /// Only present when requested with `with_related_data=true`.
    #[serde(default)]
    pub workspaces: Vec<Workspace>,
}

impl Me {
    /// Name of the default workspace, if workspace data was included.
    pub fn default_workspace_name(&self) -> Option<&str> {
        self.workspaces
            .iter()
            .find(|w| w.id == self.default_workspace_id)
            .map(|w| w.name.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Workspace {
    pub id: i64,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_running_entry() {
        let json = r#"{
            "id": 1, "workspace_id": 2, "project_id": null, "description": "Koodaus",
            "start": "2026-03-02T07:00:00+00:00", "stop": null, "duration": -1772434800,
            "tags": null, "billable": false, "at": "2026-03-02T07:00:00+00:00"
        }"#;
        let entry: TimeEntry = serde_json::from_str(json).expect("valid entry");
        assert!(entry.is_running());
        assert_eq!(entry.description.as_deref(), Some("Koodaus"));
    }

    #[test]
    fn deserializes_stopped_entry_with_z_suffix() {
        let json = r#"{"id": 1, "workspace_id": 2, "start": "2026-03-02T07:00:00Z",
            "stop": "2026-03-02T08:00:00Z", "duration": 3600}"#;
        let entry: TimeEntry = serde_json::from_str(json).expect("valid entry");
        assert!(!entry.is_running());
    }

    #[test]
    fn finds_default_workspace_name() {
        let json = r#"{"id": 1, "fullname": "Testi", "default_workspace_id": 7,
            "workspaces": [{"id": 3, "name": "Muu"}, {"id": 7, "name": "Oma"}]}"#;
        let me: Me = serde_json::from_str(json).expect("valid user");
        assert_eq!(me.default_workspace_name(), Some("Oma"));
    }
}
