// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates root")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn perf_cli_ux_bench_writes_artifacts() {
    let root = repo_root();
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["perf", "cli-ux", "bench", "--format", "json"])
        .output()
        .expect("run perf cli-ux bench command");

    assert!(
        output.status.success(),
        "perf cli-ux bench failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report = root.join("artifacts/perf/cli-ux/latest-report.json");
    let summary = root.join("artifacts/perf/cli-ux/latest-summary.md");
    assert!(report.exists(), "missing report: {}", report.display());
    assert!(summary.exists(), "missing summary: {}", summary.display());

    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&report).expect("read report"))
            .expect("parse report");
    assert_eq!(value["schema_version"], serde_json::json!(1));
    assert!(value["latency_ms"]["p95"].is_number());
}

#[test]
fn perf_cli_ux_diff_writes_regression_report() {
    let root = repo_root();
    let bench = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["perf", "cli-ux", "bench", "--format", "json"])
        .output()
        .expect("run perf cli-ux bench command");
    assert!(bench.status.success(), "{}", String::from_utf8_lossy(&bench.stderr));

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args([
            "perf",
            "cli-ux",
            "diff",
            "artifacts/perf/cli-ux/latest-report.json",
            "artifacts/perf/cli-ux/latest-report.json",
            "--format",
            "json",
        ])
        .output()
        .expect("run perf cli-ux diff command");

    assert!(
        output.status.success(),
        "perf cli-ux diff failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = root.join("artifacts/perf/cli-ux/latest-diff.json");
    assert!(report.exists(), "missing diff report: {}", report.display());
}

#[test]
fn perf_cli_ux_completion_mode_executes() {
    let root = repo_root();
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args([
            "perf",
            "cli-ux",
            "bench",
            "--mode",
            "completion",
            "--format",
            "json",
        ])
        .output()
        .expect("run perf cli-ux completion mode");
    assert!(
        output.status.success(),
        "perf cli-ux completion mode failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
