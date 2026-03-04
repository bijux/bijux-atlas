// SPDX-License-Identifier: Apache-2.0

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::io::{BufRead, BufReader};

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn count_gff3_rows(path: &std::path::Path) -> usize {
    let file = std::fs::File::open(path).expect("open gff3");
    let reader = BufReader::new(file);
    let mut count = 0_usize;
    for line in reader.lines() {
        let line = line.expect("line");
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        if line.split('\t').count() >= 9 {
            count += 1;
        }
    }
    count
}

fn bench_gff3_parsing_throughput(c: &mut Criterion) {
    let gff3_path = fixture("tests/fixtures/realistic/genes.gff3");
    c.bench_function("gff3_parsing_throughput", |b| {
        b.iter(|| {
            let rows = count_gff3_rows(&gff3_path);
            black_box(rows);
            assert!(rows > 0);
        })
    });
}

criterion_group!(benches, bench_gff3_parsing_throughput);
criterion_main!(benches);
