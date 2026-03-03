// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use bijux_dev_atlas::domains::{
    docs::runtime::router::command_registry as docs_runtime_registry,
    ops::runtime::router::command_registry as ops_runtime_registry,
    perf::runtime::router::command_registry as perf_runtime_registry,
    release::runtime::router::command_registry as release_runtime_registry,
    security::runtime::router::command_registry as security_runtime_registry,
};

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn assert_entrypoint_exists(entrypoint: &str) {
    let relative = entrypoint.trim_start_matches("crate::").replace("::", "/");
    assert!(
        crate_root().join("src").join(relative).exists(),
        "missing runtime entrypoint {entrypoint}"
    );
}

#[test]
fn domain_runtime_registries_reference_existing_entrypoints() {
    for route in docs_runtime_registry() {
        assert_entrypoint_exists(route.entrypoint);
    }
    for route in ops_runtime_registry() {
        assert_entrypoint_exists(route.entrypoint);
    }
    for route in security_runtime_registry() {
        assert_entrypoint_exists(route.entrypoint);
    }
    for route in release_runtime_registry() {
        assert_entrypoint_exists(route.entrypoint);
    }
    for route in perf_runtime_registry() {
        assert_entrypoint_exists(route.entrypoint);
    }
}
