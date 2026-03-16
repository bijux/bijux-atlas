// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::domain::dataset::DatasetId;
use bijux_atlas::domain::ingest::{ingest_dataset, IngestOptions};
use bijux_atlas::domain::policy::StrictnessMode;
use criterion::{criterion_group, criterion_main, Criterion};
use tempfile::tempdir;

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn bench_large_ingest_scenario(c: &mut Criterion) {
    c.bench_function("ingest_large_realistic_fixture", |b| {
        b.iter(|| {
            let out = tempdir().expect("tmp");
            let dataset = DatasetId::new("210", "homo_sapiens", "GRCh38").expect("dataset");
            let mut opts = IngestOptions::for_dataset(dataset);
            opts.gff3_path = fixture("tests/fixtures/realistic/genes.gff3");
            opts.fasta_path = fixture("tests/fixtures/realistic/genome.fa");
            opts.fai_path = fixture("tests/fixtures/realistic/genome.fa.fai");
            opts.output_root = out.path().to_path_buf();
            opts.strictness = StrictnessMode::Strict;
            opts.max_threads = 1;
            let _ = ingest_dataset(&opts).expect("ingest realistic");
        })
    });
}

criterion_group!(benches, bench_large_ingest_scenario);
criterion_main!(benches);
