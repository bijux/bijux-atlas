// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

#[must_use]
pub fn simulation_cluster_name() -> &'static str {
    "bijux-atlas-sim"
}

#[must_use]
pub fn simulation_cluster_context() -> String {
    format!("kind-{}", simulation_cluster_name())
}

#[must_use]
pub fn simulation_cluster_config(repo_root: &Path) -> PathBuf {
    repo_root.join("ops/k8s/kind/cluster.yaml")
}

#[must_use]
pub fn simulation_current_chart_path(repo_root: &Path) -> PathBuf {
    repo_root.join("ops/k8s/charts/bijux-atlas")
}

#[must_use]
pub fn simulation_previous_chart_path(repo_root: &Path) -> PathBuf {
    repo_root.join("artifacts/ops/chart-sources/previous/bijux-atlas.tgz")
}

pub fn simulation_report_path(
    repo_root: &Path,
    run_id: &str,
    file_name: &str,
) -> Result<PathBuf, String> {
    let path = repo_root
        .join("artifacts/ops")
        .join(run_id)
        .join("reports")
        .join(file_name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    Ok(path)
}

pub fn write_simulation_report(
    repo_root: &Path,
    run_id: &str,
    file_name: &str,
    payload: &serde_json::Value,
) -> Result<PathBuf, String> {
    let path = simulation_report_path(repo_root, run_id, file_name)?;
    std::fs::write(
        &path,
        serde_json::to_string_pretty(payload).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulation_cluster_identity_is_stable() {
        assert_eq!(simulation_cluster_name(), "bijux-atlas-sim");
        assert_eq!(simulation_cluster_context(), "kind-bijux-atlas-sim");
    }

    #[test]
    fn simulation_report_path_creates_report_directory() {
        let root = tempfile::tempdir().expect("tempdir");

        let path =
            simulation_report_path(root.path(), "atlas-run", "ops-kind.json").expect("report path");

        assert!(path.ends_with("artifacts/ops/atlas-run/reports/ops-kind.json"));
        assert!(
            path.parent().is_some_and(|parent| parent.exists()),
            "expected report directory to exist"
        );
    }
}
