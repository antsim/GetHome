//! Timer engine: day state, pure computation and Toggl synchronisation.

pub mod calendar;
pub mod compute;
pub mod scheduler;
pub mod state;

pub use compute::{compute, TimerViewModel};
pub use scheduler::{ControlError, EngineSnapshot, PollConfig, RefreshOutcome, SyncEngine};
pub use state::{SyncWarning, TimerStatus};
