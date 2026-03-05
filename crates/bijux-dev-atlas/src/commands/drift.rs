// SPDX-License-Identifier: Apache-2.0

use crate::cli::{DriftCommand, DriftCompareArgs, DriftDetectArgs, InvariantsCommonArgs};
use crate::{emit_payload, resolve_repo_root};
use bijux_dev_atlas::contracts::drift::{explain_drift_type, DriftClass, DriftSeverity, DriftType};
use bijux_dev_atlas::contracts::system_invariants;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct DriftFinding {
    drift_type: DriftType,
    severity: DriftSeverity,
    class: DriftClass,
    score: u8,
    message: String,
    path: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct DriftSummary {
    findings_total: usize,
    critical: usize,
    high: usize,
    medium: usize,
    low: usize,
    aggregate_score: usize,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct DriftIgnoreFile {
    schema_version: u32,
    ignores: Vec<DriftIgnoreRule>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct DriftIgnoreRule {
    drift_type: Option<String>,
    message_contains: Option<String>,
    path_contains: Option<String>,
}

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn read_yaml(path: &Path) -> Result<serde_yaml::Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_yaml::from_str(&text).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn score(severity: DriftSeverity) -> u8 {
    match severity {
        DriftSeverity::Critical => 100,
        DriftSeverity::High => 70,
        DriftSeverity::Medium => 40,
        DriftSeverity::Low => 15,
    }
}

fn finding(
    drift_type: DriftType,
    severity: DriftSeverity,
    class: DriftClass,
    message: String,
    path: Option<&Path>,
) -> DriftFinding {
    DriftFinding {
        drift_type,
        severity,
        class,
        score: score(severity),
        message,
        path: path.map(|p| p.display().to_string()),
    }
}

fn detect_config_drift(root: &Path, out: &mut Vec<DriftFinding>) {
    let inv = root.join("configs/inventory.json");
    match read_json(&inv) {
        Ok(value) => {
            let schema_version = value
                .get("schema_version")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0);
            if schema_version != 1 {
                out.push(finding(
                    DriftType::Configuration,
                    DriftSeverity::High,
                    DriftClass::Mismatch,
                    format!(
                        "configs inventory schema_version drifted: expected 1, found {schema_version}"
                    ),
                    Some(&inv),
                ));
            }
        }
        Err(_) => out.push(finding(
            DriftType::Configuration,
            DriftSeverity::Critical,
            DriftClass::Missing,
            "configs inventory is missing".to_string(),
            Some(&inv),
        )),
    }
}

fn detect_artifact_drift(root: &Path, out: &mut Vec<DriftFinding>) {
    let manifest = root.join("release/evidence/manifest.json");
    match read_json(&manifest) {
        Ok(value) => {
            let rows = value
                .get("artifact_list")
                .and_then(serde_json::Value::as_array)
                .cloned()
                .unwrap_or_default();
            for row in rows {
                let rel = row
                    .get("path")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default();
                if rel.is_empty() {
                    continue;
                }
                let path = root.join(rel);
                if !path.exists() {
                    out.push(finding(
                        DriftType::Artifact,
                        DriftSeverity::Critical,
                        DriftClass::Missing,
                        format!("artifact listed in manifest is missing: {rel}"),
                        Some(&manifest),
                    ));
                }
            }
        }
        Err(_) => out.push(finding(
            DriftType::Artifact,
            DriftSeverity::Critical,
            DriftClass::Missing,
            "release evidence manifest is missing".to_string(),
            Some(&manifest),
        )),
    }
}

fn detect_registry_drift(root: &Path, out: &mut Vec<DriftFinding>) {
    let registry = root.join("ops/invariants/registry.json");
    let runtime_ids = system_invariants::registry()
        .into_iter()
        .map(|row| row.id.to_string())
        .collect::<BTreeSet<_>>();
    match read_json(&registry) {
        Ok(value) => {
            let listed = value
                .get("invariants")
                .and_then(serde_json::Value::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|row| {
                    row.get("id")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string)
                })
                .collect::<BTreeSet<_>>();
            for missing in runtime_ids.difference(&listed) {
                out.push(finding(
                    DriftType::Registry,
                    DriftSeverity::High,
                    DriftClass::Missing,
                    format!("invariant missing from registry index: {missing}"),
                    Some(&registry),
                ));
            }
            for extra in listed.difference(&runtime_ids) {
                out.push(finding(
                    DriftType::Registry,
                    DriftSeverity::Low,
                    DriftClass::Unexpected,
                    format!("registry index contains unknown invariant id: {extra}"),
                    Some(&registry),
                ));
            }
        }
        Err(_) => out.push(finding(
            DriftType::Registry,
            DriftSeverity::Critical,
            DriftClass::Missing,
            "invariant registry index is missing".to_string(),
            Some(&registry),
        )),
    }
}

fn detect_runtime_config_drift(root: &Path, out: &mut Vec<DriftFinding>) {
    let offline = root.join("ops/k8s/values/offline.yaml");
    let index = root.join("ops/datasets/generated/dataset-index.json");
    match (read_yaml(&offline), read_json(&index)) {
        (Ok(values), Ok(idx)) => {
            let known = idx
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
                .and_then(|m| m.get(serde_yaml::Value::String("pinnedDatasets".to_string())))
                .and_then(serde_yaml::Value::as_sequence)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect::<Vec<_>>();
            for id in pinned {
                if !known.contains(&id) {
                    out.push(finding(
                        DriftType::RuntimeConfig,
                        DriftSeverity::High,
                        DriftClass::Mismatch,
                        format!("runtime pinned dataset not present in dataset index: {id}"),
                        Some(&offline),
                    ));
                }
            }
        }
        (Err(_), _) => out.push(finding(
            DriftType::RuntimeConfig,
            DriftSeverity::High,
            DriftClass::Missing,
            "runtime values file missing".to_string(),
            Some(&offline),
        )),
        (_, Err(_)) => out.push(finding(
            DriftType::RuntimeConfig,
            DriftSeverity::High,
            DriftClass::Missing,
            "dataset index missing for runtime drift checks".to_string(),
            Some(&index),
        )),
    }
}

fn detect_ops_profile_drift(root: &Path, out: &mut Vec<DriftFinding>) {
    let profiles = root.join("ops/stack/profile-registry.json");
    let matrix = root.join("ops/k8s/install-matrix.json");
    match (read_json(&profiles), read_json(&matrix)) {
        (Ok(profiles_json), Ok(matrix_json)) => {
            let known = profiles_json
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
                .collect::<BTreeSet<_>>();
            for row in matrix_json
                .get("profiles")
                .and_then(serde_json::Value::as_array)
                .cloned()
                .unwrap_or_default()
            {
                if let Some(name) = row.get("name").and_then(serde_json::Value::as_str) {
                    if !known.contains(name) {
                        out.push(finding(
                            DriftType::OpsProfile,
                            DriftSeverity::Medium,
                            DriftClass::Mismatch,
                            format!("install matrix references unknown profile: {name}"),
                            Some(&matrix),
                        ));
                    }
                }
            }
        }
        (Err(_), _) => out.push(finding(
            DriftType::OpsProfile,
            DriftSeverity::High,
            DriftClass::Missing,
            "ops profile registry is missing".to_string(),
            Some(&profiles),
        )),
        (_, Err(_)) => out.push(finding(
            DriftType::OpsProfile,
            DriftSeverity::High,
            DriftClass::Missing,
            "ops install matrix is missing".to_string(),
            Some(&matrix),
        )),
    }
}

fn detect_all(root: &Path) -> Vec<DriftFinding> {
    let mut findings = Vec::new();
    detect_config_drift(root, &mut findings);
    detect_artifact_drift(root, &mut findings);
    detect_registry_drift(root, &mut findings);
    detect_runtime_config_drift(root, &mut findings);
    detect_ops_profile_drift(root, &mut findings);
    findings.sort_by(|a, b| {
        (
            format!("{:?}", a.drift_type),
            a.path.as_deref().unwrap_or_default(),
            a.message.as_str(),
        )
            .cmp(&(
                format!("{:?}", b.drift_type),
                b.path.as_deref().unwrap_or_default(),
                b.message.as_str(),
            ))
    });
    findings
}

fn summarize(findings: &[DriftFinding]) -> DriftSummary {
    let mut by_level = BTreeMap::<&'static str, usize>::new();
    let mut aggregate = 0usize;
    for row in findings {
        aggregate += row.score as usize;
        let key = match row.severity {
            DriftSeverity::Critical => "critical",
            DriftSeverity::High => "high",
            DriftSeverity::Medium => "medium",
            DriftSeverity::Low => "low",
        };
        *by_level.entry(key).or_insert(0) += 1;
    }
    DriftSummary {
        findings_total: findings.len(),
        critical: by_level.get("critical").copied().unwrap_or(0),
        high: by_level.get("high").copied().unwrap_or(0),
        medium: by_level.get("medium").copied().unwrap_or(0),
        low: by_level.get("low").copied().unwrap_or(0),
        aggregate_score: aggregate,
    }
}

fn drift_type_key(kind: DriftType) -> &'static str {
    match kind {
        DriftType::Configuration => "configuration",
        DriftType::Artifact => "artifact",
        DriftType::Registry => "registry",
        DriftType::RuntimeConfig => "runtime_config",
        DriftType::OpsProfile => "ops_profile",
    }
}

fn parse_drift_type(value: &str) -> Option<DriftType> {
    match value {
        "configuration" | "config" => Some(DriftType::Configuration),
        "artifact" => Some(DriftType::Artifact),
        "registry" => Some(DriftType::Registry),
        "runtime_config" | "runtime-config" => Some(DriftType::RuntimeConfig),
        "ops_profile" | "ops-profile" | "profile" => Some(DriftType::OpsProfile),
        _ => None,
    }
}

fn load_ignore_rules(path: Option<&Path>) -> Result<Vec<DriftIgnoreRule>, String> {
    let Some(path) = path else {
        return Ok(Vec::new());
    };
    let text = fs::read_to_string(path)
        .map_err(|err| format!("failed to read drift ignore file {}: {err}", path.display()))?;
    let file: DriftIgnoreFile = serde_json::from_str(&text).map_err(|err| {
        format!(
            "failed to parse drift ignore file {}: {err}",
            path.display()
        )
    })?;
    if file.schema_version != 1 {
        return Err(format!(
            "invalid drift ignore file {}: schema_version must be 1",
            path.display()
        ));
    }
    for (idx, rule) in file.ignores.iter().enumerate() {
        if rule.drift_type.is_none()
            && rule.message_contains.is_none()
            && rule.path_contains.is_none()
        {
            return Err(format!(
                "invalid drift ignore rule #{idx}: must set at least one of drift_type, message_contains, path_contains"
            ));
        }
        if let Some(ref kind) = rule.drift_type {
            if parse_drift_type(kind).is_none() {
                return Err(format!(
                    "invalid drift ignore rule #{idx}: unknown drift_type `{kind}`"
                ));
            }
        }
    }
    Ok(file.ignores)
}

fn apply_ignores(
    findings: Vec<DriftFinding>,
    rules: &[DriftIgnoreRule],
) -> (Vec<DriftFinding>, usize) {
    if rules.is_empty() {
        return (findings, 0);
    }
    let mut kept = Vec::new();
    let mut ignored = 0usize;
    for row in findings {
        let mut matched = false;
        for rule in rules {
            if let Some(ref type_filter) = rule.drift_type {
                if parse_drift_type(type_filter) != Some(row.drift_type) {
                    continue;
                }
            }
            if let Some(ref message_filter) = rule.message_contains {
                if !row.message.contains(message_filter) {
                    continue;
                }
            }
            if let Some(ref path_filter) = rule.path_contains {
                if !row
                    .path
                    .as_deref()
                    .unwrap_or_default()
                    .contains(path_filter)
                {
                    continue;
                }
            }
            matched = true;
            break;
        }
        if matched {
            ignored += 1;
        } else {
            kept.push(row);
        }
    }
    (kept, ignored)
}

fn detect_with_ignores(
    common: &InvariantsCommonArgs,
    ignore_file: Option<&Path>,
) -> Result<(serde_json::Value, i32), String> {
    let root = resolve_repo_root(common.repo_root.clone())?;
    let rules = load_ignore_rules(ignore_file)?;
    let raw = detect_all(&root);
    let (findings, ignored_count) = apply_ignores(raw, &rules);
    let summary = summarize(&findings);
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "drift_report",
        "status": if summary.findings_total == 0 { "ok" } else { "failed" },
        "summary": summary,
        "ignored_count": ignored_count,
        "findings": findings
    });
    Ok((payload, if summary.findings_total == 0 { 0 } else { 3 }))
}

fn detect(args: DriftDetectArgs) -> Result<(String, i32), String> {
    let (payload, code) = detect_with_ignores(&args.common, args.ignore_file.as_deref())?;
    let rendered = emit_payload(args.common.format, args.common.out, &payload)?;
    Ok((rendered, code))
}

fn report(args: DriftDetectArgs) -> Result<(String, i32), String> {
    detect(args)
}

fn coverage(args: DriftDetectArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.common.repo_root.clone())?;
    let rules = load_ignore_rules(args.ignore_file.as_deref())?;
    let raw = detect_all(&root);
    let (findings, ignored_count) = apply_ignores(raw, &rules);
    let mut by_type = BTreeMap::<String, usize>::new();
    for row in &findings {
        *by_type
            .entry(drift_type_key(row.drift_type).to_string())
            .or_insert(0) += 1;
    }
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "drift_coverage",
        "status": "ok",
        "detectors": [
            "configuration",
            "artifact",
            "registry",
            "runtime_config",
            "ops_profile"
        ],
        "ignored_count": ignored_count,
        "type_counts": by_type
    });
    let rendered = emit_payload(args.common.format, args.common.out, &payload)?;
    Ok((rendered, 0))
}

fn baseline(args: crate::cli::DriftBaselineArgs) -> Result<(String, i32), String> {
    let (payload, code) =
        detect_with_ignores(&args.detect.common, args.detect.ignore_file.as_deref())?;
    let root = resolve_repo_root(args.detect.common.repo_root.clone())?;
    let out_path = args
        .snapshot_out
        .unwrap_or_else(|| root.join("artifacts/drift/baseline.json"));
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(&payload)
        .map_err(|err| format!("failed to encode baseline payload: {err}"))?;
    fs::write(&out_path, format!("{text}\n"))
        .map_err(|err| format!("failed to write {}: {err}", out_path.display()))?;
    let mut report_payload = payload;
    if let Some(obj) = report_payload.as_object_mut() {
        obj.insert(
            "baseline_path".to_string(),
            serde_json::Value::String(out_path.display().to_string()),
        );
    }
    let rendered = emit_payload(
        args.detect.common.format,
        args.detect.common.out,
        &report_payload,
    )?;
    Ok((rendered, code))
}

fn compare(args: DriftCompareArgs) -> Result<(String, i32), String> {
    let baseline_value = read_json(&args.baseline)?;
    let baseline_findings = baseline_value
        .get("findings")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|row| serde_json::from_value::<DriftFinding>(row).map_err(|err| err.to_string()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("failed to parse baseline findings: {err}"))?;

    let current_findings = if let Some(path) = args.current.as_deref() {
        let value = read_json(path)?;
        value
            .get("findings")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|row| serde_json::from_value::<DriftFinding>(row).map_err(|err| err.to_string()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| format!("failed to parse current findings: {err}"))?
    } else {
        let (value, _code) =
            detect_with_ignores(&args.detect.common, args.detect.ignore_file.as_deref())?;
        value
            .get("findings")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|row| serde_json::from_value::<DriftFinding>(row).map_err(|err| err.to_string()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| format!("failed to parse detected findings: {err}"))?
    };

    let baseline_set = baseline_findings
        .iter()
        .map(|row| {
            format!(
                "{:?}|{:?}|{}|{}",
                row.drift_type,
                row.class,
                row.path.as_deref().unwrap_or_default(),
                row.message
            )
        })
        .collect::<BTreeSet<_>>();
    let current_set = current_findings
        .iter()
        .map(|row| {
            format!(
                "{:?}|{:?}|{}|{}",
                row.drift_type,
                row.class,
                row.path.as_deref().unwrap_or_default(),
                row.message
            )
        })
        .collect::<BTreeSet<_>>();

    let mut added = current_set
        .difference(&baseline_set)
        .cloned()
        .collect::<Vec<_>>();
    let mut resolved = baseline_set
        .difference(&current_set)
        .cloned()
        .collect::<Vec<_>>();
    let mut unchanged = baseline_set
        .intersection(&current_set)
        .cloned()
        .collect::<Vec<_>>();
    added.sort();
    resolved.sort();
    unchanged.sort();

    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "drift_compare",
        "status": if added.is_empty() { "ok" } else { "failed" },
        "summary": {
            "added": added.len(),
            "resolved": resolved.len(),
            "unchanged": unchanged.len()
        },
        "added": added,
        "resolved": resolved,
        "unchanged": unchanged
    });
    let rendered = emit_payload(args.detect.common.format, args.detect.common.out, &payload)?;
    Ok((rendered, if payload["status"] == "ok" { 0 } else { 3 }))
}

fn explain(drift_type: String, common: InvariantsCommonArgs) -> Result<(String, i32), String> {
    let explanation = explain_drift_type(&drift_type.to_ascii_lowercase());
    let known = explanation.is_some();
    let payload = if let Some(info) = explanation {
        serde_json::json!({
            "schema_version": 1,
            "kind": "drift_explain",
            "status": "ok",
            "explanation": info
        })
    } else {
        serde_json::json!({
            "schema_version": 1,
            "kind": "drift_explain",
            "status": "failed",
            "error": format!("unknown drift type `{}`", drift_type)
        })
    };
    let rendered = emit_payload(common.format, common.out, &payload)?;
    Ok((rendered, if known { 0 } else { 2 }))
}

pub(crate) fn run_drift_command(
    _quiet: bool,
    command: DriftCommand,
) -> Result<(String, i32), String> {
    match command {
        DriftCommand::Detect(args) => detect(args),
        DriftCommand::Explain(args) => explain(args.drift_type, args.common),
        DriftCommand::Report(args) => report(args),
        DriftCommand::Coverage(args) => coverage(args),
        DriftCommand::Baseline(args) => baseline(args),
        DriftCommand::Compare(args) => compare(args),
    }
}
