use bijux_atlas_ingest::{ingest_dataset, IngestOptions};
use bijux_atlas_model::{
    BiotypePolicy, DatasetId, DuplicateGeneIdPolicy, GeneIdentifierPolicy, GeneNamePolicy,
    SeqidNormalizationPolicy, StrictnessMode, TranscriptTypePolicy,
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
                compute_gene_signatures: true,
            };
            ingest_dataset(&opts).expect("ingest benchmark");
        })
    });
}

criterion_group!(benches, bench_ingest_throughput);
criterion_main!(benches);
