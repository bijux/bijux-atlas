// SPDX-License-Identifier: Apache-2.0

use bijux_dev_atlas::reference::governance_enforcement::{evaluate_registry, load_registry};
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use std::path::Path;

fn governance_benchmarks(c: &mut Criterion) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let registry = load_registry(&root).expect("load governance registry");

    c.bench_function("governance_registry_load", |b| {
        b.iter(|| {
            let _ = black_box(load_registry(black_box(&root)).expect("load governance registry"));
        })
    });

    c.bench_function("governance_registry_evaluate", |b| {
        b.iter(|| {
            let _ = black_box(evaluate_registry(black_box(&root), black_box(&registry)));
        })
    });
}

criterion_group!(governance, governance_benchmarks);
criterion_main!(governance);
