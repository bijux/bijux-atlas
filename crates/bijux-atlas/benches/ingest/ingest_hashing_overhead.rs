// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::domain::ingest::compute_input_hashes;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn bench_ingest_hashing_overhead(c: &mut Criterion) {
    let gff3 = fixture("tests/fixtures/realistic/genes.gff3");
    let fasta = fixture("tests/fixtures/realistic/genome.fa");
    let fai = fixture("tests/fixtures/realistic/genome.fa.fai");
    c.bench_function("ingest_hashing_overhead", |b| {
        b.iter(|| {
            let hashes = compute_input_hashes(&gff3, &fasta, &fai).expect("compute hashes");
            black_box(hashes);
        })
    });
}

criterion_group!(benches, bench_ingest_hashing_overhead);
criterion_main!(benches);
