// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const FORBIDDEN: [&str; 4] = [
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
    [
        // Workspace and crate manifests still depend on split crates until later batches rewire imports.
        "Cargo.toml",
        "crates/bijux-dev-atlas/Cargo.toml",
        // Explicit migration planning / checkpoint docs.
        "crates/bijux-dev-atlas/CRATE_CONVERGENCE_CHECKPOINT.md",
        // Compatibility checks and docs still reference split-crate paths by design during migration.
        "crates/bijux-dev-atlas/src/core/checks/ops/governance_repo_checks.rs",
        "crates/bijux-dev-atlas/src/core/checks/ops/documentation_and_config_checks/config_and_control_plane_checks.rs",
        "ops/DETERMINISM_PROOF.md",
        ".github/workflows/ops-validate.yml",
        "configs/docs/quality-policy.json",
        // Generated/golden snapshots are updated in Batch 8 after the actual removals happen.
        "crates/bijux-dev-atlas/tests/goldens/crate_doc_governance_snapshot.json",
        "docs/registry.json",
        "docs/_generated/topic-index.json",
        "docs/_generated/sitemap.json",
        "docs/_generated/breadcrumbs.json",
        "docs/_generated/search-index.json",
        "docs/_generated/crate-doc-coverage.json",
        "docs/_generated/crate-doc-governance.json",
        "docs/_generated/crate-doc-governance.md",
        "docs/_generated/docs-dependency-graph.json",
        "docs/_generated/docs-quality-dashboard.json",
        "docs/_generated/crate-doc-pruning.json",
        "docs/_generated/crate-doc-api-table.md",
        "docs/_generated/crate-docs-slice.json",
        "docs/_generated/docs-inventory.md",
        "docs/_generated/topic-index.md",
    ]
    .into_iter()
    .collect()
}

fn ignored_subtrees(root: &Path) -> BTreeSet<PathBuf> {
    [
        root.join("target"),
        root.join(".git"),
        // Batch 7 deletes these crates. Until then, they are expected to contain legacy names.
        root.join("crates/bijux-dev-atlas-core"),
        root.join("crates/bijux-dev-atlas-model"),
        root.join("crates/bijux-dev-atlas-adapters"),
        root.join("crates/bijux-dev-atlas-policies"),
    ]
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

#[test]
fn no_unexpected_legacy_dev_atlas_crate_names_remain() {
    let root = workspace_root();
    let allowlisted = allowlisted_paths();
    let ignored = ignored_subtrees(&root);
    let mut files = Vec::new();
    for rel in [".github/workflows", "crates", "configs", "ops", "docs", "makefiles"] {
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
        if rel == "crates/bijux-dev-atlas/tests/legacy_dev_atlas_crate_names.rs" {
            continue;
        }
        let Ok(text) = fs::read_to_string(&file) else {
            continue;
        };
        for needle in FORBIDDEN {
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
