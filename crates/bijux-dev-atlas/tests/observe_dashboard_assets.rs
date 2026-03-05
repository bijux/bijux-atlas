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
fn dashboard_validation_contract_references_existing_json_dashboards() {
    let root = repo_root();
    let contract_path = root.join("ops/observe/contracts/dashboard-json-validation-contract.json");
    let contract: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(contract_path).expect("read contract"))
            .expect("parse contract");
    assert_eq!(contract["schema_version"], serde_json::json!(1));
    for rel in contract["dashboards"].as_array().expect("dashboards array") {
        let rel = rel.as_str().expect("dashboard path string");
        let payload: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(root.join(rel)).expect("read dashboard"))
                .expect("parse dashboard");
        assert!(
            payload.get("title").is_some(),
            "dashboard missing title: {rel}"
        );
        assert!(payload.get("uid").is_some(), "dashboard missing uid: {rel}");
        assert!(
            payload.get("panels").is_some(),
            "dashboard missing panels: {rel}"
        );
    }
}
