// SPDX-License-Identifier: Apache-2.0

use bijux_dev_atlas::model::{RunHistory, RunHistoryEntry};
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
