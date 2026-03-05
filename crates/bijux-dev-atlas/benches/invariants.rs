// SPDX-License-Identifier: Apache-2.0

use bijux_dev_atlas::contracts::system_invariants::registry;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_invariant_registry_iteration(c: &mut Criterion) {
    c.bench_function("invariants_registry_iteration", |b| {
        b.iter(|| {
            let rows = registry();
            black_box(rows.len())
        });
    });
}

fn bench_invariant_report_payload_encoding(c: &mut Criterion) {
    c.bench_function("invariants_report_payload_encoding", |b| {
        b.iter(|| {
            let rows = registry();
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "system_invariant_report_benchmark",
                "rows": rows
            });
            black_box(serde_json::to_vec(&payload).expect("encode"))
        });
    });
}

criterion_group!(
    invariants,
    bench_invariant_registry_iteration,
    bench_invariant_report_payload_encoding
);
criterion_main!(invariants);
