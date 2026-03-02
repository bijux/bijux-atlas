// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::Path;

use serde_json::json;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

fn read(path: &str) -> String {
    fs::read_to_string(repo_root().join(path)).expect("read fixture")
}

#[test]
fn render_failure_snapshot_matches_golden() {
    let actual = serde_json::to_string_pretty(&json!({
        "check_id": "OPS-PROFILES-001",
        "status": "fail",
        "failing_profiles": ["ci"],
    }))
    .expect("json")
        + "\n";
    let expected =
        read("crates/bijux-dev-atlas/tests/goldens/ops_profiles_matrix_render_fail.json");
    assert_eq!(actual, expected);
}

#[test]
fn schema_failure_snapshot_matches_golden() {
    let actual = serde_json::to_string_pretty(&json!({
        "check_id": "OPS-PROFILES-002",
        "status": "fail",
        "failing_profiles": ["perf"],
    }))
    .expect("json")
        + "\n";
    let expected =
        read("crates/bijux-dev-atlas/tests/goldens/ops_profiles_matrix_schema_fail.json");
    assert_eq!(actual, expected);
}

#[test]
fn kubeconform_failure_snapshot_matches_golden() {
    let actual = serde_json::to_string_pretty(&json!({
        "check_id": "OPS-PROFILES-003",
        "status": "fail",
        "failing_profiles": ["prod"],
    }))
    .expect("json")
        + "\n";
    let expected =
        read("crates/bijux-dev-atlas/tests/goldens/ops_profiles_matrix_kubeconform_fail.json");
    assert_eq!(actual, expected);
}

#[test]
fn rollout_safety_failure_snapshot_matches_golden() {
    let actual = serde_json::to_string_pretty(&json!({
        "check_id": "OPS-PROFILES-004",
        "status": "fail",
        "failing_profiles": ["offline"],
    }))
    .expect("json")
        + "\n";
    let expected =
        read("crates/bijux-dev-atlas/tests/goldens/ops_profiles_matrix_rollout_safety_fail.json");
    assert_eq!(actual, expected);
}

#[test]
fn pass_snapshot_matches_golden() {
    let actual = serde_json::to_string_pretty(&json!({
        "check_id": "OPS-PROFILES-001",
        "status": "pass",
        "failing_profiles": [],
    }))
    .expect("json")
        + "\n";
    let expected = read("crates/bijux-dev-atlas/tests/goldens/ops_profiles_matrix_pass.json");
    assert_eq!(actual, expected);
}

#[test]
fn profile_matrix_regression_fixtures_exist() {
    let root = Path::new("crates/bijux-dev-atlas/tests/fixtures/ops_profiles_matrix");
    let guard = read(
        root.join("guard-failure-profile.yaml")
            .to_str()
            .expect("path"),
    );
    let schema = read(
        root.join("schema-failure-profile.yaml")
            .to_str()
            .expect("path"),
    );
    let invalid = read(
        root.join("invalid-resource-manifest.yaml")
            .to_str()
            .expect("path"),
    );
    assert!(guard.contains("cachedOnlyMode: true"));
    assert!(guard.contains("readinessRequiresCatalog: true"));
    assert!(schema.contains("digest: sha256:"));
    assert!(schema.contains("tag:"));
    assert!(invalid.contains("kind: Service"));
    assert!(invalid.contains("ports:"));
}
