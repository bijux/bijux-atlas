// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

use serde_json::Value;

pub(super) fn sorted_dir_entries(root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut paths = entries
        .flatten()
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

pub(super) fn walk_files(root: &Path, out: &mut Vec<PathBuf>) {
    for path in sorted_dir_entries(root) {
        if path.is_dir() {
            walk_files(&path, out);
        } else {
            out.push(path);
        }
    }
}

pub(super) fn read_json(path: &Path) -> Option<Value> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}
