// SPDX-License-Identifier: Apache-2.0
//! Release rollout observation and readiness baseline record helpers.

use super::simulation_report_path;
use crate::{OpsProcess, RunId};

pub(super) fn readiness_baseline_path(
    repo_root: &std::path::Path,
) -> Result<std::path::PathBuf, String> {
    let path = repo_root.join("artifacts/ops/history/readiness-baselines.json");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    Ok(path)
}

pub(super) fn deployment_revision(
    process: &OpsProcess,
    repo_root: &std::path::Path,
    namespace: &str,
) -> Option<i64> {
    let argv = vec![
        "get".to_string(),
        "deployment".to_string(),
        "bijux-atlas".to_string(),
        "-n".to_string(),
        namespace.to_string(),
        "-o".to_string(),
        "json".to_string(),
    ];
    let (stdout, _) = process.run_subprocess("kubectl", &argv, repo_root).ok()?;
    let json: serde_json::Value = serde_json::from_str(&stdout).ok()?;
    json.get("metadata")
        .and_then(|row| row.get("annotations"))
        .and_then(|row| row.get("deployment.kubernetes.io/revision"))
        .and_then(serde_json::Value::as_str)
        .and_then(|value| value.parse::<i64>().ok())
}

pub(super) fn rollout_history(
    process: &OpsProcess,
    repo_root: &std::path::Path,
    namespace: &str,
) -> serde_json::Value {
    let argv = vec![
        "rollout".to_string(),
        "history".to_string(),
        "deployment/bijux-atlas".to_string(),
        "-n".to_string(),
        namespace.to_string(),
    ];
    match process.run_subprocess("kubectl", &argv, repo_root) {
        Ok((stdout, event)) => serde_json::json!({
            "status": "ok",
            "stdout": stdout,
            "event": event
        }),
        Err(err) => serde_json::json!({
            "status": "failed",
            "error": err.to_stable_message()
        }),
    }
}

pub(super) fn pods_restart_count(
    process: &OpsProcess,
    repo_root: &std::path::Path,
    namespace: &str,
) -> u64 {
    let argv = vec![
        "get".to_string(),
        "pods".to_string(),
        "-n".to_string(),
        namespace.to_string(),
        "-o".to_string(),
        "json".to_string(),
    ];
    let Ok((stdout, _)) = process.run_subprocess("kubectl", &argv, repo_root) else {
        return 0;
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) else {
        return 0;
    };
    json.get("items")
        .and_then(serde_json::Value::as_array)
        .map(|rows| {
            rows.iter()
                .flat_map(|row| {
                    row.get("status")
                        .and_then(|status| status.get("containerStatuses"))
                        .and_then(serde_json::Value::as_array)
                        .cloned()
                        .unwrap_or_default()
                })
                .filter_map(|container| {
                    container
                        .get("restartCount")
                        .and_then(serde_json::Value::as_u64)
                })
                .sum()
        })
        .unwrap_or(0)
}

pub(super) struct LifecycleSummaryUpdate<'a> {
    pub(super) upgrade_report_path: Option<&'a std::path::Path>,
    pub(super) upgrade_status: Option<&'a str>,
    pub(super) rollback_report_path: Option<&'a std::path::Path>,
    pub(super) rollback_status: Option<&'a str>,
}

pub(super) fn update_lifecycle_summary(
    repo_root: &std::path::Path,
    run_id: &RunId,
    profile: &str,
    namespace: &str,
    update: LifecycleSummaryUpdate<'_>,
) -> Result<std::path::PathBuf, String> {
    let LifecycleSummaryUpdate {
        upgrade_report_path,
        upgrade_status,
        rollback_report_path,
        rollback_status,
    } = update;
    let summary_path = simulation_report_path(repo_root, run_id, "ops-lifecycle-summary.json")?;
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
        .ok_or_else(|| "ops-lifecycle-summary.json profiles must be an array".to_string())?;
    if let Some(existing) = rows
        .iter_mut()
        .find(|row| row.get("profile").and_then(|v| v.as_str()) == Some(profile))
    {
        existing["namespace"] = serde_json::json!(namespace);
        if let Some(path) = upgrade_report_path {
            existing["upgrade_report_path"] = serde_json::json!(path.display().to_string());
        }
        if let Some(status) = upgrade_status {
            existing["upgrade_status"] = serde_json::json!(status);
        }
        if let Some(path) = rollback_report_path {
            existing["rollback_report_path"] = serde_json::json!(path.display().to_string());
        }
        if let Some(status) = rollback_status {
            existing["rollback_status"] = serde_json::json!(status);
        }
    } else {
        let mut row = serde_json::json!({
            "profile": profile,
            "namespace": namespace
        });
        if let Some(path) = upgrade_report_path {
            row["upgrade_report_path"] = serde_json::json!(path.display().to_string());
        }
        if let Some(status) = upgrade_status {
            row["upgrade_status"] = serde_json::json!(status);
        }
        if let Some(path) = rollback_report_path {
            row["rollback_report_path"] = serde_json::json!(path.display().to_string());
        }
        if let Some(status) = rollback_status {
            row["rollback_status"] = serde_json::json!(status);
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

pub(super) fn load_readiness_baseline(
    repo_root: &std::path::Path,
    profile: &str,
) -> Result<Option<u128>, String> {
    let path = readiness_baseline_path(repo_root)?;
    if !path.exists() {
        return Ok(None);
    }
    let value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
    Ok(value
        .get("profiles")
        .and_then(|rows| rows.as_object())
        .and_then(|rows| rows.get(profile))
        .and_then(|row| row.get("baseline_elapsed_ms"))
        .and_then(serde_json::Value::as_u64)
        .map(u128::from))
}

pub(super) fn update_readiness_baseline(
    repo_root: &std::path::Path,
    profile: &str,
    elapsed_ms: u128,
) -> Result<std::path::PathBuf, String> {
    let path = readiness_baseline_path(repo_root)?;
    let mut payload = if path.exists() {
        serde_json::from_str::<serde_json::Value>(
            &std::fs::read_to_string(&path)
                .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
        )
        .map_err(|err| format!("failed to parse {}: {err}", path.display()))?
    } else {
        serde_json::json!({
            "schema_version": 1,
            "profiles": {}
        })
    };
    if !payload["profiles"].is_object() {
        payload["profiles"] = serde_json::json!({});
    }
    payload["profiles"][profile] = serde_json::json!({
        "baseline_elapsed_ms": elapsed_ms
    });
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
    Ok(path)
}
