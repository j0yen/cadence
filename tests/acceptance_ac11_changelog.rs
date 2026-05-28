//! AC11: CHANGELOG.md has a `## v0.1.0` section enumerating the four primary subcommands and the directory schema.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;

#[test]
fn acceptance_ac11_changelog_documents_v0_1_0() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let body = std::fs::read_to_string(root.join("CHANGELOG.md"))
        .expect("CHANGELOG.md must exist at repo root");
    assert!(body.contains("## v0.1.0"), "CHANGELOG missing '## v0.1.0' section");
    for verb in ["record", "list", "latest", "where"] {
        assert!(body.contains(verb), "CHANGELOG must mention '{verb}' subcommand");
    }
    for tier in ["daily", "weekly", "monthly", "quarterly", "annual"] {
        assert!(
            body.contains(tier),
            "CHANGELOG must mention '{tier}' tier in the directory schema"
        );
    }
}
