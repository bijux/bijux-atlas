// SPDX-License-Identifier: Apache-2.0
//! Release rollout observation and readiness baseline record helpers.

use crate::{OpsProcess, RunId};
pub(super) use bijux_atlas_ops::lifecycle::release_records::LifecycleSummaryUpdate;

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

pub(super) fn update_lifecycle_summary(
    repo_root: &std::path::Path,
    run_id: &RunId,
    profile: &str,
    namespace: &str,
    update: LifecycleSummaryUpdate<'_>,
) -> Result<std::path::PathBuf, String> {
    bijux_atlas_ops::lifecycle::release_records::update_lifecycle_summary(
        repo_root,
        run_id.as_str(),
        profile,
        namespace,
        update,
    )
}

pub(super) fn load_readiness_baseline(
    repo_root: &std::path::Path,
    profile: &str,
) -> Result<Option<u128>, String> {
    bijux_atlas_ops::lifecycle::release_records::load_readiness_baseline(repo_root, profile)
}

pub(super) fn update_readiness_baseline(
    repo_root: &std::path::Path,
    profile: &str,
    elapsed_ms: u128,
) -> Result<std::path::PathBuf, String> {
    bijux_atlas_ops::lifecycle::release_records::update_readiness_baseline(
        repo_root, profile, elapsed_ms,
    )
}
