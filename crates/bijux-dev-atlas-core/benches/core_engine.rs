use bijux_dev_atlas_adapters::{Capabilities, DeniedProcessRunner, RealFs};
use bijux_dev_atlas_core::ops_inventory::load_ops_inventory_cached;
use bijux_dev_atlas_core::{run_checks, RunOptions, RunRequest, Selectors};
use criterion::{criterion_group, criterion_main, Criterion};
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

fn bench_inventory_scan(c: &mut Criterion) {
    let repo = repo_root();
    c.bench_function("ops_inventory_cached_load", |b| {
        b.iter(|| {
            load_ops_inventory_cached(&repo).expect("load ops inventory");
        })
    });
}

fn bench_check_runner(c: &mut Criterion) {
    let request = RunRequest {
        repo_root: repo_root(),
        domain: None,
        capabilities: Capabilities::deny_all(),
        artifacts_root: None,
        run_id: None,
        command: Some("check run".to_string()),
    };
    let selectors = Selectors::default();
    let options = RunOptions::default();
    c.bench_function("check_runner_default_selection", |b| {
        b.iter(|| {
            run_checks(
                &DeniedProcessRunner,
                &RealFs,
                &request,
                &selectors,
                &options,
            )
            .expect("run checks");
        })
    });
}

criterion_group!(core_engine, bench_inventory_scan, bench_check_runner);
criterion_main!(core_engine);
