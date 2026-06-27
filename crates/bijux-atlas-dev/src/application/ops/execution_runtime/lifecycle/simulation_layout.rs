// SPDX-License-Identifier: Apache-2.0
//! Simulation drill registry helpers and RunId-aware report path adapters.

use crate::RunId;
use bijux_atlas_ops::lifecycle::{simulation_paths, simulation_records};

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
    simulation_records::load_drill_registry(repo_root)
}

pub(super) fn update_drill_summary(
    repo_root: &std::path::Path,
    run_id: &RunId,
    drill: &str,
    report_path: &std::path::Path,
    status: &str,
) -> Result<std::path::PathBuf, String> {
    simulation_records::update_drill_summary(repo_root, run_id.as_str(), drill, report_path, status)
}
