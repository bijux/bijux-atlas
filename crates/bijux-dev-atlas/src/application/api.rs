// SPDX-License-Identifier: Apache-2.0

use crate::cli::{ApiCommand, ApiCommonArgs, ApiDiffArgs, ApiExplainArgs};
use crate::{emit_payload, resolve_repo_root};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const OPENAPI_GENERATED: &str = "configs/sources/runtime/openapi/v1/openapi.generated.json";
const API_SURFACE_REGISTRY: &str = "ops/api/surface-registry.json";
const OPENAPI_VERSION_TRACKING: &str = "ops/api/openapi-version-tracking.json";
const OPENAPI_VALIDATION_CONTRACT: &str =
    "ops/api/contracts/openapi-schema-validation-contract.json";
const OPENAPI_GOLDEN: &str = "ops/api/goldens/openapi-v1.snapshot.json";
const API_DIFF_ARTIFACT: &str = "artifacts/api/openapi-diff-report.json";
const API_COVERAGE_ARTIFACT: &str = "artifacts/api/api-coverage-report.json";
const API_COMPATIBILITY_REPORT_ARTIFACT: &str = "artifacts/api/api-compatibility-report.json";
const API_EVIDENCE_BUNDLE: &str = "artifacts/api/api-contract-evidence-bundle.json";
const API_REGISTRY_SNAPSHOT_ARTIFACT: &str = "artifacts/api/api-contract-registry-snapshot.json";
const API_EXAMPLE_REQUESTS_ARTIFACT: &str = "artifacts/api/api-example-requests.json";
const API_EXAMPLE_RESPONSES_ARTIFACT: &str = "artifacts/api/api-example-responses.json";
const API_EXAMPLE_DATASET_QUERIES_ARTIFACT: &str = "artifacts/api/api-example-dataset-queries.json";
const API_DOC_INDEX: &str = "docs/api/generated/endpoint-index.md";
const API_DOC_TEMPLATES: &str = "docs/api/generated/endpoint-templates.md";
const API_SCHEMA_REFERENCE: &str = "docs/api/generated/api-schema-reference.md";
const API_CONTRACT_DOC: &str = "docs/api/generated/api-contract-documentation.md";
const API_COMPATIBILITY_HARNESS: &str = "ops/api/contracts/api-compatibility-harness.json";

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
    write_text(&root.join(API_DOC_TEMPLATES), &format!("{}\n", templates))?;

    let mut schema_lines = vec![
        "# API Schema Reference".to_string(),
        "".to_string(),
        "| Schema | Kind |".to_string(),
        "|---|---|".to_string(),
    ];
    if let Some(schemas) = spec
        .get("components")
        .and_then(|v| v.get("schemas"))
        .and_then(serde_json::Value::as_object)
    {
        let mut keys = schemas.keys().cloned().collect::<Vec<_>>();
        keys.sort();
        for key in keys {
            schema_lines.push(format!("| `{}` | `component` |", key));
        }
    }
    write_text(
        &root.join(API_SCHEMA_REFERENCE),
        &format!("{}\n", schema_lines.join("\n")),
    )?;

    let contract_lines = [
        "# API Contract Documentation",
        "",
        "- OpenAPI source: `configs/sources/runtime/openapi/v1/openapi.generated.json`",
        "- Surface registry: `ops/api/surface-registry.json`",
        "- Validation contract: `ops/api/contracts/openapi-schema-validation-contract.json`",
        "- Version tracking: `ops/api/openapi-version-tracking.json`",
        "- Compatibility harness: `ops/api/contracts/api-compatibility-harness.json`",
    ]
    .join("\n");
    write_text(
        &root.join(API_CONTRACT_DOC),
        &format!("{}\n", contract_lines),
    )
}

fn generate_examples(root: &Path, spec: &serde_json::Value) -> Result<(), String> {
    let paths = spec
        .get("paths")
        .and_then(serde_json::Value::as_object)
        .cloned()
        .unwrap_or_default();
    let mut request_rows = Vec::new();
    let mut response_rows = Vec::new();
    for (path, operations) in paths {
        if let Some(obj) = operations.as_object() {
            for (method, _) in obj {
                request_rows.push(serde_json::json!({
                    "method": method.to_uppercase(),
                    "path": path,
                    "example": format!("curl -X {} http://127.0.0.1:8080{}", method.to_uppercase(), path),
                }));
                response_rows.push(serde_json::json!({
                    "method": method.to_uppercase(),
                    "path": path,
                    "status": 200,
                    "body_shape": "application/json",
                }));
            }
        }
    }
    let dataset_queries = serde_json::json!({
        "schema_version": 1,
        "kind": "api_example_dataset_queries",
        "queries": [
            "/v1/datasets?release=110&species=homo_sapiens&assembly=GRCh38&limit=5",
            "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=ENSG000001",
            "/v1/genes/ENSG000001/transcripts?release=110&species=homo_sapiens&assembly=GRCh38&limit=20"
        ]
    });
    write_json(
        &root.join(API_EXAMPLE_REQUESTS_ARTIFACT),
        &serde_json::json!({
            "schema_version": 1,
            "kind": "api_example_requests",
            "requests": request_rows
        }),
    )?;
    write_json(
        &root.join(API_EXAMPLE_RESPONSES_ARTIFACT),
        &serde_json::json!({
            "schema_version": 1,
            "kind": "api_example_responses",
            "responses": response_rows
        }),
    )?;
    write_json(
        &root.join(API_EXAMPLE_DATASET_QUERIES_ARTIFACT),
        &dataset_queries,
    )
}

fn compatibility_report(
    root: &Path,
    spec: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let harness = read_json(&root.join(API_COMPATIBILITY_HARNESS))?;
    let tracked = read_json(&root.join(OPENAPI_VERSION_TRACKING))?;
    let current_version = spec
        .get("info")
        .and_then(|v| v.get("version"))
        .and_then(serde_json::Value::as_str);
    let expected = tracked
        .get("active_openapi_version")
        .and_then(serde_json::Value::as_str);
    let report = serde_json::json!({
        "schema_version": 1,
        "kind": "api_compatibility_report",
        "status": if current_version.zip(expected).is_some_and(|(current, wanted)| current == wanted) {
            "ok"
        } else {
            "failed"
        },
        "harness": harness,
        "current_version": current_version,
        "expected_version": expected
    });
    write_json(&root.join(API_COMPATIBILITY_REPORT_ARTIFACT), &report)?;
    Ok(report)
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
        .and_then(serde_json::Value::as_str);
    let expected = tracking
        .get("active_openapi_version")
        .and_then(serde_json::Value::as_str);
    let endpoints = openapi_paths(&spec);
    let coverage = serde_json::json!({
        "schema_version": 1,
        "kind": "api_coverage_report",
        "endpoint_count": endpoints.len(),
        "paths": endpoints,
    });
    write_json(&root.join(API_COVERAGE_ARTIFACT), &coverage)?;
    let compatibility = compatibility_report(&root, &spec)?;
    let status = if version.zip(expected).is_some_and(|(current, wanted)| current == wanted) {
        "ok"
    } else {
        "failed"
    };
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
            "coverage": API_COVERAGE_ARTIFACT,
            "compatibility_report": API_COMPATIBILITY_REPORT_ARTIFACT
        },
        "compatibility": compatibility
    });
    let code = if status == "ok" { 0 } else { 2 };
    Ok((emit_payload(common.format, common.out, &payload)?, code))
}

fn validate_api_contract(common: ApiCommonArgs) -> Result<(String, i32), String> {
    let (rendered, code) = verify_api(common.clone())?;
    let parsed = serde_json::from_str::<serde_json::Value>(&rendered).unwrap_or_else(|_| {
        serde_json::json!({
            "schema_version": 1,
            "kind": "api_validate_contract",
            "status": if code == 0 { "ok" } else { "failed" },
        })
    });
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "api_validate_contract",
        "status": parsed.get("status").and_then(serde_json::Value::as_str).unwrap_or("failed"),
        "verify": parsed
    });
    Ok((emit_payload(common.format, common.out, &payload)?, code))
}

fn contract_registry_snapshot(root: &Path) -> Result<(), String> {
    let registry = read_json(&root.join(API_SURFACE_REGISTRY))?;
    write_json(
        &root.join(API_REGISTRY_SNAPSHOT_ARTIFACT),
        &serde_json::json!({
            "schema_version": 1,
            "kind": "api_contract_registry_snapshot",
            "registry": registry,
        }),
    )
}

fn contract_api(common: ApiCommonArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(common.repo_root)?;
    let spec = read_json(&root.join(OPENAPI_GENERATED))?;
    ensure_api_docs_generated(&root, &spec)?;
    generate_examples(&root, &spec)?;
    compatibility_report(&root, &spec)?;
    contract_registry_snapshot(&root)?;
    let registry = read_json(&root.join(API_SURFACE_REGISTRY))?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "api_contract",
        "status": "ok",
        "artifacts": {
            "evidence_bundle": API_EVIDENCE_BUNDLE,
            "endpoint_index": API_DOC_INDEX,
            "endpoint_templates": API_DOC_TEMPLATES,
            "schema_reference": API_SCHEMA_REFERENCE,
            "contract_documentation": API_CONTRACT_DOC,
            "registry_snapshot": API_REGISTRY_SNAPSHOT_ARTIFACT,
            "example_requests": API_EXAMPLE_REQUESTS_ARTIFACT,
            "example_responses": API_EXAMPLE_RESPONSES_ARTIFACT,
            "example_dataset_queries": API_EXAMPLE_DATASET_QUERIES_ARTIFACT,
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
                API_COMPATIBILITY_HARNESS,
            ],
            "docs": [
                API_DOC_INDEX,
                API_DOC_TEMPLATES,
                API_SCHEMA_REFERENCE,
                API_CONTRACT_DOC,
            ],
            "examples": [
                API_EXAMPLE_REQUESTS_ARTIFACT,
                API_EXAMPLE_RESPONSES_ARTIFACT,
                API_EXAMPLE_DATASET_QUERIES_ARTIFACT,
            ],
            "snapshots": [
                OPENAPI_GOLDEN,
                API_REGISTRY_SNAPSHOT_ARTIFACT,
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
        ApiCommand::Validate(args) => validate_api_contract(args),
        ApiCommand::Contract(args) => contract_api(args),
    }
}
