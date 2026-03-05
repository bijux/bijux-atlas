// SPDX-License-Identifier: Apache-2.0

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
fn audit_schema_files_exist_and_parse() {
    let root = repo_root();
    for rel in [
        "ops/audit/event.schema.json",
        "ops/audit/report.schema.json",
    ] {
        let text = fs::read_to_string(root.join(rel)).expect("read schema");
        let value: serde_json::Value = serde_json::from_str(&text).expect("parse schema json");
        assert!(value.get("$id").is_some(), "schema $id missing for {rel}");
    }
}

#[test]
fn audit_fixture_and_ci_scenario_are_present() {
    let root = repo_root();
    let fixture: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("ops/audit/fixtures/inconsistent-state.json"))
            .expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(
        fixture.get("expected_status").and_then(|v| v.as_str()),
        Some("failed")
    );

    let scenario: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("ops/audit/ci-scenario.json")).expect("read scenario"),
    )
    .expect("parse scenario");
    assert!(scenario
        .get("commands")
        .and_then(|v| v.as_array())
        .is_some());
}

#[test]
fn audit_evidence_and_metrics_assets_are_present() {
    let root = repo_root();
    for rel in [
        "ops/audit/evidence-integration.json",
        "ops/audit/metrics.json",
        "ops/audit/benchmark.json",
        "ops/audit/summary-table.md",
        "ops/audit/health-dashboard.md",
    ] {
        assert!(root.join(rel).exists(), "missing asset {rel}");
    }
}
