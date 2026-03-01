// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn crate_hygiene_docs_exist_and_are_not_placeholder_stubs() {
    let root = crate_root();
    let required = [
        "README.md",
        "CONTRACT.md",
        "docs/architecture.md",
        "docs/testing.md",
        "docs/benchmarks.md",
    ];

    for rel in required {
        let path = root.join(rel);
        assert!(
            path.exists(),
            "missing crate hygiene doc {}",
            path.display()
        );
        let text = fs::read_to_string(&path).unwrap_or_default();
        assert!(
            text.lines().count() >= 8,
            "doc too small / placeholder: {rel}"
        );
        assert!(
            !text.contains(
                "This crate-level governance page points to canonical crate docs and root docs."
            ),
            "placeholder stub text must be removed from {rel}"
        );
    }
}

#[test]
fn main_uses_adapter_workspace_root_resolver() {
    let main_rs = fs::read_to_string(crate_root().join("src/main.rs")).unwrap_or_default();
    assert!(
        main_rs.contains("WorkspaceRoot::from_cli_or_cwd"),
        "src/main.rs must use adapters::WorkspaceRoot as the single workspace root resolver entrypoint"
    );
    assert!(
        !main_rs.contains("fn discover_repo_root("),
        "src/main.rs must not define a local repo-root discovery function"
    );
}
