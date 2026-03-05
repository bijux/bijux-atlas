// SPDX-License-Identifier: Apache-2.0

use bijux_dev_atlas::contracts::tracing_registry::{span_registry, tracing_contract};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn tracing_registry_benchmarks(c: &mut Criterion) {
    c.bench_function("tracing_span_registry_iteration", |b| {
        b.iter(|| {
            let rows = black_box(span_registry());
            black_box(rows.len());
        })
    });

    c.bench_function("tracing_contract_payload_encoding", |b| {
        b.iter(|| {
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "tracing_contract_snapshot",
                "contract": tracing_contract()
            });
            let encoded = serde_json::to_vec(black_box(&payload)).expect("encode tracing payload");
            black_box(encoded.len());
        })
    });
}

criterion_group!(tracing, tracing_registry_benchmarks);
criterion_main!(tracing);
