// SPDX-License-Identifier: Apache-2.0

use super::*;

pub(super) fn run_boundary_helm_env_surface_check(
    ctx: &RunContext,
) -> (serde_json::Value, Vec<Violation>) {
    let contract_id = "REPO-003";
    let test_id = "repo.helm_env_surface.subset_of_runtime_allowlist";
    let mut violations = Vec::new();

    let output = match run_sanitized_output(
        &ctx.repo_root,
        "helm",
        &[
            "template",
            "atlas-default",
            "ops/k8s/charts/bijux-atlas",
            "-f",
            "ops/k8s/charts/bijux-atlas/values.yaml",
        ],
    ) {
        Ok(output) => output,
        Err(err) => {
            violations.push(violation(
                contract_id,
                test_id,
                Some("ops/k8s/charts/bijux-atlas".to_string()),
                err,
            ));
            let payload = serde_json::json!({
                "schema_version": 1,
                "check": "helm_env_surface",
                "status": "fail",
                "missing_env_keys": [],
                "emitted_env_keys": [],
            });
            return (payload, violations);
        }
    };
    if !output.status.success() {
        violations.push(violation(
            contract_id,
            test_id,
            Some("ops/k8s/charts/bijux-atlas".to_string()),
            format!(
                "helm template failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
    }

    let rendered = String::from_utf8_lossy(&output.stdout);
    let emitted = collect_rendered_env_keys(&rendered);
    let allowed = match read_env_allowlist(&ctx.repo_root) {
        Ok(allowed) => allowed,
        Err(err) => {
            violations.push(violation(
                contract_id,
                test_id,
                Some("configs/contracts/env.schema.json".to_string()),
                err,
            ));
            std::collections::BTreeSet::new()
        }
    };
    let missing = emitted.difference(&allowed).cloned().collect::<Vec<_>>();
    for env_key in &missing {
        violations.push(violation(
            contract_id,
            test_id,
            Some(env_key.clone()),
            "helm-emitted env key is missing from configs/contracts/env.schema.json",
        ));
    }

    let payload = serde_json::json!({
        "schema_version": 1,
        "check": "helm_env_surface",
        "status": if violations.is_empty() { "pass" } else { "fail" },
        "emitted_env_keys": emitted,
        "allowed_env_count": allowed.len(),
        "missing_env_keys": missing,
    });
    if let Err(err) = write_boundary_report(ctx, "helm-env-surface.json", &payload) {
        violations.push(violation(
            contract_id,
            test_id,
            Some("artifacts/contracts/repo/boundary-closure/helm-env-surface.json".to_string()),
            err,
        ));
    }
    (payload, violations)
}

pub(super) fn run_boundary_profile_render_matrix_check(
    ctx: &RunContext,
) -> (serde_json::Value, Vec<Violation>) {
    let contract_id = "REPO-004";
    let test_id = "repo.k8s_profile_render_matrix.installable_by_construction";
    let mut violations = Vec::new();
    if let Ok(missing_tools) = verify_declared_ops_tools(&ctx.repo_root, &["helm", "kubeconform"]) {
        for tool in missing_tools {
            violations.push(violation(
                contract_id,
                test_id,
                Some("ops/inventory/toolchain.json".to_string()),
                format!("ops toolchain inventory must declare `{tool}`"),
            ));
        }
    }

    let matrix_path = ctx.repo_root.join("ops/k8s/install-matrix.json");
    let matrix_text = match fs::read_to_string(&matrix_path) {
        Ok(text) => text,
        Err(err) => {
            violations.push(violation(
                contract_id,
                test_id,
                Some("ops/k8s/install-matrix.json".to_string()),
                format!("read failed: {err}"),
            ));
            let payload = serde_json::json!({
                "schema_version": 1,
                "check": "k8s_profile_render_matrix",
                "status": "fail",
                "profiles": [],
            });
            return (payload, violations);
        }
    };
    let matrix_json: serde_json::Value = match serde_json::from_str(&matrix_text) {
        Ok(json) => json,
        Err(err) => {
            violations.push(violation(
                contract_id,
                test_id,
                Some("ops/k8s/install-matrix.json".to_string()),
                format!("invalid json: {err}"),
            ));
            let payload = serde_json::json!({
                "schema_version": 1,
                "check": "k8s_profile_render_matrix",
                "status": "fail",
                "profiles": [],
            });
            return (payload, violations);
        }
    };

    let mut profile_rows = Vec::new();
    if let Some(profiles) = matrix_json.get("profiles").and_then(|value| value.as_array()) {
        for profile in profiles {
            let name = profile
                .get("name")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown");
            let values_rel = profile
                .get("values_file")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            let values_path = ctx.repo_root.join(values_rel);
            if !values_path.is_file() {
                violations.push(violation(
                    contract_id,
                    test_id,
                    Some(values_rel.to_string()),
                    "install profile values file is missing",
                ));
                continue;
            }

            let lint = run_sanitized_output(
                &ctx.repo_root,
                "helm",
                &["lint", "ops/k8s/charts/bijux-atlas", "-f", values_rel],
            );
            let lint_ok = lint.as_ref().is_ok_and(|output| output.status.success());
            if !lint_ok {
                violations.push(violation(
                    contract_id,
                    test_id,
                    Some(values_rel.to_string()),
                    "helm lint must pass for install profile",
                ));
            }

            let render = run_sanitized_output(
                &ctx.repo_root,
                "helm",
                &[
                    "template",
                    &format!("atlas-{name}"),
                    "ops/k8s/charts/bijux-atlas",
                    "--namespace",
                    "bijux-atlas",
                    "-f",
                    values_rel,
                ],
            );
            let render_ok = render.as_ref().is_ok_and(|output| output.status.success());
            let mut rendered_resources = 0usize;
            let mut kubeconform_ok = false;
            let mut kubeconform_note = String::new();
            if let Ok(render_output) = &render {
                if render_output.status.success() {
                    let rendered_manifest = String::from_utf8_lossy(&render_output.stdout);
                    rendered_resources =
                        serde_yaml::Deserializer::from_str(&rendered_manifest).count();
                    let temp_dir = ctx
                        .repo_root
                        .join("artifacts/contracts/repo/boundary-closure/tmp");
                    match fs::create_dir_all(&temp_dir) {
                        Ok(()) => {
                            let rendered_path = temp_dir.join(format!("{name}.yaml"));
                            match fs::write(&rendered_path, rendered_manifest.as_bytes()) {
                                Ok(()) => match run_sanitized_output(
                                    &ctx.repo_root,
                                    "kubeconform",
                                    &[
                                        "-strict",
                                        "-summary",
                                        "-ignore-missing-schemas",
                                        rendered_path.to_string_lossy().as_ref(),
                                    ],
                                ) {
                                    Ok(output) => {
                                        kubeconform_ok = output.status.success();
                                        if !kubeconform_ok {
                                            kubeconform_note =
                                                String::from_utf8_lossy(&output.stderr)
                                                    .trim()
                                                    .to_string();
                                        }
                                    }
                                    Err(err) => {
                                        kubeconform_note =
                                            format!("kubeconform failed to start: {err}");
                                    }
                                },
                                Err(err) => {
                                    kubeconform_note = format!(
                                        "failed to write rendered manifest {}: {err}",
                                        rendered_path.display()
                                    );
                                }
                            }
                        }
                        Err(err) => {
                            kubeconform_note = format!(
                                "failed to create temp render directory {}: {err}",
                                temp_dir.display()
                            );
                        }
                    }
                }
            }
            if !render_ok {
                violations.push(violation(
                    contract_id,
                    test_id,
                    Some(values_rel.to_string()),
                    "helm template must pass for install profile",
                ));
            }
            if render_ok && !kubeconform_ok {
                violations.push(violation(
                    contract_id,
                    test_id,
                    Some(values_rel.to_string()),
                    if kubeconform_note.is_empty() {
                        "kubeconform must pass for rendered install profile".to_string()
                    } else {
                        format!(
                            "kubeconform must pass for rendered install profile: {kubeconform_note}"
                        )
                    },
                ));
            }

            profile_rows.push(serde_json::json!({
                "profile": name,
                "values_file": values_rel,
                "helm_lint": lint_ok,
                "helm_template": render_ok,
                "kubeconform": kubeconform_ok,
                "rendered_resources": rendered_resources,
            }));
        }
    } else {
        violations.push(violation(
            contract_id,
            test_id,
            Some("ops/k8s/install-matrix.json".to_string()),
            "install-matrix must declare a profiles array",
        ));
    }

    let payload = serde_json::json!({
        "schema_version": 1,
        "check": "k8s_profile_render_matrix",
        "status": if violations.is_empty() { "pass" } else { "fail" },
        "profiles": profile_rows,
    });
    if let Err(err) = write_boundary_report(ctx, "k8s-profile-render-matrix.json", &payload) {
        violations.push(violation(
            contract_id,
            test_id,
            Some(
                "artifacts/contracts/repo/boundary-closure/k8s-profile-render-matrix.json"
                    .to_string(),
            ),
            err,
        ));
    }
    (payload, violations)
}
