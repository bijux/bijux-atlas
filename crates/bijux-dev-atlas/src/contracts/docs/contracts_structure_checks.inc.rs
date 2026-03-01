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

fn test_docs_056_nav_starts_with_home_and_start_here(ctx: &RunContext) -> TestResult {
    let contents = match read_mkdocs_yaml(
        ctx,
        "DOC-056",
        "docs.nav.home_and_start_here_first",
    ) {
        Ok(contents) => contents,
        Err(result) => return result,
    };
    let top_level = mkdocs_nav_lines(&contents)
        .into_iter()
        .filter_map(|line| line.trim_start().strip_prefix("- "))
        .filter(|line| !line.starts_with("- "))
        .map(|line| line.split(':').next().unwrap_or(line).trim().to_string())
        .collect::<Vec<_>>();
    let expected = ["Home", "Start Here"];
    if top_level.len() >= 2 && top_level[0] == expected[0] && top_level[1] == expected[1] {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![Violation {
            contract_id: "DOC-056".to_string(),
            test_id: "docs.nav.home_and_start_here_first".to_string(),
            file: Some("mkdocs.yml".to_string()),
            line: None,
            message: "mkdocs nav must start with `Home` then `Start Here`".to_string(),
            evidence: None,
        }])
    }
}

fn test_docs_057_governance_nested_under_development(ctx: &RunContext) -> TestResult {
    let contents = match read_mkdocs_yaml(
        ctx,
        "DOC-057",
        "docs.nav.governance_nested_under_development",
    ) {
        Ok(contents) => contents,
        Err(result) => return result,
    };
    let nav_lines = mkdocs_nav_lines(&contents);
    let mut in_development = false;
    let mut saw_docs_governance = false;
    let mut saw_docs_dashboard = false;
    let mut top_level_governance = false;
    for line in nav_lines {
        let indent = line.chars().take_while(|ch| *ch == ' ').count();
        let trimmed = line.trim();
        if indent == 2 && trimmed.starts_with("- ") {
            in_development = trimmed == "- Development:";
            if trimmed.contains("Governance") {
                top_level_governance = true;
            }
            continue;
        }
        if in_development && indent >= 6 && trimmed == "- Docs governance: _internal/governance/index.md" {
            saw_docs_governance = true;
        }
        if in_development
            && indent >= 6
            && trimmed == "- Docs Dashboard: _internal/governance/docs-dashboard.md"
        {
            saw_docs_dashboard = true;
        }
    }
    if !top_level_governance && saw_docs_governance && saw_docs_dashboard {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![Violation {
            contract_id: "DOC-057".to_string(),
            test_id: "docs.nav.governance_nested_under_development".to_string(),
            file: Some("mkdocs.yml".to_string()),
            line: None,
            message: "mkdocs nav must keep Docs governance and Docs Dashboard nested under Development without a top-level governance entry".to_string(),
            evidence: None,
        }])
    }
}

fn test_docs_063_site_map_covers_reader_spine(ctx: &RunContext) -> TestResult {
    let relative = "docs/site-map.md";
    let contents = match std::fs::read_to_string(ctx.repo_root.join(relative)) {
        Ok(contents) => contents,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-063".to_string(),
                test_id: "docs.navigation.site_map_reader_spine".to_string(),
                file: Some(relative.to_string()),
                line: None,
                message: format!("read failed: {err}"),
                evidence: None,
            }])
        }
    };
    let mut expected = match docs_spine_pages(ctx) {
        Ok(pages) => pages,
        Err(result) => return result,
    };
    expected.extend(docs_reader_utility_pages());
    expected.retain(|path| path != relative);
    expected.sort();
    expected.dedup();

    let links = markdown_links(strip_docs_frontmatter(&contents))
        .into_iter()
        .filter_map(|target| target.split('#').next().map(|value| value.to_string()))
        .filter(|target| !target.is_empty() && !target.starts_with("http://") && !target.starts_with("https://"))
        .collect::<std::collections::BTreeSet<_>>();

    let mut violations = Vec::new();
    for path in expected {
        let local_target = path.trim_start_matches("docs/").to_string();
        if !links.contains(&local_target) && !links.contains(&path) {
            violations.push(Violation {
                contract_id: "DOC-063".to_string(),
                test_id: "docs.navigation.site_map_reader_spine".to_string(),
                file: Some(relative.to_string()),
                line: None,
                message: format!("site map must link `{path}`"),
                evidence: None,
            });
        }
    }

    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.message.cmp(&b.message));
        TestResult::Fail(violations)
    }
}

fn test_docs_064_index_links_three_golden_paths(ctx: &RunContext) -> TestResult {
    let relative = "docs/index.md";
    let contents = match std::fs::read_to_string(ctx.repo_root.join(relative)) {
        Ok(contents) => contents,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-064".to_string(),
                test_id: "docs.onboarding.three_golden_paths".to_string(),
                file: Some(relative.to_string()),
                line: None,
                message: format!("read failed: {err}"),
                evidence: None,
            }]);
        }
    };

    let required_targets = [
        "operations/run-locally.md",
        "operations/deploy-kind.md",
        "operations/deploy-kubernetes-minimal.md",
    ];
    let links = markdown_links(strip_docs_frontmatter(&contents))
        .into_iter()
        .filter_map(|target| target.split('#').next().map(|value| value.to_string()))
        .collect::<Vec<_>>();

    let mut violations = Vec::new();
    for target in required_targets {
        let count = links.iter().filter(|link| link.as_str() == target).count();
        if count != 1 {
            violations.push(Violation {
                contract_id: "DOC-064".to_string(),
                test_id: "docs.onboarding.three_golden_paths".to_string(),
                file: Some(relative.to_string()),
                line: None,
                message: format!(
                    "docs/index.md must link `{target}` exactly once, found {count}"
                ),
                evidence: None,
            });
        }
    }

    let total_canonical = links
        .iter()
        .filter(|link| {
            matches!(
                link.as_str(),
                "operations/run-locally.md"
                    | "operations/deploy-kind.md"
                    | "operations/deploy-kubernetes-minimal.md"
            )
        })
        .count();
    if total_canonical != 3 {
        violations.push(Violation {
            contract_id: "DOC-064".to_string(),
            test_id: "docs.onboarding.three_golden_paths".to_string(),
            file: Some(relative.to_string()),
            line: None,
            message: format!(
                "docs/index.md must expose exactly three canonical golden-path links, found {total_canonical}"
            ),
            evidence: None,
        });
    }

    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.message.cmp(&b.message));
        TestResult::Fail(violations)
    }
}
