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
fn shared_test_all_runs_ignored_tests_once_without_retries() {
    let root = workspace_root();
    let rust_gate =
        fs::read_to_string(root.join(".bijux/shared/bijux-makes-rs/scripts/rust_gate.sh"))
            .expect("read shared Rust gate");
    let nextest =
        fs::read_to_string(root.join("configs/rust/nextest.toml")).expect("read nextest policy");

    assert!(
        rust_gate.contains("args+=(--run-ignored all --retries 0)"),
        "test-all must include ignored tests and disable retries"
    );
    assert!(
        rust_gate.contains("\"${NEXTEST_THREADS_ALL:-}\""),
        "test-all must honor the governed full-suite concurrency setting"
    );
    let full_profile = nextest
        .split("[profile.full]")
        .nth(1)
        .expect("full nextest profile")
        .split("\n[")
        .next()
        .expect("full nextest profile body");
    assert!(
        full_profile.contains("fail-fast = false"),
        "test-all must continue after individual test failures"
    );
}

#[test]
fn shared_test_gate_preserves_nextest_summary_and_exit_status() {
    let rust_gate = fs::read_to_string(
        workspace_root().join(".bijux/shared/bijux-makes-rs/scripts/rust_gate.sh"),
    )
    .expect("read shared Rust gate");

    for expected in [
        "if run_logged \"${report_path}\" env \\",
        "status=$?",
        "\"nextest-summary:\"",
        "return \"${status}\"",
    ] {
        assert!(
            rust_gate.contains(expected),
            "Rust gate must preserve `{expected}` after failed nextest runs"
        );
    }
}

#[test]
fn cargo_gate_module_declares_standardized_rust_gate_wiring() {
    let root_mk =
        fs::read_to_string(workspace_root().join("makes/root.mk")).expect("read makes/root.mk");
    let cargo_mk =
        fs::read_to_string(workspace_root().join("makes/cargo.mk")).expect("read makes/cargo.mk");

    for expected in [
        "bijux-makes/environment.mk",
        "bijux-makes/guards.mk",
        "bijux-makes-rs/bijux.mk",
    ] {
        assert!(
            root_mk.contains(expected),
            "root Make entrypoint must load `{expected}`"
        );
    }

    for expected in [
        "NEXTEST_PROFILE_FAST ?= fast-unit",
        "NEXTEST_PROFILE_SLOW ?= slow-integration",
        "NEXTEST_PROFILE_ALL ?= full",
        "NEXTEST_SLOW_NAME_EXPR ?= test(/::slow__/)",
        "RUST_GATE_BIN ?= $(ATLAS_RUST_GATE_BIN)",
        "RUST_AUDIT_PREREQUISITES += audit-policy-rs",
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
        ("fmt:", "fmt-rs"),
        ("lint:", "lint-rs"),
        ("audit:", "audit-rs"),
        ("test:", "test-rs"),
        ("test-slow:", "test-slow-rs"),
        ("test-all:", "test-all-rs"),
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
    let root = workspace_root();
    let shared_cargo = fs::read_to_string(root.join(".bijux/shared/bijux-makes-rs/cargo.mk"))
        .expect("read shared Rust Make contract");
    let pinned_gate =
        fs::read_to_string(root.join(".bijux/shared/bijux-makes/scripts/run_pinned_gate.sh"))
            .expect("read shared pinned gate");

    for (target, gate_target) in [
        ("test-all-frozen:", "PINNED_GATE_TARGET=test-all"),
        ("lint-frozen:", "PINNED_GATE_TARGET=lint"),
        ("audit-frozen:", "PINNED_GATE_TARGET=audit"),
    ] {
        assert!(
            shared_cargo.contains(target) && shared_cargo.contains(gate_target),
            "{target} must use the shared pinned gate with {gate_target}"
        );
    }
    for expected in [
        "pinned_ref=\"${PINNED_REF:-${TEST_ALL_FROZEN_REF:-HEAD}}\"",
        "unset \\",
        "PROJECT_ROOT \\",
        "export PROJECT_ROOT=\"${pinned_repo_dir}\"",
        "artifact_execution_root=\"${pinned_repo_dir}/artifacts\"",
        "export ARTIFACT_ROOT=\"${artifact_execution_root}\"",
        "expected_target=\"frozen-repo/artifacts/",
        "artifact publication conflict:",
        "ln -s ",
    ] {
        assert!(
            pinned_gate.contains(expected),
            "frozen gate must preserve `{expected}`"
        );
    }
}

#[test]
fn slow_test_policy_uses_roster_and_double_underscore_namespace() {
    let root = workspace_root();
    let roster = fs::read_to_string(root.join("configs/rust/nextest-slow-roster.txt"))
        .expect("read slow-test roster");
    assert!(roster.contains("slow__"));

    for relative_path in [
        "crates/bijux-atlas-server/src/app/server/tests/cache_lifecycle.rs",
        "crates/bijux-atlas-cli/tests/cli_runtime_parity.rs",
        "crates/bijux-atlas-dev/tests/cli_smoke.rs",
        "crates/bijux-atlas-dev/tests/support/core_engine_tests.rs",
    ] {
        let source = fs::read_to_string(root.join(relative_path)).expect("read slow-test source");
        assert!(
            !source.lines().any(|line| {
                let line = line.trim_start();
                line.starts_with("fn slow_") && !line.starts_with("fn slow__")
                    || line.starts_with("async fn slow_") && !line.starts_with("async fn slow__")
            }),
            "{relative_path} contains a slow test outside the slow__ namespace"
        );
    }
}

#[test]
fn repository_adapter_replaces_copied_gate_implementations() {
    let root = workspace_root();
    let adapter = fs::read_to_string(root.join("makes/bin/run_atlas_rust_gate.sh"))
        .expect("read Atlas Rust gate adapter");
    assert!(adapter.contains(".bijux/shared/bijux-makes-rs/scripts/rust_gate.sh"));
    assert!(adapter.contains("exec \"${shared_gate}\" \"$@\""));

    for relative_path in [
        "makes/bin/nextest_expr.sh",
        "makes/bin/rust_gate.sh",
        "makes/bin/run_pinned_ref_gate.sh",
    ] {
        assert!(
            !root.join(relative_path).exists(),
            "shared Make integration must not retain {relative_path}"
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
