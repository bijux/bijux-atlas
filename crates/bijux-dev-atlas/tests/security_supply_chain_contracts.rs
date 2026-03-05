// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates root")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn run_json(args: &[&str]) -> serde_json::Value {
    let root = workspace_root();
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(args)
        .output()
        .expect("run command");
    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("parse json")
}

#[test]
fn security_validate_emits_dependency_and_vulnerability_reports() {
    let root = workspace_root();
    let result = run_json(&["security", "validate", "--format", "json"]);
    assert_eq!(result["status"], "ok");

    let dependency_inventory = root.join("artifacts/security/dependency-inventory.json");
    let vulnerability_scan = root.join("artifacts/security/security-vulnerability-scan.json");
    assert!(
        dependency_inventory.exists(),
        "missing dependency inventory artifact"
    );
    assert!(
        vulnerability_scan.exists(),
        "missing vulnerability scan artifact"
    );

    let dependency_value: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(dependency_inventory).expect("read dependency inventory"),
    )
    .expect("parse dependency inventory");
    let vulnerability_value: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(vulnerability_scan).expect("read vulnerability scan"),
    )
    .expect("parse vulnerability scan");

    assert_eq!(dependency_value["schema_version"], serde_json::json!(1));
    assert_eq!(dependency_value["status"], "ok");
    assert!(dependency_value["rows"].is_object());
    assert_eq!(vulnerability_value["schema_version"], serde_json::json!(1));
    assert_eq!(vulnerability_value["status"], "ok");
    assert!(vulnerability_value["rows"].is_array());
}

#[test]
fn dependency_audit_command_reports_health_summary() {
    let report = run_json(&["security", "dependency-audit", "--format", "json"]);
    assert_eq!(report["kind"], "security_dependency_audit_report");
    assert_eq!(report["status"], "ok");

    let summary = report["summary"].as_object().expect("summary object");
    assert!(summary.contains_key("dependency_inventory_rows"));
    assert!(summary.contains_key("vulnerability_rows"));
    assert!(summary.contains_key("workflow_action_rows"));
}
