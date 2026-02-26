// SPDX-License-Identifier: Apache-2.0

pub(super) fn checks_ops_final_polish_contracts(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    let requirements = [
        (
            "ops/OPS_INVARIANTS.md",
            vec![
                "- Owner:",
                "- Purpose:",
                "- Consumers:",
                "## Core Invariants",
                "## Decision Rules",
                "## Enforcement Links",
            ],
        ),
        (
            "ops/WHAT_FAILS_WHEN.md",
            vec![
                "- Owner:",
                "- Purpose:",
                "- Consumers:",
                "## Failure Impact Mapping",
                "## Deletion Impact Rule",
            ],
        ),
        (
            "ops/PUBLIC_SURFACE_CONTRACT_SUMMARY.md",
            vec![
                "- Owner:",
                "- Purpose:",
                "- Consumers:",
                "## Public Surface",
                "## Public Guarantees",
                "## Enforcement Links",
            ],
        ),
        (
            "ops/THREAT_MODEL.md",
            vec![
                "- Owner:",
                "- Purpose:",
                "- Consumers:",
                "## Threat Categories",
                "## Mitigations",
                "## Residual Risk",
            ],
        ),
        (
            "ops/SUPPLY_CHAIN_MODEL.md",
            vec![
                "- Owner:",
                "- Purpose:",
                "- Consumers:",
                "## Trusted Inputs",
                "## Verification Points",
                "## Model Boundaries",
            ],
        ),
        (
            "ops/DETERMINISM_PROOF.md",
            vec![
                "- Owner:",
                "- Purpose:",
                "- Consumers:",
                "## Determinism Proof Points",
                "## In-Repo Proof Scope",
                "## External Proof Scope",
                "## Enforcement Links",
            ],
        ),
        (
            "ops/MATURITY_SCORECARD.md",
            vec![
                "- Owner:",
                "- Purpose:",
                "- Consumers:",
                "## Dimensions",
                "## Scoring Rules",
                "## Update Rule",
            ],
        ),
        (
            "ops/DELETE_HALF_OPS_SIMULATION_REPORT.md",
            vec![
                "- Owner:",
                "- Purpose:",
                "- Consumers:",
                "## Simulation Scope",
                "## Expected Breakage Categories",
                "## Use",
            ],
        ),
        (
            "docs/operations/EXAMPLE_INCIDENT_WALKTHROUGH.md",
            vec![
                "- Owner:",
                "- Purpose:",
                "- Consumers:",
                "## Scenario",
                "## Steps",
                "## Linked Contracts",
            ],
        ),
        (
            "docs/operations/EXAMPLE_RELEASE_WALKTHROUGH.md",
            vec![
                "- Owner:",
                "- Purpose:",
                "- Consumers:",
                "## Steps",
                "## Linked Contracts",
            ],
        ),
    ];

    for (rel_str, snippets) in requirements {
        let rel = Path::new(rel_str);
        let path = ctx.repo_root.join(rel);
        if !path.exists() {
            violations.push(violation(
                "OPS_FINAL_POLISH_CONTRACT_MISSING",
                format!("missing final polish contract `{}`", rel.display()),
                "add the missing Section H contract/walkthrough document",
                Some(rel),
            ));
            continue;
        }
        let text = fs::read_to_string(&path)
            .map_err(|err| CheckError::Failed(format!("read {}: {err}", rel.display())))?;
        for snippet in snippets {
            if !text.contains(snippet) {
                violations.push(violation(
                    "OPS_FINAL_POLISH_CONTRACT_INCOMPLETE",
                    format!(
                        "final polish contract `{}` is missing required content `{snippet}`",
                        rel.display()
                    ),
                    "complete the document with required metadata and sections",
                    Some(rel),
                ));
            }
        }
    }

    let invariants = fs::read_to_string(ctx.repo_root.join("ops/OPS_INVARIANTS.md"))
        .unwrap_or_default();
    if !invariants.contains("checks_ops_final_polish_contracts") {
        violations.push(violation(
            "OPS_FINAL_POLISH_SELF_ENFORCEMENT_LINK_MISSING",
            "ops/OPS_INVARIANTS.md must reference checks_ops_final_polish_contracts".to_string(),
            "add the final polish check id to ops/OPS_INVARIANTS.md enforcement links",
            Some(Path::new("ops/OPS_INVARIANTS.md")),
        ));
    }

    Ok(violations)
}
