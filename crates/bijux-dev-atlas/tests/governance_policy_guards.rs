// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn workspace_root() -> PathBuf {
    crate_root()
        .parent()
        .expect("crate parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn dev_atlas_dependency_policy_stays_minimal() {
    let cargo_toml = fs::read_to_string(crate_root().join("Cargo.toml")).expect("Cargo.toml");
    for forbidden in ["reqwest", "ureq", "axum", "tokio", "hyper", "walkdir"] {
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
fn architecture_contract_is_single_source_and_records_execution_policy() {
    let root = workspace_root();
    let architecture =
        fs::read_to_string(crate_root().join("ARCHITECTURE.md")).expect("ARCHITECTURE.md");
    assert!(
        architecture.contains("artifacts/target"),
        "ARCHITECTURE.md must document target-dir policy"
    );
    assert!(
        architecture.contains("Benchmark groups and output names remain unique"),
        "ARCHITECTURE.md must document bench isolation policy"
    );

    let checkpoint = fs::read_to_string(crate_root().join("CRATE_CONVERGENCE_CHECKPOINT.md"))
        .expect("checkpoint");
    assert!(
        !checkpoint.contains("## Internal Module Graph"),
        "internal module graph must have a single canonical description in ARCHITECTURE.md"
    );
    let docs_contract = fs::read_to_string(root.join("crates/bijux-dev-atlas/docs/CONTRACT.md"))
        .expect("crate contract doc");
    assert!(
        !docs_contract.contains("## Internal Module Graph"),
        "crate contract docs must not duplicate the internal module graph section"
    );
}

#[test]
fn command_and_ops_surface_snapshot_gates_exist() {
    let tests_root = crate_root().join("tests");
    for required in [
        "cli_help_snapshot.rs",
        "ops_surface_golden.rs",
        "cli_contracts.rs",
    ] {
        assert!(
            tests_root.join(required).exists(),
            "missing surface contract gate test {}",
            required
        );
    }
}
