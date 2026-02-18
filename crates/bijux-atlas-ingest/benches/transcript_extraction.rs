use bijux_atlas_ingest::ingest_dataset;
use bijux_atlas_ingest::IngestOptions;
use bijux_atlas_model::{
    BiotypePolicy, DatasetId, DuplicateGeneIdPolicy, DuplicateTranscriptIdPolicy,
    FeatureIdUniquenessPolicy, GeneIdentifierPolicy, GeneNamePolicy, SeqidNormalizationPolicy,
    StrictnessMode, TranscriptIdPolicy, TranscriptTypePolicy, UnknownFeaturePolicy,
};
use criterion::{criterion_group, criterion_main, Criterion};
use tempfile::tempdir;

fn make_options(root: &std::path::Path) -> IngestOptions {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tiny");
    IngestOptions {
        gff3_path: fixture.join("genes.gff3"),
        fasta_path: fixture.join("genome.fa"),
        fai_path: fixture.join("genome.fa.fai"),
        output_root: root.to_path_buf(),
        dataset: DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset id"),
        strictness: StrictnessMode::Strict,
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

fn bench_transcript_extraction(c: &mut Criterion) {
    c.bench_function("ingest_transcript_extraction", |b| {
        b.iter(|| {
            let root = tempdir().expect("tempdir");
            let opts = make_options(root.path());
            let run = ingest_dataset(&opts).expect("ingest");
            assert!(run.manifest.stats.transcript_count > 0);
        })
    });
}

criterion_group!(benches, bench_transcript_extraction);
criterion_main!(benches);
