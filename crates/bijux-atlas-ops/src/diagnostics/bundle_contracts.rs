// SPDX-License-Identifier: Apache-2.0

use super::path_contracts::{diagnose_bundle_path, diagnose_run_root};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

#[must_use]
pub fn collect_scenario_files(repo_root: &Path, scenario: Option<&str>) -> Vec<String> {
    let base = repo_root.join("artifacts/ops/scenarios");
    let mut rows = Vec::new();
    if !base.exists() {
        return rows;
    }
    let Ok(entries) = fs::read_dir(&base) else {
        return rows;
    };
    for scenario_entry in entries.flatten() {
        let scenario_name = scenario_entry.file_name().to_string_lossy().to_string();
        if let Some(filter) = scenario {
            if filter != scenario_name {
                continue;
            }
        }
        let Ok(runs) = fs::read_dir(scenario_entry.path()) else {
            continue;
        };
        for run in runs.flatten() {
            let Ok(files) = fs::read_dir(run.path()) else {
                continue;
            };
            for file in files.flatten() {
                if let Ok(relative_path) = file.path().strip_prefix(repo_root) {
                    rows.push(relative_path.display().to_string());
                }
            }
        }
    }
    rows.sort();
    rows
}

pub fn build_diagnose_bundle(
    run_id: &str,
    scenario_filter: Option<&str>,
    files: Vec<String>,
) -> Value {
    json!({
        "schema_version": 1,
        "kind": "ops_diagnose_bundle",
        "run_id": run_id,
        "scenario_filter": scenario_filter,
        "files": files,
        "sensitive_keys": ["password", "secret", "token", "api_key"]
    })
}

pub fn write_diagnose_bundle(
    repo_root: &Path,
    run_id: &str,
    bundle: &Value,
) -> Result<std::path::PathBuf, String> {
    let out_dir = diagnose_run_root(repo_root, run_id);
    fs::create_dir_all(&out_dir)
        .map_err(|err| format!("failed to create {}: {err}", out_dir.display()))?;
    let bundle_path = diagnose_bundle_path(repo_root, run_id);
    fs::write(
        &bundle_path,
        serde_json::to_string_pretty(bundle)
            .map_err(|err| format!("failed to encode {}: {err}", bundle_path.display()))?,
    )
    .map_err(|err| format!("failed to write {}: {err}", bundle_path.display()))?;
    Ok(bundle_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn scenario_files_are_collected_in_sorted_relative_form() {
        let repo_root = tempdir().expect("temp dir should exist");
        let first = repo_root
            .path()
            .join("artifacts/ops/scenarios/chaos/run-a/network.json");
        let second = repo_root
            .path()
            .join("artifacts/ops/scenarios/chaos/run-b/pods.json");
        fs::create_dir_all(first.parent().expect("first parent")).expect("create first parent");
        fs::create_dir_all(second.parent().expect("second parent")).expect("create second parent");
        fs::write(&first, "{}").expect("write first");
        fs::write(&second, "{}").expect("write second");

        let rows = collect_scenario_files(repo_root.path(), Some("chaos"));

        assert_eq!(
            rows,
            vec![
                "artifacts/ops/scenarios/chaos/run-a/network.json",
                "artifacts/ops/scenarios/chaos/run-b/pods.json"
            ]
        );
    }

    #[test]
    fn diagnose_bundle_writer_uses_owned_bundle_path() {
        let repo_root = tempdir().expect("temp dir should exist");
        let bundle = build_diagnose_bundle("atlas-run", Some("chaos"), vec!["a.json".to_string()]);

        let written = write_diagnose_bundle(repo_root.path(), "atlas-run", &bundle)
            .expect("bundle should write");

        assert_eq!(
            written,
            repo_root
                .path()
                .join("artifacts/ops/diagnose/atlas-run/bundle.json")
        );
    }
}
