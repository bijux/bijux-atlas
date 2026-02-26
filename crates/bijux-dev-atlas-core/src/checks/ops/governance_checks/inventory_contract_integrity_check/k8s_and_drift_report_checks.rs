fn validate_k8s_rollout_and_drift_reports(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
    k8s_install_matrix_rel: &Path,
    k8s_rollout_contract_rel: &Path,
    stack_drift_rel: &Path,
) -> Result<(), CheckError> {
    if ctx.adapters.fs.exists(ctx.repo_root, k8s_install_matrix_rel)
        && ctx.adapters.fs.exists(ctx.repo_root, k8s_rollout_contract_rel)
    {
        let install_matrix_text = fs::read_to_string(ctx.repo_root.join(k8s_install_matrix_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let install_matrix_json: serde_json::Value = serde_json::from_str(&install_matrix_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let matrix_profiles = install_matrix_json
            .get("profiles")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let matrix_by_name = matrix_profiles
            .iter()
            .filter_map(|entry| {
                let name = entry.get("name").and_then(|v| v.as_str())?;
                Some((name.to_string(), entry.clone()))
            })
            .collect::<BTreeMap<_, _>>();

        let rollout_contract_text = fs::read_to_string(ctx.repo_root.join(k8s_rollout_contract_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let rollout_contract_json: serde_json::Value = serde_json::from_str(&rollout_contract_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let rollout_profiles = rollout_contract_json
            .get("profiles")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        for profile in &rollout_profiles {
            let profile_name = profile.get("name").and_then(|v| v.as_str()).unwrap_or_default();
            if profile_name.is_empty() {
                continue;
            }
            let Some(matrix_entry) = matrix_by_name.get(profile_name) else {
                violations.push(violation(
                    "OPS_K8S_ROLLOUT_PROFILE_MISSING_FROM_INSTALL_MATRIX",
                    format!(
                        "rollout-safety-contract profile `{profile_name}` is missing from install-matrix"
                    ),
                    "align rollout-safety-contract profiles with ops/k8s/install-matrix.json",
                    Some(k8s_rollout_contract_rel),
                ));
                continue;
            };
            let contract_suite = profile.get("suite").and_then(|v| v.as_str()).unwrap_or_default();
            let matrix_suite = matrix_entry
                .get("suite")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if contract_suite != matrix_suite {
                violations.push(violation(
                    "OPS_K8S_ROLLOUT_SUITE_DRIFT",
                    format!(
                        "rollout-safety-contract suite drift for profile `{profile_name}`: contract=`{contract_suite}` matrix=`{matrix_suite}`"
                    ),
                    "align rollout-safety-contract suite values with install-matrix",
                    Some(k8s_rollout_contract_rel),
                ));
            }
            let values_file = profile
                .get("values_file")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let matrix_values_file = matrix_entry
                .get("values_file")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if values_file != matrix_values_file {
                violations.push(violation(
                    "OPS_K8S_ROLLOUT_VALUES_FILE_DRIFT",
                    format!(
                        "rollout-safety-contract values_file drift for profile `{profile_name}`: contract=`{values_file}` matrix=`{matrix_values_file}`"
                    ),
                    "align rollout-safety-contract values_file with install-matrix",
                    Some(k8s_rollout_contract_rel),
                ));
                continue;
            }
            let values_rel = Path::new(values_file);
            if !ctx.adapters.fs.exists(ctx.repo_root, values_rel) {
                violations.push(violation(
                    "OPS_K8S_ROLLOUT_VALUES_FILE_MISSING",
                    format!("rollout-safety-contract references missing values file `{values_file}`"),
                    "restore missing values file or update rollout-safety-contract",
                    Some(k8s_rollout_contract_rel),
                ));
                continue;
            }
            let values_text = fs::read_to_string(ctx.repo_root.join(values_rel))
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            let values_yaml: serde_yaml::Value = serde_yaml::from_str(&values_text)
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            let warmup_required = profile
                .get("warmup_required")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if warmup_required {
                let warmup_enabled = values_yaml
                    .get("cache")
                    .and_then(|v| v.get("warmupEnabled"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if !warmup_enabled {
                    violations.push(violation(
                        "OPS_K8S_WARMUP_REQUIRED_BUT_DISABLED",
                        format!(
                            "profile `{profile_name}` requires warmup but values file disables cache.warmupEnabled"
                        ),
                        "enable cache.warmupEnabled for warmup-required rollout profiles",
                        Some(values_rel),
                    ));
                }
            }
            let readiness_required = profile
                .get("readiness_path_required")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if readiness_required {
                let readiness_path = values_yaml
                    .get("server")
                    .and_then(|v| v.get("readinessProbePath"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                if readiness_path.trim().is_empty() {
                    violations.push(violation(
                        "OPS_K8S_READINESS_PATH_REQUIRED_MISSING",
                        format!(
                            "profile `{profile_name}` requires readiness probe path but server.readinessProbePath is missing"
                        ),
                        "define server.readinessProbePath in profile values file",
                        Some(values_rel),
                    ));
                }
            }
        }
    } else if !ctx.adapters.fs.exists(ctx.repo_root, k8s_rollout_contract_rel) {
        violations.push(violation(
            "OPS_K8S_ROLLOUT_CONTRACT_MISSING",
            format!(
                "missing k8s rollout safety contract `{}`",
                k8s_rollout_contract_rel.display()
            ),
            "restore ops/k8s/rollout-safety-contract.json",
            Some(k8s_rollout_contract_rel),
        ));
    }

    let registry_drift_rel = Path::new("ops/_generated.example/registry-drift-report.json");
    let control_graph_diff_rel = Path::new("ops/_generated.example/control-graph-diff-report.json");
    let inventory_completeness_rel = Path::new("ops/_generated.example/inventory-completeness-score.json");
    if !ctx.adapters.fs.exists(ctx.repo_root, registry_drift_rel) {
        violations.push(violation(
            "OPS_INVENTORY_REGISTRY_DRIFT_REPORT_MISSING",
            format!("missing registry drift report `{}`", registry_drift_rel.display()),
            "generate and commit ops/_generated.example/registry-drift-report.json",
            Some(registry_drift_rel),
        ));
    } else {
        let registry_drift_text = fs::read_to_string(ctx.repo_root.join(registry_drift_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let registry_drift_json: serde_json::Value = serde_json::from_str(&registry_drift_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        if registry_drift_json.get("status").and_then(|v| v.as_str()) != Some("pass") {
            violations.push(violation(
                "OPS_INVENTORY_REGISTRY_DRIFT_REPORT_BLOCKING",
                "registry-drift-report.json status is not `pass`".to_string(),
                "resolve inventory registry drift and regenerate registry-drift-report.json",
                Some(registry_drift_rel),
            ));
        }
    }

    if !ctx.adapters.fs.exists(ctx.repo_root, control_graph_diff_rel) {
        violations.push(violation(
            "OPS_INVENTORY_CONTROL_GRAPH_DIFF_REPORT_MISSING",
            format!(
                "missing control graph diff report `{}`",
                control_graph_diff_rel.display()
            ),
            "generate and commit ops/_generated.example/control-graph-diff-report.json",
            Some(control_graph_diff_rel),
        ));
    } else {
        let control_graph_diff_text = fs::read_to_string(ctx.repo_root.join(control_graph_diff_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let control_graph_diff_json: serde_json::Value = serde_json::from_str(&control_graph_diff_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        if control_graph_diff_json.get("status").and_then(|v| v.as_str()) != Some("pass") {
            violations.push(violation(
                "OPS_INVENTORY_CONTROL_GRAPH_DIFF_REPORT_BLOCKING",
                "control-graph-diff-report.json status is not `pass`".to_string(),
                "resolve control graph drift and regenerate control-graph-diff-report.json",
                Some(control_graph_diff_rel),
            ));
        }
    }

    if !ctx.adapters.fs.exists(ctx.repo_root, inventory_completeness_rel) {
        violations.push(violation(
            "OPS_INVENTORY_COMPLETENESS_SCORE_MISSING",
            format!(
                "missing inventory completeness score report `{}`",
                inventory_completeness_rel.display()
            ),
            "generate and commit ops/_generated.example/inventory-completeness-score.json",
            Some(inventory_completeness_rel),
        ));
    } else {
        let completeness_text = fs::read_to_string(ctx.repo_root.join(inventory_completeness_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let completeness_json: serde_json::Value = serde_json::from_str(&completeness_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        if completeness_json.get("status").and_then(|v| v.as_str()) != Some("pass") {
            violations.push(violation(
                "OPS_INVENTORY_COMPLETENESS_SCORE_BLOCKING",
                "inventory-completeness-score.json status is not `pass`".to_string(),
                "resolve inventory completeness gaps and regenerate inventory-completeness-score.json",
                Some(inventory_completeness_rel),
            ));
        }
        if completeness_json
            .get("score")
            .and_then(|v| v.as_u64())
            .is_none_or(|score| score < 90)
        {
            violations.push(violation(
                "OPS_INVENTORY_COMPLETENESS_SCORE_THRESHOLD",
                "inventory-completeness-score.json score must be >= 90".to_string(),
                "raise inventory completeness coverage and regenerate the completeness score report",
                Some(inventory_completeness_rel),
            ));
        }
        let checks = completeness_json
            .get("checks")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        for key in [
            "node_consumer_coverage",
            "node_lifecycle_coverage",
            "domain_coverage_edges",
            "path_surface_coverage",
            "edge_kind_coverage",
        ] {
            if checks.get(key).and_then(|v| v.as_u64()).is_none() {
                violations.push(violation(
                    "OPS_INVENTORY_COMPLETENESS_SCORE_CHECKS_INCOMPLETE",
                    format!(
                        "inventory completeness score report is missing numeric checks field `{key}`"
                    ),
                    "include all inventory completeness component checks in the report",
                    Some(inventory_completeness_rel),
                ));
            }
        }
    }

    if !ctx.adapters.fs.exists(ctx.repo_root, stack_drift_rel) {
        violations.push(violation(
            "OPS_STACK_DRIFT_REPORT_MISSING",
            format!("missing stack drift report `{}`", stack_drift_rel.display()),
            "generate and commit ops/_generated.example/stack-drift-report.json",
            Some(stack_drift_rel),
        ));
    } else {
        let stack_drift_text = fs::read_to_string(ctx.repo_root.join(stack_drift_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let stack_drift_json: serde_json::Value = serde_json::from_str(&stack_drift_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        if stack_drift_json.get("status").and_then(|v| v.as_str()) != Some("pass") {
            violations.push(violation(
                "OPS_STACK_DRIFT_REPORT_BLOCKING",
                "stack-drift-report.json status is not `pass`".to_string(),
                "resolve stack drift and regenerate stack-drift-report.json",
                Some(stack_drift_rel),
            ));
        }
    }

    Ok(())
}
