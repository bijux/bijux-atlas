// SPDX-License-Identifier: Apache-2.0
//! Simulation drill references and per-profile run summary helpers.

use super::simulation_report_path;
use crate::RunId;

pub(super) fn drill_check_paths(
    repo_root: &std::path::Path,
    drill: &str,
) -> Vec<(&'static str, std::path::PathBuf)> {
    match drill {
        "warmup-pod-restart" => vec![
            (
                "warmup lock doc",
                repo_root.join("docs/bijux-atlas-ops/kubernetes/runtime-configuration.md"),
            ),
            (
                "warmup lock metric contract",
                repo_root.join("configs/schemas/contracts/observability/metrics.schema.json"),
            ),
            (
                "warmup lock runtime source",
                crate::reference::workspace_layout::atlas_server_binary_source(repo_root),
            ),
        ],
        "redis-outage" => vec![
            (
                "network policy guide",
                repo_root.join("docs/bijux-atlas-ops/kubernetes/security-operations.md"),
            ),
            (
                "error registry",
                repo_root.join("configs/sources/operations/observability/error-codes.json"),
            ),
            (
                "drills guide",
                repo_root.join("docs/bijux-atlas-ops/observability/incident-response.md"),
            ),
        ],
        "offline-network-deny" | "offline-prewarm-serve" => vec![
            (
                "offline profile",
                repo_root.join("ops/k8s/values/offline.yaml"),
            ),
            (
                "network policy examples",
                repo_root.join("ops/k8s/values/networkpolicy-examples.yaml"),
            ),
            (
                "health endpoints contract",
                repo_root.join("docs/bijux-atlas/contracts/operational-contracts.md"),
            ),
        ],
        "catalog-unreachable" => vec![
            (
                "readiness handler",
                crate::reference::workspace_layout::atlas_http_route_support_source(repo_root),
            ),
            (
                "health endpoints contract",
                repo_root.join("docs/bijux-atlas/contracts/operational-contracts.md"),
            ),
            (
                "error registry",
                repo_root.join("configs/sources/operations/observability/error-codes.json"),
            ),
        ],
        "store-unreachable" => vec![
            (
                "alert rules",
                repo_root.join("ops/observe/alerts/atlas-alert-rules.yaml"),
            ),
            (
                "release evidence guide",
                repo_root.join("docs/bijux-atlas/contracts/operational-contracts.md"),
            ),
            (
                "error registry",
                repo_root.join("configs/sources/operations/observability/error-codes.json"),
            ),
        ],
        "rollout-failure-recovery" => vec![
            (
                "upgrade guide",
                repo_root.join("docs/bijux-atlas-ops/release/upgrades-and-rollback.md"),
            ),
            (
                "rollback schema",
                repo_root.join("ops/schema/k8s/ops-rollback.schema.json"),
            ),
            (
                "lifecycle contract",
                repo_root.join("docs/bijux-atlas/contracts/operational-contracts.md"),
            ),
        ],
        "invalid-config-rejected" => vec![
            (
                "environment allowlist",
                repo_root.join("configs/schemas/contracts/env.schema.json"),
            ),
            (
                "server config tests",
                crate::reference::workspace_layout::atlas_runtime_config_tests_source(repo_root),
            ),
            (
                "log schema",
                repo_root.join("configs/schemas/contracts/observability/log.schema.json"),
            ),
        ],
        _ => Vec::new(),
    }
}

pub(super) struct SimulationSummaryUpdate<'a> {
    pub(super) install_report_path: Option<&'a std::path::Path>,
    pub(super) install_status: Option<&'a str>,
    pub(super) smoke_report_path: Option<&'a std::path::Path>,
    pub(super) smoke_status: Option<&'a str>,
    pub(super) cleanup_report_path: Option<&'a std::path::Path>,
    pub(super) cleanup_status: Option<&'a str>,
}

pub(super) fn update_simulation_summary(
    repo_root: &std::path::Path,
    run_id: &RunId,
    profile: &str,
    namespace: &str,
    update: SimulationSummaryUpdate<'_>,
) -> Result<std::path::PathBuf, String> {
    let SimulationSummaryUpdate {
        install_report_path,
        install_status,
        smoke_report_path,
        smoke_status,
        cleanup_report_path,
        cleanup_status,
    } = update;
    let summary_path = simulation_report_path(repo_root, run_id, "ops-simulation-summary.json")?;
    let mut payload = if summary_path.exists() {
        serde_json::from_str::<serde_json::Value>(
            &std::fs::read_to_string(&summary_path)
                .map_err(|err| format!("failed to read {}: {err}", summary_path.display()))?,
        )
        .map_err(|err| format!("failed to parse {}: {err}", summary_path.display()))?
    } else {
        serde_json::json!({
            "schema_version": 1,
            "cluster": "kind",
            "profiles": []
        })
    };
    if !payload["profiles"].is_array() {
        payload["profiles"] = serde_json::json!([]);
    }
    let rows = payload["profiles"]
        .as_array_mut()
        .ok_or_else(|| "ops-simulation-summary.json profiles must be an array".to_string())?;
    if let Some(existing) = rows
        .iter_mut()
        .find(|row| row.get("profile").and_then(|v| v.as_str()) == Some(profile))
    {
        existing["namespace"] = serde_json::json!(namespace);
        if let Some(path) = install_report_path {
            existing["install_report_path"] = serde_json::json!(path.display().to_string());
        }
        if let Some(status) = install_status {
            existing["install_status"] = serde_json::json!(status);
        }
        if let Some(path) = smoke_report_path {
            existing["smoke_report_path"] = serde_json::json!(path.display().to_string());
        }
        if let Some(status) = smoke_status {
            existing["smoke_status"] = serde_json::json!(status);
        }
        if let Some(path) = cleanup_report_path {
            existing["cleanup_report_path"] = serde_json::json!(path.display().to_string());
        }
        if let Some(status) = cleanup_status {
            existing["cleanup_status"] = serde_json::json!(status);
        }
    } else {
        let mut row = serde_json::json!({
            "profile": profile,
            "namespace": namespace
        });
        if let Some(path) = install_report_path {
            row["install_report_path"] = serde_json::json!(path.display().to_string());
        }
        if let Some(status) = install_status {
            row["install_status"] = serde_json::json!(status);
        }
        if let Some(path) = smoke_report_path {
            row["smoke_report_path"] = serde_json::json!(path.display().to_string());
        }
        if let Some(status) = smoke_status {
            row["smoke_status"] = serde_json::json!(status);
        }
        if let Some(path) = cleanup_report_path {
            row["cleanup_report_path"] = serde_json::json!(path.display().to_string());
        }
        if let Some(status) = cleanup_status {
            row["cleanup_status"] = serde_json::json!(status);
        }
        rows.push(row);
    }
    rows.sort_by(|a, b| {
        a.get("profile")
            .and_then(serde_json::Value::as_str)
            .cmp(&b.get("profile").and_then(serde_json::Value::as_str))
    });
    std::fs::write(
        &summary_path,
        serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("failed to write {}: {err}", summary_path.display()))?;
    Ok(summary_path)
}
