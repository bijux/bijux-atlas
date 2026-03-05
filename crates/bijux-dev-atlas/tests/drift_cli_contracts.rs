// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn drift_explain_registry_returns_schema_and_detectors() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["drift", "explain", "registry", "--format", "json"])
        .output()
        .expect("drift explain");
    assert_eq!(output.status.code(), Some(0));
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(
        payload.get("schema_version").and_then(|v| v.as_u64()),
        Some(1)
    );
    assert_eq!(payload.get("status").and_then(|v| v.as_str()), Some("ok"));
}

#[test]
fn drift_detect_report_is_deterministic_for_same_state() {
    let args = ["drift", "detect", "--format", "json"];
    let first = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(args)
        .output()
        .expect("first");
    let second = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(args)
        .output()
        .expect("second");
    assert_eq!(first.status.code(), second.status.code());
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn drift_baseline_and_compare_workflow() {
    let temp = TempDir::new().expect("tempdir");
    let baseline = temp.path().join("baseline.json");

    let baseline_out = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "drift",
            "baseline",
            "--format",
            "json",
            "--snapshot-out",
            baseline.to_str().expect("utf8"),
        ])
        .output()
        .expect("baseline");
    assert!(baseline_out.status.code().is_some());
    assert!(baseline.exists());

    let compare_out = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "drift",
            "compare",
            "--format",
            "json",
            "--baseline",
            baseline.to_str().expect("utf8"),
        ])
        .output()
        .expect("compare");
    let payload: serde_json::Value = serde_json::from_slice(&compare_out.stdout).expect("json");
    assert_eq!(
        payload.get("kind").and_then(|v| v.as_str()),
        Some("drift_compare")
    );
}

#[test]
fn drift_ignore_contract_rejects_invalid_schema_version() {
    let temp = TempDir::new().expect("tempdir");
    let ignore = temp.path().join("ignore-invalid.json");
    std::fs::write(
        &ignore,
        r#"{"schema_version":2,"ignores":[{"drift_type":"registry"}]}"#,
    )
    .expect("write ignore");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "drift",
            "detect",
            "--format",
            "json",
            "--ignore-file",
            ignore.to_str().expect("utf8"),
        ])
        .output()
        .expect("drift detect");
    assert_eq!(output.status.code(), Some(1));
}
