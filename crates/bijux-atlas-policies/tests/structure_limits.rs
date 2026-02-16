use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use bijux_atlas_policies::{
    MAX_DEPTH_HARD, MAX_LOC_HARD, MAX_MODULES_PER_DIR_HARD, MAX_RS_FILES_PER_DIR_HARD,
};

fn workspace_root() -> PathBuf {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--locked")
        .arg("--format-version")
        .arg("1")
        .arg("--no-deps")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("failed to run cargo metadata for workspace root");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid cargo metadata JSON");
    PathBuf::from(
        value
            .get("workspace_root")
            .and_then(serde_json::Value::as_str)
            .expect("workspace_root missing from metadata"),
    )
}

fn collect_rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !dir.exists() {
        return out;
    }
    for entry in fs::read_dir(dir).expect("read_dir failed") {
        let entry = entry.expect("dir entry failed");
        let path = entry.path();
        if path.is_dir() {
            out.extend(collect_rs_files(&path));
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
    out
}

#[test]
fn max_loc_per_rust_file_is_enforced() {
    let root = workspace_root();
    let files = collect_rs_files(&root.join("crates"));
    let allowlist: [&str; 1] = ["crates/bijux-atlas-server/src/lib.rs"];

    let mut violators = Vec::new();
    for file in files {
        let lines = fs::read_to_string(&file)
            .expect("failed to read rust file")
            .lines()
            .count();
        if lines > MAX_LOC_HARD {
            let rel = file
                .strip_prefix(&root)
                .expect("path must be under workspace root")
                .to_string_lossy()
                .to_string();
            if !allowlist.contains(&rel.as_str()) {
                violators.push((lines, file));
            }
        }
    }

    assert!(
        violators.is_empty(),
        "max_loc policy violations (> {} lines): {:?}",
        MAX_LOC_HARD,
        violators
    );
}

#[test]
fn max_path_depth_for_rust_files_is_enforced() {
    let root = workspace_root();
    let files = collect_rs_files(&root.join("crates"));

    let mut violators = Vec::new();
    for file in files {
        let rel = file
            .strip_prefix(&root)
            .expect("path must be under workspace root");
        let depth = rel.components().count();
        if depth > MAX_DEPTH_HARD {
            violators.push((depth, rel.to_path_buf()));
        }
    }

    assert!(
        violators.is_empty(),
        "max_depth policy violations (> {} components): {:?}",
        MAX_DEPTH_HARD,
        violators
    );
}

#[test]
fn max_rs_files_per_directory_is_enforced() {
    let root = workspace_root();
    let files = collect_rs_files(&root.join("crates"));

    let mut counts: BTreeMap<PathBuf, usize> = BTreeMap::new();
    for file in files {
        let dir = file
            .parent()
            .expect("rust file must have parent")
            .strip_prefix(&root)
            .expect("parent must be under root")
            .to_path_buf();
        *counts.entry(dir).or_insert(0) += 1;
    }

    let violators: Vec<_> = counts
        .into_iter()
        .filter(|(_, count)| *count > MAX_RS_FILES_PER_DIR_HARD)
        .collect();

    assert!(
        violators.is_empty(),
        "max_rs_files_per_dir policy violations (> {}): {:?}",
        MAX_RS_FILES_PER_DIR_HARD,
        violators
    );
}

#[test]
fn max_modules_per_directory_is_enforced() {
    let root = workspace_root();
    let files = collect_rs_files(&root.join("crates"));

    let mut counts: BTreeMap<PathBuf, usize> = BTreeMap::new();
    for file in files {
        let dir = file
            .parent()
            .expect("rust file must have parent")
            .strip_prefix(&root)
            .expect("parent must be under root")
            .to_path_buf();
        *counts.entry(dir).or_insert(0) += 1;
    }

    let violators: Vec<_> = counts
        .into_iter()
        .filter(|(_, count)| *count > MAX_MODULES_PER_DIR_HARD)
        .collect();

    assert!(
        violators.is_empty(),
        "max_modules_per_dir policy violations (> {}): {:?}",
        MAX_MODULES_PER_DIR_HARD,
        violators
    );
}
