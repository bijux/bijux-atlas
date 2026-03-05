// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

#[test]
fn governance_adr_index_command_emits_registry() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .args(["governance", "adr", "index", "--format", "json"])
        .output()
        .expect("run command");

    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse governance adr index");
    assert_eq!(value["kind"], "governance_adr_index");
    assert_eq!(value["status"], "ok");
    assert!(
        value["entries"]
            .as_array()
            .is_some_and(|rows| !rows.is_empty()),
        "expected at least one adr entry"
    );
}
