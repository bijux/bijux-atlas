// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn load_suite_manifest_includes_system_load_profiles() {
    let path = repo_root().join("ops/load/suites/suites.json");
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(path).expect("read suites"))
            .expect("parse suites");

    let suites = value["suites"].as_array().expect("suites array");
    let names: std::collections::BTreeSet<&str> = suites
        .iter()
        .filter_map(|v| v["name"].as_str())
        .collect();

    for suite in [
        "mixed-workload",
        "ingest-query-workload",
        "heavy-query-workload",
        "read-heavy-workload",
        "write-heavy-workload",
        "long-running-stability",
        "memory-leak-detection",
        "cpu-saturation",
        "disk-io-saturation",
        "thread-pool-exhaustion",
    ] {
        assert!(names.contains(suite), "missing load suite `{suite}`");
    }
}
