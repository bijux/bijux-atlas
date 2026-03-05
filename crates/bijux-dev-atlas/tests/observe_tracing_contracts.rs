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
fn observe_traces_explain_emits_tracing_contract() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["observe", "traces", "explain", "--format", "json"])
        .output()
        .expect("observe traces explain");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json output");
    assert_eq!(
        payload.get("kind").and_then(|v| v.as_str()),
        Some("observe_traces_explain")
    );
    let spans = payload
        .get("contract")
        .and_then(|v| v.get("span_registry"))
        .and_then(|v| v.as_array())
        .expect("span registry array");
    assert!(spans.len() >= 10);
}
