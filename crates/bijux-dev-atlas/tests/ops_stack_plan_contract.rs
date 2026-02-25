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
fn stack_plan_kind_matches_stack_manifest_components() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "ops",
            "stack",
            "plan",
            "--profile",
            "kind",
            "--format",
            "json",
        ])
        .output()
        .expect("ops stack plan");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    let row = payload
        .get("rows")
        .and_then(|v| v.as_array())
        .and_then(|rows| rows.first())
        .expect("one row");
    let resources = row
        .get("resources")
        .and_then(|v| v.as_array())
        .expect("resources");
    let expected = vec![
        "ops/k8s/charts/bijux-atlas/Chart.yaml",
        "ops/observe/pack/k8s/namespace.yaml",
        "ops/stack/grafana/grafana.yaml",
        "ops/stack/minio/minio.yaml",
        "ops/stack/otel/otel-collector.yaml",
        "ops/stack/prometheus/prometheus.yaml",
        "ops/stack/redis/redis.yaml",
    ];
    let actual: Vec<&str> = resources.iter().filter_map(|v| v.as_str()).collect();
    assert_eq!(actual, expected);
}
