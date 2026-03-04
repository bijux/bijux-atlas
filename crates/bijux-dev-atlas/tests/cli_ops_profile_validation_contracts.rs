// SPDX-License-Identifier: Apache-2.0

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
fn ops_profile_schema_validate_requires_allow_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "profiles", "schema-validate", "--format", "json"])
        .output()
        .expect("schema-validate");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr");
    assert!(stderr.contains("requires --allow-subprocess"));
}

#[test]
fn ops_profile_kubeconform_requires_allow_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "profiles", "kubeconform", "--format", "json"])
        .output()
        .expect("kubeconform");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr");
    assert!(stderr.contains("requires --allow-subprocess"));
}

#[test]
fn ops_profile_rollout_safety_validate_requires_allow_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "ops",
            "profiles",
            "rollout-safety-validate",
            "--format",
            "json",
        ])
        .output()
        .expect("rollout-safety-validate");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr");
    assert!(stderr.contains("requires --allow-subprocess"));
}
