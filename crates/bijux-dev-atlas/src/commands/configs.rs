// SPDX-License-Identifier: Apache-2.0

use crate::*;
use std::collections::BTreeMap;

pub(crate) fn configs_context(common: &ConfigsCommonArgs) -> Result<ConfigsContext, String> {
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let artifacts_root = common
        .artifacts_root
        .clone()
        .unwrap_or_else(|| repo_root.join("artifacts"));
    let run_id = common
        .run_id
        .as_ref()
        .map(|v| RunId::parse(v))
        .transpose()?
        .unwrap_or_else(|| RunId::from_seed("configs_run"));
    Ok(ConfigsContext {
        configs_root: repo_root.join("configs"),
        repo_root,
        artifacts_root,
        run_id,
    })
}

fn configs_files(ctx: &ConfigsContext) -> Vec<PathBuf> {
    if !ctx.configs_root.exists() {
        return Vec::new();
    }
    walk_files_local(&ctx.configs_root)
}

pub(crate) fn parse_config_file(path: &Path) -> Result<(), String> {
    let ext = path
        .extension()
        .and_then(|v| v.to_str())
        .unwrap_or_default();
    let text =
        fs::read_to_string(path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    match ext {
        "json" | "schema" => serde_json::from_str::<serde_json::Value>(&text)
            .map(|_| ())
            .map_err(|e| format!("CONFIGS_PARSE_ERROR: {}: {e}", path.display())),
        "toml" => text
            .parse::<toml::Value>()
            .map(|_| ())
            .map_err(|e| format!("CONFIGS_PARSE_ERROR: {}: {e}", path.display())),
        "yml" | "yaml" => serde_yaml::from_str::<YamlValue>(&text)
            .map(|_| ())
            .map_err(|e| format!("CONFIGS_PARSE_ERROR: {}: {e}", path.display())),
        _ => Ok(()),
    }
}

fn configs_inventory_payload(
    ctx: &ConfigsContext,
    common: &ConfigsCommonArgs,
) -> Result<serde_json::Value, String> {
    let mut rows = Vec::<serde_json::Value>::new();
    let mut seen = std::collections::BTreeSet::new();
    let mut orphans = Vec::<String>::new();
    for file in configs_files(ctx) {
        let rel = file
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&file)
            .display()
            .to_string();
        let status = match parse_config_file(&file) {
            Ok(_) => "ok",
            Err(_) => "parse_error",
        };
        rows.push(serde_json::json!({"path": rel.clone(), "status": status}));
        seen.insert(rel.clone());
    }
    rows.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    for required in ["configs/INDEX.md", "configs/README.md", "configs/contracts"] {
        if !ctx.repo_root.join(required).exists() {
            orphans.push(format!("missing required config surface `{required}`"));
        }
    }
    let payload = serde_json::json!({
        "schema_version": 1,
        "run_id": ctx.run_id.as_str(),
        "rows": rows,
        "orphans": orphans,
        "capabilities": {"fs_write": common.allow_write, "subprocess": common.allow_subprocess, "network": common.allow_network},
        "options": {"strict": common.strict}
    });
    if common.allow_write {
        let out_dir = ctx
            .artifacts_root
            .join("atlas-dev")
            .join("configs")
            .join(ctx.run_id.as_str());
        fs::create_dir_all(&out_dir)
            .map_err(|e| format!("failed to create {}: {e}", out_dir.display()))?;
        let inventory_path = out_dir.join("inventory.json");
        fs::write(
            &inventory_path,
            serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?,
        )
        .map_err(|e| format!("failed to write {}: {e}", inventory_path.display()))?;
        let mut with_artifact = payload;
        with_artifact["artifacts"] = serde_json::json!({
            "inventory": inventory_path.display().to_string()
        });
        Ok(with_artifact)
    } else {
        Ok(payload)
    }
}

pub(crate) fn configs_validate_payload(
    ctx: &ConfigsContext,
    common: &ConfigsCommonArgs,
) -> Result<serde_json::Value, String> {
    let mut errors = Vec::<String>::new();
    let mut warnings = Vec::<String>::new();
    for required in [
        "configs/INDEX.md",
        "configs/README.md",
        "configs/contracts",
        "configs/schema",
    ] {
        if !ctx.repo_root.join(required).exists() {
            errors.push(format!(
                "CONFIGS_SCHEMA_ERROR: missing required config path `{required}`"
            ));
        }
    }
    for file in configs_files(ctx) {
        let rel = file
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&file)
            .display()
            .to_string();
        if let Err(e) = parse_config_file(&file) {
            errors.push(e);
        }
        if rel.contains(".example.secret") {
            warnings.push(format!("CONFIGS_SCHEMA_ERROR: secret-like config filename requires explicit allowlist `{rel}`"));
        }
    }
    if common.strict {
        errors.append(&mut warnings);
    }
    errors.sort();
    errors.dedup();
    warnings.sort();
    warnings.dedup();
    Ok(serde_json::json!({
        "schema_version": 1,
        "run_id": ctx.run_id.as_str(),
        "text": if errors.is_empty() { format!("configs validate passed (warnings={})", warnings.len()) } else { format!("configs validate failed (errors={} warnings={})", errors.len(), warnings.len()) },
        "errors": errors,
        "warnings": warnings,
        "capabilities": {"fs_write": common.allow_write, "subprocess": common.allow_subprocess, "network": common.allow_network},
        "options": {"strict": common.strict}
    }))
}

pub(crate) fn configs_lint_payload(
    ctx: &ConfigsContext,
    common: &ConfigsCommonArgs,
) -> Result<serde_json::Value, String> {
    let mut errors = Vec::<String>::new();
    for file in configs_files(ctx) {
        let rel = file
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&file)
            .display()
            .to_string();
        let name = file
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default();
        let ext = file
            .extension()
            .and_then(|v| v.to_str())
            .unwrap_or_default();
        let is_structured_config = matches!(ext, "json" | "yaml" | "yml" | "toml");
        if !is_structured_config {
            continue;
        }
        if rel.contains(' ') {
            errors.push(format!(
                "CONFIGS_PARSE_ERROR: config path contains spaces `{rel}`"
            ));
        }
        if name.chars().any(|c| c.is_ascii_uppercase()) {
            errors.push(format!(
                "CONFIGS_PARSE_ERROR: config filename should be lowercase `{rel}`"
            ));
        }
        let text = fs::read_to_string(&file).unwrap_or_default();
        for (i, line) in text.lines().enumerate() {
            if line.contains("${") {
                errors.push(format!(
                    "CONFIGS_PARSE_ERROR: {rel}:{} env interpolation defaults are forbidden",
                    i + 1
                ));
            }
            if (line.contains("password") || line.contains("secret"))
                && !rel.contains("allowlist")
                && !rel.contains("README")
            {
                errors.push(format!(
                    "CONFIGS_SCHEMA_ERROR: {rel}:{} potential secret-like key requires allowlist review",
                    i + 1
                ));
            }
        }
    }
    errors.sort();
    errors.dedup();
    Ok(
        serde_json::json!({"schema_version":1,"run_id":ctx.run_id.as_str(),"text": if errors.is_empty() {"configs lint passed"} else {"configs lint failed"},"errors":errors,"warnings":[],"capabilities":{"fs_write": common.allow_write, "subprocess": common.allow_subprocess, "network": common.allow_network},"options":{"strict": common.strict}}),
    )
}

fn configs_compile_payload(
    ctx: &ConfigsContext,
    common: &ConfigsCommonArgs,
) -> Result<serde_json::Value, String> {
    if !common.allow_write {
        return Err("configs compile requires --allow-write".to_string());
    }
    let out_dir = ctx
        .artifacts_root
        .join("atlas-dev")
        .join("configs")
        .join(ctx.run_id.as_str());
    fs::create_dir_all(&out_dir)
        .map_err(|e| format!("failed to create {}: {e}", out_dir.display()))?;
    let mut files = Vec::<serde_json::Value>::new();
    let mut merged = BTreeMap::<String, serde_json::Value>::new();
    for file in configs_files(ctx) {
        let rel = file
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&file)
            .display()
            .to_string();
        let bytes = fs::read(&file).map_err(|e| format!("failed to read {rel}: {e}"))?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        files.push(serde_json::json!({"path": rel.clone(), "sha256": format!("{:x}", hasher.finalize()), "bytes": bytes.len()}));
        merged.insert(rel, serde_json::json!({"bytes": bytes.len()}));
    }
    files.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    let compiled_path = out_dir.join("compiled.index.json");
    let compiled = serde_json::json!({"schema_version":1,"run_id": ctx.run_id.as_str(),"files": files,"summary": merged});
    fs::write(
        &compiled_path,
        serde_json::to_string_pretty(&compiled).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("failed to write {}: {e}", compiled_path.display()))?;
    Ok(serde_json::json!({
        "schema_version":1,
        "run_id":ctx.run_id.as_str(),
        "text":"configs compile ok",
        "rows":[{"artifact": compiled_path.display().to_string()}],
        "artifacts":{"root": out_dir.display().to_string(), "index": compiled_path.display().to_string()},
        "capabilities":{"fs_write": common.allow_write, "subprocess": common.allow_subprocess, "network": common.allow_network},
        "options":{"strict": common.strict}
    }))
}

fn configs_print_payload(
    ctx: &ConfigsContext,
    common: &ConfigsCommonArgs,
) -> Result<serde_json::Value, String> {
    let inventory = configs_inventory_payload(ctx, common)?;
    let mut rows = inventory["rows"].as_array().cloned().unwrap_or_default();
    rows.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    Ok(serde_json::json!({
        "schema_version": 1,
        "run_id": ctx.run_id.as_str(),
        "text": "configs print resolved inventory",
        "rows": rows,
        "orphans": inventory["orphans"].as_array().cloned().unwrap_or_default(),
        "capabilities": {"fs_write": common.allow_write, "subprocess": common.allow_subprocess, "network": common.allow_network},
        "options": {"strict": common.strict}
    }))
}

pub(crate) fn configs_diff_payload(
    ctx: &ConfigsContext,
    common: &ConfigsCommonArgs,
) -> Result<serde_json::Value, String> {
    let mut baseline_hasher = Sha256::new();
    for file in configs_files(ctx) {
        let rel = file
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&file)
            .display()
            .to_string();
        baseline_hasher.update(rel.as_bytes());
        baseline_hasher.update(fs::read(&file).unwrap_or_default());
    }
    let digest_one = format!("{:x}", baseline_hasher.finalize());
    let mut baseline_hasher_two = Sha256::new();
    for file in configs_files(ctx) {
        let rel = file
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&file)
            .display()
            .to_string();
        baseline_hasher_two.update(rel.as_bytes());
        baseline_hasher_two.update(fs::read(&file).unwrap_or_default());
    }
    let digest_two = format!("{:x}", baseline_hasher_two.finalize());
    let deterministic = digest_one == digest_two;
    Ok(serde_json::json!({
        "schema_version":1,
        "run_id":ctx.run_id.as_str(),
        "text": if deterministic {"configs diff passed"} else {"configs diff failed"},
        "rows":[{"deterministic": deterministic, "digest_one": digest_one, "digest_two": digest_two}],
        "errors": if deterministic { Vec::<String>::new() } else { vec!["CONFIGS_DRIFT_ERROR: configs compile is not deterministic".to_string()] },
        "capabilities":{"fs_write": common.allow_write, "subprocess": common.allow_subprocess, "network": common.allow_network},
        "options":{"strict": common.strict}
    }))
}

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
            ConfigsCommand::List(common) => {
                let ctx = configs_context(&common)?;
                let mut payload = configs_inventory_payload(&ctx, &common)?;
                payload["text"] = serde_json::json!("configs list inventory");
                payload["duration_ms"] = serde_json::json!(started.elapsed().as_millis() as u64);
                Ok((emit_payload(common.format, common.out, &payload)?, 0))
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
                    println!("{rendered}");
                } else {
                    eprintln!("{rendered}");
                }
            }
            code
        }
        Err(err) => {
            eprintln!("bijux-dev-atlas configs failed: {err}");
            1
        }
    }
}
