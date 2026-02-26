// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const BANLIST_PATH: &str = "crates/bijux-dev-atlas/merge_banlist.txt";

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

fn allowlisted_paths() -> BTreeSet<&'static str> {
    [BANLIST_PATH, "crates/bijux-dev-atlas/tests/legacy_dev_atlas_crate_names.rs"]
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

fn no_banlist_strings_appear_outside_allowlist(banlist_entries: &[&str]) {
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
        for needle in banlist_entries {
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
    let root = workspace_root();
    let banlist_text =
        fs::read_to_string(root.join(BANLIST_PATH)).expect("merge banlist file must exist");
    let banlist_entries = banlist_text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        banlist_entries.len(),
        4,
        "merge banlist must list the four deleted split dev-atlas crate names"
    );
    no_banlist_strings_appear_outside_allowlist(&banlist_entries);
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

    // This is a migration-overlap inventory, not a relaxed policy: any new overlap root must be
    // explicitly reviewed here, and Batch B should drive this set to empty.
    let expected_overlap_roots = BTreeSet::from(["main_tests.rs".to_string()]);

    let overlap_roots = present
        .into_iter()
        .filter(|name| expected_overlap_roots.contains(name))
        .collect::<BTreeSet<_>>();

    assert_eq!(
        overlap_roots, expected_overlap_roots,
        "duplicate migration surface roots changed; update Batch B convergence plan intentionally"
    );
}
