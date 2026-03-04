// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct OpsProfileRegistry {
    schema_version: u64,
    profiles: Vec<OpsProfile>,
}

#[derive(Debug, Deserialize)]
struct OpsProfile {
    id: String,
    doc_link: String,
    safety_level: String,
    config_source_paths: Vec<String>,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

fn load_registry() -> OpsProfileRegistry {
    let path = repo_root().join("ops/stack/profile-registry.json");
    let text = fs::read_to_string(path).expect("read registry");
    serde_json::from_str(&text).expect("parse registry")
}

#[test]
fn profile_registry_has_no_duplicate_ids() {
    let registry = load_registry();
    assert_eq!(registry.schema_version, 1);
    let mut ids = BTreeSet::new();
    for profile in &registry.profiles {
        assert!(
            ids.insert(profile.id.clone()),
            "duplicate profile id {}",
            profile.id
        );
    }
}

#[test]
fn profile_registry_order_is_deterministic() {
    let registry = load_registry();
    let ids = registry
        .profiles
        .iter()
        .map(|profile| profile.id.as_str())
        .collect::<Vec<_>>();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(ids, sorted, "profile ids must be lexicographically sorted");
}

#[test]
fn every_profile_has_doc_link() {
    let root = repo_root();
    let registry = load_registry();
    for profile in &registry.profiles {
        assert!(
            !profile.doc_link.trim().is_empty(),
            "profile {} missing doc_link",
            profile.id
        );
        let path = root.join(&profile.doc_link);
        assert!(
            path.exists(),
            "profile {} doc_link target missing: {}",
            profile.id,
            profile.doc_link
        );
    }
}

#[test]
fn every_profile_declares_safety_level() {
    let registry = load_registry();
    for profile in &registry.profiles {
        assert!(
            !profile.safety_level.trim().is_empty(),
            "profile {} missing safety_level",
            profile.id
        );
    }
}

#[test]
fn every_profile_declares_config_source_paths() {
    let root = repo_root();
    let registry = load_registry();
    for profile in &registry.profiles {
        assert!(
            !profile.config_source_paths.is_empty(),
            "profile {} missing config_source_paths",
            profile.id
        );
        for source in &profile.config_source_paths {
            assert!(
                root.join(source).exists(),
                "profile {} source path missing: {}",
                profile.id,
                source
            );
        }
    }
}
