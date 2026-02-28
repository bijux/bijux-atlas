fn test_ops_load_010_every_scenario_has_slo_impact_class(ctx: &RunContext) -> TestResult {
    let (contract_id, test_id, suites_rel, map_rel) = ("OPS-LOAD-010", "ops.load.every_scenario_has_slo_impact_class", "ops/load/suites/suites.json", "ops/inventory/scenario-slo-map.json");
    let Some(suites) = read_json(&ctx.repo_root.join(suites_rel)) else { return TestResult::Fail(vec![violation(contract_id, test_id, "load suites must be parseable json", Some(suites_rel.to_string()))]); };
    let Some(map) = read_json(&ctx.repo_root.join(map_rel)) else { return TestResult::Fail(vec![violation(contract_id, test_id, "scenario slo map must be parseable json", Some(map_rel.to_string()))]); };
    let mut mapped = BTreeSet::new();
    if let Some(rows) = map.get("mappings").and_then(|v| v.as_array()) { for row in rows { if let Some(items) = row.get("load_suites").and_then(|v| v.as_array()) { for suite in items.iter().filter_map(|v| v.as_str()) { mapped.insert(suite.to_string()); } } } }
    let mut violations = Vec::new();
    if let Some(rows) = suites.get("suites").and_then(|v| v.as_array()) { for row in rows { if let Some(name) = row.get("name").and_then(|v| v.as_str()) { if !mapped.contains(name) { violations.push(violation(contract_id, test_id, "load suite must map to scenario-slo-map load_suites coverage", Some(format!("{map_rel}#{name}")))); } } } }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_ops_datasets_005_qc_metadata_and_golden_valid(ctx: &RunContext) -> TestResult {
    let (contract_id, test_id, rel) = ("OPS-DATASETS-005", "ops.datasets.qc_metadata_and_golden_valid", "ops/datasets/qc-metadata.json");
    let Some(meta) = read_json(&ctx.repo_root.join(rel)) else { return TestResult::Fail(vec![violation(contract_id, test_id, "qc metadata must be parseable json", Some(rel.to_string()))]); };
    let golden_rel = meta.get("golden_summary").and_then(|v| v.as_str()).unwrap_or("");
    if !golden_rel.is_empty() && read_json(&ctx.repo_root.join(golden_rel)).is_some() { TestResult::Pass } else { TestResult::Fail(vec![violation(contract_id, test_id, "qc metadata golden_summary must point to parseable json", Some(rel.to_string()))]) }
}

fn test_ops_datasets_006_rollback_policy_exists_valid(ctx: &RunContext) -> TestResult {
    let (contract_id, test_id, rel) = ("OPS-DATASETS-006", "ops.datasets.rollback_policy_exists_valid", "ops/datasets/rollback-policy.json");
    let Some(policy) = read_json(&ctx.repo_root.join(rel)) else { return TestResult::Fail(vec![violation(contract_id, test_id, "rollback policy must be parseable json", Some(rel.to_string()))]); };
    let valid = policy.get("rollback_steps").and_then(|v| v.as_array()).is_some_and(|v| !v.is_empty()) && policy.get("requires").and_then(|v| v.as_array()).is_some_and(|v| !v.is_empty());
    if valid { TestResult::Pass } else { TestResult::Fail(vec![violation(contract_id, test_id, "rollback policy must include rollback_steps and requires arrays", Some(rel.to_string()))]) }
}

fn test_ops_datasets_007_promotion_rules_exists_valid(ctx: &RunContext) -> TestResult {
    let (contract_id, test_id, rel) = ("OPS-DATASETS-007", "ops.datasets.promotion_rules_exists_valid", "ops/datasets/promotion-rules.json");
    let Some(rules) = read_json(&ctx.repo_root.join(rel)) else { return TestResult::Fail(vec![violation(contract_id, test_id, "promotion rules must be parseable json", Some(rel.to_string()))]); };
    let pins = rules.get("pins_source").and_then(|v| v.as_str()).unwrap_or("");
    let lock = rules.get("manifest_lock").and_then(|v| v.as_str()).unwrap_or("");
    let envs: BTreeSet<String> = rules.get("environments").and_then(|v| v.as_array()).into_iter().flatten().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
    let required: BTreeSet<String> = ["dev", "ci", "prod"].into_iter().map(std::string::ToString::to_string).collect();
    if !pins.is_empty() && !lock.is_empty() && ctx.repo_root.join(pins).exists() && ctx.repo_root.join(lock).exists() && envs == required { TestResult::Pass } else { TestResult::Fail(vec![violation(contract_id, test_id, "promotion rules must reference pins+lock and include dev/ci/prod", Some(rel.to_string()))]) }
}

fn test_ops_datasets_008_consumer_list_consistent_with_runtime_queries(ctx: &RunContext) -> TestResult {
    let (contract_id, test_id, rel) = ("OPS-DATASETS-008", "ops.datasets.consumer_list_consistent_with_runtime_queries", "ops/datasets/consumer-list.json");
    let Some(consumers) = read_json(&ctx.repo_root.join(rel)) else { return TestResult::Fail(vec![violation(contract_id, test_id, "consumer list must be parseable json", Some(rel.to_string()))]); };
    let mut violations = Vec::new();
    if let Some(rows) = consumers.get("consumers").and_then(|v| v.as_array()) { for row in rows { let interface = row.get("interface").and_then(|v| v.as_str()).unwrap_or(""); if interface.is_empty() || !ctx.repo_root.join(interface).exists() { violations.push(violation(contract_id, test_id, "consumer interface must reference an existing repository path", Some(rel.to_string()))); } } }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_ops_datasets_009_freeze_policy_exists_enforced(ctx: &RunContext) -> TestResult {
    let (contract_id, test_id, rel) = ("OPS-DATASETS-009", "ops.datasets.freeze_policy_exists_enforced", "ops/datasets/freeze-policy.json");
    let Some(policy) = read_json(&ctx.repo_root.join(rel)) else { return TestResult::Fail(vec![violation(contract_id, test_id, "freeze policy must be parseable json", Some(rel.to_string()))]); };
    let append_only = policy.pointer("/immutability/fixture_assets_append_only").and_then(|v| v.as_bool()) == Some(true);
    let forbid_replace = policy.pointer("/immutability/allow_archive_replacement").and_then(|v| v.as_bool()) == Some(false);
    if append_only && forbid_replace { TestResult::Pass } else { TestResult::Fail(vec![violation(contract_id, test_id, "freeze policy must enforce append-only assets and forbid replacement", Some(rel.to_string()))]) }
}

fn test_ops_datasets_010_dataset_store_layout_contract_enforced(ctx: &RunContext) -> TestResult {
    let (contract_id, test_id, rel) = ("OPS-DATASETS-010", "ops.datasets.dataset_store_layout_contract_enforced", "ops/datasets/manifest.json");
    let Some(manifest) = read_json(&ctx.repo_root.join(rel)) else { return TestResult::Fail(vec![violation(contract_id, test_id, "datasets manifest must be parseable json", Some(rel.to_string()))]); };
    let mut violations = Vec::new();
    if let Some(rows) = manifest.get("datasets").and_then(|v| v.as_array()) {
        for row in rows {
            let id = row.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let parts: Vec<&str> = id.split('/').collect();
            if parts.len() != 3 || parts.iter().any(|p| p.is_empty()) {
                violations.push(violation(contract_id, test_id, "dataset id must follow release/species/assembly layout", Some(rel.to_string())));
            }
            if let Some(paths) = row.get("paths").and_then(|v| v.as_object()) {
                for value in paths.values().filter_map(|v| v.as_str()) {
                    if !value.starts_with("ops/datasets/fixtures/") {
                        violations.push(violation(contract_id, test_id, "dataset fixture paths must live under ops/datasets/fixtures", Some(value.to_string())));
                    }
                }
            }
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}
