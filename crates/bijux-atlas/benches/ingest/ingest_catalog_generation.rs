// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::domain::dataset::{DatasetId, ShardingPlan};
use bijux_atlas::domain::ingest::{ingest_dataset, IngestOptions};
use bijux_atlas::domain::policy::StrictnessMode;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use tempfile::tempdir;

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn bench_ingest_catalog_generation(c: &mut Criterion) {
    c.bench_function("ingest_catalog_generation", |b| {
        b.iter(|| {
            let out = tempdir().expect("tmp");
            let dataset = DatasetId::new("218", "homo_sapiens", "GRCh38").expect("dataset");
            let mut options = IngestOptions::for_dataset(dataset);
            options.gff3_path = fixture("tests/fixtures/realistic/genes.gff3");
            options.fasta_path = fixture("tests/fixtures/realistic/genome.fa");
            options.fai_path = fixture("tests/fixtures/realistic/genome.fa.fai");
            options.output_root = out.path().to_path_buf();
            options.strictness = StrictnessMode::Lenient;
            options.emit_shards = true;
            options.sharding_plan = ShardingPlan::Contig;
            options.shard_partitions = 4;
            let result = ingest_dataset(&options).expect("ingest for catalog generation");
            let catalog_path = result.shard_catalog_path.expect("catalog");
            let catalog_bytes = std::fs::read(catalog_path).expect("catalog bytes");
            black_box(catalog_bytes.len());
        })
    });
}

criterion_group!(benches, bench_ingest_catalog_generation);
criterion_main!(benches);
