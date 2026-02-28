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

