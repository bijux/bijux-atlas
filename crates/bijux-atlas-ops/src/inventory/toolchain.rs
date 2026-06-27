// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::path_contracts;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct ToolchainInventory {
    pub tools: BTreeMap<String, ToolDefinition>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct ToolDefinition {
    pub required: bool,
    pub version_regex: String,
    pub probe_argv: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolchainInventoryError {
    Read { path: PathBuf, message: String },
    Parse { path: PathBuf, message: String },
}

impl ToolchainInventoryError {
    #[must_use]
    pub fn detail(&self) -> String {
        match self {
            Self::Read { path, message } => {
                format!("failed to read {}: {message}", path.display())
            }
            Self::Parse { path, message } => {
                format!("failed to parse {}: {message}", path.display())
            }
        }
    }
}

pub fn load_toolchain_inventory(
    repo_root: &Path,
) -> Result<ToolchainInventory, ToolchainInventoryError> {
    let path = path_contracts::atlas_toolchain_inventory(repo_root);
    load_toolchain_inventory_from_path(&path)
}

pub fn load_toolchain_inventory_from_ops_root(
    ops_root: &Path,
) -> Result<ToolchainInventory, ToolchainInventoryError> {
    let path = path_contracts::atlas_toolchain_inventory_from_ops_root(ops_root);
    load_toolchain_inventory_from_path(&path)
}

fn load_toolchain_inventory_from_path(
    path: &Path,
) -> Result<ToolchainInventory, ToolchainInventoryError> {
    let text = std::fs::read_to_string(path).map_err(|err| ToolchainInventoryError::Read {
        path: path.to_path_buf(),
        message: err.to_string(),
    })?;
    serde_json::from_str(&text).map_err(|err| ToolchainInventoryError::Parse {
        path: path.to_path_buf(),
        message: err.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toolchain_inventory_loader_reads_canonical_contract() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops/inventory")).expect("mkdir");
        std::fs::write(
            root.path().join("ops/inventory/toolchain.json"),
            r#"{"tools":{"helm":{"required":true,"version_regex":"(\\d+\\.\\d+\\.\\d+)","probe_argv":["version","--short"]}}}"#,
        )
        .expect("write inventory");

        let inventory = load_toolchain_inventory(root.path()).expect("load inventory");
        assert!(inventory.tools.contains_key("helm"));
    }
}
