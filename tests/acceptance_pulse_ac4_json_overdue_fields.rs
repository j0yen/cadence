//! Pulse AC4: `cadence pulse --tier daily --json` for a record from 3 days ago
//! returns status "overdue" and includes all required JSON fields.
//!
//! We backdate the record using `--produced-at` to inject a 3-day-old timestamp.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use chrono::{Duration, Utc};

#[test]
fn pulse_ac4_json_overdue_has_required_fields() {
    let (_keep, home) = common::fresh_home();
    common::cmd(&home).args(["where"]).output().expect("where");

    // Record with a timestamp 3 days ago.
    let three_days_ago = (Utc::now() - Duration::days(3))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();

    let rec = common::cmd(&home)
        .args([
            "record",
            "daily",
            "--produced-by",
            "daily-receipt",
            "--path",
            "/tmp/d.escpos",
            "--produced-at",
            &three_days_ago,
        ])
        .output()
        .expect("record daily with backdate");
    assert!(rec.status.success(), "record failed: {}", String::from_utf8_lossy(&rec.stderr));

    let out = common::cmd(&home)
        .args(["pulse", "--tier", "daily", "--json"])
        .output()
        .expect("pulse --tier daily --json");

    // With a 3-day-old daily record (cadence=1d), should be overdue.
    let code = out.status.code().unwrap_or(-1);
    assert_eq!(code, 1, "expected exit 1 (1 overdue tier), got {code}");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect(&format!("parse JSON: {stdout}"));

    assert_eq!(v["status"].as_str(), Some("overdue"), "status field must be 'overdue'");
    assert!(v["last_produced_at"].is_string(), "last_produced_at must be present");
    assert!(v["cadence_days"].is_number(), "cadence_days must be present");
    assert!(v["overdue_after_days"].is_number(), "overdue_after_days must be present");
    assert!(v["overdue_delta_days"].is_number(), "overdue_delta_days must be present");

    let delta = v["overdue_delta_days"].as_i64().unwrap();
    assert!(delta >= 1, "overdue_delta_days should be ≥1 for 3-day-old daily record, got {delta}");
}
