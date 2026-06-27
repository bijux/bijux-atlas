// SPDX-License-Identifier: Apache-2.0
//! Shared evidence helpers for install-status flows.

use super::*;
use crate::OpsProcess;

pub(super) fn package_chart_for_evidence(
    process: &OpsProcess,
    repo_root: &std::path::Path,
) -> Result<std::path::PathBuf, String> {
    let evidence_root = evidence_root(repo_root)?;
    let package_dir = evidence_root.join("packages");
    std::fs::create_dir_all(&package_dir)
        .map_err(|err| format!("failed to create {}: {err}", package_dir.display()))?;
    let chart_path = simulation_current_chart_path(repo_root);
    let argv = vec![
        "package".to_string(),
        chart_path.display().to_string(),
        "--destination".to_string(),
        package_dir.display().to_string(),
    ];
    process
        .run_subprocess("helm", &argv, repo_root)
        .map_err(|err| err.to_stable_message())?;
    let mut packages = std::fs::read_dir(&package_dir)
        .map_err(|err| format!("failed to read {}: {err}", package_dir.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|v| v.to_str()) == Some("tgz"))
        .collect::<Vec<_>>();
    packages.sort();
    packages
        .pop()
        .ok_or_else(|| format!("no chart package produced in {}", package_dir.display()))
}
