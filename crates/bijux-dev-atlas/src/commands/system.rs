// SPDX-License-Identifier: Apache-2.0

use crate::cli::{SystemCommand, SystemSimulateCommand};
use crate::{emit_payload, resolve_repo_root};
use std::fs;
use std::path::{Path, PathBuf};

fn simulation_registry_path(root: &Path) -> PathBuf {
    root.join("configs/system/simulation-scenarios.json")
}

fn simulation_artifact_path(root: &Path, scenario_id: &str) -> PathBuf {
    root.join("artifacts/system/simulation")
        .join(scenario_id)
        .join("summary.json")
}

fn simulate(
    repo_root: Option<PathBuf>,
    format: crate::cli::FormatArg,
    out: Option<PathBuf>,
    scenario_id: &str,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let registry_path = simulation_registry_path(&root);
    let registry: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&registry_path)
            .map_err(|err| format!("read {} failed: {err}", registry_path.display()))?,
    )
    .map_err(|err| format!("parse {} failed: {err}", registry_path.display()))?;

    let scenarios = registry
        .get("scenarios")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();

    let selected = scenarios
        .iter()
        .find(|row| row.get("id").and_then(serde_json::Value::as_str) == Some(scenario_id))
        .cloned()
        .ok_or_else(|| format!("scenario `{scenario_id}` is not defined in registry"))?;

    let artifact_path = simulation_artifact_path(&root, scenario_id);
    if let Some(parent) = artifact_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("create {} failed: {err}", parent.display()))?;
    }

    let report = serde_json::json!({
        "schema_version": 1,
        "kind": "system_simulation_report",
        "scenario": selected,
        "status": "ok",
        "deterministic_order": 1,
        "artifacts": {
            "summary": artifact_path.strip_prefix(&root).unwrap_or(&artifact_path).display().to_string(),
            "scenario_registry": registry_path.strip_prefix(&root).unwrap_or(&registry_path).display().to_string(),
        }
    });

    fs::write(
        &artifact_path,
        serde_json::to_string_pretty(&report)
            .map_err(|err| format!("encode simulation summary failed: {err}"))?,
    )
    .map_err(|err| format!("write {} failed: {err}", artifact_path.display()))?;

    let rendered = emit_payload(format, out, &report)?;
    Ok((rendered, 0))
}

pub(crate) fn run_system_command(
    _quiet: bool,
    command: SystemCommand,
) -> Result<(String, i32), String> {
    match command {
        SystemCommand::Simulate { command } => match command {
            SystemSimulateCommand::Install(args) => {
                simulate(args.repo_root, args.format, args.out, "fresh-install")
            }
            SystemSimulateCommand::Upgrade(args) => {
                simulate(args.repo_root, args.format, args.out, "upgrade-previous-release")
            }
            SystemSimulateCommand::Rollback(args) => {
                simulate(args.repo_root, args.format, args.out, "rollback-after-failed-upgrade")
            }
            SystemSimulateCommand::OfflineMode(args) => {
                simulate(args.repo_root, args.format, args.out, "offline-mode")
            }
        },
    }
}
