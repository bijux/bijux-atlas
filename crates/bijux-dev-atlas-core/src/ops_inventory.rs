#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::Deserialize;

const OPS_STACK_PROFILES_PATH: &str = "ops/stack/profiles.json";
const OPS_STACK_VERSION_MANIFEST_PATH: &str = "ops/stack/version-manifest.json";
const OPS_TOOLCHAIN_PATH: &str = "ops/inventory/toolchain.json";
const OPS_SURFACES_PATH: &str = "ops/inventory/surfaces.json";
const OPS_MIRROR_POLICY_PATH: &str = "ops/inventory/generated-committed-mirror.json";
const OPS_CONTRACTS_PATH: &str = "ops/inventory/contracts.json";

const EXPECTED_TOOLCHAIN_SCHEMA: u64 = 1;
const EXPECTED_SURFACES_SCHEMA: u64 = 2;
const EXPECTED_MIRROR_SCHEMA: u64 = 1;
const EXPECTED_CONTRACTS_SCHEMA: u64 = 1;
const EXPECTED_STACK_PROFILES_SCHEMA: u64 = 1;

#[derive(Debug, Clone, Deserialize)]
pub struct OpsInventory {
    pub stack_profiles: StackProfilesManifest,
    pub stack_version_manifest: serde_json::Value,
    pub toolchain: ToolchainManifest,
    pub surfaces: SurfacesManifest,
    pub mirror_policy: MirrorPolicyManifest,
    pub contracts: ContractsManifest,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StackProfilesManifest {
    pub schema_version: u64,
    pub profiles: Vec<StackProfile>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StackProfile {
    pub name: String,
    pub kind_profile: String,
    pub cluster_config: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolchainManifest {
    pub schema_version: u64,
    pub images: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SurfacesManifest {
    pub schema_version: u64,
    pub actions: Vec<SurfaceAction>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SurfaceAction {
    pub id: String,
    pub domain: String,
    pub command: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MirrorPolicyManifest {
    pub schema_version: u64,
    pub mirrors: Vec<MirrorEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MirrorEntry {
    pub committed: String,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContractsManifest {
    pub schema_version: u64,
}

fn load_json<T: for<'de> Deserialize<'de>>(repo_root: &Path, rel: &str) -> Result<T, String> {
    let path = repo_root.join(rel);
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

pub fn load_ops_inventory(repo_root: &Path) -> Result<OpsInventory, String> {
    Ok(OpsInventory {
        stack_profiles: load_json(repo_root, OPS_STACK_PROFILES_PATH)?,
        stack_version_manifest: load_json(repo_root, OPS_STACK_VERSION_MANIFEST_PATH)?,
        toolchain: load_json(repo_root, OPS_TOOLCHAIN_PATH)?,
        surfaces: load_json(repo_root, OPS_SURFACES_PATH)?,
        mirror_policy: load_json(repo_root, OPS_MIRROR_POLICY_PATH)?,
        contracts: load_json(repo_root, OPS_CONTRACTS_PATH)?,
    })
}

pub fn validate_ops_inventory(repo_root: &Path) -> Vec<String> {
    let mut errors = Vec::new();

    for rel in [
        "ops/CONTRACT.md",
        "ops/ERRORS.md",
        "ops/INDEX.md",
        OPS_STACK_PROFILES_PATH,
        OPS_STACK_VERSION_MANIFEST_PATH,
        OPS_TOOLCHAIN_PATH,
        OPS_SURFACES_PATH,
        OPS_MIRROR_POLICY_PATH,
        OPS_CONTRACTS_PATH,
    ] {
        let path = repo_root.join(rel);
        if !path.exists() {
            errors.push(format!("missing required ops input `{rel}`"));
        }
    }

    let inventory = match load_ops_inventory(repo_root) {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return errors;
        }
    };

    if inventory.stack_profiles.schema_version != EXPECTED_STACK_PROFILES_SCHEMA {
        errors.push(format!(
            "{OPS_STACK_PROFILES_PATH}: expected schema_version={EXPECTED_STACK_PROFILES_SCHEMA}, got {}",
            inventory.stack_profiles.schema_version
        ));
    }
    if inventory.toolchain.schema_version != EXPECTED_TOOLCHAIN_SCHEMA {
        errors.push(format!(
            "{OPS_TOOLCHAIN_PATH}: expected schema_version={EXPECTED_TOOLCHAIN_SCHEMA}, got {}",
            inventory.toolchain.schema_version
        ));
    }
    if inventory.surfaces.schema_version != EXPECTED_SURFACES_SCHEMA {
        errors.push(format!(
            "{OPS_SURFACES_PATH}: expected schema_version={EXPECTED_SURFACES_SCHEMA}, got {}",
            inventory.surfaces.schema_version
        ));
    }
    if inventory.mirror_policy.schema_version != EXPECTED_MIRROR_SCHEMA {
        errors.push(format!(
            "{OPS_MIRROR_POLICY_PATH}: expected schema_version={EXPECTED_MIRROR_SCHEMA}, got {}",
            inventory.mirror_policy.schema_version
        ));
    }
    if inventory.contracts.schema_version != EXPECTED_CONTRACTS_SCHEMA {
        errors.push(format!(
            "{OPS_CONTRACTS_PATH}: expected schema_version={EXPECTED_CONTRACTS_SCHEMA}, got {}",
            inventory.contracts.schema_version
        ));
    }

    if inventory.stack_profiles.profiles.is_empty() {
        errors.push("ops stack profiles are empty".to_string());
    }

    let mut seen_profiles = BTreeSet::new();
    for profile in &inventory.stack_profiles.profiles {
        if !seen_profiles.insert(profile.name.clone()) {
            errors.push(format!(
                "{OPS_STACK_PROFILES_PATH}: duplicate profile `{}`",
                profile.name
            ));
        }
        if profile.kind_profile.trim().is_empty() {
            errors.push(format!(
                "{OPS_STACK_PROFILES_PATH}: profile `{}` has empty kind_profile",
                profile.name
            ));
        }
        let cluster_config = repo_root.join(&profile.cluster_config);
        if !cluster_config.exists() {
            errors.push(format!(
                "{OPS_STACK_PROFILES_PATH}: profile `{}` references missing cluster config `{}`",
                profile.name, profile.cluster_config
            ));
        }
    }

    if inventory.toolchain.images.is_empty() {
        errors.push(format!("{OPS_TOOLCHAIN_PATH}: images map must not be empty"));
    }
    for (name, image) in &inventory.toolchain.images {
        if image.contains(":latest") {
            errors.push(format!(
                "{OPS_TOOLCHAIN_PATH}: image `{name}` uses forbidden latest tag `{image}`"
            ));
        }
    }

    let mut seen_action_ids = BTreeSet::new();
    for action in &inventory.surfaces.actions {
        if action.id.trim().is_empty() {
            errors.push(format!("{OPS_SURFACES_PATH}: action id must not be empty"));
            continue;
        }
        if !seen_action_ids.insert(action.id.clone()) {
            errors.push(format!(
                "{OPS_SURFACES_PATH}: duplicate action id `{}`",
                action.id
            ));
        }
        if action.domain.trim().is_empty() {
            errors.push(format!(
                "{OPS_SURFACES_PATH}: action `{}` has empty domain",
                action.id
            ));
        }
        if action.command.is_empty() {
            errors.push(format!(
                "{OPS_SURFACES_PATH}: action `{}` has empty command",
                action.id
            ));
        }
        let joined = action.command.join(" ");
        if joined.contains("scripts/") || joined.contains(".sh") {
            errors.push(format!(
                "{OPS_SURFACES_PATH}: action `{}` references forbidden script entrypoint `{joined}`",
                action.id
            ));
        }
    }

    for mirror in &inventory.mirror_policy.mirrors {
        if !repo_root.join(&mirror.committed).exists() {
            errors.push(format!(
                "{OPS_MIRROR_POLICY_PATH}: committed path missing `{}`",
                mirror.committed
            ));
        }
        if !mirror.source.starts_with("ops/_generated/")
            && !repo_root.join(&mirror.source).exists()
        {
            errors.push(format!(
                "{OPS_MIRROR_POLICY_PATH}: source path missing `{}`",
                mirror.source
            ));
        }
    }

    if repo_root.join("ops/_lib").exists() {
        errors.push("forbidden legacy path exists: ops/_lib".to_string());
    }

    errors.sort();
    errors.dedup();
    errors
}

pub fn ops_inventory_summary(repo_root: &Path) -> Result<serde_json::Value, String> {
    let inventory = load_ops_inventory(repo_root)?;
    Ok(serde_json::json!({
        "stack_profiles": inventory.stack_profiles.profiles.len(),
        "surface_actions": inventory.surfaces.actions.len(),
        "toolchain_images": inventory.toolchain.images.len(),
        "mirror_entries": inventory.mirror_policy.mirrors.len(),
        "schema_versions": {
            "stack_profiles": inventory.stack_profiles.schema_version,
            "toolchain": inventory.toolchain.schema_version,
            "surfaces": inventory.surfaces.schema_version,
            "mirror_policy": inventory.mirror_policy.schema_version,
            "contracts": inventory.contracts.schema_version
        }
    }))
}
