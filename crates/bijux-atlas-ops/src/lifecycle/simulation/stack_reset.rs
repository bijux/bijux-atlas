// SPDX-License-Identifier: Apache-2.0

use crate::lifecycle::simulation::context::SimulationCommandRunner;
use crate::workspace::ops_artifacts::{build_reset_payload, ops_artifact_run_root};
use std::path::{Path, PathBuf};

pub fn reset_stack_state_payload<R: SimulationCommandRunner>(
    runner: Option<&R>,
    repo_root: &Path,
    reset_run_id: &str,
    ops_root: Option<PathBuf>,
    requested_profile: Option<&str>,
) -> Result<(serde_json::Value, i32), String> {
    let target = ops_artifact_run_root(repo_root, reset_run_id)?;
    if target.exists() {
        std::fs::remove_dir_all(&target)
            .map_err(|err| format!("failed to remove {}: {err}", target.display()))?;
    }
    let mut rows = vec![serde_json::json!({
        "kind": "artifacts",
        "status": "ok",
        "path": target.display().to_string()
    })];
    if let Some(runner) = runner {
        let ops_root = crate::workspace::profiles::resolve_ops_root(repo_root, ops_root)
            .map_err(|err| err.detail())?;
        let mut profiles =
            crate::workspace::profiles::load_profiles(&ops_root).map_err(|err| err.detail())?;
        profiles.sort_by(|left, right| left.name.cmp(&right.name));
        let profile = crate::workspace::profiles::resolve_profile(
            requested_profile.map(str::to_string),
            &profiles,
        )
        .map_err(|err| err.detail())?;
        let namespace_delete_args = vec![
            "delete".to_string(),
            "namespace".to_string(),
            "bijux-atlas".to_string(),
            "--ignore-not-found=true".to_string(),
        ];
        let _ = runner.run("kubectl", &namespace_delete_args, repo_root);
        let kind_delete_args = vec![
            "delete".to_string(),
            "cluster".to_string(),
            "--name".to_string(),
            profile.kind_profile.clone(),
        ];
        let _ = runner.run("kind", &kind_delete_args, repo_root);
        rows.push(serde_json::json!({
            "kind": "known_resources",
            "status": "attempted",
            "namespace": "bijux-atlas",
            "kind_profile": profile.kind_profile
        }));
    }
    Ok((build_reset_payload(reset_run_id, &target, rows), 0))
}

#[cfg(test)]
mod tests {
    use super::reset_stack_state_payload;
    use crate::lifecycle::simulation::context::SimulationCommandRunner;
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::path::Path;

    struct MockRunner {
        results: RefCell<VecDeque<Result<(String, serde_json::Value), String>>>,
    }

    impl SimulationCommandRunner for MockRunner {
        fn run(
            &self,
            _binary: &str,
            _args: &[String],
            _cwd: &Path,
        ) -> Result<(String, serde_json::Value), String> {
            self.results
                .borrow_mut()
                .pop_front()
                .expect("runner result")
        }
    }

    fn write_profiles(root: &std::path::Path) -> std::path::PathBuf {
        let ops_root = root.join("ops");
        std::fs::create_dir_all(ops_root.join("stack")).expect("mkdir stack");
        std::fs::write(
            ops_root.join("stack/profiles.json"),
            r#"{"schema_version":1,"profiles":[{"name":"kind","kind_profile":"atlas-kind","cluster_config":"ops/kind/kind.yaml"}]}"#,
        )
        .expect("write profiles");
        ops_root
    }

    #[test]
    fn reset_payload_removes_owned_artifacts_without_subprocess() {
        let root = tempfile::tempdir().expect("tempdir");
        let target = root.path().join("artifacts/atlas-dev/ops/owned-run");
        std::fs::create_dir_all(&target).expect("mkdir target");
        std::fs::write(target.join("marker.json"), "{}").expect("write marker");

        let (payload, exit_code) =
            reset_stack_state_payload::<MockRunner>(None, root.path(), "owned-run", None, None)
                .expect("reset payload");

        assert_eq!(exit_code, 0);
        assert_eq!(payload["rows"][0]["kind"], "artifacts");
        assert!(!target.exists());
    }

    #[test]
    fn reset_payload_records_known_resource_cleanup_attempts() {
        let root = tempfile::tempdir().expect("tempdir");
        let ops_root = write_profiles(root.path());
        let runner = MockRunner {
            results: RefCell::new(VecDeque::from([
                Ok((
                    "namespace deleted\n".to_string(),
                    serde_json::json!({"command":"kubectl-delete"}),
                )),
                Ok((
                    "cluster deleted\n".to_string(),
                    serde_json::json!({"command":"kind-delete"}),
                )),
            ])),
        };

        let (payload, exit_code) = reset_stack_state_payload(
            Some(&runner),
            root.path(),
            "owned-run",
            Some(ops_root),
            Some("kind"),
        )
        .expect("reset payload");

        assert_eq!(exit_code, 0);
        assert_eq!(payload["rows"][1]["kind"], "known_resources");
        assert_eq!(payload["rows"][1]["status"], "attempted");
    }
}
