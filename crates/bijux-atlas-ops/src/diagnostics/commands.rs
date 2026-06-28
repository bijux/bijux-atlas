// SPDX-License-Identifier: Apache-2.0

use crate::diagnostics::bundle_contracts::{
    build_diagnose_bundle, collect_scenario_files, write_diagnose_bundle,
};
use crate::diagnostics::bundle_payload::diagnose_bundle_payload;
use crate::diagnostics::explain_payload::diagnose_explain_payload;
use crate::diagnostics::redaction_payload::{
    diagnose_redaction_payload, redact_bundle_metadata, write_redacted_bundle,
};
use serde_json::Value;
use std::path::{Path, PathBuf};

pub fn diagnose_bundle_payload_for_run(
    repo_root: &Path,
    run_id: &str,
    scenario: Option<&str>,
) -> Result<Value, String> {
    let files = collect_scenario_files(repo_root, scenario);
    let bundle = build_diagnose_bundle(run_id, scenario, files);
    let bundle_path = write_diagnose_bundle(repo_root, run_id, &bundle)?;

    Ok(diagnose_bundle_payload(
        &bundle_path
            .strip_prefix(repo_root)
            .unwrap_or(&bundle_path)
            .display()
            .to_string(),
        bundle
            .get("files")
            .cloned()
            .unwrap_or_else(|| serde_json::json!([])),
    ))
}

pub fn diagnose_explain_payload_for_bundle(
    repo_root: &Path,
    bundle_path: &Path,
) -> Result<Value, String> {
    let raw = std::fs::read_to_string(bundle_path)
        .map_err(|err| format!("failed to read {}: {err}", bundle_path.display()))?;
    let parsed: Value = serde_json::from_str(&raw)
        .map_err(|err| format!("failed to parse {}: {err}", bundle_path.display()))?;
    let file_count = parsed
        .get("files")
        .and_then(Value::as_array)
        .map(|v| v.len())
        .unwrap_or(0);
    Ok(diagnose_explain_payload(
        &bundle_path
            .strip_prefix(repo_root)
            .unwrap_or(bundle_path)
            .display()
            .to_string(),
        parsed
            .get("kind")
            .and_then(Value::as_str)
            .unwrap_or("unknown"),
        parsed
            .get("run_id")
            .and_then(Value::as_str)
            .unwrap_or("unknown"),
        file_count,
    ))
}

pub fn diagnose_redaction_payload_for_bundle(
    repo_root: &Path,
    bundle_path: &Path,
) -> Result<Value, String> {
    let raw = std::fs::read_to_string(bundle_path)
        .map_err(|err| format!("failed to read {}: {err}", bundle_path.display()))?;
    let mut parsed: Value = serde_json::from_str(&raw)
        .map_err(|err| format!("failed to parse {}: {err}", bundle_path.display()))?;

    let redacted = redact_bundle_metadata(&mut parsed);
    let out_path = write_redacted_bundle(bundle_path, &parsed)?;
    Ok(diagnose_redaction_payload(
        &bundle_path
            .strip_prefix(repo_root)
            .unwrap_or(bundle_path)
            .display()
            .to_string(),
        &out_path
            .strip_prefix(repo_root)
            .unwrap_or(&out_path)
            .display()
            .to_string(),
        redacted,
    ))
}

#[must_use]
pub fn resolve_bundle_path(repo_root: &Path, bundle: &Path) -> PathBuf {
    if bundle.is_absolute() {
        bundle.to_path_buf()
    } else {
        repo_root.join(bundle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_bundle_path_prefers_absolute_paths() {
        let resolved = resolve_bundle_path(Path::new("/repo"), Path::new("/bundle.json"));

        assert_eq!(resolved, PathBuf::from("/bundle.json"));
    }

    #[test]
    fn diagnose_bundle_payload_for_run_writes_bundle_index() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("artifacts/ops/run-local/reports"))
            .expect("create reports");
        std::fs::write(
            root.path()
                .join("artifacts/ops/run-local/reports/status.json"),
            "{}",
        )
        .expect("write report");

        let payload =
            diagnose_bundle_payload_for_run(root.path(), "run-local", None).expect("bundle");

        assert_eq!(payload["summary"]["total"], 1);
    }
}
