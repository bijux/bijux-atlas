fn test_docs_019_entrypoint_word_budget(ctx: &RunContext) -> TestResult {
    let pages = match docs_entrypoint_pages(ctx) {
        Ok(pages) => pages,
        Err(result) => return result,
    };
    let mut violations = Vec::new();
    for (relative, _) in pages {
        let contents = match std::fs::read_to_string(ctx.repo_root.join(&relative)) {
            Ok(contents) => contents,
            Err(err) => {
                push_docs_violation(
                    &mut violations,
                    "DOC-019",
                    "docs.quality.entrypoint_word_budget",
                    Some(relative),
                    format!("read failed: {err}"),
                );
                continue;
            }
        };
        let word_count = contents.split_whitespace().count();
        if word_count > DOCS_ENTRYPOINT_WORD_BUDGET {
            push_docs_violation(
                &mut violations,
                "DOC-019",
                "docs.quality.entrypoint_word_budget",
                Some(relative),
                format!(
                    "entrypoint page word count {word_count} exceeds budget {DOCS_ENTRYPOINT_WORD_BUDGET}"
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

fn test_docs_020_no_placeholders(ctx: &RunContext) -> TestResult {
    let pages = match docs_entrypoint_pages(ctx) {
        Ok(pages) => pages,
        Err(result) => return result,
    };
    let markers = ["TODO", "TBD", "XXX"];
    let mut violations = Vec::new();
    for (relative, _) in pages {
        let contents = match std::fs::read_to_string(ctx.repo_root.join(&relative)) {
            Ok(contents) => contents,
            Err(err) => {
                push_docs_violation(
                    &mut violations,
                    "DOC-020",
                    "docs.quality.no_placeholders",
                    Some(relative),
                    format!("read failed: {err}"),
                );
                continue;
            }
        };
        let is_stable = parse_docs_field(&contents, &["Stability", "Status"]).as_deref() == Some("stable");
        if !is_stable {
            continue;
        }
        for marker in markers {
            if contents.contains(marker) {
                push_docs_violation(
                    &mut violations,
                    "DOC-020",
                    "docs.quality.no_placeholders",
                    Some(relative.clone()),
                    format!("stable entrypoint page contains placeholder marker `{marker}`"),
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

fn test_docs_021_no_tabs(ctx: &RunContext) -> TestResult {
    let pages = match docs_entrypoint_pages(ctx) {
        Ok(pages) => pages,
        Err(result) => return result,
    };
    let mut violations = Vec::new();
    for (relative, _) in pages {
        let contents = match std::fs::read_to_string(ctx.repo_root.join(&relative)) {
            Ok(contents) => contents,
            Err(err) => {
                push_docs_violation(
                    &mut violations,
                    "DOC-021",
                    "docs.format.no_tabs",
                    Some(relative),
                    format!("read failed: {err}"),
                );
                continue;
            }
        };
        if contents.contains('\t') {
            push_docs_violation(
                &mut violations,
                "DOC-021",
                "docs.format.no_tabs",
                Some(relative),
                "entrypoint page contains raw tab characters",
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

fn test_docs_022_no_trailing_whitespace(ctx: &RunContext) -> TestResult {
    let pages = match docs_entrypoint_pages(ctx) {
        Ok(pages) => pages,
        Err(result) => return result,
    };
    let mut violations = Vec::new();
    for (relative, _) in pages {
        let contents = match std::fs::read_to_string(ctx.repo_root.join(&relative)) {
            Ok(contents) => contents,
            Err(err) => {
                push_docs_violation(
                    &mut violations,
                    "DOC-022",
                    "docs.format.no_trailing_whitespace",
                    Some(relative),
                    format!("read failed: {err}"),
                );
                continue;
            }
        };
        if contents.lines().any(|line| line.ends_with(' ') || line.ends_with('\t')) {
            push_docs_violation(
                &mut violations,
                "DOC-022",
                "docs.format.no_trailing_whitespace",
                Some(relative),
                "entrypoint page contains trailing whitespace",
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

fn test_docs_023_single_h1(ctx: &RunContext) -> TestResult {
    let pages = match docs_entrypoint_pages(ctx) {
        Ok(pages) => pages,
        Err(result) => return result,
    };
    let mut violations = Vec::new();
    for (relative, _) in pages {
        let contents = match std::fs::read_to_string(ctx.repo_root.join(&relative)) {
            Ok(contents) => contents,
            Err(err) => {
                push_docs_violation(
                    &mut violations,
                    "DOC-023",
                    "docs.structure.single_h1",
                    Some(relative),
                    format!("read failed: {err}"),
                );
                continue;
            }
        };
        let h1_count = contents.lines().filter(|line| line.starts_with("# ")).count();
        if h1_count != 1 {
            push_docs_violation(
                &mut violations,
                "DOC-023",
                "docs.structure.single_h1",
                Some(relative),
                format!("entrypoint page must contain exactly one H1 heading, found {h1_count}"),
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
