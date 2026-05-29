//! Filesystem store. Reads/writes records under `$CADENCE_HOME`.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use ulid::Ulid;

use crate::manifest::Manifest;
use crate::period::{derive_period, since_cutoff, Tier};
use crate::record::Record;

/// Stats returned by `where_stats` for `cadence where`.
#[derive(Debug, Clone, Default)]
pub struct WhereStats {
    /// Resolved `$CADENCE_HOME`.
    pub home: PathBuf,
    /// Path to the manifest file.
    pub manifest_path: PathBuf,
    /// Record counts per tier (lowercase tier name → count).
    pub counts: BTreeMap<String, usize>,
}

/// Read filter for `list`.
#[derive(Debug, Clone, Default)]
pub struct ListFilter {
    /// Filter to records produced in the last N seconds (from `now`).
    pub since: Option<std::time::Duration>,
    /// Filter to records whose `period` matches.
    pub period: Option<String>,
    /// Filter to records produced by this tool.
    pub produced_by: Option<String>,
}

/// Ensure the home directory and per-tier subdirs exist; load or seed manifest.
///
/// # Errors
/// Errors on filesystem failures or malformed manifest JSON.
pub fn ensure_home(home: &Path) -> Result<Manifest> {
    fs::create_dir_all(home)
        .with_context(|| format!("create_dir_all {}", home.display()))?;
    for t in Tier::all() {
        let d = home.join(t.as_dir());
        fs::create_dir_all(&d)
            .with_context(|| format!("create_dir_all {}", d.display()))?;
    }
    let mp = manifest_path(home);
    if mp.exists() {
        let f = File::open(&mp).with_context(|| format!("open {}", mp.display()))?;
        let m: Manifest = serde_json::from_reader(BufReader::new(f))
            .with_context(|| format!("parse {}", mp.display()))?;
        Ok(m)
    } else {
        let m = Manifest::default();
        write_manifest(home, &m)?;
        Ok(m)
    }
}

fn manifest_path(home: &Path) -> PathBuf {
    home.join("manifest.json")
}

/// Read the manifest without creating the home directory.
///
/// Returns [`Manifest::default`] when no manifest file exists. Unlike
/// [`ensure_home`], this never writes to disk — it is the read path used by
/// the `pulse` readout, which must not mutate the substrate.
///
/// # Errors
/// Errors only if an existing manifest file cannot be opened or parsed.
pub fn read_manifest(home: &Path) -> Result<Manifest> {
    let mp = manifest_path(home);
    if mp.exists() {
        let f = File::open(&mp).with_context(|| format!("open {}", mp.display()))?;
        let m: Manifest = serde_json::from_reader(BufReader::new(f))
            .with_context(|| format!("parse {}", mp.display()))?;
        Ok(m)
    } else {
        Ok(Manifest::default())
    }
}

fn write_manifest(home: &Path, m: &Manifest) -> Result<()> {
    let mp = manifest_path(home);
    let tmp = mp.with_extension("json.tmp");
    let body = serde_json::to_vec_pretty(m).context("serialize manifest")?;
    {
        let mut f = File::create(&tmp)
            .with_context(|| format!("create {}", tmp.display()))?;
        f.write_all(&body)
            .with_context(|| format!("write {}", tmp.display()))?;
        f.sync_all().ok();
    }
    fs::rename(&tmp, &mp)
        .with_context(|| format!("rename {} -> {}", tmp.display(), mp.display()))?;
    Ok(())
}

/// Update or insert a tool in the manifest, then persist.
///
/// # Errors
/// Errors on filesystem failures.
pub fn register(home: &Path, name: &str, tier: Tier) -> Result<()> {
    let mut m = ensure_home(home)?;
    m.register(name, tier);
    write_manifest(home, &m)
}

/// Compute stats for `cadence where`.
///
/// # Errors
/// Errors on filesystem failures.
pub fn where_stats(home: &Path) -> Result<WhereStats> {
    ensure_home(home)?;
    let mut counts = BTreeMap::new();
    for t in Tier::all() {
        let tier_dir = home.join(t.as_dir());
        let mut n = 0usize;
        if let Ok(periods) = fs::read_dir(&tier_dir) {
            for p in periods.flatten() {
                if !p.path().is_dir() {
                    continue;
                }
                if let Ok(files) = fs::read_dir(p.path()) {
                    for f in files.flatten() {
                        if f.path().extension().and_then(|e| e.to_str()) == Some("json") {
                            n += 1;
                        }
                    }
                }
            }
        }
        counts.insert(t.as_dir().to_string(), n);
    }
    Ok(WhereStats {
        home: home.to_path_buf(),
        manifest_path: manifest_path(home),
        counts,
    })
}

/// Append a new record. Returns the assigned ULID.
///
/// # Errors
/// Errors on filesystem failures.
#[allow(clippy::too_many_arguments)]
pub fn record(
    home: &Path,
    tier: Tier,
    produced_by: &str,
    path: &str,
    produced_at: Option<DateTime<Utc>>,
    period_override: Option<&str>,
    summary: Option<&str>,
    sources: Vec<String>,
    meta: BTreeMap<String, String>,
) -> Result<Record> {
    ensure_home(home)?;
    let at = produced_at.unwrap_or_else(Utc::now);
    let period = period_override
        .map_or_else(|| derive_period(tier, at), str::to_string);
    let id = Ulid::new().to_string();
    let rec = Record {
        id: id.clone(),
        tier,
        period: period.clone(),
        produced_by: produced_by.to_string(),
        produced_at: at,
        path: path.to_string(),
        sources,
        summary: summary.map(str::to_string),
        meta,
    };
    let dir = home.join(tier.as_dir()).join(&period);
    fs::create_dir_all(&dir).with_context(|| format!("create_dir_all {}", dir.display()))?;
    let file = dir.join(format!("{id}.json"));
    if file.exists() {
        return Err(anyhow!("record file already exists: {}", file.display()));
    }
    let body = serde_json::to_vec_pretty(&rec).context("serialize record")?;
    let tmp = file.with_extension("json.tmp");
    {
        let mut f = File::create(&tmp)
            .with_context(|| format!("create {}", tmp.display()))?;
        f.write_all(&body)
            .with_context(|| format!("write {}", tmp.display()))?;
        f.sync_all().ok();
    }
    fs::rename(&tmp, &file)
        .with_context(|| format!("rename {} -> {}", tmp.display(), file.display()))?;
    Ok(rec)
}

/// Enumerate records under a tier matching `filter`.
///
/// # Errors
/// Errors on filesystem failures.
pub fn list(home: &Path, tier: Tier, filter: &ListFilter) -> Result<Vec<Record>> {
    ensure_home(home)?;
    let cutoff = match filter.since {
        Some(s) => Some(since_cutoff(Utc::now(), s)?),
        None => None,
    };
    let tier_dir = home.join(tier.as_dir());
    let mut out = Vec::new();
    let periods = match fs::read_dir(&tier_dir) {
        Ok(p) => p,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(out),
        Err(e) => return Err(anyhow!("read_dir {}: {e}", tier_dir.display())),
    };
    for p in periods.flatten() {
        if !p.path().is_dir() {
            continue;
        }
        if let Some(want_period) = filter.period.as_deref() {
            let pn = p.file_name();
            if pn.to_string_lossy() != want_period {
                continue;
            }
        }
        let Ok(files) = fs::read_dir(p.path()) else { continue };
        for f in files.flatten() {
            if f.path().extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let Ok(fh) = File::open(f.path()) else { continue };
            let Ok(rec) = serde_json::from_reader::<_, Record>(BufReader::new(fh)) else {
                continue;
            };
            if let Some(by) = filter.produced_by.as_deref() {
                if rec.produced_by != by {
                    continue;
                }
            }
            if let Some(c) = cutoff {
                if rec.produced_at < c {
                    continue;
                }
            }
            out.push(rec);
        }
    }
    out.sort_by(|a, b| b.produced_at.cmp(&a.produced_at));
    Ok(out)
}

/// Return the newest record matching the filter, if any.
///
/// # Errors
/// Errors on filesystem failures.
pub fn latest(home: &Path, tier: Tier, filter: &ListFilter) -> Result<Option<Record>> {
    let mut all = list(home, tier, filter)?;
    Ok(if all.is_empty() { None } else { Some(all.remove(0)) })
}
