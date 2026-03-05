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
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(payload["action"], "list");
    assert!(payload["counts"]["contract_files"].as_u64().unwrap_or(0) >= 1);
    assert!(payload["counts"]["evidence_items"].as_u64().unwrap_or(0) >= 1);
    assert!(payload["counts"]["dashboard_items"].as_u64().unwrap_or(0) >= 1);
}

#[test]
fn slow_tutorials_workflow_text_uses_nextest_style_summary() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["tutorials", "run", "workflow"])
        .output()
        .expect("tutorials run workflow");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).expect("utf8");
    assert!(
        text.contains("PASS"),
        "workflow output must include PASS lines"
    );
    assert!(
        text.contains("summary: total="),
        "workflow output must include summary line"
    );
}

#[test]
fn slow_tutorials_workflow_only_runs_selected_step() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "tutorials",
            "run",
            "workflow",
            "--only",
            "ingest",
            "--format",
            "json",
        ])
        .output()
        .expect("tutorials run workflow only");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    let steps = payload["steps"].as_array().expect("steps array");
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0]["name"], "ingest");
}

#[test]
fn slow_tutorials_workflow_markdown_output_writes_report() {
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
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let markdown = fs::read_to_string(&out).expect("read markdown");
    assert!(
        markdown.starts_with("# Tutorials"),
        "markdown summary should be rendered"
    );
}

#[test]
fn slow_tutorials_workflow_quiet_text_outputs_nothing() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["tutorials", "run", "workflow", "--quiet"])
        .output()
        .expect("tutorials run workflow quiet");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).trim().is_empty(),
        "quiet mode should suppress text output"
    );
}

#[test]
fn slow_tutorials_workflow_verbose_text_includes_step_details() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["tutorials", "run", "workflow", "--verbose"])
        .output()
        .expect("tutorials run workflow verbose");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).expect("utf8");
    assert!(
        text.contains("detail: step="),
        "verbose mode should include per-step detail lines"
    );
}

#[test]
fn tutorials_dataset_package_writes_tar_artifact() {
    let root = repo_root();
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["tutorials", "dataset", "package", "--format", "json"])
        .output()
        .expect("tutorials dataset package");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    let package_path = payload["package_path"]
        .as_str()
        .expect("package path in payload");
    assert!(root.join(package_path).exists() || PathBuf::from(package_path).exists());
}

#[test]
fn tutorials_dataset_package_can_refresh_sha256_manifest() {
    let root = repo_root();
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args([
            "tutorials",
            "dataset",
            "package",
            "--update-sha256",
            "--format",
            "json",
        ])
        .output()
        .expect("tutorials dataset package --update-sha256");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(payload["sha256_manifest_updated"], true);
}

#[test]
fn tutorials_dataset_integrity_check_passes_for_current_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "tutorials",
            "dataset",
            "integrity-check",
            "--format",
            "json",
        ])
        .output()
        .expect("tutorials dataset integrity-check");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(payload["success"], true);
}

#[test]
fn tutorials_reproducibility_check_writes_evidence_artifact() {
    let root = repo_root();
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["tutorials", "reproducibility-check", "--format", "json"])
        .output()
        .expect("tutorials reproducibility-check");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(
        payload["evidence_artifact"].as_str().unwrap_or_default(),
        "artifacts/tutorials/reproducibility-evidence.json"
    );
}

#[test]
fn tutorials_contracts_explain_returns_schema_context() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["tutorials", "contracts", "explain", "--format", "json"])
        .output()
        .expect("tutorials contracts explain");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(payload["action"], "contracts-explain");
    assert!(payload["required_keys"].as_array().is_some());
}

#[test]
fn tutorials_real_data_list_reports_ten_runs() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["tutorials", "real-data", "list", "--format", "json"])
        .output()
        .expect("tutorials real-data list");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(payload["action"], "real-data-list");
    assert_eq!(payload["runs"].as_array().map_or(0, |rows| rows.len()), 10);
}

#[test]
fn tutorials_real_data_plan_returns_expected_steps() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "tutorials",
            "real-data",
            "plan",
            "--run-id",
            "rdr-001-genes-baseline",
            "--format",
            "json",
        ])
        .output()
        .expect("tutorials real-data plan");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(payload["action"], "real-data-plan");
    assert_eq!(payload["plan"].as_array().map_or(0, |rows| rows.len()), 4);
}

#[test]
fn tutorials_real_data_fetch_and_ingest_write_run_artifacts() {
    let root = repo_root();
    let fetch = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args([
            "tutorials",
            "real-data",
            "fetch",
            "--run-id",
            "rdr-002-transcripts-baseline",
            "--format",
            "json",
        ])
        .output()
        .expect("tutorials real-data fetch");
    assert!(
        fetch.status.success(),
        "{}",
        String::from_utf8_lossy(&fetch.stderr)
    );
    let ingest = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args([
            "tutorials",
            "real-data",
            "ingest",
            "--run-id",
            "rdr-002-transcripts-baseline",
            "--profile",
            "local",
            "--format",
            "json",
        ])
        .output()
        .expect("tutorials real-data ingest");
    assert!(
        ingest.status.success(),
        "{}",
        String::from_utf8_lossy(&ingest.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&ingest.stdout).expect("json");
    let report_path = payload["ingest_report"].as_str().expect("ingest_report path");
    assert!(PathBuf::from(report_path).exists() || root.join(report_path).exists());
}

#[test]
fn tutorials_real_data_run_all_writes_resume_state() {
    let root = repo_root();
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args([
            "tutorials",
            "real-data",
            "run-all",
            "--no-fetch",
            "--dry-run",
            "--format",
            "json",
        ])
        .output()
        .expect("tutorials real-data run-all");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    let state_file = payload["state_file"].as_str().expect("state file");
    assert!(PathBuf::from(state_file).exists() || root.join(state_file).exists());
}

#[test]
fn tutorials_real_data_clean_run_supports_dry_run() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "tutorials",
            "real-data",
            "clean-run",
            "--run-id",
            "rdr-001-genes-baseline",
            "--dry-run",
            "--format",
            "json",
        ])
        .output()
        .expect("tutorials real-data clean-run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(payload["action"], "real-data-clean-run");
    assert_eq!(payload["dry_run"], true);
}

#[test]
fn tutorials_real_data_doctor_reports_tool_checks() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["tutorials", "real-data", "doctor", "--format", "json"])
        .output()
        .expect("tutorials real-data doctor");
    assert!(
        output.status.code().is_some(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert!(payload["tool_checks"].as_array().is_some());
}
