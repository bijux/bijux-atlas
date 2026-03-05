use criterion as _;
// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_client::{ClientMetrics, InMemoryMetrics};
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
    let path = repo_root().join("crates/bijux-atlas-client/config/client-config.schema.json");
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(path).expect("schema read"))
            .expect("schema parse");
    assert_eq!(
        value["title"],
        serde_json::json!("Atlas Rust Client Configuration")
    );
}
