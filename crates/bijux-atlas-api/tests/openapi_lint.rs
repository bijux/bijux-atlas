use bijux_atlas_api::openapi::{openapi_v1_spec, OPENAPI_V1_PINNED_SHA256};
use bijux_atlas_core::canonical;
use serde_json::Value;

#[test]
fn openapi_hash_matches_pinned_contract() {
    let hash = canonical::stable_json_hash_hex(&openapi_v1_spec()).expect("hash openapi");
    assert_eq!(hash, OPENAPI_V1_PINNED_SHA256);
}

#[test]
fn openapi_paths_and_component_schemas_are_lexicographically_sorted() {
    let spec = openapi_v1_spec();
    assert_sorted_object(spec.get("paths").expect("paths"));
    let schemas = spec
        .get("components")
        .and_then(|v| v.get("schemas"))
        .expect("components.schemas");
    assert_sorted_object(schemas);
}

#[test]
fn openapi_schema_lint_rules_hold() {
    let spec = openapi_v1_spec();
    assert_eq!(spec["openapi"], "3.0.3");
    assert_eq!(spec["info"]["version"], "v1");

    let api_error = &spec["components"]["schemas"]["ApiError"];
    assert_eq!(api_error["type"], "object");
    assert_eq!(api_error["additionalProperties"], Value::Bool(false));

    let required = api_error["required"]
        .as_array()
        .expect("ApiError.required array")
        .iter()
        .map(|v| v.as_str().expect("required string"))
        .collect::<Vec<_>>();
    assert_eq!(required, vec!["code", "message", "details", "request_id"]);
}

fn assert_sorted_object(value: &Value) {
    let object = value.as_object().expect("json object");
    let mut observed = object.keys().map(String::as_str).collect::<Vec<_>>();
    let mut sorted = observed.clone();
    sorted.sort_unstable();
    assert_eq!(observed, sorted);
    observed.clear();
}
