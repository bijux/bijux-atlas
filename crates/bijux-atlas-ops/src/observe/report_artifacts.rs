// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

#[must_use]
pub fn observe_report_root(repo_root: &Path, run_id: &str) -> PathBuf {
    repo_root.join("artifacts/ops").join(run_id).join("observe")
}

pub fn write_observe_contract_report(
    repo_root: &Path,
    run_id: &str,
    file_name: &str,
    payload: &serde_json::Value,
) -> Result<String, String> {
    let out_dir = observe_report_root(repo_root, run_id);
    std::fs::create_dir_all(&out_dir)
        .map_err(|err| format!("failed to create {}: {err}", out_dir.display()))?;
    let out_path = out_dir.join(file_name);
    std::fs::write(
        &out_path,
        serde_json::to_string_pretty(payload).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("failed to write {}: {err}", out_path.display()))?;
    Ok(out_path
        .strip_prefix(repo_root)
        .unwrap_or(&out_path)
        .display()
        .to_string())
}

pub fn write_operational_readiness_markdown(
    repo_root: &Path,
    run_id: &str,
    status: &str,
    completeness: f64,
    threshold: f64,
) -> Result<String, String> {
    let out_dir = observe_report_root(repo_root, run_id);
    std::fs::create_dir_all(&out_dir)
        .map_err(|err| format!("failed to create {}: {err}", out_dir.display()))?;
    let out_path = out_dir.join("operational-readiness-report.md");
    let lines = [
        "# Operational Readiness Report".to_string(),
        format!("- Status: {status}"),
        format!("- Completeness: {:.2}", completeness),
        format!("- Threshold: {:.2}", threshold),
        format!("- SLO report: artifacts/ops/{run_id}/observe/slo-contract-report.json"),
        format!("- Alerts report: artifacts/ops/{run_id}/observe/alerts-contract-report.json"),
        format!("- Runbooks report: artifacts/ops/{run_id}/observe/runbooks-contract-report.json"),
    ];
    std::fs::write(&out_path, lines.join("\n") + "\n")
        .map_err(|err| format!("failed to write {}: {err}", out_path.display()))?;
    Ok(out_path
        .strip_prefix(repo_root)
        .unwrap_or(&out_path)
        .display()
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        observe_report_root, write_observe_contract_report, write_operational_readiness_markdown,
    };

    #[test]
    fn observe_report_root_is_stable() {
        let root = std::path::Path::new("/repo");

        let path = observe_report_root(root, "run-local");

        assert_eq!(
            path,
            std::path::PathBuf::from("/repo/artifacts/ops/run-local/observe")
        );
    }

    #[test]
    fn write_observe_contract_report_materializes_json_report() {
        let root = tempfile::tempdir().expect("tempdir");
        let payload = serde_json::json!({"status": "ok"});

        let relative = write_observe_contract_report(
            root.path(),
            "run-local",
            "alerts-contract-report.json",
            &payload,
        )
        .expect("write observe contract report");

        assert_eq!(
            relative,
            "artifacts/ops/run-local/observe/alerts-contract-report.json"
        );
    }

    #[test]
    fn write_operational_readiness_markdown_materializes_summary() {
        let root = tempfile::tempdir().expect("tempdir");

        let relative =
            write_operational_readiness_markdown(root.path(), "run-local", "ok", 1.0, 1.0)
                .expect("write readiness markdown");

        let body = std::fs::read_to_string(root.path().join(&relative)).expect("read markdown");
        assert!(body.contains("# Operational Readiness Report"));
        assert!(body.contains("- Status: ok"));
    }
}
