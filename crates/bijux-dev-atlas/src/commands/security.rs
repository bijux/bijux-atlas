// SPDX-License-Identifier: Apache-2.0

use crate::cli::{SecurityCommand, SecurityScanArtifactsArgs, SecurityValidateArgs};
use crate::{emit_payload, resolve_repo_root};
use std::fs;
use std::path::{Path, PathBuf};

fn read_yaml(path: &Path) -> Result<serde_yaml::Value, String> {
    serde_yaml::from_str(
        &fs::read_to_string(path).map_err(|err| format!("failed to read {}: {err}", path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn report_path(root: &Path) -> Result<PathBuf, String> {
    let path = root.join("artifacts/security/security-threat-model.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    Ok(path)
}

fn named_report_path(root: &Path, name: &str) -> Result<PathBuf, String> {
    let path = root.join("artifacts/security").join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    Ok(path)
}

fn ensure_json(path: &Path) -> Result<(), String> {
    let _: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(path).map_err(|err| format!("failed to read {}: {err}", path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
    Ok(())
}

pub(crate) fn run_security_command(
    _quiet: bool,
    command: SecurityCommand,
) -> Result<(String, i32), String> {
    match command {
        SecurityCommand::Validate(args) => run_security_validate(args),
        SecurityCommand::Compliance { command } => match command {
            crate::cli::SecurityComplianceCommand::Validate(args) => run_security_compliance_validate(args),
        },
        SecurityCommand::ScanArtifacts(args) => run_security_scan_artifacts(args),
    }
}

fn list_files_recursive(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(path) = stack.pop() {
        if !path.exists() {
            continue;
        }
        for entry in fs::read_dir(&path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?
        {
            let entry = entry.map_err(|err| format!("failed to read directory entry: {err}"))?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                stack.push(entry_path);
            } else {
                files.push(entry_path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn scan_matches(
    root: &Path,
    dir: &Path,
    patterns: &[String],
) -> Result<Vec<serde_json::Value>, String> {
    let mut matches = Vec::new();
    for path in list_files_recursive(dir)? {
        let content = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(_) => continue,
        };
        for pattern in patterns {
            if content.contains(pattern) {
                matches.push(serde_json::json!({
                    "path": path.strip_prefix(root).unwrap_or(&path).display().to_string(),
                    "pattern": pattern
                }));
            }
        }
    }
    Ok(matches)
}

fn run_security_validate(args: SecurityValidateArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let assets_path = root.join("security/threat-model/assets.yaml");
    let threats_path = root.join("security/threat-model/threats.yaml");
    let mitigations_path = root.join("security/threat-model/mitigations.yaml");
    let controls_path = root.join("security/compliance/controls.yaml");
    let asset_schema_path = root.join("configs/contracts/security/assets.schema.json");
    let threats_schema_path = root.join("configs/contracts/security/threats.schema.json");
    let mitigations_schema_path = root.join("configs/contracts/security/mitigations.schema.json");
    let controls_schema_path = root.join("configs/contracts/security/controls.schema.json");
    let secrets_schema_path = root.join("configs/contracts/security/secrets.schema.json");
    let redaction_schema_path = root.join("configs/contracts/security/redaction.schema.json");
    let forbidden_patterns_schema_path = root.join("configs/contracts/security/forbidden-patterns.schema.json");
    let secrets_path = root.join("configs/security/secrets.json");
    let redaction_path = root.join("configs/security/redaction.json");
    let forbidden_patterns_path = root.join("configs/security/forbidden-patterns.json");

    ensure_json(&asset_schema_path)?;
    ensure_json(&threats_schema_path)?;
    ensure_json(&mitigations_schema_path)?;
    ensure_json(&controls_schema_path)?;
    ensure_json(&secrets_schema_path)?;
    ensure_json(&redaction_schema_path)?;
    ensure_json(&forbidden_patterns_schema_path)?;

    let assets = read_yaml(&assets_path)?;
    let threats = read_yaml(&threats_path)?;
    let mitigations = read_yaml(&mitigations_path)?;
    let controls = read_yaml(&controls_path)?;
    let secrets: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&secrets_path).map_err(|err| format!("failed to read {}: {err}", secrets_path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", secrets_path.display()))?;
    let redaction: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&redaction_path).map_err(|err| format!("failed to read {}: {err}", redaction_path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", redaction_path.display()))?;
    let forbidden_patterns: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&forbidden_patterns_path)
            .map_err(|err| format!("failed to read {}: {err}", forbidden_patterns_path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", forbidden_patterns_path.display()))?;

    let asset_rows = assets
        .get("assets")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let threat_rows = threats
        .get("threats")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let mitigation_rows = mitigations
        .get("mitigations")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let control_rows = controls
        .get("controls")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();

    let mitigation_ids = mitigation_rows
        .iter()
        .filter_map(|row| row.get("id").and_then(serde_yaml::Value::as_str))
        .collect::<std::collections::BTreeSet<_>>();
    let mut missing_mitigations = Vec::new();
    let mut missing_control_or_reason = Vec::new();
    let mut high_severity_gaps = Vec::new();
    for row in &threat_rows {
        let id = row.get("id").and_then(serde_yaml::Value::as_str).unwrap_or_default();
        let severity = row.get("severity").and_then(serde_yaml::Value::as_str).unwrap_or_default();
        let mapped = row
            .get("mitigations")
            .and_then(serde_yaml::Value::as_sequence)
            .cloned()
            .unwrap_or_default();
        if mapped.is_empty() {
            missing_mitigations.push(id.to_string());
        }
        let mapped_ids = mapped
            .iter()
            .filter_map(serde_yaml::Value::as_str)
            .collect::<Vec<_>>();
        if mapped_ids.iter().any(|name| !mitigation_ids.contains(name)) {
            missing_mitigations.push(id.to_string());
        }
        if severity == "high" {
            let has_executable_or_runbook = mitigation_rows.iter().any(|mitigation| {
                let mitigation_id = mitigation.get("id").and_then(serde_yaml::Value::as_str).unwrap_or_default();
                mapped_ids.contains(&mitigation_id)
                    && (mitigation.get("control_check_id").and_then(serde_yaml::Value::as_str).is_some()
                        || mitigation.get("runbook_page").and_then(serde_yaml::Value::as_str).is_some())
            });
            if !has_executable_or_runbook {
                high_severity_gaps.push(id.to_string());
            }
        }
    }

    let mut missing_docs_links = Vec::new();
    for row in &mitigation_rows {
        let id = row.get("id").and_then(serde_yaml::Value::as_str).unwrap_or_default();
        let has_control = row.get("control_check_id").and_then(serde_yaml::Value::as_str).is_some();
        let has_reason = row.get("documented_reason").and_then(serde_yaml::Value::as_str).is_some();
        if !has_control && !has_reason {
            missing_control_or_reason.push(id.to_string());
        }
        let docs_page = row.get("docs_page").and_then(serde_yaml::Value::as_str).unwrap_or_default();
        if docs_page.is_empty() || !root.join(docs_page).exists() {
            missing_docs_links.push(id.to_string());
        }
    }

    let mut shape_errors = Vec::new();
    if asset_rows.is_empty() {
        shape_errors.push("assets.yaml must define at least one asset".to_string());
    }
    for row in &asset_rows {
        if row.get("id").and_then(serde_yaml::Value::as_str).is_none()
            || row.get("type").and_then(serde_yaml::Value::as_str).is_none()
            || row.get("description").and_then(serde_yaml::Value::as_str).is_none()
            || row.get("sensitivity").and_then(serde_yaml::Value::as_str).is_none()
            || row.get("owner").and_then(serde_yaml::Value::as_str).is_none()
        {
            shape_errors.push("assets.yaml contains an asset missing required fields".to_string());
            break;
        }
    }
    if threat_rows.is_empty() {
        shape_errors.push("threats.yaml must define at least one threat".to_string());
    }
    for row in &threat_rows {
        if row.get("id").and_then(serde_yaml::Value::as_str).is_none()
            || row.get("category").and_then(serde_yaml::Value::as_str).is_none()
            || row.get("title").and_then(serde_yaml::Value::as_str).is_none()
            || row.get("severity").and_then(serde_yaml::Value::as_str).is_none()
            || row.get("likelihood").and_then(serde_yaml::Value::as_str).is_none()
            || row.get("affected_component").and_then(serde_yaml::Value::as_str).is_none()
            || row.get("residual_risk").and_then(serde_yaml::Value::as_str).is_none()
        {
            shape_errors.push("threats.yaml contains a threat missing required fields".to_string());
            break;
        }
    }
    if mitigation_rows.is_empty() {
        shape_errors.push("mitigations.yaml must define at least one mitigation".to_string());
    }
    for row in &mitigation_rows {
        if row.get("id").and_then(serde_yaml::Value::as_str).is_none()
            || row.get("title").and_then(serde_yaml::Value::as_str).is_none()
            || row.get("docs_page").and_then(serde_yaml::Value::as_str).is_none()
        {
            shape_errors.push("mitigations.yaml contains a mitigation missing required fields".to_string());
            break;
        }
    }
    if control_rows.is_empty() {
        shape_errors.push("controls.yaml must define at least one control".to_string());
    }

    let sec_threat_001 = shape_errors.is_empty();
    let sec_threat_002 = missing_mitigations.is_empty();
    let sec_threat_003 = missing_control_or_reason.is_empty() && missing_docs_links.is_empty();
    let sec_threat_004 = high_severity_gaps.is_empty();
    let declared_secret_keys = secrets
        .get("secrets")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("key").and_then(serde_json::Value::as_str))
        .collect::<std::collections::BTreeSet<_>>();
    let redaction_keys = redaction
        .get("rules")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("key").and_then(serde_json::Value::as_str))
        .collect::<std::collections::BTreeSet<_>>();
    let missing_redaction_keys = declared_secret_keys
        .iter()
        .filter(|key| !redaction_keys.contains(**key))
        .map(|key| (*key).to_string())
        .collect::<Vec<_>>();
    let default_scan_dir = root.join("release/evidence");
    let forbidden_literals = forbidden_patterns
        .get("patterns")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("literal").and_then(serde_json::Value::as_str))
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let evidence_matches = scan_matches(&root, &default_scan_dir, &forbidden_literals)?;
    let sec_red_001 = missing_redaction_keys.is_empty();
    let sec_red_002 = evidence_matches.is_empty();

    let payload = serde_json::json!({
        "schema_version": 1,
        "status": if sec_threat_001 && sec_threat_002 && sec_threat_003 && sec_threat_004 && sec_red_001 && sec_red_002 { "ok" } else { "failed" },
        "counts": {
            "assets": asset_rows.len(),
            "threats": threat_rows.len(),
            "mitigations": mitigation_rows.len(),
            "controls": control_rows.len(),
            "declared_secrets": declared_secret_keys.len()
        },
        "contracts": {
            "SEC-THREAT-001": sec_threat_001,
            "SEC-THREAT-002": sec_threat_002,
            "SEC-THREAT-003": sec_threat_003,
            "SEC-THREAT-004": sec_threat_004,
            "SEC-RED-001": sec_red_001,
            "SEC-RED-002": sec_red_002
        },
        "gaps": {
            "shape_errors": shape_errors,
            "missing_mitigations": missing_mitigations,
            "missing_control_or_reason": missing_control_or_reason,
            "missing_docs_links": missing_docs_links,
            "high_severity_gaps": high_severity_gaps,
            "missing_redaction_keys": missing_redaction_keys,
            "evidence_secret_matches": evidence_matches
        }
    });

    let path = report_path(&root)?;
    fs::write(
        &path,
        serde_json::to_string_pretty(&payload).map_err(|err| format!("encode security report failed: {err}"))?,
    )
    .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
    let rendered = emit_payload(
        args.format,
        args.out,
        &serde_json::json!({
            "schema_version": 1,
            "status": payload["status"].clone(),
            "text": if payload["status"] == "ok" { "security threat model validated" } else { "security threat model validation failed" },
            "rows": [{
                "report_path": path.strip_prefix(&root).unwrap_or(&path).display().to_string(),
                "contracts": payload["contracts"].clone(),
                "gaps": payload["gaps"].clone()
            }],
            "summary": {"total": 1, "errors": if payload["status"] == "ok" { 0 } else { 1 }, "warnings": 0}
        }),
    )?;
    Ok((rendered, if payload["status"] == "ok" { 0 } else { 1 }))
}

fn run_security_compliance_validate(args: SecurityValidateArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let controls_schema_path = root.join("configs/contracts/security/controls.schema.json");
    let matrix_schema_path = root.join("configs/contracts/security/compliance-matrix.schema.json");
    ensure_json(&controls_schema_path)?;
    ensure_json(&matrix_schema_path)?;
    let controls = read_yaml(&root.join("security/compliance/controls.yaml"))?;
    let matrix = read_yaml(&root.join("security/compliance/matrix.yaml"))?;
    let control_rows = controls
        .get("controls")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let matrix_rows = matrix
        .get("mappings")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let mut evidence_gaps = Vec::new();
    let mut missing_files = Vec::new();
    for control in &control_rows {
        let id = control.get("id").and_then(serde_yaml::Value::as_str).unwrap_or_default();
        let mapping = matrix_rows.iter().find(|row| {
            row.get("control_id").and_then(serde_yaml::Value::as_str) == Some(id)
        });
        let Some(mapping) = mapping else {
            evidence_gaps.push(id.to_string());
            continue;
        };
        let evidence = mapping
            .get("evidence")
            .and_then(serde_yaml::Value::as_sequence)
            .cloned()
            .unwrap_or_default();
        if evidence.is_empty() {
            evidence_gaps.push(id.to_string());
        }
        for item in evidence {
            if let Some(path) = item.as_str() {
                if !root.join(path).exists() {
                    missing_files.push(format!("{id}:{path}"));
                }
            }
        }
    }
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": if evidence_gaps.is_empty() && missing_files.is_empty() { "ok" } else { "failed" },
        "contracts": {
            "SEC-COMP-001": true,
            "SEC-COMP-002": evidence_gaps.is_empty(),
            "SEC-COMP-003": missing_files.is_empty()
        },
        "gaps": {
            "controls_without_evidence": evidence_gaps,
            "missing_evidence_files": missing_files
        },
        "counts": {
            "controls": control_rows.len(),
            "mappings": matrix_rows.len()
        }
    });
    let path = named_report_path(&root, "security-compliance.json")?;
    fs::write(
        &path,
        serde_json::to_string_pretty(&payload).map_err(|err| format!("encode security compliance report failed: {err}"))?,
    )
    .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
    let rendered = emit_payload(
        args.format,
        args.out,
        &serde_json::json!({
            "schema_version": 1,
            "status": payload["status"].clone(),
            "text": if payload["status"] == "ok" { "security compliance validated" } else { "security compliance validation failed" },
            "rows": [{
                "report_path": path.strip_prefix(&root).unwrap_or(&path).display().to_string(),
                "contracts": payload["contracts"].clone(),
                "gaps": payload["gaps"].clone()
            }],
            "summary": {"total": 1, "errors": if payload["status"] == "ok" { 0 } else { 1 }, "warnings": 0}
        }),
    )?;
    Ok((rendered, if payload["status"] == "ok" { 0 } else { 1 }))
}

fn run_security_scan_artifacts(args: SecurityScanArtifactsArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let schema_path = root.join("configs/contracts/security/forbidden-patterns.schema.json");
    let policy_path = root.join("configs/security/forbidden-patterns.json");
    ensure_json(&schema_path)?;
    let policy: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&policy_path).map_err(|err| format!("failed to read {}: {err}", policy_path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", policy_path.display()))?;
    let patterns = policy
        .get("patterns")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("literal").and_then(serde_json::Value::as_str))
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let dir = if args.dir.is_absolute() {
        args.dir
    } else {
        root.join(args.dir)
    };
    let matches = scan_matches(&root, &dir, &patterns)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": if matches.is_empty() { "ok" } else { "failed" },
        "contracts": {
            "SEC-ART-001": matches.is_empty()
        },
        "scan_root": dir.strip_prefix(&root).unwrap_or(&dir).display().to_string(),
        "matches": matches
    });
    let path = named_report_path(&root, "security-artifact-scan.json")?;
    fs::write(
        &path,
        serde_json::to_string_pretty(&payload).map_err(|err| format!("encode artifact scan report failed: {err}"))?,
    )
    .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
    let rendered = emit_payload(
        args.format,
        args.out,
        &serde_json::json!({
            "schema_version": 1,
            "status": payload["status"].clone(),
            "text": if payload["status"] == "ok" { "artifact scan passed" } else { "artifact scan found forbidden patterns" },
            "rows": [{
                "report_path": path.strip_prefix(&root).unwrap_or(&path).display().to_string(),
                "scan_root": payload["scan_root"].clone(),
                "matches": payload["matches"].clone()
            }],
            "summary": {"total": 1, "errors": if payload["status"] == "ok" { 0 } else { 1 }, "warnings": 0}
        }),
    )?;
    Ok((rendered, if payload["status"] == "ok" { 0 } else { 1 }))
}
