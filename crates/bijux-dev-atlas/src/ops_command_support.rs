use crate::*;

pub(crate) fn normalize_tool_version_with_regex(raw: &str, pattern: &str) -> Option<String> {
    let re = Regex::new(pattern).ok()?;
    re.captures(raw)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
}

pub(crate) fn resolve_ops_root(
    repo_root: &Path,
    ops_root: Option<PathBuf>,
) -> Result<PathBuf, OpsCommandError> {
    let path = ops_root.unwrap_or_else(|| repo_root.join("ops"));
    path.canonicalize().map_err(|err| {
        OpsCommandError::Manifest(format!("cannot resolve ops root {}: {err}", path.display()))
    })
}

pub(crate) fn load_profiles(ops_root: &Path) -> Result<Vec<StackProfile>, OpsCommandError> {
    let path = ops_root.join("stack/profiles.json");
    let text = std::fs::read_to_string(&path).map_err(|err| {
        OpsCommandError::Manifest(format!("failed to read {}: {err}", path.display()))
    })?;
    let payload: StackProfiles = serde_json::from_str(&text).map_err(|err| {
        OpsCommandError::Schema(format!("failed to parse {}: {err}", path.display()))
    })?;
    Ok(payload.profiles)
}

fn load_toolchain_inventory(ops_root: &Path) -> Result<ToolchainInventory, OpsCommandError> {
    let path = ops_root.join("inventory/toolchain.json");
    let text = std::fs::read_to_string(&path).map_err(|err| {
        OpsCommandError::Manifest(format!("failed to read {}: {err}", path.display()))
    })?;
    serde_json::from_str(&text).map_err(|err| {
        OpsCommandError::Schema(format!("failed to parse {}: {err}", path.display()))
    })
}

pub(crate) fn tool_definitions_sorted(
    inventory: &ToolchainInventory,
) -> Vec<(String, ToolDefinition)> {
    inventory
        .tools
        .iter()
        .map(|(name, definition)| (name.clone(), definition.clone()))
        .collect()
}

pub(crate) fn resolve_profile(
    requested: Option<String>,
    profiles: &[StackProfile],
) -> Result<StackProfile, OpsCommandError> {
    if profiles.is_empty() {
        return Err(OpsCommandError::Profile(
            "no profiles declared in ops/stack/profiles.json".to_string(),
        ));
    }
    if let Some(name) = requested {
        return profiles
            .iter()
            .find(|p| p.name == name)
            .cloned()
            .ok_or_else(|| OpsCommandError::Profile(format!("unknown profile `{name}`")));
    }
    profiles
        .iter()
        .find(|p| p.name == "developer")
        .cloned()
        .or_else(|| profiles.first().cloned())
        .ok_or_else(|| OpsCommandError::Profile("no default profile available".to_string()))
}

pub(crate) fn run_id_or_default(raw: Option<String>) -> Result<RunId, String> {
    raw.map(|v| RunId::parse(&v))
        .transpose()?
        .map_or_else(|| Ok(RunId::from_seed("ops_run")), Ok)
}

pub(crate) fn emit_payload(
    format: FormatArg,
    out: Option<PathBuf>,
    payload: &serde_json::Value,
) -> Result<String, String> {
    let rendered = match format {
        FormatArg::Text => payload
            .get("text")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string())
            .unwrap_or_else(|| serde_json::to_string_pretty(payload).unwrap_or_default()),
        FormatArg::Json => serde_json::to_string_pretty(payload).map_err(|err| err.to_string())?,
        FormatArg::Jsonl => {
            if let Some(rows) = payload.get("rows").and_then(|v| v.as_array()) {
                rows.iter()
                    .map(serde_json::to_string)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|err| err.to_string())?
                    .join("\n")
            } else {
                serde_json::to_string(payload).map_err(|err| err.to_string())?
            }
        }
    };
    write_output_if_requested(out, &rendered)?;
    Ok(rendered)
}

pub(crate) fn sha256_hex(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub(crate) fn run_ops_checks(
    common: &OpsCommonArgs,
    suite: &str,
    include_internal: bool,
    include_slow: bool,
) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let selectors = parse_selectors(
        Some(suite.to_string()),
        Some(DomainArg::Ops),
        None,
        None,
        include_internal,
        include_slow,
    )?;
    let request = RunRequest {
        repo_root: repo_root.clone(),
        domain: Some(DomainId::Ops),
        capabilities: Capabilities::deny_all(),
        artifacts_root: Some(repo_root.join("artifacts")),
        run_id: Some(run_id_or_default(common.run_id.clone())?),
        command: Some(format!("bijux dev atlas ops {suite}")),
    };
    let report = run_checks(
        &RealProcessRunner,
        &RealFs,
        &request,
        &selectors,
        &RunOptions::default(),
    )?;
    let rendered = match common.format {
        FormatArg::Text => render_text_with_durations(&report, 10),
        FormatArg::Json => render_json(&report)?,
        FormatArg::Jsonl => render_jsonl(&report)?,
    };
    write_output_if_requested(common.out.clone(), &rendered)?;
    Ok((rendered, exit_code_for_report(&report)))
}

pub(crate) fn verify_tools_snapshot(
    allow_subprocess: bool,
    inventory: &ToolchainInventory,
) -> Result<serde_json::Value, String> {
    if !allow_subprocess {
        return Ok(serde_json::json!({
            "enabled": false,
            "text": "tool verification skipped (pass --allow-subprocess)",
            "missing_required": [],
            "rows": []
        }));
    }
    let process = OpsProcess::new(true);
    let mut rows = Vec::new();
    let mut missing_required = Vec::new();
    for (name, definition) in tool_definitions_sorted(inventory) {
        let mut row = process
            .probe_tool(&name, &definition.probe_argv, &definition.version_regex)
            .map_err(|e| e.to_stable_message())?;
        row["required"] = serde_json::Value::Bool(definition.required);
        if definition.required && row["installed"] != serde_json::Value::Bool(true) {
            missing_required.push(name.clone());
        }
        rows.push(row);
    }
    rows.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    Ok(serde_json::json!({
        "enabled": true,
        "text": if missing_required.is_empty() { "all required tools available" } else { "missing required tools" },
        "missing_required": missing_required,
        "rows": rows
    }))
}

pub(crate) fn render_ops_validation_output(
    common: &OpsCommonArgs,
    mode: &str,
    inventory_errors: &[String],
    checks_rendered: &str,
    checks_exit: i32,
    summary: serde_json::Value,
) -> Result<(String, i32), String> {
    let inventory_error_count = inventory_errors.len();
    let checks_error_count = if checks_exit == 0 { 0 } else { 1 };
    let error_count = inventory_error_count + checks_error_count;
    let status = if error_count == 0 { "ok" } else { "failed" };
    let strict_failed = common.strict && error_count > 0;
    let exit = if strict_failed || checks_exit != 0 || inventory_error_count > 0 {
        1
    } else {
        0
    };
    let text = format!(
        "ops {mode}: status={status} inventory_errors={inventory_error_count} checks_exit={checks_exit}"
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "mode": mode,
        "status": status,
        "text": text,
        "rows": [{
            "inventory_errors": inventory_errors,
            "checks_exit": checks_exit,
            "checks_output": checks_rendered,
            "inventory_summary": summary
        }],
        "summary": {
            "total": 1,
            "errors": error_count,
            "warnings": 0
        }
    });
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, exit))
}

pub(crate) fn ops_pins_check_payload(
    common: &OpsCommonArgs,
    repo_root: &Path,
) -> Result<(serde_json::Value, i32), String> {
    let ops_root =
        resolve_ops_root(repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut errors = Vec::new();
    if let Err(err) =
        bijux_dev_atlas_core::ops_inventory::OpsInventory::load_and_validate(&ops_root)
    {
        errors.push(err);
    }
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": status,
        "text": if errors.is_empty() { "ops pins check passed" } else { "ops pins check failed" },
        "rows": [{
            "pins_path": "ops/inventory/pins.yaml",
            "errors": errors
        }],
        "summary": {"total": 1, "errors": if status == "ok" {0} else {1}, "warnings": 0}
    });
    Ok((payload, if status == "ok" { 0 } else { 1 }))
}

pub(crate) fn load_toolchain_inventory_for_ops(
    ops_root: &Path,
) -> Result<ToolchainInventory, OpsCommandError> {
    load_toolchain_inventory(ops_root)
}
