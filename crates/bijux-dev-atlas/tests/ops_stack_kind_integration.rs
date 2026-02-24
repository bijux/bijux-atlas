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
            "--allow-subprocess",
            "--format",
            "json",
        ])
        .output()
        .expect("ops stack status");
    assert!(output.status.success());
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
            "--allow-subprocess",
            "--format",
            "json",
        ])
        .output()
        .expect("ops k8s conformance");
    assert!(output.status.success());
}
