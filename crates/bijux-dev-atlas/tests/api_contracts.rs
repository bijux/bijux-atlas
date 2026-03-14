// SPDX-License-Identifier: Apache-2.0

use std::fs;
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
fn api_list_exposes_surface_registry() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["api", "list", "--format", "json"])
        .output()
        .expect("api list");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json output");
    assert_eq!(payload["kind"], serde_json::json!("api_list"));
    assert!(payload["registry"]["endpoints"].is_array());
}

#[test]
fn api_explain_returns_endpoint_metadata() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["api", "explain", "/v1/genes", "--format", "json"])
        .output()
        .expect("api explain");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json output");
    assert_eq!(payload["kind"], serde_json::json!("api_explain"));
    assert_eq!(payload["status"], serde_json::json!("ok"));
}

#[test]
fn api_diff_reports_openapi_changes_from_fixture_baseline() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "api",
            "diff",
            "--baseline",
            "crates/bijux-dev-atlas/tests/fixtures/api/openapi-regression-baseline.json",
            "--format",
            "json",
        ])
        .output()
        .expect("api diff");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json output");
    assert_eq!(payload["kind"], serde_json::json!("api_diff"));
    assert!(payload["report"].is_object());
}

#[test]
fn api_verify_and_contract_generate_evidence() {
    let root = repo_root();
    let verify = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["api", "verify", "--format", "json"])
        .output()
        .expect("api verify");
    assert!(verify.status.success());

    let validate = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["api", "validate", "--format", "json"])
        .output()
        .expect("api validate");
    assert!(validate.status.success());

    let contract = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["api", "contract", "--format", "json"])
        .output()
        .expect("api contract");
    assert!(contract.status.success());

    for rel in [
        "artifacts/api/api-coverage-report.json",
        "artifacts/api/api-compatibility-report.json",
        "artifacts/api/api-contract-evidence-bundle.json",
        "artifacts/api/api-contract-registry-snapshot.json",
        "artifacts/api/api-example-requests.json",
        "artifacts/api/api-example-responses.json",
        "artifacts/api/api-example-dataset-queries.json",
    ] {
        assert!(root.join(rel).exists(), "missing artifact: {rel}");
    }
}

#[test]
fn openapi_fixture_is_parseable_for_schema_regression_tests() {
    let fixture = repo_root().join("ops/api/fixtures/openapi-minimal.json");
    let payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(fixture).expect("read fixture"))
            .expect("parse fixture");
    assert_eq!(payload["openapi"], serde_json::json!("3.0.3"));
    assert!(payload["paths"].is_object());
}

#[test]
fn api_compatibility_harness_contract_is_valid() {
    let harness = repo_root().join("ops/api/contracts/api-compatibility-harness.json");
    let payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(harness).expect("read harness"))
            .expect("parse harness");
    assert_eq!(payload["schema_version"], serde_json::json!(1));
    assert!(payload["version_matrix"].is_array());
}
