use bijux_atlas_policies::{
    validate_policy_config, CacheBudget, ConcurrencyBulkheads, PolicyConfig, QueryBudget,
    RateLimitPolicy, TelemetryPolicy,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_policy_validate(c: &mut Criterion) {
    let cfg = PolicyConfig {
        schema_version: "1".to_string(),
        allow_override: false,
        network_in_unit_tests: false,
        query_budget: QueryBudget {
            max_limit: 100,
            max_region_span: 10_000_000,
            max_prefix_length: 128,
        },
        cache_budget: CacheBudget {
            max_disk_bytes: 8 * 1024 * 1024 * 1024,
            max_dataset_count: 8,
            pinned_datasets_max: 32,
        },
        rate_limit: RateLimitPolicy {
            per_ip_rps: 100,
            per_api_key_rps: 500,
        },
        concurrency_bulkheads: ConcurrencyBulkheads {
            cheap: 128,
            medium: 64,
            heavy: 16,
        },
        telemetry: TelemetryPolicy {
            metrics_enabled: true,
            tracing_enabled: true,
            slow_query_log_enabled: true,
            request_id_required: true,
        },
        documented_defaults: vec![],
    };

    c.bench_function("validate_policy_config", |b| {
        b.iter(|| validate_policy_config(black_box(&cfg)).expect("valid policy"))
    });
}

criterion_group!(benches, bench_policy_validate);
criterion_main!(benches);
