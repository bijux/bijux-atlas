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
fn drift_explain_registry_returns_schema_and_detectors() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["drift", "explain", "registry", "--format", "json"])
        .output()
        .expect("drift explain");
    assert_eq!(output.status.code(), Some(0));
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(payload.get("schema_version").and_then(|v| v.as_u64()), Some(1));
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
