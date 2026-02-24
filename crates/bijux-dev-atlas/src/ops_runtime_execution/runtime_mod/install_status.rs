// SPDX-License-Identifier: Apache-2.0

pub(crate) fn run_ops_install(args: &cli::OpsInstallArgs) -> Result<(String, i32), String> {
    let common = &args.common;
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    let profile =
        resolve_profile(common.profile.clone(), &profiles).map_err(|e| e.to_stable_message())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    if !args.plan && !common.allow_subprocess {
        return Err(OpsCommandError::Effect(
            "install execution requires --allow-subprocess".to_string(),
        )
        .to_stable_message());
    }
    if (args.apply || args.kind) && !common.allow_write {
        return Err(OpsCommandError::Effect(
            "install apply/kind requires --allow-write".to_string(),
        )
        .to_stable_message());
    }
    if (args.apply || args.kind) && !common.allow_network {
        return Err(OpsCommandError::Effect(
            "install apply/kind requires --allow-network".to_string(),
        )
        .to_stable_message());
    }

    let mut steps = Vec::new();
    let process = OpsProcess::new(common.allow_subprocess);
    if args.kind {
        steps.push("kind cluster ensure".to_string());
        if !args.plan {
            let kind_config = repo_root.join(&profile.cluster_config);
            let kind_args = vec![
                "create".to_string(),
                "cluster".to_string(),
                "--name".to_string(),
                profile.kind_profile.clone(),
                "--config".to_string(),
                kind_config.display().to_string(),
            ];
            if let Err(err) = process.run_subprocess("kind", &kind_args, &repo_root) {
                let stable = err.to_stable_message();
                if !stable.contains("already exists") {
                    return Err(stable);
                }
            }
        }
    }
    if args.apply {
        steps.push("kubectl apply".to_string());
        if !args.plan {
            ensure_kind_context(&process, &profile, common.force)
                .map_err(|e| e.to_stable_message())?;
            ensure_namespace_exists(&process, "bijux-atlas", &args.dry_run)
                .map_err(|e| e.to_stable_message())?;
            let render_path = repo_root
                .join("artifacts/ops")
                .join(run_id.as_str())
                .join(format!("render/{}/helm/render.yaml", profile.name));
            let mut apply_args = vec![
                "apply".to_string(),
                "-n".to_string(),
                "bijux-atlas".to_string(),
                "-f".to_string(),
                render_path.display().to_string(),
            ];
            if args.dry_run == "client" {
                apply_args.push("--dry-run=client".to_string());
            }
            let _ = process
                .run_subprocess("kubectl", &apply_args, &repo_root)
                .map_err(|e| e.to_stable_message())?;
        }
    }
    if !args.kind && !args.apply {
        steps.push("validate-only".to_string());
    }
    let payload = serde_json::json!({
        "schema_version": 1,
        "profile": profile.name,
        "run_id": run_id.as_str(),
        "plan_mode": args.plan,
        "dry_run": args.dry_run,
        "steps": steps,
        "kind_context_expected": expected_kind_context(&profile),
    });
    let text = if args.plan {
        format!("install plan generated for profile `{}`", profile.name)
    } else {
        format!("install completed for profile `{}`", profile.name)
    };
    let envelope = serde_json::json!({"schema_version": 1, "text": text, "rows": [payload], "summary": {"total": 1, "errors": 0, "warnings": 0}});
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, 0))
}

pub(crate) fn run_ops_status(args: &cli::OpsStatusArgs) -> Result<(String, i32), String> {
    let common = &args.common;
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    let profile =
        resolve_profile(common.profile.clone(), &profiles).map_err(|e| e.to_stable_message())?;
    let process = OpsProcess::new(common.allow_subprocess);
    let (payload, text) = match args.target {
        OpsStatusTarget::Local => {
            let toolchain_path = ops_root.join("inventory/toolchain.json");
            let toolchain = std::fs::read_to_string(&toolchain_path).map_err(|err| {
                OpsCommandError::Manifest(format!(
                    "failed to read {}: {err}",
                    toolchain_path.display()
                ))
                .to_stable_message()
            })?;
            let toolchain_json: serde_json::Value =
                serde_json::from_str(&toolchain).map_err(|err| {
                    OpsCommandError::Schema(format!(
                        "failed to parse {}: {err}",
                        toolchain_path.display()
                    ))
                    .to_stable_message()
                })?;
            (
                serde_json::json!({
                    "schema_version": 1,
                    "target": "local",
                    "repo_root": repo_root.display().to_string(),
                    "ops_root": ops_root.display().to_string(),
                    "profile": profile,
                    "toolchain": toolchain_json,
                }),
                format!(
                    "ops status local: profile={} repo_root={} ops_root={}",
                    profile.name,
                    repo_root.display(),
                    ops_root.display(),
                ),
            )
        }
        OpsStatusTarget::K8s => {
            if !common.allow_subprocess {
                return Err(OpsCommandError::Effect(
                    "status k8s requires --allow-subprocess".to_string(),
                )
                .to_stable_message());
            }
            let kubectl_args = vec![
                "get".to_string(),
                "all".to_string(),
                "-n".to_string(),
                "bijux-atlas".to_string(),
                "-o".to_string(),
                "json".to_string(),
            ];
            let (stdout, _) = process
                .run_subprocess("kubectl", &kubectl_args, &repo_root)
                .map_err(|e| e.to_stable_message())?;
            let value: serde_json::Value = serde_json::from_str(&stdout).map_err(|err| {
                OpsCommandError::Schema(format!("failed to parse kubectl json: {err}"))
                    .to_stable_message()
            })?;
            (
                serde_json::json!({
                    "schema_version": 1,
                    "target": "k8s",
                    "profile": profile.name,
                    "resources": value
                }),
                "ops status k8s collected".to_string(),
            )
        }
        OpsStatusTarget::Pods => {
            if !common.allow_subprocess {
                return Err(OpsCommandError::Effect(
                    "status pods requires --allow-subprocess".to_string(),
                )
                .to_stable_message());
            }
            let kubectl_args = vec![
                "get".to_string(),
                "pods".to_string(),
                "-n".to_string(),
                "bijux-atlas".to_string(),
                "-o".to_string(),
                "json".to_string(),
            ];
            let (stdout, _) = process
                .run_subprocess("kubectl", &kubectl_args, &repo_root)
                .map_err(|e| e.to_stable_message())?;
            let value: serde_json::Value = serde_json::from_str(&stdout).map_err(|err| {
                OpsCommandError::Schema(format!("failed to parse kubectl json: {err}"))
                    .to_stable_message()
            })?;
            let mut pods = value
                .get("items")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            pods.sort_by(|a, b| {
                a.get("metadata")
                    .and_then(|m| m.get("name"))
                    .and_then(|v| v.as_str())
                    .cmp(
                        &b.get("metadata")
                            .and_then(|m| m.get("name"))
                            .and_then(|v| v.as_str()),
                    )
            });
            (
                serde_json::json!({
                    "schema_version": 1,
                    "target": "pods",
                    "profile": profile.name,
                    "pods": pods
                }),
                "ops status pods collected".to_string(),
            )
        }
        OpsStatusTarget::Endpoints => {
            if !common.allow_subprocess {
                return Err(OpsCommandError::Effect(
                    "status endpoints requires --allow-subprocess".to_string(),
                )
                .to_stable_message());
            }
            let kubectl_args = vec![
                "get".to_string(),
                "endpoints".to_string(),
                "-n".to_string(),
                "bijux-atlas".to_string(),
                "-o".to_string(),
                "json".to_string(),
            ];
            let (stdout, _) = process
                .run_subprocess("kubectl", &kubectl_args, &repo_root)
                .map_err(|e| e.to_stable_message())?;
            let value: serde_json::Value = serde_json::from_str(&stdout).map_err(|err| {
                OpsCommandError::Schema(format!("failed to parse kubectl json: {err}"))
                    .to_stable_message()
            })?;
            (
                serde_json::json!({
                    "schema_version": 1,
                    "target": "endpoints",
                    "profile": profile.name,
                    "resources": value
                }),
                "ops status endpoints collected".to_string(),
            )
        }
    };
    let envelope = serde_json::json!({"schema_version": 1, "text": text, "rows": [payload], "summary": {"total": 1, "errors": 0, "warnings": 0}});
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, 0))
}

