// SPDX-License-Identifier: Apache-2.0

use super::chrono_like_unix_secs;
use tracing::info;

pub(super) fn redacted_audit_field(key: &str, value: &str) -> Option<String> {
    let normalized_key = key.to_ascii_lowercase();
    if [
        "authorization",
        "token",
        "api_key",
        "api-key",
        "signature",
        "secret",
        "email",
        "client_ip",
    ]
    .iter()
    .any(|needle| normalized_key.contains(needle))
    {
        return None;
    }
    let normalized_value = value.to_ascii_lowercase();
    if normalized_value.contains("bearer ")
        || normalized_value.contains("x-api-key")
        || normalized_value.contains('@')
    {
        return Some("[REDACTED]".to_string());
    }
    Some(value.to_string())
}

fn audit_dynamic_field_allowed(key: &str) -> bool {
    matches!(
        key,
        "status"
            | "decision"
            | "reason"
            | "route"
            | "source"
            | "outcome"
            | "auth_mode"
            | "admin_endpoints_enabled"
            | "audit_enabled"
            | "catalog_configured"
    )
}

pub(super) fn build_audit_event(
    event_name: &str,
    principal: Option<&str>,
    action: &str,
    resource_kind: &str,
    resource_id: &str,
    sink: bijux_atlas_runtime::runtime::config::AuditSink,
    fields: &[(&str, &str)],
) -> serde_json::Value {
    let mut object = serde_json::Map::new();
    object.insert(
        "event_id".to_string(),
        serde_json::Value::String(format!("audit_{event_name}")),
    );
    object.insert(
        "event_name".to_string(),
        serde_json::Value::String(event_name.to_string()),
    );
    object.insert(
        "timestamp_policy".to_string(),
        serde_json::Value::String("runtime-unix-seconds".to_string()),
    );
    object.insert(
        "timestamp_unix_s".to_string(),
        serde_json::Value::Number(serde_json::Number::from(chrono_like_unix_secs())),
    );
    object.insert(
        "sink".to_string(),
        serde_json::Value::String(sink.as_str().to_string()),
    );
    if let Some(value) = principal {
        if let Some(redacted) = redacted_audit_field("principal", value) {
            object.insert("principal".to_string(), serde_json::Value::String(redacted));
        }
    }
    object.insert(
        "action".to_string(),
        serde_json::Value::String(action.to_string()),
    );
    object.insert(
        "resource_kind".to_string(),
        serde_json::Value::String(resource_kind.to_string()),
    );
    if let Some(redacted) = redacted_audit_field("resource_id", resource_id) {
        object.insert(
            "resource_id".to_string(),
            serde_json::Value::String(redacted),
        );
    }
    for (key, value) in fields {
        if !audit_dynamic_field_allowed(key) {
            continue;
        }
        if let Some(redacted) = redacted_audit_field(key, value) {
            object.insert((*key).to_string(), serde_json::Value::String(redacted));
        }
    }
    serde_json::Value::Object(object)
}

pub(super) fn emit_audit_event(
    audit: &bijux_atlas_runtime::runtime::config::AuditConfig,
    event_name: &str,
    principal: Option<&str>,
    action: &str,
    resource_kind: &str,
    resource_id: &str,
    fields: &[(&str, &str)],
) {
    let payload = build_audit_event(
        event_name,
        principal,
        action,
        resource_kind,
        resource_id,
        audit.sink,
        fields,
    );
    if matches!(
        audit.sink,
        bijux_atlas_runtime::runtime::config::AuditSink::File
    ) {
        let _ = bijux_atlas_runtime::adapters::outbound::fs::write_audit_file_record(
            &audit.file_path,
            audit.max_bytes,
            &payload,
        );
    }
    info!(
        target: "atlas_audit",
        event_id = format!("audit_{event_name}"),
        audit_payload = %payload,
        "audit event"
    );
}
