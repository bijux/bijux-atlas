// SPDX-License-Identifier: Apache-2.0

use super::*;

pub(super) fn validate_render_output(
    rendered: &str,
    target: OpsRenderTarget,
    profile_name: &str,
) -> Vec<String> {
    let mut errors = Vec::new();
    let required_kinds = match target {
        OpsRenderTarget::Helm => ["Deployment", "Service"].to_vec(),
        OpsRenderTarget::Kind | OpsRenderTarget::Kustomize => Vec::new(),
    };
    for kind in required_kinds {
        let needle = format!("kind: {kind}");
        if !rendered.contains(&needle) {
            errors.push(format!("missing required rendered resource `{needle}`"));
        }
    }
    errors.extend(scan_forbidden_kinds(rendered));
    errors.extend(scan_unpinned_images(rendered, profile_name));
    errors.extend(scan_invalid_image_refs(rendered));
    errors.extend(scan_invalid_runbook_urls(rendered));
    errors.extend(scan_alert_annotation_contract(rendered));
    errors.extend(scan_timestamps(rendered));
    errors.sort();
    errors.dedup();
    errors
}

pub(super) fn run_kubeconform_validation(
    process: &OpsProcess,
    repo_root: &Path,
    rendered: &str,
) -> Result<(Vec<String>, Value), String> {
    let tmp_dir = repo_root.join("artifacts/tmp/k8s-validate");
    fs::create_dir_all(&tmp_dir)
        .map_err(|err| format!("failed to create {}: {err}", tmp_dir.display()))?;
    let manifest_path = tmp_dir.join("rendered.yaml");
    fs::write(&manifest_path, rendered)
        .map_err(|err| format!("failed to write {}: {err}", manifest_path.display()))?;
    let args = vec![
        "-strict".to_string(),
        "-ignore-missing-schemas".to_string(),
        "-summary".to_string(),
        manifest_path.display().to_string(),
    ];
    match process.run_subprocess("kubeconform", &args, repo_root) {
        Ok((stdout, event)) => Ok((
            Vec::new(),
            serde_json::json!({
                "tool":"kubeconform",
                "status":"ok",
                "stdout": stdout,
                "subprocess_event": event
            }),
        )),
        Err(err) => {
            let message = err.to_stable_message();
            Ok((
                vec![format!("kubeconform validation failed: {message}")],
                serde_json::json!({
                    "tool":"kubeconform",
                    "status":"failed",
                    "error": message
                }),
            ))
        }
    }
}

pub(crate) fn scan_timestamps(rendered: &str) -> Vec<String> {
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

fn profile_requires_digest_pins(profile_name: &str) -> bool {
    matches!(
        profile_name,
        "prod" | "prod-minimal" | "prod-ha" | "prod-airgap"
    )
}

pub(crate) fn scan_unpinned_images(rendered: &str, profile_name: &str) -> Vec<String> {
    let mut errors = Vec::new();
    if !profile_requires_digest_pins(profile_name) {
        return errors;
    }
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

fn scan_alert_annotation_contract(rendered: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let mut current_alert: Option<String> = None;
    let mut has_severity = false;
    let mut has_owner = false;
    let mut has_runbook = false;
    let flush = |errors: &mut Vec<String>,
                 current_alert: &Option<String>,
                 has_severity: bool,
                 has_owner: bool,
                 has_runbook: bool| {
        if let Some(alert) = current_alert {
            if !has_severity {
                errors.push(format!(
                    "rendered alert `{alert}` is missing required label `severity`"
                ));
            }
            if !has_owner {
                errors.push(format!(
                    "rendered alert `{alert}` is missing required label `owner`"
                ));
            }
            if !has_runbook {
                errors.push(format!(
                    "rendered alert `{alert}` is missing required annotation `runbook`"
                ));
            }
        }
    };

    for line in rendered.lines() {
        let trimmed = line.trim();
        if let Some(name) = trimmed.strip_prefix("- alert:") {
            flush(
                &mut errors,
                &current_alert,
                has_severity,
                has_owner,
                has_runbook,
            );
            current_alert = Some(name.trim().to_string());
            has_severity = false;
            has_owner = false;
            has_runbook = false;
            continue;
        }
        if current_alert.is_none() {
            continue;
        }
        if trimmed.starts_with("severity:") {
            has_severity = true;
        } else if trimmed.starts_with("owner:") {
            has_owner = true;
        } else if trimmed.starts_with("runbook:") {
            has_runbook = true;
        }
    }
    flush(
        &mut errors,
        &current_alert,
        has_severity,
        has_owner,
        has_runbook,
    );
    errors
}

pub(crate) fn scan_forbidden_kinds(rendered: &str) -> Vec<String> {
    let mut errors = Vec::new();
    if rendered.contains("kind: ClusterRole") {
        errors.push("rendered output includes forbidden resource `kind: ClusterRole`".to_string());
    }
    errors
}

pub(super) fn validate_helm_dependencies(ops_root: &Path) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use super::{scan_alert_annotation_contract, scan_invalid_image_refs, scan_unpinned_images};

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
        let errors = scan_unpinned_images(rendered, "prod");
        assert!(
            errors.is_empty(),
            "expected digest pinned image, got {errors:?}"
        );
    }

    #[test]
    fn rendered_alert_requires_owner_and_severity_and_runbook() {
        let rendered = r#"
        - alert: BijuxAtlasHigh5xxRate
          labels:
            severity: page
          annotations:
            runbook: https://docs.example/runbook
        "#;
        let errors = scan_alert_annotation_contract(rendered);
        assert!(
            errors
                .iter()
                .any(|e| e.contains("missing required label `owner`")),
            "expected owner contract violation, got {errors:?}"
        );
    }
}
