fn validate_ops_markdown_governance_budgets(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let ops_markdown_files = walk_files(&ctx.repo_root.join("ops"))
        .into_iter()
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("md"))
        .collect::<Vec<_>>();

    if ops_markdown_files.len() != 5 {
        violations.push(violation(
            "OPS_MARKDOWN_FILE_COUNT_INVALID",
            format!(
                "ops markdown surface must contain exactly 5 files; found {}",
                ops_markdown_files.len()
            ),
            "keep markdown limited to the five root docs under ops/",
            Some(Path::new("ops")),
        ));
    }

    let allowed = BTreeSet::from([
        "ops/CONTRACT.md".to_string(),
        "ops/ERRORS.md".to_string(),
        "ops/INDEX.md".to_string(),
        "ops/README.md".to_string(),
        "ops/SSOT.md".to_string(),
    ]);
    let mut line_total = 0usize;
    for doc in &ops_markdown_files {
        let rel = doc.strip_prefix(ctx.repo_root).unwrap_or(doc.as_path());
        let rel_str = rel.display().to_string();
        if !allowed.contains(&rel_str) {
            violations.push(violation(
                "OPS_MARKDOWN_PATH_FORBIDDEN",
                format!("unexpected ops markdown file `{rel_str}`"),
                "remove nested ops markdown and keep only the five root docs",
                Some(rel),
            ));
        }

        let text = fs::read_to_string(doc).map_err(|err| CheckError::Failed(err.to_string()))?;
        line_total += text.lines().count();
        if text.contains("TODO") || text.contains("TBD") {
            violations.push(violation(
                "OPS_DOC_TODO_MARKER_FORBIDDEN",
                format!("ops root doc `{}` contains TODO/TBD marker", rel.display()),
                "remove placeholder markers from ops root docs",
                Some(rel),
            ));
        }
    }

    if line_total > 400 {
        violations.push(violation(
            "OPS_MARKDOWN_LINE_BUDGET_EXCEEDED",
            format!("ops root markdown line budget exceeded: {line_total} > 400"),
            "keep root docs compact and move narrative detail into docs/",
            Some(Path::new("ops")),
        ));
    }

    let surfaces_text = fs::read_to_string(ctx.repo_root.join("ops/inventory/surfaces.json"))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let surfaces_json: serde_json::Value =
        serde_json::from_str(&surfaces_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let allowed_commands = surfaces_json
        .get("bijux-dev-atlas_commands")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();

    for doc in walk_files(&ctx.repo_root.join("docs")) {
        let rel = doc.strip_prefix(ctx.repo_root).unwrap_or(doc.as_path());
        if rel.extension().and_then(|value| value.to_str()) != Some("md") {
            continue;
        }
        let text = fs::read_to_string(&doc).map_err(|err| CheckError::Failed(err.to_string()))?;
        for command in extract_ops_command_refs(&text) {
            if !allowed_commands.contains(&command) {
                violations.push(violation(
                    "OPS_DOC_COMMAND_SURFACE_UNKNOWN",
                    format!(
                        "doc `{}` references command not in surfaces.json: `{command}`",
                        rel.display()
                    ),
                    "replace stale command references with canonical surfaces.json commands",
                    Some(rel),
                ));
            }
        }
    }

    Ok(())
}
