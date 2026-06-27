// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn ingest_benchmark_baseline_has_required_scenarios() {
    let path = crate_root().join("tests/goldens/perf/ingest-benchmark-baseline.json");
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(path).expect("read baseline"))
            .expect("parse baseline");

    assert_eq!(value["schema_version"], serde_json::json!(1));
    for scenario in [
        "ingest_small_dataset",
        "ingest_medium_dataset",
        "ingest_large_dataset",
    ] {
        assert!(
            value["scenarios"].get(scenario).is_some(),
            "missing baseline scenario: {scenario}"
        );
    }
}

#[test]
fn ingest_benchmark_golden_enables_required_overhead_checks() {
    let path = crate_root().join("tests/goldens/perf/ingest-benchmark-golden.json");
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(path).expect("read golden"))
            .expect("parse golden");

    assert_eq!(value["schema_version"], serde_json::json!(1));
    for check in [
        "anomaly_overhead_enabled",
        "validation_overhead_enabled",
        "hashing_overhead_enabled",
        "shard_generation_overhead_enabled",
        "catalog_generation_enabled",
        "parallelism_scaling_enabled",
        "thread_pool_scaling_enabled",
        "shard_distribution_enabled",
        "artifact_size_enabled",
        "artifact_compression_enabled",
    ] {
        assert_eq!(value["checks"][check], serde_json::json!(true));
    }
}
