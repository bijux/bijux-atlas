// SPDX-License-Identifier: Apache-2.0

use crate::lifecycle::simulation::context::SimulationCommandRunner;
use crate::lifecycle::simulation::stack_down_payload;
use crate::workspace::ops_artifacts::{build_cleanup_payload, clean_ops_artifacts_payload};
use std::path::{Path, PathBuf};

pub fn cleanup_stack_state_payload<R: SimulationCommandRunner>(
    runner: Option<&R>,
    repo_root: &Path,
    ops_root: Option<PathBuf>,
    requested_profile: Option<&str>,
    force: bool,
) -> Result<(serde_json::Value, i32), String> {
    let (down_detail, down_code) = if let Some(runner) = runner {
        match stack_down_payload(runner, repo_root, ops_root, requested_profile, force) {
            Ok((payload, exit_code)) => (
                payload["text"].as_str().unwrap_or("down ok").to_string(),
                exit_code,
            ),
            Err(error) => (error, 1),
        }
    } else {
        ("down skipped (subprocess disabled)".to_string(), 0)
    };
    let (clean_payload, clean_code) = clean_ops_artifacts_payload(repo_root)?;
    let clean_detail = clean_payload["text"]
        .as_str()
        .unwrap_or("clean ok")
        .to_string();
    let payload = build_cleanup_payload(down_detail, down_code, clean_detail, clean_code);
    let errors = payload["summary"]["errors"].as_u64().unwrap_or(0);
    Ok((payload, if errors == 0 { 0 } else { 1 }))
}

#[cfg(test)]
mod tests {
    use super::cleanup_stack_state_payload;
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
    fn cleanup_payload_skips_down_when_runner_is_absent() {
        let root = tempfile::tempdir().expect("tempdir");
        let artifact_root = root.path().join("artifacts/atlas-dev/ops");
        std::fs::create_dir_all(&artifact_root).expect("mkdir artifacts");

        let (payload, exit_code) =
            cleanup_stack_state_payload::<MockRunner>(None, root.path(), None, None, false)
                .expect("cleanup payload");

        assert_eq!(exit_code, 0);
        assert_eq!(
            payload["rows"][0]["detail"],
            "down skipped (subprocess disabled)"
        );
        assert_eq!(payload["rows"][1]["status"], "ok");
    }

    #[test]
    fn cleanup_payload_composes_teardown_and_clean_results() {
        let root = tempfile::tempdir().expect("tempdir");
        let artifact_root = root.path().join("artifacts/atlas-dev/ops");
        std::fs::create_dir_all(&artifact_root).expect("mkdir artifacts");
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

        let (payload, exit_code) = cleanup_stack_state_payload(
            Some(&runner),
            root.path(),
            Some(ops_root),
            Some("kind"),
            false,
        )
        .expect("cleanup payload");

        assert_eq!(exit_code, 0);
        assert_eq!(payload["summary"]["errors"], 0);
        assert!(payload["rows"][0]["detail"]
            .as_str()
            .expect("detail")
            .contains("ops down deleted kind cluster"));
    }
}
