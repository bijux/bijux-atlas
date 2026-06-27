// SPDX-License-Identifier: Apache-2.0

use super::domain_support::OpsProfileRegistry;
use crate::ops_support::StackManifestToml;
use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ToolsToml {
    pub(crate) tools: Vec<ToolTomlEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct LoadToml {
    pub(crate) suites: std::collections::BTreeMap<String, LoadSuiteToml>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct LoadSuiteToml {
    pub(crate) script: String,
    pub(crate) dataset: String,
    pub(crate) thresholds: String,
    pub(crate) env: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ToolTomlEntry {
    pub(crate) name: String,
    pub(crate) required: bool,
    pub(crate) version_regex: String,
    pub(crate) probe_argv: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct StackPinsToml {
    pub(crate) charts: std::collections::BTreeMap<String, String>,
    pub(crate) images: std::collections::BTreeMap<String, String>,
    pub(crate) crds: std::collections::BTreeMap<String, String>,
}

pub(crate) fn resolve_ops_root(
    repo_root: &Path,
    ops_root: Option<PathBuf>,
) -> Result<PathBuf, OpsCommandError> {
    let path = ops_root.unwrap_or_else(|| repo_root.join("ops"));
    path.canonicalize().map_err(|err| {
        OpsCommandError::Manifest(format!("cannot resolve ops root {}: {err}", path.display()))
    })
}

pub(crate) fn load_profiles(ops_root: &Path) -> Result<Vec<StackProfile>, OpsCommandError> {
    bijux_atlas_ops::stack::profile_catalog::load_profiles(ops_root)
        .map_err(OpsCommandError::Profile)
}

pub(crate) fn load_profile_registry(
    ops_root: &Path,
) -> Result<OpsProfileRegistry, OpsCommandError> {
    bijux_atlas_ops::stack::profile_catalog::load_profile_registry(ops_root)
        .map_err(OpsCommandError::Profile)
}

fn load_toolchain_inventory(ops_root: &Path) -> Result<ToolchainInventory, OpsCommandError> {
    let path = ops_root.join("inventory/toolchain.json");
    let text = std::fs::read_to_string(&path).map_err(|err| {
        OpsCommandError::Manifest(format!("failed to read {}: {err}", path.display()))
    })?;
    serde_json::from_str(&text).map_err(|err| {
        OpsCommandError::Schema(format!("failed to parse {}: {err}", path.display()))
    })
}

pub(crate) fn load_tools_manifest(repo_root: &Path) -> Result<ToolsToml, OpsCommandError> {
    let path = repo_root.join("ops/inventory/tools.toml");
    let text = std::fs::read_to_string(&path).map_err(|err| {
        OpsCommandError::Manifest(format!("failed to read {}: {err}", path.display()))
    })?;
    toml::from_str(&text).map_err(|err| {
        OpsCommandError::Schema(format!("failed to parse {}: {err}", path.display()))
    })
}

pub(crate) fn load_stack_pins(repo_root: &Path) -> Result<StackPinsToml, OpsCommandError> {
    let path = repo_root.join("ops/inventory/pins.yaml");
    let text = std::fs::read_to_string(&path).map_err(|err| {
        OpsCommandError::Manifest(format!("failed to read {}: {err}", path.display()))
    })?;
    let value: serde_yaml::Value = serde_yaml::from_str(&text).map_err(|err| {
        OpsCommandError::Schema(format!("failed to parse {}: {err}", path.display()))
    })?;
    let images = value
        .get("images")
        .and_then(serde_yaml::Value::as_mapping)
        .map(|mapping| {
            mapping
                .iter()
                .filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_str()?.to_string())))
                .collect::<std::collections::BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let versions = value
        .get("versions")
        .and_then(serde_yaml::Value::as_mapping)
        .map(|mapping| {
            mapping
                .iter()
                .filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_str()?.to_string())))
                .collect::<std::collections::BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let mut charts = std::collections::BTreeMap::new();
    if let Some(chart) = versions.get("chart") {
        charts.insert("bijux_atlas".to_string(), chart.clone());
    }
    let mut crds = std::collections::BTreeMap::new();
    if let Some(crd) = versions.get("prometheus_operator_crd") {
        crds.insert("prometheus_operator".to_string(), crd.clone());
    }
    Ok(StackPinsToml {
        charts,
        images,
        crds,
    })
}

pub(crate) fn load_stack_manifest(repo_root: &Path) -> Result<StackManifestToml, OpsCommandError> {
    bijux_atlas_ops::stack::manifest::load_stack_manifest(repo_root).map_err(|err| match err {
        bijux_atlas_ops::stack::manifest::StackManifestLoadError::Read { .. } => {
            OpsCommandError::Manifest(err.detail())
        }
        bijux_atlas_ops::stack::manifest::StackManifestLoadError::Parse { .. } => {
            OpsCommandError::Schema(err.detail())
        }
    })
}

pub(crate) fn load_load_manifest(repo_root: &Path) -> Result<LoadToml, OpsCommandError> {
    let path = repo_root.join("ops/load/load.toml");
    let text = std::fs::read_to_string(&path).map_err(|err| {
        OpsCommandError::Manifest(format!("failed to read {}: {err}", path.display()))
    })?;
    toml::from_str(&text).map_err(|err| {
        OpsCommandError::Schema(format!("failed to parse {}: {err}", path.display()))
    })
}

pub(crate) fn validate_load_manifest(
    repo_root: &Path,
    manifest: &LoadToml,
) -> Result<Vec<String>, OpsCommandError> {
    let mut errors = Vec::new();
    if manifest.suites.is_empty() {
        errors.push("load manifest must declare at least one suite".to_string());
    }
    for (suite, def) in &manifest.suites {
        for rel in [&def.script, &def.dataset, &def.thresholds] {
            if !repo_root.join(rel).exists() {
                errors.push(format!(
                    "load suite `{suite}` references missing file `{rel}`"
                ));
            }
        }
    }
    errors.sort();
    errors.dedup();
    Ok(errors)
}

pub(crate) fn validate_stack_manifest(
    repo_root: &Path,
    manifest: &StackManifestToml,
) -> Result<Vec<String>, OpsCommandError> {
    Ok(bijux_atlas_ops::stack::manifest::validate_stack_manifest(
        repo_root, manifest,
    ))
}

pub(crate) fn resolve_profile(
    requested: Option<String>,
    profiles: &[StackProfile],
) -> Result<StackProfile, OpsCommandError> {
    bijux_atlas_ops::stack::profile_catalog::resolve_profile(requested, profiles)
        .map_err(OpsCommandError::Profile)
}

pub(crate) fn run_id_or_default(raw: Option<String>) -> Result<RunId, String> {
    raw.map(|v| RunId::parse(&v))
        .transpose()?
        .map_or_else(|| Ok(RunId::from_seed("ops_run")), Ok)
}

pub(crate) fn load_toolchain_inventory_for_ops(
    ops_root: &Path,
) -> Result<ToolchainInventory, OpsCommandError> {
    load_toolchain_inventory(ops_root)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn tools_manifest_parses() {
        let root = tempfile::tempdir().expect("tempdir");
        let tools_dir = root.path().join("ops/inventory");
        std::fs::create_dir_all(&tools_dir).expect("mkdir");
        std::fs::write(
            tools_dir.join("tools.toml"),
            "[[tools]]\nname=\"helm\"\nrequired=true\nversion_regex=\"(\\\\d+\\\\.\\\\d+\\\\.\\\\d+)\"\nprobe_argv=[\"version\",\"--short\"]\n",
        )
        .expect("write");
        let parsed = load_tools_manifest(root.path()).expect("parse");
        assert_eq!(parsed.tools.len(), 1);
        assert_eq!(parsed.tools[0].name, "helm");
    }

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
        let errors = validate_load_manifest(root.path(), &parsed).expect("validate");
        assert!(errors.is_empty());
    }
}
