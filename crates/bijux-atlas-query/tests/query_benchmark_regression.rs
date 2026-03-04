// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn query_benchmark_baseline_has_required_scenarios() {
    let path = crate_root().join("tests/goldens/perf/query-benchmark-baseline.json");
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(path).expect("read baseline"))
            .expect("parse baseline");

    assert_eq!(value["schema_version"], serde_json::json!(1));
    for scenario in [
        "point_lookup",
        "region_query",
        "prefix_search",
        "projection_query",
        "filter_query",
        "query_concurrency",
        "query_shard_routing",
        "query_distributed_simulation",
        "query_planner_complexity",
        "query_cursor_pagination",
    ] {
        assert!(
            value["scenarios"].get(scenario).is_some(),
            "missing baseline scenario: {scenario}"
        );
    }
}

#[test]
fn query_benchmark_golden_enables_required_checks() {
    let path = crate_root().join("tests/goldens/perf/query-benchmark-golden.json");
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(path).expect("read golden"))
            .expect("parse golden");

    assert_eq!(value["schema_version"], serde_json::json!(1));
    for check in [
        "planner_latency_enabled",
        "ast_normalization_enabled",
        "cursor_generation_enabled",
        "response_serialization_enabled",
        "json_encoding_enabled",
        "cache_hit_enabled",
        "cache_miss_enabled",
        "cache_eviction_enabled",
        "cache_warmup_enabled",
        "index_scan_enabled",
        "covering_index_enabled",
        "region_overlap_enabled",
    ] {
        assert_eq!(value["checks"][check], serde_json::json!(true));
    }
}
