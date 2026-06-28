// SPDX-License-Identifier: Apache-2.0

use crate::kubernetes::access_guard::ensure_namespace_guard;
use crate::kubernetes::command_reports::{
    k8s_plan_payload, run_k8s_apply_payload, run_k8s_logs_payload,
};
use crate::kubernetes::conformance::run_conformance_payload;
use crate::kubernetes::execution::KubernetesCommandRunner;
use crate::kubernetes::port_forward::port_forward_payload;
use crate::kubernetes::service_inventory::{read_service_port_rows, service_port_payload};
use crate::kubernetes::workload_wait::run_readiness_wait_payload;
use serde_json::Value;
use std::path::Path;

const ATLAS_NAMESPACE: &str = "bijux-atlas";

pub fn k8s_plan_command_payload(
    profile_name: &str,
    run_id: &str,
    render_path: &Path,
    index_path: &Path,
    index_json: Value,
) -> Value {
    k8s_plan_payload(
        profile_name,
        run_id,
        &render_path.display().to_string(),
        &index_path.display().to_string(),
        index_json,
    )
}

pub fn k8s_apply_command_payload(
    runner: &impl KubernetesCommandRunner,
    repo_root: &Path,
    profile_name: &str,
    kind_profile: &str,
    run_id: &str,
    render_path: &Path,
    dry_run: bool,
    force: bool,
) -> Result<Value, String> {
    if !dry_run {
        ensure_namespace_guard(runner, repo_root, kind_profile, force, ATLAS_NAMESPACE)?;
    }
    run_k8s_apply_payload(
        runner,
        repo_root,
        profile_name,
        run_id,
        ATLAS_NAMESPACE,
        &render_path.display().to_string(),
        dry_run,
    )
}

pub fn k8s_conformance_command_payload(
    runner: &impl KubernetesCommandRunner,
    repo_root: &Path,
    kind_profile: &str,
    run_id: &str,
    allow_write: bool,
    force: bool,
) -> Result<(Value, i32), String> {
    ensure_namespace_guard(runner, repo_root, kind_profile, force, ATLAS_NAMESPACE)?;
    run_conformance_payload(runner, repo_root, ATLAS_NAMESPACE, run_id, allow_write)
}

pub fn k8s_ports_command_payload(
    runner: &impl KubernetesCommandRunner,
    repo_root: &Path,
    kind_profile: &str,
    force: bool,
) -> Result<Value, String> {
    ensure_namespace_guard(runner, repo_root, kind_profile, force, ATLAS_NAMESPACE)?;
    let (rows, svc_event) = read_service_port_rows(runner, repo_root, ATLAS_NAMESPACE)?;
    Ok(service_port_payload(rows, svc_event))
}

pub fn k8s_wait_command_payload(
    runner: &impl KubernetesCommandRunner,
    repo_root: &Path,
    kind_profile: &str,
    timeout_seconds: u64,
    fail_fast: bool,
    force: bool,
) -> Result<(Value, i32), String> {
    ensure_namespace_guard(runner, repo_root, kind_profile, force, ATLAS_NAMESPACE)?;
    Ok(run_readiness_wait_payload(
        runner,
        repo_root,
        ATLAS_NAMESPACE,
        timeout_seconds,
        fail_fast,
    ))
}

pub fn k8s_logs_command_payload(
    runner: &impl KubernetesCommandRunner,
    repo_root: &Path,
    kind_profile: &str,
    pod: &str,
    tail: usize,
    force: bool,
) -> Result<Value, String> {
    ensure_namespace_guard(runner, repo_root, kind_profile, force, ATLAS_NAMESPACE)?;
    run_k8s_logs_payload(runner, repo_root, ATLAS_NAMESPACE, pod, tail)
}

pub fn k8s_port_forward_command_payload(
    runner: &impl KubernetesCommandRunner,
    repo_root: &Path,
    kind_profile: &str,
    resource: &str,
    local_port: u16,
    remote_port: u16,
    force: bool,
) -> Result<Value, String> {
    ensure_namespace_guard(runner, repo_root, kind_profile, force, ATLAS_NAMESPACE)?;
    Ok(port_forward_payload(resource, local_port, remote_port))
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
    fn k8s_plan_command_payload_keeps_owned_paths() {
        let payload = k8s_plan_command_payload(
            "kind",
            "atlas-run",
            Path::new("render.yaml"),
            Path::new("render.index.json"),
            json!({"files":[]}),
        );

        assert_eq!(payload["rows"][0]["render_path"], "render.yaml");
        assert_eq!(payload["rows"][0]["render_index_path"], "render.index.json");
    }

    #[test]
    fn k8s_logs_command_payload_uses_owned_namespace_contract() {
        let runner = MockRunner {
            results: RefCell::new(VecDeque::from([
                Ok(SubprocessCapture {
                    stdout: "kind-normal\n".to_string(),
                    event: json!({}),
                }),
                Ok(SubprocessCapture {
                    stdout: "namespace/bijux-atlas\n".to_string(),
                    event: json!({}),
                }),
                Ok(SubprocessCapture {
                    stdout: "hello".to_string(),
                    event: json!({"binary":"kubectl"}),
                }),
            ])),
        };

        let payload = k8s_logs_command_payload(
            &runner,
            Path::new("/repo"),
            "normal",
            "deployment/bijux-atlas",
            25,
            false,
        )
        .expect("logs payload should build");

        assert_eq!(payload["text"], "k8s logs collected");
        assert_eq!(payload["rows"][0]["stdout"], "hello");
    }
}
