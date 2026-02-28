fn test_ops_rpt_001_report_schema_ssot(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-REPORT-001";
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
    let contract_id = "OPS-REPORT-002";
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
    let contract_id = "OPS-REPORT-003";
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
    let contract_id = "OPS-REPORT-004";
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

fn test_ops_rpt_005_readiness_score_deterministic(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-REPORT-005";
    let test_id = "ops.report.readiness_score_deterministic";
    let rel = "ops/report/generated/readiness-score.json";
    let Some(report) = read_json(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "readiness score report must be valid json",
            Some(rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    if report.get("schema_version").and_then(|v| v.as_i64()) != Some(1) {
        violations.push(violation(
            contract_id,
            test_id,
            "readiness score report must include schema_version=1",
            Some(rel.to_string()),
        ));
    }
    if report.get("status").and_then(|v| v.as_str()) != Some("ready") {
        violations.push(violation(
            contract_id,
            test_id,
            "readiness score report status must be ready",
            Some(rel.to_string()),
        ));
    }
    let Some(inputs) = report.get("inputs").and_then(|v| v.as_object()) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "readiness score report must include inputs object",
            Some(rel.to_string()),
        )]);
    };
    let expected = BTreeSet::from(["inventory_drift", "ops_doctor", "ops_validate", "schema_drift"]);
    let keys = inputs.keys().map(String::as_str).collect::<BTreeSet<_>>();
    if keys != expected {
        violations.push(violation(
            contract_id,
            test_id,
            "readiness score inputs must use canonical key set",
            Some(rel.to_string()),
        ));
    }
    if report.get("generated_by").and_then(|v| v.as_str()).is_none() {
        violations.push(violation(
            contract_id,
            test_id,
            "readiness score report must include generated_by",
            Some(rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_rpt_006_release_evidence_bundle_schema_valid(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-REPORT-006";
    let test_id = "ops.report.release_evidence_bundle_schema_valid";
    let rel = "ops/report/generated/release-evidence-bundle.json";
    let Some(bundle) = read_json(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "release evidence bundle must be valid json",
            Some(rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    if bundle.get("schema_version").and_then(|v| v.as_i64()) != Some(1) {
        violations.push(violation(
            contract_id,
            test_id,
            "release evidence bundle must include schema_version=1",
            Some(rel.to_string()),
        ));
    }
    if bundle.get("release").and_then(|v| v.as_str()).is_none() {
        violations.push(violation(
            contract_id,
            test_id,
            "release evidence bundle must include release string",
            Some(rel.to_string()),
        ));
    }
    let Some(paths) = bundle.get("bundle_paths").and_then(|v| v.as_array()) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "release evidence bundle must include bundle_paths array",
            Some(rel.to_string()),
        )]);
    };
    for item in paths {
        let Some(path) = item.as_str() else {
            violations.push(violation(
                contract_id,
                test_id,
                "bundle_paths entries must be strings",
                Some(rel.to_string()),
            ));
            continue;
        };
        if !ctx.repo_root.join(path).exists() {
            violations.push(violation(
                contract_id,
                test_id,
                "bundle path must exist",
                Some(path.to_string()),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_rpt_007_historical_comparison_schema_valid(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-REPORT-007";
    let test_id = "ops.report.historical_comparison_schema_valid";
    let rel = "ops/report/generated/historical-comparison.json";
    let Some(history) = read_json(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "historical comparison report must be valid json",
            Some(rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    if history.get("schema_version").and_then(|v| v.as_i64()) != Some(1) {
        violations.push(violation(
            contract_id,
            test_id,
            "historical comparison must include schema_version=1",
            Some(rel.to_string()),
        ));
    }
    if history.get("window").and_then(|v| v.as_str()).is_none() {
        violations.push(violation(
            contract_id,
            test_id,
            "historical comparison must include window",
            Some(rel.to_string()),
        ));
    }
    if history.get("trend").and_then(|v| v.as_str()).is_none() {
        violations.push(violation(
            contract_id,
            test_id,
            "historical comparison must include trend",
            Some(rel.to_string()),
        ));
    }
    if history
        .get("readiness_scores")
        .and_then(|v| v.as_array())
        .is_none_or(|scores| scores.is_empty())
    {
        violations.push(violation(
            contract_id,
            test_id,
            "historical comparison must include non-empty readiness_scores",
            Some(rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_rpt_008_unified_report_example_schema_valid(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-REPORT-008";
    let test_id = "ops.report.unified_report_example_schema_valid";
    let rel = "ops/report/examples/unified-report-example.json";
    let Some(unified) = read_json(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "unified report example must be valid json",
            Some(rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    if unified.get("schema_version").and_then(|v| v.as_i64()) != Some(1) {
        violations.push(violation(
            contract_id,
            test_id,
            "unified report example must include schema_version=1",
            Some(rel.to_string()),
        ));
    }
    if unified.get("report_version").and_then(|v| v.as_i64()).is_none() {
        violations.push(violation(
            contract_id,
            test_id,
            "unified report example must include report_version",
            Some(rel.to_string()),
        ));
    }
    if unified.get("lanes").and_then(|v| v.as_object()).is_none() {
        violations.push(violation(
            contract_id,
            test_id,
            "unified report example must include lanes object",
            Some(rel.to_string()),
        ));
    }
    if unified.get("summary").and_then(|v| v.as_object()).is_none() {
        violations.push(violation(
            contract_id,
            test_id,
            "unified report example must include summary object",
            Some(rel.to_string()),
        ));
    }
    if unified.get("budget_status").and_then(|v| v.as_object()).is_none() {
        violations.push(violation(
            contract_id,
            test_id,
            "unified report example must include budget_status object",
            Some(rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_rpt_009_report_outputs_canonical_json(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-REPORT-009";
    let test_id = "ops.report.outputs_canonical_json";
    let mut files = Vec::new();
    walk_files(&ctx.repo_root.join("ops/report/generated"), &mut files);
    walk_files(&ctx.repo_root.join("ops/report/examples"), &mut files);
    files.push(ctx.repo_root.join("ops/report/evidence-levels.json"));
    files.sort();
    files.dedup();
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
        if !path.exists() {
            continue;
        }
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
                "report json must be parseable",
                Some(rel),
            ));
            continue;
        };
        let expected = match serde_json::to_string_pretty(&sort_json(&parsed)) {
            Ok(value) => format!("{value}\n"),
            Err(_) => continue,
        };
        if raw != expected {
            violations.push(violation(
                contract_id,
                test_id,
                "report json must use canonical pretty formatting with trailing newline",
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

fn test_ops_rpt_010_lane_reports_aggregated_in_unified_report(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-REPORT-010";
    let test_id = "ops.report.lane_reports_aggregated_in_unified_report";
    let rel = "ops/report/examples/unified-report-example.json";
    let Some(unified) = read_json(&ctx.repo_root.join(rel)) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "unified report example must be valid json",
            Some(rel.to_string()),
        )]);
    };
    let Some(lanes) = unified.get("lanes").and_then(|v| v.as_object()) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "unified report must include lanes object",
            Some(rel.to_string()),
        )]);
    };
    let mut violations = Vec::new();
    if lanes.is_empty() {
        violations.push(violation(
            contract_id,
            test_id,
            "unified report must include at least one lane",
            Some(rel.to_string()),
        ));
    }
    let mut pass = 0usize;
    let mut fail = 0usize;
    for (lane_id, lane) in lanes {
        let status = lane.get("status").and_then(|v| v.as_str()).unwrap_or("");
        if status == "pass" {
            pass += 1;
        } else if status == "fail" {
            fail += 1;
        } else {
            violations.push(violation(
                contract_id,
                test_id,
                "lane status must be pass or fail",
                Some(format!("{rel}::{lane_id}")),
            ));
        }
        if lane.get("log").and_then(|v| v.as_str()).is_none() {
            violations.push(violation(
                contract_id,
                test_id,
                "each lane must include log artifact path",
                Some(format!("{rel}::{lane_id}")),
            ));
        }
    }
    if let Some(summary) = unified.get("summary").and_then(|v| v.as_object()) {
        let total = summary.get("total").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let passed = summary.get("passed").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let failed = summary.get("failed").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        if total != lanes.len() || passed != pass || failed != fail {
            violations.push(violation(
                contract_id,
                test_id,
                "summary totals must be derived from lane statuses",
                Some(rel.to_string()),
            ));
        }
    } else {
        violations.push(violation(
            contract_id,
            test_id,
            "unified report must include summary object",
            Some(rel.to_string()),
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

