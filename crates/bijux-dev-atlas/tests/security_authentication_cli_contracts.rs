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
fn security_authentication_cli_commands_emit_reports() {
    let api_keys = run(&["security", "authentication", "api-keys", "--format", "json"]);
    assert_eq!(api_keys["kind"], "authentication_api_key_management_report");

    let diagnostics = run(&[
        "security",
        "authentication",
        "diagnostics",
        "--format",
        "json",
    ]);
    assert_eq!(diagnostics["kind"], "authentication_diagnostics_report");

    let policy_validate = run(&[
        "security",
        "authentication",
        "policy-validate",
        "--format",
        "json",
    ]);
    assert_eq!(
        policy_validate["kind"],
        "authentication_policy_validation_report"
    );
    assert_eq!(policy_validate["status"], "ok");
    assert!(
        policy_validate["auth_methods"].as_u64().unwrap_or_default() >= 1,
        "expected auth method count"
    );
}

#[test]
fn security_authentication_token_inspect_reports_claims() {
    let token = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJzdmMtY2kiLCJpc3MiOiJhdGxhcy1hdXRoIiwiYXVkIjoiYXRsYXMtYXBpIiwiZXhwIjo0MTAyNDQ0ODAwLCJqdGkiOiJ0MSIsInNjb3BlIjoiZGF0YXNldC5yZWFkIG9wcy5hZG1pbiJ9.signature";
    let report = run(&[
        "security",
        "authentication",
        "token-inspect",
        "--token",
        token,
        "--format",
        "json",
    ]);

    assert_eq!(report["kind"], "authentication_token_inspection_report");
    assert_eq!(report["subject"], "svc-ci");
    assert_eq!(report["issuer"], "atlas-auth");
    assert_eq!(report["audience"], "atlas-api");
}
