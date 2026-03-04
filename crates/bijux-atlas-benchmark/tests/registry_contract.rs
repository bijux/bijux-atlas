// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_benchmark::config::BenchmarkConfig;
use bijux_atlas_benchmark::dataset::fixture_registry;
use bijux_atlas_benchmark::harness::{BenchmarkMetrics, BenchmarkResult};

#[test]
fn fixture_registry_is_valid() {
    let registry = fixture_registry();
    assert!(registry.validate().is_ok());
}

#[test]
fn benchmark_config_validation_accepts_canonical_units() {
    let config = BenchmarkConfig {
        schema_version: 1,
        namespace: "atlas_benchmark".to_string(),
        latency_unit: "milliseconds".to_string(),
        throughput_unit: "operations_per_second".to_string(),
        isolation: bijux_atlas_benchmark::config::IsolationConfig {
            cpu_set: "0-3".to_string(),
            memory_limit_mb: 2048,
        },
        reproducibility: bijux_atlas_benchmark::config::ReproducibilityConfig {
            fixed_seed: 20260304,
            min_repeat_runs: 3,
            max_relative_delta_percent: 3.0,
        },
    };
    assert!(config.validate().is_ok());
}

#[test]
fn benchmark_result_validation_accepts_positive_metrics() {
    let result = BenchmarkResult {
        schema_version: 1,
        benchmark_id: "query.point_lookup".to_string(),
        dataset_id: "genes-mini".to_string(),
        metrics: BenchmarkMetrics {
            latency_p50_ms: 2.5,
            latency_p95_ms: 8.0,
            latency_p99_ms: 13.2,
            throughput_ops_per_sec: 5200.0,
        },
    };
    assert!(result.validate().is_ok());
}
