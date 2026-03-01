// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

fn tracked_rust_sources(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for base in [
        root.join("crates/bijux-dev-atlas/src"),
        root.join("crates/bijux-dev-atlas/tests"),
    ] {
        let mut stack = vec![base];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                let name = path.file_name().and_then(|v| v.to_str()).unwrap_or("");
                if name.ends_with(".rs") || name.ends_with(".inc.rs") {
                    out.push(path);
                }
            }
        }
    }
    out.sort();
    out
}

fn count_lines(path: &Path) -> usize {
    std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
        .lines()
        .count()
}

#[test]
fn source_size_hard_limit_is_below_1000_lines() {
    let root = repo_root();
    let mut violations = Vec::new();
    let mut warnings = Vec::new();
    for path in tracked_rust_sources(&root) {
        let lines = count_lines(&path);
        let rel = path
            .strip_prefix(&root)
            .unwrap_or(&path)
            .display()
            .to_string();
        if lines >= 1000 {
            violations.push((rel, lines));
        } else if lines >= 800 {
            warnings.push((rel, lines));
        }
    }
    if !warnings.is_empty() {
        eprintln!("warning-zone files (>=800 LOC):");
        for (rel, lines) in warnings {
            eprintln!("  {lines:>4} {rel}");
        }
    }
    assert!(
        violations.is_empty(),
        "error-zone files (>=1000 LOC): {:?}",
        violations
    );
}

#[test]
fn critical_files_stay_below_700_lines() {
    let root = repo_root();
    let critical = [
        "crates/bijux-dev-atlas/src/contracts/docs/docs_static_metadata_checks.inc.rs",
        "crates/bijux-dev-atlas/src/contracts/crates/mod.rs",
        "crates/bijux-dev-atlas/src/contracts/docs/docs_static_artifact_checks.inc.rs",
        "crates/bijux-dev-atlas/src/contracts/make/surface_contracts.rs",
        "crates/bijux-dev-atlas/tests/docs_registry_contracts.rs",
        "crates/bijux-dev-atlas/tests/cli_contracts/contracts_surface.rs",
        "crates/bijux-dev-atlas/src/commands/docs/runtime/docs_command_router.rs",
        "crates/bijux-dev-atlas/src/core/checks/ops/governance_repo_checks.rs",
        "crates/bijux-dev-atlas/src/contracts/make/mod.rs",
    ];
    let mut violations = Vec::new();
    for rel in critical {
        let path = root.join(rel);
        let lines = count_lines(&path);
        if lines >= 700 {
            violations.push((rel.to_string(), lines));
        }
    }
    assert!(
        violations.is_empty(),
        "critical files must stay below 700 LOC: {:?}",
        violations
    );
}
