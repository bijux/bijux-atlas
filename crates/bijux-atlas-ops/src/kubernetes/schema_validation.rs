// SPDX-License-Identifier: Apache-2.0

use super::execution::KubernetesCommandRunner;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub fn run_kubeconform_validation(
    runner: &impl KubernetesCommandRunner,
    repo_root: &Path,
    rendered_manifest: &str,
) -> Result<(Vec<String>, Value), String> {
    let temporary_dir = repo_root.join("artifacts/tmp/k8s-validate");
    fs::create_dir_all(&temporary_dir)
        .map_err(|err| format!("failed to create {}: {err}", temporary_dir.display()))?;
    let manifest_path = temporary_dir.join("rendered.yaml");
    fs::write(&manifest_path, rendered_manifest)
        .map_err(|err| format!("failed to write {}: {err}", manifest_path.display()))?;
    let args = vec![
        "-strict".to_string(),
        "-ignore-missing-schemas".to_string(),
        "-summary".to_string(),
        manifest_path.display().to_string(),
    ];
    match runner.run("kubeconform", &args, repo_root) {
        Ok(result) => Ok((
            Vec::new(),
            json!({
                "tool":"kubeconform",
                "status":"ok",
                "stdout": result.stdout,
                "subprocess_event": result.event
            }),
        )),
        Err(err) => Ok((
            vec![format!("kubeconform validation failed: {err}")],
            json!({
                "tool":"kubeconform",
                "status":"failed",
                "error": err
            }),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kubernetes::execution::SubprocessCapture;
    use serde_json::json;
    use std::cell::RefCell;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    struct MockRunner {
        last_call: RefCell<Option<(String, Vec<String>, PathBuf)>>,
        result: RefCell<Result<SubprocessCapture, String>>,
    }

    impl MockRunner {
        fn success(stdout: &str) -> Self {
            Self {
                last_call: RefCell::new(None),
                result: RefCell::new(Ok(SubprocessCapture {
                    stdout: stdout.to_string(),
                    event: json!({"status":"ok"}),
                })),
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
            self.last_call.borrow_mut().replace((
                binary.to_string(),
                args.to_vec(),
                cwd.to_path_buf(),
            ));
            self.result.borrow().clone()
        }
    }

    #[test]
    fn kubeconform_validation_materializes_the_rendered_manifest() {
        let repo_root = tempdir().expect("temp dir should exist");
        let runner = MockRunner::success("summary");

        let (_, result) =
            run_kubeconform_validation(&runner, repo_root.path(), "kind: ConfigMap\n")
                .expect("validation call should succeed");

        let manifest_path = repo_root
            .path()
            .join("artifacts/tmp/k8s-validate/rendered.yaml");
        assert_eq!(
            fs::read_to_string(&manifest_path).expect("manifest should be written"),
            "kind: ConfigMap\n"
        );
        assert_eq!(result["status"], "ok");
        let call = runner.last_call.borrow();
        let (binary, args, cwd) = call.as_ref().expect("call should be recorded");
        assert_eq!(binary, "kubeconform");
        assert_eq!(cwd, repo_root.path());
        assert_eq!(
            args.last().expect("manifest path arg should exist"),
            &manifest_path.display().to_string()
        );
    }
}
