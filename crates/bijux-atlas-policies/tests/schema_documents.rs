use std::path::PathBuf;

use bijux_atlas_policies::{parse_policy_set_json, policy_config_path, policy_schema_path, PolicySchemaVersion};

#[test]
fn all_policy_documents_parse_and_validate_against_schema_contract() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf();

    let config_raw = std::fs::read_to_string(policy_config_path(&root)).expect("config read");
    let schema_raw = std::fs::read_to_string(policy_schema_path(&root)).expect("schema read");

    let parsed = parse_policy_set_json(&config_raw, &schema_raw).expect("parse + validate");
    assert_eq!(parsed.schema_version, PolicySchemaVersion::V1);
}
