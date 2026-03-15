// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::domain::ingest::{IngestOptions, ingest_dataset};
use bijux_atlas::domain::dataset::DatasetId;
use bijux_atlas::domain::policy::StrictnessMode;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tempfile::tempdir;

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn run_with_threads(max_threads: usize) -> usize {
    let out = tempdir().expect("tmp");
    let options = IngestOptions {
        gff3_path: fixture("tests/fixtures/realistic/genes.gff3"),
        fasta_path: fixture("tests/fixtures/realistic/genome.fa"),
        fai_path: fixture("tests/fixtures/realistic/genome.fa.fai"),
        output_root: out.path().to_path_buf(),
        dataset: DatasetId::new("219", "homo_sapiens", "GRCh38").expect("dataset"),
        strictness: StrictnessMode::Lenient,
        max_threads,
        ..IngestOptions::default()
    };
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
