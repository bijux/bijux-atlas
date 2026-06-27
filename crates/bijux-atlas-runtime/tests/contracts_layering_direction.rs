// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn rust_files_under(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }
    out
}

fn manifest_table<'a>(manifest: &'a toml::Value, key: &str) -> Option<&'a toml::value::Table> {
    manifest.get(key).and_then(toml::Value::as_table)
}

#[test]
fn workspace_declares_core_model_query_runtime_and_dev_crates_explicitly() {
    let root = workspace_root();
    let cargo = std::fs::read_to_string(root.join("Cargo.toml")).expect("workspace Cargo.toml");
    for member in [
        "crates/bijux-atlas",
        "crates/bijux-atlas-cli",
        "crates/bijux-atlas-core",
        "crates/bijux-atlas-ingest",
        "crates/bijux-atlas-model",
        "crates/bijux-atlas-query",
        "crates/bijux-atlas-api",
        "crates/bijux-atlas-runtime",
        "crates/bijux-atlas-server",
        "crates/bijux-atlas-store",
        "crates/bijux-atlas-dev",
    ] {
        assert!(
            cargo.contains(member),
            "workspace members missing required architecture crate `{member}`"
        );
    }
}

#[test]
fn core_crate_stays_runtime_independent_by_dependency_contract() {
    let root = workspace_root();
    let cargo = std::fs::read_to_string(root.join("crates/bijux-atlas-core/Cargo.toml"))
        .expect("core cargo");

    for forbidden in [
        "bijux-atlas =",
        "bijux-atlas-runtime =",
        "bijux-atlas-dev =",
        "axum =",
        "tokio =",
        "rusqlite =",
        "reqwest =",
    ] {
        assert!(
            !cargo.contains(forbidden),
            "core crate must not depend on runtime/dev surface `{forbidden}`"
        );
    }
}

#[test]
fn ingest_crate_stays_runtime_and_http_independent_by_dependency_contract() {
    let root = workspace_root();
    let cargo = std::fs::read_to_string(root.join("crates/bijux-atlas-ingest/Cargo.toml"))
        .expect("ingest cargo");

    for forbidden in [
        "bijux-atlas =",
        "bijux-atlas-runtime =",
        "bijux-atlas-dev =",
        "axum =",
        "reqwest =",
        "tracing-subscriber =",
    ] {
        assert!(
            !cargo.contains(forbidden),
            "ingest crate must not depend on runtime/http surface `{forbidden}`"
        );
    }
}

#[test]
fn model_crate_stays_transport_and_runtime_independent_by_dependency_contract() {
    let root = workspace_root();
    let cargo = std::fs::read_to_string(root.join("crates/bijux-atlas-model/Cargo.toml"))
        .expect("model cargo");

    for forbidden in [
        "bijux-atlas =",
        "bijux-atlas-runtime =",
        "bijux-atlas-dev =",
        "axum =",
        "tokio =",
        "rusqlite =",
        "reqwest =",
    ] {
        assert!(
            !cargo.contains(forbidden),
            "model crate must not depend on runtime/dev surface `{forbidden}`"
        );
    }
}

#[test]
fn api_crate_production_dependencies_stay_runtime_independent_by_contract() {
    let root = workspace_root();
    let manifest_path = root.join("crates/bijux-atlas-api/Cargo.toml");
    let cargo = std::fs::read_to_string(&manifest_path).expect("api cargo");
    let manifest: toml::Value = toml::from_str(&cargo)
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", manifest_path.display()));
    let dependencies = manifest_table(&manifest, "dependencies").expect("api dependencies");

    for forbidden in [
        "bijux-atlas",
        "bijux-atlas-runtime",
        "bijux-atlas-dev",
        "axum",
        "tokio",
        "rusqlite",
    ] {
        assert!(
            !dependencies.contains_key(forbidden),
            "api production dependencies must not include runtime/dev surface `{forbidden}`"
        );
    }

    assert!(
        dependencies.contains_key("reqwest"),
        "api production dependencies must retain reqwest for the published Rust client surface"
    );
}

#[test]
fn api_crate_dev_dependencies_are_scoped_to_surface_harnesses() {
    let root = workspace_root();
    let manifest_path = root.join("crates/bijux-atlas-api/Cargo.toml");
    let cargo = std::fs::read_to_string(&manifest_path).expect("api cargo");
    let manifest: toml::Value = toml::from_str(&cargo)
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", manifest_path.display()));
    let dev_dependencies =
        manifest_table(&manifest, "dev-dependencies").expect("api dev-dependencies");

    let allowlist = [
        "axum",
        "bijux-atlas-core",
        "bijux-atlas-query",
        "bijux-atlas-server",
        "criterion",
        "hex",
        "hmac",
        "regex",
        "rusqlite",
        "sha2",
        "tempfile",
        "tokio",
        "tracing",
        "tracing-subscriber",
    ];

    for key in dev_dependencies.keys() {
        assert!(
            allowlist.contains(&key.as_str()),
            "api dev-dependency `{key}` must be explicitly allowlisted as HTTP/client test harness surface"
        );
    }
}

#[test]
fn store_crate_stays_runtime_and_maintainer_independent_by_dependency_contract() {
    let root = workspace_root();
    let cargo = std::fs::read_to_string(root.join("crates/bijux-atlas-store/Cargo.toml"))
        .expect("store cargo");

    for forbidden in [
        "bijux-atlas =",
        "bijux-atlas-runtime =",
        "bijux-atlas-dev =",
        "axum =",
        "tokio =",
    ] {
        assert!(
            !cargo.contains(forbidden),
            "store crate must not depend on runtime/dev surface `{forbidden}`"
        );
    }
}

#[test]
fn query_crate_stays_runtime_and_http_independent_by_dependency_contract() {
    let root = workspace_root();
    let cargo = std::fs::read_to_string(root.join("crates/bijux-atlas-query/Cargo.toml"))
        .expect("query cargo");

    for forbidden in [
        "bijux-atlas =",
        "bijux-atlas-runtime =",
        "bijux-atlas-dev =",
        "axum =",
        "tokio =",
        "reqwest =",
    ] {
        assert!(
            !cargo.contains(forbidden),
            "query crate must not depend on runtime/dev surface `{forbidden}`"
        );
    }
}

#[test]
fn domain_and_policy_layers_do_not_depend_on_adapter_or_runtime_modules() {
    let root = workspace_root().join("crates/bijux-atlas-runtime/src/domain");
    for file in rust_files_under(&root) {
        let text = std::fs::read_to_string(&file)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", file.display()));
        for forbidden in [
            "crate::adapters::",
            "crate::runtime::",
            "crate::app::server",
        ] {
            assert!(
                !text.contains(forbidden),
                "domain layer file {} contains forbidden dependency `{forbidden}`",
                file.display()
            );
        }
    }
}

#[test]
fn runtime_crate_does_not_contain_shadow_workspace_tree() {
    let shadow_root = workspace_root().join("crates/bijux-atlas-runtime/crates");
    assert!(
        !shadow_root.exists(),
        "runtime crate must not accumulate nested shadow crate roots: {}",
        shadow_root.display()
    );
}
