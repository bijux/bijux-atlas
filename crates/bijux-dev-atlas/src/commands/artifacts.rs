// SPDX-License-Identifier: Apache-2.0

use crate::cli::{
    ArtifactsCommand, ArtifactsCommonArgs, ArtifactsGcArgs, ArtifactsReportCommand,
    ArtifactsReportDiffArgs, ArtifactsReportReadArgs, ArtifactsReportScanArgs,
};
use crate::resolve_repo_root;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};
use std::time::UNIX_EPOCH;

const DEFAULT_RUN_KEEP_LAST: usize = 5;
const RUNS_DIR_NAME: &str = "run";
const RUN_PIN_FILE: &str = ".pin";
const REPORT_SCHEMAS_DIR: &str = "configs/contracts/reports";
const REPORT_BUDGET_PATH: &str = "configs/contracts/report-budget.json";
const REPORT_SCHEMA_REGISTRY_PATH: &str = "configs/reports/schema-registry.json";
const REPORT_OWNERSHIP_PATH: &str = "configs/reports/ownership.json";
const REPORT_CHECK_MAP_PATH: &str = "configs/reports/check-report-map.json";

pub(crate) fn run_artifacts_command(quiet: bool, command: ArtifactsCommand) -> i32 {
    let result: Result<(String, i32), String> = match command {
        ArtifactsCommand::Clean(common) => run_artifacts_clean(common),
        ArtifactsCommand::Gc(args) => run_artifacts_gc(args),
        ArtifactsCommand::Report { command } => run_artifacts_report(command),
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

fn run_artifacts_report(command: ArtifactsReportCommand) -> Result<(String, i32), String> {
    match command {
        ArtifactsReportCommand::Inventory(common) => run_artifacts_report_inventory(common),
        ArtifactsReportCommand::Manifest(args) => run_artifacts_report_manifest(args),
        ArtifactsReportCommand::Index(args) => run_artifacts_report_index(args),
        ArtifactsReportCommand::Read(args) => run_artifacts_report_read(args),
        ArtifactsReportCommand::Diff(args) => run_artifacts_report_diff(args),
        ArtifactsReportCommand::Validate(args) => run_artifacts_report_validate(args),
    }
}

fn run_artifacts_report_inventory(common: ArtifactsCommonArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let rows = load_report_schema_inventory(&repo_root)?;
    let owners = load_report_ownership(&repo_root)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "report_schema_inventory",
        "summary": {
            "schema_count": rows.len()
        },
        "rows": rows.iter().map(|row| report_schema_row_json(row, owners.get(&row.report_id).map(String::as_str))).collect::<Vec<_>>()
    });
    let rendered = crate::emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

fn run_artifacts_report_manifest(args: ArtifactsReportScanArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let reports_root = resolve_reports_root(&repo_root, args.reports_root.as_deref());
    let rows = scan_report_artifacts(&reports_root, &repo_root)?;
    let total_bytes = rows.iter().map(|row| row.size_bytes).sum::<u64>();
    let payload = serde_json::json!({
        "report_id": "artifact-report-manifest",
        "version": 1,
        "run_id": stable_run_id_for_root(&reports_root),
        "inputs": {
            "reports_root": relative_or_absolute(&repo_root, &reports_root)
        },
        "summary": {
            "report_count": rows.len(),
            "total_bytes": total_bytes
        },
        "evidence": {
            "inventory_source": REPORT_SCHEMAS_DIR
        },
        "reports": rows.iter().map(report_artifact_row_json).collect::<Vec<_>>()
    });
    let rendered = crate::emit_payload(args.common.format, args.common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

fn run_artifacts_report_read(args: ArtifactsReportReadArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let target = if let Some(path) = args.report_path {
        path
    } else {
        let reports_root = resolve_reports_root(&repo_root, args.reports_root.as_deref());
        let rows = scan_report_artifacts(&reports_root, &repo_root)?;
        let Some(first) = rows.first() else {
            return Err("artifacts report read found no reports".to_string());
        };
        repo_root.join(&first.path)
    };
    let text = fs::read_to_string(&target)
        .map_err(|err| format!("read {} failed: {err}", target.display()))?;
    let value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|err| format!("parse {} failed: {err}", target.display()))?;
    let payload = serde_json::json!({
        "report_id": "artifact-report-reader",
        "version": 1,
        "run_id": stable_run_id_for_root(target.parent().unwrap_or(&repo_root)),
        "inputs": {
            "report_path": relative_or_absolute(&repo_root, &target)
        },
        "summary": {
            "payload_report_id": value.get("report_id").and_then(serde_json::Value::as_str).unwrap_or("unknown"),
            "payload_version": value.get("version").and_then(serde_json::Value::as_u64).unwrap_or(0)
        },
        "evidence": {
            "has_summary": value.get("summary").is_some(),
            "has_evidence": value.get("evidence").is_some()
        },
        "report": value
    });
    let rendered = crate::emit_payload(args.common.format, args.common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

fn run_artifacts_report_index(args: ArtifactsReportScanArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let reports_root = resolve_reports_root(&repo_root, args.reports_root.as_deref());
    let rows = scan_report_artifacts(&reports_root, &repo_root)?;
    let payload = serde_json::json!({
        "report_id": "artifact-report-index",
        "version": 1,
        "run_id": stable_run_id_for_root(&reports_root),
        "inputs": {
            "reports_root": relative_or_absolute(&repo_root, &reports_root)
        },
        "summary": {
            "report_count": rows.len()
        },
        "evidence": {
            "deterministic_sort": "report_id,path"
        },
        "index": rows.iter().map(|row| serde_json::json!({
            "report_id": row.report_id,
            "path": row.path,
            "version": row.version,
            "digest_sha256": row.digest_sha256
        })).collect::<Vec<_>>()
    });
    let rendered = crate::emit_payload(args.common.format, args.common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

fn run_artifacts_report_diff(args: ArtifactsReportDiffArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let baseline_rows = scan_report_artifacts(&args.baseline_root, &repo_root)?;
    let candidate_rows = scan_report_artifacts(&args.candidate_root, &repo_root)?;
    let baseline_map = baseline_rows
        .into_iter()
        .map(|row| (format!("{}::{}", row.report_id, row.path), row))
        .collect::<BTreeMap<_, _>>();
    let candidate_map = candidate_rows
        .into_iter()
        .map(|row| (format!("{}::{}", row.report_id, row.path), row))
        .collect::<BTreeMap<_, _>>();

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();
    for (key, row) in &candidate_map {
        match baseline_map.get(key) {
            None => added.push(report_artifact_row_json(row)),
            Some(base) if base.digest_sha256 != row.digest_sha256 => {
                changed.push(serde_json::json!({
                    "report_id": row.report_id,
                    "path": row.path,
                    "baseline_digest_sha256": base.digest_sha256,
                    "candidate_digest_sha256": row.digest_sha256
                }))
            }
            _ => {}
        }
    }
    for (key, row) in &baseline_map {
        if !candidate_map.contains_key(key) {
            removed.push(report_artifact_row_json(row));
        }
    }

    let payload = serde_json::json!({
        "report_id": "artifact-report-diff",
        "version": 1,
        "run_id": format!("{}__{}", stable_run_id_for_root(&args.baseline_root), stable_run_id_for_root(&args.candidate_root)),
        "inputs": {
            "baseline_root": relative_or_absolute(&repo_root, &args.baseline_root),
            "candidate_root": relative_or_absolute(&repo_root, &args.candidate_root)
        },
        "summary": {
            "added": added.len(),
            "removed": removed.len(),
            "changed": changed.len()
        },
        "evidence": {
            "comparison_key": "report_id+path"
        },
        "added": added,
        "removed": removed,
        "changed": changed
    });
    let changed_surface = !payload["added"].as_array().is_some_and(Vec::is_empty)
        || !payload["removed"].as_array().is_some_and(Vec::is_empty)
        || !payload["changed"].as_array().is_some_and(Vec::is_empty);
    let rendered = crate::emit_payload(args.common.format, args.common.out.clone(), &payload)?;
    Ok((rendered, if changed_surface { 1 } else { 0 }))
}

fn run_artifacts_report_validate(args: ArtifactsReportScanArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let reports_root = resolve_reports_root(&repo_root, args.reports_root.as_deref());
    let schema_inventory = load_report_schema_inventory(&repo_root)?;
    let ownership = load_report_ownership(&repo_root)?;
    let check_map = load_report_check_map(&repo_root)?;
    let evidence_levels = load_evidence_levels(&repo_root)?;
    let rows = scan_report_artifacts(&reports_root, &repo_root)?;
    let budget = load_report_budget(&repo_root)?;
    let known_reports = schema_inventory
        .iter()
        .map(|row| row.report_id.as_str())
        .collect::<BTreeSet<_>>();
    let total_bytes = rows.iter().map(|row| row.size_bytes).sum::<u64>();
    let mut errors = Vec::new();
    for row in &rows {
        if !known_reports.contains(row.report_id.as_str()) {
            errors.push(format!(
                "report `{}` at `{}` is not registered in {}",
                row.report_id, row.path, REPORT_SCHEMA_REGISTRY_PATH
            ));
        }
        if !ownership.contains_key(&row.report_id) {
            errors.push(format!(
                "report `{}` at `{}` is missing ownership in {}",
                row.report_id, row.path, REPORT_OWNERSHIP_PATH
            ));
        }
        if !check_map.contains_key(&row.report_id) {
            errors.push(format!(
                "report `{}` at `{}` is not referenced by any check in {}",
                row.report_id, row.path, REPORT_CHECK_MAP_PATH
            ));
        }
        if !row.has_summary {
            errors.push(format!(
                "report `{}` at `{}` is missing `summary`",
                row.report_id, row.path
            ));
        }
        if !row.has_evidence {
            errors.push(format!(
                "report `{}` at `{}` is missing `evidence`",
                row.report_id, row.path
            ));
        }
        if row.size_bytes > budget.max_single_report_bytes {
            errors.push(format!(
                "report `{}` at `{}` exceeds single-report budget {} > {} bytes",
                row.report_id, row.path, row.size_bytes, budget.max_single_report_bytes
            ));
        }
    }
    for (report_id, levels) in &check_map {
        if !ownership.contains_key(report_id) {
            errors.push(format!(
                "check map references unknown report `{}` not present in {}",
                report_id, REPORT_OWNERSHIP_PATH
            ));
        }
        for level in levels {
            if !evidence_levels.contains(level) {
                errors.push(format!(
                    "report `{}` uses unknown evidence level `{}` in {}",
                    report_id, level, REPORT_CHECK_MAP_PATH
                ));
            }
        }
    }
    if rows.len() > budget.max_report_count {
        errors.push(format!(
            "report count exceeds budget {} > {}",
            rows.len(),
            budget.max_report_count
        ));
    }
    if total_bytes > budget.max_total_bytes {
        errors.push(format!(
            "total report bytes exceed budget {} > {}",
            total_bytes, budget.max_total_bytes
        ));
    }

    let payload = serde_json::json!({
        "report_id": "artifact-report-validation",
        "version": 1,
        "run_id": stable_run_id_for_root(&reports_root),
        "inputs": {
            "reports_root": relative_or_absolute(&repo_root, &reports_root),
            "budget_file": REPORT_BUDGET_PATH
        },
        "summary": {
            "report_count": rows.len(),
            "total_bytes": total_bytes,
            "error_count": errors.len()
        },
        "evidence": {
            "known_report_schemas": schema_inventory.len(),
            "ownership_entries": ownership.len(),
            "check_map_reports": check_map.len()
        },
        "errors": errors,
        "reports": rows.iter().map(report_artifact_row_json).collect::<Vec<_>>()
    });
    let rendered = crate::emit_payload(args.common.format, args.common.out.clone(), &payload)?;
    let code = if payload["errors"]
        .as_array()
        .is_some_and(|rows| rows.is_empty())
    {
        0
    } else {
        1
    };
    Ok((rendered, code))
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

#[derive(Clone)]
struct ReportSchemaRow {
    schema_path: String,
    report_id: String,
    version: u64,
}

#[derive(Clone)]
struct ReportArtifactRow {
    report_id: String,
    version: u64,
    path: String,
    size_bytes: u64,
    digest_sha256: String,
    has_summary: bool,
    has_evidence: bool,
}

struct ReportBudget {
    max_report_count: usize,
    max_total_bytes: u64,
    max_single_report_bytes: u64,
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

fn load_report_schema_inventory(repo_root: &Path) -> Result<Vec<ReportSchemaRow>, String> {
    let registry_path = repo_root.join(REPORT_SCHEMA_REGISTRY_PATH);
    if registry_path.exists() {
        let value: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&registry_path)
                .map_err(|err| format!("read {} failed: {err}", registry_path.display()))?,
        )
        .map_err(|err| format!("parse {} failed: {err}", registry_path.display()))?;
        let mut rows = value
            .get("reports")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|row| ReportSchemaRow {
                schema_path: row
                    .get("schema_path")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                report_id: row
                    .get("report_id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                version: row
                    .get("version")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0),
            })
            .collect::<Vec<_>>();
        rows.sort_by(|a, b| {
            a.report_id
                .cmp(&b.report_id)
                .then_with(|| a.schema_path.cmp(&b.schema_path))
        });
        return Ok(rows);
    }
    let mut rows = Vec::new();
    for path in walk_json_files(&repo_root.join(REPORT_SCHEMAS_DIR))? {
        let value: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&path)
                .map_err(|err| format!("read {} failed: {err}", path.display()))?,
        )
        .map_err(|err| format!("parse {} failed: {err}", path.display()))?;
        let report_id = value
            .get("properties")
            .and_then(|value| value.get("report_id"))
            .and_then(|value| value.get("const"))
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("{} must define properties.report_id.const", path.display()))?;
        let version = value
            .get("properties")
            .and_then(|value| value.get("version"))
            .and_then(|value| value.get("const"))
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| format!("{} must define properties.version.const", path.display()))?;
        rows.push(ReportSchemaRow {
            schema_path: relative_or_absolute(repo_root, &path),
            report_id: report_id.to_string(),
            version,
        });
    }
    rows.sort_by(|a, b| {
        a.report_id
            .cmp(&b.report_id)
            .then_with(|| a.schema_path.cmp(&b.schema_path))
    });
    Ok(rows)
}

fn load_report_ownership(repo_root: &Path) -> Result<BTreeMap<String, String>, String> {
    let value = load_json(repo_root, REPORT_OWNERSHIP_PATH)?;
    let mut rows = BTreeMap::new();
    for report in value
        .get("reports")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
    {
        let Some(report_id) = report.get("report_id").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let Some(owner) = report.get("owner").and_then(serde_json::Value::as_str) else {
            continue;
        };
        rows.insert(report_id.to_string(), owner.to_string());
    }
    Ok(rows)
}

fn load_report_check_map(repo_root: &Path) -> Result<BTreeMap<String, Vec<String>>, String> {
    let value = load_json(repo_root, REPORT_CHECK_MAP_PATH)?;
    let mut rows = BTreeMap::<String, Vec<String>>::new();
    for mapping in value
        .get("mappings")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
    {
        let Some(report_id) = mapping.get("report_id").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let Some(level) = mapping
            .get("evidence_level")
            .and_then(serde_json::Value::as_str)
        else {
            continue;
        };
        rows.entry(report_id.to_string())
            .or_default()
            .push(level.to_string());
    }
    for levels in rows.values_mut() {
        levels.sort();
        levels.dedup();
    }
    Ok(rows)
}

fn load_evidence_levels(repo_root: &Path) -> Result<BTreeSet<String>, String> {
    let value = load_json(repo_root, "ops/report/evidence-levels.json")?;
    Ok(value
        .get("levels")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| {
            row.get("id")
                .and_then(serde_json::Value::as_str)
                .map(ToString::to_string)
        })
        .collect())
}

fn load_json(repo_root: &Path, rel: &str) -> Result<serde_json::Value, String> {
    let path = repo_root.join(rel);
    serde_json::from_str(
        &fs::read_to_string(&path)
            .map_err(|err| format!("read {} failed: {err}", path.display()))?,
    )
    .map_err(|err| format!("parse {} failed: {err}", path.display()))
}

fn scan_report_artifacts(
    reports_root: &Path,
    repo_root: &Path,
) -> Result<Vec<ReportArtifactRow>, String> {
    let mut rows = Vec::new();
    for path in walk_json_files(reports_root)? {
        let text = fs::read_to_string(&path)
            .map_err(|err| format!("read {} failed: {err}", path.display()))?;
        let value: serde_json::Value = serde_json::from_str(&text)
            .map_err(|err| format!("parse {} failed: {err}", path.display()))?;
        let Some(report_id) = value.get("report_id").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let Some(version) = value.get("version").and_then(serde_json::Value::as_u64) else {
            continue;
        };
        if value
            .get("inputs")
            .and_then(serde_json::Value::as_object)
            .is_none()
        {
            continue;
        }
        rows.push(ReportArtifactRow {
            report_id: report_id.to_string(),
            version,
            path: relative_or_absolute(repo_root, &path),
            size_bytes: text.as_bytes().len() as u64,
            digest_sha256: sha256_hex(&text),
            has_summary: value.get("summary").is_some(),
            has_evidence: value.get("evidence").is_some(),
        });
    }
    rows.sort_by(|a, b| {
        a.report_id
            .cmp(&b.report_id)
            .then_with(|| a.path.cmp(&b.path))
    });
    Ok(rows)
}

fn walk_json_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut stack = vec![root.to_path_buf()];
    let mut files = Vec::new();
    while let Some(path) = stack.pop() {
        if !path.exists() {
            continue;
        }
        if path.is_dir() {
            let mut entries = fs::read_dir(&path)
                .map_err(|err| format!("read {} failed: {err}", path.display()))?
                .flatten()
                .map(|entry| entry.path())
                .collect::<Vec<_>>();
            entries.sort();
            entries.reverse();
            stack.extend(entries);
            continue;
        }
        if path.extension().and_then(|value| value.to_str()) == Some("json") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn resolve_reports_root(repo_root: &Path, reports_root: Option<&Path>) -> PathBuf {
    reports_root
        .map(Path::to_path_buf)
        .unwrap_or_else(|| repo_root.join("artifacts"))
}

fn relative_or_absolute(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn stable_run_id_for_root(root: &Path) -> String {
    root.components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("-")
}

fn load_report_budget(repo_root: &Path) -> Result<ReportBudget, String> {
    let path = repo_root.join(REPORT_BUDGET_PATH);
    let value: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&path)
            .map_err(|err| format!("read {} failed: {err}", path.display()))?,
    )
    .map_err(|err| format!("parse {} failed: {err}", path.display()))?;
    Ok(ReportBudget {
        max_report_count: value
            .get("max_report_count")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| format!("{} must define max_report_count", path.display()))?
            as usize,
        max_total_bytes: value
            .get("max_total_bytes")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| format!("{} must define max_total_bytes", path.display()))?,
        max_single_report_bytes: value
            .get("max_single_report_bytes")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| format!("{} must define max_single_report_bytes", path.display()))?,
    })
}

fn report_schema_row_json(row: &ReportSchemaRow, owner: Option<&str>) -> serde_json::Value {
    serde_json::json!({
        "schema_path": row.schema_path,
        "report_id": row.report_id,
        "version": row.version,
        "owner": owner.unwrap_or("")
    })
}

fn report_artifact_row_json(row: &ReportArtifactRow) -> serde_json::Value {
    serde_json::json!({
        "report_id": row.report_id,
        "version": row.version,
        "path": row.path,
        "size_bytes": row.size_bytes,
        "digest_sha256": row.digest_sha256,
        "has_summary": row.has_summary,
        "has_evidence": row.has_evidence
    })
}

fn sha256_hex(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}
