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
fn audit_run_emits_expected_checks() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["audit", "run", "--format", "json"])
        .output()
        .expect("audit run");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(bytes).expect("json output");
    assert_eq!(
        payload.get("kind").and_then(|v| v.as_str()),
        Some("audit_run")
    );
    assert_eq!(
        payload
            .get("checks")
            .and_then(|v| v.as_array())
            .map(|v| v.len()),
        Some(5)
    );
}

#[test]
fn audit_report_wraps_run_report() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["audit", "report", "--format", "json"])
        .output()
        .expect("audit report");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(bytes).expect("json output");
    assert_eq!(
        payload.get("kind").and_then(|v| v.as_str()),
        Some("audit_report")
    );
    assert_eq!(
        payload
            .get("report")
            .and_then(|v| v.get("kind"))
            .and_then(|v| v.as_str()),
        Some("audit_run")
    );
}

#[test]
fn audit_explain_lists_commands_and_schemas() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["audit", "explain", "--format", "json"])
        .output()
        .expect("audit explain");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json output");
    assert_eq!(
        payload.get("kind").and_then(|v| v.as_str()),
        Some("audit_explain")
    );
    assert!(payload.get("schemas").is_some());
}
