use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn fixture_root(name: &str) -> PathBuf {
    repo_root()
        .join("crates/bijux-dev-atlas/tests/fixtures")
        .join(name)
}

#[test]
fn tutorial_contract_file_exists_and_validates() {
    let root = repo_root();
    let contract_path = root.join("ops/tutorials/contracts/tutorial-dataset-contract.json");
    assert!(contract_path.exists(), "contract file must exist");
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["tutorials", "contracts", "validate", "--format", "json"])
        .output()
        .expect("tutorials contracts validate");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json payload");
    assert_eq!(payload["success"], true);
}

#[test]
fn tutorial_contract_references_existing_files() {
    let fixture = fixture_root("tutorials_missing_dataset_file");
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&fixture)
        .args([
            "tutorials",
            "dataset",
            "integrity-check",
            "--repo-root",
            ".",
            "--format",
            "json",
        ])
        .output()
        .expect("integrity check");
    assert!(
        !output.status.success(),
        "missing file must fail integrity check"
    );
}

#[test]
fn tutorial_sha256_manifest_matches_packaged_files() {
    let fixture = fixture_root("tutorials_wrong_sha256");
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&fixture)
        .args([
            "tutorials",
            "dataset",
            "integrity-check",
            "--repo-root",
            ".",
            "--format",
            "json",
        ])
        .output()
        .expect("integrity check");
    assert!(
        !output.status.success(),
        "wrong digest must fail integrity check"
    );
}

#[test]
fn tutorial_dashboards_validate_against_required_shape() {
    let fixture = fixture_root("tutorials_invalid_dashboard_json");
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&fixture)
        .args([
            "tutorials",
            "dashboards",
            "validate",
            "--repo-root",
            ".",
            "--format",
            "json",
        ])
        .output()
        .expect("dashboards validate");
    assert!(!output.status.success(), "invalid dashboard json must fail");
}

#[test]
fn tutorial_evidence_validate_against_required_shape() {
    let fixture = fixture_root("tutorials_evidence_schema_mismatch");
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&fixture)
        .args([
            "tutorials",
            "evidence",
            "validate",
            "--repo-root",
            ".",
            "--format",
            "json",
        ])
        .output()
        .expect("evidence validate");
    assert!(
        !output.status.success(),
        "invalid evidence schema must fail"
    );
}

#[test]
fn tutorial_reproducibility_run_writes_evidence_artifact() {
    let root = repo_root();
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["tutorials", "reproducibility-check", "--format", "json"])
        .output()
        .expect("reproducibility check");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let artifact = root.join("artifacts/tutorials/reproducibility-evidence.json");
    assert!(artifact.exists(), "evidence artifact must be written");
}

#[test]
fn slow_tutorial_workflow_report_contains_summary_counts() {
    let root = repo_root();
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["tutorials", "run", "workflow", "--format", "json"])
        .output()
        .expect("workflow run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json payload");
    assert!(payload["summary"]["total"].as_u64().is_some());
    assert!(payload["summary"]["passed"].as_u64().is_some());
    assert!(payload["summary"]["failed"].as_u64().is_some());
}

#[test]
fn tutorial_list_output_is_deterministic_for_stable_inputs() {
    let root = repo_root();
    let first = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["tutorials", "list", "--format", "json"])
        .output()
        .expect("tutorial list first");
    let second = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["tutorials", "list", "--format", "json"])
        .output()
        .expect("tutorial list second");
    assert!(first.status.success() && second.status.success());
    let one: serde_json::Value = serde_json::from_slice(&first.stdout).expect("first json");
    let two: serde_json::Value = serde_json::from_slice(&second.stdout).expect("second json");
    assert_eq!(one, two, "tutorial list output should be deterministic");
}

#[test]
fn tutorial_workspace_cleanup_refuses_outside_repo_targets() {
    let root = repo_root();
    let outside = root.parent().expect("parent").join("do-not-delete");
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args([
            "tutorials",
            "workspace",
            "cleanup",
            "--path",
            outside.to_string_lossy().as_ref(),
            "--format",
            "json",
        ])
        .output()
        .expect("workspace cleanup");
    assert!(!output.status.success(), "cleanup must reject outside path");
}
