use bijux_atlas_policies::{
    evaluate_policy_set, evaluate_repository_metrics, load_policy_set_from_workspace,
    RepositoryMetrics,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn bench_policy_evaluation(c: &mut Criterion) {
    let policy = load_policy_set_from_workspace(&workspace_root()).expect("load policy");

    let small = RepositoryMetrics {
        dataset_count: 2,
        open_shards_per_pod: 4,
        disk_bytes: 128 * 1024 * 1024,
    };
    let medium = RepositoryMetrics {
        dataset_count: 8,
        open_shards_per_pod: 16,
        disk_bytes: 8 * 1024 * 1024 * 1024,
    };
    let large = RepositoryMetrics {
        dataset_count: 64,
        open_shards_per_pod: 64,
        disk_bytes: 512 * 1024 * 1024 * 1024,
    };

    c.bench_function("policy_set_evaluate", |b| {
        b.iter(|| evaluate_policy_set(black_box(&policy)))
    });

    c.bench_function("policy_repo_metrics_small", |b| {
        b.iter(|| evaluate_repository_metrics(black_box(&policy), black_box(&small)))
    });

    c.bench_function("policy_repo_metrics_medium", |b| {
        b.iter(|| evaluate_repository_metrics(black_box(&policy), black_box(&medium)))
    });

    c.bench_function("policy_repo_metrics_large", |b| {
        b.iter(|| evaluate_repository_metrics(black_box(&policy), black_box(&large)))
    });
}

criterion_group!(benches, bench_policy_evaluation);
criterion_main!(benches);
