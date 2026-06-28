// SPDX-License-Identifier: Apache-2.0

use crate::lifecycle::evidence::artifacts::write_debug_artifact;
use crate::lifecycle::simulation::context::SimulationCommandRunner;
use crate::lifecycle::simulation::paths::write_simulation_report;
use serde_json::Value;
use std::path::{Path, PathBuf};

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

pub fn logs_collect_payload(
    runner: &impl SimulationCommandRunner,
    repo_root: &Path,
    run_id: &str,
    namespace: &str,
) -> Result<Value, String> {
    debug_collect_payload(
        runner,
        repo_root,
        run_id,
        namespace,
        "logs",
        "pod-logs.txt",
        vec![
            "logs".to_string(),
            "-n".to_string(),
            namespace.to_string(),
            "deployment/bijux-atlas".to_string(),
            "--tail=500".to_string(),
        ],
    )
}

pub fn describe_collect_payload(
    runner: &impl SimulationCommandRunner,
    repo_root: &Path,
    run_id: &str,
    namespace: &str,
) -> Result<Value, String> {
    debug_collect_payload(
        runner,
        repo_root,
        run_id,
        namespace,
        "describe",
        "describe.txt",
        vec![
            "describe".to_string(),
            "-n".to_string(),
            namespace.to_string(),
            "deployment/bijux-atlas".to_string(),
            "service/bijux-atlas".to_string(),
        ],
    )
}

pub fn events_collect_payload(
    runner: &impl SimulationCommandRunner,
    repo_root: &Path,
    run_id: &str,
    namespace: &str,
) -> Result<Value, String> {
    debug_collect_payload(
        runner,
        repo_root,
        run_id,
        namespace,
        "events",
        "events.txt",
        vec![
            "get".to_string(),
            "events".to_string(),
            "-n".to_string(),
            namespace.to_string(),
            "--sort-by=.metadata.creationTimestamp".to_string(),
        ],
    )
}

pub fn resources_snapshot_payload(
    runner: &impl SimulationCommandRunner,
    repo_root: &Path,
    run_id: &str,
    namespace: &str,
) -> Result<Value, String> {
    debug_collect_payload(
        runner,
        repo_root,
        run_id,
        namespace,
        "resources",
        "resources.yaml",
        vec![
            "get".to_string(),
            "all".to_string(),
            "-n".to_string(),
            namespace.to_string(),
            "-o".to_string(),
            "yaml".to_string(),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::{debug_collect_payload, logs_collect_payload, resources_snapshot_payload};
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
    fn logs_collect_payload_owns_kubectl_contract() {
        let root = tempfile::tempdir().expect("tempdir");
        let runner = MockRunner {
            results: RefCell::new(VecDeque::from([Ok((
                "hello".to_string(),
                serde_json::json!({"binary":"kubectl"}),
            ))])),
        };

        let payload = logs_collect_payload(&runner, root.path(), "atlas-run", "bijux-atlas-kind")
            .expect("logs payload");

        assert_eq!(payload["rows"][0]["category"], "logs");
        assert_eq!(payload["rows"][0]["status"], "ok");
    }

    #[test]
    fn resources_snapshot_payload_owns_yaml_capture_contract() {
        let root = tempfile::tempdir().expect("tempdir");
        let runner = MockRunner {
            results: RefCell::new(VecDeque::from([Ok((
                "apiVersion: v1\nkind: Pod\n".to_string(),
                serde_json::json!({"binary":"kubectl"}),
            ))])),
        };

        let payload =
            resources_snapshot_payload(&runner, root.path(), "atlas-run", "bijux-atlas-kind")
                .expect("resources payload");

        assert_eq!(payload["rows"][0]["category"], "resources");
        assert_eq!(payload["summary"]["errors"], 0);
    }
}
