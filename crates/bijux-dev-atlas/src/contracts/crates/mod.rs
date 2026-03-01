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

pub fn contracts(_repo_root: &Path) -> Result<Vec<Contract>, String> {
    Ok(vec![Contract {
        id: ContractId("CRATES-001".to_string()),
        title: "crate roots include required README and CONTRACT files",
        tests: vec![TestCase {
            id: TestId("crates.docs.root_markdown_contract".to_string()),
            title: "each crate root contains README.md and CONTRACT.md",
            kind: TestKind::Pure,
            run: test_crates_001_each_crate_has_readme_and_contract,
        }],
    }])
}

pub fn contract_explain(contract_id: &str) -> String {
    match contract_id {
        "CRATES-001" => "Ensures every crate root has canonical documentation entrypoints: README.md and CONTRACT.md.".to_string(),
        _ => "Fix the listed violations and rerun `bijux dev atlas contracts crates`.".to_string(),
    }
}

pub fn contract_gate_command(_contract_id: &str) -> &'static str {
    "bijux dev atlas contracts crates --mode static"
}
