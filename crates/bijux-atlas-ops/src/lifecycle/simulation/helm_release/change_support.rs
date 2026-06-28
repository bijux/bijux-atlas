// SPDX-License-Identifier: Apache-2.0

use crate::kubernetes::execution::KubernetesCommandRunner;
use crate::kubernetes::service_probe::run_kubectl_service_smoke_checks;
use crate::kubernetes::workload_wait::run_readiness_wait;
use crate::lifecycle::release::records::load_readiness_baseline;
use crate::lifecycle::simulation::paths::write_simulation_report;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub const READINESS_REGRESSION_THRESHOLD_PERCENT: u64 = 125;

pub struct ReleaseHealthAssessment {
    pub wait_rows: Vec<Value>,
    pub wait_errors: Vec<String>,
    pub wait_ms: u128,
    pub smoke_rows: Vec<Value>,
    pub smoke_errors: Vec<String>,
    pub smoke_report_path: PathBuf,
    pub baseline_elapsed_ms: Option<u128>,
    pub readiness_threshold_percent: u64,
    pub regression_ok: bool,
}

impl ReleaseHealthAssessment {
    #[must_use]
    pub fn errors(&self) -> Vec<String> {
        let mut errors = self
            .wait_errors
            .iter()
            .cloned()
            .chain(self.smoke_errors.iter().cloned())
            .collect::<Vec<_>>();
        if !self.regression_ok {
            errors.push(format!(
                "readiness regression exceeded {}% of baseline",
                self.readiness_threshold_percent
            ));
        }
        errors
    }
}

pub fn assess_release_health(
    runner: &impl KubernetesCommandRunner,
    repo_root: &Path,
    run_id: &str,
    profile: &str,
    namespace: &str,
    timeout_seconds: u64,
) -> Result<ReleaseHealthAssessment, String> {
    let (wait_rows, wait_errors, wait_ms) =
        run_readiness_wait(runner, repo_root, namespace, timeout_seconds);
    let smoke_rows = if wait_errors.is_empty() {
        run_kubectl_service_smoke_checks(repo_root, namespace, 18080)?
    } else {
        Vec::new()
    };
    let smoke_errors = smoke_rows
        .iter()
        .filter(|row| row["status"].as_u64().unwrap_or(0) != 200)
        .map(|row| {
            format!(
                "{} returned status {}",
                row["path"].as_str().unwrap_or("unknown"),
                row["status"].as_u64().unwrap_or(0)
            )
        })
        .collect::<Vec<_>>();
    let smoke_payload = serde_json::json!({
        "schema_version": 1,
        "cluster": "kind",
        "namespace": namespace,
        "status": if wait_errors.is_empty() && smoke_errors.is_empty() { "ok" } else { "failed" },
        "checks": smoke_rows
    });
    let smoke_report_path =
        write_simulation_report(repo_root, run_id, "ops-smoke.json", &smoke_payload)?;
    let baseline_elapsed_ms = load_readiness_baseline(repo_root, profile)?;
    let regression_ok = baseline_elapsed_ms
        .map(|baseline| {
            wait_ms.saturating_mul(100)
                <= baseline.saturating_mul(u128::from(READINESS_REGRESSION_THRESHOLD_PERCENT))
        })
        .unwrap_or(true);
    Ok(ReleaseHealthAssessment {
        wait_rows,
        wait_errors,
        wait_ms,
        smoke_rows: smoke_payload["checks"]
            .as_array()
            .cloned()
            .unwrap_or_default(),
        smoke_errors,
        smoke_report_path,
        baseline_elapsed_ms,
        readiness_threshold_percent: READINESS_REGRESSION_THRESHOLD_PERCENT,
        regression_ok,
    })
}
