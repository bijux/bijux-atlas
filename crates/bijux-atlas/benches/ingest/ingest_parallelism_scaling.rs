// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::domain::dataset::DatasetId;
use bijux_atlas::domain::ingest::{ingest_dataset, IngestOptions};
use bijux_atlas::domain::policy::StrictnessMode;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use tempfile::tempdir;

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn run_with_threads(max_threads: usize) -> usize {
    let out = tempdir().expect("tmp");
    let dataset = DatasetId::new("219", "homo_sapiens", "GRCh38").expect("dataset");
    let mut options = IngestOptions::for_dataset(dataset);
    options.gff3_path = fixture("tests/fixtures/realistic/genes.gff3");
    options.fasta_path = fixture("tests/fixtures/realistic/genome.fa");
    options.fai_path = fixture("tests/fixtures/realistic/genome.fa.fai");
    options.output_root = out.path().to_path_buf();
    options.strictness = StrictnessMode::Lenient;
    options.max_threads = max_threads;
    let result = ingest_dataset(&options).expect("ingest with scaling thread count");
    result.events.len()
}

fn bench_ingest_parallelism_scaling(c: &mut Criterion) {
    c.bench_function("ingest_parallelism_scaling", |b| {
        b.iter(|| {
            let one = run_with_threads(1);
            let two = run_with_threads(2);
            let four = run_with_threads(4);
            black_box((one, two, four));
        })
    });
}

criterion_group!(benches, bench_ingest_parallelism_scaling);
criterion_main!(benches);
