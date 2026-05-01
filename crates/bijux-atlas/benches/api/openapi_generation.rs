// SPDX-License-Identifier: Apache-2.0

#![allow(missing_docs)]

use bijux_atlas::contracts::api::openapi_v1_spec;
use bijux_atlas::domain::canonical;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

fn bench_openapi_generation(c: &mut Criterion) {
    c.bench_function("openapi_generate_stable_bytes", |b| {
        b.iter(|| {
            let spec = openapi_v1_spec();
            black_box(canonical::stable_json_bytes(&spec).expect("stable json"));
        });
    });
}

criterion_group!(benches, bench_openapi_generation);
criterion_main!(benches);
