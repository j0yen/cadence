//! AC4: record daily creates a file at ~/.claude/cadence/daily/<date>/<ulid>.json and prints the ulid.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use chrono::Local;

#[test]
fn acceptance_ac4_record_daily_writes_and_prints_ulid() {
    let (_keep, home) = common::fresh_home();
    let out = common::cmd(&home)
        .args([
            "record",
            "daily",
            "--produced-by",
            "daily-receipt",
            "--path",
            "/tmp/test.escpos",
            "--summary",
            "manual test",
        ])
        .output()
        .expect("spawn record");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let printed = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert!(!printed.is_empty(), "stdout was empty");
    let today = Local::now().format("%Y-%m-%d").to_string();
    let dir = home.join("daily").join(&today);
    let file = dir.join(format!("{printed}.json"));
    assert!(file.exists(), "expected {file:?} to exist");
    let body = std::fs::read_to_string(&file).expect("read record");
    let v: serde_json::Value = serde_json::from_str(&body).expect("record is JSON");
    assert_eq!(v["id"].as_str(), Some(printed.as_str()));
    assert_eq!(v["produced_by"].as_str(), Some("daily-receipt"));
    assert_eq!(v["path"].as_str(), Some("/tmp/test.escpos"));
    assert_eq!(v["summary"].as_str(), Some("manual test"));
}
