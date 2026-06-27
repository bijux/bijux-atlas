pub(super) fn check_ops_required_files_contracts(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    for file in walk_files(&ctx.repo_root.join("ops")) {
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
        if rel.file_name().and_then(|name| name.to_str()) != Some("REQUIRED_FILES.md") {
            continue;
        }
        violations.push(violation(
            "OPS_REQUIRED_FILES_MARKDOWN_FORBIDDEN",
            format!("retired required-files contract must not exist: `{}`", rel.display()),
            "remove REQUIRED_FILES.md and express requirements in machine-readable inventories",
            Some(rel),
        ));
    }
    Ok(violations)
}
