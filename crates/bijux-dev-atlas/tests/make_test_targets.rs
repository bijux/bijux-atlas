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

#[test]
fn nextest_default_profile_marks_tests_slow_after_ten_seconds() {
    let nextest = fs::read_to_string(workspace_root().join("configs/nextest/nextest.toml"))
        .expect("read nextest config");
    assert!(
        nextest.contains("slow-timeout = { period = \"10s\", terminate-after = 12 }"),
        "default nextest profile must mark >10s tests as slow with a long termination window"
    );
}

#[test]
fn checks_all_runs_one_human_facing_check_command() {
    let root_mk =
        fs::read_to_string(workspace_root().join("make/root.mk")).expect("read make/root.mk");
    let start = root_mk
        .find("checks-all: ## Run the full non-test quality gates")
        .expect("checks-all target");
    let tail = &root_mk[start..];
    let end = tail.find("\n\n").unwrap_or(tail.len());
    let target_block = &tail[..end];

    assert_eq!(
        target_block.matches("$(DEV_ATLAS) check run").count(),
        1,
        "checks-all should execute one direct check run"
    );
    assert!(
        !target_block.contains("suites run"),
        "checks-all should not shell through the suite runner"
    );
    assert!(
        target_block.contains("--format text"),
        "checks-all should default to human-readable nextest-style output"
    );
    assert!(
        target_block.contains("--suite deep"),
        "checks-all should target the full deep checks suite"
    );
    assert!(
        target_block.contains("--include-internal --include-slow"),
        "checks-all should include internal and slow checks"
    );
    assert!(
        target_block.contains("--allow-subprocess --allow-git --allow-write --allow-network"),
        "checks-all should grant the full effects envelope needed for all checks"
    );
    assert!(
        !target_block.contains("--format json"),
        "checks-all should not emit the legacy json summary by default"
    );
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
        assert!(
            target_block.contains("$(DEV_ATLAS) check run"),
            "{marker} should use the human-facing check runner"
        );
        assert!(
            !target_block.contains("suites run"),
            "{marker} should not shell through the suite runner"
        );
        assert!(
            target_block.contains("--format text"),
            "{marker} should use human output"
        );
        assert!(
            !target_block.contains("--format json"),
            "{marker} should not emit legacy json by default"
        );
    }
}
