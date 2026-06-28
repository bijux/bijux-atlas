// SPDX-License-Identifier: Apache-2.0

use crate::kubernetes::execution::KubernetesCommandRunner;
use serde_json::{json, Value};
use std::path::Path;

pub fn local_status_row(
    repo_root: &Path,
    ops_root: &Path,
    profile: Value,
    toolchain: Value,
) -> Value {
    json!({
        "schema_version": 1,
        "target": "local",
        "repo_root": repo_root.display().to_string(),
        "ops_root": ops_root.display().to_string(),
        "profile": profile,
        "toolchain": toolchain,
    })
}

pub fn read_namespace_resource_json(
    runner: &impl KubernetesCommandRunner,
    repo_root: &Path,
    namespace: &str,
    resource: &str,
) -> Result<Value, String> {
    let args = vec![
        "get".to_string(),
        resource.to_string(),
        "-n".to_string(),
        namespace.to_string(),
        "-o".to_string(),
        "json".to_string(),
    ];
    let capture = runner.run("kubectl", &args, repo_root)?;
    serde_json::from_str(&capture.stdout)
        .map_err(|err| format!("failed to parse kubectl {resource} json: {err}"))
}

pub fn cluster_status_row(profile_name: &str, resources: Value) -> Value {
    json!({
        "schema_version": 1,
        "target": "k8s",
        "profile": profile_name,
        "resources": resources,
    })
}

pub fn endpoints_status_row(profile_name: &str, resources: Value) -> Value {
    json!({
        "schema_version": 1,
        "target": "endpoints",
        "profile": profile_name,
        "resources": resources,
    })
}

pub fn pods_status_row(profile_name: &str, value: Value) -> Value {
    let mut pods = value
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    pods.sort_by(|a, b| {
        a.get("metadata")
            .and_then(|meta| meta.get("name"))
            .and_then(Value::as_str)
            .cmp(
                &b.get("metadata")
                    .and_then(|meta| meta.get("name"))
                    .and_then(Value::as_str),
            )
    });
    json!({
        "schema_version": 1,
        "target": "pods",
        "profile": profile_name,
        "pods": pods,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kubernetes::execution::SubprocessCapture;
    use std::cell::RefCell;
    use std::collections::VecDeque;

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
    fn local_status_row_preserves_owned_paths() {
        let row = local_status_row(
            Path::new("/repo"),
            Path::new("/repo/ops"),
            json!({"name":"kind"}),
            json!({"tools":[]}),
        );

        assert_eq!(row["target"], "local");
        assert_eq!(row["repo_root"], "/repo");
        assert_eq!(row["ops_root"], "/repo/ops");
    }

    #[test]
    fn read_namespace_resource_json_parses_runner_stdout() {
        let runner = MockRunner {
            results: RefCell::new(VecDeque::from([Ok(SubprocessCapture {
                stdout: "{\"items\":[]}".to_string(),
                event: json!({"binary":"kubectl"}),
            })])),
        };

        let value = read_namespace_resource_json(&runner, Path::new("/repo"), "atlas", "pods")
            .expect("pods json should parse");

        assert_eq!(value["items"], json!([]));
    }

    #[test]
    fn pods_status_row_sorts_items_by_name() {
        let row = pods_status_row(
            "kind",
            json!({
                "items": [
                    {"metadata": {"name": "pod-b"}},
                    {"metadata": {"name": "pod-a"}}
                ]
            }),
        );

        assert_eq!(row["pods"][0]["metadata"]["name"], "pod-a");
        assert_eq!(row["pods"][1]["metadata"]["name"], "pod-b");
    }
}
