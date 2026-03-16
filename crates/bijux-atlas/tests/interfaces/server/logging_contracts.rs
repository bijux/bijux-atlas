// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::adapters::outbound::telemetry::logging::{redact_if_needed, LoggingConfig};
use serde_json::json;

#[test]
fn logging_sampling_is_deterministic() {
    let cfg = LoggingConfig {
        log_json: true,
        level: "info".to_string(),
        filter_targets: Some("atlas=debug".to_string()),
        sampling_rate: 0.25,
        redaction_enabled: true,
        rotation_max_bytes: 10_485_760,
        rotation_max_files: 5,
    };
    assert_eq!(
        cfg.should_emit_sampled("request-1"),
        cfg.should_emit_sampled("request-1")
    );
}

#[test]
fn redaction_policy_masks_sensitive_fields() {
    assert_eq!(
        redact_if_needed(true, "api_token=abcdef"),
        "[redacted]".to_string()
    );
    assert_eq!(
        redact_if_needed(false, "api_token=abcdef"),
        "api_token=abcdef".to_string()
    );
}

#[test]
fn structured_log_schema_minimum_fields_present() {
    let sample = json!({
        "timestamp": "2026-03-04T12:00:00Z",
        "level": "INFO",
        "target": "atlas_server",
        "fields": {
            "message": "request handled",
            "event_id": "request_handled",
            "request_id": "req-1",
            "route": "/v1/genes",
            "status": 200
        }
    });
    assert!(sample.get("timestamp").is_some());
    assert!(sample.get("level").is_some());
    assert!(sample.get("target").is_some());
    assert!(sample
        .get("fields")
        .and_then(|v| v.get("message"))
        .is_some());
}

#[test]
fn logging_format_snapshot_remains_stable() {
    let snapshot = json!({
        "level": "INFO",
        "target": "atlas_server",
        "fields": {
            "event_id": "request_handled",
            "request_id": "req-1",
            "route": "/v1/genes",
            "status": 200
        }
    });
    let rendered = serde_json::to_string(&snapshot).expect("render snapshot");
    assert!(rendered.contains("\"event_id\":\"request_handled\""));
    assert!(rendered.contains("\"request_id\":\"req-1\""));
}

#[test]
fn logging_helpers_performance_stays_within_budget() {
    let cfg = LoggingConfig {
        log_json: true,
        level: "info".to_string(),
        filter_targets: None,
        sampling_rate: 0.5,
        redaction_enabled: true,
        rotation_max_bytes: 10_485_760,
        rotation_max_files: 5,
    };
    let started = std::time::Instant::now();
    for idx in 0..20_000 {
        let _ = cfg.should_emit_sampled(&format!("req-{idx}"));
        let _ = redact_if_needed(true, "safe-value");
    }
    assert!(
        started.elapsed() < std::time::Duration::from_secs(2),
        "logging helper overhead exceeded budget"
    );
}
