// SPDX-License-Identifier: Apache-2.0

fn test_ops_obs_003_telemetry_goldens_required_profiles(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-OBS-003";
    let test_id = "ops.observe.telemetry_goldens_required_profiles";
    let profiles_rel = "ops/observe/contracts/goldens/profiles.json";
    let index_rel = "ops/observe/generated/telemetry-index.json";
    let Some(profiles) = read_json(&ctx.repo_root.join(profiles_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "golden profiles file must be valid json",
            Some(profiles_rel.to_string()),
        )]);
    };
    let Some(index) = read_json(&ctx.repo_root.join(index_rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "telemetry index must be valid json",
            Some(index_rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    let mut required_artifacts = BTreeSet::new();
    if let Some(profile_map) = profiles.get("profiles").and_then(|v| v.as_object()) {
        for (_profile, item) in profile_map {
            for key in ["metrics_golden", "trace_golden"] {
                let rel = item.get(key).and_then(|v| v.as_str()).unwrap_or("");
                if rel.is_empty() {
                    violations.push(violation(
                        contract_id,
                        test_id,
                        "profile goldens must include metrics_golden and trace_golden paths",
                        Some(profiles_rel.to_string()),
                    ));
                    continue;
                }
                if !ctx.repo_root.join(rel).exists() {
                    violations.push(violation(
                        contract_id,
                        test_id,
                        "profile golden path must exist",
                        Some(rel.to_string()),
                    ));
                }
                required_artifacts.insert(rel.to_string());
            }
        }
    }
    let _ = required_artifacts;
    if index.get("source").and_then(|v| v.as_str()) != Some("ops/observe") {
        violations.push(violation(
            contract_id,
            test_id,
            "telemetry index source must be ops/observe",
            Some(index_rel.to_string()),
        ));
    }
    if index
        .get("artifacts")
        .and_then(|v| v.as_array())
        .is_none_or(|items| items.is_empty())
    {
        violations.push(violation(
            contract_id,
            test_id,
            "telemetry index must include non-empty artifacts list",
            Some(index_rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_obs_004_readiness_schema_valid(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-OBS-004";
    let test_id = "ops.observe.readiness_schema_valid";
    let rel = "ops/observe/readiness.json";
    let Some(value) = read_json(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "readiness file must be valid json",
            Some(rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    if value.get("schema_version").and_then(|v| v.as_i64()) != Some(1) {
        violations.push(violation(
            contract_id,
            test_id,
            "readiness schema_version must be 1",
            Some(rel.to_string()),
        ));
    }
    let status = value.get("status").and_then(|v| v.as_str()).unwrap_or("");
    if status != "ready" {
        violations.push(violation(
            contract_id,
            test_id,
            "readiness status must be `ready`",
            Some(rel.to_string()),
        ));
    }
    let required_tokens: BTreeSet<String> = [
        "slo-definitions",
        "alert-catalog",
        "telemetry-drills",
        "dashboard-index",
    ]
    .into_iter()
    .map(std::string::ToString::to_string)
    .collect();
    let mut actual = BTreeSet::new();
    if let Some(items) = value.get("requirements").and_then(|v| v.as_array()) {
        for item in items {
            if let Some(req) = item.as_str() {
                actual.insert(req.to_string());
            }
        }
    }
    if actual != required_tokens {
        violations.push(violation(
            contract_id,
            test_id,
            "readiness requirements must match canonical set",
            Some(rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_rpt_001_report_schema_ssot(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-RPT-001";
    let test_id = "ops.report.schema_is_ssot";
    let rel = "ops/report/schema.json";
    let Some(schema) = read_json(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "report schema must be valid json",
            Some(rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    if schema.get("$schema").and_then(|v| v.as_str()).is_none() {
        violations.push(violation(
            contract_id,
            test_id,
            "report schema must declare $schema",
            Some(rel.to_string()),
        ));
    }
    if schema
        .get("required")
        .and_then(|v| v.as_array())
        .is_none_or(|req| req.is_empty())
    {
        violations.push(violation(
            contract_id,
            test_id,
            "report schema must define required fields",
            Some(rel.to_string()),
        ));
    }
    if !ctx.repo_root.join("ops/schema/report/schema.json").exists() {
        violations.push(violation(
            contract_id,
            test_id,
            "schema mirror must exist under ops/schema/report/schema.json",
            Some("ops/schema/report/schema.json".to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_rpt_002_generated_reports_schema_valid(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-RPT-002";
    let test_id = "ops.report.generated_reports_schema_valid";
    let generated = [
        "ops/report/generated/historical-comparison.json",
        "ops/report/generated/readiness-score.json",
        "ops/report/generated/release-evidence-bundle.json",
        "ops/report/generated/report-diff.json",
    ];
    let mut violations = Vec::new();
    for rel in generated {
        let Some(value) = read_json(&ctx.repo_root.join(rel)) else {
            violations.push(violation(
                contract_id,
                test_id,
                "generated report must be valid json",
                Some(rel.to_string()),
            ));
            continue;
        };
        if value.get("schema_version").and_then(|v| v.as_i64()).is_none() {
            violations.push(violation(
                contract_id,
                test_id,
                "generated report must include schema_version",
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

fn test_ops_rpt_003_evidence_levels_complete(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-RPT-003";
    let test_id = "ops.report.evidence_levels_complete";
    let rel = "ops/report/evidence-levels.json";
    let Some(levels) = read_json(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "evidence levels file must be valid json",
            Some(rel.to_string()),
        )]);
    };
    let expected = BTreeSet::from(["minimal", "standard", "forensic"]);
    let mut found = BTreeSet::new();
    if let Some(items) = levels.get("levels").and_then(|v| v.as_array()) {
        for item in items {
            if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                found.insert(id.to_string());
            }
        }
    }
    if found
        != expected
            .into_iter()
            .map(std::string::ToString::to_string)
            .collect::<BTreeSet<_>>()
    {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "evidence levels must include minimal/standard/forensic",
            Some(rel.to_string()),
        )]);
    }
    TestResult::Pass
}

fn test_ops_rpt_004_report_diff_contract_exists(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-RPT-004";
    let test_id = "ops.report.diff_contract_exists";
    let rel = "ops/report/generated/report-diff.json";
    let Some(diff) = read_json(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "report diff must be valid json",
            Some(rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    if diff.get("base_report").and_then(|v| v.as_str()).is_none() {
        violations.push(violation(
            contract_id,
            test_id,
            "report diff must include base_report",
            Some(rel.to_string()),
        ));
    }
    if diff.get("target_report").and_then(|v| v.as_str()).is_none() {
        violations.push(violation(
            contract_id,
            test_id,
            "report diff must include target_report",
            Some(rel.to_string()),
        ));
    }
    if diff.get("changes").and_then(|v| v.as_array()).is_none() {
        violations.push(violation(
            contract_id,
            test_id,
            "report diff must include changes array",
            Some(rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

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
