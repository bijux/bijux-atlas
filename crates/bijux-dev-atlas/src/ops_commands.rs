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

fn tool_definitions_sorted(inventory: &ToolchainInventory) -> Vec<(String, ToolDefinition)> {
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

fn run_ops_checks(
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

fn verify_tools_snapshot(
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

fn render_ops_validation_output(
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

fn ops_pins_check_payload(
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

pub(crate) fn run_ops_command(quiet: bool, debug: bool, command: OpsCommand) -> i32 {
    let run: Result<(String, i32), String> = (|| match command {
        OpsCommand::Doctor(common) => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root = resolve_ops_root(&repo_root, common.ops_root.clone())
                .map_err(|e| e.to_stable_message())?;
            let inventory_errors =
                match bijux_dev_atlas_core::ops_inventory::OpsInventory::load_and_validate(
                    &ops_root,
                ) {
                    Ok(_) => Vec::new(),
                    Err(err) => vec![err],
                };
            let summary = ops_inventory_summary(&repo_root).unwrap_or_else(
                |err| serde_json::json!({"error": format!("OPS_MANIFEST_ERROR: {err}")}),
            );
            let (checks_rendered, checks_exit) = run_ops_checks(&common, "ops_fast", false, false)?;
            let toolchain =
                load_toolchain_inventory(&ops_root).map_err(|e| e.to_stable_message())?;
            let tools_snapshot = verify_tools_snapshot(common.allow_subprocess, &toolchain)?;
            let mut inventory_errors = inventory_errors;
            if tools_snapshot
                .get("missing_required")
                .and_then(|v| v.as_array())
                .is_some_and(|v| !v.is_empty())
            {
                inventory_errors.push("required external tools are missing".to_string());
            }
            let summary = serde_json::json!({
                "inventory": summary,
                "tools": tools_snapshot
            });
            render_ops_validation_output(
                &common,
                "doctor",
                &inventory_errors,
                &checks_rendered,
                checks_exit,
                summary,
            )
        }
        OpsCommand::Validate(common) => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root = resolve_ops_root(&repo_root, common.ops_root.clone())
                .map_err(|e| e.to_stable_message())?;
            let inventory_errors =
                match bijux_dev_atlas_core::ops_inventory::OpsInventory::load_and_validate(
                    &ops_root,
                ) {
                    Ok(_) => Vec::new(),
                    Err(err) => vec![err],
                };
            let summary = ops_inventory_summary(&repo_root).unwrap_or_else(
                |err| serde_json::json!({"error": format!("OPS_MANIFEST_ERROR: {err}")}),
            );
            let (checks_rendered, checks_exit) = run_ops_checks(&common, "ops_all", true, true)?;
            render_ops_validation_output(
                &common,
                "validate",
                &inventory_errors,
                &checks_rendered,
                checks_exit,
                summary,
            )
        }
        OpsCommand::Render(args) => crate::ops_runtime_execution::run_ops_render(&args),
        OpsCommand::Install(args) => crate::ops_runtime_execution::run_ops_install(&args),
        OpsCommand::Status(args) => crate::ops_runtime_execution::run_ops_status(&args),
        OpsCommand::ListProfiles(common) => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root = resolve_ops_root(&repo_root, common.ops_root.clone())
                .map_err(|e| e.to_stable_message())?;
            let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
            profiles.sort_by(|a, b| a.name.cmp(&b.name));
            let rows = profiles
                .iter()
                .map(|p| serde_json::json!({"name": p.name, "kind_profile": p.kind_profile, "cluster_config": p.cluster_config}))
                .collect::<Vec<_>>();
            let text = profiles
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>()
                .join("\n");
            let envelope = serde_json::json!({"schema_version": 1, "text": text, "rows": rows, "summary": {"total": profiles.len(), "errors": 0, "warnings": 0}});
            let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
            Ok((rendered, 0))
        }
        OpsCommand::ExplainProfile { name, common } => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root = resolve_ops_root(&repo_root, common.ops_root.clone())
                .map_err(|e| e.to_stable_message())?;
            let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
            profiles.sort_by(|a, b| a.name.cmp(&b.name));
            let profile =
                resolve_profile(Some(name), &profiles).map_err(|e| e.to_stable_message())?;
            let text = format!(
                "profile={} kind_profile={} cluster_config={}",
                profile.name, profile.kind_profile, profile.cluster_config
            );
            let envelope = serde_json::json!({"schema_version": 1, "text": text, "rows": [profile], "summary": {"total": 1, "errors": 0, "warnings": 0}});
            let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
            Ok((rendered, 0))
        }
        OpsCommand::ListTools(common) => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root = resolve_ops_root(&repo_root, common.ops_root.clone())
                .map_err(|e| e.to_stable_message())?;
            let inventory =
                load_toolchain_inventory(&ops_root).map_err(|e| e.to_stable_message())?;
            let process = OpsProcess::new(common.allow_subprocess);
            let mut rows = Vec::new();
            for (name, definition) in tool_definitions_sorted(&inventory) {
                let mut row = process
                    .probe_tool(&name, &definition.probe_argv, &definition.version_regex)
                    .map_err(|e| e.to_stable_message())?;
                row["required"] = serde_json::Value::Bool(definition.required);
                rows.push(row);
            }
            rows.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
            let text = rows
                .iter()
                .map(|r| {
                    format!(
                        "{} required={} installed={}",
                        r["name"].as_str().unwrap_or(""),
                        r["required"],
                        r["installed"]
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            let envelope = serde_json::json!({"schema_version": 1, "text": text, "rows": rows, "summary": {"total": rows.len(), "errors": 0, "warnings": 0}});
            let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
            Ok((rendered, 0))
        }
        OpsCommand::VerifyTools(common) => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root = resolve_ops_root(&repo_root, common.ops_root.clone())
                .map_err(|e| e.to_stable_message())?;
            let inventory =
                load_toolchain_inventory(&ops_root).map_err(|e| e.to_stable_message())?;
            let process = OpsProcess::new(common.allow_subprocess);
            let mut rows = Vec::new();
            let mut missing = Vec::new();
            let mut warnings = Vec::new();
            for (name, definition) in tool_definitions_sorted(&inventory) {
                let row = process
                    .probe_tool(&name, &definition.probe_argv, &definition.version_regex)
                    .map_err(|e| e.to_stable_message())?;
                if row["installed"] == serde_json::Value::Bool(false) {
                    if definition.required {
                        missing.push(name.clone());
                    } else {
                        warnings.push(name.clone());
                    }
                }
                rows.push(row);
            }
            rows.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
            let text = if missing.is_empty() {
                "all required ops tools are installed".to_string()
            } else {
                format!("missing required ops tools: {}", missing.join(", "))
            };
            let envelope = serde_json::json!({"schema_version": 1, "text": text, "rows": rows, "missing": missing, "warnings": warnings, "summary": {"total": rows.len(), "errors": missing.len(), "warnings": warnings.len()}});
            let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
            let has_errors = !envelope["missing"]
                .as_array()
                .map(|v| v.is_empty())
                .unwrap_or(true);
            let has_warnings = !envelope["warnings"]
                .as_array()
                .map(|v| v.is_empty())
                .unwrap_or(true);
            let code = if has_errors || (common.strict && has_warnings) {
                1
            } else {
                0
            };
            Ok((rendered, code))
        }
        OpsCommand::ListActions(common) => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root = resolve_ops_root(&repo_root, common.ops_root.clone())
                .map_err(|e| e.to_stable_message())?;
            let fs_adapter = OpsFs::new(repo_root, ops_root);
            let mut payload: SurfacesInventory = fs_adapter
                .read_ops_json("inventory/surfaces.json")
                .map_err(|e| e.to_stable_message())?;
            payload.actions.sort_by(|a, b| a.id.cmp(&b.id));
            let rows = payload.actions.iter()
                .map(|a| serde_json::json!({"id": a.id, "domain": a.domain, "command": a.command, "argv": a.argv}))
                .collect::<Vec<_>>();
            let text = payload
                .actions
                .iter()
                .map(|a| a.id.clone())
                .collect::<Vec<_>>()
                .join("\n");
            let envelope = serde_json::json!({"schema_version": 1, "text": text, "rows": rows, "summary": {"total": payload.actions.len(), "errors": 0, "warnings": 0}});
            let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
            Ok((rendered, 0))
        }
        OpsCommand::Up(common) => {
            if !common.allow_subprocess {
                return Err(
                    OpsCommandError::Effect("up requires --allow-subprocess".to_string())
                        .to_stable_message(),
                );
            }
            if !common.allow_write {
                return Err(
                    OpsCommandError::Effect("up requires --allow-write".to_string())
                        .to_stable_message(),
                );
            }
            let text = "ops up delegates to install --kind --apply --plan".to_string();
            let rendered = emit_payload(
                common.format,
                common.out.clone(),
                &serde_json::json!({"schema_version": 1, "text": text, "rows": [], "summary": {"total": 0, "errors": 0, "warnings": 0}}),
            )?;
            Ok((rendered, 0))
        }
        OpsCommand::Down(common) => {
            if !common.allow_subprocess {
                return Err(OpsCommandError::Effect(
                    "down requires --allow-subprocess".to_string(),
                )
                .to_stable_message());
            }
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root = resolve_ops_root(&repo_root, common.ops_root.clone())
                .map_err(|e| e.to_stable_message())?;
            let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
            profiles.sort_by(|a, b| a.name.cmp(&b.name));
            let profile = resolve_profile(common.profile.clone(), &profiles)
                .map_err(|e| e.to_stable_message())?;
            let process = OpsProcess::new(common.allow_subprocess);
            let args = vec![
                "delete".to_string(),
                "cluster".to_string(),
                "--name".to_string(),
                profile.kind_profile.clone(),
            ];
            let _ = process
                .run_subprocess("kind", &args, &repo_root)
                .map_err(|e| e.to_stable_message())?;
            let text = format!("ops down deleted kind cluster `{}`", profile.kind_profile);
            let rendered = emit_payload(
                common.format,
                common.out.clone(),
                &serde_json::json!({"schema_version": 1, "text": text, "rows": [], "summary": {"total": 0, "errors": 0, "warnings": 0}}),
            )?;
            Ok((rendered, 0))
        }
        OpsCommand::Clean(common) => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let path = repo_root.join("artifacts/atlas-dev/ops");
            if path.exists() {
                std::fs::remove_dir_all(&path)
                    .map_err(|err| format!("failed to remove {}: {err}", path.display()))?;
            }
            let text = format!("cleaned {}", path.display());
            let rendered = emit_payload(
                common.format,
                common.out.clone(),
                &serde_json::json!({"schema_version": 1, "text": text, "rows": [], "summary": {"total": 0, "errors": 0, "warnings": 0}}),
            )?;
            Ok((rendered, 0))
        }
        OpsCommand::Reset(args) => {
            let common = &args.common;
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let run_id = RunId::parse(&args.reset_id).map_err(|err| err.to_string())?;
            let target = repo_root
                .join("artifacts/atlas-dev/ops")
                .join(run_id.as_str());
            if !target.starts_with(repo_root.join("artifacts/atlas-dev/ops")) {
                return Err("reset path guard failed".to_string());
            }
            if target.exists() {
                std::fs::remove_dir_all(&target)
                    .map_err(|err| format!("failed to remove {}: {err}", target.display()))?;
            }
            let text = format!(
                "reset artifacts for run_id={} at {}",
                run_id.as_str(),
                target.display()
            );
            let rendered = emit_payload(
                common.format,
                common.out.clone(),
                &serde_json::json!({"schema_version": 1, "text": text, "rows": [], "summary": {"total": 1, "errors": 0, "warnings": 0}}),
            )?;
            Ok((rendered, 0))
        }
        OpsCommand::Pins { command } => match command {
            OpsPinsCommand::Check(common) => {
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let (payload, code) = ops_pins_check_payload(&common, &repo_root)?;
                let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
                Ok((rendered, code))
            }
            OpsPinsCommand::Update {
                i_know_what_im_doing,
                common,
            } => {
                if !i_know_what_im_doing {
                    Err("ops pins update requires --i-know-what-im-doing".to_string())
                } else if !common.allow_subprocess {
                    Err(OpsCommandError::Effect(
                        "pins update requires --allow-subprocess".to_string(),
                    )
                    .to_stable_message())
                } else if !common.allow_write {
                    Err(
                        OpsCommandError::Effect("pins update requires --allow-write".to_string())
                            .to_stable_message(),
                    )
                } else {
                    let repo_root = resolve_repo_root(common.repo_root.clone())?;
                    let target = repo_root.join("ops/inventory/pins.yaml");
                    let text =
                        "ops pins update is migration-gated; no mutation performed".to_string();
                    let rendered = emit_payload(
                        common.format,
                        common.out.clone(),
                        &serde_json::json!({"schema_version": 1, "text": text, "rows": [{"target_path": target.display().to_string()}], "summary": {"total": 1, "errors": 0, "warnings": 0}}),
                    )?;
                    Ok((rendered, 0))
                }
            }
        },
        OpsCommand::Generate { command } => match command {
            OpsGenerateCommand::PinsIndex(common) => {
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let run_id = run_id_or_default(common.run_id.clone())?;
                let fs_adapter = OpsFs::new(repo_root.clone(), repo_root.join("ops"));
                let pins_rel = "ops/inventory/pins.yaml";
                let toolchain_rel = "ops/inventory/toolchain.json";
                let stack_rel = "ops/stack/version-manifest.json";
                let pins_raw = fs::read_to_string(repo_root.join(pins_rel))
                    .map_err(|err| format!("failed to read {pins_rel}: {err}"))?;
                let toolchain_raw = fs::read_to_string(repo_root.join(toolchain_rel))
                    .map_err(|err| format!("failed to read {toolchain_rel}: {err}"))?;
                let stack_raw = fs::read_to_string(repo_root.join(stack_rel))
                    .map_err(|err| format!("failed to read {stack_rel}: {err}"))?;
                let mut files = vec![
                    serde_json::json!({"path": pins_rel, "sha256": sha256_hex(&pins_raw), "bytes": pins_raw.len()}),
                    serde_json::json!({"path": stack_rel, "sha256": sha256_hex(&stack_raw), "bytes": stack_raw.len()}),
                    serde_json::json!({"path": toolchain_rel, "sha256": sha256_hex(&toolchain_raw), "bytes": toolchain_raw.len()}),
                ];
                files.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "run_id": run_id.as_str(),
                    "generator": "ops generate pins-index",
                    "files": files
                });
                let rel = "generate/pins.index.json";
                let out = fs_adapter
                    .write_artifact_json(&run_id, rel, &payload)
                    .map_err(|e| e.to_stable_message())?;
                let text = format!("generated deterministic pins index at {}", out.display());
                let rendered = emit_payload(
                    common.format,
                    common.out.clone(),
                    &serde_json::json!({"schema_version": 1, "text": text, "rows": [payload], "summary": {"total": 1, "errors": 0, "warnings": 0}}),
                )?;
                Ok((rendered, 0))
            }
        },
    })();

    if debug {
        eprintln!(
            "{}",
            serde_json::json!({
                "event": "ops.command.completed",
                "ok": run.is_ok(),
            })
        );
    }

    match run {
        Ok((rendered, code)) => {
            if !quiet && !rendered.is_empty() {
                println!("{rendered}");
            }
            code
        }
        Err(err) => {
            eprintln!("bijux-dev-atlas ops failed: {err}");
            1
        }
    }
}
use crate::*;
