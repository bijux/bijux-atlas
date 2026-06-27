// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::manifest::LoadSuiteToml;
use super::path_contracts::load_summary_path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoadMetrics {
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub error_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoadThresholds {
    pub p95_ms_max: f64,
    pub p99_ms_max: f64,
    pub error_rate_max: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoadReportContract {
    pub schema_version: u64,
    pub kind: String,
    pub suite: String,
    pub run_id: String,
    pub metrics: LoadMetrics,
    pub thresholds: LoadThresholds,
    pub violations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadReportError {
    Read { path: PathBuf, message: String },
    Parse { path: PathBuf, message: String },
}

impl LoadReportError {
    #[must_use]
    pub fn detail(&self) -> String {
        match self {
            Self::Read { path, message } => {
                format!("failed to read {}: {message}", path.display())
            }
            Self::Parse { path, message } => {
                format!("failed to parse {}: {message}", path.display())
            }
        }
    }
}

pub fn evaluate_load_report(
    repo_root: &Path,
    suite: &str,
    suite_cfg: &LoadSuiteToml,
    run_id: &str,
) -> Result<LoadReportContract, LoadReportError> {
    let summary_path = load_summary_path(repo_root, run_id, suite);
    let summary_json = load_json_value(&summary_path)?;
    let thresholds = load_json_value(&repo_root.join(&suite_cfg.thresholds))?;
    let metrics = parse_k6_summary(&summary_json);
    let thresholds = parse_thresholds(&thresholds);
    let violations = threshold_violations(&metrics, &thresholds);

    Ok(LoadReportContract {
        schema_version: 1,
        kind: "ops_load_report_v1".to_string(),
        suite: suite.to_string(),
        run_id: run_id.to_string(),
        metrics,
        thresholds,
        violations,
    })
}

fn load_json_value(path: &Path) -> Result<Value, LoadReportError> {
    let raw = std::fs::read_to_string(path).map_err(|err| LoadReportError::Read {
        path: path.to_path_buf(),
        message: err.to_string(),
    })?;
    serde_json::from_str(&raw).map_err(|err| LoadReportError::Parse {
        path: path.to_path_buf(),
        message: err.to_string(),
    })
}

fn parse_k6_summary(summary: &Value) -> LoadMetrics {
    LoadMetrics {
        p95_ms: summary
            .get("metrics")
            .and_then(|value| value.get("http_req_duration"))
            .and_then(|value| value.get("values"))
            .and_then(|value| value.get("p(95)"))
            .and_then(Value::as_f64)
            .unwrap_or(0.0),
        p99_ms: summary
            .get("metrics")
            .and_then(|value| value.get("http_req_duration"))
            .and_then(|value| value.get("values"))
            .and_then(|value| value.get("p(99)"))
            .and_then(Value::as_f64)
            .unwrap_or(0.0),
        error_rate: summary
            .get("metrics")
            .and_then(|value| value.get("http_req_failed"))
            .and_then(|value| value.get("values"))
            .and_then(|value| value.get("rate"))
            .and_then(Value::as_f64)
            .unwrap_or(0.0),
    }
}

fn parse_thresholds(value: &Value) -> LoadThresholds {
    LoadThresholds {
        p95_ms_max: value
            .get("p95_ms_max")
            .and_then(Value::as_f64)
            .unwrap_or(f64::MAX),
        p99_ms_max: value
            .get("p99_ms_max")
            .and_then(Value::as_f64)
            .unwrap_or(f64::MAX),
        error_rate_max: value
            .get("error_rate_max")
            .and_then(Value::as_f64)
            .unwrap_or(f64::MAX),
    }
}

fn threshold_violations(metrics: &LoadMetrics, thresholds: &LoadThresholds) -> Vec<String> {
    let mut violations = Vec::new();
    if metrics.p95_ms > thresholds.p95_ms_max {
        violations.push(format!(
            "threshold breach p95 {} > {}",
            metrics.p95_ms, thresholds.p95_ms_max
        ));
    }
    if metrics.p99_ms > thresholds.p99_ms_max {
        violations.push(format!(
            "threshold breach p99 {} > {}",
            metrics.p99_ms, thresholds.p99_ms_max
        ));
    }
    if metrics.error_rate > thresholds.error_rate_max {
        violations.push(format!(
            "threshold breach error_rate {} > {}",
            metrics.error_rate, thresholds.error_rate_max
        ));
    }
    violations
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn evaluate_load_report_emits_threshold_breaches() {
        let root = tempfile::tempdir().expect("tempdir");
        let summary_path = load_summary_path(root.path(), "atlas-run", "mixed");
        std::fs::create_dir_all(summary_path.parent().expect("summary parent"))
            .expect("mkdir summary parent");
        std::fs::create_dir_all(root.path().join("ops/load/thresholds")).expect("mkdir thresholds");
        std::fs::write(
            &summary_path,
            "{\"metrics\":{\"http_req_duration\":{\"values\":{\"p(95)\":1200,\"p(99)\":1500}},\"http_req_failed\":{\"values\":{\"rate\":0.02}}}}",
        )
        .expect("write summary");
        std::fs::write(
            root.path()
                .join("ops/load/thresholds/mixed.thresholds.json"),
            "{\"p95_ms_max\":900,\"p99_ms_max\":1200,\"error_rate_max\":0.01}",
        )
        .expect("write thresholds");

        let suite_cfg = LoadSuiteToml {
            script: "ops/load/k6/suites/mixed-80-20.js".to_string(),
            dataset: "ops/load/queries/pinned-v1.json".to_string(),
            thresholds: "ops/load/thresholds/mixed.thresholds.json".to_string(),
            env: BTreeMap::new(),
        };

        let report =
            evaluate_load_report(root.path(), "mixed", &suite_cfg, "atlas-run").expect("report");

        assert_eq!(report.kind, "ops_load_report_v1");
        assert_eq!(report.violations.len(), 3);
    }
}
