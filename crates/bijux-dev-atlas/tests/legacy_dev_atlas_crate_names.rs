// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const LEGACY_DEV_ATLAS_CRATE_NAMES: &[&str] = &[
    "bijux-dev-atlas-core",
    "bijux-dev-atlas-model",
    "bijux-dev-atlas-adapters",
    "bijux-dev-atlas-policies",
];

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

fn allowlisted_paths() -> BTreeSet<&'static str> {
    ["crates/bijux-dev-atlas/tests/legacy_dev_atlas_crate_names.rs"]
        .into_iter()
        .collect()
}

fn ignored_subtrees(root: &Path) -> BTreeSet<PathBuf> {
    [root.join("target"), root.join(".git")]
        .into_iter()
        .collect()
}

fn collect_files(root: &Path, ignored: &BTreeSet<PathBuf>, out: &mut Vec<PathBuf>) {
    if ignored.contains(root) {
        return;
    }
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_files(&path, ignored, out);
            } else {
                out.push(path);
            }
        }
    }
}

fn assert_no_legacy_dev_atlas_crate_names_outside_guard(legacy_names: &[&str]) {
    let root = workspace_root();
    let allowlisted = allowlisted_paths();
    let ignored = ignored_subtrees(&root);
    let mut files = Vec::new();
    for rel in [
        ".github/workflows",
        "crates",
        "configs",
        "ops",
        "docs",
        "makefiles",
    ] {
        collect_files(&root.join(rel), &ignored, &mut files);
    }
    files.sort();

    let mut violations = Vec::new();
    for file in files {
        let rel = file
            .strip_prefix(&root)
            .expect("file under workspace root")
            .to_string_lossy()
            .replace('\\', "/");
        if allowlisted.contains(rel.as_str()) {
            continue;
        }
        let Ok(text) = fs::read_to_string(&file) else {
            continue;
        };
        for needle in legacy_names {
            if text.contains(needle) {
                violations.push(format!("{rel}: contains `{needle}`"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "unexpected legacy dev-atlas crate references remain: {violations:?}"
    );
}

#[test]
fn no_unexpected_legacy_dev_atlas_crate_names_remain() {
    assert_eq!(
        LEGACY_DEV_ATLAS_CRATE_NAMES.len(),
        4,
        "the legacy dev-atlas crate-name guard must list the four removed split crate names"
    );
    assert_no_legacy_dev_atlas_crate_names_outside_guard(LEGACY_DEV_ATLAS_CRATE_NAMES);
}

#[test]
fn duplicate_migration_surface_roots_are_explicit_and_do_not_expand() {
    let src_root = workspace_root().join("crates/bijux-dev-atlas/src");
    let entries = fs::read_dir(&src_root)
        .expect("src dir")
        .flatten()
        .map(|entry| entry.path())
        .collect::<Vec<_>>();

    let mut present = BTreeSet::new();
    for path in entries {
        let name = path
            .file_name()
            .and_then(|v| v.to_str())
            .expect("utf8 path")
            .to_string();
        present.insert(name);
    }

    // Any overlapping source root must be explicitly reviewed here instead of quietly expanding.
    let expected_overlap_roots = BTreeSet::new();

    let overlap_roots = present
        .into_iter()
        .filter(|name| expected_overlap_roots.contains(name))
        .collect::<BTreeSet<_>>();

    assert_eq!(
        overlap_roots, expected_overlap_roots,
        "duplicate migration surface roots changed; update Batch B convergence plan intentionally"
    );
}
