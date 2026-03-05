// SPDX-License-Identifier: Apache-2.0

use bijux_dev_atlas::core::load_harness::{
    concurrency_stress_scenarios, ingest_load_generator, mixed_workload_generator,
    query_load_generator, WorkloadKind,
};
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn load_generators_emit_expected_workload_shapes() {
    let q = query_load_generator(60);
    let i = ingest_load_generator(60);
    let m = mixed_workload_generator(60);
    assert_eq!(q.kind, WorkloadKind::Query);
    assert_eq!(i.kind, WorkloadKind::Ingest);
    assert_eq!(m.kind, WorkloadKind::Mixed);
}

#[test]
fn concurrency_stress_registry_has_expected_scenarios() {
    let rows = concurrency_stress_scenarios();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].id, "load-single-client-baseline");
}

#[test]
fn load_harness_contract_artifacts_are_parseable() {
    for rel in [
        "ops/load/contracts/load-harness-spec.json",
        "ops/load/contracts/performance-regression-thresholds.json",
        "ops/load/generated/performance-baseline-metrics.json",
        "ops/load/generated/concurrency-stress-scenarios.json",
    ] {
        let path = repo_root().join(rel);
        let payload: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(path).expect("read json")).expect("parse");
        assert_eq!(
            payload.get("schema_version").and_then(|v| v.as_u64()),
            Some(1)
        );
    }
}
