//! AC6: latest daily --produced-by daily-receipt --json returns the path from AC4.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

#[test]
fn acceptance_ac6_latest_returns_recorded_path() {
    let (_keep, home) = common::fresh_home();
    let rec = common::cmd(&home)
        .args([
            "record", "daily",
            "--produced-by", "daily-receipt",
            "--path", "/tmp/expected.escpos",
        ])
        .output()
        .expect("record");
    assert!(rec.status.success(), "record stderr: {}", String::from_utf8_lossy(&rec.stderr));

    let lat = common::cmd(&home)
        .args(["latest", "daily", "--produced-by", "daily-receipt", "--json"])
        .output()
        .expect("latest");
    assert!(lat.status.success(), "latest stderr: {}", String::from_utf8_lossy(&lat.stderr));
    let v: serde_json::Value =
        serde_json::from_slice(&lat.stdout).expect("latest --json must be valid JSON");
    assert_eq!(v["path"].as_str(), Some("/tmp/expected.escpos"));
}
