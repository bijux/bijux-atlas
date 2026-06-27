// SPDX-License-Identifier: Apache-2.0

use crate::emit_payload;
use crate::ops_commands::run_id_or_default;
use crate::resolve_repo_root;
use bijux_atlas_ops::diagnostics::bundle_contracts::{
    build_diagnose_bundle, collect_scenario_files, write_diagnose_bundle,
};
use bijux_atlas_ops::diagnostics::bundle_payload::diagnose_bundle_payload;
use bijux_atlas_ops::diagnostics::explain_payload::diagnose_explain_payload;

pub(crate) fn run_ops_diagnose_bundle(
    args: &crate::cli::OpsDiagnoseBundleArgs,
) -> Result<(String, i32), String> {
    if !args.common.allow_write {
        return Err("diagnose bundle requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let run_id = run_id_or_default(args.common.run_id.clone())?;
    let files = collect_scenario_files(&repo_root, args.scenario.as_deref());
    let bundle = build_diagnose_bundle(run_id.as_str(), args.scenario.as_deref(), files);
    let bundle_path = write_diagnose_bundle(&repo_root, run_id.as_str(), &bundle)?;

    let payload = diagnose_bundle_payload(
        &bundle_path
            .strip_prefix(&repo_root)
            .unwrap_or(&bundle_path)
            .display()
            .to_string(),
        bundle
            .get("files")
            .cloned()
            .unwrap_or_else(|| serde_json::json!([])),
    );
    let rendered = emit_payload(args.common.format, args.common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_ops_diagnose_explain(
    args: &crate::cli::OpsDiagnoseExplainArgs,
) -> Result<(String, i32), String> {
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
    let payload = diagnose_explain_payload(
        &bundle_path
            .strip_prefix(&repo_root)
            .unwrap_or(&bundle_path)
            .display()
            .to_string(),
        parsed
            .get("kind")
            .and_then(|value| value.as_str())
            .unwrap_or("unknown"),
        parsed
            .get("run_id")
            .and_then(|value| value.as_str())
            .unwrap_or("unknown"),
        file_count,
    );
    let rendered = emit_payload(args.common.format, args.common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_ops_diagnose_redact(
    args: &crate::cli::OpsDiagnoseRedactArgs,
) -> Result<(String, i32), String> {
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
        object.insert(
            "redaction_policy".to_string(),
            serde_json::json!(redact_keys),
        );
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
