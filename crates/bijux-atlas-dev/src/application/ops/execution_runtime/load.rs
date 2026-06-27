// SPDX-License-Identifier: Apache-2.0

use crate::cli::OpsCommonArgs;
use crate::ops_commands::{emit_payload, run_id_or_default};
use crate::ops_support::{load_load_manifest, validate_load_manifest};
use crate::{resolve_repo_root, OpsProcess, RunId};
use bijux_atlas_ops::load::path_contracts::{load_report_path, load_run_root, load_summary_path};
use bijux_atlas_ops::load::report_contract::evaluate_load_report;
use serde_json::Value;
use std::fs;

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
    let out_dir = load_run_root(&repo_root, run_id.as_str(), suite);
    fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let summary_path = load_summary_path(&repo_root, run_id.as_str(), suite);
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
    let report = evaluate_load_report(&repo_root, suite, suite_cfg, run_id.as_str()).map_err(
        |err| match err {
            bijux_atlas_ops::load::report_contract::LoadReportError::Read { .. } => {
                format!("OPS_MANIFEST_ERROR: {}", err.detail())
            }
            bijux_atlas_ops::load::report_contract::LoadReportError::Parse { .. } => {
                format!("OPS_SCHEMA_ERROR: {}", err.detail())
            }
        },
    )?;
    let report_path = load_report_path(&repo_root, run_id.as_str(), suite);
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
        "summary":{"total":1,"errors": if report.violations.is_empty() {0} else {1},"warnings":0}
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
