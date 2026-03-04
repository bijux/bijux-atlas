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
    assert_eq!(baseline["metadata"]["schema_version"], serde_json::json!(1));
    assert_eq!(baseline["name"], serde_json::json!("system-load-baseline"));
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
            baseline["rows"]
                .as_array()
                .is_some_and(|rows| rows.iter().any(|row| row["suite"] == suite)),
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
fn load_comparison_is_available_via_control_plane_command() {
    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "-p",
            "bijux-dev-atlas",
            "--",
            "perf",
            "diff",
            "ops/load/baselines/system-load-baseline.json",
            "ops/load/baselines/system-load-baseline.json",
            "--format",
            "json",
        ])
        .current_dir(repo_root())
        .output()
        .expect("run perf diff command");
    assert!(
        output.status.success(),
        "perf diff command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("perf diff json output");
    assert_eq!(value["status"], serde_json::json!("ok"));
}
