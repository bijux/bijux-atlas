// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use bijux_atlas::domain::dataset::{ArtifactManifest, DatasetId};
use bijux_atlas::domain::ingest::{ingest_dataset_with_events, IngestOptions, TimestampPolicy};
use bijux_atlas::domain::policy::StrictnessMode;
use bijux_atlas::domain::sha256_hex;
use tempfile::tempdir;

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

#[test]
fn fixture_ingest_produces_expected_artifacts_and_hashes() {
    let out = tempdir().expect("tmp");
    let dataset = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset");
    let opts = IngestOptions {
        gff3_path: fixture("tests/fixtures/tiny/genes.gff3"),
        fasta_path: fixture("tests/fixtures/tiny/genome.fa"),
        fai_path: fixture("tests/fixtures/tiny/genome.fa.fai"),
        output_root: out.path().to_path_buf(),
        strictness: StrictnessMode::Strict,
        timestamp_policy: TimestampPolicy::DeterministicZero,
        ..IngestOptions::for_dataset(dataset)
    };

    let (result, events) = ingest_dataset_with_events(&opts).expect("ingest");

    assert!(result.manifest_path.exists());
    assert!(result.sqlite_path.exists());
    assert!(result.release_gene_index_path.exists());

    let manifest_bytes = std::fs::read(&result.manifest_path).expect("manifest bytes");
    let sqlite_bytes = std::fs::read(&result.sqlite_path).expect("sqlite bytes");
    let manifest: ArtifactManifest =
        serde_json::from_slice(&manifest_bytes).expect("manifest json");

    assert_eq!(manifest.checksums.sqlite_sha256, sha256_hex(&sqlite_bytes));
    assert_eq!(
        manifest.input_hashes.gff3_sha256,
        sha256_hex(&std::fs::read(&opts.gff3_path).expect("gff"))
    );
    assert_eq!(
        manifest.input_hashes.fasta_sha256,
        sha256_hex(&std::fs::read(&opts.fasta_path).expect("fasta"))
    );
    assert_eq!(
        manifest.input_hashes.fai_sha256,
        sha256_hex(&std::fs::read(&opts.fai_path).expect("fai"))
    );

    assert!(
        !events.is_empty(),
        "structured ingest events must be recorded"
    );
}
