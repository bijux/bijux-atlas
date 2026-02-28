fn read_contract_gate_map(root: &Path) -> Result<serde_json::Value, String> {
    read_json(&root.join("ops/inventory/contract-gate-map.json"))
        .ok_or_else(|| "ops/inventory/contract-gate-map.json is missing or invalid".to_string())
}

fn test_ops_inv_debt_001_debt_list_exists_and_complete(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-INV-DEBT-001";
    let test_id = "ops.inventory.contract_debt.exists_and_complete";
    let debt_path = ctx.repo_root.join("ops/inventory/contract-debt.json");
    let schema_path = ctx
        .repo_root
        .join("ops/schema/inventory/contract-debt.schema.json");
    let Some(payload) = read_json(&debt_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "contract debt registry must exist and be valid json",
            Some("ops/inventory/contract-debt.json".to_string()),
        )]);
    };
    if !schema_path.is_file() {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "contract debt schema must exist",
            Some("ops/schema/inventory/contract-debt.schema.json".to_string()),
        )]);
    }
    let Some(items) = payload.get("items").and_then(|value| value.as_array()) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "contract debt registry must define items array",
            Some("ops/inventory/contract-debt.json".to_string()),
        )]);
    };
    if let Err(errors) = crate::schema_support::require_object_keys(
        &payload,
        &["schema_version", "items"],
    ) {
        return TestResult::Fail(
            errors
                .into_iter()
                .map(|message| {
                    violation(
                        contract_id,
                        test_id,
                        &format!("contract debt registry is structurally incomplete: {message}"),
                        Some("ops/inventory/contract-debt.json".to_string()),
                    )
                })
                .collect(),
        );
    }
    let mut violations = Vec::new();
    let mut ids = BTreeSet::new();
    for item in items {
        let item_id = item.get("id").and_then(|value| value.as_str()).unwrap_or("");
        if item_id.is_empty() {
            violations.push(violation(
                contract_id,
                test_id,
                "debt item id must be non-empty",
                Some("ops/inventory/contract-debt.json".to_string()),
            ));
            continue;
        }
        if !ids.insert(item_id.to_string()) {
            violations.push(violation(
                contract_id,
                test_id,
                "debt item ids must be unique",
                Some(format!("ops/inventory/contract-debt.json:{item_id}")),
            ));
        }
        for field in ["owner", "target_milestone"] {
            if item
                .get(field)
                .and_then(|value| value.as_str())
                .is_none_or(str::is_empty)
            {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "debt items must define owner and target_milestone",
                    Some(format!("ops/inventory/contract-debt.json:{item_id}:{field}")),
                ));
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_inv_map_001_every_contract_id_mapped(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-INV-MAP-001";
    let test_id = "ops.inventory.contract_gate_map.every_contract_mapped";
    let rows = match contracts(&ctx.repo_root) {
        Ok(v) => v,
        Err(err) => return TestResult::Error(err),
    };
    let map = match read_contract_gate_map(&ctx.repo_root) {
        Ok(v) => v,
        Err(_) => {
            return TestResult::Fail(vec![violation(
                contract_id,
                test_id,
                "contract-gate-map must exist and be valid json",
                Some("ops/inventory/contract-gate-map.json".to_string()),
            )]);
        }
    };
    let mapped: BTreeSet<String> = map
        .get("mappings")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("contract_id").and_then(|v| v.as_str()))
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let mut violations = Vec::new();
    for row in rows {
        if !mapped.contains(&row.id.0) {
            violations.push(violation(
                contract_id,
                test_id,
                "contract id is missing from contract-gate-map",
                Some(row.id.0),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_inv_map_002_mapped_gates_exist(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-INV-MAP-002";
    let test_id = "ops.inventory.contract_gate_map.mapped_gates_exist";
    let map = match read_contract_gate_map(&ctx.repo_root) {
        Ok(v) => v,
        Err(_) => {
            return TestResult::Fail(vec![violation(
                contract_id,
                test_id,
                "contract-gate-map must exist and be valid json",
                Some("ops/inventory/contract-gate-map.json".to_string()),
            )]);
        }
    };
    let gates = read_json(&ctx.repo_root.join("ops/inventory/gates.json"));
    let gate_ids: BTreeSet<String> = gates
        .as_ref()
        .and_then(|v| v.get("gates"))
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("id").and_then(|v| v.as_str()))
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let mut violations = Vec::new();
    for item in map
        .get("mappings")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        let cid = item
            .get("contract_id")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        for gate_id in item
            .get("gate_ids")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
            .filter_map(|v| v.as_str())
        {
            if !gate_ids.contains(gate_id) {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "mapped gate id must exist in ops/inventory/gates.json",
                    Some(format!("{cid}:{gate_id}")),
                ));
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_inv_map_003_mapped_commands_registered(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-INV-MAP-003";
    let test_id = "ops.inventory.contract_gate_map.mapped_commands_registered";
    let map = match read_contract_gate_map(&ctx.repo_root) {
        Ok(v) => v,
        Err(_) => {
            return TestResult::Fail(vec![violation(
                contract_id,
                test_id,
                "contract-gate-map must exist and be valid json",
                Some("ops/inventory/contract-gate-map.json".to_string()),
            )]);
        }
    };
    let surfaces = read_json(&ctx.repo_root.join("ops/inventory/surfaces.json"));
    let known_commands: BTreeSet<String> = surfaces
        .as_ref()
        .and_then(|v| v.get("bijux-dev-atlas_commands"))
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let mut violations = Vec::new();
    for item in map
        .get("mappings")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        let cid = item
            .get("contract_id")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let command = item
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if command.is_empty() || !known_commands.contains(command) {
            violations.push(violation(
                contract_id,
                test_id,
                "mapped command must be registered in ops surface router",
                Some(format!("{cid}:{command}")),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_inv_map_004_effects_annotation_matches_contract_mode(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-INV-MAP-004";
    let test_id = "ops.inventory.contract_gate_map.effects_annotation_matches_mode";
    let rows = match contracts(&ctx.repo_root) {
        Ok(v) => v,
        Err(err) => return TestResult::Error(err),
    };
    let contract_index: BTreeMap<String, Contract> =
        rows.into_iter().map(|row| (row.id.0.clone(), row)).collect();
    let map = match read_contract_gate_map(&ctx.repo_root) {
        Ok(v) => v,
        Err(_) => {
            return TestResult::Fail(vec![violation(
                contract_id,
                test_id,
                "contract-gate-map must exist and be valid json",
                Some("ops/inventory/contract-gate-map.json".to_string()),
            )]);
        }
    };
    let mut violations = Vec::new();
    for item in map
        .get("mappings")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        let cid = item
            .get("contract_id")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let Some(contract) = contract_index.get(cid) else {
            continue;
        };
        let effects: BTreeSet<String> = item
            .get("effects_required")
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(ToOwned::to_owned)
                    .collect()
            })
            .unwrap_or_default();
        let has_effect_tests = contract.tests.iter().any(|t| !matches!(t.kind, TestKind::Pure));
        if has_effect_tests && effects.is_empty() {
            violations.push(violation(
                contract_id,
                test_id,
                "effect contracts must declare non-empty effects_required",
                Some(cid.to_string()),
            ));
        }
        if !has_effect_tests && !effects.is_empty() {
            violations.push(violation(
                contract_id,
                test_id,
                "pure contracts must not declare effects_required",
                Some(cid.to_string()),
            ));
        }
        for effect in effects {
            if effect != "subprocess" && effect != "network" && effect != "fs" {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "effects_required entries must be one of subprocess/network/fs",
                    Some(format!("{cid}:{effect}")),
                ));
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_inv_map_005_no_orphan_gates(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-INV-MAP-005";
    let test_id = "ops.inventory.contract_gate_map.no_orphan_gates";
    let gates = match read_json(&ctx.repo_root.join("ops/inventory/gates.json")) {
        Some(v) => v,
        None => {
            return TestResult::Fail(vec![violation(
                contract_id,
                test_id,
                "gates registry must exist and be valid json",
                Some("ops/inventory/gates.json".to_string()),
            )]);
        }
    };
    let map = match read_contract_gate_map(&ctx.repo_root) {
        Ok(v) => v,
        Err(_) => {
            return TestResult::Fail(vec![violation(
                contract_id,
                test_id,
                "contract-gate-map must exist and be valid json",
                Some("ops/inventory/contract-gate-map.json".to_string()),
            )]);
        }
    };
    let all_gate_ids: BTreeSet<String> = gates
        .get("gates")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("id").and_then(|v| v.as_str()))
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let mapped_gate_ids: BTreeSet<String> = map
        .get("mappings")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .flat_map(|item| {
                    item.get("gate_ids")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default()
                })
                .filter_map(|v| v.as_str().map(ToOwned::to_owned))
                .collect()
        })
        .unwrap_or_default();
    let mut violations = Vec::new();
    for gate_id in all_gate_ids {
        if !mapped_gate_ids.contains(&gate_id) {
            violations.push(violation(
                contract_id,
                test_id,
                "gate must be referenced by at least one contract mapping",
                Some(gate_id),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_inv_map_006_no_orphan_contracts(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-INV-MAP-006";
    let test_id = "ops.inventory.contract_gate_map.no_orphan_contracts";
    let rows = match contracts(&ctx.repo_root) {
        Ok(v) => v,
        Err(err) => return TestResult::Error(err),
    };
    let map = match read_contract_gate_map(&ctx.repo_root) {
        Ok(v) => v,
        Err(_) => {
            return TestResult::Fail(vec![violation(
                contract_id,
                test_id,
                "contract-gate-map must exist and be valid json",
                Some("ops/inventory/contract-gate-map.json".to_string()),
            )]);
        }
    };
    let map_index: BTreeMap<String, &serde_json::Value> = map
        .get("mappings")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    item.get("contract_id")
                        .and_then(|v| v.as_str())
                        .map(|id| (id.to_string(), item))
                })
                .collect()
        })
        .unwrap_or_default();
    let mut violations = Vec::new();
    for row in rows {
        let Some(mapping) = map_index.get(&row.id.0) else {
            violations.push(violation(
                contract_id,
                test_id,
                "contract is missing from contract-gate-map",
                Some(row.id.0),
            ));
            continue;
        };
        let has_gate = mapping
            .get("gate_ids")
            .and_then(|v| v.as_array())
            .is_some_and(|items| !items.is_empty());
        let static_only = mapping
            .get("static_only")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if !has_gate && !static_only {
            violations.push(violation(
                contract_id,
                test_id,
                "contract must map to gate_ids or be marked static_only",
                Some(row.id.0),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_inv_map_007_static_only_contracts_are_pure(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-INV-MAP-007";
    let test_id = "ops.inventory.contract_gate_map.static_only_contracts_are_pure";
    let rows = match contracts(&ctx.repo_root) {
        Ok(v) => v,
        Err(err) => return TestResult::Error(err),
    };
    let contract_index: BTreeMap<String, Contract> =
        rows.into_iter().map(|row| (row.id.0.clone(), row)).collect();
    let map = match read_contract_gate_map(&ctx.repo_root) {
        Ok(v) => v,
        Err(_) => {
            return TestResult::Fail(vec![violation(
                contract_id,
                test_id,
                "contract-gate-map must exist and be valid json",
                Some("ops/inventory/contract-gate-map.json".to_string()),
            )]);
        }
    };
    let mut violations = Vec::new();
    for item in map
        .get("mappings")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        let static_only = item
            .get("static_only")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if !static_only {
            continue;
        }
        let cid = item
            .get("contract_id")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let Some(contract) = contract_index.get(cid) else {
            continue;
        };
        if contract.tests.iter().any(|t| !matches!(t.kind, TestKind::Pure)) {
            violations.push(violation(
                contract_id,
                test_id,
                "static_only contract must contain only pure tests",
                Some(cid.to_string()),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_inv_map_008_effect_contracts_require_effect_kind(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-INV-MAP-008";
    let test_id = "ops.inventory.contract_gate_map.effect_contracts_require_effect_kind";
    let rows = match contracts(&ctx.repo_root) {
        Ok(v) => v,
        Err(err) => return TestResult::Error(err),
    };
    let contract_index: BTreeMap<String, Contract> =
        rows.into_iter().map(|row| (row.id.0.clone(), row)).collect();
    let map = match read_contract_gate_map(&ctx.repo_root) {
        Ok(v) => v,
        Err(_) => {
            return TestResult::Fail(vec![violation(
                contract_id,
                test_id,
                "contract-gate-map must exist and be valid json",
                Some("ops/inventory/contract-gate-map.json".to_string()),
            )]);
        }
    };
    let mut violations = Vec::new();
    for item in map
        .get("mappings")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        let static_only = item
            .get("static_only")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if static_only {
            continue;
        }
        let cid = item
            .get("contract_id")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let Some(contract) = contract_index.get(cid) else {
            continue;
        };
        let has_effect_kind = contract.tests.iter().any(|case| {
            matches!(case.kind, TestKind::Subprocess) || matches!(case.kind, TestKind::Network)
        });
        if !has_effect_kind {
            violations.push(violation(
                contract_id,
                test_id,
                "effect contract mapping requires at least one Subprocess or Network test",
                Some(cid.to_string()),
            ));
        }
        let effects: BTreeSet<String> = item
            .get("effects_required")
            .and_then(|v| v.as_array())
            .map(|values| {
                values
                    .iter()
                    .filter_map(|v| v.as_str().map(ToOwned::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        if !effects.contains("subprocess") && !effects.contains("network") {
            violations.push(violation(
                contract_id,
                test_id,
                "effect contract mapping must declare subprocess or network in effects_required",
                Some(cid.to_string()),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_inv_map_009_explain_shows_mapped_gates(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-INV-MAP-009";
    let test_id = "ops.inventory.contract_gate_map.explain_shows_mapped_gates";
    let map = match read_contract_gate_map(&ctx.repo_root) {
        Ok(v) => v,
        Err(_) => {
            return TestResult::Fail(vec![violation(
                contract_id,
                test_id,
                "contract-gate-map must exist and be valid json",
                Some("ops/inventory/contract-gate-map.json".to_string()),
            )]);
        }
    };
    let mut violations = Vec::new();
    for item in map
        .get("mappings")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        let cid = item
            .get("contract_id")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let gate_ids = item
            .get("gate_ids")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let static_only = item
            .get("static_only")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if gate_ids.is_empty() && !static_only {
            violations.push(violation(
                contract_id,
                test_id,
                "mapped contracts must expose at least one gate id in explain output source",
                Some(cid.to_string()),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_inv_map_010_mapping_sorted_canonical(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-INV-MAP-010";
    let test_id = "ops.inventory.contract_gate_map.mapping_sorted_canonical";
    let map_path = ctx.repo_root.join("ops/inventory/contract-gate-map.json");
    let Some(raw_map) = read_json(&map_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "contract-gate-map must exist and be valid json",
            Some("ops/inventory/contract-gate-map.json".to_string()),
        )]);
    };
    let mut mappings = match raw_map.get("mappings").and_then(|v| v.as_array()) {
        Some(rows) => rows.clone(),
        None => {
            return TestResult::Fail(vec![violation(
                contract_id,
                test_id,
                "contract-gate-map must define mappings array",
                Some("ops/inventory/contract-gate-map.json".to_string()),
            )]);
        }
    };
    mappings.sort_by(|a, b| {
        let a_id = a.get("contract_id").and_then(|v| v.as_str()).unwrap_or_default();
        let b_id = b.get("contract_id").and_then(|v| v.as_str()).unwrap_or_default();
        a_id.cmp(b_id)
    });
    let canonical = serde_json::json!({
        "schema_version": raw_map.get("schema_version").and_then(|v| v.as_i64()).unwrap_or(1),
        "mappings": mappings,
    });
    let actual_text = match fs::read_to_string(&map_path) {
        Ok(v) => v,
        Err(err) => return TestResult::Error(err.to_string()),
    };
    let expected_text = match serde_json::to_string_pretty(&canonical) {
        Ok(v) => format!("{v}\n"),
        Err(err) => return TestResult::Error(err.to_string()),
    };
    if actual_text == expected_text {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "contract-gate-map must be sorted by contract_id and canonical pretty json",
            Some("ops/inventory/contract-gate-map.json".to_string()),
        )])
    }
}

