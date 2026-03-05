// SPDX-License-Identifier: Apache-2.0

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates root")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn security_supply_chain_benchmarks(c: &mut Criterion) {
    let root = workspace_root();
    let dependency_policy_path = root.join("configs/security/dependency-source-policy.json");
    let actions_inventory_path = root.join("artifacts/security/security-github-actions.json");

    c.bench_function("security_dependency_policy_parse", |b| {
        b.iter(|| {
            let raw = fs::read_to_string(black_box(&dependency_policy_path)).expect("read policy");
            let value: serde_json::Value =
                serde_json::from_str(black_box(&raw)).expect("parse policy");
            black_box(value.get("dependency_lock_posture").is_some());
        })
    });

    c.bench_function("security_actions_inventory_parse", |b| {
        b.iter(|| {
            let raw = fs::read_to_string(black_box(&actions_inventory_path))
                .expect("read actions inventory");
            let value: serde_json::Value =
                serde_json::from_str(black_box(&raw)).expect("parse actions inventory");
            let rows = value
                .get("rows")
                .and_then(serde_json::Value::as_array)
                .map_or(0, std::vec::Vec::len);
            black_box(rows);
        })
    });
}

criterion_group!(security_supply_chain, security_supply_chain_benchmarks);
criterion_main!(security_supply_chain);
