// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use serde::Deserialize;

const OPS_STACK_PROFILES_PATH: &str = "ops/stack/profiles.json";
const OPS_STACK_VERSION_MANIFEST_PATH: &str = "ops/stack/version-manifest.json";
const OPS_TOOLCHAIN_PATH: &str = "ops/inventory/toolchain.json";
const OPS_PINS_PATH: &str = "ops/inventory/pins.yaml";
const OPS_SURFACES_PATH: &str = "ops/inventory/surfaces.json";
const OPS_MIRROR_POLICY_PATH: &str = "ops/inventory/generated-committed-mirror.json";
const OPS_CONTRACTS_PATH: &str = "ops/inventory/contracts.json";
const OPS_DATASETS_MANIFEST_PATH: &str = "ops/datasets/manifest.json";

const EXPECTED_TOOLCHAIN_SCHEMA: u64 = 1;
const EXPECTED_SURFACES_SCHEMA: u64 = 2;
const EXPECTED_MIRROR_SCHEMA: u64 = 1;
const EXPECTED_CONTRACTS_SCHEMA: u64 = 1;
const EXPECTED_STACK_PROFILES_SCHEMA: u64 = 1;
const EXPECTED_STACK_VERSION_SCHEMA: u64 = 1;
const EXPECTED_PINS_SCHEMA: u64 = 1;

const INVENTORY_INPUTS: [&str; 7] = [
    OPS_STACK_PROFILES_PATH,
    OPS_STACK_VERSION_MANIFEST_PATH,
    OPS_TOOLCHAIN_PATH,
    OPS_PINS_PATH,
    OPS_SURFACES_PATH,
    OPS_MIRROR_POLICY_PATH,
    OPS_CONTRACTS_PATH,
];

#[derive(Debug, Clone)]
struct CacheEntry {
    fingerprint: u64,
    inventory: OpsInventory,
}

static OPS_INVENTORY_CACHE: OnceLock<Mutex<HashMap<PathBuf, CacheEntry>>> = OnceLock::new();

#[derive(Debug, Clone, Deserialize)]
pub struct OpsInventory {
    pub stack_profiles: StackProfilesManifest,
    pub stack_version_manifest: StackVersionManifest,
    pub toolchain: ToolchainManifest,
    pub surfaces: SurfacesManifest,
    pub mirror_policy: MirrorPolicyManifest,
    pub contracts: ContractsManifest,
}

impl OpsInventory {
    pub fn load_and_validate(ops_root: &Path) -> Result<Self, String> {
        let repo_root = ops_root
            .parent()
            .ok_or_else(|| "ops root must be under repo root".to_string())?;
        let errors = validate_ops_inventory(repo_root);
        if !errors.is_empty() {
            return Err(errors.join("; "));
        }
        load_ops_inventory_cached(repo_root)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct StackVersionManifest {
    pub schema_version: u64,
    #[serde(flatten)]
    pub components: BTreeMap<String, String>,
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
    pub tools: BTreeMap<String, ToolSpec>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolSpec {
    pub required: bool,
    pub version_regex: String,
    pub probe_argv: Vec<String>,
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

#[derive(Debug, Clone, Deserialize)]
struct PinsManifest {
    pub schema_version: u64,
    #[serde(default)]
    pub images: BTreeMap<String, String>,
    #[serde(default)]
    pub dataset_ids: Vec<String>,
    #[serde(default)]
    pub versions: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
struct DatasetsManifest {
    pub schema_version: u64,
    #[serde(default)]
    pub datasets: Vec<DatasetEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct DatasetEntry {
    pub id: String,
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

fn inventory_fingerprint(repo_root: &Path) -> Result<u64, String> {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for rel in INVENTORY_INPUTS {
        rel.hash(&mut hasher);
        let path = repo_root.join(rel);
        let bytes =
            fs::read(&path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
        bytes.hash(&mut hasher);
    }
    Ok(hasher.finish())
}

pub fn load_ops_inventory_cached(repo_root: &Path) -> Result<OpsInventory, String> {
    let canonical_root = repo_root
        .canonicalize()
        .unwrap_or_else(|_| repo_root.to_path_buf());
    let fingerprint = inventory_fingerprint(&canonical_root)?;
    let cache = OPS_INVENTORY_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(entry) = cache
        .lock()
        .map_err(|_| "ops inventory cache mutex poisoned".to_string())?
        .get(&canonical_root)
        .cloned()
    {
        if entry.fingerprint == fingerprint {
            return Ok(entry.inventory);
        }
    }
    let inventory = load_ops_inventory(&canonical_root)?;
    cache
        .lock()
        .map_err(|_| "ops inventory cache mutex poisoned".to_string())?
        .insert(
            canonical_root,
            CacheEntry {
                fingerprint,
                inventory: inventory.clone(),
            },
        );
    Ok(inventory)
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
        OPS_PINS_PATH,
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

    validate_pins_file_content(
        repo_root,
        inventory.toolchain.images.keys().cloned().collect(),
        inventory
            .stack_version_manifest
            .components
            .keys()
            .cloned()
            .collect(),
        &mut errors,
    );

    if inventory.stack_profiles.schema_version != EXPECTED_STACK_PROFILES_SCHEMA {
        errors.push(format!(
            "{OPS_STACK_PROFILES_PATH}: expected schema_version={EXPECTED_STACK_PROFILES_SCHEMA}, got {}",
            inventory.stack_profiles.schema_version
        ));
    }
    if inventory.stack_version_manifest.schema_version != EXPECTED_STACK_VERSION_SCHEMA {
        errors.push(format!(
            "{OPS_STACK_VERSION_MANIFEST_PATH}: expected schema_version={EXPECTED_STACK_VERSION_SCHEMA}, got {}",
            inventory.stack_version_manifest.schema_version
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
        errors.push(format!(
            "{OPS_TOOLCHAIN_PATH}: images map must not be empty"
        ));
    }
    if inventory.toolchain.tools.is_empty() {
        errors.push(format!("{OPS_TOOLCHAIN_PATH}: tools map must not be empty"));
    }
    for (name, spec) in &inventory.toolchain.tools {
        if name.trim().is_empty() {
            errors.push(format!("{OPS_TOOLCHAIN_PATH}: tools key must not be empty"));
        }
        if spec.version_regex.trim().is_empty() {
            errors.push(format!(
                "{OPS_TOOLCHAIN_PATH}: tool `{name}` must define version_regex"
            ));
        }
        if spec.probe_argv.is_empty() {
            errors.push(format!(
                "{OPS_TOOLCHAIN_PATH}: tool `{name}` must define probe_argv"
            ));
        }
    }
    for (name, image) in &inventory.toolchain.images {
        if image.contains(":latest") {
            errors.push(format!(
                "{OPS_TOOLCHAIN_PATH}: image `{name}` uses forbidden latest tag `{image}`"
            ));
        }
    }
    if inventory.stack_version_manifest.components.is_empty() {
        errors.push(format!(
            "{OPS_STACK_VERSION_MANIFEST_PATH}: components must not be empty"
        ));
    }
    for (name, image) in &inventory.stack_version_manifest.components {
        if image.contains(":latest") {
            errors.push(format!(
                "{OPS_STACK_VERSION_MANIFEST_PATH}: component `{name}` uses forbidden latest tag `{image}`"
            ));
        }
    }
    for name in inventory.stack_version_manifest.components.keys() {
        if !inventory.toolchain.images.contains_key(name) {
            errors.push(format!(
                "pin coverage mismatch: `{name}` is present in {OPS_STACK_VERSION_MANIFEST_PATH} but missing in {OPS_TOOLCHAIN_PATH}"
            ));
        }
    }
    for name in inventory.toolchain.images.keys() {
        if !inventory
            .stack_version_manifest
            .components
            .contains_key(name)
        {
            errors.push(format!(
                "pin coverage mismatch: `{name}` is present in {OPS_TOOLCHAIN_PATH} but missing in {OPS_STACK_VERSION_MANIFEST_PATH}"
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
        if !mirror.source.starts_with("ops/_generated/") && !repo_root.join(&mirror.source).exists()
        {
            errors.push(format!(
                "{OPS_MIRROR_POLICY_PATH}: source path missing `{}`",
                mirror.source
            ));
        }
    }
    let sorted_mirror_keys = inventory
        .mirror_policy
        .mirrors
        .iter()
        .map(|entry| entry.committed.clone())
        .collect::<Vec<_>>();
    let mut dedup = sorted_mirror_keys.clone();
    dedup.sort();
    dedup.dedup();
    if dedup.len() != sorted_mirror_keys.len() {
        errors.push(format!(
            "{OPS_MIRROR_POLICY_PATH}: mirror committed paths must be unique"
        ));
    }
    if sorted_mirror_keys != dedup {
        errors.push(format!(
            "{OPS_MIRROR_POLICY_PATH}: mirror committed paths must be sorted for deterministic output"
        ));
    }

    let allowed_top_level: BTreeSet<&str> = [
        "_evidence",
        "_examples",
        "_generated",
        "_generated.example",
        "_meta",
        "atlas-dev",
        "datasets",
        "docs",
        "e2e",
        "env",
        "fixtures",
        "helm",
        "inventory",
        "k8s",
        "kind",
        "load",
        "manifests",
        "obs",
        "observe",
        "quarantine",
        "registry",
        "report",
        "schema",
        "schemas",
        "stack",
        "tools",
        "vendor",
        "CONTRACT.md",
        "CONTROL_PLANE.md",
        "ERRORS.md",
        "INDEX.md",
        "README.md",
    ]
    .into_iter()
    .collect();
    if let Ok(entries) = fs::read_dir(repo_root.join("ops")) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !allowed_top_level.contains(name.as_ref()) {
                errors.push(format!("unexpected ops top-level entry `ops/{name}`"));
            }
        }
    }

    let bash_like = fs::read_dir(repo_root.join("ops"))
        .ok()
        .into_iter()
        .flat_map(|entries| entries.flatten())
        .flat_map(|entry| collect_files_recursive(entry.path()))
        .filter(|path| {
            path.extension()
                .and_then(|v| v.to_str())
                .is_some_and(|ext| ext == "sh" || ext == "bash")
        })
        .collect::<Vec<_>>();
    for path in bash_like {
        let rel = path
            .strip_prefix(repo_root)
            .unwrap_or(path.as_path())
            .display()
            .to_string();
        errors.push(format!(
            "forbidden bash helper outside rust control-plane: `{rel}`"
        ));
    }

    if repo_root.join("ops/_lib").exists() {
        errors.push("forbidden retired path exists: ops/_lib".to_string());
    }

    errors.sort();
    errors.dedup();
    errors
}

fn validate_pins_file_content(
    repo_root: &Path,
    toolchain_image_keys: BTreeSet<String>,
    stack_component_keys: BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    let path = repo_root.join(OPS_PINS_PATH);
    let Ok(text) = fs::read_to_string(&path) else {
        return;
    };
    let parsed: PinsManifest = match serde_yaml::from_str(&text) {
        Ok(value) => value,
        Err(err) => {
            errors.push(format!("{OPS_PINS_PATH}: invalid yaml: {err}"));
            return;
        }
    };
    if parsed.schema_version != EXPECTED_PINS_SCHEMA {
        errors.push(format!(
            "{OPS_PINS_PATH}: expected schema_version={EXPECTED_PINS_SCHEMA}, got {}",
            parsed.schema_version
        ));
    }
    if parsed.images.is_empty() {
        errors.push(format!("{OPS_PINS_PATH}: images must not be empty"));
    }
    if parsed.dataset_ids.is_empty() {
        errors.push(format!("{OPS_PINS_PATH}: dataset_ids must not be empty"));
    }
    for (name, image) in &parsed.images {
        if image.contains(":latest") {
            errors.push(format!(
                "{OPS_PINS_PATH}: image `{name}` uses forbidden latest tag"
            ));
        }
        validate_image_hash(name, image, errors);
    }

    for required in toolchain_image_keys.union(&stack_component_keys) {
        if !parsed.images.contains_key(required) {
            errors.push(format!(
                "{OPS_PINS_PATH}: missing image pin `{required}` required by toolchain/stack manifests"
            ));
        }
    }
    for key in parsed.images.keys() {
        if !toolchain_image_keys.contains(key) || !stack_component_keys.contains(key) {
            errors.push(format!(
                "{OPS_PINS_PATH}: unused image pin `{key}` not present in both toolchain and stack manifests"
            ));
        }
    }

    let mut seen_dataset_ids = BTreeSet::new();
    for id in &parsed.dataset_ids {
        if id.trim().is_empty() {
            errors.push(format!("{OPS_PINS_PATH}: dataset_ids must not contain empty entries"));
            continue;
        }
        if !seen_dataset_ids.insert(id.clone()) {
            errors.push(format!("{OPS_PINS_PATH}: duplicate dataset pin `{id}`"));
        }
    }

    let datasets_path = repo_root.join(OPS_DATASETS_MANIFEST_PATH);
    if let Ok(dataset_text) = fs::read_to_string(&datasets_path) {
        match serde_json::from_str::<DatasetsManifest>(&dataset_text) {
            Ok(manifest) => {
                if manifest.schema_version < 1 {
                    errors.push(format!(
                        "{OPS_DATASETS_MANIFEST_PATH}: schema_version must be >= 1"
                    ));
                }
                let known_ids = manifest
                    .datasets
                    .iter()
                    .map(|entry| entry.id.clone())
                    .collect::<BTreeSet<_>>();
                for known in &known_ids {
                    if !seen_dataset_ids.contains(known) {
                        errors.push(format!(
                            "{OPS_PINS_PATH}: missing dataset pin `{known}` from {OPS_DATASETS_MANIFEST_PATH}"
                        ));
                    }
                }
                for pinned in &seen_dataset_ids {
                    if !known_ids.contains(pinned) {
                        errors.push(format!(
                            "{OPS_PINS_PATH}: unused dataset pin `{pinned}` not present in {OPS_DATASETS_MANIFEST_PATH}"
                        ));
                    }
                }
            }
            Err(err) => errors.push(format!(
                "{OPS_DATASETS_MANIFEST_PATH}: invalid json for dataset pin validation: {err}"
            )),
        }
    }

    for (name, version) in &parsed.versions {
        if !is_semver(version) {
            errors.push(format!(
                "{OPS_PINS_PATH}: version `{name}` must be semver (x.y.z), got `{version}`"
            ));
        }
    }
}

fn validate_image_hash(name: &str, image: &str, errors: &mut Vec<String>) {
    let Some(at_pos) = image.find('@') else {
        return;
    };
    let digest = &image[at_pos + 1..];
    if !digest.starts_with("sha256:") {
        errors.push(format!(
            "{OPS_PINS_PATH}: image `{name}` uses unsupported digest format (expected sha256)"
        ));
        return;
    }
    let raw = &digest["sha256:".len()..];
    if raw.len() != 64 || !raw.chars().all(|c| c.is_ascii_hexdigit()) {
        errors.push(format!(
            "{OPS_PINS_PATH}: image `{name}` has invalid sha256 digest length/content"
        ));
    }
}

fn is_semver(value: &str) -> bool {
    let mut parts = value.split('.');
    let (Some(major), Some(minor), Some(patch), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    [major, minor, patch]
        .iter()
        .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
}

pub fn ops_inventory_summary(repo_root: &Path) -> Result<serde_json::Value, String> {
    let inventory = load_ops_inventory_cached(repo_root)?;
    Ok(serde_json::json!({
        "stack_profiles": inventory.stack_profiles.profiles.len(),
        "surface_actions": inventory.surfaces.actions.len(),
        "toolchain_images": inventory.toolchain.images.len(),
        "mirror_entries": inventory.mirror_policy.mirrors.len(),
            "schema_versions": {
                "stack_profiles": inventory.stack_profiles.schema_version,
                "stack_version_manifest": inventory.stack_version_manifest.schema_version,
                "toolchain": inventory.toolchain.schema_version,
                "surfaces": inventory.surfaces.schema_version,
                "mirror_policy": inventory.mirror_policy.schema_version,
            "contracts": inventory.contracts.schema_version
        }
    }))
}

fn collect_files_recursive(path: PathBuf) -> Vec<PathBuf> {
    if path.is_file() {
        return vec![path];
    }
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            out.extend(collect_files_recursive(entry.path()));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{load_ops_inventory_cached, validate_pins_file_content};
    use std::collections::BTreeSet;
    use std::fs;

    #[test]
    fn pins_file_forbids_latest_and_invalid_digest_formats() {
        let root = tempfile::tempdir().expect("tempdir");
        let ops_inventory = root.path().join("ops/inventory");
        fs::create_dir_all(&ops_inventory).expect("mkdir");
        fs::create_dir_all(root.path().join("ops/datasets")).expect("mkdir datasets");
        fs::write(
            ops_inventory.join("pins.yaml"),
            "schema_version: 1\nimages:\n  app: repo/app:latest\n  good: repo/app:v1@sha256:abc\n  bad: repo/app:v1@sha1:abc\ndataset_ids:\n  - 110/homo_sapiens/GRCh38\nversions:\n  chart: 0.1.0\n",
        )
        .expect("write pins");
        fs::write(
            root.path().join("ops/datasets/manifest.json"),
            r#"{"schema_version":1,"datasets":[{"id":"110/homo_sapiens/GRCh38"}]}"#,
        )
        .expect("write datasets");
        let mut errors = Vec::new();
        validate_pins_file_content(
            root.path(),
            BTreeSet::from(["app".to_string(), "good".to_string(), "bad".to_string()]),
            BTreeSet::from(["app".to_string(), "good".to_string(), "bad".to_string()]),
            &mut errors,
        );
        let text = errors.join("\n");
        assert!(text.contains("forbidden latest tag"), "{text}");
        assert!(text.contains("unsupported digest format"), "{text}");
        assert!(text.contains("invalid sha256 digest"), "{text}");
    }

    #[test]
    fn cached_inventory_reload_detects_content_changes() {
        let root = tempfile::tempdir().expect("tempdir");
        let repo = root.path();
        fs::create_dir_all(repo.join("ops/stack")).expect("mkdir");
        fs::create_dir_all(repo.join("ops/inventory")).expect("mkdir");
        fs::write(
            repo.join("ops/stack/profiles.json"),
            r#"{"schema_version":1,"profiles":[{"name":"dev","kind_profile":"kind","cluster_config":"ops/kind/dev.yaml"}]}"#,
        )
        .expect("write profiles");
        fs::write(
            repo.join("ops/stack/version-manifest.json"),
            r#"{"schema_version":1,"rust":"ghcr.io/x/rust:1"}"#,
        )
        .expect("write version manifest");
        fs::write(
            repo.join("ops/inventory/toolchain.json"),
            r#"{"schema_version":1,"images":{"rust":"ghcr.io/x/rust:1"},"tools":{"cargo":{"required":true,"version_regex":"1\\..*","probe_argv":["cargo","--version"]}}}"#,
        )
        .expect("write toolchain");
        fs::write(repo.join("ops/inventory/pins.yaml"), "images: {}\n").expect("write pins");
        fs::write(
            repo.join("ops/inventory/surfaces.json"),
            r#"{"schema_version":2,"actions":[{"id":"check","domain":"ops","command":["bijux","dev","atlas","check","run"]}]}"#,
        )
        .expect("write surfaces");
        fs::write(
            repo.join("ops/inventory/generated-committed-mirror.json"),
            r#"{"schema_version":1,"mirrors":[]}"#,
        )
        .expect("write mirror");
        fs::write(
            repo.join("ops/inventory/contracts.json"),
            r#"{"schema_version":1}"#,
        )
        .expect("write contracts");

        let first = load_ops_inventory_cached(repo).expect("first");
        assert_eq!(
            first.toolchain.images.get("rust"),
            Some(&"ghcr.io/x/rust:1".to_string())
        );

        fs::write(
            repo.join("ops/inventory/toolchain.json"),
            r#"{"schema_version":1,"images":{"rust":"ghcr.io/x/rust:2"},"tools":{"cargo":{"required":true,"version_regex":"1\\..*","probe_argv":["cargo","--version"]}}}"#,
        )
        .expect("rewrite toolchain");

        let second = load_ops_inventory_cached(repo).expect("second");
        assert_eq!(
            second.toolchain.images.get("rust"),
            Some(&"ghcr.io/x/rust:2".to_string())
        );
    }

    #[test]
    fn pins_file_flags_missing_and_unused_pins() {
        let root = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(root.path().join("ops/inventory")).expect("mkdir inventory");
        fs::create_dir_all(root.path().join("ops/datasets")).expect("mkdir datasets");
        fs::write(
            root.path().join("ops/inventory/pins.yaml"),
            "schema_version: 1\nimages:\n  redis: redis:7.4-alpine\n  orphan: ghcr.io/example/orphan:1.0.0\ndataset_ids:\n  - 111/homo_sapiens/GRCh38\nversions:\n  chart: not-semver\n",
        )
        .expect("write pins");
        fs::write(
            root.path().join("ops/datasets/manifest.json"),
            r#"{"schema_version":1,"datasets":[{"id":"110/homo_sapiens/GRCh38"}]}"#,
        )
        .expect("write datasets");

        let mut errors = Vec::new();
        validate_pins_file_content(
            root.path(),
            BTreeSet::from(["redis".to_string()]),
            BTreeSet::from(["redis".to_string()]),
            &mut errors,
        );

        let text = errors.join("\n");
        assert!(text.contains("unused image pin `orphan`"), "{text}");
        assert!(text.contains("missing dataset pin `110/homo_sapiens/GRCh38`"), "{text}");
        assert!(text.contains("unused dataset pin `111/homo_sapiens/GRCh38`"), "{text}");
        assert!(text.contains("must be semver"), "{text}");
    }
}
