// SPDX-License-Identifier: Apache-2.0

use crate::kubernetes::execution::KubernetesCommandRunner;
use serde_json::{json, Value};
use std::path::Path;

pub fn k8s_plan_payload(
    profile: &str,
    run_id: &str,
    render_path: &str,
    render_index_path: &str,
    index: Value,
) -> Value {
    json!({
        "schema_version": 1,
        "text": format!("k8s plan profile={profile} run_id={run_id}"),
        "rows": [{
            "profile": profile,
            "run_id": run_id,
            "render_path": render_path,
            "render_index_path": render_index_path,
            "index": index
        }],
        "summary": {"total":1,"errors":0,"warnings":0}
    })
}

pub fn k8s_apply_payload(
    profile: &str,
    run_id: &str,
    dry_run: bool,
    render_path: &str,
    stdout: &str,
    subprocess_event: Value,
) -> Value {
    json!({
        "schema_version": 1,
        "text": if dry_run { "k8s dry-run completed" } else { "k8s apply completed" },
        "rows": [{
            "profile": profile,
            "run_id": run_id,
            "dry_run": dry_run,
            "render_path": render_path,
            "stdout": stdout,
            "subprocess_event": subprocess_event
        }],
        "summary": {"total":1,"errors":0,"warnings":0}
    })
}

pub fn k8s_logs_payload(stdout: &str, subprocess_event: Value) -> Value {
    json!({
        "schema_version": 1,
        "text": "k8s logs collected",
        "rows": [{
            "stdout": stdout,
            "event": subprocess_event
        }],
        "summary": {"total":1,"errors":0,"warnings":0}
    })
}

pub fn run_k8s_apply_payload(
    runner: &impl KubernetesCommandRunner,
    repo_root: &Path,
    profile: &str,
    run_id: &str,
    namespace: &str,
    render_path: &str,
    dry_run: bool,
) -> Result<Value, String> {
    let mut args = vec![
        "apply".to_string(),
        "-n".to_string(),
        namespace.to_string(),
        "-f".to_string(),
        render_path.to_string(),
    ];
    if dry_run {
        args.push("--dry-run=client".to_string());
    }
    let capture = runner.run("kubectl", &args, repo_root)?;
    Ok(k8s_apply_payload(
        profile,
        run_id,
        dry_run,
        render_path,
        &capture.stdout,
        capture.event,
    ))
}

pub fn run_k8s_logs_payload(
    runner: &impl KubernetesCommandRunner,
    repo_root: &Path,
    namespace: &str,
    pod: &str,
    tail: usize,
) -> Result<Value, String> {
    let args = vec![
        "logs".to_string(),
        "-n".to_string(),
        namespace.to_string(),
        pod.to_string(),
        format!("--tail={tail}"),
    ];
    let capture = runner.run("kubectl", &args, repo_root)?;
    Ok(k8s_logs_payload(&capture.stdout, capture.event))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kubernetes::execution::SubprocessCapture;
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::path::Path;

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
    fn plan_payload_preserves_render_pointers() {
        let payload = k8s_plan_payload(
            "kind",
            "atlas-run",
            "artifacts/ops/atlas-run/render/kind/helm/render.yaml",
            "artifacts/ops/atlas-run/render/kind/helm/render.index.json",
            json!({"files":[]}),
        );

        assert_eq!(payload["rows"][0]["profile"], "kind");
        assert_eq!(payload["rows"][0]["run_id"], "atlas-run");
        assert!(payload["text"]
            .as_str()
            .is_some_and(|text| text.contains("k8s plan")));
    }

    #[test]
    fn apply_payload_tracks_dry_run_status() {
        let payload = k8s_apply_payload(
            "kind",
            "atlas-run",
            true,
            "render.yaml",
            "applied",
            json!({"binary":"kubectl"}),
        );

        assert_eq!(payload["text"], "k8s dry-run completed");
        assert_eq!(payload["rows"][0]["dry_run"], true);
    }

    #[test]
    fn logs_payload_wraps_stdout_and_event() {
        let payload = k8s_logs_payload("hello", json!({"binary":"kubectl"}));
        assert_eq!(payload["text"], "k8s logs collected");
        assert_eq!(payload["rows"][0]["stdout"], "hello");
        assert_eq!(payload["rows"][0]["event"]["binary"], "kubectl");
    }

    #[test]
    fn run_k8s_apply_payload_executes_owned_apply_contract() {
        let runner = MockRunner {
            results: RefCell::new(VecDeque::from([Ok(SubprocessCapture {
                stdout: "configured".to_string(),
                event: json!({"binary":"kubectl"}),
            })])),
        };

        let payload = run_k8s_apply_payload(
            &runner,
            Path::new("/repo"),
            "kind",
            "atlas-run",
            "bijux-atlas",
            "render.yaml",
            true,
        )
        .expect("apply payload should build");

        assert_eq!(payload["text"], "k8s dry-run completed");
        assert_eq!(payload["rows"][0]["render_path"], "render.yaml");
    }

    #[test]
    fn run_k8s_logs_payload_executes_owned_logs_contract() {
        let runner = MockRunner {
            results: RefCell::new(VecDeque::from([Ok(SubprocessCapture {
                stdout: "hello".to_string(),
                event: json!({"binary":"kubectl"}),
            })])),
        };

        let payload = run_k8s_logs_payload(
            &runner,
            Path::new("/repo"),
            "bijux-atlas",
            "deployment/bijux-atlas",
            25,
        )
        .expect("logs payload should build");

        assert_eq!(payload["text"], "k8s logs collected");
        assert_eq!(payload["rows"][0]["stdout"], "hello");
    }
}
