// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use super::{Contract, ContractId, RunContext, TestCase, TestId, TestKind, TestResult, Violation};

fn test_docs_001_index_exists(ctx: &RunContext) -> TestResult {
    if ctx.repo_root.join("docs/INDEX.md").is_file() {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![Violation {
            contract_id: "DOC-001".to_string(),
            test_id: "docs.index.exists".to_string(),
            file: Some("docs/INDEX.md".to_string()),
            line: None,
            message: "docs/INDEX.md is required as the docs entrypoint".to_string(),
            evidence: None,
        }])
    }
}

pub fn contracts(_repo_root: &Path) -> Result<Vec<Contract>, String> {
    Ok(vec![Contract {
        id: ContractId("DOC-001".to_string()),
        title: "docs entrypoint exists",
        tests: vec![TestCase {
            id: TestId("docs.index.exists".to_string()),
            title: "docs index exists",
            kind: TestKind::Pure,
            run: test_docs_001_index_exists,
        }],
    }])
}

pub fn contract_explain(contract_id: &str) -> String {
    match contract_id {
        "DOC-001" => "The docs surface needs a single canonical entrypoint at docs/INDEX.md.".to_string(),
        _ => "Fix the listed violations and rerun `bijux dev atlas contracts docs`.".to_string(),
    }
}

pub fn contract_gate_command(_contract_id: &str) -> &'static str {
    "bijux dev atlas contracts docs --mode static"
}
