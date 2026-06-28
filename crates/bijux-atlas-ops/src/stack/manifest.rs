// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::path_contracts;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct StackManifestToml {
    pub profiles: BTreeMap<String, StackManifestProfile>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct StackManifestProfile {
    pub kind_profile: String,
    pub cluster_config: String,
    pub namespace: String,
    pub components: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StackManifestLoadError {
    Read { path: PathBuf, message: String },
    Parse { path: PathBuf, message: String },
}

impl StackManifestLoadError {
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

pub fn load_stack_manifest(repo_root: &Path) -> Result<StackManifestToml, StackManifestLoadError> {
    let path = path_contracts::atlas_stack_manifest_from_repo_root(repo_root);
    let text = std::fs::read_to_string(&path).map_err(|err| StackManifestLoadError::Read {
        path: path.clone(),
        message: err.to_string(),
    })?;
    toml::from_str(&text).map_err(|err| StackManifestLoadError::Parse {
        path,
        message: err.to_string(),
    })
}

#[must_use]
pub fn validate_stack_manifest(repo_root: &Path, manifest: &StackManifestToml) -> Vec<String> {
    let mut errors = Vec::new();
    if manifest.profiles.is_empty() {
        errors.push("stack manifest must declare at least one profile".to_string());
    }
    for (name, profile) in &manifest.profiles {
        let cluster_path = repo_root.join(&profile.cluster_config);
        if !cluster_path.exists() {
            errors.push(format!(
                "stack profile `{name}` references missing cluster config `{}`",
                profile.cluster_config
            ));
        }
        if profile.components.is_empty() {
            errors.push(format!("stack profile `{name}` must declare components"));
            continue;
        }
        let mut sorted = profile.components.clone();
        sorted.sort();
        sorted.dedup();
        if sorted.len() != profile.components.len() {
            errors.push(format!(
                "stack profile `{name}` has duplicate components; ordering must be deterministic"
            ));
        }
        if profile.components != sorted {
            errors.push(format!(
                "stack profile `{name}` components must be lexicographically sorted"
            ));
        }
        for component in &profile.components {
            let component_path = repo_root.join(component);
            if !component_path.exists() {
                errors.push(format!(
                    "stack profile `{name}` references missing component `{component}`"
                ));
            }
        }
    }
    errors.sort();
    errors.dedup();
    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_manifest_validation_checks_component_order_and_paths() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops/stack/kind")).expect("mkdir");
        std::fs::write(
            root.path().join("ops/stack/kind/cluster.yaml"),
            "kind: Cluster\n",
        )
        .expect("write cluster");
        let manifest = StackManifestToml {
            profiles: BTreeMap::from([(
                "kind".to_string(),
                StackManifestProfile {
                    kind_profile: "normal".to_string(),
                    cluster_config: "ops/stack/kind/cluster.yaml".to_string(),
                    namespace: "bijux-atlas".to_string(),
                    components: vec![
                        "ops/stack/redis/redis.yaml".to_string(),
                        "ops/observe/pack/k8s/namespace.yaml".to_string(),
                    ],
                },
            )]),
        };
        let errors = validate_stack_manifest(root.path(), &manifest);
        assert!(errors
            .iter()
            .any(|entry| entry.contains("components must be lexicographically sorted")));
        assert!(errors
            .iter()
            .any(|entry| entry.contains("missing component")));
    }

    #[test]
    fn stack_manifest_loader_reads_canonical_contract() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops/stack")).expect("mkdir");
        std::fs::write(
            root.path().join("ops/stack/stack.toml"),
            "[profiles.developer]\nkind_profile=\"kind\"\ncluster_config=\"ops/stack/kind/cluster.yaml\"\nnamespace=\"bijux-atlas\"\ncomponents=[]\n",
        )
        .expect("write manifest");

        let manifest = load_stack_manifest(root.path()).expect("load stack manifest");
        assert!(manifest.profiles.contains_key("developer"));
    }
}
