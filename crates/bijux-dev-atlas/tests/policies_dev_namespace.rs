// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

use bijux_dev_atlas::policies::dev;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn dev_namespace_loads_workspace_policy_from_ssot_paths() {
    let root = workspace_root();
    let set = dev::DevAtlasPolicySet::load(&root).expect("load dev policy");
    assert_eq!(set.schema_version, dev::PolicySchemaVersion::V1);

    let config_path = dev::policy_config_path(&root);
    let schema_path = dev::policy_schema_path(&root);
    assert!(config_path.ends_with("ops/inventory/policies/dev-atlas-policy.json"));
    assert!(schema_path.ends_with("ops/inventory/policies/dev-atlas-policy.schema.json"));
    assert!(config_path.exists(), "missing {}", config_path.display());
    assert!(schema_path.exists(), "missing {}", schema_path.display());
}

#[test]
fn dev_policy_schema_version_matches_schema_document_const() {
    let root = workspace_root();
    let config: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(dev::policy_config_path(&root)).expect("config text"),
    )
    .expect("config json");
    let schema: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(dev::policy_schema_path(&root)).expect("schema text"),
    )
    .expect("schema json");

    let cfg_version = config
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        .expect("config schema_version");
    let schema_const = schema
        .get("properties")
        .and_then(serde_json::Value::as_object)
        .and_then(|props| props.get("schema_version"))
        .and_then(serde_json::Value::as_object)
        .and_then(|field| field.get("const"))
        .and_then(serde_json::Value::as_str)
        .expect("schema schema_version const");

    assert_eq!(cfg_version, schema_const);
}

#[test]
fn dev_policy_registry_validation_rejects_unknown_ratchet_id() {
    let config = serde_json::json!({
        "documented_defaults": [],
        "ratchets": [{"id": "repo.not_a_real_policy", "reason": "test"}],
        "relaxations": []
    });
    let err = dev::validate_policy_registry_ids(&config).expect_err("must reject unknown ratchet");
    assert!(err.to_string().contains("ratchet references unknown policy id"));
}

