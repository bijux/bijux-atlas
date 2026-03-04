// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_ingest::{ingest_dataset_with_events, IngestOptions};
use bijux_atlas_model::{DatasetId, StrictnessMode};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Instant;
use tempfile::tempdir;

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn current_rss_mb() -> f64 {
    let pid = std::process::id().to_string();
    let output = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &pid])
        .output()
        .expect("sample rss");
    let rss_kb = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<f64>()
        .unwrap_or(0.0);
    rss_kb / 1024.0
}

fn current_cpu_percent() -> f64 {
    let pid = std::process::id().to_string();
    let output = std::process::Command::new("ps")
        .args(["-o", "%cpu=", "-p", &pid])
        .output()
        .expect("sample cpu");
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<f64>()
        .unwrap_or(0.0)
}

fn bench_ingest_resource_tracking(c: &mut Criterion) {
    c.bench_function("ingest_resource_tracking", |b| {
        b.iter(|| {
            let out = tempdir().expect("tmp");
            let options = IngestOptions {
                gff3_path: fixture("tests/fixtures/realistic/genes.gff3"),
                fasta_path: fixture("tests/fixtures/realistic/genome.fa"),
                fai_path: fixture("tests/fixtures/realistic/genome.fa.fai"),
                output_root: out.path().to_path_buf(),
                dataset: DatasetId::new("214", "homo_sapiens", "GRCh38").expect("dataset"),
                strictness: StrictnessMode::Lenient,
                ..IngestOptions::default()
            };

            let read_started = Instant::now();
            let gff_bytes = std::fs::read(&options.gff3_path).expect("read gff3 fixture");
            let file_read_latency_ms = read_started.elapsed().as_secs_f64() * 1000.0;

            let rss_before = current_rss_mb();
            let cpu_before = current_cpu_percent();
            let ingest_started = Instant::now();
            let (result, events) = ingest_dataset_with_events(&options).expect("ingest run");
            let artifact_generation_latency_ms = ingest_started.elapsed().as_secs_f64() * 1000.0;
            let rss_after = current_rss_mb();
            let cpu_after = current_cpu_percent();
            let io_throughput_bytes_per_sec =
                (gff_bytes.len() as f64) / (artifact_generation_latency_ms / 1000.0).max(0.001);

            black_box((
                file_read_latency_ms,
                artifact_generation_latency_ms,
                io_throughput_bytes_per_sec,
                rss_before.max(rss_after),
                cpu_before.max(cpu_after),
                events.len(),
            ));
            assert!(result.sqlite_path.exists());
            assert!(!events.is_empty());
        })
    });
}

criterion_group!(benches, bench_ingest_resource_tracking);
criterion_main!(benches);
