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

pub fn validate_pins_completeness(
    repo_root: &Path,
    pins: &StackPinsToml,
) -> Result<Vec<String>, WorkspaceInventoryError> {
    crate::inventory::pins_policy::validate_pins_completeness(repo_root, pins)
        .map_err(map_pins_policy_error)
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

fn map_pins_policy_error(
    error: crate::inventory::pins_policy::PinsPolicyError,
) -> WorkspaceInventoryError {
    match error {
        crate::inventory::pins_policy::PinsPolicyError::Read { .. } => {
            WorkspaceInventoryError::Manifest(error.detail())
        }
        crate::inventory::pins_policy::PinsPolicyError::Parse { .. } => {
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

    #[test]
    fn workspace_inventory_validates_pins_completeness_through_owned_surface() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops/stack/generated")).expect("mkdir generated");
        std::fs::create_dir_all(root.path().join("ops/k8s/charts/bijux-atlas"))
            .expect("mkdir chart");
        std::fs::create_dir_all(root.path().join("ops/inventory")).expect("mkdir inventory");
        std::fs::write(
            root.path()
                .join("ops/stack/generated/version-manifest.json"),
            "{\"schema_version\":1,\"redis\":\"redis:latest\"}",
        )
        .expect("write manifest");
        std::fs::write(
            root.path().join("ops/k8s/charts/bijux-atlas/values.yaml"),
            "image: redis:latest\n",
        )
        .expect("write values");
        std::fs::write(
            root.path()
                .join("ops/k8s/charts/bijux-atlas/values-offline.yaml"),
            "image: redis:latest\n",
        )
        .expect("write values offline");
        std::fs::write(
            root.path().join("ops/inventory/contracts.json"),
            "{\"contracts\":[{\"path\":\"ops/inventory/tools.toml\"},{\"path\":\"ops/inventory/pins.yaml\"}]}",
        )
        .expect("write contracts");
        let pins = StackPinsToml {
            charts: std::collections::BTreeMap::new(),
            images: std::collections::BTreeMap::from([(
                "redis".to_string(),
                "redis:latest".to_string(),
            )]),
            crds: std::collections::BTreeMap::new(),
        };

        let errors = validate_pins_completeness(root.path(), &pins).expect("validate pins");

        assert!(errors
            .iter()
            .any(|entry| entry.contains("floating tag forbidden")));
    }
}
