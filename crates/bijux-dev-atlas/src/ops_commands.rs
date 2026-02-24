use crate::cli::OpsInstallArgs;
use crate::ops_command_support::{
    build_ops_run_report, load_stack_manifest, load_stack_pins, load_toolchain_inventory_for_ops,
    load_tools_manifest, ops_exit, ops_pins_check_payload, parse_tool_overrides, render_ops_human,
    render_ops_validation_output, run_ops_checks, validate_pins_completeness,
    validate_stack_manifest, verify_tools_snapshot, ToolMismatchCode,
};
pub(crate) use crate::ops_command_support::{
    emit_payload, load_profiles, normalize_tool_version_with_regex, resolve_ops_root,
    resolve_profile, run_id_or_default, sha256_hex,
};

pub(crate) fn run_ops_command(quiet: bool, debug: bool, command: OpsCommand) -> i32 {
    let command = match command {
        OpsCommand::Stack { command } => match command {
            OpsStackCommand::Plan(common) => OpsCommand::Plan(common),
            OpsStackCommand::Up(common) => OpsCommand::Up(common),
            OpsStackCommand::Down(common) => OpsCommand::Down(common),
            OpsStackCommand::Status(mut args) => {
                args.target = OpsStatusTarget::K8s;
                OpsCommand::Status(args)
            }
            OpsStackCommand::Reset(args) => OpsCommand::Reset(args),
        },
        OpsCommand::K8s { command } => match command {
            OpsK8sCommand::Render(args) => OpsCommand::Render(args),
            OpsK8sCommand::Test(common) => OpsCommand::Conformance(common),
            OpsK8sCommand::Status(args) => OpsCommand::Status(args),
        },
        OpsCommand::Load { command } => match command {
            OpsLoadCommand::Run(common) => OpsCommand::Explain {
                action: "load-run".to_string(),
                common,
            },
        },
        OpsCommand::E2e { command } => match command {
            OpsE2eCommand::Run(common) => OpsCommand::Explain {
                action: "e2e-run".to_string(),
                common,
            },
        },
        OpsCommand::Obs { command } => match command {
            OpsObsCommand::Drill { command } => match command {
                OpsObsDrillCommand::Run(common) => OpsCommand::Explain {
                    action: "obs-drill-run".to_string(),
                    common,
                },
            },
            OpsObsCommand::Verify(common) => OpsCommand::Explain {
                action: "obs-verify".to_string(),
                common,
            },
        },
        other => other,
    };
    let run: Result<(String, i32), String> = (|| match command {
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
                "render" | "k8s-render" => serde_json::json!({"action":"render","purpose":"render deterministic ops manifests","effects_required":["subprocess"],"flags":["--allow-subprocess","--allow-write (optional)"]}),
                "stack-plan" => serde_json::json!({"action":"stack-plan","purpose":"resolve stack resources for a profile without executing subprocesses","effects_required":[]}),
                "install" | "stack-up" => serde_json::json!({"action":"install","purpose":"plan/apply ops stack to local cluster","effects_required":["subprocess","fs_write","network"],"flags":["--allow-subprocess","--allow-write","--allow-network"]}),
                "down" | "stack-down" => serde_json::json!({"action":"down","purpose":"teardown local ops stack resources","effects_required":["subprocess"],"flags":["--allow-subprocess"]}),
                "status" | "stack-status" => serde_json::json!({"action":"status","purpose":"collect local/k8s status rows","effects_required":["subprocess (for k8s/pods/endpoints)"]}),
                "conformance" | "k8s-test" => serde_json::json!({"action":"conformance","purpose":"run ops conformance status checks","effects_required":["subprocess"],"flags":["--allow-subprocess"]}),
                "load-run" => serde_json::json!({"action":"load-run","purpose":"reserved for k6 orchestration under ops load","status":"not_implemented"}),
                "e2e-run" => serde_json::json!({"action":"e2e-run","purpose":"reserved for scenario orchestration","status":"not_implemented"}),
                "obs-drill-run" => serde_json::json!({"action":"obs-drill-run","purpose":"reserved for observability drill orchestration","status":"not_implemented"}),
                "obs-verify" => serde_json::json!({"action":"obs-verify","purpose":"verify observability contracts","effects_required":[]}),
                "tools-doctor" => serde_json::json!({"action":"tools-doctor","purpose":"show required tools and missing requirements without subprocess by default","effects_required":[]}),
                "suite-list" => serde_json::json!({"kind":"suite","action":"list","suites":["e2e","k8s","load","obs"]}),
                value if value.starts_with("suite-run:") => serde_json::json!({"kind":"suite","action":"run","suite":value.trim_start_matches("suite-run:")}),
                "cleanup" => serde_json::json!({"action":"cleanup","purpose":"remove scoped artifacts and local ops resources","effects_required":["subprocess (optional)"]}),
                _ => {
                    return Err(format!(
                        "unknown ops action `{}` (try inventory|validate|render|install|down|status|conformance|cleanup|load-run|e2e-run|obs-drill-run)",
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
                match bijux_dev_atlas_core::ops_inventory::OpsInventory::load_and_validate(
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
                match bijux_dev_atlas_core::ops_inventory::OpsInventory::load_and_validate(
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
                match bijux_dev_atlas_core::ops_inventory::OpsInventory::load_and_validate(
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
                        "ops/stack/version-manifest.json",
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
            let (status_rendered, status_code) =
                crate::ops_runtime_execution::run_ops_status(&status_args)?;
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
            let code = if payload["status"] == serde_json::Value::String("ok".to_string()) {
                0
            } else {
                1
            };
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
        OpsCommand::Suite { command } => match command {
            OpsSuiteCommand::List(common) => {
                let mut suites = vec!["e2e", "k8s", "load", "obs"];
                suites.sort();
                let rows = suites
                    .iter()
                    .map(|suite| serde_json::json!({"suite": suite}))
                    .collect::<Vec<_>>();
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "text": suites.join("\n"),
                    "rows": rows,
                    "summary": {"total": suites.len(), "errors": 0, "warnings": 0}
                });
                let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
                Ok((rendered, ops_exit::PASS))
            }
            OpsSuiteCommand::Run { suite, common } => {
                let suite_norm = suite.trim().to_ascii_lowercase();
                let mapped = match suite_norm.as_str() {
                    "load" | "e2e" | "k8s" | "obs" => "ops_all",
                    _ => {
                        return Err(format!(
                            "unknown suite `{suite_norm}` (expected load|e2e|k8s|obs)"
                        ))
                    }
                };
                let (checks_rendered, checks_code) = run_ops_checks(&common, mapped, true, true)?;
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let ops_root = resolve_ops_root(&repo_root, common.ops_root.clone())
                    .map_err(|e| e.to_stable_message())?;
                let run_id = run_id_or_default(common.run_id.clone())?;
                let status = if checks_code == 0 { "ok" } else { "failed" };
                let report = build_ops_run_report(
                    "ops suite run",
                    &common,
                    &run_id,
                    &repo_root,
                    &ops_root,
                    Some(suite_norm.clone()),
                    status,
                    if checks_code == 0 {
                        ops_exit::PASS
                    } else {
                        ops_exit::FAIL
                    },
                    Vec::new(),
                    if checks_code == 0 {
                        Vec::new()
                    } else {
                        vec!["suite checks failed".to_string()]
                    },
                    vec![serde_json::json!({"checks_output": checks_rendered})],
                );
                let rendered = match common.format {
                    FormatArg::Text => render_ops_human(&report),
                    FormatArg::Json => {
                        serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
                    }
                    FormatArg::Jsonl => {
                        serde_json::to_string(&report).map_err(|e| e.to_string())?
                    }
                };
                write_output_if_requested(common.out.clone(), &rendered)?;
                Ok((
                    rendered,
                    if checks_code == 0 {
                        ops_exit::PASS
                    } else {
                        ops_exit::FAIL
                    },
                ))
            }
        },
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
            let envelope = serde_json::json!({"schema_version": 1, "text": text, "rows": rows, "summary": {"total": rows.len(), "errors": 0, "warnings": 0}});
            let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
            Ok((rendered, 0))
        }
        OpsCommand::VerifyTools(common) => {
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
        OpsCommand::Plan(common) => {
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let manifest = load_stack_manifest(&repo_root).map_err(|e| e.to_stable_message())?;
            let stack_errors = validate_stack_manifest(&repo_root, &manifest)
                .map_err(|e| e.to_stable_message())?;
            let profile_name = common.profile.clone().unwrap_or_else(|| "kind".to_string());
            let profile = manifest.profiles.get(&profile_name).ok_or_else(|| {
                format!("OPS_PROFILE_ERROR: unknown stack profile `{profile_name}`")
            })?;
            let mut resources = profile.components.clone();
            resources.sort();
            let mut tools = vec!["kind".to_string(), "kubectl".to_string()];
            if resources.iter().any(|c| c.contains("charts")) {
                tools.push("helm".to_string());
            }
            tools.sort();
            let payload = serde_json::json!({
                "schema_version": 1,
                "status": if stack_errors.is_empty() { "ok" } else { "failed" },
                "text": format!("ops stack plan profile={profile_name} resources={}", resources.len()),
                "rows": [{
                    "profile": profile_name,
                    "kind_profile": profile.kind_profile,
                    "cluster_config": profile.cluster_config,
                    "namespace": profile.namespace,
                    "resources": resources,
                    "tools": tools
                }],
                "summary": {"total": 1, "errors": stack_errors.len(), "warnings": 0},
                "errors": stack_errors
            });
            let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
            Ok((
                rendered,
                if payload["summary"]["errors"] == serde_json::json!(0) {
                    ops_exit::PASS
                } else {
                    ops_exit::FAIL
                },
            ))
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
            if !common.allow_network {
                return Err(
                    OpsCommandError::Effect("up requires --allow-network".to_string())
                        .to_stable_message(),
                );
            }
            let args = OpsInstallArgs {
                common: common.clone(),
                kind: true,
                apply: true,
                plan: false,
                dry_run: "none".to_string(),
            };
            match crate::ops_runtime_execution::run_ops_install(&args) {
                Ok(ok) => Ok(ok),
                Err(err) => {
                    let rollback = "rollback guidance: run `bijux dev atlas ops stack down --profile kind --allow-subprocess --allow-write --allow-network`";
                    let payload = serde_json::json!({
                        "schema_version": 1,
                        "text": "ops stack up failed",
                        "rows": [{"error": err, "rollback": rollback}],
                        "summary": {"total": 1, "errors": 1, "warnings": 0}
                    });
                    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
                    Ok((rendered, ops_exit::FAIL))
                }
            }
        }
        OpsCommand::Down(common) => {
            if !common.allow_subprocess {
                return Err(OpsCommandError::Effect(
                    "down requires --allow-subprocess".to_string(),
                )
                .to_stable_message());
            }
            if !common.allow_write {
                return Err(
                    OpsCommandError::Effect("down requires --allow-write".to_string())
                        .to_stable_message(),
                );
            }
            if !common.allow_network {
                return Err(
                    OpsCommandError::Effect("down requires --allow-network".to_string())
                        .to_stable_message(),
                );
            }
            let repo_root = resolve_repo_root(common.repo_root.clone())?;
            let ops_root = resolve_ops_root(&repo_root, common.ops_root.clone())
                .map_err(|e| e.to_stable_message())?;
            let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
            profiles.sort_by(|a, b| a.name.cmp(&b.name));
            let profile = resolve_profile(common.profile.clone(), &profiles)
                .map_err(|e| e.to_stable_message())?;
            let process = OpsProcess::new(common.allow_subprocess);
            let expected_context = format!("kind-{}", profile.kind_profile);
            let current_context = process
                .run_subprocess(
                    "kubectl",
                    &["config".to_string(), "current-context".to_string()],
                    &repo_root,
                )
                .map(|(stdout, _)| stdout.trim().to_string())
                .unwrap_or_default();
            if current_context != expected_context && !common.force {
                return Err(OpsCommandError::Effect(format!(
                    "context guard failed: expected `{expected_context}` got `{current_context}`; pass --force to override"
                ))
                .to_stable_message());
            }
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
        OpsCommand::Cleanup(common) => {
            let cleanup_common = common.clone();
            let (down_detail, down_code) = if cleanup_common.allow_subprocess {
                let down_common = cleanup_common.clone();
                match run_ops_command(true, debug, OpsCommand::Down(down_common)) {
                    0 => ("down ok".to_string(), 0),
                    code => (format!("down exit={code}"), code),
                }
            } else {
                ("down skipped (subprocess disabled)".to_string(), 0)
            };
            let clean_code =
                run_ops_command(true, debug, OpsCommand::Clean(cleanup_common.clone()));
            let clean_detail = if clean_code == 0 {
                "clean ok".to_string()
            } else {
                format!("clean exit={clean_code}")
            };
            let errors = usize::from(down_code != 0) + usize::from(clean_code != 0);
            let payload = serde_json::json!({
                "schema_version": 1,
                "text": if errors == 0 { "ops cleanup passed" } else { "ops cleanup failed" },
                "rows": [
                    {"action":"down","status": if down_code == 0 { "ok" } else { "failed" }, "detail": down_detail},
                    {"action":"clean","status": if clean_code == 0 { "ok" } else { "failed" }, "detail": clean_detail}
                ],
                "summary": {"total": 2, "errors": errors, "warnings": 0}
            });
            let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
            Ok((rendered, if errors == 0 { 0 } else { 1 }))
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
                let mut errors = Vec::new();
                let (payload_base, code_base) = ops_pins_check_payload(&common, &repo_root)?;
                if code_base != 0 {
                    errors.push("base pins validation failed".to_string());
                }
                let pins = load_stack_pins(&repo_root).map_err(|e| e.to_stable_message())?;
                errors.extend(
                    validate_pins_completeness(&repo_root, &pins)
                        .map_err(|e| e.to_stable_message())?,
                );
                let status = if errors.is_empty() { "ok" } else { "failed" };
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "status": status,
                    "text": if errors.is_empty() { "ops pins check passed" } else { "ops pins check failed" },
                    "rows": [payload_base],
                    "errors": errors,
                    "summary": {"total": 1, "errors": if status == "ok" {0} else {1}, "warnings": 0}
                });
                let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
                Ok((
                    rendered,
                    if errors.is_empty() {
                        ops_exit::PASS
                    } else {
                        ops_exit::FAIL
                    },
                ))
            }
            OpsPinsCommand::Update {
                i_know_what_im_doing,
                common,
            } => {
                if !i_know_what_im_doing {
                    Err("ops pins update requires --i-know-what-im-doing".to_string())
                } else if !common.allow_write {
                    Err(
                        OpsCommandError::Effect("pins update requires --allow-write".to_string())
                            .to_stable_message(),
                    )
                } else {
                    let repo_root = resolve_repo_root(common.repo_root.clone())?;
                    let target = repo_root.join("ops/stack/pins.toml");
                    let old = load_stack_pins(&repo_root).map_err(|e| e.to_stable_message())?;
                    let mut updated = old.clone();
                    let stack_manifest: serde_json::Value = serde_json::from_str(
                        &std::fs::read_to_string(repo_root.join("ops/stack/version-manifest.json"))
                            .map_err(|err| format!("failed to read version manifest: {err}"))?,
                    )
                    .map_err(|err| format!("invalid version manifest json: {err}"))?;
                    if let Some(obj) = stack_manifest.as_object() {
                        for (k, v) in obj {
                            if k == "schema_version" {
                                continue;
                            }
                            if let Some(value) = v.as_str() {
                                updated.images.insert(k.clone(), value.to_string());
                            }
                        }
                    }
                    let mut changed = Vec::new();
                    for (k, v) in &updated.images {
                        let old_v = old.images.get(k).cloned().unwrap_or_default();
                        if &old_v != v {
                            changed.push(serde_json::json!({
                                "key": format!("images.{k}"),
                                "old": old_v,
                                "new": v,
                                "reason": "sync_from_ops_stack_version_manifest"
                            }));
                        }
                    }
                    let serialized = toml::to_string_pretty(&updated)
                        .map_err(|err| format!("failed to render pins toml: {err}"))?;
                    std::fs::write(&target, serialized)
                        .map_err(|err| format!("failed to write {}: {err}", target.display()))?;
                    let text = "ops pins updated from stack version manifest".to_string();
                    let rendered = emit_payload(
                        common.format,
                        common.out.clone(),
                        &serde_json::json!({"schema_version": 1, "text": text, "rows": [{"target_path": target.display().to_string(),"changes":changed}], "summary": {"total": 1, "errors": 0, "warnings": 0}}),
                    )?;
                    Ok((rendered, ops_exit::PASS))
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
                    let expected_json: serde_json::Value = serde_json::from_str(&existing)
                        .map_err(|err| {
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
                        format!("pins index drift detected for {}", expected_path.display())
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
        OpsCommand::Stack { .. }
        | OpsCommand::K8s { .. }
        | OpsCommand::Load { .. }
        | OpsCommand::E2e { .. }
        | OpsCommand::Obs { .. } => {
            unreachable!("ops nested wrapper variants are normalized before execution")
        }
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
            if err.contains("unknown ops action")
                || err.contains("unknown suite")
                || err.contains("requires --")
            {
                ops_exit::USAGE
            } else if err.contains("missing required ops tools")
                || err.contains("required external tools are missing")
            {
                ops_exit::TOOL_MISSING
            } else if err.contains("OPS_MANIFEST_ERROR")
                || err.contains("OPS_SCHEMA_ERROR")
                || err.contains("cannot resolve ops root")
            {
                ops_exit::INFRA
            } else {
                ops_exit::FAIL
            }
        }
    }
}
use crate::cli::{
    OpsE2eCommand, OpsK8sCommand, OpsLoadCommand, OpsObsCommand, OpsObsDrillCommand,
    OpsStackCommand, OpsSuiteCommand, OpsToolsCommand,
};
use crate::*;
