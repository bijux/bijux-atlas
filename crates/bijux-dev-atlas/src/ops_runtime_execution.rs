use crate::ops_commands::{
    emit_payload, load_profiles, resolve_ops_root, resolve_profile, run_id_or_default, sha256_hex,
};
use crate::*;
use std::io::Write;

pub(crate) fn run_ops_render(args: &cli::OpsRenderArgs) -> Result<(String, i32), String> {
    let common = &args.common;
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let fs_adapter = OpsFs::new(repo_root.clone(), ops_root.clone());
    let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    let profile =
        resolve_profile(common.profile.clone(), &profiles).map_err(|e| e.to_stable_message())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let process = OpsProcess::new(common.allow_subprocess);
    let target_name = match args.target {
        OpsRenderTarget::Helm => "helm",
        OpsRenderTarget::Kustomize => "kustomize",
        OpsRenderTarget::Kind => "kind",
    };

    let (rendered_manifest, subprocess_events): (String, Vec<serde_json::Value>) = match args.target
    {
        OpsRenderTarget::Helm => {
            if !common.allow_subprocess {
                return Err(OpsCommandError::Effect(
                    "helm render requires --allow-subprocess".to_string(),
                )
                .to_stable_message());
            }
            let helm_binary = args
                .helm_binary
                .clone()
                .unwrap_or_else(|| "helm".to_string());
            let chart_path = ops_root.join("k8s/charts/bijux-atlas");
            let values_path = ops_root.join("k8s/charts/bijux-atlas/values.yaml");
            let cmd_args = vec![
                "template".to_string(),
                "bijux-atlas".to_string(),
                chart_path.display().to_string(),
                "--namespace".to_string(),
                "bijux-atlas".to_string(),
                "-f".to_string(),
                values_path.display().to_string(),
            ];
            let (stdout, event) = process
                .run_subprocess(&helm_binary, &cmd_args, &repo_root)
                .map_err(|e| e.to_stable_message())?;
            (stdout, vec![event])
        }
        OpsRenderTarget::Kind => {
            let cluster_config_path = repo_root.join(&profile.cluster_config);
            let content = fs::read_to_string(&cluster_config_path).map_err(|err| {
                OpsCommandError::Manifest(format!(
                    "failed to read cluster config {}: {err}",
                    cluster_config_path.display()
                ))
                .to_stable_message()
            })?;
            (
                format!("# source: {}\n{content}", profile.cluster_config),
                Vec::new(),
            )
        }
        OpsRenderTarget::Kustomize => {
            return Err(OpsCommandError::Effect(
                "kustomize render is not enabled; use --target helm or --target kind".to_string(),
            )
            .to_stable_message())
        }
    };

    let mut validation_errors = validate_render_output(&rendered_manifest, args.target);
    if matches!(args.target, OpsRenderTarget::Helm) {
        validation_errors.extend(validate_helm_dependencies(&ops_root));
    }
    validation_errors.sort();
    validation_errors.dedup();

    if args.write && !common.allow_write {
        return Err(OpsCommandError::Effect(
            "ops render --write requires --allow-write".to_string(),
        )
        .to_stable_message());
    }
    let write_enabled = args.write;
    let rel_base = render_profile_artifact_base(&profile.name, args.target);
    let rel_yaml = format!("{rel_base}/render.yaml");
    let rel_index = format!("{rel_base}/render.index.json");
    let mut written_files = Vec::new();
    let mut rows = Vec::new();

    let render_sha = sha256_hex(&rendered_manifest);
    let manifest_row = serde_json::json!({
        "path": rel_yaml,
        "sha256": render_sha,
        "bytes": rendered_manifest.len(),
    });
    rows.push(manifest_row.clone());

    if write_enabled {
        let yaml_path = repo_root
            .join("artifacts/atlas-dev/ops")
            .join(run_id.as_str())
            .join(&rel_yaml);
        if let Some(parent) = yaml_path.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                OpsCommandError::Manifest(format!("failed to create {}: {err}", parent.display()))
                    .to_stable_message()
            })?;
        }
        let mut file = fs::File::create(&yaml_path).map_err(|err| {
            OpsCommandError::Manifest(format!("failed to create {}: {err}", yaml_path.display()))
                .to_stable_message()
        })?;
        file.write_all(rendered_manifest.as_bytes())
            .map_err(|err| {
                OpsCommandError::Manifest(format!("failed to write {}: {err}", yaml_path.display()))
                    .to_stable_message()
            })?;
        written_files.push(rel_yaml.clone());

        let index_payload = serde_json::json!({
            "schema_version": 1,
            "run_id": run_id.as_str(),
            "profile": profile.name,
            "target": target_name,
            "files": rows
        });
        let index_path = fs_adapter
            .write_artifact_json(&run_id, &rel_index, &index_payload)
            .map_err(|e| e.to_stable_message())?;
        written_files.push(
            index_path
                .strip_prefix(
                    repo_root
                        .join("artifacts/atlas-dev/ops")
                        .join(run_id.as_str()),
                )
                .unwrap_or(index_path.as_path())
                .display()
                .to_string(),
        );
    }

    let text = if args.stdout {
        rendered_manifest.clone()
    } else {
        format!(
            "render target={target_name} profile={} run_id={} wrote={} validation_errors={}",
            profile.name,
            run_id.as_str(),
            write_enabled,
            validation_errors.len()
        )
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "text": text,
        "rows": [{
            "repo_root": repo_root.display().to_string(),
            "ops_root": ops_root.display().to_string(),
            "profile": profile.name,
            "kind_profile": profile.kind_profile,
            "cluster_config": profile.cluster_config,
            "run_id": run_id.as_str(),
            "target": target_name,
            "write_enabled": write_enabled,
            "check_only": args.check,
            "stdout_mode": args.stdout,
            "diff_mode": args.diff,
            "written_files": written_files,
            "render_index_files": rows,
            "validation_errors": validation_errors,
            "subprocess_events": subprocess_events
        }],
        "summary": {
            "total": 1,
            "errors": if validation_errors.is_empty() { 0 } else { validation_errors.len() },
            "warnings": 0
        }
    });
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    let exit = if validation_errors.is_empty() { 0 } else { 1 };
    Ok((rendered, exit))
}

fn validate_render_output(rendered: &str, target: OpsRenderTarget) -> Vec<String> {
    let mut errors = Vec::new();
    let required_kinds = match target {
        OpsRenderTarget::Helm => ["Namespace", "Deployment", "Service"].to_vec(),
        OpsRenderTarget::Kind | OpsRenderTarget::Kustomize => Vec::new(),
    };
    for kind in required_kinds {
        let needle = format!("kind: {kind}");
        if !rendered.contains(&needle) {
            errors.push(format!("missing required rendered resource `{needle}`"));
        }
    }
    if rendered.contains("kind: ClusterRole") {
        errors.push("rendered output includes forbidden resource `kind: ClusterRole`".to_string());
    }
    for line in rendered.lines() {
        if line.trim_start().starts_with("image:") && line.contains(":latest") {
            errors.push(format!(
                "rendered image uses forbidden latest tag: {}",
                line.trim()
            ));
        }
    }
    for marker in ["generatedAt:", "timestamp:", "creationTimestamp:"] {
        if rendered.contains(marker) {
            errors.push(format!(
                "render output contains forbidden timestamp marker `{marker}`"
            ));
        }
    }
    errors.sort();
    errors.dedup();
    errors
}

fn validate_helm_dependencies(ops_root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let chart_dir = ops_root.join("k8s/charts/bijux-atlas");
    let chart_yaml_path = chart_dir.join("Chart.yaml");
    let chart_yaml = match fs::read_to_string(&chart_yaml_path) {
        Ok(value) => value,
        Err(err) => {
            return vec![format!(
                "failed to read {}: {err}",
                chart_yaml_path.display()
            )];
        }
    };
    if chart_yaml.contains("\ndependencies:") {
        let lock_path = chart_dir.join("Chart.lock");
        if !lock_path.exists() {
            errors.push(format!(
                "helm dependencies are declared but {} is missing",
                lock_path.display()
            ));
        }
    }
    errors
}

fn render_profile_artifact_base(profile: &str, target: OpsRenderTarget) -> String {
    let target = match target {
        OpsRenderTarget::Helm => "helm",
        OpsRenderTarget::Kustomize => "kustomize",
        OpsRenderTarget::Kind => "kind",
    };
    format!("render/{profile}/{target}")
}

fn expected_kind_context(profile: &StackProfile) -> String {
    format!("kind-{}", profile.kind_profile)
}

fn ensure_kind_context(
    process: &OpsProcess,
    profile: &StackProfile,
    force: bool,
) -> Result<(), OpsCommandError> {
    let args = vec!["config".to_string(), "current-context".to_string()];
    let (stdout, _) = process.run_subprocess("kubectl", &args, Path::new("."))?;
    let current = stdout.trim();
    let expected = expected_kind_context(profile);
    if current == expected || force {
        Ok(())
    } else {
        Err(OpsCommandError::Effect(format!(
            "kubectl context guard failed: expected `{expected}` got `{current}`; pass --force to override"
        )))
    }
}

fn ensure_namespace_exists(
    process: &OpsProcess,
    namespace: &str,
    dry_run: &str,
) -> Result<(), OpsCommandError> {
    let get_args = vec![
        "get".to_string(),
        "namespace".to_string(),
        namespace.to_string(),
        "-o".to_string(),
        "name".to_string(),
    ];
    if process
        .run_subprocess("kubectl", &get_args, Path::new("."))
        .is_ok()
    {
        return Ok(());
    }
    let mut create_args = vec![
        "create".to_string(),
        "namespace".to_string(),
        namespace.to_string(),
    ];
    if dry_run == "client" {
        create_args.push("--dry-run=client".to_string());
    }
    let _ = process.run_subprocess("kubectl", &create_args, Path::new("."))?;
    Ok(())
}

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
            let _ = process
                .run_subprocess("kind", &kind_args, &repo_root)
                .map_err(|e| e.to_stable_message())?;
        }
    }
    if args.apply {
        steps.push("kubectl apply".to_string());
        if !args.plan {
            ensure_kind_context(&process, &profile, args.force)
                .map_err(|e| e.to_stable_message())?;
            ensure_namespace_exists(&process, "bijux-atlas", &args.dry_run)
                .map_err(|e| e.to_stable_message())?;
            let render_path = repo_root
                .join("artifacts/atlas-dev/ops")
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
