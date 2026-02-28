// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use super::{Contract, ContractId, RunContext, TestCase, TestId, TestKind, TestResult, Violation};

fn fail(contract_id: &str, test_id: &str, file: &str, message: impl Into<String>) -> TestResult {
    TestResult::Fail(vec![Violation {
        contract_id: contract_id.to_string(),
        test_id: test_id.to_string(),
        file: Some(file.to_string()),
        line: Some(1),
        message: message.into(),
        evidence: None,
    }])
}

fn read_text(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|err| format!("read {} failed: {err}", path.display()))
}

fn test_configs_001_inventory_manifest_exists(ctx: &RunContext) -> TestResult {
    let path = ctx.repo_root.join("configs/inventory.json");
    match read_text(&path) {
        Ok(_) => TestResult::Pass,
        Err(err) => fail(
            "CONFIGS-001",
            "configs.inventory.manifest_exists",
            "configs/inventory.json",
            err,
        ),
    }
}

fn test_configs_002_inventory_manifest_parses(ctx: &RunContext) -> TestResult {
    let path = ctx.repo_root.join("configs/inventory.json");
    let text = match read_text(&path) {
        Ok(text) => text,
        Err(err) => {
            return fail(
                "CONFIGS-002",
                "configs.inventory.manifest_parses",
                "configs/inventory.json",
                err,
            )
        }
    };
    match serde_json::from_str::<serde_json::Value>(&text) {
        Ok(_) => TestResult::Pass,
        Err(err) => fail(
            "CONFIGS-002",
            "configs.inventory.manifest_parses",
            "configs/inventory.json",
            format!("parse configs/inventory.json failed: {err}"),
        ),
    }
}

fn test_configs_003_output_schemas_exist(ctx: &RunContext) -> TestResult {
    let required = [
        "configs/contracts/contracts-output.schema.json",
        "configs/contracts/contracts-coverage-output.schema.json",
    ];
    for rel in required {
        let path = ctx.repo_root.join(rel);
        if !path.is_file() {
            return fail(
                "CONFIGS-003",
                "configs.contracts.output_schemas_exist",
                rel,
                "required contracts output schema is missing",
            );
        }
    }
    TestResult::Pass
}

fn test_configs_004_contract_docs_exist(ctx: &RunContext) -> TestResult {
    for rel in ["configs/README.md", "configs/CONTRACT.md"] {
        let path = ctx.repo_root.join(rel);
        if !path.is_file() {
            return fail(
                "CONFIGS-004",
                "configs.docs.root_contract_docs_exist",
                rel,
                "configs root must keep README.md and CONTRACT.md",
            );
        }
    }
    TestResult::Pass
}

pub fn contracts(_repo_root: &Path) -> Result<Vec<Contract>, String> {
    Ok(vec![
        Contract {
            id: ContractId("CONFIGS-001".to_string()),
            title: "configs inventory manifest exists",
            tests: vec![TestCase {
                id: TestId("configs.inventory.manifest_exists".to_string()),
                title: "configs inventory manifest is committed",
                kind: TestKind::Pure,
                run: test_configs_001_inventory_manifest_exists,
            }],
        },
        Contract {
            id: ContractId("CONFIGS-002".to_string()),
            title: "configs inventory manifest parses",
            tests: vec![TestCase {
                id: TestId("configs.inventory.manifest_parses".to_string()),
                title: "configs inventory manifest is valid json",
                kind: TestKind::Pure,
                run: test_configs_002_inventory_manifest_parses,
            }],
        },
        Contract {
            id: ContractId("CONFIGS-003".to_string()),
            title: "contracts output schemas exist",
            tests: vec![TestCase {
                id: TestId("configs.contracts.output_schemas_exist".to_string()),
                title: "contracts output schemas exist under configs/contracts",
                kind: TestKind::Pure,
                run: test_configs_003_output_schemas_exist,
            }],
        },
        Contract {
            id: ContractId("CONFIGS-004".to_string()),
            title: "configs root contract docs exist",
            tests: vec![TestCase {
                id: TestId("configs.docs.root_contract_docs_exist".to_string()),
                title: "configs root keeps readme and contract docs",
                kind: TestKind::Pure,
                run: test_configs_004_contract_docs_exist,
            }],
        },
    ])
}

pub fn contract_explain(contract_id: &str) -> String {
    match contract_id {
        "CONFIGS-001" => {
            "The configs inventory manifest is the stable anchor for config discovery.".to_string()
        }
        "CONFIGS-002" => {
            "The configs inventory manifest must remain parseable so contracts can consume it."
                .to_string()
        }
        "CONFIGS-003" => {
            "Contracts output schemas must exist under configs/contracts for machine consumers."
                .to_string()
        }
        "CONFIGS-004" => {
            "Configs keeps root-level human docs so the contract surface has a clear pointer."
                .to_string()
        }
        _ => "Fix the listed violations and rerun `bijux dev atlas contracts configs`.".to_string(),
    }
}

pub fn contract_gate_command(_contract_id: &str) -> &'static str {
    "bijux dev atlas contracts configs --mode static"
}
