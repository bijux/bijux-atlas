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
        .find("test-all-rs: ## Run all workspace tests including slow_ and ignored tests")
        .expect("test-all-rs target");
    let tail = &cargo_mk[start..];
    let end = tail.find("\n\n").unwrap_or(tail.len());
    let target_block = &tail[..end];

    assert!(
        target_block.contains("\"$(RUST_GATE_BIN)\" test-all"),
        "test-all should delegate through the standardized rust gate wrapper"
    );
    assert!(
        target_block.contains("NEXTEST_THREADS_ALL"),
        "test-all must pass the governed full-suite concurrency setting"
    );
}

#[test]
fn cargo_gate_module_declares_standardized_rust_gate_wiring() {
    let cargo_mk =
        fs::read_to_string(workspace_root().join("makes/cargo.mk")).expect("read makes/cargo.mk");

    for expected in [
        "NEXTEST_PROFILE_FAST ?= fast-unit",
        "NEXTEST_PROFILE_SLOW ?= slow-integration",
        "NEXTEST_PROFILE_ALL ?= full",
        "RUST_GATE_BIN ?= makes/bin/rust_gate.sh",
        "NEXTEST_EXPR_BIN ?= makes/bin/nextest_expr.sh",
    ] {
        assert!(
            cargo_mk.contains(expected),
            "cargo gate wiring should declare `{expected}`"
        );
    }
}

#[test]
fn cargo_gate_aliases_delegate_to_rust_lanes() {
    let cargo_mk =
        fs::read_to_string(workspace_root().join("makes/cargo.mk")).expect("read makes/cargo.mk");

    for (target, delegate) in [
        ("fmt:", "$(MAKE) fmt-rs"),
        ("lint:", "$(MAKE) lint-rs"),
        ("audit:", "$(MAKE) audit-rs"),
        ("test:", "$(MAKE) test-rs"),
        ("test-all:", "$(MAKE) test-all-rs"),
    ] {
        let start = cargo_mk.find(target).expect("alias target");
        let tail = &cargo_mk[start..];
        let end = tail.find("\n\n").unwrap_or(tail.len());
        let target_block = &tail[..end];
        assert!(
            target_block.contains(delegate),
            "{target} should delegate to {delegate}"
        );
    }
}

#[test]
fn frozen_gate_targets_delegate_to_pinned_ref_launcher() {
    let cargo_mk =
        fs::read_to_string(workspace_root().join("makes/cargo.mk")).expect("read makes/cargo.mk");

    for (target, gate_target) in [
        ("test-all-frozen:", "PINNED_REF_GATE_TARGET=\"test-all\""),
        ("lint-frozen:", "PINNED_REF_GATE_TARGET=\"lint\""),
        ("audit-frozen:", "PINNED_REF_GATE_TARGET=\"audit\""),
    ] {
        let start = cargo_mk.find(target).expect("frozen target");
        let tail = &cargo_mk[start..];
        let end = tail.find("\n\n").unwrap_or(tail.len());
        let target_block = &tail[..end];
        assert!(
            target_block.contains("$(PINNED_REF_GATE_BIN)"),
            "{target} should use the pinned-ref launcher"
        );
        assert!(
            target_block.contains(gate_target),
            "{target} should set {gate_target}"
        );
    }
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
    let entrypoints_mk = fs::read_to_string(workspace_root().join("makes/entrypoints.mk"))
        .expect("read makes/entrypoints.mk");
    for marker in [
        "checks-group: ## Run one checks suite group (GROUP=<name>)",
        "checks-tag: ## Run checks suite entries with a shared tag (TAG=<name>)",
        "checks-pure: ## Run only pure checks suite entries",
        "checks-effect: ## Run only effectful checks suite entries",
    ] {
        let start = entrypoints_mk.find(marker).expect("target block");
        let tail = &entrypoints_mk[start..];
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
    let entrypoints_mk = fs::read_to_string(workspace_root().join("makes/entrypoints.mk"))
        .expect("read makes/entrypoints.mk");
    let start = entrypoints_mk
        .find("makes-target-list: ## Regenerate the makes public target list artifact")
        .expect("makes-target-list target");
    let tail = &entrypoints_mk[start..];
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
