// SPDX-License-Identifier: Apache-2.0

use crate::*;
use bijux_atlas_ops::inventory::pins_manifest::StackPinsToml;
pub(crate) use bijux_atlas_ops::inventory::tooling_support::{
    build_tool_probe_snapshot, normalize_tool_version_with_regex, parse_tool_overrides,
    tool_probe_skipped_snapshot, ToolMismatchCode,
};

pub(crate) fn validate_pins_completeness(
    repo_root: &Path,
    pins: &StackPinsToml,
) -> Result<Vec<String>, OpsCommandError> {
    bijux_atlas_ops::inventory::pins_policy::validate_pins_completeness(repo_root, pins).map_err(
        |err| match err {
            bijux_atlas_ops::inventory::pins_policy::PinsPolicyError::Read { .. } => {
                OpsCommandError::Manifest(err.detail())
            }
            bijux_atlas_ops::inventory::pins_policy::PinsPolicyError::Parse { .. } => {
                OpsCommandError::Schema(err.detail())
            }
        },
    )
}

pub(crate) fn verify_tools_snapshot(
    allow_subprocess: bool,
    inventory: &ToolchainInventory,
) -> Result<serde_json::Value, String> {
    if !allow_subprocess {
        return Ok(tool_probe_skipped_snapshot());
    }
    build_tool_probe_snapshot(&OpsProcess::new(true), inventory)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn pins_validation_rejects_latest_tag() {
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
            charts: BTreeMap::new(),
            images: BTreeMap::from([("redis".to_string(), "redis:latest".to_string())]),
            crds: BTreeMap::new(),
        };
        let errors = validate_pins_completeness(root.path(), &pins).expect("validate");
        assert!(errors.iter().any(|e| e.contains("floating tag forbidden")));
    }
}
