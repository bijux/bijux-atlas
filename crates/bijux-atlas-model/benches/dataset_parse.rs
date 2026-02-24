// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_model::DatasetId;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_dataset_parse(c: &mut Criterion) {
    c.bench_function("dataset_id_parse", |b| {
        b.iter(|| {
            DatasetId::new(
                black_box("110"),
                black_box("homo_sapiens"),
                black_box("GRCh38"),
            )
            .expect("dataset parse")
        })
    });
}

criterion_group!(benches, bench_dataset_parse);
criterion_main!(benches);
