use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

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

fn sha256(path: &Path) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(fs::read(path).expect("read file"));
    format!("{:x}", hasher.finalize())
}

#[test]
fn tutorial_dataset_contract_required_fields_match_metadata() {
    let root = repo_root();
    let contract_path = root.join("tutorials/contracts/tutorial-dataset-contract.json");
    let contract: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(contract_path).expect("read contract"))
            .expect("parse contract");
    let required = contract["required"]
        .as_array()
        .expect("required array")
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>();

    for dataset in [
        "atlas-example-minimal",
        "atlas-example-medium",
        "atlas-example-large-synthetic",
    ] {
        let metadata_path = root.join(format!("configs/examples/datasets/{dataset}/metadata.json"));
        let metadata: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(metadata_path).expect("read metadata"))
                .expect("parse metadata");
        for key in &required {
            assert!(
                metadata.get(*key).is_some(),
                "dataset {dataset} missing required key {key}"
            );
        }
        assert!(
            metadata["record_count"].as_u64().unwrap_or_default() > 0,
            "dataset {dataset} must have positive record_count"
        );
    }
}

#[test]
fn tutorial_gene_row_contains_required_fields() {
    let root = repo_root();
    let genes = root.join("configs/examples/datasets/atlas-example-minimal/genes.jsonl");
    let first_line = fs::read_to_string(genes)
        .expect("read genes")
        .lines()
        .next()
        .expect("first row")
        .to_string();
    let row: serde_json::Value = serde_json::from_str(&first_line).expect("parse row");
    for field in ["gene_id", "symbol", "chromosome", "biotype", "length_bp"] {
        assert!(row.get(field).is_some(), "missing field {field}");
    }
}

#[test]
fn tutorial_workflow_command_replaces_legacy_script() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "tutorials",
            "run",
            "workflow",
            "--only",
            "verify",
            "--format",
            "json",
        ])
        .output()
        .expect("run workflow");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn tutorial_dataset_package_command_replaces_legacy_script() {
    let root = repo_root();
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&root)
        .args(["tutorials", "dataset", "package", "--format", "json"])
        .output()
        .expect("run package");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    let package_path = payload["package_path"].as_str().expect("package_path");
    let resolved = root.join(package_path);
    assert!(
        resolved.exists(),
        "package must exist at {}",
        resolved.display()
    );
}

#[test]
fn fixture_missing_dataset_file_fails_integrity_check() {
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
    assert!(!output.status.success(), "expected integrity failure");
}

#[test]
fn fixture_wrong_sha256_fails_integrity_check() {
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
    assert!(!output.status.success(), "expected checksum failure");
}

#[test]
fn fixture_invalid_dashboard_json_fails_validation() {
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
    assert!(!output.status.success(), "expected dashboard parse failure");
}

#[test]
fn fixture_missing_required_panel_fails_validation() {
    let fixture = fixture_root("tutorials_missing_required_panel");
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
    assert!(!output.status.success(), "expected dashboard panel failure");
}

#[test]
fn fixture_missing_evidence_field_fails_validation() {
    let fixture = fixture_root("tutorials_missing_evidence_field");
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
    assert!(!output.status.success(), "expected evidence key failure");
}

#[test]
fn fixture_evidence_schema_mismatch_fails_validation() {
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
    assert!(!output.status.success(), "expected evidence schema failure");
}

#[test]
fn unstable_timestamp_fixture_yields_different_archives_when_disabled() {
    let fixture = fixture_root("tutorials_unstable_tar_ordering");
    let archive = fixture.join("artifacts/tutorials/datasets/atlas-example-minimal.tar");
    let first = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&fixture)
        .args([
            "tutorials",
            "dataset",
            "package",
            "--repo-root",
            ".",
            "--stable-timestamps=false",
            "--format",
            "json",
        ])
        .output()
        .expect("first package run");
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let first_hash = sha256(&archive);
    thread::sleep(Duration::from_secs(1));
    let second = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&fixture)
        .args([
            "tutorials",
            "dataset",
            "package",
            "--repo-root",
            ".",
            "--stable-timestamps=false",
            "--format",
            "json",
        ])
        .output()
        .expect("second package run");
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    let second_hash = sha256(&archive);
    assert_ne!(
        first_hash, second_hash,
        "archive hash should change when timestamps are unstable"
    );
}
