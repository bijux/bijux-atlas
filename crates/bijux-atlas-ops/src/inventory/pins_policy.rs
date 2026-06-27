// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::kubernetes::path_contracts as kubernetes_paths;
use crate::stack::path_contracts as stack_paths;

use super::path_contracts as inventory_paths;
use super::pins_manifest::StackPinsToml;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinsPolicyError {
    Read { path: PathBuf, message: String },
    Parse { path: PathBuf, message: String },
}

impl PinsPolicyError {
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

pub fn validate_pins_completeness(
    repo_root: &Path,
    pins: &StackPinsToml,
) -> Result<Vec<String>, PinsPolicyError> {
    let mut errors = Vec::new();

    let stack_manifest =
        load_json_value(&stack_paths::atlas_generated_version_manifest_from_repo_root(repo_root))?;
    if let Some(entries) = stack_manifest.as_object() {
        for (key, value) in entries {
            if key == "schema_version" {
                continue;
            }
            if !pins.images.contains_key(key) {
                errors.push(format!("pins missing image key `{key}`"));
            }
            if let Some(image) = value.as_str() {
                if image.contains(":latest") {
                    errors.push(format!("floating tag forbidden in stack manifest `{key}`"));
                }
            }
        }
    }

    for (key, value) in &pins.images {
        if value.contains(":latest") {
            errors.push(format!("floating tag forbidden in pins image `{key}`"));
        }
    }
    for (key, value) in &pins.charts {
        if value.contains(":latest") {
            errors.push(format!("floating tag forbidden in pins chart `{key}`"));
        }
    }

    let contracts_json = load_json_value(
        &inventory_paths::atlas_contracts_registry_from_repo_root(repo_root),
    )?;
    let contract_paths = contracts_json
        .get("contracts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|entry| {
            entry
                .get("path")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .collect::<BTreeSet<_>>();
    for required in ["ops/inventory/tools.toml", "ops/inventory/pins.yaml"] {
        if !contract_paths.contains(required) {
            errors.push(format!(
                "contracts inventory missing required entry `{required}`"
            ));
        }
    }

    for path in [
        kubernetes_paths::atlas_values_file_from_ops_root(&kubernetes_paths::atlas_ops_root(
            repo_root,
        )),
        kubernetes_paths::atlas_offline_values_file_from_ops_root(
            &kubernetes_paths::atlas_ops_root(repo_root),
        ),
    ] {
        let file_label = path
            .strip_prefix(repo_root)
            .unwrap_or(path.as_path())
            .display()
            .to_string();
        let text = std::fs::read_to_string(&path).map_err(|err| PinsPolicyError::Read {
            path: path.clone(),
            message: err.to_string(),
        })?;
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.contains(":latest") {
                errors.push(format!(
                    "floating latest forbidden in {file_label}: `{trimmed}`"
                ));
            }
            if trimmed.contains("image:")
                && trimmed.contains(':')
                && !trimmed.contains("@sha256:")
                && !trimmed.ends_with(':')
            {
                errors.push(format!(
                    "base image pin must include digest in {file_label}: `{trimmed}`"
                ));
            }
        }
    }

    for root_name in ["makefiles", ".github/workflows"] {
        let walk_root = repo_root.join(root_name);
        if !walk_root.exists() {
            continue;
        }
        for path in walk_files(&walk_root) {
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            for pattern in ["helm ", "kubectl ", "kind ", "k6 "] {
                if text.contains(pattern)
                    && !text.contains("bijux dev atlas")
                    && !text.contains("bijux-atlas-dev")
                {
                    let relative_path = path
                        .strip_prefix(repo_root)
                        .unwrap_or(path.as_path())
                        .display()
                        .to_string();
                    errors.push(format!(
                        "hardcoded tool invocation forbidden (`{pattern}`) in {relative_path}"
                    ));
                }
            }
            if text.contains("kubectl apply") && !text.contains("bijux dev atlas ops k8s apply") {
                let relative_path = path
                    .strip_prefix(repo_root)
                    .unwrap_or(path.as_path())
                    .display()
                    .to_string();
                errors.push(format!(
                    "direct kubectl apply forbidden in {relative_path}; use `bijux dev atlas ops k8s apply`"
                ));
            }
        }
    }

    errors.sort();
    errors.dedup();
    Ok(errors)
}

fn load_json_value(path: &Path) -> Result<Value, PinsPolicyError> {
    let text = std::fs::read_to_string(path).map_err(|err| PinsPolicyError::Read {
        path: path.to_path_buf(),
        message: err.to_string(),
    })?;
    serde_json::from_str(&text).map_err(|err| PinsPolicyError::Parse {
        path: path.to_path_buf(),
        message: err.to_string(),
    })
}

fn walk_files(root: &Path) -> Vec<PathBuf> {
    let mut entries = Vec::new();
    if root.is_file() {
        entries.push(root.to_path_buf());
        return entries;
    }
    if let Ok(children) = std::fs::read_dir(root) {
        for child in children.flatten() {
            entries.extend(walk_files(&child.path()));
        }
    }
    entries
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

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
        assert!(errors
            .iter()
            .any(|entry| entry.contains("floating tag forbidden")));
    }
}
