// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::path_contracts;

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct StackProfiles {
    pub profiles: Vec<StackProfile>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct StackProfile {
    pub name: String,
    pub kind_profile: String,
    pub cluster_config: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct OpsProfileRegistry {
    pub schema_version: u64,
    pub profiles: Vec<OpsProfileSpec>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct OpsProfileSpec {
    pub id: String,
    pub description: String,
    #[serde(rename = "class")]
    pub class_name: String,
    pub safety_level: String,
    pub required_tools: Vec<String>,
    pub allowed_namespaces: Vec<String>,
    pub required_services: Vec<String>,
    pub optional_components: Vec<String>,
    pub doc_link: String,
    pub config_source_paths: Vec<String>,
}

pub fn load_profiles(ops_root: &Path) -> Result<Vec<StackProfile>, String> {
    let path = path_contracts::atlas_stack_profiles_manifest_from_ops_root(ops_root);
    let text = std::fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let payload: StackProfiles = serde_json::from_str(&text)
        .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
    Ok(payload.profiles)
}

pub fn load_profile_registry(ops_root: &Path) -> Result<OpsProfileRegistry, String> {
    let path = path_contracts::atlas_stack_profile_registry_from_ops_root(ops_root);
    let text = std::fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let payload: OpsProfileRegistry = serde_json::from_str(&text)
        .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
    if payload.schema_version == 0 {
        return Err(format!(
            "invalid {}: schema_version must be >=1",
            path.display()
        ));
    }
    Ok(payload)
}

pub fn resolve_profile(
    requested: Option<String>,
    profiles: &[StackProfile],
) -> Result<StackProfile, String> {
    if profiles.is_empty() {
        return Err("no profiles declared in ops/stack/profiles.json".to_string());
    }
    if let Some(name) = requested {
        return profiles
            .iter()
            .find(|profile| profile.name == name)
            .cloned()
            .ok_or_else(|| format!("unknown profile `{name}`"));
    }
    profiles
        .iter()
        .find(|profile| profile.name == "developer")
        .cloned()
        .or_else(|| profiles.first().cloned())
        .ok_or_else(|| "no default profile available".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_root() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .expect("repository root")
            .to_path_buf()
    }

    #[test]
    fn profile_catalog_loaders_read_canonical_ops_manifests() {
        let ops_root = repo_root().join("ops");
        let profiles = load_profiles(&ops_root).expect("profiles");
        let registry = load_profile_registry(&ops_root).expect("registry");
        assert!(!profiles.is_empty());
        assert!(!registry.profiles.is_empty());
        assert!(registry.schema_version >= 1);
    }

    #[test]
    fn resolve_profile_prefers_requested_then_developer_default() {
        let profiles = vec![
            StackProfile {
                name: "developer".to_string(),
                kind_profile: "kind".to_string(),
                cluster_config: "ops/stack/kind/cluster.yaml".to_string(),
            },
            StackProfile {
                name: "perf".to_string(),
                kind_profile: "perf".to_string(),
                cluster_config: "ops/stack/kind/cluster-perf.yaml".to_string(),
            },
        ];
        assert_eq!(
            resolve_profile(Some("perf".to_string()), &profiles)
                .expect("requested profile")
                .name,
            "perf"
        );
        assert_eq!(
            resolve_profile(None, &profiles)
                .expect("default profile")
                .name,
            "developer"
        );
    }
}
