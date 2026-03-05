// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn load_explain_emits_registry_and_measurement_coverage() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["load", "explain", "--format", "json"])
        .output()
        .expect("load explain");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json output");
    assert_eq!(
        payload.get("kind").and_then(|v| v.as_str()),
        Some("load_explain")
    );
}

#[test]
fn load_baseline_run_and_compare_flow() {
    let root = repo_root();
    let baseline = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args([
            "load",
            "baseline",
            "--scenario",
            "mixed_workload",
            "--duration-secs",
            "120",
            "--format",
            "json",
        ])
        .output()
        .expect("baseline");
    assert!(baseline.status.success());

    let run = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args([
            "load",
            "run",
            "--scenario",
            "mixed_workload",
            "--duration-secs",
            "120",
            "--format",
            "json",
        ])
        .output()
        .expect("run");
    assert!(run.status.success());

    let compare = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["load", "compare", "--format", "json"])
        .output()
        .expect("compare");
    assert!(compare.status.success() || compare.status.code() == Some(2));
    let payload: serde_json::Value = serde_json::from_slice(&compare.stdout).expect("json output");
    assert_eq!(
        payload.get("kind").and_then(|v| v.as_str()),
        Some("load_compare")
    );
}
