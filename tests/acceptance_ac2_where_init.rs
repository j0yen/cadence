//! AC2: `cadence where` on a fresh laptop creates ~/.claude/cadence/ with zero counts.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

#[test]
fn acceptance_ac2_where_creates_home_and_zero_counts() {
    let (_keep, home) = common::fresh_home();
    let out = common::cmd(&home)
        .args(["where", "--json"])
        .output()
        .expect("spawn where");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert!(home.exists(), "home was not created");
    for sub in ["daily", "weekly", "monthly", "quarterly", "annual"] {
        assert!(home.join(sub).is_dir(), "missing tier dir: {sub}");
    }
    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("where --json must be valid JSON");
    for sub in ["daily", "weekly", "monthly", "quarterly", "annual"] {
        let n = v["counts"][sub].as_u64().expect("count integer");
        assert_eq!(n, 0, "{sub} count expected 0, got {n}");
    }
}
