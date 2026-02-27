// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn ops_contract_registry_snapshot_matches_code() {
    let root = workspace_root();
    let expected = bijux_dev_atlas::contracts::ops::render_registry_snapshot_json(&root)
        .expect("render ops contract registry snapshot");
    let snapshot_path = root.join("ops/_generated.example/contracts-registry-snapshot.json");
    let snapshot_text = fs::read_to_string(&snapshot_path).expect("read committed ops snapshot");
    let actual: serde_json::Value = serde_json::from_str(&snapshot_text).expect("parse snapshot");
    assert_eq!(actual, expected, "ops contract registry snapshot drift detected");
}
