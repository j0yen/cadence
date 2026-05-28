//! Single record persisted to disk as JSON.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::period::Tier;

/// One cadence record. Append-only on disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    /// ULID for the record; unique per file.
    pub id: String,
    /// Tier this record was filed under.
    pub tier: Tier,
    /// Period string (e.g. `2026-05-24`, `2026-W21`, `2026-Q2`).
    pub period: String,
    /// Tool that produced this record.
    pub produced_by: String,
    /// When the upstream tool produced the artifact.
    pub produced_at: DateTime<Utc>,
    /// Filesystem path to the artifact.
    pub path: String,
    /// Cadence record IDs from the tier below that composed into this one.
    #[serde(default)]
    pub sources: Vec<String>,
    /// Optional human-readable summary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Free-form key/value metadata; deterministic key order via `BTreeMap`.
    #[serde(default)]
    pub meta: BTreeMap<String, String>,
}
