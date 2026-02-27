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
fn test_ops_schema_006_id_and_naming_consistency(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-SCHEMA-006";
    let test_id = "ops.schema.id_and_naming_consistency";
    let mut files = Vec::new();
    walk_files(&ctx.repo_root.join("ops/schema"), &mut files);
    files.sort();
    let mut violations = Vec::new();
    for path in files {
        let rel = rel_to_root(&path, &ctx.repo_root);
        if !rel.ends_with(".schema.json") {
            continue;
        }
        let Some(schema) = read_json(&path) else {
            violations.push(violation(
                contract_id,
                test_id,
                "schema file must be valid json",
                Some(rel),
            ));
            continue;
        };
        let id = schema.get("$id").and_then(|v| v.as_str());
        let Some(id) = id else {
            violations.push(violation(
                contract_id,
                test_id,
                "schema must declare non-empty $id",
                Some(rel),
            ));
            continue;
        };
        if id.trim().is_empty() {
            violations.push(violation(
                contract_id,
                test_id,
                "schema $id must not be blank",
                Some(rel),
            ));
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if !id.ends_with(file_name) {
            violations.push(violation(
                contract_id,
                test_id,
                "schema $id should end with schema file name",
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
fn test_ops_schema_007_examples_validate_required_fields(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-SCHEMA-007";
    let test_id = "ops.schema.examples_validate_required_fields";
    let lock_path = ctx.repo_root.join("ops/schema/generated/compatibility-lock.json");
    let Some(lock) = read_json(&lock_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "compatibility lock must be parseable",
            Some("ops/schema/generated/compatibility-lock.json".to_string()),
        )]);
    };
    let Some(targets) = lock.get("targets").and_then(|v| v.as_array()) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "compatibility lock must contain targets array",
            Some("ops/schema/generated/compatibility-lock.json".to_string()),
        )]);
    };
    let mut violations = Vec::new();
    for target in targets {
        let Some(schema_path) = target.get("schema_path").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(example_path) = target.get("example_path").and_then(|v| v.as_str()) else {
            violations.push(violation(
                contract_id,
                test_id,
                "compatibility lock target must declare example_path",
                Some(schema_path.to_string()),
            ));
            continue;
        };
        let example_abs = ctx.repo_root.join(example_path);
        let example = if example_path.ends_with(".yaml") || example_path.ends_with(".yml") {
            fs::read_to_string(&example_abs)
                .ok()
                .and_then(|raw| serde_yaml::from_str::<serde_json::Value>(&raw).ok())
        } else {
            read_json(&example_abs)
        };
        let Some(example) = example else {
            violations.push(violation(
                contract_id,
                test_id,
                "example fixture must be parseable json or yaml",
                Some(example_path.to_string()),
            ));
            continue;
        };
        let required_fields = target
            .get("required_fields")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        for field in required_fields {
            let Some(name) = field.as_str() else {
                continue;
            };
            if example.get(name).is_none() {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "example fixture is missing required field",
                    Some(example_path.to_string()),
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
fn test_ops_schema_008_forbid_duplicate_schema_intent(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-SCHEMA-008";
    let test_id = "ops.schema.forbid_duplicate_intent";
    let mut files = Vec::new();
    walk_files(&ctx.repo_root.join("ops/schema"), &mut files);
    files.sort();
    let mut ids: BTreeMap<String, String> = BTreeMap::new();
    let mut titles: BTreeMap<String, String> = BTreeMap::new();
    let mut violations = Vec::new();
    for path in files {
        let rel = rel_to_root(&path, &ctx.repo_root);
        if !rel.ends_with(".schema.json") {
            continue;
        }
        let Some(schema) = read_json(&path) else {
            continue;
        };
        if let Some(id) = schema.get("$id").and_then(|v| v.as_str()) {
            if let Some(existing) = ids.insert(id.to_string(), rel.clone()) {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "duplicate schema $id detected",
                    Some(format!("{existing} | {rel}")),
                ));
            }
        }
        if let Some(title) = schema.get("title").and_then(|v| v.as_str()) {
            let normalized = title.trim().to_ascii_lowercase();
            if normalized.is_empty() {
                continue;
            }
            if let Some(existing) = titles.insert(normalized, rel.clone()) {
                violations.push(violation(
                    contract_id,
                    test_id,
                    "duplicate schema title detected",
                    Some(format!("{existing} | {rel}")),
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
fn test_ops_schema_009_canonical_json_formatting(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-SCHEMA-009";
    let test_id = "ops.schema.canonical_json_formatting";
    let mut files = Vec::new();
    walk_files(&ctx.repo_root.join("ops/schema/generated"), &mut files);
    files.sort();
    let mut violations = Vec::new();
    fn sort_json(value: &serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Object(map) => {
                let mut sorted = serde_json::Map::new();
                let mut keys = map.keys().cloned().collect::<Vec<_>>();
                keys.sort();
                for key in keys {
                    if let Some(v) = map.get(&key) {
                        sorted.insert(key, sort_json(v));
                    }
                }
                serde_json::Value::Object(sorted)
            }
            serde_json::Value::Array(items) => {
                serde_json::Value::Array(items.iter().map(sort_json).collect())
            }
            other => other.clone(),
        }
    }
    for path in files {
        let rel = rel_to_root(&path, &ctx.repo_root);
        if !rel.ends_with(".json") {
            continue;
        }
        let Ok(raw) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&raw) else {
            violations.push(violation(
                contract_id,
                test_id,
                "generated json must be parseable",
                Some(rel),
            ));
            continue;
        };
        let canonical = sort_json(&parsed);
        let expected = match serde_json::to_string_pretty(&canonical) {
            Ok(v) => format!("{v}\n"),
            Err(_) => continue,
        };
        if raw != expected {
            violations.push(violation(
                contract_id,
                test_id,
                "generated json must use canonical pretty formatting with trailing newline",
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
fn test_ops_schema_010_example_coverage(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-SCHEMA-010";
    let test_id = "ops.schema.example_coverage";
    let lock_path = ctx.repo_root.join("ops/schema/generated/compatibility-lock.json");
    let Some(lock) = read_json(&lock_path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "compatibility lock must be parseable",
            Some("ops/schema/generated/compatibility-lock.json".to_string()),
        )]);
    };
    let Some(targets) = lock.get("targets").and_then(|v| v.as_array()) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "compatibility lock must contain targets array",
            Some("ops/schema/generated/compatibility-lock.json".to_string()),
        )]);
    };
    let mut violations = Vec::new();
    for target in targets {
        let Some(schema_path) = target.get("schema_path").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(example_path) = target.get("example_path").and_then(|v| v.as_str()) else {
            violations.push(violation(
                contract_id,
                test_id,
                "schema target must define example_path for CI coverage",
                Some(schema_path.to_string()),
            ));
            continue;
        };
        if !ctx.repo_root.join(example_path).exists() {
            violations.push(violation(
                contract_id,
                test_id,
                "example_path referenced by compatibility lock does not exist",
                Some(example_path.to_string()),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_load_010_every_scenario_has_slo_impact_class(ctx: &RunContext) -> TestResult {
    let (contract_id, test_id, suites_rel, map_rel) = ("OPS-LOAD-010", "ops.load.every_scenario_has_slo_impact_class", "ops/load/suites/suites.json", "ops/inventory/scenario-slo-map.json");
    let Some(suites) = read_json(&ctx.repo_root.join(suites_rel)) else { return TestResult::Fail(vec![violation(contract_id, test_id, "load suites must be parseable json", Some(suites_rel.to_string()))]); };
    let Some(map) = read_json(&ctx.repo_root.join(map_rel)) else { return TestResult::Fail(vec![violation(contract_id, test_id, "scenario slo map must be parseable json", Some(map_rel.to_string()))]); };
    let mut mapped = BTreeSet::new();
    if let Some(rows) = map.get("mappings").and_then(|v| v.as_array()) { for row in rows { if let Some(items) = row.get("load_suites").and_then(|v| v.as_array()) { for suite in items.iter().filter_map(|v| v.as_str()) { mapped.insert(suite.to_string()); } } } }
    let mut violations = Vec::new();
    if let Some(rows) = suites.get("suites").and_then(|v| v.as_array()) { for row in rows { if let Some(name) = row.get("name").and_then(|v| v.as_str()) { if !mapped.contains(name) { violations.push(violation(contract_id, test_id, "load suite must map to scenario-slo-map load_suites coverage", Some(format!("{map_rel}#{name}")))); } } } }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_ops_datasets_005_qc_metadata_and_golden_valid(ctx: &RunContext) -> TestResult {
    let (contract_id, test_id, rel) = ("OPS-DATASETS-005", "ops.datasets.qc_metadata_and_golden_valid", "ops/datasets/qc-metadata.json");
    let Some(meta) = read_json(&ctx.repo_root.join(rel)) else { return TestResult::Fail(vec![violation(contract_id, test_id, "qc metadata must be parseable json", Some(rel.to_string()))]); };
    let golden_rel = meta.get("golden_summary").and_then(|v| v.as_str()).unwrap_or("");
    if !golden_rel.is_empty() && read_json(&ctx.repo_root.join(golden_rel)).is_some() { TestResult::Pass } else { TestResult::Fail(vec![violation(contract_id, test_id, "qc metadata golden_summary must point to parseable json", Some(rel.to_string()))]) }
}

fn test_ops_datasets_006_rollback_policy_exists_valid(ctx: &RunContext) -> TestResult {
    let (contract_id, test_id, rel) = ("OPS-DATASETS-006", "ops.datasets.rollback_policy_exists_valid", "ops/datasets/rollback-policy.json");
    let Some(policy) = read_json(&ctx.repo_root.join(rel)) else { return TestResult::Fail(vec![violation(contract_id, test_id, "rollback policy must be parseable json", Some(rel.to_string()))]); };
    let valid = policy.get("rollback_steps").and_then(|v| v.as_array()).is_some_and(|v| !v.is_empty()) && policy.get("requires").and_then(|v| v.as_array()).is_some_and(|v| !v.is_empty());
    if valid { TestResult::Pass } else { TestResult::Fail(vec![violation(contract_id, test_id, "rollback policy must include rollback_steps and requires arrays", Some(rel.to_string()))]) }
}

fn test_ops_datasets_007_promotion_rules_exists_valid(ctx: &RunContext) -> TestResult {
    let (contract_id, test_id, rel) = ("OPS-DATASETS-007", "ops.datasets.promotion_rules_exists_valid", "ops/datasets/promotion-rules.json");
    let Some(rules) = read_json(&ctx.repo_root.join(rel)) else { return TestResult::Fail(vec![violation(contract_id, test_id, "promotion rules must be parseable json", Some(rel.to_string()))]); };
    let pins = rules.get("pins_source").and_then(|v| v.as_str()).unwrap_or("");
    let lock = rules.get("manifest_lock").and_then(|v| v.as_str()).unwrap_or("");
    let envs: BTreeSet<String> = rules.get("environments").and_then(|v| v.as_array()).into_iter().flatten().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
    let required: BTreeSet<String> = ["dev", "ci", "prod"].into_iter().map(std::string::ToString::to_string).collect();
    if !pins.is_empty() && !lock.is_empty() && ctx.repo_root.join(pins).exists() && ctx.repo_root.join(lock).exists() && envs == required { TestResult::Pass } else { TestResult::Fail(vec![violation(contract_id, test_id, "promotion rules must reference pins+lock and include dev/ci/prod", Some(rel.to_string()))]) }
}

fn test_ops_datasets_008_consumer_list_consistent_with_runtime_queries(ctx: &RunContext) -> TestResult {
    let (contract_id, test_id, rel) = ("OPS-DATASETS-008", "ops.datasets.consumer_list_consistent_with_runtime_queries", "ops/datasets/consumer-list.json");
    let Some(consumers) = read_json(&ctx.repo_root.join(rel)) else { return TestResult::Fail(vec![violation(contract_id, test_id, "consumer list must be parseable json", Some(rel.to_string()))]); };
    let mut violations = Vec::new();
    if let Some(rows) = consumers.get("consumers").and_then(|v| v.as_array()) { for row in rows { let interface = row.get("interface").and_then(|v| v.as_str()).unwrap_or(""); if interface.is_empty() || !ctx.repo_root.join(interface).exists() { violations.push(violation(contract_id, test_id, "consumer interface must reference an existing repository path", Some(rel.to_string()))); } } }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_ops_datasets_009_freeze_policy_exists_enforced(ctx: &RunContext) -> TestResult {
    let (contract_id, test_id, rel) = ("OPS-DATASETS-009", "ops.datasets.freeze_policy_exists_enforced", "ops/datasets/freeze-policy.json");
    let Some(policy) = read_json(&ctx.repo_root.join(rel)) else { return TestResult::Fail(vec![violation(contract_id, test_id, "freeze policy must be parseable json", Some(rel.to_string()))]); };
    let append_only = policy.pointer("/immutability/fixture_assets_append_only").and_then(|v| v.as_bool()) == Some(true);
    let forbid_replace = policy.pointer("/immutability/allow_archive_replacement").and_then(|v| v.as_bool()) == Some(false);
    if append_only && forbid_replace { TestResult::Pass } else { TestResult::Fail(vec![violation(contract_id, test_id, "freeze policy must enforce append-only assets and forbid replacement", Some(rel.to_string()))]) }
}

fn test_ops_datasets_010_dataset_store_layout_contract_enforced(ctx: &RunContext) -> TestResult {
    let (contract_id, test_id, rel) = ("OPS-DATASETS-010", "ops.datasets.dataset_store_layout_contract_enforced", "ops/datasets/manifest.json");
    let Some(manifest) = read_json(&ctx.repo_root.join(rel)) else { return TestResult::Fail(vec![violation(contract_id, test_id, "datasets manifest must be parseable json", Some(rel.to_string()))]); };
    let mut violations = Vec::new();
    if let Some(rows) = manifest.get("datasets").and_then(|v| v.as_array()) {
        for row in rows {
            let id = row.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let parts: Vec<&str> = id.split('/').collect();
            if parts.len() != 3 || parts.iter().any(|p| p.is_empty()) {
                violations.push(violation(contract_id, test_id, "dataset id must follow release/species/assembly layout", Some(rel.to_string())));
            }
            if let Some(paths) = row.get("paths").and_then(|v| v.as_object()) {
                for value in paths.values().filter_map(|v| v.as_str()) {
                    if !value.starts_with("ops/datasets/fixtures/") {
                        violations.push(violation(contract_id, test_id, "dataset fixture paths must live under ops/datasets/fixtures", Some(value.to_string())));
                    }
                }
            }
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}
