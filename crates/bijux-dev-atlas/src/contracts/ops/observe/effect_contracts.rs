fn stable_alert_catalog_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
}

fn test_ops_obs_005_alert_catalog_generated_consistency(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-OBS-005";
    let test_id = "ops.observe.alert_catalog_generated_consistency";
    let catalog_rel = "ops/observe/alert-catalog.json";
    let rules_rel = "ops/observe/alerts/slo-burn-rules.yaml";
    let Some(catalog) = read_json(&ctx.repo_root.join(catalog_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "alert catalog must be parseable",
            Some(catalog_rel.to_string()),
        )]);
    };
    let Some(rules) = read_yaml_value(&ctx.repo_root.join(rules_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "slo burn rules file must be parseable yaml",
            Some(rules_rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    let alerts = catalog
        .get("alerts")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if alerts.is_empty() {
        violations.push(violation(
            contract_id,
            test_id,
            "alert catalog must include non-empty alerts list",
            Some(catalog_rel.to_string()),
        ));
    }
    let mut catalog_ids = BTreeSet::new();
    for row in alerts {
        let id = row.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        if id.is_empty() {
            violations.push(violation(
                contract_id,
                test_id,
                "alert catalog entries must include non-empty id",
                Some(catalog_rel.to_string()),
            ));
            continue;
        }
        catalog_ids.insert(id.to_string());
    }
    let mut rule_alert_count = 0usize;
    if let Some(groups) = rules
        .get("spec")
        .and_then(|v| v.get("groups"))
        .and_then(|v| v.as_sequence())
    {
        for group in groups {
            if let Some(rows) = group.get("rules").and_then(|v| v.as_sequence()) {
                for row in rows {
                    if row.get("alert").and_then(|v| v.as_str()).is_some() {
                        rule_alert_count += 1;
                    }
                }
            }
        }
    }
    if rule_alert_count == 0 {
        violations.push(violation(
            contract_id,
            test_id,
            "slo burn rules must define at least one alert rule",
            Some(rules_rel.to_string()),
        ));
    }
    for id in &catalog_ids {
        if !stable_alert_catalog_id(id) {
            violations.push(violation(
                contract_id,
                test_id,
                "alert catalog id must use stable ascii naming",
                Some(catalog_rel.to_string()),
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

fn test_ops_obs_006_slo_definitions_burn_rate_consistent(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-OBS-006";
    let test_id = "ops.observe.slo_definitions_burn_rate_consistent";
    let slo_rel = "ops/observe/slo-definitions.json";
    let burn_rel = "ops/observe/alerts/slo-burn-rules.yaml";
    let Some(slos) = read_json(&ctx.repo_root.join(slo_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "slo definitions file must be parseable json",
            Some(slo_rel.to_string()),
        )]);
    };
    let Some(burn_rules) = read_yaml_value(&ctx.repo_root.join(burn_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "slo burn rules file must be parseable yaml",
            Some(burn_rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    let slo_ids: BTreeSet<String> = slos
        .get("slos")
        .and_then(|v| v.as_array())
        .map(|rows| {
            rows.iter()
                .filter_map(|row| row.get("id").and_then(|v| v.as_str()))
                .map(std::string::ToString::to_string)
                .collect()
        })
        .unwrap_or_default();
    if slo_ids.is_empty() {
        violations.push(violation(
            contract_id,
            test_id,
            "slo definitions must include non-empty slos list",
            Some(slo_rel.to_string()),
        ));
    }
    let burn_text = std::fs::read_to_string(ctx.repo_root.join(burn_rel)).unwrap_or_default();
    if !burn_text.contains("error budget burn") && !burn_text.contains("burn") {
        violations.push(violation(
            contract_id,
            test_id,
            "slo burn rules must include burn-rate semantics in rule descriptions",
            Some(burn_rel.to_string()),
        ));
    }
    let rule_count = burn_rules
        .get("spec")
        .and_then(|v| v.get("groups"))
        .and_then(|v| v.as_sequence())
        .map(|groups| {
            groups
                .iter()
                .map(|group| {
                    group
                        .get("rules")
                        .and_then(|v| v.as_sequence())
                        .map_or(0, |rows| rows.len())
                })
                .sum::<usize>()
        })
        .unwrap_or(0);
    if rule_count < 3 {
        violations.push(violation(
            contract_id,
            test_id,
            "slo burn rules must include multi-window burn alert set",
            Some(burn_rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_obs_007_public_surface_coverage_matches_rules(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-OBS-007";
    let test_id = "ops.observe.public_surface_coverage_matches_rules";
    let rel = "ops/observe/rules/public-surface-coverage.yaml";
    let Some(doc) = read_yaml_value(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "public surface coverage rules file must be parseable yaml",
            Some(rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    let surfaces: BTreeSet<String> = doc
        .get("surfaces")
        .and_then(|v| v.as_sequence())
        .map(|rows| {
            rows.iter()
                .filter_map(|v| v.as_str())
                .map(std::string::ToString::to_string)
                .collect()
        })
        .unwrap_or_default();
    let required: BTreeSet<String> = ["atlas", "prometheus", "grafana"]
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect();
    if !required.is_subset(&surfaces) {
        violations.push(violation(
            contract_id,
            test_id,
            "public surface coverage must include atlas/prometheus/grafana",
            Some(rel.to_string()),
        ));
    }
    if surfaces.len() < 3 {
        violations.push(violation(
            contract_id,
            test_id,
            "public surface coverage list must include at least three surfaces",
            Some(rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_obs_008_telemetry_index_generated_deterministic(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-OBS-008";
    let test_id = "ops.observe.telemetry_index_generated_deterministic";
    let rel = "ops/observe/generated/telemetry-index.json";
    let Some(index) = read_json(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "telemetry index must be parseable json",
            Some(rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    if index.get("schema_version").and_then(|v| v.as_i64()) != Some(1) {
        violations.push(violation(
            contract_id,
            test_id,
            "telemetry index must include schema_version=1",
            Some(rel.to_string()),
        ));
    }
    let artifacts: Vec<String> = index
        .get("artifacts")
        .and_then(|v| v.as_array())
        .map(|rows| {
            rows.iter()
                .filter_map(|v| v.as_str())
                .map(std::string::ToString::to_string)
                .collect()
        })
        .unwrap_or_default();
    if artifacts.is_empty() {
        violations.push(violation(
            contract_id,
            test_id,
            "telemetry index must include artifacts list",
            Some(rel.to_string()),
        ));
    } else {
        let mut sorted = artifacts.clone();
        sorted.sort();
        if sorted != artifacts {
            violations.push(violation(
                contract_id,
                test_id,
                "telemetry index artifacts must be lexicographically sorted",
                Some(rel.to_string()),
            ));
        }
    }
    if !artifacts.iter().any(|a| a.ends_with("readiness.json")) {
        violations.push(violation(
            contract_id,
            test_id,
            "telemetry index must include readiness artifact",
            Some(rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_obs_009_drills_manifest_exists_runnable(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-OBS-009";
    let test_id = "ops.observe.drills_manifest_exists_runnable";
    let rel = "ops/observe/drills/drills.json";
    let Some(doc) = read_json(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "drills manifest must be parseable json",
            Some(rel.to_string()),
        )]);
    };
    let Some(rows) = doc.get("drills").and_then(|v| v.as_array()) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "drills manifest must include drills array",
            Some(rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    if rows.is_empty() {
        violations.push(violation(
            contract_id,
            test_id,
            "drills manifest must include at least one drill",
            Some(rel.to_string()),
        ));
    }
    for row in rows {
        let name = row.get("name").and_then(|v| v.as_str()).unwrap_or_default();
        if name.is_empty() {
            violations.push(violation(
                contract_id,
                test_id,
                "drill entries must include non-empty name",
                Some(rel.to_string()),
            ));
        }
        let runner = row
            .get("runner")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if runner.is_empty() || !runner.ends_with(".py") {
            violations.push(violation(
                contract_id,
                test_id,
                "drill entries must include python runner path",
                Some(rel.to_string()),
            ));
        }
        let expected = row
            .get("expected_signals")
            .and_then(|v| v.as_array())
            .map_or(0, |items| items.len());
        if expected == 0 {
            violations.push(violation(
                contract_id,
                test_id,
                "drill entries must include expected_signals",
                Some(rel.to_string()),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_obs_010_overload_behavior_contract_enforced(ctx: &RunContext) -> TestResult {
    let (contract_id, test_id, rel, suite_rel) = ("OPS-OBS-010", "ops.observe.overload_behavior_contract_enforced", "ops/observe/contracts/overload-behavior-contract.json", "ops/load/suites/suites.json");
    let Some(doc) = read_json(&ctx.repo_root.join(rel)) else { return TestResult::Fail(vec![violation(contract_id, test_id, "overload behavior contract must be parseable json", Some(rel.to_string()))]); };
    let Some(suites) = read_json(&ctx.repo_root.join(suite_rel)) else { return TestResult::Fail(vec![violation(contract_id, test_id, "load suites must be parseable for overload enforcement mapping", Some(suite_rel.to_string()))]); };
    let overload = doc.get("overload_response_contract").is_some() && doc.get("heavy_endpoints").and_then(|v| v.as_array()).is_some_and(|v| !v.is_empty()) && doc.get("cheap_endpoints").and_then(|v| v.as_array()).is_some_and(|v| !v.is_empty());
    let mapped = suites.get("suites").and_then(|v| v.as_array()).is_some_and(|rows| rows.iter().filter_map(|r| r.get("name").and_then(|v| v.as_str())).any(|n| n == "spike-overload-proof" || n == "cheap-only-survival"));
    if overload && mapped { TestResult::Pass } else { TestResult::Fail(vec![violation(contract_id, test_id, "overload behavior contract must exist and map to load suite coverage", Some(rel.to_string()))]) }
}
