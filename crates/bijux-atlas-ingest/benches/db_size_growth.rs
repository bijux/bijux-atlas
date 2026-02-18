use bijux_atlas_ingest::{ingest_dataset, IngestOptions};
use bijux_atlas_model::{
    BiotypePolicy, DatasetId, DuplicateGeneIdPolicy, DuplicateTranscriptIdPolicy,
    FeatureIdUniquenessPolicy, GeneIdentifierPolicy, GeneNamePolicy, SeqidNormalizationPolicy,
    StrictnessMode, TranscriptIdPolicy, TranscriptTypePolicy, UnknownFeaturePolicy,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::PathBuf;
use tempfile::tempdir;

fn opts_for_fixture(base: &std::path::Path, fixture_dir: &str) -> IngestOptions {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(fixture_dir);
    IngestOptions {
        gff3_path: fixture.join("genes.gff3"),
        fasta_path: fixture.join("genome.fa"),
        fai_path: fixture.join("genome.fa.fai"),
        output_root: base.to_path_buf(),
        dataset: DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset"),
        strictness: StrictnessMode::Lenient,
        duplicate_gene_id_policy: DuplicateGeneIdPolicy::Fail,
        gene_identifier_policy: GeneIdentifierPolicy::Gff3Id,
        gene_name_policy: GeneNamePolicy::default(),
        biotype_policy: BiotypePolicy::default(),
        transcript_type_policy: TranscriptTypePolicy::default(),
        seqid_policy: SeqidNormalizationPolicy::default(),
        max_threads: 1,
        report_only: false,
        fail_on_warn: false,
        allow_overlap_gene_ids_across_contigs: false,
        emit_shards: false,
        shard_partitions: 0,
        compute_gene_signatures: true,
        compute_contig_fractions: false,
        fasta_scanning_enabled: false,
        fasta_scan_max_bases: 2_000_000_000,
        compute_transcript_spliced_length: false,
        compute_transcript_cds_length: false,
        dev_allow_auto_generate_fai: false,
        duplicate_transcript_id_policy: DuplicateTranscriptIdPolicy::Reject,
        transcript_id_policy: TranscriptIdPolicy::default(),
        unknown_feature_policy: UnknownFeaturePolicy::IgnoreWithWarning,
        feature_id_uniqueness_policy: FeatureIdUniquenessPolicy::Reject,
        reject_normalized_seqid_collisions: true,
    }
}

fn bench_sqlite_size_growth(c: &mut Criterion) {
    c.bench_function("sqlite_size_growth_tiny_vs_realistic", |b| {
        b.iter(|| {
            let tiny_root = tempdir().expect("tiny tempdir");
            let tiny = ingest_dataset(&opts_for_fixture(tiny_root.path(), "tests/fixtures/tiny"))
                .expect("tiny ingest");
            let tiny_bytes = std::fs::metadata(&tiny.sqlite_path)
                .expect("tiny metadata")
                .len();

            let real_root = tempdir().expect("real tempdir");
            let realistic = ingest_dataset(&opts_for_fixture(
                real_root.path(),
                "tests/fixtures/realistic",
            ))
            .expect("real ingest");
            let realistic_bytes = std::fs::metadata(&realistic.sqlite_path)
                .expect("real metadata")
                .len();

            black_box((tiny_bytes, realistic_bytes));
            assert!(realistic_bytes > tiny_bytes);
        })
    });
}

criterion_group!(benches, bench_sqlite_size_growth);
criterion_main!(benches);
