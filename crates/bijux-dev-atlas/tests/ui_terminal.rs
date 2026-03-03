// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use bijux_dev_atlas::model::{
    CheckId, CheckResult, CheckStatus, RunHistory, RunHistoryEntry, RunId, RunReport, RunSummary,
};
use bijux_dev_atlas::ui::terminal::report::render_check_run_report;
use bijux_dev_atlas::ui::terminal::{failure_hint, render_registry_status, render_suite_summary};

#[test]
fn suite_summary_renderer_includes_totals_failures_and_next_actions() {
    let summary = serde_json::json!({
      "suite": "checks",
      "run_id": "checks-001",
      "artifacts_root": "artifacts/run/checks-001",
      "summary": {
        "pass": 2,
        "fail": 1,
        "warn": 0,
        "skip": 1,
        "total": 4
      },
      "failures": [
        {"id": "OPS-001"}
      ]
    });
    let rendered = render_suite_summary(&summary);
    assert!(rendered.contains("pass=2 fail=1"));
    assert!(rendered.contains("Failed IDs: OPS-001"));
    assert!(rendered.contains("Artifacts: artifacts/run/checks-001"));
    assert!(rendered.contains("Next actions:"));
}

#[test]
fn registry_status_renderer_summarizes_readiness() {
    let report = serde_json::json!({
      "summary": {
        "errors": 1,
        "warnings": 2
      },
      "rows": [{}, {}]
    });
    let rendered = render_registry_status(&report);
    assert!(rendered.contains("rows=2"));
    assert!(rendered.contains("errors=1"));
    assert!(rendered.contains("resolve missing metadata"));
}

#[test]
fn run_history_model_filters_by_suite() {
    let history = RunHistory {
        entries: vec![
            RunHistoryEntry {
                suite: "checks".to_string(),
                run_id: "checks-001".to_string(),
                task_id: "OPS-001".to_string(),
                group: "ops".to_string(),
                mode: "pure".to_string(),
                status: "pass".to_string(),
                duration_ms: 5,
                timestamp: "2026-03-03T00:00:00Z".to_string(),
                result_path: "artifacts/run/checks-001/OPS-001/result.json".to_string(),
            },
            RunHistoryEntry {
                suite: "contracts".to_string(),
                run_id: "contracts-001".to_string(),
                task_id: "DOCS-001".to_string(),
                group: "docs".to_string(),
                mode: "effect".to_string(),
                status: "fail".to_string(),
                duration_ms: 7,
                timestamp: "2026-03-03T00:00:01Z".to_string(),
                result_path: "artifacts/run/contracts-001/DOCS-001/result.json".to_string(),
            },
        ],
    };
    let checks = history.by_suite("checks");
    assert_eq!(checks.len(), 1);
    assert_eq!(checks[0].task_id, "OPS-001");
}

#[test]
fn failure_hints_cover_common_codes() {
    assert!(failure_hint("missing_tool").contains("install the required tool"));
    assert!(failure_hint("unknown").contains("inspect the generated artifact"));
}

#[test]
fn check_run_renderer_uses_nextest_style_lines() {
    let report = RunReport {
        schema_version: 1,
        run_id: RunId::parse("checks_run").expect("run id"),
        repo_root: "/repo".to_string(),
        command: "bijux dev atlas check run".to_string(),
        selections: BTreeMap::new(),
        capabilities: BTreeMap::new(),
        results: vec![
            CheckResult {
                schema_version: 1,
                id: CheckId::parse("checks_docs_index_links").expect("check id"),
                status: CheckStatus::Pass,
                skip_reason: None,
                violations: Vec::new(),
                duration_ms: 16,
                evidence: Vec::new(),
            },
            CheckResult {
                schema_version: 1,
                id: CheckId::parse("checks_make_wrapper_commands").expect("check id"),
                status: CheckStatus::Fail,
                skip_reason: None,
                violations: vec![bijux_dev_atlas::model::Violation {
                    schema_version: 1,
                    code: bijux_dev_atlas::model::ViolationId::parse("check_execution_error")
                        .expect("violation id"),
                    message: "wrapper drift".to_string(),
                    hint: None,
                    path: None,
                    line: None,
                    severity: bijux_dev_atlas::model::Severity::Error,
                }],
                duration_ms: 10123,
                evidence: Vec::new(),
            },
        ],
        durations_ms: BTreeMap::new(),
        counts: RunSummary {
            schema_version: 1,
            passed: 1,
            failed: 1,
            skipped: 0,
            errors: 0,
            total: 2,
        },
        summary: RunSummary {
            schema_version: 1,
            passed: 1,
            failed: 1,
            skipped: 0,
            errors: 0,
            total: 2,
        },
        timings_ms: BTreeMap::new(),
    };

    let rendered = render_check_run_report(&report, false);
    assert!(rendered.contains("check-run: run_id=checks_run total=2 fail-fast=false"));
    assert!(rendered.contains("PASS [  0.016s] (1/2) checks::checks_docs_index_links main"));
    assert!(rendered.contains(
        "FAIL [ 10.123s] (2/2) checks::checks_make_wrapper_commands main (wrapper drift)"
    ));
    assert!(rendered.contains("check-summary: total=2 passed=1 failed=1 skipped=0 errors=0"));
    assert!(rendered.contains("failed-tests:\nchecks::checks_make_wrapper_commands main"));
}
