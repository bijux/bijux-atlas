// SPDX-License-Identifier: Apache-2.0

use super::*;
pub(super) fn dispatch_core(command: OpsCommand, debug: bool) -> Result<(String, i32), String> {
    let _ = debug;
    match command {
        OpsCommand::List(common) => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root = resolve_ops_root(&repo_root, common.ops_root.clone())
                .map_err(|e| e.to_stable_message())?;
            let profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
            let actions: SurfacesInventory = OpsFs::new(repo_root.clone(), ops_root.clone())
                .read_ops_json("inventory/surfaces.json")
                .map_err(|e| e.to_stable_message())?;
            let mut action_ids = actions
                .actions
                .iter()
                .map(|a| a.id.clone())
                .collect::<Vec<_>>();
            action_ids.sort();
            let rows = vec![
                serde_json::json!({"kind":"capability","name":"inventory","subprocess":false,"write":false}),
                serde_json::json!({"kind":"capability","name":"validate","subprocess":false,"write":false}),
                serde_json::json!({"kind":"capability","name":"render","subprocess":true,"write":"flag_gated"}),
                serde_json::json!({"kind":"capability","name":"install","subprocess":true,"write":"flag_gated"}),
                serde_json::json!({"kind":"capability","name":"status","subprocess":"target_gated","write":false}),
                serde_json::json!({"kind":"capability","name":"cleanup","subprocess":"profile_dependent","write":false}),
                serde_json::json!({"kind":"profiles","count": profiles.len()}),
                serde_json::json!({"kind":"actions","count": action_ids.len(), "action_ids": action_ids}),
            ];
            let payload = serde_json::json!({
                "schema_version": 1,
                "text": "ops list capabilities and actions",
                "rows": rows,
                "summary": {"total": 8, "errors": 0, "warnings": 0}
            });
            let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
            Ok((rendered, ops_exit::PASS))
        }
        OpsCommand::Explain { action, common } => {
            let action_lc = action.trim().to_ascii_lowercase();
            let row = match action_lc.as_str() {
                "inventory" => serde_json::json!({"action":"inventory","purpose":"list ops manifests and inventory validity","effects_required":[]}),
                "validate" => serde_json::json!({"action":"validate","purpose":"validate ops SSOT inputs and checks","effects_required":[]}),
                "render" | "k8s-render" => serde_json::json!({"action":"render","purpose":"render deterministic ops manifests","effects_required":["subprocess"],"flags":["--allow-subprocess","--allow-write"]}),
                "k8s-plan" => serde_json::json!({"action":"k8s-plan","purpose":"show what rendered resources would be applied","effects_required":[]}),
                "stack-plan" => serde_json::json!({"action":"stack-plan","purpose":"resolve stack resources for a profile without executing subprocesses","effects_required":[]}),
                "install" | "stack-up" => serde_json::json!({"action":"install","purpose":"plan/apply ops stack to local cluster","effects_required":["subprocess","fs_write","network"],"flags":["--allow-subprocess","--allow-write","--allow-network"]}),
                "down" | "stack-down" => serde_json::json!({"action":"down","purpose":"teardown local ops stack resources","effects_required":["subprocess"],"flags":["--allow-subprocess"]}),
                "status" | "stack-status" => serde_json::json!({"action":"status","purpose":"collect local/k8s status rows","effects_required":["subprocess (for k8s/pods/endpoints)"]}),
                "conformance" | "k8s-test" => serde_json::json!({"action":"conformance","purpose":"run ops conformance status checks","effects_required":["subprocess"],"flags":["--allow-subprocess"]}),
                "load-plan" => serde_json::json!({"action":"load-plan","purpose":"resolve load suite to script env and thresholds","effects_required":[]}),
                "load-run" => serde_json::json!({"action":"load-run","purpose":"run k6 load suite and collect summary","effects_required":["subprocess","network","fs_write"]}),
                "load-report" => serde_json::json!({"action":"load-report","purpose":"parse k6 summary into structured report","effects_required":[]}),
                "e2e-run" => serde_json::json!({"action":"e2e-run","purpose":"reserved for scenario orchestration","status":"not_implemented"}),
                "obs-drill-run" => serde_json::json!({"action":"obs-drill-run","purpose":"reserved for observability drill orchestration","status":"not_implemented"}),
                "obs-verify" => serde_json::json!({"action":"obs-verify","purpose":"verify observability contracts","effects_required":[]}),
                "tools-doctor" => serde_json::json!({"action":"tools-doctor","purpose":"show required tools and missing requirements without subprocess by default","effects_required":[]}),
                "suite-list" => serde_json::json!({"kind":"suite","action":"list","suites":["e2e","k8s","load","obs"]}),
                value if value.starts_with("suite-run:") => serde_json::json!({"kind":"suite","action":"run","suite":value.trim_start_matches("suite-run:")}),
                "cleanup" => serde_json::json!({"action":"cleanup","purpose":"remove scoped artifacts and local ops resources","effects_required":["subprocess (optional)"]}),
                _ => {
                    return Err(format!(
                        "unknown ops action `{}` (try inventory|validate|render|install|down|status|conformance|cleanup|load-plan|load-run|load-report|e2e-run|obs-drill-run)",
                        action
                    ))
                }
            };
            let payload = serde_json::json!({
                "schema_version": 1,
                "text": format!("ops explain {}", action_lc),
                "rows": [row],
                "summary": {"total": 1, "errors": 0, "warnings": 0}
            });
            let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
            Ok((rendered, ops_exit::PASS))
        }
        OpsCommand::Doctor(common) => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root = resolve_ops_root(&repo_root, common.ops_root.clone())
                .map_err(|e| e.to_stable_message())?;
            let mut inventory_errors =
                match bijux_dev_atlas::core::ops_inventory::OpsInventory::load_and_validate(
                    &ops_root,
                ) {
                    Ok(_) => Vec::new(),
                    Err(err) => vec![err],
                };
            if let Ok(pins) = load_stack_pins(&repo_root) {
                if let Ok(pin_errors) = validate_pins_completeness(&repo_root, &pins) {
                    inventory_errors.extend(pin_errors);
                }
            }
            if let Ok(stack_manifest) = load_stack_manifest(&repo_root) {
                if let Ok(stack_errors) = validate_stack_manifest(&repo_root, &stack_manifest) {
                    inventory_errors.extend(stack_errors);
                }
            }
            if let Ok(load_manifest) = load_load_manifest(&repo_root) {
                if let Ok(load_errors) = validate_load_manifest(&repo_root, &load_manifest) {
                    inventory_errors.extend(load_errors);
                }
            }
            let summary = ops_inventory_summary(&repo_root).unwrap_or_else(
                |err| serde_json::json!({"error": format!("OPS_MANIFEST_ERROR: {err}")}),
            );
            let (checks_rendered, checks_exit) = run_ops_checks(&common, "ops_fast", false, false)?;
            let toolchain =
                load_toolchain_inventory_for_ops(&ops_root).map_err(|e| e.to_stable_message())?;
            let tools_snapshot = verify_tools_snapshot(common.allow_subprocess, &toolchain)?;
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
            let mut inventory_errors =
                match bijux_dev_atlas::core::ops_inventory::OpsInventory::load_and_validate(
                    &ops_root,
                ) {
                    Ok(_) => Vec::new(),
                    Err(err) => vec![err],
                };
            let pins = load_stack_pins(&repo_root).map_err(|e| e.to_stable_message())?;
            inventory_errors.extend(
                validate_pins_completeness(&repo_root, &pins).map_err(|e| e.to_stable_message())?,
            );
            let stack_manifest =
                load_stack_manifest(&repo_root).map_err(|e| e.to_stable_message())?;
            inventory_errors.extend(
                validate_stack_manifest(&repo_root, &stack_manifest)
                    .map_err(|e| e.to_stable_message())?,
            );
            let load_manifest =
                load_load_manifest(&repo_root).map_err(|e| e.to_stable_message())?;
            inventory_errors.extend(
                validate_load_manifest(&repo_root, &load_manifest)
                    .map_err(|e| e.to_stable_message())?,
            );
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
            let profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
            let surfaces: SurfacesInventory = OpsFs::new(repo_root.clone(), ops_root.clone())
                .read_ops_json("inventory/surfaces.json")
                .map_err(|e| e.to_stable_message())?;
            let toolchain =
                load_toolchain_inventory_for_ops(&ops_root).map_err(|e| e.to_stable_message())?;
            let inventory_errors =
                match bijux_dev_atlas::core::ops_inventory::OpsInventory::load_and_validate(
                    &ops_root,
                ) {
                    Ok(_) => Vec::new(),
                    Err(err) => vec![err],
                };
            let mut summary = ops_inventory_summary(&repo_root).unwrap_or_else(
                |err| serde_json::json!({"error": format!("OPS_MANIFEST_ERROR: {err}")}),
            );
            let toolchain_images = summary
                .get("toolchain_images")
                .cloned()
                .unwrap_or(serde_json::json!(0));
            if let Some(map) = summary.as_object_mut() {
                map.insert(
                    "inventory_errors".to_string(),
                    serde_json::json!(inventory_errors.clone()),
                );
                map.insert("profiles".to_string(), serde_json::json!(profiles));
                map.insert("components".to_string(), toolchain_images);
                map.insert(
                    "charts".to_string(),
                    serde_json::json!(surfaces
                        .actions
                        .iter()
                        .filter(|a| a.id.contains("render"))
                        .count()),
                );
                map.insert(
                    "tools".to_string(),
                    serde_json::json!(toolchain.tools.keys().cloned().collect::<Vec<_>>()),
                );
                map.insert(
                    "suites".to_string(),
                    serde_json::json!(["load", "e2e", "k8s", "obs"]),
                );
                map.insert(
                    "scenarios".to_string(),
                    serde_json::json!(["load.run", "e2e.run", "obs.drill.run", "obs.verify"]),
                );
                map.insert(
                    "schemas".to_string(),
                    serde_json::json!([
                        "ops/stack/stack.toml",
                        "ops/stack/profiles.json",
                        "ops/stack/generated/version-manifest.json",
                        "ops/inventory/toolchain.json",
                        "ops/inventory/surfaces.json",
                        "ops/inventory/contracts.json"
                    ]),
                );
            }
            let status = if inventory_errors.is_empty() {
                "ok"
            } else {
                "failed"
            };
            let payload = serde_json::json!({
                "schema_version": 1,
                "status": status,
                "text": format!("ops inventory: status={status}"),
                "rows": [summary],
                "summary": {"total": 1, "errors": inventory_errors.len(), "warnings": 0}
            });
            let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
            Ok((
                rendered,
                if inventory_errors.is_empty() {
                    ops_exit::PASS
                } else {
                    ops_exit::FAIL
                },
            ))
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
                return Err(OpsCommandError::Effect(
                    "conformance requires --allow-subprocess".to_string(),
                )
                .to_stable_message());
            }
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root = resolve_ops_root(&repo_root, common.ops_root.clone())
                .map_err(|e| e.to_stable_message())?;
            let inventory_errors =
                match bijux_dev_atlas::core::ops_inventory::OpsInventory::load_and_validate(
                    &ops_root,
                ) {
                    Ok(_) => Vec::new(),
                    Err(err) => vec![err],
                };
            let status_args = crate::cli::OpsStatusArgs {
                common: common.clone(),
                target: crate::cli::OpsStatusTarget::K8s,
            };
            let (status_rendered, status_code) =
                crate::ops_execution_runtime::run_ops_status(&status_args)?;
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
            Ok((
                rendered,
                if status == "ok" {
                    ops_exit::PASS
                } else {
                    ops_exit::FAIL
                },
            ))
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
            let summary = ops_inventory_summary(&repo_root).unwrap_or_else(
                |err| serde_json::json!({"error": format!("OPS_MANIFEST_ERROR: {err}")}),
            );
            let inventory_errors =
                match bijux_dev_atlas::core::ops_inventory::OpsInventory::load_and_validate(
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
            let code = if payload["status"] == serde_json::Value::String("ok".to_string()) {
                0
            } else {
                1
            };
            Ok((rendered, code))
        }
        OpsCommand::Render(args) => crate::ops_execution_runtime::run_ops_render(&args),
        OpsCommand::Install(args) => crate::ops_execution_runtime::run_ops_install(&args),
        OpsCommand::Status(args) => crate::ops_execution_runtime::run_ops_status(&args),
        OpsCommand::K8sPlan(common) => crate::ops_execution_runtime::run_ops_k8s_plan(&common),
        OpsCommand::K8sApply(args) => crate::ops_execution_runtime::run_ops_k8s_apply(&args, false),
        OpsCommand::K8sDryRun(common) => {
            let args = crate::cli::OpsK8sApplyArgs {
                common: common.clone(),
                apply: true,
            };
            crate::ops_execution_runtime::run_ops_k8s_apply(&args, true)
        }
        OpsCommand::K8sConformance(common) => {
            crate::ops_execution_runtime::run_ops_k8s_conformance(&common)
        }
        OpsCommand::K8sWait(args) => crate::ops_execution_runtime::run_ops_k8s_wait(&args),
        OpsCommand::K8sLogs(args) => crate::ops_execution_runtime::run_ops_k8s_logs(&args),
        OpsCommand::K8sPortForward(args) => {
            crate::ops_execution_runtime::run_ops_k8s_port_forward(&args)
        }
        OpsCommand::LoadPlan { suite, common } => {
            crate::ops_execution_runtime::run_ops_load_plan(&common, &suite)
        }
        OpsCommand::LoadRun { suite, common } => {
            crate::ops_execution_runtime::run_ops_load_run(&common, &suite)
        }
        OpsCommand::LoadReport { suite, common } => {
            crate::ops_execution_runtime::run_ops_load_report(&common, &suite, None)
        }
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
        OpsCommand::Tools { command } => match command {
            OpsToolsCommand::List(common) => {
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let tools = load_tools_manifest(&repo_root).map_err(|e| e.to_stable_message())?;
                let mut rows = tools
                    .tools
                    .iter()
                    .map(|t| {
                        serde_json::json!({
                            "name": t.name,
                            "required": t.required,
                            "version_regex": t.version_regex,
                            "probe_argv": t.probe_argv
                        })
                    })
                    .collect::<Vec<_>>();
                rows.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
                let text = rows
                    .iter()
                    .map(|r| {
                        format!(
                            "{} required={}",
                            r["name"].as_str().unwrap_or(""),
                            r["required"]
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                let payload = serde_json::json!({"schema_version":1,"text":text,"rows":rows,"summary":{"total":rows.len(),"errors":0,"warnings":0}});
                let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
                Ok((rendered, ops_exit::PASS))
            }
            OpsToolsCommand::Verify(common) => {
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let tools = load_tools_manifest(&repo_root).map_err(|e| e.to_stable_message())?;
                let overrides = parse_tool_overrides(&common.tool_overrides)?;
                let process = OpsProcess::new(common.allow_subprocess);
                let mut rows = Vec::new();
                let mut missing = Vec::new();
                let mut warnings = Vec::new();
                for tool in tools.tools {
                    let binary = overrides
                        .get(&tool.name)
                        .cloned()
                        .unwrap_or_else(|| tool.name.clone());
                    let mut row = process
                        .probe_tool(&binary, &tool.probe_argv, &tool.version_regex)
                        .map_err(|e| e.to_stable_message())?;
                    row["name"] = serde_json::Value::String(tool.name.clone());
                    if row["installed"] == serde_json::Value::Bool(false) {
                        if tool.required {
                            missing.push(format!(
                                "{}:{}",
                                ToolMismatchCode::MissingBinary.as_str(),
                                tool.name
                            ));
                        } else {
                            warnings.push(format!(
                                "{}:{}",
                                ToolMismatchCode::MissingBinary.as_str(),
                                tool.name
                            ));
                        }
                    } else if row["version"].is_null() {
                        if tool.required {
                            missing.push(format!(
                                "{}:{}",
                                ToolMismatchCode::VersionMismatch.as_str(),
                                tool.name
                            ));
                        } else {
                            warnings.push(format!(
                                "{}:{}",
                                ToolMismatchCode::VersionMismatch.as_str(),
                                tool.name
                            ));
                        }
                    }
                    rows.push(row);
                }
                rows.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
                let payload = serde_json::json!({
                    "schema_version":1,
                    "text": if missing.is_empty() {"all required tools verified"} else {"required tools mismatch"},
                    "rows":rows,
                    "missing":missing,
                    "warnings":warnings,
                    "summary":{"total":rows.len(),"errors":missing.len(),"warnings":warnings.len()}
                });
                let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
                Ok((
                    rendered,
                    if payload["missing"].as_array().is_some_and(|v| v.is_empty()) {
                        ops_exit::PASS
                    } else {
                        ops_exit::TOOL_MISSING
                    },
                ))
            }
            OpsToolsCommand::Doctor(common) => {
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let tools = load_tools_manifest(&repo_root).map_err(|e| e.to_stable_message())?;
                let rows = tools
                    .tools
                    .iter()
                    .map(|t| {
                        serde_json::json!({
                            "name": t.name,
                            "required": t.required,
                            "version_regex": t.version_regex,
                            "status": if common.allow_subprocess {"verify_with_subprocess"} else {"requires_allow_subprocess_for_verification"}
                        })
                    })
                    .collect::<Vec<_>>();
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "text": if common.allow_subprocess {"tool verification can run"} else {"tool verification is dry-run without subprocess"},
                    "rows": rows,
                    "summary": {"total": tools.tools.len(), "errors": 0, "warnings": if common.allow_subprocess {0} else {tools.tools.len()}}
                });
                let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
                Ok((rendered, ops_exit::PASS))
            }
        },
        _ => Err("__UNHANDLED__".to_string()),
    }
}
