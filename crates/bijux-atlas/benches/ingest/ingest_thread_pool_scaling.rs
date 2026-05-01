// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::domain::dataset::DatasetId;
use bijux_atlas::domain::ingest::{ingest_dataset_with_events, IngestOptions};
use bijux_atlas::domain::policy::StrictnessMode;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use tempfile::tempdir;

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn bench_ingest_thread_pool_scaling(c: &mut Criterion) {
    c.bench_function("ingest_thread_pool_scaling", |b| {
        b.iter(|| {
            for threads in [1_usize, 2, 4] {
                let out = tempdir().expect("tmp");
                let dataset = DatasetId::new("220", "homo_sapiens", "GRCh38").expect("dataset");
                let mut options = IngestOptions::for_dataset(dataset);
                options.gff3_path = fixture("tests/fixtures/realistic/genes.gff3");
                options.fasta_path = fixture("tests/fixtures/realistic/genome.fa");
                options.fai_path = fixture("tests/fixtures/realistic/genome.fa.fai");
                options.output_root = out.path().to_path_buf();
                options.strictness = StrictnessMode::Lenient;
                options.max_threads = threads;
                let (_result, events) =
                    ingest_dataset_with_events(&options).expect("ingest with thread pool");
                black_box((threads, events.len()));
            }
        })
    });
}

criterion_group!(benches, bench_ingest_thread_pool_scaling);
criterion_main!(benches);
