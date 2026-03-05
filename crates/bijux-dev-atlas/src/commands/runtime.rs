// SPDX-License-Identifier: Apache-2.0

use crate::cli::{RuntimeCommand, RuntimeCommandArgs};
use crate::{emit_payload, resolve_repo_root};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

fn runtime_config_schema_path(root: &std::path::Path) -> PathBuf {
    root.join("crates/bijux-atlas-server/docs/generated/runtime-startup-config.schema.json")
}

fn read_runtime_config_schema(root: &std::path::Path) -> Result<Value, String> {
    let schema_path = runtime_config_schema_path(root);
    let schema_text = fs::read_to_string(&schema_path)
        .map_err(|e| format!("failed to read {}: {e}", schema_path.display()))?;
    serde_json::from_str(&schema_text)
        .map_err(|e| format!("invalid runtime config schema JSON: {e}"))
}

fn stable_schema_sha256(schema: &Value) -> Result<String, String> {
    let bytes = serde_json::to_vec(schema).map_err(|e| format!("encode schema bytes failed: {e}"))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn run_runtime_self_check(args: RuntimeCommandArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let schema = read_runtime_config_schema(&root)?;
    let schema_path = runtime_config_schema_path(&root);
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "runtime_self_check",
        "status": "ok",
        "checks": [
            {"id": "runtime_schema_exists", "status": "ok", "path": schema_path.strip_prefix(&root).unwrap_or(&schema_path).display().to_string()},
            {"id": "runtime_schema_parses", "status": "ok"},
            {"id": "runtime_schema_digest", "status": "ok", "sha256": stable_schema_sha256(&schema)?}
        ]
    });
    Ok((emit_payload(args.format, args.out, &payload)?, 0))
}

fn run_runtime_print_config_schema(args: RuntimeCommandArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let schema = read_runtime_config_schema(&root)?;
    Ok((emit_payload(args.format, args.out, &schema)?, 0))
}

fn run_runtime_explain_config_schema(args: RuntimeCommandArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let schema = read_runtime_config_schema(&root)?;
    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .map(|rows| rows.iter().filter_map(Value::as_str).collect::<Vec<_>>())
        .unwrap_or_default();
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "runtime_explain_config_schema",
        "status": "ok",
        "schema_path": runtime_config_schema_path(&root).strip_prefix(&root).unwrap_or(&runtime_config_schema_path(&root)).display().to_string(),
        "required_fields": required,
        "top_level_keys": schema.as_object().map(|m| m.keys().cloned().collect::<Vec<_>>()).unwrap_or_default()
    });
    Ok((emit_payload(args.format, args.out, &payload)?, 0))
}

pub(crate) fn run_runtime_command(
    _quiet: bool,
    command: RuntimeCommand,
) -> Result<(String, i32), String> {
    match command {
        RuntimeCommand::SelfCheck(args) => run_runtime_self_check(args),
        RuntimeCommand::PrintConfigSchema(args) => run_runtime_print_config_schema(args),
        RuntimeCommand::ExplainConfigSchema(args) => run_runtime_explain_config_schema(args),
    }
}
