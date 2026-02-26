// SPDX-License-Identifier: Apache-2.0

use super::*;
pub(super) fn dispatch_profiles(command: OpsCommand, debug: bool) -> Result<(String, i32), String> {
    let _ = debug;
    match command {
        OpsCommand::Suite { command } => match command {
            OpsSuiteCommand::List(common) => {
                let mut suites = ["e2e", "k8s", "load", "obs"];
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
        _ => Err("__UNHANDLED__".to_string()),
    }
}
