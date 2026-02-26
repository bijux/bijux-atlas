// SPDX-License-Identifier: Apache-2.0

use bijux_dev_atlas::adapters::sorted_non_empty_lines;
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_sorted_lines(c: &mut Criterion) {
    let mut text = String::new();
    for i in (0..5000).rev() {
        text.push_str(&format!("line_{i}\n"));
    }
    c.bench_function("sorted_non_empty_lines", |b| {
        b.iter(|| {
            sorted_non_empty_lines(&text);
        })
    });
}

criterion_group!(file_walk, bench_sorted_lines);
criterion_main!(file_walk);
