// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::{Contract, ContractId, RunContext, TestCase, TestId, TestKind, TestResult, Violation};

include!("helpers.inc.rs");

fn test_crates_001_each_crate_has_readme_and_contract(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    for crate_dir in collect_crate_dirs(&ctx.repo_root) {
        let rel = crate_dir
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&crate_dir)
            .display()
            .to_string();
        for required in ["README.md", "CONTRACT.md"] {
            let target = crate_dir.join(required);
            if !target.exists() {
                violations.push(violation(
                    "CRATES-001",
                    "crates.docs.root_markdown_contract",
                    Some(format!("{rel}/{required}")),
                    format!("crate root missing required file `{required}`"),
                ));
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_crates_002_root_markdown_allowlist(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    for crate_dir in collect_crate_dirs(&ctx.repo_root) {
        let rel = crate_dir
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&crate_dir)
            .display()
            .to_string();
        let Ok(entries) = fs::read_dir(&crate_dir) else {
            continue;
        };
        for path in entries.flatten().map(|entry| entry.path()) {
            if !path.is_file() {
                continue;
            }
            if path.extension().and_then(|value| value.to_str()) != Some("md") {
                continue;
            }
            let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };
            if name != "README.md" && name != "CONTRACT.md" {
                violations.push(violation(
                    "CRATES-002",
                    "crates.docs.root_markdown_allowlist",
                    Some(format!("{rel}/{name}")),
                    "crate root markdown allowlist only permits README.md and CONTRACT.md",
                ));
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_crates_003_docs_file_budget(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    for crate_dir in collect_crate_dirs(&ctx.repo_root) {
        let crate_name = crate_dir
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("unknown");
        let files = crate_docs_markdown_files(&crate_dir);
        if files.len() > 15 {
            violations.push(violation(
                "CRATES-003",
                "crates.docs.docs_file_budget",
                Some(format!("crates/{crate_name}/docs")),
                format!(
                    "crate docs budget exceeded: `{crate_name}` has {} markdown files in docs/ (max 15)",
                    files.len()
                ),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_crates_004_docs_kebab_case_filenames(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    for crate_dir in collect_crate_dirs(&ctx.repo_root) {
        for path in crate_docs_markdown_files(&crate_dir) {
            let rel = path
                .strip_prefix(&ctx.repo_root)
                .unwrap_or(&path)
                .display()
                .to_string();
            let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };
            if !is_kebab_case_markdown_filename(name) {
                violations.push(violation(
                    "CRATES-004",
                    "crates.docs.kebab_case_filenames",
                    Some(rel),
                    "crate docs markdown filename must be lowercase kebab-case",
                ));
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_crates_005_docs_relative_links_resolve(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    for crate_dir in collect_crate_dirs(&ctx.repo_root) {
        for source in crate_docs_markdown_files(&crate_dir) {
            let rel_source = source
                .strip_prefix(&ctx.repo_root)
                .unwrap_or(&source)
                .display()
                .to_string();
            let Ok(contents) = fs::read_to_string(&source) else {
                continue;
            };
            let base_dir = source.parent().unwrap_or(&source);
            for target in markdown_links(&contents) {
                if target.starts_with("http://")
                    || target.starts_with("https://")
                    || target.starts_with('#')
                    || target.starts_with("mailto:")
                {
                    continue;
                }
                let clean = target.split('#').next().unwrap_or(&target);
                if clean.is_empty() {
                    continue;
                }
                let resolved = base_dir.join(clean);
                if !resolved.exists() {
                    violations.push(violation(
                        "CRATES-005",
                        "crates.docs.relative_links_resolve",
                        Some(rel_source.clone()),
                        format!("broken relative link target: {target}"),
                    ));
                }
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_crates_006_readme_has_required_sections(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    for crate_dir in collect_crate_dirs(&ctx.repo_root) {
        let crate_name = crate_dir
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("unknown");
        let readme = crate_dir.join("README.md");
        let rel_readme = readme
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&readme)
            .display()
            .to_string();
        let Ok(contents) = fs::read_to_string(&readme) else {
            continue;
        };
        let lower = contents.to_ascii_lowercase();
        let has_purpose = lower.contains("## purpose");
        let has_how_to_use = lower.contains("## how to use");
        let has_where_docs_live = lower.contains("## where docs live");
        if !(has_purpose && has_how_to_use && has_where_docs_live) {
            violations.push(violation(
                "CRATES-006",
                "crates.docs.readme_required_sections",
                Some(rel_readme),
                format!(
                    "crate `{crate_name}` README.md must include sections: `## Purpose`, `## How to use`, `## Where docs live`"
                ),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_crates_007_contract_required_sections(ctx: &RunContext) -> TestResult {
    let required_headers = [
        "## Inputs",
        "## Outputs",
        "## Invariants",
        "## Effects policy",
        "## Error policy",
        "## Versioning/stability",
        "## Tests expectations",
        "## Dependencies allowed",
        "## Anti-patterns",
    ];
    let mut violations = Vec::new();
    for crate_dir in collect_crate_dirs(&ctx.repo_root) {
        let crate_name = crate_dir
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("unknown");
        let contract = crate_dir.join("CONTRACT.md");
        let rel = contract
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&contract)
            .display()
            .to_string();
        let Ok(text) = fs::read_to_string(&contract) else {
            continue;
        };
        for required in required_headers {
            if !text.contains(required) {
                violations.push(violation(
                    "CRATES-007",
                    "crates.contract.required_sections",
                    Some(rel.clone()),
                    format!(
                        "crate `{crate_name}` CONTRACT.md missing required section `{required}`"
                    ),
                ));
            }
        }
        if crate_dir.join("benches").is_dir() && !text.contains("## Bench expectations") {
            violations.push(violation(
                "CRATES-007",
                "crates.contract.required_sections",
                Some(rel.clone()),
                format!(
                    "crate `{crate_name}` has benches/ and CONTRACT.md must include `## Bench expectations`"
                ),
            ));
        }
        if crate_dir.join("src/lib.rs").is_file() && !text.contains("## Public API surface") {
            violations.push(violation(
                "CRATES-007",
                "crates.contract.required_sections",
                Some(rel.clone()),
                format!(
                    "library crate `{crate_name}` must include `## Public API surface` in CONTRACT.md"
                ),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_crates_008_contract_links_resolve(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    for crate_dir in collect_crate_dirs(&ctx.repo_root) {
        let source = crate_dir.join("CONTRACT.md");
        let rel_source = source
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&source)
            .display()
            .to_string();
        let Ok(contents) = fs::read_to_string(&source) else {
            continue;
        };
        let base_dir = source.parent().unwrap_or(&source);
        for target in markdown_links(&contents) {
            if target.starts_with("http://")
                || target.starts_with("https://")
                || target.starts_with('#')
                || target.starts_with("mailto:")
            {
                continue;
            }
            let clean = target.split('#').next().unwrap_or(&target);
            if clean.is_empty() {
                continue;
            }
            let resolved = base_dir.join(clean);
            if !resolved.exists() {
                violations.push(violation(
                    "CRATES-008",
                    "crates.contract.links_resolve",
                    Some(rel_source.clone()),
                    format!("broken relative link target: {target}"),
                ));
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_crates_009_readme_links_contract_and_budget(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    for crate_dir in collect_crate_dirs(&ctx.repo_root) {
        let crate_name = crate_dir
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("unknown");
        let readme = crate_dir.join("README.md");
        let rel = readme
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&readme)
            .display()
            .to_string();
        let Ok(contents) = fs::read_to_string(&readme) else {
            continue;
        };
        if !contents.contains("CONTRACT.md") {
            violations.push(violation(
                "CRATES-009",
                "crates.docs.readme_links_contract",
                Some(rel.clone()),
                format!("crate `{crate_name}` README.md must link to CONTRACT.md"),
            ));
        }
        let docs_links = markdown_links(&contents)
            .into_iter()
            .filter(|target| target.starts_with("docs/"))
            .count();
        if docs_links > 5 {
            violations.push(violation(
                "CRATES-009",
                "crates.docs.readme_docs_link_budget",
                Some(rel),
                format!(
                    "crate `{crate_name}` README.md links to {docs_links} docs/* pages (max 5)"
                ),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_crates_010_docs_index_allowlist(ctx: &RunContext) -> TestResult {
    let allowlist = load_allowlist(
        &ctx.repo_root,
        "configs/contracts/crate-docs-index-allowlist.json",
    );
    let mut violations = Vec::new();
    for crate_dir in collect_crate_dirs(&ctx.repo_root) {
        let crate_name = crate_dir
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("unknown");
        let index = crate_dir.join("docs/index.md");
        if index.exists() && !allowlist.contains(crate_name) {
            let rel = index
                .strip_prefix(&ctx.repo_root)
                .unwrap_or(&index)
                .display()
                .to_string();
            violations.push(violation(
                "CRATES-010",
                "crates.docs.index_allowlist",
                Some(rel),
                format!("crate `{crate_name}` has docs/index.md but is not in configs/contracts/crate-docs-index-allowlist.json"),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_crates_011_docs_forbidden_paths_and_terms(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    for crate_dir in collect_crate_dirs(&ctx.repo_root) {
        for source in crate_docs_markdown_files(&crate_dir) {
            let rel = source
                .strip_prefix(&ctx.repo_root)
                .unwrap_or(&source)
                .display()
                .to_string();
            let Ok(contents) = fs::read_to_string(&source) else {
                continue;
            };
            for target in markdown_links(&contents) {
                let target_lower = target.to_ascii_lowercase();
                for forbidden in ["_internal/", "_generated/", "artifacts/"] {
                    if !target_lower.contains(forbidden) {
                        continue;
                    }
                    violations.push(violation(
                        "CRATES-011",
                        "crates.docs.forbidden_paths",
                        Some(rel.clone()),
                        format!(
                            "crate docs link target must not reference `{forbidden}`: `{target}`"
                        ),
                    ));
                }
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_crates_012_docs_code_fence_integrity(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    for crate_dir in collect_crate_dirs(&ctx.repo_root) {
        for source in crate_docs_markdown_files(&crate_dir) {
            let rel = source
                .strip_prefix(&ctx.repo_root)
                .unwrap_or(&source)
                .display()
                .to_string();
            let Ok(contents) = fs::read_to_string(&source) else {
                continue;
            };
            let mut fence_open = false;
            for line in contents.lines() {
                let trimmed = line.trim_start();
                if trimmed.starts_with("```") {
                    let lang = trimmed.trim_start_matches("```").trim();
                    if !fence_open && lang.is_empty() {
                        violations.push(violation(
                            "CRATES-012",
                            "crates.docs.code_fence_language",
                            Some(rel.clone()),
                            "code fences must specify a language",
                        ));
                    }
                    fence_open = !fence_open;
                }
            }
            if fence_open {
                violations.push(violation(
                    "CRATES-012",
                    "crates.docs.code_fence_balance",
                    Some(rel),
                    "markdown code fences are unbalanced",
                ));
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_crates_013_docs_readme_file_forbidden(ctx: &RunContext) -> TestResult {
    let allowlist = load_allowlist(
        &ctx.repo_root,
        "configs/contracts/crate-docs-readme-allowlist.json",
    );
    let mut violations = Vec::new();
    for crate_dir in collect_crate_dirs(&ctx.repo_root) {
        let crate_name = crate_dir
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("unknown");
        let readme_in_docs = crate_dir.join("docs/README.md");
        if readme_in_docs.exists() && !allowlist.contains(crate_name) {
            let rel = readme_in_docs
                .strip_prefix(&ctx.repo_root)
                .unwrap_or(&readme_in_docs)
                .display()
                .to_string();
            violations.push(violation(
                "CRATES-013",
                "crates.docs.docs_readme_forbidden",
                Some(rel),
                "crate docs/README.md is forbidden unless allowlisted",
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_crates_014_docs_file_size_limit(ctx: &RunContext) -> TestResult {
    let max_lines = 400usize;
    let mut violations = Vec::new();
    for crate_dir in collect_crate_dirs(&ctx.repo_root) {
        for source in crate_docs_markdown_files(&crate_dir) {
            let rel = source
                .strip_prefix(&ctx.repo_root)
                .unwrap_or(&source)
                .display()
                .to_string();
            let Ok(contents) = fs::read_to_string(&source) else {
                continue;
            };
            let lines = contents.lines().count();
            if lines > max_lines {
                violations.push(violation(
                    "CRATES-014",
                    "crates.docs.file_size_limit",
                    Some(rel),
                    format!("crate doc exceeds line budget: {lines} lines (max {max_lines})"),
                ));
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_crates_015_published_docs_contracts(ctx: &RunContext) -> TestResult {
    let allowlist = load_allowlist(
        &ctx.repo_root,
        "configs/contracts/crate-docs-publish-allowlist.json",
    );
    let mkdocs = fs::read_to_string(ctx.repo_root.join("mkdocs.yml")).unwrap_or_default();
    let mut violations = Vec::new();
    for crate_dir in collect_crate_dirs(&ctx.repo_root) {
        let crate_name = crate_dir
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("unknown");
        let docs_files = crate_docs_markdown_files(&crate_dir);
        for source in docs_files {
            let rel = source
                .strip_prefix(&ctx.repo_root)
                .unwrap_or(&source)
                .display()
                .to_string();
            let published = allowlist.contains(&rel);
            if mkdocs.contains(&rel) && !published {
                violations.push(violation(
                    "CRATES-015",
                    "crates.docs.publish_allowlist",
                    Some(rel.clone()),
                    "crate doc appears in mkdocs without publish allowlist entry",
                ));
            }
            if !published {
                continue;
            }
            let Ok(contents) = fs::read_to_string(&source) else {
                continue;
            };
            let header = contents.lines().take(24).collect::<Vec<_>>().join("\n");
            if !header.contains("- Owner:") || !header.contains("- Last reviewed:") {
                violations.push(violation(
                    "CRATES-015",
                    "crates.docs.publish_metadata",
                    Some(rel.clone()),
                    "published crate doc must include `- Owner:` and `- Last reviewed:` metadata near top",
                ));
            }
            let expected_prefix = format!("# {crate_name}:");
            if !contents
                .lines()
                .next()
                .unwrap_or_default()
                .starts_with(&expected_prefix)
            {
                violations.push(violation(
                    "CRATES-015",
                    "crates.docs.publish_title_prefix",
                    Some(rel.clone()),
                    format!("published crate doc title must start with `{expected_prefix}`"),
                ));
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

include!("contract_catalog.inc.rs");
