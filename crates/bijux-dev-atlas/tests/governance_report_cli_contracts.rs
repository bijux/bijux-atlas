// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

fn parse_payload(output: &std::process::Output) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&stdout) {
        return value;
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    serde_json::from_str::<serde_json::Value>(&stderr).expect("parse json payload from stderr")
}

#[test]
fn governance_report_command_emits_health_summary() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .args(["governance", "report", "--format", "json"])
        .output()
        .expect("run command");

    let value = parse_payload(&output);
    assert_eq!(value["kind"], "governance_report");
    assert!(value["status"].is_string());
    assert!(value["report_path"].as_str().is_some());
}

#[test]
fn governance_validate_includes_contributor_and_docs_sections() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .args(["governance", "validate", "--format", "json"])
        .output()
        .expect("run command");

    let value = parse_payload(&output);
    assert_eq!(value["kind"], "governance_validate");
    assert!(value["governance_docs_validation"].is_object());
    assert!(value["contributor_guidelines_validation"].is_object());
}
