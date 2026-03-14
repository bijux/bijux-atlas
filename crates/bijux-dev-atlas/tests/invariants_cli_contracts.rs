// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("mkdir");
    }
    fs::write(path, content).expect("write");
}

fn fixture_repo_with_profile_cycle() -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    let root = dir.path();

    write(
        &root.join("configs/inventory.json"),
        r#"{"schema_version":1}"#,
    );
    write(
        &root.join("ops/release/evidence/manifest.json"),
        r#"{"artifact_list":[]}"#,
    );
    write(
        &root.join("ops/datasets/generated/dataset-index.json"),
        r#"{"schema_version":1,"dataset_ids":["ds1"]}"#,
    );
    write(
        &root.join("ops/datasets/manifest.lock"),
        r#"{"schema_version":1}"#,
    );
    write(
        &root.join("ops/k8s/values/offline.yaml"),
        "cache:\n  pinnedDatasets: []\n",
    );
    write(
        &root.join("ops/stack/profile-registry.json"),
        r#"{"profiles":[{"id":"local"}]}"#,
    );
    write(
        &root.join("ops/k8s/install-matrix.json"),
        r#"{"profiles":[{"name":"local"}]}"#,
    );
    write(
        &root.join("ops/k8s/values/profiles.json"),
        r#"{"profiles":[{"id":"local","inherits_from":"hardened","values_file":"ops/k8s/values/local.yaml"},{"id":"hardened","inherits_from":"local","values_file":"ops/k8s/values/hardened.yaml"}]}"#,
    );
    write(&root.join("ops/k8s/values/local.yaml"), "replicaCount: 1\n");
    write(
        &root.join("ops/k8s/values/hardened.yaml"),
        "replicaCount: 2\n",
    );

    let registry = fs::read_to_string(repo_root().join("ops/invariants/registry.json"))
        .expect("source registry");
    write(&root.join("ops/invariants/registry.json"), &registry);
    dir
}

#[test]
fn invariants_fail_with_stable_exit_code_when_contracts_break() {
    let fixture = fixture_repo_with_profile_cycle();
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "invariants",
            "run",
            "--repo-root",
            fixture.path().to_str().expect("utf8"),
            "--format",
            "json",
        ])
        .output()
        .expect("invariants run");

    assert_eq!(output.status.code(), Some(3));
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(bytes).expect("json");
    let runtime_gate = payload
        .get("results")
        .and_then(serde_json::Value::as_array)
        .and_then(|rows| {
            rows.iter().find(|row| {
                row.get("id").and_then(serde_json::Value::as_str)
                    == Some("INV-RUNTIME-START-GATE-001")
            })
        })
        .expect("runtime gate row");
    assert_eq!(
        runtime_gate
            .get("status")
            .and_then(serde_json::Value::as_str),
        Some("fail")
    );
}

#[test]
fn invariants_report_is_deterministic_for_same_repo_state() {
    let fixture = fixture_repo_with_profile_cycle();
    let args = [
        "invariants",
        "run",
        "--repo-root",
        fixture.path().to_str().expect("utf8"),
        "--format",
        "json",
    ];

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

    assert_eq!(first.status.code(), Some(3));
    assert_eq!(second.status.code(), Some(3));
    assert_eq!(first.stdout, second.stdout);
}
