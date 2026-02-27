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
