// SPDX-License-Identifier: Apache-2.0

use crate::cli::{DriftCommand, InvariantsCommonArgs};
use crate::{emit_payload, resolve_repo_root};
use bijux_dev_atlas::contracts::drift::{
    explain_drift_type, DriftClass, DriftSeverity, DriftType,
};
use bijux_dev_atlas::contracts::system_invariants;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize)]
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

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let text =
        fs::read_to_string(path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn read_yaml(path: &Path) -> Result<serde_yaml::Value, String> {
    let text =
        fs::read_to_string(path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
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
                    format!("configs inventory schema_version drifted: expected 1, found {schema_version}"),
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
    findings.sort_by(|a, b| a.message.cmp(&b.message));
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

fn detect(common: InvariantsCommonArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(common.repo_root)?;
    let findings = detect_all(&root);
    let summary = summarize(&findings);
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "drift_report",
        "status": if summary.findings_total == 0 { "ok" } else { "failed" },
        "summary": summary,
        "findings": findings
    });
    let rendered = emit_payload(common.format, common.out, &payload)?;
    Ok((rendered, if summary.findings_total == 0 { 0 } else { 3 }))
}

fn report(common: InvariantsCommonArgs) -> Result<(String, i32), String> {
    detect(common)
}

fn explain(
    drift_type: String,
    common: InvariantsCommonArgs,
) -> Result<(String, i32), String> {
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
    }
}
