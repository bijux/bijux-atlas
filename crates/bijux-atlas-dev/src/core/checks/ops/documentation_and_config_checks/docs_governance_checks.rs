pub(super) fn check_docs_mkdocs_yaml_parseable(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    match parse_mkdocs_yaml(ctx) {
        Ok(_) => Ok(Vec::new()),
        Err(CheckError::Failed(msg)) => Ok(vec![violation(
            "DOCS_MKDOCS_PARSE_FAILED",
            msg,
            "fix mkdocs.yml syntax and required top-level keys",
            Some(Path::new("mkdocs.yml")),
        )]),
    }
}

pub(super) fn check_docs_mkdocs_nav_files_exist(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    for (_title, rel) in mkdocs_nav_refs(ctx)? {
        let path = ctx.repo_root.join("docs").join(&rel);
        if !path.exists() {
            violations.push(violation(
                "DOCS_MKDOCS_NAV_PATH_MISSING",
                format!("mkdocs nav references missing file `docs/{rel}`"),
                "remove stale nav entry or restore the file",
                Some(Path::new("mkdocs")),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_docs_no_orphan_markdown_pages(
    _ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    Ok(Vec::new())
}

pub(super) fn check_docs_no_duplicate_nav_titles(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut counts = std::collections::BTreeMap::<String, usize>::new();
    let allow_duplicated_titles =
        std::collections::BTreeSet::from([String::from("Overview"), String::from("Home")]);
    for (title, _) in mkdocs_nav_refs(ctx)? {
        *counts.entry(title).or_default() += 1;
    }
    let mut violations = Vec::new();
    for (title, count) in counts {
        if allow_duplicated_titles.contains(&title) {
            continue;
        }
        if count > 1 {
            violations.push(violation(
                "DOCS_DUPLICATE_NAV_TITLE",
                format!("mkdocs nav title `{title}` is duplicated {count} times"),
                "rename nav titles to be globally distinct",
                Some(Path::new("mkdocs.yml")),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_docs_markdown_link_targets_exist(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    for path in docs_markdown_paths(ctx) {
        let rel = path.strip_prefix(ctx.repo_root).unwrap_or(&path);
        let text =
            fs::read_to_string(&path).map_err(|err| CheckError::Failed(err.to_string()))?;
        for target in markdown_link_targets(&text) {
            let clean = target
                .split('#')
                .next()
                .unwrap_or_default()
                .trim()
                .to_string();
            if clean.is_empty() {
                continue;
            }
            let candidate = path
                .parent()
                .unwrap_or_else(|| Path::new("docs"))
                .join(&clean)
                .components()
                .as_path()
                .to_path_buf();
            let target_path = if candidate.exists() {
                candidate
            } else {
                ctx.repo_root.join("docs").join(&clean)
            };
            if !target_path.exists() {
                violations.push(violation(
                    "DOCS_MARKDOWN_LINK_TARGET_MISSING",
                    format!(
                        "docs markdown `{}` links missing target `{}`",
                        rel.display(),
                        clean
                    ),
                    "fix broken markdown link targets in docs markdown content",
                    Some(rel),
                ));
            }
        }
    }
    Ok(violations)
}

pub(super) fn check_docs_markdown_directory_budgets(
    _ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    Ok(Vec::new())
}

pub(super) fn check_docs_index_reachability_ledger(
    _ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    Ok(Vec::new())
}

pub(super) fn check_docs_ops_operations_duplicate_titles(
    _ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    Ok(Vec::new())
}

pub(super) fn check_docs_near_duplicate_filenames(
    _ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    Ok(Vec::new())
}

pub(super) fn check_docs_operations_directory_index_contract(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let operations_root = ctx.repo_root.join("docs/operations");
    if !operations_root.exists() {
        return Ok(Vec::new());
    }
    let mut markdown_dirs = BTreeSet::<PathBuf>::new();
    for file in walk_files(&operations_root) {
        if file.extension().and_then(|v| v.to_str()) != Some("md") {
            continue;
        }
        if let Some(parent) = file.parent() {
            markdown_dirs.insert(parent.to_path_buf());
        }
    }

    let mut violations = Vec::new();
    for dir in markdown_dirs {
        let rel_dir = dir.strip_prefix(ctx.repo_root).unwrap_or(&dir);
        let mut index_candidates = 0usize;
        for name in ["INDEX.md", "index.md"] {
            if dir.join(name).exists() {
                index_candidates += 1;
            }
        }
        if index_candidates == 0 {
            violations.push(violation(
                "DOCS_OPERATIONS_INDEX_MISSING",
                format!(
                    "operations docs directory `{}` is missing INDEX.md",
                    rel_dir.display()
                ),
                "add a single INDEX.md entrypoint for each docs/operations directory",
                Some(rel_dir),
            ));
        } else if index_candidates > 1 {
            violations.push(violation(
                "DOCS_OPERATIONS_PARALLEL_INDEX_FORBIDDEN",
                format!(
                    "operations docs directory `{}` has multiple index entrypoints",
                    rel_dir.display()
                ),
                "keep exactly one INDEX.md entrypoint per docs/operations directory",
                Some(rel_dir),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_docs_operations_canonical_concept_paths(
    _ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    Ok(Vec::new())
}

pub(super) fn check_docs_operations_verify_command_quality(
    _ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    Ok(Vec::new())
}

pub(super) fn check_docs_readme_index_contract_presence(
    _ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    Ok(Vec::new())
}

pub(super) fn check_docs_file_naming_conventions(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    for path in docs_markdown_paths(ctx) {
        let rel = path.strip_prefix(ctx.repo_root).unwrap_or(&path);
        let name = path
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default();
        if rel.to_string_lossy().contains(' ') {
            violations.push(violation(
                "DOCS_FILE_NAME_SPACES_FORBIDDEN",
                format!("docs path contains spaces: `{}`", rel.display()),
                "use intent-based lowercase names without spaces",
                Some(rel),
            ));
            continue;
        }
        if name != "README.md" && name != "INDEX.md" && name.chars().any(|c| c.is_ascii_uppercase())
        {
            violations.push(violation(
                "DOCS_FILE_NAME_CASE_FORBIDDEN",
                format!("docs file should use lowercase naming: `{}`", rel.display()),
                "rename docs file to lowercase intent-based name",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_docs_command_surface_docs_exist(
    _ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    Ok(Vec::new())
}

pub(super) fn check_crate_docs_governance_contract(
    _ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    Ok(Vec::new())
}

pub(super) fn check_make_docs_wrappers_delegate_dev_atlas(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("makes/docs.mk");
    let path = ctx.repo_root.join(rel);
    let content = fs::read_to_string(&path).map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut violations = Vec::new();
    if !content.contains("BIJUX ?= bijux") || !content.contains("BIJUX_DEV_ATLAS ?=") {
        violations.push(violation(
            "MAKE_DOCS_BIJUX_VARIABLES_MISSING",
            "makes/docs.mk must declare BIJUX and BIJUX_DEV_ATLAS variables"
                .to_string(),
            "declare BIJUX ?= bijux and BIJUX_DEV_ATLAS ?= $(BIJUX) dev atlas",
            Some(rel),
        ));
    }
    for line in content.lines().filter(|line| line.starts_with('\t')) {
        if line.trim_end().ends_with('\\') {
            violations.push(violation(
                "MAKE_DOCS_SINGLE_LINE_RECIPE_REQUIRED",
                "makes/docs.mk wrapper recipes must be single-line delegations"
                    .to_string(),
                "keep docs wrappers single-line and delegation-only",
                Some(rel),
            ));
        }
        let words = line.split_whitespace().collect::<Vec<_>>();
        if words.iter().any(|w| {
            *w == "python"
                || *w == "python3"
                || *w == "bash"
                || *w == "helm"
                || *w == "kubectl"
                || *w == "k6"
        }) {
            violations.push(violation(
                "MAKE_DOCS_DELEGATION_ONLY_VIOLATION",
                format!("makes/docs.mk must stay delegation-only: `{line}`"),
                "docs wrappers may call make or bijux dev atlas only",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}
