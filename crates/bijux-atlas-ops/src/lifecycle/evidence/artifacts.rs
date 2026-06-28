// SPDX-License-Identifier: Apache-2.0

use super::support::{evidence_root, reset_directory, sha256_file};
use std::io::Cursor;
use std::path::Path;

pub fn debug_artifact_path(
    repo_root: &Path,
    run_id: &str,
    namespace: &str,
    file_name: &str,
) -> Result<std::path::PathBuf, String> {
    let path = repo_root
        .join("artifacts/ops")
        .join(run_id)
        .join("debug")
        .join(namespace)
        .join(file_name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    Ok(path)
}

pub fn write_debug_artifact(
    repo_root: &Path,
    run_id: &str,
    namespace: &str,
    file_name: &str,
    content: &str,
) -> Result<std::path::PathBuf, String> {
    let path = debug_artifact_path(repo_root, run_id, namespace, file_name)?;
    std::fs::write(&path, content)
        .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
    Ok(path)
}

pub fn build_lifecycle_evidence_bundle(
    repo_root: &Path,
    run_id: &str,
) -> Result<serde_json::Value, String> {
    let run_root = repo_root.join("artifacts/ops").join(run_id);
    let evidence_dir = run_root.join("evidence");
    std::fs::create_dir_all(&evidence_dir)
        .map_err(|err| format!("failed to create {}: {err}", evidence_dir.display()))?;
    let list_path = evidence_dir.join("ops-lifecycle-evidence.list");
    let tar_path = evidence_dir.join("ops-lifecycle-evidence.tar");
    let mut files = Vec::<String>::new();
    for dir in [run_root.join("reports"), run_root.join("debug")] {
        if !dir.exists() {
            continue;
        }
        let mut stack = vec![dir];
        while let Some(path) = stack.pop() {
            let entries = std::fs::read_dir(&path)
                .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
            for entry in entries {
                let entry =
                    entry.map_err(|err| format!("failed to read directory entry: {err}"))?;
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    stack.push(entry_path);
                    continue;
                }
                let rel = entry_path
                    .strip_prefix(repo_root)
                    .map_err(|err| format!("failed to relativize {}: {err}", entry_path.display()))?
                    .display()
                    .to_string();
                files.push(rel);
            }
        }
    }
    files.sort();
    files.dedup();
    std::fs::write(&list_path, files.join("\n"))
        .map_err(|err| format!("failed to write {}: {err}", list_path.display()))?;
    if files.is_empty() {
        return Ok(serde_json::json!({
            "status": "skipped",
            "tar_path": tar_path.display().to_string(),
            "list_path": list_path.display().to_string(),
            "files": files
        }));
    }
    let bundle_result = write_lifecycle_bundle_tarball(repo_root, &tar_path, &files);
    let status = if bundle_result.is_ok() {
        "ok"
    } else {
        "failed"
    };
    let error = bundle_result.err().unwrap_or_default();
    Ok(serde_json::json!({
        "status": status,
        "tar_path": tar_path.display().to_string(),
        "list_path": list_path.display().to_string(),
        "files": files,
        "stdout": "",
        "stderr": error
    }))
}

fn write_lifecycle_bundle_tarball(
    repo_root: &Path,
    tar_path: &Path,
    files: &[String],
) -> Result<(), String> {
    let file = std::fs::File::create(tar_path)
        .map_err(|err| format!("failed to create {}: {err}", tar_path.display()))?;
    let mut builder = tar::Builder::new(file);
    for relative in files {
        let source_path = repo_root.join(relative);
        let data = std::fs::read(&source_path)
            .map_err(|err| format!("failed to read {}: {err}", source_path.display()))?;
        let mut header = tar::Header::new_gnu();
        header.set_entry_type(tar::EntryType::Regular);
        header.set_mode(0o644);
        header.set_uid(0);
        header.set_gid(0);
        header.set_mtime(0);
        header.set_size(data.len() as u64);
        header.set_cksum();
        builder
            .append_data(&mut header, relative.as_str(), Cursor::new(data))
            .map_err(|err| {
                format!(
                    "failed to append {relative} to {}: {err}",
                    tar_path.display()
                )
            })?;
    }
    builder
        .finish()
        .map_err(|err| format!("failed to finalize {}: {err}", tar_path.display()))
}

pub fn collect_scan_reports(repo_root: &Path) -> Result<Vec<serde_json::Value>, String> {
    let scan_dir = evidence_root(repo_root)?.join("scans");
    if !scan_dir.exists() {
        return Ok(Vec::new());
    }
    let mut rows = std::fs::read_dir(&scan_dir)
        .map_err(|err| format!("failed to read {}: {err}", scan_dir.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?;
            let format = if name.ends_with(".json") {
                Some("json")
            } else if name.ends_with(".sarif") || name.ends_with(".sarif.json") {
                Some("sarif")
            } else {
                None
            }?;
            Some((path, format.to_string()))
        })
        .map(|(path, format)| {
            Ok(serde_json::json!({
                "path": path.strip_prefix(repo_root).unwrap_or(&path).display().to_string(),
                "format": format,
                "sha256": sha256_file(&path)?
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    rows.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    Ok(rows)
}

pub fn redact_sensitive_text(text: &str) -> String {
    let mut lines = Vec::new();
    for line in text.lines() {
        let upper = line.to_ascii_uppercase();
        if let Some((prefix, _)) = line.split_once('=') {
            let normalized = prefix.trim().to_ascii_uppercase();
            if ["PASSWORD", "TOKEN", "SECRET", "API_KEY"].contains(&normalized.as_str()) {
                lines.push(format!("{prefix}=[REDACTED]"));
                continue;
            }
        }
        if upper.contains("AUTHORIZATION: BEARER ") {
            lines.push("Authorization: Bearer [REDACTED]".to_string());
        } else {
            lines.push(line.to_string());
        }
    }
    if text.ends_with('\n') {
        format!("{}\n", lines.join("\n"))
    } else {
        lines.join("\n")
    }
}

#[must_use]
pub fn contains_common_secret_pattern(text: &str) -> bool {
    for line in text.lines() {
        let upper = line.to_ascii_uppercase();
        if let Some((prefix, value)) = line.split_once('=') {
            let normalized = prefix.trim().to_ascii_uppercase();
            if ["PASSWORD", "TOKEN", "SECRET", "API_KEY"].contains(&normalized.as_str())
                && value.trim() != "[REDACTED]"
            {
                return true;
            }
        }
        if upper.contains("AUTHORIZATION: BEARER ")
            && !upper.contains("AUTHORIZATION: BEARER [REDACTED]")
        {
            return true;
        }
    }
    false
}

pub fn collect_redacted_logs(repo_root: &Path) -> Result<Vec<String>, String> {
    let source_root = repo_root.join("artifacts/ops");
    let redacted_root = evidence_root(repo_root)?.join("redacted-logs");
    reset_directory(&redacted_root)?;
    if !source_root.exists() {
        return Ok(Vec::new());
    }
    let mut stack = vec![source_root];
    let mut outputs = Vec::new();
    while let Some(path) = stack.pop() {
        for entry in std::fs::read_dir(&path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?
        {
            let entry = entry.map_err(|err| format!("failed to read directory entry: {err}"))?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                stack.push(entry_path);
                continue;
            }
            let relative = entry_path
                .strip_prefix(repo_root)
                .unwrap_or(&entry_path)
                .display()
                .to_string();
            if !relative.contains("/debug/") {
                continue;
            }
            let output_name = relative.replace('/', "__");
            let output_path = redacted_root.join(output_name);
            let source = std::fs::read_to_string(&entry_path).unwrap_or_else(|_| {
                String::from_utf8_lossy(&std::fs::read(&entry_path).unwrap_or_default()).to_string()
            });
            let redacted = redact_sensitive_text(&source);
            std::fs::write(&output_path, redacted)
                .map_err(|err| format!("failed to write {}: {err}", output_path.display()))?;
            outputs.push(
                output_path
                    .strip_prefix(repo_root)
                    .unwrap_or(&output_path)
                    .display()
                    .to_string(),
            );
        }
    }
    outputs.sort();
    Ok(outputs)
}

pub fn render_evidence_index_html(
    repo_root: &Path,
    manifest: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let index_path = evidence_root(repo_root)?.join("index.html");
    let html = format!(
        "<!doctype html>\n<html lang=\"en\">\n<head><meta charset=\"utf-8\"><title>Release Evidence</title></head>\n<body>\n<h1>Release Evidence</h1>\n<p>Generated by bijux dev atlas ops evidence collect.</p>\n<ul>\n<li>Manifest: {}</li>\n<li>Identity: {}</li>\n<li>Chart package: {}</li>\n<li>SBOM count: {}</li>\n<li>Scan report count: {}</li>\n<li>Redacted logs: {}</li>\n</ul>\n</body>\n</html>\n",
        "ops/release/evidence/manifest.json",
        manifest
            .get("identity_path")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("ops/release/evidence/identity.json"),
        manifest
            .get("chart_package")
            .and_then(|value| value.get("path"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("ops/release/evidence/packages"),
        manifest
            .get("sboms")
            .and_then(serde_json::Value::as_array)
            .map(|rows| rows.len())
            .unwrap_or(0),
        manifest
            .get("scan_reports")
            .and_then(serde_json::Value::as_array)
            .map(|rows| rows.len())
            .unwrap_or(0),
        manifest
            .get("redacted_logs")
            .and_then(serde_json::Value::as_array)
            .map(|rows| rows.len())
            .unwrap_or(0),
    );
    std::fs::write(&index_path, html)
        .map_err(|err| format!("failed to write {}: {err}", index_path.display()))?;
    Ok(serde_json::json!({
        "path": index_path.strip_prefix(repo_root).unwrap_or(&index_path).display().to_string(),
        "sha256": sha256_file(&index_path)?
    }))
}

#[cfg(test)]
mod tests {
    use super::{
        build_lifecycle_evidence_bundle, collect_redacted_logs, collect_scan_reports,
        contains_common_secret_pattern, debug_artifact_path, redact_sensitive_text,
        render_evidence_index_html, write_debug_artifact,
    };
    use crate::lifecycle::evidence::support::evidence_root;

    #[test]
    fn lifecycle_bundle_indexes_reports_and_debug_logs() {
        let root = tempfile::tempdir().expect("tempdir");
        let run_root = root.path().join("artifacts/ops/run-local");
        std::fs::create_dir_all(run_root.join("reports")).expect("mkdir reports");
        std::fs::create_dir_all(run_root.join("debug")).expect("mkdir debug");
        std::fs::write(run_root.join("reports/report.json"), "{}").expect("write report");
        std::fs::write(run_root.join("debug/session.log"), "ok").expect("write log");

        let bundle =
            build_lifecycle_evidence_bundle(root.path(), "run-local").expect("build bundle");

        assert_eq!(bundle["status"].as_str(), Some("ok"));
        assert!(bundle["files"]
            .as_array()
            .expect("files")
            .iter()
            .any(|row| row.as_str() == Some("artifacts/ops/run-local/reports/report.json")));
    }

    #[test]
    fn scan_reports_include_hash_and_format() {
        let root = tempfile::tempdir().expect("tempdir");
        let scan_dir = evidence_root(root.path())
            .expect("evidence root")
            .join("scans");
        std::fs::create_dir_all(&scan_dir).expect("mkdir scans");
        std::fs::write(scan_dir.join("report.sarif"), "{}").expect("write scan report");

        let rows = collect_scan_reports(root.path()).expect("collect scan reports");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["format"].as_str(), Some("sarif"));
    }

    #[test]
    fn redaction_contract_masks_common_secret_patterns() {
        let redacted = redact_sensitive_text("TOKEN=secret-value\nAuthorization: Bearer abc123\n");

        assert!(!contains_common_secret_pattern(&redacted));
        assert!(!redacted.contains("secret-value"));
        assert!(!redacted.contains("abc123"));
    }

    #[test]
    fn redacted_logs_only_materialize_debug_sources() {
        let root = tempfile::tempdir().expect("tempdir");
        let debug_dir = root.path().join("artifacts/ops/run-local/debug");
        let reports_dir = root.path().join("artifacts/ops/run-local/reports");
        std::fs::create_dir_all(&debug_dir).expect("mkdir debug");
        std::fs::create_dir_all(&reports_dir).expect("mkdir reports");
        std::fs::write(debug_dir.join("session.log"), "TOKEN=secret").expect("write debug");
        std::fs::write(reports_dir.join("report.json"), "{}").expect("write report");

        let rows = collect_redacted_logs(root.path()).expect("collect redacted logs");

        assert_eq!(rows.len(), 1);
        let output = root.path().join(&rows[0]);
        let text = std::fs::read_to_string(output).expect("read redacted output");
        assert!(text.contains("[REDACTED]"));
    }

    #[test]
    fn evidence_index_renders_manifest_summary() {
        let root = tempfile::tempdir().expect("tempdir");
        let manifest = serde_json::json!({
            "identity_path": "ops/release/evidence/identity.json",
            "chart_package": {"path": "ops/release/evidence/packages/chart.tgz"},
            "sboms": [{}],
            "scan_reports": [{}, {}],
            "redacted_logs": [{}]
        });

        let row = render_evidence_index_html(root.path(), &manifest).expect("render index");

        assert_eq!(
            row["path"].as_str(),
            Some("ops/release/evidence/index.html")
        );
        assert!(root.path().join("ops/release/evidence/index.html").exists());
    }

    #[test]
    fn debug_artifact_writer_materializes_namespace_scoped_outputs() {
        let root = tempfile::tempdir().expect("tempdir");

        let path = write_debug_artifact(
            root.path(),
            "ops_run",
            "bijux-atlas-kind",
            "pods.txt",
            "pod/bijux-atlas-0",
        )
        .expect("write debug artifact");

        assert_eq!(
            path,
            root.path()
                .join("artifacts/ops/ops_run/debug/bijux-atlas-kind/pods.txt")
        );
        assert_eq!(
            std::fs::read_to_string(&path).expect("read debug artifact"),
            "pod/bijux-atlas-0"
        );
    }

    #[test]
    fn debug_artifact_path_creates_parent_directory_contract() {
        let root = tempfile::tempdir().expect("tempdir");

        let path = debug_artifact_path(root.path(), "ops_run", "bijux-atlas-kind", "events.json")
            .expect("debug artifact path");

        assert!(path
            .parent()
            .expect("debug parent")
            .ends_with("artifacts/ops/ops_run/debug/bijux-atlas-kind"));
        assert!(path.parent().expect("debug parent").exists());
    }
}
