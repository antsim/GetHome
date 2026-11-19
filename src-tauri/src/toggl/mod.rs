//! Toggl integration. Supports two products with different APIs:
//!
//! - **Toggl Track** (API v9, `api.track.toggl.com`, HTTP Basic auth) – see
//!   [`track`].
//! - **Toggl 2.0** (`focus.toggl.com/api`, Bearer API keys starting with
//!   `toggl_sk_`) – see [`focus`].
//!
//! [`TogglClient`] picks the API from the token ([`Flavor::detect`]) and maps
//! both wire formats to the shared domain [`models`]. Every request goes
//! through the client so that it is counted by the
//! [`RateBudget`](rate_budget::RateBudget).

pub mod client;
pub mod focus;
pub mod models;
pub mod rate_budget;
pub mod track;

pub use client::{Flavor, TogglClient, TogglError};
pub use models::{Account, EntryTags, StartEntry, TimeEntry};
