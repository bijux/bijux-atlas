// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn performance_regression_assets_are_present_and_parseable() {
    let root = repo_root();
    let required = [
        "configs/perf/regression-policy.json",
        "configs/perf/regression-notifications.json",
        "ops/report/generated/performance-regression-report.json",
        "ops/report/generated/performance-history.json",
        "ops/report/generated/performance-trend.json",
        "ops/report/generated/performance-anomalies.json",
        "ops/report/generated/performance-dashboard.json",
        "ops/report/generated/performance-ci-artifacts.json",
        "ops/report/generated/performance-ci-summary.json",
        "ops/report/generated/performance-badges.json",
        "ops/report/generated/performance-trend-graph.json",
        "ops/report/generated/performance-metadata.json",
        "ops/report/generated/performance-baseline-verification.json",
        "ops/report/generated/performance-audit-report.json",
        "ops/report/generated/performance-completion-report.json",
        "configs/contracts/perf/performance-metadata.schema.json",
    ];
    for rel in required {
        let path = root.join(rel);
        assert!(path.exists(), "missing required asset `{rel}`");
        let raw = fs::read_to_string(&path).expect("read asset");
        let _: serde_json::Value = serde_json::from_str(&raw).expect("parse asset json");
    }
}
