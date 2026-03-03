// SPDX-License-Identifier: Apache-2.0

use super::*;

pub(super) fn render_helm_env_surface(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    if !common.allow_subprocess {
        return Err("ops k8s env-surface requires --allow-subprocess".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let process = OpsProcess::new(common.allow_subprocess);
    let chart_path = ops_root.join("k8s/charts/bijux-atlas");
    let values_path = chart_path.join("values.yaml");
    let cmd_args = vec![
        "template".to_string(),
        "atlas-default".to_string(),
        chart_path.display().to_string(),
        "-f".to_string(),
        values_path.display().to_string(),
    ];
    let (stdout, event) = process
        .run_subprocess("helm", &cmd_args, &repo_root)
        .map_err(|e| e.to_stable_message())?;
    let env_keys = collect_rendered_env_keys(&stdout);
    let rows = env_keys
        .iter()
        .map(|name| serde_json::json!({"env_key": name}))
        .collect::<Vec<_>>();
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "ops_k8s_env_surface",
        "text": format!("rendered {} helm-emitted env keys", env_keys.len()),
        "rows": rows,
        "summary": {"total": env_keys.len(), "errors": 0, "warnings": 0},
        "env_keys": env_keys,
        "subprocess_events": [event]
    });
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, ops_exit::PASS))
}

pub(super) fn render_helm_configmap_env_report(
    args: &crate::cli::OpsHelmEnvArgs,
) -> Result<(String, i32), String> {
    if !args.common.allow_subprocess {
        return Err("ops helm-env requires --allow-subprocess".to_string());
    }
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let helm_binary =
        bijux_dev_atlas::ops::helm_env::resolve_helm_binary_from_inventory(&repo_root)?;
    let release_name = args.release_name.clone().unwrap_or_else(|| {
        args.chart
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("bijux-atlas")
            .to_string()
    });
    let render_result = bijux_dev_atlas::ops::helm_env::render_chart_with_options(
        &repo_root,
        &helm_binary,
        &args.chart,
        &args.values_files,
        &release_name,
        &bijux_dev_atlas::ops::helm_env::RenderChartOptions {
            set_overrides: args.set_overrides.clone(),
            timeout_seconds: args.timeout_seconds,
            debug: args.verbose,
        },
    );
    let (config_maps, env_keys, helm_report, exit_code) = match render_result {
        Ok(rendered_chart) => {
            let config_maps = bijux_dev_atlas::ops::helm_env::extract_configmap_rows(
                &rendered_chart.yaml_docs,
                &release_name,
            );
            let env_keys = bijux_dev_atlas::ops::helm_env::extract_configmap_env_keys(
                &rendered_chart.yaml_docs,
                &release_name,
            );
            (
                config_maps,
                env_keys,
                bijux_dev_atlas::ops::helm_env::HelmInvocationReport {
                    status: "ok".to_string(),
                    debug_enabled: rendered_chart.debug_enabled,
                    timeout_seconds: rendered_chart.timeout_seconds,
                    stderr: rendered_chart.stderr,
                },
                ops_exit::PASS,
            )
        }
        Err(message) => (
            Vec::new(),
            std::collections::BTreeSet::new(),
            bijux_dev_atlas::ops::helm_env::HelmInvocationReport {
                status: "error".to_string(),
                debug_enabled: args.verbose,
                timeout_seconds: args.timeout_seconds.max(1),
                stderr: message,
            },
            ops_exit::FAIL,
        ),
    };
    if exit_code == ops_exit::PASS && args.fail_on_empty && env_keys.is_empty() {
        return Err(format!(
            "no ATLAS_ or BIJUX_ ConfigMap data keys extracted for release `{release_name}`"
        ));
    }
    let report = bijux_dev_atlas::ops::helm_env::build_report(
        bijux_dev_atlas::ops::helm_env::build_inputs(
            &args.chart,
            &args.values_files,
            &release_name,
            &helm_binary,
        ),
        &env_keys,
        &config_maps,
        args.include_names,
        helm_report,
    );
    let payload = serde_json::to_value(&report).map_err(|err| err.to_string())?;
    let schema_path = repo_root.join("configs/contracts/reports/helm-env.schema.json");
    bijux_dev_atlas::ops::helm_env::validate_report_value(&payload, &schema_path)?;
    let rendered = emit_payload(args.common.format, args.common.out.clone(), &payload)?;
    Ok((rendered, exit_code))
}

pub(super) fn validate_helm_profile_matrix(
    args: &crate::cli::OpsProfilesValidateArgs,
) -> Result<(String, i32), String> {
    if !args.common.allow_subprocess {
        return Err("ops k8s validate-profiles requires --allow-subprocess".to_string());
    }
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let ops_root = resolve_ops_root(&repo_root, args.common.ops_root.clone())
        .map_err(|e| e.to_stable_message())?;
    let report = bijux_dev_atlas::ops::profiles_matrix::validate_profiles(
        &repo_root,
        &bijux_dev_atlas::ops::profiles_matrix::ValidateProfilesOptions {
            chart_dir: ops_root.join("k8s/charts/bijux-atlas"),
            values_root: ops_root.join("k8s/values"),
            schema_path: ops_root.join("k8s/charts/bijux-atlas/values.schema.json"),
            dataset_manifest_path: ops_root.join("datasets/manifest.json"),
            install_matrix_path: ops_root.join("k8s/install-matrix.json"),
            rollout_safety_path: ops_root.join("k8s/rollout-safety-contract.json"),
            profile: args.common.profile.clone(),
            profile_set: args.profile_set.clone(),
            timeout_seconds: args.timeout_seconds,
            run_kubeconform: args.kubeconform,
        },
    )?;
    let payload = serde_json::to_value(&report).map_err(|err| err.to_string())?;
    let schema_path = repo_root.join("configs/contracts/reports/ops-profiles.schema.json");
    bijux_dev_atlas::ops::profiles_matrix::validate_report_value(&payload, &schema_path)?;
    let exe =
        std::env::current_exe().map_err(|err| format!("ops profiles validate failed: {err}"))?;
    let mut deprecations_args = vec![
        "governance".to_string(),
        "deprecations".to_string(),
        "validate".to_string(),
        "--format".to_string(),
        "json".to_string(),
    ];
    deprecations_args.push("--repo-root".to_string());
    deprecations_args.push(repo_root.display().to_string());
    let deprecations_out = std::process::Command::new(exe)
        .args(&deprecations_args)
        .output()
        .map_err(|err| format!("ops profiles validate failed: {err}"))?;
    if !deprecations_out.status.success() {
        return Err(format!(
            "ops profiles validate failed: governance deprecations validate returned {}",
            deprecations_out.status
        ));
    }
    let rendered = emit_payload(args.common.format, args.common.out.clone(), &payload)?;
    let exit = if report.summary.helm_failures == 0
        && report.summary.schema_failures == 0
        && report.summary.kubeconform_failures == 0
    {
        ops_exit::PASS
    } else {
        ops_exit::FAIL
    };
    Ok((rendered, exit))
}

fn collect_rendered_env_keys(rendered_yaml: &str) -> std::collections::BTreeSet<String> {
    fn collect_from_value(
        value: &serde_yaml::Value,
        env_keys: &mut std::collections::BTreeSet<String>,
    ) {
        match value {
            serde_yaml::Value::Mapping(map) => {
                for (key, child) in map {
                    if let Some(key_text) = key.as_str() {
                        if (key_text.starts_with("ATLAS_") || key_text.starts_with("BIJUX_"))
                            && key_text.len() > "ATLAS_".len()
                        {
                            env_keys.insert(key_text.to_string());
                        }
                        if key_text == "name" {
                            if let Some(env_name) = child.as_str() {
                                if (env_name.starts_with("ATLAS_")
                                    || env_name.starts_with("BIJUX_"))
                                    && env_name.len() > "ATLAS_".len()
                                {
                                    env_keys.insert(env_name.to_string());
                                }
                            }
                        }
                    }
                    collect_from_value(child, env_keys);
                }
            }
            serde_yaml::Value::Sequence(items) => {
                for child in items {
                    collect_from_value(child, env_keys);
                }
            }
            _ => {}
        }
    }

    let mut env_keys = std::collections::BTreeSet::<String>::new();
    for document in serde_yaml::Deserializer::from_str(rendered_yaml) {
        let value = match serde_yaml::Value::deserialize(document) {
            Ok(value) => value,
            Err(_) => continue,
        };
        collect_from_value(&value, &mut env_keys);
    }
    env_keys
}
