// SPDX-License-Identifier: Apache-2.0

use assert_cmd::Command;
use serde_json::Value;
use std::path::PathBuf;

fn fixture_tiny_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tiny").join(name)
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

fn dataset_db_path(root: &std::path::Path) -> PathBuf {
    root.join("release=110")
        .join("species=homo_sapiens")
        .join("assembly=GRCh38")
        .join("derived/gene_summary.sqlite")
}

#[test]
fn config_json_workflow_is_parseable() {
    let root = repo_root();
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .current_dir(&root)
        .args(["--json", "config"])
        .output()
        .expect("run config");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("config output json");
    assert!(payload.get("workspace_config").is_some());
    assert!(payload.get("cache_dir").is_some());
}

#[test]
fn openapi_generate_workflow_writes_contract_file() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("openapi.generated.json");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .current_dir(&root)
        .args(["--json", "openapi", "generate", "--out"])
        .arg(&out)
        .output()
        .expect("run openapi generate");
    assert!(output.status.success());

    let raw = std::fs::read(&out).expect("openapi file");
    let parsed: serde_json::Value = serde_json::from_slice(&raw).expect("openapi json");
    assert_eq!(parsed["openapi"], "3.0.3");
    assert!(parsed.get("paths").is_some());
}

#[test]
fn cli_fixture_workflow_covers_dataset_create_validate_query_and_refusal() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tempdir");
    let source_root = tmp.path().join("source");
    let store_root = tmp.path().join("store");
    let export_path = tmp.path().join("rows.jsonl");

    let ingest = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .current_dir(&root)
        .args([
            "--json",
            "ingest",
            "--gff3",
            fixture_tiny_path("genes.gff3").to_str().expect("gff3 path"),
            "--fasta",
            fixture_tiny_path("genome.fa").to_str().expect("fasta path"),
            "--fai",
            fixture_tiny_path("genome.fa.fai").to_str().expect("fai path"),
            "--output-root",
            source_root.to_str().expect("source_root"),
            "--release",
            "110",
            "--species",
            "homo_sapiens",
            "--assembly",
            "GRCh38",
        ])
        .output()
        .expect("run ingest");
    assert!(ingest.status.success(), "ingest failed: {}", String::from_utf8_lossy(&ingest.stderr));

    let verify = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .current_dir(&root)
        .args([
            "--json",
            "dataset",
            "verify",
            "--root",
            source_root.to_str().expect("source_root"),
            "--release",
            "110",
            "--species",
            "homo_sapiens",
            "--assembly",
            "GRCh38",
        ])
        .output()
        .expect("run dataset verify");
    assert!(
        verify.status.success(),
        "dataset verify failed: {}",
        String::from_utf8_lossy(&verify.stderr)
    );

    let db = dataset_db_path(&source_root);
    assert!(db.exists(), "expected sqlite output at {}", db.display());

    let query_success = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .current_dir(&root)
        .args(["--json", "query", "run", "--db"])
        .arg(&db)
        .args(["--gene-id", "gene1"])
        .output()
        .expect("run query success");
    assert!(query_success.status.success());
    let query_success_payload: Value =
        serde_json::from_slice(&query_success.stdout).expect("query success payload");
    assert_eq!(query_success_payload["command"].as_str(), Some("atlas query run"));
    assert!(query_success_payload["rows"].as_array().map(|rows| !rows.is_empty()).unwrap_or(false));

    let inspect_dataset = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .current_dir(&root)
        .args([
            "--json",
            "inspect",
            "dataset",
            "--root",
            source_root.to_str().expect("source_root"),
            "--release",
            "110",
            "--species",
            "homo_sapiens",
            "--assembly",
            "GRCh38",
        ])
        .output()
        .expect("run inspect dataset");
    assert!(inspect_dataset.status.success());
    let inspect_dataset_payload: Value =
        serde_json::from_slice(&inspect_dataset.stdout).expect("inspect dataset payload");
    assert_eq!(inspect_dataset_payload["command"].as_str(), Some("atlas inspect dataset"));

    let inspect_db = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .current_dir(&root)
        .args(["--json", "inspect", "db", "--db"])
        .arg(&db)
        .output()
        .expect("run inspect db");
    assert!(inspect_db.status.success());
    let inspect_db_payload: Value =
        serde_json::from_slice(&inspect_db.stdout).expect("inspect db payload");
    assert_eq!(inspect_db_payload["command"].as_str(), Some("atlas inspect db"));

    let query_refusal = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .current_dir(&root)
        .args(["--json", "query", "run", "--db"])
        .arg(&db)
        .output()
        .expect("run query refusal");
    assert_eq!(query_refusal.status.code(), Some(3));
    let query_refusal_stderr = String::from_utf8(query_refusal.stderr).expect("stderr");
    assert!(query_refusal_stderr.contains("validation_error"));

    let export_rows = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .current_dir(&root)
        .args(["--json", "export", "query", "--db"])
        .arg(&db)
        .args(["--out"])
        .arg(&export_path)
        .args(["--gene-id", "gene1", "--format", "jsonl"])
        .output()
        .expect("run export query");
    assert!(export_rows.status.success());
    assert!(export_path.exists());

    let publish_dry_run = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .current_dir(&root)
        .args([
            "--json",
            "dataset",
            "publish",
            "--source-root",
            source_root.to_str().expect("source_root"),
            "--store-root",
            store_root.to_str().expect("store_root"),
            "--release",
            "110",
            "--species",
            "homo_sapiens",
            "--assembly",
            "GRCh38",
            "--dry-run",
        ])
        .output()
        .expect("run dataset publish dry-run");
    assert!(publish_dry_run.status.success());
    let publish_payload: Value =
        serde_json::from_slice(&publish_dry_run.stdout).expect("publish payload");
    assert_eq!(publish_payload["mode"].as_str(), Some("dry-run"));
    assert_eq!(publish_payload["writes_artifacts"].as_bool(), Some(false));

    let publish_explain = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .current_dir(&root)
        .args([
            "--json",
            "dataset",
            "publish",
            "--source-root",
            source_root.to_str().expect("source_root"),
            "--store-root",
            store_root.to_str().expect("store_root"),
            "--release",
            "110",
            "--species",
            "homo_sapiens",
            "--assembly",
            "GRCh38",
            "--explain",
        ])
        .output()
        .expect("run dataset publish explain");
    assert!(publish_explain.status.success());
    let publish_explain_payload: Value =
        serde_json::from_slice(&publish_explain.stdout).expect("publish explain payload");
    assert_eq!(publish_explain_payload["mode"].as_str(), Some("explain"));
    assert_eq!(publish_explain_payload["writes_artifacts"].as_bool(), Some(false));
}

#[test]
fn ingest_dry_run_and_explain_do_not_materialize_artifacts() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tempdir");
    let source_root = tmp.path().join("source");
    let expected_db = dataset_db_path(&source_root);

    let dry_run = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .current_dir(&root)
        .args([
            "--json",
            "ingest",
            "--gff3",
            fixture_tiny_path("genes.gff3").to_str().expect("gff3 path"),
            "--fasta",
            fixture_tiny_path("genome.fa").to_str().expect("fasta path"),
            "--fai",
            fixture_tiny_path("genome.fa.fai").to_str().expect("fai path"),
            "--output-root",
            source_root.to_str().expect("source_root"),
            "--release",
            "110",
            "--species",
            "homo_sapiens",
            "--assembly",
            "GRCh38",
            "--dry-run",
        ])
        .output()
        .expect("run ingest dry-run");
    assert!(dry_run.status.success());
    let payload: Value = serde_json::from_slice(&dry_run.stdout).expect("dry-run payload");
    assert_eq!(payload["mode"].as_str(), Some("dry-run"));
    assert_eq!(payload["writes_artifacts"].as_bool(), Some(false));
    assert!(!expected_db.exists(), "dry-run should not create sqlite");

    let explain = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .current_dir(&root)
        .args([
            "--json",
            "ingest",
            "--gff3",
            fixture_tiny_path("genes.gff3").to_str().expect("gff3 path"),
            "--fasta",
            fixture_tiny_path("genome.fa").to_str().expect("fasta path"),
            "--fai",
            fixture_tiny_path("genome.fa.fai").to_str().expect("fai path"),
            "--output-root",
            source_root.to_str().expect("source_root"),
            "--release",
            "110",
            "--species",
            "homo_sapiens",
            "--assembly",
            "GRCh38",
            "--explain",
        ])
        .output()
        .expect("run ingest explain");
    assert!(explain.status.success());
    let explain_payload: Value = serde_json::from_slice(&explain.stdout).expect("explain payload");
    assert_eq!(explain_payload["mode"].as_str(), Some("explain"));
    assert_eq!(explain_payload["writes_artifacts"].as_bool(), Some(false));
    assert!(!expected_db.exists(), "explain should not create sqlite");
}
