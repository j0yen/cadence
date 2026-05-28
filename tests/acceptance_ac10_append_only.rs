//! AC10: two records on the same day by the same tool both persist; latest returns the newer.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::thread;
use std::time::Duration;

#[test]
fn acceptance_ac10_two_records_persist_latest_is_newer() {
    let (_keep, home) = common::fresh_home();
    let r1 = common::cmd(&home)
        .args(["record", "daily", "--produced-by", "t", "--path", "/tmp/first"])
        .output()
        .expect("record 1");
    assert!(r1.status.success());
    let id1 = String::from_utf8_lossy(&r1.stdout).trim().to_string();

    // ULIDs are millisecond-precision; a small sleep avoids tie-breaking ambiguity
    // in tests that look at produced_at ordering.
    thread::sleep(Duration::from_millis(20));

    let r2 = common::cmd(&home)
        .args(["record", "daily", "--produced-by", "t", "--path", "/tmp/second"])
        .output()
        .expect("record 2");
    assert!(r2.status.success());
    let id2 = String::from_utf8_lossy(&r2.stdout).trim().to_string();
    assert_ne!(id1, id2);

    let lst = common::cmd(&home)
        .args(["list", "daily", "--json"])
        .output()
        .expect("list");
    let arr: serde_json::Value = serde_json::from_slice(&lst.stdout).expect("JSON array");
    let ids: Vec<String> = arr
        .as_array()
        .expect("array")
        .iter()
        .filter_map(|v| v["id"].as_str().map(String::from))
        .collect();
    assert!(ids.contains(&id1) && ids.contains(&id2), "both ids must persist: {ids:?}");

    let lat = common::cmd(&home)
        .args(["latest", "daily", "--produced-by", "t", "--json"])
        .output()
        .expect("latest");
    let v: serde_json::Value = serde_json::from_slice(&lat.stdout).expect("JSON");
    assert_eq!(v["path"].as_str(), Some("/tmp/second"), "latest must return newer record");
    assert_eq!(v["id"].as_str(), Some(id2.as_str()));
}
