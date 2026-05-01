// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::domain::dataset::DatasetId;
use bijux_atlas::domain::ingest::{ingest_dataset, IngestOptions};
use bijux_atlas::domain::policy::StrictnessMode;
use bijux_atlas::domain::query::UnknownFeaturePolicy;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use tempfile::tempdir;

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn bench_ingest_anomaly_detection_overhead(c: &mut Criterion) {
    c.bench_function("ingest_anomaly_detection_overhead", |b| {
        b.iter(|| {
            let out = tempdir().expect("tmp");
            let dataset = DatasetId::new("215", "homo_sapiens", "GRCh38").expect("dataset");
            let mut options = IngestOptions::for_dataset(dataset);
            options.gff3_path = fixture("tests/fixtures/tiny/genes_missing_parent.gff3");
            options.fasta_path = fixture("tests/fixtures/tiny/genome.fa");
            options.fai_path = fixture("tests/fixtures/tiny/genome.fa.fai");
            options.output_root = out.path().to_path_buf();
            options.strictness = StrictnessMode::Lenient;
            options.unknown_feature_policy = UnknownFeaturePolicy::IgnoreWithWarning;
            let result = ingest_dataset(&options).expect("ingest with anomalies");
            black_box(result.anomaly_report.missing_parents.len());
        })
    });
}

criterion_group!(benches, bench_ingest_anomaly_detection_overhead);
criterion_main!(benches);
