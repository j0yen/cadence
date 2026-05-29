//! Pulse AC7: boundary math — exactly-at-cadence is ok; exactly-at-overdue-threshold
//! is overdue. Uses the unit-test `assess` function indirectly via the binary
//! to verify the boundary conditions hold end-to-end.
//!
//! Daily defaults: cadence=1d, overdue_after=2d (2 × cadence).
//! • age=1d (exactly at cadence boundary) → ok
//! • age=2d (exactly at overdue boundary) → overdue

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use chrono::{Duration, Utc};

fn record_with_age(home: &std::path::Path, days_ago: i64) {
    let ts = (Utc::now() - Duration::days(days_ago))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();
    // Delete any existing daily records to start clean per-call
    let daily_dir = home.join("daily");
    if daily_dir.exists() {
        std::fs::remove_dir_all(&daily_dir).expect("cleanup daily dir");
    }
    let rec = common::cmd(home)
        .args([
            "record",
            "daily",
            "--produced-by",
            "test-tool",
            "--path",
            "/tmp/boundary.json",
            "--produced-at",
            &ts,
        ])
        .output()
        .expect("record daily");
    assert!(rec.status.success(), "record failed: {}", String::from_utf8_lossy(&rec.stderr));
}

#[test]
fn pulse_ac7_exactly_at_cadence_is_ok() {
    let (_keep, home) = common::fresh_home();
    common::cmd(&home).args(["where"]).output().expect("where");

    record_with_age(&home, 1); // age == cadence (1d) → ok

    let out = common::cmd(&home)
        .args(["pulse", "--tier", "daily", "--json"])
        .output()
        .expect("pulse");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect(&format!("parse JSON: {stdout}"));
    assert_eq!(
        v["status"].as_str(), Some("ok"),
        "age==cadence should be ok, got status={:?}\nJSON: {stdout}",
        v["status"]
    );
    let code = out.status.code().unwrap_or(-1);
    assert_eq!(code, 0, "exit code should be 0 when single tier is ok");
}

#[test]
fn pulse_ac7_exactly_at_overdue_threshold_is_overdue() {
    let (_keep, home) = common::fresh_home();
    common::cmd(&home).args(["where"]).output().expect("where");

    record_with_age(&home, 2); // age == overdue_after (2d) → should be overdue

    let out = common::cmd(&home)
        .args(["pulse", "--tier", "daily", "--json"])
        .output()
        .expect("pulse");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect(&format!("parse JSON: {stdout}"));
    assert_eq!(
        v["status"].as_str(), Some("overdue"),
        "age==overdue_after should be overdue, got status={:?}\nJSON: {stdout}",
        v["status"]
    );
}
