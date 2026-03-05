// SPDX-License-Identifier: Apache-2.0

use crate::cli::{FormatArg, MigrationsCommand};
use crate::{emit_payload, resolve_repo_root};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

fn collect_repo_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(cursor) = stack.pop() {
        let entries =
            fs::read_dir(&cursor).map_err(|err| format!("read {} failed: {err}", cursor.display()))?;
        for entry in entries {
            let entry = entry.map_err(|err| format!("read directory entry failed: {err}"))?;
            let path = entry.path();
            let rel = path
                .strip_prefix(root)
                .map_err(|err| format!("normalize {} failed: {err}", path.display()))?;
            if rel.starts_with(".git")
                || rel.starts_with("target")
                || rel.starts_with("node_modules")
                || rel.starts_with("artifacts")
            {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
            } else {
                files.push(rel.to_path_buf());
            }
        }
    }
    files.sort();
    Ok(files)
}

fn collect_repo_dirs(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut dirs = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(cursor) = stack.pop() {
        let entries =
            fs::read_dir(&cursor).map_err(|err| format!("read {} failed: {err}", cursor.display()))?;
        for entry in entries {
            let entry = entry.map_err(|err| format!("read directory entry failed: {err}"))?;
            let path = entry.path();
            let rel = path
                .strip_prefix(root)
                .map_err(|err| format!("normalize {} failed: {err}", path.display()))?;
            if rel.starts_with(".git")
                || rel.starts_with("target")
                || rel.starts_with("node_modules")
                || rel.starts_with("artifacts")
            {
                continue;
            }
            if path.is_dir() {
                dirs.push(rel.to_path_buf());
                stack.push(path);
            }
        }
    }
    dirs.sort();
    Ok(dirs)
}

fn is_legacy_path(rel: &Path) -> bool {
    let text = rel.display().to_string();
    text == "clients"
        || text == "tools"
        || text == "scripts"
        || text == "tutorials/scripts"
        || text == "tutorials/tests"
        || text == "clients/atlas-client/tools"
}

fn is_legacy_file(rel: &Path) -> bool {
    let text = rel.display().to_string();
    let ext = rel.extension().and_then(|v| v.to_str()).unwrap_or_default();
    if text.starts_with("tutorials/") && (ext == "py" || ext == "sh") {
        return true;
    }
    if text.contains("/__pycache__/") || text.ends_with(".pyc") {
        return true;
    }
    false
}

fn run_migrations_status(
    repo_root: Option<PathBuf>,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let dirs = collect_repo_dirs(&root)?;
    let files = collect_repo_files(&root)?;
    let mut violations = Vec::new();

    for rel in dirs {
        if is_legacy_path(&rel) {
            violations.push(format!("{}/", rel.display()));
        }
    }
    for rel in files {
        if is_legacy_file(&rel) {
            violations.push(rel.display().to_string());
        }
    }
    violations.sort();
    violations.dedup();

    let status = if violations.is_empty() { "ok" } else { "failed" };
    let report = serde_json::json!({
        "schema_version": 1,
        "kind": "migration_status",
        "status": status,
        "legacy_paths": violations,
        "summary": {
            "legacy_path_count": violations.len()
        }
    });

    let report_path = root.join("artifacts/governance/migration-complete-status.json");
    if let Some(parent) = report_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("create {} failed: {err}", parent.display()))?;
    }
    fs::write(
        &report_path,
        serde_json::to_string_pretty(&report)
            .map_err(|err| format!("encode migration report failed: {err}"))?,
    )
    .map_err(|err| format!("write {} failed: {err}", report_path.display()))?;

    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "migration_status",
        "status": status,
        "report_path": "artifacts/governance/migration-complete-status.json",
        "summary": report["summary"].clone(),
        "legacy_paths": report["legacy_paths"].clone(),
    });
    let rendered = emit_payload(format, out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

pub(crate) fn run_migrations_command(
    quiet: bool,
    command: MigrationsCommand,
) -> i32 {
    let result = match command {
        MigrationsCommand::Status {
            repo_root,
            format,
            out,
        } => run_migrations_status(repo_root, format, out),
    };
    match result {
        Ok((rendered, code)) => {
            if !quiet && !rendered.is_empty() {
                if code == 0 {
                    let _ = writeln!(std::io::stdout(), "{rendered}");
                } else {
                    let _ = writeln!(std::io::stderr(), "{rendered}");
                }
            }
            code
        }
        Err(err) => {
            let _ = writeln!(std::io::stderr(), "bijux-dev-atlas migrations failed: {err}");
            1
        }
    }
}
