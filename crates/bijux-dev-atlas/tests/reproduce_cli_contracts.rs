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
    assert_eq!(payload.get("schema_version").and_then(|v| v.as_u64()), Some(1));
    assert!(payload
        .get("environment")
        .and_then(|v| v.get("source_snapshot_hash"))
        .and_then(|v| v.as_str())
        .is_some());
}

#[test]
fn reproduce_verify_requires_all_core_scenarios() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["reproduce", "verify", "--format", "json"])
        .output()
        .expect("reproduce verify");
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(payload.get("kind").and_then(|v| v.as_str()), Some("reproduce_verify"));
    assert_eq!(payload.get("status").and_then(|v| v.as_str()), Some("ok"));
}
