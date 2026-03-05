// SPDX-License-Identifier: Apache-2.0

use bijux_dev_atlas::contracts::logging_registry::{schema_contract, validate_log_record};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn logging_contract_benchmarks(c: &mut Criterion) {
    c.bench_function("logging_schema_contract_encoding", |b| {
        b.iter(|| {
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "logging_schema_contract",
                "contract": schema_contract(),
            });
            let encoded = serde_json::to_vec(black_box(&payload)).expect("encode logging payload");
            black_box(encoded.len());
        })
    });

    c.bench_function("logging_format_validation", |b| {
        let record = serde_json::json!({
            "timestamp": "2026-03-05T10:00:00Z",
            "level": "INFO",
            "target": "atlas::runtime",
            "message": "request completed",
            "request_id": "req-bench",
            "trace_id": "trace-bench",
            "event_name": "query_execute"
        });
        b.iter(|| {
            let violations = validate_log_record(black_box(&record));
            black_box(violations.len());
        })
    });
}

criterion_group!(logging, logging_contract_benchmarks);
criterion_main!(logging);
