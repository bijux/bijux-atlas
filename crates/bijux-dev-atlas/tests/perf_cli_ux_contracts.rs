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
    assert!(value["latency_ms"]["p99"].is_number());
    assert!(value["latency_ms"]["p50"].is_number());
    assert!(value["system_metadata"].is_object());
    assert!(value["env_snapshot"].is_object());
    assert!(value["runs"].is_number());
    assert!(value["warmup_runs"].is_number());
    assert!(value["command_line_used"].is_string());
}

#[test]
fn perf_cli_ux_diff_writes_regression_report() {
    let root = repo_root();
    let bench = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["perf", "cli-ux", "bench", "--format", "json"])
        .output()
        .expect("run perf cli-ux bench command");
    assert!(
        bench.status.success(),
        "{}",
        String::from_utf8_lossy(&bench.stderr)
    );

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
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&report).expect("read diff report"))
            .expect("parse diff report");
    assert!(value["threshold_flags"]["p95_regressed"].is_boolean());
    assert!(value["threshold_flags"]["p99_regressed"].is_boolean());
}

#[test]
fn slow_perf_cli_ux_completion_mode_executes() {
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

#[test]
fn perf_cli_ux_diff_regression_fixture_sets_threshold_flags() {
    let root = repo_root();
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args([
            "perf",
            "cli-ux",
            "diff",
            "ops/cli/perf/fixtures/cli_ux_report_baseline.json",
            "ops/cli/perf/fixtures/cli_ux_report_regression.json",
            "--format",
            "json",
        ])
        .output()
        .expect("run perf cli-ux diff fixture");
    assert!(
        !output.status.success(),
        "regression fixture should fail with threshold flags"
    );
    let raw = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(raw).expect("parse json");
    assert_eq!(payload["status"], serde_json::json!("failed"));
    assert_eq!(
        payload["threshold_flags"]["p95_regressed"],
        serde_json::json!(true)
    );
    assert_eq!(
        payload["threshold_flags"]["p99_regressed"],
        serde_json::json!(true)
    );
}

#[test]
fn perf_cli_ux_diff_text_matches_golden_format() {
    let root = repo_root();
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args([
            "perf",
            "cli-ux",
            "diff",
            "ops/cli/perf/fixtures/cli_ux_report_baseline.json",
            "ops/cli/perf/fixtures/cli_ux_report_regression.json",
        ])
        .output()
        .expect("run perf cli-ux diff text");
    assert!(
        !output.status.success(),
        "text diff should fail on regression fixture"
    );
    let raw = if output.stdout.is_empty() {
        output.stderr
    } else {
        output.stdout
    };
    let text = String::from_utf8(raw).expect("utf8");
    let normalized = text
        .replace(repo_root().display().to_string().as_str(), "")
        .replace("report=/artifacts/", "report=artifacts/")
        .trim()
        .replace("//", "/");
    let golden = fs::read_to_string(
        root.join("crates/bijux-dev-atlas/tests/goldens/perf-cli-ux-diff-text.golden"),
    )
    .expect("read golden");
    assert_eq!(normalized, golden.trim(), "diff text output drifted");
}
