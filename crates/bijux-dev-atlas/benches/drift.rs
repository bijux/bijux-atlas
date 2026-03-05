// SPDX-License-Identifier: Apache-2.0

use bijux_dev_atlas::contracts::drift::explain_drift_type;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_drift_explain_lookup(c: &mut Criterion) {
    c.bench_function("drift_explain_lookup", |b| {
        b.iter(|| {
            let out = explain_drift_type("registry");
            black_box(out.is_some())
        });
    });
}

criterion_group!(drift, bench_drift_explain_lookup);
criterion_main!(drift);
