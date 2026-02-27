// SPDX-License-Identifier: Apache-2.0

use crate::ops_support::{load_load_manifest, validate_load_manifest};
use crate::ops_commands::{
    emit_payload, load_profiles, resolve_ops_root, resolve_profile, run_id_or_default, sha256_hex,
};
use crate::*;
use serde_json::Value;
use std::io::Write;
use std::time::Instant;

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
            "write_enabled": write_enabled,
            "check_only": args.check,
            "stdout_mode": args.stdout,
            "diff_mode": args.diff,
            "diff_result": diff,
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
    errors.extend(scan_forbidden_kinds(rendered));
    errors.extend(scan_unpinned_images(rendered));
    errors.extend(scan_invalid_image_refs(rendered));
    errors.extend(scan_invalid_runbook_urls(rendered));
    errors.extend(scan_timestamps(rendered));
    errors.sort();
    errors.dedup();
    errors
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

fn scan_timestamps(rendered: &str) -> Vec<String> {
    let mut errors = Vec::new();
    for marker in ["generatedAt:", "timestamp:", "creationTimestamp:"] {
        if rendered.contains(marker) {
            errors.push(format!(
                "render output contains forbidden timestamp marker `{marker}`"
            ));
        }
    }
    errors
}

fn scan_unpinned_images(rendered: &str) -> Vec<String> {
    let mut errors = Vec::new();
    for line in rendered.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("image:") {
            continue;
        }
        if trimmed.contains(":latest") {
            errors.push(format!(
                "rendered image uses forbidden latest tag: {trimmed}"
            ));
            continue;
        }
        if !trimmed.contains("@sha256:") {
            errors.push(format!("rendered image is not digest pinned: {trimmed}"));
        }
    }
    errors
}

fn scan_invalid_image_refs(rendered: &str) -> Vec<String> {
    let mut errors = Vec::new();
    for line in rendered.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("image:") {
            continue;
        }
        let ref_value = trimmed
            .trim_start_matches("image:")
            .trim()
            .trim_matches('"')
            .trim_matches('\'');
        let at_count = ref_value.matches('@').count();
        if at_count > 1 {
            errors.push(format!(
                "rendered image contains multiple digest separators: {trimmed}"
            ));
        }
        if at_count == 1 && !ref_value.contains("@sha256:") {
            errors.push(format!(
                "rendered image uses invalid digest format (expected @sha256:...): {trimmed}"
            ));
        }
    }
    errors
}

fn scan_invalid_runbook_urls(rendered: &str) -> Vec<String> {
    let mut errors = Vec::new();
    for line in rendered.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("runbook:") {
            continue;
        }
        let value = trimmed
            .trim_start_matches("runbook:")
            .trim()
            .trim_matches('"')
            .trim_matches('\'');
        if !(value.starts_with("https://") || value.starts_with("http://")) {
            errors.push(format!(
                "rendered alert runbook must be absolute URL: {trimmed}"
            ));
        }
    }
    errors
}

fn scan_forbidden_kinds(rendered: &str) -> Vec<String> {
    let mut errors = Vec::new();
    if rendered.contains("kind: ClusterRole") {
        errors.push("rendered output includes forbidden resource `kind: ClusterRole`".to_string());
    }
    errors
}

#[cfg(test)]
mod render_tests {
    use super::{scan_invalid_image_refs, scan_unpinned_images};

    #[test]
    fn rendered_image_reference_must_not_have_multiple_digest_separators() {
        let rendered = "image: ghcr.io/bijux/bijux-atlas@sha256:abc@sha256:def";
        let errors = scan_invalid_image_refs(rendered);
        assert!(
            errors
                .iter()
                .any(|e| e.contains("multiple digest separators")),
            "expected invalid image reference error, got {errors:?}"
        );
    }

    #[test]
    fn rendered_image_reference_accepts_digest_form() {
        let rendered = "image: ghcr.io/bijux/bijux-atlas@sha256:1111111111111111111111111111111111111111111111111111111111111111";
        let errors = scan_unpinned_images(rendered);
        assert!(errors.is_empty(), "expected digest pinned image, got {errors:?}");
    }
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
    if is_context_allowed(&expected, current, force) {
        Ok(())
    } else {
        Err(OpsCommandError::Effect(format!(
            "kubectl context guard failed: expected `{expected}` got `{current}`; pass --force to override"
        )))
    }
}

fn is_context_allowed(expected: &str, current: &str, force: bool) -> bool {
    current == expected || force
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

fn ensure_k8s_safety(
    process: &OpsProcess,
    repo_root: &Path,
    profile: &StackProfile,
    force: bool,
    namespace: &str,
) -> Result<(), OpsCommandError> {
    ensure_kind_context(process, profile, force)?;
    let args = vec![
        "get".to_string(),
        "namespace".to_string(),
        namespace.to_string(),
        "-o".to_string(),
        "name".to_string(),
    ];
    process
        .run_subprocess("kubectl", &args, repo_root)
        .map(|_| ())
        .map_err(|e| {
            OpsCommandError::Effect(format!(
                "namespace guard failed for `{namespace}`: {}",
                e.to_stable_message()
            ))
        })
}

fn resolve_render_inputs(
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
