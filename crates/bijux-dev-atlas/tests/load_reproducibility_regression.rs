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
fn load_reproducibility_and_baseline_assets_are_present() {
    let root = repo_root();

    let deterministic_seed: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("ops/load/contracts/deterministic-seed-policy.json"))
            .expect("read deterministic seed policy"),
    )
    .expect("parse deterministic seed policy");
    assert_eq!(deterministic_seed["schema_version"], serde_json::json!(1));
    assert_eq!(
        deterministic_seed["deterministic_seed"],
        serde_json::json!(11001)
    );

    let baseline: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("ops/load/baselines/system-load-baseline.json"))
            .expect("read baseline"),
    )
    .expect("parse baseline");
    assert_eq!(baseline["schema_version"], serde_json::json!(1));
    for suite in [
        "mixed-workload",
        "ingest-query-workload",
        "heavy-query-workload",
        "read-heavy-workload",
        "write-heavy-workload",
        "long-running-stability",
        "memory-leak-detection",
        "cpu-saturation",
        "disk-io-saturation",
        "thread-pool-exhaustion",
        "shard-hot-spot",
        "cache-thrashing",
        "dataset-churn",
        "artifact-reload",
        "cursor-stress",
    ] {
        assert!(
            baseline["suites"].get(suite).is_some(),
            "missing baseline suite `{suite}`"
        );
    }

    let summary: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("ops/load/generated/system-load-summary.json"))
            .expect("read generated summary"),
    )
    .expect("parse generated summary");
    assert_eq!(summary["schema_version"], serde_json::json!(1));
    assert_eq!(summary["kind"], serde_json::json!("system_load_summary_v1"));
}

#[test]
fn load_comparison_tool_exists() {
    let tool_path = repo_root().join("ops/load/tools/compare-load-report.sh");
    assert!(tool_path.exists(), "missing load comparison tool");
}
