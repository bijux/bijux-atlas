// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct LoadToml {
    pub suites: BTreeMap<String, LoadSuiteToml>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct LoadSuiteToml {
    pub script: String,
    pub dataset: String,
    pub thresholds: String,
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadManifestError {
    Read { path: PathBuf, message: String },
    Parse { path: PathBuf, message: String },
}

impl LoadManifestError {
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

pub fn load_load_manifest(repo_root: &Path) -> Result<LoadToml, LoadManifestError> {
    let path = repo_root.join("ops/load/load.toml");
    let text = std::fs::read_to_string(&path).map_err(|err| LoadManifestError::Read {
        path: path.clone(),
        message: err.to_string(),
    })?;
    toml::from_str(&text).map_err(|err| LoadManifestError::Parse {
        path,
        message: err.to_string(),
    })
}

#[must_use]
pub fn validate_load_manifest(repo_root: &Path, manifest: &LoadToml) -> Vec<String> {
    let mut errors = Vec::new();
    if manifest.suites.is_empty() {
        errors.push("load manifest must declare at least one suite".to_string());
    }
    for (suite, definition) in &manifest.suites {
        for relative_path in [
            &definition.script,
            &definition.dataset,
            &definition.thresholds,
        ] {
            if !repo_root.join(relative_path).exists() {
                errors.push(format!(
                    "load suite `{suite}` references missing file `{relative_path}`"
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
    fn load_manifest_parses_and_validates() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops/load")).expect("mkdir");
        std::fs::write(
            root.path().join("ops/load/load.toml"),
            "[suites.smoke]\nscript=\"ops/load/k6/suites/mixed-80-20.js\"\ndataset=\"ops/load/queries/pinned-v1.json\"\nthresholds=\"ops/load/thresholds/mixed.thresholds.json\"\n[suites.smoke.env]\nK6_OUT=\"json=/tmp/out.json\"\n",
        )
        .expect("write");
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
        let parsed = load_load_manifest(root.path()).expect("parse");
        let errors = validate_load_manifest(root.path(), &parsed);
        assert!(errors.is_empty());
    }
}
