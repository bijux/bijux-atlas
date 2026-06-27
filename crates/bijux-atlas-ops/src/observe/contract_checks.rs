// SPDX-License-Identifier: Apache-2.0

use crate::reference::workspace_surfaces::{
    atlas_http_response_contract_source, atlas_server_binary_source,
};
use std::path::Path;

fn load_required_metric_names(repo_root: &Path) -> Result<Vec<String>, String> {
    let contract_path =
        repo_root.join("configs/schemas/contracts/observability/metrics.schema.json");
    let contract: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&contract_path)
            .map_err(|err| format!("failed to read {}: {err}", contract_path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", contract_path.display()))?;
    let mut rows = contract
        .get("required_metrics")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("name").and_then(serde_json::Value::as_str))
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    rows.sort();
    rows.dedup();
    Ok(rows)
}

pub fn observability_contract_checks(
    repo_root: &Path,
    metrics_body: &str,
) -> Result<serde_json::Value, String> {
    let required_metric_names = load_required_metric_names(repo_root)?;
    let required_metrics_present = required_metric_names
        .iter()
        .filter(|name| metrics_body.contains(&format!("{}{{", name)))
        .cloned()
        .collect::<Vec<_>>();
    let missing_metrics = required_metric_names
        .iter()
        .filter(|name| !metrics_body.contains(&format!("{}{{", name)))
        .cloned()
        .collect::<Vec<_>>();
    let warmup_lock_metrics_present = [
        "bijux_warmup_lock_contention_total{",
        "bijux_warmup_lock_expired_total{",
        "bijux_warmup_lock_wait_p95_seconds{",
    ]
    .iter()
    .all(|needle| metrics_body.contains(needle));

    let response_contract = std::fs::read_to_string(atlas_http_response_contract_source(repo_root))
        .map_err(|err| format!("failed to read response contract source: {err}"))?;
    let error_registry = std::fs::read_to_string(
        repo_root.join("configs/sources/operations/observability/error-codes.json"),
    )
    .map_err(|err| format!("failed to read error registry: {err}"))?;
    let openapi = std::fs::read_to_string(
        repo_root.join("configs/generated/openapi/v1/openapi.snapshot.json"),
    )
    .map_err(|err| format!("failed to read openapi: {err}"))?;
    let error_registry_aligned = error_registry.contains("NotReady")
        && error_registry.contains("RateLimited")
        && response_contract.contains("ApiErrorCode::NotReady")
        && response_contract.contains("ApiErrorCode::RateLimited")
        && openapi.contains("\"ApiErrorCode\"");

    let main_rs = std::fs::read_to_string(atlas_server_binary_source(repo_root))
        .map_err(|err| format!("failed to read main.rs: {err}"))?;
    let startup_log_fields_present = main_rs.contains("event_id = \"startup\"")
        && main_rs.contains("release_id = %release_id")
        && main_rs.contains("governance_version = %governance_version");

    let redacted = crate::lifecycle::evidence_artifacts::redact_sensitive_text(
        "TOKEN=secret-value\nAuthorization: Bearer abc123\n",
    );
    let redaction_contract_passed =
        !crate::lifecycle::evidence_artifacts::contains_common_secret_pattern(&redacted)
            && !redacted.contains("secret-value")
            && !redacted.contains("abc123");

    let dashboard_schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/schema/observe/dashboard.schema.json"))
            .map_err(|err| format!("failed to read dashboard schema: {err}"))?,
    )
    .map_err(|err| format!("failed to parse dashboard schema: {err}"))?;
    let dashboard_contract: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            repo_root.join("ops/observe/contracts/dashboard-panels-contract.json"),
        )
        .map_err(|err| format!("failed to read dashboard contract: {err}"))?,
    )
    .map_err(|err| format!("failed to parse dashboard contract: {err}"))?;
    let dashboard_text = std::fs::read_to_string(
        repo_root.join("ops/observe/dashboards/atlas-observability-dashboard.json"),
    )
    .map_err(|err| format!("failed to read dashboard: {err}"))?;
    let dashboard: serde_json::Value = serde_json::from_str(&dashboard_text)
        .map_err(|err| format!("failed to parse dashboard: {err}"))?;
    let panel_titles = dashboard
        .get("panels")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("title").and_then(serde_json::Value::as_str))
        .collect::<std::collections::BTreeSet<_>>();
    let required_panels = dashboard_contract
        .get("required_panels")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    let required_rows = dashboard_contract
        .get("required_rows")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    let dashboard_contract_valid = dashboard_schema.get("type")
        == Some(&serde_json::Value::String("object".to_string()))
        && dashboard
            .get("uid")
            .and_then(serde_json::Value::as_str)
            .is_some()
        && dashboard
            .get("schemaVersion")
            .and_then(serde_json::Value::as_i64)
            .is_some()
        && !required_panels
            .iter()
            .any(|name| !panel_titles.contains(name))
        && !required_rows
            .iter()
            .any(|name| !panel_titles.contains(name));

    let slo_path = repo_root.join("ops/observe/slo-definitions.json");
    let slo_schema_path = repo_root.join("ops/schema/observe/slo-definitions.schema.json");
    let slo: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&slo_path)
            .map_err(|err| format!("failed to read {}: {err}", slo_path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", slo_path.display()))?;
    let slo_schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&slo_schema_path)
            .map_err(|err| format!("failed to read {}: {err}", slo_schema_path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", slo_schema_path.display()))?;
    let slo_contract_valid = slo["schema_version"]
        == slo_schema["properties"]["schema_version"]["const"]
        && slo
            .get("slos")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|rows| !rows.is_empty());

    let alert_schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/schema/observe/prometheus-rule.schema.json"))
            .map_err(|err| format!("failed to read alert schema: {err}"))?,
    )
    .map_err(|err| format!("failed to parse alert schema: {err}"))?;
    let alert_contract: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/observe/contracts/alerts-contract.json"))
            .map_err(|err| format!("failed to read alert contract: {err}"))?,
    )
    .map_err(|err| format!("failed to parse alert contract: {err}"))?;
    let alerts_path = repo_root.join("ops/observe/alerts/atlas-alert-rules.yaml");
    let alert_rules: serde_yaml::Value = serde_yaml::from_str(
        &std::fs::read_to_string(&alerts_path)
            .map_err(|err| format!("failed to read {}: {err}", alerts_path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", alerts_path.display()))?;
    let groups = alert_rules
        .get("spec")
        .and_then(|row| row.get("groups"))
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let mut label_policy_passed = true;
    let mut alert_rules_reference_known_metrics = true;
    let label_policy: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            repo_root.join("configs/sources/operations/observability/label-policy.json"),
        )
        .map_err(|err| format!("failed to read label policy: {err}"))?,
    )
    .map_err(|err| format!("failed to parse label policy: {err}"))?;
    let alert_required_labels = label_policy["alerts_required_labels"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    let metric_required_labels = label_policy["metrics_required_labels"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    if !metric_required_labels
        .iter()
        .all(|label| metrics_body.contains(&format!("{label}=\"")))
    {
        label_policy_passed = false;
    }
    let required_alerts = alert_contract
        .get("required_alerts")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    let mut observed_alerts = std::collections::BTreeSet::new();
    for group in &groups {
        let rules = group
            .get("rules")
            .and_then(serde_yaml::Value::as_sequence)
            .cloned()
            .unwrap_or_default();
        for rule in rules {
            if let Some(alert_name) = rule.get("alert").and_then(serde_yaml::Value::as_str) {
                observed_alerts.insert(alert_name.to_string());
            }
            let labels = rule
                .get("labels")
                .and_then(serde_yaml::Value::as_mapping)
                .cloned()
                .unwrap_or_default();
            for required in &alert_required_labels {
                let key = serde_yaml::Value::String((*required).to_string());
                if !labels.contains_key(&key) {
                    label_policy_passed = false;
                }
            }
            if let Some(expr) = rule.get("expr").and_then(serde_yaml::Value::as_str) {
                let known_metric = required_metric_names.iter().any(|name| expr.contains(name))
                    || [
                        "atlas_overload_active",
                        "bijux_store_download_failure_total",
                        "bijux_dataset_hits",
                        "bijux_dataset_misses",
                        "atlas_ingest_failure_total",
                        "atlas_shard_integrity_violation_total",
                        "atlas_disk_free_bytes",
                        "kube_pod_container_status_restarts_total",
                        "atlas_loaded_datasets",
                    ]
                    .iter()
                    .any(|name| expr.contains(name));
                if !known_metric {
                    alert_rules_reference_known_metrics = false;
                }
            }
        }
    }
    let alert_rules_contract_valid = alert_schema
        .get("properties")
        .and_then(|row| row.get("kind"))
        .and_then(|row| row.get("const"))
        == Some(&serde_json::Value::String("PrometheusRule".to_string()))
        && alert_rules.get("kind").and_then(serde_yaml::Value::as_str) == Some("PrometheusRule")
        && !groups.is_empty();
    if required_alerts
        .iter()
        .any(|name| !observed_alerts.contains(*name))
    {
        return Ok(serde_json::json!({
            "required_metrics_present": required_metrics_present,
            "missing_metrics": missing_metrics,
            "warmup_lock_metrics_present": warmup_lock_metrics_present,
            "error_registry_aligned": error_registry_aligned,
            "startup_log_fields_present": startup_log_fields_present,
            "redaction_contract_passed": redaction_contract_passed,
            "dashboard_contract_valid": dashboard_contract_valid,
            "slo_contract_valid": slo_contract_valid,
            "alert_rules_contract_valid": false,
            "alert_rules_reference_known_metrics": alert_rules_reference_known_metrics,
            "label_policy_passed": label_policy_passed
        }));
    }

    Ok(serde_json::json!({
        "required_metrics_present": required_metrics_present,
        "missing_metrics": missing_metrics,
        "warmup_lock_metrics_present": warmup_lock_metrics_present,
        "error_registry_aligned": error_registry_aligned,
        "startup_log_fields_present": startup_log_fields_present,
        "redaction_contract_passed": redaction_contract_passed,
        "dashboard_contract_valid": dashboard_contract_valid,
        "slo_contract_valid": slo_contract_valid,
        "alert_rules_contract_valid": alert_rules_contract_valid,
        "alert_rules_reference_known_metrics": alert_rules_reference_known_metrics,
        "label_policy_passed": label_policy_passed
    }))
}

#[cfg(test)]
mod tests {
    use super::observability_contract_checks;
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .expect("workspace root")
            .to_path_buf()
    }

    #[test]
    fn observability_contract_checks_use_current_workspace_sources() {
        let value = observability_contract_checks(&repo_root(), "").expect("observability checks");
        assert!(value.get("error_registry_aligned").is_some());
    }
}
