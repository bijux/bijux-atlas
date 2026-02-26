// SPDX-License-Identifier: Apache-2.0

use bijux_dev_atlas::adapters::AdapterError;
use bijux_dev_atlas::core::ops_inventory::load_ops_inventory_cached;
use bijux_dev_atlas::core::{
    run_checks, Capabilities, Fs, ProcessRunner, RunOptions, RunRequest, Selectors,
};
use criterion::{criterion_group, criterion_main, Criterion};
use std::path::{Path, PathBuf};
use std::{fs, io};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

struct BenchFs;
impl Fs for BenchFs {
    fn read_text(&self, repo_root: &Path, path: &Path) -> Result<String, AdapterError> {
        let target = if path.is_absolute() {
            path.to_path_buf()
        } else {
            repo_root.join(path)
        };
        fs::read_to_string(target).map_err(|err| AdapterError::Io {
            op: "read_to_string",
            path: repo_root.join(path),
            detail: err.to_string(),
        })
    }
    fn exists(&self, repo_root: &Path, path: &Path) -> bool {
        let target = if path.is_absolute() {
            path.to_path_buf()
        } else {
            repo_root.join(path)
        };
        target.exists()
    }
    fn canonicalize(&self, repo_root: &Path, path: &Path) -> Result<PathBuf, AdapterError> {
        let target = if path.is_absolute() {
            path.to_path_buf()
        } else {
            repo_root.join(path)
        };
        target.canonicalize().map_err(|err| AdapterError::Io {
            op: "canonicalize",
            path: target,
            detail: err.to_string(),
        })
    }
}

struct DeniedProcessRunner;
impl ProcessRunner for DeniedProcessRunner {
    fn run(
        &self,
        _program: &str,
        _args: &[String],
        _repo_root: &Path,
    ) -> Result<i32, AdapterError> {
        Err(AdapterError::EffectDenied {
            effect: "subprocess",
            detail: io::Error::other("disabled in bench").to_string(),
        })
    }
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
                &BenchFs,
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
