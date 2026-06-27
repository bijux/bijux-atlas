// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::path_contracts;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ToolsToml {
    pub tools: Vec<ToolTomlEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ToolTomlEntry {
    pub name: String,
    pub required: bool,
    pub version_regex: String,
    pub probe_argv: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolsManifestError {
    Read { path: PathBuf, message: String },
    Parse { path: PathBuf, message: String },
}

impl ToolsManifestError {
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

pub fn load_tools_manifest(repo_root: &Path) -> Result<ToolsToml, ToolsManifestError> {
    let path = path_contracts::atlas_tools_manifest_from_repo_root(repo_root);
    let text = std::fs::read_to_string(&path).map_err(|err| ToolsManifestError::Read {
        path: path.clone(),
        message: err.to_string(),
    })?;
    toml::from_str(&text).map_err(|err| ToolsManifestError::Parse {
        path,
        message: err.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tools_manifest_loader_reads_canonical_contract() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops/inventory")).expect("mkdir");
        std::fs::write(
            root.path().join("ops/inventory/tools.toml"),
            "[[tools]]\nname=\"helm\"\nrequired=true\nversion_regex=\"(\\\\d+\\\\.\\\\d+\\\\.\\\\d+)\"\nprobe_argv=[\"version\",\"--short\"]\n",
        )
        .expect("write manifest");

        let manifest = load_tools_manifest(root.path()).expect("load manifest");
        assert_eq!(manifest.tools.len(), 1);
        assert_eq!(manifest.tools[0].name, "helm");
    }
}
