//! Pulse AC3: after recording a daily artifact, `cadence pulse --tier daily`
//! reports status "ok" and exits 0.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

#[test]
fn pulse_ac3_fresh_daily_record_is_ok() {
    let (_keep, home) = common::fresh_home();
    common::cmd(&home).args(["where"]).output().expect("where");

    // Record a fresh daily artifact.
    let rec = common::cmd(&home)
        .args([
            "record",
            "daily",
            "--produced-by",
            "daily-receipt",
            "--path",
            "/tmp/d.escpos",
        ])
        .output()
        .expect("record daily");
    assert!(rec.status.success(), "record failed: {}", String::from_utf8_lossy(&rec.stderr));

    // Now pulse --tier daily should be ok (exit 0).
    let out = common::cmd(&home)
        .args(["pulse", "--tier", "daily"])
        .output()
        .expect("pulse --tier daily");

    let code = out.status.code().unwrap_or(-1);
    assert_eq!(
        code, 0,
        "expected exit 0 after recording today, got {code}\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("ok"),
        "expected 'ok' status in output: {stdout}"
    );
}
