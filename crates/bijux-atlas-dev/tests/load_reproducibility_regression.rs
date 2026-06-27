// SPDX-License-Identifier: Apache-2.0

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
fn slow_load_comparison_is_available_via_control_plane_command() {
    let output = std::process::Command::new("cargo")
        .args([
            "run",
            "-p",
            "bijux-atlas-dev",
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
