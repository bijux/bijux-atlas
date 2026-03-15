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
        fs::read_to_string(workspace_root().join("makes/cargo.mk")).expect("read makes/cargo.mk");
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
fn ci_lane_targets_use_check_run_surface() {
    let ci_mk = fs::read_to_string(workspace_root().join("makes/ci.mk")).expect("read makes/ci.mk");
    for marker in [
        "ci-fast: ## CI fast lane wrapper",
        "ci-pr: ## CI PR lane wrapper",
        "ci-nightly: ## CI nightly lane (includes slow checks)",
    ] {
        let start = ci_mk.find(marker).expect("target block");
        let tail = &ci_mk[start..];
        let end = tail.find("\n\n").unwrap_or(tail.len());
        let target_block = &tail[..end];
        assert!(
            target_block.contains("$(DEV_ATLAS) check run --suite"),
            "{marker} should use the live checks surface"
        );
        assert!(
            !target_block.contains("suites run"),
            "{marker} should not use the retired suites lane surface"
        );
    }
}

#[test]
fn checks_variant_targets_use_human_check_run_surface() {
    let root_mk =
        fs::read_to_string(workspace_root().join("makes/root.mk")).expect("read makes/root.mk");
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

#[test]
fn make_target_list_wrapper_uses_target_list_surface() {
    let public_mk =
        fs::read_to_string(workspace_root().join("makes/public.mk")).expect("read makes/public.mk");
    let start = public_mk
        .find("makes-target-list: ## Regenerate the makes public target list artifact")
        .expect("makes-target-list target");
    let tail = &public_mk[start..];
    let end = tail.find("\n\n").unwrap_or(tail.len());
    let target_block = &tail[..end];

    assert!(
        target_block.contains("$(DEV_ATLAS) makes target-list --allow-write"),
        "makes-target-list should use the dedicated target-list surface"
    );
    assert!(
        !target_block.contains("make surface"),
        "makes-target-list should not reuse the surface report envelope"
    );
}
