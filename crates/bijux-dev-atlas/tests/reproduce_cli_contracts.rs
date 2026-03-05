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
fn reproduce_run_emits_source_snapshot_hash() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["reproduce", "run", "--format", "json"])
        .output()
        .expect("reproduce run");
    assert_eq!(output.status.code(), Some(0));
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(
        payload.get("schema_version").and_then(|v| v.as_u64()),
        Some(1)
    );
    assert!(payload
        .get("environment")
        .and_then(|v| v.get("source_snapshot_hash"))
        .and_then(|v| v.as_str())
        .is_some());
}

#[test]
fn slow_reproduce_verify_requires_all_core_scenarios() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["reproduce", "verify", "--format", "json"])
        .output()
        .expect("reproduce verify");
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(
        payload.get("kind").and_then(|v| v.as_str()),
        Some("reproduce_verify")
    );
    assert_eq!(payload.get("status").and_then(|v| v.as_str()), Some("ok"));
}

#[test]
fn slow_reproduce_reports_are_deterministic_and_include_artifact_hashes() {
    let args = ["reproduce", "run", "--format", "json"];
    let first = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(args)
        .output()
        .expect("first");
    let second = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(args)
        .output()
        .expect("second");
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    let first_json: serde_json::Value = serde_json::from_slice(&first.stdout).expect("first json");
    let second_json: serde_json::Value =
        serde_json::from_slice(&second.stdout).expect("second json");
    assert_eq!(
        first_json
            .get("environment")
            .and_then(|v| v.get("source_snapshot_hash")),
        second_json
            .get("environment")
            .and_then(|v| v.get("source_snapshot_hash"))
    );
    assert!(first_json
        .get("artifact_hashes")
        .and_then(serde_json::Value::as_object)
        .is_some());
}

#[test]
fn slow_reproduce_status_reports_summary_shape() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["reproduce", "status", "--format", "json"])
        .output()
        .expect("reproduce status");
    assert!(output.status.code().is_some());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(
        payload.get("kind").and_then(|v| v.as_str()),
        Some("reproduce_status")
    );
    assert!(payload.get("verify").is_some());
}

#[test]
fn slow_reproduce_audit_metrics_lineage_and_summary_emit_expected_kinds() {
    let commands = [
        ("audit-report", "reproducibility_audit_report"),
        ("metrics", "reproducibility_metrics"),
        ("lineage-validate", "reproducibility_lineage_validate"),
        ("summary-table", "reproducibility_summary_table"),
    ];
    for (subcommand, kind) in commands {
        let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
            .current_dir(repo_root())
            .args(["reproduce", subcommand, "--format", "json"])
            .output()
            .expect("repro command");
        assert!(
            output.status.code().is_some(),
            "command {subcommand} did not exit cleanly"
        );
        let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
        assert_eq!(payload.get("kind").and_then(|v| v.as_str()), Some(kind));
    }
}
