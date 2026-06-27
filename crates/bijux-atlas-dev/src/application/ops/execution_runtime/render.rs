// SPDX-License-Identifier: Apache-2.0

use crate::ops_commands::{
    emit_payload, load_profiles, resolve_ops_root, resolve_profile, run_id_or_default, sha256_hex,
};
use crate::*;
use bijux_atlas_ops::kubernetes::render_policy::{
    validate_helm_dependencies, validate_render_output, RenderSurfaceTarget,
};
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::Path;

pub(crate) mod cluster_safety;
pub(crate) mod kubeconform;

use self::kubeconform::run_kubeconform_validation;

pub(crate) fn run_ops_render(args: &cli::OpsRenderArgs) -> Result<(String, i32), String> {
    let common = &args.common;
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
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

    let mut validation_errors = validate_render_output(
        &rendered_manifest,
        render_surface_target(args.target),
        &profile.name,
    );
    let mut kubeconform_result = None;
    if matches!(args.target, OpsRenderTarget::Helm) {
        validation_errors.extend(validate_helm_dependencies(&ops_root));
        if args.check {
            if common.allow_subprocess {
                let (kube_errors, result) =
                    run_kubeconform_validation(&process, &repo_root, &rendered_manifest)?;
                validation_errors.extend(kube_errors);
                kubeconform_result = Some(result);
            } else {
                kubeconform_result = Some(serde_json::json!({
                    "tool":"kubeconform",
                    "status":"skipped",
                    "reason":"kubeconform requires --allow-subprocess"
                }));
            }
        }
    }
    validation_errors.sort();
    validation_errors.dedup();

    if args.write && !common.allow_write {
        return Err(OpsCommandError::Effect(
            "ops render --write requires --allow-write".to_string(),
        )
        .to_stable_message());
    }
    let write_enabled = !(args.check || args.stdout);
    if write_enabled && !common.allow_write {
        return Err(
            OpsCommandError::Effect("ops render write requires --allow-write".to_string())
                .to_stable_message(),
        );
    }
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
    rows.sort_by(|a, b| {
        a.get("path")
            .and_then(Value::as_str)
            .cmp(&b.get("path").and_then(Value::as_str))
    });

    if write_enabled {
        let yaml_path = repo_root
            .join("artifacts/ops")
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
        let index_path = repo_root
            .join("artifacts/ops")
            .join(run_id.as_str())
            .join(&rel_index);
        if let Some(parent) = index_path.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                OpsCommandError::Manifest(format!("failed to create {}: {err}", parent.display()))
                    .to_stable_message()
            })?;
        }
        fs::write(
            &index_path,
            serde_json::to_string_pretty(&index_payload).map_err(|e| e.to_string())?,
        )
        .map_err(|err| {
            OpsCommandError::Manifest(format!("failed to write {}: {err}", index_path.display()))
                .to_stable_message()
        })?;
        written_files.push(
            index_path
                .strip_prefix(repo_root.join("artifacts/ops").join(run_id.as_str()))
                .unwrap_or(index_path.as_path())
                .display()
                .to_string(),
        );
    }
    let previous_hash = latest_render_hash(&repo_root, run_id.as_str(), &profile.name, target_name);
    if args.check {
        if let Some(previous_hash) = &previous_hash {
            if previous_hash != &render_sha {
                validation_errors.push(format!(
                    "render stability violation: previous_sha256={previous_hash} current_sha256={render_sha}"
                ));
            }
        }
    }
    let changed = previous_hash.as_deref().is_some_and(|v| v != render_sha);
    let diff = if args.diff {
        Some(serde_json::json!({
            "compared_against_previous_run": previous_hash.is_some(),
            "previous_sha256": previous_hash.clone(),
            "current_sha256": render_sha,
            "changed": changed
        }))
    } else {
        None
    };

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
            "evidence_mode": common.evidence,
            "write_enabled": write_enabled,
            "check_only": args.check,
            "stdout_mode": args.stdout,
            "diff_mode": args.diff,
            "diff_result": diff,
            "written_files": written_files.clone(),
            "render_index_files": rows.clone(),
            "validation_errors": validation_errors.clone(),
            "kubeconform": kubeconform_result.clone(),
            "subprocess_events": subprocess_events.clone()
        }],
        "summary": {
            "total": 1,
            "errors": if validation_errors.is_empty() { 0 } else { validation_errors.len() },
            "warnings": 0
        }
    });
    if common.evidence {
        if !common.allow_write {
            return Err(OpsCommandError::Effect(
                "ops render --evidence requires --allow-write".to_string(),
            )
            .to_stable_message());
        }
        let evidence_rel = format!(
            "artifacts/ops/evidence/{}/render-evidence.json",
            run_id.as_str()
        );
        let evidence_path = repo_root.join(&evidence_rel);
        if let Some(parent) = evidence_path.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                OpsCommandError::Manifest(format!("failed to create {}: {err}", parent.display()))
                    .to_stable_message()
            })?;
        }
        let evidence_payload = serde_json::json!({
            "schema_version": 1,
            "kind": "ops_render_evidence",
            "run_id": run_id.as_str(),
            "profile": profile.name,
            "target": target_name,
            "render_index_files": rows,
            "validation_errors": validation_errors,
            "kubeconform": payload["rows"][0]["kubeconform"],
            "written_files": written_files,
        });
        fs::write(
            &evidence_path,
            serde_json::to_string_pretty(&evidence_payload).map_err(|e| e.to_string())?,
        )
        .map_err(|err| {
            OpsCommandError::Manifest(format!(
                "failed to write {}: {err}",
                evidence_path.display()
            ))
            .to_stable_message()
        })?;
    }
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    let exit = if validation_errors.is_empty() { 0 } else { 1 };
    Ok((rendered, exit))
}

const fn render_surface_target(target: OpsRenderTarget) -> RenderSurfaceTarget {
    match target {
        OpsRenderTarget::Helm => RenderSurfaceTarget::Helm,
        OpsRenderTarget::Kustomize => RenderSurfaceTarget::Kustomize,
        OpsRenderTarget::Kind => RenderSurfaceTarget::Kind,
    }
}

fn latest_render_hash(
    repo_root: &Path,
    run_id: &str,
    profile: &str,
    target: &str,
) -> Option<String> {
    let root = repo_root.join("artifacts/ops");
    let mut candidates = fs::read_dir(root).ok()?;
    let mut runs = candidates
        .by_ref()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .filter(|p| p.file_name().and_then(|n| n.to_str()) != Some(run_id))
        .collect::<Vec<_>>();
    runs.sort();
    runs.reverse();
    for run in runs {
        let index_path = run.join(format!("render/{profile}/{target}/render.index.json"));
        let Ok(raw) = fs::read_to_string(index_path) else {
            continue;
        };
        let Ok(json) = serde_json::from_str::<Value>(&raw) else {
            continue;
        };
        if let Some(hash) = json
            .get("files")
            .and_then(Value::as_array)
            .and_then(|files| files.first())
            .and_then(|f| f.get("sha256"))
            .and_then(Value::as_str)
        {
            return Some(hash.to_string());
        }
    }
    None
}

fn render_profile_artifact_base(profile: &str, target: OpsRenderTarget) -> String {
    let target = match target {
        OpsRenderTarget::Helm => "helm",
        OpsRenderTarget::Kustomize => "kustomize",
        OpsRenderTarget::Kind => "kind",
    };
    format!("render/{profile}/{target}")
}

pub(crate) fn resolve_render_inputs(
    repo_root: &Path,
    run_id: &RunId,
    profile: &str,
) -> Result<(std::path::PathBuf, std::path::PathBuf), OpsCommandError> {
    let base = repo_root
        .join("artifacts/ops")
        .join(run_id.as_str())
        .join(format!("render/{profile}/helm"));
    let render_path = base.join("render.yaml");
    let index_path = base.join("render.index.json");
    if !render_path.exists() {
        return Err(OpsCommandError::Manifest(format!(
            "missing render artifact {}; run `bijux dev atlas ops render --target helm --allow-subprocess --allow-write --run-id {}` first",
            render_path.display(),
            run_id.as_str()
        )));
    }
    if !index_path.exists() {
        return Err(OpsCommandError::Manifest(format!(
            "missing render index {}",
            index_path.display()
        )));
    }
    Ok((render_path, index_path))
}
