//! cadence — shared time-pyramid record store.
//!
//! Substrate for the laptop's reflective-artifact tools (daily-receipt,
//! confidant, letters-we-never-sent, conversations-zine, memory-reliquary).
//! Records are flat JSON files under `$CADENCE_HOME/<tier>/<period>/<ulid>.json`,
//! with a manifest at `$CADENCE_HOME/manifest.json` tracking registered tools.

#![cfg_attr(not(test), forbid(unsafe_code))]

pub mod home;
pub mod manifest;
pub mod period;
pub mod pulse;
pub mod record;
pub mod store;

pub use home::resolve_home;
pub use manifest::{Manifest, TierConfig, ToolEntry};
pub use period::{derive_period, parse_since, Tier};
pub use pulse::{home_initialized, overdue_count, pulse_all, pulse_tier, PulseRow};
pub use record::Record;
pub use store::{
    latest, list, read_manifest, record as record_store, register, where_stats, ListFilter,
    WhereStats,
};
