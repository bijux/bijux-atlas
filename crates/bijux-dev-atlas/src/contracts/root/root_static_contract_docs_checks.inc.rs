struct ContractDocDomain {
    name: &'static str,
    title: &'static str,
    file: &'static str,
}

const CONTRACT_DOC_DOMAINS: [ContractDocDomain; 6] = [
    ContractDocDomain {
        name: "root",
        title: "Root",
        file: "CONTRACT.md",
    },
    ContractDocDomain {
        name: "configs",
        title: "Configs",
        file: "configs/CONTRACT.md",
    },
    ContractDocDomain {
        name: "docs",
        title: "Docs",
        file: "docs/CONTRACT.md",
    },
    ContractDocDomain {
        name: "docker",
        title: "Docker",
        file: "docker/CONTRACT.md",
    },
    ContractDocDomain {
        name: "make",
        title: "Make",
        file: "make/CONTRACT.md",
    },
    ContractDocDomain {
        name: "ops",
        title: "Ops",
        file: "ops/CONTRACT.md",
    },
];

fn contracts_for_domain(repo_root: &std::path::Path, name: &str) -> Result<Vec<Contract>, String> {
    match name {
        "root" => super::root::contracts(repo_root),
        "configs" => super::configs::contracts(repo_root),
        "docs" => super::docs::contracts(repo_root),
        "docker" => super::docker::contracts(repo_root),
        "make" => super::make::contracts(repo_root),
        "ops" => super::ops::contracts(repo_root),
        _ => Err(format!("unsupported contracts domain `{name}`")),
    }
}

fn contract_type_label(contract_id: &str) -> &'static str {
    if contract_id.contains("-E-")
        || contract_id.starts_with("DOCKER-1")
        || contract_id.starts_with("DOCKER-2")
    {
        "effect"
    } else {
        "static"
    }
}

fn severity_label(severity: &str) -> &'static str {
    match severity {
        "blocker" => "blocker",
        "must" => "high",
        "should" => "medium",
        _ => "low",
    }
}

fn render_canonical_contract_doc(domain: &ContractDocDomain, contracts: &[Contract]) -> String {
    let mut rows = contracts.iter().collect::<Vec<_>>();
    rows.sort_by(|a, b| a.id.0.cmp(&b.id.0));
    let mut out = String::new();
    out.push_str(&format!("# {} Contract\n\n", domain.title));
    out.push_str("## Scope\n\n");
    out.push_str(&format!(
        "- Governed surface: `{}/` and `{}`.\n",
        domain.name, domain.file
    ));
    out.push_str("- SSOT = bijux-dev-atlas contracts runner.\n");
    if rows
        .iter()
        .any(|contract| contract_type_label(&contract.id.0) == "effect")
    {
        out.push_str("- Effects boundary: effect contracts require explicit runtime opt-in flags.\n");
    } else {
        out.push_str("- Effects boundary: this group runs static contracts only.\n");
    }
    out.push_str("- Non-goals:\n");
    out.push_str("- This document does not replace executable contract checks.\n");
    out.push_str("- This document does not grant manual exception authority.\n\n");
    out.push_str("## Contract IDs\n\n");
    out.push_str("| ID | Title | Severity | Type(static/effect) | Enforced by | Artifacts |\n");
    out.push_str("| --- | --- | --- | --- | --- | --- |\n");
    let snapshot = super::registry_snapshot(domain.name, contracts)
        .into_iter()
        .map(|row| (row.id, row.severity))
        .collect::<std::collections::BTreeMap<_, _>>();
    for contract in &rows {
        let severity = snapshot
            .get(&contract.id.0)
            .map_or("low", |value| severity_label(value));
        out.push_str(&format!(
            "| `{}` | {} | `{}` | `{}` | `bijux dev atlas contracts {}` | `artifacts/contracts/{}/report.json` |\n",
            contract.id.0,
            contract.title,
            severity,
            contract_type_label(&contract.id.0),
            domain.name,
            domain.name
        ));
    }
    out.push('\n');
    out.push_str("## Enforcement mapping\n\n");
    out.push_str("| Contract | Command(s) |\n");
    out.push_str("| --- | --- |\n");
    for contract in &rows {
        let mode = if contract_type_label(&contract.id.0) == "effect" {
            "--mode effect"
        } else {
            "--mode static"
        };
        out.push_str(&format!(
            "| `{}` | `bijux dev atlas contracts {} {}` |\n",
            contract.id.0, domain.name, mode
        ));
    }
    out.push('\n');
    out.push_str("## Output artifacts\n\n");
    out.push_str(&format!("- `artifacts/contracts/{}/report.json`\n", domain.name));
    out.push_str(&format!(
        "- `artifacts/contracts/{}/registry-snapshot.json`\n\n",
        domain.name
    ));
    out.push_str("## Contract to Gate mapping\n\n");
    out.push_str(&format!("- Gate: `contracts::{}`\n", domain.name));
    out.push_str("- Aggregate gate: `contracts::all`\n\n");
    out.push_str("## Exceptions policy\n\n");
    out.push_str("- No exceptions are allowed by this document.\n");
    out
}

fn test_root_041_contract_docs_canonical_template(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    for domain in CONTRACT_DOC_DOMAINS {
        let contracts = match contracts_for_domain(&ctx.repo_root, domain.name) {
            Ok(value) => value,
            Err(err) => {
                push_root_violation(
                    &mut violations,
                    "ROOT-041",
                    "root.contract_docs.canonical_template",
                    Some(domain.file.to_string()),
                    format!("load contracts failed: {err}"),
                );
                continue;
            }
        };
        let expected = render_canonical_contract_doc(&domain, &contracts);
        let actual = match read_root_text(
            ctx,
            domain.file,
            "ROOT-041",
            "root.contract_docs.canonical_template",
        ) {
            Ok(value) => value,
            Err(result) => return result,
        };
        if actual.contains('\t') {
            push_root_violation(
                &mut violations,
                "ROOT-041",
                "root.contract_docs.canonical_template",
                Some(domain.file.to_string()),
                "contract markdown must not contain tabs",
            );
        }
        if actual.contains('\r') {
            push_root_violation(
                &mut violations,
                "ROOT-041",
                "root.contract_docs.canonical_template",
                Some(domain.file.to_string()),
                "contract markdown must use LF line endings",
            );
        }
        if actual.contains("\n\n\n") {
            push_root_violation(
                &mut violations,
                "ROOT-041",
                "root.contract_docs.canonical_template",
                Some(domain.file.to_string()),
                "contract markdown must use a single blank line policy",
            );
        }
        if actual.trim_end() != expected.trim_end() {
            push_root_violation(
                &mut violations,
                "ROOT-041",
                "root.contract_docs.canonical_template",
                Some(domain.file.to_string()),
                "contract markdown drifted from canonical template; regenerate with contracts registry data",
            );
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_root_042_meta_registry_integrity(ctx: &RunContext) -> TestResult {
    let mut rows = Vec::new();
    for domain in CONTRACT_DOC_DOMAINS {
        let contracts = match contracts_for_domain(&ctx.repo_root, domain.name) {
            Ok(value) => value,
            Err(err) => {
                return TestResult::Fail(vec![Violation {
                    contract_id: "ROOT-042".to_string(),
                    test_id: "root.contracts.meta_registry_integrity".to_string(),
                    file: Some(domain.file.to_string()),
                    line: None,
                    message: format!("load contracts failed: {err}"),
                    evidence: None,
                }]);
            }
        };
        rows.extend(super::registry_snapshot(domain.name, &contracts));
    }
    let lints = super::lint_registry_rows(&rows);
    let mut violations = Vec::new();
    for lint in lints {
        if lint.code == "duplicate-contract-id" || lint.code == "empty-contract" {
            violations.push(Violation {
                contract_id: "ROOT-042".to_string(),
                test_id: "root.contracts.meta_registry_integrity".to_string(),
                file: None,
                line: None,
                message: lint.message,
                evidence: Some(lint.code.to_string()),
            });
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}
