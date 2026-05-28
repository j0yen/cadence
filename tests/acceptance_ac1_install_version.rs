//! AC1: `cadence --version` reports 0.1.0 (proxy for installability).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::process::Command;

#[test]
fn acceptance_ac1_version_reports_0_1_0() {
    let out = Command::new(common::bin_path())
        .arg("--version")
        .output()
        .expect("spawn cadence --version");
    assert!(out.status.success(), "exit: {}", out.status);
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(
        s.contains("0.1.0"),
        "stdout did not contain '0.1.0': {s:?}"
    );
}
