// SPDX-License-Identifier: Apache-2.0

use super::*;

fn ops_runbook_source_rel() -> &'static str {
    "ops/RUNBOOK_GENERATION_FROM_GRAPH.md"
}

fn load_ops_runbook_rows(repo_root: &std::path::Path) -> Result<Vec<serde_json::Value>, String> {
    let install_matrix_path = repo_root.join("ops/k8s/install-matrix.json");
    let install_matrix: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&install_matrix_path)
            .map_err(|err| format!("failed to read {}: {err}", install_matrix_path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", install_matrix_path.display()))?;

    let mut profiles = std::collections::BTreeMap::<String, serde_json::Value>::new();
    for profile in install_matrix
        .get("profiles")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
    {
        let Some(name) = profile.get("name").and_then(|value| value.as_str()) else {
            continue;
        };
        profiles.insert(name.to_string(), profile.clone());
    }

    let profile_intent_path = repo_root.join("ops/stack/profile-intent.json");
    let mut profile_intents = std::collections::BTreeMap::<String, serde_json::Value>::new();
    if profile_intent_path.exists() {
        let value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&profile_intent_path).map_err(
                |err| format!("failed to read {}: {err}", profile_intent_path.display()),
            )?)
            .map_err(|err| format!("failed to parse {}: {err}", profile_intent_path.display()))?;
        for profile in value
            .get("profiles")
            .and_then(|value| value.as_array())
            .into_iter()
            .flatten()
        {
            let Some(name) = profile.get("name").and_then(|value| value.as_str()) else {
                continue;
            };
            profile_intents.insert(name.to_string(), profile.clone());
        }
    }

    let toolchain_path = repo_root.join("ops/inventory/toolchain.json");
    let toolchain: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&toolchain_path)
            .map_err(|err| format!("failed to read {}: {err}", toolchain_path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", toolchain_path.display()))?;
    let tool_versions = toolchain
        .get("tools")
        .and_then(|value| value.as_object())
        .map(|tools| {
            tools.iter()
                .map(|(binary, detail)| {
                    serde_json::json!({
                        "binary": binary,
                        "probe_argv": detail.get("probe_argv").cloned().unwrap_or_else(|| serde_json::json!([])),
                        "required": detail.get("required").and_then(|value| value.as_bool()).unwrap_or(false),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let failure_modes = vec![
        serde_json::json!({"code": "OPS_MANIFEST_ERROR", "meaning": "required ops manifest or generated input is missing or unreadable"}),
        serde_json::json!({"code": "OPS_SCHEMA_ERROR", "meaning": "authored inputs drifted outside their governed schema"}),
        serde_json::json!({"code": "OPS_TOOL_ERROR", "meaning": "required tool invocation failed or a required tool is unavailable"}),
        serde_json::json!({"code": "OPS_PROFILE_ERROR", "meaning": "selected profile is unknown or not declared in the governed registries"}),
        serde_json::json!({"code": "OPS_EFFECT_ERROR", "meaning": "effectful install action was requested without the required capability flags"}),
    ];

    let mut scenarios = install_matrix
        .get("scenarios")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    scenarios.sort_by(|left, right| {
        left.get("name")
            .and_then(|value| value.as_str())
            .cmp(&right.get("name").and_then(|value| value.as_str()))
    });

    let mut rows = Vec::new();
    for scenario in scenarios {
        let Some(name) = scenario.get("name").and_then(|value| value.as_str()) else {
            continue;
        };
        let Some(kind) = scenario.get("kind").and_then(|value| value.as_str()) else {
            continue;
        };
        let Some(profile) = scenario.get("profile").and_then(|value| value.as_str()) else {
            continue;
        };
        let suite = scenario
            .get("suite")
            .and_then(|value| value.as_str())
            .unwrap_or_default();
        let values_file = profiles
            .get(profile)
            .and_then(|value| value.get("values_file"))
            .and_then(|value| value.as_str())
            .unwrap_or_default();
        let steps = match kind {
            "install" => vec![
                format!(
                    "bijux dev atlas ops render --profile {profile} --target helm --allow-subprocess --allow-write --format json"
                ),
                format!("bijux dev atlas ops install --profile {profile} --plan --format json"),
                format!(
                    "bijux dev atlas ops install --profile {profile} --kind --apply --allow-subprocess --allow-write --allow-network --format json"
                ),
            ],
            "upgrade" => vec![
                format!(
                    "bijux dev atlas ops render --profile {profile} --target helm --allow-subprocess --allow-write --format json"
                ),
                format!(
                    "bijux dev atlas ops install --profile {profile} --kind --apply --allow-subprocess --allow-write --allow-network --format json"
                ),
            ],
            "rollback" => vec![
                format!(
                    "bijux dev atlas ops install --profile {profile} --kind --apply --allow-subprocess --allow-write --allow-network --format json"
                ),
                format!(
                    "bijux dev atlas ops stack down --profile {profile} --allow-subprocess --allow-write --allow-network --force --format json"
                ),
            ],
            _ => Vec::new(),
        };
        let verification_commands = vec![
            format!("bijux dev atlas ops install --profile {profile} --plan --format json"),
            "kubectl get pods -n bijux-atlas".to_string(),
            "kubectl get svc -n bijux-atlas".to_string(),
            "curl -fsS http://127.0.0.1:8080/health".to_string(),
        ];
        let rollback_commands = vec![
            format!(
                "bijux dev atlas ops stack down --profile {profile} --allow-subprocess --allow-write --allow-network --force --format json"
            ),
            "kubectl delete namespace bijux-atlas --ignore-not-found".to_string(),
        ];
        rows.push(serde_json::json!({
            "scenario": name,
            "scenario_kind": kind,
            "profile": profile,
            "suite": suite,
            "values_file": values_file,
            "baseline_ref": scenario.get("baseline_ref").cloned(),
            "target_ref": scenario.get("target_ref").cloned(),
            "profile_intent": profile_intents.get(profile).cloned(),
            "steps": steps,
            "verification_commands": verification_commands,
            "rollback_commands": rollback_commands,
            "failure_modes": failure_modes,
            "tool_versions": tool_versions,
        }));
    }

    Ok(rows)
}

pub(super) fn dispatch_execution(
    command: OpsCommand,
    debug: bool,
) -> Result<(String, i32), String> {
    match command {
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
            match crate::ops_execution_runtime::run_ops_install(&args) {
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
            let mut rows = vec![serde_json::json!({
                "kind": "artifacts",
                "status": "ok",
                "path": target.display().to_string()
            })];
            if common.allow_subprocess {
                let ops_root = resolve_ops_root(&repo_root, common.ops_root.clone())
                    .map_err(|e| e.to_stable_message())?;
                let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
                profiles.sort_by(|a, b| a.name.cmp(&b.name));
                let profile = resolve_profile(common.profile.clone(), &profiles)
                    .map_err(|e| e.to_stable_message())?;
                let process = OpsProcess::new(true);
                let namespace_delete_args = vec![
                    "delete".to_string(),
                    "namespace".to_string(),
                    "bijux-atlas".to_string(),
                    "--ignore-not-found=true".to_string(),
                ];
                let _ = process.run_subprocess("kubectl", &namespace_delete_args, &repo_root);
                let kind_delete_args = vec![
                    "delete".to_string(),
                    "cluster".to_string(),
                    "--name".to_string(),
                    profile.kind_profile.clone(),
                ];
                let _ = process.run_subprocess("kind", &kind_delete_args, &repo_root);
                rows.push(serde_json::json!({
                    "kind": "known_resources",
                    "status": "attempted",
                    "namespace": "bijux-atlas",
                    "kind_profile": profile.kind_profile
                }));
            }
            let text = format!(
                "reset artifacts for run_id={} at {}",
                run_id.as_str(),
                target.display()
            );
            let rendered = emit_payload(
                common.format,
                common.out.clone(),
                &serde_json::json!({"schema_version": 1, "text": text, "rows": rows, "summary": {"total": 1, "errors": 0, "warnings": 0}}),
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
                    let target = repo_root.join("ops/inventory/pins.yaml");
                    let old = load_stack_pins(&repo_root).map_err(|e| e.to_stable_message())?;
                    let mut updated = old.clone();
                    let stack_manifest: serde_json::Value = serde_json::from_str(
                        &std::fs::read_to_string(
                            repo_root.join("ops/stack/generated/version-manifest.json"),
                        )
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
                                "reason": "sync_from_generated_stack_version_manifest"
                            }));
                        }
                    }
                    let mut pins_yaml = std::fs::read_to_string(&target)
                        .map_err(|err| format!("failed to read {}: {err}", target.display()))?;
                    for (key, value) in &updated.images {
                        let needle = format!("{key}: ");
                        let mut replaced = false;
                        let mut lines = Vec::new();
                        for line in pins_yaml.lines() {
                            let trimmed = line.trim_start();
                            if trimmed.starts_with(&needle) {
                                lines.push(format!("  {key}: \"{value}\""));
                                replaced = true;
                            } else {
                                lines.push(line.to_string());
                            }
                        }
                        if !replaced {
                            return Err(format!(
                                "failed to sync image `{key}` into {}; missing key in pins.yaml",
                                target.display()
                            ));
                        }
                        pins_yaml = lines.join("\n");
                        pins_yaml.push('\n');
                    }
                    std::fs::write(&target, pins_yaml)
                        .map_err(|err| format!("failed to write {}: {err}", target.display()))?;
                    let text = "ops pins updated from generated stack version manifest".to_string();
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
                let stack_rel = "ops/stack/generated/version-manifest.json";
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
            OpsGenerateCommand::SurfaceList {
                check,
                write_example,
                common,
            } => {
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let run_id = run_id_or_default(common.run_id.clone())?;
                let ops_registry = bijux_dev_atlas::core::ops_registry::builtin_ops_registry();
                let domains = {
                    let mut set = std::collections::BTreeSet::new();
                    for entry in &ops_registry {
                        set.insert(entry.domain);
                    }
                    set.into_iter().collect::<Vec<_>>()
                };
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "generated_by": "bijux dev atlas ops generate surface-list --write-example",
                    "status": "pass",
                    "surfaces": ["check", "configs", "docs", "ops"],
                    "crate_alignment": {
                        "source": "cargo metadata",
                        "status": "pass"
                    },
                    "ops_taxonomy": {
                        "domains": domains,
                        "entries": ops_registry.into_iter().map(|entry| {
                            serde_json::json!({
                                "domain": entry.domain,
                                "verb": entry.verb,
                                "subverb": entry.subverb,
                                "tags": entry.tags.iter().map(|tag| format!("{tag:?}").to_ascii_lowercase()).collect::<Vec<_>>()
                            })
                        }).collect::<Vec<_>>()
                    }
                });

                let expected =
                    repo_root.join("ops/_generated.example/control-plane-surface-list.json");
                if check {
                    let existing = std::fs::read_to_string(&expected).map_err(|err| {
                        format!(
                            "surface-list check failed: missing {}: {err}",
                            expected.display()
                        )
                    })?;
                    let expected_json: serde_json::Value = serde_json::from_str(&existing)
                        .map_err(|err| {
                            format!(
                                "surface-list check failed: invalid json {}: {err}",
                                expected.display()
                            )
                        })?;
                    let matches = expected_json == payload;
                    let rendered = emit_payload(
                        common.format,
                        common.out.clone(),
                        &serde_json::json!({
                            "schema_version": 1,
                            "text": if matches { "control-plane surface list matches expected example" } else { "control-plane surface list drift detected" },
                            "rows": [{"path": expected.display().to_string(), "matches": matches}],
                            "summary": {"total": 1, "errors": if matches { 0 } else { 1 }, "warnings": 0}
                        }),
                    )?;
                    return Ok((rendered, if matches { 0 } else { 1 }));
                }

                if write_example {
                    if !common.allow_write {
                        return Err("surface-list generation requires --allow-write".to_string());
                    }
                    let encoded = serde_json::to_string_pretty(&payload)
                        .map_err(|err| format!("surface-list encode failed: {err}"))?;
                    if let Some(parent) = expected.parent() {
                        std::fs::create_dir_all(parent).map_err(|err| {
                            format!("failed to create {}: {err}", parent.display())
                        })?;
                    }
                    std::fs::write(&expected, encoded)
                        .map_err(|err| format!("failed to write {}: {err}", expected.display()))?;
                    let generated =
                        repo_root.join("ops/_generated/control-plane-surface-list.json");
                    if let Some(parent) = generated.parent() {
                        std::fs::create_dir_all(parent).map_err(|err| {
                            format!("failed to create {}: {err}", parent.display())
                        })?;
                    }
                    std::fs::copy(&expected, &generated).map_err(|err| {
                        format!(
                            "failed to mirror {} to {}: {err}",
                            expected.display(),
                            generated.display()
                        )
                    })?;
                }

                let fs_adapter = OpsFs::new(repo_root.clone(), repo_root.join("ops"));
                let rel = "generate/control-plane-surface-list.json";
                let out = fs_adapter
                    .write_artifact_json(&run_id, rel, &payload)
                    .map_err(|e| e.to_stable_message())?;
                let rendered = emit_payload(
                    common.format,
                    common.out.clone(),
                    &serde_json::json!({
                        "schema_version": 1,
                        "text": format!("generated control-plane surface list at {}", out.display()),
                        "rows": [{"artifact_path": out.display().to_string(), "example_path": expected.display().to_string(), "write_example": write_example}],
                        "summary": {"total": 1, "errors": 0, "warnings": 0}
                    }),
                )?;
                Ok((rendered, 0))
            }
            OpsGenerateCommand::Runbook { check, common } => {
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let run_id = run_id_or_default(common.run_id.clone())?;
                let fs_adapter = OpsFs::new(repo_root.clone(), repo_root.join("ops"));
                let source_rel = ops_runbook_source_rel();
                let source_text = std::fs::read_to_string(repo_root.join(source_rel))
                    .map_err(|err| format!("failed to read {source_rel}: {err}"))?;
                let rows = load_ops_runbook_rows(&repo_root)?;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "run_id": run_id.as_str(),
                    "generator": "ops generate runbook",
                    "source": source_rel,
                    "source_sha256": sha256_hex(&source_text),
                    "status": "pass",
                    "rows": rows,
                    "summary": {"total": rows.len(), "errors": 0, "warnings": 0}
                });
                if check {
                    let rendered = emit_payload(
                        common.format,
                        common.out.clone(),
                        &serde_json::json!({
                            "schema_version": 1,
                            "text": "runbook generation contract is present and loadable",
                            "rows": [payload],
                            "summary": {"total": 1, "errors": 0, "warnings": 0}
                        }),
                    )?;
                    Ok((rendered, 0))
                } else {
                    let out = fs_adapter
                        .write_artifact_json(&run_id, "generate/runbook.index.json", &payload)
                        .map_err(|e| e.to_stable_message())?;
                    let rendered = emit_payload(
                        common.format,
                        common.out.clone(),
                        &serde_json::json!({
                            "schema_version": 1,
                            "text": format!("generated runbook index artifact at {}", out.display()),
                            "rows": [payload],
                            "summary": {"total": 1, "errors": 0, "warnings": 0}
                        }),
                    )?;
                    Ok((rendered, 0))
                }
            }
            OpsGenerateCommand::ChartDependencySbom { check, common } => {
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let run_id = run_id_or_default(common.run_id.clone())?;
                let chart_yaml_path = repo_root.join("ops/k8s/charts/bijux-atlas/Chart.yaml");
                let chart_yaml_text = std::fs::read_to_string(&chart_yaml_path).map_err(|err| {
                    format!("failed to read {}: {err}", chart_yaml_path.display())
                })?;
                let chart_yaml: serde_yaml::Value = serde_yaml::from_str(&chart_yaml_text)
                    .map_err(|err| {
                        format!("failed to parse {}: {err}", chart_yaml_path.display())
                    })?;
                let dependencies = chart_yaml
                    .as_mapping()
                    .and_then(|map| map.get(serde_yaml::Value::String("dependencies".to_string())))
                    .and_then(serde_yaml::Value::as_sequence)
                    .cloned()
                    .unwrap_or_default();
                let mut rows = Vec::new();
                let mut errors = Vec::new();
                for dep in dependencies {
                    let Some(dep_map) = dep.as_mapping() else {
                        errors.push("Chart.yaml dependencies entries must be objects".to_string());
                        continue;
                    };
                    let name = dep_map
                        .get(serde_yaml::Value::String("name".to_string()))
                        .and_then(serde_yaml::Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    let version = dep_map
                        .get(serde_yaml::Value::String("version".to_string()))
                        .and_then(serde_yaml::Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    let repository = dep_map
                        .get(serde_yaml::Value::String("repository".to_string()))
                        .and_then(serde_yaml::Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    if version.contains('^')
                        || version.contains('~')
                        || version.contains('>')
                        || version.contains('<')
                        || version.contains('*')
                        || version.contains('x')
                    {
                        errors.push(format!(
                            "dependency `{name}` must pin an exact version, found `{version}`"
                        ));
                    }
                    rows.push(serde_json::json!({
                        "name": name,
                        "version": version,
                        "repository": repository
                    }));
                }
                rows.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));

                let chart_lock_path = repo_root.join("ops/k8s/charts/bijux-atlas/Chart.lock");
                let lock_exists = chart_lock_path.is_file();
                if !rows.is_empty() && !lock_exists {
                    errors.push(format!(
                        "Chart.lock is required when Chart.yaml declares dependencies: {}",
                        chart_lock_path.display()
                    ));
                }
                if lock_exists {
                    let lock_text = std::fs::read_to_string(&chart_lock_path).map_err(|err| {
                        format!("failed to read {}: {err}", chart_lock_path.display())
                    })?;
                    let lock_yaml: serde_yaml::Value =
                        serde_yaml::from_str(&lock_text).map_err(|err| {
                            format!("failed to parse {}: {err}", chart_lock_path.display())
                        })?;
                    let lock_rows = lock_yaml
                        .as_mapping()
                        .and_then(|map| {
                            map.get(serde_yaml::Value::String("dependencies".to_string()))
                        })
                        .and_then(serde_yaml::Value::as_sequence)
                        .cloned()
                        .unwrap_or_default();
                    let mut lock_set = std::collections::BTreeSet::new();
                    for dep in lock_rows {
                        let Some(dep_map) = dep.as_mapping() else {
                            continue;
                        };
                        let name = dep_map
                            .get(serde_yaml::Value::String("name".to_string()))
                            .and_then(serde_yaml::Value::as_str)
                            .unwrap_or_default()
                            .to_string();
                        let version = dep_map
                            .get(serde_yaml::Value::String("version".to_string()))
                            .and_then(serde_yaml::Value::as_str)
                            .unwrap_or_default()
                            .to_string();
                        lock_set.insert((name, version));
                    }
                    let mut chart_set = std::collections::BTreeSet::new();
                    for row in &rows {
                        chart_set.insert((
                            row["name"].as_str().unwrap_or_default().to_string(),
                            row["version"].as_str().unwrap_or_default().to_string(),
                        ));
                    }
                    if chart_set != lock_set {
                        errors.push(
                            "Chart.lock dependencies must match Chart.yaml dependency name/version pairs"
                                .to_string(),
                        );
                    }
                }

                let payload = serde_json::json!({
                    "schema_version": 1,
                    "kind": "ops_chart_dependency_sbom",
                    "chart": "ops/k8s/charts/bijux-atlas",
                    "dependencies": rows,
                    "lock_file": {
                        "path": "ops/k8s/charts/bijux-atlas/Chart.lock",
                        "exists": lock_exists
                    },
                    "summary": {
                        "total": rows.len(),
                        "errors": errors.len(),
                        "warnings": 0
                    },
                    "errors": errors
                });
                let exit = if payload["summary"]["errors"].as_u64().unwrap_or(0) == 0 {
                    ops_exit::PASS
                } else {
                    ops_exit::FAIL
                };
                if check {
                    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
                    return Ok((rendered, exit));
                }
                let fs_adapter = OpsFs::new(repo_root.clone(), repo_root.join("ops"));
                let out = fs_adapter
                    .write_artifact_json(&run_id, "generate/chart-dependencies-sbom.json", &payload)
                    .map_err(|e| e.to_stable_message())?;
                let rendered = emit_payload(
                    common.format,
                    common.out.clone(),
                    &serde_json::json!({
                        "schema_version": 1,
                        "text": format!("generated chart dependency sbom at {}", out.display()),
                        "rows": [payload],
                        "summary": {"total": 1, "errors": if exit == ops_exit::PASS { 0 } else { 1 }, "warnings": 0}
                    }),
                )?;
                Ok((rendered, exit))
            }
        },
        OpsCommand::Stack { .. }
        | OpsCommand::K8s { .. }
        | OpsCommand::Load { .. }
        | OpsCommand::E2e { .. }
        | OpsCommand::Drills { .. }
        | OpsCommand::Obs { .. } => {
            unreachable!("ops nested wrapper variants are normalized before execution")
        }
        _ => Err("__UNHANDLED__".to_string()),
    }
}
