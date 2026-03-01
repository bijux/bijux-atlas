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
        "DOC-017" => "Every indexed top-level docs section must explicitly declare whether it belongs on the root docs entrypoint.".to_string(),
        "DOC-018" => "docs/index.md must link every top-level docs section that is explicitly marked as a root entrypoint section.".to_string(),
        "DOC-019" => "The current docs entrypoint pages must stay within the approved word budget.".to_string(),
        "DOC-020" => "Stable docs entrypoint pages may not contain TODO-style placeholder markers.".to_string(),
        "DOC-021" => "Docs entrypoint pages may not contain raw tab characters.".to_string(),
        "DOC-022" => "Docs entrypoint pages may not contain trailing whitespace.".to_string(),
        "DOC-023" => "Docs entrypoint pages must contain exactly one H1 heading.".to_string(),
        "DOC-024" => "Docs entrypoint pages may not use absolute local file links.".to_string(),
        "DOC-025" => "Docs entrypoint pages may not use raw http links.".to_string(),
        "DOC-026" => "docs/index.md may not link the same section index more than once.".to_string(),
        "DOC-027" => "Each required top-level section INDEX.md must link at least one local markdown page.".to_string(),
        "DOC-028" => "Each top-level section INDEX.md may link a direct local markdown page at most once.".to_string(),
        "DOC-029" => "The root docs entrypoint pages may link a local markdown page at most once per page.".to_string(),
        "DOC-030" => "Docs contracts must be able to render a deterministic index correctness report artifact.".to_string(),
        "DOC-031" => "Docs uses docs/index.md as canonical entrypoint and keeps docs/index.md content-identical as a compatibility alias.".to_string(),
        "DOC-032" => "Docs contracts must generate a deterministic broken-links artifact report.".to_string(),
        "DOC-033" => "Docs contracts must generate a deterministic orphan-pages artifact report.".to_string(),
        "DOC-034" => "Docs contracts must generate a deterministic metadata-coverage artifact report.".to_string(),
        "DOC-035" => "Docs contracts must generate a deterministic duplication artifact report.".to_string(),
        "DOC-036" => "Docs contracts must generate a deterministic contract coverage artifact report.".to_string(),
        "DOC-037" => "Reader spine pages must use YAML frontmatter with the required shared metadata keys.".to_string(),
        "DOC-038" => "Spine page frontmatter must use normalized audience, type, and stability values.".to_string(),
        "DOC-039" => "Stable spine pages must carry owner, review date, tags, and related links in frontmatter.".to_string(),
        "DOC-040" => "Reference spine pages must declare source links in frontmatter.".to_string(),
        "DOC-041" => "Internal docs must declare internal frontmatter and stay off the user audience path.".to_string(),
        "DOC-042" => "Stable spine pages must keep review dates normalized in frontmatter.".to_string(),
        "DOC-043" => "How-to spine pages must declare verification in frontmatter.".to_string(),
        "DOC-044" => "The shared docs frontmatter schema must exist and require the core metadata keys.".to_string(),
        "DOC-045" => "Reader utility pages must keep the shared frontmatter metadata required for published guidance pages.".to_string(),
        "DOC-046" => "Reader utility pages must stay on the published surface and may not link directly into docs/_internal.".to_string(),
        "DOC-047" => "Reader spine pages must stay on the published surface and may not link directly into docs/_internal.".to_string(),
        _ => "Fix the listed violations and rerun `bijux dev atlas contracts docs`.".to_string(),
    }
}

pub fn contract_gate_command(_contract_id: &str) -> &'static str {
    "bijux dev atlas contracts docs --mode static"
}
