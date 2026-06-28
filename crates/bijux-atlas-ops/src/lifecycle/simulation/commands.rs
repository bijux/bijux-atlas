// SPDX-License-Identifier: Apache-2.0

use crate::kubernetes::access_guard::ensure_simulation_cluster_context;
use crate::kubernetes::service_probe::run_kubectl_service_smoke_checks;
use crate::lifecycle::evidence::artifacts::write_debug_artifact;
use crate::lifecycle::simulation::paths::write_simulation_report;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub trait SimulationCommandRunner {
    fn run(&self, binary: &str, args: &[String], cwd: &Path) -> Result<(String, Value), String>;
}

pub fn ensure_owned_simulation_context(
    runner: &impl SimulationCommandRunner,
    force: bool,
) -> Result<(), String> {
    struct Adapter<'a, T>(&'a T);

    impl<T: SimulationCommandRunner> crate::kubernetes::execution::KubernetesCommandRunner
        for Adapter<'_, T>
    {
        fn run(
            &self,
            binary: &str,
            args: &[String],
            cwd: &Path,
        ) -> Result<crate::kubernetes::execution::SubprocessCapture, String> {
            let (stdout, event) = self.0.run(binary, args, cwd)?;
            Ok(crate::kubernetes::execution::SubprocessCapture { stdout, event })
        }
    }

    ensure_simulation_cluster_context(&Adapter(runner), force)
}

pub fn emit_debug_bundle_report(
    repo_root: &Path,
    run_id: &str,
    namespace: &str,
    category: &str,
    files: &[PathBuf],
) -> Result<PathBuf, String> {
    let payload = serde_json::json!({
        "schema_version": 1,
        "cluster": "kind",
        "namespace": namespace,
        "category": category,
        "status": "ok",
        "files": files.iter().map(|path| path.display().to_string()).collect::<Vec<_>>()
    });
    write_simulation_report(
        repo_root,
        run_id,
        &format!("ops-debug-bundle-{category}.json"),
        &payload,
    )
}

pub fn debug_collect_payload(
    runner: &impl SimulationCommandRunner,
    repo_root: &Path,
    run_id: &str,
    namespace: &str,
    category: &str,
    file_name: &str,
    argv: Vec<String>,
) -> Result<Value, String> {
    let (stdout, event) = runner.run("kubectl", &argv, repo_root)?;
    let artifact_path = write_debug_artifact(repo_root, run_id, namespace, file_name, &stdout)?;
    let report_path = emit_debug_bundle_report(
        repo_root,
        run_id,
        namespace,
        category,
        std::slice::from_ref(&artifact_path),
    )?;
    Ok(serde_json::json!({
        "schema_version": 1,
        "text": format!("{category} collected"),
        "rows": [{
            "schema_version": 1,
            "cluster": "kind",
            "namespace": namespace,
            "category": category,
            "status": "ok",
            "files": [artifact_path.display().to_string()],
            "report_path": report_path.display().to_string(),
            "event": event
        }],
        "summary": {"total": 1, "errors": 0, "warnings": 0}
    }))
}

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

pub fn drill_contract_payload(
    repo_root: &Path,
    run_id: &str,
    drill_name: &str,
) -> Result<(Value, i32), String> {
    let drills = crate::lifecycle::simulation::records::load_drill_registry(repo_root)?;
    let drill = drills
        .iter()
        .find(|row| row.get("name").and_then(serde_json::Value::as_str) == Some(drill_name))
        .cloned()
        .ok_or_else(|| format!("unknown drill `{drill_name}`"))?;
    let mut checks = Vec::new();
    for (name, path) in
        crate::lifecycle::simulation::records::drill_check_paths(repo_root, drill_name)
    {
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
    let evidence_paths =
        crate::lifecycle::simulation::records::drill_check_paths(repo_root, drill_name)
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
    let summary_path = crate::lifecycle::simulation::records::update_drill_summary(
        repo_root,
        run_id,
        drill_name,
        &report_path,
        status,
    )?;
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
    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    struct MockRunner {
        results: RefCell<VecDeque<Result<(String, Value), String>>>,
    }

    impl SimulationCommandRunner for MockRunner {
        fn run(
            &self,
            _binary: &str,
            _args: &[String],
            _cwd: &Path,
        ) -> Result<(String, Value), String> {
            self.results
                .borrow_mut()
                .pop_front()
                .expect("mock result should exist")
        }
    }

    #[test]
    fn debug_collect_payload_writes_owned_artifacts() {
        let root = tempfile::tempdir().expect("tempdir");
        let runner = MockRunner {
            results: RefCell::new(VecDeque::from([Ok((
                "hello".to_string(),
                serde_json::json!({"binary":"kubectl"}),
            ))])),
        };

        let payload = debug_collect_payload(
            &runner,
            root.path(),
            "atlas-run",
            "bijux-atlas-kind",
            "logs",
            "pod-logs.txt",
            vec!["logs".to_string()],
        )
        .expect("debug collect payload");

        assert_eq!(payload["text"], "logs collected");
        assert!(payload["rows"][0]["report_path"]
            .as_str()
            .is_some_and(|path| path.ends_with("ops-debug-bundle-logs.json")));
    }

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
