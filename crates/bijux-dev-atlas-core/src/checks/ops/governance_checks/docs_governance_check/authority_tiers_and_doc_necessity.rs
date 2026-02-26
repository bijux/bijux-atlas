fn validate_ops_authority_tiers_and_doc_necessity(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let authority_tiers_rel = Path::new("ops/AUTHORITY_TIERS.md");
    let authority_tiers_text = fs::read_to_string(ctx.repo_root.join(authority_tiers_rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", authority_tiers_rel.display())))?;
    for required in [
        "- Authority Tier:",
        "- Audience:",
        "## Tiers",
        "## Tier Rules",
        "## Audience Tags",
        "machine",
        "explanatory",
        "generated",
    ] {
        if !authority_tiers_text.contains(required) {
            violations.push(violation(
                "OPS_DOC_AUTHORITY_TIERS_CONTRACT_INCOMPLETE",
                format!(
                    "authority tiers contract `{}` is missing `{required}`",
                    authority_tiers_rel.display()
                ),
                "complete ops/AUTHORITY_TIERS.md with required metadata and tier definitions",
                Some(authority_tiers_rel),
            ));
        }
    }

    let doc_necessity_rel = Path::new("ops/DOC_NECESSITY_CHECKLIST.md");
    let doc_necessity_text = fs::read_to_string(ctx.repo_root.join(doc_necessity_rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", doc_necessity_rel.display())))?;
    for required in [
        "- Authority Tier:",
        "- Audience:",
        "## Checklist",
        "single clear consumer",
        "duplicating semantics",
        "why deletion would break",
    ] {
        if !doc_necessity_text.contains(required) {
            violations.push(violation(
                "OPS_DOC_NECESSITY_CHECKLIST_INCOMPLETE",
                format!(
                    "doc necessity checklist `{}` is missing `{required}`",
                    doc_necessity_rel.display()
                ),
                "complete ops/DOC_NECESSITY_CHECKLIST.md with required checklist criteria",
                Some(doc_necessity_rel),
            ));
        }
    }

    let valid_tiers = ["machine", "explanatory", "generated"];
    let valid_audiences = ["contributors", "operators", "reviewers", "mixed"];
    for file in walk_files(&ctx.repo_root.join("ops")) {
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
        if rel.components().count() != 2 {
            continue;
        }
        if file.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let text = fs::read_to_string(&file)
            .map_err(|err| CheckError::Failed(format!("read {}: {err}", rel.display())))?;
        let tier = text
            .lines()
            .find_map(|line| line.trim().strip_prefix("- Authority Tier: `"))
            .and_then(|v| v.strip_suffix('`'))
            .unwrap_or_default()
            .to_string();
        let audience = text
            .lines()
            .find_map(|line| line.trim().strip_prefix("- Audience: `"))
            .and_then(|v| v.strip_suffix('`'))
            .unwrap_or_default()
            .to_string();
        if tier.is_empty() {
            violations.push(violation(
                "OPS_TOP_LEVEL_DOC_AUTHORITY_TIER_MISSING",
                format!("top-level ops doc `{}` must declare `- Authority Tier:`", rel.display()),
                "add Authority Tier metadata (`machine`, `explanatory`, or `generated`) to every top-level ops/*.md doc",
                Some(rel),
            ));
            continue;
        }
        if !valid_tiers.contains(&tier.as_str()) {
            violations.push(violation(
                "OPS_TOP_LEVEL_DOC_AUTHORITY_TIER_INVALID",
                format!(
                    "top-level ops doc `{}` has invalid Authority Tier `{}`",
                    rel.display(),
                    tier
                ),
                "use one of: machine, explanatory, generated",
                Some(rel),
            ));
        }
        if audience.is_empty() {
            violations.push(violation(
                "OPS_TOP_LEVEL_DOC_AUDIENCE_MISSING",
                format!("top-level ops doc `{}` must declare `- Audience:`", rel.display()),
                "add Audience metadata (`contributors`, `operators`, `reviewers`, or `mixed`) to every top-level ops/*.md doc",
                Some(rel),
            ));
        } else if !valid_audiences.contains(&audience.as_str()) {
            violations.push(violation(
                "OPS_TOP_LEVEL_DOC_AUDIENCE_INVALID",
                format!(
                    "top-level ops doc `{}` has invalid Audience `{}`",
                    rel.display(),
                    audience
                ),
                "use one of: contributors, operators, reviewers, mixed",
                Some(rel),
            ));
        }

        if tier == "explanatory" {
            for forbidden_header in ["## Invariants", "## Contract", "## Rules"] {
                if text.contains(forbidden_header) {
                    violations.push(violation(
                        "OPS_EXPLANATORY_DOC_DEFINES_NORMATIVE_SECTION",
                        format!(
                            "explanatory top-level ops doc `{}` contains normative section header `{}`",
                            rel.display(),
                            forbidden_header
                        ),
                        "move normative rules to machine-tier contracts/policies and keep explanatory docs descriptive",
                        Some(rel),
                    ));
                }
            }
        }
    }

    let docs_shrink_rel = Path::new("ops/_generated.example/docs-shrink-report.json");
    let docs_shrink_text = fs::read_to_string(ctx.repo_root.join(docs_shrink_rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", docs_shrink_rel.display())))?;
    let docs_shrink_json: serde_json::Value = serde_json::from_str(&docs_shrink_text)
        .map_err(|err| CheckError::Failed(format!("parse {}: {err}", docs_shrink_rel.display())))?;
    if docs_shrink_json.get("status").and_then(|v| v.as_str()) != Some("pass") {
        violations.push(violation(
            "OPS_DOCS_SHRINK_REPORT_BLOCKING",
            format!("docs shrink report `{}` status is not `pass`", docs_shrink_rel.display()),
            "resolve docs compression budget failures and regenerate docs-shrink-report.json",
            Some(docs_shrink_rel),
        ));
    }
    let max_md_per_dir = docs_shrink_json
        .get("budgets")
        .and_then(|v| v.get("max_markdown_files_per_ops_domain_dir"))
        .and_then(|v| v.as_u64())
        .unwrap_or(18);
    let top_dirs = docs_shrink_json
        .get("top_directories")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if top_dirs.is_empty() {
        violations.push(violation(
            "OPS_DOCS_SHRINK_REPORT_EMPTY",
            format!("docs shrink report `{}` must include top_directories entries", docs_shrink_rel.display()),
            "include top_directories markdown counts for canonical ops domains",
            Some(docs_shrink_rel),
        ));
    }

    for domain in [
        "ops/datasets",
        "ops/e2e",
        "ops/env",
        "ops/inventory",
        "ops/k8s",
        "ops/load",
        "ops/observe",
        "ops/report",
        "ops/schema",
        "ops/stack",
    ] {
        let domain_dir = ctx.repo_root.join(domain);
        if !domain_dir.exists() {
            continue;
        }
        let md_count = walk_files(&domain_dir)
            .into_iter()
            .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("md"))
            .count() as u64;
        if md_count > max_md_per_dir {
            violations.push(violation(
                "OPS_DOMAIN_DOC_COUNT_BUDGET_EXCEEDED",
                format!(
                    "domain markdown file count exceeds budget: `{domain}` has {md_count}, budget={max_md_per_dir}"
                ),
                "consolidate docs, generate references, or raise docs-shrink-report budget with justification",
                Some(Path::new(domain)),
            ));
        }
    }

    Ok(())
}
