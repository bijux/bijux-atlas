// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_benchmark::config::BenchmarkConfig;
use bijux_atlas_benchmark::dataset::fixture_registry;
use bijux_atlas_benchmark::harness::{
    compare_results, reproducibility_ok, BenchmarkMetrics, BenchmarkResult,
};

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

#[test]
fn benchmark_csv_export_is_stable() {
    let result = BenchmarkResult {
        schema_version: 1,
        benchmark_id: "query.region".to_string(),
        dataset_id: "genes-medium".to_string(),
        metrics: BenchmarkMetrics {
            latency_p50_ms: 5.0,
            latency_p95_ms: 10.0,
            latency_p99_ms: 15.0,
            throughput_ops_per_sec: 1200.0,
        },
    };
    assert_eq!(
        BenchmarkResult::csv_header(),
        "benchmark_id,dataset_id,latency_p50_ms,latency_p95_ms,latency_p99_ms,throughput_ops_per_sec"
    );
    assert_eq!(
        result.to_csv_row(),
        "query.region,genes-medium,5.000,10.000,15.000,1200.000"
    );
}

#[test]
fn benchmark_diff_flags_regression() {
    let baseline = BenchmarkResult {
        schema_version: 1,
        benchmark_id: "query.point_lookup".to_string(),
        dataset_id: "genes-mini".to_string(),
        metrics: BenchmarkMetrics {
            latency_p50_ms: 2.0,
            latency_p95_ms: 6.0,
            latency_p99_ms: 9.0,
            throughput_ops_per_sec: 5000.0,
        },
    };
    let candidate = BenchmarkResult {
        schema_version: 1,
        benchmark_id: "query.point_lookup".to_string(),
        dataset_id: "genes-mini".to_string(),
        metrics: BenchmarkMetrics {
            latency_p50_ms: 2.2,
            latency_p95_ms: 6.4,
            latency_p99_ms: 10.0,
            throughput_ops_per_sec: 4800.0,
        },
    };
    let diff = compare_results(&baseline, &candidate);
    assert!(diff.regressed);
}

#[test]
fn benchmark_reproducibility_window_is_enforced() {
    let samples = vec![
        BenchmarkResult {
            schema_version: 1,
            benchmark_id: "query.prefix".to_string(),
            dataset_id: "genes-mini".to_string(),
            metrics: BenchmarkMetrics {
                latency_p50_ms: 1.5,
                latency_p95_ms: 4.0,
                latency_p99_ms: 7.5,
                throughput_ops_per_sec: 9000.0,
            },
        },
        BenchmarkResult {
            schema_version: 1,
            benchmark_id: "query.prefix".to_string(),
            dataset_id: "genes-mini".to_string(),
            metrics: BenchmarkMetrics {
                latency_p50_ms: 1.6,
                latency_p95_ms: 4.1,
                latency_p99_ms: 7.7,
                throughput_ops_per_sec: 8900.0,
            },
        },
        BenchmarkResult {
            schema_version: 1,
            benchmark_id: "query.prefix".to_string(),
            dataset_id: "genes-mini".to_string(),
            metrics: BenchmarkMetrics {
                latency_p50_ms: 1.5,
                latency_p95_ms: 4.1,
                latency_p99_ms: 7.6,
                throughput_ops_per_sec: 8950.0,
            },
        },
    ];
    assert!(reproducibility_ok(&samples, 3.0));
}
