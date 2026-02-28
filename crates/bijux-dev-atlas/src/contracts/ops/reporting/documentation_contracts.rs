fn test_ops_root_010_forbid_deleted_doc_names(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-010";
    let test_id = "ops.root.forbid_deleted_doc_names";
    let mut files = Vec::new();
    walk_files(&ops_root(&ctx.repo_root), &mut files);
    files.sort();

    let forbidden = BTreeSet::from([
        "ARTIFACTS.md",
        "DRIFT.md",
        "NAMING.md",
        "INDEX.md",
        "OWNER.md",
        "REQUIRED_FILES.md",
        "DIRECTORY_BUDGET_POLICY.md",
        "GENERATED_LIFECYCLE.md",
    ]);

    let mut violations = Vec::new();
    for path in files {
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if forbidden.contains(name) {
            violations.push(violation(
                contract_id,
                test_id,
                "forbidden legacy ops markdown document reintroduced",
                Some(rel_to_root(&path, &ctx.repo_root)),
            ));
        }
    }

    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_contract_doc_generated_match(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-022";
    let test_id = "ops.contract_doc.generated_match";
    let expected = match render_contract_markdown(&ctx.repo_root) {
        Ok(text) => text,
        Err(err) => return TestResult::Error(err),
    };
    let path = ctx.repo_root.join("ops/CONTRACT.md");
    let actual = std::fs::read_to_string(&path).unwrap_or_default();
    if actual == expected {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "ops/CONTRACT.md drifted from generated contract registry",
            Some("ops/CONTRACT.md".to_string()),
        )])
    }
}

fn has_policy_keyword(content: &str) -> bool {
    let lower = content.to_ascii_lowercase();
    lower.contains("must ") || lower.contains("shall ") || lower.contains("forbidden")
}

fn has_ops_contract_id(content: &str) -> bool {
    content.contains("OPS-")
}

fn test_ops_docs_001_policy_keyword_requires_contract_id(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-023";
    let test_id = "ops.docs.policy_keyword_requires_contract_id";
    let root = ctx.repo_root.join("docs/operations");
    let Ok(entries) = std::fs::read_dir(&root) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "docs/operations directory is missing",
            Some("docs/operations".to_string()),
        )]);
    };

    let mut violations = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if !path
            .extension()
            .and_then(|v| v.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        if has_policy_keyword(&content) && !has_ops_contract_id(&content) {
            violations.push(violation(
                contract_id,
                test_id,
                "operations doc declares policy keywords without OPS contract id reference",
                Some(rel_to_root(&path, &ctx.repo_root)),
            ));
        }
    }

    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_ops_docs_002_index_crosslinks_contracts(ctx: &RunContext) -> TestResult {
    let contract_id = "OPS-ROOT-023";
    let test_id = "ops.docs.index_crosslinks_contracts";
    let path = ctx.repo_root.join("docs/operations/INDEX.md");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "docs/operations/INDEX.md is missing",
            Some("docs/operations/INDEX.md".to_string()),
        )]);
    };
    let has_boundary = content.contains("Operational policies are enforced by contracts");
    let has_contract_ref = content.contains("OPS-");
    if has_boundary && has_contract_ref {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            "docs/operations/INDEX.md must state docs/contracts boundary and include OPS contract references",
            Some("docs/operations/INDEX.md".to_string()),
        )])
    }
}

pub fn render_contract_markdown(repo_root: &Path) -> Result<String, String> {
    let mut rows = contracts(repo_root)?;
    rows.sort_by_key(|c| c.id.0.clone());
    let mut out = String::new();
    out.push_str("# Ops Contract\n\n");
    out.push_str("- Owner: `bijux-atlas-operations`\n");
    out.push_str("- Enforced by: `bijux dev atlas contracts ops`\n\n");
    out.push_str("## Naming\n\n");
    out.push_str("- Contract ID: `OPS-<PILLAR>-NNN`\n");
    out.push_str("- Test ID: `ops.<pillar>.<topic>[.<name>]`\n\n");
    out.push_str("## Contract Registry\n\n");

    let mut by_pillar: BTreeMap<String, Vec<&Contract>> = BTreeMap::new();
    for contract in &rows {
        let pillar = classify_contract_pillar(&contract.id.0).unwrap_or("unclassified");
        by_pillar
            .entry(pillar.to_string())
            .or_default()
            .push(contract);
    }

    for (pillar, contracts) in by_pillar {
        out.push_str(&format!("### Pillar: {pillar}\n\n"));
        for contract in contracts {
            out.push_str(&format!("#### {} {}\n\n", contract.id.0, contract.title));
            out.push_str("Tests:\n");
            for case in &contract.tests {
                let mode = match case.kind {
                    TestKind::Pure => "static",
                    TestKind::Subprocess | TestKind::Network => "effect",
                };
                out.push_str(&format!(
                    "- `{}` ({mode}, {:?}): {}\n",
                    case.id.0, case.kind, case.title
                ));
            }
            out.push('\n');
        }
    }
    out.push_str("## Rule\n\n");
    out.push_str("- Contract ID or test ID missing from this document means it does not exist.\n");
    Ok(out)
}

pub fn sync_contract_markdown(repo_root: &Path) -> Result<(), String> {
    let rendered = render_contract_markdown(repo_root)?;
    let path = repo_root.join("ops/CONTRACT.md");
    std::fs::write(&path, rendered).map_err(|e| format!("write {} failed: {e}", path.display()))
}
