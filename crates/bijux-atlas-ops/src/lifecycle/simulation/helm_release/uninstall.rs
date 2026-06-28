// SPDX-License-Identifier: Apache-2.0

use crate::lifecycle::simulation::context::SimulationCommandRunner;
use crate::lifecycle::simulation::paths::write_simulation_report;
use crate::lifecycle::simulation::records::{update_simulation_summary, SimulationSummaryUpdate};
use serde_json::Value;
use std::path::Path;

pub fn helm_uninstall_payload(
    runner: &impl SimulationCommandRunner,
    repo_root: &Path,
    run_id: &str,
    profile: &str,
    namespace: &str,
) -> Result<(Value, i32), String> {
    let helm_args = vec![
        "uninstall".to_string(),
        "bijux-atlas".to_string(),
        "--namespace".to_string(),
        namespace.to_string(),
    ];
    let (helm_stdout, helm_event) = runner.run("helm", &helm_args, repo_root)?;
    let cleanup_args = vec![
        "get".to_string(),
        "all".to_string(),
        "-n".to_string(),
        namespace.to_string(),
        "-o".to_string(),
        "name".to_string(),
    ];
    let (cleanup_stdout, cleanup_event) = runner.run("kubectl", &cleanup_args, repo_root)?;
    let leftovers = cleanup_stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    let status = if leftovers.is_empty() { "ok" } else { "failed" };
    let cleanup_payload = serde_json::json!({
        "schema_version": 1,
        "cluster": "kind",
        "namespace": namespace,
        "status": status,
        "leftovers": leftovers
    });
    let cleanup_report_path =
        write_simulation_report(repo_root, run_id, "ops-cleanup.json", &cleanup_payload)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "profile": profile,
        "cluster": "kind",
        "namespace": cleanup_payload["namespace"].clone(),
        "status": status,
        "details": {
            "helm": {
                "stdout": helm_stdout,
                "event": helm_event
            },
            "cleanup_check": {
                "report_path": cleanup_report_path.display().to_string(),
                "leftovers": cleanup_payload["leftovers"].clone(),
                "event": cleanup_event
            }
        }
    });
    let report_path = write_simulation_report(repo_root, run_id, "ops-uninstall.json", &payload)?;
    let summary_path = update_simulation_summary(
        repo_root,
        run_id,
        profile,
        namespace,
        SimulationSummaryUpdate {
            install_report_path: None,
            install_status: None,
            smoke_report_path: None,
            smoke_status: None,
            cleanup_report_path: Some(&cleanup_report_path),
            cleanup_status: Some(status),
        },
    )?;
    Ok((
        serde_json::json!({
            "schema_version": 1,
            "text": if status == "ok" { "helm uninstall completed" } else { "helm uninstall left resources" },
            "rows": [{
                "schema_version": 1,
                "profile": payload["profile"].clone(),
                "cluster": "kind",
                "namespace": payload["namespace"].clone(),
                "status": status,
                "report_path": report_path.display().to_string(),
                "summary_report_path": summary_path.display().to_string(),
                "details": payload["details"].clone()
            }],
            "summary": {"total": 1, "errors": if status == "ok" { 0 } else { 1 }, "warnings": 0}
        }),
        if status == "ok" { 0 } else { 1 },
    ))
}

#[cfg(test)]
mod tests {
    use super::helm_uninstall_payload;
    use crate::lifecycle::simulation::context::SimulationCommandRunner;
    use serde_json::Value;
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::path::Path;

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
    fn helm_uninstall_payload_reports_clean_namespace() {
        let root = tempfile::tempdir().expect("tempdir");
        let runner = MockRunner {
            results: RefCell::new(VecDeque::from([
                Ok((
                    "release removed".to_string(),
                    serde_json::json!({"binary":"helm"}),
                )),
                Ok(("".to_string(), serde_json::json!({"binary":"kubectl"}))),
            ])),
        };

        let (payload, exit_code) = helm_uninstall_payload(
            &runner,
            root.path(),
            "atlas-run",
            "kind",
            "bijux-atlas-kind",
        )
        .expect("helm uninstall payload");

        assert_eq!(exit_code, 0);
        assert_eq!(payload["rows"][0]["status"], "ok");
    }
}
