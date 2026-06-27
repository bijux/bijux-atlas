// SPDX-License-Identifier: Apache-2.0

use super::*;

fn diagnose_root(repo_root: &std::path::Path) -> Result<std::path::PathBuf, String> {
    let path = repo_root.join("artifacts/ops/diagnose");
    std::fs::create_dir_all(&path)
        .map_err(|err| format!("failed to create {}: {err}", path.display()))?;
    Ok(path)
}

fn collect_scenario_files(repo_root: &std::path::Path, scenario: Option<&str>) -> Vec<String> {
    let base = repo_root.join("artifacts/ops/scenarios");
    let mut rows = Vec::new();
    if !base.exists() {
        return rows;
    }
    let Ok(entries) = std::fs::read_dir(&base) else {
        return rows;
    };
    for scenario_entry in entries.flatten() {
        let scenario_name = scenario_entry.file_name().to_string_lossy().to_string();
        if let Some(filter) = scenario {
            if filter != scenario_name {
                continue;
            }
        }
        let Ok(runs) = std::fs::read_dir(scenario_entry.path()) else {
            continue;
        };
        for run in runs.flatten() {
            let Ok(files) = std::fs::read_dir(run.path()) else {
                continue;
            };
            for file in files.flatten() {
                if let Ok(rel) = file.path().strip_prefix(repo_root) {
                    rows.push(rel.display().to_string());
                }
            }
        }
    }
    rows.sort();
    rows
}

pub(crate) fn run_ops_diagnose_bundle(args: &crate::cli::OpsDiagnoseBundleArgs) -> Result<(String, i32), String> {
    if !args.common.allow_write {
        return Err("diagnose bundle requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let run_id = run_id_or_default(args.common.run_id.clone())?;
    let out_dir = diagnose_root(&repo_root)?.join(run_id.as_str());
    std::fs::create_dir_all(&out_dir)
        .map_err(|err| format!("failed to create {}: {err}", out_dir.display()))?;

    let files = collect_scenario_files(&repo_root, args.scenario.as_deref());
    let bundle = serde_json::json!({
        "schema_version": 1,
        "kind": "ops_diagnose_bundle",
        "run_id": run_id.as_str(),
        "scenario_filter": args.scenario,
        "files": files,
        "sensitive_keys": ["password", "secret", "token", "api_key"]
    });
    let bundle_path = out_dir.join("bundle.json");
    std::fs::write(
        &bundle_path,
        serde_json::to_string_pretty(&bundle)
            .map_err(|err| format!("failed to encode {}: {err}", bundle_path.display()))?,
    )
    .map_err(|err| format!("failed to write {}: {err}", bundle_path.display()))?;

    let payload = serde_json::json!({
      "schema_version": 1,
      "text": "ops diagnose bundle",
      "rows": [{
        "bundle": bundle_path.strip_prefix(&repo_root).unwrap_or(&bundle_path).display().to_string(),
        "files": bundle.get("files").cloned().unwrap_or_else(|| serde_json::json!([]))
      }],
      "summary": {"total": 1, "errors": 0, "warnings": 0}
    });
    let rendered = emit_payload(args.common.format, args.common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_ops_diagnose_explain(args: &crate::cli::OpsDiagnoseExplainArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let bundle_path = if args.bundle.is_absolute() {
        args.bundle.clone()
    } else {
        repo_root.join(&args.bundle)
    };
    let raw = std::fs::read_to_string(&bundle_path)
        .map_err(|err| format!("failed to read {}: {err}", bundle_path.display()))?;
    let parsed: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|err| format!("failed to parse {}: {err}", bundle_path.display()))?;
    let file_count = parsed
        .get("files")
        .and_then(|v| v.as_array())
        .map(|v| v.len())
        .unwrap_or(0);
    let payload = serde_json::json!({
      "schema_version": 1,
      "text": "ops diagnose explain",
      "rows": [{
        "bundle": bundle_path.strip_prefix(&repo_root).unwrap_or(&bundle_path).display().to_string(),
        "kind": parsed.get("kind").and_then(|v| v.as_str()).unwrap_or("unknown"),
        "run_id": parsed.get("run_id").and_then(|v| v.as_str()).unwrap_or("unknown"),
        "file_count": file_count,
        "summary": if file_count == 0 { "no evidence files discovered" } else { "bundle contains evidence files" }
      }],
      "summary": {"total": 1, "errors": 0, "warnings": 0}
    });
    let rendered = emit_payload(args.common.format, args.common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_ops_diagnose_redact(args: &crate::cli::OpsDiagnoseRedactArgs) -> Result<(String, i32), String> {
    if !args.common.allow_write {
        return Err("diagnose redact requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let bundle_path = if args.bundle.is_absolute() {
        args.bundle.clone()
    } else {
        repo_root.join(&args.bundle)
    };
    let raw = std::fs::read_to_string(&bundle_path)
        .map_err(|err| format!("failed to read {}: {err}", bundle_path.display()))?;
    let mut parsed: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|err| format!("failed to parse {}: {err}", bundle_path.display()))?;

    // Explicit redaction policy keys that must never leak from bundle metadata.
    let redact_keys = ["password", "secret", "token", "api_key"];
    let mut redacted = Vec::new();
    if let Some(object) = parsed.as_object_mut() {
        for key in redact_keys {
            if object.remove(key).is_some() {
                redacted.push(key.to_string());
            }
        }
        object.insert("redaction_policy".to_string(), serde_json::json!(redact_keys));
        object.insert("redaction_applied".to_string(), serde_json::json!(true));
    }

    let out_path = bundle_path
        .parent()
        .unwrap_or(&repo_root)
        .join("bundle.redacted.json");
    std::fs::write(
        &out_path,
        serde_json::to_string_pretty(&parsed)
            .map_err(|err| format!("failed to encode {}: {err}", out_path.display()))?,
    )
    .map_err(|err| format!("failed to write {}: {err}", out_path.display()))?;

    let payload = serde_json::json!({
      "schema_version": 1,
      "text": "ops diagnose redact",
      "rows": [{
        "source": bundle_path.strip_prefix(&repo_root).unwrap_or(&bundle_path).display().to_string(),
        "redacted": out_path.strip_prefix(&repo_root).unwrap_or(&out_path).display().to_string(),
        "redacted_keys": redacted,
        "policy_keys": redact_keys
      }],
      "summary": {"total": 1, "errors": 0, "warnings": 0}
    });
    let rendered = emit_payload(args.common.format, args.common.out.clone(), &payload)?;
    Ok((rendered, 0))
}
