// SPDX-License-Identifier: Apache-2.0

use super::*;

pub(super) fn run_kubeconform_validation(
    process: &OpsProcess,
    repo_root: &Path,
    rendered: &str,
) -> Result<(Vec<String>, Value), String> {
    let tmp_dir = repo_root.join("artifacts/tmp/k8s-validate");
    fs::create_dir_all(&tmp_dir)
        .map_err(|err| format!("failed to create {}: {err}", tmp_dir.display()))?;
    let manifest_path = tmp_dir.join("rendered.yaml");
    fs::write(&manifest_path, rendered)
        .map_err(|err| format!("failed to write {}: {err}", manifest_path.display()))?;
    let args = vec![
        "-strict".to_string(),
        "-ignore-missing-schemas".to_string(),
        "-summary".to_string(),
        manifest_path.display().to_string(),
    ];
    match process.run_subprocess("kubeconform", &args, repo_root) {
        Ok((stdout, event)) => Ok((
            Vec::new(),
            serde_json::json!({
                "tool":"kubeconform",
                "status":"ok",
                "stdout": stdout,
                "subprocess_event": event
            }),
        )),
        Err(err) => {
            let message = err.to_stable_message();
            Ok((
                vec![format!("kubeconform validation failed: {message}")],
                serde_json::json!({
                    "tool":"kubeconform",
                    "status":"failed",
                    "error": message
                }),
            ))
        }
    }
}
