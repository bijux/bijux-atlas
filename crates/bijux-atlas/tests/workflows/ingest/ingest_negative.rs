// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use bijux_atlas::domain::dataset::DatasetId;
use bijux_atlas::domain::ingest::{ingest_dataset, IngestOptions};
use bijux_atlas::domain::policy::StrictnessMode;
use tempfile::tempdir;

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

#[test]
fn malformed_gff3_coordinates_fail_with_validation_error() {
    let out = tempdir().expect("tmp");
    let dataset = DatasetId::new("111", "homo_sapiens", "GRCh38").expect("dataset");
    let opts = IngestOptions {
        gff3_path: fixture("tests/fixtures/contigs/genes_invalid_coord.gff3"),
        fasta_path: fixture("tests/fixtures/contigs/genome.fa"),
        fai_path: fixture("tests/fixtures/contigs/genome.fa.fai"),
        output_root: out.path().to_path_buf(),
        strictness: StrictnessMode::Strict,
        ..IngestOptions::for_dataset(dataset)
    };

    let err = ingest_dataset(&opts).expect_err("invalid coordinate must fail");
    assert!(
        err.0.contains("GFF3_INVALID_START_COORDINATE")
            || err.0.contains("GFF3_INVALID_END_COORDINATE")
            || err.0.contains("GFF3_INVALID_COORDINATE_SPAN")
            || err.0.contains("exceeds contig"),
        "unexpected error: {}",
        err.0
    );
}

#[test]
fn missing_fai_without_opt_in_auto_generation_fails() {
    let out = tempdir().expect("tmp");
    let dataset = DatasetId::new("112", "homo_sapiens", "GRCh38").expect("dataset");
    let opts = IngestOptions {
        gff3_path: fixture("tests/fixtures/tiny/genes.gff3"),
        fasta_path: fixture("tests/fixtures/tiny/genome.fa"),
        fai_path: out.path().join("missing.fa.fai"),
        output_root: out.path().to_path_buf(),
        strictness: StrictnessMode::Strict,
        dev_allow_auto_generate_fai: false,
        ..IngestOptions::for_dataset(dataset)
    };

    let err = ingest_dataset(&opts).expect_err("missing fai must fail");
    assert!(
        err.0.contains("FAI_REQUIRED_FOR_INGEST"),
        "unexpected error: {}",
        err.0
    );
}

#[test]
fn gff3_reference_names_must_exist_in_fasta_fai() {
    let out = tempdir().expect("tmp");
    let dataset = DatasetId::new("113", "homo_sapiens", "GRCh38").expect("dataset");
    let opts = IngestOptions {
        gff3_path: fixture("tests/fixtures/edgecases/case_9_unknown_contig.gff3"),
        fasta_path: fixture("tests/fixtures/tiny/genome.fa"),
        fai_path: fixture("tests/fixtures/tiny/genome.fa.fai"),
        output_root: out.path().to_path_buf(),
        strictness: StrictnessMode::Strict,
        ..IngestOptions::for_dataset(dataset)
    };

    let err = ingest_dataset(&opts).expect_err("unknown seqid must fail");
    assert!(err.0.contains("GFF3_REFERENCE_NOT_IN_FASTA_FAI"));
}

#[test]
fn malformed_fasta_fixture_is_rejected_before_ingest_build() {
    let out = tempdir().expect("tmp");
    let dataset = DatasetId::new("114", "homo_sapiens", "GRCh38").expect("dataset");
    let opts = IngestOptions {
        gff3_path: fixture("tests/fixtures/tiny/genes.gff3"),
        fasta_path: fixture("tests/fixtures/ingest_guardrails/malformed_fasta_missing_header.fa"),
        fai_path: out.path().join("autogen.fai"),
        output_root: out.path().to_path_buf(),
        strictness: StrictnessMode::Strict,
        dev_allow_auto_generate_fai: true,
        ..IngestOptions::for_dataset(dataset)
    };
    let err = ingest_dataset(&opts).expect_err("malformed fasta should fail");
    assert!(err.0.contains("FASTA sequence line seen before header"));
}

#[test]
fn malformed_gff3_fixture_with_wrong_column_count_is_rejected() {
    let out = tempdir().expect("tmp");
    let dataset = DatasetId::new("115", "homo_sapiens", "GRCh38").expect("dataset");
    let opts = IngestOptions {
        gff3_path: fixture("tests/fixtures/ingest_guardrails/malformed_gff3_columns.gff3"),
        fasta_path: fixture("tests/fixtures/tiny/genome.fa"),
        fai_path: fixture("tests/fixtures/tiny/genome.fa.fai"),
        output_root: out.path().to_path_buf(),
        strictness: StrictnessMode::Strict,
        ..IngestOptions::for_dataset(dataset)
    };
    let err = ingest_dataset(&opts).expect_err("malformed gff3 should fail");
    assert!(err.0.contains("invalid GFF3 row"));
}

#[test]
fn conflicting_sequence_region_fixture_is_rejected() {
    let out = tempdir().expect("tmp");
    let dataset = DatasetId::new("116", "homo_sapiens", "GRCh38").expect("dataset");
    let opts = IngestOptions {
        gff3_path: fixture("tests/fixtures/ingest_guardrails/conflicting_sequence_region.gff3"),
        fasta_path: fixture("tests/fixtures/tiny/genome.fa"),
        fai_path: fixture("tests/fixtures/tiny/genome.fa.fai"),
        output_root: out.path().to_path_buf(),
        strictness: StrictnessMode::Strict,
        ..IngestOptions::for_dataset(dataset)
    };
    let err = ingest_dataset(&opts).expect_err("conflicting seq region should fail");
    assert!(err.0.contains("GFF3_CONFLICTING_SEQUENCE_REGION"));
}

#[test]
fn duplicate_feature_id_fixture_is_rejected() {
    let out = tempdir().expect("tmp");
    let dataset = DatasetId::new("117", "homo_sapiens", "GRCh38").expect("dataset");
    let opts = IngestOptions {
        gff3_path: fixture("tests/fixtures/ingest_guardrails/duplicate_feature_ids.gff3"),
        fasta_path: fixture("tests/fixtures/tiny/genome.fa"),
        fai_path: fixture("tests/fixtures/tiny/genome.fa.fai"),
        output_root: out.path().to_path_buf(),
        strictness: StrictnessMode::Strict,
        ..IngestOptions::for_dataset(dataset)
    };
    let err = ingest_dataset(&opts).expect_err("duplicate feature ids should fail");
    assert!(err.0.contains("duplicate feature ID"));
}
