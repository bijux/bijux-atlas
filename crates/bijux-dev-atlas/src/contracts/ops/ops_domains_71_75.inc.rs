// SPDX-License-Identifier: Apache-2.0

fn test_ops_load_e_001_k6_suite_executes_contract(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-LOAD-E-001";
    let test_id = "ops.load.effect.k6_suite_executes_contract";
    let rel = "ops/load/suites/suites.json";
    let Some(suites) = read_json(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "load suites contract must be parseable",
            rel,
        )]);
    };
    let has_k6 = suites
        .get("suites")
        .and_then(|v| v.as_array())
        .is_some_and(|rows| {
            rows.iter()
                .any(|row| row.get("kind").and_then(|v| v.as_str()) == Some("k6"))
        });
    if !has_k6 {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "load suites contract must define at least one k6 suite",
            rel,
        )]);
    }
    TestResult::Pass
}

fn test_ops_load_e_002_thresholds_enforced_report_emitted(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-LOAD-E-002";
    let test_id = "ops.load.effect.thresholds_enforced_report_emitted";
    let thresholds_rel = "ops/load/contracts/k6-thresholds.v1.json";
    let summary_rel = "ops/load/generated/load-summary.json";
    let Some(thresholds) = read_json(&ctx.repo_root.join(thresholds_rel)) else {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "k6 thresholds contract must be parseable",
            thresholds_rel,
        )]);
    };
    let Some(summary) = read_json(&ctx.repo_root.join(summary_rel)) else {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "load summary report must be parseable",
            summary_rel,
        )]);
    };
    let Some(threshold_rows) = thresholds.as_object() else {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "k6 thresholds contract must be a JSON object keyed by suite id",
            thresholds_rel,
        )]);
    };
    if threshold_rows.is_empty() {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "k6 thresholds contract must include at least one suite threshold entry",
            thresholds_rel,
        )]);
    }
    let invalid_row = threshold_rows
        .iter()
        .find(|(_, value)| !value.is_object())
        .map(|(suite, _)| suite.to_string());
    if let Some(suite) = invalid_row {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            &format!("k6 threshold entry `{suite}` must be an object"),
            thresholds_rel,
        )]);
    }
    if summary.get("schema_version").and_then(|v| v.as_i64()).is_none() {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "load summary report must include schema_version",
            summary_rel,
        )]);
    }
    TestResult::Pass
}

fn test_ops_load_e_003_baseline_comparison_produced(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-LOAD-E-003";
    let test_id = "ops.load.effect.baseline_comparison_produced";
    let drift_rel = "ops/load/generated/load-drift-report.json";
    let Some(drift) = read_json(&ctx.repo_root.join(drift_rel)) else {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "load drift report must be parseable",
            drift_rel,
        )]);
    };
    if drift.get("schema_version").and_then(|v| v.as_i64()).is_none() {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "load drift report must include schema_version",
            drift_rel,
        )]);
    }
    TestResult::Pass
}

fn test_ops_e2e_e_001_smoke_suite_passes_contract(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-E2E-E-001";
    let test_id = "ops.e2e.effect.smoke_suite_passes_contract";
    let rel = "ops/e2e/suites/suites.json";
    let Some(suites) = read_json(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "e2e suites contract must be parseable",
            rel,
        )]);
    };
    let has_smoke = suites
        .get("suites")
        .and_then(|v| v.as_array())
        .is_some_and(|rows| {
            rows.iter()
                .any(|row| row.get("id").and_then(|v| v.as_str()) == Some("smoke"))
        });
    if !has_smoke {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "e2e suites contract must include smoke suite",
            rel,
        )]);
    }
    TestResult::Pass
}

fn test_ops_e2e_e_002_realdata_scenario_passes_contract(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-E2E-E-002";
    let test_id = "ops.e2e.effect.realdata_scenario_passes_contract";
    let rel = "ops/e2e/realdata/scenarios.json";
    let Some(realdata) = read_json(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "realdata scenarios contract must be parseable",
            rel,
        )]);
    };
    if realdata
        .get("scenarios")
        .and_then(|v| v.as_array())
        .is_none_or(|rows| rows.is_empty())
    {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "realdata scenarios contract must include non-empty scenarios array",
            rel,
        )]);
    }
    TestResult::Pass
}
fn test_ops_k8s_005_rbac_minimalism(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-K8S-005";
    let test_id = "ops.k8s.rbac_minimalism";
    let templates_dir = ctx.repo_root.join("ops/k8s/charts/bijux-atlas/templates");
    let mut files = Vec::new();
    walk_files(&templates_dir, &mut files);
    files.sort();
    let mut violations = Vec::new();
    for path in files {
        let rel = rel_to_root(&path, &ctx.repo_root);
        if !rel.ends_with(".yaml") && !rel.ends_with(".tpl") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            violations.push(violation(
                contract_id,
                test_id,
                "k8s template must be readable",
                Some(rel),
            ));
            continue;
        };
        let lower = text.to_ascii_lowercase();
        for marker in [
            "cluster-admin",
            "clusterrolebinding",
            "verbs: [\"*\"]",
            "resources: [\"*\"]",
            "apigroups: [\"*\"]",
        ] {
            if lower.contains(marker) {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "rbac wildcard and cluster-admin privileges are forbidden",
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

fn test_ops_k8s_006_pod_security_and_probes(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-K8S-006";
    let test_id = "ops.k8s.pod_security_and_probes";
    let rel = "ops/k8s/charts/bijux-atlas/templates/deployment.yaml";
    let path = ctx.repo_root.join(rel);
    let Ok(text) = std::fs::read_to_string(path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "deployment template must be readable",
            Some(rel.to_string()),
        )]);
    };
    let lower = text.to_ascii_lowercase();
    let mut violations = Vec::new();
    for marker in ["readinessprobe:", "livenessprobe:", "securitycontext:"] {
        if !lower.contains(marker) {
            violations.push(violation(
                contract_id,
                test_id,
                "deployment template must include required security/probe markers",
                Some(rel.to_string()),
            ));
        }
    }
    let values_rel = "ops/k8s/charts/bijux-atlas/values.yaml";
    let Ok(values_text) = std::fs::read_to_string(ctx.repo_root.join(values_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "k8s values file must be readable",
            Some(values_rel.to_string()),
        )]);
    };
    let values_lower = values_text.to_ascii_lowercase();
    for marker in ["runasnonroot:", "readonlyrootfilesystem:", "drop:"] {
        if !values_lower.contains(marker) {
            violations.push(violation(
                contract_id,
                test_id,
                "k8s values must define baseline pod security settings",
                Some(values_rel.to_string()),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_k8s_007_rollout_safety_enforced(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-K8S-007";
    let test_id = "ops.k8s.rollout_safety_enforced";
    let rollout_rel = "ops/k8s/rollout-safety-contract.json";
    let Some(rollout) = read_json(&ctx.repo_root.join(rollout_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "rollout safety contract must be parseable",
            Some(rollout_rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    let Some(profiles) = rollout.get("profiles").and_then(|v| v.as_array()) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "rollout safety contract must include profiles array",
            Some(rollout_rel.to_string()),
        )]);
    };
    if profiles.is_empty() {
        violations.push(violation(
            contract_id,
            test_id,
            "rollout safety profiles must not be empty",
            Some(rollout_rel.to_string()),
        ));
    }
    for row in profiles {
        let mode = row
            .get("rollout_mode")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if mode != "deployment" && mode != "rollout" {
            violations.push(violation(
                contract_id,
                test_id,
                "rollout_mode must be deployment or rollout",
                Some(rollout_rel.to_string()),
            ));
        }
    }
    let template_rel = "ops/k8s/charts/bijux-atlas/templates/rollout.yaml";
    let Ok(text) = std::fs::read_to_string(ctx.repo_root.join(template_rel)) else {
        violations.push(violation(
            contract_id,
            test_id,
            "rollout template must exist and be readable",
            Some(template_rel.to_string()),
        ));
        return TestResult::Fail(violations);
    };
    if !text.contains("rollout.steps") {
        violations.push(violation(
            contract_id,
            test_id,
            "rollout template must consume rollout.steps to enforce safe rollout behavior",
            Some(template_rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_k8s_008_conformance_suite_runnable(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-K8S-008";
    let test_id = "ops.k8s.conformance_suite_runnable";
    let suites_rel = "ops/k8s/tests/suites.json";
    let Some(suites) = read_json(&ctx.repo_root.join(suites_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "k8s suites contract must be parseable",
            Some(suites_rel.to_string()),
        )]);
    };
    let has_smoke = suites
        .get("suites")
        .and_then(|v| v.as_array())
        .is_some_and(|rows| {
            rows.iter()
                .filter_map(|v| v.get("id").and_then(|id| id.as_str()))
                .any(|id| id == "smoke" || id == "full")
        });
    if !has_smoke {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "k8s suites must include smoke or full suite for conformance execution",
            Some(suites_rel.to_string()),
        )]);
    }
    let surface_rel = "ops/_generated/control-plane-surface-list.json";
    let Some(surface) = read_json(&ctx.repo_root.join(surface_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "control-plane surface snapshot must be parseable",
            Some(surface_rel.to_string()),
        )]);
    };
    let has_k8s_conformance = surface
        .get("ops_taxonomy")
        .and_then(|v| v.get("entries"))
        .and_then(|v| v.as_array())
        .is_some_and(|rows| {
            rows.iter().any(|row| {
                row.get("domain").and_then(|v| v.as_str()) == Some("k8s")
                    && row.get("verb").and_then(|v| v.as_str()) == Some("conformance")
            })
        });
    if has_k8s_conformance {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "control-plane command surface must include k8s conformance verb",
            Some(surface_rel.to_string()),
        )])
    }
}

fn test_ops_k8s_009_install_matrix_and_generated_consistency(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-K8S-009";
    let test_id = "ops.k8s.install_matrix_and_generated_consistency";
    let matrix_rel = "ops/k8s/install-matrix.json";
    let Some(matrix) = read_json(&ctx.repo_root.join(matrix_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "install-matrix must be parseable",
            Some(matrix_rel.to_string()),
        )]);
    };
    let snapshot_rel = "ops/k8s/generated/release-snapshot.json";
    let Some(snapshot) = read_json(&ctx.repo_root.join(snapshot_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "release snapshot must be parseable",
            Some(snapshot_rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    let matrix_profiles: BTreeSet<String> = matrix
        .get("profiles")
        .and_then(|v| v.as_array())
        .map(|rows| {
            rows.iter()
                .filter_map(|r| r.get("name").and_then(|v| v.as_str()))
                .map(std::string::ToString::to_string)
                .collect()
        })
        .unwrap_or_default();
    if matrix_profiles.is_empty() {
        violations.push(violation(
            contract_id,
            test_id,
            "install-matrix must include non-empty profiles",
            Some(matrix_rel.to_string()),
        ));
    }
    for profile in snapshot
        .get("install_profiles")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str())
    {
        if !matrix_profiles.contains(profile) {
            violations.push(violation(
                contract_id,
                test_id,
                "release snapshot install profile must exist in install-matrix",
                Some(snapshot_rel.to_string()),
            ));
        }
    }
    for rel in [
        "ops/k8s/generated/inventory-index.json",
        "ops/k8s/generated/release-snapshot.json",
        "ops/k8s/generated/render-artifact-index.json",
    ] {
        let Some(doc) = read_json(&ctx.repo_root.join(rel)) else {
            violations.push(violation(
                contract_id,
                test_id,
                "generated k8s artifact must be parseable json",
                Some(rel.to_string()),
            ));
            continue;
        };
        if doc.get("schema_version").and_then(|v| v.as_i64()).is_none() {
            violations.push(violation(
                contract_id,
                test_id,
                "generated k8s artifact must include schema_version",
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

fn test_ops_k8s_010_generated_indexes_deterministic_schema_valid(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-K8S-010";
    let test_id = "ops.k8s.generated_indexes_deterministic_schema_valid";
    let mut violations = Vec::new();
    for rel in [
        "ops/k8s/generated/inventory-index.json",
        "ops/k8s/generated/release-snapshot.json",
        "ops/k8s/generated/render-artifact-index.json",
    ] {
        let Some(value) = read_json(&ctx.repo_root.join(rel)) else {
            violations.push(violation(
                contract_id,
                test_id,
                "generated k8s index must be parseable json",
                Some(rel.to_string()),
            ));
            continue;
        };
        if value.get("schema_version").and_then(|v| v.as_i64()) != Some(1) {
            violations.push(violation(
                contract_id,
                test_id,
                "generated k8s index must include schema_version=1",
                Some(rel.to_string()),
            ));
        }
        if value
            .get("generated_by")
            .and_then(|v| v.as_str())
            .is_none_or(|v| v.is_empty())
        {
            violations.push(violation(
                contract_id,
                test_id,
                "generated k8s index must include generated_by marker",
                Some(rel.to_string()),
            ));
        }
    }
    let inventory_rel = "ops/k8s/generated/inventory-index.json";
    if let Some(inventory) = read_json(&ctx.repo_root.join(inventory_rel)) {
        if let Some(files) = inventory.get("files").and_then(|v| v.as_array()) {
            let names: Vec<String> = files
                .iter()
                .filter_map(|v| v.as_str())
                .map(std::string::ToString::to_string)
                .collect();
            let mut sorted = names.clone();
            sorted.sort();
            if names != sorted {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "inventory-index files must be lexicographically sorted for deterministic output",
                    Some(inventory_rel.to_string()),
                ));
            }
        }
    }
    let render_rel = "ops/k8s/generated/render-artifact-index.json";
    if let Some(render) = read_json(&ctx.repo_root.join(render_rel)) {
        let required_output = "artifacts/ops/<run-id>/render/k8s/rendered.yaml";
        let has_required_output = render
            .get("render_outputs")
            .and_then(|v| v.as_array())
            .is_some_and(|rows| rows.iter().any(|v| v.as_str() == Some(required_output)));
        if !has_required_output {
            violations.push(violation(
                contract_id,
                test_id,
                "render artifact index must declare canonical rendered manifest output path",
                Some(render_rel.to_string()),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
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
        if !id.contains('.') {
            violations.push(violation(
                contract_id,
                test_id,
                "alert catalog id must use dotted stable naming",
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
