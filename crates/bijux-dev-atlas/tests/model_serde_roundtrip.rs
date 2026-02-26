// SPDX-License-Identifier: Apache-2.0

use bijux_dev_atlas::model::{
    schema_version, ArtifactPath, CheckId, CheckResult, CheckStatus, EvidenceRef, RunId, RunReport,
    RunSummary, Severity, Violation, ViolationId,
};
use std::collections::BTreeMap;

fn sample_report() -> RunReport {
    let violation = Violation {
        schema_version: schema_version(),
        code: ViolationId::parse("ops_contract_missing").expect("id"),
        message: "missing contract".to_string(),
        hint: Some("restore file".to_string()),
        path: Some(ArtifactPath::parse("ops/CONTRACT.md").expect("path")),
        line: Some(1),
        severity: Severity::Error,
    };
    let evidence = EvidenceRef {
        schema_version: schema_version(),
        kind: "text".to_string(),
        path: ArtifactPath::parse("artifacts/atlas-dev/registry_run/report.json").expect("path"),
        content_type: "application/json".to_string(),
        description: "report output".to_string(),
    };
    let result = CheckResult {
        schema_version: schema_version(),
        id: CheckId::parse("checks_ops_surface_manifest").expect("id"),
        status: CheckStatus::Fail,
        skip_reason: None,
        violations: vec![violation],
        duration_ms: 12,
        evidence: vec![evidence],
    };
    let mut timings = BTreeMap::new();
    timings.insert(
        CheckId::parse("checks_ops_surface_manifest").expect("id"),
        12,
    );
    let summary = RunSummary {
        schema_version: schema_version(),
        passed: 0,
        failed: 1,
        skipped: 0,
        errors: 0,
        total: 1,
    };
    RunReport {
        schema_version: schema_version(),
        run_id: RunId::from_seed("registry_run"),
        repo_root: "/repo".to_string(),
        command: "check run".to_string(),
        selections: BTreeMap::new(),
        capabilities: BTreeMap::new(),
        results: vec![result],
        durations_ms: timings.clone(),
        counts: summary.clone(),
        summary,
        timings_ms: timings,
    }
}

#[test]
fn serde_roundtrip_report_types() {
    let report = sample_report();
    let json = serde_json::to_string_pretty(&report).expect("json");
    let restored: RunReport = serde_json::from_str(&json).expect("restore");
    assert_eq!(report, restored);
}

#[test]
fn fingerprint_is_stable_for_same_violation() {
    let report = sample_report();
    let violation = &report.results[0].violations[0];
    let one = bijux_dev_atlas::model::fingerprint::violation_fingerprint(violation);
    let two = bijux_dev_atlas::model::fingerprint::violation_fingerprint(violation);
    assert_eq!(one, two);
}
