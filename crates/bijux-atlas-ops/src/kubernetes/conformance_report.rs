// SPDX-License-Identifier: Apache-2.0

use super::path_contracts::atlas_conformance_report;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub fn build_conformance_report(run_id: &str, errors: &[String]) -> Value {
    let error_count = errors.len();
    let failed_sections = if error_count == 0 {
        Vec::<String>::new()
    } else {
        vec!["workload_readiness".to_string()]
    };
    json!({
        "schema_version": 1,
        "run_id": run_id,
        "suite_id": "k8s_conformance",
        "status": if error_count == 0 { "pass" } else { "fail" },
        "failed_sections": failed_sections,
        "sections": {
            "workload_readiness": {
                "status": if error_count == 0 { "pass" } else { "fail" },
                "missing": [],
                "failed": errors
            }
        }
    })
}

pub fn write_conformance_report(repo_root: &Path, report: &Value) -> Result<PathBuf, String> {
    let target = atlas_conformance_report(repo_root);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed creating {}: {err}", parent.display()))?;
    }
    fs::write(
        &target,
        serde_json::to_string_pretty(report).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("failed writing {}: {err}", target.display()))?;
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn conformance_report_uses_workload_readiness_section() {
        let report = build_conformance_report("run-42", &["pod `atlas` phase=Pending".to_string()]);
        assert_eq!(report["suite_id"], "k8s_conformance");
        assert_eq!(report["status"], "fail");
        assert_eq!(
            report["failed_sections"],
            serde_json::json!(["workload_readiness"])
        );
        assert_eq!(
            report["sections"]["workload_readiness"]["failed"],
            serde_json::json!(["pod `atlas` phase=Pending"])
        );
    }

    #[test]
    fn conformance_report_writes_to_the_owned_generated_path() {
        let repo_root = tempdir().expect("temp dir should exist");
        fs::create_dir_all(repo_root.path().join("ops/k8s/generated"))
            .expect("generated path should be creatable");
        let report = build_conformance_report("run-42", &[]);
        let written =
            write_conformance_report(repo_root.path(), &report).expect("report should write");
        assert_eq!(
            written,
            repo_root
                .path()
                .join("ops/k8s/generated/conformance-report.json")
        );
        let persisted = fs::read_to_string(&written).expect("report should be readable");
        assert!(persisted.contains("\"suite_id\": \"k8s_conformance\""));
    }
}
