// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::domain::dataset::DatasetId;
use bijux_atlas::domain::ingest::{ingest_dataset, IngestOptions};
use bijux_atlas::domain::policy::StrictnessMode;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tempfile::tempdir;

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn bench_ingest_validation_overhead(c: &mut Criterion) {
    c.bench_function("ingest_validation_overhead", |b| {
        b.iter(|| {
            let out = tempdir().expect("tmp");
            let dataset = DatasetId::new("216", "homo_sapiens", "GRCh38").expect("dataset");
            let mut options = IngestOptions::for_dataset(dataset);
            options.gff3_path = fixture("tests/fixtures/realistic/genes.gff3");
            options.fasta_path = fixture("tests/fixtures/realistic/genome.fa");
            options.fai_path = fixture("tests/fixtures/realistic/genome.fa.fai");
            options.output_root = out.path().to_path_buf();
            options.strictness = StrictnessMode::Strict;
            options.fail_on_warn = true;
            let result = ingest_dataset(&options).expect("validated ingest");
            black_box(result.qc_report_path.exists());
        })
    });
}

criterion_group!(benches, bench_ingest_validation_overhead);
criterion_main!(benches);
