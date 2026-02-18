use bijux_atlas_policies::{
    validate_policy_config, CacheBudget, ConcurrencyBulkheads, DocumentedDefault,
    EndpointClassBudget, PolicyConfig, PolicyMode, PolicyModeProfile, PolicyModes,
    PolicySchemaVersion, PublishGates, QueryBudgetPolicy, RateLimitPolicy, ResponseBudgetPolicy,
    StoreResiliencePolicy, TelemetryPolicy,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_policy_validate(c: &mut Criterion) {
    let cfg = PolicyConfig {
        schema_version: PolicySchemaVersion::V1,
        mode: PolicyMode::Strict,
        allow_override: false,
        network_in_unit_tests: false,
        modes: PolicyModes {
            strict: PolicyModeProfile {
                allow_override: false,
                max_page_size: 100,
                max_region_span: 10_000_000,
                max_response_bytes: 1_048_576,
            },
            compat: PolicyModeProfile {
                allow_override: true,
                max_page_size: 200,
                max_region_span: 25_000_000,
                max_response_bytes: 2_097_152,
            },
            dev: PolicyModeProfile {
                allow_override: true,
                max_page_size: 500,
                max_region_span: 50_000_000,
                max_response_bytes: 4_194_304,
            },
        },
        query_budget: QueryBudgetPolicy {
            cheap: EndpointClassBudget {
                max_limit: 100,
                max_region_span: 1_000_000,
                max_region_estimated_rows: 10_000,
                max_prefix_cost_units: 25_000,
            },
            medium: EndpointClassBudget {
                max_limit: 100,
                max_region_span: 5_000_000,
                max_region_estimated_rows: 50_000,
                max_prefix_cost_units: 80_000,
            },
            heavy: EndpointClassBudget {
                max_limit: 200,
                max_region_span: 10_000_000,
                max_region_estimated_rows: 100_000,
                max_prefix_cost_units: 120_000,
            },
            max_limit: 100,
            max_transcript_limit: 100,
            heavy_projection_limit: 200,
            max_prefix_length: 128,
            max_sequence_bases: 20_000,
            sequence_api_key_required_bases: 5_000,
        },
        response_budget: ResponseBudgetPolicy {
            cheap_max_bytes: 262_144,
            medium_max_bytes: 524_288,
            heavy_max_bytes: 1_048_576,
            max_serialization_bytes: 524_288,
        },
        cache_budget: CacheBudget {
            max_disk_bytes: 8 * 1024 * 1024 * 1024,
            max_dataset_count: 8,
            pinned_datasets_max: 32,
            shard_count_policy_max: 512,
            max_open_shards_per_pod: 16,
        },
        store_resilience: StoreResiliencePolicy {
            retry_budget: 20,
            retry_attempts: 4,
            retry_base_backoff_ms: 120,
            breaker_failure_threshold: 5,
            breaker_open_ms: 20_000,
        },
        rate_limit: RateLimitPolicy {
            per_ip_rps: 100,
            per_api_key_rps: 500,
            sequence_per_ip_rps: 15,
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
            required_metric_labels: vec![
                "subsystem".to_string(),
                "version".to_string(),
                "dataset".to_string(),
            ],
            trace_sampling_per_10k: 100,
        },
        publish_gates: PublishGates {
            required_indexes: vec!["idx_gene_summary_gene_id".to_string()],
            min_gene_count: 1,
            max_missing_parents: 1000,
        },
        documented_defaults: vec![DocumentedDefault {
            field: "query_budget.max_limit".to_string(),
            reason: "default page-size guard".to_string(),
        }],
    };

    c.bench_function("validate_policy_config", |b| {
        b.iter(|| validate_policy_config(black_box(&cfg)).expect("valid policy"))
    });
}

criterion_group!(benches, bench_policy_validate);
criterion_main!(benches);
