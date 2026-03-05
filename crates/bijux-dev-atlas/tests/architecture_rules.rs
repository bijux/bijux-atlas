// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path).expect("expected file to be readable")
}

fn rust_sources(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        let Ok(entries) = fs::read_dir(&path) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

#[test]
fn main_routes_through_app_module() {
    let main_rs = read(crate_root().join("src/main.rs"));
    assert!(main_rs.contains("mod app;"));
    assert!(main_rs.contains("std::process::exit(app::run());"));
}

#[test]
fn architecture_docs_and_template_exist() {
    let root = crate_root();
    for relative in [
        "docs/architecture.md",
        "docs/internal/domain-module-contract.md",
        "src/domains/_template/README.md",
        "src/domains/_template/mod.rs",
        "src/domains/_template/contracts.rs",
        "src/domains/_template/checks.rs",
        "src/domains/_template/commands.rs",
        "src/domains/_template/runtime.rs",
        "src/domains/configs/commands.rs",
        "src/domains/docs/commands.rs",
        "src/domains/docker/commands.rs",
        "src/domains/governance/commands.rs",
        "src/domains/ops/commands.rs",
        "src/domains/perf/commands.rs",
        "src/domains/release/commands.rs",
        "src/domains/security/commands.rs",
    ] {
        assert!(root.join(relative).exists(), "missing {relative}");
    }
}

#[test]
fn ui_sources_do_not_import_domains() {
    let offenders = rust_sources(&crate_root().join("src/ui"))
        .into_iter()
        .filter(|file| {
            let text = read(file);
            text.contains("crate::domains") || text.contains("bijux_dev_atlas::domains")
        })
        .collect::<Vec<_>>();
    assert!(
        offenders.is_empty(),
        "ui layer must not import domain modules directly: {offenders:?}"
    );
}

#[test]
fn domain_names_are_registered_once() {
    let names = bijux_dev_atlas::domains::all_domains();
    assert_eq!(
        names,
        &[
            "configs",
            "docker",
            "docs",
            "governance",
            "ops",
            "perf",
            "release",
            "security",
            "tutorials",
        ]
    );
}
