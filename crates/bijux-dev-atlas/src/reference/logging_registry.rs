// SPDX-License-Identifier: Apache-2.0
//! Canonical structured logging reference, classification rules, and format validation.

use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogSeverity {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogClass {
    Runtime,
    Query,
    Ingest,
    Artifact,
    Configuration,
    Startup,
    Shutdown,
    Security,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogFieldSpec {
    pub name: &'static str,
    pub required: bool,
    pub pii_allowed: bool,
    pub description: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogSchemaContract {
    pub schema_version: u32,
    pub message_convention: &'static str,
    pub redaction_policy: &'static str,
    pub rotation_policy: &'static str,
    pub sampling_policy: &'static str,
    pub required_fields: Vec<LogFieldSpec>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogValidationViolation {
    pub code: &'static str,
    pub message: String,
}

pub fn schema_contract() -> LogSchemaContract {
    LogSchemaContract {
        schema_version: 1,
        message_convention:
            "messages are imperative, event-focused, and include stable context keys",
        redaction_policy: "secret-like values must be redacted before emitting structured records",
        rotation_policy: "rotation max bytes and max files must both be positive",
        sampling_policy: "sampling is deterministic and keyed by stable request identifiers",
        required_fields: vec![
            LogFieldSpec {
                name: "timestamp",
                required: true,
                pii_allowed: false,
                description: "RFC3339 timestamp for log event time",
            },
            LogFieldSpec {
                name: "level",
                required: true,
                pii_allowed: false,
                description: "severity level",
            },
            LogFieldSpec {
                name: "target",
                required: true,
                pii_allowed: false,
                description: "logger target or subsystem",
            },
            LogFieldSpec {
                name: "message",
                required: true,
                pii_allowed: false,
                description: "human-readable event summary",
            },
            LogFieldSpec {
                name: "request_id",
                required: true,
                pii_allowed: false,
                description: "request correlation identifier",
            },
            LogFieldSpec {
                name: "trace_id",
                required: false,
                pii_allowed: false,
                description: "trace correlation identifier",
            },
            LogFieldSpec {
                name: "event_name",
                required: true,
                pii_allowed: false,
                description: "stable event key",
            },
        ],
    }
}

pub fn classify_event_name(event_name: &str) -> LogClass {
    if event_name.starts_with("runtime_") {
        LogClass::Runtime
    } else if event_name.starts_with("query_") {
        LogClass::Query
    } else if event_name.starts_with("ingest_") {
        LogClass::Ingest
    } else if event_name.starts_with("artifact_") {
        LogClass::Artifact
    } else if event_name.starts_with("config_") {
        LogClass::Configuration
    } else if event_name.starts_with("startup_") {
        LogClass::Startup
    } else if event_name.starts_with("shutdown_") {
        LogClass::Shutdown
    } else if event_name.starts_with("security_") {
        LogClass::Security
    } else {
        LogClass::Unknown
    }
}

pub fn validate_log_record(record: &serde_json::Value) -> Vec<LogValidationViolation> {
    let contract = schema_contract();
    let mut violations = Vec::<LogValidationViolation>::new();

    let required = contract
        .required_fields
        .iter()
        .filter(|field| field.required)
        .map(|field| field.name)
        .collect::<BTreeSet<_>>();

    for field in required {
        if record.get(field).is_none() {
            violations.push(LogValidationViolation {
                code: "missing_required_field",
                message: format!("missing required field `{field}`"),
            });
        }
    }

    if let Some(level) = record.get("level").and_then(serde_json::Value::as_str) {
        if ![
            "trace", "debug", "info", "warn", "error", "TRACE", "DEBUG", "INFO", "WARN", "ERROR",
        ]
        .contains(&level)
        {
            violations.push(LogValidationViolation {
                code: "invalid_level",
                message: format!("invalid log level `{level}`"),
            });
        }
    }

    if let Some(message) = record.get("message").and_then(serde_json::Value::as_str) {
        let lowered = message.to_ascii_lowercase();
        for banned in ["password", "secret", "token"] {
            if lowered.contains(banned) {
                violations.push(LogValidationViolation {
                    code: "redaction_violation",
                    message: format!("message contains sensitive pattern `{banned}`"),
                });
            }
        }
    }

    if let Some(event_name) = record.get("event_name").and_then(serde_json::Value::as_str) {
        if matches!(classify_event_name(event_name), LogClass::Unknown) {
            violations.push(LogValidationViolation {
                code: "unknown_log_class",
                message: format!("event `{event_name}` does not map to a known logging class"),
            });
        }
    }

    violations.sort_by(|a, b| a.message.cmp(&b.message));
    violations
}

pub fn summarize_classes(records: &[serde_json::Value]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::<String, usize>::new();
    for record in records {
        if let Some(event_name) = record.get("event_name").and_then(serde_json::Value::as_str) {
            let class = classify_event_name(event_name);
            let key = serde_json::to_value(class)
                .ok()
                .and_then(|value| value.as_str().map(str::to_string))
                .unwrap_or_else(|| "unknown".to_string());
            *counts.entry(key).or_insert(0) += 1;
        }
    }
    counts
}
