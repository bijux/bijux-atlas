// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use super::{Contract, ContractId, RunContext, TestCase, TestId, TestKind, TestResult, Violation};

include!("contracts_registry.inc.rs");
include!("contracts_static_checks.inc.rs");
include!("contracts_link_checks.inc.rs");
include!("contracts_structure_checks.inc.rs");

pub fn contract_explain(contract_id: &str) -> String {
    match contract_id {
        "DOC-001" => "The docs root may only expose the declared top-level section directories.".to_string(),
        "DOC-002" => "The docs root markdown surface is explicit and must stay within the allowlist.".to_string(),
        "DOC-003" => "Docs path depth stays bounded so the navigation tree remains navigable.".to_string(),
        "DOC-004" => "Docs directories must stay within the sibling-count budget.".to_string(),
        "DOC-005" => "Docs filenames must avoid whitespace and control characters.".to_string(),
        "DOC-006" => "Docs needs a single canonical entrypoint at docs/index.md.".to_string(),
        "DOC-007" => "The docs root may only expose the declared non-markdown files.".to_string(),
        "DOC-008" => "Every top-level docs section directory must have a non-empty owner mapping in docs/owners.json.".to_string(),
        "DOC-009" => "Every top-level docs section must be declared in docs/sections.json, with no dead section entries.".to_string(),
        "DOC-010" => "Each top-level docs section must either require or forbid INDEX.md exactly as declared in docs/sections.json.".to_string(),
        "DOC-011" => "Links inside top-level section INDEX.md files must resolve to real repo files.".to_string(),
        "DOC-012" => "Links inside the docs root entrypoint pages must resolve to real repo files.".to_string(),
        "DOC-013" => "The current docs entrypoint pages must declare a non-empty `- Owner:` metadata line near the top.".to_string(),
        "DOC-014" => "When entrypoint pages declare stability metadata, it must use only the approved normalized values.".to_string(),
        "DOC-015" => "Deprecated docs entrypoint pages must point readers to a replacement page.".to_string(),
        "DOC-016" => "Each required section INDEX.md must use the owner declared for that section in docs/owners.json.".to_string(),
        _ => "Fix the listed violations and rerun `bijux dev atlas contracts docs`.".to_string(),
    }
}

pub fn contract_gate_command(_contract_id: &str) -> &'static str {
    "bijux dev atlas contracts docs --mode static"
}
