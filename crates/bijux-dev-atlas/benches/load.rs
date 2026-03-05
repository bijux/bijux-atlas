// SPDX-License-Identifier: Apache-2.0

use bijux_dev_atlas::core::load_harness::{
    concurrency_stress_scenarios, mixed_workload_generator, query_load_generator,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn load_harness_benchmarks(c: &mut Criterion) {
    c.bench_function("load_query_generator", |b| {
        b.iter(|| {
            let spec = black_box(query_load_generator(300));
            black_box(spec.target_rps);
        })
    });

    c.bench_function("load_mixed_generator", |b| {
        b.iter(|| {
            let spec = black_box(mixed_workload_generator(300));
            black_box(spec.query_mix_read_ratio);
        })
    });

    c.bench_function("load_concurrency_scenarios", |b| {
        b.iter(|| {
            let rows = black_box(concurrency_stress_scenarios());
            black_box(rows.len());
        })
    });
}

criterion_group!(load, load_harness_benchmarks);
criterion_main!(load);
