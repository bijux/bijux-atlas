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
    let cargo_mk =
        fs::read_to_string(workspace_root().join("make/cargo.mk")).expect("read make/cargo.mk");
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

#[test]
fn suites_all_runs_only_suites_surface_commands() {
    let root_mk =
        fs::read_to_string(workspace_root().join("make/root.mk")).expect("read make/root.mk");
    let start = root_mk
        .find("suites-all: ## Run the governed validation suites sequentially")
        .expect("suites-all target");
    let tail = &root_mk[start..];
    let end = tail.find("\n\n").unwrap_or(tail.len());
    let target_block = &tail[..end];

    assert_eq!(
        target_block.matches("$(DEV_ATLAS) suites run").count(),
        2,
        "suites-all should execute the deep and contracts suites directly"
    );
    assert!(
        !target_block.contains("checks run"),
        "suites-all should not use the retired checks surface"
    );
    assert!(
        !target_block.contains("contract run"),
        "suites-all should not use the retired contract surface"
    );
    assert!(target_block.contains("--suite deep"));
    assert!(target_block.contains("--suite contracts"));
}

#[test]
fn checks_variant_targets_use_human_check_run_surface() {
    let root_mk =
        fs::read_to_string(workspace_root().join("make/root.mk")).expect("read make/root.mk");
    for marker in [
        "checks-group: ## Run one checks suite group (GROUP=<name>)",
        "checks-tag: ## Run checks suite entries with a shared tag (TAG=<name>)",
        "checks-pure: ## Run only pure checks suite entries",
        "checks-effect: ## Run only effectful checks suite entries",
    ] {
        let start = root_mk.find(marker).expect("target block");
        let tail = &root_mk[start..];
        let end = tail.find("\n\n").unwrap_or(tail.len());
        let target_block = &tail[..end];
        assert!(target_block.contains("$(DEV_ATLAS) checks run"));
        assert!(
            !target_block.contains("suites run"),
            "{marker} should not shell through the suite runner"
        );
        assert!(target_block.contains("--format $(FORMAT)"));
        assert!(
            !target_block.contains("--format json"),
            "{marker} should not emit legacy json by default"
        );
    }
}
