// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use bijux_dev_atlas::registry::{
    ContractMode, ContractModesFile, CONTRACT_MODES_PATH, CONTRACT_MODES_SCHEMA_PATH,
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn contract_modes_registry_validates_against_contracts_registry() {
    let validation = ContractModesFile::validate(&repo_root()).expect("contract mode validation");
    assert!(
        validation.total_contracts > 0,
        "expected contracts in configs/governance/contracts.registry.json"
    );
    assert!(
        validation.errors.is_empty(),
        "unexpected contract mode errors: {:?}",
        validation.errors
    );
}

#[test]
fn contract_modes_registry_resolves_modes_from_contract_metadata() {
    let resolved = ContractModesFile::resolved_for_registry(&repo_root()).expect("resolved modes");
    assert!(!resolved.static_ids.is_empty());
    assert!(!resolved.effect_ids.is_empty());
    assert!(repo_root().join(CONTRACT_MODES_PATH).exists());
    assert!(repo_root().join(CONTRACT_MODES_SCHEMA_PATH).exists());
    let all_mode = ContractMode::All;
    assert_eq!(all_mode, ContractMode::All);
}
