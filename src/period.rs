//! Tier + datetime → period string. ISO 8601 weeks, calendar quarters.

use std::str::FromStr;
use std::time::Duration;

use anyhow::{anyhow, bail, Result};
use chrono::{DateTime, Datelike, Local, Utc};
use serde::{Deserialize, Serialize};

/// Time-pyramid tiers, ordered finest → coarsest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    /// Day buckets (`YYYY-MM-DD`).
    Daily,
    /// ISO 8601 week buckets (`YYYY-Www`).
    Weekly,
    /// Calendar month buckets (`YYYY-MM`).
    Monthly,
    /// Calendar quarter buckets (`YYYY-Q[1-4]`).
    Quarterly,
    /// Calendar year buckets (`YYYY`).
    Annual,
}

impl Tier {
    /// All tiers in pyramid order (daily → annual).
    #[must_use]
    pub const fn all() -> [Self; 5] {
        [Self::Daily, Self::Weekly, Self::Monthly, Self::Quarterly, Self::Annual]
    }

    /// Lower-case directory name for this tier.
    #[must_use]
    pub const fn as_dir(self) -> &'static str {
        match self {
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
            Self::Quarterly => "quarterly",
            Self::Annual => "annual",
        }
    }
}

impl FromStr for Tier {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "daily" => Ok(Self::Daily),
            "weekly" => Ok(Self::Weekly),
            "monthly" => Ok(Self::Monthly),
            "quarterly" => Ok(Self::Quarterly),
            "annual" => Ok(Self::Annual),
            other => Err(anyhow!("unknown tier '{other}' (expected daily|weekly|monthly|quarterly|annual)")),
        }
    }
}

impl std::fmt::Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_dir())
    }
}

/// Derive a period string from a tier + UTC instant (converted to local tz for date math).
#[must_use]
pub fn derive_period(tier: Tier, at: DateTime<Utc>) -> String {
    let local = at.with_timezone(&Local);
    match tier {
        Tier::Daily => local.format("%Y-%m-%d").to_string(),
        Tier::Weekly => {
            let iso = local.iso_week();
            format!("{}-W{:02}", iso.year(), iso.week())
        }
        Tier::Monthly => local.format("%Y-%m").to_string(),
        Tier::Quarterly => {
            let q = (local.month() - 1) / 3 + 1;
            format!("{}-Q{q}", local.year())
        }
        Tier::Annual => local.format("%Y").to_string(),
    }
}

/// Parse a duration shorthand: `7d`, `2w`, `1mo`, `1y`, `48h`.
///
/// # Errors
/// Returns an error if the input is empty, has no recognized suffix, or the
/// number is unparseable.
pub fn parse_since(s: &str) -> Result<Duration> {
    let s = s.trim();
    if s.is_empty() {
        bail!("empty duration");
    }
    let (num_str, unit) = split_num_unit(s)?;
    let n: u64 = num_str
        .parse()
        .map_err(|e| anyhow!("bad duration number '{num_str}': {e}"))?;
    let secs = match unit {
        "s" => n,
        "m" | "min" => n * 60,
        "h" => n * 3600,
        "d" => n * 86_400,
        "w" => n * 86_400 * 7,
        "mo" => n * 86_400 * 30,
        "y" => n * 86_400 * 365,
        other => bail!("unknown duration unit '{other}' (expected s|m|h|d|w|mo|y)"),
    };
    Ok(Duration::from_secs(secs))
}

fn split_num_unit(s: &str) -> Result<(&str, &str)> {
    let split_at = s
        .char_indices()
        .find_map(|(i, c)| if c.is_ascii_alphabetic() { Some(i) } else { None })
        .ok_or_else(|| anyhow!("duration '{s}' has no unit suffix"))?;
    if split_at == 0 {
        bail!("duration '{s}' has no leading number");
    }
    let (num, unit) = s.split_at(split_at);
    Ok((num, unit))
}

/// Convert a `chrono::Duration`-style age cutoff into a UTC instant.
///
/// # Errors
/// Returns an error if subtraction overflows (i.e. the duration is absurdly large).
pub fn since_cutoff(now: DateTime<Utc>, since: Duration) -> Result<DateTime<Utc>> {
    let secs = i64::try_from(since.as_secs())
        .map_err(|e| anyhow!("duration too large: {e}"))?;
    let cd = chrono::Duration::try_seconds(secs)
        .ok_or_else(|| anyhow!("duration overflow constructing chrono::Duration"))?;
    now.checked_sub_signed(cd)
        .ok_or_else(|| anyhow!("cutoff underflow subtracting {secs}s from {now}"))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn tier_roundtrip() {
        for t in Tier::all() {
            let s = t.to_string();
            let back: Tier = s.parse().unwrap();
            assert_eq!(t, back);
        }
    }

    #[test]
    fn quarter_boundaries() {
        let q1 = Utc.with_ymd_and_hms(2026, 1, 15, 12, 0, 0).unwrap();
        let q2 = Utc.with_ymd_and_hms(2026, 4, 1, 12, 0, 0).unwrap();
        let q3 = Utc.with_ymd_and_hms(2026, 7, 1, 12, 0, 0).unwrap();
        let q4 = Utc.with_ymd_and_hms(2026, 12, 31, 12, 0, 0).unwrap();
        assert_eq!(derive_period(Tier::Quarterly, q1), "2026-Q1");
        assert_eq!(derive_period(Tier::Quarterly, q2), "2026-Q2");
        assert_eq!(derive_period(Tier::Quarterly, q3), "2026-Q3");
        assert_eq!(derive_period(Tier::Quarterly, q4), "2026-Q4");
    }

    #[test]
    fn weekly_iso() {
        // 2026-01-01 is Thursday → ISO week 1.
        let d = Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap();
        let p = derive_period(Tier::Weekly, d);
        assert!(p.starts_with("2026-W") || p.starts_with("2025-W"));
    }

    #[test]
    fn parse_since_basics() {
        assert_eq!(parse_since("7d").unwrap().as_secs(), 7 * 86_400);
        assert_eq!(parse_since("2w").unwrap().as_secs(), 14 * 86_400);
        assert_eq!(parse_since("3h").unwrap().as_secs(), 3 * 3600);
        assert_eq!(parse_since("1mo").unwrap().as_secs(), 30 * 86_400);
        assert!(parse_since("").is_err());
        assert!(parse_since("abc").is_err());
        assert!(parse_since("10xyz").is_err());
    }
}
