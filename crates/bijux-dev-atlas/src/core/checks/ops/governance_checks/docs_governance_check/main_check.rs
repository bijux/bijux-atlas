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

    let root_index_rel = Path::new("ops/INDEX.md");
    let root_index_text = fs::read_to_string(ctx.repo_root.join(root_index_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    for required_doc in ["README.md", "CONTRACT.md", "ERRORS.md", "SSOT.md"] {
        if !root_index_text.contains(required_doc) {
            violations.push(violation(
                "OPS_ROOT_INDEX_REQUIRED_LINK_MISSING",
                format!("ops/INDEX.md must link `{required_doc}`"),
                "keep the root index aligned with the five root docs",
                Some(root_index_rel),
            ));
        }
    }

    validate_ops_generated_docs_and_reports(ctx, &mut violations)?;
    validate_ops_markdown_governance_budgets(ctx, &mut violations)?;
    validate_ops_root_index_and_root_doc_budgets(ctx, &mut violations)?;
    validate_ops_authority_tiers_and_doc_necessity(ctx, &mut violations)?;

    Ok(violations)
}
