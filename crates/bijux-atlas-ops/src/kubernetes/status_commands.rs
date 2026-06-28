// SPDX-License-Identifier: Apache-2.0

use crate::kubernetes::execution::KubernetesCommandRunner;
use crate::kubernetes::status_snapshot::{
    cluster_status_row, endpoints_status_row, local_status_row, pods_status_row,
    read_namespace_resource_json,
};
use serde_json::Value;
use std::path::Path;

pub fn local_status_payload(
    repo_root: &Path,
    ops_root: &Path,
    profile: Value,
    toolchain: Value,
) -> (Value, String) {
    (
        local_status_row(repo_root, ops_root, profile.clone(), toolchain),
        format!(
            "ops status local: profile={} repo_root={} ops_root={}",
            profile
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            repo_root.display(),
            ops_root.display(),
        ),
    )
}

pub fn cluster_status_payload(
    runner: &impl KubernetesCommandRunner,
    repo_root: &Path,
    profile_name: &str,
) -> Result<(Value, String), String> {
    let value = read_namespace_resource_json(runner, repo_root, "bijux-atlas", "all")?;
    Ok((
        cluster_status_row(profile_name, value),
        "ops status k8s collected".to_string(),
    ))
}

pub fn pods_status_payload(
    runner: &impl KubernetesCommandRunner,
    repo_root: &Path,
    profile_name: &str,
) -> Result<(Value, String), String> {
    let value = read_namespace_resource_json(runner, repo_root, "bijux-atlas", "pods")?;
    Ok((
        pods_status_row(profile_name, value),
        "ops status pods collected".to_string(),
    ))
}

pub fn endpoints_status_payload(
    runner: &impl KubernetesCommandRunner,
    repo_root: &Path,
    profile_name: &str,
) -> Result<(Value, String), String> {
    let value = read_namespace_resource_json(runner, repo_root, "bijux-atlas", "endpoints")?;
    Ok((
        endpoints_status_row(profile_name, value),
        "ops status endpoints collected".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kubernetes::execution::SubprocessCapture;
    use serde_json::json;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    struct MockRunner {
        results: RefCell<VecDeque<Result<SubprocessCapture, String>>>,
    }

    impl KubernetesCommandRunner for MockRunner {
        fn run(
            &self,
            _binary: &str,
            _args: &[String],
            _cwd: &Path,
        ) -> Result<SubprocessCapture, String> {
            self.results
                .borrow_mut()
                .pop_front()
                .expect("mock result should exist")
        }
    }

    #[test]
    fn local_status_payload_keeps_profile_name_in_text() {
        let (_, text) = local_status_payload(
            Path::new("/repo"),
            Path::new("/repo/ops"),
            json!({"name": "kind"}),
            json!({"tools": []}),
        );

        assert_eq!(
            text,
            "ops status local: profile=kind repo_root=/repo ops_root=/repo/ops"
        );
    }

    #[test]
    fn cluster_status_payload_wraps_k8s_status_row() {
        let runner = MockRunner {
            results: RefCell::new(VecDeque::from([Ok(SubprocessCapture {
                stdout: "{\"items\":[]}".to_string(),
                event: json!({"binary":"kubectl"}),
            })])),
        };

        let (payload, text) =
            cluster_status_payload(&runner, Path::new("/repo"), "kind").expect("cluster payload");

        assert_eq!(text, "ops status k8s collected");
        assert_eq!(payload["target"], "k8s");
        assert_eq!(payload["profile"], "kind");
    }
}
