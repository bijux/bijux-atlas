// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "kind_integration")]

use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

fn assert_success_or_environment_skip(output: &std::process::Output, test_name: &str) {
    if output.status.success() {
        return;
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let skip_markers = [
        "namespace guard failed",
        "kubectl context guard failed",
        "no kind cluster",
        "namespaces \"bijux-atlas\" not found",
    ];
    if skip_markers.iter().any(|marker| stderr.contains(marker)) {
        eprintln!("skipping {test_name}: {stderr}");
        return;
    }
    panic!(
        "{test_name} failed with non-skippable error: {}",
        stderr.trim()
    );
}

#[test]
#[ignore = "requires local kind+kubectl toolchain and network access"]
fn stack_status_kind_profile_k8s_target() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "ops",
            "stack",
            "status",
            "--profile",
            "kind",
            "--force",
            "--allow-subprocess",
            "--format",
            "json",
        ])
        .output()
        .expect("ops stack status");
    assert_success_or_environment_skip(&output, "stack_status_kind_profile_k8s_target");
}

#[test]
#[ignore = "requires local kind+kubectl toolchain and network access"]
fn k8s_conformance_kind_profile() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "ops",
            "k8s",
            "conformance",
            "--profile",
            "kind",
            "--force",
            "--allow-subprocess",
            "--format",
            "json",
        ])
        .output()
        .expect("ops k8s conformance");
    assert_success_or_environment_skip(&output, "k8s_conformance_kind_profile");
}
