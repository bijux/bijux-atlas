// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn policy_document_matches_declared_schema_shape() {
    let root = workspace_root();
    let schema_path = root.join("ops/inventory/policies/dev-atlas-policy.schema.json");
    let document_path = root.join("ops/inventory/policies/dev-atlas-policy.json");

    let schema: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(schema_path).expect("schema text"))
            .expect("schema json");
    let document: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(document_path).expect("document text"))
            .expect("document json");

    let required = schema
        .get("required")
        .and_then(serde_json::Value::as_array)
        .expect("schema required");
    for key in required {
        let key = key.as_str().expect("required key str");
        assert!(
            document.get(key).is_some(),
            "missing required key `{key}` in policy document"
        );
    }

    let properties = schema
        .get("properties")
        .and_then(serde_json::Value::as_object)
        .expect("schema properties");
    let doc_obj = document.as_object().expect("document object");
    for key in doc_obj.keys() {
        assert!(
            properties.contains_key(key),
            "unexpected key `{key}` not declared in policy schema"
        );
    }
}

#[test]
fn policy_loader_validates_schema_version_and_defaults() {
    let root = workspace_root();
    let loaded = bijux_dev_atlas_policies::DevAtlasPolicySet::load(&root).expect("load policy");
    assert!(!loaded.documented_defaults.is_empty());
}
