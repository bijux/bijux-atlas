// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

fn update_goldens_enabled() -> bool {
    std::env::var("UPDATE_GOLDENS").ok().as_deref() == Some("1")
}

fn assert_json_matches_or_update(path: &PathBuf, expected: &serde_json::Value, message: &str) {
    if update_goldens_enabled() {
        let text = serde_json::to_string_pretty(expected).expect("serialize expected json");
        fs::write(path, format!("{text}\n")).expect("write updated golden");
    }
    let text = fs::read_to_string(path).expect("read committed golden");
    let actual: serde_json::Value = serde_json::from_str(&text).expect("parse committed golden");
    assert_eq!(actual, *expected, "{message}");
}

#[test]
fn ops_contract_index_matches_committed_example() {
    let root = workspace_root();
    let expected = bijux_dev_atlas::contracts::ops::render_contract_index_json(&root)
        .expect("render ops contract index");
    let path = root.join("ops/_generated.example/ops-contract-index.json");
    assert_json_matches_or_update(&path, &expected, "ops contract index drift detected");
}

#[test]
fn ops_contract_coverage_report_matches_committed_example() {
    let root = workspace_root();
    let expected = bijux_dev_atlas::contracts::ops::render_contract_coverage_report_json(&root)
        .expect("render ops contract coverage report");
    let path = root.join("ops/_generated.example/contract-coverage-report.json");
    assert_json_matches_or_update(&path, &expected, "ops contract coverage drift detected");
}

#[test]
fn ops_inventory_contract_ids_match_contract_index() {
    let root = workspace_root();
    let index = bijux_dev_atlas::contracts::ops::render_contract_index_json(&root)
        .expect("render ops contract index");
    let expected_ids = index
        .get("contracts")
        .and_then(|v| v.as_array())
        .expect("contracts array")
        .iter()
        .filter_map(|row| row.get("id").and_then(|v| v.as_str()))
        .collect::<Vec<_>>();

    let inventory_path = root.join("ops/inventory/contracts.json");
    let inventory_text = fs::read_to_string(&inventory_path).expect("read ops inventory contracts");
    let inventory: serde_json::Value =
        serde_json::from_str(&inventory_text).expect("parse ops inventory contracts");
    let actual_ids = inventory
        .get("contract_ids")
        .and_then(|v| v.as_array())
        .expect("contract_ids array")
        .iter()
        .filter_map(|row| row.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        actual_ids, expected_ids,
        "ops inventory contract_ids drift detected"
    );
}
