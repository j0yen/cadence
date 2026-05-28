//! Shared test helpers. Keep the acceptance tests in their own crates clean.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stdout,
    clippy::print_stderr,
    dead_code
)]

use std::path::PathBuf;
use std::process::Command;

use tempfile::TempDir;

/// Returns the path of the test-built `cadence` binary.
pub fn bin_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cadence"))
}

/// Creates an isolated cadence home and returns the tempdir (keep alive!) plus the cmd factory.
pub fn fresh_home() -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let home = dir.path().join("cadence-home");
    (dir, home)
}

/// Build a Command running the cadence binary with `CADENCE_HOME` pointed at `home`.
pub fn cmd(home: &std::path::Path) -> Command {
    let mut c = Command::new(bin_path());
    c.env("CADENCE_HOME", home);
    c
}
