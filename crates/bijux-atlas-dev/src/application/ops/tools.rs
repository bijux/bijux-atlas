// SPDX-License-Identifier: Apache-2.0

use crate::*;
use bijux_atlas_ops::inventory::pins_manifest::StackPinsToml;
pub(crate) use bijux_atlas_ops::inventory::tooling_support::{
    normalize_tool_version_with_regex, ToolMismatchCode,
};

pub(crate) fn parse_tool_overrides(
    values: &[String],
) -> Result<std::collections::BTreeMap<String, String>, String> {
    let mut out = std::collections::BTreeMap::new();
    for raw in values {
        let Some((name, path)) = raw.split_once('=') else {
            return Err(format!(
                "invalid --tool override `{raw}`; expected name=path"
            ));
        };
        let name = name.trim();
        let path = path.trim();
        if name.is_empty() || path.is_empty() {
            return Err(format!(
                "invalid --tool override `{raw}`; expected name=path"
            ));
        }
        out.insert(name.to_string(), path.to_string());
    }
    Ok(out)
}

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

pub(crate) fn tool_definitions_sorted(
    inventory: &ToolchainInventory,
) -> Vec<(String, ToolDefinition)> {
    inventory
        .tools
        .iter()
        .map(|(name, definition)| (name.clone(), definition.clone()))
        .collect()
}

pub(crate) fn verify_tools_snapshot(
    allow_subprocess: bool,
    inventory: &ToolchainInventory,
) -> Result<serde_json::Value, String> {
    if !allow_subprocess {
        return Ok(serde_json::json!({
            "enabled": false,
            "text": "tool verification skipped (pass --allow-subprocess)",
            "missing_required": [],
            "rows": []
        }));
    }
    let process = OpsProcess::new(true);
    let mut rows = Vec::new();
    let mut missing_required = Vec::new();
    for (name, definition) in tool_definitions_sorted(inventory) {
        let mut row = process
            .probe_tool(&name, &definition.probe_argv, &definition.version_regex)
            .map_err(|e| e.to_stable_message())?;
        row["required"] = serde_json::Value::Bool(definition.required);
        if definition.required && row["installed"] != serde_json::Value::Bool(true) {
            missing_required.push(name.clone());
        }
        rows.push(row);
    }
    rows.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    Ok(serde_json::json!({
        "enabled": true,
        "text": if missing_required.is_empty() { "all required tools available" } else { "missing required tools" },
        "missing_required": missing_required,
        "rows": rows
    }))
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
