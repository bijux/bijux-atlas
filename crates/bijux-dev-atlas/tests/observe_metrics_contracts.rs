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
fn observe_metrics_list_emits_registry_snapshot() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["observe", "metrics", "list", "--format", "json"])
        .output()
        .expect("observe metrics list");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(bytes).expect("json output");
    assert_eq!(
        payload.get("kind").and_then(|v| v.as_str()),
        Some("observe_metrics_list")
    );
    assert!(payload.get("rows").and_then(|v| v.as_array()).is_some());
    assert!(payload
        .get("artifacts")
        .and_then(|v| v.get("metrics_registry_snapshot"))
        .is_some());
    assert_eq!(
        payload
            .get("budget_validation_errors")
            .and_then(|v| v.as_array())
            .map(|v| v.len()),
        Some(0)
    );
}

#[test]
fn observe_metrics_explain_works_for_known_metric() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "observe",
            "metrics",
            "explain",
            "MET-RUNTIME-UPTIME-001",
            "--format",
            "json",
        ])
        .output()
        .expect("observe metrics explain");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json output");
    assert_eq!(payload.get("status").and_then(|v| v.as_str()), Some("ok"));
}

#[test]
fn observe_metrics_docs_writes_reference_page() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["observe", "metrics", "docs", "--format", "json"])
        .output()
        .expect("observe metrics docs");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json output");
    assert_eq!(
        payload.get("kind").and_then(|v| v.as_str()),
        Some("observe_metrics_docs")
    );
}
