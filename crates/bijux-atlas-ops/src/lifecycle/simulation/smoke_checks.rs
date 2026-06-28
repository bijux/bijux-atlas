// SPDX-License-Identifier: Apache-2.0

use crate::kubernetes::service_probe::run_kubectl_service_smoke_checks;
use crate::lifecycle::simulation::paths::write_simulation_report;
use serde_json::Value;
use std::path::Path;

pub fn smoke_command_payload(
    repo_root: &Path,
    run_id: &str,
    namespace: &str,
    local_port: u16,
) -> Result<(Value, i32), String> {
    let checks = run_kubectl_service_smoke_checks(repo_root, namespace, local_port)?;
    let errors = checks
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
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "cluster": "kind",
        "namespace": namespace,
        "status": status,
        "checks": checks
    });
    let report_path = write_simulation_report(repo_root, run_id, "ops-smoke.json", &payload)?;
    let envelope = serde_json::json!({
        "schema_version": 1,
        "text": if status == "ok" { "smoke checks passed" } else { "smoke checks failed" },
        "rows": [{
            "schema_version": 1,
            "cluster": "kind",
            "namespace": payload["namespace"].clone(),
            "status": status,
            "checks": payload["checks"].clone(),
            "report_path": report_path.display().to_string()
        }],
        "summary": {"total": 1, "errors": errors.len(), "warnings": 0}
    });
    Ok((envelope, if errors.is_empty() { 0 } else { 1 }))
}
