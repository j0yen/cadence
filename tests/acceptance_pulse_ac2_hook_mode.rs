//! Pulse AC2: `cadence pulse --hook` on a fresh substrate prints one line per
//! overdue tier to stderr; nothing to stdout; exits with code 5.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

#[test]
fn pulse_ac2_hook_mode_stderr_only() {
    let (_keep, home) = common::fresh_home();
    common::cmd(&home).args(["where"]).output().expect("where");

    let out = common::cmd(&home)
        .args(["pulse", "--hook"])
        .output()
        .expect("spawn pulse --hook");

    let code = out.status.code().unwrap_or(-1);
    assert_eq!(code, 5, "expected exit 5, got {code}");

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.trim().is_empty(),
        "stdout must be empty in --hook mode, got: {stdout:?}"
    );

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!stderr.trim().is_empty(), "stderr must contain overdue hints");
    // Should have at least 5 lines (one per overdue tier)
    let lines: Vec<&str> = stderr.lines().filter(|l| !l.is_empty()).collect();
    assert!(
        lines.len() >= 5,
        "expected ≥5 stderr lines (one per tier), got {}: {stderr:?}",
        lines.len()
    );
}
