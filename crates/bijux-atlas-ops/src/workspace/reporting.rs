// SPDX-License-Identifier: Apache-2.0

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub fn ops_report_path(repo_root: &Path, run_id: &str) -> PathBuf {
    repo_root
        .join("artifacts/reports/dev-atlas/ops")
        .join(format!("{run_id}.json"))
}

pub fn write_ops_report_artifact(
    repo_root: &Path,
    run_id: &str,
    inventory_summary: Value,
    inventory_errors: &[String],
    allow_write: bool,
    allow_subprocess: bool,
) -> Result<(Value, PathBuf), String> {
    let effective_config_snapshot =
        repo_root.join("configs/generated/runtime/effective-config.snapshot.json");
    let effective_config_hash = std::fs::read(&effective_config_snapshot)
        .ok()
        .map(|bytes| sha256_hex(&String::from_utf8_lossy(&bytes)));
    let report = json!({
        "schema_version": 1,
        "kind": "ops_report",
        "run_id": run_id,
        "repo_root": repo_root.display().to_string(),
        "inventory_summary": inventory_summary,
        "inventory_errors": inventory_errors,
        "effective_config_snapshot": effective_config_snapshot.display().to_string(),
        "effective_config_hash": effective_config_hash,
        "capabilities": {
            "fs_write": allow_write,
            "subprocess": allow_subprocess
        }
    });
    let out_path = ops_report_path(repo_root, run_id);
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    std::fs::write(
        &out_path,
        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("failed to write {}: {err}", out_path.display()))?;
    Ok((report, out_path))
}

pub fn ops_report_command_payload(report: &Value, out_path: &Path) -> Value {
    let status = if report["inventory_errors"]
        .as_array()
        .is_some_and(|values| values.is_empty())
    {
        "ok"
    } else {
        "failed"
    };
    let error_count = report["inventory_errors"]
        .as_array()
        .map_or(1, |values| values.len());

    json!({
        "schema_version": 1,
        "status": status,
        "text": format!("wrote ops report {}", out_path.display()),
        "rows": [{"path": out_path.display().to_string()}],
        "summary": {"total": 1, "errors": error_count, "warnings": 0}
    })
}

fn sha256_hex(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::{ops_report_command_payload, ops_report_path, write_ops_report_artifact};
    use std::path::Path;

    #[test]
    fn write_ops_report_artifact_uses_owned_path_and_payload() {
        let repo_root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(repo_root.path().join("configs/generated/runtime"))
            .expect("create config dir");
        std::fs::write(
            repo_root
                .path()
                .join("configs/generated/runtime/effective-config.snapshot.json"),
            "{}",
        )
        .expect("write effective config snapshot");

        let (report, out_path) = write_ops_report_artifact(
            repo_root.path(),
            "ops-run",
            serde_json::json!({"toolchain_images": 2}),
            &[],
            true,
            false,
        )
        .expect("write report");

        assert_eq!(out_path, ops_report_path(repo_root.path(), "ops-run"));
        assert_eq!(report["capabilities"]["fs_write"], true);
        assert_eq!(report["capabilities"]["subprocess"], false);
        assert!(out_path.exists());
    }

    #[test]
    fn ops_report_command_payload_reports_failures() {
        let report = serde_json::json!({
            "inventory_errors": ["missing inventory"]
        });
        let payload = ops_report_command_payload(
            &report,
            Path::new("artifacts/reports/dev-atlas/ops/run.json"),
        );

        assert_eq!(payload["status"], "failed");
        assert_eq!(payload["summary"]["errors"], 1);
    }
}
