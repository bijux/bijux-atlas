// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

#[test]
fn runtime_command_surface_exposes_schema_and_self_check() {
    let out = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .args(["runtime", "--help"])
        .output()
        .expect("run runtime help");
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("self-check"));
    assert!(text.contains("print-config-schema"));
    assert!(text.contains("explain-config-schema"));
}

#[test]
fn runtime_self_check_returns_ok_payload() {
    let out = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .args(["runtime", "self-check", "--format", "json"])
        .output()
        .expect("run self-check");
    assert!(out.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json payload");
    assert_eq!(payload["kind"], "runtime_self_check");
    assert_eq!(payload["status"], "ok");
}
