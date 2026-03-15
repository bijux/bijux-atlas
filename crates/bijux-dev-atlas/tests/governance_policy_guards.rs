// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repo_root() -> PathBuf {
    crate_root()
        .parent()
        .and_then(|path| path.parent())
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn dev_atlas_dependency_policy_stays_minimal() {
    let cargo_toml = fs::read_to_string(crate_root().join("Cargo.toml")).expect("Cargo.toml");
    for forbidden in ["ureq", "axum", "tokio", "hyper", "walkdir"] {
        assert!(
            !cargo_toml.contains(&format!("{forbidden} ="))
                && !cargo_toml.contains(&format!("{forbidden}.workspace")),
            "forbidden dependency `{forbidden}` found in dev-atlas Cargo.toml"
        );
    }
}

#[test]
fn benchmark_groups_are_unique_and_named_for_files() {
    let benches_root = crate_root().join("benches");
    let mut names = BTreeSet::new();
    for entry in fs::read_dir(&benches_root).expect("benches dir") {
        let entry = entry.expect("bench entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let text = fs::read_to_string(&path).expect("bench source");
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("file stem");
        let marker = "criterion_group!(";
        let idx = text
            .find(marker)
            .unwrap_or_else(|| panic!("missing criterion_group! in {}", path.display()));
        let after = &text[idx + marker.len()..];
        let group = after
            .split(',')
            .next()
            .expect("group name")
            .trim()
            .to_string();
        assert!(
            names.insert(group.clone()),
            "duplicate criterion group name `{group}`"
        );
        assert!(
            group == stem || stem.contains(&group) || group.contains(stem),
            "criterion group `{group}` should map clearly to bench file `{stem}.rs`"
        );
    }
}

#[test]
fn command_and_ops_surface_snapshot_gates_exist() {
    let tests_root = crate_root().join("tests");
    for required in ["cli_help_snapshot.rs", "ops_surface_golden.rs"] {
        assert!(
            tests_root.join(required).exists(),
            "missing required surface snapshot test {}",
            required
        );
    }
}

#[test]
fn crate_roots_do_not_accumulate_local_artifacts_directories() {
    let crates_root = repo_root().join("crates");
    let mut forbidden = Vec::new();
    for entry in fs::read_dir(&crates_root).expect("crates dir") {
        let entry = entry.expect("crate entry");
        if !entry.file_type().expect("crate entry type").is_dir() {
            continue;
        }
        let artifacts_dir = entry.path().join("artifacts");
        if artifacts_dir.is_dir() {
            forbidden.push(
                artifacts_dir
                    .strip_prefix(repo_root())
                    .expect("repo-relative artifacts dir")
                    .display()
                    .to_string(),
            );
        }
    }
    assert!(
        forbidden.is_empty(),
        "crate-local artifacts directories are forbidden; move outputs under repo-root artifacts/: {}",
        forbidden.join(", ")
    );
}

#[test]
fn atlas_app_server_shims_do_not_reappear() {
    let root = repo_root();
    for path in [
        "crates/bijux-atlas/src/app/server/state/router.rs",
        "crates/bijux-atlas/src/app/server/state/request_utils.rs",
    ] {
        assert!(
            !root.join(path).exists(),
            "removed app-server shim must not reappear: {path}"
        );
    }
}

#[test]
fn atlas_domain_surface_does_not_reexport_runtime_config_helpers() {
    let root = repo_root();
    let text = fs::read_to_string(root.join("crates/bijux-atlas/src/domain/mod.rs"))
        .expect("domain surface");
    assert!(
        !text.contains("resolve_bijux_cache_dir")
            && !text.contains("resolve_bijux_config_path")
            && !text.contains("crate::runtime::config")
            && !text.contains("pub use distributed_config::{\n    default_metadata_store, load_cluster_config_from_path, load_node_config_from_path,")
            && !text.contains("pub use security_runtime::{\n    load_security_config_from_path, validate_security_config,"),
        "domain surface must not depend on runtime config or runtime loader helpers"
    );
}

#[test]
fn atlas_http_handlers_utilities_stays_a_compatibility_surface() {
    let root = repo_root();
    let text = fs::read_to_string(root.join(
        "crates/bijux-atlas/src/adapters/inbound/http/handlers_utilities.rs",
    ))
    .expect("handlers utilities surface");

    for expected in [
        "pub(crate) use crate::adapters::inbound::http::cache_headers::*;",
        "pub(crate) use crate::adapters::inbound::http::dto::*;",
        "pub(crate) use crate::adapters::inbound::http::presenters::*;",
        "pub(crate) use crate::adapters::inbound::http::request_identity::*;",
        "pub(crate) use crate::adapters::inbound::http::response_encoding::*;",
    ] {
        assert!(
            text.contains(expected),
            "handlers utilities must delegate reusable concerns to named HTTP modules"
        );
    }
    assert!(
        text.lines().count() <= 1100,
        "handlers utilities must stay below the compatibility-surface budget"
    );
}

#[test]
fn atlas_lib_hides_legacy_ownership_roots() {
    let root = repo_root();
    let text =
        fs::read_to_string(root.join("crates/bijux-atlas/src/lib.rs")).expect("atlas lib surface");

    for expected in [
        "pub mod adapters;",
        "pub mod app;",
        "pub mod contracts;",
        "pub mod domain;",
        "pub mod runtime;",
        "pub(crate) use crate::app::server::{AppState, DatasetCacheConfig, DatasetCacheManager};",
        "pub(crate) use crate::app::ports::{CatalogFetch, DatasetStoreBackend};",
        "pub(crate) use crate::runtime::config::{RateLimitConfig, runtime_build_hash};",
    ] {
        assert!(
            text.contains(expected),
            "atlas lib surface must prefer canonical architecture roots"
        );
    }

    for forbidden in [
        "pub mod application;",
        "pub mod infrastructure;",
        "pub mod interfaces;",
        "pub mod bootstrap;",
        "pub mod core;",
        "pub mod model;",
        "pub mod foundation;",
        "pub use crate::app::server::{",
        "pub use crate::adapters::inbound::cli;",
        "pub use crate::adapters::inbound::client;",
        "pub use crate::adapters::outbound::store;",
        "pub use crate::interfaces::cli;",
        "pub use crate::interfaces::client;",
        "pub use crate::infrastructure::store;",
        "pub use crate::interfaces::http;",
        "pub use crate::infrastructure::redis;",
        "pub use crate::infrastructure::sqlite;",
        "pub use crate::infrastructure::telemetry;",
    ] {
        assert!(
            !text.contains(forbidden),
            "atlas lib surface must not re-expose legacy ownership roots publicly"
        );
    }
}

#[test]
fn atlas_removed_legacy_root_modules_do_not_reappear() {
    let root = repo_root();
    for path in ["crates/bijux-atlas/src/bootstrap/mod.rs"] {
        assert!(
            !root.join(path).exists(),
            "removed legacy root must not reappear: {path}"
        );
    }
}
