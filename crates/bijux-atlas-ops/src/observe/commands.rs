// SPDX-License-Identifier: Apache-2.0

use crate::kubernetes::service_probe::probe_kubectl_service_http_path;
use crate::lifecycle::simulation::paths::write_simulation_report;
use crate::observe::contract_checks::observability_contract_checks;
use crate::observe::report_artifacts::{
    observe_report_root, write_observe_contract_report, write_operational_readiness_markdown,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::Path;

fn sha256_text(body: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(body.as_bytes());
    hex::encode(hasher.finalize())
}

fn read_json(repo_root: &Path, relative_path: &str) -> Result<serde_json::Value, String> {
    let path = repo_root.join(relative_path);
    serde_json::from_str(
        &std::fs::read_to_string(&path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn read_yaml(repo_root: &Path, relative_path: &str) -> Result<serde_yaml::Value, String> {
    let path = repo_root.join(relative_path);
    serde_yaml::from_str(
        &std::fs::read_to_string(&path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn read_observe_report(base: &Path, file_name: &str) -> serde_json::Value {
    let path = base.join(file_name);
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .unwrap_or_else(|| {
            serde_json::json!({
                "status":"missing",
                "errors":[format!("missing report {}", path.display())]
            })
        })
}

pub fn render_slo_list_payload(repo_root: &Path) -> Result<serde_json::Value, String> {
    let slo = read_json(repo_root, "ops/observe/slo-definitions.json")?;
    let rows = slo
        .get("slos")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    Ok(serde_json::json!({
        "schema_version": 1,
        "text": "observe slo list",
        "rows": rows,
        "summary": {"total": rows.len(), "errors": 0, "warnings": 0}
    }))
}

pub fn verify_slo_contract(
    repo_root: &Path,
    run_id: &str,
) -> Result<(serde_json::Value, i32), String> {
    let slo = read_json(repo_root, "ops/observe/slo-definitions.json")?;
    let measurement = read_json(repo_root, "ops/observe/slo-measurement.json")?;
    let metric_map = read_json(repo_root, "ops/observe/slo-metric-map.json")?;
    let mut errors = Vec::new();
    let slos = slo
        .get("slos")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let measurement_obj = measurement
        .get("measurement_method")
        .and_then(serde_json::Value::as_object)
        .cloned()
        .unwrap_or_default();
    let map_rows = metric_map
        .get("slo_metric_map")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    for slo_row in &slos {
        let Some(id) = slo_row.get("id").and_then(serde_json::Value::as_str) else {
            errors.push("slo missing id".to_string());
            continue;
        };
        if !measurement_obj.contains_key(id) {
            errors.push(format!("measurement method missing for slo `{id}`"));
        }
        let map_exists = map_rows.iter().any(|row| {
            row.get("slo_id")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| value == id)
        });
        if !map_exists {
            errors.push(format!("metric map missing for slo `{id}`"));
        }
    }
    let report = serde_json::json!({
        "schema_version": 1,
        "status": if errors.is_empty() { "ok" } else { "failed" },
        "slos_total": slos.len(),
        "errors": errors,
    });
    let report_rel =
        write_observe_contract_report(repo_root, run_id, "slo-contract-report.json", &report)?;
    Ok((
        serde_json::json!({
            "schema_version": 1,
            "status": report["status"].clone(),
            "text": "observe slo verify",
            "rows": [{"report_path": report_rel, "errors": report["errors"].clone()}],
            "summary": {"total": 1, "errors": report["errors"].as_array().map(|v| v.len()).unwrap_or(0), "warnings": 0}
        }),
        if errors.is_empty() { 0 } else { 1 },
    ))
}

pub fn verify_alert_contract(
    repo_root: &Path,
    run_id: &str,
) -> Result<(serde_json::Value, i32), String> {
    let contract = read_json(repo_root, "ops/observe/contracts/alerts-contract.json")?;
    let required = contract
        .get("required_alerts")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| row.as_str().map(ToString::to_string))
        .collect::<BTreeSet<_>>();
    let mut observed = BTreeSet::new();
    let mut errors = Vec::new();
    for alerts_file in [
        "ops/observe/alerts/atlas-alert-rules.yaml",
        "ops/observe/alerts/slo-burn-rules.yaml",
    ] {
        let alert_rules = read_yaml(repo_root, alerts_file)?;
        let groups = alert_rules
            .get("spec")
            .and_then(|row| row.get("groups"))
            .and_then(serde_yaml::Value::as_sequence)
            .cloned()
            .unwrap_or_default();
        for group in &groups {
            let rules = group
                .get("rules")
                .and_then(serde_yaml::Value::as_sequence)
                .cloned()
                .unwrap_or_default();
            for rule in rules {
                if let Some(name) = rule.get("alert").and_then(serde_yaml::Value::as_str) {
                    observed.insert(name.to_string());
                }
                let labels = rule
                    .get("labels")
                    .and_then(serde_yaml::Value::as_mapping)
                    .cloned()
                    .unwrap_or_default();
                for required_label in ["severity", "subsystem", "alert_contract_version"] {
                    let key = serde_yaml::Value::String(required_label.to_string());
                    if !labels.contains_key(&key) {
                        errors.push(format!(
                            "alert missing label `{required_label}` in {alerts_file}"
                        ));
                    }
                }
                let runbook = rule
                    .get("annotations")
                    .and_then(|row| row.get("runbook"))
                    .and_then(serde_yaml::Value::as_str)
                    .unwrap_or_default();
                if runbook.is_empty() {
                    errors.push(format!(
                        "alert missing annotations.runbook in {alerts_file}"
                    ));
                }
            }
        }
    }
    for alert in required {
        if !observed.contains(&alert) {
            errors.push(format!(
                "required alert missing from alert rules: `{alert}`"
            ));
        }
    }
    let report = serde_json::json!({
        "schema_version": 1,
        "status": if errors.is_empty() { "ok" } else { "failed" },
        "alerts_total": observed.len(),
        "errors": errors
    });
    let report_rel =
        write_observe_contract_report(repo_root, run_id, "alerts-contract-report.json", &report)?;
    Ok((
        serde_json::json!({
            "schema_version": 1,
            "status": report["status"].clone(),
            "text": "observe alerts verify",
            "rows": [{"report_path": report_rel, "errors": report["errors"].clone()}],
            "summary": {"total": 1, "errors": report["errors"].as_array().map(|v| v.len()).unwrap_or(0), "warnings": 0}
        }),
        if report["errors"].as_array().is_some_and(|v| v.is_empty()) {
            0
        } else {
            1
        },
    ))
}

pub fn verify_runbook_contract(
    repo_root: &Path,
    run_id: &str,
) -> Result<(serde_json::Value, i32), String> {
    let mut errors = Vec::new();
    let mut checked = 0usize;
    for alerts_file in [
        "ops/observe/alerts/atlas-alert-rules.yaml",
        "ops/observe/alerts/slo-burn-rules.yaml",
    ] {
        let alert_rules = read_yaml(repo_root, alerts_file)?;
        let groups = alert_rules
            .get("spec")
            .and_then(|row| row.get("groups"))
            .and_then(serde_yaml::Value::as_sequence)
            .cloned()
            .unwrap_or_default();
        for group in &groups {
            let rules = group
                .get("rules")
                .and_then(serde_yaml::Value::as_sequence)
                .cloned()
                .unwrap_or_default();
            for rule in rules {
                let runbook = rule
                    .get("annotations")
                    .and_then(|row| row.get("runbook"))
                    .and_then(serde_yaml::Value::as_str)
                    .unwrap_or_default();
                if runbook.is_empty() {
                    errors.push(format!("alert missing runbook path in {alerts_file}"));
                    continue;
                }
                let runbook_path = repo_root.join(runbook);
                checked += 1;
                if !runbook_path.exists() {
                    errors.push(format!("runbook file does not exist: {runbook}"));
                    continue;
                }
                let content = std::fs::read_to_string(&runbook_path)
                    .map_err(|err| format!("failed to read {}: {err}", runbook_path.display()))?;
                if !content.to_ascii_lowercase().contains("evidence") {
                    errors.push(format!(
                        "runbook does not describe required evidence bundle: {runbook}"
                    ));
                }
            }
        }
    }
    let report = serde_json::json!({
        "schema_version": 1,
        "status": if errors.is_empty() { "ok" } else { "failed" },
        "runbooks_checked": checked,
        "errors": errors
    });
    let report_rel =
        write_observe_contract_report(repo_root, run_id, "runbooks-contract-report.json", &report)?;
    Ok((
        serde_json::json!({
            "schema_version": 1,
            "status": report["status"].clone(),
            "text": "observe runbooks verify",
            "rows": [{"report_path": report_rel, "errors": report["errors"].clone()}],
            "summary": {"total": 1, "errors": report["errors"].as_array().map(|v| v.len()).unwrap_or(0), "warnings": 0}
        }),
        if report["errors"].as_array().is_some_and(|v| v.is_empty()) {
            0
        } else {
            1
        },
    ))
}

pub fn build_operational_readiness_payload(
    repo_root: &Path,
    run_id: &str,
) -> Result<(serde_json::Value, i32), String> {
    let base = observe_report_root(repo_root, run_id);
    let slo = read_observe_report(&base, "slo-contract-report.json");
    let alerts = read_observe_report(&base, "alerts-contract-report.json");
    let runbooks = read_observe_report(&base, "runbooks-contract-report.json");
    let checks = [slo.clone(), alerts.clone(), runbooks.clone()];
    let passed = checks
        .iter()
        .filter(|row| row.get("status").and_then(serde_json::Value::as_str) == Some("ok"))
        .count();
    let total = checks.len();
    let completeness = if total == 0 {
        0.0
    } else {
        passed as f64 / total as f64
    };
    let threshold = 1.0f64;
    let status = if completeness >= threshold {
        "ok"
    } else {
        "failed"
    };
    let report = serde_json::json!({
        "schema_version": 1,
        "status": status,
        "completeness": completeness,
        "threshold": threshold,
        "reports": {
            "slo": format!("artifacts/ops/{run_id}/observe/slo-contract-report.json"),
            "alerts": format!("artifacts/ops/{run_id}/observe/alerts-contract-report.json"),
            "runbooks": format!("artifacts/ops/{run_id}/observe/runbooks-contract-report.json")
        }
    });
    let report_rel = write_observe_contract_report(
        repo_root,
        run_id,
        "operational-readiness-report.json",
        &report,
    )?;
    let human_rel =
        write_operational_readiness_markdown(repo_root, run_id, status, completeness, threshold)?;
    Ok((
        serde_json::json!({
            "schema_version": 1,
            "status": status,
            "text": "observe readiness report",
            "rows": [{
                "report_path": report_rel,
                "human_report_path": human_rel,
                "completeness": completeness,
                "threshold": threshold,
                "slo": slo,
                "alerts": alerts,
                "runbooks": runbooks
            }],
            "summary": {"total": 1, "errors": if status == "ok" { 0 } else { 1 }, "warnings": 0}
        }),
        if status == "ok" { 0 } else { 1 },
    ))
}

pub fn verify_observability_runtime(
    repo_root: &Path,
    run_id: &str,
    profile: &str,
    namespace: &str,
) -> Result<(serde_json::Value, i32), String> {
    let metrics = probe_kubectl_service_http_path(repo_root, namespace, 18081, 8080, "/metrics")?;
    let checks = observability_contract_checks(repo_root, &metrics.body)?;
    let missing = checks
        .get("missing_metrics")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let status = metrics.status == 200
        && missing.is_empty()
        && checks
            .get("warmup_lock_metrics_present")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
        && checks
            .get("error_registry_aligned")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
        && checks
            .get("startup_log_fields_present")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
        && checks
            .get("redaction_contract_passed")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
        && checks
            .get("dashboard_contract_valid")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
        && checks
            .get("slo_contract_valid")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
        && checks
            .get("alert_rules_contract_valid")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
        && checks
            .get("alert_rules_reference_known_metrics")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
        && checks
            .get("label_policy_passed")
            .and_then(serde_json::Value::as_bool)
            == Some(true);
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": if status { "ok" } else { "failed" },
        "checks": {
            "metrics_endpoint": {
                "path": "/metrics",
                "status": metrics.status,
                "latency_ms": metrics.latency_ms,
                "body_sha256": sha256_text(&metrics.body)
            },
            "required_metrics_present": checks["required_metrics_present"].clone(),
            "missing_metrics": checks["missing_metrics"].clone(),
            "warmup_lock_metrics_present": checks["warmup_lock_metrics_present"].clone(),
            "error_registry_aligned": checks["error_registry_aligned"].clone(),
            "startup_log_fields_present": checks["startup_log_fields_present"].clone(),
            "redaction_contract_passed": checks["redaction_contract_passed"].clone(),
            "dashboard_contract_valid": checks["dashboard_contract_valid"].clone(),
            "slo_contract_valid": checks["slo_contract_valid"].clone(),
            "alert_rules_contract_valid": checks["alert_rules_contract_valid"].clone(),
            "alert_rules_reference_known_metrics": checks["alert_rules_reference_known_metrics"].clone(),
            "label_policy_passed": checks["label_policy_passed"].clone()
        }
    });
    let report_path = write_simulation_report(repo_root, run_id, "ops-obs-verify.json", &payload)?;
    Ok((
        serde_json::json!({
            "schema_version": 1,
            "status": payload["status"].clone(),
            "text": if status { "observability checks passed" } else { "observability checks failed" },
            "rows": [{
                "profile": profile,
                "report_path": report_path.display().to_string(),
                "namespace": namespace,
                "checks": payload["checks"].clone()
            }],
            "summary": {"total": 1, "errors": if status { 0 } else { 1 }, "warnings": 0}
        }),
        if status { 0 } else { 1 },
    ))
}

#[cfg(test)]
mod tests {
    use super::{build_operational_readiness_payload, render_slo_list_payload};

    #[test]
    fn render_slo_list_payload_reads_owned_contract_rows() {
        let repo_root = std::path::Path::new("/Users/bijan/bijux/bijux-atlas");

        let payload = render_slo_list_payload(repo_root).expect("slo list payload");

        assert_eq!(payload["text"], "observe slo list");
        assert!(payload["rows"]
            .as_array()
            .is_some_and(|rows| !rows.is_empty()));
    }

    #[test]
    fn build_operational_readiness_payload_reports_missing_contracts() {
        let root = tempfile::tempdir().expect("tempdir");

        let (payload, exit_code) = build_operational_readiness_payload(root.path(), "run-local")
            .expect("operational readiness payload");

        assert_eq!(exit_code, 1);
        assert_eq!(payload["status"], "failed");
        assert!(payload["rows"][0]["slo"]["errors"]
            .as_array()
            .is_some_and(|rows| !rows.is_empty()));
    }
}
