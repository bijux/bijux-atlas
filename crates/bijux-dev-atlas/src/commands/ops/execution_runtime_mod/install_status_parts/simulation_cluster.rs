// SPDX-License-Identifier: Apache-2.0
//! Kind and simulation-cluster operations for install-status flows.

use super::*;
use std::time::Duration;

fn write_observe_contract_report(
    repo_root: &std::path::Path,
    run_id: &RunId,
    file_name: &str,
    payload: &serde_json::Value,
) -> Result<String, String> {
    let out_dir = repo_root.join("artifacts/ops").join(run_id.as_str()).join("observe");
    std::fs::create_dir_all(&out_dir)
        .map_err(|err| format!("failed to create {}: {err}", out_dir.display()))?;
    let out_path = out_dir.join(file_name);
    std::fs::write(
        &out_path,
        serde_json::to_string_pretty(payload).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("failed to write {}: {err}", out_path.display()))?;
    Ok(out_path
        .strip_prefix(repo_root)
        .unwrap_or(&out_path)
        .display()
        .to_string())
}

pub(crate) fn run_ops_observe_slo_list(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let slo_path = repo_root.join("ops/observe/slo-definitions.json");
    let slo: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&slo_path)
            .map_err(|err| format!("failed to read {}: {err}", slo_path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", slo_path.display()))?;
    let rows = slo
        .get("slos")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let payload = serde_json::json!({
        "schema_version": 1,
        "text": "observe slo list",
        "rows": rows,
        "summary": {"total": rows.len(), "errors": 0, "warnings": 0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_ops_observe_slo_verify(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    if !common.allow_write {
        return Err("observe slo verify requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let slo: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/observe/slo-definitions.json"))
            .map_err(|err| format!("failed to read slo-definitions.json: {err}"))?,
    )
    .map_err(|err| format!("failed to parse slo-definitions.json: {err}"))?;
    let measurement: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/observe/slo-measurement.json"))
            .map_err(|err| format!("failed to read slo-measurement.json: {err}"))?,
    )
    .map_err(|err| format!("failed to parse slo-measurement.json: {err}"))?;
    let metric_map: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/observe/slo-metric-map.json"))
            .map_err(|err| format!("failed to read slo-metric-map.json: {err}"))?,
    )
    .map_err(|err| format!("failed to parse slo-metric-map.json: {err}"))?;
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
    let report_rel = write_observe_contract_report(&repo_root, &run_id, "slo-contract-report.json", &report)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": report["status"].clone(),
        "text": "observe slo verify",
        "rows": [{"report_path": report_rel, "errors": report["errors"].clone()}],
        "summary": {"total": 1, "errors": report["errors"].as_array().map(|v| v.len()).unwrap_or(0), "warnings": 0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, if errors.is_empty() { 0 } else { 1 }))
}

pub(crate) fn run_ops_observe_alerts_verify(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    if !common.allow_write {
        return Err("observe alerts verify requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let contract: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/observe/contracts/alerts-contract.json"))
            .map_err(|err| format!("failed to read alerts-contract.json: {err}"))?,
    )
    .map_err(|err| format!("failed to parse alerts-contract.json: {err}"))?;
    let required = contract
        .get("required_alerts")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| row.as_str().map(ToString::to_string))
        .collect::<std::collections::BTreeSet<_>>();
    let mut observed = std::collections::BTreeSet::new();
    let mut errors = Vec::new();
    for alerts_file in [
        "ops/observe/alerts/atlas-alert-rules.yaml",
        "ops/observe/alerts/slo-burn-rules.yaml",
    ] {
        let alerts_path = repo_root.join(alerts_file);
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
                        errors.push(format!("alert missing label `{required_label}` in {alerts_file}"));
                    }
                }
                let runbook = rule
                    .get("annotations")
                    .and_then(|row| row.get("runbook"))
                    .and_then(serde_yaml::Value::as_str)
                    .unwrap_or_default();
                if runbook.is_empty() {
                    errors.push(format!("alert missing annotations.runbook in {alerts_file}"));
                }
            }
        }
    }
    for alert in required {
        if !observed.contains(&alert) {
            errors.push(format!("required alert missing from alert rules: `{alert}`"));
        }
    }
    let report = serde_json::json!({
        "schema_version": 1,
        "status": if errors.is_empty() { "ok" } else { "failed" },
        "alerts_total": observed.len(),
        "errors": errors
    });
    let report_rel =
        write_observe_contract_report(&repo_root, &run_id, "alerts-contract-report.json", &report)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": report["status"].clone(),
        "text": "observe alerts verify",
        "rows": [{"report_path": report_rel, "errors": report["errors"].clone()}],
        "summary": {"total": 1, "errors": report["errors"].as_array().map(|v| v.len()).unwrap_or(0), "warnings": 0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, if report["errors"].as_array().is_some_and(|v| v.is_empty()) { 0 } else { 1 }))
}

pub(crate) fn run_ops_observe_runbooks_verify(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    if !common.allow_write {
        return Err("observe runbooks verify requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let mut errors = Vec::new();
    let mut checked = 0usize;
    for alerts_file in [
        "ops/observe/alerts/atlas-alert-rules.yaml",
        "ops/observe/alerts/slo-burn-rules.yaml",
    ] {
        let alerts_path = repo_root.join(alerts_file);
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
                    errors.push(format!("runbook does not describe required evidence bundle: {runbook}"));
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
    let report_rel = write_observe_contract_report(
        &repo_root,
        &run_id,
        "runbooks-contract-report.json",
        &report,
    )?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": report["status"].clone(),
        "text": "observe runbooks verify",
        "rows": [{"report_path": report_rel, "errors": report["errors"].clone()}],
        "summary": {"total": 1, "errors": report["errors"].as_array().map(|v| v.len()).unwrap_or(0), "warnings": 0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, if report["errors"].as_array().is_some_and(|v| v.is_empty()) { 0 } else { 1 }))
}

pub(crate) fn run_ops_observe_readiness(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    if !common.allow_write {
        return Err("observe readiness requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let base = repo_root.join("artifacts/ops").join(run_id.as_str()).join("observe");
    let read_report = |name: &str| -> serde_json::Value {
        let path = base.join(name);
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
            .unwrap_or_else(|| serde_json::json!({
                "status":"missing",
                "errors":[format!("missing report {}", path.display())]
            }))
    };
    let slo = read_report("slo-contract-report.json");
    let alerts = read_report("alerts-contract-report.json");
    let runbooks = read_report("runbooks-contract-report.json");
    let checks = [slo.clone(), alerts.clone(), runbooks.clone()];
    let passed = checks
        .iter()
        .filter(|row| row.get("status").and_then(serde_json::Value::as_str) == Some("ok"))
        .count();
    let total = checks.len();
    let completeness = if total == 0 { 0.0 } else { passed as f64 / total as f64 };
    let threshold = 1.0f64;
    let status = if completeness >= threshold { "ok" } else { "failed" };
    let report = serde_json::json!({
        "schema_version": 1,
        "status": status,
        "completeness": completeness,
        "threshold": threshold,
        "reports": {
            "slo": format!("artifacts/ops/{}/observe/slo-contract-report.json", run_id.as_str()),
            "alerts": format!("artifacts/ops/{}/observe/alerts-contract-report.json", run_id.as_str()),
            "runbooks": format!("artifacts/ops/{}/observe/runbooks-contract-report.json", run_id.as_str())
        }
    });
    let report_rel = write_observe_contract_report(
        &repo_root,
        &run_id,
        "operational-readiness-report.json",
        &report,
    )?;
    let human_rel = {
        let out_dir = repo_root.join("artifacts/ops").join(run_id.as_str()).join("observe");
        std::fs::create_dir_all(&out_dir)
            .map_err(|err| format!("failed to create {}: {err}", out_dir.display()))?;
        let out_path = out_dir.join("operational-readiness-report.md");
        let lines = [
            "# Operational Readiness Report".to_string(),
            format!("- Status: {}", status),
            format!("- Completeness: {:.2}", completeness),
            format!("- Threshold: {:.2}", threshold),
            format!(
                "- SLO report: artifacts/ops/{}/observe/slo-contract-report.json",
                run_id.as_str()
            ),
            format!(
                "- Alerts report: artifacts/ops/{}/observe/alerts-contract-report.json",
                run_id.as_str()
            ),
            format!(
                "- Runbooks report: artifacts/ops/{}/observe/runbooks-contract-report.json",
                run_id.as_str()
            ),
        ];
        std::fs::write(&out_path, lines.join("\n") + "\n")
            .map_err(|err| format!("failed to write {}: {err}", out_path.display()))?;
        out_path
            .strip_prefix(&repo_root)
            .unwrap_or(&out_path)
            .display()
            .to_string()
    };
    let payload = serde_json::json!({
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
    });
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

pub(crate) fn run_ops_obs_verify(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    if !common.allow_subprocess {
        return Err("obs verify requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("obs verify requires --allow-write".to_string());
    }
    if !common.allow_network {
        return Err("obs verify requires --allow-network".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let profile = common
        .profile
        .clone()
        .unwrap_or_else(|| "profile-baseline".to_string());
    let namespace = simulation_namespace(&profile, None);
    let mut child = std::process::Command::new("kubectl")
        .args([
            "port-forward",
            "-n",
            &namespace,
            "--address",
            "127.0.0.1",
            "service/bijux-atlas",
            "18081:8080",
        ])
        .current_dir(&repo_root)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|err| format!("failed to start kubectl port-forward: {err}"))?;
    let result = (|| -> Result<serde_json::Value, String> {
        wait_for_local_port(18081, Duration::from_secs(10))?;
        let metrics = perform_http_request(18081, "/metrics")?;
        let checks = observability_contract_checks(&repo_root, &metrics.body)?;
        let missing = checks
            .get("missing_metrics")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        let status = metrics.status == 200
            && missing.is_empty()
            && checks.get("warmup_lock_metrics_present").and_then(serde_json::Value::as_bool) == Some(true)
            && checks.get("error_registry_aligned").and_then(serde_json::Value::as_bool) == Some(true)
            && checks.get("startup_log_fields_present").and_then(serde_json::Value::as_bool) == Some(true)
            && checks.get("redaction_contract_passed").and_then(serde_json::Value::as_bool) == Some(true)
            && checks.get("dashboard_contract_valid").and_then(serde_json::Value::as_bool) == Some(true)
            && checks.get("slo_contract_valid").and_then(serde_json::Value::as_bool) == Some(true)
            && checks.get("alert_rules_contract_valid").and_then(serde_json::Value::as_bool) == Some(true)
            && checks.get("alert_rules_reference_known_metrics").and_then(serde_json::Value::as_bool) == Some(true)
            && checks.get("label_policy_passed").and_then(serde_json::Value::as_bool) == Some(true);
        Ok(serde_json::json!({
            "schema_version": 1,
            "status": if status { "ok" } else { "failed" },
            "checks": {
                "metrics_endpoint": {
                    "path": "/metrics",
                    "status": metrics.status,
                    "latency_ms": metrics.latency_ms,
                    "body_sha256": sha256_hex(&metrics.body)
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
        }))
    })();
    let _ = child.kill();
    let _ = child.wait();
    let payload = result?;
    let report_path = write_simulation_report(&repo_root, &run_id, "ops-obs-verify.json", &payload)?;
    let status = payload["status"].as_str().unwrap_or("failed");
    let rendered = emit_payload(
        common.format,
        common.out.clone(),
        &serde_json::json!({
            "schema_version": 1,
            "status": status,
            "text": if status == "ok" { "observability checks passed" } else { "observability checks failed" },
            "rows": [{
                "report_path": report_path.display().to_string(),
                "namespace": namespace,
                "checks": payload["checks"].clone()
            }],
            "summary": {"total": 1, "errors": if status == "ok" { 0 } else { 1 }, "warnings": 0}
        }),
    )?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

pub(crate) fn run_ops_drill(
    args: &crate::cli::OpsDrillRunArgs,
) -> Result<(String, i32), String> {
    let common = &args.common;
    if !common.allow_write {
        return Err("drills run requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let drills = load_drill_registry(&repo_root)?;
    let drill = drills
        .iter()
        .find(|row| row.get("name").and_then(serde_json::Value::as_str) == Some(args.name.as_str()))
        .cloned()
        .ok_or_else(|| format!("unknown drill `{}`", args.name))?;
    let mut checks = Vec::new();
    for (name, path) in drill_check_paths(&repo_root, &args.name) {
        checks.push(serde_json::json!({
            "name": name,
            "status": if path.exists() { "pass" } else { "fail" },
            "detail": if path.exists() {
                format!("verified {}", path.strip_prefix(&repo_root).unwrap_or(&path).display())
            } else {
                format!("missing {}", path.strip_prefix(&repo_root).unwrap_or(&path).display())
            }
        }));
    }
    let status = if checks
        .iter()
        .all(|row| row.get("status").and_then(serde_json::Value::as_str) == Some("pass"))
    {
        "pass"
    } else {
        "fail"
    };
    let evidence_paths = drill_check_paths(&repo_root, &args.name)
        .into_iter()
        .map(|(_, path)| path.strip_prefix(&repo_root).unwrap_or(&path).display().to_string())
        .collect::<Vec<_>>();
    let payload = serde_json::json!({
        "schema_version": 1,
        "drill": args.name,
        "status": status,
        "execution_mode": "contract-verification",
        "expected_outcome": drill.get("expected_outcome").cloned().unwrap_or(serde_json::Value::String(String::new())),
        "checks": checks,
        "evidence_paths": evidence_paths
    });
    let report_path = write_simulation_report(
        &repo_root,
        &run_id,
        &format!("ops-drill-{}.json", args.name),
        &payload,
    )?;
    let summary_path = update_drill_summary(&repo_root, &run_id, &args.name, &report_path, status)?;
    let rendered = emit_payload(
        common.format,
        common.out.clone(),
        &serde_json::json!({
            "schema_version": 1,
            "status": status,
            "text": if status == "pass" { "drill checks passed" } else { "drill checks failed" },
            "rows": [{
                "drill": args.name,
                "report_path": report_path.display().to_string(),
                "summary_path": summary_path.display().to_string(),
                "expected_outcome": drill.get("expected_outcome").cloned().unwrap_or(serde_json::Value::String(String::new()))
            }],
            "summary": {"total": 1, "errors": if status == "pass" { 0 } else { 1 }, "warnings": 0}
        }),
    )?;
    Ok((rendered, if status == "pass" { 0 } else { 1 }))
}

pub(crate) fn run_ops_kind_up(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    if !common.allow_subprocess {
        return Err("kind up requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("kind up requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let process = OpsProcess::new(true);
    let run_id = run_id_or_default(common.run_id.clone())?;
    let config_path = simulation_cluster_config(&repo_root);
    let args = vec![
        "create".to_string(),
        "cluster".to_string(),
        "--name".to_string(),
        simulation_cluster_name().to_string(),
        "--config".to_string(),
        config_path.display().to_string(),
    ];
    let result = process.run_subprocess("kind", &args, &repo_root);
    let (status, detail) = match result {
        Ok((stdout, event)) => ("ok", serde_json::json!({"stdout": stdout, "event": event})),
        Err(err) => {
            let stable = err.to_stable_message();
            if stable.contains("already exists") {
                ("ok", serde_json::json!({"detail": "cluster already exists"}))
            } else {
                ("failed", serde_json::json!({"error": stable}))
            }
        }
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "cluster": "kind",
        "action": "up",
        "status": status,
        "details": {
            "cluster_name": simulation_cluster_name(),
            "cluster_config": config_path.display().to_string(),
            "context": simulation_cluster_context(),
            "result": detail
        }
    });
    let report_path = write_simulation_report(&repo_root, &run_id, "ops-kind.json", &payload)?;
    let envelope = serde_json::json!({
        "schema_version": 1,
        "text": if status == "ok" { "kind cluster ready" } else { "kind cluster failed" },
        "rows": [{
            "schema_version": 1,
            "cluster": "kind",
            "action": "up",
            "status": status,
            "report_path": report_path.display().to_string(),
            "details": payload["details"].clone()
        }],
        "summary": {"total": 1, "errors": if status == "ok" { 0 } else { 1 }, "warnings": 0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

pub(crate) fn run_ops_kind_down(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    if !common.allow_subprocess {
        return Err("kind down requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("kind down requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let process = OpsProcess::new(true);
    let run_id = run_id_or_default(common.run_id.clone())?;
    let args = vec![
        "delete".to_string(),
        "cluster".to_string(),
        "--name".to_string(),
        simulation_cluster_name().to_string(),
    ];
    let result = process.run_subprocess("kind", &args, &repo_root);
    let (status, detail) = match result {
        Ok((stdout, event)) => ("ok", serde_json::json!({"stdout": stdout, "event": event})),
        Err(err) => ("failed", serde_json::json!({"error": err.to_stable_message()})),
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "cluster": "kind",
        "action": "down",
        "status": status,
        "details": {
            "cluster_name": simulation_cluster_name(),
            "result": detail
        }
    });
    let report_path = write_simulation_report(&repo_root, &run_id, "ops-kind.json", &payload)?;
    let envelope = serde_json::json!({
        "schema_version": 1,
        "text": if status == "ok" { "kind cluster deleted" } else { "kind cluster delete failed" },
        "rows": [{
            "schema_version": 1,
            "cluster": "kind",
            "action": "down",
            "status": status,
            "report_path": report_path.display().to_string(),
            "details": payload["details"].clone()
        }],
        "summary": {"total": 1, "errors": if status == "ok" { 0 } else { 1 }, "warnings": 0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

pub(crate) fn run_ops_kind_status(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    if !common.allow_subprocess {
        return Err("kind status requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("kind status requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let process = OpsProcess::new(true);
    let run_id = run_id_or_default(common.run_id.clone())?;
    let args = vec![
        "--context".to_string(),
        simulation_cluster_context(),
        "get".to_string(),
        "nodes".to_string(),
        "-o".to_string(),
        "json".to_string(),
    ];
    let result = process.run_subprocess("kubectl", &args, &repo_root);
    let (status, details) = match result {
        Ok((stdout, event)) => {
            let json: serde_json::Value = serde_json::from_str(&stdout)
                .map_err(|err| format!("failed to parse kubectl nodes json: {err}"))?;
            let rows = json
                .get("items")
                .and_then(|value| value.as_array())
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|item| {
                    let name = item["metadata"]["name"].as_str().unwrap_or("unknown");
                    let ready = item["status"]["conditions"]
                        .as_array()
                        .is_some_and(|conditions| {
                            conditions.iter().any(|condition| {
                                condition["type"].as_str() == Some("Ready")
                                    && condition["status"].as_str() == Some("True")
                            })
                        });
                    serde_json::json!({"name": name, "ready": ready})
                })
                .collect::<Vec<_>>();
            ("ok", serde_json::json!({"event": event, "nodes": rows}))
        }
        Err(err) => ("failed", serde_json::json!({"error": err.to_stable_message()})),
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "cluster": "kind",
        "action": "status",
        "status": status,
        "details": details
    });
    let report_path = write_simulation_report(&repo_root, &run_id, "ops-kind.json", &payload)?;
    let envelope = serde_json::json!({
        "schema_version": 1,
        "text": if status == "ok" { "kind cluster status collected" } else { "kind cluster status failed" },
        "rows": [{
            "schema_version": 1,
            "cluster": "kind",
            "action": "status",
            "status": status,
            "report_path": report_path.display().to_string(),
            "details": payload["details"].clone()
        }],
        "summary": {"total": 1, "errors": if status == "ok" { 0 } else { 1 }, "warnings": 0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

pub(crate) fn run_ops_kind_preload(
    args: &crate::cli::OpsKindPreloadArgs,
) -> Result<(String, i32), String> {
    let common = &args.common;
    if !common.allow_subprocess {
        return Err("kind preload-image requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("kind preload-image requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let process = OpsProcess::new(true);
    let run_id = run_id_or_default(common.run_id.clone())?;
    let argv = vec![
        "load".to_string(),
        "docker-image".to_string(),
        args.image.clone(),
        "--name".to_string(),
        simulation_cluster_name().to_string(),
    ];
    let result = process.run_subprocess("kind", &argv, &repo_root);
    let (status, details) = match result {
        Ok((stdout, event)) => ("ok", serde_json::json!({"stdout": stdout, "event": event})),
        Err(err) => ("failed", serde_json::json!({"error": err.to_stable_message()})),
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "cluster": "kind",
        "action": "preload-image",
        "status": status,
        "details": {
            "image": args.image,
            "result": details
        }
    });
    let report_path = write_simulation_report(&repo_root, &run_id, "ops-kind.json", &payload)?;
    let envelope = serde_json::json!({
        "schema_version": 1,
        "text": if status == "ok" { "kind image preload complete" } else { "kind image preload failed" },
        "rows": [{
            "schema_version": 1,
            "cluster": "kind",
            "action": "preload-image",
            "status": status,
            "report_path": report_path.display().to_string(),
            "details": payload["details"].clone()
        }],
        "summary": {"total": 1, "errors": if status == "ok" { 0 } else { 1 }, "warnings": 0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

pub(crate) fn run_ops_helm_install(
    args: &crate::cli::OpsHelmInstallArgs,
) -> Result<(String, i32), String> {
    let common = &args.release.common;
    match args.release.cluster {
        crate::cli::OpsClusterTarget::Kind => {}
    }
    if !common.allow_subprocess {
        return Err("helm install requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("helm install requires --allow-write".to_string());
    }
    if !common.allow_network {
        return Err("helm install requires --allow-network".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let process = OpsProcess::new(true);
    ensure_simulation_context(&process, common.force)?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let profile = common
        .profile
        .clone()
        .unwrap_or_else(|| "kind".to_string());
    let namespace = simulation_namespace(&profile, args.release.namespace.as_deref());
    let values_file = resolve_profile_values_file(&repo_root, &profile)?;
    let chart_path = resolve_chart_source(&repo_root, args.chart_source)?;
    let helm_args = vec![
        "upgrade".to_string(),
        "--install".to_string(),
        "bijux-atlas".to_string(),
        chart_path.display().to_string(),
        "--namespace".to_string(),
        namespace.clone(),
        "--create-namespace".to_string(),
        "--values".to_string(),
        values_file.display().to_string(),
    ];
    let (helm_stdout, helm_event) = process
        .run_subprocess("helm", &helm_args, &repo_root)
        .map_err(|err| err.to_stable_message())?;
    let (wait_rows, wait_errors, wait_ms) =
        run_simulation_wait(&process, &repo_root, &namespace, args.release.timeout_seconds);
    let smoke_rows = if wait_errors.is_empty() {
        run_smoke_checks(&repo_root, &namespace, 18080)?
    } else {
        Vec::new()
    };
    let smoke_errors = smoke_rows
        .iter()
        .filter(|row| row["status"].as_u64().unwrap_or(0) != 200)
        .map(|row| {
            format!(
                "{} returned status {}",
                row["path"].as_str().unwrap_or("unknown"),
                row["status"].as_u64().unwrap_or(0)
            )
        })
        .collect::<Vec<_>>();
    let errors = wait_errors
        .iter()
        .cloned()
        .chain(smoke_errors.iter().cloned())
        .collect::<Vec<_>>();
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let smoke_payload = serde_json::json!({
        "schema_version": 1,
        "cluster": "kind",
        "namespace": namespace,
        "status": if wait_errors.is_empty() && smoke_errors.is_empty() { "ok" } else { "failed" },
        "checks": smoke_rows
    });
    let smoke_report_path =
        write_simulation_report(&repo_root, &run_id, "ops-smoke.json", &smoke_payload)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "profile": profile,
        "cluster": "kind",
        "namespace": namespace,
        "status": status,
        "details": {
            "helm": {
                "stdout": helm_stdout,
                "event": helm_event,
                "values_file": values_file.display().to_string(),
                "chart_path": chart_path.display().to_string(),
                "chart_source": match args.chart_source {
                    crate::cli::OpsHelmChartSource::Current => "current",
                    crate::cli::OpsHelmChartSource::Previous => "previous"
                }
            },
            "readiness_wait": {
                "elapsed_ms": wait_ms,
                "rows": wait_rows,
                "errors": wait_errors
            },
            "kubeconform": record_kubeconform_result(&process, &repo_root, &run_id, &profile),
            "configmap_env_keys": extract_configmap_env_keys(&repo_root, &run_id, &profile)?,
            "runtime_allowlist": runtime_allowlist_status(&repo_root),
            "smoke": {
                "report_path": smoke_report_path.display().to_string(),
                "checks": smoke_payload["checks"].clone()
            },
            "profile_intent": load_profile_intent(&repo_root, &profile)?,
            "profile_metadata": load_profile_registry(&repo_root, &profile)?
        }
    });
    let report_path = write_simulation_report(&repo_root, &run_id, "ops-install.json", &payload)?;
    let summary_path = update_simulation_summary(
        &repo_root,
        &run_id,
        &profile,
        &namespace,
        SimulationSummaryUpdate {
            install_report_path: Some(&report_path),
            install_status: Some(status),
            smoke_report_path: Some(&smoke_report_path),
            smoke_status: Some(smoke_payload["status"].as_str().unwrap_or("failed")),
            cleanup_report_path: None,
            cleanup_status: None,
        },
    )?;
    let envelope = serde_json::json!({
        "schema_version": 1,
        "text": if status == "ok" { "helm install completed" } else { "helm install failed" },
        "rows": [{
            "schema_version": 1,
            "profile": payload["profile"].clone(),
            "cluster": "kind",
            "namespace": payload["namespace"].clone(),
            "status": status,
            "report_path": report_path.display().to_string(),
            "summary_report_path": summary_path.display().to_string(),
            "details": payload["details"].clone()
        }],
        "summary": {"total": 1, "errors": errors.len(), "warnings": 0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, if errors.is_empty() { 0 } else { 1 }))
}

pub(crate) fn run_ops_helm_uninstall(
    args: &crate::cli::OpsHelmReleaseArgs,
) -> Result<(String, i32), String> {
    let common = &args.common;
    match args.cluster {
        crate::cli::OpsClusterTarget::Kind => {}
    }
    if !common.allow_subprocess {
        return Err("helm uninstall requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("helm uninstall requires --allow-write".to_string());
    }
    if !common.allow_network {
        return Err("helm uninstall requires --allow-network".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let process = OpsProcess::new(true);
    ensure_simulation_context(&process, common.force)?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let profile = common
        .profile
        .clone()
        .unwrap_or_else(|| "kind".to_string());
    let namespace = simulation_namespace(&profile, args.namespace.as_deref());
    let helm_args = vec![
        "uninstall".to_string(),
        "bijux-atlas".to_string(),
        "--namespace".to_string(),
        namespace.clone(),
    ];
    let (helm_stdout, helm_event) = process
        .run_subprocess("helm", &helm_args, &repo_root)
        .map_err(|err| err.to_stable_message())?;
    let cleanup_args = vec![
        "get".to_string(),
        "all".to_string(),
        "-n".to_string(),
        namespace.clone(),
        "-o".to_string(),
        "name".to_string(),
    ];
    let (cleanup_stdout, cleanup_event) = process
        .run_subprocess("kubectl", &cleanup_args, &repo_root)
        .map_err(|err| err.to_stable_message())?;
    let leftovers = cleanup_stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    let status = if leftovers.is_empty() { "ok" } else { "failed" };
    let cleanup_payload = serde_json::json!({
        "schema_version": 1,
        "cluster": "kind",
        "namespace": namespace,
        "status": status,
        "leftovers": leftovers
    });
    let cleanup_report_path =
        write_simulation_report(&repo_root, &run_id, "ops-cleanup.json", &cleanup_payload)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "profile": profile,
        "cluster": "kind",
        "namespace": cleanup_payload["namespace"].clone(),
        "status": status,
        "details": {
            "helm": {
                "stdout": helm_stdout,
                "event": helm_event
            },
            "cleanup_check": {
                "report_path": cleanup_report_path.display().to_string(),
                "leftovers": cleanup_payload["leftovers"].clone(),
                "event": cleanup_event
            }
        }
    });
    let report_path =
        write_simulation_report(&repo_root, &run_id, "ops-uninstall.json", &payload)?;
    let summary_path = update_simulation_summary(
        &repo_root,
        &run_id,
        &profile,
        &namespace,
        SimulationSummaryUpdate {
            install_report_path: None,
            install_status: None,
            smoke_report_path: None,
            smoke_status: None,
            cleanup_report_path: Some(&cleanup_report_path),
            cleanup_status: Some(status),
        },
    )?;
    let envelope = serde_json::json!({
        "schema_version": 1,
        "text": if status == "ok" { "helm uninstall completed" } else { "helm uninstall left resources" },
        "rows": [{
            "schema_version": 1,
            "profile": payload["profile"].clone(),
            "cluster": "kind",
            "namespace": payload["namespace"].clone(),
            "status": status,
            "report_path": report_path.display().to_string(),
            "summary_report_path": summary_path.display().to_string(),
            "details": payload["details"].clone()
        }],
        "summary": {"total": 1, "errors": if status == "ok" { 0 } else { 1 }, "warnings": 0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}
