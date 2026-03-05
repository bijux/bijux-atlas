// SPDX-License-Identifier: Apache-2.0

use crate::cli::{ReproduceCommand, ReproduceCommonArgs, ReproduceExplainArgs};
use crate::{emit_payload, resolve_repo_root};
use bijux_dev_atlas::contracts::reproducibility::scenario_catalog;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let text =
        fs::read_to_string(path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn file_sha(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn collect_source_snapshot_hash(root: &Path) -> Result<String, String> {
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(root).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(root)
            .map_err(|err| err.to_string())?
            .to_path_buf();
        let rel_str = rel.to_string_lossy();
        if rel_str.starts_with(".git/")
            || rel_str.starts_with("artifacts/")
            || rel_str.starts_with("target/")
        {
            continue;
        }
        files.push(rel);
    }
    files.sort();
    let mut hasher = Sha256::new();
    for rel in files {
        hasher.update(rel.to_string_lossy().as_bytes());
        let digest = file_sha(&root.join(&rel))?;
        hasher.update(digest.as_bytes());
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn run(common: ReproduceCommonArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(common.repo_root)?;
    let scenarios = scenario_catalog();
    let source_hash = collect_source_snapshot_hash(&root)?;
    let manifest_path = root.join("release/manifest.json");
    let manifest = read_json(&manifest_path).unwrap_or_else(|_| serde_json::json!({}));
    let artifacts_count = manifest
        .get("artifacts")
        .and_then(serde_json::Value::as_array)
        .map_or(0, |v| v.len());

    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "reproduce_run",
        "status": "ok",
        "environment": {
            "source_snapshot_hash": source_hash
        },
        "scenarios": scenarios,
        "release_manifest_artifact_count": artifacts_count
    });
    let rendered = emit_payload(common.format, common.out, &payload)?;
    Ok((rendered, 0))
}

fn verify(common: ReproduceCommonArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(common.repo_root)?;
    let scenarios_path = root.join("ops/reproducibility/scenarios.json");
    let scenarios = read_json(&scenarios_path)?;
    let rows = scenarios
        .get("scenarios")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let required = [
        "rebuild-crates",
        "rebuild-docker-image",
        "rebuild-helm-chart",
        "rebuild-docs-site",
        "rebuild-release-bundle",
    ];
    let ids = rows
        .iter()
        .filter_map(|row| row.get("id").and_then(serde_json::Value::as_str))
        .collect::<std::collections::BTreeSet<_>>();
    let mut missing = Vec::new();
    for id in required {
        if !ids.contains(id) {
            missing.push(id.to_string());
        }
    }
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "reproduce_verify",
        "status": if missing.is_empty() { "ok" } else { "failed" },
        "missing_required_scenarios": missing,
        "scenario_count": rows.len()
    });
    let rendered = emit_payload(common.format, common.out, &payload)?;
    Ok((
        rendered,
        if payload.get("status").and_then(serde_json::Value::as_str) == Some("ok") {
            0
        } else {
            3
        },
    ))
}

fn explain(args: ReproduceExplainArgs) -> Result<(String, i32), String> {
    let all = scenario_catalog();
    let payload = if let Some(ref id) = args.scenario {
        if let Some(found) = all.iter().find(|row| &row.id == id) {
            serde_json::json!({
                "schema_version": 1,
                "kind": "reproduce_explain",
                "status": "ok",
                "scenario": found
            })
        } else {
            serde_json::json!({
                "schema_version": 1,
                "kind": "reproduce_explain",
                "status": "failed",
                "error": format!("unknown reproducibility scenario `{}`", id)
            })
        }
    } else {
        serde_json::json!({
            "schema_version": 1,
            "kind": "reproduce_explain",
            "status": "ok",
            "scenarios": all
        })
    };
    let rendered = emit_payload(args.common.format, args.common.out, &payload)?;
    Ok((
        rendered,
        if payload.get("status").and_then(serde_json::Value::as_str) == Some("ok") {
            0
        } else {
            2
        },
    ))
}

pub(crate) fn run_reproduce_command(
    _quiet: bool,
    command: ReproduceCommand,
) -> Result<(String, i32), String> {
    match command {
        ReproduceCommand::Run(args) => run(args),
        ReproduceCommand::Verify(args) => verify(args),
        ReproduceCommand::Explain(args) => explain(args),
    }
}
