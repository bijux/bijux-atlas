fn validate_ops_authority_tiers_and_doc_necessity(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let ops_root_docs = [
        "ops/README.md",
        "ops/CONTRACT.md",
        "ops/INDEX.md",
        "ops/ERRORS.md",
        "ops/SSOT.md",
    ];
    let mut seen = 0usize;
    for rel_str in ops_root_docs {
        let rel = Path::new(rel_str);
        if ctx.adapters.fs.exists(ctx.repo_root, rel) {
            seen += 1;
        }
    }
    if seen != 5 {
        violations.push(violation(
            "OPS_ROOT_DOC_SURFACE_DRIFT",
            format!("ops root markdown surface must contain exactly 5 docs; found {seen}"),
            "keep ops markdown limited to the five root docs",
            Some(Path::new("ops")),
        ));
    }

    let schema_ref_rel = Path::new("docs/07-reference/index.md");
    if ctx.adapters.fs.exists(ctx.repo_root, schema_ref_rel) {
        let text = fs::read_to_string(ctx.repo_root.join(schema_ref_rel))
            .map_err(|err| CheckError::Failed(format!("read {}: {err}", schema_ref_rel.display())))?;
        if !text.contains("schema-index") {
            violations.push(violation(
                "OPS_REFERENCE_INDEX_MISSING_SCHEMA_LINK",
                "docs/07-reference/index.md should link the schema index surface".to_string(),
                "keep the reference index linked to the schema index page",
                Some(schema_ref_rel),
            ));
        }
    }

    let schema_page_rel = Path::new("docs/07-reference/error-codes-and-exit-codes.md");
    if !ctx.adapters.fs.exists(ctx.repo_root, schema_page_rel) {
        violations.push(violation(
            "OPS_REFERENCE_PAGE_MISSING",
            "missing docs/07-reference/error-codes-and-exit-codes.md".to_string(),
            "restore the stable reference page linked from ops/ERRORS.md",
            Some(schema_page_rel),
        ));
    }

    Ok(())
}
