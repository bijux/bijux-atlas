// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn rust_sources(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        let Ok(entries) = fs::read_dir(&path) else {
            continue;
        };
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                stack.push(entry_path);
            } else if entry_path.extension().is_some_and(|ext| ext == "rs") {
                files.push(entry_path);
            }
        }
    }
    files.sort();
    files
}

fn rel(path: &Path) -> String {
    path.strip_prefix(crate_root())
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/")
}

fn assert_no_pattern(root: &Path, pattern: &str, reason: &str) {
    let offenders = rust_sources(root)
        .into_iter()
        .filter(|file| {
            fs::read_to_string(file)
                .map(|text| text.contains(pattern))
                .unwrap_or(false)
        })
        .map(|file| rel(&file))
        .collect::<Vec<_>>();
    assert!(offenders.is_empty(), "{reason}: {offenders:?}");
}

#[test]
fn domain_sources_do_not_import_cli_layer() {
    let root = crate_root().join("src");
    for rel_root in ["core/checks", "docs", "ops"] {
        assert_no_pattern(
            &root.join(rel_root),
            "crate::cli",
            "domain-oriented sources must not depend on the CLI layer",
        );
    }
}

#[test]
fn engine_sources_do_not_import_command_or_cli_layers() {
    let root = crate_root().join("src/engine");
    assert_no_pattern(
        &root,
        "crate::cli",
        "engine must not depend on CLI parsing modules",
    );
    assert_no_pattern(
        &root,
        "crate::commands",
        "engine must not depend on command handlers",
    );
}

#[test]
fn model_sources_do_not_import_runtime_layer() {
    let root = crate_root().join("src/model");
    assert_no_pattern(
        &root,
        "crate::runtime",
        "model must stay independent from runtime adapters",
    );
}
