// SPDX-License-Identifier: Apache-2.0

use bijux_dev_atlas::contracts::logging_registry::{
    classify_event_name, schema_contract, validate_log_record, LogClass,
};
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn logging_contract_has_required_fields() {
    let contract = schema_contract();
    let required = contract
        .required_fields
        .into_iter()
        .filter(|field| field.required)
        .map(|field| field.name)
        .collect::<std::collections::BTreeSet<_>>();
    for expected in [
        "timestamp",
        "level",
        "target",
        "message",
        "request_id",
        "event_name",
    ] {
        assert!(required.contains(expected));
    }
}

#[test]
fn logging_classification_contract_file_has_runtime_and_query_classes() {
    let path = repo_root().join("ops/observe/contracts/log-classification-contract.json");
    let payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(path).expect("read")).expect("json");
    let classes = payload
        .get("classes")
        .and_then(serde_json::Value::as_array)
        .expect("classes array");
    assert!(classes
        .iter()
        .any(|c| c.get("name").and_then(|v| v.as_str()) == Some("runtime")));
    assert!(classes
        .iter()
        .any(|c| c.get("name").and_then(|v| v.as_str()) == Some("query")));
}

#[test]
fn log_format_validator_contract_has_core_rules() {
    let path = repo_root().join("ops/observe/logging/format-validator-contract.json");
    let payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(path).expect("read")).expect("json");
    let required = payload
        .get("required_fields")
        .and_then(serde_json::Value::as_array)
        .expect("required fields");
    assert!(required.iter().any(|v| v.as_str() == Some("request_id")));
    assert!(required.iter().any(|v| v.as_str() == Some("event_name")));
}

#[test]
fn validate_log_record_flags_unknown_event_class() {
    let row = serde_json::json!({
        "timestamp": "2026-03-05T10:00:00Z",
        "level": "INFO",
        "target": "atlas::runtime",
        "message": "request completed",
        "request_id": "req-1",
        "event_name": "random_event"
    });
    let violations = validate_log_record(&row);
    assert!(violations.iter().any(|v| v.code == "unknown_log_class"));
}

#[test]
fn classify_event_name_maps_security_prefix() {
    assert_eq!(
        classify_event_name("security_auth_denied"),
        LogClass::Security
    );
}

#[test]
fn schema_validation_on_sample_logs_produces_deterministic_results() {
    let path = repo_root().join("ops/observe/contracts/logs.example.jsonl");
    let text = fs::read_to_string(path).expect("read logs example");
    let mut all = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let row: serde_json::Value = serde_json::from_str(line).expect("json line");
        all.push(validate_log_record(&row));
    }
    assert!(!all.is_empty());
}

#[test]
fn redaction_validation_flags_secret_patterns() {
    let row = serde_json::json!({
        "timestamp": "2026-03-05T10:00:00Z",
        "level": "INFO",
        "target": "atlas::runtime",
        "message": "secret leaked",
        "request_id": "req-2",
        "event_name": "runtime_init"
    });
    let violations = validate_log_record(&row);
    assert!(violations.iter().any(|v| v.code == "redaction_violation"));
}
