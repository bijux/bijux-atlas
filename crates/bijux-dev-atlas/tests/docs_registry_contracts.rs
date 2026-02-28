// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

fn load_json(path: &Path) -> Value {
    serde_json::from_str(&read(path))
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()))
}

fn markdown_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().and_then(|value| value.to_str()) == Some("md") {
                    out.push(path);
                }
            }
        }
    }
    out.sort();
    out
}

fn parse_docs_field(contents: &str, labels: &[&str]) -> Option<String> {
    for line in contents.lines().take(20) {
        let trimmed = line.trim();
        for label in labels {
            let prefix = format!("- {label}:");
            if let Some(value) = trimmed.strip_prefix(&prefix) {
                let normalized = value.trim().trim_matches('`').trim();
                if !normalized.is_empty() {
                    return Some(normalized.to_string());
                }
            }
        }
    }
    None
}

#[test]
fn generated_docs_surface_is_committed_and_non_empty() {
    let root = repo_root();
    for rel in [
        "docs/_generated/docs-inventory.md",
        "docs/_generated/topic-index.md",
        "docs/_generated/topic-index.json",
        "docs/_generated/search-index.json",
        "docs/_generated/sitemap.json",
        "docs/_generated/breadcrumbs.json",
        "docs/_generated/docs-dependency-graph.json",
        "docs/_generated/command-index.json",
        "docs/_generated/schema-index.json",
        "docs/_generated/docs-quality-dashboard.json",
        "docs/_generated/make-targets.md",
    ] {
        let path = root.join(rel);
        assert!(path.is_file(), "missing committed generated docs artifact: {rel}");
        let text = read(&path);
        assert!(
            !text.trim().is_empty(),
            "generated docs artifact must be non-empty: {rel}"
        );
    }
}

#[test]
fn docs_registry_points_to_real_files_and_stability_matches_metadata() {
    let root = repo_root();
    let registry = load_json(&root.join("docs/registry.json"));
    let documents = registry["documents"].as_array().expect("documents array");
    assert!(!documents.is_empty(), "docs registry must not be empty");

    let allowed = BTreeSet::from(["stable", "evolving", "deprecated"]);
    for row in documents {
        let rel = row["path"].as_str().expect("registry path");
        let path = root.join(rel);
        assert!(path.exists(), "docs registry path must exist: {rel}");

        let stability = row["stability"].as_str().expect("registry stability");
        assert!(
            allowed.contains(stability),
            "docs registry stability must be normalized: {rel} -> {stability}"
        );

        if path.extension().and_then(|value| value.to_str()) == Some("md") {
            let text = read(&path);
            if let Some(status) = parse_docs_field(&text, &["Status", "Stability"]) {
                assert_eq!(
                    status, stability,
                    "docs registry stability must match page metadata: {rel}"
                );
            }
        }
    }
}

#[test]
fn deprecated_docs_entries_name_existing_replacements() {
    let root = repo_root();
    let registry = load_json(&root.join("docs/registry.json"));
    let documents = registry["documents"].as_array().expect("documents array");
    for row in documents {
        if row["stability"].as_str() != Some("deprecated") {
            continue;
        }
        let rel = row["path"].as_str().expect("registry path");
        let path = root.join(rel);
        let text = read(&path);
        let replacement = text
            .lines()
            .find_map(|line| {
                let trimmed = line.trim();
                if !trimmed.to_ascii_lowercase().contains("replacement") {
                    return None;
                }
                trimmed.split('`').nth(1).map(str::to_string)
            })
            .unwrap_or_else(|| panic!("deprecated docs page must name a replacement path: {rel}"));
        assert!(
            root.join(&replacement).exists(),
            "deprecated docs replacement must exist: {rel} -> {replacement}"
        );
    }
}

#[test]
fn policy_docs_cite_contract_ids() {
    let root = repo_root();
    let contract_id = regex::Regex::new(r"\b(?:ROOT|DOC|CONFIGS|MAKE|OPS|META-REQ)-[A-Z0-9-]*\d{3}\b")
        .expect("contract id regex");
    let mut violations = Vec::new();

    let files = [
        root.join("CONTRACT.md"),
        root.join("docs/CONTRACT.md"),
        root.join("docs/operations/DOCS_CONVERGENCE_POLICY.md"),
        root.join("docs/operations/ops-docs-contract.md"),
    ];

    for path in files {
        let rel = path.strip_prefix(&root).expect("repo relative").display().to_string();
        let text = read(&path);
        if !contract_id.is_match(&text) {
            violations.push(rel);
        }
    }

    assert!(
        violations.is_empty(),
        "policy-oriented docs must cite contract ids:\n{}",
        violations.join("\n")
    );
}

#[test]
fn required_contract_docs_include_lane_map_snippet() {
    let root = repo_root();
    let lane_snippet = [
        "- `local`:",
        "- `pr`:",
        "- `merge`:",
        "- `release`:",
    ];
    let mut violations = Vec::new();
    for path in markdown_files(&root.join("docs")) {
        let rel = path.strip_prefix(&root).expect("repo relative").display().to_string();
        let text = read(&path);
        if !text.to_ascii_lowercase().contains("required contracts") {
            continue;
        }
        let has_lane_map = lane_snippet.iter().all(|snippet| text.contains(snippet));
        if !has_lane_map {
            violations.push(rel);
        }
    }
    assert!(
        violations.is_empty(),
        "docs that mention required contracts must include the canonical lane map snippet:\n{}",
        violations.join("\n")
    );
}

#[test]
fn docs_index_stays_navigation_only_and_links_the_spine() {
    let root = repo_root();
    let text = read(&root.join("docs/index.md"));
    for required in [
        "## Docs Spine",
        "- Start: [Start Here](START_HERE.md)",
        "- Product: [What Is Bijux Atlas](product/what-is-bijux-atlas.md)",
        "- Architecture: [Architecture Index](architecture/INDEX.md)",
        "- API: [API Surface Index](api/INDEX.md)",
        "- Ops: [Operations Index](operations/INDEX.md)",
        "- Dev: [Development Index](development/INDEX.md)",
        "- Reference: [Reference Index](reference/INDEX.md)",
    ] {
        assert!(text.contains(required), "docs/index.md missing `{required}`");
    }
    for forbidden in [
        "## What",
        "## Why",
        "## Scope",
        "## Non-goals",
        "## Failure modes",
        "## How to verify",
    ] {
        assert!(
            !text.contains(forbidden),
            "docs/index.md must stay navigation-only and not contain `{forbidden}`"
        );
    }
}

#[test]
fn start_here_is_the_only_top_level_onboarding_page() {
    let root = repo_root();
    let start_here = read(&root.join("docs/START_HERE.md"));
    assert!(start_here.contains("This is the only onboarding root in `docs/`."));

    let mut offenders = Vec::new();
    for path in markdown_files(&root.join("docs")) {
        if path == root.join("docs/START_HERE.md") {
            continue;
        }
        if path.parent() != Some(root.join("docs").as_path()) {
            continue;
        }
        let rel = path.strip_prefix(&root).expect("repo relative").display().to_string();
        let text = read(&path).to_ascii_lowercase();
        if text.contains("only onboarding root") || text.contains("this is the only onboarding root") {
            offenders.push(rel);
        }
    }
    assert!(
        offenders.is_empty(),
        "only docs/START_HERE.md may declare onboarding-root authority:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn docs_spine_pages_exist_and_index_links_every_node() {
    let root = repo_root();
    let index = read(&root.join("docs/index.md"));
    for rel in [
        "docs/START_HERE.md",
        "docs/product/what-is-bijux-atlas.md",
        "docs/architecture/INDEX.md",
        "docs/api/INDEX.md",
        "docs/operations/INDEX.md",
        "docs/development/INDEX.md",
        "docs/reference/INDEX.md",
    ] {
        let path = root.join(rel);
        assert!(path.exists(), "docs spine page missing: {rel}");
        let link = rel.trim_start_matches("docs/");
        assert!(
            index.contains(link),
            "docs/index.md must link spine page `{rel}`"
        );
    }
}

#[test]
fn concept_registry_exists_and_points_to_a_canonical_map() {
    let root = repo_root();
    let text = read(&root.join("docs/_style/CONCEPT_REGISTRY.md"));
    for required in [
        "Defines canonical concepts and their single source pages.",
        "docs/_style/concepts.yml",
        "Each concept has exactly one canonical page.",
    ] {
        assert!(
            text.contains(required),
            "concept registry doc missing `{required}`"
        );
    }
}

#[test]
fn docs_front_matter_index_matches_registry_metadata_contract() {
    let root = repo_root();
    let index = load_json(&root.join("docs/metadata/front-matter.index.json"));
    let documents = index["documents"].as_array().expect("documents array");
    assert!(!documents.is_empty(), "front matter index must not be empty");
    for row in documents {
        let path = row["path"].as_str().expect("path");
        assert!(root.join(path).exists(), "front matter path must exist: {path}");
        for field in ["title", "owner", "area", "stability", "audience"] {
            assert!(
                row[field].as_str().is_some_and(|value| !value.trim().is_empty()),
                "front matter index field `{field}` must be non-empty for {path}"
            );
        }
    }
}

#[test]
fn docs_audience_policy_is_curated_and_front_matter_uses_allowed_values() {
    let root = repo_root();
    let policy = load_json(&root.join("docs/metadata/audiences.json"));
    let allowed = policy["allowed"]
        .as_array()
        .expect("allowed array")
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        allowed,
        BTreeSet::from(["contributors", "developers", "mixed", "operators", "reviewers"])
    );
    let index = load_json(&root.join("docs/metadata/front-matter.index.json"));
    for row in index["documents"].as_array().expect("documents array") {
        let path = row["path"].as_str().expect("path");
        let audience = row["audience"].as_str().expect("audience");
        assert!(
            allowed.contains(audience),
            "front matter audience must use allowed values: {path} -> {audience}"
        );
    }
}
