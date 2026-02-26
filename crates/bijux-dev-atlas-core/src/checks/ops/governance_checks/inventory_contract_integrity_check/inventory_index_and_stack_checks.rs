fn validate_generated_inventory_index_and_stack_contracts(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
    inventory_root: &Path,
    inventory_index_rel: &Path,
    stack_toml_rel: &Path,
    stack_dependency_graph_rel: &Path,
    stack_service_contract_rel: &Path,
    stack_evolution_policy_rel: &Path,
) -> Result<(), CheckError> {
    if !ctx.adapters.fs.exists(ctx.repo_root, inventory_index_rel) {
        violations.push(violation(
            "OPS_INVENTORY_INDEX_ARTIFACT_MISSING",
            format!(
                "missing generated inventory index artifact `{}`",
                inventory_index_rel.display()
            ),
            "generate and commit ops/_generated.example/inventory-index.json",
            Some(inventory_index_rel),
        ));
    } else {
        let inventory_index_text = fs::read_to_string(ctx.repo_root.join(inventory_index_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let inventory_index_json: serde_json::Value = serde_json::from_str(&inventory_index_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let indexed_paths = inventory_index_json
            .get("items")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.get("path").and_then(|v| v.as_str()))
                    .map(ToString::to_string)
                    .collect::<std::collections::BTreeSet<_>>()
            })
            .unwrap_or_default();
        let expected_inventory_paths = walk_files(inventory_root)
            .into_iter()
            .filter_map(|file| file.strip_prefix(ctx.repo_root).ok().map(PathBuf::from))
            .filter(|rel| {
                rel.extension()
                    .and_then(|v| v.to_str())
                    .is_some_and(|ext| matches!(ext, "json" | "yaml" | "yml" | "toml"))
            })
            .map(|rel| rel.display().to_string())
            .collect::<std::collections::BTreeSet<_>>();
        for rel in expected_inventory_paths.difference(&indexed_paths) {
            violations.push(violation(
                "OPS_INVENTORY_INDEX_COVERAGE_MISSING",
                format!("inventory-index.json is missing inventory artifact `{rel}`"),
                "regenerate ops/_generated.example/inventory-index.json to include every ops/inventory data artifact",
                Some(inventory_index_rel),
            ));
        }
    }

    if ctx.adapters.fs.exists(ctx.repo_root, stack_toml_rel)
        && ctx.adapters.fs.exists(ctx.repo_root, stack_dependency_graph_rel)
    {
        let stack_toml_text = fs::read_to_string(ctx.repo_root.join(stack_toml_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let stack_toml: toml::Value =
            toml::from_str(&stack_toml_text).map_err(|err| CheckError::Failed(err.to_string()))?;
        let graph_text = fs::read_to_string(ctx.repo_root.join(stack_dependency_graph_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let graph_json: serde_json::Value =
            serde_json::from_str(&graph_text).map_err(|err| CheckError::Failed(err.to_string()))?;
        let toml_profiles = stack_toml
            .get("profiles")
            .and_then(|v| v.as_table())
            .cloned()
            .unwrap_or_default();
        let graph_profiles = graph_json
            .get("profiles")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        let toml_profile_keys = toml_profiles.keys().cloned().collect::<BTreeSet<_>>();
        let graph_profile_keys = graph_profiles.keys().cloned().collect::<BTreeSet<_>>();
        if toml_profile_keys != graph_profile_keys {
            violations.push(violation(
                "OPS_STACK_DEPENDENCY_GRAPH_PROFILE_DRIFT",
                format!(
                    "dependency-graph profile keys drift from stack.toml: stack={toml_profile_keys:?} graph={graph_profile_keys:?}"
                ),
                "regenerate ops/stack/generated/dependency-graph.json from ops/stack/stack.toml",
                Some(stack_dependency_graph_rel),
            ));
        }

        if ctx.adapters.fs.exists(ctx.repo_root, stack_service_contract_rel) {
            let service_contract_text = fs::read_to_string(ctx.repo_root.join(stack_service_contract_rel))
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            let service_contract_json: serde_json::Value = serde_json::from_str(&service_contract_text)
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            let services = service_contract_json
                .get("services")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let mut has_critical_service = false;
            for service in services {
                let service_id = service
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown-service");
                let component = service
                    .get("component")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                if component.is_empty() {
                    continue;
                }
                if service
                    .get("critical")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    has_critical_service = true;
                }
                let component_rel = Path::new(component);
                if !ctx.adapters.fs.exists(ctx.repo_root, component_rel) {
                    violations.push(violation(
                        "OPS_STACK_SERVICE_COMPONENT_MISSING",
                        format!(
                            "stack service `{service_id}` references missing component `{component}`"
                        ),
                        "fix component path in ops/stack/service-dependency-contract.json",
                        Some(stack_service_contract_rel),
                    ));
                    continue;
                }
                for profile in service
                    .get("required_profiles")
                    .and_then(|v| v.as_array())
                    .into_iter()
                    .flatten()
                    .filter_map(|v| v.as_str())
                {
                    let Some(profile_value) = toml_profiles.get(profile) else {
                        violations.push(violation(
                            "OPS_STACK_SERVICE_REQUIRED_PROFILE_MISSING",
                            format!(
                                "stack service `{service_id}` requires unknown profile `{profile}`"
                            ),
                            "align required_profiles with ops/stack/stack.toml profile keys",
                            Some(stack_service_contract_rel),
                        ));
                        continue;
                    };
                    let profile_components = profile_value
                        .get("components")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();
                    let profile_component_paths =
                        profile_components.iter().filter_map(|v| v.as_str()).collect::<BTreeSet<_>>();
                    if !profile_component_paths.contains(component) {
                        violations.push(violation(
                            "OPS_STACK_SERVICE_PROFILE_COMPONENT_DRIFT",
                            format!(
                                "stack service `{service_id}` component `{component}` missing from profile `{profile}` components"
                            ),
                            "keep service-dependency-contract required_profiles aligned with stack.toml components",
                            Some(stack_service_contract_rel),
                        ));
                    }
                }
            }
            if !has_critical_service {
                violations.push(violation(
                    "OPS_STACK_SERVICE_CRITICAL_COVERAGE_MISSING",
                    "service-dependency-contract must define at least one critical service".to_string(),
                    "mark core stack services as critical in ops/stack/service-dependency-contract.json",
                    Some(stack_service_contract_rel),
                ));
            }
        } else {
            violations.push(violation(
                "OPS_STACK_SERVICE_CONTRACT_MISSING",
                format!(
                    "missing stack service dependency contract `{}`",
                    stack_service_contract_rel.display()
                ),
                "restore ops/stack/service-dependency-contract.json",
                Some(stack_service_contract_rel),
            ));
        }
    }

    if ctx.adapters.fs.exists(ctx.repo_root, stack_evolution_policy_rel) {
        let evolution_text = fs::read_to_string(ctx.repo_root.join(stack_evolution_policy_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let evolution_json: serde_json::Value =
            serde_json::from_str(&evolution_text).map_err(|err| CheckError::Failed(err.to_string()))?;
        for key in [
            "schema_version",
            "policy_version",
            "freeze_rules",
            "change_controls",
            "compatibility",
        ] {
            if evolution_json.get(key).is_none() {
                violations.push(violation(
                    "OPS_STACK_EVOLUTION_POLICY_FIELD_MISSING",
                    format!("stack evolution policy missing required key `{key}`"),
                    "add missing required keys to ops/stack/evolution-policy.json",
                    Some(stack_evolution_policy_rel),
                ));
            }
        }
    } else {
        violations.push(violation(
            "OPS_STACK_EVOLUTION_POLICY_MISSING",
            format!(
                "missing stack evolution policy `{}`",
                stack_evolution_policy_rel.display()
            ),
            "restore ops/stack/evolution-policy.json",
            Some(stack_evolution_policy_rel),
        ));
    }

    Ok(())
}
