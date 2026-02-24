use std::path::PathBuf;

use bijux_atlas_policies::{
    parse_policy_set_json, policy_config_path, policy_schema_path, PolicySchemaVersion,
};

#[test]
fn all_policy_documents_parse_and_validate_against_schema_contract() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf();

    let schema_raw = std::fs::read_to_string(policy_schema_path(&root)).expect("schema read");
    let policy_dir = policy_config_path(&root)
        .parent()
        .expect("policy directory")
        .to_path_buf();

    for entry in std::fs::read_dir(policy_dir).expect("read policy dir") {
        let entry = entry.expect("entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if path.file_name().and_then(|n| n.to_str()) == Some("policy.schema.json") {
            continue;
        }
        let config_raw = std::fs::read_to_string(&path).expect("config read");
        let json: serde_json::Value = serde_json::from_str(&config_raw)
            .unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        let is_policy_set = json.get("schema_version").is_some() && json.get("mode").is_some();
        if !is_policy_set {
            continue;
        }
        let parsed = parse_policy_set_json(&config_raw, &schema_raw)
            .unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        assert_eq!(parsed.schema_version, PolicySchemaVersion::V1);
    }
}
