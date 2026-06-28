// SPDX-License-Identifier: Apache-2.0

use crate::lifecycle::simulation::paths::write_simulation_report;
use crate::lifecycle::simulation::records::{
    drill_check_paths, load_drill_registry, update_drill_summary,
};
use serde_json::Value;
use std::path::Path;

pub fn drill_contract_payload(
    repo_root: &Path,
    run_id: &str,
    drill_name: &str,
) -> Result<(Value, i32), String> {
    let drills = load_drill_registry(repo_root)?;
    let drill = drills
        .iter()
        .find(|row| row.get("name").and_then(serde_json::Value::as_str) == Some(drill_name))
        .cloned()
        .ok_or_else(|| format!("unknown drill `{drill_name}`"))?;
    let mut checks = Vec::new();
    for (name, path) in drill_check_paths(repo_root, drill_name) {
        checks.push(serde_json::json!({
            "name": name,
            "status": if path.exists() { "pass" } else { "fail" },
            "detail": if path.exists() {
                format!("verified {}", path.strip_prefix(repo_root).unwrap_or(&path).display())
            } else {
                format!("missing {}", path.strip_prefix(repo_root).unwrap_or(&path).display())
            }
        }));
    }
    let status = if checks
        .iter()
        .all(|row| row.get("status").and_then(serde_json::Value::as_str) == Some("pass"))
    {
        "pass"
    } else {
        "fail"
    };
    let evidence_paths = drill_check_paths(repo_root, drill_name)
        .into_iter()
        .map(|(_, path)| {
            path.strip_prefix(repo_root)
                .unwrap_or(&path)
                .display()
                .to_string()
        })
        .collect::<Vec<_>>();
    let payload = serde_json::json!({
        "schema_version": 1,
        "drill": drill_name,
        "status": status,
        "execution_mode": "contract-verification",
        "expected_outcome": drill.get("expected_outcome").cloned().unwrap_or(serde_json::Value::String(String::new())),
        "checks": checks,
        "evidence_paths": evidence_paths
    });
    let report_path = write_simulation_report(
        repo_root,
        run_id,
        &format!("ops-drill-{drill_name}.json"),
        &payload,
    )?;
    let summary_path = update_drill_summary(repo_root, run_id, drill_name, &report_path, status)?;
    Ok((
        serde_json::json!({
            "schema_version": 1,
            "status": status,
            "text": if status == "pass" { "drill checks passed" } else { "drill checks failed" },
            "rows": [{
                "drill": drill_name,
                "report_path": report_path.display().to_string(),
                "summary_path": summary_path.display().to_string(),
                "expected_outcome": drill.get("expected_outcome").cloned().unwrap_or(serde_json::Value::String(String::new()))
            }],
            "summary": {"total": 1, "errors": if status == "pass" { 0 } else { 1 }, "warnings": 0}
        }),
        if status == "pass" { 0 } else { 1 },
    ))
}

#[cfg(test)]
mod tests {
    use super::drill_contract_payload;

    #[test]
    fn drill_contract_payload_writes_owned_reports() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops/observe"))
            .expect("create drill registry root");
        std::fs::write(
            root.path().join("ops/observe/drills.json"),
            serde_json::json!({
                "drills": [{
                    "name": "catalog-unreachable",
                    "expected_outcome": "catalog readiness returns unavailable"
                }]
            })
            .to_string(),
        )
        .expect("write drill registry");
        for relative in [
            "docs/bijux-atlas/contracts/operational-contracts.md",
            "configs/sources/operations/observability/error-codes.json",
        ] {
            let path = root.path().join(relative);
            std::fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
            std::fs::write(path, "owned").expect("write owned evidence");
        }
        let readiness_source = root
            .path()
            .join("crates/bijux-atlas-server/src/adapters/inbound/http/route_support.rs");
        std::fs::create_dir_all(readiness_source.parent().expect("parent")).expect("create parent");
        std::fs::write(&readiness_source, "owned").expect("write readiness source");

        let (payload, exit_code) =
            drill_contract_payload(root.path(), "atlas-run", "catalog-unreachable")
                .expect("drill payload");

        assert_eq!(exit_code, 0);
        assert_eq!(payload["status"], "pass");
        assert!(payload["rows"][0]["summary_path"]
            .as_str()
            .is_some_and(|path| path.ends_with("ops-drills-summary.json")));
    }
}
