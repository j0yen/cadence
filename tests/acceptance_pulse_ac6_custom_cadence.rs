//! Pulse AC6: setting `tiers.weekly.cadence_days: 3` in manifest.json makes
//! a 5-day-old weekly record overdue.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use chrono::{Duration, Utc};

#[test]
fn pulse_ac6_custom_cadence_overrides() {
    let (_keep, home) = common::fresh_home();
    // Initialize home via `where`.
    common::cmd(&home).args(["where"]).output().expect("where");

    // Write a manifest.json with tiers.weekly.cadence_days = 3.
    let manifest_path = home.join("manifest.json");
    let manifest = serde_json::json!({
        "schema": "cadence.manifest.v1",
        "version": "0.2.0",
        "tools": [],
        "tiers": {
            "weekly": {
                "cadence_days": 3
            }
        }
    });
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest).unwrap())
        .expect("write manifest");

    // Record a weekly artifact from 5 days ago.
    let five_days_ago = (Utc::now() - Duration::days(5))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();

    let rec = common::cmd(&home)
        .args([
            "record",
            "weekly",
            "--produced-by",
            "test-tool",
            "--path",
            "/tmp/w.json",
            "--produced-at",
            &five_days_ago,
        ])
        .output()
        .expect("record weekly with backdate");
    assert!(rec.status.success(), "record failed: {}", String::from_utf8_lossy(&rec.stderr));

    // With cadence_days=3, a 5-day-old record is overdue.
    let out = common::cmd(&home)
        .args(["pulse", "--tier", "weekly", "--json"])
        .output()
        .expect("pulse --tier weekly --json");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect(&format!("parse JSON: {stdout}"));

    assert_eq!(
        v["status"].as_str(), Some("overdue"),
        "weekly tier should be overdue with cadence_days=3 and a 5-day-old record"
    );
    assert_eq!(
        v["cadence_days"].as_u64(), Some(3),
        "cadence_days should reflect the manifest override"
    );
}
