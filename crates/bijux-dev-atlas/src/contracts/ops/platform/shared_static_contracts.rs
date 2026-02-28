// SPDX-License-Identifier: Apache-2.0

fn test_ops_k8s_002_values_files_validate_schema(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-K8S-002";
    let test_id = "ops.k8s.values_files_validate_schema";
    let schema_rel = "ops/k8s/charts/bijux-atlas/values.schema.json";
    let Some(schema) = read_json(&ctx.repo_root.join(schema_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "values.schema.json must be valid json",
            Some(schema_rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    if !schema.get("properties").is_some_and(|v| v.is_object()) {
        violations.push(violation(
            contract_id,
            test_id,
            "values.schema.json must include properties object",
            Some(schema_rel.to_string()),
        ));
    }

    let matrix_rel = "ops/k8s/install-matrix.json";
    let Some(matrix) = read_json(&ctx.repo_root.join(matrix_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "install-matrix must be valid json",
            Some(matrix_rel.to_string()),
        )]);
    };
    if let Some(items) = matrix.get("profiles").and_then(|v| v.as_array()) {
        for item in items {
            let values_file = item
                .get("values_file")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if values_file.is_empty() {
                continue;
            }
            let path = ctx.repo_root.join(values_file);
            if !path.exists() {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "install-matrix values_file must exist",
                    Some(values_file.to_string()),
                ));
                continue;
            }
            if read_yaml_value(&path).is_none() {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "values_file must be parseable yaml",
                    Some(values_file.to_string()),
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

fn test_ops_k8s_003_install_matrix_complete(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-K8S-003";
    let test_id = "ops.k8s.install_matrix_complete";
    let matrix_rel = "ops/k8s/install-matrix.json";
    let Some(matrix) = read_json(&ctx.repo_root.join(matrix_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "install-matrix must be valid json",
            Some(matrix_rel.to_string()),
        )]);
    };
    let expected: BTreeSet<String> = [
        "ci",
        "dev",
        "ingress",
        "kind",
        "local",
        "multi-registry",
        "offline",
        "perf",
        "prod",
    ]
    .into_iter()
    .map(std::string::ToString::to_string)
    .collect();
    let mut seen = BTreeSet::new();
    let mut violations = Vec::new();
    if let Some(items) = matrix.get("profiles").and_then(|v| v.as_array()) {
        for item in items {
            let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let values_file = item
                .get("values_file")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if name.is_empty() || !seen.insert(name.to_string()) {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "profile names must be non-empty and unique",
                    Some(matrix_rel.to_string()),
                ));
            }
            if !values_file.is_empty() && !ctx.repo_root.join(values_file).exists() {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "install-matrix profile references missing values file",
                    Some(values_file.to_string()),
                ));
            }
        }
    }
    if seen != expected {
        violations.push(violation(
            contract_id,
            test_id,
            "install-matrix must include the canonical profile set",
            Some(matrix_rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_k8s_004_no_forbidden_k8s_objects(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-K8S-004";
    let test_id = "ops.k8s.no_forbidden_k8s_objects";
    let mut files = Vec::new();
    walk_files(&ctx.repo_root.join("ops/k8s/charts/bijux-atlas/templates"), &mut files);
    files.sort();
    let forbidden = ["kind: ClusterRole", "kind: ClusterRoleBinding", "kind: PodSecurityPolicy"];
    let mut violations = Vec::new();
    for path in files {
        let rel = rel_to_root(&path, &ctx.repo_root);
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        for token in forbidden {
            if text.contains(token) {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "forbidden cluster-scope object detected in chart template",
                    Some(rel.clone()),
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

fn test_ops_load_001_scenarios_schema_valid(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-LOAD-001";
    let test_id = "ops.load.scenarios_schema_valid";
    let mut files = Vec::new();
    walk_files(&ctx.repo_root.join("ops/load/scenarios"), &mut files);
    files.sort();
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
                "scenario file must be valid json",
                Some(rel),
            ));
            continue;
        };
        let Some(obj) = value.as_object() else {
            violations.push(violation(
                contract_id,
                test_id,
                "scenario file must be a json object",
                Some(rel),
            ));
            continue;
        };
        for key in ["name", "suite"] {
            if obj
                .get(key)
                .and_then(|v| v.as_str())
                .is_none_or(|v| v.is_empty())
            {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "scenario file missing required non-empty key",
                    Some(rel.clone()),
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

fn test_ops_load_002_thresholds_exist_for_each_suite(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-LOAD-002";
    let test_id = "ops.load.thresholds_exist_for_each_suite";
    let suites_rel = "ops/load/suites/suites.json";
    let Some(suites) = read_json(&ctx.repo_root.join(suites_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "load suites file must be valid json",
            Some(suites_rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    if let Some(items) = suites.get("suites").and_then(|v| v.as_array()) {
        for suite in items {
            let name = suite.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if name.is_empty() {
                continue;
            }
            let rel = format!("ops/load/thresholds/{name}.thresholds.json");
            if !ctx.repo_root.join(&rel).exists() {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "suite must have corresponding thresholds file",
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

fn test_ops_load_003_pinned_queries_lock_consistent(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-LOAD-003";
    let test_id = "ops.load.pinned_queries_lock_consistent";
    let source_rel = "ops/load/queries/pinned-v1.json";
    let lock_rel = "ops/load/queries/pinned-v1.lock";
    let source_path = ctx.repo_root.join(source_rel);
    let lock_path = ctx.repo_root.join(lock_rel);
    let Some(lock) = read_json(&lock_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "pinned query lock must be valid json",
            Some(lock_rel.to_string()),
        )]);
    };
    let Some(source) = read_json(&source_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "pinned query source must be valid json",
            Some(source_rel.to_string()),
        )]);
    };
    let expected_file_sha = file_sha256(&source_path).unwrap_or_default();
    let actual_file_sha = lock
        .get("file_sha256")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let mut violations = Vec::new();
    if expected_file_sha != actual_file_sha {
        violations.push(violation(
            contract_id,
            test_id,
            "lock file_sha256 must match pinned-v1.json digest",
            Some(lock_rel.to_string()),
        ));
    }

    let mut expected_hashes = BTreeMap::new();
    if let Some(queries) = source.get("queries").and_then(|v| v.as_array()) {
        for query in queries {
            if let Some(q) = query.as_str() {
                expected_hashes.insert(q.to_string(), sha256_text(q));
            }
        }
    }
    if let Some(actual_hashes) = lock.get("query_hashes").and_then(|v| v.as_object()) {
        for (query, digest) in expected_hashes {
            let got = actual_hashes.get(&query).and_then(|v| v.as_str()).unwrap_or("");
            if got != digest {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "query hash mismatch in lock file",
                    Some(lock_rel.to_string()),
                ));
            }
        }
    } else {
        violations.push(violation(
            contract_id,
            test_id,
            "pinned query lock must include query_hashes object",
            Some(lock_rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_load_004_baselines_schema_valid(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-LOAD-004";
    let test_id = "ops.load.baselines_schema_valid";
    let mut files = Vec::new();
    walk_files(&ctx.repo_root.join("ops/load/baselines"), &mut files);
    files.sort();
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
                "baseline file must be valid json",
                Some(rel),
            ));
            continue;
        };
        for key in ["name", "source", "metadata", "rows"] {
            if value.get(key).is_none() {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "baseline file missing required key",
                    Some(rel.clone()),
                ));
            }
        }
        if value
            .get("rows")
            .and_then(|v| v.as_array())
            .is_none_or(|rows| rows.is_empty())
        {
            violations.push(violation(
                contract_id,
                test_id,
                "baseline rows must be non-empty",
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

fn test_ops_load_005_no_scenario_without_slo_mapping(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-LOAD-005";
    let test_id = "ops.load.no_scenario_without_slo_mapping";
    let suites_rel = "ops/load/suites/suites.json";
    let map_rel = "ops/inventory/scenario-slo-map.json";
    let Some(suites) = read_json(&ctx.repo_root.join(suites_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "load suites file must be valid json",
            Some(suites_rel.to_string()),
        )]);
    };
    let Some(mappings) = read_json(&ctx.repo_root.join(map_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "scenario-slo-map must be valid json",
            Some(map_rel.to_string()),
        )]);
    };
    let mut mapped = BTreeSet::new();
    if let Some(items) = mappings.get("mappings").and_then(|v| v.as_array()) {
        for item in items {
            if let Some(load_suites) = item.get("load_suites").and_then(|v| v.as_array()) {
                for suite in load_suites {
                    if let Some(name) = suite.as_str() {
                        mapped.insert(name.to_string());
                    }
                }
            }
        }
    }
    let mut violations = Vec::new();
    if let Some(items) = suites.get("suites").and_then(|v| v.as_array()) {
        for suite in items {
            let name = suite.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if name.is_empty() {
                continue;
            }
            let run_in_smoke_or_pr = suite
                .get("run_in")
                .and_then(|v| v.as_array())
                .is_some_and(|runs| {
                    runs.iter()
                        .filter_map(|v| v.as_str())
                        .any(|run| run == "smoke" || run == "pr")
                });
            if run_in_smoke_or_pr && !mapped.contains(name) {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "load suite in smoke/pr lane must map to SLO entry",
                    Some(suites_rel.to_string()),
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

fn test_ops_obs_001_alert_rules_exist_parseable(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-OBS-001";
    let test_id = "ops.observe.alert_rules_exist_parseable";
    let required = [
        "ops/observe/alerts/atlas-alert-rules.yaml",
        "ops/observe/alerts/slo-burn-rules.yaml",
        "ops/observe/alert-catalog.json",
    ];
    let mut violations = Vec::new();
    for rel in required {
        let path = ctx.repo_root.join(rel);
        if !path.exists() {
            violations.push(violation(
                contract_id,
                test_id,
                "required observability alert file is missing",
                Some(rel.to_string()),
            ));
            continue;
        }
        if rel.ends_with(".json") && read_json(&path).is_none() {
            violations.push(violation(
                contract_id,
                test_id,
                "alert catalog must be valid json",
                Some(rel.to_string()),
            ));
        }
        if rel.ends_with(".yaml") && read_yaml_value(&path).is_none() {
            violations.push(violation(
                contract_id,
                test_id,
                "alert rules file must be parseable yaml",
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

fn test_ops_obs_002_dashboard_json_parseable_golden_diff(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-OBS-002";
    let test_id = "ops.observe.dashboard_json_parseable_golden_diff";
    let live_rel = "ops/observe/dashboards/atlas-observability-dashboard.json";
    let golden_rel = "ops/observe/dashboards/atlas-observability-dashboard.golden.json";
    let Some(live) = read_json(&ctx.repo_root.join(live_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "live dashboard json must parse",
            Some(live_rel.to_string()),
        )]);
    };
    let Some(golden) = read_json(&ctx.repo_root.join(golden_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "golden dashboard json must parse",
            Some(golden_rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    let live_title = live.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let golden_title = golden.get("title").and_then(|v| v.as_str()).unwrap_or("");
    if live_title != golden_title {
        violations.push(violation(
            contract_id,
            test_id,
            "dashboard title must match golden",
            Some(live_rel.to_string()),
        ));
    }
    let live_uid = live.get("uid").and_then(|v| v.as_str()).unwrap_or("");
    let golden_uid = golden.get("uid").and_then(|v| v.as_str()).unwrap_or("");
    if live_uid != golden_uid {
        violations.push(violation(
            contract_id,
            test_id,
            "dashboard uid must match golden",
            Some(live_rel.to_string()),
        ));
    }
    let live_panels = live
        .get("panels")
        .and_then(|v| v.as_array())
        .map_or(0, |v| v.len());
    let golden_panels = golden
        .get("panels")
        .and_then(|v| v.as_array())
        .map_or(0, |v| v.len());
    if live_panels != golden_panels {
        violations.push(violation(
            contract_id,
            test_id,
            "dashboard panel count must match golden",
            Some(live_rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}
