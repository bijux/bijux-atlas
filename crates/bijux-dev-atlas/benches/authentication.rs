// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::domain::security::runtime::load_security_config_from_path;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
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

fn authentication_benchmarks(c: &mut Criterion) {
    let root = workspace_root();
    let auth_model_path = root.join("configs/sources/security/auth-model.yaml");
    let runtime_security_path = root.join("configs/sources/security/runtime-security.yaml");

    c.bench_function("security_auth_model_parse", |b| {
        b.iter(|| {
            let raw = fs::read_to_string(black_box(&auth_model_path)).expect("read auth model");
            let value: serde_yaml::Value =
                serde_yaml::from_str(black_box(&raw)).expect("parse auth model");
            let methods = value
                .get("supported_methods")
                .and_then(serde_yaml::Value::as_sequence)
                .map_or(0, std::vec::Vec::len);
            black_box(methods);
        })
    });

    c.bench_function("security_runtime_auth_config_parse", |b| {
        b.iter(|| {
            let config = load_security_config_from_path(black_box(runtime_security_path.as_path()))
                .expect("load runtime security config");
            black_box(config.auth.mode);
        })
    });
}

criterion_group!(authentication, authentication_benchmarks);
criterion_main!(authentication);
