// SPDX-License-Identifier: Apache-2.0

use crate::kubernetes::execution::KubernetesCommandRunner;
use crate::kubernetes::schema_validation::record_kubeconform_result;
use crate::kubernetes::service_probe::run_kubectl_service_smoke_checks;
use crate::kubernetes::workload_wait::run_readiness_wait;
use crate::lifecycle::install_status::{
    extract_configmap_env_keys, install_render_path, load_profile_intent,
    runtime_env_allowlist_status,
};
use crate::lifecycle::release::contracts::{release_chart_source_path, ReleaseChartSource};
use crate::lifecycle::simulation::context::SimulationCommandRunner;
use crate::lifecycle::simulation::paths::write_simulation_report;
use crate::lifecycle::simulation::records::{update_simulation_summary, SimulationSummaryUpdate};
use crate::workspace::profiles::{load_profile_values_entry, resolve_profile_values_file};
use serde_json::Value;
use std::path::Path;

fn chart_source_name(chart_source: ReleaseChartSource) -> &'static str {
    match chart_source {
        ReleaseChartSource::Current => "current",
        ReleaseChartSource::Previous => "previous",
    }
}

pub fn helm_install_payload(
    runner: &(impl SimulationCommandRunner + KubernetesCommandRunner),
    repo_root: &Path,
    run_id: &str,
    profile: &str,
    namespace: &str,
    chart_source: ReleaseChartSource,
    timeout_seconds: u64,
) -> Result<(Value, i32), String> {
    let values_file =
        resolve_profile_values_file(repo_root, profile).map_err(|err| err.detail())?;
    let chart_path = release_chart_source_path(repo_root, chart_source)?;
    let helm_args = vec![
        "upgrade".to_string(),
        "--install".to_string(),
        "bijux-atlas".to_string(),
        chart_path.display().to_string(),
        "--namespace".to_string(),
        namespace.to_string(),
        "--create-namespace".to_string(),
        "--values".to_string(),
        values_file.display().to_string(),
    ];
    let (helm_stdout, helm_event) =
        SimulationCommandRunner::run(runner, "helm", &helm_args, repo_root)?;
    let (wait_rows, wait_errors, wait_ms) =
        run_readiness_wait(runner, repo_root, namespace, timeout_seconds);
    let smoke_rows = if wait_errors.is_empty() {
        run_kubectl_service_smoke_checks(repo_root, namespace, 18080)?
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
        write_simulation_report(repo_root, run_id, "ops-smoke.json", &smoke_payload)?;
    let render_path = install_render_path(repo_root, run_id, profile);
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
                "chart_source": chart_source_name(chart_source)
            },
            "readiness_wait": {
                "elapsed_ms": wait_ms,
                "rows": wait_rows,
                "errors": wait_errors
            },
            "kubeconform": record_kubeconform_result(runner, repo_root, &render_path),
            "configmap_env_keys": extract_configmap_env_keys(repo_root, run_id, profile)?,
            "runtime_allowlist": runtime_env_allowlist_status(repo_root),
            "smoke": {
                "report_path": smoke_report_path.display().to_string(),
                "checks": smoke_payload["checks"].clone()
            },
            "profile_intent": load_profile_intent(repo_root, profile)?,
            "profile_metadata": load_profile_values_entry(repo_root, profile).map_err(|err| err.detail())?
        }
    });
    let report_path = write_simulation_report(repo_root, run_id, "ops-install.json", &payload)?;
    let summary_path = update_simulation_summary(
        repo_root,
        run_id,
        profile,
        namespace,
        SimulationSummaryUpdate {
            install_report_path: Some(&report_path),
            install_status: Some(status),
            smoke_report_path: Some(&smoke_report_path),
            smoke_status: Some(smoke_payload["status"].as_str().unwrap_or("failed")),
            cleanup_report_path: None,
            cleanup_status: None,
        },
    )?;
    Ok((
        serde_json::json!({
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
        }),
        if errors.is_empty() { 0 } else { 1 },
    ))
}
