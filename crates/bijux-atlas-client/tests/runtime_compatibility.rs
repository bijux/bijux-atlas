use bijux_atlas_client as _;
use criterion as _;
// SPDX-License-Identifier: Apache-2.0

use reqwest as _;
use serde as _;
use serde_json as _;
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn runtime_compatibility_matrix_contains_v1() {
    let value: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            repo_root().join("ops/api/contracts/rust-client-runtime-compatibility.json"),
        )
        .expect("read compatibility matrix"),
    )
    .expect("parse compatibility matrix");
    assert_eq!(value["schema_version"], serde_json::json!(1));
    let rows = value["matrix"].as_array().expect("matrix array");
    assert!(rows.iter().any(|row| row["api_version"] == "v1"));
}
