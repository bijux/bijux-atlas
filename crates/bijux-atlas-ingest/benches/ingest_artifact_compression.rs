// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_ingest::{ingest_dataset, IngestOptions};
use bijux_atlas_model::{DatasetId, StrictnessMode};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tempfile::tempdir;

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn bench_ingest_artifact_compression(c: &mut Criterion) {
    c.bench_function("ingest_artifact_compression", |b| {
        b.iter(|| {
            let out = tempdir().expect("tmp");
            let options = IngestOptions {
                gff3_path: fixture("tests/fixtures/realistic/genes.gff3"),
                fasta_path: fixture("tests/fixtures/realistic/genome.fa"),
                fai_path: fixture("tests/fixtures/realistic/genome.fa.fai"),
                output_root: out.path().to_path_buf(),
                dataset: DatasetId::new("223", "homo_sapiens", "GRCh38").expect("dataset"),
                strictness: StrictnessMode::Lenient,
                ..IngestOptions::default()
            };
            let result = ingest_dataset(&options).expect("ingest for compression");
            let sqlite_bytes = std::fs::read(&result.sqlite_path).expect("read sqlite");
            let compressed = zstd::encode_all(&sqlite_bytes[..], 3).expect("compress sqlite bytes");
            black_box((sqlite_bytes.len(), compressed.len()));
            assert!(compressed.len() < sqlite_bytes.len());
        })
    });
}

criterion_group!(benches, bench_ingest_artifact_compression);
criterion_main!(benches);
