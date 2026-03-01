// SPDX-License-Identifier: Apache-2.0

use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn plugin_metadata_handshake_has_required_fields() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .arg("--bijux-plugin-metadata")
        .output()
        .expect("run plugin metadata command");
    assert!(output.status.success());

    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid JSON metadata");
    for key in [
        "schema_version",
        "name",
        "version",
        "compatible_umbrella",
        "compatible_umbrella_min",
        "compatible_umbrella_max_exclusive",
        "build_hash",
    ] {
        assert!(payload.get(key).is_some(), "missing required field {key}");
    }
    assert_eq!(
        payload.get("name").and_then(Value::as_str),
        Some("bijux-atlas")
    );
}

#[test]
fn plugin_metadata_matches_snapshot_contract() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .arg("--bijux-plugin-metadata")
        .output()
        .expect("run plugin metadata command");
    assert!(output.status.success());
    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid JSON metadata");

    let snapshot_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs/PLUGIN_METADATA_SNAPSHOT.json");
    let snapshot_text = std::fs::read_to_string(snapshot_path).expect("read metadata snapshot");
    let snapshot_text = snapshot_text.replace("__CARGO_PKG_VERSION__", env!("CARGO_PKG_VERSION"));
    let expected: Value = serde_json::from_str(&snapshot_text).expect("parse metadata snapshot");
    assert_eq!(payload, expected);
}

#[test]
fn plugin_contract_doc_includes_required_sections() {
    let text = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/plugin-contract.md"),
    )
    .expect("read plugin contract doc");
    for needle in [
        "--bijux-plugin-metadata",
        "--json",
        "--quiet",
        "--verbose",
        "--trace",
        "compatible_umbrella_min",
        "compatible_umbrella_max_exclusive",
        "build_hash",
        "PLUGIN_METADATA_SNAPSHOT.json",
    ] {
        assert!(
            text.contains(needle),
            "plugin contract doc missing `{needle}`"
        );
    }
}

#[test]
fn atlas_validate_command_supports_deep_mode() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .args(["atlas", "validate", "--help"])
        .output()
        .expect("run atlas validate help");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("utf8 help");
    assert!(text.contains("--deep"));
    assert!(text.contains("--release"));
    assert!(text.contains("--species"));
    assert!(text.contains("--assembly"));
}

#[test]
fn umbrella_version_compatibility_is_enforced() {
    let bad = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .args(["--json", "--umbrella-version", "0.2.1", "version"])
        .output()
        .expect("run with incompatible umbrella version");
    assert_eq!(bad.status.code(), Some(2));
    let stderr = String::from_utf8(bad.stderr).expect("stderr utf8");
    assert!(stderr.contains("\"code\":\"umbrella_incompatible\""));
}

#[test]
fn help_contains_standard_plugin_flags() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .arg("--help")
        .output()
        .expect("run help");
    assert!(output.status.success());

    let text = String::from_utf8(output.stdout).expect("utf8 help");
    for needle in [
        "--json",
        "--quiet",
        "--verbose",
        "--trace",
        "--bijux-plugin-metadata",
    ] {
        assert!(text.contains(needle), "help missing {needle}");
    }
}

#[test]
fn atlas_namespace_help_is_stable() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .args(["atlas", "--help"])
        .output()
        .expect("run atlas help");
    assert!(output.status.success());

    let text = String::from_utf8(output.stdout).expect("utf8 help");
    for needle in ["ingest", "serve", "catalog", "dataset", "openapi"] {
        assert!(text.contains(needle), "atlas help missing {needle}");
    }
}

#[test]
fn unknown_arguments_exit_with_usage_code() {
    let status = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .arg("--not-a-real-flag")
        .status()
        .expect("run with bad flag");
    assert_eq!(status.code(), Some(2));
}

#[test]
fn json_error_contract_is_stable() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .args(["--json", "--not-a-real-flag"])
        .output()
        .expect("run with bad flag");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
    assert!(stderr.contains("\"code\":\"usage_error\""));
    assert!(stderr.contains("\"message\":\"invalid command line arguments\""));
}

#[test]
fn atlas_validate_deep_requires_manifest_lock() {
    let root = tempdir().expect("tempdir");
    let dataset =
        bijux_atlas_model::DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset");
    let paths = bijux_atlas_model::artifact_paths(root.path(), &dataset);
    fs::create_dir_all(&paths.inputs_dir).expect("inputs dir");
    fs::create_dir_all(&paths.derived_dir).expect("derived dir");

    fs::write(&paths.gff3, b"##gff-version 3\n").expect("write gff3");
    fs::write(&paths.fasta, b">chr1\nACGT\n").expect("write fasta");
    fs::write(&paths.fai, b"chr1\t4\t6\t4\t5\n").expect("write fai");
    create_minimal_valid_sqlite(&paths.sqlite);
    fs::write(
        &paths.anomaly_report,
        serde_json::to_vec(&bijux_atlas_model::IngestAnomalyReport::default())
            .expect("anomaly json"),
    )
    .expect("write anomaly");

    let sqlite_bytes = fs::read(&paths.sqlite).expect("read sqlite");
    let mut manifest = bijux_atlas_model::ArtifactManifest::new(
        "1".to_string(),
        "1".to_string(),
        dataset,
        bijux_atlas_model::ArtifactChecksums::new(
            bijux_atlas_core::sha256_hex(b"##gff-version 3\n"),
            bijux_atlas_core::sha256_hex(b">chr1\nACGT\n"),
            bijux_atlas_core::sha256_hex(b"chr1\t4\t6\t4\t5\n"),
            bijux_atlas_core::sha256_hex(&sqlite_bytes),
        ),
        bijux_atlas_model::ManifestStats::new(1, 1, 1),
    );
    let sqlite_sha = bijux_atlas_core::sha256_hex(&sqlite_bytes);
    manifest.input_hashes = bijux_atlas_model::ManifestInputHashes::new(
        bijux_atlas_core::sha256_hex(b"##gff-version 3\n"),
        bijux_atlas_core::sha256_hex(b">chr1\nACGT\n"),
        bijux_atlas_core::sha256_hex(b"chr1\t4\t6\t4\t5\n"),
        "policy-hash".to_string(),
    );
    manifest.toolchain_hash = "toolchain-hash".to_string();
    manifest.db_hash = sqlite_sha.clone();
    manifest.artifact_hash = sqlite_sha;
    fs::write(
        &paths.manifest,
        serde_json::to_vec(&manifest).expect("manifest json"),
    )
    .expect("write manifest");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-atlas"))
        .args([
            "--json",
            "atlas",
            "validate",
            "--root",
            root.path().to_str().expect("root str"),
            "--release",
            "110",
            "--species",
            "homo_sapiens",
            "--assembly",
            "GRCh38",
            "--deep",
        ])
        .output()
        .expect("run atlas validate --deep");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
    assert!(stderr.contains("\"code\":\"internal_error\""));
    assert!(
        stderr.contains("manifest.lock missing")
            || stderr.contains("manifest input_hashes are required"),
        "unexpected stderr: {stderr}"
    );
}

fn create_minimal_valid_sqlite(path: &Path) {
    let conn = rusqlite::Connection::open(path).expect("open sqlite");
    conn.execute_batch(
        "CREATE TABLE gene_summary(
            id INTEGER PRIMARY KEY,
            gene_id TEXT,
            name TEXT,
            name_normalized TEXT,
            biotype TEXT,
            seqid TEXT,
            start INT,
            end INT,
            transcript_count INT,
            exon_count INT DEFAULT 0,
            total_exon_span INT DEFAULT 0,
            cds_present INT DEFAULT 0,
            sequence_length INT
        );
        CREATE TABLE transcript_summary(
            id INTEGER PRIMARY KEY,
            transcript_id TEXT,
            parent_gene_id TEXT,
            transcript_type TEXT,
            biotype TEXT,
            seqid TEXT,
            start INT,
            end INT,
            exon_count INT,
            total_exon_span INT,
            cds_present INT
        );
        CREATE VIRTUAL TABLE gene_summary_rtree USING rtree(gene_rowid, start, end);
        CREATE TABLE atlas_meta(k TEXT PRIMARY KEY, v TEXT NOT NULL);
        INSERT INTO atlas_meta(k, v) VALUES ('analyze_completed', 'true');
        INSERT INTO atlas_meta(k, v) VALUES ('schema_version', '1');
        CREATE INDEX idx_gene_summary_gene_id ON gene_summary(gene_id);
        CREATE INDEX idx_gene_summary_name ON gene_summary(name);
        CREATE INDEX idx_gene_summary_name_normalized ON gene_summary(name_normalized);
        CREATE INDEX idx_gene_summary_biotype ON gene_summary(biotype);
        CREATE INDEX idx_gene_summary_region ON gene_summary(seqid,start,end);
        CREATE INDEX idx_gene_summary_cover_lookup ON gene_summary(gene_id,name,biotype,transcript_count,sequence_length);
        CREATE INDEX idx_gene_summary_cover_region ON gene_summary(seqid,start,gene_id,name);
        CREATE INDEX idx_transcript_summary_transcript_id ON transcript_summary(transcript_id);
        CREATE INDEX idx_transcript_summary_parent_gene_id ON transcript_summary(parent_gene_id);
        CREATE INDEX idx_transcript_summary_biotype ON transcript_summary(biotype);
        CREATE INDEX idx_transcript_summary_type ON transcript_summary(transcript_type);
        CREATE INDEX idx_transcript_summary_region ON transcript_summary(seqid,start,end);
        INSERT INTO gene_summary(id,gene_id,name,name_normalized,biotype,seqid,start,end,transcript_count,exon_count,total_exon_span,cds_present,sequence_length)
          VALUES (1,'g1','G1','g1','protein_coding','chr1',1,4,1,0,0,0,4);
        INSERT INTO transcript_summary(id,transcript_id,parent_gene_id,transcript_type,biotype,seqid,start,end,exon_count,total_exon_span,cds_present)
          VALUES (1,'tx1','g1','transcript','protein_coding','chr1',1,4,0,0,0);
        PRAGMA user_version=1;",
    )
    .expect("seed sqlite");
}
