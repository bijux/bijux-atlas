fn test_ops_stack_001_stack_toml_parseable_complete(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-STACK-001";
    let test_id = "ops.stack.stack_toml_parseable_complete";
    let rel = "ops/stack/stack.toml";
    let Ok(raw) = std::fs::read_to_string(ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "stack.toml must exist and be readable",
            Some(rel.to_string()),
        )]);
    };
    let parsed: toml::Value = match toml::from_str(&raw) {
        Ok(value) => value,
        Err(_) => {
            return TestResult::Fail(vec![violation(
                contract_id,
                test_id,
                "stack.toml must be valid toml",
                Some(rel.to_string()),
            )]);
        }
    };
    let Some(profiles) = parsed.get("profiles").and_then(|v| v.as_table()) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "stack.toml must define profiles table",
            Some(rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    for name in ["ci", "kind", "local"] {
        if !profiles.contains_key(name) {
            violations.push(violation(
                contract_id,
                test_id,
                "stack.toml must include ci/kind/local profiles",
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

fn test_ops_stack_002_service_dependency_contract_valid(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-STACK-002";
    let test_id = "ops.stack.service_dependency_contract_valid";
    let rel = "ops/stack/service-dependency-contract.json";
    let Some(contract) = read_json(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "service dependency contract must be valid json",
            Some(rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    let Some(services) = contract.get("services").and_then(|v| v.as_array()) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "service dependency contract must include services array",
            Some(rel.to_string()),
        )]);
    };
    for item in services {
        let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let component = item.get("component").and_then(|v| v.as_str()).unwrap_or("");
        if id.is_empty() || component.is_empty() {
            violations.push(violation(
                contract_id,
                test_id,
                "service entry requires id and component",
                Some(rel.to_string()),
            ));
        }
        if !component.is_empty() && !ctx.repo_root.join(component).exists() {
            violations.push(violation(
                contract_id,
                test_id,
                "service component path must exist",
                Some(component.to_string()),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_stack_003_versions_manifest_schema_valid(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-STACK-003";
    let test_id = "ops.stack.versions_manifest_schema_valid";
    let rel = "ops/stack/generated/version-manifest.json";
    let Some(versions) = read_json(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "version-manifest must be valid json",
            Some(rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    if versions.get("schema_version").and_then(|v| v.as_i64()).is_none() {
        violations.push(violation(
            contract_id,
            test_id,
            "version-manifest must include schema_version",
            Some(rel.to_string()),
        ));
    }
    for key in [
        "kind_node_image",
        "minio",
        "prometheus",
        "otel_collector",
        "redis",
    ] {
        let value = versions.get(key).and_then(|v| v.as_str()).unwrap_or("");
        if value.is_empty() || !value.contains("@sha256:") {
            violations.push(violation(
                contract_id,
                test_id,
                "version-manifest images must be digest-pinned",
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

fn test_ops_stack_004_dependency_graph_generated_acyclic(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-STACK-004";
    let test_id = "ops.stack.dependency_graph_generated_acyclic";
    let rel = "ops/stack/generated/dependency-graph.json";
    let Some(graph) = read_json(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "dependency-graph must be valid json",
            Some(rel.to_string()),
        )]);
    };
    let Some(profiles) = graph.get("profiles").and_then(|v| v.as_object()) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "dependency-graph must define profiles object",
            Some(rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    let mut cluster_to_profiles: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (profile, value) in profiles {
        let cluster = value
            .get("cluster_config")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if cluster.is_empty() || !ctx.repo_root.join(cluster).exists() {
            violations.push(violation(
                contract_id,
                test_id,
                "profile cluster_config must exist",
                Some(rel.to_string()),
            ));
            continue;
        }
        cluster_to_profiles
            .entry(cluster.to_string())
            .or_default()
            .insert(profile.to_string());
        if let Some(components) = value.get("components").and_then(|v| v.as_array()) {
            for component in components {
                let component = component.as_str().unwrap_or("");
                if component.is_empty() || !ctx.repo_root.join(component).exists() {
                    violations.push(violation(
                        contract_id,
                        test_id,
                        "dependency-graph component path must exist",
                        Some(rel.to_string()),
                    ));
                }
            }
        }
    }
    if cluster_to_profiles.values().all(|set| set.len() <= 1) {
        violations.push(violation(
            contract_id,
            test_id,
            "dependency graph should share at least one cluster_config across profiles",
            Some(rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}
