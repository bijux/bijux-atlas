fn validate_ops_root_index_and_root_doc_budgets(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let ops_index_rel = Path::new("ops/INDEX.md");
    let ops_index_text = fs::read_to_string(ctx.repo_root.join(ops_index_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    for root_doc in [
        "CONTRACT.md",
        "CONTROL_PLANE.md",
        "DRIFT.md",
        "ERRORS.md",
        "NAMING.md",
        "README.md",
        "SSOT.md",
    ] {
        let rel = Path::new("ops").join(root_doc);
        if ctx.adapters.fs.exists(ctx.repo_root, &rel) && !ops_index_text.contains(root_doc) {
            violations.push(violation(
                "OPS_ROOT_DOC_INDEX_LINK_MISSING",
                format!(
                    "ops root document `{}` must be linked from `ops/INDEX.md`",
                    rel.display()
                ),
                "link all root ops docs from ops/INDEX.md",
                Some(ops_index_rel),
            ));
        }
    }
    let index_line_count = ops_index_text.lines().count();
    if index_line_count > 80 {
        violations.push(violation(
            "OPS_ROOT_INDEX_SIZE_BUDGET_EXCEEDED",
            format!(
                "ops/INDEX.md exceeds max line budget (80): {} lines",
                index_line_count
            ),
            "keep ops/INDEX.md compact and move details to linked docs",
            Some(ops_index_rel),
        ));
    }
    let root_doc_line_budgets = [
        ("ops/README.md", 80usize),
        ("ops/CONTRACT.md", 140usize),
        ("ops/CONTROL_PLANE.md", 80usize),
        ("ops/DRIFT.md", 80usize),
        ("ops/ERRORS.md", 80usize),
        ("ops/NAMING.md", 80usize),
        ("ops/SSOT.md", 80usize),
    ];
    for (rel_str, max_lines) in root_doc_line_budgets {
        let rel = Path::new(rel_str);
        if !ctx.adapters.fs.exists(ctx.repo_root, rel) {
            continue;
        }
        let text = fs::read_to_string(ctx.repo_root.join(rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let line_count = text.lines().count();
        if line_count > max_lines {
            violations.push(violation(
                "OPS_ROOT_DOC_SIZE_BUDGET_EXCEEDED",
                format!(
                    "ops root doc exceeds line budget: `{}` has {} lines (max {})",
                    rel.display(),
                    line_count,
                    max_lines
                ),
                "keep root governance docs compact and move extended narrative into docs/",
                Some(rel),
            ));
        }
    }

    Ok(())
}
