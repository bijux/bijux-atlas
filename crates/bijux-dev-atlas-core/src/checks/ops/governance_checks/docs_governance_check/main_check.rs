pub(super) fn check_ops_docs_governance(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    let retired_docs_subtree = Path::new("docs/ops");
    if ctx.adapters.fs.exists(ctx.repo_root, retired_docs_subtree) {
        violations.push(violation(
            "OPS_DOCS_RETIRED_SUBTREE_REINTRODUCED",
            format!(
                "retired docs subtree `{}` must not exist",
                retired_docs_subtree.display()
            ),
            "keep ops handbook docs under docs/operations and remove docs/ops",
            Some(retired_docs_subtree),
        ));
    }
    let forbidden_transitional_tokens = ["phase", "task"];
    for root in ["ops", "docs"] {
        for file in walk_files(&ctx.repo_root.join(root)) {
            let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
            let rel_str = rel.display().to_string();
            let has_forbidden_segment = rel
                .components()
                .filter_map(|c| c.as_os_str().to_str())
                .any(|segment| {
                    let lowercase = segment.to_ascii_lowercase();
                    forbidden_transitional_tokens.iter().any(|token| {
                        lowercase == *token
                            || lowercase.starts_with(&format!("{token}-"))
                            || lowercase.ends_with(&format!("-{token}"))
                            || lowercase.contains(&format!("-{token}-"))
                            || lowercase.starts_with(&format!("{token}_"))
                            || lowercase.ends_with(&format!("_{token}"))
                            || lowercase.contains(&format!("_{token}_"))
                    })
                });
            if has_forbidden_segment {
                violations.push(violation(
                    "OPS_NAMING_TRANSITIONAL_TOKEN_FORBIDDEN",
                    format!(
                        "path uses transitional naming token (`phase`/`task`): `{rel_str}`"
                    ),
                    "rename files/directories to durable intent-based names",
                    Some(rel),
                ));
            }
        }
    }

    let domain_dirs = [
        "ops/datasets",
        "ops/e2e",
        "ops/k8s",
        "ops/load",
        "ops/observe",
        "ops/report",
        "ops/stack",
        "ops/env",
        "ops/inventory",
        "ops/schema",
    ];
    for domain in domain_dirs {
        let index_rel = Path::new(domain).join("INDEX.md");
        if ctx.adapters.fs.exists(ctx.repo_root, &index_rel) {
            let index_text = fs::read_to_string(ctx.repo_root.join(&index_rel))
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            for line in index_text.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                if !trimmed.starts_with("- ") {
                    violations.push(violation(
                        "OPS_DOC_INDEX_NON_LINK_CONTENT",
                        format!(
                            "domain index must be links-only; found non-link content in `{}`: `{trimmed}`",
                            index_rel.display()
                        ),
                        "keep domain INDEX.md files links-only with headings and bullet links",
                        Some(&index_rel),
                    ));
                }
            }

            for required_doc in ["README.md", "CONTRACT.md", "REQUIRED_FILES.md", "OWNER.md"] {
                let doc_rel = Path::new(domain).join(required_doc);
                if ctx.adapters.fs.exists(ctx.repo_root, &doc_rel)
                    && !index_text.contains(required_doc)
                {
                    violations.push(violation(
                        "OPS_DOC_INDEX_REQUIRED_LINK_MISSING",
                        format!(
                            "domain index `{}` must link `{}`",
                            index_rel.display(),
                            doc_rel.display()
                        ),
                        "add README.md and CONTRACT.md links to domain INDEX.md when files exist",
                        Some(&index_rel),
                    ));
                }
            }
        }

        let readme_rel = Path::new(domain).join("README.md");
        if ctx.adapters.fs.exists(ctx.repo_root, &readme_rel) {
            let readme_text = fs::read_to_string(ctx.repo_root.join(&readme_rel))
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            let line_count = readme_text.lines().count();
            if line_count > 30 {
                violations.push(violation(
                    "OPS_DOC_README_SIZE_BUDGET_EXCEEDED",
                    format!(
                        "domain README exceeds 30 line budget: `{}` has {} lines",
                        readme_rel.display(),
                        line_count
                    ),
                    "keep domain README focused on what it is and where to start within 30 lines",
                    Some(&readme_rel),
                ));
            }
        }
    }

    validate_ops_generated_docs_and_reports(ctx, &mut violations)?;
    validate_ops_markdown_governance_budgets(ctx, &mut violations)?;
    validate_ops_root_index_and_root_doc_budgets(ctx, &mut violations)?;
    validate_ops_authority_tiers_and_doc_necessity(ctx, &mut violations)?;

    Ok(violations)
}
