fn test_ops_load_006_drift_report_generator_schema_valid(ctx: &RunContext) -> TestResult {
    let (contract_id, test_id, rel) = ("OPS-LOAD-006", "ops.load.drift_report_generator_schema_valid", "ops/load/generated/load-drift-report.json");
    let Some(doc) = read_json(&ctx.repo_root.join(rel)) else { return TestResult::Fail(vec![violation(contract_id, test_id, "load drift report must be parseable json", Some(rel.to_string()))]); };
    let valid = doc.get("schema_version").and_then(|v| v.as_i64()) == Some(1) && doc.get("status").and_then(|v| v.as_str()).is_some_and(|v| !v.is_empty()) && doc.get("checks").and_then(|v| v.as_array()).is_some_and(|v| !v.is_empty());
    if valid { TestResult::Pass } else { TestResult::Fail(vec![violation(contract_id, test_id, "load drift report must include schema_version status and checks", Some(rel.to_string()))]) }
}

fn test_ops_load_007_result_schema_validates_sample_output(ctx: &RunContext) -> TestResult {
    let (contract_id, test_id, schema_rel, sample_rel) = ("OPS-LOAD-007", "ops.load.result_schema_validates_sample_output", "ops/load/contracts/result-schema.json", "ops/load/generated/load-summary.json");
    let Some(schema) = read_json(&ctx.repo_root.join(schema_rel)) else { return TestResult::Fail(vec![violation(contract_id, test_id, "result schema must be parseable json", Some(schema_rel.to_string()))]); };
    let Some(sample) = read_json(&ctx.repo_root.join(sample_rel)) else { return TestResult::Fail(vec![violation(contract_id, test_id, "load summary sample must be parseable json", Some(sample_rel.to_string()))]); };
    let schema_ok = schema.get("properties").and_then(|v| v.get("metrics")).is_some() && schema.get("required").and_then(|v| v.as_array()).is_some_and(|r| r.iter().any(|v| v.as_str() == Some("metrics")));
    let sample_ok = sample.get("schema_version").and_then(|v| v.as_i64()).is_some() && sample.get("suites").and_then(|v| v.as_array()).is_some_and(|v| !v.is_empty());
    if schema_ok && sample_ok { TestResult::Pass } else { TestResult::Fail(vec![violation(contract_id, test_id, "result schema and load summary sample must expose required metrics envelope", Some(sample_rel.to_string()))]) }
}

fn test_ops_load_008_cheap_survival_in_minimal_gate_suite(ctx: &RunContext) -> TestResult { ops_load_suite_gate_membership(ctx, "OPS-LOAD-008", "ops.load.cheap_survival_in_minimal_gate_suite", "cheap-only-survival", &["smoke", "pr", "load-ci"]) }
fn test_ops_load_009_cold_start_p99_in_minimal_gate_suite(ctx: &RunContext) -> TestResult { ops_load_suite_gate_membership(ctx, "OPS-LOAD-009", "ops.load.cold_start_p99_in_minimal_gate_suite", "cold-start-p99", &["full", "nightly", "load-nightly"]) }

fn ops_load_suite_gate_membership(ctx: &RunContext, contract_id: &str, test_id: &str, suite_name: &str, required_lanes: &[&str]) -> TestResult {
    let rel = "ops/load/suites/suites.json";
    let Some(doc) = read_json(&ctx.repo_root.join(rel)) else { return TestResult::Fail(vec![violation(contract_id, test_id, "load suites registry must be parseable json", Some(rel.to_string()))]); };
    let Some(row) = doc.get("suites").and_then(|v| v.as_array()).and_then(|rows| rows.iter().find(|row| row.get("name").and_then(|v| v.as_str()) == Some(suite_name))) else { return TestResult::Fail(vec![violation(contract_id, test_id, "required load suite entry is missing", Some(rel.to_string()))]); };
    let run_in: BTreeSet<String> = row.get("run_in").and_then(|v| v.as_array()).map(|rows| rows.iter().filter_map(|v| v.as_str()).map(std::string::ToString::to_string).collect()).unwrap_or_default();
    if required_lanes.iter().all(|lane| run_in.contains(*lane)) { TestResult::Pass } else { TestResult::Fail(vec![violation(contract_id, test_id, "required load suite must participate in mandated gate lanes", Some(rel.to_string()))]) }
}
