// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate dir parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn test_all_runs_nextest_once_without_retries() {
    let cargo_mk = fs::read_to_string(workspace_root().join("make/cargo.mk"))
        .expect("read make/cargo.mk");
    let start = cargo_mk
        .find("test-all: ## Run all workspace tests including slow_ and ignored tests")
        .expect("test-all target");
    let tail = &cargo_mk[start..];
    let end = tail.find("\n\n").unwrap_or(tail.len());
    let target_block = &tail[..end];

    assert_eq!(
        target_block.matches("cargo nextest run").count(),
        2,
        "test-all should define one printed command and one execution command"
    );
    assert!(
        target_block.contains("--retries 0"),
        "test-all must force retries to zero"
    );
}
