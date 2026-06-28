// SPDX-License-Identifier: Apache-2.0

use super::change_support::assess_release_health;
use crate::kubernetes::execution::KubernetesCommandRunner;
use crate::lifecycle::evidence::artifacts::build_lifecycle_evidence_bundle;
use crate::lifecycle::release::commands::{helm_release_manifest, ReleaseCommandRunner};
use crate::lifecycle::release::contracts::{
    lifecycle_compatibility_checks, manifest_diff_summary, ReleaseChartSource,
};
use crate::lifecycle::release::observation::{
    deployment_revision, pods_restart_count, rollout_history,
};
use crate::lifecycle::release::records::{
    update_lifecycle_summary, update_readiness_baseline, LifecycleSummaryUpdate,
};
use crate::lifecycle::simulation::context::SimulationCommandRunner;
use crate::lifecycle::simulation::paths::write_simulation_report;
use crate::workspace::profiles::resolve_profile_values_file;
use serde_json::Value;
use std::path::Path;

pub struct HelmUpgradeRequest<'a> {
    pub run_id: &'a str,
    pub profile: &'a str,
    pub namespace: &'a str,
    pub target: ReleaseChartSource,
    pub timeout_seconds: u64,
}

pub fn helm_upgrade_payload(
    runner: &(impl SimulationCommandRunner + KubernetesCommandRunner + ReleaseCommandRunner),
    repo_root: &Path,
    request: HelmUpgradeRequest<'_>,
) -> Result<(Value, i32), String> {
    let values_file =
        resolve_profile_values_file(repo_root, request.profile).map_err(|err| err.detail())?;
    let chart_path =
        crate::lifecycle::release::contracts::release_chart_source_path(repo_root, request.target)?;
    if !chart_path.exists() {
        return Err(format!(
            "missing upgrade target {}; current uses the working tree chart and previous uses artifacts/ops/chart-sources/previous/bijux-atlas.tgz",
            chart_path.display()
        ));
    }
    let before_manifest = helm_release_manifest(runner, repo_root, request.namespace)?;
    let before_revision = deployment_revision(runner, repo_root, request.namespace);
    let helm_args = vec![
        "upgrade".to_string(),
        "bijux-atlas".to_string(),
        chart_path.display().to_string(),
        "--namespace".to_string(),
        request.namespace.to_string(),
        "--values".to_string(),
        values_file.display().to_string(),
    ];
    let (helm_stdout, helm_event) =
        SimulationCommandRunner::run(runner, "helm", &helm_args, repo_root)?;
    let health = assess_release_health(
        runner,
        repo_root,
        request.run_id,
        request.profile,
        request.namespace,
        request.timeout_seconds,
    )?;
    let after_manifest = helm_release_manifest(runner, repo_root, request.namespace)?;
    let after_revision = deployment_revision(runner, repo_root, request.namespace);
    let diff_summary = manifest_diff_summary(&before_manifest, &after_manifest);
    let compatibility = lifecycle_compatibility_checks(&before_manifest, &after_manifest);
    let rollout_history = rollout_history(runner, repo_root, request.namespace);
    let pods_restarted = pods_restart_count(runner, repo_root, request.namespace);
    let mut errors = health.errors();
    if compatibility["immutable_fields_safe"].as_bool() == Some(false) {
        errors.push("immutable field compatibility check failed".to_string());
    }
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "profile": request.profile,
        "cluster": "kind",
        "namespace": request.namespace,
        "status": status,
        "details": {
            "target": match request.target {
                ReleaseChartSource::Current => "current",
                ReleaseChartSource::Previous => "previous",
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
                "elapsed_ms": health.wait_ms,
                "rows": health.wait_rows,
                "errors": health.wait_errors
            },
            "readiness_regression": {
                "threshold_percent": health.readiness_threshold_percent,
                "baseline_elapsed_ms": health.baseline_elapsed_ms,
                "current_elapsed_ms": health.wait_ms,
                "status": if health.regression_ok { "ok" } else { "failed" }
            },
            "rollout_history": rollout_history,
            "pods_restarted_count": pods_restarted,
            "smoke": {
                "report_path": health.smoke_report_path.display().to_string(),
                "checks": health.smoke_rows
            }
        }
    });
    let report_path =
        write_simulation_report(repo_root, request.run_id, "ops-upgrade.json", &payload)?;
    let baseline_path = if errors.is_empty() {
        Some(update_readiness_baseline(
            repo_root,
            request.profile,
            health.wait_ms,
        )?)
    } else {
        None
    };
    let lifecycle_summary_path = update_lifecycle_summary(
        repo_root,
        request.run_id,
        request.profile,
        request.namespace,
        LifecycleSummaryUpdate {
            upgrade_report_path: Some(&report_path),
            upgrade_status: Some(status),
            rollback_report_path: None,
            rollback_status: None,
        },
    )?;
    let lifecycle_bundle = build_lifecycle_evidence_bundle(repo_root, request.run_id)?;
    Ok((
        serde_json::json!({
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
        }),
        if errors.is_empty() { 0 } else { 1 },
    ))
}
