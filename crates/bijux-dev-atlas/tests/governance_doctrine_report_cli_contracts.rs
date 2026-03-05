// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

#[test]
fn governance_doctrine_report_emits_envelope() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .args(["governance", "doctrine-report", "--format", "json"])
        .output()
        .expect("run doctrine report command");

    let raw = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(raw).expect("parse doctrine report");
    assert_eq!(payload["kind"], "doctrine_compliance");
    assert!(
        payload["report_path"].as_str().is_some_and(
            |path| path.contains("artifacts/governance/doctrine-compliance-report.json")
        ),
        "doctrine report path must be emitted"
    );
}
