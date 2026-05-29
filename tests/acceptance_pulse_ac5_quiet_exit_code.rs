//! Pulse AC5: `cadence pulse --quiet` prints nothing; exit code matches
//! the count of overdue tiers from `cadence pulse --json`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

#[test]
fn pulse_ac5_quiet_no_output_code_matches() {
    let (_keep, home) = common::fresh_home();
    common::cmd(&home).args(["where"]).output().expect("where");

    // --quiet: no output, exit code = overdue count.
    let quiet = common::cmd(&home)
        .args(["pulse", "--quiet"])
        .output()
        .expect("pulse --quiet");

    let quiet_code = quiet.status.code().unwrap_or(-1);

    // stdout and stderr must be empty
    let stdout = String::from_utf8_lossy(&quiet.stdout);
    let stderr = String::from_utf8_lossy(&quiet.stderr);
    assert!(
        stdout.trim().is_empty(),
        "--quiet stdout must be empty: {stdout:?}"
    );
    assert!(
        stderr.trim().is_empty(),
        "--quiet stderr must be empty: {stderr:?}"
    );

    // Get JSON count for verification.
    let json_out = common::cmd(&home)
        .args(["pulse", "--json"])
        .output()
        .expect("pulse --json");
    let json_str = String::from_utf8_lossy(&json_out.stdout);
    let rows: serde_json::Value =
        serde_json::from_str(json_str.trim()).expect(&format!("parse JSON: {json_str}"));
    let json_overdue = rows
        .as_array()
        .map(|arr| arr.iter().filter(|r| r["status"] == "overdue").count())
        .unwrap_or(0);

    assert_eq!(
        quiet_code as usize, json_overdue,
        "--quiet exit code {quiet_code} should match JSON overdue count {json_overdue}"
    );
}
