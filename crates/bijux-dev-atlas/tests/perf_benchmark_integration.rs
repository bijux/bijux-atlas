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
fn perf_run_writes_benchmark_artifacts() {
    let root = repo_root();
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["perf", "run", "--scenario", "gene-lookup", "--format", "json"])
        .output()
        .expect("run perf command");

    assert!(
        output.status.success(),
        "perf run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let benchmark_json = root.join("artifacts/benchmarks/gene-lookup-result.json");
    let benchmark_csv = root.join("artifacts/benchmarks/gene-lookup-result.csv");
    let benchmark_summary = root.join("artifacts/benchmarks/gene-lookup-summary.json");
    let benchmark_history = root.join("artifacts/benchmarks/gene-lookup-history.json");

    for path in [
        &benchmark_json,
        &benchmark_csv,
        &benchmark_summary,
        &benchmark_history,
    ] {
        assert!(path.exists(), "expected artifact missing: {}", path.display());
    }

    let summary_value: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&benchmark_summary).expect("read summary artifact"),
    )
    .expect("parse summary artifact");
    assert_eq!(summary_value["schema_version"], serde_json::json!(1));
}
