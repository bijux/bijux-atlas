// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

pub use crate::stack::manifest::{StackManifestProfile, StackManifestToml};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceStackError {
    Manifest(String),
    Schema(String),
}

impl WorkspaceStackError {
    #[must_use]
    pub fn detail(&self) -> String {
        match self {
            Self::Manifest(detail) | Self::Schema(detail) => detail.clone(),
        }
    }
}

pub fn load_stack_manifest(repo_root: &Path) -> Result<StackManifestToml, WorkspaceStackError> {
    crate::stack::manifest::load_stack_manifest(repo_root).map_err(map_stack_manifest_error)
}

#[must_use]
pub fn validate_stack_manifest(repo_root: &Path, manifest: &StackManifestToml) -> Vec<String> {
    crate::stack::manifest::validate_stack_manifest(repo_root, manifest)
}

fn map_stack_manifest_error(
    error: crate::stack::manifest::StackManifestLoadError,
) -> WorkspaceStackError {
    match error {
        crate::stack::manifest::StackManifestLoadError::Read { .. } => {
            WorkspaceStackError::Manifest(error.detail())
        }
        crate::stack::manifest::StackManifestLoadError::Parse { .. } => {
            WorkspaceStackError::Schema(error.detail())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn workspace_stack_loads_manifest_through_owned_surface() {
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

    #[test]
    fn workspace_stack_validates_manifest_through_owned_surface() {
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
}
