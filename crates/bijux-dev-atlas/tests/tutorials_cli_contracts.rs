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
fn tutorials_list_reports_asset_counts() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["tutorials", "list", "--format", "json"])
        .output()
        .expect("tutorials list");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(payload["action"], "list");
    assert!(payload["counts"]["contract_files"].as_u64().unwrap_or(0) >= 1);
    assert!(payload["counts"]["evidence_items"].as_u64().unwrap_or(0) >= 1);
    assert!(payload["counts"]["dashboard_items"].as_u64().unwrap_or(0) >= 1);
}

#[test]
fn tutorials_workflow_text_uses_nextest_style_summary() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["tutorials", "run", "workflow"])
        .output()
        .expect("tutorials run workflow");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let text = String::from_utf8(output.stdout).expect("utf8");
    assert!(text.contains("PASS"), "workflow output must include PASS lines");
    assert!(text.contains("summary: total="), "workflow output must include summary line");
}

#[test]
fn tutorials_workflow_markdown_output_writes_report() {
    let root = repo_root();
    let out = root.join("artifacts/tutorials/tests-workflow.md");
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args([
            "tutorials",
            "run",
            "workflow",
            "--markdown",
            "--out",
            out.to_str().expect("path utf8"),
        ])
        .output()
        .expect("tutorials workflow markdown");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let markdown = fs::read_to_string(&out).expect("read markdown");
    assert!(markdown.starts_with("# Tutorials"), "markdown summary should be rendered");
}

#[test]
fn tutorials_dataset_package_writes_tar_artifact() {
    let root = repo_root();
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["tutorials", "dataset", "package", "--format", "json"])
        .output()
        .expect("tutorials dataset package");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    let package_path = payload["package_path"]
        .as_str()
        .expect("package path in payload");
    assert!(root.join(package_path).exists() || PathBuf::from(package_path).exists());
}

#[test]
fn tutorials_dataset_integrity_check_passes_for_current_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["tutorials", "dataset", "integrity-check", "--format", "json"])
        .output()
        .expect("tutorials dataset integrity-check");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(payload["success"], true);
}

