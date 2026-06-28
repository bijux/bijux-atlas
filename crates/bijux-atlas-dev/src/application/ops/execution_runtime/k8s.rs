// SPDX-License-Identifier: Apache-2.0

use crate::cli::OpsCommonArgs;
use crate::ops_commands::{emit_payload, load_profiles, resolve_profile, run_id_or_default};
use crate::ops_support::resolve_ops_root;
use crate::{resolve_repo_root, OpsProcess};
use bijux_atlas_ops::kubernetes::access_guard::ensure_namespace_guard;
use bijux_atlas_ops::kubernetes::command_reports::{
    k8s_apply_payload, k8s_logs_payload, k8s_plan_payload,
};
use bijux_atlas_ops::kubernetes::conformance::conformance_summary;
use bijux_atlas_ops::kubernetes::conformance_report::{
    build_conformance_report, write_conformance_report,
};
use bijux_atlas_ops::kubernetes::port_forward::port_forward_payload;
use bijux_atlas_ops::kubernetes::service_inventory::{
    read_service_port_rows, service_port_payload,
};
use bijux_atlas_ops::kubernetes::workload_wait::run_readiness_wait_payload;
use serde_json::Value;
use std::fs;

use super::resolve_render_inputs;

pub(crate) fn run_ops_k8s_plan(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    let profile =
        resolve_profile(common.profile.clone(), &profiles).map_err(|e| e.to_stable_message())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let (render_path, index_path) = resolve_render_inputs(&repo_root, &run_id, &profile.name)
        .map_err(|e| e.to_stable_message())?;
    let index_json: Value = serde_json::from_str(
        &fs::read_to_string(&index_path)
            .map_err(|err| format!("failed to read {}: {err}", index_path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", index_path.display()))?;
    let payload = k8s_plan_payload(
        &profile.name,
        run_id.as_str(),
        &render_path.display().to_string(),
        &index_path.display().to_string(),
        index_json,
    );
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_ops_k8s_apply(
    args: &crate::cli::OpsK8sApplyArgs,
    dry_run: bool,
) -> Result<(String, i32), String> {
    let common = &args.common;
    if !args.apply && !dry_run {
        return Err("OPS_USAGE_ERROR: k8s apply requires explicit --apply".to_string());
    }
    if !common.allow_subprocess {
        return Err("OPS_EFFECT_ERROR: k8s apply requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("OPS_EFFECT_ERROR: k8s apply requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    let profile =
        resolve_profile(common.profile.clone(), &profiles).map_err(|e| e.to_stable_message())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let (render_path, _) = resolve_render_inputs(&repo_root, &run_id, &profile.name)
        .map_err(|e| e.to_stable_message())?;
    let process = OpsProcess::new(true);
    if !dry_run {
        ensure_namespace_guard(
            &process,
            &repo_root,
            &profile.kind_profile,
            common.force,
            "bijux-atlas",
        )?;
    }
    let mut apply_args = vec![
        "apply".to_string(),
        "-n".to_string(),
        "bijux-atlas".to_string(),
        "-f".to_string(),
        render_path.display().to_string(),
    ];
    if dry_run {
        apply_args.push("--dry-run=client".to_string());
    }
    let (stdout, event) = process
        .run_subprocess("kubectl", &apply_args, &repo_root)
        .map_err(|e| e.to_stable_message())?;
    let payload = k8s_apply_payload(
        &profile.name,
        run_id.as_str(),
        dry_run,
        &render_path.display().to_string(),
        &stdout,
        event,
    );
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_ops_k8s_conformance(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    if !common.allow_subprocess {
        return Err("OPS_EFFECT_ERROR: k8s conformance requires --allow-subprocess".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    let profile =
        resolve_profile(common.profile.clone(), &profiles).map_err(|e| e.to_stable_message())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let process = OpsProcess::new(true);
    ensure_namespace_guard(
        &process,
        &repo_root,
        &profile.kind_profile,
        common.force,
        "bijux-atlas",
    )?;
    let (dep_stdout, _) = process
        .run_subprocess(
            "kubectl",
            &[
                "get".to_string(),
                "deployments".to_string(),
                "-n".to_string(),
                "bijux-atlas".to_string(),
                "-o".to_string(),
                "json".to_string(),
            ],
            &repo_root,
        )
        .map_err(|e| e.to_stable_message())?;
    let (pod_stdout, _) = process
        .run_subprocess(
            "kubectl",
            &[
                "get".to_string(),
                "pods".to_string(),
                "-n".to_string(),
                "bijux-atlas".to_string(),
                "-o".to_string(),
                "json".to_string(),
            ],
            &repo_root,
        )
        .map_err(|e| e.to_stable_message())?;
    let deployments: Value = serde_json::from_str(&dep_stdout)
        .map_err(|e| format!("failed parsing deployments json: {e}"))?;
    let pods: Value =
        serde_json::from_str(&pod_stdout).map_err(|e| format!("failed parsing pods json: {e}"))?;
    let (mut errors, mut rows) = conformance_summary(&deployments, &pods);
    let hpa_enabled = process
        .run_subprocess(
            "kubectl",
            &[
                "get".to_string(),
                "hpa".to_string(),
                "-n".to_string(),
                "bijux-atlas".to_string(),
                "-o".to_string(),
                "json".to_string(),
            ],
            &repo_root,
        )
        .ok()
        .and_then(|(stdout, _)| serde_json::from_str::<Value>(&stdout).ok())
        .and_then(|json| {
            json.get("items")
                .and_then(Value::as_array)
                .map(|items| !items.is_empty())
        })
        .unwrap_or(false);
    if hpa_enabled {
        match process.run_subprocess(
            "kubectl",
            &[
                "api-resources".to_string(),
                "--api-group=custom.metrics.k8s.io".to_string(),
                "-o".to_string(),
                "name".to_string(),
            ],
            &repo_root,
        ) {
            Ok((stdout, _)) => {
                let has_custom_metrics = stdout.lines().any(|line| !line.trim().is_empty());
                rows.push(
                    serde_json::json!({"kind":"hpa_metrics_api","enabled":has_custom_metrics}),
                );
                if !has_custom_metrics {
                    errors.push(
                        "hpa enabled but custom metrics API is not available (missing adapter)"
                            .to_string(),
                    );
                }
            }
            Err(err) => {
                rows.push(serde_json::json!({"kind":"hpa_metrics_api","enabled":false}));
                errors.push(format!(
                    "hpa enabled but custom metrics API probe failed: {}",
                    err.to_stable_message()
                ));
            }
        }
    }
    let error_count = errors.len();
    let conformance_report = build_conformance_report(run_id.as_str(), &errors);
    let mut report_path: Option<String> = None;
    if common.allow_write {
        report_path = Some(
            write_conformance_report(&repo_root, &conformance_report)?
                .display()
                .to_string(),
        );
    }
    let payload = serde_json::json!({
        "schema_version":1,
        "text": if errors.is_empty() {"k8s conformance passed"} else {"k8s conformance failed"},
        "rows": rows,
        "errors": errors,
        "conformance_report": conformance_report,
        "conformance_report_path": report_path,
        "summary":{"total":1,"errors": error_count,"warnings":0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, if error_count == 0 { 0 } else { 1 }))
}

pub(crate) fn run_ops_k8s_ports(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    if !common.allow_subprocess {
        return Err("OPS_EFFECT_ERROR: k8s ports requires --allow-subprocess".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    let profile =
        resolve_profile(common.profile.clone(), &profiles).map_err(|e| e.to_stable_message())?;
    let process = OpsProcess::new(true);
    ensure_namespace_guard(
        &process,
        &repo_root,
        &profile.kind_profile,
        common.force,
        "bijux-atlas",
    )?;
    let (rows, svc_event) = read_service_port_rows(&process, &repo_root, "bijux-atlas")?;
    let payload = service_port_payload(rows, svc_event);
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_ops_k8s_wait(args: &crate::cli::OpsK8sWaitArgs) -> Result<(String, i32), String> {
    let common = &args.common;
    if !common.allow_subprocess {
        return Err("OPS_EFFECT_ERROR: k8s wait requires --allow-subprocess".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    let profile =
        resolve_profile(common.profile.clone(), &profiles).map_err(|e| e.to_stable_message())?;
    let process = OpsProcess::new(true);
    ensure_namespace_guard(
        &process,
        &repo_root,
        &profile.kind_profile,
        common.force,
        "bijux-atlas",
    )?;
    let (payload, exit_code) = run_readiness_wait_payload(
        &process,
        &repo_root,
        "bijux-atlas",
        args.timeout_seconds,
        common.fail_fast,
    );
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, exit_code))
}

pub(crate) fn run_ops_k8s_logs(args: &crate::cli::OpsK8sLogsArgs) -> Result<(String, i32), String> {
    let common = &args.common;
    if !common.allow_subprocess {
        return Err("OPS_EFFECT_ERROR: k8s logs requires --allow-subprocess".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    let profile =
        resolve_profile(common.profile.clone(), &profiles).map_err(|e| e.to_stable_message())?;
    let process = OpsProcess::new(true);
    ensure_namespace_guard(
        &process,
        &repo_root,
        &profile.kind_profile,
        common.force,
        "bijux-atlas",
    )?;
    let pod = args
        .pod
        .clone()
        .unwrap_or_else(|| "deployment/bijux-atlas".to_string());
    let argv = vec![
        "logs".to_string(),
        "-n".to_string(),
        "bijux-atlas".to_string(),
        pod,
        format!("--tail={}", args.tail),
    ];
    let (stdout, event) = process
        .run_subprocess("kubectl", &argv, &repo_root)
        .map_err(|e| e.to_stable_message())?;
    let payload = k8s_logs_payload(&stdout, event);
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_ops_k8s_port_forward(
    args: &crate::cli::OpsK8sPortForwardArgs,
) -> Result<(String, i32), String> {
    let common = &args.common;
    if !common.allow_subprocess {
        return Err("OPS_EFFECT_ERROR: k8s port-forward requires --allow-subprocess".to_string());
    }
    if !common.allow_network {
        return Err("OPS_EFFECT_ERROR: k8s port-forward requires --allow-network".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    let profile =
        resolve_profile(common.profile.clone(), &profiles).map_err(|e| e.to_stable_message())?;
    let process = OpsProcess::new(true);
    ensure_namespace_guard(
        &process,
        &repo_root,
        &profile.kind_profile,
        common.force,
        "bijux-atlas",
    )?;
    let payload = port_forward_payload(&args.resource, args.local_port, args.remote_port);
    let _ = repo_root;
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, 0))
}
