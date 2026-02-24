// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use bijux_atlas_ingest::{ingest_dataset, IngestOptions};
use bijux_atlas_model::{DatasetId, StrictnessMode};
use tempfile::tempdir;

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

#[test]
fn malformed_gff3_coordinates_fail_with_validation_error() {
    let out = tempdir().expect("tmp");
    let opts = IngestOptions {
        gff3_path: fixture("tests/fixtures/contigs/genes_invalid_coord.gff3"),
        fasta_path: fixture("tests/fixtures/contigs/genome.fa"),
        fai_path: fixture("tests/fixtures/contigs/genome.fa.fai"),
        output_root: out.path().to_path_buf(),
        dataset: DatasetId::new("111", "homo_sapiens", "GRCh38").expect("dataset"),
        strictness: StrictnessMode::Strict,
        ..IngestOptions::default()
    };

    let err = ingest_dataset(&opts).expect_err("invalid coordinate must fail");
    assert!(
        err.0.contains("invalid coordinate")
            || err.0.contains("invalid coordinate span")
            || err.0.contains("exceeds contig"),
        "unexpected error: {}",
        err.0
    );
}

#[test]
fn missing_fai_without_opt_in_auto_generation_fails() {
    let out = tempdir().expect("tmp");
    let opts = IngestOptions {
        gff3_path: fixture("tests/fixtures/tiny/genes.gff3"),
        fasta_path: fixture("tests/fixtures/tiny/genome.fa"),
        fai_path: out.path().join("missing.fa.fai"),
        output_root: out.path().to_path_buf(),
        dataset: DatasetId::new("112", "homo_sapiens", "GRCh38").expect("dataset"),
        strictness: StrictnessMode::Strict,
        dev_allow_auto_generate_fai: false,
        ..IngestOptions::default()
    };

    let err = ingest_dataset(&opts).expect_err("missing fai must fail");
    assert!(
        err.0.contains("FAI index is required"),
        "unexpected error: {}",
        err.0
    );
}
