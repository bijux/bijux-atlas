// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::ingest::{ingest_dataset, IngestOptions};
use bijux_atlas::model::{DatasetId, StrictnessMode};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tempfile::tempdir;

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn bench_manifest_generation_latency(c: &mut Criterion) {
    c.bench_function("manifest_generation_latency", |b| {
        b.iter(|| {
            let out = tempdir().expect("tmp");
            let options = IngestOptions {
                gff3_path: fixture("tests/fixtures/tiny/genes.gff3"),
                fasta_path: fixture("tests/fixtures/tiny/genome.fa"),
                fai_path: fixture("tests/fixtures/tiny/genome.fa.fai"),
                output_root: out.path().to_path_buf(),
                dataset: DatasetId::new("213", "homo_sapiens", "GRCh38").expect("dataset"),
                strictness: StrictnessMode::Strict,
                ..IngestOptions::default()
            };
            let result = ingest_dataset(&options).expect("ingest tiny");
            let manifest_size = std::fs::metadata(&result.manifest_path)
                .expect("manifest metadata")
                .len();
            black_box(manifest_size);
            assert!(manifest_size > 0);
        })
    });
}

criterion_group!(benches, bench_manifest_generation_latency);
criterion_main!(benches);
