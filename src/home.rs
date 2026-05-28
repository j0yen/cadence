//! Resolves `CADENCE_HOME`. Default is `~/.claude/cadence/`. Override via env var.

use std::path::PathBuf;

use anyhow::{anyhow, Result};

/// Resolve the cadence home directory.
///
/// Precedence:
///   1. `$CADENCE_HOME` if set and non-empty.
///   2. `~/.claude/cadence` (HOME + .claude/cadence).
///
/// # Errors
/// Returns an error if neither `$CADENCE_HOME` nor `$HOME` is set.
pub fn resolve_home() -> Result<PathBuf> {
    if let Ok(v) = std::env::var("CADENCE_HOME") {
        if !v.trim().is_empty() {
            return Ok(PathBuf::from(v));
        }
    }
    let home = std::env::var("HOME")
        .map_err(|_| anyhow!("HOME unset and CADENCE_HOME not provided"))?;
    Ok(PathBuf::from(home).join(".claude").join("cadence"))
}
