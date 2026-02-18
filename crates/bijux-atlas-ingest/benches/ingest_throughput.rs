use bijux_atlas_ingest::{ingest_dataset, IngestOptions};
use bijux_atlas_model::{
    BiotypePolicy, DatasetId, DuplicateGeneIdPolicy, DuplicateTranscriptIdPolicy,
    FeatureIdUniquenessPolicy, GeneIdentifierPolicy, GeneNamePolicy, SeqidNormalizationPolicy,
    ShardingPlan, StrictnessMode, TranscriptIdPolicy, TranscriptTypePolicy, UnknownFeaturePolicy,
};
use criterion::{criterion_group, criterion_main, Criterion};
use std::path::PathBuf;
use tempfile::tempdir;

fn bench_ingest_throughput(c: &mut Criterion) {
    let realistic = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/realistic");
    c.bench_function("ingest_realistic_fixture", |b| {
        b.iter(|| {
            let out = tempdir().expect("tempdir");
            let opts = IngestOptions {
                gff3_path: realistic.join("genes.gff3"),
                fasta_path: realistic.join("genome.fa"),
                fai_path: realistic.join("genome.fa.fai"),
                output_root: out.path().to_path_buf(),
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
                sharding_plan: ShardingPlan::None,
                max_shards: 512,
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
                emit_normalized_debug: false,
                normalized_replay_mode: false,
                prod_mode: false,
            };
            ingest_dataset(&opts).expect("ingest benchmark");
        })
    });
}

criterion_group!(benches, bench_ingest_throughput);
criterion_main!(benches);
