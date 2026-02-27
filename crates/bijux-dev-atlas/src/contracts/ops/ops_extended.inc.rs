// SPDX-License-Identifier: Apache-2.0

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

fn test_ops_schema_001_parseable_documents(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-SCHEMA-001";
    let test_id = "ops.schema.parseable_documents";
    let mut files = Vec::new();
    walk_files(&ops_root(&ctx.repo_root), &mut files);
    files.sort();
    let mut violations = Vec::new();
    for path in files {
        let rel = rel_to_root(&path, &ctx.repo_root);
        if rel.starts_with("ops/k8s/charts/") && rel.contains("/templates/") {
            continue;
        }
        if rel.ends_with(".json") {
            if read_json(&path).is_none() {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "json document under ops must be parseable",
                    Some(rel),
                ));
            }
            continue;
        }
        if rel.ends_with(".yaml") || rel.ends_with(".yml") {
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            if text.contains("{{") || text.contains("{%") {
                continue;
            }
            let mut parsed_any = false;
            let mut parsed_all = true;
            for doc in serde_yaml::Deserializer::from_str(&text) {
                parsed_any = true;
                if serde_yaml::Value::deserialize(doc).is_err() {
                    parsed_all = false;
                    break;
                }
            }
            if !parsed_any || !parsed_all {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "yaml document under ops must be parseable",
                    Some(rel),
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

fn test_ops_schema_002_schema_index_complete(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-SCHEMA-002";
    let test_id = "ops.schema.index_complete";
    let index_path = ctx.repo_root.join("ops/schema/generated/schema-index.json");
    let Some(index) = read_json(&index_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "schema index must be parseable",
            Some("ops/schema/generated/schema-index.json".to_string()),
        )]);
    };
    let mut indexed = BTreeSet::new();
    if let Some(files) = index.get("files").and_then(|v| v.as_array()) {
        for item in files {
            if let Some(path) = item.as_str() {
                indexed.insert(path.to_string());
            }
        }
    }
    let mut actual = BTreeSet::new();
    let mut files = Vec::new();
    walk_files(&ctx.repo_root.join("ops/schema"), &mut files);
    for path in files {
        let rel = rel_to_root(&path, &ctx.repo_root);
        if rel.ends_with(".schema.json") {
            actual.insert(rel);
        }
    }
    let mut violations = Vec::new();
    for path in actual.difference(&indexed) {
        violations.push(violation(
            contract_id,
            test_id,
            "schema file missing from generated schema index",
            Some(path.clone()),
        ));
    }
    for path in indexed.difference(&actual) {
        violations.push(violation(
            contract_id,
            test_id,
            "schema index references schema file that is missing on disk",
            Some(path.clone()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_schema_003_no_unversioned_schemas(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-SCHEMA-003";
    let test_id = "ops.schema.no_unversioned";
    let mut files = Vec::new();
    walk_files(&ctx.repo_root.join("ops/schema"), &mut files);
    files.sort();
    let mut violations = Vec::new();
    for path in files {
        let rel = rel_to_root(&path, &ctx.repo_root);
        if rel.starts_with("ops/schema/generated/") {
            continue;
        }
        if rel == "ops/schema/report/schema.json" {
            continue;
        }
        if rel.ends_with(".json") && !rel.ends_with(".schema.json") {
            violations.push(violation(
                contract_id,
                test_id,
                "schema source files must use .schema.json naming",
                Some(rel),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_schema_004_budget_policy(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-SCHEMA-004";
    let test_id = "ops.schema.budget_policy";
    let budgets: BTreeMap<&str, usize> = BTreeMap::from([
        ("configs", 5),
        ("datasets", 20),
        ("e2e", 12),
        ("env", 5),
        ("inventory", 30),
        ("k8s", 12),
        ("load", 15),
        ("meta", 20),
        ("observe", 15),
        ("report", 25),
        ("stack", 12),
    ]);
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut files = Vec::new();
    walk_files(&ctx.repo_root.join("ops/schema"), &mut files);
    for path in files {
        let rel = rel_to_root(&path, &ctx.repo_root);
        if !rel.ends_with(".schema.json") {
            continue;
        }
        let parts = rel.split('/').collect::<Vec<_>>();
        if parts.len() < 4 {
            continue;
        }
        let domain = parts[2].to_string();
        *counts.entry(domain).or_insert(0) += 1;
    }
    let mut violations = Vec::new();
    for (domain, count) in counts {
        if let Some(limit) = budgets.get(domain.as_str()) {
            if count > *limit {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "schema count exceeds per-domain budget",
                    Some(format!("ops/schema/{domain}")),
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

fn test_ops_schema_005_evolution_lock(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-SCHEMA-005";
    let test_id = "ops.schema.evolution_lock";
    let lock_path = ctx
        .repo_root
        .join("ops/schema/generated/compatibility-lock.json");
    let Some(lock) = read_json(&lock_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "schema compatibility lock must be valid json",
            Some("ops/schema/generated/compatibility-lock.json".to_string()),
        )]);
    };
    let mut violations = Vec::new();
    let targets = lock.get("targets").and_then(|v| v.as_array());
    if targets.is_none_or(|arr| arr.is_empty()) {
        violations.push(violation(
            contract_id,
            test_id,
            "compatibility lock must contain non-empty targets",
            Some("ops/schema/generated/compatibility-lock.json".to_string()),
        ));
    }
    if let Some(targets) = targets {
        for target in targets {
            let Some(schema_path) = target.get("schema_path").and_then(|v| v.as_str()) else {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "compatibility lock target missing schema_path",
                    Some("ops/schema/generated/compatibility-lock.json".to_string()),
                ));
                continue;
            };
            if !ctx.repo_root.join(schema_path).exists() {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "compatibility lock target references missing schema path",
                    Some(schema_path.to_string()),
                ));
            }
            let required = target.get("required_fields").and_then(|v| v.as_array());
            if required.is_none_or(|r| r.is_empty()) {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "compatibility lock target requires non-empty required_fields",
                    Some(schema_path.to_string()),
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

fn test_ops_dataset_001_manifest_and_lock(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-DATASET-001";
    let test_id = "ops.dataset.manifest_and_lock_consistent";
    let manifest_path = ctx.repo_root.join("ops/datasets/manifest.json");
    let lock_path = ctx.repo_root.join("ops/datasets/manifest.lock");
    let Some(manifest) = read_json(&manifest_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "datasets manifest must be valid json",
            Some("ops/datasets/manifest.json".to_string()),
        )]);
    };
    let Some(lock) = read_json(&lock_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "datasets manifest lock must be valid json",
            Some("ops/datasets/manifest.lock".to_string()),
        )]);
    };
    let mut manifest_ids = BTreeSet::new();
    if let Some(items) = manifest.get("datasets").and_then(|v| v.as_array()) {
        for item in items {
            if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                manifest_ids.insert(id.to_string());
            }
        }
    }
    let mut lock_ids = BTreeSet::new();
    if let Some(items) = lock.get("entries").and_then(|v| v.as_array()) {
        for item in items {
            if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                lock_ids.insert(id.to_string());
            }
        }
    }
    let mut violations = Vec::new();
    if manifest_ids != lock_ids {
        violations.push(violation(
            contract_id,
            test_id,
            "datasets manifest ids must match manifest.lock ids",
            Some("ops/datasets/manifest.lock".to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_dataset_002_fixture_inventory_matches_disk(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-DATASET-002";
    let test_id = "ops.dataset.fixture_inventory_matches_disk";
    let inventory_path = ctx.repo_root.join("ops/datasets/generated/fixture-inventory.json");
    let Some(inventory) = read_json(&inventory_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "fixture inventory must be valid json",
            Some("ops/datasets/generated/fixture-inventory.json".to_string()),
        )]);
    };
    let fixtures_dir = ctx.repo_root.join("ops/datasets/fixtures");
    let mut disk_names = BTreeSet::new();
    if let Ok(entries) = std::fs::read_dir(&fixtures_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|v| v.to_str()) {
                disk_names.insert(name.to_string());
            }
        }
        }
    }
    let mut inventory_names = BTreeSet::new();
    if let Some(items) = inventory.get("fixtures").and_then(|v| v.as_array()) {
        for item in items {
            if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                inventory_names.insert(name.to_string());
            }
            let lock = item.get("manifest_lock").and_then(|v| v.as_str()).unwrap_or("");
            let src_dir = item.get("src_dir").and_then(|v| v.as_str()).unwrap_or("");
            if !lock.is_empty() && !ctx.repo_root.join(lock).exists() {
                return TestResult::Fail(vec![violation(
                    contract_id,
                    test_id,
                    "fixture inventory references missing manifest_lock",
                    Some(lock.to_string()),
                )]);
            }
            if !src_dir.is_empty() && !ctx.repo_root.join(src_dir).exists() {
                return TestResult::Fail(vec![violation(
                    contract_id,
                    test_id,
                    "fixture inventory references missing src_dir",
                    Some(src_dir.to_string()),
                )]);
            }
        }
    }
    let mut violations = Vec::new();
    if inventory_names != disk_names {
        violations.push(violation(
            contract_id,
            test_id,
            "fixture inventory names must match fixture directories on disk",
            Some("ops/datasets/generated/fixture-inventory.json".to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}


include!("ops_domains_31_40.inc.rs");
include!("ops_domains_41_50.inc.rs");
