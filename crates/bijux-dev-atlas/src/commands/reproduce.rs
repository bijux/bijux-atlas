// SPDX-License-Identifier: Apache-2.0

use crate::cli::{ReproduceCommand, ReproduceCommonArgs, ReproduceExplainArgs};
use crate::{emit_payload, resolve_repo_root};
use bijux_dev_atlas::contracts::reproducibility::{scenario_catalog, ReproFailureClass};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn file_sha(path: &Path) -> Result<String, String> {
    let bytes =
        fs::read(path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn walk_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(cursor) = stack.pop() {
        let entries = fs::read_dir(&cursor)
            .map_err(|err| format!("failed to read {}: {err}", cursor.display()))?;
        for entry in entries {
            let entry = entry.map_err(|err| format!("failed to read directory entry: {err}"))?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn collect_source_snapshot_hash(root: &Path) -> Result<String, String> {
    let mut files = Vec::new();
    for path in walk_files(root)? {
        let rel = path
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

fn core_artifact_hashes(root: &Path) -> BTreeMap<String, String> {
    let candidates = [
        "Cargo.lock",
        "ops/reproducibility/spec.json",
        "ops/reproducibility/report.schema.json",
        "ops/reproducibility/scenarios.json",
        "release/manifest.json",
    ];
    let mut out = BTreeMap::new();
    for rel in candidates {
        let path = root.join(rel);
        if path.exists() {
            if let Ok(digest) = file_sha(&path) {
                out.insert(rel.to_string(), digest);
            }
        }
    }
    out
}

fn run_payload(root: &Path) -> Result<serde_json::Value, String> {
    let started = Instant::now();
    let mut scenarios = scenario_catalog();
    scenarios.sort_by(|a, b| a.id.cmp(&b.id));
    let source_hash = collect_source_snapshot_hash(root)?;
    let manifest_path = root.join("release/manifest.json");
    let manifest = read_json(&manifest_path).unwrap_or_else(|_| serde_json::json!({}));
    let artifacts_count = manifest
        .get("artifacts")
        .and_then(serde_json::Value::as_array)
        .map_or(0, |v| v.len());
    let environment = serde_json::json!({
        "source_snapshot_hash": source_hash,
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "offline_safe": true
    });
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "reproduce_run",
        "status": "ok",
        "metrics": {
            "execution_time_ms": started.elapsed().as_millis() as u64
        },
        "environment": environment,
        "artifact_hashes": core_artifact_hashes(root),
        "scenarios": scenarios,
        "release_manifest_artifact_count": artifacts_count
    });
    Ok(payload)
}

fn normalize_for_determinism(mut payload: serde_json::Value) -> serde_json::Value {
    if let Some(metrics) = payload
        .get_mut("metrics")
        .and_then(serde_json::Value::as_object_mut)
    {
        metrics.remove("execution_time_ms");
    }
    payload
}

fn write_run_evidence(root: &Path, payload: &serde_json::Value) -> Result<PathBuf, String> {
    let out = root.join("artifacts/reproducibility/run-report.json");
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(payload)
        .map_err(|err| format!("failed to encode reproducibility payload: {err}"))?;
    fs::write(&out, format!("{text}\n"))
        .map_err(|err| format!("failed to write {}: {err}", out.display()))?;
    Ok(out)
}

fn run(common: ReproduceCommonArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(common.repo_root)?;
    let payload = run_payload(&root)?;
    let evidence_path = write_run_evidence(&root, &payload)?;
    let mut emitted = payload;
    if let Some(obj) = emitted.as_object_mut() {
        obj.insert(
            "evidence_path".to_string(),
            serde_json::Value::String(evidence_path.display().to_string()),
        );
    }
    let rendered = emit_payload(common.format, common.out, &emitted)?;
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
    missing.sort();

    let first = run_payload(&root)?;
    let second = run_payload(&root)?;
    let deterministic =
        normalize_for_determinism(first.clone()) == normalize_for_determinism(second.clone());

    let mut failures = Vec::new();
    if !missing.is_empty() {
        failures.push(serde_json::json!({
            "class": ReproFailureClass::MissingScenario,
            "message": "required reproducibility scenarios are missing"
        }));
    }
    if !deterministic {
        failures.push(serde_json::json!({
            "class": ReproFailureClass::NondeterministicOutput,
            "message": "reproduce run payload is not deterministic"
        }));
    }
    let offline_safe = first
        .get("environment")
        .and_then(|v| v.get("offline_safe"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    if !offline_safe {
        failures.push(serde_json::json!({
            "class": ReproFailureClass::OfflineViolation,
            "message": "reproducibility run must be offline-safe"
        }));
    }

    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "reproduce_verify",
        "status": if failures.is_empty() { "ok" } else { "failed" },
        "missing_required_scenarios": missing,
        "deterministic_report": deterministic,
        "offline_safe": offline_safe,
        "failure_classification": failures,
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
    let mut all = scenario_catalog();
    all.sort_by(|a, b| a.id.cmp(&b.id));
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

fn status(common: ReproduceCommonArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(common.repo_root)?;
    let evidence = root.join("artifacts/reproducibility/run-report.json");
    let verify_payload = verify(ReproduceCommonArgs {
        repo_root: Some(root.clone()),
        format: crate::cli::FormatArg::Json,
        out: None,
    })?;
    let verify_json: serde_json::Value = serde_json::from_str(&verify_payload.0)
        .map_err(|err| format!("failed to parse verify payload: {err}"))?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "reproduce_status",
        "status": verify_json.get("status").and_then(serde_json::Value::as_str).unwrap_or("failed"),
        "evidence_present": evidence.exists(),
        "verify": verify_json
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

fn write_json_artifact(
    root: &Path,
    rel_path: &str,
    payload: &serde_json::Value,
) -> Result<(), String> {
    let path = root.join(rel_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(payload)
        .map_err(|err| format!("failed to encode reproducibility payload: {err}"))?;
    fs::write(&path, format!("{text}\n"))
        .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
    Ok(())
}

fn audit_report(common: ReproduceCommonArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(common.repo_root)?;
    let now_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| format!("system clock is before unix epoch: {err}"))?
        .as_secs();
    let verify_payload = verify(ReproduceCommonArgs {
        repo_root: Some(root.clone()),
        format: crate::cli::FormatArg::Json,
        out: None,
    })?;
    let verify_json: serde_json::Value = serde_json::from_str(&verify_payload.0)
        .map_err(|err| format!("failed to parse verify payload: {err}"))?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "reproducibility_audit_report",
        "status": verify_json.get("status").and_then(serde_json::Value::as_str).unwrap_or("failed"),
        "recorded_at_unix": now_unix,
        "verify": verify_json
    });
    write_json_artifact(
        &root,
        "artifacts/reproducibility/audit-report.json",
        &payload,
    )?;
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

fn metrics(common: ReproduceCommonArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(common.repo_root)?;
    let run = run_payload(&root)?;
    let scenario_count = run
        .get("scenarios")
        .and_then(serde_json::Value::as_array)
        .map_or(0_u64, |rows| rows.len() as u64);
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "reproducibility_metrics",
        "status": "ok",
        "metrics": {
            "scenario_count": scenario_count,
            "artifact_hash_count": run.get("artifact_hashes").and_then(serde_json::Value::as_object).map_or(0, |rows| rows.len()) as u64,
            "source_snapshot_present": run
                .get("environment")
                .and_then(|v| v.get("source_snapshot_hash"))
                .and_then(serde_json::Value::as_str)
                .is_some()
        }
    });
    write_json_artifact(&root, "artifacts/reproducibility/metrics.json", &payload)?;
    let rendered = emit_payload(common.format, common.out, &payload)?;
    Ok((rendered, 0))
}

fn lineage_validate(common: ReproduceCommonArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(common.repo_root)?;
    let run = run_payload(&root)?;
    let required = [
        "Cargo.lock",
        "ops/reproducibility/spec.json",
        "ops/reproducibility/report.schema.json",
        "ops/reproducibility/scenarios.json",
    ];
    let hashes = run
        .get("artifact_hashes")
        .and_then(serde_json::Value::as_object)
        .cloned()
        .unwrap_or_default();
    let mut missing = Vec::new();
    for id in required {
        if !hashes.contains_key(id) {
            missing.push(id.to_string());
        }
    }
    missing.sort();
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "reproducibility_lineage_validate",
        "status": if missing.is_empty() { "ok" } else { "failed" },
        "required_artifacts": required,
        "missing_artifacts": missing
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

fn summary_table(common: ReproduceCommonArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(common.repo_root)?;
    let run = run_payload(&root)?;
    let scenarios = run
        .get("scenarios")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut markdown = String::from("| Scenario | Kind | Stability |\n|---|---|---|\n");
    for row in scenarios {
        let id = row
            .get("id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        let kind = row
            .get("kind")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        let stability = row
            .get("stability")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        markdown.push_str(&format!("| {id} | {kind} | {stability} |\n"));
    }
    let out_path = root.join("artifacts/reproducibility/summary-table.md");
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    fs::write(&out_path, markdown)
        .map_err(|err| format!("failed to write {}: {err}", out_path.display()))?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "reproducibility_summary_table",
        "status": "ok",
        "path": out_path.display().to_string()
    });
    let rendered = emit_payload(common.format, common.out, &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_reproduce_command(
    _quiet: bool,
    command: ReproduceCommand,
) -> Result<(String, i32), String> {
    match command {
        ReproduceCommand::Run(args) => run(args),
        ReproduceCommand::Verify(args) => verify(args),
        ReproduceCommand::Explain(args) => explain(args),
        ReproduceCommand::Status(args) => status(args),
        ReproduceCommand::AuditReport(args) => audit_report(args),
        ReproduceCommand::Metrics(args) => metrics(args),
        ReproduceCommand::LineageValidate(args) => lineage_validate(args),
        ReproduceCommand::SummaryTable(args) => summary_table(args),
    }
}
