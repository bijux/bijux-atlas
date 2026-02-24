#![allow(missing_docs)]

use bijux_atlas_api::openapi_v1_spec;
use bijux_atlas_core::canonical;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

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
