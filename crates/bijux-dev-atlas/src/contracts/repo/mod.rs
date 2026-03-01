// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::Path;

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

fn test_repo_001_law_registry_exists_and_is_valid(ctx: &RunContext) -> TestResult {
    let rel = "docs/_internal/contracts/repo-laws.json";
    let path = ctx.repo_root.join(rel);
    let text = match fs::read_to_string(&path) {
        Ok(value) => value,
        Err(err) => {
            return TestResult::Fail(vec![violation(
                "REPO-001",
                "repo.laws.registry_present",
                Some(rel.to_string()),
                format!("read failed: {err}"),
            )])
        }
    };
    let json: serde_json::Value = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(err) => {
            return TestResult::Fail(vec![violation(
                "REPO-001",
                "repo.laws.registry_present",
                Some(rel.to_string()),
                format!("invalid json: {err}"),
            )])
        }
    };
    let mut violations = Vec::new();
    if json.get("schema_version").and_then(|v| v.as_u64()) != Some(1) {
        violations.push(violation(
            "REPO-001",
            "repo.laws.registry_present",
            Some(rel.to_string()),
            "repo laws registry must declare schema_version=1",
        ));
    }
    if json.get("laws").and_then(|v| v.as_array()).is_none() {
        violations.push(violation(
            "REPO-001",
            "repo.laws.registry_present",
            Some(rel.to_string()),
            "repo laws registry must contain a laws array",
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_repo_002_root_allowlist_config_present(ctx: &RunContext) -> TestResult {
    let rel = "configs/repo/root-file-allowlist.json";
    if ctx.repo_root.join(rel).exists() {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            "REPO-002",
            "repo.surface.root_allowlist_present",
            Some(rel.to_string()),
            "root allowlist config is missing",
        )])
    }
}

pub fn contracts(_repo_root: &Path) -> Result<Vec<Contract>, String> {
    Ok(vec![
        Contract {
            id: ContractId("REPO-001".to_string()),
            title: "repo laws registry remains valid and parseable",
            tests: vec![TestCase {
                id: TestId("repo.laws.registry_present".to_string()),
                title: "repo laws registry exists and parses",
                kind: TestKind::Pure,
                run: test_repo_001_law_registry_exists_and_is_valid,
            }],
        },
        Contract {
            id: ContractId("REPO-002".to_string()),
            title: "repo root allowlist config remains present",
            tests: vec![TestCase {
                id: TestId("repo.surface.root_allowlist_present".to_string()),
                title: "root allowlist config exists",
                kind: TestKind::Pure,
                run: test_repo_002_root_allowlist_config_present,
            }],
        },
    ])
}

pub fn contract_explain(contract_id: &str) -> String {
    match contract_id {
        "REPO-001" => "Ensures canonical repo law registry exists and is valid JSON with required metadata.".to_string(),
        "REPO-002" => "Ensures root allowlist authority config exists for root surface governance.".to_string(),
        _ => "Fix the listed violations and rerun `bijux dev atlas contracts repo`.".to_string(),
    }
}

pub fn contract_gate_command(_contract_id: &str) -> &'static str {
    "bijux dev atlas contracts repo --mode static"
}
