// SPDX-License-Identifier: Apache-2.0

use crate::kubernetes::execution::KubernetesCommandRunner;
use serde_json::{json, Value};
use std::path::Path;
use std::time::Instant;

pub fn readiness_wait_commands(namespace: &str, timeout_seconds: u64) -> Vec<Vec<String>> {
    let timeout = format!("{timeout_seconds}s");
    vec![
        vec![
            "wait".to_string(),
            "deployment".to_string(),
            "--all".to_string(),
            "-n".to_string(),
            namespace.to_string(),
            "--for=condition=Available".to_string(),
            format!("--timeout={timeout}"),
        ],
        vec![
            "wait".to_string(),
            "pod".to_string(),
            "--all".to_string(),
            "-n".to_string(),
            namespace.to_string(),
            "--for=condition=Ready".to_string(),
            format!("--timeout={timeout}"),
        ],
    ]
}

pub fn readiness_wait_success_row(argv: &[String], stdout: &str, event: Value) -> Value {
    json!({
        "argv": argv,
        "stdout": stdout,
        "event": event,
        "status": "ok"
    })
}

pub fn readiness_wait_failure_row(argv: &[String]) -> Value {
    json!({
        "argv": argv,
        "status": "failed"
    })
}

pub fn readiness_wait_payload(rows: Vec<Value>, errors: &[String], elapsed_ms: u128) -> Value {
    json!({
        "schema_version": 1,
        "text": if errors.is_empty() { "k8s wait passed" } else { "k8s wait failed" },
        "rows": rows,
        "errors": errors,
        "summary": { "total": 1, "errors": errors.len(), "warnings": 0 },
        "elapsed_ms": elapsed_ms
    })
}

pub fn run_readiness_wait(
    runner: &impl KubernetesCommandRunner,
    repo_root: &Path,
    namespace: &str,
    timeout_seconds: u64,
) -> (Vec<Value>, Vec<String>, u128) {
    run_readiness_wait_with_policy(runner, repo_root, namespace, timeout_seconds, false)
}

pub fn run_readiness_wait_with_policy(
    runner: &impl KubernetesCommandRunner,
    repo_root: &Path,
    namespace: &str,
    timeout_seconds: u64,
    fail_fast: bool,
) -> (Vec<Value>, Vec<String>, u128) {
    let start = Instant::now();
    let mut rows = Vec::new();
    let mut errors = Vec::new();
    for argv in readiness_wait_commands(namespace, timeout_seconds) {
        match runner.run("kubectl", &argv, repo_root) {
            Ok(capture) => rows.push(readiness_wait_success_row(
                &argv,
                &capture.stdout,
                capture.event,
            )),
            Err(err) => {
                errors.push(err);
                rows.push(readiness_wait_failure_row(&argv));
                if fail_fast {
                    break;
                }
            }
        }
    }
    (rows, errors, start.elapsed().as_millis())
}

pub fn run_readiness_wait_payload(
    runner: &impl KubernetesCommandRunner,
    repo_root: &Path,
    namespace: &str,
    timeout_seconds: u64,
    fail_fast: bool,
) -> (Value, i32) {
    let (rows, errors, elapsed_ms) =
        run_readiness_wait_with_policy(runner, repo_root, namespace, timeout_seconds, fail_fast);
    let payload = readiness_wait_payload(rows, &errors, elapsed_ms);
    let exit_code = if errors.is_empty() { 0 } else { 1 };
    (payload, exit_code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kubernetes::execution::SubprocessCapture;
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::path::{Path, PathBuf};

    #[test]
    fn readiness_wait_commands_cover_deployments_and_pods() {
        let commands = readiness_wait_commands("bijux-atlas", 90);
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0][0], "wait");
        assert!(commands[0].contains(&"deployment".to_string()));
        assert!(commands[1].contains(&"pod".to_string()));
        assert!(commands[0].contains(&"--timeout=90s".to_string()));
    }

    #[test]
    fn readiness_wait_payload_tracks_failures() {
        let payload = readiness_wait_payload(
            vec![readiness_wait_failure_row(&[
                "wait".to_string(),
                "pod".to_string(),
            ])],
            &["pod wait failed".to_string()],
            1200,
        );
        assert_eq!(payload["text"], "k8s wait failed");
        assert_eq!(payload["summary"]["errors"], 1);
        assert_eq!(payload["elapsed_ms"], 1200);
    }

    struct MockRunner {
        cwd: PathBuf,
        calls: RefCell<Vec<Vec<String>>>,
        results: RefCell<VecDeque<Result<SubprocessCapture, String>>>,
    }

    impl crate::kubernetes::execution::KubernetesCommandRunner for MockRunner {
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

    #[test]
    fn run_readiness_wait_collects_rows_and_errors() {
        let root = tempfile::tempdir().expect("tempdir");
        let runner = MockRunner {
            cwd: root.path().to_path_buf(),
            calls: RefCell::new(Vec::new()),
            results: RefCell::new(VecDeque::from([
                Ok(SubprocessCapture {
                    stdout: "deployment available".to_string(),
                    event: json!({"binary": "kubectl"}),
                }),
                Err("pod wait failed".to_string()),
            ])),
        };

        let (rows, errors, elapsed_ms) = run_readiness_wait(&runner, root.path(), "atlas-kind", 30);

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["status"], "ok");
        assert_eq!(rows[1]["status"], "failed");
        assert_eq!(errors, vec!["pod wait failed".to_string()]);
        assert!(elapsed_ms < 5_000);
        assert_eq!(runner.calls.borrow().len(), 2);
    }

    #[test]
    fn run_readiness_wait_with_policy_stops_after_first_failure() {
        let root = tempfile::tempdir().expect("tempdir");
        let runner = MockRunner {
            cwd: root.path().to_path_buf(),
            calls: RefCell::new(Vec::new()),
            results: RefCell::new(VecDeque::from([
                Err("deployment wait failed".to_string()),
                Ok(SubprocessCapture {
                    stdout: "pod ready".to_string(),
                    event: json!({"binary": "kubectl"}),
                }),
            ])),
        };

        let (rows, errors, _elapsed_ms) =
            run_readiness_wait_with_policy(&runner, root.path(), "atlas-kind", 30, true);

        assert_eq!(rows.len(), 1);
        assert_eq!(errors, vec!["deployment wait failed".to_string()]);
        assert_eq!(runner.calls.borrow().len(), 1);
    }

    #[test]
    fn run_readiness_wait_payload_returns_failure_exit_code() {
        let root = tempfile::tempdir().expect("tempdir");
        let runner = MockRunner {
            cwd: root.path().to_path_buf(),
            calls: RefCell::new(Vec::new()),
            results: RefCell::new(VecDeque::from([Err("pod wait failed".to_string())])),
        };

        let (payload, exit_code) =
            run_readiness_wait_payload(&runner, root.path(), "atlas-kind", 30, true);

        assert_eq!(payload["text"], "k8s wait failed");
        assert_eq!(exit_code, 1);
    }
}
