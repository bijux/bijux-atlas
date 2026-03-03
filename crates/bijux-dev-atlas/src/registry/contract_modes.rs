// SPDX-License-Identifier: Apache-2.0
//! Contract mode classification registry and validators.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

pub const CONTRACT_MODES_PATH: &str = "configs/governance/contract-modes.json";
pub const CONTRACT_MODES_SCHEMA_PATH: &str =
    "configs/contracts/governance/contract-modes.schema.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractMode {
    Static,
    Effect,
    All,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractModesFile {
    pub schema_version: u64,
    pub contract_set_id: String,
    #[serde(default)]
    pub static_ids: Vec<String>,
    #[serde(default)]
    pub effect_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractModesValidation {
    pub total_contracts: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ContractsRegistry {
    contracts: Vec<ContractRegistryEntry>,
}

#[derive(Debug, Deserialize)]
struct ContractRegistryEntry {
    contract_id: String,
    mode: String,
}

impl ContractModesFile {
    pub fn load(repo_root: &Path) -> Result<Self, String> {
        let path = repo_root.join(CONTRACT_MODES_PATH);
        let text = fs::read_to_string(&path)
            .map_err(|err| format!("read {} failed: {err}", path.display()))?;
        let file: Self = serde_json::from_str(&text)
            .map_err(|err| format!("parse {} failed: {err}", path.display()))?;
        if file.schema_version != 1 || file.contract_set_id != "contract-modes" {
            return Err(format!(
                "{} must declare schema_version=1 and contract_set_id=contract-modes",
                path.display()
            ));
        }
        Ok(file)
    }

    pub fn resolved_for_registry(repo_root: &Path) -> Result<Self, String> {
        let file = Self::load(repo_root)?;
        if !file.static_ids.is_empty() || !file.effect_ids.is_empty() {
            return Ok(file);
        }
        let registry = load_contracts_registry(repo_root)?;
        let mut static_ids = Vec::new();
        let mut effect_ids = Vec::new();
        for contract in registry.contracts {
            match contract.mode.as_str() {
                "pure" => static_ids.push(contract.contract_id),
                "effect" => effect_ids.push(contract.contract_id),
                other => {
                    return Err(format!(
                        "contracts registry uses unsupported mode `{other}` for `{}`",
                        contract.contract_id
                    ))
                }
            }
        }
        Ok(Self {
            schema_version: file.schema_version,
            contract_set_id: file.contract_set_id,
            static_ids,
            effect_ids,
        })
    }

    pub fn validate(repo_root: &Path) -> Result<ContractModesValidation, String> {
        let file = Self::load(repo_root)?;
        let registry = load_contracts_registry(repo_root)?;
        let known_ids = registry
            .contracts
            .iter()
            .map(|entry| entry.contract_id.as_str())
            .collect::<BTreeSet<_>>();
        let mut errors = Vec::new();

        let raw_declared = file
            .static_ids
            .iter()
            .chain(file.effect_ids.iter())
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        for id in &raw_declared {
            if !known_ids.contains(id) {
                errors.push(format!(
                    "contract-modes references unknown contract id `{id}` not present in configs/governance/contracts.registry.json"
                ));
            }
        }

        let resolved = Self::resolved_for_registry(repo_root)?;
        let classified = resolved
            .static_ids
            .iter()
            .chain(resolved.effect_ids.iter())
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        for id in &known_ids {
            if !classified.contains(id) {
                errors.push(format!("contract `{id}` is missing from contract-modes"));
            }
        }

        Ok(ContractModesValidation {
            total_contracts: known_ids.len(),
            errors,
        })
    }
}

fn load_contracts_registry(repo_root: &Path) -> Result<ContractsRegistry, String> {
    let path = repo_root.join("configs/governance/contracts.registry.json");
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("read {} failed: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("parse {} failed: {err}", path.display()))
}
