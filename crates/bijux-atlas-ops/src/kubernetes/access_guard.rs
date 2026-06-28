// SPDX-License-Identifier: Apache-2.0

use super::execution::KubernetesCommandRunner;
use super::safety_policy::{expected_kind_context, ClusterSafetyPolicy};
use crate::lifecycle::simulation::paths::simulation_cluster_context;
use std::path::Path;

pub fn ensure_kind_context(
    runner: &impl KubernetesCommandRunner,
    kind_profile: &str,
    force: bool,
) -> Result<(), String> {
    let args = vec!["config".to_string(), "current-context".to_string()];
    let current = runner.run("kubectl", &args, Path::new("."))?.stdout;
    let current = current.trim();
    let policy = ClusterSafetyPolicy::for_kind_profile(kind_profile, "bijux-atlas");
    if policy.allows_context(current, force) {
        Ok(())
    } else {
        Err(policy.context_guard_message(current))
    }
}

pub fn ensure_namespace_exists(
    runner: &impl KubernetesCommandRunner,
    namespace: &str,
    dry_run: &str,
) -> Result<(), String> {
    let get_args = vec![
        "get".to_string(),
        "namespace".to_string(),
        namespace.to_string(),
        "-o".to_string(),
        "name".to_string(),
    ];
    if runner.run("kubectl", &get_args, Path::new(".")).is_ok() {
        return Ok(());
    }
    let mut create_args = vec![
        "create".to_string(),
        "namespace".to_string(),
        namespace.to_string(),
    ];
    if dry_run == "client" {
        create_args.push("--dry-run=client".to_string());
    }
    runner.run("kubectl", &create_args, Path::new("."))?;
    Ok(())
}

pub fn ensure_namespace_guard(
    runner: &impl KubernetesCommandRunner,
    repo_root: &Path,
    kind_profile: &str,
    force: bool,
    namespace: &str,
) -> Result<(), String> {
    let policy = ClusterSafetyPolicy::for_kind_profile(kind_profile, namespace);
    ensure_kind_context(runner, kind_profile, force)?;
    let args = vec![
        "get".to_string(),
        "namespace".to_string(),
        namespace.to_string(),
        "-o".to_string(),
        "name".to_string(),
    ];
    runner
        .run("kubectl", &args, repo_root)
        .map(|_| ())
        .map_err(|err| policy.namespace_guard_message(&err))
}

pub fn expected_cluster_context(kind_profile: &str) -> String {
    expected_kind_context(kind_profile)
}

pub fn ensure_simulation_cluster_context(
    runner: &impl KubernetesCommandRunner,
    force: bool,
) -> Result<(), String> {
    let args = vec!["config".to_string(), "current-context".to_string()];
    let current = runner.run("kubectl", &args, Path::new("."))?.stdout;
    let current = current.trim();
    let expected = simulation_cluster_context();
    if current == expected || force {
        Ok(())
    } else {
        Err(format!(
            "kubectl context guard failed: expected `{expected}` got `{current}`; pass --force to override"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kubernetes::execution::SubprocessCapture;
    use serde_json::json;
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::path::{Path, PathBuf};

    #[derive(Debug, Clone)]
    struct RecordedCall {
        binary: String,
        args: Vec<String>,
        cwd: PathBuf,
    }

    struct MockRunner {
        calls: RefCell<Vec<RecordedCall>>,
        results: RefCell<VecDeque<Result<SubprocessCapture, String>>>,
    }

    impl MockRunner {
        fn with_results(results: Vec<Result<SubprocessCapture, String>>) -> Self {
            Self {
                calls: RefCell::new(Vec::new()),
                results: RefCell::new(results.into()),
            }
        }
    }

    impl KubernetesCommandRunner for MockRunner {
        fn run(
            &self,
            binary: &str,
            args: &[String],
            cwd: &Path,
        ) -> Result<SubprocessCapture, String> {
            self.calls.borrow_mut().push(RecordedCall {
                binary: binary.to_string(),
                args: args.to_vec(),
                cwd: cwd.to_path_buf(),
            });
            self.results
                .borrow_mut()
                .pop_front()
                .expect("mock result should be configured")
        }
    }

    fn ok(stdout: &str) -> Result<SubprocessCapture, String> {
        Ok(SubprocessCapture {
            stdout: stdout.to_string(),
            event: json!({"status":"ok"}),
        })
    }

    #[test]
    fn namespace_guard_accepts_matching_context() {
        let runner =
            MockRunner::with_results(vec![ok("kind-normal\n"), ok("namespace/bijux-atlas\n")]);
        ensure_namespace_guard(
            &runner,
            Path::new("/tmp/repo"),
            "normal",
            false,
            "bijux-atlas",
        )
        .expect("guard should pass");
        let calls = runner.calls.borrow();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].binary, "kubectl");
        assert_eq!(calls[0].args, vec!["config", "current-context"]);
        assert_eq!(calls[1].cwd, Path::new("/tmp/repo"));
    }

    #[test]
    fn namespace_guard_reports_context_mismatch() {
        let runner = MockRunner::with_results(vec![ok("prod-cluster\n")]);
        let err = ensure_namespace_guard(
            &runner,
            Path::new("/tmp/repo"),
            "normal",
            false,
            "bijux-atlas",
        )
        .expect_err("guard should reject a mismatched context");
        assert!(err.contains("kubectl context guard failed"));
        assert!(err.contains("kind-normal"));
    }

    #[test]
    fn namespace_creation_uses_client_dry_run_when_requested() {
        let runner = MockRunner::with_results(vec![
            Err("missing namespace".to_string()),
            ok("namespace/bijux-atlas created"),
        ]);
        ensure_namespace_exists(&runner, "bijux-atlas", "client")
            .expect("namespace create should succeed");
        let calls = runner.calls.borrow();
        assert_eq!(calls.len(), 2);
        assert!(calls[1].args.contains(&"--dry-run=client".to_string()));
    }

    #[test]
    fn simulation_context_guard_accepts_owned_simulation_cluster() {
        let runner = MockRunner::with_results(vec![ok("kind-bijux-atlas-sim\n")]);

        ensure_simulation_cluster_context(&runner, false).expect("simulation guard should pass");

        assert_eq!(runner.calls.borrow().len(), 1);
    }

    #[test]
    fn simulation_context_guard_reports_context_mismatch() {
        let runner = MockRunner::with_results(vec![ok("kind-normal\n")]);

        let error = ensure_simulation_cluster_context(&runner, false)
            .expect_err("simulation guard should reject a mismatched context");

        assert!(error.contains("kind-bijux-atlas-sim"));
        assert!(error.contains("kind-normal"));
    }
}
