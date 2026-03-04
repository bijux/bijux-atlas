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

#[test]
fn ops_profile_policy_validate_supports_json_without_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "ops",
            "profiles",
            "policy-validate",
            "--profile",
            "dev",
            "--format",
            "json",
        ])
        .output()
        .expect("policy-validate");
    assert!(output.status.success());
    let stdout: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(stdout["kind"], "ops_policy_validate");
}

#[test]
fn ops_profile_resource_validate_supports_json_without_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "ops",
            "profiles",
            "resource-validate",
            "--profile",
            "dev",
            "--format",
            "json",
        ])
        .output()
        .expect("resource-validate");
    assert!(output.status.success());
    let stdout: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(stdout["kind"], "ops_resource_validate");
}

#[test]
fn ops_profile_securitycontext_validate_supports_json_without_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "ops",
            "profiles",
            "securitycontext-validate",
            "--profile",
            "dev",
            "--format",
            "json",
        ])
        .output()
        .expect("securitycontext-validate");
    assert!(output.status.success());
    let stdout: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(stdout["kind"], "ops_securitycontext_validate");
}

#[test]
fn ops_profile_service_monitor_validate_supports_json_without_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "ops",
            "profiles",
            "service-monitor-validate",
            "--profile",
            "dev",
            "--format",
            "json",
        ])
        .output()
        .expect("service-monitor-validate");
    assert!(output.status.success());
    let stdout: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(stdout["kind"], "ops_service_monitor_validate");
}

#[test]
fn ops_profile_hpa_validate_supports_json_without_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "ops",
            "profiles",
            "hpa-validate",
            "--profile",
            "dev",
            "--format",
            "json",
        ])
        .output()
        .expect("hpa-validate");
    assert!(output.status.success());
    let stdout: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(stdout["kind"], "ops_hpa_validate");
}
