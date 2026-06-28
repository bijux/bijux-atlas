// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

use crate::lifecycle::simulation::paths::simulation_report_path;

pub fn readiness_baseline_path(repo_root: &Path) -> Result<PathBuf, String> {
    let path = repo_root.join("artifacts/ops/history/readiness-baselines.json");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    Ok(path)
}

pub struct LifecycleSummaryUpdate<'a> {
    pub upgrade_report_path: Option<&'a Path>,
    pub upgrade_status: Option<&'a str>,
    pub rollback_report_path: Option<&'a Path>,
    pub rollback_status: Option<&'a str>,
}

pub fn update_lifecycle_summary(
    repo_root: &Path,
    run_id: &str,
    profile: &str,
    namespace: &str,
    update: LifecycleSummaryUpdate<'_>,
) -> Result<PathBuf, String> {
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

pub fn load_readiness_baseline(repo_root: &Path, profile: &str) -> Result<Option<u128>, String> {
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

pub fn update_readiness_baseline(
    repo_root: &Path,
    profile: &str,
    elapsed_ms: u128,
) -> Result<PathBuf, String> {
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

#[cfg(test)]
mod tests {
    use super::{
        load_readiness_baseline, update_lifecycle_summary, update_readiness_baseline,
        LifecycleSummaryUpdate,
    };

    #[test]
    fn lifecycle_summary_rows_remain_sorted_by_profile() {
        let root = tempfile::tempdir().expect("tempdir");

        update_lifecycle_summary(
            root.path(),
            "atlas-run",
            "zeta",
            "atlas-zeta",
            LifecycleSummaryUpdate {
                upgrade_report_path: None,
                upgrade_status: Some("ok"),
                rollback_report_path: None,
                rollback_status: None,
            },
        )
        .expect("write zeta summary");
        let path = update_lifecycle_summary(
            root.path(),
            "atlas-run",
            "alpha",
            "atlas-alpha",
            LifecycleSummaryUpdate {
                upgrade_report_path: None,
                upgrade_status: Some("ok"),
                rollback_report_path: None,
                rollback_status: None,
            },
        )
        .expect("write alpha summary");

        let payload: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).expect("read lifecycle summary"))
                .expect("parse lifecycle summary");
        let profiles = payload["profiles"]
            .as_array()
            .expect("profiles array")
            .iter()
            .map(|row| row["profile"].as_str().unwrap_or_default())
            .collect::<Vec<_>>();

        assert_eq!(profiles, vec!["alpha", "zeta"]);
    }

    #[test]
    fn readiness_baseline_round_trips_elapsed_ms() {
        let root = tempfile::tempdir().expect("tempdir");

        update_readiness_baseline(root.path(), "kind", 1337).expect("write baseline");
        let baseline =
            load_readiness_baseline(root.path(), "kind").expect("load readiness baseline");

        assert_eq!(baseline, Some(1337));
    }
}
