// SPDX-License-Identifier: Apache-2.0

use crate::lifecycle::simulation::context::SimulationCommandRunner;
use std::path::{Path, PathBuf};

pub fn stack_down_payload(
    runner: &impl SimulationCommandRunner,
    repo_root: &Path,
    ops_root: Option<PathBuf>,
    requested_profile: Option<&str>,
    force: bool,
) -> Result<(serde_json::Value, i32), String> {
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
    let expected_context = format!("kind-{}", profile.kind_profile);
    let current_context = runner
        .run(
            "kubectl",
            &["config".to_string(), "current-context".to_string()],
            repo_root,
        )
        .map(|(stdout, _)| stdout.trim().to_string())
        .unwrap_or_default();
    if current_context != expected_context && !force {
        return Err(format!(
            "context guard failed: expected `{expected_context}` got `{current_context}`; pass --force to override"
        ));
    }
    runner.run(
        "kind",
        &[
            "delete".to_string(),
            "cluster".to_string(),
            "--name".to_string(),
            profile.kind_profile.clone(),
        ],
        repo_root,
    )?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "text": format!("ops down deleted kind cluster `{}`", profile.kind_profile),
        "rows": [{
            "kind_profile": profile.kind_profile,
            "expected_context": expected_context,
            "current_context": current_context,
        }],
        "summary": {"total": 1, "errors": 0, "warnings": 0}
    });
    Ok((payload, 0))
}

#[cfg(test)]
mod tests {
    use super::stack_down_payload;
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
    fn stack_down_payload_deletes_owned_kind_cluster() {
        let root = tempfile::tempdir().expect("tempdir");
        let ops_root = write_profiles(root.path());
        let runner = MockRunner {
            results: RefCell::new(VecDeque::from([
                Ok((
                    "kind-atlas-kind\n".to_string(),
                    serde_json::json!({"command":"current-context"}),
                )),
                Ok((
                    "deleted\n".to_string(),
                    serde_json::json!({"command":"delete-cluster"}),
                )),
            ])),
        };

        let (payload, exit_code) =
            stack_down_payload(&runner, root.path(), Some(ops_root), Some("kind"), false)
                .expect("down payload");

        assert_eq!(exit_code, 0);
        assert_eq!(payload["summary"]["errors"], 0);
        assert_eq!(payload["rows"][0]["kind_profile"], "atlas-kind");
    }

    #[test]
    fn stack_down_payload_enforces_context_guard_without_force() {
        let root = tempfile::tempdir().expect("tempdir");
        let ops_root = write_profiles(root.path());
        let runner = MockRunner {
            results: RefCell::new(VecDeque::from([Ok((
                "kind-other\n".to_string(),
                serde_json::json!({"command":"current-context"}),
            ))])),
        };

        let error = stack_down_payload(&runner, root.path(), Some(ops_root), Some("kind"), false)
            .expect_err("context guard");

        assert!(error.contains("context guard failed"));
        assert!(error.contains("kind-atlas-kind"));
    }
}
