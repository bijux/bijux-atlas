// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

use crate::reference::workspace_surfaces::{
    atlas_http_route_support_source, atlas_runtime_config_tests_source, atlas_server_binary_source,
};

use super::simulation_paths::simulation_report_path;

#[must_use]
pub fn drill_check_paths(repo_root: &Path, drill: &str) -> Vec<(&'static str, PathBuf)> {
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
                atlas_server_binary_source(repo_root),
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
                atlas_http_route_support_source(repo_root),
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
                atlas_runtime_config_tests_source(repo_root),
            ),
            (
                "log schema",
                repo_root.join("configs/schemas/contracts/observability/log.schema.json"),
            ),
        ],
        _ => Vec::new(),
    }
}

pub struct SimulationSummaryUpdate<'a> {
    pub install_report_path: Option<&'a Path>,
    pub install_status: Option<&'a str>,
    pub smoke_report_path: Option<&'a Path>,
    pub smoke_status: Option<&'a str>,
    pub cleanup_report_path: Option<&'a Path>,
    pub cleanup_status: Option<&'a str>,
}

pub fn update_simulation_summary(
    repo_root: &Path,
    run_id: &str,
    profile: &str,
    namespace: &str,
    update: SimulationSummaryUpdate<'_>,
) -> Result<PathBuf, String> {
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

#[cfg(test)]
mod tests {
    use super::{drill_check_paths, update_simulation_summary, SimulationSummaryUpdate};
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .expect("workspace root")
            .to_path_buf()
    }

    #[test]
    fn drill_source_paths_exist_for_current_workspace_layout() {
        let root = repo_root();
        for drill in ["catalog-unreachable", "invalid-config-rejected"] {
            for (_, path) in drill_check_paths(&root, drill) {
                assert!(
                    path.exists(),
                    "missing drill source path: {}",
                    path.display()
                );
            }
        }
    }

    #[test]
    fn simulation_summary_rows_remain_sorted_by_profile() {
        let root = tempfile::tempdir().expect("tempdir");

        update_simulation_summary(
            root.path(),
            "atlas-run",
            "zeta",
            "atlas-zeta",
            SimulationSummaryUpdate {
                install_report_path: None,
                install_status: Some("ok"),
                smoke_report_path: None,
                smoke_status: None,
                cleanup_report_path: None,
                cleanup_status: None,
            },
        )
        .expect("write zeta summary");
        let path = update_simulation_summary(
            root.path(),
            "atlas-run",
            "alpha",
            "atlas-alpha",
            SimulationSummaryUpdate {
                install_report_path: None,
                install_status: Some("ok"),
                smoke_report_path: None,
                smoke_status: None,
                cleanup_report_path: None,
                cleanup_status: None,
            },
        )
        .expect("write alpha summary");

        let payload: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).expect("read simulation summary"))
                .expect("parse simulation summary");
        let profiles = payload["profiles"]
            .as_array()
            .expect("profiles array")
            .iter()
            .map(|row| row["profile"].as_str().unwrap_or_default())
            .collect::<Vec<_>>();

        assert_eq!(profiles, vec!["alpha", "zeta"]);
    }
}
