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

fn load_json(path: &str) -> serde_json::Value {
    let root = repo_root();
    serde_json::from_str(&fs::read_to_string(root.join(path)).expect("read json")).expect("parse json")
}

#[test]
fn sustainability_artifacts_exist_with_schema() {
    for path in [
        "ops/governance/sustainability/governance-evidence-artifact.json",
        "ops/governance/sustainability/governance-compliance-report.json",
        "ops/governance/sustainability/governance-health-dashboard-artifact.json",
        "ops/governance/sustainability/sustainability-metrics.json",
        "ops/governance/sustainability/project-health-indicators-report.json",
        "ops/governance/sustainability/contributor-growth-tracking-artifact.json",
        "ops/governance/sustainability/governance-maturity-index-report.json",
    ] {
        let value = load_json(path);
        assert_eq!(value.get("schema_version").and_then(|v| v.as_u64()), Some(1));
    }
}

#[test]
fn sustainability_scenarios_and_workflow_exist() {
    let root = repo_root();
    for path in [
        "ops/governance/sustainability/governance-ci-validation-scenario.json",
        "ops/governance/sustainability/governance-audit-scenario.json",
        ".github/workflows/governance-sustainability-validation.yml",
    ] {
        assert!(root.join(path).exists(), "missing required governance file: {path}");
    }
}
