// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::domain::dataset::{DatasetId, ShardingPlan};
use bijux_atlas::domain::ingest::{ingest_dataset, IngestOptions};
use bijux_atlas::domain::policy::StrictnessMode;
use criterion::{criterion_group, criterion_main, Criterion};
use tempfile::tempdir;

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn base_options(out: &std::path::Path) -> IngestOptions {
    let dataset = DatasetId::new("211", "homo_sapiens", "GRCh38").expect("dataset");
    let mut options = IngestOptions::for_dataset(dataset);
    options.gff3_path = fixture("tests/fixtures/realistic/genes.gff3");
    options.fasta_path = fixture("tests/fixtures/realistic/genome.fa");
    options.fai_path = fixture("tests/fixtures/realistic/genome.fa.fai");
    options.output_root = out.to_path_buf();
    options.strictness = StrictnessMode::Lenient;
    options.max_threads = 1;
    options
}

fn bench_ingest_small_dataset(c: &mut Criterion) {
    c.bench_function("ingest_small_dataset", |b| {
        b.iter(|| {
            let out = tempdir().expect("tmp");
            let mut options = base_options(out.path());
            options.gff3_path = fixture("tests/fixtures/tiny/genes.gff3");
            options.fasta_path = fixture("tests/fixtures/tiny/genome.fa");
            options.fai_path = fixture("tests/fixtures/tiny/genome.fa.fai");
            ingest_dataset(&options).expect("ingest tiny");
        })
    });
}

fn bench_ingest_medium_dataset(c: &mut Criterion) {
    c.bench_function("ingest_medium_dataset", |b| {
        b.iter(|| {
            let out = tempdir().expect("tmp");
            let mut options = base_options(out.path());
            options.gff3_path = fixture("tests/fixtures/realistic/genes.gff3");
            options.fasta_path = fixture("tests/fixtures/realistic/genome.fa");
            options.fai_path = fixture("tests/fixtures/realistic/genome.fa.fai");
            ingest_dataset(&options).expect("ingest realistic");
        })
    });
}

fn bench_ingest_large_dataset(c: &mut Criterion) {
    c.bench_function("ingest_large_dataset", |b| {
        b.iter(|| {
            let out = tempdir().expect("tmp");
            let mut options = base_options(out.path());
            options.strictness = StrictnessMode::Strict;
            ingest_dataset(&options).expect("ingest large profile");
        })
    });
}

fn bench_ingest_sharded_dataset(c: &mut Criterion) {
    c.bench_function("ingest_sharded_dataset", |b| {
        b.iter(|| {
            let out = tempdir().expect("tmp");
            let mut options = base_options(out.path());
            options.emit_shards = true;
            options.sharding_plan = ShardingPlan::Contig;
            options.shard_partitions = 4;
            ingest_dataset(&options).expect("ingest sharded");
        })
    });
}

fn bench_ingest_concurrent_ingestion(c: &mut Criterion) {
    c.bench_function("ingest_concurrent_ingestion", |b| {
        b.iter(|| {
            std::thread::scope(|scope| {
                let mut handles = Vec::new();
                for index in 0..2 {
                    handles.push(scope.spawn(move || {
                        let out = tempdir().expect("tmp");
                        let mut options = base_options(out.path());
                        options.dataset =
                            DatasetId::new("211", "homo_sapiens", &format!("GRCh3{}", 8 + index))
                                .expect("dataset");
                        ingest_dataset(&options).expect("ingest concurrent");
                    }));
                }
                for handle in handles {
                    handle.join().expect("join");
                }
            });
        })
    });
}

criterion_group!(
    benches,
    bench_ingest_small_dataset,
    bench_ingest_medium_dataset,
    bench_ingest_large_dataset,
    bench_ingest_sharded_dataset,
    bench_ingest_concurrent_ingestion,
);
criterion_main!(benches);
