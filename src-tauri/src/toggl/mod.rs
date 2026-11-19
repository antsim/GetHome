//! Toggl Track API v9 integration.
//!
//! Every request to Toggl must go through [`TogglClient`] so that it is
//! counted by the [`RateBudget`](rate_budget::RateBudget).

pub mod client;
pub mod models;
pub mod rate_budget;

pub use client::{TogglClient, TogglError};
pub use models::{Me, NewTimeEntry, TimeEntry, Workspace};
