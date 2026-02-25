// SPDX-License-Identifier: Apache-2.0

use bijux_dev_atlas_policies::{
    evaluate_policy_set_pure, DevAtlasPolicyMode, DevAtlasPolicySet, OpsPolicy,
    PolicyInputSnapshot, PolicySchemaVersion, RepoPolicy,
};
use criterion::{criterion_group, criterion_main, Criterion};

fn worst_case_input(size: usize) -> PolicyInputSnapshot {
    PolicyInputSnapshot {
        rust_file_line_counts: (0..size)
            .map(|i| (format!("crates/x/src/file_{i}.rs"), 5_000))
            .collect(),
        registry_relpath_exists: false,
    }
}

fn sample_policy() -> DevAtlasPolicySet {
    DevAtlasPolicySet {
        schema_version: PolicySchemaVersion::V1,
        mode: DevAtlasPolicyMode::Strict,
        repo_policy: RepoPolicy {
            max_loc_warn: 800,
            max_loc_hard: 1_000,
            max_depth_hard: 7,
            max_rs_files_per_dir_hard: 10,
            max_modules_per_dir_hard: 16,
            loc_allowlist: Vec::new(),
            rs_files_per_dir_allowlist: Vec::new(),
        },
        ops_policy: OpsPolicy {
            registry_relpath: "ops/inventory/registry.toml".to_string(),
        },
        compatibility: Vec::new(),
        documented_defaults: Vec::new(),
        ratchets: Vec::new(),
        relaxations: Vec::new(),
    }
}

fn bench_worst_case_evaluation(c: &mut Criterion) {
    let policy = sample_policy();
    let snapshot = worst_case_input(10_000);
    c.bench_function("policy_eval_worst_case_inventory", |b| {
        b.iter(|| {
            evaluate_policy_set_pure(&policy, &snapshot);
        })
    });
}

criterion_group!(policy_eval, bench_worst_case_evaluation);
criterion_main!(policy_eval);
