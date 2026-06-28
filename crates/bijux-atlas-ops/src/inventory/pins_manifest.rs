// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::path_contracts;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct StackPinsToml {
    pub charts: BTreeMap<String, String>,
    pub images: BTreeMap<String, String>,
    pub crds: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinsManifestError {
    Read { path: PathBuf, message: String },
    Parse { path: PathBuf, message: String },
}

impl PinsManifestError {
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

pub fn load_pins_manifest(repo_root: &Path) -> Result<StackPinsToml, PinsManifestError> {
    let path = path_contracts::atlas_pins_manifest_from_repo_root(repo_root);
    let text = std::fs::read_to_string(&path).map_err(|err| PinsManifestError::Read {
        path: path.clone(),
        message: err.to_string(),
    })?;
    let value: serde_yaml::Value =
        serde_yaml::from_str(&text).map_err(|err| PinsManifestError::Parse {
            path: path.clone(),
            message: err.to_string(),
        })?;

    let images = value
        .get("images")
        .and_then(serde_yaml::Value::as_mapping)
        .map(|mapping| {
            mapping
                .iter()
                .filter_map(|(key, entry)| {
                    Some((key.as_str()?.to_string(), entry.as_str()?.to_string()))
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    let versions = value
        .get("versions")
        .and_then(serde_yaml::Value::as_mapping)
        .map(|mapping| {
            mapping
                .iter()
                .filter_map(|(key, entry)| {
                    Some((key.as_str()?.to_string(), entry.as_str()?.to_string()))
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    let mut charts = BTreeMap::new();
    if let Some(chart) = versions.get("chart") {
        charts.insert("bijux_atlas".to_string(), chart.clone());
    }

    let mut crds = BTreeMap::new();
    if let Some(crd) = versions.get("prometheus_operator_crd") {
        crds.insert("prometheus_operator".to_string(), crd.clone());
    }

    Ok(StackPinsToml {
        charts,
        images,
        crds,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pins_manifest_loader_reads_canonical_contract() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops/inventory")).expect("mkdir");
        std::fs::write(
            root.path().join("ops/inventory/pins.yaml"),
            "images:\n  redis: \"redis@sha256:123\"\nversions:\n  chart: \"1.2.3\"\n  prometheus_operator_crd: \"0.78.2\"\n",
        )
        .expect("write pins");

        let pins = load_pins_manifest(root.path()).expect("load pins");
        assert_eq!(
            pins.images.get("redis"),
            Some(&"redis@sha256:123".to_string())
        );
        assert_eq!(pins.charts.get("bijux_atlas"), Some(&"1.2.3".to_string()));
        assert_eq!(
            pins.crds.get("prometheus_operator"),
            Some(&"0.78.2".to_string())
        );
    }
}
