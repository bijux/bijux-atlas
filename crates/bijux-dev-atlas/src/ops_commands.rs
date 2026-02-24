pub(crate) use crate::ops_command_support::{
    emit_payload, load_profiles, normalize_tool_version_with_regex, resolve_ops_root,
    resolve_profile, run_id_or_default, sha256_hex,
};
use crate::ops_command_support::{
    load_toolchain_inventory_for_ops, ops_pins_check_payload, render_ops_validation_output,
    run_ops_checks, tool_definitions_sorted, verify_tools_snapshot,
};

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
            let toolchain = load_toolchain_inventory_for_ops(&ops_root)
                .map_err(|e| e.to_stable_message())?;
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
        OpsCommand::Inventory(common) => {
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
            let mut summary = ops_inventory_summary(&repo_root)
                .unwrap_or_else(|err| serde_json::json!({"error": format!("OPS_MANIFEST_ERROR: {err}")}));
            if let Some(map) = summary.as_object_mut() {
                map.insert(
                    "inventory_errors".to_string(),
                    serde_json::json!(inventory_errors.clone()),
                );
            }
            let status = if inventory_errors.is_empty() { "ok" } else { "failed" };
            let payload = serde_json::json!({
                "schema_version": 1,
                "status": status,
                "text": format!("ops inventory: status={status}"),
                "rows": [summary],
                "summary": {"total": 1, "errors": inventory_errors.len(), "warnings": 0}
            });
            let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
            Ok((rendered, if inventory_errors.is_empty() { 0 } else { 1 }))
        }
        OpsCommand::Docs(common) => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let selectors = parse_selectors(
                Some("ops".to_string()),
                Some(DomainArg::Ops),
                None,
                None,
                true,
                true,
            )?;
            let request = RunRequest {
                repo_root: repo_root.clone(),
                domain: Some(DomainId::Ops),
                capabilities: Capabilities::deny_all(),
                artifacts_root: Some(repo_root.join("artifacts")),
                run_id: Some(run_id_or_default(common.run_id.clone())?),
                command: Some("bijux dev atlas ops docs".to_string()),
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
        OpsCommand::Conformance(common) => {
            if !common.allow_subprocess {
                return Err(
                    OpsCommandError::Effect("conformance requires --allow-subprocess".to_string())
                        .to_stable_message(),
                );
            }
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
            let status_args = crate::cli::OpsStatusArgs {
                common: common.clone(),
                target: crate::cli::OpsStatusTarget::K8s,
            };
            let (status_rendered, status_code) = crate::ops_runtime_execution::run_ops_status(&status_args)?;
            let errors = inventory_errors.len() + usize::from(status_code != 0);
            let status = if errors == 0 { "ok" } else { "failed" };
            let payload = serde_json::json!({
                "schema_version": 1,
                "status": status,
                "text": format!("ops conformance: status={status}"),
                "rows": [{
                    "inventory_errors": inventory_errors,
                    "status_exit": status_code,
                    "status_output": status_rendered
                }],
                "summary": {"total": 1, "errors": errors, "warnings": 0}
            });
            let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
            Ok((rendered, if status == "ok" { 0 } else { 1 }))
        }
        OpsCommand::Report(common) => {
            if !common.allow_write {
                return Err(
                    OpsCommandError::Effect("report requires --allow-write".to_string())
                        .to_stable_message(),
                );
            }
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root = resolve_ops_root(&repo_root, common.ops_root.clone())
                .map_err(|e| e.to_stable_message())?;
            let run_id = run_id_or_default(common.run_id.clone())?;
            let summary = ops_inventory_summary(&repo_root)
                .unwrap_or_else(|err| serde_json::json!({"error": format!("OPS_MANIFEST_ERROR: {err}")}));
            let inventory_errors =
                match bijux_dev_atlas_core::ops_inventory::OpsInventory::load_and_validate(
                    &ops_root,
                ) {
                    Ok(_) => Vec::new(),
                    Err(err) => vec![err],
                };
            let report = serde_json::json!({
                "schema_version": 1,
                "kind": "ops_report",
                "run_id": run_id.as_str(),
                "repo_root": repo_root.display().to_string(),
                "inventory_summary": summary,
                "inventory_errors": inventory_errors,
                "capabilities": {
                    "fs_write": common.allow_write,
                    "subprocess": common.allow_subprocess
                }
            });
            let out_dir = repo_root.join("artifacts/reports/dev-atlas/ops");
            std::fs::create_dir_all(&out_dir)
                .map_err(|err| format!("failed to create {}: {err}", out_dir.display()))?;
            let out_path = out_dir.join(format!("{}.json", run_id.as_str()));
            std::fs::write(
                &out_path,
                serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?,
            )
            .map_err(|err| format!("failed to write {}: {err}", out_path.display()))?;
            let payload = serde_json::json!({
                "schema_version": 1,
                "status": if report["inventory_errors"].as_array().is_some_and(|v| v.is_empty()) { "ok" } else { "failed" },
                "text": format!("wrote ops report {}", out_path.display()),
                "rows": [{"path": out_path.display().to_string()}],
                "summary": {"total": 1, "errors": report["inventory_errors"].as_array().map_or(1, |v| v.len()), "warnings": 0}
            });
            let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
            let code = if payload["status"] == serde_json::Value::String("ok".to_string()) { 0 } else { 1 };
            Ok((rendered, code))
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
                load_toolchain_inventory_for_ops(&ops_root).map_err(|e| e.to_stable_message())?;
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
                load_toolchain_inventory_for_ops(&ops_root).map_err(|e| e.to_stable_message())?;
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
            OpsGenerateCommand::PinsIndex { check, common } => {
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
                if check {
                    let expected_path = repo_root
                        .join("artifacts/atlas-dev/ops")
                        .join(run_id.as_str())
                        .join(rel);
                    let existing = std::fs::read_to_string(&expected_path).map_err(|err| {
                        format!(
                            "pins-index check failed: missing {}: {err}",
                            expected_path.display()
                        )
                    })?;
                    let expected_json: serde_json::Value =
                        serde_json::from_str(&existing).map_err(|err| {
                            format!(
                                "pins-index check failed: invalid json {}: {err}",
                                expected_path.display()
                            )
                        })?;
                    let matches = expected_json == payload;
                    let text = if matches {
                        format!(
                            "pins index matches existing artifact {}",
                            expected_path.display()
                        )
                    } else {
                        format!(
                            "pins index drift detected for {}",
                            expected_path.display()
                        )
                    };
                    let rendered = emit_payload(
                        common.format,
                        common.out.clone(),
                        &serde_json::json!({"schema_version": 1, "text": text, "rows": [payload], "summary": {"total": 1, "errors": if matches {0} else {1}, "warnings": 0}}),
                    )?;
                    Ok((rendered, if matches { 0 } else { 1 }))
                } else {
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
