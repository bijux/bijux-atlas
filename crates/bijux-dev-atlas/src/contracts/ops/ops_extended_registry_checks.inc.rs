// SPDX-License-Identifier: Apache-2.0
fn test_ops_inv_pillars_001_exists_and_validates(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-INV-PILLARS-001";
    let test_id = "ops.inventory.pillars.exists_and_validates";
    match read_pillars_doc(&ctx.repo_root) {
        Ok(doc) if !doc.pillars.is_empty() => TestResult::Pass,
        Ok(_) => TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "pillars.json must declare at least one pillar",
            Some("ops/inventory/pillars.json".to_string()),
        )]),
        Err(_) => TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "pillars.json is missing or invalid json",
            Some("ops/inventory/pillars.json".to_string()),
        )]),
    }
}
fn test_ops_inv_pillars_002_every_pillar_dir_exists(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-INV-PILLARS-002";
    let test_id = "ops.inventory.pillars.every_pillar_dir_exists";
    let doc = match read_pillars_doc(&ctx.repo_root) {
        Ok(v) => v,
        Err(err) => return TestResult::Error(err),
    };
    let mut violations = Vec::new();
    for pillar in doc.pillars {
        if pillar.id == "root-surface" {
            continue;
        }
        let path = ctx.repo_root.join(format!("ops/{}", pillar.id));
        if !path.is_dir() {
            violations.push(violation(
                contract_id,
                test_id,
                "pillar directory listed in pillars.json is missing",
                Some(format!("ops/{}", pillar.id)),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}
fn test_ops_inv_pillars_003_no_extra_pillar_dirs(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-INV-PILLARS-003";
    let test_id = "ops.inventory.pillars.no_extra_pillar_dirs";
    let doc = match read_pillars_doc(&ctx.repo_root) {
        Ok(v) => v,
        Err(err) => return TestResult::Error(err),
    };
    let declared: BTreeSet<String> = doc
        .pillars
        .into_iter()
        .filter(|p| p.id != "root-surface")
        .map(|p| p.id)
        .collect();
    let root = ops_root(&ctx.repo_root);
    let Ok(entries) = fs::read_dir(&root) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "ops root directory is missing",
            Some("ops".to_string()),
        )]);
    };
    let mut violations = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if name == "_generated" || name == "_generated.example" {
            continue;
        }
        if !declared.contains(name) {
            violations.push(violation(
                contract_id,
                test_id,
                "ops root includes undeclared pillar directory",
                Some(format!("ops/{name}")),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}
fn test_ops_inv_006_contract_id_format(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-INV-006";
    let test_id = "ops.inventory.contract_id_format";
    let rows = match contracts(&ctx.repo_root) {
        Ok(v) => v,
        Err(err) => return TestResult::Error(err),
    };
    let id_pattern = match regex::Regex::new(r"^OPS-(?:[A-Z0-9]+(?:-[A-Z0-9]+)*-)?\d{3}$") {
        Ok(v) => v,
        Err(err) => return TestResult::Error(err.to_string()),
    };
    let mut violations = Vec::new();
    for row in rows {
        if !id_pattern.is_match(&row.id.0) {
            violations.push(violation(
                contract_id,
                test_id,
                "contract id does not match required OPS id format",
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
fn is_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 {
        return false;
    }
    bytes[0..4].iter().all(u8::is_ascii_digit)
        && bytes[4] == b'-'
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[7] == b'-'
        && bytes[8..10].iter().all(u8::is_ascii_digit)
}
fn test_ops_inv_004_authority_tiers_enforced(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-INV-004";
    let test_id = "ops.inventory.authority_tiers_enforced";
    let path = ctx
        .repo_root
        .join("ops/inventory/authority-tier-exceptions.json");
    let Some(value) = read_json(&path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "authority-tier exceptions file must be valid json",
            Some("ops/inventory/authority-tier-exceptions.json".to_string()),
        )]);
    };
    let mut violations = Vec::new();
    let Some(exceptions) = value.get("exceptions").and_then(|v| v.as_array()) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "authority-tier exceptions must contain `exceptions` array",
            Some("ops/inventory/authority-tier-exceptions.json".to_string()),
        )]);
    };
    for (idx, entry) in exceptions.iter().enumerate() {
        let prefix = format!("exception #{idx}");
        let path = entry.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let rule = entry.get("rule").and_then(|v| v.as_str()).unwrap_or("");
        let reason = entry.get("reason").and_then(|v| v.as_str()).unwrap_or("");
        let expires_on = entry
            .get("expires_on")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if path.is_empty() || rule.is_empty() || reason.is_empty() || expires_on.is_empty() {
            violations.push(violation(
                contract_id,
                test_id,
                &format!("{prefix}: path/rule/reason/expires_on are required"),
                Some("ops/inventory/authority-tier-exceptions.json".to_string()),
            ));
            continue;
        }
        if !is_iso_date(expires_on) {
            violations.push(violation(
                contract_id,
                test_id,
                &format!("{prefix}: expires_on must be YYYY-MM-DD"),
                Some("ops/inventory/authority-tier-exceptions.json".to_string()),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}
fn test_ops_inv_005_control_graph_validated(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-INV-005";
    let test_id = "ops.inventory.control_graph_validated";
    let path = ctx.repo_root.join("ops/inventory/control-graph.json");
    let Some(value) = read_json(&path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "control-graph must be valid json",
            Some("ops/inventory/control-graph.json".to_string()),
        )]);
    };
    let mut violations = Vec::new();
    let Some(nodes) = value.get("nodes").and_then(|v| v.as_array()) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "control-graph must include nodes array",
            Some("ops/inventory/control-graph.json".to_string()),
        )]);
    };
    let Some(edges) = value.get("edges").and_then(|v| v.as_array()) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "control-graph must include edges array",
            Some("ops/inventory/control-graph.json".to_string()),
        )]);
    };

    let mut node_ids = BTreeSet::new();
    for node in nodes {
        let id = node.get("id").and_then(|v| v.as_str()).unwrap_or("");
        if id.is_empty() {
            violations.push(violation(
                contract_id,
                test_id,
                "control-graph nodes must include non-empty id",
                Some("ops/inventory/control-graph.json".to_string()),
            ));
            continue;
        }
        if !node_ids.insert(id.to_string()) {
            violations.push(violation(
                contract_id,
                test_id,
                "control-graph node ids must be unique",
                Some("ops/inventory/control-graph.json".to_string()),
            ));
        }
        let consumer = node.get("consumer").and_then(|v| v.as_str()).unwrap_or("");
        if consumer.is_empty() {
            violations.push(violation(
                contract_id,
                test_id,
                "control-graph nodes must include consumer contract mapping",
                Some("ops/inventory/control-graph.json".to_string()),
            ));
        }
    }

    let cycle_kinds = BTreeSet::from(["dependency", "lifecycle"]);
    let mut adjacency: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for edge in edges {
        let from = edge.get("from").and_then(|v| v.as_str()).unwrap_or("");
        let to = edge.get("to").and_then(|v| v.as_str()).unwrap_or("");
        let kind = edge.get("kind").and_then(|v| v.as_str()).unwrap_or("");
        if from.is_empty() || to.is_empty() || kind.is_empty() {
            violations.push(violation(
                contract_id,
                test_id,
                "control-graph edges require from/to/kind",
                Some("ops/inventory/control-graph.json".to_string()),
            ));
            continue;
        }
        if !node_ids.contains(from) || !node_ids.contains(to) {
            violations.push(violation(
                contract_id,
                test_id,
                "control-graph edge references unknown node id",
                Some("ops/inventory/control-graph.json".to_string()),
            ));
        }
        if cycle_kinds.contains(kind) {
            adjacency
                .entry(from.to_string())
                .or_default()
                .push(to.to_string());
        }
    }

    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    fn has_cycle(
        node: &str,
        adjacency: &BTreeMap<String, Vec<String>>,
        visiting: &mut BTreeSet<String>,
        visited: &mut BTreeSet<String>,
    ) -> bool {
        if visited.contains(node) {
            return false;
        }
        if !visiting.insert(node.to_string()) {
            return true;
        }
        if let Some(next) = adjacency.get(node) {
            for child in next {
                if has_cycle(child, adjacency, visiting, visited) {
                    return true;
                }
            }
        }
        visiting.remove(node);
        visited.insert(node.to_string());
        false
    }
    for node in adjacency.keys() {
        if has_cycle(node, &adjacency, &mut visiting, &mut visited) {
            violations.push(violation(
                contract_id,
                test_id,
                "control-graph contains forbidden cycle in dependency/consumer/producer/lifecycle/drift edges",
                Some("ops/inventory/control-graph.json".to_string()),
            ));
            break;
        }
    }

    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_inv_007_gates_registry_mapped(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-INV-007";
    let test_id = "ops.inventory.gates_registry_mapped";
    let path = ctx.repo_root.join("ops/inventory/gates.json");
    let Some(value) = read_json(&path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "gates registry must exist and be valid json",
            Some("ops/inventory/gates.json".to_string()),
        )]);
    };
    let Some(gates) = value.get("gates").and_then(|v| v.as_array()) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "gates registry must include a gates array",
            Some("ops/inventory/gates.json".to_string()),
        )]);
    };
    let mut ids = BTreeSet::new();
    let mut violations = Vec::new();
    for gate in gates {
        let gate_id = gate.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let action_id = gate.get("action_id").and_then(|v| v.as_str()).unwrap_or("");
        if gate_id.is_empty() {
            violations.push(violation(
                contract_id,
                test_id,
                "gate entry must have non-empty id",
                Some("ops/inventory/gates.json".to_string()),
            ));
            continue;
        }
        if !ids.insert(gate_id.to_string()) {
            violations.push(violation(
                contract_id,
                test_id,
                "gate ids must be unique",
                Some("ops/inventory/gates.json".to_string()),
            ));
        }
        if action_id.is_empty() || !action_id.starts_with("ops.") {
            violations.push(violation(
                contract_id,
                test_id,
                "gate action_id must be non-empty and start with ops.",
                Some("ops/inventory/gates.json".to_string()),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_inv_008_drills_registry_mapped(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-INV-008";
    let test_id = "ops.inventory.drills_registry_mapped";
    let registry_path = ctx.repo_root.join("ops/inventory/drills.json");
    let links_path = ctx.repo_root.join("ops/inventory/drill-contract-links.json");
    let Some(registry) = read_json(&registry_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "drills registry must exist and be valid json",
            Some("ops/inventory/drills.json".to_string()),
        )]);
    };
    let Some(links) = read_json(&links_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "drill-contract-links must exist and be valid json",
            Some("ops/inventory/drill-contract-links.json".to_string()),
        )]);
    };
    let registry_ids: BTreeSet<String> = registry
        .get("drills")
        .and_then(|v| v.as_array())
        .map(|rows| {
            rows.iter()
                .filter_map(|v| v.as_str().map(ToOwned::to_owned))
                .collect()
        })
        .unwrap_or_default();
    if registry_ids.is_empty() {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "drills registry must list at least one drill id",
            Some("ops/inventory/drills.json".to_string()),
        )]);
    }
    let mapped_ids: BTreeSet<String> = links
        .get("links")
        .and_then(|v| v.as_array())
        .map(|rows| {
            rows.iter()
                .filter_map(|v| v.get("drill_id").and_then(|v| v.as_str()))
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let mut violations = Vec::new();
    for drill_id in &registry_ids {
        if !drill_id.starts_with("ops.drill.") {
            violations.push(violation(
                contract_id,
                test_id,
                "drill ids must use ops.drill.<name> format",
                Some("ops/inventory/drills.json".to_string()),
            ));
            continue;
        }
        if !mapped_ids.contains(drill_id) {
            violations.push(violation(
                contract_id,
                test_id,
                "drill id must map to at least one contract link entry",
                Some(drill_id.to_string()),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_inv_009_owners_registry_complete(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-INV-009";
    let test_id = "ops.inventory.owners_registry_complete";
    let owners_path = ctx.repo_root.join("ops/inventory/owners.json");
    let Some(owners_doc) = read_json(&owners_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "owners registry must exist and be valid json",
            Some("ops/inventory/owners.json".to_string()),
        )]);
    };
    let Some(areas) = owners_doc.get("areas").and_then(|v| v.as_object()) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "owners registry must contain areas object",
            Some("ops/inventory/owners.json".to_string()),
        )]);
    };
    let mut violations = Vec::new();
    for domain in DOMAIN_DIRS {
        let key = format!("ops/{domain}");
        if !areas.contains_key(&key) {
            violations.push(violation(
                contract_id,
                test_id,
                "owners registry must include every ops pillar/domain directory",
                Some(key),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_inv_010_inventory_schema_coverage(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-INV-010";
    let test_id = "ops.inventory.schema_coverage";
    let required = [
        "contracts",
        "tools",
        "pins",
        "gates",
        "drills",
        "owners",
    ];
    let mut violations = Vec::new();
    for name in required {
        let schema_path = ctx
            .repo_root
            .join(format!("ops/schema/inventory/{name}.schema.json"));
        if !schema_path.exists() {
            violations.push(violation(
                contract_id,
                test_id,
                "required inventory schema file missing",
                Some(rel_to_root(&schema_path, &ctx.repo_root)),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

