// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;
use std::process::Command;

use bijux_dev_atlas::policies::{
    canonical_policy_json, validate_policy_change_requires_version_bump, DevAtlasPolicySet,
    PolicySchemaVersion,
};

fn workspace_root() -> PathBuf {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--locked")
        .arg("--format-version")
        .arg("1")
        .arg("--no-deps")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("failed to run cargo metadata for workspace root");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid cargo metadata JSON");
    PathBuf::from(
        value
            .get("workspace_root")
            .and_then(serde_json::Value::as_str)
            .expect("workspace_root missing from metadata"),
    )
}

#[test]
fn workspace_policy_loads_from_ssot_paths() {
    let root = workspace_root();
    let policy = DevAtlasPolicySet::load(&root).expect("load dev policy");
    assert_eq!(policy.schema_version, PolicySchemaVersion::V1);
}

#[test]
fn deterministic_resolution_of_policy_set() {
    let root = workspace_root();
    let a = DevAtlasPolicySet::load(&root).expect("load a");
    let b = DevAtlasPolicySet::load(&root).expect("load b");
    assert_eq!(a, b);
}

#[test]
fn stable_policy_json_matches_golden() {
    let root = workspace_root();
    let policy = DevAtlasPolicySet::load(&root).expect("load dev policy");
    let json = canonical_policy_json(&policy.to_document()).expect("canonical");
    let golden = include_str!("policies_goldens/dev_atlas_policy_resolved.json");
    assert_eq!(json.trim(), golden.trim());
}

#[test]
fn policy_changes_require_schema_bump() {
    let root = workspace_root();
    let old = DevAtlasPolicySet::load(&root).expect("load old");
    let mut changed = old.to_document();
    changed.ops.registry_relpath = "ops/inventory/registry.v2.toml".to_string();
    let old_doc = old.to_document();
    assert!(validate_policy_change_requires_version_bump(&old_doc, &changed).is_err());
}

#[test]
fn policies_crate_dependency_minimalism() {
    let cargo = std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("read Cargo.toml");
    for forbidden in [
        "tokio",
        "axum",
        "hyper",
        "bijux-atlas",
        "bijux-atlas-query",
    ] {
        assert!(
            !cargo.contains(forbidden),
            "forbidden dependency in dev policies crate: {forbidden}"
        );
    }
}
