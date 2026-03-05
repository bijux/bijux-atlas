// SPDX-License-Identifier: Apache-2.0

use crate::cli::{InvariantsCommand, InvariantsCommonArgs};
use crate::{emit_payload, resolve_repo_root};
use bijux_dev_atlas::contracts::system_invariants::{
    registry, InvariantGroup, InvariantSeverity, SystemInvariant,
};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::time::Instant;

const INVARIANT_REGISTRY_INDEX_PATH: &str = "ops/invariants/registry.json";

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    serde_json::from_str(
        &fs::read_to_string(path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn read_yaml(path: &Path) -> Result<serde_yaml::Value, String> {
    serde_yaml::from_str(
        &fs::read_to_string(path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes =
        fs::read(path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum ViolationClass {
    MissingFile,
    SchemaMismatch,
    IntegrityMismatch,
    ReferenceMismatch,
    DuplicateValue,
    OrderingMismatch,
    InvalidValue,
}

#[derive(Debug, Clone, serde::Serialize)]
struct InvariantViolation {
    class: ViolationClass,
    message: String,
    path: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct InvariantResult {
    id: String,
    title: String,
    severity: InvariantSeverity,
    group: InvariantGroup,
    status: String,
    violations: Vec<InvariantViolation>,
}

fn runtime_start_gate_result(failed_ids: &[String]) -> InvariantResult {
    if failed_ids.is_empty() {
        return InvariantResult {
            id: "INV-RUNTIME-START-GATE-001".to_string(),
            title: "Runtime start requires invariant pass".to_string(),
            severity: InvariantSeverity::Critical,
            group: InvariantGroup::Runtime,
            status: "pass".to_string(),
            violations: Vec::new(),
        };
    }

    InvariantResult {
        id: "INV-RUNTIME-START-GATE-001".to_string(),
        title: "Runtime start requires invariant pass".to_string(),
        severity: InvariantSeverity::Critical,
        group: InvariantGroup::Runtime,
        status: "fail".to_string(),
        violations: vec![InvariantViolation {
            class: ViolationClass::ReferenceMismatch,
            message: format!(
                "runtime start is blocked because invariants failed: {}",
                failed_ids.join(", ")
            ),
            path: None,
        }],
    }
}

fn validate_registry_index(root: &Path) -> Result<(Vec<String>, Vec<String>), String> {
    let index_path = root.join(INVARIANT_REGISTRY_INDEX_PATH);
    let value = read_json(&index_path)?;
    let listed_ids = value
        .get("invariants")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|entry| {
            entry
                .get("id")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })
        .collect::<BTreeSet<_>>();
    let runtime_ids = registry()
        .into_iter()
        .map(|row| row.id.to_string())
        .collect::<BTreeSet<_>>();
    let missing_in_index = runtime_ids
        .difference(&listed_ids)
        .cloned()
        .collect::<Vec<_>>();
    let extra_in_index = listed_ids
        .difference(&runtime_ids)
        .cloned()
        .collect::<Vec<_>>();
    Ok((missing_in_index, extra_in_index))
}

fn record_missing(path: &Path, message: &str) -> InvariantViolation {
    InvariantViolation {
        class: ViolationClass::MissingFile,
        message: message.to_string(),
        path: Some(path.display().to_string()),
    }
}

fn record_violation(
    class: ViolationClass,
    message: String,
    path: Option<&Path>,
) -> InvariantViolation {
    InvariantViolation {
        class,
        message,
        path: path.map(|p| p.display().to_string()),
    }
}

fn evaluate_one(root: &Path, inv: SystemInvariant) -> InvariantResult {
    let mut violations = Vec::<InvariantViolation>::new();
    match inv.id {
        "INV-CONFIG-SCHEMA-VERSION-001" => {
            let cfg = root.join("configs/inventory.json");
            match read_json(&cfg) {
                Ok(value) => {
                    let got = value
                        .get("schema_version")
                        .and_then(serde_json::Value::as_i64)
                        .unwrap_or(0);
                    if got != 1 {
                        violations.push(record_violation(
                            ViolationClass::SchemaMismatch,
                            format!("config inventory schema_version must be 1 but found {got}"),
                            Some(&cfg),
                        ));
                    }
                }
                Err(_) => violations.push(record_missing(
                    &cfg,
                    "config inventory schema file is missing",
                )),
            }
        }
        "INV-ARTIFACT-HASH-REGISTRY-001"
        | "INV-REGISTRY-REF-EXISTS-001"
        | "INV-REGISTRY-CHECKSUM-001"
        | "INV-SHARD-DIR-REGISTRY-001"
        | "INV-SHARD-ORPHAN-001" => {
            let manifest_path = root.join("release/evidence/manifest.json");
            match read_json(&manifest_path) {
                Ok(value) => {
                    let rows = value
                        .get("artifact_list")
                        .and_then(serde_json::Value::as_array)
                        .cloned()
                        .unwrap_or_default();
                    let mut seen_paths = BTreeSet::<String>::new();
                    for row in rows {
                        let path = row
                            .get("path")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or_default()
                            .to_string();
                        let sha = row
                            .get("sha256")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or_default()
                            .to_string();
                        if path.is_empty() {
                            violations.push(record_violation(
                                ViolationClass::InvalidValue,
                                "manifest artifact_list entry has empty path".to_string(),
                                Some(&manifest_path),
                            ));
                            continue;
                        }
                        seen_paths.insert(path.clone());
                        let abs = root.join(&path);
                        if !abs.exists() {
                            if inv.id == "INV-REGISTRY-REF-EXISTS-001" {
                                violations.push(record_violation(
                                    ViolationClass::ReferenceMismatch,
                                    format!("manifest references missing artifact `{path}`"),
                                    Some(&manifest_path),
                                ));
                            }
                            continue;
                        }
                        if inv.id == "INV-REGISTRY-CHECKSUM-001" && sha.trim().is_empty() {
                            violations.push(record_violation(
                                ViolationClass::InvalidValue,
                                format!("manifest entry `{path}` is missing sha256"),
                                Some(&manifest_path),
                            ));
                        }
                        if inv.id == "INV-ARTIFACT-HASH-REGISTRY-001" && !sha.trim().is_empty() {
                            match sha256_file(&abs) {
                                Ok(actual) if actual != sha => violations.push(record_violation(
                                    ViolationClass::IntegrityMismatch,
                                    format!("artifact hash mismatch for `{path}`"),
                                    Some(&abs),
                                )),
                                Ok(_) => {}
                                Err(err) => violations.push(record_violation(
                                    ViolationClass::IntegrityMismatch,
                                    err,
                                    Some(&abs),
                                )),
                            }
                        }
                    }
                    if inv.id == "INV-SHARD-DIR-REGISTRY-001" || inv.id == "INV-SHARD-ORPHAN-001" {
                        let shard_like = root.join("ops/datasets/generated");
                        if shard_like.exists() {
                            for entry in fs::read_dir(&shard_like).into_iter().flatten().flatten() {
                                let p = entry.path();
                                if p.extension().and_then(|v| v.to_str()) == Some("json") {
                                    let rel =
                                        p.strip_prefix(root).unwrap_or(&p).display().to_string();
                                    if !seen_paths.contains(&rel) {
                                        violations.push(record_violation(
                                            ViolationClass::ReferenceMismatch,
                                            format!("generated dataset artifact `{rel}` is not represented in release evidence manifest"),
                                            Some(&manifest_path),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
                Err(_) => violations.push(record_missing(
                    &manifest_path,
                    "release evidence manifest is missing",
                )),
            }
        }
        "INV-DATASET-ID-UNIQUE-001"
        | "INV-DATASET-SCHEMA-METADATA-001"
        | "INV-SHARD-ID-DETERMINISTIC-001" => {
            let index_path = root.join("ops/datasets/generated/dataset-index.json");
            let lock_path = root.join("ops/datasets/manifest.lock");
            let index = read_json(&index_path);
            let lock = read_json(&lock_path);
            match (index, lock) {
                (Ok(index), Ok(lock)) => {
                    let ids = index
                        .get("dataset_ids")
                        .and_then(serde_json::Value::as_array)
                        .cloned()
                        .unwrap_or_default()
                        .into_iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect::<Vec<_>>();
                    if inv.id == "INV-DATASET-ID-UNIQUE-001" {
                        let mut seen = BTreeSet::new();
                        for id in &ids {
                            if !seen.insert(id.clone()) {
                                violations.push(record_violation(
                                    ViolationClass::DuplicateValue,
                                    format!("duplicate dataset id `{id}`"),
                                    Some(&index_path),
                                ));
                            }
                        }
                    }
                    if inv.id == "INV-DATASET-SCHEMA-METADATA-001" {
                        let index_version = index
                            .get("schema_version")
                            .and_then(serde_json::Value::as_i64)
                            .unwrap_or(0);
                        let lock_version = lock
                            .get("schema_version")
                            .and_then(serde_json::Value::as_i64)
                            .unwrap_or(0);
                        if index_version != lock_version {
                            violations.push(record_violation(
                                ViolationClass::SchemaMismatch,
                                format!("dataset index schema_version ({index_version}) differs from manifest.lock schema_version ({lock_version})"),
                                Some(&index_path),
                            ));
                        }
                    }
                    if inv.id == "INV-SHARD-ID-DETERMINISTIC-001" {
                        let mut sorted = ids.clone();
                        sorted.sort();
                        if sorted != ids {
                            violations.push(record_violation(
                                ViolationClass::OrderingMismatch,
                                "dataset_ids must be deterministically sorted".to_string(),
                                Some(&index_path),
                            ));
                        }
                    }
                }
                (Err(_), _) => {
                    violations.push(record_missing(&index_path, "dataset index is missing"))
                }
                (_, Err(_)) => violations.push(record_missing(
                    &lock_path,
                    "dataset manifest lock is missing",
                )),
            }
        }
        "INV-CONFIG-DATASET-REF-001" => {
            let index_path = root.join("ops/datasets/generated/dataset-index.json");
            let offline_values = root.join("ops/k8s/values/offline.yaml");
            match (read_json(&index_path), read_yaml(&offline_values)) {
                (Ok(index), Ok(values)) => {
                    let known = index
                        .get("dataset_ids")
                        .and_then(serde_json::Value::as_array)
                        .cloned()
                        .unwrap_or_default()
                        .into_iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect::<BTreeSet<_>>();
                    let pinned = values
                        .get("cache")
                        .and_then(serde_yaml::Value::as_mapping)
                        .and_then(|cache| {
                            cache.get(serde_yaml::Value::String("pinnedDatasets".to_string()))
                        })
                        .and_then(serde_yaml::Value::as_sequence)
                        .cloned()
                        .unwrap_or_default()
                        .into_iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect::<Vec<_>>();
                    for id in pinned {
                        if !known.contains(&id) {
                            violations.push(record_violation(
                                ViolationClass::ReferenceMismatch,
                                format!("offline values references unknown dataset `{id}`"),
                                Some(&offline_values),
                            ));
                        }
                    }
                }
                (Err(_), _) => {
                    violations.push(record_missing(&index_path, "dataset index is missing"))
                }
                (_, Err(_)) => violations.push(record_missing(
                    &offline_values,
                    "offline values file is missing",
                )),
            }
        }
        "INV-CONFIG-PROFILE-REF-001" | "INV-PROFILE-NAME-UNIQUE-001" => {
            let profiles_path = root.join("ops/stack/profile-registry.json");
            let matrix_path = root.join("ops/k8s/install-matrix.json");
            match (read_json(&profiles_path), read_json(&matrix_path)) {
                (Ok(profiles), Ok(matrix)) => {
                    let profile_ids = profiles
                        .get("profiles")
                        .and_then(serde_json::Value::as_array)
                        .cloned()
                        .unwrap_or_default()
                        .into_iter()
                        .filter_map(|v| {
                            v.get("id")
                                .and_then(serde_json::Value::as_str)
                                .map(str::to_string)
                        })
                        .collect::<Vec<_>>();
                    if inv.id == "INV-PROFILE-NAME-UNIQUE-001" {
                        let mut seen = BTreeSet::new();
                        for id in &profile_ids {
                            if !seen.insert(id.clone()) {
                                violations.push(record_violation(
                                    ViolationClass::DuplicateValue,
                                    format!("duplicate profile id `{id}`"),
                                    Some(&profiles_path),
                                ));
                            }
                        }
                    }
                    if inv.id == "INV-CONFIG-PROFILE-REF-001" {
                        let known = profile_ids.into_iter().collect::<BTreeSet<_>>();
                        for row in matrix
                            .get("profiles")
                            .and_then(serde_json::Value::as_array)
                            .cloned()
                            .unwrap_or_default()
                        {
                            if let Some(name) = row.get("name").and_then(serde_json::Value::as_str)
                            {
                                if !known.contains(name) {
                                    violations.push(record_violation(
                                        ViolationClass::ReferenceMismatch,
                                        format!(
                                            "install matrix references unknown profile `{name}`"
                                        ),
                                        Some(&matrix_path),
                                    ));
                                }
                            }
                        }
                    }
                }
                (Err(_), _) => violations.push(record_missing(
                    &profiles_path,
                    "profile registry is missing",
                )),
                (_, Err(_)) => {
                    violations.push(record_missing(&matrix_path, "install matrix is missing"))
                }
            }
        }
        "INV-PROFILE-INHERIT-CYCLE-001" => {
            let path = root.join("ops/k8s/values/profiles.json");
            match read_json(&path) {
                Ok(value) => {
                    let rows = value
                        .get("profiles")
                        .and_then(serde_json::Value::as_array)
                        .cloned()
                        .unwrap_or_default();
                    let mut graph = BTreeMap::<String, Option<String>>::new();
                    for row in rows {
                        let id = row
                            .get("id")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or_default()
                            .to_string();
                        let parent = row
                            .get("inherits_from")
                            .and_then(serde_json::Value::as_str)
                            .map(str::to_string);
                        if !id.is_empty() {
                            graph.insert(id, parent);
                        }
                    }
                    for node in graph.keys() {
                        let mut seen = BTreeSet::<String>::new();
                        let mut cursor = Some(node.clone());
                        while let Some(current) = cursor {
                            if !seen.insert(current.clone()) {
                                violations.push(record_violation(
                                    ViolationClass::ReferenceMismatch,
                                    format!("profile inheritance cycle detected at `{current}`"),
                                    Some(&path),
                                ));
                                break;
                            }
                            cursor = graph.get(&current).cloned().flatten();
                        }
                    }
                }
                Err(_) => violations.push(record_missing(
                    &path,
                    "profile inheritance source is missing",
                )),
            }
        }
        "INV-PROFILE-OVERRIDE-SCHEMA-001" => {
            let path = root.join("ops/k8s/values/profiles.json");
            match read_json(&path) {
                Ok(value) => {
                    for row in value
                        .get("profiles")
                        .and_then(serde_json::Value::as_array)
                        .cloned()
                        .unwrap_or_default()
                    {
                        let values_file = row
                            .get("values_file")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or_default();
                        if values_file.is_empty() {
                            continue;
                        }
                        let abs = root.join(values_file);
                        if !abs.exists() {
                            violations.push(record_violation(
                                ViolationClass::MissingFile,
                                format!("profile values file is missing: {values_file}"),
                                Some(&abs),
                            ));
                            continue;
                        }
                        if read_yaml(&abs).is_err() {
                            violations.push(record_violation(
                                ViolationClass::InvalidValue,
                                format!("profile values file is not valid YAML: {values_file}"),
                                Some(&abs),
                            ));
                        }
                    }
                }
                Err(_) => {
                    violations.push(record_missing(&path, "profile values source is missing"))
                }
            }
        }
        "INV-RUNTIME-START-GATE-001" => {}
        _ => {}
    }
    InvariantResult {
        id: inv.id.to_string(),
        title: inv.title.to_string(),
        severity: inv.severity,
        group: inv.group,
        status: if violations.is_empty() {
            "pass".to_string()
        } else {
            "fail".to_string()
        },
        violations,
    }
}

fn run_invariants(common: InvariantsCommonArgs) -> Result<(String, i32), String> {
    let started = Instant::now();
    let root = resolve_repo_root(common.repo_root)?;
    let mut results = registry()
        .into_iter()
        .filter(|inv| inv.id != "INV-RUNTIME-START-GATE-001")
        .map(|inv| evaluate_one(&root, inv))
        .collect::<Vec<_>>();
    let prereq_failures = results
        .iter()
        .filter(|row| row.status == "fail")
        .map(|row| row.id.clone())
        .collect::<Vec<_>>();
    results.push(runtime_start_gate_result(&prereq_failures));
    results.sort_by(|a, b| a.id.cmp(&b.id));
    let (missing_in_index, extra_in_index) = validate_registry_index(&root)
        .unwrap_or_else(|_| (vec!["index_unreadable".to_string()], Vec::<String>::new()));
    let failed = results.iter().filter(|row| row.status == "fail").count();
    let registry_complete = missing_in_index.is_empty() && extra_in_index.is_empty();
    let status = if failed == 0 && registry_complete {
        "ok"
    } else {
        "failed"
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "system_invariant_report",
        "status": status,
        "metrics": {
            "execution_time_ms": started.elapsed().as_millis() as u64
        },
        "summary": {
            "total": results.len(),
            "failed": failed,
            "passed": results.len().saturating_sub(failed)
        },
        "registry_completeness": {
            "status": if registry_complete { "pass" } else { "fail" },
            "missing_in_index": missing_in_index,
            "extra_in_index": extra_in_index
        },
        "results": results
    });
    let rendered = emit_payload(common.format, common.out, &payload)?;
    Ok((
        rendered,
        if failed == 0 && registry_complete {
            0
        } else {
            3
        },
    ))
}

fn list_invariants(common: InvariantsCommonArgs) -> Result<(String, i32), String> {
    let mut rows = registry()
        .into_iter()
        .map(|inv| {
            serde_json::json!({
                "id": inv.id,
                "title": inv.title,
                "severity": inv.severity,
                "group": inv.group
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        a["id"]
            .as_str()
            .unwrap_or_default()
            .cmp(b["id"].as_str().unwrap_or_default())
    });
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "system_invariant_list",
        "status": "ok",
        "rows": rows
    });
    let rendered = emit_payload(common.format, common.out, &payload)?;
    Ok((rendered, 0))
}

fn explain_invariant(id: String, common: InvariantsCommonArgs) -> Result<(String, i32), String> {
    let Some(inv) = registry().into_iter().find(|row| row.id == id) else {
        let payload = serde_json::json!({
            "schema_version": 1,
            "kind": "system_invariant_explain",
            "status": "failed",
            "error": format!("unknown invariant id `{id}`")
        });
        let rendered = emit_payload(common.format, common.out, &payload)?;
        return Ok((rendered, 2));
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "system_invariant_explain",
        "status": "ok",
        "invariant": inv
    });
    let rendered = emit_payload(common.format, common.out, &payload)?;
    Ok((rendered, 0))
}

fn coverage_report(common: InvariantsCommonArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(common.repo_root)?;
    let rows = registry();
    let mut by_group = BTreeMap::<String, usize>::new();
    let mut by_severity = BTreeMap::<String, usize>::new();
    for row in &rows {
        *by_group
            .entry(format!("{:?}", row.group).to_lowercase())
            .or_insert(0) += 1;
        *by_severity
            .entry(format!("{:?}", row.severity).to_lowercase())
            .or_insert(0) += 1;
    }
    let (missing_in_index, extra_in_index) = validate_registry_index(&root)?;
    let complete = missing_in_index.is_empty() && extra_in_index.is_empty();
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "system_invariant_coverage",
        "status": if complete { "ok" } else { "failed" },
        "summary": {
            "total_invariants": rows.len(),
            "group_counts": by_group,
            "severity_counts": by_severity
        },
        "registry_completeness": {
            "missing_in_index": missing_in_index,
            "extra_in_index": extra_in_index
        }
    });
    let rendered = emit_payload(common.format, common.out, &payload)?;
    Ok((rendered, if complete { 0 } else { 3 }))
}

fn generate_docs(common: InvariantsCommonArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(common.repo_root)?;
    let mut rows = registry();
    rows.sort_by(|a, b| a.id.cmp(b.id));

    let mut lines = vec![
        "# System Invariants Reference".to_string(),
        "".to_string(),
        "Generated by `bijux-dev-atlas invariants docs`.".to_string(),
        "".to_string(),
    ];
    for row in &rows {
        lines.push(format!("## {}", row.id));
        lines.push(String::new());
        lines.push(format!("- title: {}", row.title));
        lines.push(format!(
            "- severity: {}",
            format!("{:?}", row.severity).to_lowercase()
        ));
        lines.push(format!(
            "- group: {}",
            format!("{:?}", row.group).to_lowercase()
        ));
        lines.push(format!("- summary: {}", row.summary));
        lines.push(String::new());
    }
    let out_path = root.join("docs/operations/system-invariants-reference.md");
    fs::write(&out_path, format!("{}\n", lines.join("\n")))
        .map_err(|err| format!("failed to write {}: {err}", out_path.display()))?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "system_invariant_docs",
        "status": "ok",
        "output": out_path.display().to_string(),
        "invariant_count": rows.len()
    });
    let rendered = emit_payload(common.format, common.out, &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_invariants_command(
    _quiet: bool,
    command: InvariantsCommand,
) -> Result<(String, i32), String> {
    match command {
        InvariantsCommand::Run(args) => run_invariants(args),
        InvariantsCommand::List(args) => list_invariants(args),
        InvariantsCommand::Explain(args) => explain_invariant(args.id, args.common),
        InvariantsCommand::Coverage(args) => coverage_report(args),
        InvariantsCommand::Docs(args) => generate_docs(args),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn runtime_gate_fails_when_any_invariant_fails() {
        let gate = runtime_start_gate_result(&["INV-A-001".to_string()]);
        assert_eq!(gate.status, "fail");
        assert_eq!(gate.id, "INV-RUNTIME-START-GATE-001");
        assert!(!gate.violations.is_empty());
    }

    #[test]
    fn runtime_gate_passes_when_no_failures() {
        let gate = runtime_start_gate_result(&[]);
        assert_eq!(gate.status, "pass");
        assert!(gate.violations.is_empty());
    }

    #[test]
    fn registry_index_completeness_detects_missing_entries() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        let index_path = root.join(INVARIANT_REGISTRY_INDEX_PATH);
        fs::create_dir_all(index_path.parent().expect("parent")).expect("mkdir");
        fs::write(
            &index_path,
            r#"{"schema_version":1,"invariants":[{"id":"INV-CONFIG-SCHEMA-VERSION-001"}]}"#,
        )
        .expect("write index");
        let (missing, extra) = validate_registry_index(root).expect("validate");
        assert!(!missing.is_empty());
        assert!(extra.is_empty());
    }
}
