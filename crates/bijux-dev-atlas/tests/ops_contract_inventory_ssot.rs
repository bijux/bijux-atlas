// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
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

#[test]
fn ops_inventory_contract_ids_match_code_registry() {
    let root = workspace_root();
    let inventory_path = root.join("ops/inventory/contracts.json");
    let inventory_text = fs::read_to_string(&inventory_path).expect("read ops inventory contracts");
    let inventory: serde_json::Value =
        serde_json::from_str(&inventory_text).expect("parse ops inventory contracts");
    let listed_ids = inventory
        .get("contract_ids")
        .and_then(|value| value.as_array())
        .expect("contract_ids array")
        .iter()
        .map(|value| value.as_str().expect("contract id").to_string())
        .collect::<BTreeSet<_>>();

    let registry_ids = bijux_dev_atlas::contracts::ops::render_registry_snapshot_json(&root)
        .expect("render ops contract registry snapshot")
        .get("contracts")
        .and_then(|value| value.as_array())
        .expect("contracts array")
        .iter()
        .map(|value| {
            value
                .get("id")
                .and_then(|id| id.as_str())
                .expect("registry id")
                .to_string()
        })
        .collect::<BTreeSet<_>>();

    assert_eq!(
        listed_ids, registry_ids,
        "ops/inventory/contracts.json contract_ids must match ops code registry"
    );
}
