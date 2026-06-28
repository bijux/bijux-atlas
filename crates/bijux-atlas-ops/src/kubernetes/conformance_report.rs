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

pub fn ops_conformance_payload(
    inventory_errors: &[String],
    status_code: i32,
    status_rendered: &str,
) -> Value {
    let errors = inventory_errors.len() + usize::from(status_code != 0);
    let status = if errors == 0 { "ok" } else { "failed" };

    json!({
        "schema_version": 1,
        "status": status,
        "text": format!("ops conformance: status={status}"),
        "rows": [{
            "inventory_errors": inventory_errors,
            "status_exit": status_code,
            "status_output": status_rendered
        }],
        "summary": {"total": 1, "errors": errors, "warnings": 0}
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

    #[test]
    fn ops_conformance_payload_fails_when_inventory_or_status_fails() {
        let payload = ops_conformance_payload(&["missing service".to_string()], 1, "status rows");

        assert_eq!(payload["status"], "failed");
        assert_eq!(payload["summary"]["errors"], 2);
        assert_eq!(payload["rows"][0]["status_exit"], 1);
    }
}
