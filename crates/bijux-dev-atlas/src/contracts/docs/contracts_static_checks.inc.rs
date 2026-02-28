fn docs_root_path(ctx: &RunContext) -> std::path::PathBuf {
    ctx.repo_root.join("docs")
}

fn push_docs_violation(
    violations: &mut Vec<Violation>,
    contract_id: &str,
    test_id: &str,
    file: impl Into<Option<String>>,
    message: impl Into<String>,
) {
    violations.push(Violation {
        contract_id: contract_id.to_string(),
        test_id: test_id.to_string(),
        file: file.into(),
        line: None,
        message: message.into(),
        evidence: None,
    });
}

fn test_docs_001_allowed_root_dirs(ctx: &RunContext) -> TestResult {
    let docs_root = docs_root_path(ctx);
    let entries = match std::fs::read_dir(&docs_root) {
        Ok(entries) => entries,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-001".to_string(),
                test_id: "docs.surface.allowed_root_dirs".to_string(),
                file: Some("docs".to_string()),
                line: None,
                message: format!("read docs root failed: {err}"),
                evidence: None,
            }])
        }
    };
    let mut violations = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if !path.is_dir() {
            continue;
        }
        let allowed = DOCS_ALLOWED_ROOT_DIRS
            .iter()
            .chain(DOCS_ALLOWED_ROOT_DIRS_TAIL.iter())
            .any(|candidate| *candidate == name);
        if !allowed {
            push_docs_violation(
                &mut violations,
                "DOC-001",
                "docs.surface.allowed_root_dirs",
                Some(format!("docs/{name}")),
                "unexpected top-level docs directory",
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

fn test_docs_002_allowed_root_markdown(ctx: &RunContext) -> TestResult {
    let docs_root = docs_root_path(ctx);
    let entries = match std::fs::read_dir(&docs_root) {
        Ok(entries) => entries,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: "DOC-002".to_string(),
                test_id: "docs.surface.allowed_root_markdown".to_string(),
                file: Some("docs".to_string()),
                line: None,
                message: format!("read docs root failed: {err}"),
                evidence: None,
            }])
        }
    };
    let mut violations = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() || !name.ends_with(".md") {
            continue;
        }
        if !DOCS_ALLOWED_ROOT_MARKDOWN.iter().any(|candidate| *candidate == name) {
            push_docs_violation(
                &mut violations,
                "DOC-002",
                "docs.surface.allowed_root_markdown",
                Some(format!("docs/{name}")),
                "unexpected top-level docs markdown file",
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

fn test_docs_005_no_whitespace_names(ctx: &RunContext) -> TestResult {
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
                    "DOC-005",
                    "docs.naming.no_whitespace",
                    Some(rel),
                    format!("read dir failed: {err}"),
                );
                continue;
            }
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name.chars().any(|ch| ch.is_whitespace()) {
                let rel = path
                    .strip_prefix(&ctx.repo_root)
                    .map(|value| value.display().to_string())
                    .unwrap_or_else(|_| path.display().to_string());
                push_docs_violation(
                    &mut violations,
                    "DOC-005",
                    "docs.naming.no_whitespace",
                    Some(rel),
                    "docs names may not contain whitespace",
                );
            }
            if path.is_dir() {
                stack.push(path);
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

fn test_docs_006_index_exists(ctx: &RunContext) -> TestResult {
    if docs_root_path(ctx).join("index.md").is_file() {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![Violation {
            contract_id: "DOC-006".to_string(),
            test_id: "docs.index.exists".to_string(),
            file: Some("docs/index.md".to_string()),
            line: None,
            message: "docs/index.md is required as the docs entrypoint".to_string(),
            evidence: None,
        }])
    }
}
