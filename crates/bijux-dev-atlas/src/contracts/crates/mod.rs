// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::{Contract, ContractId, RunContext, TestCase, TestId, TestKind, TestResult, Violation};

fn violation(contract_id: &str, test_id: &str, file: Option<String>, message: impl Into<String>) -> Violation {
    Violation {
        contract_id: contract_id.to_string(),
        test_id: test_id.to_string(),
        file,
        line: None,
        message: message.into(),
        evidence: None,
    }
}

fn collect_crate_dirs(repo_root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let root = repo_root.join("crates");
    let Ok(entries) = fs::read_dir(root) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.push(path);
        }
    }
    out.sort();
    out
}

fn markdown_links(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut links = Vec::new();
    let mut idx = 0usize;
    while idx + 3 < bytes.len() {
        if bytes[idx] == b'[' {
            if let Some(close_bracket) = text[idx..].find("](") {
                let open_paren = idx + close_bracket + 1;
                if let Some(close_paren_rel) = text[open_paren + 1..].find(')') {
                    let target = &text[open_paren + 1..open_paren + 1 + close_paren_rel];
                    links.push(target.to_string());
                    idx = open_paren + 1 + close_paren_rel + 1;
                    continue;
                }
            }
        }
        idx += 1;
    }
    links
}

fn is_kebab_case_markdown_filename(name: &str) -> bool {
    if !name.ends_with(".md") {
        return false;
    }
    let stem = &name[..name.len() - 3];
    !stem.is_empty()
        && stem
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn crate_docs_markdown_files(crate_dir: &Path) -> Vec<PathBuf> {
    let docs_dir = crate_dir.join("docs");
    let Ok(entries) = fs::read_dir(docs_dir) else {
        return Vec::new();
    };
    let mut files = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("md"))
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn load_allowlist(repo_root: &Path, rel: &str) -> std::collections::BTreeSet<String> {
    let path = repo_root.join(rel);
    let Ok(text) = fs::read_to_string(path) else {
        return std::collections::BTreeSet::new();
    };
    let Ok(value) = serde_json::from_str::<Value>(&text) else {
        return std::collections::BTreeSet::new();
    };
    let mut out = std::collections::BTreeSet::new();
    if let Some(items) = value.get("allowlist").and_then(|v| v.as_array()) {
        for item in items {
            if let Some(entry) = item.as_str() {
                out.insert(entry.to_string());
            }
        }
    }
    out
}

fn test_crates_001_each_crate_has_readme_and_contract(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    for crate_dir in collect_crate_dirs(&ctx.repo_root) {
        let rel = crate_dir.strip_prefix(&ctx.repo_root).unwrap_or(&crate_dir).display().to_string();
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
                    format!("crate `{crate_name}` CONTRACT.md missing required section `{required}`"),
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
                format!("crate `{crate_name}` README.md links to {docs_links} docs/* pages (max 5)"),
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
    let allowlist = load_allowlist(&ctx.repo_root, "configs/contracts/crate-docs-index-allowlist.json");
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
                        format!("crate docs link target must not reference `{forbidden}`: `{target}`"),
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
    let allowlist = load_allowlist(&ctx.repo_root, "configs/contracts/crate-docs-readme-allowlist.json");
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
    let allowlist = load_allowlist(&ctx.repo_root, "configs/contracts/crate-docs-publish-allowlist.json");
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
            if !contents.lines().next().unwrap_or_default().starts_with(&expected_prefix) {
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

pub fn contracts(_repo_root: &Path) -> Result<Vec<Contract>, String> {
    Ok(vec![
        Contract {
            id: ContractId("CRATES-001".to_string()),
            title: "crate roots include required README and CONTRACT files",
            tests: vec![TestCase {
                id: TestId("crates.docs.root_markdown_contract".to_string()),
                title: "each crate root contains README.md and CONTRACT.md",
                kind: TestKind::Pure,
                run: test_crates_001_each_crate_has_readme_and_contract,
            }],
        },
        Contract {
            id: ContractId("CRATES-002".to_string()),
            title: "crate root markdown files follow strict allowlist",
            tests: vec![TestCase {
                id: TestId("crates.docs.root_markdown_allowlist".to_string()),
                title: "crate roots only contain README.md and CONTRACT.md markdown files",
                kind: TestKind::Pure,
                run: test_crates_002_root_markdown_allowlist,
            }],
        },
        Contract {
            id: ContractId("CRATES-003".to_string()),
            title: "crate docs directory respects markdown file budget",
            tests: vec![TestCase {
                id: TestId("crates.docs.docs_file_budget".to_string()),
                title: "crate docs directories contain at most 15 markdown files",
                kind: TestKind::Pure,
                run: test_crates_003_docs_file_budget,
            }],
        },
        Contract {
            id: ContractId("CRATES-004".to_string()),
            title: "crate docs markdown filenames use lowercase kebab-case",
            tests: vec![TestCase {
                id: TestId("crates.docs.kebab_case_filenames".to_string()),
                title: "crate docs markdown filenames are lowercase kebab-case",
                kind: TestKind::Pure,
                run: test_crates_004_docs_kebab_case_filenames,
            }],
        },
        Contract {
            id: ContractId("CRATES-005".to_string()),
            title: "crate docs relative markdown links resolve",
            tests: vec![TestCase {
                id: TestId("crates.docs.relative_links_resolve".to_string()),
                title: "crate docs links to local relative targets must resolve",
                kind: TestKind::Pure,
                run: test_crates_005_docs_relative_links_resolve,
            }],
        },
        Contract {
            id: ContractId("CRATES-006".to_string()),
            title: "crate readme sections include purpose usage and docs location",
            tests: vec![TestCase {
                id: TestId("crates.docs.readme_required_sections".to_string()),
                title: "crate README includes Purpose, How to use, and Where docs live sections",
                kind: TestKind::Pure,
                run: test_crates_006_readme_has_required_sections,
            }],
        },
        Contract {
            id: ContractId("CRATES-007".to_string()),
            title: "crate contract sections are complete and explicit",
            tests: vec![TestCase {
                id: TestId("crates.contract.required_sections".to_string()),
                title: "crate CONTRACT includes required sections for interface and policy guarantees",
                kind: TestKind::Pure,
                run: test_crates_007_contract_required_sections,
            }],
        },
        Contract {
            id: ContractId("CRATES-008".to_string()),
            title: "crate contract links resolve to existing files",
            tests: vec![TestCase {
                id: TestId("crates.contract.links_resolve".to_string()),
                title: "crate CONTRACT relative links resolve",
                kind: TestKind::Pure,
                run: test_crates_008_contract_links_resolve,
            }],
        },
        Contract {
            id: ContractId("CRATES-009".to_string()),
            title: "crate readme links contract and stays within docs link budget",
            tests: vec![TestCase {
                id: TestId("crates.docs.readme_links_contract".to_string()),
                title: "crate README links CONTRACT and limits docs links",
                kind: TestKind::Pure,
                run: test_crates_009_readme_links_contract_and_budget,
            }],
        },
        Contract {
            id: ContractId("CRATES-010".to_string()),
            title: "crate docs index usage follows explicit allowlist",
            tests: vec![TestCase {
                id: TestId("crates.docs.index_allowlist".to_string()),
                title: "crate docs/index.md appears only for allowlisted crates",
                kind: TestKind::Pure,
                run: test_crates_010_docs_index_allowlist,
            }],
        },
        Contract {
            id: ContractId("CRATES-011".to_string()),
            title: "crate docs avoid forbidden paths and governance/procedure leakage",
            tests: vec![TestCase {
                id: TestId("crates.docs.forbidden_paths".to_string()),
                title: "crate docs avoid forbidden path references and non-crate procedure content",
                kind: TestKind::Pure,
                run: test_crates_011_docs_forbidden_paths_and_terms,
            }],
        },
        Contract {
            id: ContractId("CRATES-012".to_string()),
            title: "crate docs code fences are valid and typed",
            tests: vec![TestCase {
                id: TestId("crates.docs.code_fence_language".to_string()),
                title: "crate docs code fences are balanced and specify language",
                kind: TestKind::Pure,
                run: test_crates_012_docs_code_fence_integrity,
            }],
        },
        Contract {
            id: ContractId("CRATES-013".to_string()),
            title: "crate docs disallow secondary readme without allowlist",
            tests: vec![TestCase {
                id: TestId("crates.docs.docs_readme_forbidden".to_string()),
                title: "crate docs/README.md only allowed for allowlisted crates",
                kind: TestKind::Pure,
                run: test_crates_013_docs_readme_file_forbidden,
            }],
        },
        Contract {
            id: ContractId("CRATES-014".to_string()),
            title: "crate docs files remain under size budget",
            tests: vec![TestCase {
                id: TestId("crates.docs.file_size_limit".to_string()),
                title: "crate docs markdown files remain under line budget",
                kind: TestKind::Pure,
                run: test_crates_014_docs_file_size_limit,
            }],
        },
        Contract {
            id: ContractId("CRATES-015".to_string()),
            title: "published crate docs require allowlist and metadata contract",
            tests: vec![TestCase {
                id: TestId("crates.docs.publish_allowlist".to_string()),
                title: "published crate docs are allowlisted and include ownership metadata and title prefix",
                kind: TestKind::Pure,
                run: test_crates_015_published_docs_contracts,
            }],
        },
    ])
}

pub fn contract_explain(contract_id: &str) -> String {
    match contract_id {
        "CRATES-001" => "Ensures every crate root has canonical documentation entrypoints: README.md and CONTRACT.md.".to_string(),
        "CRATES-002" => "Ensures crate root markdown files are limited to README.md and CONTRACT.md.".to_string(),
        "CRATES-003" => "Ensures each crate docs/ directory stays within the markdown file budget (max 15).".to_string(),
        "CRATES-004" => "Ensures crate docs markdown filenames use lowercase kebab-case.".to_string(),
        "CRATES-005" => "Ensures relative links in crate docs markdown resolve to existing files.".to_string(),
        "CRATES-006" => "Ensures crate README.md has Purpose, How to use, and Where docs live sections.".to_string(),
        "CRATES-007" => "Ensures each crate CONTRACT.md includes required sections for inputs, outputs, invariants, and policy expectations.".to_string(),
        "CRATES-008" => "Ensures every relative link in each crate CONTRACT.md resolves.".to_string(),
        "CRATES-009" => "Ensures each crate README links CONTRACT.md and keeps docs link sprawl under budget.".to_string(),
        "CRATES-010" => "Ensures crate docs/index.md exists only for crates listed in the explicit index allowlist.".to_string(),
        "CRATES-011" => "Ensures crate docs avoid forbidden internal/generated/artifacts references and policy/procedure leakage.".to_string(),
        "CRATES-012" => "Ensures crate docs code fences are balanced and language-tagged.".to_string(),
        "CRATES-013" => "Ensures docs/README.md in crate docs is forbidden unless explicitly allowlisted.".to_string(),
        "CRATES-014" => "Ensures crate docs files stay under a per-file size budget to avoid mega-doc drift.".to_string(),
        "CRATES-015" => "Ensures published crate docs are explicitly allowlisted and include owner/review metadata plus crate-prefixed titles.".to_string(),
        _ => "Fix the listed violations and rerun `bijux dev atlas contracts crates`.".to_string(),
    }
}

pub fn contract_gate_command(_contract_id: &str) -> &'static str {
    "bijux dev atlas contracts crates --mode static"
}
