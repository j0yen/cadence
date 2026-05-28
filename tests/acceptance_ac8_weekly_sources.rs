//! AC8: weekly record with --sources lands under weekly/<YYYY-Www>/ with sources populated.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use chrono::{Datelike, Local};

#[test]
fn acceptance_ac8_weekly_sources_persisted() {
    let (_keep, home) = common::fresh_home();
    // A plausible source ulid.
    let src = "01HZZZZZZZZZZZZZZZZZZZZZZZ";
    let out = common::cmd(&home)
        .args([
            "record", "weekly",
            "--produced-by", "confidant",
            "--path", "/tmp/w.md",
            "--sources", src,
        ])
        .output()
        .expect("record weekly");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let id = String::from_utf8_lossy(&out.stdout).trim().to_string();

    let now = Local::now();
    let iso = now.iso_week();
    let period = format!("{}-W{:02}", iso.year(), iso.week());
    let dir = home.join("weekly").join(&period);
    let file = dir.join(format!("{id}.json"));
    assert!(file.exists(), "expected {file:?} to exist");

    let body = std::fs::read_to_string(&file).expect("read record");
    let v: serde_json::Value = serde_json::from_str(&body).expect("record JSON");
    let srcs = v["sources"].as_array().expect("sources array");
    assert_eq!(srcs.len(), 1);
    assert_eq!(srcs[0].as_str(), Some(src));
}
