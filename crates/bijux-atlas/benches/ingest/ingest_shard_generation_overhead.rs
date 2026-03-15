// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::domain::ingest::{IngestOptions, ingest_dataset};
use bijux_atlas::domain::dataset::{DatasetId, ShardingPlan};
use bijux_atlas::domain::policy::StrictnessMode;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Instant;
use tempfile::tempdir;

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn bench_ingest_shard_generation_overhead(c: &mut Criterion) {
    c.bench_function("ingest_shard_generation_overhead", |b| {
        b.iter(|| {
            let out = tempdir().expect("tmp");
            let mut options = IngestOptions {
                gff3_path: fixture("tests/fixtures/realistic/genes.gff3"),
                fasta_path: fixture("tests/fixtures/realistic/genome.fa"),
                fai_path: fixture("tests/fixtures/realistic/genome.fa.fai"),
                output_root: out.path().to_path_buf(),
                dataset: DatasetId::new("217", "homo_sapiens", "GRCh38").expect("dataset"),
                strictness: StrictnessMode::Lenient,
                ..IngestOptions::default()
            };
            options.emit_shards = true;
            options.sharding_plan = ShardingPlan::Contig;
            options.shard_partitions = 8;
            let started = Instant::now();
            let result = ingest_dataset(&options).expect("ingest shard generation");
            let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
            black_box((elapsed_ms, result.shard_catalog_path.is_some()));
        })
    });
}

criterion_group!(benches, bench_ingest_shard_generation_overhead);
criterion_main!(benches);
