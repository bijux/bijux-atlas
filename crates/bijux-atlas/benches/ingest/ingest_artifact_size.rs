// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::domain::dataset::DatasetId;
use bijux_atlas::domain::ingest::{ingest_dataset, IngestOptions};
use bijux_atlas::domain::policy::StrictnessMode;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tempfile::tempdir;

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn bench_ingest_artifact_size(c: &mut Criterion) {
    c.bench_function("ingest_artifact_size", |b| {
        b.iter(|| {
            let out = tempdir().expect("tmp");
            let dataset = DatasetId::new("222", "homo_sapiens", "GRCh38").expect("dataset");
            let mut options = IngestOptions::for_dataset(dataset);
            options.gff3_path = fixture("tests/fixtures/realistic/genes.gff3");
            options.fasta_path = fixture("tests/fixtures/realistic/genome.fa");
            options.fai_path = fixture("tests/fixtures/realistic/genome.fa.fai");
            options.output_root = out.path().to_path_buf();
            options.strictness = StrictnessMode::Lenient;
            let result = ingest_dataset(&options).expect("ingest artifact size");
            let sqlite_size = std::fs::metadata(&result.sqlite_path)
                .expect("sqlite metadata")
                .len();
            let manifest_size = std::fs::metadata(&result.manifest_path)
                .expect("manifest metadata")
                .len();
            let anomaly_size = std::fs::metadata(&result.anomaly_report_path)
                .expect("anomaly metadata")
                .len();
            black_box((sqlite_size, manifest_size, anomaly_size));
        })
    });
}

criterion_group!(benches, bench_ingest_artifact_size);
criterion_main!(benches);
