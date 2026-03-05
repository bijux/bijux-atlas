// SPDX-License-Identifier: Apache-2.0

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn audit_benchmarks(c: &mut Criterion) {
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "audit_run",
        "status": "ok",
        "checks": [
            {"id": "AUDIT-CONFIG-INTEGRITY-001", "status": "ok"},
            {"id": "AUDIT-ARTIFACT-INTEGRITY-001", "status": "ok"},
            {"id": "AUDIT-REGISTRY-CONSISTENCY-001", "status": "ok"},
            {"id": "AUDIT-RUNTIME-CONFIG-STATE-001", "status": "ok"},
            {"id": "AUDIT-OPS-DEPLOY-INTEGRITY-001", "status": "ok"}
        ],
        "metrics": {"total_checks": 5, "failed_checks": 0, "passed_checks": 5}
    });

    c.bench_function("audit_run_and_report", |b| {
        b.iter(|| {
            let bytes = serde_json::to_vec(black_box(&payload)).expect("encode audit payload");
            black_box(bytes);
        })
    });
}

criterion_group!(audit, audit_benchmarks);
criterion_main!(audit);
