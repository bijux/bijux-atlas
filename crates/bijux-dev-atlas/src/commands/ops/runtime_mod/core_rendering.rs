// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Clone, Copy)]
pub(super) enum ProfileValidationMode {
    SchemaOnly,
    KubeconformOnly,
    RolloutSafety,
    Policy,
    Resources,
    SecurityContext,
    ServiceMonitor,
    Hpa,
}

#[derive(Clone)]
struct ProfileValuesRow {
    id: String,
    class_name: String,
    safety_level: String,
    values: serde_json::Value,
}

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

pub(super) fn validate_profile_mode(
    args: &crate::cli::OpsProfileValidationArgs,
    mode: ProfileValidationMode,
) -> Result<(String, i32), String> {
    if matches!(
        mode,
        ProfileValidationMode::SchemaOnly
            | ProfileValidationMode::KubeconformOnly
            | ProfileValidationMode::RolloutSafety
    ) && !args.common.allow_subprocess
    {
        return Err("ops profile validation requires --allow-subprocess".to_string());
    }
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let ops_root = resolve_ops_root(&repo_root, args.common.ops_root.clone())
        .map_err(|e| e.to_stable_message())?;

    if matches!(
        mode,
        ProfileValidationMode::Policy
            | ProfileValidationMode::Resources
            | ProfileValidationMode::SecurityContext
            | ProfileValidationMode::ServiceMonitor
            | ProfileValidationMode::Hpa
    ) {
        return validate_profile_static_mode(args, &repo_root, &ops_root, mode);
    }

    let profile_set =
        matches!(mode, ProfileValidationMode::RolloutSafety).then(|| "rollout-safety".to_string());
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
            profile_set,
            timeout_seconds: args.timeout_seconds,
            run_kubeconform: true,
        },
    )?;

    let (rows, failures, kind) = match mode {
        ProfileValidationMode::SchemaOnly => {
            let rows = report
                .rows
                .iter()
                .map(|row| {
                    serde_json::json!({
                        "profile": row.profile,
                        "status": row.values_schema.status,
                        "errors": row.values_schema.errors
                    })
                })
                .collect::<Vec<_>>();
            (rows, report.summary.schema_failures, "ops_schema_validate")
        }
        ProfileValidationMode::KubeconformOnly => {
            let rows = report
                .rows
                .iter()
                .map(|row| {
                    serde_json::json!({
                        "profile": row.profile,
                        "status": row.kubeconform.status,
                        "errors": row.kubeconform.errors,
                        "resource_kind_summary": row.rendered_resource_kind_summary,
                        "resource_refs": row.rendered_resource_refs,
                    })
                })
                .collect::<Vec<_>>();
            (
                rows,
                report.summary.kubeconform_failures,
                "ops_kubeconform_validate",
            )
        }
        ProfileValidationMode::RolloutSafety => {
            let rows = report
                .rows
                .iter()
                .map(|row| {
                    serde_json::json!({
                        "profile": row.profile,
                        "status": row.rollout_safety.status,
                        "errors": row.rollout_safety.errors,
                        "helm_template": row.helm_template.status,
                        "values_schema": row.values_schema.status,
                        "kubeconform": row.kubeconform.status,
                        "resource_kind_summary": row.rendered_resource_kind_summary,
                    })
                })
                .collect::<Vec<_>>();
            (
                rows,
                report
                    .rows
                    .iter()
                    .filter(|row| row.rollout_safety.status == "fail")
                    .count(),
                "ops_rollout_safety_validate",
            )
        }
        ProfileValidationMode::Policy
        | ProfileValidationMode::Resources
        | ProfileValidationMode::SecurityContext
        | ProfileValidationMode::ServiceMonitor
        | ProfileValidationMode::Hpa => unreachable!("handled in static mode branch"),
    };

    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": kind,
        "profile_selector": report.inputs.profile_selector,
        "tooling": report.tooling,
        "rows": rows,
        "summary": {
            "total": report.rows.len(),
            "failures": failures,
            "status": if failures == 0 { "ok" } else { "failed" }
        }
    });
    let rendered = emit_payload(args.common.format, args.common.out.clone(), &payload)?;
    Ok((
        rendered,
        if failures == 0 {
            ops_exit::PASS
        } else {
            ops_exit::FAIL
        },
    ))
}

pub(super) fn run_profile_validation_pipeline(
    common: &OpsCommonArgs,
    repo_root: &Path,
    ops_root: &Path,
) -> Result<(serde_json::Value, i32), String> {
    if !common.allow_subprocess {
        return Err("ops validate requires --allow-subprocess".to_string());
    }
    let report = bijux_dev_atlas::ops::profiles_matrix::validate_profiles(
        repo_root,
        &bijux_dev_atlas::ops::profiles_matrix::ValidateProfilesOptions {
            chart_dir: ops_root.join("k8s/charts/bijux-atlas"),
            values_root: ops_root.join("k8s/values"),
            schema_path: ops_root.join("k8s/charts/bijux-atlas/values.schema.json"),
            dataset_manifest_path: ops_root.join("datasets/manifest.json"),
            install_matrix_path: ops_root.join("k8s/install-matrix.json"),
            rollout_safety_path: ops_root.join("k8s/rollout-safety-contract.json"),
            profile: common.profile.clone(),
            profile_set: None,
            timeout_seconds: 30,
            run_kubeconform: true,
        },
    )?;

    let rows = load_profile_values_rows(repo_root, ops_root, common.profile.as_deref())?;
    let hpa_policy_path = ops_root.join("stack/hpa-policy.json");
    let hpa_policy_json = std::fs::read_to_string(&hpa_policy_path)
        .map_err(|err| format!("failed to read {}: {err}", hpa_policy_path.display()))?;
    let hpa_policy_value: serde_json::Value = serde_json::from_str(&hpa_policy_json)
        .map_err(|err| format!("failed to parse {}: {err}", hpa_policy_path.display()))?;
    let mut max_by_class = std::collections::BTreeMap::new();
    if let Some(obj) = hpa_policy_value["max_replicas_by_class"].as_object() {
        for (class_name, value) in obj {
            if let Some(max) = value.as_u64() {
                max_by_class.insert(class_name.clone(), max);
            }
        }
    }

    let mut stages = vec![
        (
            "ops_render_validate".to_string(),
            report
                .rows
                .iter()
                .filter(|row| row.helm_template.status == "fail")
                .count(),
            report.rows.len(),
        ),
        (
            "ops_schema_validate".to_string(),
            report.summary.schema_failures,
            report.rows.len(),
        ),
        (
            "ops_kubeconform_validate".to_string(),
            report.summary.kubeconform_failures,
            report.rows.len(),
        ),
        (
            "ops_rollout_safety_validate".to_string(),
            report
                .rows
                .iter()
                .filter(|row| row.rollout_safety.status == "fail")
                .count(),
            report.rows.len(),
        ),
    ];
    let policy_failures = rows
        .iter()
        .filter(|profile| !validate_policy_rules(profile).is_empty())
        .count();
    stages.push((
        "ops_policy_validate".to_string(),
        policy_failures,
        rows.len(),
    ));
    let resource_failures = rows
        .iter()
        .filter(|profile| !validate_resource_rules(profile).is_empty())
        .count();
    stages.push((
        "ops_resource_validate".to_string(),
        resource_failures,
        rows.len(),
    ));
    let security_failures = rows
        .iter()
        .filter(|profile| !validate_security_context_rules(profile).is_empty())
        .count();
    stages.push((
        "ops_securitycontext_validate".to_string(),
        security_failures,
        rows.len(),
    ));
    let service_monitor_failures = rows
        .iter()
        .filter(|profile| !validate_service_monitor_rules(profile).is_empty())
        .count();
    stages.push((
        "ops_service_monitor_validate".to_string(),
        service_monitor_failures,
        rows.len(),
    ));
    let hpa_failures = rows
        .iter()
        .filter(|profile| !validate_hpa_rules(profile, &max_by_class).is_empty())
        .count();
    stages.push(("ops_hpa_validate".to_string(), hpa_failures, rows.len()));

    let total = stages.len();
    let failed = stages
        .iter()
        .filter(|(_, failures, _)| *failures > 0)
        .count();
    let passed = total.saturating_sub(failed);
    let mut lines = Vec::new();
    let width = total.to_string().len().max(2);
    for (index, (name, failures, profile_total)) in stages.iter().enumerate() {
        let status = if *failures == 0 { "PASS" } else { "FAIL" };
        lines.push(format!(
            "{status:>4} [  0.000s] ({:>width$}/{total}) {name} profiles={profile_total} failures={failures}",
            index + 1,
            width = width
        ));
    }
    lines.push(format!(
        "ops-validate-summary: total={total} passed={passed} failed={failed} skipped=0"
    ));
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "ops_validate_pipeline",
        "profile_selector": report.inputs.profile_selector,
        "rows": stages.iter().map(|(name, failures, profile_total)| serde_json::json!({
            "name": name,
            "status": if *failures == 0 {"pass"} else {"fail"},
            "profile_total": profile_total,
            "failures": failures
        })).collect::<Vec<_>>(),
        "summary": {
            "total": total,
            "passed": passed,
            "failed": failed,
            "skipped": 0
        },
        "text": lines.join("\n")
    });
    let exit = if failed == 0 {
        ops_exit::PASS
    } else {
        ops_exit::FAIL
    };
    Ok((payload, exit))
}

fn merge_values(base: &mut serde_json::Value, overlay: serde_json::Value) {
    match (base, overlay) {
        (serde_json::Value::Object(base_obj), serde_json::Value::Object(overlay_obj)) => {
            for (key, overlay_value) in overlay_obj {
                match base_obj.get_mut(&key) {
                    Some(base_value) => merge_values(base_value, overlay_value),
                    None => {
                        base_obj.insert(key, overlay_value);
                    }
                }
            }
        }
        (base_slot, overlay_value) => *base_slot = overlay_value,
    }
}

fn load_profile_values_rows(
    repo_root: &Path,
    ops_root: &Path,
    selected_profile: Option<&str>,
) -> Result<Vec<ProfileValuesRow>, String> {
    let mut registry =
        crate::ops_support::load_profile_registry(ops_root).map_err(|e| e.to_stable_message())?;
    registry.profiles.sort_by(|a, b| a.id.cmp(&b.id));
    let chart_values_path = ops_root.join("k8s/charts/bijux-atlas/values.yaml");
    let chart_values = std::fs::read_to_string(&chart_values_path)
        .map_err(|err| format!("failed to read {}: {err}", chart_values_path.display()))?;
    let base_values_yaml: serde_yaml::Value = serde_yaml::from_str(&chart_values)
        .map_err(|err| format!("failed to parse {}: {err}", chart_values_path.display()))?;
    let base_values = serde_json::to_value(base_values_yaml).map_err(|err| {
        format!(
            "failed to convert {} to json: {err}",
            chart_values_path.display()
        )
    })?;
    let mut rows = Vec::new();
    for profile in registry.profiles {
        if let Some(filter) = selected_profile {
            if profile.id != filter {
                continue;
            }
        }
        let overlay_path = profile
            .config_source_paths
            .iter()
            .find_map(|path| {
                if path.ends_with(".yaml") {
                    let absolute = repo_root.join(path);
                    absolute.is_file().then_some(absolute)
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                format!(
                    "OPS_PROFILE_ERROR: profile `{}` has no existing values yaml in config_source_paths",
                    profile.id
                )
            })?;
        let overlay_text = std::fs::read_to_string(&overlay_path)
            .map_err(|err| format!("failed to read {}: {err}", overlay_path.display()))?;
        let overlay_yaml: serde_yaml::Value = serde_yaml::from_str(&overlay_text)
            .map_err(|err| format!("failed to parse {}: {err}", overlay_path.display()))?;
        let overlay = serde_json::to_value(overlay_yaml).map_err(|err| {
            format!(
                "failed to convert {} to json: {err}",
                overlay_path.display()
            )
        })?;
        let mut merged = base_values.clone();
        merge_values(&mut merged, overlay);
        rows.push(ProfileValuesRow {
            id: profile.id,
            class_name: profile.class_name,
            safety_level: profile.safety_level,
            values: merged,
        });
    }
    if rows.is_empty() {
        return Err("OPS_PROFILE_ERROR: selected profile set is empty".to_string());
    }
    Ok(rows)
}

fn build_row(profile: &ProfileValuesRow, errors: Vec<String>) -> serde_json::Value {
    serde_json::json!({
        "profile": profile.id,
        "class": profile.class_name,
        "safety_level": profile.safety_level,
        "status": if errors.is_empty() { "pass" } else { "fail" },
        "errors": errors
    })
}

fn parse_duration_seconds(value: Option<&str>) -> Option<u64> {
    let raw = value?;
    let trimmed = raw.trim();
    let number = trimmed.strip_suffix('s').unwrap_or(trimmed);
    number.parse::<u64>().ok()
}

fn validate_policy_rules(profile: &ProfileValuesRow) -> Vec<String> {
    let mut errors = Vec::new();
    let network_policy = &profile.values["networkPolicy"];
    let class_is_prod = profile.class_name == "prod";
    if class_is_prod {
        if network_policy["ingress"]["mode"].as_str().is_none() {
            errors.push("$.networkPolicy.ingress.mode: required for prod class".to_string());
        }
        if network_policy["egress"]["mode"].as_str().is_none() {
            errors.push("$.networkPolicy.egress.mode: required for prod class".to_string());
        }
        if network_policy["enabled"].as_bool() != Some(true) {
            errors.push(
                "$.networkPolicy.enabled: prod class requires networkPolicy.enabled=true"
                    .to_string(),
            );
        }
    }
    let service_type = profile.values["service"]["type"]
        .as_str()
        .unwrap_or("ClusterIP");
    if service_type == "NodePort" && profile.class_name != "dev" {
        errors.push("$.service.type: NodePort is only allowed for dev class profiles".to_string());
    }
    let allowed_ports = [80u64, 443, 8080, 6379, 9000, 4317, 4318];
    if let Some(port) = profile.values["service"]["port"].as_u64() {
        if !allowed_ports.contains(&port) {
            errors.push(format!(
                "$.service.port: port `{port}` is outside the allowed service port list"
            ));
        }
    }
    errors
}

fn validate_resource_rules(profile: &ProfileValuesRow) -> Vec<String> {
    let mut errors = Vec::new();
    let class_is_prod = profile.class_name == "prod";
    let resources = &profile.values["resources"];
    if class_is_prod {
        if resources["requests"]["cpu"].as_str().is_none() {
            errors.push("$.resources.requests.cpu: required for prod class".to_string());
        }
        if resources["requests"]["memory"].as_str().is_none() {
            errors.push("$.resources.requests.memory: required for prod class".to_string());
        }
        if resources["limits"]["cpu"].as_str().is_none() {
            errors.push("$.resources.limits.cpu: required for prod class".to_string());
        }
        if resources["limits"]["memory"].as_str().is_none() {
            errors.push("$.resources.limits.memory: required for prod class".to_string());
        }
    }
    if profile.values["cache"]["initPrewarm"]["enabled"].as_bool() == Some(true)
        && resources["requests"]["ephemeral-storage"]
            .as_str()
            .is_none()
    {
        errors.push(
            "$.resources.requests.ephemeral-storage: required when cache.initPrewarm.enabled=true"
                .to_string(),
        );
    }
    if class_is_prod {
        if let Some(init_containers) = profile.values["initContainers"].as_array() {
            for (index, init_container) in init_containers.iter().enumerate() {
                if init_container["resources"]["requests"]["cpu"]
                    .as_str()
                    .is_none()
                    || init_container["resources"]["requests"]["memory"]
                        .as_str()
                        .is_none()
                {
                    errors.push(format!(
                        "$.initContainers[{index}].resources.requests: cpu and memory required for prod class"
                    ));
                }
            }
        }
    }
    errors
}

fn validate_security_context_rules(profile: &ProfileValuesRow) -> Vec<String> {
    let mut errors = Vec::new();
    let sec = &profile.values["securityContext"];
    let class_is_prod = profile.class_name == "prod";
    if class_is_prod && sec["runAsNonRoot"].as_bool() != Some(true) {
        errors.push("$.securityContext.runAsNonRoot: required=true for prod class".to_string());
    }
    if class_is_prod && sec["readOnlyRootFilesystem"].as_bool() != Some(true) {
        errors.push(
            "$.securityContext.readOnlyRootFilesystem: required=true for prod class".to_string(),
        );
    }
    if class_is_prod {
        let drop_all = sec["capabilities"]["drop"]
            .as_array()
            .is_some_and(|items| items.iter().any(|v| v.as_str() == Some("ALL")));
        if !drop_all {
            errors.push(
                "$.securityContext.capabilities.drop: must include `ALL` for prod class"
                    .to_string(),
            );
        }
    }
    if sec["privileged"].as_bool() == Some(true) {
        errors
            .push("$.securityContext.privileged: privileged containers are forbidden".to_string());
    }
    errors
}

fn validate_service_monitor_rules(profile: &ProfileValuesRow) -> Vec<String> {
    let mut errors = Vec::new();
    let svc_mon = &profile.values["metrics"]["serviceMonitor"];
    if svc_mon["enabled"].as_bool() == Some(true) {
        if !svc_mon["labels"].is_object() {
            errors.push("$.metrics.serviceMonitor.labels: labels object is required when ServiceMonitor is enabled".to_string());
        }
        let interval = parse_duration_seconds(svc_mon["interval"].as_str());
        let expected_max = if profile.class_name == "prod" || profile.class_name == "stage" {
            15
        } else {
            60
        };
        if let Some(seconds) = interval {
            if seconds > expected_max {
                errors.push(format!(
                    "$.metrics.serviceMonitor.interval: `{seconds}s` exceeds class limit `{expected_max}s`"
                ));
            }
        } else {
            errors.push(
                "$.metrics.serviceMonitor.interval: required duration format like `15s`"
                    .to_string(),
            );
        }
        if profile.values["service"]["type"].as_str() == Some("LoadBalancer") {
            errors.push(
                "$.service.type: LoadBalancer is not allowed when ServiceMonitor is enabled"
                    .to_string(),
            );
        }
    }
    errors
}

fn validate_hpa_rules(
    profile: &ProfileValuesRow,
    max_by_class: &std::collections::BTreeMap<String, u64>,
) -> Vec<String> {
    let mut errors = Vec::new();
    let hpa = &profile.values["hpa"];
    if hpa["enabled"].as_bool() != Some(true) {
        return errors;
    }
    let min = hpa["minReplicas"].as_u64();
    let max = hpa["maxReplicas"].as_u64();
    let target = hpa["cpuUtilization"].as_u64();
    if target.is_none() {
        errors.push("$.hpa.cpuUtilization: required when hpa.enabled=true".to_string());
    }
    if profile.class_name == "prod" && min.unwrap_or(0) < 2 {
        errors.push("$.hpa.minReplicas: prod class requires minReplicas >= 2".to_string());
    }
    if let Some(class_max) = max_by_class.get(&profile.class_name) {
        if max.unwrap_or(0) > *class_max {
            errors.push(format!(
                "$.hpa.maxReplicas: `{}` exceeds class max `{}`",
                max.unwrap_or(0),
                class_max
            ));
        }
    }
    if let Some(util) = target {
        if !(30..=90).contains(&util) {
            errors.push(format!(
                "$.hpa.cpuUtilization: `{util}` outside sane range [30, 90]"
            ));
        }
    }
    errors
}

fn validate_profile_static_mode(
    args: &crate::cli::OpsProfileValidationArgs,
    repo_root: &Path,
    ops_root: &Path,
    mode: ProfileValidationMode,
) -> Result<(String, i32), String> {
    let rows = load_profile_values_rows(repo_root, ops_root, args.common.profile.as_deref())?;
    let hpa_policy_path = ops_root.join("stack/hpa-policy.json");
    let hpa_policy_json = std::fs::read_to_string(&hpa_policy_path)
        .map_err(|err| format!("failed to read {}: {err}", hpa_policy_path.display()))?;
    let hpa_policy_value: serde_json::Value = serde_json::from_str(&hpa_policy_json)
        .map_err(|err| format!("failed to parse {}: {err}", hpa_policy_path.display()))?;
    let mut max_by_class = std::collections::BTreeMap::new();
    if let Some(obj) = hpa_policy_value["max_replicas_by_class"].as_object() {
        for (class_name, value) in obj {
            if let Some(max) = value.as_u64() {
                max_by_class.insert(class_name.clone(), max);
            }
        }
    }

    let (kind, evaluated_rows): (&str, Vec<serde_json::Value>) = match mode {
        ProfileValidationMode::Policy => (
            "ops_policy_validate",
            rows.iter()
                .map(|profile| build_row(profile, validate_policy_rules(profile)))
                .collect(),
        ),
        ProfileValidationMode::Resources => (
            "ops_resource_validate",
            rows.iter()
                .map(|profile| build_row(profile, validate_resource_rules(profile)))
                .collect(),
        ),
        ProfileValidationMode::SecurityContext => (
            "ops_securitycontext_validate",
            rows.iter()
                .map(|profile| build_row(profile, validate_security_context_rules(profile)))
                .collect(),
        ),
        ProfileValidationMode::ServiceMonitor => (
            "ops_service_monitor_validate",
            rows.iter()
                .map(|profile| build_row(profile, validate_service_monitor_rules(profile)))
                .collect(),
        ),
        ProfileValidationMode::Hpa => (
            "ops_hpa_validate",
            rows.iter()
                .map(|profile| build_row(profile, validate_hpa_rules(profile, &max_by_class)))
                .collect(),
        ),
        ProfileValidationMode::SchemaOnly
        | ProfileValidationMode::KubeconformOnly
        | ProfileValidationMode::RolloutSafety => unreachable!("handled before static mode"),
    };

    let failures = evaluated_rows
        .iter()
        .filter(|row| row["status"].as_str() == Some("fail"))
        .count();
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": kind,
        "profile_selector": args.common.profile.clone().unwrap_or_else(|| "all".to_string()),
        "rows": evaluated_rows,
        "summary": {
            "total": rows.len(),
            "failures": failures,
            "status": if failures == 0 { "ok" } else { "failed" }
        }
    });
    let rendered = emit_payload(args.common.format, args.common.out.clone(), &payload)?;
    Ok((
        rendered,
        if failures == 0 {
            ops_exit::PASS
        } else {
            ops_exit::FAIL
        },
    ))
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
