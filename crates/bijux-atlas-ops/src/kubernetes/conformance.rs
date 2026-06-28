// SPDX-License-Identifier: Apache-2.0

use super::conformance_report::{build_conformance_report, write_conformance_report};
use crate::kubernetes::execution::KubernetesCommandRunner;
use crate::kubernetes::status_snapshot::read_namespace_resource_json;
use serde_json::Value;
use std::path::Path;

pub fn conformance_summary(deployments: &Value, pods: &Value) -> (Vec<String>, Vec<Value>) {
    let mut errors = Vec::new();
    let mut rows = Vec::new();
    if let Some(items) = deployments.get("items").and_then(Value::as_array) {
        for item in items {
            let name = item
                .get("metadata")
                .and_then(|value| value.get("name"))
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let desired = item
                .get("status")
                .and_then(|value| value.get("replicas"))
                .and_then(Value::as_u64)
                .unwrap_or(0);
            let ready = item
                .get("status")
                .and_then(|value| value.get("readyReplicas"))
                .and_then(Value::as_u64)
                .unwrap_or(0);
            if ready < desired {
                errors.push(format!("deployment `{name}` ready {ready}/{desired}"));
            }
            rows.push(serde_json::json!({
                "kind":"deployment",
                "name":name,
                "desired":desired,
                "ready":ready
            }));
        }
    }
    if let Some(items) = pods.get("items").and_then(Value::as_array) {
        for item in items {
            let name = item
                .get("metadata")
                .and_then(|value| value.get("name"))
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let phase = item
                .get("status")
                .and_then(|value| value.get("phase"))
                .and_then(Value::as_str)
                .unwrap_or("Unknown");
            if phase != "Running" && phase != "Succeeded" {
                errors.push(format!("pod `{name}` phase={phase}"));
            }
            rows.push(serde_json::json!({
                "kind":"pod",
                "name":name,
                "phase":phase
            }));
        }
    }
    (errors, rows)
}

fn hpa_enabled(runner: &impl KubernetesCommandRunner, repo_root: &Path, namespace: &str) -> bool {
    read_namespace_resource_json(runner, repo_root, namespace, "hpa")
        .ok()
        .and_then(|json| {
            json.get("items")
                .and_then(Value::as_array)
                .map(|items| !items.is_empty())
        })
        .unwrap_or(false)
}

fn custom_metrics_api_available(
    runner: &impl KubernetesCommandRunner,
    repo_root: &Path,
) -> Result<bool, String> {
    let args = vec![
        "api-resources".to_string(),
        "--api-group=custom.metrics.k8s.io".to_string(),
        "-o".to_string(),
        "name".to_string(),
    ];
    let capture = runner.run("kubectl", &args, repo_root)?;
    Ok(capture.stdout.lines().any(|line| !line.trim().is_empty()))
}

pub fn run_conformance_payload(
    runner: &impl KubernetesCommandRunner,
    repo_root: &Path,
    namespace: &str,
    run_id: &str,
    write_report: bool,
) -> Result<(Value, i32), String> {
    let deployments = read_namespace_resource_json(runner, repo_root, namespace, "deployments")?;
    let pods = read_namespace_resource_json(runner, repo_root, namespace, "pods")?;
    let (mut errors, mut rows) = conformance_summary(&deployments, &pods);
    if hpa_enabled(runner, repo_root, namespace) {
        match custom_metrics_api_available(runner, repo_root) {
            Ok(enabled) => {
                rows.push(serde_json::json!({"kind":"hpa_metrics_api","enabled":enabled}));
                if !enabled {
                    errors.push(
                        "hpa enabled but custom metrics API is not available (missing adapter)"
                            .to_string(),
                    );
                }
            }
            Err(err) => {
                rows.push(serde_json::json!({"kind":"hpa_metrics_api","enabled":false}));
                errors.push(format!(
                    "hpa enabled but custom metrics API probe failed: {err}"
                ));
            }
        }
    }
    let error_count = errors.len();
    let conformance_report = build_conformance_report(run_id, &errors);
    let mut report_path: Option<String> = None;
    if write_report {
        report_path = Some(
            write_conformance_report(repo_root, &conformance_report)?
                .display()
                .to_string(),
        );
    }
    let payload = serde_json::json!({
        "schema_version":1,
        "text": if errors.is_empty() {"k8s conformance passed"} else {"k8s conformance failed"},
        "rows": rows,
        "errors": errors,
        "conformance_report": conformance_report,
        "conformance_report_path": report_path,
        "summary":{"total":1,"errors": error_count,"warnings":0}
    });
    Ok((payload, if error_count == 0 { 0 } else { 1 }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kubernetes::execution::SubprocessCapture;
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use tempfile::tempdir;

    struct MockRunner {
        results: RefCell<VecDeque<Result<SubprocessCapture, String>>>,
    }

    impl KubernetesCommandRunner for MockRunner {
        fn run(
            &self,
            binary: &str,
            _args: &[String],
            _cwd: &Path,
        ) -> Result<SubprocessCapture, String> {
            assert_eq!(binary, "kubectl");
            self.results
                .borrow_mut()
                .pop_front()
                .expect("mock result should exist")
        }
    }

    #[test]
    fn conformance_summary_flags_unready_workloads() {
        let deployments = serde_json::json!({
            "items":[{"metadata":{"name":"atlas"},"status":{"replicas":2,"readyReplicas":1}}]
        });
        let pods = serde_json::json!({
            "items":[{"metadata":{"name":"atlas-1"},"status":{"phase":"Pending"}}]
        });

        let (errors, rows) = conformance_summary(&deployments, &pods);

        assert_eq!(rows.len(), 2);
        assert!(errors.iter().any(|entry| entry.contains("deployment")));
        assert!(errors.iter().any(|entry| entry.contains("pod")));
    }

    #[test]
    fn run_conformance_payload_collects_owner_report_and_rows() {
        let repo_root = tempdir().expect("temp dir should exist");
        std::fs::create_dir_all(repo_root.path().join("ops/k8s/generated"))
            .expect("generated path should exist");
        let runner = MockRunner {
            results: RefCell::new(VecDeque::from([
                Ok(SubprocessCapture {
                    stdout: serde_json::json!({
                        "items":[{"metadata":{"name":"atlas"},"status":{"replicas":1,"readyReplicas":1}}]
                    })
                    .to_string(),
                    event: serde_json::json!({}),
                }),
                Ok(SubprocessCapture {
                    stdout: serde_json::json!({
                        "items":[{"metadata":{"name":"atlas-0"},"status":{"phase":"Running"}}]
                    })
                    .to_string(),
                    event: serde_json::json!({}),
                }),
                Ok(SubprocessCapture {
                    stdout: serde_json::json!({"items":[]}).to_string(),
                    event: serde_json::json!({}),
                }),
            ])),
        };

        let (payload, exit_code) =
            run_conformance_payload(&runner, repo_root.path(), "bijux-atlas", "run-42", true)
                .expect("conformance payload should build");

        assert_eq!(exit_code, 0);
        assert_eq!(payload["text"], "k8s conformance passed");
        assert!(payload["conformance_report_path"].as_str().is_some());
        assert_eq!(payload["rows"].as_array().map(|rows| rows.len()), Some(2));
    }
}
