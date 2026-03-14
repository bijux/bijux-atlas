use criterion as _;
// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::client::{ClientMetrics, InMemoryMetrics};
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
fn metrics_export_emits_expected_shape() {
    let metrics = InMemoryMetrics::default();
    metrics.observe_request("/v1/genes", 11, true);
    let payload = metrics.export_json();
    assert_eq!(payload["schema_version"], serde_json::json!(1));
    assert_eq!(
        payload["kind"],
        serde_json::json!("rust_client_request_metrics")
    );
}

#[test]
fn client_config_schema_is_parseable() {
    let path = repo_root().join("configs/clients/atlas-rust-client-config.schema.json");
    let schema = match fs::read_to_string(&path) {
        Ok(value) => value,
        Err(error) => panic!("failed to read schema {}: {error}", path.display()),
    };
    let value: serde_json::Value = match serde_json::from_str(&schema) {
        Ok(value) => value,
        Err(error) => panic!("failed to parse schema {}: {error}", path.display()),
    };
    assert_eq!(
        value["title"],
        serde_json::json!("Atlas Rust Client Configuration")
    );
}
