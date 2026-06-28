// SPDX-License-Identifier: Apache-2.0

use crate::inventory::surfaces_manifest::SurfacesInventory;
use crate::inventory::toolchain::ToolchainInventory;
use crate::stack::profile_catalog::StackProfile;
use serde_json::{json, Value};

pub fn ops_inventory_payload(
    mut summary: Value,
    inventory_errors: &[String],
    profiles: &[StackProfile],
    surfaces: &SurfacesInventory,
    toolchain: &ToolchainInventory,
) -> Value {
    let toolchain_images = summary
        .get("toolchain_images")
        .cloned()
        .unwrap_or_else(|| json!(0));
    if let Some(map) = summary.as_object_mut() {
        map.insert("inventory_errors".to_string(), json!(inventory_errors));
        map.insert("profiles".to_string(), json!(profiles));
        map.insert("components".to_string(), toolchain_images);
        map.insert(
            "charts".to_string(),
            json!(surfaces
                .actions
                .iter()
                .filter(|action| action.id.contains("render"))
                .count()),
        );
        map.insert(
            "tools".to_string(),
            json!(toolchain.tools.keys().cloned().collect::<Vec<_>>()),
        );
        map.insert("suites".to_string(), json!(["load", "e2e", "k8s", "obs"]));
        map.insert(
            "scenarios".to_string(),
            json!(["load.run", "e2e.run", "obs.drill.run", "obs.verify"]),
        );
        map.insert(
            "schemas".to_string(),
            json!([
                "ops/stack/stack.toml",
                "ops/stack/profiles.json",
                "ops/stack/generated/version-manifest.json",
                "ops/inventory/toolchain.json",
                "ops/inventory/surfaces.json",
                "ops/inventory/contracts.json"
            ]),
        );
    }
    let status = if inventory_errors.is_empty() {
        "ok"
    } else {
        "failed"
    };

    json!({
        "schema_version": 1,
        "status": status,
        "text": format!("ops inventory: status={status}"),
        "rows": [summary],
        "summary": {"total": 1, "errors": inventory_errors.len(), "warnings": 0}
    })
}

#[cfg(test)]
mod tests {
    use super::ops_inventory_payload;
    use crate::inventory::surfaces_manifest::{SurfaceAction, SurfacesInventory};
    use crate::inventory::toolchain::{ToolDefinition, ToolchainInventory};
    use crate::stack::profile_catalog::StackProfile;
    use std::collections::BTreeMap;

    #[test]
    fn ops_inventory_payload_augments_summary_with_owned_contracts() {
        let payload = ops_inventory_payload(
            serde_json::json!({"toolchain_images": 4}),
            &["missing contract".to_string()],
            &[StackProfile {
                name: "developer".to_string(),
                kind_profile: "kind".to_string(),
                cluster_config: "ops/stack/kind/cluster.yaml".to_string(),
            }],
            &SurfacesInventory {
                actions: vec![SurfaceAction {
                    id: "k8s-render".to_string(),
                    domain: "k8s".to_string(),
                    command: vec!["ops".to_string(), "render".to_string()],
                    argv: vec![],
                }],
            },
            &ToolchainInventory {
                tools: BTreeMap::from([(
                    "helm".to_string(),
                    ToolDefinition {
                        required: true,
                        version_regex: "(\\d+\\.\\d+\\.\\d+)".to_string(),
                        probe_argv: vec!["version".to_string(), "--short".to_string()],
                    },
                )]),
            },
        );

        assert_eq!(payload["status"], "failed");
        assert_eq!(payload["rows"][0]["components"], 4);
        assert_eq!(payload["rows"][0]["charts"], 1);
        assert_eq!(payload["rows"][0]["tools"], serde_json::json!(["helm"]));
    }
}
