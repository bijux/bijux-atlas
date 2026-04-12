// SPDX-License-Identifier: Apache-2.0

use std::fs;
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
fn slow_doctor_smoke() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docs", "doctor", "--format", "json"])
        .output()
        .expect("doctor");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(bytes).expect("json");
    assert_eq!(
        payload.get("schema_version").and_then(|v| v.as_u64()),
        Some(1)
    );
    assert!(payload
        .get("rows")
        .and_then(|v| v.as_array())
        .is_some());
    assert!(payload.get("counts").is_some());
}

#[test]
fn run_smoke() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["check", "run", "--format", "text"])
        .output()
        .expect("run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success() || !stdout.is_empty(),
        "stdout={stdout}\nstderr={stderr}"
    );
    assert!(stdout.contains("summary:") || stdout.contains("CI_SUMMARY"));
}

#[test]
fn help_snapshot_stable() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .arg("--help")
        .output()
        .expect("help");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("utf8");

    let golden_path = repo_root().join("crates/bijux-dev-atlas/tests/goldens/help.txt");
    let golden = fs::read_to_string(&golden_path).expect("golden");
    assert_eq!(text, golden);
}
