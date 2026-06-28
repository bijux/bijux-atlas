// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct SurfacesInventory {
    pub actions: Vec<SurfaceAction>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct SurfaceAction {
    pub id: String,
    pub domain: String,
    pub command: Vec<String>,
    pub argv: Vec<String>,
}

pub fn load_surfaces_inventory(ops_root: &Path) -> Result<SurfacesInventory, String> {
    let path = ops_root.join("inventory/surfaces.json");
    let text = std::fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

pub fn surfaces_listing_payload(mut inventory: SurfacesInventory) -> serde_json::Value {
    inventory
        .actions
        .sort_by(|left, right| left.id.cmp(&right.id));
    let rows = inventory
        .actions
        .iter()
        .map(|action| {
            serde_json::json!({
                "id": action.id,
                "domain": action.domain,
                "command": action.command,
                "argv": action.argv
            })
        })
        .collect::<Vec<_>>();
    let text = inventory
        .actions
        .iter()
        .map(|action| action.id.clone())
        .collect::<Vec<_>>()
        .join("\n");

    serde_json::json!({
        "schema_version": 1,
        "text": text,
        "rows": rows,
        "summary": {"total": inventory.actions.len(), "errors": 0, "warnings": 0}
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_surfaces_inventory_reads_governed_manifest() {
        let root = tempfile::tempdir().expect("tempdir");
        let inventory_dir = root.path().join("inventory");
        std::fs::create_dir_all(&inventory_dir).expect("create inventory dir");
        std::fs::write(
            inventory_dir.join("surfaces.json"),
            r#"{
  "actions": [
    {
      "id": "render",
      "domain": "k8s",
      "command": ["ops", "render"],
      "argv": ["--profile", "kind"]
    }
  ]
}"#,
        )
        .expect("write surfaces manifest");

        let manifest = load_surfaces_inventory(root.path()).expect("load surfaces manifest");

        assert_eq!(manifest.actions.len(), 1);
        assert_eq!(manifest.actions[0].id, "render");
        assert_eq!(manifest.actions[0].domain, "k8s");
    }

    #[test]
    fn surfaces_listing_payload_sorts_actions_by_identifier() {
        let payload = surfaces_listing_payload(SurfacesInventory {
            actions: vec![
                SurfaceAction {
                    id: "stack-down".to_string(),
                    domain: "stack".to_string(),
                    command: vec!["ops".to_string(), "down".to_string()],
                    argv: vec![],
                },
                SurfaceAction {
                    id: "inventory".to_string(),
                    domain: "inventory".to_string(),
                    command: vec!["ops".to_string(), "inventory".to_string()],
                    argv: vec!["--format".to_string(), "json".to_string()],
                },
            ],
        });

        assert_eq!(payload["summary"]["total"], 2);
        assert_eq!(payload["rows"][0]["id"], "inventory");
        assert_eq!(payload["rows"][1]["id"], "stack-down");
    }
}
