// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::domain::dataset::DatasetId;
use bijux_atlas::domain::ingest::{ingest_dataset, IngestOptions};
use bijux_atlas::domain::policy::StrictnessMode;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tempfile::tempdir;

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn bench_sqlite_write_performance(c: &mut Criterion) {
    c.bench_function("sqlite_write_performance", |b| {
        b.iter(|| {
            let out = tempdir().expect("tmp");
            let dataset = DatasetId::new("212", "homo_sapiens", "GRCh38").expect("dataset");
            let mut options = IngestOptions::for_dataset(dataset);
            options.gff3_path = fixture("tests/fixtures/tiny/genes.gff3");
            options.fasta_path = fixture("tests/fixtures/tiny/genome.fa");
            options.fai_path = fixture("tests/fixtures/tiny/genome.fa.fai");
            options.output_root = out.path().to_path_buf();
            options.strictness = StrictnessMode::Lenient;
            let result = ingest_dataset(&options).expect("ingest tiny");
            let sqlite_bytes = std::fs::metadata(&result.sqlite_path)
                .expect("sqlite metadata")
                .len();
            black_box(sqlite_bytes);
            assert!(sqlite_bytes > 0);
        })
    });
}

criterion_group!(benches, bench_sqlite_write_performance);
criterion_main!(benches);
