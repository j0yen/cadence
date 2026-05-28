//! AC5: list daily --json includes the ulid from AC4.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

#[test]
fn acceptance_ac5_list_includes_recorded_ulid() {
    let (_keep, home) = common::fresh_home();
    let rec = common::cmd(&home)
        .args([
            "record", "daily",
            "--produced-by", "daily-receipt",
            "--path", "/tmp/x.escpos",
        ])
        .output()
        .expect("record");
    assert!(rec.status.success(), "record stderr: {}", String::from_utf8_lossy(&rec.stderr));
    let ulid = String::from_utf8_lossy(&rec.stdout).trim().to_string();

    let lst = common::cmd(&home)
        .args(["list", "daily", "--json"])
        .output()
        .expect("list");
    assert!(lst.status.success(), "list stderr: {}", String::from_utf8_lossy(&lst.stderr));
    let arr: serde_json::Value =
        serde_json::from_slice(&lst.stdout).expect("list --json must be JSON array");
    let ids: Vec<String> = arr
        .as_array()
        .expect("array")
        .iter()
        .filter_map(|v| v["id"].as_str().map(String::from))
        .collect();
    assert!(ids.contains(&ulid), "ulid {ulid} not in {ids:?}");
}
