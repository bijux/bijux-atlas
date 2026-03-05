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
fn governance_rules_snapshot_matches_registry() {
    let root = repo_root();
    let live = fs::read_to_string(root.join("configs/governance/enforcement/rules.json"))
        .expect("read live rules");
    let snapshot = fs::read_to_string(root.join("ops/governance/enforcement/rules.snapshot.json"))
        .expect("read snapshot rules");
    assert_eq!(
        live, snapshot,
        "governance enforcement snapshot drift detected"
    );
}

#[test]
fn governance_violation_examples_are_schema_shaped() {
    let root = repo_root();
    let value: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("ops/governance/enforcement/violation-examples.json"))
            .expect("read violation examples"),
    )
    .expect("parse violation examples");
    assert_eq!(
        value.get("schema_version").and_then(|v| v.as_u64()),
        Some(1)
    );
    let rows = value
        .get("examples")
        .and_then(|v| v.as_array())
        .expect("examples array");
    assert!(!rows.is_empty());
    assert!(rows
        .iter()
        .all(|row| row.get("rule_id").is_some() && row.get("message").is_some()));
}

#[test]
fn governance_enforcement_fixture_catalog_is_complete() {
    let root = repo_root();
    for rel in [
        "ops/governance/enforcement/fixtures/missing-required-file.json",
        "ops/governance/enforcement/fixtures/empty-checks-registry.json",
        "ops/governance/enforcement/fixtures/invalid-docs-navigation.json",
    ] {
        let payload: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(root.join(rel)).expect("read fixture"))
                .expect("parse fixture");
        assert!(
            payload.get("fixture_id").is_some(),
            "fixture_id missing for {rel}"
        );
        assert!(
            payload.get("expected_violation").is_some(),
            "expected_violation missing for {rel}"
        );
    }
}

#[test]
fn governance_enforcement_coverage_report_is_current() {
    let root = repo_root();
    let report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("ops/governance/enforcement/coverage-report.json"))
            .expect("read coverage report"),
    )
    .expect("parse coverage report");
    assert_eq!(
        report.get("schema_version").and_then(|v| v.as_u64()),
        Some(1)
    );
    assert_eq!(report.get("rule_count").and_then(|v| v.as_u64()), Some(10));
}
