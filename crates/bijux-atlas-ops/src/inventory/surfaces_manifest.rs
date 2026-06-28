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
}
