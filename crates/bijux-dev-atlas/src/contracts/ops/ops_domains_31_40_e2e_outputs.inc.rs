fn test_ops_e2e_006_reproducibility_policy_enforced(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-E2E-006";
    let test_id = "ops.e2e.reproducibility_policy_enforced";
    let policy_rel = "ops/e2e/reproducibility-policy.json";
    let summary_rel = "ops/e2e/generated/e2e-summary.json";
    let Some(policy) = read_json(&ctx.repo_root.join(policy_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "reproducibility-policy.json must be valid json",
            Some(policy_rel.to_string()),
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
    if policy.get("schema_version").and_then(|v| v.as_i64()) != Some(1) {
        violations.push(violation(
            contract_id,
            test_id,
            "reproducibility policy schema_version must be 1",
            Some(policy_rel.to_string()),
        ));
    }
    if policy.get("ordering").and_then(|v| v.as_str()) != Some("stable") {
        violations.push(violation(
            contract_id,
            test_id,
            "reproducibility policy ordering must be stable",
            Some(policy_rel.to_string()),
        ));
    }
    let required_checks = BTreeSet::from([
        "scenario-order-deterministic".to_string(),
        "fixture-allowlist-enforced".to_string(),
        "expectation-catalog-complete".to_string(),
    ]);
    let checks: BTreeSet<String> = policy
        .get("required_checks")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    if checks != required_checks {
        violations.push(violation(
            contract_id,
            test_id,
            "reproducibility policy required_checks must match canonical checks",
            Some(policy_rel.to_string()),
        ));
    }
    let seed_source = policy
        .get("seed_source")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    if seed_source.is_empty() || !ctx.repo_root.join(seed_source).exists() {
        violations.push(violation(
            contract_id,
            test_id,
            "reproducibility policy seed_source must reference an existing file",
            Some(policy_rel.to_string()),
        ));
    }
    let suite_ids: Vec<String> = summary
        .get("suite_ids")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    let mut sorted_suite_ids = suite_ids.clone();
    sorted_suite_ids.sort();
    if suite_ids != sorted_suite_ids {
        violations.push(violation(
            contract_id,
            test_id,
            "e2e summary suite_ids must be deterministically sorted",
            Some(summary_rel.to_string()),
        ));
    }
    let scenario_ids: Vec<String> = summary
        .get("scenario_ids")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    let mut sorted_scenario_ids = scenario_ids.clone();
    sorted_scenario_ids.sort();
    if scenario_ids != sorted_scenario_ids {
        violations.push(violation(
            contract_id,
            test_id,
            "e2e summary scenario_ids must be deterministically sorted",
            Some(summary_rel.to_string()),
        ));
    }
    if summary.get("suite_count").and_then(|v| v.as_u64()) != Some(suite_ids.len() as u64) {
        violations.push(violation(
            contract_id,
            test_id,
            "e2e summary suite_count must match suite_ids length",
            Some(summary_rel.to_string()),
        ));
    }
    if summary.get("scenario_count").and_then(|v| v.as_u64()) != Some(scenario_ids.len() as u64) {
        violations.push(violation(
            contract_id,
            test_id,
            "e2e summary scenario_count must match scenario_ids length",
            Some(summary_rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_e2e_007_coverage_matrix_deterministic(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-E2E-007";
    let test_id = "ops.e2e.coverage_matrix_deterministic";
    let matrix_rel = "ops/e2e/generated/coverage-matrix.json";
    let scenarios_rel = "ops/e2e/scenarios/scenarios.json";
    let Some(matrix) = read_json(&ctx.repo_root.join(matrix_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "coverage-matrix.json must be valid json",
            Some(matrix_rel.to_string()),
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
    let mut violations = Vec::new();
    if matrix.get("schema_version").and_then(|v| v.as_i64()) != Some(1) {
        violations.push(violation(
            contract_id,
            test_id,
            "coverage matrix schema_version must be 1",
            Some(matrix_rel.to_string()),
        ));
    }
    if matrix
        .get("missing_domains")
        .and_then(|v| v.as_array())
        .is_some_and(|v| !v.is_empty())
    {
        violations.push(violation(
            contract_id,
            test_id,
            "coverage matrix missing_domains must be empty",
            Some(matrix_rel.to_string()),
        ));
    }
    let mut scenario_ids_from_rows = Vec::new();
    for row in matrix
        .get("rows")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        let Some(scenario_id) = row.get("scenario_id").and_then(|v| v.as_str()) else {
            continue;
        };
        scenario_ids_from_rows.push(scenario_id.to_string());
        let covers: Vec<String> = row
            .get("covers")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        let mut covers_sorted = covers.clone();
        covers_sorted.sort();
        covers_sorted.dedup();
        if covers != covers_sorted {
            violations.push(violation(
                contract_id,
                test_id,
                "coverage matrix covers entries must be unique and sorted",
                Some(matrix_rel.to_string()),
            ));
        }
    }
    let mut sorted_rows = scenario_ids_from_rows.clone();
    sorted_rows.sort();
    sorted_rows.dedup();
    if scenario_ids_from_rows != sorted_rows {
        violations.push(violation(
            contract_id,
            test_id,
            "coverage matrix rows must be unique and sorted by scenario_id",
            Some(matrix_rel.to_string()),
        ));
    }
    let scenario_ids: BTreeSet<String> = scenarios
        .get("scenarios")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.get("id").and_then(|id| id.as_str()).map(|s| s.to_string()))
        .collect();
    let covered_ids: BTreeSet<String> = scenario_ids_from_rows.into_iter().collect();
    if scenario_ids != covered_ids {
        violations.push(violation(
            contract_id,
            test_id,
            "coverage matrix rows must match scenarios registry ids",
            Some(matrix_rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_e2e_008_realdata_registry_and_snapshots_valid(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-E2E-008";
    let test_id = "ops.e2e.realdata_registry_and_snapshots_valid";
    let scenarios_rel = "ops/e2e/realdata/scenarios.json";
    let snapshots_rel = "ops/e2e/realdata/snapshots";
    let Some(scenarios) = read_json(&ctx.repo_root.join(scenarios_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "realdata scenarios.json must be valid json",
            Some(scenarios_rel.to_string()),
        )]);
    };
    let mut files = Vec::new();
    walk_files(&ctx.repo_root.join(snapshots_rel), &mut files);
    files.sort();
    let mut violations = Vec::new();
    if scenarios.get("schema_version").and_then(|v| v.as_i64()) != Some(1) {
        violations.push(violation(
            contract_id,
            test_id,
            "realdata scenarios schema_version must be 1",
            Some(scenarios_rel.to_string()),
        ));
    }
    let scenario_rows = scenarios
        .get("scenarios")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if scenario_rows.is_empty() {
        violations.push(violation(
            contract_id,
            test_id,
            "realdata scenarios registry must include at least one scenario",
            Some(scenarios_rel.to_string()),
        ));
    }
    for row in scenario_rows {
        let id = row.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        let runner = row.get("runner").and_then(|v| v.as_str()).unwrap_or_default();
        let action_id = row.get("action_id").and_then(|v| v.as_str()).unwrap_or_default();
        if id.is_empty() || runner.is_empty() || action_id.is_empty() {
            violations.push(violation(
                contract_id,
                test_id,
                "realdata scenario rows must include id runner and action_id",
                Some(scenarios_rel.to_string()),
            ));
            continue;
        }
        if !runner.ends_with(".py")
            || !runner.starts_with("crates/bijux-dev-atlas/src/bijux-dev-atlas/commands/ops/e2e/")
        {
            violations.push(violation(
                contract_id,
                test_id,
                "realdata scenario runner must use canonical ops/e2e python path",
                Some(scenarios_rel.to_string()),
            ));
        }
    }
    if files.is_empty() {
        violations.push(violation(
            contract_id,
            test_id,
            "realdata snapshots directory must include at least one snapshot",
            Some(snapshots_rel.to_string()),
        ));
    }
    for path in files {
        if !path
            .extension()
            .and_then(|v| v.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        {
            continue;
        }
        let rel = rel_to_root(&path, &ctx.repo_root);
        let Some(snapshot) = read_json(&path) else {
            violations.push(violation(
                contract_id,
                test_id,
                "realdata snapshot must be valid json",
                Some(rel),
            ));
            continue;
        };
        if snapshot.get("entries").and_then(|v| v.as_array()).is_none() {
            violations.push(violation(
                contract_id,
                test_id,
                "realdata snapshot must include entries array",
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

fn test_ops_e2e_009_no_stray_e2e_artifacts(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-E2E-009";
    let test_id = "ops.e2e.no_stray_e2e_artifacts";
    let e2e_root = ctx.repo_root.join("ops/e2e");
    let Ok(entries) = fs::read_dir(&e2e_root) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "ops/e2e directory must exist",
            Some("ops/e2e".to_string()),
        )]);
    };
    let allowed_entries = BTreeSet::from([
        "CONTRACT.md",
        "README.md",
        "datasets",
        "expectations",
        "fixtures",
        "generated",
        "manifests",
        "realdata",
        "reproducibility-policy.json",
        "scenarios",
        "smoke",
        "suites",
        "taxonomy.json",
    ]);
    let mut violations = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if !allowed_entries.contains(name) {
            violations.push(violation(
                contract_id,
                test_id,
                "ops/e2e contains artifact outside declared surface",
                Some(rel_to_root(&path, &ctx.repo_root)),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_e2e_010_summary_schema_valid(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-E2E-010";
    let test_id = "ops.e2e.summary_schema_valid";
    let summary_rel = "ops/e2e/generated/e2e-summary.json";
    let suites_rel = "ops/e2e/suites/suites.json";
    let scenarios_rel = "ops/e2e/scenarios/scenarios.json";
    let expectations_rel = "ops/e2e/expectations/expectations.json";
    let Some(summary) = read_json(&ctx.repo_root.join(summary_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "e2e summary must be valid json",
            Some(summary_rel.to_string()),
        )]);
    };
    let Some(suites) = read_json(&ctx.repo_root.join(suites_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "e2e suites registry must be valid json",
            Some(suites_rel.to_string()),
        )]);
    };
    let Some(scenarios) = read_json(&ctx.repo_root.join(scenarios_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "e2e scenarios registry must be valid json",
            Some(scenarios_rel.to_string()),
        )]);
    };
    let Some(expectations) = read_json(&ctx.repo_root.join(expectations_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "e2e expectations registry must be valid json",
            Some(expectations_rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    if summary.get("schema_version").and_then(|v| v.as_i64()) != Some(1) {
        violations.push(violation(
            contract_id,
            test_id,
            "e2e summary schema_version must be 1",
            Some(summary_rel.to_string()),
        ));
    }
    if summary.get("status").and_then(|v| v.as_str()) != Some("stable") {
        violations.push(violation(
            contract_id,
            test_id,
            "e2e summary status must be stable",
            Some(summary_rel.to_string()),
        ));
    }
    let suite_ids: Vec<String> = summary
        .get("suite_ids")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    let scenario_ids: Vec<String> = summary
        .get("scenario_ids")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    let mut sorted_suite_ids = suite_ids.clone();
    sorted_suite_ids.sort();
    if suite_ids != sorted_suite_ids {
        violations.push(violation(
            contract_id,
            test_id,
            "e2e summary suite_ids must be sorted",
            Some(summary_rel.to_string()),
        ));
    }
    let mut sorted_scenario_ids = scenario_ids.clone();
    sorted_scenario_ids.sort();
    if scenario_ids != sorted_scenario_ids {
        violations.push(violation(
            contract_id,
            test_id,
            "e2e summary scenario_ids must be sorted",
            Some(summary_rel.to_string()),
        ));
    }
    if summary.get("suite_count").and_then(|v| v.as_u64()) != Some(suite_ids.len() as u64) {
        violations.push(violation(
            contract_id,
            test_id,
            "e2e summary suite_count must equal suite_ids length",
            Some(summary_rel.to_string()),
        ));
    }
    if summary.get("scenario_count").and_then(|v| v.as_u64()) != Some(scenario_ids.len() as u64) {
        violations.push(violation(
            contract_id,
            test_id,
            "e2e summary scenario_count must equal scenario_ids length",
            Some(summary_rel.to_string()),
        ));
    }
    let suites_declared: BTreeSet<String> = suites
        .get("suites")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.get("id").and_then(|id| id.as_str()).map(|s| s.to_string()))
        .collect();
    let scenarios_declared: BTreeSet<String> = scenarios
        .get("scenarios")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.get("id").and_then(|id| id.as_str()).map(|s| s.to_string()))
        .collect();
    if suites_declared != suite_ids.iter().cloned().collect() {
        violations.push(violation(
            contract_id,
            test_id,
            "e2e summary suite_ids must match suites registry",
            Some(summary_rel.to_string()),
        ));
    }
    if scenarios_declared != scenario_ids.iter().cloned().collect() {
        violations.push(violation(
            contract_id,
            test_id,
            "e2e summary scenario_ids must match scenarios registry",
            Some(summary_rel.to_string()),
        ));
    }
    let expectation_ids: BTreeSet<String> = expectations
        .get("expectations")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| {
            v.get("scenario_id")
                .and_then(|id| id.as_str())
                .map(|s| s.to_string())
        })
        .collect();
    if expectation_ids != scenarios_declared {
        violations.push(violation(
            contract_id,
            test_id,
            "e2e expectations must cover every declared public scenario exactly once",
            Some(expectations_rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

