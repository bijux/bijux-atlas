// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_ingest::read_fai_contig_lengths;
use criterion::{criterion_group, criterion_main, Criterion};
use std::path::PathBuf;

fn bench_fai_validation_overhead(c: &mut Criterion) {
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/realistic/genome.fa.fai");
    c.bench_function("fai_validation_overhead", |b| {
        b.iter(|| {
            let map = read_fai_contig_lengths(&fixture).expect("read fai");
            assert!(!map.is_empty());
        })
    });
}

criterion_group!(benches, bench_fai_validation_overhead);
criterion_main!(benches);
