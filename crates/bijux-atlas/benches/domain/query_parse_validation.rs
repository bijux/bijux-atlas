// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::domain::query::{GeneId, Region, TranscriptId};
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

fn bench_gene_id_parse(c: &mut Criterion) {
    c.bench_function("gene_id_parse", |b| {
        b.iter(|| GeneId::parse(black_box("ENSG00000123456")).expect("gene id"))
    });
}

fn bench_transcript_id_parse(c: &mut Criterion) {
    c.bench_function("transcript_id_parse", |b| {
        b.iter(|| TranscriptId::parse(black_box("ENST00000123456")).expect("tx id"))
    });
}

fn bench_region_parse(c: &mut Criterion) {
    c.bench_function("region_parse", |b| {
        b.iter(|| Region::parse(black_box("chr1:100-200")).expect("region"))
    });
}

criterion_group!(
    benches,
    bench_gene_id_parse,
    bench_transcript_id_parse,
    bench_region_parse
);
criterion_main!(benches);
