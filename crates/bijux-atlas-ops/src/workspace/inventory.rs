// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

pub use crate::inventory::pins_manifest::StackPinsToml;
pub use crate::inventory::toolchain::ToolchainInventory;
pub use crate::inventory::tools_manifest::ToolsToml;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceInventoryError {
    Manifest(String),
    Schema(String),
}

impl WorkspaceInventoryError {
    #[must_use]
    pub fn detail(&self) -> String {
        match self {
            Self::Manifest(detail) | Self::Schema(detail) => detail.clone(),
        }
    }
}

pub fn load_toolchain_inventory_from_ops_root(
    ops_root: &Path,
) -> Result<ToolchainInventory, WorkspaceInventoryError> {
    crate::inventory::toolchain::load_toolchain_inventory_from_ops_root(ops_root)
        .map_err(map_toolchain_error)
}

pub fn load_tools_manifest(repo_root: &Path) -> Result<ToolsToml, WorkspaceInventoryError> {
    crate::inventory::tools_manifest::load_tools_manifest(repo_root).map_err(map_tools_error)
}

pub fn load_stack_pins(repo_root: &Path) -> Result<StackPinsToml, WorkspaceInventoryError> {
    crate::inventory::pins_manifest::load_pins_manifest(repo_root).map_err(map_pins_error)
}

fn map_toolchain_error(
    error: crate::inventory::toolchain::ToolchainInventoryError,
) -> WorkspaceInventoryError {
    match error {
        crate::inventory::toolchain::ToolchainInventoryError::Read { .. } => {
            WorkspaceInventoryError::Manifest(error.detail())
        }
        crate::inventory::toolchain::ToolchainInventoryError::Parse { .. } => {
            WorkspaceInventoryError::Schema(error.detail())
        }
    }
}

fn map_tools_error(
    error: crate::inventory::tools_manifest::ToolsManifestError,
) -> WorkspaceInventoryError {
    match error {
        crate::inventory::tools_manifest::ToolsManifestError::Read { .. } => {
            WorkspaceInventoryError::Manifest(error.detail())
        }
        crate::inventory::tools_manifest::ToolsManifestError::Parse { .. } => {
            WorkspaceInventoryError::Schema(error.detail())
        }
    }
}

fn map_pins_error(
    error: crate::inventory::pins_manifest::PinsManifestError,
) -> WorkspaceInventoryError {
    match error {
        crate::inventory::pins_manifest::PinsManifestError::Read { .. } => {
            WorkspaceInventoryError::Manifest(error.detail())
        }
        crate::inventory::pins_manifest::PinsManifestError::Parse { .. } => {
            WorkspaceInventoryError::Schema(error.detail())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_inventory_loads_tools_manifest_through_owned_surface() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops/inventory")).expect("mkdir");
        std::fs::write(
            root.path().join("ops/inventory/tools.toml"),
            "[[tools]]\nname=\"helm\"\nrequired=true\nversion_regex=\"(\\\\d+\\\\.\\\\d+\\\\.\\\\d+)\"\nprobe_argv=[\"version\",\"--short\"]\n",
        )
        .expect("write manifest");

        let manifest = load_tools_manifest(root.path()).expect("load tools manifest");

        assert_eq!(manifest.tools.len(), 1);
        assert_eq!(manifest.tools[0].name, "helm");
    }

    #[test]
    fn workspace_inventory_loads_pins_manifest_through_owned_surface() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops/inventory")).expect("mkdir");
        std::fs::write(
            root.path().join("ops/inventory/pins.yaml"),
            "images:\n  redis: \"redis@sha256:123\"\nversions:\n  chart: \"1.2.3\"\n  prometheus_operator_crd: \"0.78.2\"\n",
        )
        .expect("write pins");

        let pins = load_stack_pins(root.path()).expect("load pins");

        assert_eq!(
            pins.images.get("redis"),
            Some(&"redis@sha256:123".to_string())
        );
        assert_eq!(pins.charts.get("bijux_atlas"), Some(&"1.2.3".to_string()));
    }

    #[test]
    fn workspace_inventory_loads_toolchain_inventory_from_ops_root() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops/inventory")).expect("mkdir");
        std::fs::write(
            root.path().join("ops/inventory/toolchain.json"),
            r#"{"tools":{"helm":{"required":true,"version_regex":"(\\d+\\.\\d+\\.\\d+)","probe_argv":["version","--short"]}}}"#,
        )
        .expect("write toolchain");

        let inventory = load_toolchain_inventory_from_ops_root(&root.path().join("ops"))
            .expect("load toolchain");

        assert!(inventory.tools.contains_key("helm"));
    }
}
