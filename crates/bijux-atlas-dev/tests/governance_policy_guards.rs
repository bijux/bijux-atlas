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
        "crates/bijux-atlas-runtime/src/app/server/state/router.rs",
        "crates/bijux-atlas-runtime/src/app/server/state/request_utils.rs",
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
    let text = fs::read_to_string(root.join("crates/bijux-atlas-runtime/src/domain/mod.rs"))
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
fn atlas_server_route_support_delegates_named_http_helpers() {
    let root = repo_root();
    let text = fs::read_to_string(
        root.join("crates/bijux-atlas-server/src/adapters/inbound/http/route_support.rs"),
    )
    .expect("route support surface");

    for expected in [
        "pub(crate) use crate::adapters::inbound::http::cache_headers::*;",
        "pub(crate) use crate::adapters::inbound::http::dto::*;",
        "pub(crate) use crate::adapters::inbound::http::presenters::*;",
        "pub(crate) use crate::adapters::inbound::http::request_identity::*;",
        "pub(crate) use crate::adapters::inbound::http::response_encoding::*;",
    ] {
        assert!(
            text.contains(expected),
            "server route support must delegate reusable concerns to named HTTP modules"
        );
    }
    assert!(
        text.lines().count() <= 32,
        "server route support must stay thin"
    );
}

#[test]
fn atlas_lib_hides_legacy_ownership_roots() {
    let root = repo_root();
    let text = fs::read_to_string(root.join("crates/bijux-atlas-runtime/src/lib.rs"))
        .expect("atlas lib surface");

    for expected in [
        "pub mod adapters;",
        "pub mod app;",
        "pub mod contracts;",
        "pub mod domain;",
        "pub mod runtime;",
        "pub(crate) use crate::runtime::config::DatasetCacheConfig;",
        "pub(crate) use crate::app::ports::{CatalogFetch, DatasetStoreBackend};",
    ] {
        assert!(
            text.contains(expected),
            "atlas lib surface must prefer canonical architecture roots"
        );
    }
    let runtime_config_import = text
        .lines()
        .find(|line| line.contains("pub(crate) use crate::runtime::config::{"))
        .expect("runtime lib config re-export");
    for required in ["RateLimitConfig", "runtime_build_hash"] {
        assert!(
            runtime_config_import.contains(required),
            "atlas lib surface must expose runtime config helper `{required}` from the canonical runtime root"
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
        "pub(crate) use crate::app::server::{AppState, DatasetCacheManager};",
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
    for path in [
        "crates/bijux-atlas-runtime/src/bootstrap/mod.rs",
        "crates/bijux-atlas-runtime/src/runtime/wiring/mod.rs",
        "crates/bijux-atlas-runtime/src/runtime/wiring/server.rs",
        "crates/bijux-atlas-runtime/src/runtime/wiring/cli.rs",
        "crates/bijux-atlas-runtime/src/contracts/generated/mod.rs",
        "crates/bijux-atlas-runtime/src/contracts/store/mod.rs",
        "crates/bijux-atlas-runtime/src/contracts/telemetry/mod.rs",
    ] {
        assert!(
            !root.join(path).exists(),
            "removed legacy root must not reappear: {path}"
        );
    }
}

#[test]
fn atlas_runtime_surface_keeps_wiring_internal() {
    let root = repo_root();
    let text = fs::read_to_string(root.join("crates/bijux-atlas-runtime/src/runtime/mod.rs"))
        .expect("runtime");
    assert!(
        text.contains("pub mod config;"),
        "runtime root must expose config as the canonical public runtime surface"
    );
    assert!(
        !text.contains("pub mod wiring;"),
        "runtime root must not expose wiring as a second public surface"
    );
}

#[test]
fn atlas_contract_roots_stay_contract_owned() {
    let root = repo_root();
    let contracts =
        fs::read_to_string(root.join("crates/bijux-atlas-runtime/src/contracts/mod.rs"))
            .expect("contracts");
    let config =
        fs::read_to_string(root.join("crates/bijux-atlas-runtime/src/contracts/config/mod.rs"))
            .expect("contracts config");

    for expected in ["pub mod api;", "pub mod config;", "pub mod errors;"] {
        assert!(
            contracts.contains(expected),
            "contracts root must expose only real contract owners"
        );
    }
    for forbidden in ["pub mod generated;", "pub mod store;", "pub mod telemetry;"] {
        assert!(
            !contracts.contains(forbidden),
            "contracts root must not mirror adapter or generated roots"
        );
    }
    assert!(
        !config.contains("validate_runtime_env_contract"),
        "contracts config must not re-export runtime validation helpers"
    );
}

#[test]
fn atlas_runtime_test_surface_does_not_duplicate_api_client_suite() {
    let root = repo_root();
    assert!(
        !root
            .join("crates/bijux-atlas-runtime/tests/interfaces/client.rs")
            .exists(),
        "runtime test surface must not mirror the API-owned client suite"
    );
    assert!(
        root.join("crates/bijux-atlas-runtime/tests/interfaces/client_compatibility.rs")
            .is_file(),
        "runtime must keep the narrow legacy client compatibility check"
    );
    assert!(
        root.join("crates/bijux-atlas-api/tests/client_contracts.rs")
            .is_file(),
        "api crate must own the full Rust client test suite"
    );
}

#[test]
fn atlas_leaf_crate_test_roots_use_owned_surface_names() {
    let root = repo_root();

    for forbidden in [
        "crates/bijux-atlas-api/tests/contracts.rs",
        "crates/bijux-atlas-api/tests/client.rs",
        "crates/bijux-atlas-core/tests/contracts.rs",
        "crates/bijux-atlas-ingest/tests/workflows.rs",
        "crates/bijux-atlas-model/tests/contracts.rs",
        "crates/bijux-atlas-query/tests/contracts.rs",
        "crates/bijux-atlas-store/tests/infrastructure_store.rs",
    ] {
        assert!(
            !root.join(forbidden).exists(),
            "leaf crate test root must not use a generic ownership-free name: {forbidden}"
        );
    }

    for required in [
        "crates/bijux-atlas-api/tests/api_contracts.rs",
        "crates/bijux-atlas-api/tests/client_contracts.rs",
        "crates/bijux-atlas-core/tests/core_contracts.rs",
        "crates/bijux-atlas-ingest/tests/ingest_workflows.rs",
        "crates/bijux-atlas-model/tests/model_contracts.rs",
        "crates/bijux-atlas-query/tests/query_contracts.rs",
        "crates/bijux-atlas-store/tests/store_backend_contracts.rs",
    ] {
        assert!(
            root.join(required).is_file(),
            "leaf crate test root must keep an owned, crate-specific name: {required}"
        );
    }
}

#[test]
fn atlas_source_tree_avoids_os_junk_and_disposable_test_artifacts() {
    let root = repo_root();

    for forbidden in [
        "crates/bijux-atlas-runtime/src/.DS_Store",
        "crates/bijux-atlas-runtime/src/app/.DS_Store",
        "crates/bijux-atlas-api/src/client/client_tests.rs",
        "crates/bijux-atlas-ingest/src/engine/tests.rs",
        "crates/bijux-atlas-runtime/src/app/server/dataset_cache_manager_tests.rs",
        "crates/bijux-atlas-runtime/src/app/server/tests.rs",
        "crates/bijux-atlas-query/src/engine/query_tests/mod.rs",
    ] {
        assert!(
            !root.join(forbidden).exists(),
            "atlas source tree must not keep disposable artifacts or ownership-free test files: {forbidden}"
        );
    }

    for required in [
        "crates/bijux-atlas-server/src/app/server/tests/mod.rs",
        "crates/bijux-atlas-api/src/client/mod.rs",
        "crates/bijux-atlas-ingest/src/engine/tests/mod.rs",
        "crates/bijux-atlas-query/src/engine/tests/mod.rs",
    ] {
        assert!(
            root.join(required).is_file(),
            "atlas source tree must keep stable source-owned test surfaces: {required}"
        );
    }
}

#[test]
fn atlas_binary_ownership_matches_crate_boundaries() {
    let root = repo_root();

    for required in [
        "crates/bijux-atlas-cli/src/bin/bijux-atlas.rs",
        "crates/bijux-atlas-server/src/bin/bijux-atlas-server.rs",
        "crates/bijux-atlas-server/src/app/server/host.rs",
        "crates/bijux-atlas-api/src/bin/bijux-atlas-openapi.rs",
    ] {
        assert!(
            root.join(required).is_file(),
            "atlas binary owner is missing required entrypoint: {required}"
        );
    }

    for forbidden in [
        "crates/bijux-atlas-runtime/src/bin/bijux-atlas.rs",
        "crates/bijux-atlas-runtime/src/bin/bijux-atlas-server.rs",
        "crates/bijux-atlas-runtime/src/app/server/host.rs",
        "crates/bijux-atlas-runtime/src/bin/bijux-atlas-openapi.rs",
        "crates/bijux-atlas-runtime/src/bin/bijux-atlas-client.rs",
    ] {
        assert!(
            !root.join(forbidden).exists(),
            "atlas runtime crate must not own stale or non-runtime binaries: {forbidden}"
        );
    }

    let allowlist = fs::read_to_string(
        root.join("configs/sources/governance/governance/repo-bin-allowlist.txt"),
    )
    .expect("repo bin allowlist");
    for expected in [
        "crates/bijux-atlas-cli/src/bin/bijux-atlas.rs",
        "crates/bijux-atlas-server/src/bin/bijux-atlas-server.rs",
        "crates/bijux-atlas-api/src/bin/bijux-atlas-openapi.rs",
    ] {
        assert!(
            allowlist.contains(expected),
            "repo bin allowlist must track the owned Atlas binary surface: {expected}"
        );
    }
    for forbidden in [
        "crates/bijux-atlas-runtime/src/bin/bijux-atlas.rs",
        "crates/bijux-atlas-runtime/src/bin/bijux-atlas-server.rs",
        "crates/bijux-atlas-runtime/src/app/server/host.rs",
        "crates/bijux-atlas-runtime/src/bin/bijux-atlas-openapi.rs",
        "crates/bijux-atlas-runtime/src/bin/bijux-atlas-client.rs",
    ] {
        assert!(
            !allowlist.contains(forbidden),
            "repo bin allowlist must not preserve stale binary ownership: {forbidden}"
        );
    }
}

#[test]
fn atlas_cli_contract_tests_live_with_cli_owner_crate() {
    let root = repo_root();

    for required in [
        "crates/bijux-atlas-cli/tests/cli_surface.rs",
        "crates/bijux-atlas-cli/tests/help_surface.rs",
        "crates/bijux-atlas-cli/tests/cli_runtime_parity.rs",
        "crates/bijux-atlas-cli/tests/user_cli_surface.rs",
        "crates/bijux-atlas-cli/tests/plugin_surface.rs",
        "crates/bijux-atlas-cli/tests/snapshots/help.commands.txt",
    ] {
        assert!(
            root.join(required).is_file(),
            "cli owner crate must keep user-facing command-contract coverage: {required}"
        );
    }

    for forbidden in [
        "crates/bijux-atlas-runtime/tests/interfaces/cli_surface.rs",
        "crates/bijux-atlas-runtime/tests/interfaces/help_surface.rs",
        "crates/bijux-atlas-runtime/tests/interfaces/cli_runtime_parity.rs",
        "crates/bijux-atlas-runtime/tests/interfaces/user_cli_surface.rs",
        "crates/bijux-atlas-runtime/tests/interfaces/plugin_surface.rs",
        "crates/bijux-atlas-runtime/tests/interfaces/snapshots/help.commands.txt",
    ] {
        assert!(
            !root.join(forbidden).exists(),
            "runtime crate must not retain cli-owned command-contract tests: {forbidden}"
        );
    }
}

#[test]
fn atlas_server_contract_surfaces_live_with_server_owner_crate() {
    let root = repo_root();

    for required in [
        "crates/bijux-atlas-server/tests/interfaces_server.rs",
        "crates/bijux-atlas-server/tests/server/async_runtime_contract.rs",
        "crates/bijux-atlas-server/tests/server/download_then_serve.rs",
        "crates/bijux-atlas-server/tests/server/import_boundary_guardrails.rs",
        "crates/bijux-atlas-server/tests/server/logging_contracts.rs",
        "crates/bijux-atlas-server/tests/server/p99-regression.rs",
        "crates/bijux-atlas-server/tests/server/redis_optional.rs",
        "crates/bijux-atlas-server/tests/server/runtime_env_contract_startup.rs",
        "crates/bijux-atlas-server/tests/server/s3_backend.rs",
        "crates/bijux-atlas-server/tests/server/schema_evolution_regression.rs",
        "crates/bijux-atlas-server/tests/server/snapshots/api-surface.responses.v1.json",
        "crates/bijux-atlas-server/benches/cache_manager.rs",
        "crates/bijux-atlas-server/benches/sequence_fetch.rs",
        "crates/bijux-atlas-server/benches/diff_merge.rs",
        "crates/bijux-atlas-server/benches/bulkhead_tuning.rs",
        "crates/bijux-atlas-server/benches/gene_lookup_p99.rs",
    ] {
        assert!(
            root.join(required).is_file(),
            "server owner crate must keep server-facing contracts and benchmarks: {required}"
        );
    }

    for forbidden in [
        "crates/bijux-atlas-runtime/tests/interfaces/server.rs",
        "crates/bijux-atlas-runtime/tests/interfaces/server/async_runtime_contract.rs",
        "crates/bijux-atlas-runtime/tests/interfaces/server/download_then_serve.rs",
        "crates/bijux-atlas-runtime/tests/interfaces/server/import_boundary_guardrails.rs",
        "crates/bijux-atlas-runtime/tests/interfaces/server/logging_contracts.rs",
        "crates/bijux-atlas-runtime/tests/interfaces/server/p99-regression.rs",
        "crates/bijux-atlas-runtime/tests/interfaces/server/redis_optional.rs",
        "crates/bijux-atlas-runtime/tests/interfaces/server/runtime_env_contract_startup.rs",
        "crates/bijux-atlas-runtime/tests/interfaces/server/s3_backend.rs",
        "crates/bijux-atlas-runtime/tests/interfaces/server/schema_evolution_regression.rs",
        "crates/bijux-atlas-runtime/tests/interfaces/server/snapshots/api-surface.responses.v1.json",
        "crates/bijux-atlas-runtime/benches/server/cache_manager.rs",
        "crates/bijux-atlas-runtime/benches/server/sequence_fetch.rs",
        "crates/bijux-atlas-runtime/benches/server/diff_merge.rs",
        "crates/bijux-atlas-runtime/benches/server/bulkhead_tuning.rs",
        "crates/bijux-atlas-runtime/benches/server/gene_lookup_p99.rs",
    ] {
        assert!(
            !root.join(forbidden).exists(),
            "runtime crate must not retain server-owned integration or benchmark surfaces: {forbidden}"
        );
    }
}

#[test]
fn atlas_domain_barrel_stays_thin() {
    let root = repo_root();
    let text = fs::read_to_string(root.join("crates/bijux-atlas-runtime/src/domain/mod.rs"))
        .expect("domain");

    assert!(
        text.contains("pub use canonical::{sha256, sha256_hex, Hash256};"),
        "domain barrel must keep only canonical hashing helpers as top-level reexports"
    );
    for forbidden in [
        "pub mod dataset",
        "pub mod query",
        "pub use cluster::",
        "pub use distributed::",
        "pub use membership::",
        "pub use replication::",
        "pub use resilience::",
        "pub use security::",
        "pub use sharding::",
    ] {
        assert!(
            !text.contains(forbidden),
            "domain barrel must not re-export mixed cluster or security surfaces"
        );
    }
}

#[test]
fn atlas_app_server_surface_stays_app_owned() {
    let root = repo_root();
    let app_server =
        fs::read_to_string(root.join("crates/bijux-atlas-server/src/app/server/mod.rs"))
            .expect("app server");
    let app_state =
        fs::read_to_string(root.join("crates/bijux-atlas-server/src/app/server/state/mod.rs"))
            .expect("app server state");

    for forbidden in [
        "FederatedBackend",
        "LocalFsBackend",
        "RegistrySource",
        "RetryPolicy",
        "S3LikeBackend",
    ] {
        assert!(
            !app_server.contains(forbidden),
            "app server surface must not export adapter-owned store backends or registry helpers"
        );
    }
    for forbidden in [
        "pub use crate::adapters::outbound::store::backends::{",
        "pub use crate::adapters::outbound::store::federated::{",
    ] {
        assert!(
            !app_state.contains(forbidden),
            "app server surface must not export adapter-owned store backends or registry helpers"
        );
    }
}
