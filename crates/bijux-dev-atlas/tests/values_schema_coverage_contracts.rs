use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Map, Value};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates dir")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn schema_path(root: &Path) -> PathBuf {
    root.join("ops/k8s/charts/bijux-atlas/values.schema.json")
}

fn load_json(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).expect("read json")).expect("parse json")
}

fn descend(
    prefix: &str,
    node: &Value,
    missing_types: &mut Vec<String>,
    description_only: &mut Vec<String>,
) {
    let Some(object) = node.as_object() else {
        return;
    };
    let has_type = object.contains_key("type");
    let has_ref = object.contains_key("$ref");
    let has_all_of = object.contains_key("allOf");
    let has_description = object.contains_key("description");
    if !prefix.is_empty() && !has_type && !has_ref && !has_all_of {
        missing_types.push(prefix.to_string());
        if has_description {
            description_only.push(prefix.to_string());
        }
    }
    if let Some(properties) = object.get("properties").and_then(Value::as_object) {
        for (key, value) in properties {
            let child = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{prefix}.{key}")
            };
            descend(&child, value, missing_types, description_only);
        }
    }
}

fn generate_report(root: &Path) -> Value {
    let schema = load_json(&schema_path(root));
    let properties = schema["properties"]
        .as_object()
        .expect("top-level properties");
    let mut missing_types = Vec::new();
    let mut description_only = Vec::new();
    for (key, value) in properties {
        descend(key, value, &mut missing_types, &mut description_only);
    }
    missing_types.sort();
    description_only.sort();
    json!({
        "generated_by": "cargo test -p bijux-dev-atlas values_schema_coverage_contracts -- --nocapture",
        "schema_path": "ops/k8s/charts/bijux-atlas/values.schema.json",
        "top_level_keys": properties.keys().cloned().collect::<Vec<_>>(),
        "missing_type_nodes": missing_types,
        "description_only_nodes": description_only
    })
}

#[test]
fn values_schema_generates_coverage_report_without_missing_type_nodes() {
    let root = repo_root();
    let report = generate_report(&root);
    let artifact_dir = root.join("artifacts/ops");
    fs::create_dir_all(&artifact_dir).expect("create artifacts/ops");
    let artifact_path = artifact_dir.join("values-schema-coverage.json");
    fs::write(
        &artifact_path,
        serde_json::to_string_pretty(&report).expect("encode report"),
    )
    .expect("write coverage report");
    let missing = report["missing_type_nodes"]
        .as_array()
        .expect("missing_type_nodes");
    assert!(
        missing.is_empty(),
        "values schema nodes missing explicit type/$ref/allOf: {}",
        serde_json::to_string_pretty(missing).expect("encode missing")
    );
}

#[test]
fn high_risk_values_keys_are_not_description_only() {
    let root = repo_root();
    let policy_path = root.join("ops/k8s/values-schema-high-risk-policy.json");
    let policy = load_json(&policy_path);
    let high_risk = policy["high_risk_keys"].as_array().expect("high_risk_keys");
    let schema = load_json(&schema_path(&root));
    let properties = schema["properties"]
        .as_object()
        .expect("top-level properties");
    let mut offenders = Map::new();
    for key in high_risk {
        let key = key.as_str().expect("key");
        let node = properties
            .get(key)
            .unwrap_or_else(|| panic!("missing schema key {key}"));
        let object = node.as_object().expect("schema object");
        let typed = object.contains_key("type")
            || object.contains_key("$ref")
            || object.contains_key("allOf");
        let description_only = object.contains_key("description") && !typed;
        if description_only {
            offenders.insert(key.to_string(), node.clone());
        }
    }
    assert!(
        offenders.is_empty(),
        "high-risk keys may not be description-only: {}",
        serde_json::to_string_pretty(&Value::Object(offenders)).expect("encode offenders")
    );
}
