// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use super::{Contract, ContractId, RunContext, TestCase, TestId, TestKind, TestResult, Violation};

include!("contracts_registry.inc.rs");
include!("contracts_static_checks.inc.rs");

pub fn contract_explain(contract_id: &str) -> String {
    match contract_id {
        "ROOT-001" => {
            "The repo root is a sealed interface: only the declared top-level files and directories are allowed.".to_string()
        }
        _ => "Fix the listed violations and rerun `bijux dev atlas contracts root`.".to_string(),
    }
}

pub fn contract_gate_command(_contract_id: &str) -> &'static str {
    "bijux dev atlas contracts root --mode static"
}
