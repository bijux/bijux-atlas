// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};

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
        _ => "Fix the listed violations and rerun `bijux dev atlas contracts crates`.".to_string(),
    }
}

pub fn contract_gate_command(_contract_id: &str) -> &'static str {
    "bijux dev atlas contracts crates --mode static"
}
