fn test_ops_e2e_001_suites_reference_real_scenarios(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-E2E-001";
    let test_id = "ops.e2e.suites_reference_real_scenarios";
    let suites_path = ctx.repo_root.join("ops/e2e/suites/suites.json");
    let Some(suites) = read_json(&suites_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "e2e suites must be valid json",
            Some("ops/e2e/suites/suites.json".to_string()),
        )]);
    };
    let mut violations = Vec::new();
    let mut seen_ids = BTreeSet::new();
    let Some(items) = suites.get("suites").and_then(|v| v.as_array()) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "e2e suites file must contain suites array",
            Some("ops/e2e/suites/suites.json".to_string()),
        )]);
    };
    for suite in items {
        if let Some(scenarios) = suite.get("scenarios").and_then(|v| v.as_array()) {
            for scenario in scenarios {
                let id = scenario.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let runner = scenario
                    .get("runner")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if id.is_empty() || !seen_ids.insert(id.to_string()) {
                    violations.push(violation(
                        contract_id,
                        test_id,
                        "scenario id must be non-empty and globally unique in suites.json",
                        Some("ops/e2e/suites/suites.json".to_string()),
                    ));
                }
                if runner.is_empty() {
                    violations.push(violation(
                        contract_id,
                        test_id,
                        "scenario runner must be non-empty",
                        Some("ops/e2e/suites/suites.json".to_string()),
                    ));
                }
                if runner.contains("python3 ") && !runner.contains(".py") {
                    violations.push(violation(
                        contract_id,
                        test_id,
                        "python runner command must reference a python source path",
                        Some("ops/e2e/suites/suites.json".to_string()),
                    ));
                }
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_e2e_002_smoke_manifest_valid(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-E2E-002";
    let test_id = "ops.e2e.smoke_manifest_valid";
    let manifest_rel = "ops/e2e/manifests/smoke.manifest.json";
    let Some(manifest) = read_json(&ctx.repo_root.join(manifest_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "smoke manifest must be valid json",
            Some(manifest_rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    if manifest
        .get("schema_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .is_empty()
    {
        violations.push(violation(
            contract_id,
            test_id,
            "smoke manifest requires schema_name",
            Some(manifest_rel.to_string()),
        ));
    }
    let queries_lock = manifest
        .get("queries_lock")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if queries_lock.is_empty() || !ctx.repo_root.join(queries_lock).exists() {
        violations.push(violation(
            contract_id,
            test_id,
            "smoke manifest queries_lock must reference existing file",
            Some(manifest_rel.to_string()),
        ));
    } else if let Ok(contents) = fs::read_to_string(ctx.repo_root.join(queries_lock)) {
        let entries = contents
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        if entries.is_empty() {
            violations.push(violation(
                contract_id,
                test_id,
                "smoke manifest queries_lock must contain at least one query",
                Some(queries_lock.to_string()),
            ));
        } else {
            let unique = entries.iter().cloned().collect::<BTreeSet<_>>();
            if unique.len() != entries.len() {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "smoke manifest queries_lock entries must be unique",
                    Some(queries_lock.to_string()),
                ));
            }
            let mut sorted = entries.clone();
            sorted.sort();
            if entries != sorted {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "smoke manifest queries_lock entries must stay lexicographically sorted",
                    Some(queries_lock.to_string()),
                ));
            }
        }
    }
    if manifest
        .get("steps")
        .and_then(|v| v.as_array())
        .is_none_or(|steps| steps.is_empty())
    {
        violations.push(violation(
            contract_id,
            test_id,
            "smoke manifest must include at least one step",
            Some(manifest_rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_e2e_003_fixtures_lock_matches_allowlist(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-E2E-003";
    let test_id = "ops.e2e.fixtures_lock_matches_allowlist";
    let lock_rel = "ops/e2e/fixtures/fixtures.lock";
    let allow_rel = "ops/e2e/fixtures/allowlist.json";
    let lock_path = ctx.repo_root.join(lock_rel);
    let allow_path = ctx.repo_root.join(allow_rel);

    let Some(lock) = read_json(&lock_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "fixtures.lock must be valid json",
            Some(lock_rel.to_string()),
        )]);
    };
    let Some(allow) = read_json(&allow_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "allowlist.json must be valid json",
            Some(allow_rel.to_string()),
        )]);
    };

    let mut violations = Vec::new();
    let expected_sha = file_sha256(&allow_path);
    let actual_sha = lock
        .get("allowlist_sha256")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if expected_sha.as_deref().is_some_and(|sha| sha != actual_sha) {
        violations.push(violation(
            contract_id,
            test_id,
            "fixtures.lock allowlist_sha256 must match allowlist.json digest",
            Some(lock_rel.to_string()),
        ));
    }

    let mut allowed_paths = BTreeSet::new();
    if let Some(paths) = allow.get("allowed_paths").and_then(|v| v.as_array()) {
        for path in paths {
            if let Some(path) = path.as_str() {
                allowed_paths.insert(path.to_string());
            }
        }
    }
    let fixtures_dir = ctx.repo_root.join("ops/e2e/fixtures");
    if let Ok(entries) = std::fs::read_dir(fixtures_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let rel = rel_to_root(&path, &ctx.repo_root);
            if !allowed_paths.contains(&rel) {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "fixture file is not declared in allowlist",
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

fn test_ops_e2e_004_realdata_snapshots_schema_valid_and_pinned(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-E2E-004";
    let test_id = "ops.e2e.realdata_snapshots_schema_valid_and_pinned";
    let snapshots_root = ctx.repo_root.join("ops/e2e/realdata/snapshots");
    let canonical_rel = "ops/e2e/realdata/canonical_queries.json";
    let mut files = Vec::new();
    walk_files(&snapshots_root, &mut files);
    files.sort();
    if files.is_empty() {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "at least one realdata snapshot is required",
            Some("ops/e2e/realdata/snapshots".to_string()),
        )]);
    }
    let mut violations = Vec::new();
    for path in files {
        let rel = rel_to_root(&path, &ctx.repo_root);
        if !rel.ends_with(".json") {
            continue;
        }
        let Some(value) = read_json(&path) else {
            violations.push(violation(
                contract_id,
                test_id,
                "snapshot file must be valid json",
                Some(rel),
            ));
            continue;
        };
        let generated_from = value
            .get("generated_from")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if generated_from != canonical_rel {
            violations.push(violation(
                contract_id,
                test_id,
                "snapshot generated_from must pin canonical query source",
                Some(rel.clone()),
            ));
        }
        if value.get("entries").and_then(|v| v.as_array()).is_none() {
            violations.push(violation(
                contract_id,
                test_id,
                "snapshot must expose entries array",
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

fn test_ops_e2e_005_taxonomy_covers_scenarios(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-E2E-005";
    let test_id = "ops.e2e.taxonomy_covers_scenarios";
    let taxonomy_rel = "ops/e2e/taxonomy.json";
    let scenarios_rel = "ops/e2e/scenarios/scenarios.json";
    let summary_rel = "ops/e2e/generated/e2e-summary.json";
    let Some(taxonomy) = read_json(&ctx.repo_root.join(taxonomy_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "taxonomy.json must be valid json",
            Some(taxonomy_rel.to_string()),
        )]);
    };
    let Some(scenarios) = read_json(&ctx.repo_root.join(scenarios_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "scenarios.json must be valid json",
            Some(scenarios_rel.to_string()),
        )]);
    };
    let Some(summary) = read_json(&ctx.repo_root.join(summary_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "e2e summary must be valid json",
            Some(summary_rel.to_string()),
        )]);
    };

    let mut violations = Vec::new();
    if taxonomy.get("schema_version").and_then(|v| v.as_i64()) != Some(1) {
        violations.push(violation(
            contract_id,
            test_id,
            "taxonomy schema_version must be 1",
            Some(taxonomy_rel.to_string()),
        ));
    }

    let mut category_ids = BTreeSet::new();
    for category in taxonomy
        .get("categories")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        let Some(id) = category.get("id").and_then(|v| v.as_str()) else {
            continue;
        };
        if !category_ids.insert(id.to_string()) {
            violations.push(violation(
                contract_id,
                test_id,
                "taxonomy categories must be unique",
                Some(taxonomy_rel.to_string()),
            ));
        }
    }
    let required_categories = BTreeSet::from([
        "smoke".to_string(),
        "kubernetes".to_string(),
        "realdata".to_string(),
        "performance".to_string(),
    ]);
    if category_ids != required_categories {
        violations.push(violation(
            contract_id,
            test_id,
            "taxonomy categories must exactly match required category set",
            Some(taxonomy_rel.to_string()),
        ));
    }

    let scenario_category = BTreeMap::from([
        ("smoke", "smoke"),
        ("k8s-suite", "kubernetes"),
        ("realdata", "realdata"),
        ("perf-e2e", "performance"),
    ]);
    let mut scenario_ids = BTreeSet::new();
    for scenario in scenarios
        .get("scenarios")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        let Some(id) = scenario.get("id").and_then(|v| v.as_str()) else {
            continue;
        };
        scenario_ids.insert(id.to_string());
        let Some(category) = scenario_category.get(id) else {
            violations.push(violation(
                contract_id,
                test_id,
                "scenario id is missing category mapping",
                Some(scenarios_rel.to_string()),
            ));
            continue;
        };
        if !category_ids.contains(*category) {
            violations.push(violation(
                contract_id,
                test_id,
                "scenario references taxonomy category that does not exist",
                Some(scenarios_rel.to_string()),
            ));
        }
    }
    let summary_scenarios: BTreeSet<String> = summary
        .get("scenario_ids")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    if summary_scenarios != scenario_ids {
        violations.push(violation(
            contract_id,
            test_id,
            "summary scenario_ids must match scenarios registry ids",
            Some(summary_rel.to_string()),
        ));
    }

    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

