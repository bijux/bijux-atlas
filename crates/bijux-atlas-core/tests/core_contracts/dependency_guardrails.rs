// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

fn core_crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn core_sources() -> Vec<PathBuf> {
    let manifest_dir = core_crate_dir();
    let roots = [manifest_dir.join("src")];

    let mut files = Vec::new();
    for root in roots {
        if root.is_file() {
            files.push(root);
            continue;
        }
        if !root.exists() {
            continue;
        }
        for entry in fs::read_dir(root).expect("read core source dir") {
            let entry = entry.expect("entry");
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }
    files
}

#[test]
fn core_module_has_no_runtime_io_or_async_imports() {
    for path in core_sources() {
        let text = fs::read_to_string(&path).expect("read source");
        for forbidden in ["reqwest", "tokio", "hyper", "axum", "rusqlite"] {
            assert!(
                !text.contains(forbidden),
                "forbidden runtime token `{forbidden}` in {}",
                path.display()
            );
        }
    }
}

#[test]
fn core_module_rand_dependency_is_forbidden() {
    for path in core_sources() {
        let text = fs::read_to_string(&path).expect("read source");
        let rand_dependency_markers = ["use rand", "rand::", "extern crate rand", " rand ="];
        assert!(
            !rand_dependency_markers
                .iter()
                .any(|marker| text.contains(marker)),
            "core module must not reference rand in {}",
            path.display()
        );
    }
}

#[test]
fn core_module_isolated_from_runtime_modules() {
    for path in core_sources() {
        let text = fs::read_to_string(&path).expect("read source");
        for forbidden in [
            "crate::adapters::",
            "crate::app::server",
            "crate::runtime::wiring",
        ] {
            assert!(
                !text.contains(forbidden),
                "core module must not reference runtime module `{forbidden}` in {}",
                path.display()
            );
        }
    }
}

#[test]
fn serde_json_usage_is_limited_to_canonical_module() {
    for path in core_sources() {
        if path.ends_with("src/canonical.rs") {
            continue;
        }
        let content = std::fs::read_to_string(&path).expect("read source");
        assert!(
            !content.contains("serde_json"),
            "serde_json usage must be limited to src/canonical.rs: {}",
            path.display()
        );
    }
}
