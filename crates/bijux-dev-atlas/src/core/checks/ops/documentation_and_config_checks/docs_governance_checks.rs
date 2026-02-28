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
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let nav_set = mkdocs_nav_refs(ctx)?
        .into_iter()
        .map(|(_, p)| p)
        .collect::<BTreeSet<_>>();
    let mut violations = Vec::new();
    for path in docs_markdown_paths(ctx) {
        let rel = path
            .strip_prefix(ctx.repo_root.join("docs"))
            .unwrap_or(&path);
        let rels = rel.display().to_string();
        if rels.starts_with("_assets/") || rels.starts_with("_drafts/") {
            continue;
        }
        if !nav_set.contains(&rels) {
            violations.push(violation(
                "DOCS_ORPHAN_MARKDOWN_PAGE",
                format!("markdown page is not referenced in mkdocs nav: `docs/{rels}`"),
                "add the page to mkdocs nav or explicitly exclude it from docs surface",
                Some(Path::new("docs")),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_docs_no_duplicate_nav_titles(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut counts = std::collections::BTreeMap::<String, usize>::new();
    for (title, _) in mkdocs_nav_refs(ctx)? {
        *counts.entry(title).or_default() += 1;
    }
    let mut violations = Vec::new();
    for (title, count) in counts {
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
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let budgets = BTreeMap::from([
        ("docs/operations".to_string(), 200usize),
        ("docs/reference".to_string(), 200usize),
        ("docs/development".to_string(), 160usize),
        ("docs/contracts".to_string(), 120usize),
        ("docs/ops".to_string(), 120usize),
    ]);
    let mut counts = BTreeMap::<String, usize>::new();
    for path in docs_markdown_paths(ctx) {
        let rel = path.strip_prefix(ctx.repo_root).unwrap_or(&path);
        let rel_str = rel.display().to_string();
        for prefix in budgets.keys() {
            if rel_str == *prefix || rel_str.starts_with(&(prefix.clone() + "/")) {
                *counts.entry(prefix.clone()).or_default() += 1;
            }
        }
    }
    let mut violations = Vec::new();
    for (prefix, max) in budgets {
        let count = *counts.get(&prefix).unwrap_or(&0usize);
        if count > max {
            violations.push(violation(
                "DOCS_MARKDOWN_DIRECTORY_BUDGET_EXCEEDED",
                format!("docs markdown budget exceeded for `{prefix}`: {count} > {max}"),
                "consolidate duplicate docs and keep one canonical page per concept",
                Some(Path::new(&prefix)),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_docs_index_reachability_ledger(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    let mut index_paths = Vec::new();
    for path in docs_markdown_paths(ctx) {
        let rel = path.strip_prefix(ctx.repo_root).unwrap_or(&path).to_path_buf();
        if rel.file_name().and_then(|n| n.to_str()) == Some("INDEX.md") {
            index_paths.push(path);
        }
    }

    let mut reachable = BTreeSet::new();
    for index in &index_paths {
        let text = fs::read_to_string(index).map_err(|err| CheckError::Failed(err.to_string()))?;
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
            let candidate = index.parent().unwrap_or_else(|| Path::new("docs")).join(&clean);
            let target_path = if candidate.exists() {
                candidate
            } else {
                ctx.repo_root.join("docs").join(&clean)
            };
            if target_path.exists() && target_path.extension().and_then(|v| v.to_str()) == Some("md") {
                if let Ok(rel) = target_path.strip_prefix(ctx.repo_root) {
                    reachable.insert(rel.display().to_string());
                }
            }
        }
    }

    let mut computed_entries = Vec::new();
    for path in docs_markdown_paths(ctx) {
        let rel = path
            .strip_prefix(ctx.repo_root)
            .unwrap_or(&path)
            .display()
            .to_string();
        let text = fs::read_to_string(&path).map_err(|err| CheckError::Failed(err.to_string()))?;
        let title = markdown_h1_title(&text).unwrap_or_else(|| "(untitled)".to_string());
        let is_index = rel.ends_with("/INDEX.md") || rel == "docs/INDEX.md" || rel == "docs/index.md";
        let is_reachable = is_index || reachable.contains(&rel);
        if !is_reachable {
            violations.push(violation(
                "DOCS_INDEX_REACHABILITY_MISSING",
                format!("docs markdown `{rel}` is not linked from any docs/**/INDEX.md"),
                "link the page from a canonical INDEX.md or remove it",
                Some(Path::new(&rel)),
            ));
        }
        computed_entries.push(serde_json::json!({
            "path": rel,
            "title": title,
            "is_index": is_index,
            "reachable_from_index": is_reachable
        }));
    }
    computed_entries.sort_by(|a, b| {
        a.get("path")
            .and_then(|v| v.as_str())
            .cmp(&b.get("path").and_then(|v| v.as_str()))
    });
    let computed_ledger = serde_json::json!({
        "schema_version": 1,
        "generated_by": "bijux dev atlas docs ledger --write-example",
        "entries": computed_entries
    });
    let ledger_rel = Path::new("docs/_generated/docs-ledger.json");
    if !ctx.adapters.fs.exists(ctx.repo_root, ledger_rel) {
        violations.push(violation(
            "DOCS_LEDGER_MISSING",
            "missing docs ledger artifact `docs/_generated/docs-ledger.json`".to_string(),
            "generate and commit docs/_generated/docs-ledger.json",
            Some(ledger_rel),
        ));
    } else {
        let committed = fs::read_to_string(ctx.repo_root.join(ledger_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let committed_json: serde_json::Value = serde_json::from_str(&committed)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        if committed_json != computed_ledger {
            violations.push(violation(
                "DOCS_LEDGER_STALE",
                "docs/_generated/docs-ledger.json is stale against current docs index reachability"
                    .to_string(),
                "regenerate and commit docs/_generated/docs-ledger.json",
                Some(ledger_rel),
            ));
        }
    }

    Ok(violations)
}

pub(super) fn check_docs_ops_operations_duplicate_titles(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut seen_titles = BTreeMap::<String, String>::new();
    let mut violations = Vec::new();
    for path in docs_markdown_paths(ctx) {
        let rel = path
            .strip_prefix(ctx.repo_root)
            .unwrap_or(&path)
            .display()
            .to_string();
        if !rel.starts_with("docs/operations/") {
            continue;
        }
        let text = fs::read_to_string(&path).map_err(|err| CheckError::Failed(err.to_string()))?;
        let title = markdown_h1_title(&text).unwrap_or_default();
        if title.is_empty() {
            continue;
        }
        if let Some(dup) = seen_titles.get(&title.to_ascii_lowercase()) {
            violations.push(violation(
                "DOCS_DUPLICATE_TITLE_ACROSS_OPS_GROUPS",
                format!("duplicate title in docs/operations: `{}` in `{dup}` and `{rel}`", title),
                "keep one canonical page title per concept in docs/operations",
                Some(Path::new(&rel)),
            ));
        } else {
            seen_titles.insert(title.to_ascii_lowercase(), rel);
        }
    }
    Ok(violations)
}

pub(super) fn check_docs_near_duplicate_filenames(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut by_dir = BTreeMap::<String, BTreeSet<String>>::new();
    for path in docs_markdown_paths(ctx) {
        let rel = path.strip_prefix(ctx.repo_root).unwrap_or(&path);
        let parent = rel
            .parent()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let stem = rel
            .file_stem()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_string();
        by_dir.entry(parent).or_default().insert(stem);
    }
    let mut violations = Vec::new();
    for (dir, stems) in by_dir {
        for stem in &stems {
            if let Some(stripped) = stem.strip_suffix("ly") {
                if stems.contains(stripped) {
                    violations.push(violation(
                        "DOCS_NEAR_DUPLICATE_FILENAME",
                        format!(
                            "near-duplicate filenames in `{dir}`: `{}` and `{}`",
                            stem, stripped
                        ),
                        "keep one canonical filename and remove redirect-style duplicates",
                        Some(Path::new(&dir)),
                    ));
                }
            }
        }
    }
    Ok(violations)
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
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let deprecated = Path::new("docs/operations/full-stack-locally.md");
    if ctx.repo_root.join(deprecated).exists() {
        Ok(vec![violation(
            "DOCS_OPERATIONS_DUPLICATE_CONCEPT_PATH",
            "deprecated duplicate concept path still exists: docs/operations/full-stack-locally.md"
                .to_string(),
            "keep docs/operations/full-stack-local.md as canonical and remove duplicate aliases",
            Some(deprecated),
        )])
    } else {
        Ok(Vec::new())
    }
}

pub(super) fn check_docs_operations_verify_command_quality(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("docs/operations/INDEX.md");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if text.contains("$ make docs") || text.contains("`make docs`") {
        Ok(vec![violation(
            "DOCS_OPERATIONS_VERIFY_COMMAND_STALE",
            "docs/operations/INDEX.md uses stale verification command `make docs`".to_string(),
            "replace with canonical docs doctor command and expected status output",
            Some(rel),
        )])
    } else {
        Ok(Vec::new())
    }
}

pub(super) fn check_docs_readme_index_contract_presence(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let required = [
        "docs/INDEX.md",
        "docs/reference/contracts/index.md",
        "ops/CONTRACT.md",
        "ops/INDEX.md",
    ];
    let mut violations = Vec::new();
    for rel in required {
        let p = Path::new(rel);
        if !ctx.repo_root.join(p).exists() {
            violations.push(violation(
                "DOCS_CONTRACT_PRESENCE_MISSING",
                format!("required contract/index document missing `{rel}`"),
                "restore required INDEX/CONTRACT documents for docs and ops areas",
                Some(p),
            ));
        }
    }
    Ok(violations)
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
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    for rel in [
        "docs/reference/contracts/plugins/mode.md",
        "crates/bijux-atlas-cli/docs/CLI_COMMAND_LIST.md",
        "crates/bijux-dev-atlas/docs/CLI_COMMAND_LIST.md",
    ] {
        let p = Path::new(rel);
        if !ctx.repo_root.join(p).exists() {
            violations.push(violation(
                "DOCS_COMMAND_SURFACE_DOC_MISSING",
                format!("missing command surface document `{rel}`"),
                "restore runtime and dev command surface contract docs",
                Some(p),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_crate_docs_governance_contract(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let policy_path = Path::new("docs/governance/metadata/crate-doc-governance.json");
    let policy_text = fs::read_to_string(ctx.repo_root.join(policy_path)).map_err(|err| {
        CheckError::Failed(format!("failed to read {}: {err}", policy_path.display()))
    })?;
    let policy: serde_json::Value = serde_json::from_str(&policy_text).map_err(|err| {
        CheckError::Failed(format!("failed to parse {}: {err}", policy_path.display()))
    })?;
    let max_docs = policy
        .get("max_docs_per_crate")
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;
    let allowed_doc_types = policy
        .get("allowed_doc_types")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(ToString::to_string))
        .collect::<BTreeSet<_>>();
    let public_crates = policy
        .get("public_crates")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(ToString::to_string))
        .collect::<BTreeSet<_>>();

    let mut violations = Vec::new();
    let crates_root = ctx.repo_root.join("crates");
    for crate_dir in read_dir_entries(&crates_root) {
        if !crate_dir.is_dir() {
            continue;
        }
        let crate_name = crate_dir
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_string();
        if crate_name.is_empty() {
            continue;
        }

        for required in ["README.md", "ARCHITECTURE.md", "CONTRACT.md", "TESTING.md"] {
            let rel = Path::new("crates").join(&crate_name).join(required);
            if !ctx.repo_root.join(&rel).exists() {
                violations.push(violation(
                    "CRATE_DOC_REQUIRED_FILE_MISSING",
                    format!("crate `{crate_name}` missing required doc `{required}`"),
                    "add required crate-level governance docs",
                    Some(&rel),
                ));
            }
        }

        if public_crates.contains(&crate_name) {
            let examples_rel = Path::new("crates").join(&crate_name).join("EXAMPLES.md");
            if !ctx.repo_root.join(&examples_rel).exists() {
                violations.push(violation(
                    "CRATE_DOC_PUBLIC_EXAMPLES_MISSING",
                    format!("public crate `{crate_name}` must provide EXAMPLES.md"),
                    "add EXAMPLES.md with runnable snippets",
                    Some(&examples_rel),
                ));
            }
        }

        let docs_dir = crate_dir.join("docs");
        if !docs_dir.exists() {
            continue;
        }
        let docs = walk_files(&docs_dir)
            .into_iter()
            .filter(|p| p.extension().and_then(|v| v.to_str()) == Some("md"))
            .collect::<Vec<_>>();
        if docs.len() > max_docs {
            let rel = docs_dir.strip_prefix(ctx.repo_root).unwrap_or(&docs_dir);
            violations.push(violation(
                "CRATE_DOC_BUDGET_EXCEEDED",
                format!(
                    "crate `{crate_name}` has {} docs in docs/ (max {})",
                    docs.len(),
                    max_docs
                ),
                "prune duplicate docs or move details into canonical central docs",
                Some(rel),
            ));
        }

        for path in docs {
            let rel = path.strip_prefix(ctx.repo_root).unwrap_or(&path);
            let stem = path
                .file_stem()
                .and_then(|v| v.to_str())
                .unwrap_or_default();
            let inferred = if stem.eq_ignore_ascii_case("index") {
                "index"
            } else if stem.to_ascii_lowercase().contains("architecture") {
                "architecture"
            } else if stem.to_ascii_lowercase().contains("contract")
                || stem.to_ascii_lowercase().contains("public-api")
            {
                "contract"
            } else if stem.to_ascii_lowercase().contains("testing") {
                "testing"
            } else if stem.to_ascii_lowercase().contains("perf")
                || stem.to_ascii_lowercase().contains("bench")
            {
                "performance"
            } else if stem.to_ascii_lowercase().contains("error")
                || stem.to_ascii_lowercase().contains("failure")
            {
                "error-taxonomy"
            } else if stem.to_ascii_lowercase().contains("effect")
                || stem.to_ascii_lowercase().contains("boundary")
            {
                "boundary"
            } else if stem.to_ascii_lowercase().contains("version") {
                "versioning"
            } else if stem.to_ascii_lowercase().contains("example") {
                "examples"
            } else {
                "concept"
            };
            if !allowed_doc_types.contains(inferred) {
                violations.push(violation(
                    "CRATE_DOC_TYPE_FORBIDDEN",
                    format!(
                        "crate `{crate_name}` doc `{}` inferred type `{inferred}` is not allowed",
                        rel.display()
                    ),
                    "rename or consolidate docs to allowed governance types",
                    Some(rel),
                ));
            }
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            let header = text.lines().take(40).collect::<Vec<_>>().join("\n");
            if !header.contains("- Owner:") {
                violations.push(violation(
                    "CRATE_DOC_OWNER_METADATA_MISSING",
                    format!("crate doc missing `- Owner:` metadata: `{}`", rel.display()),
                    "add owner metadata in doc header",
                    Some(rel),
                ));
            }
        }

        let index_rel = Path::new("crates").join(&crate_name).join("docs/INDEX.md");
        if ctx.repo_root.join(&index_rel).exists() {
            let text = fs::read_to_string(ctx.repo_root.join(&index_rel)).unwrap_or_default();
            if !text.contains("README.md") && !text.contains("../README.md") {
                violations.push(violation(
                    "CRATE_DOC_INDEX_README_LINK_MISSING",
                    format!("crate `{crate_name}` docs/INDEX.md should link to crate README"),
                    "add crate README link to docs index",
                    Some(&index_rel),
                ));
            }
            if !text.contains("docs/index.md") && !text.contains("docs/INDEX.md") {
                violations.push(violation(
                    "CRATE_DOC_CENTRAL_LINK_MISSING",
                    format!("crate `{crate_name}` docs/INDEX.md should link to central docs index"),
                    "add link to docs/index.md for cross-navigation",
                    Some(&index_rel),
                ));
            }
        }
    }

    Ok(violations)
}

pub(super) fn check_make_docs_wrappers_delegate_dev_atlas(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("make/makefiles/docs.mk");
    let path = ctx.repo_root.join(rel);
    let content = fs::read_to_string(&path).map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut violations = Vec::new();
    if !content.contains("BIJUX ?= bijux") || !content.contains("BIJUX_DEV_ATLAS ?=") {
        violations.push(violation(
            "MAKE_DOCS_BIJUX_VARIABLES_MISSING",
            "make/makefiles/docs.mk must declare BIJUX and BIJUX_DEV_ATLAS variables"
                .to_string(),
            "declare BIJUX ?= bijux and BIJUX_DEV_ATLAS ?= $(BIJUX) dev atlas",
            Some(rel),
        ));
    }
    for line in content.lines().filter(|line| line.starts_with('\t')) {
        if line.trim_end().ends_with('\\') {
            violations.push(violation(
                "MAKE_DOCS_SINGLE_LINE_RECIPE_REQUIRED",
                "make/makefiles/docs.mk wrapper recipes must be single-line delegations"
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
                format!("make/makefiles/docs.mk must stay delegation-only: `{line}`"),
                "docs wrappers may call make or bijux dev atlas only",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}
