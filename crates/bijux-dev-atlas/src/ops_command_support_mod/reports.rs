// SPDX-License-Identifier: Apache-2.0

use bijux_dev_atlas_model::OpsRunReport;

use super::manifests::{load_stack_pins, resolve_ops_root, run_id_or_default};
use super::tools::validate_pins_completeness;
use crate::*;

pub(crate) fn emit_payload(
    format: FormatArg,
    out: Option<PathBuf>,
    payload: &serde_json::Value,
) -> Result<String, String> {
    let rendered = match format {
        FormatArg::Text => payload
            .get("text")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string())
            .unwrap_or_else(|| serde_json::to_string_pretty(payload).unwrap_or_default()),
        FormatArg::Json => serde_json::to_string_pretty(payload).map_err(|err| err.to_string())?,
        FormatArg::Jsonl => {
            if let Some(rows) = payload.get("rows").and_then(|v| v.as_array()) {
                rows.iter()
                    .map(serde_json::to_string)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|err| err.to_string())?
                    .join("\n")
            } else {
                serde_json::to_string(payload).map_err(|err| err.to_string())?
            }
        }
    };
    write_output_if_requested(out, &rendered)?;
    Ok(rendered)
}

pub(crate) mod ops_exit {
    pub const PASS: i32 = 0;
    pub const FAIL: i32 = 1;
    pub const USAGE: i32 = 2;
    pub const INFRA: i32 = 3;
    pub const TOOL_MISSING: i32 = 4;
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_ops_run_report(
    command: &str,
    common: &OpsCommonArgs,
    run_id: &RunId,
    repo_root: &Path,
    ops_root: &Path,
    suite: Option<String>,
    status: &str,
    exit_code: i32,
    warnings: Vec<String>,
    errors: Vec<String>,
    rows: Vec<serde_json::Value>,
) -> OpsRunReport {
    let mut capabilities = std::collections::BTreeMap::new();
    capabilities.insert(
        "subprocess".to_string(),
        if common.allow_subprocess {
            "enabled: requested by flag".to_string()
        } else {
            "disabled: default deny".to_string()
        },
    );
    capabilities.insert(
        "fs_write".to_string(),
        if common.allow_write {
            "enabled: requested by flag".to_string()
        } else {
            "disabled: default deny".to_string()
        },
    );
    capabilities.insert(
        "network".to_string(),
        if common.allow_network {
            "enabled: requested by flag".to_string()
        } else {
            "disabled: default deny".to_string()
        },
    );
    let mut summary = std::collections::BTreeMap::new();
    summary.insert("warnings".to_string(), warnings.len() as u64);
    summary.insert("errors".to_string(), errors.len() as u64);
    summary.insert("rows".to_string(), rows.len() as u64);
    OpsRunReport {
        schema_version: bijux_dev_atlas_model::schema_version(),
        kind: "ops_run_report_v1".to_string(),
        command: command.to_string(),
        run_id: run_id.clone(),
        repo_root: repo_root.display().to_string(),
        ops_root: ops_root.display().to_string(),
        profile: common.profile.clone(),
        suite,
        status: status.to_string(),
        exit_code,
        checks: Vec::new(),
        warnings,
        errors,
        capabilities,
        summary,
        rows,
    }
}

pub(crate) fn render_ops_human(report: &OpsRunReport) -> String {
    let mut lines = vec![
        format!("ops {} [{}]", report.command, report.status),
        format!("run_id={}", report.run_id),
        format!(
            "errors={} warnings={}",
            report.summary.get("errors").copied().unwrap_or(0),
            report.summary.get("warnings").copied().unwrap_or(0)
        ),
    ];
    let mut errs = report.errors.clone();
    errs.sort();
    for err in errs {
        lines.push(format!("E {err}"));
    }
    let mut warns = report.warnings.clone();
    warns.sort();
    for warn in warns {
        lines.push(format!("W {warn}"));
    }
    lines.join("\n")
}

pub(crate) fn sha256_hex(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub(crate) fn run_ops_checks(
    common: &OpsCommonArgs,
    suite: &str,
    include_internal: bool,
    include_slow: bool,
) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let selectors = parse_selectors(
        Some(suite.to_string()),
        Some(DomainArg::Ops),
        None,
        None,
        include_internal,
        include_slow,
    )?;
    let request = RunRequest {
        repo_root: repo_root.clone(),
        domain: Some(DomainId::Ops),
        capabilities: Capabilities::deny_all(),
        artifacts_root: Some(
            common
                .artifacts_root
                .clone()
                .unwrap_or_else(|| repo_root.join("artifacts")),
        ),
        run_id: Some(run_id_or_default(common.run_id.clone())?),
        command: Some(format!("bijux dev atlas ops {suite}")),
    };
    let report = run_checks(
        &RealProcessRunner,
        &RealFs,
        &request,
        &selectors,
        &RunOptions {
            fail_fast: common.fail_fast,
            max_failures: common.max_failures,
        },
    )?;
    let rendered = match common.format {
        FormatArg::Text => render_text_with_durations(&report, 10),
        FormatArg::Json => render_json(&report)?,
        FormatArg::Jsonl => render_jsonl(&report)?,
    };
    write_output_if_requested(common.out.clone(), &rendered)?;
    Ok((rendered, exit_code_for_report(&report)))
}

pub(crate) fn render_ops_validation_output(
    common: &OpsCommonArgs,
    mode: &str,
    inventory_errors: &[String],
    checks_rendered: &str,
    checks_exit: i32,
    summary: serde_json::Value,
) -> Result<(String, i32), String> {
    let inventory_error_count = inventory_errors.len();
    let checks_error_count = if checks_exit == 0 { 0 } else { 1 };
    let error_count = inventory_error_count + checks_error_count;
    let status = if error_count == 0 { "ok" } else { "failed" };
    let strict_failed = common.strict && error_count > 0;
    let exit = if strict_failed || checks_exit != 0 || inventory_error_count > 0 {
        1
    } else {
        0
    };
    let text = format!(
        "ops {mode}: status={status} inventory_errors={inventory_error_count} checks_exit={checks_exit}"
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "mode": mode,
        "status": status,
        "text": text,
        "rows": [{
            "inventory_errors": inventory_errors,
            "checks_exit": checks_exit,
            "checks_output": checks_rendered,
            "inventory_summary": summary
        }],
        "summary": {
            "total": 1,
            "errors": error_count,
            "warnings": 0
        }
    });
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, exit))
}

pub(crate) fn ops_pins_check_payload(
    common: &OpsCommonArgs,
    repo_root: &Path,
) -> Result<(serde_json::Value, i32), String> {
    let ops_root =
        resolve_ops_root(repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut errors = Vec::new();
    if let Err(err) =
        bijux_dev_atlas_core::ops_inventory::OpsInventory::load_and_validate(&ops_root)
    {
        errors.push(err);
    }
    let pins = load_stack_pins(repo_root).map_err(|e| e.to_stable_message())?;
    errors.extend(validate_pins_completeness(repo_root, &pins).map_err(|e| e.to_stable_message())?);
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": status,
        "text": if errors.is_empty() { "ops pins check passed" } else { "ops pins check failed" },
        "rows": [{
            "pins_path": "ops/inventory/pins.yaml",
            "errors": errors
        }],
        "summary": {"total": 1, "errors": if status == "ok" {0} else {1}, "warnings": 0}
    });
    Ok((payload, if status == "ok" { 0 } else { 1 }))
}
