// SPDX-License-Identifier: Apache-2.0

use crate::inventory::path_contracts::atlas_pins_manifest_from_repo_root;
use crate::stack::path_contracts::atlas_generated_version_manifest_from_repo_root;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinsSyncChange {
    pub key: String,
    pub old: String,
    pub new: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinsSyncResult {
    pub target_path: PathBuf,
    pub changes: Vec<PinsSyncChange>,
}

pub fn sync_pins_from_generated_stack_manifest(repo_root: &Path) -> Result<PinsSyncResult, String> {
    let target_path = atlas_pins_manifest_from_repo_root(repo_root);
    let old =
        crate::workspace::inventory::load_stack_pins(repo_root).map_err(|err| err.detail())?;
    let mut updated = old.clone();
    let stack_manifest_path = atlas_generated_version_manifest_from_repo_root(repo_root);
    let stack_manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&stack_manifest_path)
            .map_err(|err| format!("failed to read {}: {err}", stack_manifest_path.display()))?,
    )
    .map_err(|err| format!("invalid version manifest json: {err}"))?;
    if let Some(entries) = stack_manifest.as_object() {
        for (key, value) in entries {
            if key == "schema_version" {
                continue;
            }
            if let Some(value) = value.as_str() {
                updated.images.insert(key.clone(), value.to_string());
            }
        }
    }
    let mut changes = Vec::new();
    for (key, value) in &updated.images {
        let old_value = old.images.get(key).cloned().unwrap_or_default();
        if &old_value != value {
            changes.push(PinsSyncChange {
                key: format!("images.{key}"),
                old: old_value,
                new: value.clone(),
                reason: "generated_stack_version_manifest".to_string(),
            });
        }
    }
    let mut pins_yaml = std::fs::read_to_string(&target_path)
        .map_err(|err| format!("failed to read {}: {err}", target_path.display()))?;
    for (key, value) in &updated.images {
        let needle = format!("{key}: ");
        let mut replaced = false;
        let mut lines = Vec::new();
        for line in pins_yaml.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with(&needle) {
                lines.push(format!("  {key}: \"{value}\""));
                replaced = true;
            } else {
                lines.push(line.to_string());
            }
        }
        if !replaced {
            return Err(format!(
                "failed to sync image `{key}` into {}; missing key in pins.yaml",
                target_path.display()
            ));
        }
        pins_yaml = lines.join("\n");
        pins_yaml.push('\n');
    }
    std::fs::write(&target_path, pins_yaml)
        .map_err(|err| format!("failed to write {}: {err}", target_path.display()))?;
    Ok(PinsSyncResult {
        target_path,
        changes,
    })
}

pub fn build_pins_update_payload(result: &PinsSyncResult) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "text": "ops pins updated from generated stack version manifest",
        "rows": [{
            "target_path": result.target_path.display().to_string(),
            "changes": result.changes.iter().map(|change| {
                serde_json::json!({
                    "key": change.key,
                    "old": change.old,
                    "new": change.new,
                    "reason": change.reason,
                })
            }).collect::<Vec<_>>()
        }],
        "summary": {"total": 1, "errors": 0, "warnings": 0}
    })
}

#[cfg(test)]
mod tests {
    use super::{build_pins_update_payload, sync_pins_from_generated_stack_manifest};

    fn write_inventory(root: &std::path::Path, pins_yaml: &str) {
        std::fs::create_dir_all(root.join("ops/inventory")).expect("mkdir inventory");
        std::fs::create_dir_all(root.join("ops/stack/generated")).expect("mkdir generated");
        std::fs::write(
            root.join("ops/stack/generated/version-manifest.json"),
            "{\"schema_version\":1,\"redis\":\"redis@sha256:999\"}",
        )
        .expect("write version manifest");
        std::fs::write(root.join("ops/inventory/pins.yaml"), pins_yaml).expect("write pins");
    }

    #[test]
    fn pins_sync_updates_owned_manifest_entries() {
        let root = tempfile::tempdir().expect("tempdir");
        write_inventory(
            root.path(),
            "images:\n  redis: \"redis@sha256:123\"\nversions:\n  chart: \"1.2.3\"\n  prometheus_operator_crd: \"0.78.2\"\n",
        );

        let result = sync_pins_from_generated_stack_manifest(root.path()).expect("sync");
        let rewritten = std::fs::read_to_string(root.path().join("ops/inventory/pins.yaml"))
            .expect("read rewritten pins");

        assert_eq!(result.changes.len(), 1);
        assert!(rewritten.contains("redis@sha256:999"));
        let payload = build_pins_update_payload(&result);
        assert_eq!(payload["summary"]["errors"], 0);
    }

    #[test]
    fn pins_sync_requires_owned_image_key() {
        let root = tempfile::tempdir().expect("tempdir");
        write_inventory(
            root.path(),
            "images:\n  postgres: \"postgres@sha256:123\"\nversions:\n  chart: \"1.2.3\"\n  prometheus_operator_crd: \"0.78.2\"\n",
        );

        let error = sync_pins_from_generated_stack_manifest(root.path()).expect_err("sync error");

        assert!(error.contains("missing key in pins.yaml"));
        assert!(error.contains("redis"));
    }
}
