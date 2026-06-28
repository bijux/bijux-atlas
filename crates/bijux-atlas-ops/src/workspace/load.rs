// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

pub use crate::load::manifest::{LoadSuiteToml, LoadToml};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceLoadError {
    Manifest(String),
    Schema(String),
}

impl WorkspaceLoadError {
    #[must_use]
    pub fn detail(&self) -> String {
        match self {
            Self::Manifest(detail) | Self::Schema(detail) => detail.clone(),
        }
    }
}

pub fn load_load_manifest(repo_root: &Path) -> Result<LoadToml, WorkspaceLoadError> {
    crate::load::manifest::load_load_manifest(repo_root).map_err(map_load_manifest_error)
}

#[must_use]
pub fn validate_load_manifest(repo_root: &Path, manifest: &LoadToml) -> Vec<String> {
    crate::load::manifest::validate_load_manifest(repo_root, manifest)
}

fn map_load_manifest_error(error: crate::load::manifest::LoadManifestError) -> WorkspaceLoadError {
    match error {
        crate::load::manifest::LoadManifestError::Read { .. } => {
            WorkspaceLoadError::Manifest(error.detail())
        }
        crate::load::manifest::LoadManifestError::Parse { .. } => {
            WorkspaceLoadError::Schema(error.detail())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_load_parses_manifest_through_owned_surface() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops/load")).expect("mkdir");
        std::fs::write(
            root.path().join("ops/load/load.toml"),
            "[suites.smoke]\nscript=\"ops/load/k6/suites/mixed-80-20.js\"\ndataset=\"ops/load/queries/pinned-v1.json\"\nthresholds=\"ops/load/thresholds/mixed.thresholds.json\"\n[suites.smoke.env]\nK6_OUT=\"json=/tmp/out.json\"\n",
        )
        .expect("write manifest");
        std::fs::create_dir_all(root.path().join("ops/load/k6/suites")).expect("mkdir suites");
        std::fs::create_dir_all(root.path().join("ops/load/queries")).expect("mkdir queries");
        std::fs::create_dir_all(root.path().join("ops/load/thresholds")).expect("mkdir thresholds");
        std::fs::write(root.path().join("ops/load/k6/suites/mixed-80-20.js"), "").expect("script");
        std::fs::write(root.path().join("ops/load/queries/pinned-v1.json"), "{}").expect("dataset");
        std::fs::write(
            root.path()
                .join("ops/load/thresholds/mixed.thresholds.json"),
            "{}",
        )
        .expect("thresholds");

        let manifest = load_load_manifest(root.path()).expect("load manifest");

        assert!(manifest.suites.contains_key("smoke"));
    }

    #[test]
    fn workspace_load_validates_manifest_through_owned_surface() {
        let root = tempfile::tempdir().expect("tempdir");
        let manifest = LoadToml {
            suites: std::collections::BTreeMap::from([(
                "smoke".to_string(),
                LoadSuiteToml {
                    script: "ops/load/k6/suites/mixed-80-20.js".to_string(),
                    dataset: "ops/load/queries/pinned-v1.json".to_string(),
                    thresholds: "ops/load/thresholds/mixed.thresholds.json".to_string(),
                    env: std::collections::BTreeMap::new(),
                },
            )]),
        };

        let errors = validate_load_manifest(root.path(), &manifest);

        assert_eq!(errors.len(), 3);
        assert!(errors
            .iter()
            .all(|entry| entry.contains("load suite `smoke` references missing file")));
    }
}
