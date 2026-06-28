// SPDX-License-Identifier: Apache-2.0
//! Release lifecycle simulation commands for install-status flows.

use super::*;
use crate::cli;
use crate::ops_commands::{emit_payload, run_id_or_default};
use crate::{resolve_repo_root, OpsCommandError, OpsProcess};

pub(crate) fn run_ops_helm_upgrade(
    args: &crate::cli::OpsHelmUpgradeArgs,
) -> Result<(String, i32), String> {
    let common = &args.release.common;
    match args.release.cluster {
        crate::cli::OpsClusterTarget::Kind => {}
    }
    if !common.allow_subprocess {
        return Err("helm upgrade requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("helm upgrade requires --allow-write".to_string());
    }
    if !common.allow_network {
        return Err("helm upgrade requires --allow-network".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let process = OpsProcess::new(true);
    ensure_simulation_context(&process, common.force)?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let profile = common.profile.clone().unwrap_or_else(|| "kind".to_string());
    let namespace = bijux_atlas_ops::workspace::profiles::simulation_namespace(
        &profile,
        args.release.namespace.as_deref(),
    );
    let values_file =
        bijux_atlas_ops::workspace::profiles::resolve_profile_values_file(&repo_root, &profile)
            .map_err(|err| err.detail())?;
    let chart_path = match args.to {
        crate::cli::OpsHelmTarget::Current => simulation_current_chart_path(&repo_root),
        crate::cli::OpsHelmTarget::Previous => simulation_previous_chart_path(&repo_root),
    };
    if !chart_path.exists() {
        return Err(format!(
            "missing upgrade target {}; current uses the working tree chart and previous uses artifacts/ops/chart-sources/previous/bijux-atlas.tgz",
            chart_path.display()
        ));
    }
    let before_manifest = bijux_atlas_ops::lifecycle::release::commands::helm_release_manifest(
        &process, &repo_root, &namespace,
    )?;
    let before_revision = bijux_atlas_ops::lifecycle::release::observation::deployment_revision(
        &process, &repo_root, &namespace,
    );
    let helm_args = vec![
        "upgrade".to_string(),
        "bijux-atlas".to_string(),
        chart_path.display().to_string(),
        "--namespace".to_string(),
        namespace.clone(),
        "--values".to_string(),
        values_file.display().to_string(),
    ];
    let (helm_stdout, helm_event) = process
        .run_subprocess("helm", &helm_args, &repo_root)
        .map_err(|err| err.to_stable_message())?;
    let (wait_rows, wait_errors, wait_ms) =
        bijux_atlas_ops::kubernetes::workload_wait::run_readiness_wait(
            &process,
            &repo_root,
            &namespace,
            args.release.timeout_seconds,
        );
    let smoke_rows = if wait_errors.is_empty() {
        bijux_atlas_ops::kubernetes::service_probe::run_kubectl_service_smoke_checks(
            &repo_root, &namespace, 18080,
        )?
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
    let after_manifest = bijux_atlas_ops::lifecycle::release::commands::helm_release_manifest(
        &process, &repo_root, &namespace,
    )?;
    let after_revision = bijux_atlas_ops::lifecycle::release::observation::deployment_revision(
        &process, &repo_root, &namespace,
    );
    let diff_summary = bijux_atlas_ops::lifecycle::release::contracts::manifest_diff_summary(
        &before_manifest,
        &after_manifest,
    );
    let compatibility =
        bijux_atlas_ops::lifecycle::release::contracts::lifecycle_compatibility_checks(
            &before_manifest,
            &after_manifest,
        );
    let rollout_history = bijux_atlas_ops::lifecycle::release::observation::rollout_history(
        &process, &repo_root, &namespace,
    );
    let pods_restarted = bijux_atlas_ops::lifecycle::release::observation::pods_restart_count(
        &process, &repo_root, &namespace,
    );
    let baseline_elapsed_ms =
        bijux_atlas_ops::lifecycle::release::records::load_readiness_baseline(
            &repo_root, &profile,
        )?;
    let readiness_threshold_percent = 125u64;
    let regression_ok = baseline_elapsed_ms
        .map(|baseline| {
            wait_ms.saturating_mul(100)
                <= baseline.saturating_mul(u128::from(readiness_threshold_percent))
        })
        .unwrap_or(true);
    let errors = wait_errors
        .iter()
        .cloned()
        .chain(smoke_errors.iter().cloned())
        .chain(
            compatibility["immutable_fields_safe"]
                .as_bool()
                .filter(|safe| !safe)
                .map(|_| "immutable field compatibility check failed".to_string()),
        )
        .chain((!regression_ok).then_some(format!(
            "readiness regression exceeded {}% of baseline",
            readiness_threshold_percent
        )))
        .collect::<Vec<_>>();
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let smoke_payload = serde_json::json!({
        "schema_version": 1,
        "cluster": "kind",
        "namespace": namespace,
        "status": if wait_errors.is_empty() && smoke_errors.is_empty() { "ok" } else { "failed" },
        "checks": smoke_rows
    });
    let smoke_report_path = bijux_atlas_ops::lifecycle::simulation::paths::write_simulation_report(
        &repo_root,
        run_id.as_str(),
        "ops-smoke.json",
        &smoke_payload,
    )?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "profile": profile,
        "cluster": "kind",
        "namespace": namespace,
        "status": status,
        "details": {
            "target": match args.to {
                crate::cli::OpsHelmTarget::Current => "current",
                crate::cli::OpsHelmTarget::Previous => "previous"
            },
            "helm": {
                "stdout": helm_stdout,
                "event": helm_event,
                "values_file": values_file.display().to_string(),
                "chart_path": chart_path.display().to_string(),
                "upgrade_target": "current-working-tree-chart"
            },
            "diff_summary": diff_summary,
            "compatibility_checks": compatibility,
            "configmap_restart_verified": {
                "before_revision": before_revision,
                "after_revision": after_revision,
                "status": if diff_summary["changed_lines"].as_u64().unwrap_or(0) == 0 {
                    "not-needed"
                } else if after_revision.unwrap_or_default() > before_revision.unwrap_or_default() {
                    "ok"
                } else {
                    "failed"
                }
            },
            "readiness_wait": {
                "elapsed_ms": wait_ms,
                "rows": wait_rows,
                "errors": wait_errors
            },
            "readiness_regression": {
                "threshold_percent": readiness_threshold_percent,
                "baseline_elapsed_ms": baseline_elapsed_ms,
                "current_elapsed_ms": wait_ms,
                "status": if regression_ok { "ok" } else { "failed" }
            },
            "rollout_history": rollout_history,
            "pods_restarted_count": pods_restarted,
            "smoke": {
                "report_path": smoke_report_path.display().to_string(),
                "checks": smoke_payload["checks"].clone()
            }
        }
    });
    let report_path = bijux_atlas_ops::lifecycle::simulation::paths::write_simulation_report(
        &repo_root,
        run_id.as_str(),
        "ops-upgrade.json",
        &payload,
    )?;
    let baseline_path = if errors.is_empty() {
        Some(
            bijux_atlas_ops::lifecycle::release::records::update_readiness_baseline(
                &repo_root, &profile, wait_ms,
            )?,
        )
    } else {
        None
    };
    let lifecycle_summary_path =
        bijux_atlas_ops::lifecycle::release::records::update_lifecycle_summary(
            &repo_root,
            run_id.as_str(),
            &profile,
            &namespace,
            bijux_atlas_ops::lifecycle::release::records::LifecycleSummaryUpdate {
                upgrade_report_path: Some(&report_path),
                upgrade_status: Some(status),
                rollback_report_path: None,
                rollback_status: None,
            },
        )?;
    let lifecycle_bundle = build_lifecycle_evidence_bundle(&repo_root, run_id.as_str())?;
    let envelope = serde_json::json!({
        "schema_version": 1,
        "text": if status == "ok" { "helm upgrade completed" } else { "helm upgrade failed" },
        "rows": [{
            "schema_version": 1,
            "profile": payload["profile"].clone(),
            "cluster": "kind",
            "namespace": payload["namespace"].clone(),
            "status": status,
            "report_path": report_path.display().to_string(),
            "summary_report_path": lifecycle_summary_path.display().to_string(),
            "baseline_history_path": baseline_path.map(|path| path.display().to_string()),
            "evidence_bundle": lifecycle_bundle,
            "details": payload["details"].clone()
        }],
        "summary": {"total": 1, "errors": errors.len(), "warnings": 0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, if errors.is_empty() { 0 } else { 1 }))
}

pub(crate) fn run_ops_helm_rollback(
    args: &crate::cli::OpsHelmRollbackArgs,
) -> Result<(String, i32), String> {
    let common = &args.release.common;
    match args.release.cluster {
        crate::cli::OpsClusterTarget::Kind => {}
    }
    if !common.allow_subprocess {
        return Err("helm rollback requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("helm rollback requires --allow-write".to_string());
    }
    if !common.allow_network {
        return Err("helm rollback requires --allow-network".to_string());
    }
    if !matches!(args.to, crate::cli::OpsHelmTarget::Previous) {
        return Err("helm rollback currently supports only --to previous".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let process = OpsProcess::new(true);
    ensure_simulation_context(&process, common.force)?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let profile = common.profile.clone().unwrap_or_else(|| "kind".to_string());
    let namespace = bijux_atlas_ops::workspace::profiles::simulation_namespace(
        &profile,
        args.release.namespace.as_deref(),
    );
    let before_manifest = bijux_atlas_ops::lifecycle::release::commands::helm_release_manifest(
        &process, &repo_root, &namespace,
    )?;
    let before_revision = bijux_atlas_ops::lifecycle::release::observation::deployment_revision(
        &process, &repo_root, &namespace,
    );
    let revision = bijux_atlas_ops::lifecycle::release::commands::prior_release_revision(
        &process, &repo_root, &namespace,
    )?;
    let helm_args = vec![
        "rollback".to_string(),
        "bijux-atlas".to_string(),
        revision.clone(),
        "--namespace".to_string(),
        namespace.clone(),
    ];
    let (helm_stdout, helm_event) = process
        .run_subprocess("helm", &helm_args, &repo_root)
        .map_err(|err| err.to_stable_message())?;
    let (wait_rows, wait_errors, wait_ms) =
        bijux_atlas_ops::kubernetes::workload_wait::run_readiness_wait(
            &process,
            &repo_root,
            &namespace,
            args.release.timeout_seconds,
        );
    let smoke_rows = if wait_errors.is_empty() {
        bijux_atlas_ops::kubernetes::service_probe::run_kubectl_service_smoke_checks(
            &repo_root, &namespace, 18080,
        )?
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
    let after_manifest = bijux_atlas_ops::lifecycle::release::commands::helm_release_manifest(
        &process, &repo_root, &namespace,
    )?;
    let after_revision = bijux_atlas_ops::lifecycle::release::observation::deployment_revision(
        &process, &repo_root, &namespace,
    );
    let diff_summary = bijux_atlas_ops::lifecycle::release::contracts::manifest_diff_summary(
        &before_manifest,
        &after_manifest,
    );
    let compatibility =
        bijux_atlas_ops::lifecycle::release::contracts::lifecycle_compatibility_checks(
            &before_manifest,
            &after_manifest,
        );
    let rollout_history = bijux_atlas_ops::lifecycle::release::observation::rollout_history(
        &process, &repo_root, &namespace,
    );
    let pods_restarted = bijux_atlas_ops::lifecycle::release::observation::pods_restart_count(
        &process, &repo_root, &namespace,
    );
    let baseline_elapsed_ms =
        bijux_atlas_ops::lifecycle::release::records::load_readiness_baseline(
            &repo_root, &profile,
        )?;
    let readiness_threshold_percent = 125u64;
    let regression_ok = baseline_elapsed_ms
        .map(|baseline| {
            wait_ms.saturating_mul(100)
                <= baseline.saturating_mul(u128::from(readiness_threshold_percent))
        })
        .unwrap_or(true);
    let errors = wait_errors
        .iter()
        .cloned()
        .chain(smoke_errors.iter().cloned())
        .chain(
            compatibility["immutable_fields_safe"]
                .as_bool()
                .filter(|safe| !safe)
                .map(|_| "immutable field compatibility check failed".to_string()),
        )
        .chain((!regression_ok).then_some(format!(
            "readiness regression exceeded {}% of baseline",
            readiness_threshold_percent
        )))
        .collect::<Vec<_>>();
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let smoke_payload = serde_json::json!({
        "schema_version": 1,
        "cluster": "kind",
        "namespace": namespace,
        "status": if wait_errors.is_empty() && smoke_errors.is_empty() { "ok" } else { "failed" },
        "checks": smoke_rows
    });
    let smoke_report_path = bijux_atlas_ops::lifecycle::simulation::paths::write_simulation_report(
        &repo_root,
        run_id.as_str(),
        "ops-smoke.json",
        &smoke_payload,
    )?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "profile": profile,
        "cluster": "kind",
        "namespace": namespace,
        "status": status,
        "details": {
            "target": "previous",
            "helm": {
                "stdout": helm_stdout,
                "event": helm_event,
                "revision": revision
            },
            "diff_summary": diff_summary,
            "compatibility_checks": compatibility,
            "configmap_restart_verified": {
                "before_revision": before_revision,
                "after_revision": after_revision,
                "status": if diff_summary["changed_lines"].as_u64().unwrap_or(0) == 0 {
                    "not-needed"
                } else if after_revision.unwrap_or_default() >= before_revision.unwrap_or_default() {
                    "ok"
                } else {
                    "failed"
                }
            },
            "readiness_wait": {
                "elapsed_ms": wait_ms,
                "rows": wait_rows,
                "errors": wait_errors
            },
            "readiness_regression": {
                "threshold_percent": readiness_threshold_percent,
                "baseline_elapsed_ms": baseline_elapsed_ms,
                "current_elapsed_ms": wait_ms,
                "status": if regression_ok { "ok" } else { "failed" }
            },
            "rollout_history": rollout_history,
            "pods_restarted_count": pods_restarted,
            "service_healthy_after_rollback": wait_errors.is_empty() && smoke_errors.is_empty(),
            "smoke": {
                "report_path": smoke_report_path.display().to_string(),
                "checks": smoke_payload["checks"].clone()
            }
        }
    });
    let report_path = bijux_atlas_ops::lifecycle::simulation::paths::write_simulation_report(
        &repo_root,
        run_id.as_str(),
        "ops-rollback.json",
        &payload,
    )?;
    let lifecycle_summary_path =
        bijux_atlas_ops::lifecycle::release::records::update_lifecycle_summary(
            &repo_root,
            run_id.as_str(),
            &profile,
            &namespace,
            bijux_atlas_ops::lifecycle::release::records::LifecycleSummaryUpdate {
                upgrade_report_path: None,
                upgrade_status: None,
                rollback_report_path: Some(&report_path),
                rollback_status: Some(status),
            },
        )?;
    let baseline_path = if errors.is_empty() {
        Some(
            bijux_atlas_ops::lifecycle::release::records::update_readiness_baseline(
                &repo_root, &profile, wait_ms,
            )?,
        )
    } else {
        None
    };
    let lifecycle_bundle = build_lifecycle_evidence_bundle(&repo_root, run_id.as_str())?;
    let envelope = serde_json::json!({
        "schema_version": 1,
        "text": if status == "ok" { "helm rollback completed" } else { "helm rollback failed" },
        "rows": [{
            "schema_version": 1,
            "profile": payload["profile"].clone(),
            "cluster": "kind",
            "namespace": payload["namespace"].clone(),
            "status": status,
            "report_path": report_path.display().to_string(),
            "summary_report_path": lifecycle_summary_path.display().to_string(),
            "baseline_history_path": baseline_path.map(|path| path.display().to_string()),
            "evidence_bundle": lifecycle_bundle,
            "details": payload["details"].clone()
        }],
        "summary": {"total": 1, "errors": errors.len(), "warnings": 0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, if errors.is_empty() { 0 } else { 1 }))
}

pub(crate) fn run_ops_smoke(args: &crate::cli::OpsSmokeArgs) -> Result<(String, i32), String> {
    let common = &args.common;
    match args.cluster {
        crate::cli::OpsClusterTarget::Kind => {}
    }
    if !common.allow_subprocess {
        return Err("k8s conformance requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("smoke requires --allow-write".to_string());
    }
    if !common.allow_network {
        return Err("smoke requires --allow-network".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let process = OpsProcess::new(true);
    ensure_simulation_context(&process, common.force)?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let profile = common.profile.clone().unwrap_or_else(|| "kind".to_string());
    let namespace = bijux_atlas_ops::workspace::profiles::simulation_namespace(
        &profile,
        args.namespace.as_deref(),
    );
    let (envelope, exit_code) = bijux_atlas_ops::lifecycle::simulation::smoke_command_payload(
        &repo_root,
        run_id.as_str(),
        &namespace,
        args.local_port,
    )?;
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, exit_code))
}

fn run_collect_command(
    args: &crate::cli::OpsCollectArgs,
    action: impl FnOnce(&OpsProcess, &std::path::Path, &str, &str) -> Result<serde_json::Value, String>,
) -> Result<(String, i32), String> {
    let common = &args.common;
    match args.cluster {
        crate::cli::OpsClusterTarget::Kind => {}
    }
    if !common.allow_subprocess {
        return Err(format!("{category} collect requires --allow-subprocess"));
    }
    if !common.allow_write {
        return Err(format!("{category} collect requires --allow-write"));
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let process = OpsProcess::new(true);
    ensure_simulation_context(&process, common.force)?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let profile = common.profile.clone().unwrap_or_else(|| "kind".to_string());
    let namespace = bijux_atlas_ops::workspace::profiles::simulation_namespace(
        &profile,
        args.namespace.as_deref(),
    );
    let envelope = action(&process, &repo_root, run_id.as_str(), &namespace)?;
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, 0))
}

pub(crate) fn run_ops_logs_collect(
    args: &crate::cli::OpsCollectArgs,
) -> Result<(String, i32), String> {
    run_collect_command(args, |process, repo_root, run_id, namespace| {
        bijux_atlas_ops::lifecycle::simulation::logs_collect_payload(
            process, repo_root, run_id, namespace,
        )
    })
}

pub(crate) fn run_ops_describe_collect(
    args: &crate::cli::OpsCollectArgs,
) -> Result<(String, i32), String> {
    run_collect_command(args, |process, repo_root, run_id, namespace| {
        bijux_atlas_ops::lifecycle::simulation::describe_collect_payload(
            process, repo_root, run_id, namespace,
        )
    })
}

pub(crate) fn run_ops_events_collect(
    args: &crate::cli::OpsCollectArgs,
) -> Result<(String, i32), String> {
    run_collect_command(args, |process, repo_root, run_id, namespace| {
        bijux_atlas_ops::lifecycle::simulation::events_collect_payload(
            process, repo_root, run_id, namespace,
        )
    })
}

pub(crate) fn run_ops_resources_snapshot(
    args: &crate::cli::OpsCollectArgs,
) -> Result<(String, i32), String> {
    run_collect_command(args, |process, repo_root, run_id, namespace| {
        bijux_atlas_ops::lifecycle::simulation::resources_snapshot_payload(
            process, repo_root, run_id, namespace,
        )
    })
}

pub(crate) fn run_ops_install(args: &cli::OpsInstallArgs) -> Result<(String, i32), String> {
    let common = &args.common;
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let process = OpsProcess::new(common.allow_subprocess);
    let (envelope, exit_code) = bijux_atlas_ops::lifecycle::simulation::stack_install_payload(
        &process,
        &repo_root,
        bijux_atlas_ops::lifecycle::simulation::StackInstallRequest {
            ops_root: common.ops_root.clone(),
            requested_profile: common.profile.clone(),
            run_id: run_id.as_str(),
            evidence_mode: common.evidence,
            plan_mode: args.plan,
            dry_run_mode: &args.dry_run,
            enable_kind: args.kind,
            enable_apply: args.apply,
            allow_subprocess: common.allow_subprocess,
            allow_write: common.allow_write,
            allow_network: common.allow_network,
            force: common.force,
        },
    )
    .map_err(OpsCommandError::Effect)
    .map_err(|err| err.to_stable_message())?;
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, exit_code))
}
