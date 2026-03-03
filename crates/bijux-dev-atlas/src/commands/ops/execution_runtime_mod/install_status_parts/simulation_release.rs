// SPDX-License-Identifier: Apache-2.0
//! Release lifecycle simulation commands for install-status flows.

use super::*;

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
    let profile = common
        .profile
        .clone()
        .unwrap_or_else(|| "kind".to_string());
    let namespace = simulation_namespace(&profile, args.release.namespace.as_deref());
    let values_file = resolve_profile_values_file(&repo_root, &profile)?;
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
    let before_manifest = helm_release_manifest(&process, &repo_root, &namespace)?;
    let before_revision = deployment_revision(&process, &repo_root, &namespace);
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
    let after_manifest = helm_release_manifest(&process, &repo_root, &namespace)?;
    let after_revision = deployment_revision(&process, &repo_root, &namespace);
    let diff_summary = manifest_diff_summary(&before_manifest, &after_manifest);
    let compatibility = lifecycle_compatibility_checks(&before_manifest, &after_manifest);
    let rollout_history = rollout_history(&process, &repo_root, &namespace);
    let pods_restarted = pods_restart_count(&process, &repo_root, &namespace);
    let baseline_elapsed_ms = load_readiness_baseline(&repo_root, &profile)?;
    let readiness_threshold_percent = 125u64;
    let regression_ok = baseline_elapsed_ms
        .map(|baseline| wait_ms.saturating_mul(100) <= baseline.saturating_mul(u128::from(readiness_threshold_percent)))
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
    let smoke_report_path =
        write_simulation_report(&repo_root, &run_id, "ops-smoke.json", &smoke_payload)?;
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
    let report_path = write_simulation_report(&repo_root, &run_id, "ops-upgrade.json", &payload)?;
    let baseline_path = if errors.is_empty() {
        Some(update_readiness_baseline(&repo_root, &profile, wait_ms)?)
    } else {
        None
    };
    let lifecycle_summary_path = update_lifecycle_summary(
        &repo_root,
        &run_id,
        &profile,
        &namespace,
        LifecycleSummaryUpdate {
            upgrade_report_path: Some(&report_path),
            upgrade_status: Some(status),
            rollback_report_path: None,
            rollback_status: None,
        },
    )?;
    let lifecycle_bundle = build_lifecycle_evidence_bundle(&repo_root, &run_id)?;
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
    let profile = common
        .profile
        .clone()
        .unwrap_or_else(|| "kind".to_string());
    let namespace = simulation_namespace(&profile, args.release.namespace.as_deref());
    let before_manifest = helm_release_manifest(&process, &repo_root, &namespace)?;
    let before_revision = deployment_revision(&process, &repo_root, &namespace);
    let revision = prior_release_revision(&process, &repo_root, &namespace)?;
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
    let after_manifest = helm_release_manifest(&process, &repo_root, &namespace)?;
    let after_revision = deployment_revision(&process, &repo_root, &namespace);
    let diff_summary = manifest_diff_summary(&before_manifest, &after_manifest);
    let compatibility = lifecycle_compatibility_checks(&before_manifest, &after_manifest);
    let rollout_history = rollout_history(&process, &repo_root, &namespace);
    let pods_restarted = pods_restart_count(&process, &repo_root, &namespace);
    let baseline_elapsed_ms = load_readiness_baseline(&repo_root, &profile)?;
    let readiness_threshold_percent = 125u64;
    let regression_ok = baseline_elapsed_ms
        .map(|baseline| wait_ms.saturating_mul(100) <= baseline.saturating_mul(u128::from(readiness_threshold_percent)))
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
    let smoke_report_path =
        write_simulation_report(&repo_root, &run_id, "ops-smoke.json", &smoke_payload)?;
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
    let report_path = write_simulation_report(&repo_root, &run_id, "ops-rollback.json", &payload)?;
    let lifecycle_summary_path = update_lifecycle_summary(
        &repo_root,
        &run_id,
        &profile,
        &namespace,
        LifecycleSummaryUpdate {
            upgrade_report_path: None,
            upgrade_status: None,
            rollback_report_path: Some(&report_path),
            rollback_status: Some(status),
        },
    )?;
    let baseline_path = if errors.is_empty() {
        Some(update_readiness_baseline(&repo_root, &profile, wait_ms)?)
    } else {
        None
    };
    let lifecycle_bundle = build_lifecycle_evidence_bundle(&repo_root, &run_id)?;
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
    let profile = common
        .profile
        .clone()
        .unwrap_or_else(|| "kind".to_string());
    let namespace = simulation_namespace(&profile, args.namespace.as_deref());
    let checks = run_smoke_checks(&repo_root, &namespace, args.local_port)?;
    let errors = checks
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
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "cluster": "kind",
        "namespace": namespace,
        "status": status,
        "checks": checks
    });
    let report_path = write_simulation_report(&repo_root, &run_id, "ops-smoke.json", &payload)?;
    let envelope = serde_json::json!({
        "schema_version": 1,
        "text": if status == "ok" { "smoke checks passed" } else { "smoke checks failed" },
        "rows": [{
            "schema_version": 1,
            "cluster": "kind",
            "namespace": payload["namespace"].clone(),
            "status": status,
            "checks": payload["checks"].clone(),
            "report_path": report_path.display().to_string()
        }],
        "summary": {"total": 1, "errors": errors.len(), "warnings": 0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, if errors.is_empty() { 0 } else { 1 }))
}

fn run_collect_command(
    args: &crate::cli::OpsCollectArgs,
    category: &str,
    file_name: &str,
    argv: Vec<String>,
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
    let profile = common
        .profile
        .clone()
        .unwrap_or_else(|| "kind".to_string());
    let namespace = simulation_namespace(&profile, args.namespace.as_deref());
    let (stdout, event) = process
        .run_subprocess("kubectl", &argv, &repo_root)
        .map_err(|err| err.to_stable_message())?;
    let artifact_path = write_debug_artifact(&repo_root, &run_id, &namespace, file_name, &stdout)?;
    let report_path = emit_debug_bundle_report(
        &repo_root,
        &run_id,
        &namespace,
        category,
        std::slice::from_ref(&artifact_path),
    )?;
    let envelope = serde_json::json!({
        "schema_version": 1,
        "text": format!("{category} collected"),
        "rows": [{
            "schema_version": 1,
            "cluster": "kind",
            "namespace": namespace,
            "category": category,
            "status": "ok",
            "files": [artifact_path.display().to_string()],
            "report_path": report_path.display().to_string(),
            "event": event
        }],
        "summary": {"total": 1, "errors": 0, "warnings": 0}
    });
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, 0))
}

pub(crate) fn run_ops_logs_collect(
    args: &crate::cli::OpsCollectArgs,
) -> Result<(String, i32), String> {
    let profile = args
        .common
        .profile
        .clone()
        .unwrap_or_else(|| "kind".to_string());
    let namespace = simulation_namespace(&profile, args.namespace.as_deref());
    run_collect_command(
        args,
        "logs",
        "pod-logs.txt",
        vec![
            "logs".to_string(),
            "-n".to_string(),
            namespace,
            "deployment/bijux-atlas".to_string(),
            "--tail=500".to_string(),
        ],
    )
}

pub(crate) fn run_ops_describe_collect(
    args: &crate::cli::OpsCollectArgs,
) -> Result<(String, i32), String> {
    let profile = args
        .common
        .profile
        .clone()
        .unwrap_or_else(|| "kind".to_string());
    let namespace = simulation_namespace(&profile, args.namespace.as_deref());
    run_collect_command(
        args,
        "describe",
        "describe.txt",
        vec![
            "describe".to_string(),
            "-n".to_string(),
            namespace,
            "deployment/bijux-atlas".to_string(),
            "service/bijux-atlas".to_string(),
        ],
    )
}

pub(crate) fn run_ops_events_collect(
    args: &crate::cli::OpsCollectArgs,
) -> Result<(String, i32), String> {
    let profile = args
        .common
        .profile
        .clone()
        .unwrap_or_else(|| "kind".to_string());
    let namespace = simulation_namespace(&profile, args.namespace.as_deref());
    run_collect_command(
        args,
        "events",
        "events.txt",
        vec![
            "get".to_string(),
            "events".to_string(),
            "-n".to_string(),
            namespace,
            "--sort-by=.metadata.creationTimestamp".to_string(),
        ],
    )
}

pub(crate) fn run_ops_resources_snapshot(
    args: &crate::cli::OpsCollectArgs,
) -> Result<(String, i32), String> {
    let profile = args
        .common
        .profile
        .clone()
        .unwrap_or_else(|| "kind".to_string());
    let namespace = simulation_namespace(&profile, args.namespace.as_deref());
    run_collect_command(
        args,
        "resources",
        "resources.yaml",
        vec![
            "get".to_string(),
            "all".to_string(),
            "-n".to_string(),
            namespace,
            "-o".to_string(),
            "yaml".to_string(),
        ],
    )
}

pub(crate) fn run_ops_install(args: &cli::OpsInstallArgs) -> Result<(String, i32), String> {
    let common = &args.common;
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    let profile =
        resolve_profile(common.profile.clone(), &profiles).map_err(|e| e.to_stable_message())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    if !args.plan && !common.allow_subprocess {
        return Err(OpsCommandError::Effect(
            "install execution requires --allow-subprocess".to_string(),
        )
        .to_stable_message());
    }
    if (args.apply || args.kind) && !common.allow_write {
        return Err(OpsCommandError::Effect(
            "install apply/kind requires --allow-write".to_string(),
        )
        .to_stable_message());
    }
    if (args.apply || args.kind) && !common.allow_network {
        return Err(OpsCommandError::Effect(
            "install apply/kind requires --allow-network".to_string(),
        )
        .to_stable_message());
    }

    let mut steps = Vec::new();
    let process = OpsProcess::new(common.allow_subprocess);
    if args.kind {
        steps.push("kind cluster ensure".to_string());
        if !args.plan {
            let kind_config = repo_root.join(&profile.cluster_config);
            let kind_args = vec![
                "create".to_string(),
                "cluster".to_string(),
                "--name".to_string(),
                profile.kind_profile.clone(),
                "--config".to_string(),
                kind_config.display().to_string(),
            ];
            if let Err(err) = process.run_subprocess("kind", &kind_args, &repo_root) {
                let stable = err.to_stable_message();
                if !stable.contains("already exists") {
                    return Err(stable);
                }
            }
        }
    }
    if args.apply {
        steps.push("kubectl apply".to_string());
        if !args.plan {
            ensure_kind_context(&process, &profile, common.force)
                .map_err(|e| e.to_stable_message())?;
            ensure_namespace_exists(&process, "bijux-atlas", &args.dry_run)
                .map_err(|e| e.to_stable_message())?;
            let render_path = repo_root
                .join("artifacts/ops")
                .join(run_id.as_str())
                .join(format!("render/{}/helm/render.yaml", profile.name));
            let mut apply_args = vec![
                "apply".to_string(),
                "-n".to_string(),
                "bijux-atlas".to_string(),
                "-f".to_string(),
                render_path.display().to_string(),
            ];
            if args.dry_run == "client" {
                apply_args.push("--dry-run=client".to_string());
            }
            let _ = process
                .run_subprocess("kubectl", &apply_args, &repo_root)
                .map_err(|e| e.to_stable_message())?;
        }
    }
    if !args.kind && !args.apply {
        steps.push("validate-only".to_string());
    }
    let render_path = install_render_path(&repo_root, run_id.as_str(), &profile.name);
    let render_inventory = if render_path.exists() {
        let rendered_manifest = std::fs::read_to_string(&render_path)
            .map_err(|err| format!("failed to read {}: {err}", render_path.display()))?;
        install_plan_inventory(&rendered_manifest)
    } else {
        serde_json::json!({
            "resources": [],
            "resource_kinds": {},
            "namespaces": [],
            "namespace_isolated": true,
            "has_crds": false,
            "has_rbac": false,
            "forbidden_objects": [],
            "missing_render_path": render_path.display().to_string(),
        })
    };
    let profile_intent = load_profile_intent(&repo_root, &profile.name)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "profile": profile.name,
        "run_id": run_id.as_str(),
        "plan_mode": args.plan,
        "dry_run": args.dry_run,
        "steps": steps,
        "kind_context_expected": expected_kind_context(&profile),
        "profile_intent": profile_intent,
        "install_plan": render_inventory,
    });
    let text = if args.plan {
        format!("install plan generated for profile `{}`", profile.name)
    } else {
        format!("install completed for profile `{}`", profile.name)
    };
    let envelope = serde_json::json!({"schema_version": 1, "text": text, "rows": [payload], "summary": {"total": 1, "errors": 0, "warnings": 0}});
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, 0))
}
