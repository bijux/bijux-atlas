// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

fn run(args: &[&str]) -> serde_json::Value {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .args(args)
        .output()
        .expect("run command");
    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("parse json output")
}

#[test]
fn security_authorization_cli_commands_emit_reports() {
    let roles = run(&["security", "authorization", "roles", "--format", "json"]);
    assert_eq!(roles["kind"], "authorization_role_management_report");
    assert!(roles["role_count"].as_u64().unwrap_or_default() >= 1);

    let permissions = run(&[
        "security",
        "authorization",
        "permissions",
        "--format",
        "json",
    ]);
    assert_eq!(
        permissions["kind"],
        "authorization_permission_inspection_report"
    );

    let diagnostics = run(&[
        "security",
        "authorization",
        "diagnostics",
        "--format",
        "json",
    ]);
    assert_eq!(diagnostics["kind"], "authorization_diagnostics_report");
    assert!(diagnostics["assignment_count"].as_u64().unwrap_or_default() >= 1);
    assert_eq!(diagnostics["default_decision"], "deny");

    let validate = run(&["security", "authorization", "validate", "--format", "json"]);
    assert_eq!(
        validate["kind"],
        "authorization_permission_validation_report"
    );
    assert_eq!(validate["status"], "ok");
}

#[test]
fn security_policy_inspector_returns_policy_rows() {
    let report = run(&[
        "security",
        "policy-inspect",
        "--policy-id",
        "AUTH-POLICY-READ",
        "--format",
        "json",
    ]);
    assert_eq!(report["kind"], "security_policy_inspection_report");
    assert_eq!(report["policy_filter"], "AUTH-POLICY-READ");
    let policies = report["policies"].as_array().expect("policies array");
    assert_eq!(policies.len(), 1);
}
