// SPDX-License-Identifier: Apache-2.0
//! Simulation drill registry helpers and RunId-aware report path adapters.

use crate::RunId;
use bijux_atlas_ops::lifecycle::simulation_paths;

pub(super) fn simulation_report_path(
    repo_root: &std::path::Path,
    run_id: &RunId,
    file_name: &str,
) -> Result<std::path::PathBuf, String> {
    simulation_paths::simulation_report_path(repo_root, run_id.as_str(), file_name)
}

pub(super) fn write_simulation_report(
    repo_root: &std::path::Path,
    run_id: &RunId,
    file_name: &str,
    payload: &serde_json::Value,
) -> Result<std::path::PathBuf, String> {
    simulation_paths::write_simulation_report(repo_root, run_id.as_str(), file_name, payload)
}

pub(super) fn load_drill_registry(
    repo_root: &std::path::Path,
) -> Result<Vec<serde_json::Value>, String> {
    let path = repo_root.join("ops/observe/drills.json");
    let payload: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
    Ok(payload
        .get("drills")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default())
}

pub(super) fn update_drill_summary(
    repo_root: &std::path::Path,
    run_id: &RunId,
    drill: &str,
    report_path: &std::path::Path,
    status: &str,
) -> Result<std::path::PathBuf, String> {
    let summary_path = simulation_report_path(repo_root, run_id, "ops-drills-summary.json")?;
    let mut payload = if summary_path.exists() {
        serde_json::from_str::<serde_json::Value>(
            &std::fs::read_to_string(&summary_path)
                .map_err(|err| format!("failed to read {}: {err}", summary_path.display()))?,
        )
        .map_err(|err| format!("failed to parse {}: {err}", summary_path.display()))?
    } else {
        serde_json::json!({
            "schema_version": 1,
            "drills": []
        })
    };
    if !payload["drills"].is_array() {
        payload["drills"] = serde_json::json!([]);
    }
    let rows = payload["drills"]
        .as_array_mut()
        .ok_or_else(|| "drill summary rows must be an array".to_string())?;
    rows.retain(|row| row.get("name").and_then(serde_json::Value::as_str) != Some(drill));
    rows.push(serde_json::json!({
        "name": drill,
        "status": status,
        "report_path": report_path.strip_prefix(repo_root).unwrap_or(report_path).display().to_string()
    }));
    rows.sort_by(|left, right| {
        left.get("name")
            .and_then(serde_json::Value::as_str)
            .cmp(&right.get("name").and_then(serde_json::Value::as_str))
    });
    std::fs::write(
        &summary_path,
        serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("failed to write {}: {err}", summary_path.display()))?;
    Ok(summary_path)
}
