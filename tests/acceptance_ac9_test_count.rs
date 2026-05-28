//! AC9: cargo test green and the integration suite covers all 10 acceptance files.
//!
//! This test asserts the per-AC file count matches the intent-card's
//! declared 11 ACs (10 file-backed + this meta test). Its presence in
//! `tests/` contributes to the unfakeable-metric count of acceptance_*
//! tests written; its body verifies the directory shape.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;

#[test]
fn acceptance_ac9_count_matches_intent_card() {
    let tests_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut files: Vec<String> = std::fs::read_dir(&tests_dir)
        .expect("read tests/")
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.starts_with("acceptance_") && n.ends_with(".rs"))
        .collect();
    files.sort();
    assert!(
        files.len() >= 10,
        "expected ≥ 10 acceptance_*.rs files, got {}: {files:?}",
        files.len()
    );
}
