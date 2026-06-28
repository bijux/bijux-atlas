// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

pub fn ops_artifacts_root(repo_root: &Path) -> PathBuf {
    repo_root.join("artifacts/atlas-dev/ops")
}

pub fn ops_artifact_run_root(repo_root: &Path, run_id: &str) -> Result<PathBuf, String> {
    let root = ops_artifacts_root(repo_root);
    let target = root.join(run_id);
    if !target.starts_with(&root) {
        return Err("reset path guard failed".to_string());
    }
    Ok(target)
}

pub fn ops_artifact_report_path(repo_root: &Path, run_id: &str, rel: &str) -> PathBuf {
    ops_artifacts_root(repo_root).join(run_id).join(rel)
}

pub fn clean_ops_artifacts_payload(repo_root: &Path) -> Result<(serde_json::Value, i32), String> {
    let path = ops_artifacts_root(repo_root);
    if path.exists() {
        std::fs::remove_dir_all(&path)
            .map_err(|err| format!("failed to remove {}: {err}", path.display()))?;
    }
    let payload = serde_json::json!({
        "schema_version": 1,
        "text": format!("cleaned {}", path.display()),
        "rows": [],
        "summary": {"total": 0, "errors": 0, "warnings": 0}
    });
    Ok((payload, 0))
}

pub fn build_cleanup_payload(
    down_detail: String,
    down_code: i32,
    clean_detail: String,
    clean_code: i32,
) -> serde_json::Value {
    let errors = usize::from(down_code != 0) + usize::from(clean_code != 0);
    serde_json::json!({
        "schema_version": 1,
        "text": if errors == 0 { "ops cleanup passed" } else { "ops cleanup failed" },
        "rows": [
            {"action":"down","status": if down_code == 0 { "ok" } else { "failed" }, "detail": down_detail},
            {"action":"clean","status": if clean_code == 0 { "ok" } else { "failed" }, "detail": clean_detail}
        ],
        "summary": {"total": 2, "errors": errors, "warnings": 0}
    })
}

pub fn build_reset_payload(
    run_id: &str,
    target: &Path,
    rows: Vec<serde_json::Value>,
) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "text": format!("reset artifacts for run_id={run_id} at {}", target.display()),
        "rows": rows,
        "summary": {"total": 1, "errors": 0, "warnings": 0}
    })
}

#[cfg(test)]
mod tests {
    use super::{
        build_cleanup_payload, build_reset_payload, clean_ops_artifacts_payload,
        ops_artifact_report_path, ops_artifact_run_root, ops_artifacts_root,
    };

    #[test]
    fn ops_artifact_paths_stay_under_owned_root() {
        let root = tempfile::tempdir().expect("tempdir");
        let artifacts_root = ops_artifacts_root(root.path());
        let run_root = ops_artifact_run_root(root.path(), "owned-run").expect("run root");
        let report = ops_artifact_report_path(root.path(), "owned-run", "generate/pins.index.json");

        assert_eq!(artifacts_root, root.path().join("artifacts/atlas-dev/ops"));
        assert!(run_root.starts_with(&artifacts_root));
        assert!(report.starts_with(&artifacts_root));
    }

    #[test]
    fn cleanup_payload_reports_failed_rows() {
        let payload =
            build_cleanup_payload("down exit=1".to_string(), 1, "clean ok".to_string(), 0);

        assert_eq!(payload["summary"]["errors"], 1);
        assert_eq!(payload["rows"][0]["status"], "failed");
        assert_eq!(payload["rows"][1]["status"], "ok");
    }

    #[test]
    fn reset_payload_reports_target_path() {
        let root = tempfile::tempdir().expect("tempdir");
        let target = root.path().join("artifacts/atlas-dev/ops/owned-run");
        let payload = build_reset_payload(
            "owned-run",
            &target,
            vec![serde_json::json!({"kind":"artifacts","status":"ok"})],
        );

        assert_eq!(payload["summary"]["total"], 1);
        assert!(payload["text"]
            .as_str()
            .expect("text")
            .contains("owned-run"));
    }

    #[test]
    fn clean_payload_removes_owned_artifact_root() {
        let root = tempfile::tempdir().expect("tempdir");
        let artifact_root = ops_artifacts_root(root.path());
        std::fs::create_dir_all(&artifact_root).expect("create artifact root");
        std::fs::write(artifact_root.join("marker.json"), "{}").expect("write marker");

        let (payload, exit_code) = clean_ops_artifacts_payload(root.path()).expect("clean");

        assert_eq!(exit_code, 0);
        assert_eq!(payload["summary"]["errors"], 0);
        assert!(!artifact_root.exists());
    }
}
