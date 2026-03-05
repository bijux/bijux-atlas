// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn ops_root_markdown_files_are_boundary_stubs() {
    let root = repo_root();
    let allowed: BTreeSet<&str> = BTreeSet::from([
        "CONTRACT.md",
        "ERRORS.md",
        "INDEX.md",
        "MINIMAL_RELEASE_SURFACE.md",
        "README.md",
        "RUNBOOK_GENERATION_FROM_GRAPH.md",
        "SSOT.md",
    ]);
    let mut violations = Vec::new();
    for entry in fs::read_dir(root.join("ops")).expect("read ops root") {
        let path = entry.expect("entry").path();
        if !path.is_file() || path.extension().and_then(|v| v.to_str()) != Some("md") {
            continue;
        }
        let name = path.file_name().and_then(|v| v.to_str()).expect("name");
        if !allowed.contains(name) {
            violations.push(name.to_string());
        }
    }
    assert!(
        violations.is_empty(),
        "ops root must keep artifact-boundary markdown stubs only: {:?}",
        violations
    );
}

#[test]
fn configs_root_markdown_files_are_boundary_stubs() {
    let root = repo_root();
    let allowed: BTreeSet<&str> = BTreeSet::from(["CONTRACT.md", "INDEX.md", "README.md"]);
    let mut violations = Vec::new();
    for entry in fs::read_dir(root.join("configs")).expect("read configs root") {
        let path = entry.expect("entry").path();
        if !path.is_file() || path.extension().and_then(|v| v.to_str()) != Some("md") {
            continue;
        }
        let name = path.file_name().and_then(|v| v.to_str()).expect("name");
        if !allowed.contains(name) {
            violations.push(name.to_string());
        }
    }
    assert!(
        violations.is_empty(),
        "configs root must keep artifact-boundary markdown stubs only: {:?}",
        violations
    );
}

#[test]
fn docs_surface_boundary_policy_exists_and_defines_no_cross_boundary_duplication() {
    let root = repo_root();
    let policy = root.join("docs/governance/docs-surface-boundaries.md");
    assert!(policy.exists(), "missing docs surface boundary policy");
    let text = fs::read_to_string(policy).expect("read boundary policy");
    let lower = text.to_lowercase();
    assert!(lower.contains("public docs root"));
    assert!(lower.contains("internal docs surfaces"));
    assert!(lower.contains("non-doc roots may provide short boundary readmes"));
}

#[test]
fn redirect_registry_targets_existing_docs_pages() {
    let root = repo_root();
    let redirects_path = root.join("docs/redirects.json");
    let text = fs::read_to_string(&redirects_path).expect("read redirects");
    let map: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(&text).expect("parse redirects.json");
    let mut missing_targets = Vec::new();
    for (source, target_value) in map {
        assert!(
            source.starts_with("docs/"),
            "redirect source must stay under docs/: {source}"
        );
        let target = target_value
            .as_str()
            .unwrap_or_else(|| panic!("redirect target must be a string: {source}"));
        let target_path = root.join(target);
        if !target_path.exists() {
            missing_targets.push(format!("{source} -> {target}"));
        }
    }
    assert!(
        missing_targets.is_empty(),
        "redirect targets must exist:\n{}",
        missing_targets.join("\n")
    );
}

#[test]
fn mkdocs_top_level_sections_stay_within_budget() {
    let root = repo_root();
    let mkdocs = fs::read_to_string(root.join("mkdocs.yml")).expect("read mkdocs.yml");
    let mut in_nav = false;
    let mut top_level = 0usize;
    for line in mkdocs.lines() {
        if line.trim() == "nav:" {
            in_nav = true;
            continue;
        }
        if !in_nav {
            continue;
        }
        if !line.starts_with(' ') && !line.trim().is_empty() {
            break;
        }
        if line.starts_with("  - ") {
            top_level += 1;
        }
    }
    assert!(
        top_level <= 7,
        "mkdocs top-level sections budget exceeded: {top_level} > 7"
    );
}

#[test]
fn public_docs_nav_must_not_expose_dev_atlas_internal_docs_tree() {
    let root = repo_root();
    let mkdocs = fs::read_to_string(root.join("mkdocs.yml")).expect("read mkdocs.yml");
    assert!(
        !mkdocs.contains("crates/bijux-dev-atlas/docs/"),
        "public docs nav must not include crate-internal docs tree paths"
    );
}
