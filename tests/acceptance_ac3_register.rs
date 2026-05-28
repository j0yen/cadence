//! AC3: register adds tool entry to manifest.json.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

#[test]
fn acceptance_ac3_register_writes_manifest_entry() {
    let (_keep, home) = common::fresh_home();
    let out = common::cmd(&home)
        .args(["register", "daily-receipt", "--tier", "daily"])
        .output()
        .expect("spawn register");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let body = std::fs::read_to_string(home.join("manifest.json")).expect("manifest.json");
    let v: serde_json::Value = serde_json::from_str(&body).expect("manifest is JSON");
    let tools = v["tools"].as_array().expect("tools array");
    let found = tools.iter().any(|t| t["name"].as_str() == Some("daily-receipt"));
    assert!(found, "daily-receipt not in tools[]: {body}");
}
