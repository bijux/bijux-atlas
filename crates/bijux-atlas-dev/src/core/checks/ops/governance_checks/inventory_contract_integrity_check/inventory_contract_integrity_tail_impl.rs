struct InventoryContractTailPaths<'a> {
    gates_rel: &'a Path,
    owners_rel: &'a Path,
    registry_rel: &'a Path,
    tools_rel: &'a Path,
    policy_rel: &'a Path,
    policy_schema_rel: &'a Path,
    pins_rel: &'a Path,
    stack_manifest_rel: &'a Path,
    inventory_root: &'a Path,
    inventory_index_rel: &'a Path,
    stack_toml_rel: &'a Path,
    stack_dependency_graph_rel: &'a Path,
    stack_service_contract_rel: &'a Path,
    stack_evolution_policy_rel: &'a Path,
    k8s_install_matrix_rel: &'a Path,
    k8s_rollout_contract_rel: &'a Path,
    stack_drift_rel: &'a Path,
}

fn check_ops_inventory_contract_integrity_tail(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
    gates_json: &serde_json::Value,
    action_ids: &std::collections::BTreeSet<String>,
    paths: InventoryContractTailPaths<'_>,
) -> Result<(), CheckError> {
    let InventoryContractTailPaths {
        gates_rel,
        owners_rel,
        registry_rel,
        tools_rel,
        policy_rel,
        policy_schema_rel,
        pins_rel,
        stack_manifest_rel,
        inventory_root,
        inventory_index_rel,
        stack_toml_rel,
        stack_dependency_graph_rel,
        stack_service_contract_rel,
        stack_evolution_policy_rel,
        k8s_install_matrix_rel,
        k8s_rollout_contract_rel,
        stack_drift_rel,
    } = paths;
    if let Some(gates) = gates_json.get("gates").and_then(|v| v.as_array()) {
        let gate_ids = gates
            .iter()
            .filter_map(|gate| gate.get("id").and_then(|v| v.as_str()))
            .collect::<std::collections::BTreeSet<_>>();
        let required_release_gates = [
            "ops.gate.ssot",
            "ops.gate.validate",
            "ops.gate.structure",
            "ops.gate.docs",
            "ops.gate.generated",
            "ops.gate.evidence",
            "ops.gate.fixtures",
            "ops.gate.naming",
            "ops.gate.inventory",
            "ops.gate.schema",
        ];
        for required_gate in required_release_gates {
            if !gate_ids.contains(required_gate) {
                violations.push(violation(
                    "OPS_INVENTORY_RELEASE_GATE_MISSING",
                    format!("required release gate id missing from gates.json: `{required_gate}`"),
                    "add the missing release gate id to ops/inventory/gates.json",
                    Some(gates_rel),
                ));
            }
        }
        for gate in gates {
            let Some(action_id) = gate.get("action_id").and_then(|v| v.as_str()) else {
                continue;
            };
            if !action_ids.contains(action_id) {
                violations.push(violation(
                    "OPS_INVENTORY_GATE_ACTION_NOT_FOUND",
                    format!("gate action id `{action_id}` is not present in surfaces actions"),
                    "align ops/inventory/gates.json action_id fields with ops/inventory/surfaces.json",
                    Some(gates_rel),
                ));
            }
        }
    }

    validate_inventory_owners_registry_and_policy_contracts(
        ctx,
        violations,
        owners_rel,
        registry_rel,
        tools_rel,
        policy_rel,
        policy_schema_rel,
    )?;

    let pins_text = fs::read_to_string(ctx.repo_root.join(pins_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let pins_yaml: serde_yaml::Value =
        serde_yaml::from_str(&pins_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let pins_images = pins_yaml
        .get("images")
        .and_then(|v| v.as_mapping())
        .cloned()
        .unwrap_or_default();
    let stack_manifest_text = fs::read_to_string(ctx.repo_root.join(stack_manifest_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let stack_manifest_json: serde_json::Value = serde_json::from_str(&stack_manifest_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if let Some(obj) = stack_manifest_json.as_object() {
        for (key, value) in obj {
            if key == "schema_version" {
                continue;
            }
            let image_value = value.as_str().unwrap_or_default();
            let pin_value = pins_images
                .get(serde_yaml::Value::String(key.clone()))
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if pin_value != image_value {
                violations.push(violation(
                    "OPS_INVENTORY_PIN_STACK_DRIFT",
                    format!("stack manifest image `{key}` differs from inventory pin value"),
                    "regenerate stack generated version-manifest from inventory pins",
                    Some(stack_manifest_rel),
                ));
            }
        }
    }

    validate_generated_inventory_index_and_stack_contracts(
        ctx,
        violations,
        InventoryAndStackContractPaths {
            inventory_root,
            inventory_index_rel,
            stack_toml_rel,
            stack_dependency_graph_rel,
            stack_service_contract_rel,
            stack_evolution_policy_rel,
        },
    )?;
    validate_k8s_rollout_and_drift_reports(
        ctx,
        violations,
        k8s_install_matrix_rel,
        k8s_rollout_contract_rel,
        stack_drift_rel,
    )?;

    Ok(())
}
