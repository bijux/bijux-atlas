// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn collect_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, out);
        } else {
            out.push(path);
        }
    }
}

#[test]
fn source_tree_does_not_contain_part_files() {
    let mut files = Vec::new();
    collect_files(&crate_root().join("src"), &mut files);
    let offenders = files
        .into_iter()
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("part"))
        .collect::<Vec<_>>();
    assert!(
        offenders.is_empty(),
        "source tree must not contain .part files: {offenders:?}"
    );
}

#[test]
fn converged_module_roots_respect_depth_budget() {
    let roots = ["app", "cli", "domains", "engine", "model", "registry", "runtime", "ui"];
    let mut offenders = Vec::new();
    for root in roots {
        let start = crate_root().join("src").join(root);
        let mut files = Vec::new();
        collect_files(&start, &mut files);
        for path in files {
            if path.extension().and_then(|value| value.to_str()) != Some("rs") {
                continue;
            }
            let rel = path.strip_prefix(crate_root().join("src")).unwrap_or(&path);
            let depth = rel.components().count();
            if depth > 4 {
                offenders.push((rel.display().to_string(), depth));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "converged module roots exceed depth budget 4: {offenders:?}"
    );
}
