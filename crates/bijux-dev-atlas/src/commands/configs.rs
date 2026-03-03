// SPDX-License-Identifier: Apache-2.0

mod configs_path_drift;
mod configs_payloads;

use crate::*;
use bijux_dev_atlas::contracts;
pub(crate) use configs_path_drift::configs_context;
pub(crate) use configs_path_drift::parse_config_file;
use configs_path_drift::{configs_files, configs_verify_payload};
pub(crate) use configs_payloads::{
    configs_artifact_dir, configs_compile_payload, configs_diff_payload, configs_inventory_payload,
    configs_lint_payload, configs_print_payload, configs_validate_payload,
};
use std::io::{self, Write};

pub(crate) fn run_configs_command(quiet: bool, command: ConfigsCommand) -> i32 {
    let run = (|| -> Result<(String, i32), String> {
        let started = std::time::Instant::now();
        match command {
            ConfigsCommand::Print(common) => {
                let ctx = configs_context(&common)?;
                let mut payload = configs_print_payload(&ctx, &common)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                Ok((emit_payload(common.format, common.out, &payload)?, 0))
            }
            ConfigsCommand::Verify(common) => {
                let ctx = configs_context(&common)?;
                let mut payload = configs_verify_payload(&ctx, &common)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                let code = if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) {
                    1
                } else {
                    0
                };
                Ok((emit_payload(common.format, common.out, &payload)?, code))
            }
            ConfigsCommand::List(common) => {
                let ctx = configs_context(&common)?;
                let mut payload = contracts::configs::list_payload(&ctx.repo_root)?;
                if common.allow_write {
                    let index_path = contracts::configs::ensure_generated_index(&ctx.repo_root)?;
                    let schema_index_path =
                        contracts::configs::ensure_generated_schema_index(&ctx.repo_root)?;
                    let coverage_path = contracts::configs::write_cfg_contract_coverage_artifact(
                        &ctx.repo_root,
                        &ctx.artifacts_root,
                        ctx.run_id.as_str(),
                    )?;
                    payload["artifacts"] = serde_json::json!({
                        "generated_index": index_path,
                        "schema_index": schema_index_path,
                        "cfg_contract_coverage": coverage_path
                    });
                }
                payload["run_id"] = serde_json::json!(ctx.run_id.as_str());
                payload["text"] = serde_json::json!("configs list registry");
                payload["capabilities"] = serde_json::json!({
                    "fs_write": common.allow_write,
                    "subprocess": common.allow_subprocess,
                    "network": common.allow_network
                });
                payload["options"] = serde_json::json!({
                    "strict": common.strict
                });
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                Ok((emit_payload(common.format, common.out, &payload)?, 0))
            }
            ConfigsCommand::Graph(common) => {
                let ctx = configs_context(&common)?;
                let mut payload = contracts::configs::graph_payload(&ctx.repo_root)?;
                if common.allow_write {
                    let out_dir = configs_artifact_dir(&ctx);
                    fs::create_dir_all(&out_dir)
                        .map_err(|e| format!("failed to create {}: {e}", out_dir.display()))?;
                    let graph_path = out_dir.join("graph.json");
                    fs::write(
                        &graph_path,
                        serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?,
                    )
                    .map_err(|e| format!("failed to write {}: {e}", graph_path.display()))?;
                    payload["artifacts"] = serde_json::json!({
                        "graph": graph_path.display().to_string()
                    });
                }
                payload["run_id"] = serde_json::json!(ctx.run_id.as_str());
                payload["text"] = serde_json::json!("configs graph");
                payload["capabilities"] = serde_json::json!({
                    "fs_write": common.allow_write,
                    "subprocess": common.allow_subprocess,
                    "network": common.allow_network
                });
                payload["options"] = serde_json::json!({
                    "strict": common.strict
                });
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                let code = if payload["orphans"].as_array().is_some_and(|v| !v.is_empty()) {
                    1
                } else {
                    0
                };
                Ok((emit_payload(common.format, common.out, &payload)?, code))
            }
            ConfigsCommand::Explain(args) => {
                let ctx = configs_context(&args.common)?;
                let mut payload = contracts::configs::explain_payload(&ctx.repo_root, &args.file)?;
                payload["run_id"] = serde_json::json!(ctx.run_id.as_str());
                payload["text"] = serde_json::json!("configs explain registry");
                payload["capabilities"] = serde_json::json!({
                    "fs_write": args.common.allow_write,
                    "subprocess": args.common.allow_subprocess,
                    "network": args.common.allow_network
                });
                payload["options"] = serde_json::json!({
                    "strict": args.common.strict
                });
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                Ok((
                    emit_payload(args.common.format, args.common.out.clone(), &payload)?,
                    0,
                ))
            }
            ConfigsCommand::Inventory(common) => {
                let ctx = configs_context(&common)?;
                let mut payload = configs_inventory_payload(&ctx, &common)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                Ok((emit_payload(common.format, common.out, &payload)?, 0))
            }
            ConfigsCommand::Validate(common) => {
                let ctx = configs_context(&common)?;
                let mut payload = configs_validate_payload(&ctx, &common)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                let code = if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) {
                    1
                } else {
                    0
                };
                if code != 0 && payload.get("error_code").is_none() {
                    payload["error_code"] = serde_json::json!("CONFIGS_SCHEMA_ERROR");
                }
                Ok((emit_payload(common.format, common.out, &payload)?, code))
            }
            ConfigsCommand::Lint(common) => {
                let ctx = configs_context(&common)?;
                let mut payload = configs_lint_payload(&ctx, &common)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                let code = if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) {
                    1
                } else {
                    0
                };
                Ok((emit_payload(common.format, common.out, &payload)?, code))
            }
            ConfigsCommand::Fmt { check, common } => {
                let ctx = configs_context(&common)?;
                let mut payload = configs_lint_payload(&ctx, &common)?;
                payload["text"] = serde_json::json!(if check {
                    if payload["errors"].as_array().is_some_and(|v| v.is_empty()) {
                        "configs fmt --check passed"
                    } else {
                        "configs fmt --check failed"
                    }
                } else if payload["errors"].as_array().is_some_and(|v| v.is_empty()) {
                    "configs fmt passed"
                } else {
                    "configs fmt failed"
                });
                payload["mode"] = serde_json::json!(if check { "check" } else { "apply" });
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                if !check {
                    return Err("configs fmt apply is not implemented; use --check".to_string());
                }
                let code = if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) {
                    1
                } else {
                    0
                };
                Ok((emit_payload(common.format, common.out, &payload)?, code))
            }
            ConfigsCommand::Compile(common) => {
                let ctx = configs_context(&common)?;
                let mut payload = configs_compile_payload(&ctx, &common)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                Ok((emit_payload(common.format, common.out, &payload)?, 0))
            }
            ConfigsCommand::Diff(common) => {
                let ctx = configs_context(&common)?;
                let mut payload = configs_diff_payload(&ctx, &common)?;
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                let code = if payload["errors"].as_array().is_some_and(|v| !v.is_empty()) {
                    1
                } else {
                    0
                };
                Ok((emit_payload(common.format, common.out, &payload)?, code))
            }
            ConfigsCommand::Doctor(common) => {
                let ctx = configs_context(&common)?;
                let validate = configs_validate_payload(&ctx, &common)?;
                let lint = configs_lint_payload(&ctx, &common)?;
                let diff = configs_diff_payload(&ctx, &common)?;
                let mut compile_status = "skipped";
                if common.allow_write {
                    let _ = configs_compile_payload(&ctx, &common)?;
                    compile_status = "ok";
                }
                let errors = validate["errors"].as_array().map(|v| v.len()).unwrap_or(0)
                    + lint["errors"].as_array().map(|v| v.len()).unwrap_or(0)
                    + diff["errors"].as_array().map(|v| v.len()).unwrap_or(0);
                let payload = serde_json::json!({
                    "schema_version":1,
                    "run_id":ctx.run_id.as_str(),
                    "text": if errors == 0 { format!("configs: 4 checks collected, 0 failed, compile={compile_status}") } else { format!("configs: 4 checks collected, {errors} failed, compile={compile_status}") },
                    "rows":[
                        {"name":"validate","errors": validate["errors"].as_array().map(|v| v.len()).unwrap_or(0)},
                        {"name":"lint","errors": lint["errors"].as_array().map(|v| v.len()).unwrap_or(0)},
                        {"name":"diff","errors": diff["errors"].as_array().map(|v| v.len()).unwrap_or(0)},
                        {"name":"compile","status": compile_status}
                    ],
                    "counts":{"errors": errors},
                    "error_code": if errors == 0 { serde_json::Value::Null } else { serde_json::Value::String("CONFIGS_DRIFT_ERROR".to_string()) },
                    "capabilities":{"fs_write": common.allow_write, "subprocess": common.allow_subprocess, "network": common.allow_network},
                    "options":{"strict": common.strict},
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                Ok((
                    emit_payload(common.format, common.out, &payload)?,
                    if errors == 0 { 0 } else { 1 },
                ))
            }
        }
    })();
    match run {
        Ok((rendered, code)) => {
            if !quiet && !rendered.is_empty() {
                if code == 0 {
                    let _ = writeln!(io::stdout(), "{rendered}");
                } else {
                    let _ = writeln!(io::stderr(), "{rendered}");
                }
            }
            code
        }
        Err(err) => {
            let _ = writeln!(io::stderr(), "bijux-dev-atlas configs failed: {err}");
            1
        }
    }
}
