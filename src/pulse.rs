//! `cadence pulse` — per-tier overdue readout.
//!
//! The substrate records *when* each tier last produced an artifact;
//! `pulse` is the consumer that reports *what is overdue*. For each tier it
//! reports the last-produced timestamp, the expected cadence, the overdue
//! threshold, and a status (`ok` / `overdue`). The number of overdue tiers
//! doubles as a process exit code so a `SessionStart` hook can trip an alert
//! without parsing output.
//!
//! This module never creates or mutates the substrate; it only reads. The
//! caller decides whether a missing home directory is fatal (see
//! [`home_initialized`]).

use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::manifest::Manifest;
use crate::period::Tier;
use crate::record::Record;

/// Built-in expected cadence (days between artifacts) for a tier.
#[must_use]
pub const fn default_cadence_days(tier: Tier) -> u64 {
    match tier {
        Tier::Daily => 1,
        Tier::Weekly => 7,
        Tier::Monthly => 30,
        Tier::Quarterly => 92,
        Tier::Annual => 365,
    }
}

/// One tier's overdue assessment. Serializes to the `pulse --json` element.
#[derive(Debug, Clone, Serialize)]
pub struct PulseRow {
    /// Lowercase tier name (`daily` … `annual`).
    pub tier: String,
    /// Newest record's `produced_at`, or `None` if the tier is empty.
    pub last_produced_at: Option<DateTime<Utc>>,
    /// Whole days since `last_produced_at`, or `None` if never produced.
    pub age_days: Option<i64>,
    /// Effective expected cadence in days (default or manifest override).
    pub cadence_days: u64,
    /// Effective overdue threshold in days (default `2 × cadence` or override).
    pub overdue_after_days: u64,
    /// Days past the cadence (`age_days - cadence_days`); `None` if never
    /// produced. Negative while still inside the cadence window.
    pub overdue_delta_days: Option<i64>,
    /// `"ok"` or `"overdue"`.
    pub status: &'static str,
}

/// Status string for a tier that has never produced an artifact.
const STATUS_OVERDUE: &str = "overdue";
const STATUS_OK: &str = "ok";

impl PulseRow {
    /// Whether this tier counts toward the overdue exit code.
    #[must_use]
    pub fn is_overdue(&self) -> bool {
        self.status == STATUS_OVERDUE
    }
}

/// Returns `true` if the substrate home directory exists.
///
/// `pulse` treats a missing home as "not initialized" (exit 127) rather than
/// silently creating it, so the readout stays side-effect free.
#[must_use]
pub fn home_initialized(home: &Path) -> bool {
    home.is_dir()
}

fn cadence_for(manifest: &Manifest, tier: Tier) -> (u64, u64) {
    let cfg = manifest.tiers.get(tier.as_dir());
    let cadence = cfg
        .and_then(|c| c.cadence_days)
        .unwrap_or_else(|| default_cadence_days(tier));
    let overdue = cfg
        .and_then(|c| c.overdue_after_days)
        .unwrap_or_else(|| cadence.saturating_mul(2));
    (cadence, overdue)
}

/// Find the newest `produced_at` under a tier directory without creating it.
///
/// # Errors
/// Errors only on filesystem failures other than a missing tier directory
/// (which yields `Ok(None)`).
fn newest_produced_at(home: &Path, tier: Tier) -> Result<Option<DateTime<Utc>>> {
    let tier_dir = home.join(tier.as_dir());
    let periods = match fs::read_dir(&tier_dir) {
        Ok(p) => p,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e).with_context(|| format!("read_dir {}", tier_dir.display())),
    };
    let mut newest: Option<DateTime<Utc>> = None;
    for p in periods.flatten() {
        if !p.path().is_dir() {
            continue;
        }
        let Ok(files) = fs::read_dir(p.path()) else {
            continue;
        };
        for f in files.flatten() {
            if f.path().extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let Ok(fh) = File::open(f.path()) else {
                continue;
            };
            let Ok(rec) = serde_json::from_reader::<_, Record>(BufReader::new(fh)) else {
                continue;
            };
            newest = match newest {
                Some(cur) if cur >= rec.produced_at => Some(cur),
                _ => Some(rec.produced_at),
            };
        }
    }
    Ok(newest)
}

fn assess(tier: Tier, last: Option<DateTime<Utc>>, manifest: &Manifest, now: DateTime<Utc>) -> PulseRow {
    let (cadence_days, overdue_after_days) = cadence_for(manifest, tier);
    let cadence_i = i64::try_from(cadence_days).unwrap_or(i64::MAX);
    let age_days = last.map(|at| now.signed_duration_since(at).num_days());
    let overdue = age_days.is_none_or(|age| age > cadence_i);
    let overdue_delta_days = age_days.map(|age| age.saturating_sub(cadence_i));
    PulseRow {
        tier: tier.as_dir().to_string(),
        last_produced_at: last,
        age_days,
        cadence_days,
        overdue_after_days,
        overdue_delta_days,
        status: if overdue { STATUS_OVERDUE } else { STATUS_OK },
    }
}

/// Build pulse rows for every tier (daily → annual).
///
/// # Errors
/// Errors on filesystem failures while scanning tier directories.
pub fn pulse_all(home: &Path, manifest: &Manifest, now: DateTime<Utc>) -> Result<Vec<PulseRow>> {
    let mut rows = Vec::with_capacity(Tier::all().len());
    for tier in Tier::all() {
        let last = newest_produced_at(home, tier)?;
        rows.push(assess(tier, last, manifest, now));
    }
    Ok(rows)
}

/// Build the pulse row for a single tier.
///
/// # Errors
/// Errors on filesystem failures while scanning the tier directory.
pub fn pulse_tier(home: &Path, manifest: &Manifest, tier: Tier, now: DateTime<Utc>) -> Result<PulseRow> {
    let last = newest_produced_at(home, tier)?;
    Ok(assess(tier, last, manifest, now))
}

/// Count of overdue rows, clamped to the `0..=5` exit-code range.
#[must_use]
pub fn overdue_count(rows: &[PulseRow]) -> u8 {
    let n = rows.iter().filter(|r| r.is_overdue()).count();
    u8::try_from(n).unwrap_or(u8::MAX)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;
    use chrono::TimeZone;

    fn at(days_ago: i64, now: DateTime<Utc>) -> DateTime<Utc> {
        now - chrono::Duration::days(days_ago)
    }

    fn now_fixed() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 1, 12, 0, 0).unwrap()
    }

    #[test]
    fn never_produced_is_overdue() {
        let m = Manifest::default();
        let row = assess(Tier::Daily, None, &m, now_fixed());
        assert_eq!(row.status, STATUS_OVERDUE);
        assert!(row.last_produced_at.is_none());
        assert!(row.age_days.is_none());
        assert!(row.is_overdue());
    }

    #[test]
    fn exactly_at_cadence_is_ok() {
        let now = now_fixed();
        let m = Manifest::default();
        // daily cadence = 1 day; age exactly 1 day → not overdue (age > cadence is false).
        let row = assess(Tier::Daily, Some(at(1, now)), &m, now);
        assert_eq!(row.status, STATUS_OK, "age == cadence must be ok");
        assert_eq!(row.age_days, Some(1));
        assert_eq!(row.overdue_delta_days, Some(0));
    }

    #[test]
    fn one_day_past_cadence_is_overdue() {
        let now = now_fixed();
        let m = Manifest::default();
        let row = assess(Tier::Daily, Some(at(2, now)), &m, now);
        assert_eq!(row.status, STATUS_OVERDUE);
        assert_eq!(row.overdue_delta_days, Some(1));
    }

    #[test]
    fn fresh_record_is_ok() {
        let now = now_fixed();
        let m = Manifest::default();
        let row = assess(Tier::Daily, Some(at(0, now)), &m, now);
        assert_eq!(row.status, STATUS_OK);
    }

    #[test]
    fn manifest_override_tightens_cadence() {
        let now = now_fixed();
        let mut m = Manifest::default();
        m.tiers.insert(
            "weekly".to_string(),
            crate::manifest::TierConfig { cadence_days: Some(3), overdue_after_days: None },
        );
        let (cadence, overdue) = cadence_for(&m, Tier::Weekly);
        assert_eq!(cadence, 3);
        assert_eq!(overdue, 6, "overdue defaults to 2× the overridden cadence");
        // 5 days old with cadence 3 → overdue.
        let row = assess(Tier::Weekly, Some(at(5, now)), &m, now);
        assert_eq!(row.status, STATUS_OVERDUE);
    }

    #[test]
    fn default_cadences_match_pyramid() {
        assert_eq!(default_cadence_days(Tier::Daily), 1);
        assert_eq!(default_cadence_days(Tier::Weekly), 7);
        assert_eq!(default_cadence_days(Tier::Monthly), 30);
        assert_eq!(default_cadence_days(Tier::Quarterly), 92);
        assert_eq!(default_cadence_days(Tier::Annual), 365);
    }
}
