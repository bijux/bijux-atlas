// SPDX-License-Identifier: Apache-2.0
//! Kind and simulation-cluster operations for install-status flows.

use super::*;
use crate::cli::OpsCommonArgs;
use crate::ops_commands::{emit_payload, run_id_or_default};
use crate::{resolve_repo_root, OpsProcess};

pub(crate) fn run_ops_observe_slo_list(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let payload = bijux_atlas_ops::observe::commands::render_slo_list_payload(&repo_root)?;
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_ops_observe_slo_verify(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    if !common.allow_write {
        return Err("observe slo verify requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let (payload, exit_code) =
        bijux_atlas_ops::observe::commands::verify_slo_contract(&repo_root, run_id.as_str())?;
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, exit_code))
}

pub(crate) fn run_ops_observe_alerts_verify(
    common: &OpsCommonArgs,
) -> Result<(String, i32), String> {
    if !common.allow_write {
        return Err("observe alerts verify requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let (payload, exit_code) =
        bijux_atlas_ops::observe::commands::verify_alert_contract(&repo_root, run_id.as_str())?;
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, exit_code))
}

pub(crate) fn run_ops_observe_runbooks_verify(
    common: &OpsCommonArgs,
) -> Result<(String, i32), String> {
    if !common.allow_write {
        return Err("observe runbooks verify requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let (payload, exit_code) =
        bijux_atlas_ops::observe::commands::verify_runbook_contract(&repo_root, run_id.as_str())?;
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, exit_code))
}

pub(crate) fn run_ops_observe_readiness(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    if !common.allow_write {
        return Err("observe readiness requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let (payload, exit_code) =
        bijux_atlas_ops::observe::commands::build_operational_readiness_payload(
            &repo_root,
            run_id.as_str(),
        )?;
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, exit_code))
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
    let namespace = bijux_atlas_ops::workspace::profiles::simulation_namespace(&profile, None);
    let (payload, exit_code) = bijux_atlas_ops::observe::commands::verify_observability_runtime(
        &repo_root,
        run_id.as_str(),
        &profile,
        &namespace,
    )?;
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, exit_code))
}

pub(crate) fn run_ops_drill(args: &crate::cli::OpsDrillRunArgs) -> Result<(String, i32), String> {
    let common = &args.common;
    if !common.allow_write {
        return Err("drills run requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let drills = bijux_atlas_ops::lifecycle::simulation::records::load_drill_registry(&repo_root)?;
    let drill = drills
        .iter()
        .find(|row| row.get("name").and_then(serde_json::Value::as_str) == Some(args.name.as_str()))
        .cloned()
        .ok_or_else(|| format!("unknown drill `{}`", args.name))?;
    let mut checks = Vec::new();
    for (name, path) in
        bijux_atlas_ops::lifecycle::simulation::records::drill_check_paths(&repo_root, &args.name)
    {
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
    let evidence_paths =
        bijux_atlas_ops::lifecycle::simulation::records::drill_check_paths(&repo_root, &args.name)
            .into_iter()
            .map(|(_, path)| {
                path.strip_prefix(&repo_root)
                    .unwrap_or(&path)
                    .display()
                    .to_string()
            })
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
    let report_path = bijux_atlas_ops::lifecycle::simulation::paths::write_simulation_report(
        &repo_root,
        run_id.as_str(),
        &format!("ops-drill-{}.json", args.name),
        &payload,
    )?;
    let summary_path = bijux_atlas_ops::lifecycle::simulation::records::update_drill_summary(
        &repo_root,
        run_id.as_str(),
        &args.name,
        &report_path,
        status,
    )?;
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
                (
                    "ok",
                    serde_json::json!({"detail": "cluster already exists"}),
                )
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
    let report_path = bijux_atlas_ops::lifecycle::simulation::paths::write_simulation_report(
        &repo_root,
        run_id.as_str(),
        "ops-kind.json",
        &payload,
    )?;
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
        Err(err) => (
            "failed",
            serde_json::json!({"error": err.to_stable_message()}),
        ),
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
    let report_path = bijux_atlas_ops::lifecycle::simulation::paths::write_simulation_report(
        &repo_root,
        run_id.as_str(),
        "ops-kind.json",
        &payload,
    )?;
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
        Err(err) => (
            "failed",
            serde_json::json!({"error": err.to_stable_message()}),
        ),
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "cluster": "kind",
        "action": "status",
        "status": status,
        "details": details
    });
    let report_path = bijux_atlas_ops::lifecycle::simulation::paths::write_simulation_report(
        &repo_root,
        run_id.as_str(),
        "ops-kind.json",
        &payload,
    )?;
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
        Err(err) => (
            "failed",
            serde_json::json!({"error": err.to_stable_message()}),
        ),
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
    let report_path = bijux_atlas_ops::lifecycle::simulation::paths::write_simulation_report(
        &repo_root,
        run_id.as_str(),
        "ops-kind.json",
        &payload,
    )?;
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
    let profile = common.profile.clone().unwrap_or_else(|| "kind".to_string());
    let namespace = bijux_atlas_ops::workspace::profiles::simulation_namespace(
        &profile,
        args.release.namespace.as_deref(),
    );
    let values_file =
        bijux_atlas_ops::workspace::profiles::resolve_profile_values_file(&repo_root, &profile)
            .map_err(|err| err.detail())?;
    let chart_source = match args.chart_source {
        crate::cli::OpsHelmChartSource::Current => {
            bijux_atlas_ops::lifecycle::release::contracts::ReleaseChartSource::Current
        }
        crate::cli::OpsHelmChartSource::Previous => {
            bijux_atlas_ops::lifecycle::release::contracts::ReleaseChartSource::Previous
        }
    };
    let chart_path = bijux_atlas_ops::lifecycle::release::contracts::release_chart_source_path(
        &repo_root,
        chart_source,
    )?;
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
    let smoke_report_path = bijux_atlas_ops::lifecycle::simulation::paths::write_simulation_report(
        &repo_root,
        run_id.as_str(),
        "ops-smoke.json",
        &smoke_payload,
    )?;
    let render_path = install_render_path(&repo_root, run_id.as_str(), &profile);
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
            "kubeconform": record_kubeconform_result(&process, &repo_root, &render_path),
            "configmap_env_keys": extract_configmap_env_keys(&repo_root, run_id.as_str(), &profile)?,
            "runtime_allowlist": bijux_atlas_ops::lifecycle::install_status::runtime_env_allowlist_status(&repo_root),
            "smoke": {
                "report_path": smoke_report_path.display().to_string(),
                "checks": smoke_payload["checks"].clone()
            },
            "profile_intent": load_profile_intent(&repo_root, &profile)?,
            "profile_metadata": bijux_atlas_ops::workspace::profiles::load_profile_values_entry(&repo_root, &profile)
                .map_err(|err| err.detail())?
        }
    });
    let report_path = bijux_atlas_ops::lifecycle::simulation::paths::write_simulation_report(
        &repo_root,
        run_id.as_str(),
        "ops-install.json",
        &payload,
    )?;
    let summary_path = bijux_atlas_ops::lifecycle::simulation::records::update_simulation_summary(
        &repo_root,
        run_id.as_str(),
        &profile,
        &namespace,
        bijux_atlas_ops::lifecycle::simulation::records::SimulationSummaryUpdate {
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
    let profile = common.profile.clone().unwrap_or_else(|| "kind".to_string());
    let namespace = bijux_atlas_ops::workspace::profiles::simulation_namespace(
        &profile,
        args.namespace.as_deref(),
    );
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
        bijux_atlas_ops::lifecycle::simulation::paths::write_simulation_report(
            &repo_root,
            run_id.as_str(),
            "ops-cleanup.json",
            &cleanup_payload,
        )?;
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
    let report_path = bijux_atlas_ops::lifecycle::simulation::paths::write_simulation_report(
        &repo_root,
        run_id.as_str(),
        "ops-uninstall.json",
        &payload,
    )?;
    let summary_path = bijux_atlas_ops::lifecycle::simulation::records::update_simulation_summary(
        &repo_root,
        run_id.as_str(),
        &profile,
        &namespace,
        bijux_atlas_ops::lifecycle::simulation::records::SimulationSummaryUpdate {
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
