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
