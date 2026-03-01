// SPDX-License-Identifier: Apache-2.0

use crate::cli::{ArtifactsCommand, ArtifactsCommonArgs, ArtifactsGcArgs};
use crate::resolve_repo_root;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

const DEFAULT_RUN_KEEP_LAST: usize = 5;
const RUNS_DIR_NAME: &str = "run";
const RUN_PIN_FILE: &str = ".pin";

pub(crate) fn run_artifacts_command(quiet: bool, command: ArtifactsCommand) -> i32 {
    let result: Result<(String, i32), String> = {
        match command {
            ArtifactsCommand::Clean(common) => run_artifacts_clean(common),
            ArtifactsCommand::Gc(args) => run_artifacts_gc(args),
        }
    };
    match result {
        Ok((rendered, code)) => {
            if !quiet && !rendered.is_empty() {
                let _ = writeln!(io::stdout(), "{rendered}");
            }
            code
        }
        Err(err) => {
            let _ = writeln!(io::stderr(), "bijux-dev-atlas artifacts failed: {err}");
            1
        }
    }
}

pub(crate) fn repo_artifacts_root(repo_root: &Path) -> PathBuf {
    repo_root.join("artifacts")
}

pub(crate) fn artifact_runs_root(repo_root: &Path) -> PathBuf {
    repo_artifacts_root(repo_root).join(RUNS_DIR_NAME)
}

fn run_artifacts_clean(common: ArtifactsCommonArgs) -> Result<(String, i32), String> {
    if !common.allow_write {
        return Err("artifacts clean requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let artifacts_root = repo_artifacts_root(&repo_root);
    clean_artifacts_children(&artifacts_root)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "action": "artifacts-clean",
        "text": "artifacts directory cleaned",
        "artifacts_root": artifacts_root.display().to_string(),
    });
    let rendered = crate::emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

fn run_artifacts_gc(args: ArtifactsGcArgs) -> Result<(String, i32), String> {
    if !args.common.allow_write {
        return Err("artifacts gc requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let runs_root = artifact_runs_root(&repo_root);
    let keep_last = args.keep_last.max(DEFAULT_RUN_KEEP_LAST);
    let summary = gc_artifact_runs(&runs_root, keep_last)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "action": "artifacts-gc",
        "text": "artifact run directories garbage collected",
        "runs_root": runs_root.display().to_string(),
        "keep_last": keep_last,
        "deleted_runs": summary.deleted_runs,
        "kept_runs": summary.kept_runs,
        "pinned_runs": summary.pinned_runs,
        "deleted_count": summary.deleted_runs.len(),
        "kept_count": summary.kept_runs.len(),
        "pinned_count": summary.pinned_runs.len(),
    });
    let rendered = crate::emit_payload(args.common.format, args.common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

fn clean_artifacts_children(artifacts_root: &Path) -> Result<(), String> {
    if !artifacts_root.exists() {
        return Ok(());
    }
    let entries = fs::read_dir(artifacts_root)
        .map_err(|err| format!("read {} failed: {err}", artifacts_root.display()))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.file_name().and_then(|value| value.to_str()) == Some(".gitkeep") {
            continue;
        }
        if path.is_dir() {
            fs::remove_dir_all(&path)
                .map_err(|err| format!("remove {} failed: {err}", path.display()))?;
        } else {
            fs::remove_file(&path)
                .map_err(|err| format!("remove {} failed: {err}", path.display()))?;
        }
    }
    Ok(())
}

struct ArtifactGcSummary {
    deleted_runs: Vec<String>,
    kept_runs: Vec<String>,
    pinned_runs: Vec<String>,
}

fn gc_artifact_runs(runs_root: &Path, keep_last: usize) -> Result<ArtifactGcSummary, String> {
    if !runs_root.exists() {
        return Ok(ArtifactGcSummary {
            deleted_runs: Vec::new(),
            kept_runs: Vec::new(),
            pinned_runs: Vec::new(),
        });
    }
    let mut runs = Vec::new();
    let entries = fs::read_dir(runs_root)
        .map_err(|err| format!("read {} failed: {err}", runs_root.display()))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let metadata =
            fs::metadata(&path).map_err(|err| format!("stat {} failed: {err}", path.display()))?;
        let modified = metadata
            .modified()
            .ok()
            .and_then(|ts| ts.duration_since(UNIX_EPOCH).ok())
            .map(|ts| ts.as_secs())
            .unwrap_or(0);
        let pinned = path.join(RUN_PIN_FILE).is_file();
        runs.push((name, path, modified, pinned));
    }
    runs.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)));

    let mut summary = ArtifactGcSummary {
        deleted_runs: Vec::new(),
        kept_runs: Vec::new(),
        pinned_runs: Vec::new(),
    };

    let mut kept_unpinned = 0usize;
    for (name, path, _, pinned) in runs {
        if pinned {
            summary.pinned_runs.push(name);
            continue;
        }
        if kept_unpinned < keep_last {
            summary.kept_runs.push(name);
            kept_unpinned += 1;
            continue;
        }
        fs::remove_dir_all(&path)
            .map_err(|err| format!("remove {} failed: {err}", path.display()))?;
        summary.deleted_runs.push(name);
    }

    summary.deleted_runs.sort();
    summary.kept_runs.sort();
    summary.pinned_runs.sort();
    Ok(summary)
}
