// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repo_root() -> PathBuf {
    crate_root()
        .parent()
        .and_then(|path| path.parent())
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn dev_atlas_dependency_policy_stays_minimal() {
    let cargo_toml = fs::read_to_string(crate_root().join("Cargo.toml")).expect("Cargo.toml");
    for forbidden in ["ureq", "axum", "tokio", "hyper", "walkdir"] {
        assert!(
            !cargo_toml.contains(&format!("{forbidden} ="))
                && !cargo_toml.contains(&format!("{forbidden}.workspace")),
            "forbidden dependency `{forbidden}` found in dev-atlas Cargo.toml"
        );
    }
}

#[test]
fn benchmark_groups_are_unique_and_named_for_files() {
    let benches_root = crate_root().join("benches");
    let mut names = BTreeSet::new();
    for entry in fs::read_dir(&benches_root).expect("benches dir") {
        let entry = entry.expect("bench entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let text = fs::read_to_string(&path).expect("bench source");
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("file stem");
        let marker = "criterion_group!(";
        let idx = text
            .find(marker)
            .unwrap_or_else(|| panic!("missing criterion_group! in {}", path.display()));
        let after = &text[idx + marker.len()..];
        let group = after
            .split(',')
            .next()
            .expect("group name")
            .trim()
            .to_string();
        assert!(
            names.insert(group.clone()),
            "duplicate criterion group name `{group}`"
        );
        assert!(
            group == stem || stem.contains(&group) || group.contains(stem),
            "criterion group `{group}` should map clearly to bench file `{stem}.rs`"
        );
    }
}

#[test]
fn command_and_ops_surface_snapshot_gates_exist() {
    let tests_root = crate_root().join("tests");
    for required in ["cli_help_snapshot.rs", "ops_surface_golden.rs"] {
        assert!(
            tests_root.join(required).exists(),
            "missing required surface snapshot test {}",
            required
        );
    }
}

#[test]
fn crate_roots_do_not_accumulate_local_artifacts_directories() {
    let crates_root = repo_root().join("crates");
    let mut forbidden = Vec::new();
    for entry in fs::read_dir(&crates_root).expect("crates dir") {
        let entry = entry.expect("crate entry");
        if !entry.file_type().expect("crate entry type").is_dir() {
            continue;
        }
        let artifacts_dir = entry.path().join("artifacts");
        if artifacts_dir.is_dir() {
            forbidden.push(
                artifacts_dir
                    .strip_prefix(repo_root())
                    .expect("repo-relative artifacts dir")
                    .display()
                    .to_string(),
            );
        }
    }
    assert!(
        forbidden.is_empty(),
        "crate-local artifacts directories are forbidden; move outputs under repo-root artifacts/: {}",
        forbidden.join(", ")
    );
}
