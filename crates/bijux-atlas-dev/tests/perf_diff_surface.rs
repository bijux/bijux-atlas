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
fn perf_diff_command_accepts_system_load_baselines() {
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
