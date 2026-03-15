use bijux_atlas::adapters::inbound::client::AtlasClient;
// SPDX-License-Identifier: Apache-2.0

use reqwest as _;
use serde as _;
use serde_json as _;
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let Some(workspace_root) = manifest_dir.parent() else {
        panic!(
            "missing workspace root for manifest dir: {}",
            manifest_dir.display()
        );
    };
    let Some(repo_root) = workspace_root.parent() else {
        panic!(
            "missing repository root for workspace dir: {}",
            workspace_root.display()
        );
    };
    repo_root.to_path_buf()
}

#[test]
fn runtime_compatibility_matrix_contains_v1() {
    let _ = std::mem::size_of::<Option<AtlasClient>>();
    let path = repo_root().join("ops/api/contracts/rust-client-runtime-compatibility.json");
    let matrix_json = match fs::read_to_string(&path) {
        Ok(value) => value,
        Err(error) => panic!(
            "failed to read compatibility matrix {}: {error}",
            path.display()
        ),
    };
    let value: serde_json::Value = match serde_json::from_str(&matrix_json) {
        Ok(value) => value,
        Err(error) => panic!(
            "failed to parse compatibility matrix {}: {error}",
            path.display()
        ),
    };
    assert_eq!(value["schema_version"], serde_json::json!(1));
    let Some(rows) = value["matrix"].as_array() else {
        panic!("compatibility matrix is missing array at `matrix`");
    };
    assert!(rows.iter().any(|row| row["api_version"] == "v1"));
}
