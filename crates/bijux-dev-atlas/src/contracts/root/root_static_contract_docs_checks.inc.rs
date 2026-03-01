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
        file: "docs/contract.md",
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
        "runtime" => super::runtime::contracts(repo_root),
        "control-plane" => super::control_plane::contracts(repo_root),
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

fn render_canonical_contract_doc(
    repo_root: &std::path::Path,
    domain: &ContractDocDomain,
    contracts: &[Contract],
) -> String {
    if domain.name == "docker" {
        return super::docker::render_contract_markdown(repo_root).unwrap_or_default();
    }
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
    if domain.name == "make" {
        out.push_str("- Make is not a control plane; it is a thin wrapper over `bijux dev atlas`.\n\n");
    }
    if domain.name == "root" {
        out.push_str("## Lane policy\n\n");
        out.push_str("- `local`: ad hoc developer runs; no merge-blocking selection is implied.\n");
        out.push_str("- `pr`: runs all required contracts plus static coverage.\n");
        out.push_str("- `merge`: runs required contracts plus effect coverage.\n");
        out.push_str("- `release`: runs the full matrix of required, effect, and slow coverage.\n");
        out.push_str("- Required contracts artifact: `ops/_generated.example/contracts-required.json`.\n");
        out.push_str("- Lane guarantees reference: `docs/operations/release/lane-guarantees.md`.\n\n");
    }
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
            "| `{}` | {} | `{}` | `{}` | `bijux dev atlas contracts {}` | `artifacts/run/<run_id>/gates/contracts/{}/<profile>/<mode>/{}.json` |\n",
            contract.id.0,
            contract.title,
            severity,
            contract_type_label(&contract.id.0),
            domain.name,
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
    out.push_str(&format!(
        "- `artifacts/run/<run_id>/gates/contracts/{}/<profile>/<mode>/{}.json`\n",
        domain.name, domain.name
    ));
    out.push_str(&format!(
        "- `artifacts/run/<run_id>/gates/contracts/{}/<profile>/<mode>/{}.inventory.json`\n\n",
        domain.name, domain.name
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
        let expected = render_canonical_contract_doc(&ctx.repo_root, &domain, &contracts);
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

fn test_root_043_meta_test_mapping_integrity(ctx: &RunContext) -> TestResult {
    let mut rows = Vec::new();
    for domain in CONTRACT_DOC_DOMAINS {
        let contracts = match contracts_for_domain(&ctx.repo_root, domain.name) {
            Ok(value) => value,
            Err(err) => {
                return TestResult::Fail(vec![Violation {
                    contract_id: "ROOT-043".to_string(),
                    test_id: "root.contracts.meta_test_mapping_integrity".to_string(),
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
        if lint.code == "duplicate-test-id" {
            violations.push(Violation {
                contract_id: "ROOT-043".to_string(),
                test_id: "root.contracts.meta_test_mapping_integrity".to_string(),
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

fn test_root_044_meta_ordering_stable(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    let expected_run_order = "vec![\n                    \"root\",\n                    \"runtime\",\n                    \"control-plane\",\n                    \"docker\",\n                    \"make\",\n                    \"ops\",\n                    \"configs\",\n                    \"docs\",\n                ]";
    let command_source = match std::fs::read_to_string(
        ctx.repo_root
            .join("crates/bijux-dev-atlas/src/commands/control_plane_contracts.rs"),
    ) {
        Ok(value) => value,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "ROOT-044".to_string(),
                test_id: "root.contracts.meta_ordering_stable".to_string(),
                file: Some(
                    "crates/bijux-dev-atlas/src/commands/control_plane_contracts.rs".to_string(),
                ),
                line: None,
                message: format!("read contracts command source failed: {err}"),
                evidence: None,
            }]);
        }
    };
    if !command_source.contains(expected_run_order) {
        violations.push(Violation {
            contract_id: "ROOT-044".to_string(),
            test_id: "root.contracts.meta_ordering_stable".to_string(),
            file: Some("crates/bijux-dev-atlas/src/commands/control_plane_contracts.rs".to_string()),
            line: None,
            message: "all-domain contracts execution order changed".to_string(),
            evidence: Some(expected_run_order.to_string()),
        });
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn markdown_list_items_under_heading(text: &str, heading: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut in_section = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == heading {
            in_section = true;
            continue;
        }
        if in_section && (trimmed.ends_with(':') || trimmed.starts_with('#')) {
            break;
        }
        if !in_section {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("- ") {
            let value = rest.trim();
            if let Some(stripped) = value.strip_prefix('`') {
                if let Some((name, _)) = stripped.split_once('`') {
                    items.push(name.to_string());
                    continue;
                }
            }
            items.push(value.trim_matches('`').to_string());
        }
    }
    items
}

fn test_root_045_required_status_checks_workflow_drift(ctx: &RunContext) -> TestResult {
    let contract_id = "ROOT-045";
    let test_id = "root.required_status_checks.workflow_drift";
    let path = " .github/required-status-checks.md";
    let actual_text = match read_root_text(ctx, ".github/required-status-checks.md", contract_id, test_id) {
        Ok(value) => value,
        Err(result) => return result,
    };
    let expected_required = vec![
        "ci-pr / minimal-root-policies".to_string(),
        "ci-pr / validate-pr".to_string(),
        "ci-pr / supply-chain".to_string(),
        "ci-pr / workflow-policy".to_string(),
        "docs-only / docs".to_string(),
        "ops-validate / validate".to_string(),
    ];
    let expected_optional = vec!["ops-integration-kind / kind-integration".to_string()];
    let expected_nightly = vec!["ci-nightly / nightly-validation".to_string()];
    let actual_required =
        markdown_list_items_under_heading(&actual_text, "Required branch-protection checks for `main`:");
    let actual_optional = markdown_list_items_under_heading(&actual_text, "Optional PR checks:");
    let actual_nightly = markdown_list_items_under_heading(&actual_text, "Nightly required checks:");
    let mut violations = Vec::new();
    if actual_required != expected_required {
        violations.push(Violation {
            contract_id: contract_id.to_string(),
            test_id: test_id.to_string(),
            file: Some(path.trim().to_string()),
            line: None,
            message: "required status checks drifted from the governed workflow surface".to_string(),
            evidence: Some(format!(
                "expected {:?}, found {:?}",
                expected_required, actual_required
            )),
        });
    }
    if actual_optional != expected_optional {
        violations.push(Violation {
            contract_id: contract_id.to_string(),
            test_id: test_id.to_string(),
            file: Some(path.trim().to_string()),
            line: None,
            message: "optional status checks drifted from the governed workflow surface".to_string(),
            evidence: Some(format!(
                "expected {:?}, found {:?}",
                expected_optional, actual_optional
            )),
        });
    }
    if actual_nightly != expected_nightly {
        violations.push(Violation {
            contract_id: contract_id.to_string(),
            test_id: test_id.to_string(),
            file: Some(path.trim().to_string()),
            line: None,
            message: "nightly status checks drifted from the governed workflow surface".to_string(),
            evidence: Some(format!(
                "expected {:?}, found {:?}",
                expected_nightly, actual_nightly
            )),
        });
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_root_046_repo_law_errors_catalogued(ctx: &RunContext) -> TestResult {
    let contract_id = "ROOT-046";
    let test_id = "root.repo_law_errors.catalogued";
    let text = match read_root_text(ctx, "ops/ERRORS.md", contract_id, test_id) {
        Ok(value) => value,
        Err(result) => return result,
    };
    let expected_codes = [
        "REPO-LAW-001",
        "REPO-LAW-002",
        "REPO-LAW-003",
        "REPO-LAW-004",
    ];
    let mut violations = Vec::new();
    if !text.starts_with("# Ops Error Catalog\n") {
        violations.push(Violation {
            contract_id: contract_id.to_string(),
            test_id: test_id.to_string(),
            file: Some("ops/ERRORS.md".to_string()),
            line: None,
            message: "ops error catalog must start with the canonical title".to_string(),
            evidence: None,
        });
    }
    if !text.contains("`bijux dev atlas contracts root --mode static`") {
        violations.push(Violation {
            contract_id: contract_id.to_string(),
            test_id: test_id.to_string(),
            file: Some("ops/ERRORS.md".to_string()),
            line: None,
            message: "ops error catalog must point to the root contracts command".to_string(),
            evidence: None,
        });
    }
    for code in expected_codes {
        if !text.contains(&format!("| `{code}` |")) {
            violations.push(Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some("ops/ERRORS.md".to_string()),
                line: None,
                message: format!("missing repo law error code `{code}`"),
                evidence: None,
            });
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn required_contract_rows(
    repo_root: &std::path::Path,
) -> Result<Vec<(String, String, Vec<String>)>, String> {
    let mut rows = super::required_contract_map(repo_root)?
        .into_iter()
        .flat_map(|(domain, contracts)| {
            contracts.into_iter().map(move |(contract_id, lanes)| {
                (
                    domain.clone(),
                    contract_id,
                    lanes
                        .into_iter()
                        .map(|lane| lane.as_str().to_string())
                        .collect::<Vec<_>>(),
                )
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    Ok(rows)
}

fn approval_metadata_active(repo_root: &std::path::Path) -> Result<bool, String> {
    let path = super::required_contract_change_path(repo_root);
    let text =
        std::fs::read_to_string(&path).map_err(|e| format!("read {} failed: {e}", path.display()))?;
    let json = serde_json::from_str::<serde_json::Value>(&text)
        .map_err(|e| format!("parse {} failed: {e}", path.display()))?;
    let owner = json.get("owner").and_then(|value| value.as_str()).unwrap_or("");
    let rationale = json
        .get("rationale")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let expiry = json.get("expiry").and_then(|value| value.as_str()).unwrap_or("");
    let changes = json
        .get("approved_contract_changes")
        .and_then(|value| value.as_array())
        .map(|rows| !rows.is_empty())
        .unwrap_or(false);
    Ok(!owner.trim().is_empty()
        && !rationale.trim().is_empty()
        && !expiry.trim().is_empty()
        && changes)
}

fn test_meta_req_001_required_contracts_stable_and_approved(ctx: &RunContext) -> TestResult {
    let contract_id = "META-REQ-001";
    let test_id = "root.required_contracts.stable_and_approved";
    let expected = match required_contract_rows(&ctx.repo_root) {
        Ok(rows) => rows,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some("ops/policy/required-contracts.json".to_string()),
                line: None,
                message: err,
                evidence: None,
            }]);
        }
    };
    let artifact_path = ctx.repo_root.join("ops/_generated.example/contracts-required.json");
    let actual_text = match std::fs::read_to_string(&artifact_path) {
        Ok(value) => value,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some("ops/_generated.example/contracts-required.json".to_string()),
                line: None,
                message: format!("read {} failed: {err}", artifact_path.display()),
                evidence: None,
            }]);
        }
    };
    let actual_json = match serde_json::from_str::<serde_json::Value>(&actual_text) {
        Ok(value) => value,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some("ops/_generated.example/contracts-required.json".to_string()),
                line: None,
                message: format!("parse required artifact failed: {err}"),
                evidence: None,
            }]);
        }
    };
    let mut actual = actual_json
        .get("contracts")
        .and_then(|value| value.as_array())
        .map(|rows| {
            rows.iter()
                .map(|row| {
                    (
                        row.get("domain")
                            .and_then(|value| value.as_str())
                            .unwrap_or("")
                            .to_string(),
                        row.get("contract_id")
                            .and_then(|value| value.as_str())
                            .unwrap_or("")
                            .to_string(),
                        row.get("lanes")
                            .and_then(|value| value.as_array())
                            .into_iter()
                            .flatten()
                            .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    actual.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    if actual == expected {
        return TestResult::Pass;
    }
    let approval_active = approval_metadata_active(&ctx.repo_root).unwrap_or(false);
    if approval_active {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![Violation {
            contract_id: contract_id.to_string(),
            test_id: test_id.to_string(),
            file: Some("ops/_generated.example/contracts-required.json".to_string()),
            line: None,
            message: "required contracts drifted from the committed artifact without active approval metadata".to_string(),
            evidence: Some("refresh ops/_generated.example/contracts-required.json and populate ops/policy/required-contracts-change.json when the required set changes".to_string()),
        }])
    }
}

fn test_meta_req_002_required_contracts_cover_every_pillar(ctx: &RunContext) -> TestResult {
    let contract_id = "META-REQ-002";
    let test_id = "root.required_contracts.cover_every_pillar";
    let rows = match required_contract_rows(&ctx.repo_root) {
        Ok(rows) => rows,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some("ops/policy/required-contracts.json".to_string()),
                line: None,
                message: err,
                evidence: None,
            }]);
        }
    };
    let present = rows
        .into_iter()
        .map(|(domain, _, _)| domain)
        .collect::<std::collections::BTreeSet<_>>();
    let mut violations = Vec::new();
    for domain in [
        "root",
        "runtime",
        "control-plane",
        "docs",
        "make",
        "configs",
        "docker",
        "ops",
    ] {
        if !present.contains(domain) {
            violations.push(Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some("ops/policy/required-contracts.json".to_string()),
                line: None,
                message: format!("required contracts must include at least one `{domain}` contract"),
                evidence: None,
            });
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_meta_req_003_required_contracts_no_placeholder_stubs(ctx: &RunContext) -> TestResult {
    let contract_id = "META-REQ-003";
    let test_id = "root.required_contracts.no_placeholder_stubs";
    let placeholder_words = ["placeholder", "stub", "todo", "tbd"];
    let rows = match required_contract_rows(&ctx.repo_root) {
        Ok(rows) => rows,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some("ops/policy/required-contracts.json".to_string()),
                line: None,
                message: err,
                evidence: None,
            }]);
        }
    };
    let mut violations = Vec::new();
    for (domain, required_contract_id, _) in rows {
        let contracts = match contracts_for_domain(&ctx.repo_root, &domain) {
            Ok(value) => value,
            Err(err) => {
                violations.push(Violation {
                    contract_id: contract_id.to_string(),
                    test_id: test_id.to_string(),
                    file: Some("ops/policy/required-contracts.json".to_string()),
                    line: None,
                    message: format!("load contracts for `{domain}` failed: {err}"),
                    evidence: None,
                });
                continue;
            }
        };
        let Some(contract) = contracts.into_iter().find(|row| row.id.0 == required_contract_id) else {
            violations.push(Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some("ops/policy/required-contracts.json".to_string()),
                line: None,
                message: format!("required contract `{required_contract_id}` is missing from `{domain}` registry"),
                evidence: None,
            });
            continue;
        };
        let title = contract.title.to_ascii_lowercase();
        if placeholder_words.iter().any(|word| title.contains(word)) {
            violations.push(Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some("ops/policy/required-contracts.json".to_string()),
                line: None,
                message: format!("required contract `{required_contract_id}` uses placeholder wording in its title"),
                evidence: Some(contract.title.to_string()),
            });
        }
        for case in &contract.tests {
            let test_title = case.title.to_ascii_lowercase();
            if placeholder_words.iter().any(|word| test_title.contains(word)) {
                violations.push(Violation {
                    contract_id: contract_id.to_string(),
                    test_id: test_id.to_string(),
                    file: Some("ops/policy/required-contracts.json".to_string()),
                    line: None,
                    message: format!("required contract `{required_contract_id}` contains placeholder wording in test `{}`", case.id.0),
                    evidence: Some(case.title.to_string()),
                });
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}
