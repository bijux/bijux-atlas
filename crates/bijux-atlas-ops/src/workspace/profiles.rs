// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

use crate::reference::ops_paths::OpsRootError;
pub use crate::stack::profile_catalog::{OpsProfileRegistry, StackProfile};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceProfilesError {
    Manifest(String),
    Schema(String),
    Profile(String),
}

impl WorkspaceProfilesError {
    #[must_use]
    pub fn detail(&self) -> String {
        match self {
            Self::Manifest(detail) | Self::Schema(detail) | Self::Profile(detail) => detail.clone(),
        }
    }
}

pub fn resolve_ops_root(
    repo_root: &Path,
    ops_root: Option<PathBuf>,
) -> Result<PathBuf, WorkspaceProfilesError> {
    crate::reference::ops_paths::resolve_ops_root(repo_root, ops_root).map_err(map_ops_root_error)
}

pub fn load_profiles(ops_root: &Path) -> Result<Vec<StackProfile>, WorkspaceProfilesError> {
    crate::stack::profile_catalog::load_profiles(ops_root).map_err(WorkspaceProfilesError::Manifest)
}

pub fn load_profile_registry(
    ops_root: &Path,
) -> Result<OpsProfileRegistry, WorkspaceProfilesError> {
    crate::stack::profile_catalog::load_profile_registry(ops_root)
        .map_err(WorkspaceProfilesError::Schema)
}

pub fn resolve_profile(
    requested: Option<String>,
    profiles: &[StackProfile],
) -> Result<StackProfile, WorkspaceProfilesError> {
    crate::stack::profile_catalog::resolve_profile(requested, profiles)
        .map_err(WorkspaceProfilesError::Profile)
}

pub fn load_profile_values_entry(
    repo_root: &Path,
    profile: &str,
) -> Result<Option<serde_json::Value>, WorkspaceProfilesError> {
    let path = repo_root.join("ops/k8s/values/profiles.json");
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&path).map_err(|err| {
        WorkspaceProfilesError::Manifest(format!("failed to read {}: {err}", path.display()))
    })?;
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|err| {
        WorkspaceProfilesError::Schema(format!("failed to parse {}: {err}", path.display()))
    })?;
    Ok(value
        .get("profiles")
        .and_then(|rows| rows.as_array())
        .and_then(|rows| {
            rows.iter()
                .find(|row| row.get("id").and_then(|v| v.as_str()) == Some(profile))
                .cloned()
        }))
}

pub fn resolve_profile_values_file(
    repo_root: &Path,
    profile: &str,
) -> Result<PathBuf, WorkspaceProfilesError> {
    let path = repo_root
        .join("ops/k8s/values")
        .join(format!("{profile}.yaml"));
    if path.exists() {
        Ok(path)
    } else {
        Err(WorkspaceProfilesError::Manifest(format!(
            "missing values file {}; expected profile values at ops/k8s/values/{profile}.yaml",
            path.display()
        )))
    }
}

#[must_use]
pub fn simulation_namespace(profile: &str, override_namespace: Option<&str>) -> String {
    override_namespace
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("bijux-atlas-{profile}"))
}

fn map_ops_root_error(error: OpsRootError) -> WorkspaceProfilesError {
    WorkspaceProfilesError::Manifest(error.detail())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_profiles_resolve_ops_root_through_owned_surface() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops")).expect("create ops root");

        let resolved = resolve_ops_root(root.path(), None).expect("resolve ops root");

        assert!(resolved.ends_with("ops"));
    }

    #[test]
    fn workspace_profiles_load_profile_values_entry_filters_by_profile_id() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops/k8s/values")).expect("create values root");
        std::fs::write(
            root.path().join("ops/k8s/values/profiles.json"),
            r#"{"profiles":[{"id":"developer","namespace":"bijux-atlas"},{"id":"perf","namespace":"bijux-atlas-perf"}]}"#,
        )
        .expect("write profile values");

        let entry = load_profile_values_entry(root.path(), "perf")
            .expect("load profile values")
            .expect("profile entry");

        assert_eq!(entry["id"], "perf");
        assert_eq!(entry["namespace"], "bijux-atlas-perf");
    }

    #[test]
    fn workspace_profiles_load_profile_values_entry_returns_none_when_missing() {
        let root = tempfile::tempdir().expect("tempdir");

        let entry =
            load_profile_values_entry(root.path(), "developer").expect("load profile values");

        assert!(entry.is_none());
    }

    #[test]
    fn workspace_profiles_resolve_values_file_reports_owned_contract_path() {
        let root = tempfile::tempdir().expect("tempdir");

        let error =
            resolve_profile_values_file(root.path(), "developer").expect_err("missing values file");

        assert_eq!(
            error.detail(),
            format!(
                "missing values file {}; expected profile values at ops/k8s/values/developer.yaml",
                root.path().join("ops/k8s/values/developer.yaml").display()
            )
        );
    }

    #[test]
    fn workspace_profiles_simulation_namespace_prefers_non_empty_override() {
        assert_eq!(
            simulation_namespace("perf", Some("custom-namespace")),
            "custom-namespace"
        );
        assert_eq!(simulation_namespace("perf", Some("  ")), "bijux-atlas-perf");
        assert_eq!(simulation_namespace("perf", None), "bijux-atlas-perf");
    }
}
