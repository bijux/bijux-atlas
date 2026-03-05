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
fn load_capacity_docs_and_dataset_assets_exist() {
    let root = repo_root();
    for rel in [
        "docs/operations/load/capacity-planning-guide.md",
        "docs/operations/load/interpretation-guide.md",
        "docs/operations/load/performance-troubleshooting-guide.md",
        "docs/operations/load/example-datasets.md",
        "docs/operations/load/capacity-engineering-summary.md",
        "ops/load/data/query-sample.jsonl",
        "ops/load/data/ingest-sample.jsonl",
    ] {
        assert!(root.join(rel).exists(), "missing required file: {rel}");
    }
}

#[test]
fn load_capacity_contract_and_dashboard_assets_are_valid_json() {
    let root = repo_root();
    for rel in [
        "ops/load/ci/load-harness-ci-scenario.json",
        "ops/load/contracts/performance-regression-ci-contract.json",
        "ops/load/generated/load-testing-dashboards.json",
    ] {
        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(root.join(rel)).expect("read json"))
                .expect("parse json");
        assert_eq!(value["schema_version"], serde_json::json!(1));
    }
}
