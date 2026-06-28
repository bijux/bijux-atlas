// SPDX-License-Identifier: Apache-2.0

use crate::cli::OpsCommonArgs;
use crate::ops_commands::{emit_payload, run_id_or_default};
use crate::ops_support::{load_load_manifest, validate_load_manifest};
use crate::{resolve_repo_root, OpsProcess, RunId};
use bijux_atlas_ops::load::commands::{
    load_plan_command_payload, load_report_command_payload, load_run_command_payload,
};
use serde_json::Value;

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
    let payload = load_plan_command_payload(suite, suite_cfg, manifest_errors);
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
    let run_id = run_id_or_default(common.run_id.clone())?;
    let process = OpsProcess::new(true);
    let (report_payload, report_code) = run_ops_load_report(common, suite, Some(run_id.clone()))?;
    let report_json: Value =
        serde_json::from_str(&report_payload).unwrap_or_else(|_| serde_json::json!({}));
    let payload = load_run_command_payload(
        &process,
        &repo_root,
        suite,
        suite_cfg,
        run_id.as_str(),
        report_json,
        report_code,
    )?;
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
    let (payload, exit_code) =
        load_report_command_payload(&repo_root, suite, suite_cfg, run_id.as_str())?;
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, exit_code))
}
