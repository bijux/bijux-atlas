// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

#[test]
fn security_dependency_audit_command_emits_audit_report() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .args(["security", "dependency-audit", "--format", "json"])
        .output()
        .expect("run command");

    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse dependency audit report");
    assert_eq!(value["kind"], "security_dependency_audit_report");
    assert_eq!(value["status"], "ok");
}
