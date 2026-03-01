fn test_docs_003_depth_budget(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    let docs_root = docs_root_path(ctx);
    let mut stack = vec![docs_root.clone()];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(err) => {
                let rel = dir
                    .strip_prefix(&ctx.repo_root)
                    .map(|value| value.display().to_string())
                    .unwrap_or_else(|_| dir.display().to_string());
                push_docs_violation(
                    &mut violations,
                    "DOC-003",
                    "docs.structure.depth_budget",
                    Some(rel),
                    format!("read dir failed: {err}"),
                );
                continue;
            }
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let rel = match path.strip_prefix(&docs_root) {
                Ok(value) => value,
                Err(_) => continue,
            };
            let depth = rel.components().count();
            if depth > DOCS_MAX_DEPTH {
                let rel_path = path
                    .strip_prefix(&ctx.repo_root)
                    .map(|value| value.display().to_string())
                    .unwrap_or_else(|_| path.display().to_string());
                push_docs_violation(
                    &mut violations,
                    "DOC-003",
                    "docs.structure.depth_budget",
                    Some(rel_path),
                    format!("docs depth {depth} exceeds budget {DOCS_MAX_DEPTH}"),
                );
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}

fn test_docs_004_sibling_budget(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    let docs_root = docs_root_path(ctx);
    let mut stack = vec![docs_root.clone()];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(err) => {
                let rel = dir
                    .strip_prefix(&ctx.repo_root)
                    .map(|value| value.display().to_string())
                    .unwrap_or_else(|_| dir.display().to_string());
                push_docs_violation(
                    &mut violations,
                    "DOC-004",
                    "docs.structure.sibling_budget",
                    Some(rel),
                    format!("read dir failed: {err}"),
                );
                continue;
            }
        };
        let mut count = 0usize;
        for entry in entries.flatten() {
            count += 1;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            }
        }
        if count > DOCS_MAX_SIBLINGS {
            let rel = dir
                .strip_prefix(&ctx.repo_root)
                .map(|value| value.display().to_string())
                .unwrap_or_else(|_| dir.display().to_string());
            push_docs_violation(
                &mut violations,
                "DOC-004",
                "docs.structure.sibling_budget",
                Some(rel),
                format!("docs sibling count {count} exceeds budget {DOCS_MAX_SIBLINGS}"),
            );
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}

fn test_docs_048_published_titles_unique(ctx: &RunContext) -> TestResult {
    let mut titles = std::collections::BTreeMap::<String, Vec<String>>::new();
    let mut violations = Vec::new();
    for relative in docs_published_markdown_files(ctx) {
        let contents = match std::fs::read_to_string(ctx.repo_root.join(&relative)) {
            Ok(contents) => contents,
            Err(err) => {
                push_docs_violation(
                    &mut violations,
                    "DOC-048",
                    "docs.structure.unique_titles",
                    Some(relative),
                    format!("read failed: {err}"),
                );
                continue;
            }
        };
        let Some(title) = docs_first_h1(&contents) else {
            continue;
        };
        titles.entry(title).or_default().push(relative);
    }
    for (title, files) in titles {
        if files.len() > 1 {
            for file in files {
                push_docs_violation(
                    &mut violations,
                    "DOC-048",
                    "docs.structure.unique_titles",
                    Some(file),
                    format!("published docs title `{title}` is duplicated"),
                );
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}

fn test_docs_049_published_single_h1(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    for relative in docs_published_markdown_files(ctx) {
        let contents = match std::fs::read_to_string(ctx.repo_root.join(&relative)) {
            Ok(contents) => contents,
            Err(err) => {
                push_docs_violation(
                    &mut violations,
                    "DOC-049",
                    "docs.structure.single_h1_published",
                    Some(relative),
                    format!("read failed: {err}"),
                );
                continue;
            }
        };
        let count = docs_h1_count(&contents);
        if count != 1 {
            push_docs_violation(
                &mut violations,
                "DOC-049",
                "docs.structure.single_h1_published",
                Some(relative),
                format!("published page must contain exactly one H1 heading, found {count}"),
            );
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}

fn test_docs_051_home_line_budget(ctx: &RunContext) -> TestResult {
    let path = ctx.repo_root.join("docs/index.md");
    let contents = match std::fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-051".to_string(),
                test_id: "docs.index.home_line_budget".to_string(),
                file: Some("docs/index.md".to_string()),
                line: None,
                message: format!("read failed: {err}"),
                evidence: None,
            }]);
        }
    };
    let line_count = contents.lines().count();
    if line_count <= DOCS_HOME_LINE_BUDGET {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![Violation {
            contract_id: "DOC-051".to_string(),
            test_id: "docs.index.home_line_budget".to_string(),
            file: Some("docs/index.md".to_string()),
            line: None,
            message: format!(
                "docs/index.md has {line_count} lines and exceeds the home budget {DOCS_HOME_LINE_BUDGET}"
            ),
            evidence: None,
        }])
    }
}

fn test_docs_052_single_start_here(ctx: &RunContext) -> TestResult {
    let count = docs_count_named_files(ctx, "start-here.md");
    if count == 1 && ctx.repo_root.join("docs/start-here.md").is_file() {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![Violation {
            contract_id: "DOC-052".to_string(),
            test_id: "docs.structure.single_start_here".to_string(),
            file: Some("docs".to_string()),
            line: None,
            message: format!("docs must contain exactly one start-here.md, found {count}"),
            evidence: None,
        }])
    }
}

fn test_docs_053_single_glossary(ctx: &RunContext) -> TestResult {
    let count = docs_count_named_files(ctx, "glossary.md");
    if count == 1 && ctx.repo_root.join("docs/glossary.md").is_file() {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![Violation {
            contract_id: "DOC-053".to_string(),
            test_id: "docs.structure.single_glossary".to_string(),
            file: Some("docs".to_string()),
            line: None,
            message: format!("docs must contain exactly one glossary.md, found {count}"),
            evidence: None,
        }])
    }
}

fn test_docs_054_mkdocs_excludes_internal_and_drafts(ctx: &RunContext) -> TestResult {
    let contents = match read_mkdocs_yaml(
        ctx,
        "DOC-054",
        "docs.build.mkdocs_excludes_internal_and_drafts",
    ) {
        Ok(contents) => contents,
        Err(result) => return result,
    };
    let required_patterns = [
        "exclude_docs: |",
        "  _drafts/**",
        "not_in_nav: |",
        "  _internal/**",
        "  _drafts/**",
    ];
    let mut violations = Vec::new();
    for pattern in required_patterns {
        if !contents.contains(pattern) {
            push_docs_violation(
                &mut violations,
                "DOC-054",
                "docs.build.mkdocs_excludes_internal_and_drafts",
                Some("mkdocs.yml".to_string()),
                format!("mkdocs.yml must contain `{pattern}`"),
            );
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_docs_055_section_index_link_budget(ctx: &RunContext) -> TestResult {
    let indexes = match docs_required_section_indexes(ctx) {
        Ok(indexes) => indexes,
        Err(result) => return result,
    };
    let mut violations = Vec::new();
    for relative in indexes {
        let contents = match std::fs::read_to_string(ctx.repo_root.join(&relative)) {
            Ok(contents) => contents,
            Err(err) => {
                push_docs_violation(
                    &mut violations,
                    "DOC-055",
                    "docs.index.section_link_budget",
                    Some(relative),
                    format!("read failed: {err}"),
                );
                continue;
            }
        };
        let link_count = markdown_links(strip_docs_frontmatter(&contents)).len();
        if link_count > DOCS_SECTION_INDEX_LINK_BUDGET {
            push_docs_violation(
                &mut violations,
                "DOC-055",
                "docs.index.section_link_budget",
                Some(relative),
                format!(
                    "section index has {link_count} markdown links and exceeds the curated budget {DOCS_SECTION_INDEX_LINK_BUDGET}"
                ),
            );
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}
