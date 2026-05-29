//! Pulse AC1: `cadence pulse` on a fresh substrate (no records) reports all five
//! tiers as overdue: never and exits with code 5.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

#[test]
fn pulse_ac1_fresh_substrate_five_overdue() {
    let (_keep, home) = common::fresh_home();
    // Initialize the home so pulse doesn't exit 127.
    common::cmd(&home).args(["where"]).output().expect("where");

    let out = common::cmd(&home)
        .args(["pulse"])
        .output()
        .expect("spawn pulse");

    // Exit code should be 5 (all five tiers overdue).
    let code = out.status.code().unwrap_or(-1);
    assert_eq!(code, 5, "expected exit 5 (5 overdue), got {code}\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr));

    let stdout = String::from_utf8_lossy(&out.stdout);
    for tier in ["daily", "weekly", "monthly", "quarterly", "annual"] {
        assert!(stdout.contains(tier), "pulse output should contain tier '{tier}'");
    }
    // All rows should mention overdue state
    assert!(
        stdout.contains("overdue"),
        "fresh substrate should report overdue tiers: {stdout}"
    );
}
