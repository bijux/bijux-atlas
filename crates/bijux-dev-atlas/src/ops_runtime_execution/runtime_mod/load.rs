// SPDX-License-Identifier: Apache-2.0

pub(crate) fn run_ops_load_plan(
    common: &OpsCommonArgs,
    suite: &str,
) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let manifest = load_load_manifest(&repo_root).map_err(|e| e.to_stable_message())?;
    let manifest_errors =
        validate_load_manifest(&repo_root, &manifest).map_err(|e| e.to_stable_message())?;
    let suite_cfg = manifest
        .suites
        .get(suite)
        .ok_or_else(|| format!("OPS_USAGE_ERROR: unknown load suite `{suite}`"))?;
    let mut env_rows = suite_cfg
        .env
        .iter()
        .map(|(k, v)| serde_json::json!({"name":k,"value":v}))
        .collect::<Vec<_>>();
    env_rows.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    let payload = serde_json::json!({
        "schema_version":1,
        "text": format!("ops load plan suite={suite}"),
        "rows":[{
            "suite":suite,
            "script":suite_cfg.script,
            "dataset":suite_cfg.dataset,
            "thresholds":suite_cfg.thresholds,
            "env":env_rows
        }],
        "errors":manifest_errors,
        "summary":{"total":1,"errors":manifest_errors.len(),"warnings":0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((
        rendered,
        if payload["summary"]["errors"] == serde_json::json!(0) {
            0
        } else {
            1
        },
    ))
}

pub(crate) fn run_ops_load_run(
    common: &OpsCommonArgs,
    suite: &str,
) -> Result<(String, i32), String> {
    if !common.allow_subprocess {
        return Err("OPS_EFFECT_ERROR: load run requires --allow-subprocess".to_string());
    }
    if !common.allow_network {
        return Err("OPS_EFFECT_ERROR: load run requires --allow-network".to_string());
    }
    if !common.allow_write {
        return Err("OPS_EFFECT_ERROR: load run requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let manifest = load_load_manifest(&repo_root).map_err(|e| e.to_stable_message())?;
    let suite_cfg = manifest
        .suites
        .get(suite)
        .ok_or_else(|| format!("OPS_USAGE_ERROR: unknown load suite `{suite}`"))?;
    let dataset_path = repo_root.join(&suite_cfg.dataset);
    if !dataset_path.exists() {
        return Err(format!(
            "OPS_MANIFEST_ERROR: dataset path missing `{}` and downloads are disabled by default",
            suite_cfg.dataset
        ));
    }
    let run_id = run_id_or_default(common.run_id.clone())?;
    let out_dir = repo_root
        .join("artifacts/ops")
        .join(run_id.as_str())
        .join(format!("load/{suite}"));
    fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let summary_path = out_dir.join("k6-summary.json");
    let process = OpsProcess::new(true);
    let script_path = repo_root.join(&suite_cfg.script);
    let mut argv = vec![
        "run".to_string(),
        script_path.display().to_string(),
        "--summary-export".to_string(),
        summary_path.display().to_string(),
    ];
    for (k, v) in &suite_cfg.env {
        argv.push("-e".to_string());
        argv.push(format!("{k}={v}"));
    }
    let (stdout, event) = process
        .run_subprocess("k6", &argv, &repo_root)
        .map_err(|e| e.to_stable_message())?;
    let (report_payload, report_code) = run_ops_load_report(common, suite, Some(run_id.clone()))?;
    let report_json: Value =
        serde_json::from_str(&report_payload).unwrap_or_else(|_| serde_json::json!({}));
    let payload = serde_json::json!({
        "schema_version":1,
        "text": format!("ops load run suite={suite}"),
        "rows":[{
            "suite":suite,
            "run_id":run_id.as_str(),
            "k6_stdout":stdout,
            "subprocess_event":event,
            "summary_path":summary_path.display().to_string(),
            "report":report_json
        }],
        "summary":{"total":1,"errors": if report_code==0 {0} else {1},"warnings":0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, if report_code == 0 { 0 } else { 1 }))
}

fn load_threshold_limits(repo_root: &Path, threshold_rel: &str) -> Result<Value, String> {
    let path = repo_root.join(threshold_rel);
    let raw =
        fs::read_to_string(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    serde_json::from_str(&raw).map_err(|e| format!("failed to parse {}: {e}", path.display()))
}

fn parse_k6_summary(summary: &Value) -> (f64, f64, f64) {
    let p95 = summary
        .get("metrics")
        .and_then(|v| v.get("http_req_duration"))
        .and_then(|v| v.get("values"))
        .and_then(|v| v.get("p(95)"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let p99 = summary
        .get("metrics")
        .and_then(|v| v.get("http_req_duration"))
        .and_then(|v| v.get("values"))
        .and_then(|v| v.get("p(99)"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let error_rate = summary
        .get("metrics")
        .and_then(|v| v.get("http_req_failed"))
        .and_then(|v| v.get("values"))
        .and_then(|v| v.get("rate"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    (p95, p99, error_rate)
}

pub(crate) fn run_ops_load_report(
    common: &OpsCommonArgs,
    suite: &str,
    run_override: Option<RunId>,
) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let manifest = load_load_manifest(&repo_root).map_err(|e| e.to_stable_message())?;
    let suite_cfg = manifest
        .suites
        .get(suite)
        .ok_or_else(|| format!("OPS_USAGE_ERROR: unknown load suite `{suite}`"))?;
    let run_id = if let Some(v) = run_override {
        v
    } else {
        run_id_or_default(common.run_id.clone())?
    };
    let summary_path = repo_root
        .join("artifacts/ops")
        .join(run_id.as_str())
        .join(format!("load/{suite}/k6-summary.json"));
    let summary_raw = fs::read_to_string(&summary_path).map_err(|e| {
        format!(
            "OPS_MANIFEST_ERROR: failed to read {}: {e}",
            summary_path.display()
        )
    })?;
    let summary_json: Value =
        serde_json::from_str(&summary_raw).map_err(|e| format!("OPS_SCHEMA_ERROR: {e}"))?;
    let thresholds = load_threshold_limits(&repo_root, &suite_cfg.thresholds)?;
    let (p95, p99, error_rate) = parse_k6_summary(&summary_json);
    let p95_max = thresholds
        .get("p95_ms_max")
        .and_then(Value::as_f64)
        .unwrap_or(f64::MAX);
    let p99_max = thresholds
        .get("p99_ms_max")
        .and_then(Value::as_f64)
        .unwrap_or(f64::MAX);
    let error_max = thresholds
        .get("error_rate_max")
        .and_then(Value::as_f64)
        .unwrap_or(f64::MAX);
    let mut violations = Vec::new();
    if p95 > p95_max {
        violations.push(format!("threshold breach p95 {p95} > {p95_max}"));
    }
    if p99 > p99_max {
        violations.push(format!("threshold breach p99 {p99} > {p99_max}"));
    }
    if error_rate > error_max {
        violations.push(format!(
            "threshold breach error_rate {error_rate} > {error_max}"
        ));
    }
    let report = serde_json::json!({
        "schema_version":1,
        "kind":"ops_load_report_v1",
        "suite":suite,
        "run_id":run_id.as_str(),
        "metrics":{"p95_ms":p95,"p99_ms":p99,"error_rate":error_rate},
        "thresholds":{"p95_ms_max":p95_max,"p99_ms_max":p99_max,"error_rate_max":error_max},
        "violations":violations
    });
    let report_path = repo_root
        .join("artifacts/ops")
        .join(run_id.as_str())
        .join(format!("load/{suite}/report.json"));
    if let Some(parent) = report_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(
        &report_path,
        serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let payload = serde_json::json!({
        "schema_version":1,
        "text": format!("ops load report suite={suite}"),
        "rows":[{"report_path":report_path.display().to_string(),"report":report}],
        "summary":{"total":1,"errors": if report["violations"].as_array().is_some_and(|v| v.is_empty()) {0} else {1},"warnings":0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((
        rendered,
        if payload["summary"]["errors"] == serde_json::json!(0) {
            0
        } else {
            1
        },
    ))
}

