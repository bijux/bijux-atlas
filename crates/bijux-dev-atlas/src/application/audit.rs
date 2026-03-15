// SPDX-License-Identifier: Apache-2.0

use crate::cli::{AuditBundleCommand, AuditCommand, AuditComplianceCommand, AuditReadinessCommand};
use crate::{emit_payload, resolve_repo_root};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

fn checklist_path(root: &Path) -> PathBuf {
    root.join("configs/sources/governance/audit/audit-artifact-checklist.json")
}

fn schema_path(root: &Path) -> PathBuf {
    root.join("configs/sources/governance/audit/audit-bundle.schema.json")
}

fn bundle_path(root: &Path) -> PathBuf {
    root.join("artifacts/audit/bundle.json")
}

fn bundle_summary_path(root: &Path) -> PathBuf {
    root.join("artifacts/audit/bundle-summary.json")
}

fn bundle_hash_path(root: &Path) -> PathBuf {
    root.join("artifacts/audit/bundle.sha256")
}

fn compliance_matrix_path(root: &Path) -> PathBuf {
    root.join("configs/sources/governance/audit/compliance-matrix-template.json")
}

fn compliance_report_path(root: &Path) -> PathBuf {
    root.join("artifacts/audit/compliance-coverage.json")
}

fn readiness_report_path(root: &Path) -> PathBuf {
    root.join("artifacts/audit/readiness-report.json")
}

fn audit_run_path(root: &Path) -> PathBuf {
    root.join("artifacts/audit/run-report.json")
}

fn audit_event_schema_path(root: &Path) -> PathBuf {
    root.join("ops/audit/event.schema.json")
}

fn audit_report_schema_path(root: &Path) -> PathBuf {
    root.join("ops/audit/report.schema.json")
}

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    serde_json::from_str(
        &fs::read_to_string(path)
            .map_err(|err| format!("read {} failed: {err}", path.display()))?,
    )
    .map_err(|err| format!("parse {} failed: {err}", path.display()))
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("create {} failed: {err}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(value)
        .map_err(|err| format!("encode {} failed: {err}", path.display()))?;
    fs::write(path, text).map_err(|err| format!("write {} failed: {err}", path.display()))
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let data = fs::read(path).map_err(|err| format!("read {} failed: {err}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(data)))
}

fn sha256_json(value: &serde_json::Value) -> Result<String, String> {
    let bytes = serde_json::to_vec(value)
        .map_err(|err| format!("encode audit bundle hash failed: {err}"))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn bundle_generate(
    repo_root: Option<PathBuf>,
    format: crate::cli::FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let checklist = read_json(&checklist_path(&root))?;
    let required = checklist
        .get("required_artifacts")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut artifacts = Vec::new();
    let mut missing = Vec::new();
    for row in required {
        let Some(id) = row.get("id").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let Some(path) = row.get("path").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let abs = root.join(path);
        if !abs.exists() {
            missing.push(serde_json::json!({"id": id, "path": path}));
            continue;
        }
        let digest = sha256_file(&abs)?;
        artifacts.push(serde_json::json!({"id": id, "path": path, "sha256": digest}));
    }

    let bundle = serde_json::json!({
        "schema_version": 1,
        "kind": "audit_bundle",
        "status": if missing.is_empty() {"ok"} else {"failed"},
        "artifacts": artifacts,
        "missing": missing,
        "checklist": checklist_path(&root).strip_prefix(&root).unwrap_or(&checklist_path(&root)).display().to_string(),
        "metadata": {
            "generator": "bijux-dev-atlas audit bundle generate",
            "control_plane_version": env!("CARGO_PKG_VERSION"),
        },
    });
    let hash = sha256_json(&bundle)?;
    let summary = serde_json::json!({
        "schema_version": 1,
        "kind": "audit_bundle_summary",
        "status": bundle["status"],
        "artifact_count": bundle["artifacts"].as_array().map_or(0, |rows| rows.len()),
        "missing_count": bundle["missing"].as_array().map_or(0, |rows| rows.len()),
        "bundle_hash_sha256": hash,
        "bundle_path": bundle_path(&root).strip_prefix(&root).unwrap_or(&bundle_path(&root)).display().to_string(),
    });

    write_json(&bundle_path(&root), &bundle)?;
    write_json(&bundle_summary_path(&root), &summary)?;
    fs::write(bundle_hash_path(&root), format!("{hash}\n"))
        .map_err(|err| format!("write {} failed: {err}", bundle_hash_path(&root).display()))?;
    let rendered = emit_payload(format, out, &bundle)?;
    let code = if bundle["status"] == "ok" { 0 } else { 1 };
    Ok((rendered, code))
}

fn bundle_validate(
    repo_root: Option<PathBuf>,
    format: crate::cli::FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let schema = read_json(&schema_path(&root))?;
    let required = schema
        .get("required")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let bundle = read_json(&bundle_path(&root))?;
    let Some(bundle_obj) = bundle.as_object() else {
        return Err("audit bundle must be an object".to_string());
    };
    let mut errors = Vec::new();
    for key in required.iter().filter_map(serde_json::Value::as_str) {
        if !bundle_obj.contains_key(key) {
            errors.push(format!("audit bundle missing required key `{key}`"));
        }
    }
    if bundle["missing"]
        .as_array()
        .is_some_and(|rows| !rows.is_empty())
    {
        errors.push("audit bundle must contain all required artifacts".to_string());
    }
    let computed_hash = sha256_json(&bundle)?;
    let stored_hash = fs::read_to_string(bundle_hash_path(&root))
        .ok()
        .map(|text| text.trim().to_string())
        .unwrap_or_default();
    if !stored_hash.is_empty() && stored_hash != computed_hash {
        errors.push("audit bundle hash does not match bundle content".to_string());
    }
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "audit_bundle_validate",
        "status": if errors.is_empty() {"ok"} else {"failed"},
        "bundle": bundle_path(&root).strip_prefix(&root).unwrap_or(&bundle_path(&root)).display().to_string(),
        "bundle_summary": bundle_summary_path(&root).strip_prefix(&root).unwrap_or(&bundle_summary_path(&root)).display().to_string(),
        "bundle_hash": bundle_hash_path(&root).strip_prefix(&root).unwrap_or(&bundle_hash_path(&root)).display().to_string(),
        "errors": errors,
    });
    let rendered = emit_payload(format, out, &payload)?;
    let code = if payload["status"] == "ok" { 0 } else { 1 };
    Ok((rendered, code))
}

fn compliance_report(
    repo_root: Option<PathBuf>,
    format: crate::cli::FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let matrix = read_json(&compliance_matrix_path(&root))?;
    let controls = matrix
        .get("controls")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut uncovered = Vec::new();
    let mut rows = Vec::new();
    for row in controls {
        let id = row
            .get("id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let contracts = row
            .get("contracts")
            .and_then(serde_json::Value::as_array)
            .map_or(0, |v| v.len());
        let checks = row
            .get("checks")
            .and_then(serde_json::Value::as_array)
            .map_or(0, |v| v.len());
        let lanes = row
            .get("lanes")
            .and_then(serde_json::Value::as_array)
            .map_or(0, |v| v.len());
        let covered = contracts + checks + lanes > 0;
        if !covered {
            uncovered.push(id.to_string());
        }
        if row.get("critical").and_then(serde_json::Value::as_bool) == Some(true) && !covered {
            uncovered.push(format!("critical:{id}"));
        }
        rows.push(serde_json::json!({
            "id": id,
            "critical": row.get("critical").and_then(serde_json::Value::as_bool).unwrap_or(false),
            "covered": covered,
            "contracts": contracts,
            "checks": checks,
            "lanes": lanes
        }));
    }
    let report = serde_json::json!({
        "schema_version": 1,
        "kind": "audit_compliance_coverage",
        "status": if uncovered.is_empty() {"ok"} else {"failed"},
        "matrix": compliance_matrix_path(&root).strip_prefix(&root).unwrap_or(&compliance_matrix_path(&root)).display().to_string(),
        "rows": rows,
        "uncovered": uncovered,
    });
    write_json(&compliance_report_path(&root), &report)?;
    let rendered = emit_payload(format, out, &report)?;
    let code = if report["status"] == "ok" { 0 } else { 1 };
    Ok((rendered, code))
}

fn readiness_validate(
    repo_root: Option<PathBuf>,
    format: crate::cli::FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let bundle = read_json(&bundle_path(&root))?;
    if !compliance_report_path(&root).exists() {
        let _ = compliance_report(Some(root.clone()), crate::cli::FormatArg::Json, None)?;
    }
    let compliance = read_json(&compliance_report_path(&root))?;
    let mut errors = Vec::new();
    if bundle["status"] != "ok" {
        errors.push("audit bundle must be ok".to_string());
    }
    if compliance["status"] != "ok" {
        errors.push("compliance coverage must be ok".to_string());
    }
    for path in [
        "docs/operations/audit-procedure.md",
        "docs/operations/institutional-support-policy.md",
        "docs/operations/long-term-support-policy.md",
        "docs/operations/backward-compatibility-guarantee.md",
        "docs/operations/deprecation-lifecycle-policy.md",
        "docs/operations/security-disclosure-policy.md",
        "docs/operations/upgrade-compatibility-guide.md",
        "docs/operations/release-support-window-policy.md",
        "docs/operations/maintenance-policy.md",
        "docs/operations/final-readiness-checklist.md",
    ] {
        if !root.join(path).exists() {
            errors.push(format!("missing required readiness document `{path}`"));
        }
    }
    let report = serde_json::json!({
        "schema_version": 1,
        "kind": "audit_readiness_validate",
        "status": if errors.is_empty() {"ok"} else {"failed"},
        "bundle": bundle_path(&root).strip_prefix(&root).unwrap_or(&bundle_path(&root)).display().to_string(),
        "compliance": compliance_report_path(&root).strip_prefix(&root).unwrap_or(&compliance_report_path(&root)).display().to_string(),
        "errors": errors,
    });
    write_json(&readiness_report_path(&root), &report)?;
    let rendered = emit_payload(format, out, &report)?;
    let code = if report["status"] == "ok" { 0 } else { 1 };
    Ok((rendered, code))
}

fn run_audit_checks(root: &Path) -> serde_json::Value {
    let mut checks = Vec::new();

    // configuration integrity
    let config_path = root.join("configs/registry/inventory/index.json");
    let config_status = match read_json(&config_path) {
        Ok(v) if v.get("schema_version").and_then(serde_json::Value::as_i64) == Some(1) => "ok",
        _ => "failed",
    };
    checks.push(serde_json::json!({
        "id": "AUDIT-CONFIG-INTEGRITY-001",
        "title": "configuration integrity",
        "classification": "configuration",
        "severity": "high",
        "status": config_status,
        "path": "configs/registry/inventory/index.json"
    }));

    // artifact integrity
    let artifact_manifest = root.join("ops/release/evidence/manifest.json");
    let artifact_status = match read_json(&artifact_manifest) {
        Ok(v) if v.get("schema_version").is_some() => "ok",
        _ => "failed",
    };
    checks.push(serde_json::json!({
        "id": "AUDIT-ARTIFACT-INTEGRITY-001",
        "title": "artifact integrity",
        "classification": "artifact",
        "severity": "high",
        "status": artifact_status,
        "path": "ops/release/evidence/manifest.json"
    }));

    // registry consistency
    let registry_path = root.join("ops/invariants/registry.json");
    let registry_status = match read_json(&registry_path) {
        Ok(v) => {
            if v.get("invariants")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|rows| !rows.is_empty())
            {
                "ok"
            } else {
                "failed"
            }
        }
        _ => "failed",
    };
    checks.push(serde_json::json!({
        "id": "AUDIT-REGISTRY-CONSISTENCY-001",
        "title": "registry consistency",
        "classification": "registry",
        "severity": "high",
        "status": registry_status,
        "path": "ops/invariants/registry.json"
    }));

    // runtime configuration state
    let runtime_path = root.join("ops/k8s/values/offline.yaml");
    let runtime_status = match fs::read_to_string(&runtime_path) {
        Ok(text) if serde_yaml::from_str::<serde_yaml::Value>(&text).is_ok() => "ok",
        _ => "failed",
    };
    checks.push(serde_json::json!({
        "id": "AUDIT-RUNTIME-CONFIG-STATE-001",
        "title": "runtime configuration state",
        "classification": "runtime",
        "severity": "medium",
        "status": runtime_status,
        "path": "ops/k8s/values/offline.yaml"
    }));

    // ops deployment integrity
    let deploy_path = root.join("ops/k8s/charts/bijux-atlas/Chart.yaml");
    let deploy_status = match fs::read_to_string(&deploy_path) {
        Ok(text) if serde_yaml::from_str::<serde_yaml::Value>(&text).is_ok() => "ok",
        _ => "failed",
    };
    checks.push(serde_json::json!({
        "id": "AUDIT-OPS-DEPLOY-INTEGRITY-001",
        "title": "ops deployment integrity",
        "classification": "ops",
        "severity": "medium",
        "status": deploy_status,
        "path": "ops/k8s/charts/bijux-atlas/Chart.yaml"
    }));

    let failed_count = checks
        .iter()
        .filter(|row| row.get("status").and_then(serde_json::Value::as_str) == Some("failed"))
        .count();
    let failure_classification = checks
        .iter()
        .filter(|row| row.get("status").and_then(serde_json::Value::as_str) == Some("failed"))
        .map(|row| {
            serde_json::json!({
                "id": row.get("id").cloned().unwrap_or(serde_json::Value::Null),
                "title": row.get("title").cloned().unwrap_or(serde_json::Value::Null),
                "classification": row.get("classification").cloned().unwrap_or(serde_json::Value::Null),
                "severity": row.get("severity").cloned().unwrap_or(serde_json::Value::Null),
            })
        })
        .collect::<Vec<_>>();
    let status = if failed_count == 0 { "ok" } else { "failed" };
    serde_json::json!({
        "schema_version": 1,
        "kind": "audit_run",
        "status": status,
        "checks": checks,
        "failure_classification": failure_classification,
        "metrics": {
            "total_checks": 5,
            "failed_checks": failed_count,
            "passed_checks": 5 - failed_count
        }
    })
}

fn audit_run(common: crate::cli::AuditBundleArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(common.repo_root)?;
    let payload = run_audit_checks(&root);
    write_json(&audit_run_path(&root), &payload)?;
    let rendered = emit_payload(common.format, common.out, &payload)?;
    let code = if payload["status"] == "ok" { 0 } else { 1 };
    Ok((rendered, code))
}

fn audit_report(common: crate::cli::AuditBundleArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(common.repo_root)?;
    let payload = if audit_run_path(&root).exists() {
        read_json(&audit_run_path(&root))?
    } else {
        let generated = run_audit_checks(&root);
        write_json(&audit_run_path(&root), &generated)?;
        generated
    };
    let report = serde_json::json!({
        "schema_version": 1,
        "kind": "audit_report",
        "status": payload["status"],
        "source": audit_run_path(&root).strip_prefix(&root).unwrap_or(&audit_run_path(&root)).display().to_string(),
        "report": payload
    });
    let rendered = emit_payload(common.format, common.out, &report)?;
    let code = if report["status"] == "ok" { 0 } else { 1 };
    Ok((rendered, code))
}

fn audit_explain(common: crate::cli::AuditBundleArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(common.repo_root)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "audit_explain",
        "status": "ok",
        "schemas": {
            "event": audit_event_schema_path(&root).strip_prefix(&root).unwrap_or(&audit_event_schema_path(&root)).display().to_string(),
            "report": audit_report_schema_path(&root).strip_prefix(&root).unwrap_or(&audit_report_schema_path(&root)).display().to_string()
        },
        "checks": [
            "configuration integrity",
            "artifact integrity",
            "registry consistency",
            "runtime configuration state",
            "ops deployment integrity"
        ],
        "commands": [
            "bijux-dev-atlas audit run --format json",
            "bijux-dev-atlas audit report --format json",
            "bijux-dev-atlas audit explain --format json"
        ]
    });
    let rendered = emit_payload(common.format, common.out, &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_audit_command(
    _quiet: bool,
    command: AuditCommand,
) -> Result<(String, i32), String> {
    match command {
        AuditCommand::Run(args) => audit_run(args),
        AuditCommand::Report(args) => audit_report(args),
        AuditCommand::Explain(args) => audit_explain(args),
        AuditCommand::Bundle { command } => match command {
            AuditBundleCommand::Generate(args) => {
                bundle_generate(args.repo_root, args.format, args.out)
            }
            AuditBundleCommand::Validate(args) => {
                bundle_validate(args.repo_root, args.format, args.out)
            }
        },
        AuditCommand::Compliance { command } => match command {
            AuditComplianceCommand::Report(args) => {
                compliance_report(args.repo_root, args.format, args.out)
            }
        },
        AuditCommand::Readiness { command } => match command {
            AuditReadinessCommand::Validate(args) => {
                readiness_validate(args.repo_root, args.format, args.out)
            }
        },
    }
}
