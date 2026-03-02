// SPDX-License-Identifier: Apache-2.0

fn test_ops_stack_005_kind_profiles_consistent(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-STACK-005";
    let test_id = "ops.stack.kind_profiles_consistent";
    let profiles_rel = "ops/stack/profiles.json";
    let Some(profiles) = read_json(&ctx.repo_root.join(profiles_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "stack profiles manifest must be parseable",
            Some(profiles_rel.to_string()),
        )]);
    };
    let Some(rows) = profiles.get("profiles").and_then(|v| v.as_array()) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "stack profiles manifest must include profiles array",
            Some(profiles_rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    let mut required = BTreeSet::from([
        "ops/stack/kind/cluster-dev.yaml".to_string(),
        "ops/stack/kind/cluster-perf.yaml".to_string(),
        "ops/stack/kind/cluster-small.yaml".to_string(),
    ]);
    for item in rows {
        let cluster = item
            .get("cluster_config")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if cluster.is_empty() {
            violations.push(violation(
                contract_id,
                test_id,
                "profile entry must include cluster_config",
                Some(profiles_rel.to_string()),
            ));
            continue;
        }
        if !ctx.repo_root.join(&cluster).exists() {
            violations.push(violation(
                contract_id,
                test_id,
                "cluster_config file referenced by profile must exist",
                Some(cluster.clone()),
            ));
            continue;
        }
        required.remove(&cluster);
    }
    for cluster in &required {
        violations.push(violation(
            contract_id,
            test_id,
            "dev, perf, and small kind cluster configs must be referenced by profiles",
            Some(cluster.to_string()),
        ));
    }
    for rel in [
        "ops/stack/kind/cluster-dev.yaml",
        "ops/stack/kind/cluster-perf.yaml",
        "ops/stack/kind/cluster-small.yaml",
    ] {
        let Some(doc) = read_yaml_value(&ctx.repo_root.join(rel)) else {
            violations.push(violation(
                contract_id,
                test_id,
                "kind cluster config must be parseable yaml",
                Some(rel.to_string()),
            ));
            continue;
        };
        let has_kind = doc.get("kind").and_then(|v| v.as_str()) == Some("Cluster");
        let has_api = doc.get("apiVersion").and_then(|v| v.as_str()).is_some();
        let has_nodes = doc
            .get("nodes")
            .and_then(|v| v.as_sequence())
            .is_some_and(|nodes| !nodes.is_empty());
        if !has_kind || !has_api || !has_nodes {
            violations.push(violation(
                contract_id,
                test_id,
                "kind cluster config must include kind/apiVersion/nodes",
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

fn test_ops_stack_006_ports_inventory_matches_stack(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-STACK-006";
    let test_id = "ops.stack.ports_inventory_matches_stack";
    let sample_rel = "ops/stack/tests/goldens/stack-ports-inventory.sample.json";
    let Some(sample) = read_json(&ctx.repo_root.join(sample_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "stack ports inventory sample must be parseable",
            Some(sample_rel.to_string()),
        )]);
    };
    let Some(ports) = sample.as_object() else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "stack ports inventory sample must be a json object",
            Some(sample_rel.to_string()),
        )]);
    };
    let graph_rel = "ops/stack/generated/dependency-graph.json";
    let Some(graph) = read_json(&ctx.repo_root.join(graph_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "stack dependency graph must be parseable",
            Some(graph_rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    let mut seen_endpoints = BTreeSet::new();
    for (service, endpoint) in ports {
        let Some(endpoint) = endpoint.as_str() else {
            violations.push(violation(
                contract_id,
                test_id,
                "ports inventory entries must map to endpoint strings",
                Some(sample_rel.to_string()),
            ));
            continue;
        };
        if !endpoint.starts_with("http://127.0.0.1:") {
            violations.push(violation(
                contract_id,
                test_id,
                "ports inventory endpoint must use loopback host and explicit port",
                Some(sample_rel.to_string()),
            ));
            continue;
        }
        if !seen_endpoints.insert(endpoint.to_string()) {
            violations.push(violation(
                contract_id,
                test_id,
                "ports inventory must not contain duplicate endpoints",
                Some(sample_rel.to_string()),
            ));
        }
        if service.trim().is_empty() {
            violations.push(violation(
                contract_id,
                test_id,
                "ports inventory service key must not be empty",
                Some(sample_rel.to_string()),
            ));
        }
    }
    let component_text = graph
        .get("profiles")
        .and_then(|v| v.as_object())
        .map(|profiles| format!("{profiles:?}"))
        .unwrap_or_default();
    for service in ["atlas", "prometheus"] {
        if !ports.contains_key(service) {
            violations.push(violation(
                contract_id,
                test_id,
                "ports inventory must include required stack services",
                Some(sample_rel.to_string()),
            ));
        }
    }
    if !component_text.contains("grafana.yaml") && ports.contains_key("grafana") {
        violations.push(violation(
            contract_id,
            test_id,
            "grafana endpoint cannot be declared when grafana component is absent from dependency graph",
            Some(graph_rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_stack_007_health_report_generator_contract(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-STACK-007";
    let test_id = "ops.stack.health_report_generator_contract";
    let sample_rel = "ops/stack/tests/goldens/stack-health-report.sample.json";
    let Some(sample) = read_json(&ctx.repo_root.join(sample_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "stack health report sample must be parseable",
            Some(sample_rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    if sample
        .get("schema_version")
        .and_then(|v| v.as_i64())
        .is_none_or(|version| version < 1)
    {
        violations.push(violation(
            contract_id,
            test_id,
            "stack health report sample must include schema_version >= 1",
            Some(sample_rel.to_string()),
        ));
    }
    if sample
        .get("checks")
        .and_then(|v| v.as_object())
        .is_none_or(|checks| checks.is_empty())
    {
        violations.push(violation(
            contract_id,
            test_id,
            "stack health report sample must include non-empty checks object",
            Some(sample_rel.to_string()),
        ));
    }
    let index_rel = "ops/stack/generated/stack-index.json";
    let Some(index) = read_json(&ctx.repo_root.join(index_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "stack index must be parseable",
            Some(index_rel.to_string()),
        )]);
    };
    if index
        .get("generated_by")
        .and_then(|v| v.as_str())
        .is_none_or(|v| !v.contains("ops generate"))
    {
        violations.push(violation(
            contract_id,
            test_id,
            "stack index must declare ops generator provenance",
            Some(index_rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}
