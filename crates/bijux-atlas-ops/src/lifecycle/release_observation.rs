// SPDX-License-Identifier: Apache-2.0

use crate::kubernetes::execution::KubernetesCommandRunner;
use serde_json::Value;
use std::path::Path;

fn kubectl_json_capture(
    runner: &impl KubernetesCommandRunner,
    repo_root: &Path,
    namespace: &str,
    resource_args: &[&str],
) -> Result<String, String> {
    let mut argv = resource_args
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    argv.extend([
        "-n".to_string(),
        namespace.to_string(),
        "-o".to_string(),
        "json".to_string(),
    ]);
    runner
        .run("kubectl", &argv, repo_root)
        .map(|capture| capture.stdout)
}

pub fn deployment_revision(
    runner: &impl KubernetesCommandRunner,
    repo_root: &Path,
    namespace: &str,
) -> Option<i64> {
    let stdout = kubectl_json_capture(
        runner,
        repo_root,
        namespace,
        &["get", "deployment", "bijux-atlas"],
    )
    .ok()?;
    let json: Value = serde_json::from_str(&stdout).ok()?;
    json.get("metadata")
        .and_then(|row| row.get("annotations"))
        .and_then(|row| row.get("deployment.kubernetes.io/revision"))
        .and_then(Value::as_str)
        .and_then(|value| value.parse::<i64>().ok())
}

pub fn rollout_history(
    runner: &impl KubernetesCommandRunner,
    repo_root: &Path,
    namespace: &str,
) -> Value {
    let argv = vec![
        "rollout".to_string(),
        "history".to_string(),
        "deployment/bijux-atlas".to_string(),
        "-n".to_string(),
        namespace.to_string(),
    ];
    match runner.run("kubectl", &argv, repo_root) {
        Ok(capture) => serde_json::json!({
            "status": "ok",
            "stdout": capture.stdout,
            "event": capture.event
        }),
        Err(err) => serde_json::json!({
            "status": "failed",
            "error": err
        }),
    }
}

pub fn pods_restart_count(
    runner: &impl KubernetesCommandRunner,
    repo_root: &Path,
    namespace: &str,
) -> u64 {
    let Ok(stdout) = kubectl_json_capture(runner, repo_root, namespace, &["get", "pods"]) else {
        return 0;
    };
    let Ok(json) = serde_json::from_str::<Value>(&stdout) else {
        return 0;
    };
    json.get("items")
        .and_then(Value::as_array)
        .map(|rows| {
            rows.iter()
                .flat_map(|row| {
                    row.get("status")
                        .and_then(|status| status.get("containerStatuses"))
                        .and_then(Value::as_array)
                        .cloned()
                        .unwrap_or_default()
                })
                .filter_map(|container| container.get("restartCount").and_then(Value::as_u64))
                .sum()
        })
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kubernetes::execution::SubprocessCapture;
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::path::PathBuf;

    struct MockRunner {
        cwd: PathBuf,
        calls: RefCell<Vec<Vec<String>>>,
        results: RefCell<VecDeque<Result<SubprocessCapture, String>>>,
    }

    impl KubernetesCommandRunner for MockRunner {
        fn run(
            &self,
            binary: &str,
            args: &[String],
            cwd: &Path,
        ) -> Result<SubprocessCapture, String> {
            assert_eq!(binary, "kubectl");
            assert_eq!(cwd, self.cwd);
            self.calls.borrow_mut().push(args.to_vec());
            self.results
                .borrow_mut()
                .pop_front()
                .expect("mock result should exist")
        }
    }

    fn capture(stdout: &str) -> Result<SubprocessCapture, String> {
        Ok(SubprocessCapture {
            stdout: stdout.to_string(),
            event: serde_json::json!({"binary": "kubectl"}),
        })
    }

    #[test]
    fn deployment_revision_reads_rollout_annotation() {
        let root = tempfile::tempdir().expect("tempdir");
        let runner = MockRunner {
            cwd: root.path().to_path_buf(),
            calls: RefCell::new(Vec::new()),
            results: RefCell::new(VecDeque::from([capture(
                r#"{"metadata":{"annotations":{"deployment.kubernetes.io/revision":"17"}}}"#,
            )])),
        };

        let revision = deployment_revision(&runner, root.path(), "atlas-kind");

        assert_eq!(revision, Some(17));
        assert_eq!(
            runner.calls.borrow().first(),
            Some(&vec![
                "get".to_string(),
                "deployment".to_string(),
                "bijux-atlas".to_string(),
                "-n".to_string(),
                "atlas-kind".to_string(),
                "-o".to_string(),
                "json".to_string(),
            ])
        );
    }

    #[test]
    fn rollout_history_records_failures_without_panicking() {
        let root = tempfile::tempdir().expect("tempdir");
        let runner = MockRunner {
            cwd: root.path().to_path_buf(),
            calls: RefCell::new(Vec::new()),
            results: RefCell::new(VecDeque::from([Err(
                "kubectl rollout history failed".to_string()
            )])),
        };

        let payload = rollout_history(&runner, root.path(), "atlas-kind");

        assert_eq!(payload["status"], "failed");
        assert_eq!(payload["error"], "kubectl rollout history failed");
    }

    #[test]
    fn pods_restart_count_sums_container_restart_counters() {
        let root = tempfile::tempdir().expect("tempdir");
        let runner = MockRunner {
            cwd: root.path().to_path_buf(),
            calls: RefCell::new(Vec::new()),
            results: RefCell::new(VecDeque::from([capture(
                r#"{
                    "items":[
                        {"status":{"containerStatuses":[{"restartCount":2},{"restartCount":1}]}},
                        {"status":{"containerStatuses":[{"restartCount":4}]}}
                    ]
                }"#,
            )])),
        };

        let restart_count = pods_restart_count(&runner, root.path(), "atlas-kind");

        assert_eq!(restart_count, 7);
    }
}
