// SPDX-License-Identifier: Apache-2.0

use bijux_dev_atlas::contracts::metrics_registry::registry;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn metrics_registry_benchmarks(c: &mut Criterion) {
    c.bench_function("metrics_registry_iteration", |b| {
        b.iter(|| {
            let rows = black_box(registry());
            black_box(rows.len());
        })
    });

    c.bench_function("metrics_registry_payload_encoding", |b| {
        b.iter(|| {
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "metrics_registry_snapshot",
                "metrics": registry()
            });
            let encoded = serde_json::to_vec(black_box(&payload)).expect("encode metrics payload");
            black_box(encoded.len());
        })
    });
}

criterion_group!(metrics, metrics_registry_benchmarks);
criterion_main!(metrics);
