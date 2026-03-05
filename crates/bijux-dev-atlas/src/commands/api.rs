// SPDX-License-Identifier: Apache-2.0

use crate::cli::{ApiCommand, ApiCommonArgs, ApiDiffArgs, ApiExplainArgs};
use crate::{emit_payload, resolve_repo_root};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const OPENAPI_GENERATED: &str = "configs/openapi/v1/openapi.generated.json";
const API_SURFACE_REGISTRY: &str = "ops/api/surface-registry.json";
const OPENAPI_VERSION_TRACKING: &str = "ops/api/openapi-version-tracking.json";
const OPENAPI_VALIDATION_CONTRACT: &str =
    "ops/api/contracts/openapi-schema-validation-contract.json";
const OPENAPI_GOLDEN: &str = "ops/api/goldens/openapi-v1.snapshot.json";
const API_DIFF_ARTIFACT: &str = "artifacts/api/openapi-diff-report.json";
const API_COVERAGE_ARTIFACT: &str = "artifacts/api/api-coverage-report.json";
const API_EVIDENCE_BUNDLE: &str = "artifacts/api/api-contract-evidence-bundle.json";
const API_DOC_INDEX: &str = "docs/api/generated/endpoint-index.md";
const API_DOC_TEMPLATES: &str = "docs/api/generated/endpoint-templates.md";

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn write_json(path: &Path, payload: &serde_json::Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(payload)
        .map_err(|err| format!("failed to encode {}: {err}", path.display()))?;
    fs::write(path, format!("{text}\n"))
        .map_err(|err| format!("failed to write {}: {err}", path.display()))
}

fn write_text(path: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    fs::write(path, text).map_err(|err| format!("failed to write {}: {err}", path.display()))
}

fn openapi_paths(spec: &serde_json::Value) -> BTreeSet<String> {
    spec.get("paths")
        .and_then(serde_json::Value::as_object)
        .map(|rows| rows.keys().cloned().collect::<BTreeSet<_>>())
        .unwrap_or_default()
}

fn endpoint_doc_lines(spec: &serde_json::Value) -> Vec<String> {
    let mut lines = vec![
        "# API Endpoint Index".to_string(),
        "".to_string(),
        "| Endpoint | Methods |".to_string(),
        "|---|---|".to_string(),
    ];
    if let Some(paths) = spec.get("paths").and_then(serde_json::Value::as_object) {
        let mut keys = paths.keys().cloned().collect::<Vec<_>>();
        keys.sort();
        for path in keys {
            let methods = paths
                .get(&path)
                .and_then(serde_json::Value::as_object)
                .map(|obj| {
                    let mut verbs = obj.keys().cloned().collect::<Vec<_>>();
                    verbs.sort();
                    verbs.join(",")
                })
                .unwrap_or_else(|| "-".to_string());
            lines.push(format!("| `{}` | `{}` |", path, methods));
        }
    }
    lines
}

fn ensure_api_docs_generated(root: &Path, spec: &serde_json::Value) -> Result<(), String> {
    let index = endpoint_doc_lines(spec).join("\n");
    write_text(&root.join(API_DOC_INDEX), &format!("{}\n", index))?;
    let templates = [
        "# API Endpoint Documentation Templates",
        "",
        "## Endpoint Template",
        "- Path:",
        "- Methods:",
        "- Stability:",
        "- Lifecycle:",
        "- Rate limit class:",
        "- Error codes:",
        "- Security notes:",
    ]
    .join("\n");
    write_text(&root.join(API_DOC_TEMPLATES), &format!("{}\n", templates))
}

fn list_api(common: ApiCommonArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(common.repo_root)?;
    let registry = read_json(&root.join(API_SURFACE_REGISTRY))?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "api_list",
        "status": "ok",
        "registry": registry,
    });
    Ok((emit_payload(common.format, common.out, &payload)?, 0))
}

fn explain_api(args: ApiExplainArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.common.repo_root)?;
    let registry = read_json(&root.join(API_SURFACE_REGISTRY))?;
    let rows = registry
        .get("endpoints")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let item = args
        .endpoint
        .as_deref()
        .and_then(|needle| {
            rows.iter()
                .find(|row| row.get("path").and_then(serde_json::Value::as_str) == Some(needle))
                .cloned()
        })
        .or_else(|| rows.first().cloned());
    let status = if item.is_some() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "api_explain",
        "status": status,
        "endpoint": item,
    });
    let code = if status == "ok" { 0 } else { 2 };
    Ok((
        emit_payload(args.common.format, args.common.out, &payload)?,
        code,
    ))
}

fn diff_api(args: ApiDiffArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.common.repo_root)?;
    let current = read_json(&root.join(OPENAPI_GENERATED))?;
    let baseline_path = args.baseline.unwrap_or_else(|| root.join(OPENAPI_GOLDEN));
    let baseline = read_json(&baseline_path)?;
    let current_paths = openapi_paths(&current);
    let baseline_paths = openapi_paths(&baseline);
    let added = current_paths
        .difference(&baseline_paths)
        .cloned()
        .collect::<Vec<_>>();
    let removed = baseline_paths
        .difference(&current_paths)
        .cloned()
        .collect::<Vec<_>>();
    let report = serde_json::json!({
        "schema_version": 1,
        "kind": "openapi_diff_report",
        "added_paths": added,
        "removed_paths": removed,
    });
    write_json(&root.join(API_DIFF_ARTIFACT), &report)?;
    let status = if report["added_paths"]
        .as_array()
        .is_some_and(|v| v.is_empty())
        && report["removed_paths"]
            .as_array()
            .is_some_and(|v| v.is_empty())
    {
        "ok"
    } else {
        "changed"
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "api_diff",
        "status": status,
        "report": report,
        "artifact": API_DIFF_ARTIFACT,
    });
    Ok((
        emit_payload(args.common.format, args.common.out, &payload)?,
        0,
    ))
}

fn verify_api(common: ApiCommonArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(common.repo_root)?;
    let spec = read_json(&root.join(OPENAPI_GENERATED))?;
    let tracking = read_json(&root.join(OPENAPI_VERSION_TRACKING))?;
    let contract = read_json(&root.join(OPENAPI_VALIDATION_CONTRACT))?;
    let version = spec
        .get("info")
        .and_then(|v| v.get("version"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");
    let expected = tracking
        .get("active_openapi_version")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");
    let endpoints = openapi_paths(&spec);
    let coverage = serde_json::json!({
        "schema_version": 1,
        "kind": "api_coverage_report",
        "endpoint_count": endpoints.len(),
        "paths": endpoints,
    });
    write_json(&root.join(API_COVERAGE_ARTIFACT), &coverage)?;
    let status = if version == expected { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "api_verify",
        "status": status,
        "checks": {
            "openapi_info_version": version,
            "tracked_version": expected,
            "version_matches_tracking": version == expected,
            "validation_contract": contract,
        },
        "artifacts": {
            "coverage": API_COVERAGE_ARTIFACT
        }
    });
    let code = if status == "ok" { 0 } else { 2 };
    Ok((emit_payload(common.format, common.out, &payload)?, code))
}

fn contract_api(common: ApiCommonArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(common.repo_root)?;
    let spec = read_json(&root.join(OPENAPI_GENERATED))?;
    ensure_api_docs_generated(&root, &spec)?;
    let registry = read_json(&root.join(API_SURFACE_REGISTRY))?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "api_contract",
        "status": "ok",
        "artifacts": {
            "evidence_bundle": API_EVIDENCE_BUNDLE,
            "endpoint_index": API_DOC_INDEX,
            "endpoint_templates": API_DOC_TEMPLATES,
        }
    });
    write_json(
        &root.join(API_EVIDENCE_BUNDLE),
        &serde_json::json!({
            "schema_version": 1,
            "kind": "api_contract_evidence_bundle",
            "openapi": OPENAPI_GENERATED,
            "registry": registry,
            "contracts": [
                OPENAPI_VALIDATION_CONTRACT,
                OPENAPI_VERSION_TRACKING,
            ],
            "docs": [
                API_DOC_INDEX,
                API_DOC_TEMPLATES,
            ]
        }),
    )?;
    Ok((emit_payload(common.format, common.out, &payload)?, 0))
}

pub(crate) fn run_api_command(_quiet: bool, command: ApiCommand) -> Result<(String, i32), String> {
    match command {
        ApiCommand::List(args) => list_api(args),
        ApiCommand::Explain(args) => explain_api(args),
        ApiCommand::Diff(args) => diff_api(args),
        ApiCommand::Verify(args) => verify_api(args),
        ApiCommand::Contract(args) => contract_api(args),
    }
}
