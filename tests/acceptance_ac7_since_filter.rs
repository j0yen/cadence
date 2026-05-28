//! AC7: list daily --since 7d --json returns only records within the last 7 days.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use chrono::{Duration as ChDuration, Utc};

#[test]
fn acceptance_ac7_since_filters_old_records() {
    let (_keep, home) = common::fresh_home();
    // Fresh (now): included.
    let r_now = common::cmd(&home)
        .args(["record", "daily", "--produced-by", "t", "--path", "/tmp/now"])
        .output()
        .expect("record now");
    assert!(r_now.status.success());
    let now_id = String::from_utf8_lossy(&r_now.stdout).trim().to_string();

    // 30 days ago: excluded by --since 7d.
    let old = Utc::now() - ChDuration::days(30);
    let old_iso = old.to_rfc3339();
    let r_old = common::cmd(&home)
        .args([
            "record", "daily",
            "--produced-by", "t",
            "--path", "/tmp/old",
            "--produced-at", &old_iso,
            "--period", &old.format("%Y-%m-%d").to_string(),
        ])
        .output()
        .expect("record old");
    assert!(r_old.status.success(), "old stderr: {}", String::from_utf8_lossy(&r_old.stderr));
    let old_id = String::from_utf8_lossy(&r_old.stdout).trim().to_string();

    let lst = common::cmd(&home)
        .args(["list", "daily", "--since", "7d", "--json"])
        .output()
        .expect("list");
    assert!(lst.status.success(), "list stderr: {}", String::from_utf8_lossy(&lst.stderr));
    let arr: serde_json::Value = serde_json::from_slice(&lst.stdout).expect("JSON array");
    let ids: Vec<String> = arr
        .as_array()
        .expect("array")
        .iter()
        .filter_map(|v| v["id"].as_str().map(String::from))
        .collect();
    assert!(ids.contains(&now_id), "recent record missing: {ids:?}");
    assert!(!ids.contains(&old_id), "old record should be filtered out: {ids:?}");
}
