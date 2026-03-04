// SPDX-License-Identifier: Apache-2.0

use crate::cli::{AuditBundleCommand, AuditCommand};
use crate::{emit_payload, resolve_repo_root};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

fn checklist_path(root: &Path) -> PathBuf {
    root.join("configs/audit/audit-artifact-checklist.json")
}

fn schema_path(root: &Path) -> PathBuf {
    root.join("configs/audit/audit-bundle.schema.json")
}

fn bundle_path(root: &Path) -> PathBuf {
    root.join("artifacts/audit/bundle.json")
}

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    serde_json::from_str(
        &fs::read_to_string(path)
            .map_err(|err| format!("read {} failed: {err}", path.display()))?,
    )
    .map_err(|err| format!("parse {} failed: {err}", path.display()))
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("create {} failed: {err}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(value)
        .map_err(|err| format!("encode {} failed: {err}", path.display()))?;
    fs::write(path, text).map_err(|err| format!("write {} failed: {err}", path.display()))
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let data = fs::read(path).map_err(|err| format!("read {} failed: {err}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(data)))
}

fn bundle_generate(
    repo_root: Option<PathBuf>,
    format: crate::cli::FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let checklist = read_json(&checklist_path(&root))?;
    let required = checklist
        .get("required_artifacts")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut artifacts = Vec::new();
    let mut missing = Vec::new();
    for row in required {
        let Some(id) = row.get("id").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let Some(path) = row.get("path").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let abs = root.join(path);
        if !abs.exists() {
            missing.push(serde_json::json!({"id": id, "path": path}));
            continue;
        }
        let digest = sha256_file(&abs)?;
        artifacts.push(serde_json::json!({"id": id, "path": path, "sha256": digest}));
    }

    let bundle = serde_json::json!({
        "schema_version": 1,
        "kind": "audit_bundle",
        "status": if missing.is_empty() {"ok"} else {"failed"},
        "artifacts": artifacts,
        "missing": missing,
        "checklist": checklist_path(&root).strip_prefix(&root).unwrap_or(&checklist_path(&root)).display().to_string(),
    });
    write_json(&bundle_path(&root), &bundle)?;
    let rendered = emit_payload(format, out, &bundle)?;
    let code = if bundle["status"] == "ok" { 0 } else { 1 };
    Ok((rendered, code))
}

fn bundle_validate(
    repo_root: Option<PathBuf>,
    format: crate::cli::FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let schema = read_json(&schema_path(&root))?;
    let required = schema
        .get("required")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let bundle = read_json(&bundle_path(&root))?;
    let Some(bundle_obj) = bundle.as_object() else {
        return Err("audit bundle must be an object".to_string());
    };
    let mut errors = Vec::new();
    for key in required.iter().filter_map(serde_json::Value::as_str) {
        if !bundle_obj.contains_key(key) {
            errors.push(format!("audit bundle missing required key `{key}`"));
        }
    }
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "audit_bundle_validate",
        "status": if errors.is_empty() {"ok"} else {"failed"},
        "bundle": bundle_path(&root).strip_prefix(&root).unwrap_or(&bundle_path(&root)).display().to_string(),
        "errors": errors,
    });
    let rendered = emit_payload(format, out, &payload)?;
    let code = if payload["status"] == "ok" { 0 } else { 1 };
    Ok((rendered, code))
}

pub(crate) fn run_audit_command(
    _quiet: bool,
    command: AuditCommand,
) -> Result<(String, i32), String> {
    match command {
        AuditCommand::Bundle { command } => match command {
            AuditBundleCommand::Generate(args) => {
                bundle_generate(args.repo_root, args.format, args.out)
            }
            AuditBundleCommand::Validate(args) => {
                bundle_validate(args.repo_root, args.format, args.out)
            }
        },
    }
}
