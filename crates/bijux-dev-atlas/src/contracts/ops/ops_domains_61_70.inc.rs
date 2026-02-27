// SPDX-License-Identifier: Apache-2.0

fn effect_violation(contract_id: &str, test_id: &str, message: &str, file: &str) -> Violation {
    violation(contract_id, test_id, message, Some(file.to_string()))
}

fn test_ops_stack_e_001_kind_cluster_up_profile_dev(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-STACK-E-001";
    let test_id = "ops.stack.effect.kind_cluster_up_profile_dev";
    let profile = ctx.repo_root.join("ops/stack/kind/cluster-dev.yaml");
    if !profile.exists() {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "kind dev profile config must exist for effect execution",
            "ops/stack/kind/cluster-dev.yaml",
        )]);
    }
    TestResult::Pass
}

fn test_ops_stack_e_002_core_components_present(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-STACK-E-002";
    let test_id = "ops.stack.effect.core_components_present";
    let required = [
        "ops/stack/minio/minio.yaml",
        "ops/stack/redis/redis.yaml",
        "ops/stack/prometheus/prometheus.yaml",
        "ops/stack/otel/otel-collector.yaml",
        "ops/stack/grafana/grafana.yaml",
    ];
    let mut violations = Vec::new();
    for rel in required {
        if !ctx.repo_root.join(rel).exists() {
            violations.push(effect_violation(
                contract_id,
                test_id,
                "effect component manifest is missing",
                rel,
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_stack_e_003_ports_inventory_mapped(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-STACK-E-003";
    let test_id = "ops.stack.effect.ports_inventory_mapped";
    let sample_rel = "ops/stack/tests/goldens/stack-ports-inventory.sample.json";
    let Some(sample) = read_json(&ctx.repo_root.join(sample_rel)) else {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "stack ports inventory sample must be parseable",
            sample_rel,
        )]);
    };
    let has_endpoints = sample
        .as_object()
        .is_some_and(|obj| !obj.is_empty() && obj.values().all(|v| v.as_str().is_some()));
    if !has_endpoints {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "stack ports inventory sample must map service names to endpoint strings",
            sample_rel,
        )]);
    }
    TestResult::Pass
}

fn test_ops_stack_e_004_stack_health_report_generated(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-STACK-E-004";
    let test_id = "ops.stack.effect.health_report_generated";
    let sample_rel = "ops/stack/tests/goldens/stack-health-report.sample.json";
    let Some(sample) = read_json(&ctx.repo_root.join(sample_rel)) else {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "stack health report sample must be parseable",
            sample_rel,
        )]);
    };
    if sample
        .get("checks")
        .and_then(|v| v.as_object())
        .is_none_or(|checks| checks.is_empty())
    {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "stack health report sample must include non-empty checks object",
            sample_rel,
        )]);
    }
    TestResult::Pass
}

fn test_ops_k8s_e_001_helm_install_contract_defined(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-K8S-E-001";
    let test_id = "ops.k8s.effect.helm_install_contract_defined";
    let matrix_rel = "ops/k8s/install-matrix.json";
    let Some(matrix) = read_json(&ctx.repo_root.join(matrix_rel)) else {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "install-matrix must be parseable for effect execution",
            matrix_rel,
        )]);
    };
    let has_kind = matrix
        .get("profiles")
        .and_then(|v| v.as_array())
        .is_some_and(|rows| {
            rows.iter()
                .filter_map(|v| v.get("name").and_then(|n| n.as_str()))
                .any(|name| name == "kind")
        });
    if !has_kind {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "k8s install-matrix must include kind profile",
            matrix_rel,
        )]);
    }
    TestResult::Pass
}

fn test_ops_k8s_e_002_rollout_safety_contract_satisfied(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-K8S-E-002";
    let test_id = "ops.k8s.effect.rollout_safety_contract_satisfied";
    let rel = "ops/k8s/rollout-safety-contract.json";
    let Some(contract) = read_json(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "rollout safety contract must be parseable",
            rel,
        )]);
    };
    if contract
        .get("profiles")
        .and_then(|v| v.as_array())
        .is_none_or(|rows| rows.is_empty())
    {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "rollout safety contract must include non-empty profiles array",
            rel,
        )]);
    }
    TestResult::Pass
}

fn test_ops_k8s_e_003_service_endpoints_reachable_contract(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-K8S-E-003";
    let test_id = "ops.k8s.effect.service_endpoints_reachable_contract";
    let rel = "ops/k8s/tests/suites.json";
    let Some(suites) = read_json(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "k8s suites contract must be parseable",
            rel,
        )]);
    };
    if suites
        .get("suites")
        .and_then(|v| v.as_array())
        .is_none_or(|rows| rows.is_empty())
    {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "k8s suites contract must include non-empty suites array",
            rel,
        )]);
    }
    TestResult::Pass
}

fn test_ops_obs_e_001_scrape_metrics_contract(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-OBS-E-001";
    let test_id = "ops.observe.effect.scrape_metrics_contract";
    let rel = "ops/observe/contracts/metrics.golden.prom";
    let Ok(text) = std::fs::read_to_string(ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "metrics golden contract file must be readable",
            rel,
        )]);
    };
    if text.trim().is_empty() {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "metrics golden contract must not be empty",
            rel,
        )]);
    }
    TestResult::Pass
}

fn test_ops_obs_e_002_trace_structure_contract(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-OBS-E-002";
    let test_id = "ops.observe.effect.trace_structure_contract";
    let rel = "ops/observe/contracts/trace-structure.golden.json";
    let Some(value) = read_json(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "trace structure golden must be parseable",
            rel,
        )]);
    };
    if !value.is_object() {
        return TestResult::Fail(vec![effect_violation(
            contract_id,
            test_id,
            "trace structure golden must be a json object",
            rel,
        )]);
    }
    TestResult::Pass
}

fn test_ops_obs_e_003_alerts_load_contract(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-OBS-E-003";
    let test_id = "ops.observe.effect.alerts_load_contract";
    let files = [
        "ops/observe/alerts/atlas-alert-rules.yaml",
        "ops/observe/alerts/slo-burn-rules.yaml",
    ];
    let mut violations = Vec::new();
    for rel in files {
        if read_yaml_value(&ctx.repo_root.join(rel)).is_none() {
            violations.push(effect_violation(
                contract_id,
                test_id,
                "alert rule yaml must be parseable for effect checks",
                rel,
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}
