// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};

use bijux_dev_atlas::domains::load_domains;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
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

#[test]
fn every_loaded_domain_has_runnables() {
    let catalogs = load_domains(&repo_root()).expect("load domains");
    assert!(!catalogs.is_empty(), "expected registered domains");
    for catalog in catalogs {
        assert!(
            !catalog.runnables.is_empty(),
            "domain {} must register at least one runnable",
            catalog.registration.name
        );
    }
}

#[test]
fn domain_sources_do_not_print_directly() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/domains");
    let offenders = rust_sources(&root)
        .into_iter()
        .filter(|file| {
            fs::read_to_string(file)
                .map(|text| text.contains("println!(") || text.contains("eprintln!("))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    assert!(
        offenders.is_empty(),
        "domain sources must return structured data instead of printing: {offenders:?}"
    );
}
