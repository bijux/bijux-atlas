// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::domain::dataset::{DatasetId, ShardingPlan};
use bijux_atlas::domain::ingest::{ingest_dataset, IngestOptions};
use bijux_atlas::domain::policy::StrictnessMode;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::collections::BTreeMap;
use tempfile::tempdir;

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn bench_ingest_shard_distribution(c: &mut Criterion) {
    c.bench_function("ingest_shard_distribution", |b| {
        b.iter(|| {
            let out = tempdir().expect("tmp");
            let dataset = DatasetId::new("221", "homo_sapiens", "GRCh38").expect("dataset");
            let mut options = IngestOptions::for_dataset(dataset);
            options.gff3_path = fixture("tests/fixtures/realistic/genes.gff3");
            options.fasta_path = fixture("tests/fixtures/realistic/genome.fa");
            options.fai_path = fixture("tests/fixtures/realistic/genome.fa.fai");
            options.output_root = out.path().to_path_buf();
            options.strictness = StrictnessMode::Lenient;
            options.emit_shards = true;
            options.sharding_plan = ShardingPlan::Contig;
            options.shard_partitions = 8;
            let result = ingest_dataset(&options).expect("ingest shard distribution");
            let catalog = result.shard_catalog.expect("shard catalog");
            let mut by_contig = BTreeMap::<String, usize>::new();
            for shard in &catalog.shards {
                for seqid in &shard.seqids {
                    *by_contig.entry(seqid.as_str().to_string()).or_insert(0) += 1;
                }
            }
            black_box(by_contig);
        })
    });
}

criterion_group!(benches, bench_ingest_shard_distribution);
criterion_main!(benches);
