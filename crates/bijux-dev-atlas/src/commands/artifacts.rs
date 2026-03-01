// SPDX-License-Identifier: Apache-2.0

use crate::cli::{ArtifactsCommand, ArtifactsCommonArgs};
use crate::resolve_repo_root;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

pub(crate) fn run_artifacts_command(quiet: bool, command: ArtifactsCommand) -> i32 {
    let result = (|| -> Result<(String, i32), String> {
        match command {
            ArtifactsCommand::Clean(common) => run_artifacts_clean(common),
        }
    })();
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

fn run_artifacts_clean(common: ArtifactsCommonArgs) -> Result<(String, i32), String> {
    if !common.allow_write {
        return Err("artifacts clean requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let artifacts_root = repo_root.join("artifacts");
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

fn clean_artifacts_children(artifacts_root: &PathBuf) -> Result<(), String> {
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
