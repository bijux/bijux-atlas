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
fn ops_profile_list_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "profile", "list", "--format", "json"])
        .output()
        .expect("ops profile list");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(payload["schema_version"], serde_json::json!(1));
    assert!(payload["rows"]
        .as_array()
        .is_some_and(|rows| !rows.is_empty()));
}

#[test]
fn ops_profile_explain_shows_invariants_for_selected_profile() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "profile", "explain", "kind", "--format", "json"])
        .output()
        .expect("ops profile explain");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    let row = payload["rows"]
        .as_array()
        .and_then(|rows| rows.first())
        .cloned()
        .expect("row");
    assert_eq!(row["id"], serde_json::json!("kind"));
    assert!(row["required_tools"].as_array().is_some());
    assert!(row["required_services"].as_array().is_some());
}
