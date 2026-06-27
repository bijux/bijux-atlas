// SPDX-License-Identifier: Apache-2.0

use super::path_contracts::load_report_path;
use super::report_contract::LoadReportContract;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub fn write_load_report(
    repo_root: &Path,
    run_id: &str,
    suite: &str,
    report: &LoadReportContract,
) -> Result<std::path::PathBuf, String> {
    let report_path = load_report_path(repo_root, run_id, suite);
    if let Some(parent) = report_path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(
        &report_path,
        serde_json::to_string_pretty(report).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    Ok(report_path)
}

pub fn load_report_payload(report_path: &str, report: &LoadReportContract) -> Value {
    json!({
        "schema_version": 1,
        "text": format!("ops load report suite={}", report.suite),
        "rows": [{
            "report_path": report_path,
            "report": report
        }],
        "summary": {"total": 1, "errors": if report.violations.is_empty() { 0 } else { 1 }, "warnings": 0}
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load::report_contract::{LoadMetrics, LoadReportContract, LoadThresholds};
    use tempfile::tempdir;

    fn sample_report() -> LoadReportContract {
        LoadReportContract {
            schema_version: 1,
            kind: "ops_load_report_v1".to_string(),
            suite: "mixed".to_string(),
            run_id: "atlas-run".to_string(),
            metrics: LoadMetrics {
                p95_ms: 100.0,
                p99_ms: 120.0,
                error_rate: 0.0,
            },
            thresholds: LoadThresholds {
                p95_ms_max: 200.0,
                p99_ms_max: 250.0,
                error_rate_max: 0.01,
            },
            violations: Vec::new(),
        }
    }

    #[test]
    fn load_report_writer_uses_owned_path_contract() {
        let repo_root = tempdir().expect("temp dir should exist");
        let report = sample_report();
        let written = write_load_report(repo_root.path(), "atlas-run", "mixed", &report)
            .expect("report should write");
        assert_eq!(
            written,
            repo_root
                .path()
                .join("artifacts/ops/atlas-run/load/mixed/report.json")
        );
    }

    #[test]
    fn load_report_payload_tracks_violation_status() {
        let mut report = sample_report();
        report.violations.push("threshold breach".to_string());
        let payload =
            load_report_payload("artifacts/ops/atlas-run/load/mixed/report.json", &report);
        assert_eq!(payload["summary"]["errors"], 1);
        assert_eq!(payload["rows"][0]["report"]["suite"], "mixed");
    }
}
