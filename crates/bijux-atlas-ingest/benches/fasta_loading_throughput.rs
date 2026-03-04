// SPDX-License-Identifier: Apache-2.0

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::io::BufRead;

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn bench_fasta_loading_throughput(c: &mut Criterion) {
    let fasta_path = fixture("tests/fixtures/realistic/genome.fa");
    c.bench_function("fasta_loading_throughput", |b| {
        b.iter(|| {
            let file = std::fs::File::open(&fasta_path).expect("open fasta");
            let reader = std::io::BufReader::new(file);
            let mut contig_count = 0_usize;
            let mut total_bases = 0_usize;
            for line in reader.lines() {
                let line = line.expect("line");
                if line.starts_with('>') {
                    contig_count += 1;
                } else {
                    total_bases += line.trim().len();
                }
            }
            black_box((contig_count, total_bases));
            assert!(contig_count > 0);
            assert!(total_bases > 0);
        })
    });
}

criterion_group!(benches, bench_fasta_loading_throughput);
criterion_main!(benches);
