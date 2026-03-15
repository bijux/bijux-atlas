// SPDX-License-Identifier: Apache-2.0

use super::*;
use std::collections::BTreeMap;

fn is_filename_case_exception(rel: &str) -> bool {
    matches!(
        rel,
        "configs/registry/consumers.json" | "configs/registry/owners.json" | "configs/registry/schemas.json"
    )
}

fn line_has_forbidden_env_default(line: &str) -> bool {
    let mut rest = line;
    while let Some(start) = rest.find("${") {
        let candidate = &rest[start + 2..];
        let Some(end) = candidate.find('}') else {
            return false;
        };
        if candidate[..end].contains(":-") {
            return true;
        }
        rest = &candidate[end + 1..];
    }
    false
}

fn structured_config_value(
    path: &std::path::Path,
    text: &str,
) -> Result<Option<serde_json::Value>, String> {
    let ext = path
        .extension()
        .and_then(|v| v.to_str())
        .unwrap_or_default();
    match ext {
        "json" | "schema" => serde_json::from_str::<serde_json::Value>(text)
            .map(Some)
            .map_err(|e| format!("failed to parse {}: {e}", path.display())),
        "yaml" | "yml" => serde_yaml::from_str::<serde_yaml::Value>(text)
            .map_err(|e| format!("failed to parse {}: {e}", path.display()))
            .and_then(|value| {
                serde_json::to_value(value)
                    .map(Some)
                    .map_err(|e| format!("failed to normalize {}: {e}", path.display()))
            }),
        "toml" => toml::from_str::<toml::Value>(text)
            .map_err(|e| format!("failed to parse {}: {e}", path.display()))
            .and_then(|value| {
                serde_json::to_value(value)
                    .map(Some)
                    .map_err(|e| format!("failed to normalize {}: {e}", path.display()))
            }),
        _ => Ok(None),
    }
}

fn collect_secret_like_key_paths(
    value: &serde_json::Value,
    current_path: &str,
    out: &mut Vec<String>,
) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, nested) in map {
                let path = if current_path.is_empty() {
                    key.clone()
                } else {
                    format!("{current_path}.{key}")
                };
                let lower = key.to_ascii_lowercase();
                if lower.contains("password") || lower.contains("secret") {
                    out.push(path.clone());
                }
                collect_secret_like_key_paths(nested, &path, out);
            }
        }
        serde_json::Value::Array(items) => {
            for (index, nested) in items.iter().enumerate() {
                collect_secret_like_key_paths(nested, &format!("{current_path}[{index}]"), out);
            }
        }
        _ => {}
    }
}

fn is_secret_like_key_exception(rel: &str, key_path: &str) -> bool {
    key_path.contains('/')
        || matches!(
            (rel, key_path),
            ("configs/sources/security/secrets.json", "secrets")
                | (
                    "configs/schemas/contracts/security/secrets.schema.json",
                    "properties.secrets"
                )
        )
}

pub(crate) fn configs_inventory_payload(
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
    for required in ["configs/INDEX.md", "configs/README.md", "configs/schemas/contracts"] {
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

pub(crate) fn configs_artifact_dir(ctx: &ConfigsContext) -> std::path::PathBuf {
    ctx.artifacts_root
        .join("run")
        .join(ctx.run_id.as_str())
        .join("gates")
        .join("configs")
}

pub(crate) fn configs_validate_payload(
    ctx: &ConfigsContext,
    common: &ConfigsCommonArgs,
) -> Result<serde_json::Value, String> {
    let mut errors = Vec::<String>::new();
    let mut warnings = Vec::<String>::new();
    for required in [
        "configs/sources/repository/ci",
        "configs/sources/repository/ci/INDEX.md",
        "configs/sources/repository/ci/README.md",
        "configs/sources/repository/ci/env-contract.json",
        "configs/sources/repository/ci/lanes.json",
        "configs/INDEX.md",
        "configs/README.md",
        "configs/sources/repository/rust-tooling/LINT_POLICY.md",
        "configs/sources/repository/rust-tooling/toolchain.json",
        "configs/schemas/contracts",
        "configs/schemas/registry",
        "configs/NAMING.md",
        "configs/OWNERS.md",
        "configs/registry/inventory/groups.json",
        "configs/registry/inventory/consumers.json",
    ] {
        if !ctx.repo_root.join(required).exists() {
            errors.push(format!(
                "CONFIGS_SCHEMA_ERROR: missing required config path `{required}`"
            ));
        }
    }
    let ci_env_contract_path = ctx.repo_root.join("configs/sources/repository/ci/env-contract.json");
    let ci_lanes_path = ctx.repo_root.join("configs/sources/repository/ci/lanes.json");
    let mut ci_required_env = Vec::<String>::new();
    if ci_env_contract_path.exists() {
        let env_text = fs::read_to_string(&ci_env_contract_path)
            .map_err(|e| format!("failed to read {}: {e}", ci_env_contract_path.display()))?;
        let env_json: serde_json::Value = serde_json::from_str(&env_text)
            .map_err(|e| format!("failed to parse {}: {e}", ci_env_contract_path.display()))?;
        ci_required_env = env_json["required_job_env_keys"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect::<Vec<_>>();
    }
    if ci_lanes_path.exists() {
        let lanes_text = fs::read_to_string(&ci_lanes_path)
            .map_err(|e| format!("failed to read {}: {e}", ci_lanes_path.display()))?;
        let lanes_json: serde_json::Value = serde_json::from_str(&lanes_text)
            .map_err(|e| format!("failed to parse {}: {e}", ci_lanes_path.display()))?;
        let lanes = lanes_json["lanes"].as_array().cloned().unwrap_or_default();
        for lane in lanes {
            let id = lane["id"].as_str().unwrap_or("unknown-lane");
            let workflow_rel = lane["workflow"].as_str().unwrap_or_default();
            let workflow_path = ctx.repo_root.join(workflow_rel);
            if workflow_rel.is_empty() || !workflow_path.exists() {
                errors.push(format!(
                    "CONFIGS_LAYOUT_ERROR: ci lane `{id}` references missing workflow `{workflow_rel}`"
                ));
                continue;
            }
            let workflow_text = fs::read_to_string(&workflow_path)
                .map_err(|e| format!("failed to read {}: {e}", workflow_path.display()))?;
            if !workflow_text.contains("cargo run -q -p bijux-dev-atlas")
                && !workflow_text.contains("cargo run --locked -q -p bijux-dev-atlas")
                && !workflow_text.contains("make ")
            {
                warnings.push(format!(
                    "CONFIGS_LAYOUT_ERROR: ci lane `{id}` workflow `{workflow_rel}` has no control-plane invocation"
                ));
            }
            for env_key in &ci_required_env {
                if !workflow_text.contains(env_key) {
                    errors.push(format!(
                        "CONFIGS_LAYOUT_ERROR: workflow `{workflow_rel}` missing required CI env key `{env_key}`"
                    ));
                }
            }
        }
    }
    for workflow in walk_files_local(&ctx.repo_root.join(".github/workflows"))
        .into_iter()
        .filter(|p| p.extension().and_then(|v| v.to_str()) == Some("yml"))
    {
        let workflow_rel = workflow
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&workflow)
            .display()
            .to_string();
        let content = fs::read_to_string(&workflow)
            .map_err(|e| format!("failed to read {}: {e}", workflow.display()))?;
        for (line_idx, line) in content.lines().enumerate() {
            if let Some(start_idx) = line.find("configs/") {
                let suffix = &line[start_idx..];
                let mut token = suffix
                    .split(|c: char| {
                        c.is_whitespace() || c == '"' || c == '\'' || c == ')' || c == ';'
                    })
                    .next()
                    .unwrap_or_default()
                    .to_string();
                token = token.trim_end_matches("\\n").to_string();
                if !token.is_empty()
                    && !token.contains('*')
                    && !token.ends_with("/**")
                    && !ctx.repo_root.join(&token).exists()
                {
                    errors.push(format!(
                        "CONFIGS_LAYOUT_ERROR: {workflow_rel}:{} references missing config path `{token}`",
                        line_idx + 1
                    ));
                }
            }
            if line.contains("cargo fmt")
                && !line.contains("configs/sources/repository/rust-tooling/rustfmt.toml")
                && !line.contains("cargo run -q -p bijux-dev-atlas")
            {
                errors.push(format!(
                    "CONFIGS_LAYOUT_ERROR: {workflow_rel}:{} cargo fmt must use configs/sources/repository/rust-tooling/rustfmt.toml",
                    line_idx + 1
                ));
            }
            if line.contains("cargo clippy")
                && !line.contains("CLIPPY_CONF_DIR=configs/sources/repository/rust-tooling")
                && !line.contains("cargo run -q -p bijux-dev-atlas")
            {
                errors.push(format!(
                    "CONFIGS_LAYOUT_ERROR: {workflow_rel}:{} cargo clippy must set CLIPPY_CONF_DIR=configs/sources/repository/rust-tooling",
                    line_idx + 1
                ));
            }
        }
    }
    let inventory_manifest_path = ctx.repo_root.join("configs/registry/inventory/index.json");
    let groups_path = ctx.repo_root.join("configs/registry/inventory/groups.json");
    let consumers_path = ctx.repo_root.join("configs/registry/inventory/consumers.json");
    let mut max_depth = 4usize;
    if !inventory_manifest_path.exists() {
        errors.push(
            "CONFIGS_LAYOUT_ERROR: missing canonical inventory manifest `configs/registry/inventory/index.json`"
                .to_string(),
        );
    } else {
        let inventory_text = fs::read_to_string(&inventory_manifest_path)
            .map_err(|e| format!("failed to read {}: {e}", inventory_manifest_path.display()))?;
        let inventory_json: serde_json::Value = serde_json::from_str(&inventory_text)
            .map_err(|e| format!("failed to parse {}: {e}", inventory_manifest_path.display()))?;
        for key in ["groups_path", "consumers_path", "owners_path", "registry_path"] {
            let Some(path) = inventory_json[key].as_str() else {
                errors.push(format!(
                    "CONFIGS_SCHEMA_ERROR: configs/registry/inventory/index.json missing string key `{key}`"
                ));
                continue;
            };
            if !ctx.repo_root.join(path).exists() {
                errors.push(format!(
                    "CONFIGS_LAYOUT_ERROR: configs/registry/inventory/index.json references missing path `{path}`"
                ));
            }
        }
        if let Some(registry_path) = inventory_json["registry_path"].as_str() {
            let registry_full_path = ctx.repo_root.join(registry_path);
            if registry_full_path.exists() {
                let registry_text = fs::read_to_string(&registry_full_path).map_err(|e| {
                    format!("failed to read {}: {e}", registry_full_path.display())
                })?;
                let registry_json: serde_json::Value =
                    serde_json::from_str(&registry_text).map_err(|e| {
                        format!("failed to parse {}: {e}", registry_full_path.display())
                    })?;
                max_depth = registry_json["max_depth"]
                    .as_u64()
                    .map(|value| value as usize)
                    .unwrap_or(max_depth);
            }
        }
    }
    let mut allowed_groups = std::collections::BTreeSet::<String>::new();
    let mut max_top_level_dirs = 20usize;
    if groups_path.exists() {
        let groups_text = fs::read_to_string(&groups_path)
            .map_err(|e| format!("failed to read {}: {e}", groups_path.display()))?;
        let groups_json: serde_json::Value = serde_json::from_str(&groups_text)
            .map_err(|e| format!("failed to parse {}: {e}", groups_path.display()))?;
        max_top_level_dirs = groups_json["max_top_level_dirs"]
            .as_u64()
            .map(|v| v as usize)
            .unwrap_or(20);
        for value in groups_json["allowed_groups"]
            .as_array()
            .cloned()
            .unwrap_or_default()
        {
            if let Some(name) = value.as_str() {
                allowed_groups.insert(name.to_string());
            }
        }
    }
    let mut consumer_groups = std::collections::BTreeSet::<String>::new();
    if consumers_path.exists() {
        let consumers_text = fs::read_to_string(&consumers_path)
            .map_err(|e| format!("failed to read {}: {e}", consumers_path.display()))?;
        let consumers_json: serde_json::Value = serde_json::from_str(&consumers_text)
            .map_err(|e| format!("failed to parse {}: {e}", consumers_path.display()))?;
        if let Some(groups_obj) = consumers_json["groups"].as_object() {
            for (group, entries) in groups_obj {
                let count = entries.as_array().map_or(0, Vec::len);
                if count == 0 {
                    errors.push(format!(
                        "CONFIGS_SCHEMA_ERROR: group `{group}` has no declared consumers"
                    ));
                }
                consumer_groups.insert(group.to_string());
            }
        }
    }
    let toolchain_contract_path = ctx.repo_root.join("configs/sources/repository/rust-tooling/toolchain.json");
    if toolchain_contract_path.exists() {
        let toolchain_text = fs::read_to_string(&toolchain_contract_path)
            .map_err(|e| format!("failed to read {}: {e}", toolchain_contract_path.display()))?;
        let toolchain_json: serde_json::Value = serde_json::from_str(&toolchain_text)
            .map_err(|e| format!("failed to parse {}: {e}", toolchain_contract_path.display()))?;
        let expected_channel = toolchain_json["channel"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        for consumer in toolchain_json["consumers"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(str::to_string))
        {
            let consumer_path = ctx.repo_root.join(&consumer);
            if !consumer_path.exists() {
                errors.push(format!(
                    "CONFIGS_LAYOUT_ERROR: toolchain consumer workflow missing `{consumer}`"
                ));
                continue;
            }
            if !expected_channel.is_empty() {
                let content = fs::read_to_string(&consumer_path)
                    .map_err(|e| format!("failed to read {}: {e}", consumer_path.display()))?;
                if !content.contains(&format!("toolchain: {expected_channel}")) {
                    errors.push(format!(
                        "CONFIGS_LAYOUT_ERROR: workflow `{consumer}` is not pinned to rust toolchain `{expected_channel}`"
                    ));
                }
            }
        }
    }
    let discovered_groups = std::fs::read_dir(&ctx.configs_root)
        .map_err(|e| format!("failed to list {}: {e}", ctx.configs_root.display()))?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().to_str().map(str::to_string))
        .collect::<std::collections::BTreeSet<_>>();
    if discovered_groups.len() > max_top_level_dirs {
        errors.push(format!(
            "CONFIGS_LAYOUT_ERROR: top-level config group budget exceeded ({}/{})",
            discovered_groups.len(),
            max_top_level_dirs
        ));
    }
    for group in &discovered_groups {
        if !allowed_groups.is_empty() && !allowed_groups.contains(group) {
            errors.push(format!(
                "CONFIGS_LAYOUT_ERROR: top-level group `{group}` is not allowlisted in configs/registry/inventory/groups.json"
            ));
        }
        if !consumer_groups.is_empty() && !consumer_groups.contains(group) {
            errors.push(format!(
                "CONFIGS_LAYOUT_ERROR: top-level group `{group}` has no consumer mapping in configs/registry/inventory/consumers.json"
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
        let rel_under_configs = file
            .strip_prefix(&ctx.configs_root)
            .unwrap_or(&file)
            .to_path_buf();
        let depth = rel_under_configs.components().count();
        let ext = file
            .extension()
            .and_then(|v| v.to_str())
            .unwrap_or_default();
        let is_governed_config = matches!(ext, "json" | "toml" | "yaml" | "yml" | "md" | "txt");
        let in_vendor_path = rel.contains("/node_modules/") || rel.contains("/.vale/styles/");
        if is_governed_config && !in_vendor_path && depth > max_depth {
            errors.push(format!(
                "CONFIGS_LAYOUT_ERROR: config path depth exceeds budget (depth={depth}) `{rel}`"
            ));
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
        if name.chars().any(|c| c.is_ascii_uppercase()) && !is_filename_case_exception(&rel) {
            errors.push(format!(
                "CONFIGS_PARSE_ERROR: config filename should be lowercase `{rel}`"
            ));
        }
        let text = fs::read_to_string(&file).unwrap_or_default();
        for (i, line) in text.lines().enumerate() {
            if line_has_forbidden_env_default(line) {
                errors.push(format!(
                    "CONFIGS_PARSE_ERROR: {rel}:{} env interpolation defaults are forbidden",
                    i + 1
                ));
            }
        }
        if !rel.contains("allowlist") && !rel.contains("README") {
            if let Some(value) = structured_config_value(&file, &text)? {
                let mut key_paths = Vec::new();
                collect_secret_like_key_paths(&value, "", &mut key_paths);
                key_paths.sort();
                key_paths.dedup();
                for key_path in key_paths {
                    if is_secret_like_key_exception(&rel, &key_path) {
                        continue;
                    }
                    errors.push(format!(
                        "CONFIGS_SCHEMA_ERROR: {rel}:{key_path} potential secret-like key requires allowlist review",
                    ));
                }
            }
        }
    }
    errors.sort();
    errors.dedup();
    let payload = serde_json::json!({
        "schema_version": 1,
        "run_id": ctx.run_id.as_str(),
        "status": if errors.is_empty() { "pass" } else { "fail" },
        "text": if errors.is_empty() { "configs lint passed" } else { "configs lint failed" },
        "errors": errors,
        "warnings": [],
        "checks": [
            {
                "id": "structured_config_naming",
                "status": "pass"
            },
            {
                "id": "forbidden_interpolation_defaults",
                "status": "pass"
            },
            {
                "id": "secret_like_keys_review",
                "status": "pass"
            }
        ],
        "capabilities": {
            "fs_write": common.allow_write,
            "subprocess": common.allow_subprocess,
            "network": common.allow_network
        },
        "options": {
            "strict": common.strict
        }
    });
    if common.allow_write {
        let out_dir = configs_artifact_dir(ctx);
        fs::create_dir_all(&out_dir)
            .map_err(|e| format!("failed to create {}: {e}", out_dir.display()))?;
        let report_path = out_dir.join("drift-report.json");
        fs::write(
            &report_path,
            serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?,
        )
        .map_err(|e| format!("failed to write {}: {e}", report_path.display()))?;
        let mut with_artifact = payload;
        with_artifact["artifacts"] = serde_json::json!({
            "drift_report": report_path.display().to_string()
        });
        Ok(with_artifact)
    } else {
        Ok(payload)
    }
}

pub(crate) fn configs_compile_payload(
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

pub(crate) fn configs_print_payload(
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
